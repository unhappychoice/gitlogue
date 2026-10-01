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
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::animation::{AnimationEngine, SpeedRule, StepMode};
use crate::git::{CommitMetadata, DiffMode, GitRepository};
use crate::panes::{EditorPane, FileTreePane, StatusBarPane, TerminalPane};
use crate::theme::Theme;
use crate::watch::{adaptive_speed_multiplier, CommitWatcher};
use crate::PlaybackOrder;

#[derive(Debug, Clone, PartialEq)]
enum UIState {
    Playing,
    WaitingForNext { resume_at: Instant },
    Watching,
    Menu,
    KeyBindings,
    About,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlaybackState {
    Playing,
    Paused,
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
    playback_state: PlaybackState,
    history: Vec<CommitMetadata>,
    history_index: Option<usize>,
    menu_index: usize,
    prev_state: Option<Box<UIState>>,
    watcher: Option<CommitWatcher>,
    force_redraw: bool,
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
        Self::build(
            speed_ms,
            repo,
            theme,
            order,
            loop_playback,
            commit_spec,
            is_range_mode,
            speed_rules,
            should_exit,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        speed_ms: u64,
        repo: Option<&'a GitRepository>,
        theme: Theme,
        order: PlaybackOrder,
        loop_playback: bool,
        commit_spec: Option<String>,
        is_range_mode: bool,
        speed_rules: Vec<SpeedRule>,
        should_exit: Arc<AtomicBool>,
    ) -> Self {
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
            playback_state: PlaybackState::Playing,
            history: Vec::new(),
            history_index: None,
            menu_index: 0,
            prev_state: None,
            watcher: None,
            force_redraw: true,
        }
    }

    /// Sets the diff mode for working tree diff playback.
    pub fn set_diff_mode(&mut self, mode: Option<DiffMode>) {
        self.diff_mode = mode;
    }

    /// Enables watch mode: new commits detected by the watcher are played as they arrive.
    pub fn enable_watch(&mut self, watcher: CommitWatcher) {
        self.watcher = Some(watcher);
        self.state = UIState::Watching;
    }

    fn open_menu(&mut self) {
        self.prev_state = Some(Box::new(self.state.clone()));
        self.menu_index = 0;
        self.state = UIState::Menu;
        self.engine.pause();
    }

    fn close_menu(&mut self) {
        let restored = self
            .prev_state
            .take()
            .map(|s| *s)
            .unwrap_or(UIState::Playing);
        self.state = match restored {
            UIState::WaitingForNext { .. } => UIState::Playing,
            other => other,
        };
        if self.playback_state == PlaybackState::Playing {
            self.engine.resume();
        }
    }

    fn setup_signal_handler(should_exit: Arc<AtomicBool>) {
        ctrlc::set_handler(Self::build_signal_handler(
            should_exit,
            io::stdout,
            std::process::exit,
        ))
        .expect("Error setting Ctrl-C handler");
    }

    fn build_signal_handler<W, MakeWriter, Exit, ExitResult>(
        should_exit: Arc<AtomicBool>,
        make_writer: MakeWriter,
        exit: Exit,
    ) -> impl FnMut() + Send + 'static
    where
        W: io::Write,
        MakeWriter: Fn() -> W + Send + Sync + 'static,
        Exit: Fn(i32) -> ExitResult + Send + Sync + 'static,
    {
        let make_writer = Arc::new(make_writer);
        let exit = Arc::new(exit);
        move || {
            let mut writer = make_writer.as_ref()();
            Self::handle_external_signal(should_exit.as_ref(), &mut writer, |code| {
                exit.as_ref()(code)
            });
        }
    }

    fn handle_external_signal<W: io::Write, F, T>(
        should_exit: &AtomicBool,
        writer: &mut W,
        exit: F,
    ) -> T
    where
        F: FnOnce(i32) -> T,
    {
        // Restore terminal state before exiting
        let _ = disable_raw_mode();
        let _ = Self::leave_terminal_ui(writer);
        should_exit.store(true, Ordering::SeqCst);
        // Exit immediately for external signals (SIGTERM)
        exit(0)
    }

    fn leave_terminal_ui<W: io::Write>(writer: &mut W) -> io::Result<()> {
        execute!(
            writer,
            LeaveAlternateScreen,
            DisableMouseCapture,
            crossterm::cursor::Show
        )
    }

    /// Loads a commit and starts the animation.
    pub fn load_commit(&mut self, metadata: CommitMetadata) {
        self.play_commit(metadata, true);
    }

    fn play_commit(&mut self, metadata: CommitMetadata, record_history: bool) {
        if record_history {
            self.record_history(&metadata);
        }
        self.engine.load_commit(&metadata);
        match self.playback_state {
            PlaybackState::Playing => self.engine.resume(),
            PlaybackState::Paused => self.engine.pause(),
        }
        self.state = UIState::Playing;
    }

    fn record_history(&mut self, metadata: &CommitMetadata) {
        if let Some(index) = self.history_index {
            if index + 1 < self.history.len() {
                self.history.truncate(index + 1);
            }
        } else {
            self.history.clear();
        }

        self.history.push(metadata.clone());
        self.history_index = Some(self.history.len() - 1);
    }

    fn play_history_commit(&mut self, index: usize) -> bool {
        if let Some(metadata) = self.history.get(index).cloned() {
            self.history_index = Some(index);
            self.play_commit(metadata, false);
            return true;
        }

        false
    }

    fn toggle_pause(&mut self) {
        match self.playback_state {
            PlaybackState::Playing => {
                self.playback_state = PlaybackState::Paused;
                self.engine.pause();
            }
            PlaybackState::Paused => {
                self.playback_state = PlaybackState::Playing;
                self.engine.resume();
            }
        }
    }

    fn ensure_manual_pause(&mut self) {
        if self.playback_state != PlaybackState::Paused {
            self.playback_state = PlaybackState::Paused;
            self.engine.pause();
        }
    }

    fn step_line(&mut self) {
        self.ensure_manual_pause();
        let _ = self.engine.manual_step(StepMode::Line);
    }

    fn step_change(&mut self) {
        self.ensure_manual_pause();
        let _ = self.engine.manual_step(StepMode::Change);
    }

    fn step_line_back(&mut self) {
        self.ensure_manual_pause();
        let _ = self.engine.restore_line_checkpoint();
    }

    fn step_change_back(&mut self) {
        self.ensure_manual_pause();
        let _ = self.engine.restore_change_checkpoint();
    }

    fn handle_prev(&mut self) {
        if let Some(index) = self.history_index {
            if index > 0 {
                let target = index - 1;
                self.play_history_commit(target);
            }
        }
    }

    fn handle_next(&mut self) {
        if let Some(index) = self.history_index {
            if index + 1 < self.history.len() {
                let _ = self.play_history_commit(index + 1);
                return;
            }
        }

        if self.repo.is_none() && self.diff_mode.is_none() {
            return;
        }

        self.advance_to_next_commit();
    }

    fn advance_to_next_commit(&mut self) -> bool {
        if self.watcher.is_some() {
            return self.play_next_watched_commit();
        }

        if let Some(diff_mode) = self.diff_mode {
            if let Some(repo) = self.repo {
                match repo.get_working_tree_diff(diff_mode) {
                    Ok(metadata) if !metadata.changes.is_empty() => {
                        self.load_commit(metadata);
                        return true;
                    }
                    _ => {
                        self.state = UIState::Finished;
                        return false;
                    }
                }
            }
            self.state = UIState::Finished;
            return false;
        }

        let Some(repo) = self.repo else {
            self.state = UIState::Finished;
            return false;
        };

        match self.fetch_repo_commit(repo) {
            Ok(metadata) => {
                self.load_commit(metadata);
                true
            }
            Err(_) => {
                if self.loop_playback {
                    repo.reset_index();
                    if let Ok(metadata) = self.fetch_repo_commit(repo) {
                        self.load_commit(metadata);
                        true
                    } else {
                        self.state = UIState::Finished;
                        false
                    }
                } else {
                    self.state = UIState::Finished;
                    false
                }
            }
        }
    }

    fn play_next_watched_commit(&mut self) -> bool {
        let next = self
            .watcher
            .as_mut()
            .and_then(CommitWatcher::next_commit)
            .zip(self.repo)
            .and_then(|(hash, repo)| repo.get_commit(&hash).ok());

        match next {
            Some(metadata) => {
                self.load_commit(metadata);
                true
            }
            None => {
                self.state = UIState::Watching;
                false
            }
        }
    }

    /// Shows the startup HEAD commit in its final state so the idle screen is not empty.
    fn show_initial_head(&mut self) {
        let Some(metadata) = self
            .watcher
            .as_mut()
            .and_then(CommitWatcher::take_initial_head)
            .zip(self.repo)
            .and_then(|(hash, repo)| repo.get_commit(&hash).ok())
        else {
            return;
        };
        self.engine.load_commit(&metadata);
        while self.engine.manual_step(StepMode::Change) {}
        self.force_redraw = true;
    }

    fn poll_watcher(&mut self, now: Instant) {
        let (Some(watcher), Some(repo)) = (self.watcher.as_mut(), self.repo) else {
            return;
        };
        watcher.poll(repo, now);
        self.engine
            .set_speed_multiplier(adaptive_speed_multiplier(watcher.pending_len()));
    }

    fn fetch_repo_commit(&self, repo: &GitRepository) -> Result<CommitMetadata> {
        if self.is_range_mode {
            return match self.order {
                PlaybackOrder::Random => repo.random_range_commit(),
                PlaybackOrder::Asc => repo.next_range_commit_asc(),
                PlaybackOrder::Desc => repo.next_range_commit_desc(),
            };
        }

        if let Some(spec) = &self.commit_spec {
            return repo.get_commit(spec);
        }

        match self.order {
            PlaybackOrder::Random => repo.random_commit(),
            PlaybackOrder::Asc => repo.next_asc_commit(),
            PlaybackOrder::Desc => repo.next_desc_commit(),
        }
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        match &self.state {
            UIState::Menu => self.handle_menu_key(key.code),
            UIState::KeyBindings | UIState::About => self.handle_overlay_key(key.code),
            UIState::Finished => self.handle_finished_key(key),
            _ => self.handle_playback_key(key),
        }
    }

    fn handle_menu_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.close_menu(),
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu_index = self.menu_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu_index = (self.menu_index + 1).min(2);
            }
            KeyCode::Enter => self.select_menu_item(),
            _ => {}
        }
    }

    fn handle_overlay_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.state = UIState::Menu;
            }
            _ => {}
        }
    }

    fn handle_finished_key(&mut self, key: event::KeyEvent) {
        if Self::is_quit_key(key) {
            self.state = UIState::Finished;
        }
    }

    fn handle_playback_key(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Esc => self.open_menu(),
            KeyCode::Char(' ') => self.toggle_pause(),
            _ if Self::is_ctrl_c(key) => {
                self.state = UIState::Finished;
            }
            KeyCode::Char(ch) => match ch {
                'q' => self.state = UIState::Finished,
                'h' => self.step_line_back(),
                'l' => self.step_line(),
                'H' => self.step_change_back(),
                'L' => self.step_change(),
                'p' => self.handle_prev(),
                'n' => self.handle_next(),
                _ => {}
            },
            _ => {}
        }
    }

    fn select_menu_item(&mut self) {
        self.state = match self.menu_index {
            0 => UIState::KeyBindings,
            1 => UIState::About,
            _ => UIState::Finished,
        };
    }

    fn is_ctrl_c(key: event::KeyEvent) -> bool {
        key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    fn is_quit_key(key: event::KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('q')) || Self::is_ctrl_c(key)
    }

    fn advance_state_after_tick(&mut self, now: Instant) -> bool {
        self.show_initial_head();
        self.poll_watcher(now);

        match self.state {
            UIState::Playing if self.engine.is_finished() && self.watcher.is_some() => {
                self.state = UIState::Watching;
            }
            UIState::Watching if self.playback_state != PlaybackState::Paused => {
                self.play_next_watched_commit();
            }
            UIState::Playing if self.engine.is_finished() => {
                self.state = if self.repo.is_some() {
                    UIState::WaitingForNext {
                        resume_at: now + Duration::from_millis(self.speed_ms * 100),
                    }
                } else {
                    UIState::Finished
                };
            }
            UIState::WaitingForNext { resume_at }
                if now >= resume_at && self.playback_state != PlaybackState::Paused =>
            {
                self.advance_to_next_commit();
            }
            UIState::Finished => return false,
            _ => {}
        }

        true
    }

    fn sync_exit_state(&mut self) {
        if self.should_exit.load(Ordering::Relaxed) {
            self.state = UIState::Finished;
        }
    }

    fn draw_if_needed<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        needs_redraw: bool,
    ) -> Result<()>
    where
        B::Error: std::error::Error + Send + Sync + 'static,
    {
        needs_redraw
            .then(|| terminal.draw(|f| self.render(f)))
            .transpose()?;
        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        self.force_redraw = true;
        if let Event::Key(key) = event {
            self.handle_key_event(key);
        }
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
        Self::leave_terminal_ui(terminal.backend_mut())?;
        Ok(())
    }

    fn update_viewport(&mut self, size: Rect) {
        // Editor area: 70% (right column) × 80% (editor pane) = 56% of total height
        let viewport_height = (size.height as f32 * 0.70 * 0.80) as usize;
        // Editor width: 70% (right column)
        let content_width = (size.width as f32 * 0.70) as usize;
        self.engine.set_viewport_height(viewport_height);
        self.engine.set_content_width(content_width);
    }

    fn handle_pending_event<Poll, Read>(&mut self, poll: &mut Poll, read: &mut Read) -> Result<()>
    where
        Poll: FnMut(Duration) -> io::Result<bool>,
        Read: FnMut() -> io::Result<Event>,
    {
        if poll(Duration::from_millis(8))? {
            self.handle_event(read()?);
        }
        Ok(())
    }

    fn run_loop_with<B, Poll, Read, Now>(
        &mut self,
        terminal: &mut Terminal<B>,
        mut poll: Poll,
        mut read: Read,
        mut now: Now,
    ) -> Result<()>
    where
        B: Backend,
        B::Error: std::error::Error + Send + Sync + 'static,
        Poll: FnMut(Duration) -> io::Result<bool>,
        Read: FnMut() -> io::Result<Event>,
        Now: FnMut() -> Instant,
    {
        loop {
            self.sync_exit_state();

            self.update_viewport(terminal.size()?.into());
            let needs_redraw = self.engine.tick() | std::mem::take(&mut self.force_redraw);
            self.draw_if_needed(terminal, needs_redraw)?;
            self.handle_pending_event(&mut poll, &mut read)?;

            if !self.advance_state_after_tick(now()) {
                break;
            }
        }

        Ok(())
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        self.run_loop_with(terminal, event::poll, event::read, Instant::now)
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

        if self.state == UIState::Watching && self.engine.current_metadata().is_none() {
            self.render_watching_message(f, right_layout[0]);
        }

        // Render menu / key bindings / about overlays
        match self.state {
            UIState::Menu => self.render_menu(f, size),
            UIState::KeyBindings => self.render_keybindings(f, size),
            UIState::About => self.render_about(f, size),
            _ => {}
        }
    }

    fn render_watching_message(&self, f: &mut Frame, area: Rect) {
        let message = "Watching for new commits...";
        let message_area = Self::centered_rect(area, message.width() as u16, 1);
        let paragraph = Paragraph::new(message).style(
            Style::default()
                .fg(self.theme.status_no_commit)
                .bg(self.theme.background_right),
        );
        f.render_widget(paragraph, message_area);
    }

    fn render_menu(&self, f: &mut Frame, size: Rect) {
        let items = ["Key Bindings", "About", "Exit"];
        let lines: Vec<Line> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let marker = if i == self.menu_index { "> " } else { "  " };
                let style = if i == self.menu_index {
                    Style::default().fg(self.theme.file_tree_current_file_fg)
                } else {
                    Style::default().fg(self.theme.status_message)
                };
                Line::from(Span::styled(format!("{marker}{item}"), style))
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Menu (Esc to close) ")
            .padding(Padding::new(2, 2, 1, 1))
            .style(
                Style::default()
                    .fg(self.theme.file_tree_current_file_fg)
                    .bg(self.theme.editor_cursor_line_bg),
            );

        let dialog_width = 30u16;
        let dialog_height = (items.len() as u16) + 4; // borders + padding
        let area = Self::centered_rect(size, dialog_width, dialog_height);

        f.render_widget(Clear, area);
        f.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn render_keybindings(&self, f: &mut Frame, size: Rect) {
        let lines = vec![
            Line::from(Span::styled(
                "General",
                Style::default().fg(self.theme.file_tree_current_file_fg),
            )),
            Line::from("  Esc     Menu"),
            Line::from("  q       Quit"),
            Line::from("  Ctrl+c  Quit"),
            Line::from(""),
            Line::from(Span::styled(
                "Playback Controls",
                Style::default().fg(self.theme.file_tree_current_file_fg),
            )),
            Line::from("  Space   Play / Pause"),
            Line::from("  h / l   Step line back / forward"),
            Line::from("  H / L   Step change back / forward"),
            Line::from("  p / n   Previous / Next commit"),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Key Bindings (Esc to close) ")
            .padding(Padding::new(2, 2, 1, 1))
            .style(
                Style::default()
                    .fg(self.theme.status_message)
                    .bg(self.theme.editor_cursor_line_bg),
            );

        let dialog_height = (lines.len() as u16) + 4;
        let area = Self::centered_rect(size, 44, dialog_height);

        f.render_widget(Clear, area);
        f.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn render_about(&self, f: &mut Frame, size: Rect) {
        let version = env!("CARGO_PKG_VERSION");
        let lines = vec![
            Line::from(Span::styled(
                "gitlogue",
                Style::default().fg(self.theme.file_tree_current_file_fg),
            )),
            Line::from(format!("Version {version}")),
            Line::from(""),
            Line::from("A cinematic Git commit replay tool"),
            Line::from("for the terminal."),
            Line::from(""),
            Line::from("https://github.com/unhappychoice/gitlogue"),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" About (Esc to close) ")
            .padding(Padding::new(2, 2, 1, 1))
            .style(
                Style::default()
                    .fg(self.theme.status_message)
                    .bg(self.theme.editor_cursor_line_bg),
            );

        let dialog_height = (lines.len() as u16) + 4;
        let area = Self::centered_rect(size, 48, dialog_height);

        f.render_widget(Clear, area);
        f.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn centered_rect(outer: Rect, width: u16, height: u16) -> Rect {
        Rect {
            x: outer.x + (outer.width.saturating_sub(width)) / 2,
            y: outer.y + (outer.height.saturating_sub(height)) / 2,
            width: width.min(outer.width),
            height: height.min(outer.height),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use git2::{Repository, Signature, Time};
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};
    use std::cell::Cell;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering, Ordering as CounterOrdering};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestRepo {
        path: PathBuf,
        repo: Repository,
    }

    impl TestRepo {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique_id = format!(
                "{}_{}_{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                COUNTER.fetch_add(1, CounterOrdering::SeqCst)
            );
            let path = std::env::temp_dir().join(format!("gitlogue_ui_test_{unique_id}"));
            fs::create_dir_all(&path).unwrap();

            let repo = Repository::init(&path).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test User").unwrap();
            config.set_str("user.email", "test@example.com").unwrap();

            Self { path, repo }
        }

        fn write_file(&self, relative_path: &str, content: &str) {
            let file_path = self.path.join(relative_path);
            file_path
                .parent()
                .map(fs::create_dir_all)
                .transpose()
                .unwrap();
            fs::write(file_path, content).unwrap();
        }

        fn stage_file(&self, relative_path: &str) {
            let mut index = self.repo.index().unwrap();
            index.add_path(Path::new(relative_path)).unwrap();
            index.write().unwrap();
        }

        fn commit_file(
            &self,
            relative_path: &str,
            content: &str,
            message: &str,
            timestamp: i64,
        ) -> String {
            self.write_file(relative_path, content);
            self.stage_file(relative_path);

            let mut index = self.repo.index().unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = self.repo.find_tree(tree_id).unwrap();
            let signature =
                Signature::new("Test User", "test@example.com", &Time::new(timestamp, 0)).unwrap();
            let parent = self
                .repo
                .head()
                .ok()
                .and_then(|head| head.peel_to_commit().ok());

            let oid = match parent.as_ref() {
                Some(parent_commit) => self.repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[parent_commit],
                ),
                None => self
                    .repo
                    .commit(Some("HEAD"), &signature, &signature, message, &tree, &[]),
            }
            .unwrap();

            oid.to_string()
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn metadata(hash: &str, message: &str) -> CommitMetadata {
        CommitMetadata {
            hash: hash.to_string(),
            author: "Author".to_string(),
            date: DateTime::from_timestamp(1_704_067_200, 0)
                .unwrap()
                .with_timezone(&Utc),
            message: message.to_string(),
            changes: vec![],
        }
    }

    fn test_ui_with_repo<'a>(repo: Option<&'a GitRepository>) -> UI<'a> {
        UI::build(
            16,
            repo,
            Theme::default(),
            PlaybackOrder::Asc,
            false,
            None,
            false,
            Vec::new(),
            Arc::new(AtomicBool::new(false)),
        )
    }

    fn test_ui_with_exit_flag(should_exit: Arc<AtomicBool>) -> UI<'static> {
        UI::build(
            16,
            None,
            Theme::default(),
            PlaybackOrder::Asc,
            false,
            None,
            false,
            Vec::new(),
            should_exit,
        )
    }

    fn test_ui() -> UI<'static> {
        test_ui_with_repo(None)
    }

    fn ctrl_key_event(ch: char) -> event::KeyEvent {
        event::KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
    }

    fn key_event(code: KeyCode) -> event::KeyEvent {
        event::KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn quit_event() -> io::Result<Event> {
        Ok(Event::Key(key_event(KeyCode::Char('q'))))
    }

    #[derive(Clone)]
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    impl io::Write for SharedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn apply_until_metadata_is_visible(ui: &mut UI<'_>) {
        while ui.engine.current_metadata().is_none() {
            assert!(ui.engine.manual_step(StepMode::Change));
        }
    }

    fn render_buffer(ui: &mut UI<'static>, width: u16, height: u16) -> Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| ui.render(f)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn buffer_text(buffer: &Buffer) -> String {
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn leave_terminal_ui_writes_restore_escape_sequences() {
        let mut output = Vec::new();

        UI::leave_terminal_ui(&mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("\u{1b}[?1049l"));
        assert!(output.contains("\u{1b}[?1000l"));
        assert!(output.contains("\u{1b}[?25h"));
    }

    #[test]
    fn handle_external_signal_restores_terminal_sets_exit_flag_and_requests_exit() {
        let should_exit = AtomicBool::new(false);
        let exit_code = Cell::new(None);
        let mut output = Vec::new();

        UI::handle_external_signal(&should_exit, &mut output, |code| {
            exit_code.set(Some(code));
        });

        assert!(should_exit.load(Ordering::SeqCst));
        assert_eq!(exit_code.get(), Some(0));
        assert!(!output.is_empty());
    }

    #[test]
    fn build_signal_handler_invokes_external_signal_cleanup() {
        let should_exit = Arc::new(AtomicBool::new(false));
        let exit_code = Arc::new(Mutex::new(None));
        let output = Arc::new(Mutex::new(Vec::new()));

        let mut handler = UI::build_signal_handler(
            should_exit.clone(),
            {
                let output = output.clone();
                move || SharedWriter(output.clone())
            },
            {
                let exit_code = exit_code.clone();
                move |code| *exit_code.lock().unwrap() = Some(code)
            },
        );

        handler();

        assert!(should_exit.load(Ordering::SeqCst));
        assert_eq!(*exit_code.lock().unwrap(), Some(0));
        assert!(!output.lock().unwrap().is_empty());
    }

    #[test]
    fn load_commit_tracks_history_navigation_and_truncates_future_entries() {
        let mut ui = test_ui();
        let first = metadata("1111111", "first");
        let second = metadata("2222222", "second");
        let third = metadata("3333333", "third");

        ui.load_commit(first.clone());
        ui.load_commit(second.clone());

        assert_eq!(ui.state, UIState::Playing);
        assert_eq!(ui.history_index, Some(1));
        assert_eq!(
            ui.history
                .iter()
                .map(|item| item.hash.as_str())
                .collect::<Vec<_>>(),
            vec!["1111111", "2222222"]
        );

        ui.handle_prev();
        assert_eq!(ui.history_index, Some(0));

        ui.handle_next();
        assert_eq!(ui.history_index, Some(1));

        ui.handle_prev();
        ui.load_commit(third);
        assert_eq!(ui.history_index, Some(1));
        assert_eq!(
            ui.history
                .iter()
                .map(|item| item.hash.as_str())
                .collect::<Vec<_>>(),
            vec!["1111111", "3333333"]
        );
    }

    #[test]
    fn menu_round_trip_restores_playing_state_from_waiting() {
        let mut ui = test_ui();
        ui.state = UIState::WaitingForNext {
            resume_at: Instant::now() + Duration::from_secs(1),
        };

        ui.handle_key_event(key_event(KeyCode::Esc));
        assert_eq!(ui.state, UIState::Menu);
        assert_eq!(ui.menu_index, 0);
        assert!(matches!(
            ui.prev_state.as_deref(),
            Some(UIState::WaitingForNext { .. })
        ));

        ui.handle_key_event(key_event(KeyCode::Esc));
        assert_eq!(ui.state, UIState::Playing);
        assert!(ui.prev_state.is_none());
    }

    #[test]
    fn close_menu_restores_previous_state_without_resuming_paused_playback() {
        let mut ui = test_ui();
        ui.state = UIState::Menu;
        ui.prev_state = Some(Box::new(UIState::About));
        ui.playback_state = PlaybackState::Paused;

        ui.close_menu();

        assert_eq!(ui.state, UIState::About);
        assert!(ui.prev_state.is_none());
        assert_eq!(ui.playback_state, PlaybackState::Paused);
    }

    #[test]
    fn sync_exit_state_marks_ui_finished_only_after_signal_flag_is_set() {
        let should_exit = Arc::new(AtomicBool::new(false));
        let mut ui = test_ui_with_exit_flag(should_exit.clone());

        ui.sync_exit_state();
        assert_eq!(ui.state, UIState::Playing);

        should_exit.store(true, Ordering::Relaxed);
        ui.sync_exit_state();
        assert_eq!(ui.state, UIState::Finished);
    }

    #[test]
    fn draw_if_needed_renders_only_when_requested() {
        let mut ui = test_ui();
        ui.state = UIState::Menu;

        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        ui.draw_if_needed(&mut terminal, false).unwrap();
        assert!(buffer_text(terminal.backend().buffer()).trim().is_empty());

        ui.draw_if_needed(&mut terminal, true).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Menu"));
        assert!(rendered.contains("Exit"));
    }

    #[test]
    fn handle_event_dispatches_keys_and_ignores_non_key_events() {
        let mut ignored_ui = test_ui();
        ignored_ui.handle_event(Event::Resize(80, 24));
        assert_eq!(ignored_ui.state, UIState::Playing);

        let mut quit_ui = test_ui();
        quit_ui.handle_event(Event::Key(key_event(KeyCode::Char('q'))));
        assert_eq!(quit_ui.state, UIState::Finished);
    }

    #[test]
    fn handle_pending_event_reads_input_only_when_poll_reports_ready() {
        let mut idle_ui = test_ui();
        let idle_polls = Cell::new(0);
        idle_ui
            .handle_pending_event(
                &mut |duration| {
                    idle_polls.set(idle_polls.get() + 1);
                    assert_eq!(duration, Duration::from_millis(8));
                    Ok(false)
                },
                &mut quit_event,
            )
            .unwrap();
        assert_eq!(idle_polls.get(), 1);
        assert_eq!(idle_ui.state, UIState::Playing);

        let mut ready_ui = test_ui();
        let ready_polls = Cell::new(0);
        let reads = Cell::new(0);
        ready_ui
            .handle_pending_event(
                &mut |duration| {
                    ready_polls.set(ready_polls.get() + 1);
                    assert_eq!(duration, Duration::from_millis(8));
                    Ok(true)
                },
                &mut || {
                    reads.set(reads.get() + 1);
                    quit_event()
                },
            )
            .unwrap();
        assert_eq!(ready_polls.get(), 1);
        assert_eq!(reads.get(), 1);
        assert_eq!(ready_ui.state, UIState::Finished);
    }

    #[test]
    fn run_loop_with_breaks_when_state_advance_requests_exit() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut finished_ui = test_ui();
        finished_ui.state = UIState::Finished;
        let idle_polls = Cell::new(0);
        finished_ui
            .run_loop_with(
                &mut terminal,
                |duration| {
                    idle_polls.set(idle_polls.get() + 1);
                    assert_eq!(duration, Duration::from_millis(8));
                    Ok(false)
                },
                quit_event,
                Instant::now,
            )
            .unwrap();
        assert_eq!(idle_polls.get(), 1);

        let mut quitting_ui = test_ui();
        let ready_polls = Cell::new(0);
        let reads = Cell::new(0);
        quitting_ui
            .run_loop_with(
                &mut terminal,
                |duration| {
                    ready_polls.set(ready_polls.get() + 1);
                    assert_eq!(duration, Duration::from_millis(8));
                    Ok(true)
                },
                || {
                    reads.set(reads.get() + 1);
                    quit_event()
                },
                Instant::now,
            )
            .unwrap();
        assert_eq!(ready_polls.get(), 1);
        assert_eq!(reads.get(), 1);
        assert_eq!(quitting_ui.state, UIState::Finished);

        let mut looping_ui = test_ui();
        let loop_polls = Cell::new(0);
        let loop_reads = Cell::new(0);
        looping_ui
            .run_loop_with(
                &mut terminal,
                |duration| {
                    let next = loop_polls.get() + 1;
                    loop_polls.set(next);
                    assert_eq!(duration, Duration::from_millis(8));
                    Ok(next > 1)
                },
                || {
                    loop_reads.set(loop_reads.get() + 1);
                    quit_event()
                },
                Instant::now,
            )
            .unwrap();
        assert_eq!(loop_polls.get(), 2);
        assert_eq!(loop_reads.get(), 1);
        assert_eq!(looping_ui.state, UIState::Finished);
    }

    #[test]
    fn handle_key_event_navigates_menu_and_dialogs() {
        let mut ui = test_ui();
        ui.state = UIState::Menu;

        ui.handle_key_event(key_event(KeyCode::Down));
        ui.handle_key_event(key_event(KeyCode::Char('j')));
        ui.handle_key_event(key_event(KeyCode::Down));
        assert_eq!(ui.menu_index, 2);

        ui.handle_key_event(key_event(KeyCode::Up));
        ui.handle_key_event(key_event(KeyCode::Char('k')));
        assert_eq!(ui.menu_index, 0);

        ui.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(ui.state, UIState::KeyBindings);

        ui.handle_key_event(key_event(KeyCode::Char('q')));
        assert_eq!(ui.state, UIState::Menu);

        ui.menu_index = 1;
        ui.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(ui.state, UIState::About);

        ui.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(ui.state, UIState::Menu);

        ui.menu_index = 2;
        ui.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(ui.state, UIState::Finished);
    }

    #[test]
    fn handle_key_event_ignores_irrelevant_menu_overlay_and_finished_keys() {
        let mut menu_ui = test_ui();
        menu_ui.state = UIState::Menu;
        menu_ui.menu_index = 1;
        menu_ui.handle_key_event(key_event(KeyCode::Left));
        assert_eq!(menu_ui.menu_index, 1);

        let mut overlay_ui = test_ui();
        overlay_ui.state = UIState::About;
        overlay_ui.handle_key_event(key_event(KeyCode::Left));
        assert_eq!(overlay_ui.state, UIState::About);

        let mut finished_ui = test_ui();
        finished_ui.state = UIState::Finished;
        finished_ui.handle_key_event(key_event(KeyCode::Left));
        assert_eq!(finished_ui.state, UIState::Finished);
    }

    #[test]
    fn toggle_pause_and_manual_pause_update_playback_state() {
        let mut ui = test_ui();

        ui.toggle_pause();
        assert_eq!(ui.playback_state, PlaybackState::Paused);

        ui.ensure_manual_pause();
        assert_eq!(ui.playback_state, PlaybackState::Paused);

        ui.toggle_pause();
        assert_eq!(ui.playback_state, PlaybackState::Playing);
    }

    #[test]
    fn step_helpers_pause_playback_without_needing_checkpoints() {
        let mut ui = test_ui();

        ui.step_line();
        ui.step_change();
        ui.step_line_back();
        ui.step_change_back();

        assert_eq!(ui.playback_state, PlaybackState::Paused);
        assert_eq!(ui.state, UIState::Playing);
    }

    #[test]
    fn advance_to_next_commit_without_repo_finishes_ui() {
        let mut default_ui = test_ui();
        assert!(!default_ui.advance_to_next_commit());
        assert_eq!(default_ui.state, UIState::Finished);

        let mut diff_ui = test_ui();
        diff_ui.set_diff_mode(Some(DiffMode::Staged));
        assert!(!diff_ui.advance_to_next_commit());
        assert_eq!(diff_ui.state, UIState::Finished);
    }

    #[test]
    fn history_navigation_bounds_are_noops_without_targets() {
        let mut ui = test_ui();

        assert!(!ui.play_history_commit(0));

        ui.load_commit(metadata("1111111", "only"));
        ui.handle_prev();
        ui.handle_next();

        assert_eq!(ui.history_index, Some(0));
        assert_eq!(ui.state, UIState::Playing);
    }

    #[test]
    fn handle_prev_without_history_and_manual_shortcuts_are_safe() {
        let mut ui = test_ui();
        ui.handle_prev();
        ui.handle_key_event(key_event(KeyCode::Char('h')));
        ui.handle_key_event(key_event(KeyCode::Char('H')));
        ui.handle_key_event(key_event(KeyCode::Char('L')));
        ui.handle_key_event(key_event(KeyCode::Char('x')));
        ui.handle_key_event(key_event(KeyCode::Up));

        assert_eq!(ui.playback_state, PlaybackState::Paused);
        assert_eq!(ui.state, UIState::Playing);
        assert!(ui.history_index.is_none());
    }

    #[test]
    fn handle_key_event_maps_playback_shortcuts() {
        let mut ui = test_ui();
        ui.load_commit(metadata("1111111", "first"));
        ui.load_commit(metadata("2222222", "second"));

        ui.handle_key_event(key_event(KeyCode::Char(' ')));
        assert_eq!(ui.playback_state, PlaybackState::Paused);

        ui.handle_key_event(key_event(KeyCode::Char('p')));
        assert_eq!(ui.history_index, Some(0));

        ui.handle_key_event(key_event(KeyCode::Char('n')));
        assert_eq!(ui.history_index, Some(1));

        ui.handle_key_event(key_event(KeyCode::Char('l')));
        assert_eq!(ui.playback_state, PlaybackState::Paused);

        ui.handle_key_event(key_event(KeyCode::Esc));
        assert_eq!(ui.state, UIState::Menu);
    }

    #[test]
    fn handle_key_event_quit_shortcuts_finish_ui() {
        let mut playing_ui = test_ui();
        playing_ui.handle_key_event(ctrl_key_event('c'));
        assert_eq!(playing_ui.state, UIState::Finished);

        let mut quit_ui = test_ui();
        quit_ui.handle_key_event(key_event(KeyCode::Char('q')));
        assert_eq!(quit_ui.state, UIState::Finished);

        let mut finished_ui = test_ui();
        finished_ui.state = UIState::Finished;
        finished_ui.handle_key_event(ctrl_key_event('c'));
        assert_eq!(finished_ui.state, UIState::Finished);
    }

    #[test]
    fn key_helpers_only_treat_expected_inputs_as_quit_shortcuts() {
        assert!(UI::is_ctrl_c(ctrl_key_event('c')));
        assert!(!UI::is_ctrl_c(key_event(KeyCode::Char('c'))));
        assert!(!UI::is_ctrl_c(ctrl_key_event('x')));

        assert!(UI::is_quit_key(key_event(KeyCode::Char('q'))));
        assert!(UI::is_quit_key(ctrl_key_event('c')));
        assert!(!UI::is_quit_key(key_event(KeyCode::Enter)));
    }

    #[test]
    fn advance_to_next_commit_uses_repo_order_and_finishes_after_last_commit() -> Result<()> {
        let test_repo = TestRepo::new();
        let first = test_repo.commit_file("src/lib.rs", "fn first() {}\n", "first", 1);
        let second = test_repo.commit_file("src/lib.rs", "fn second() {}\n", "second", 2);
        let repo = GitRepository::open(&test_repo.path)?;
        let mut ui = test_ui_with_repo(Some(&repo));
        ui.order = PlaybackOrder::Asc;

        assert!(ui.advance_to_next_commit());
        assert_eq!(
            ui.history.last().map(|item| item.hash.as_str()),
            Some(first.as_str())
        );

        assert!(ui.advance_to_next_commit());
        assert_eq!(
            ui.history.last().map(|item| item.hash.as_str()),
            Some(second.as_str())
        );

        assert!(!ui.advance_to_next_commit());
        assert_eq!(ui.state, UIState::Finished);

        Ok(())
    }

    #[test]
    fn handle_next_advances_repo_when_history_has_no_future() -> Result<()> {
        let test_repo = TestRepo::new();
        let first = test_repo.commit_file("src/lib.rs", "fn first() {}\n", "first", 1);
        let repo = GitRepository::open(&test_repo.path)?;
        let mut ui = test_ui_with_repo(Some(&repo));

        ui.handle_next();

        assert_eq!(
            ui.history.last().map(|item| item.hash.as_str()),
            Some(first.as_str())
        );
        assert_eq!(ui.state, UIState::Playing);

        Ok(())
    }

    #[test]
    fn advance_to_next_commit_restarts_from_beginning_when_looping() -> Result<()> {
        let test_repo = TestRepo::new();
        let only = test_repo.commit_file("src/lib.rs", "fn only() {}\n", "only", 1);
        let repo = GitRepository::open(&test_repo.path)?;
        let mut ui = test_ui_with_repo(Some(&repo));
        ui.order = PlaybackOrder::Desc;
        ui.loop_playback = true;

        assert!(ui.advance_to_next_commit());
        assert!(ui.advance_to_next_commit());

        let hashes = ui
            .history
            .iter()
            .map(|item| item.hash.as_str())
            .collect::<Vec<_>>();

        assert_eq!(hashes, vec![only.as_str(), only.as_str()]);
        assert_eq!(ui.state, UIState::Playing);

        Ok(())
    }

    #[test]
    fn advance_to_next_commit_finishes_when_loop_reset_cannot_fetch() -> Result<()> {
        let test_repo = TestRepo::new();
        test_repo.commit_file("src/lib.rs", "fn only() {}\n", "only", 1);
        let repo = GitRepository::open(&test_repo.path)?;
        let mut ui = test_ui_with_repo(Some(&repo));
        ui.commit_spec = Some("missing".to_string());
        ui.loop_playback = true;

        assert!(!ui.advance_to_next_commit());
        assert_eq!(ui.state, UIState::Finished);

        Ok(())
    }

    #[test]
    fn fetch_repo_commit_supports_random_commit_specs_and_ranges() -> Result<()> {
        let test_repo = TestRepo::new();
        let first = test_repo.commit_file("src/lib.rs", "fn first() {}\n", "first", 1);
        let second = test_repo.commit_file("src/lib.rs", "fn second() {}\n", "second", 2);
        let third = test_repo.commit_file("src/lib.rs", "fn third() {}\n", "third", 3);
        let repo = GitRepository::open(&test_repo.path)?;

        let mut random_ui = test_ui_with_repo(Some(&repo));
        random_ui.order = PlaybackOrder::Random;
        let random_hash = random_ui.fetch_repo_commit(&repo)?.hash;
        assert!([first.as_str(), second.as_str(), third.as_str()].contains(&random_hash.as_str()));

        let mut commit_ui = test_ui_with_repo(Some(&repo));
        commit_ui.commit_spec = Some(second.clone());
        assert_eq!(commit_ui.fetch_repo_commit(&repo)?.hash, second);

        let range = format!("{first}..{third}");
        repo.set_commit_range(&range)?;

        let mut range_random_ui = test_ui_with_repo(Some(&repo));
        range_random_ui.is_range_mode = true;
        range_random_ui.order = PlaybackOrder::Random;
        let random_hash = range_random_ui.fetch_repo_commit(&repo)?.hash;
        assert!([second.as_str(), third.as_str()].contains(&random_hash.as_str()));

        repo.set_commit_range(&range)?;

        let mut range_asc_ui = test_ui_with_repo(Some(&repo));
        range_asc_ui.is_range_mode = true;
        range_asc_ui.order = PlaybackOrder::Asc;
        assert_eq!(range_asc_ui.fetch_repo_commit(&repo)?.hash, second);

        repo.set_commit_range(&range)?;

        let mut range_desc_ui = test_ui_with_repo(Some(&repo));
        range_desc_ui.is_range_mode = true;
        range_desc_ui.order = PlaybackOrder::Desc;
        assert_eq!(range_desc_ui.fetch_repo_commit(&repo)?.hash, third);

        Ok(())
    }

    #[test]
    fn diff_mode_refreshes_working_tree_and_finishes_when_clean() -> Result<()> {
        let clean_repo = TestRepo::new();
        clean_repo.commit_file("src/lib.rs", "fn clean() {}\n", "clean", 1);
        let clean_git_repo = GitRepository::open(&clean_repo.path)?;
        let mut clean_ui = test_ui_with_repo(Some(&clean_git_repo));
        clean_ui.set_diff_mode(Some(DiffMode::Staged));

        assert!(!clean_ui.advance_to_next_commit());
        assert_eq!(clean_ui.state, UIState::Finished);

        let dirty_repo = TestRepo::new();
        dirty_repo.commit_file("src/lib.rs", "fn clean() {}\n", "clean", 1);
        dirty_repo.write_file("src/lib.rs", "fn dirty() {\n    println!(\"hi\");\n}\n");
        dirty_repo.stage_file("src/lib.rs");

        let dirty_git_repo = GitRepository::open(&dirty_repo.path)?;
        let mut dirty_ui = test_ui_with_repo(Some(&dirty_git_repo));
        dirty_ui.set_diff_mode(Some(DiffMode::Staged));

        assert!(dirty_ui.advance_to_next_commit());
        assert_eq!(
            dirty_ui
                .history
                .last()
                .map(|item| (item.hash.as_str(), item.message.as_str())),
            Some(("working-tree", "Staged changes"))
        );

        Ok(())
    }

    #[test]
    fn render_menu_shows_dialog_and_selected_item() {
        let mut ui = test_ui();
        ui.state = UIState::Menu;
        ui.menu_index = 1;
        ui.engine.dialog_title = Some("Open File...".to_string());
        ui.engine.dialog_typing_text = "src/ui.rs".to_string();

        let text = buffer_text(&render_buffer(&mut ui, 80, 24));

        assert!(text.contains("Menu (Esc to close)"));
        assert!(text.contains("> About"));
        assert!(text.contains("Open File"));
        assert!(text.contains("src/ui.rs"));
    }

    #[test]
    fn render_keybindings_and_about_overlays_include_expected_copy() {
        let mut keybindings_ui = test_ui();
        keybindings_ui.state = UIState::KeyBindings;
        let keybindings = buffer_text(&render_buffer(&mut keybindings_ui, 80, 24));

        assert!(keybindings.contains("Key Bindings (Esc to close)"));
        assert!(keybindings.contains("h / l   Step line back / forward"));
        assert!(keybindings.contains("p / n   Previous / Next commit"));

        let mut about_ui = test_ui();
        about_ui.state = UIState::About;
        let about = buffer_text(&render_buffer(&mut about_ui, 80, 24));

        assert!(about.contains("About (Esc to close)"));
        assert!(about.contains(&format!("Version {}", env!("CARGO_PKG_VERSION"))));
        assert!(about.contains("https://github.com/unhappychoice/gitlogue"));
    }

    fn watch_ui(repo: &GitRepository, now: Instant) -> UI<'_> {
        let mut ui = test_ui_with_repo(Some(repo));
        ui.enable_watch(CommitWatcher::new(repo, now));
        ui
    }

    fn finish_playback(ui: &mut UI<'_>) {
        while ui.engine.manual_step(StepMode::Change) {}
    }

    #[test]
    fn run_loop_draws_first_frame_even_without_animation_progress() {
        let mut ui = test_ui();
        ui.state = UIState::Watching;
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();

        ui.run_loop_with(&mut terminal, |_| Ok(true), quit_event, Instant::now)
            .unwrap();

        assert!(buffer_text(terminal.backend().buffer()).contains("Watching for new commits..."));
    }

    #[test]
    fn watch_mode_shows_head_in_final_state_without_playing_it() {
        let test_repo = TestRepo::new();
        let head = test_repo.commit_file("src/lib.rs", "fn old() {}\n", "existing", 1_700_000_000);
        let repo = GitRepository::open(&test_repo.path).unwrap();
        let now = Instant::now();
        let mut ui = watch_ui(&repo, now);

        assert!(ui.advance_state_after_tick(now + Duration::from_secs(2)));

        assert_eq!(ui.state, UIState::Watching);
        assert!(ui.history.is_empty());
        assert!(ui.engine.is_finished());
        assert_eq!(ui.engine.current_metadata().unwrap().hash, head);
        assert!(ui.force_redraw);
    }

    #[test]
    fn watch_mode_plays_new_commits_in_order_then_returns_to_watching() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("src/lib.rs", "fn old() {}\n", "existing", 1_700_000_000);
        let repo = GitRepository::open(&test_repo.path).unwrap();
        let now = Instant::now();
        let mut ui = watch_ui(&repo, now);
        let first = test_repo.commit_file("src/lib.rs", "fn first() {}\n", "first", 1_700_000_060);
        let second =
            test_repo.commit_file("src/lib.rs", "fn second() {}\n", "second", 1_700_000_120);

        ui.advance_state_after_tick(now + Duration::from_secs(2));
        assert_eq!(ui.state, UIState::Playing);
        assert_eq!(ui.history.last().unwrap().hash, first);

        finish_playback(&mut ui);
        ui.advance_state_after_tick(now + Duration::from_secs(2));
        ui.advance_state_after_tick(now + Duration::from_secs(2));
        assert_eq!(ui.history.last().unwrap().hash, second);

        finish_playback(&mut ui);
        ui.advance_state_after_tick(now + Duration::from_secs(2));
        ui.advance_state_after_tick(now + Duration::from_secs(2));
        assert_eq!(ui.state, UIState::Watching);
        assert_eq!(ui.history.len(), 2);
    }

    #[test]
    fn watch_mode_ignores_commits_brought_in_by_branch_switch() {
        let test_repo = TestRepo::new();
        let base = test_repo.commit_file("src/lib.rs", "fn old() {}\n", "existing", 1_700_000_000);
        let base_commit = test_repo
            .repo
            .find_commit(git2::Oid::from_str(&base).unwrap())
            .unwrap();
        test_repo
            .repo
            .branch("feature", &base_commit, false)
            .unwrap();
        let main_branch = test_repo.repo.head().unwrap().name().unwrap().to_string();
        test_repo.repo.set_head("refs/heads/feature").unwrap();
        test_repo.commit_file("src/lib.rs", "fn feature() {}\n", "feature", 1_700_000_060);
        test_repo.repo.set_head(&main_branch).unwrap();
        let repo = GitRepository::open(&test_repo.path).unwrap();
        let now = Instant::now();
        let mut ui = watch_ui(&repo, now);

        test_repo.repo.set_head("refs/heads/feature").unwrap();
        ui.advance_state_after_tick(now + Duration::from_secs(2));
        assert_eq!(ui.state, UIState::Watching);

        let next = test_repo.commit_file("src/lib.rs", "fn next() {}\n", "next", 1_700_000_120);
        ui.advance_state_after_tick(now + Duration::from_secs(4));
        assert_eq!(ui.state, UIState::Playing);
        assert_eq!(ui.history.last().unwrap().hash, next);
    }

    #[test]
    fn watch_mode_does_not_start_new_commit_while_paused() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("src/lib.rs", "fn old() {}\n", "existing", 1_700_000_000);
        let repo = GitRepository::open(&test_repo.path).unwrap();
        let now = Instant::now();
        let mut ui = watch_ui(&repo, now);
        ui.toggle_pause();
        test_repo.commit_file("src/lib.rs", "fn new() {}\n", "new", 1_700_000_060);

        ui.advance_state_after_tick(now + Duration::from_secs(2));
        assert_eq!(ui.state, UIState::Watching);

        ui.toggle_pause();
        ui.advance_state_after_tick(now + Duration::from_secs(2));
        assert_eq!(ui.state, UIState::Playing);
    }

    #[test]
    fn render_watching_state_shows_waiting_message() {
        let mut ui = test_ui();
        ui.state = UIState::Watching;

        let text = buffer_text(&render_buffer(&mut ui, 100, 30));

        assert!(text.contains("Watching for new commits..."));
    }

    #[test]
    fn centered_rect_clamps_requested_size_to_outer_area() {
        let rect = UI::centered_rect(Rect::new(4, 2, 20, 10), 40, 12);

        assert_eq!(rect, Rect::new(4, 2, 20, 10));
    }

    #[test]
    fn render_playing_state_uses_loaded_metadata_for_file_tree_and_status_bar() {
        let mut ui = test_ui();
        ui.load_commit(metadata("4444444", "render commit"));
        apply_until_metadata_is_visible(&mut ui);

        let text = buffer_text(&render_buffer(&mut ui, 80, 40));

        assert!(text.contains("hash: 4444444"));
        assert!(text.contains("render commit"));
    }

    #[test]
    fn advance_state_after_tick_transitions_finished_playback_to_waiting_or_finished() -> Result<()>
    {
        let now = Instant::now();

        let standalone_ui = {
            let mut ui = test_ui();
            ui.engine.state = crate::animation::AnimationState::Finished;
            ui
        };
        let mut standalone_ui = standalone_ui;
        assert!(standalone_ui.advance_state_after_tick(now));
        assert_eq!(standalone_ui.state, UIState::Finished);

        let test_repo = TestRepo::new();
        test_repo.commit_file("src/lib.rs", "fn keep() {}\n", "keep", 1);
        let repo = GitRepository::open(&test_repo.path)?;
        let mut repo_ui = test_ui_with_repo(Some(&repo));
        repo_ui.engine.state = crate::animation::AnimationState::Finished;

        assert!(repo_ui.advance_state_after_tick(now));
        assert!(matches!(
            repo_ui.state,
            UIState::WaitingForNext { resume_at }
                if resume_at == now + Duration::from_millis(repo_ui.speed_ms * 100)
        ));

        Ok(())
    }

    #[test]
    fn advance_state_after_tick_waits_while_paused_and_advances_when_resumed() -> Result<()> {
        let test_repo = TestRepo::new();
        let first = test_repo.commit_file("src/lib.rs", "fn first() {}\n", "first", 1);
        let repo = GitRepository::open(&test_repo.path)?;
        let resume_at = Instant::now() - Duration::from_millis(1);
        let mut ui = test_ui_with_repo(Some(&repo));
        ui.state = UIState::WaitingForNext { resume_at };
        ui.playback_state = PlaybackState::Paused;

        assert!(ui.advance_state_after_tick(Instant::now()));
        assert!(matches!(ui.state, UIState::WaitingForNext { .. }));
        assert!(ui.history.is_empty());

        ui.playback_state = PlaybackState::Playing;
        assert!(ui.advance_state_after_tick(Instant::now()));
        assert_eq!(ui.state, UIState::Playing);
        assert_eq!(
            ui.history.last().map(|item| item.hash.as_str()),
            Some(first.as_str())
        );

        Ok(())
    }

    #[test]
    fn advance_state_after_tick_keeps_waiting_before_resume_deadline() -> Result<()> {
        let test_repo = TestRepo::new();
        test_repo.commit_file("src/lib.rs", "fn first() {}\n", "first", 1);
        let repo = GitRepository::open(&test_repo.path)?;
        let resume_at = Instant::now() + Duration::from_secs(1);
        let mut ui = test_ui_with_repo(Some(&repo));
        ui.state = UIState::WaitingForNext { resume_at };

        assert!(ui.advance_state_after_tick(Instant::now()));
        assert!(matches!(ui.state, UIState::WaitingForNext { .. }));
        assert!(ui.history.is_empty());

        Ok(())
    }

    #[test]
    fn advance_state_after_tick_stops_loop_once_ui_is_finished() {
        let mut ui = test_ui();
        ui.state = UIState::Finished;

        assert!(!ui.advance_state_after_tick(Instant::now()));
    }
}
