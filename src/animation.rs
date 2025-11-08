use crate::git::{CommitMetadata, DiffHunk, FileChange, LineChangeType};
use rand::Rng;
use std::time::{Duration, Instant};

/// Represents the current state of the editor buffer
#[derive(Debug, Clone)]
pub struct EditorBuffer {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
}

impl EditorBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
        }
    }

    pub fn from_content(content: &str) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };

        Self {
            lines,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
        }
    }

    pub fn insert_char(&mut self, line: usize, col: usize, ch: char) {
        if line >= self.lines.len() {
            self.lines.resize(line + 1, String::new());
        }
        let line_str = &mut self.lines[line];

        // Convert char index to byte index
        let byte_idx = line_str
            .char_indices()
            .nth(col)
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| line_str.len());

        line_str.insert(byte_idx, ch);
    }

    pub fn delete_char(&mut self, line: usize, col: usize) {
        if line >= self.lines.len() {
            return;
        }

        let line_str = &mut self.lines[line];

        // Convert char index to byte index
        if let Some((byte_idx, _)) = line_str.char_indices().nth(col) {
            line_str.remove(byte_idx);
        }
    }

    pub fn insert_line(&mut self, line: usize, content: String) {
        if line > self.lines.len() {
            self.lines.resize(line, String::new());
        }
        self.lines.insert(line, content);
    }

    pub fn delete_line(&mut self, line: usize) {
        if line < self.lines.len() {
            self.lines.remove(line);
        }
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }
}

/// Individual animation step
#[derive(Debug, Clone)]
pub enum AnimationStep {
    InsertChar { line: usize, col: usize, ch: char },
    DeleteChar { line: usize, col: usize },
    InsertLine { line: usize, content: String },
    DeleteLine { line: usize },
    MoveCursor { line: usize, col: usize },
    Pause { duration_ms: u64 },
    SwitchFile { file_index: usize, content: String },
    TerminalPrompt,
    TerminalTypeChar { ch: char },
    TerminalOutput { text: String },
}

/// Animation state machine
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationState {
    Idle,
    Playing,
    Paused,
    Finished,
}

/// Which pane is currently active
#[derive(Debug, Clone, PartialEq)]
pub enum ActivePane {
    Editor,
    Terminal,
}

/// Main animation engine
pub struct AnimationEngine {
    pub buffer: EditorBuffer,
    pub state: AnimationState,
    steps: Vec<AnimationStep>,
    current_step: usize,
    last_update: Instant,
    speed_ms: u64,
    next_step_delay: u64,
    pause_until: Option<Instant>,
    pub cursor_visible: bool,
    cursor_blink_timer: Instant,
    viewport_height: usize,
    pub current_file_index: usize,
    pub terminal_lines: Vec<String>,
    pub active_pane: ActivePane,
}

impl AnimationEngine {
    pub fn new(speed_ms: u64) -> Self {
        Self {
            buffer: EditorBuffer::new(),
            state: AnimationState::Idle,
            steps: Vec::new(),
            current_step: 0,
            last_update: Instant::now(),
            speed_ms,
            next_step_delay: speed_ms,
            pause_until: None,
            cursor_visible: true,
            cursor_blink_timer: Instant::now(),
            viewport_height: 20, // Default, will be updated from UI
            current_file_index: 0,
            terminal_lines: Vec::new(),
            active_pane: ActivePane::Terminal, // Start with terminal (git checkout)
        }
    }

    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
    }

    /// Add a terminal command with typing animation
    fn add_terminal_command(&mut self, command: &str) {
        self.steps.push(AnimationStep::TerminalPrompt);
        for ch in command.chars() {
            self.steps.push(AnimationStep::TerminalTypeChar { ch });
        }
    }

    /// Load a commit and generate animation steps
    pub fn load_commit(&mut self, metadata: &CommitMetadata) {
        self.steps.clear();
        self.current_step = 0;
        self.state = AnimationState::Playing;
        self.current_file_index = 0;
        self.terminal_lines.clear();

        // Git checkout to parent commit (simulation)
        let parent_hash = format!("{}^", &metadata.hash[..7]);
        self.add_terminal_command(&format!("git checkout {}", parent_hash));
        self.steps.push(AnimationStep::Pause { duration_ms: 500 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!("HEAD is now at {} Previous commit", parent_hash),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: 1000 });

        // Process all file changes
        for (index, change) in metadata.changes.iter().enumerate() {
            // Open file in editor
            if index == 0 {
                self.steps.push(AnimationStep::Pause { duration_ms: 1000 });
            } else {
                self.steps.push(AnimationStep::Pause { duration_ms: 1500 });
            }
            self.add_terminal_command(&format!("open {}", change.path));
            self.steps.push(AnimationStep::Pause { duration_ms: 500 });

            // Add file switch step
            let content = change.old_content.clone().unwrap_or_default();
            self.steps.push(AnimationStep::SwitchFile {
                file_index: index,
                content: content.clone(),
            });

            // Add pause before starting file animation
            self.steps.push(AnimationStep::Pause { duration_ms: 800 });

            // Generate animation steps for this file
            self.generate_steps_for_file(change);

            // Git add this file after editing
            self.steps.push(AnimationStep::Pause { duration_ms: 1000 });
            self.add_terminal_command(&format!("git add {}", change.path));
            self.steps.push(AnimationStep::Pause { duration_ms: 500 });
        }

        // Git commit
        let commit_message = metadata.message.lines().next().unwrap_or("Update");
        self.add_terminal_command(&format!("git commit -m \"{}\"", commit_message));
        self.steps.push(AnimationStep::Pause { duration_ms: 800 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!("[main {}] {}", &metadata.hash[..7], commit_message),
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!(
                " {} file{} changed",
                metadata.changes.len(),
                if metadata.changes.len() == 1 { "" } else { "s" }
            ),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: 1000 });

        // Git push
        self.add_terminal_command("git push origin main");
        self.steps.push(AnimationStep::Pause { duration_ms: 500 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "Enumerating objects: 5, done.".to_string(),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: 300 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "Counting objects: 100% (5/5), done.".to_string(),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: 300 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "Writing objects: 100% (3/3), done.".to_string(),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: 500 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!("To https://github.com/user/repo.git"),
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!("   {}..{} main -> main", &parent_hash[..7], &metadata.hash[..7]),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: 2000 });

        // Start with empty editor (no file opened yet)
        self.buffer = EditorBuffer::new();
    }

    /// Generate animation steps for a file change
    fn generate_steps_for_file(&mut self, change: &FileChange) {
        let mut current_cursor_line = 0;

        // Process each hunk
        for hunk in &change.hunks {
            // Move cursor line by line to the start of the hunk
            let target_line = hunk.old_start;
            current_cursor_line =
                self.generate_cursor_movement(current_cursor_line, target_line);

            current_cursor_line = self.generate_steps_for_hunk(hunk, current_cursor_line);

            // Add pause between hunks
            self.steps.push(AnimationStep::Pause { duration_ms: 1500 });
        }
    }

    /// Generate cursor movement steps from current line to target line
    fn generate_cursor_movement(&mut self, from_line: usize, to_line: usize) -> usize {
        if from_line == to_line {
            return to_line;
        }

        if from_line < to_line {
            // Move down
            for line in (from_line + 1)..=to_line {
                self.steps.push(AnimationStep::MoveCursor { line, col: 0 });
                self.steps.push(AnimationStep::Pause { duration_ms: 50 });
            }
        } else {
            // Move up
            for line in (to_line..from_line).rev() {
                self.steps.push(AnimationStep::MoveCursor { line, col: 0 });
                self.steps.push(AnimationStep::Pause { duration_ms: 50 });
            }
        }

        self.steps.push(AnimationStep::Pause { duration_ms: 300 });
        to_line
    }

    /// Generate animation steps for a diff hunk
    /// Returns the final cursor line position
    fn generate_steps_for_hunk(&mut self, hunk: &DiffHunk, start_line: usize) -> usize {
        // buffer_line tracks the actual line number in the current buffer
        let mut buffer_line = hunk.old_start;
        let mut cursor_line = start_line;

        for line_change in &hunk.lines {
            match line_change.change_type {
                LineChangeType::Deletion => {
                    // Delete the entire line at current buffer position
                    self.steps.push(AnimationStep::DeleteLine {
                        line: buffer_line,
                    });
                    self.steps.push(AnimationStep::Pause { duration_ms: 300 });
                    cursor_line = buffer_line;
                    // After deletion, buffer_line stays the same
                    // (the next line moves up to this position)
                }
                LineChangeType::Addition => {
                    // Insert empty line at current buffer position
                    self.steps.push(AnimationStep::InsertLine {
                        line: buffer_line,
                        content: String::new(),
                    });

                    // Type each character
                    let mut col = 0;
                    for ch in line_change.content.chars() {
                        self.steps.push(AnimationStep::InsertChar {
                            line: buffer_line,
                            col,
                            ch,
                        });
                        col += 1;
                    }

                    cursor_line = buffer_line;
                    buffer_line += 1; // Move to next line after insertion
                    self.steps.push(AnimationStep::Pause { duration_ms: 200 });
                }
                LineChangeType::Context => {
                    // Move cursor to next line if needed
                    if buffer_line != cursor_line {
                        self.steps.push(AnimationStep::MoveCursor {
                            line: buffer_line,
                            col: 0,
                        });
                        self.steps.push(AnimationStep::Pause { duration_ms: 50 });
                    }
                    cursor_line = buffer_line;
                    buffer_line += 1; // Move to next line
                }
            }
        }

        cursor_line
    }

    /// Update animation state and return true if display needs refresh
    pub fn tick(&mut self) -> bool {
        // Handle cursor blinking
        if self.cursor_blink_timer.elapsed() >= Duration::from_millis(500) {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink_timer = Instant::now();
        }

        // Check if we're paused
        if let Some(pause_until) = self.pause_until {
            if Instant::now() < pause_until {
                return true; // Keep blinking cursor during pause
            }
            self.pause_until = None;
        }

        if self.state != AnimationState::Playing {
            return false;
        }

        // Check if it's time for next step
        if self.last_update.elapsed() < Duration::from_millis(self.next_step_delay) {
            return false;
        }

        // Execute next step
        if self.current_step >= self.steps.len() {
            self.state = AnimationState::Finished;
            return false;
        }

        let step = self.steps[self.current_step].clone();
        self.execute_step(step);
        self.current_step += 1;
        self.last_update = Instant::now();

        true
    }

    fn execute_step(&mut self, step: AnimationStep) {
        // Calculate delay for next step with randomization for typing steps
        let mut rng = rand::thread_rng();
        self.next_step_delay = match &step {
            AnimationStep::InsertChar { .. } | AnimationStep::TerminalTypeChar { .. } => {
                // Add 70-130% variation to typing speed
                let variation = rng.gen_range(0.7..=1.3);
                ((self.speed_ms as f64) * variation) as u64
            }
            _ => {
                // Other steps use base speed
                self.speed_ms
            }
        };

        match step {
            AnimationStep::InsertChar { line, col, ch } => {
                self.active_pane = ActivePane::Editor;
                self.buffer.insert_char(line, col, ch);
                self.buffer.cursor_line = line;
                self.buffer.cursor_col = col + 1;
            }
            AnimationStep::DeleteChar { line, col } => {
                self.active_pane = ActivePane::Editor;
                self.buffer.delete_char(line, col);
                self.buffer.cursor_line = line;
                self.buffer.cursor_col = col;
            }
            AnimationStep::InsertLine { line, content } => {
                self.active_pane = ActivePane::Editor;
                self.buffer.insert_line(line, content);
                self.buffer.cursor_line = line;
                self.buffer.cursor_col = 0;
            }
            AnimationStep::DeleteLine { line } => {
                self.active_pane = ActivePane::Editor;
                self.buffer.delete_line(line);
                self.buffer.cursor_line = line;
                self.buffer.cursor_col = 0;
            }
            AnimationStep::MoveCursor { line, col } => {
                self.active_pane = ActivePane::Editor;
                self.buffer.cursor_line = line;
                self.buffer.cursor_col = col;
            }
            AnimationStep::Pause { duration_ms } => {
                self.pause_until = Some(Instant::now() + Duration::from_millis(duration_ms));
            }
            AnimationStep::SwitchFile {
                file_index,
                content,
            } => {
                self.active_pane = ActivePane::Editor;
                // Switch to new file
                self.current_file_index = file_index;
                self.buffer = EditorBuffer::from_content(&content);
            }
            AnimationStep::TerminalPrompt => {
                self.active_pane = ActivePane::Terminal;
                // Start a new command line with prompt
                self.terminal_lines.push("$ ".to_string());
            }
            AnimationStep::TerminalTypeChar { ch } => {
                self.active_pane = ActivePane::Terminal;
                // Add character to the last terminal line
                if let Some(last_line) = self.terminal_lines.last_mut() {
                    last_line.push(ch);
                }
            }
            AnimationStep::TerminalOutput { text } => {
                self.active_pane = ActivePane::Terminal;
                // Add output line
                self.terminal_lines.push(text);
            }
        }

        // Update scroll to keep cursor centered
        self.update_scroll();
    }

    fn update_scroll(&mut self) {
        if self.viewport_height == 0 {
            return;
        }

        let cursor_line = self.buffer.cursor_line;
        let total_lines = self.buffer.lines.len();
        let half_viewport = self.viewport_height / 2;

        // Try to center the cursor line
        let target_offset = if cursor_line < half_viewport {
            // Near the top of file, don't scroll
            0
        } else if cursor_line + half_viewport >= total_lines {
            // Near the bottom of file, show as much as possible
            total_lines.saturating_sub(self.viewport_height)
        } else {
            // Middle of file, center the cursor
            cursor_line.saturating_sub(half_viewport)
        };

        self.buffer.scroll_offset = target_offset;
    }

    pub fn is_finished(&self) -> bool {
        self.state == AnimationState::Finished
    }
}
