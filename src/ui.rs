use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::animation::{AnimationEngine, SpeedRule};
use crate::git::{CommitMetadata, DiffMode, GitRepository};
use crate::panes::{EditorPane, FileTreePane, StatusBarPane, TerminalPane};
use crate::theme::Theme;
use crate::PlaybackOrder;

#[derive(Debug, Clone, PartialEq)]
enum UIState {
    Playing,
    WaitingForNext { resume_at: Instant },
    Finished,
}

/// Main UI controller for the gitlogue terminal interface.
pub struct UI<'a> {
    state: UIState,
    speed_ms: u64,
    file_tree: FileTreePane,
    editor: EditorPane,
    terminal: TerminalPane,
    status_bar: StatusBarPane,
    engine: AnimationEngine,
    repo: Option<&'a GitRepository>,
    should_exit: Arc<AtomicBool>,
    theme: Theme,
    order: PlaybackOrder,
    loop_playback: bool,
    commit_spec: Option<String>,
    is_range_mode: bool,
    diff_mode: Option<DiffMode>,
}

impl<'a> UI<'a> {
    /// Creates a new UI instance with the specified configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        speed_ms: u64,
        repo: Option<&'a GitRepository>,
        theme: Theme,
        order: PlaybackOrder,
        loop_playback: bool,
        commit_spec: Option<String>,
        is_range_mode: bool,
        speed_rules: Vec<SpeedRule>,
    ) -> Self {
        let should_exit = Arc::new(AtomicBool::new(false));
        Self::setup_signal_handler(should_exit.clone());

        let mut engine = AnimationEngine::new(speed_ms);
        engine.set_speed_rules(speed_rules);

        Self {
            state: UIState::Playing,
            speed_ms,
            file_tree: FileTreePane::new(),
            editor: EditorPane,
            terminal: TerminalPane,
            status_bar: StatusBarPane,
            engine,
            repo,
            should_exit,
            theme,
            order,
            loop_playback,
            commit_spec,
            is_range_mode,
            diff_mode: None,
        }
    }

    /// Sets the diff mode for working tree diff playback.
    pub fn set_diff_mode(&mut self, mode: Option<DiffMode>) {
        self.diff_mode = mode;
    }

    fn setup_signal_handler(should_exit: Arc<AtomicBool>) {
        ctrlc::set_handler(move || {
            // Restore terminal state before exiting
            let _ = disable_raw_mode();
            let _ = execute!(
                io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture,
                crossterm::cursor::Show
            );
            should_exit.store(true, Ordering::SeqCst);
            // Exit immediately for external signals (SIGTERM)
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }

    /// Loads a commit and starts the animation.
    pub fn load_commit(&mut self, metadata: CommitMetadata) {
        self.engine.load_commit(&metadata);
        self.state = UIState::Playing;
    }

    /// Runs the main UI event loop.
    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal);

        self.cleanup(&mut terminal)?;

        result
    }

    fn cleanup(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            // Check for Ctrl+C signal
            if self.should_exit.load(Ordering::Relaxed) {
                self.state = UIState::Finished;
            }

            // Update viewport dimensions for scroll calculation
            let size = terminal.size()?;
            // Editor area: 70% (right column) × 80% (editor pane) = 56% of total height
            let viewport_height = (size.height as f32 * 0.70 * 0.80) as usize;
            // Editor width: 70% (right column)
            let content_width = (size.width as f32 * 0.70) as usize;
            self.engine.set_viewport_height(viewport_height);
            self.engine.set_content_width(content_width);

            // Tick the animation engine
            let needs_redraw = self.engine.tick();

            if needs_redraw {
                terminal.draw(|f| self.render(f))?;
            }

            // Poll for keyboard events at frame rate
            if event::poll(std::time::Duration::from_millis(8))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.state = UIState::Finished;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.state = UIState::Finished;
                        }
                        _ => {}
                    }
                }
            }

            // State machine
            match self.state {
                UIState::Playing => {
                    if self.engine.is_finished() {
                        if self.repo.is_some() {
                            // Schedule next commit
                            // Wait time proportional to speed (100x the typing speed)
                            self.state = UIState::WaitingForNext {
                                resume_at: Instant::now()
                                    + Duration::from_millis(self.speed_ms * 100),
                            };
                        } else {
                            // Single commit mode without loop - quit
                            self.state = UIState::Finished;
                        }
                    }
                }
                UIState::WaitingForNext { resume_at } => {
                    if Instant::now() >= resume_at {
                        // Handle diff mode looping
                        if let Some(diff_mode) = self.diff_mode {
                            if let Some(repo) = self.repo {
                                // Refresh the working tree diff
                                match repo.get_working_tree_diff(diff_mode) {
                                    Ok(metadata) if !metadata.changes.is_empty() => {
                                        self.load_commit(metadata);
                                    }
                                    _ => {
                                        // No more changes, finish
                                        self.state = UIState::Finished;
                                    }
                                }
                            } else {
                                self.state = UIState::Finished;
                            }
                        } else if let Some(repo) = self.repo {
                            let result = if self.is_range_mode {
                                match self.order {
                                    PlaybackOrder::Random => repo.random_range_commit(),
                                    PlaybackOrder::Asc => repo.next_range_commit_asc(),
                                    PlaybackOrder::Desc => repo.next_range_commit_desc(),
                                }
                            } else if self.commit_spec.is_some() {
                                repo.get_commit(self.commit_spec.as_ref().unwrap())
                            } else {
                                match self.order {
                                    PlaybackOrder::Random => repo.random_commit(),
                                    PlaybackOrder::Asc => repo.next_asc_commit(),
                                    PlaybackOrder::Desc => repo.next_desc_commit(),
                                }
                            };
                            match result {
                                Ok(metadata) => {
                                    self.load_commit(metadata);
                                }
                                Err(_) => {
                                    if self.loop_playback {
                                        repo.reset_index();
                                        let restart_result = if self.is_range_mode {
                                            match self.order {
                                                PlaybackOrder::Random => repo.random_range_commit(),
                                                PlaybackOrder::Asc => repo.next_range_commit_asc(),
                                                PlaybackOrder::Desc => {
                                                    repo.next_range_commit_desc()
                                                }
                                            }
                                        } else {
                                            match self.order {
                                                PlaybackOrder::Random => repo.random_commit(),
                                                PlaybackOrder::Asc => repo.next_asc_commit(),
                                                PlaybackOrder::Desc => repo.next_desc_commit(),
                                            }
                                        };
                                        match restart_result {
                                            Ok(metadata) => {
                                                self.load_commit(metadata);
                                            }
                                            Err(_) => {
                                                self.state = UIState::Finished;
                                            }
                                        }
                                    } else {
                                        self.state = UIState::Finished;
                                    }
                                }
                            }
                        } else {
                            self.state = UIState::Finished;
                        }
                    }
                }
                UIState::Finished => {
                    break;
                }
            }
        }

        Ok(())
    }

    fn render(&mut self, f: &mut Frame) {
        let size = f.area();

        // Split horizontally: left column | right column
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Left column (file tree + commit info)
                Constraint::Percentage(70), // Right column (editor + terminal)
            ])
            .margin(0)
            .spacing(0)
            .split(size);

        // Split left column vertically: file tree | separator | commit info
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(80), // File tree
                Constraint::Length(1),      // Horizontal separator
                Constraint::Percentage(20), // Commit info
            ])
            .margin(0)
            .spacing(0)
            .split(main_layout[0]);

        // Split right column vertically: editor | separator | terminal
        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(80), // Editor
                Constraint::Length(1),      // Horizontal separator
                Constraint::Percentage(20), // Terminal
            ])
            .margin(0)
            .spacing(0)
            .split(main_layout[1]);

        let separator_color = self.theme.separator;

        // Update file tree data if needed
        if let Some(metadata) = self.engine.current_metadata() {
            self.file_tree.set_commit_metadata(
                metadata,
                self.engine.current_file_index,
                &self.theme,
            );
        }

        // Render file tree
        self.file_tree.render(f, left_layout[0], &self.theme);

        // Render horizontal separator between file tree and commit info (left column)
        let left_sep = Paragraph::new(Line::from("─".repeat(left_layout[1].width as usize))).style(
            Style::default()
                .fg(separator_color)
                .bg(self.theme.background_left),
        );
        f.render_widget(left_sep, left_layout[1]);

        // Render commit info
        self.status_bar.render(
            f,
            left_layout[2],
            self.engine.current_metadata(),
            &self.theme,
        );

        // Render editor
        self.editor
            .render(f, right_layout[0], &self.engine, &self.theme);

        // Render horizontal separator between editor and terminal (right column)
        let right_sep = Paragraph::new(Line::from("─".repeat(right_layout[1].width as usize)))
            .style(
                Style::default()
                    .fg(separator_color)
                    .bg(self.theme.background_right),
            );
        f.render_widget(right_sep, right_layout[1]);

        // Render terminal
        self.terminal
            .render(f, right_layout[2], &self.engine, &self.theme);

        // Render dialog if present
        if let Some(ref title) = self.engine.dialog_title {
            let text = &self.engine.dialog_typing_text;
            let text_display_width = text.width();
            let dialog_width = (text_display_width + 10).max(60).min(size.width as usize) as u16;
            let dialog_height = 3;
            let dialog_x = (size.width.saturating_sub(dialog_width)) / 2;
            let dialog_y = (size.height.saturating_sub(dialog_height)) / 2;

            let dialog_area = Rect {
                x: dialog_x,
                y: dialog_y,
                width: dialog_width,
                height: dialog_height,
            };

            // Calculate content width (dialog_width - borders(2) - padding(2))
            let content_width = dialog_width.saturating_sub(4) as usize;
            let padding_len = content_width.saturating_sub(text_display_width);

            let spans = vec![
                Span::styled(
                    text.clone(),
                    Style::default().fg(self.theme.file_tree_current_file_fg),
                ),
                Span::styled(
                    " ".repeat(padding_len),
                    Style::default().bg(self.theme.editor_cursor_line_bg),
                ),
            ];

            let dialog_text = vec![Line::from(spans)];

            let block = Block::default()
                .borders(Borders::ALL)
                .title(title.clone())
                .padding(Padding::horizontal(1))
                .style(
                    Style::default()
                        .fg(self.theme.file_tree_current_file_fg)
                        .bg(self.theme.editor_cursor_line_bg),
                );

            let dialog = Paragraph::new(dialog_text).block(block);
            f.render_widget(dialog, dialog_area);
        }
    }
}
