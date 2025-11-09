use crate::git::{CommitMetadata, DiffHunk, FileChange, LineChangeType};
use crate::syntax::Highlighter;
use rand::Rng;
use std::cell::RefCell;
use std::time::{Duration, Instant};

// Duration multipliers relative to typing speed
const CURSOR_MOVE_PAUSE: f64 = 0.5;       // Cursor movement between lines (base speed)
const CURSOR_MOVE_END_PAUSE: f64 = 10.0;  // After cursor movement completes
const CURSOR_MOVE_SHORT_MULTIPLIER: f64 = 1.0;   // Speed for short distances (1-5 lines)
const CURSOR_MOVE_MEDIUM_MULTIPLIER: f64 = 0.3;  // Speed for medium distances (6-20 lines)
const CURSOR_MOVE_LONG_MULTIPLIER: f64 = 0.1;    // Speed for long distances (21+ lines)
const DELETE_LINE_PAUSE: f64 = 10.0;      // After deleting a line
const INSERT_LINE_PAUSE: f64 = 6.7;       // After inserting a line
const HUNK_PAUSE: f64 = 50.0;             // Between hunks
const CHECKOUT_PAUSE: f64 = 16.7;         // After git checkout command
const CHECKOUT_OUTPUT_PAUSE: f64 = 33.3;  // After git checkout output
const OPEN_FILE_FIRST_PAUSE: f64 = 33.3;  // Before opening first file
const OPEN_FILE_PAUSE: f64 = 50.0;        // Before opening subsequent files
const OPEN_CMD_PAUSE: f64 = 16.7;         // After open command
const FILE_SWITCH_PAUSE: f64 = 26.7;      // After switching file
const GIT_ADD_PAUSE: f64 = 33.3;          // Before git add
const GIT_ADD_CMD_PAUSE: f64 = 16.7;      // After git add command
const GIT_COMMIT_PAUSE: f64 = 26.7;       // After git commit command
const COMMIT_OUTPUT_PAUSE: f64 = 33.3;    // After commit output
const GIT_PUSH_PAUSE: f64 = 16.7;         // After git push command
const PUSH_OUTPUT_PAUSE: f64 = 10.0;      // Between push output lines
const PUSH_FINAL_PAUSE: f64 = 66.7;       // After final push output

/// Represents the current state of the editor buffer
#[derive(Debug, Clone)]
pub struct EditorBuffer {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub cached_highlights: Vec<crate::syntax::HighlightSpan>,
    /// Track which line is being edited and its byte offset change since last highlight update
    pub editing_line: Option<usize>,
    pub editing_insert_byte_position: usize, // Byte position within line where insertion started
    pub editing_line_byte_offset: isize, // Cumulative byte offset from insertion start
}

impl EditorBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            cached_highlights: Vec::new(),
            editing_line: None,
            editing_insert_byte_position: 0,
            editing_line_byte_offset: 0,
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
            cached_highlights: Vec::new(),
            editing_line: None,
            editing_insert_byte_position: 0,
            editing_line_byte_offset: 0,
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
}

/// Individual animation step
#[derive(Debug, Clone)]
pub enum AnimationStep {
    InsertChar { line: usize, col: usize, ch: char },
    InsertLine { line: usize, content: String },
    DeleteLine { line: usize },
    MoveCursor { line: usize, col: usize },
    Pause { duration_ms: u64 },
    SwitchFile { file_index: usize, content: String, path: String },
    UpdateHighlights,
    TerminalPrompt,
    TerminalTypeChar { ch: char },
    TerminalOutput { text: String },
}

/// Animation state machine
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationState {
    Idle,
    Playing,
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
    pub current_file_path: Option<String>,
    pub terminal_lines: Vec<String>,
    pub active_pane: ActivePane,
    pub highlighter: RefCell<Highlighter>,
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
            current_file_path: None,
            terminal_lines: Vec::new(),
            active_pane: ActivePane::Terminal, // Start with terminal (git checkout)
            highlighter: RefCell::new(Highlighter::new()),
        }
    }

    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
    }

    /// Update syntax highlighting for current buffer
    fn update_buffer_highlights(&mut self) {
        let full_content = self.buffer.lines.join("\n");
        self.buffer.cached_highlights = self.highlighter.borrow_mut().highlight(&full_content);
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
        self.last_update = Instant::now();
        self.pause_until = None;

        // Time travel to commit date
        let parent_hash = format!("{}^", &metadata.hash[..7]);
        let datetime_str = metadata.date.format("%Y-%m-%d %H:%M:%S").to_string();
        self.add_terminal_command(&format!("time-travel {}", datetime_str));
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * CHECKOUT_PAUSE) as u64 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "⚡ Initializing temporal displacement field...".to_string(),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * CHECKOUT_OUTPUT_PAUSE * 0.5) as u64 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "✨ Warping through spacetime...".to_string(),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * CHECKOUT_OUTPUT_PAUSE * 0.5) as u64 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!("🕰️  Arrived at {}", datetime_str),
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!("📍 Location: commit {} by {}", &metadata.hash[..7], metadata.author),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * CHECKOUT_OUTPUT_PAUSE) as u64 });

        // Process all file changes
        for (index, change) in metadata.changes.iter().enumerate() {
            // Open file in editor
            if index == 0 {
                self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * OPEN_FILE_FIRST_PAUSE) as u64 });
            } else {
                self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * OPEN_FILE_PAUSE) as u64 });
            }
            self.add_terminal_command(&format!("open {}", change.path));
            self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * OPEN_CMD_PAUSE) as u64 });

            // Add file switch step
            let content = change.old_content.clone().unwrap_or_default();
            self.steps.push(AnimationStep::SwitchFile {
                file_index: index,
                content: content.clone(),
                path: change.path.clone(),
            });

            // Update highlights for the new file
            self.steps.push(AnimationStep::UpdateHighlights);

            // Add pause before starting file animation
            self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * FILE_SWITCH_PAUSE) as u64 });

            // Generate animation steps for this file
            self.generate_steps_for_file(change);

            // Git add this file after editing
            self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * GIT_ADD_PAUSE) as u64 });
            self.add_terminal_command(&format!("git add {}", change.path));
            self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * GIT_ADD_CMD_PAUSE) as u64 });
        }

        // Git commit
        let commit_message = metadata.message.lines().next().unwrap_or("Update");
        self.add_terminal_command(&format!("git commit -m \"{}\"", commit_message));
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * GIT_COMMIT_PAUSE) as u64 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!("💾 [main {}] {}", &metadata.hash[..7], commit_message),
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!(
                "📝 {} file{} changed - immortalized forever!",
                metadata.changes.len(),
                if metadata.changes.len() == 1 { "" } else { "s" }
            ),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * COMMIT_OUTPUT_PAUSE) as u64 });

        // Git push
        self.add_terminal_command("git push origin main");
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * GIT_PUSH_PAUSE) as u64 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "🚀 Launching code into the cloud...".to_string(),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * PUSH_OUTPUT_PAUSE) as u64 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "📦 Compressing digital dreams: 100% (5/5)".to_string(),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * PUSH_OUTPUT_PAUSE) as u64 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "✍️  Signing with invisible ink: done.".to_string(),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * GIT_PUSH_PAUSE) as u64 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "📡 Beaming to origin/main via satellite...".to_string(),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * PUSH_OUTPUT_PAUSE) as u64 });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!("   {}..{} ✨ SUCCESS", &parent_hash[..7], &metadata.hash[..7]),
        });
        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * PUSH_FINAL_PAUSE) as u64 });

        // Start with empty editor (no file opened yet)
        self.buffer = EditorBuffer::new();
    }

    /// Generate animation steps for a file change
    fn generate_steps_for_file(&mut self, change: &FileChange) {
        let mut current_cursor_line = 0;
        let mut line_offset = 0i64; // Track how buffer lines differ from old file

        // Process each hunk
        for hunk in &change.hunks {
            // Calculate target line in current buffer
            // hunk.old_start is 1-indexed (Git line numbers start at 1)
            // We need to convert to 0-indexed and adjust by how many lines we've added/removed
            let target_line = ((hunk.old_start as i64) - 1 + line_offset).max(0) as usize;

            // Calculate distance for speed adjustment
            let distance = if current_cursor_line < target_line {
                target_line - current_cursor_line
            } else {
                current_cursor_line - target_line
            };

            current_cursor_line =
                self.generate_cursor_movement(current_cursor_line, target_line, distance);

            let (final_cursor_line, _final_buffer_line) =
                self.generate_steps_for_hunk(hunk, current_cursor_line, target_line);

            current_cursor_line = final_cursor_line;

            // Update offset based on changes in this hunk
            // Count additions and deletions to update the offset
            let additions = hunk.lines.iter()
                .filter(|l| matches!(l.change_type, LineChangeType::Addition))
                .count() as i64;
            let deletions = hunk.lines.iter()
                .filter(|l| matches!(l.change_type, LineChangeType::Deletion))
                .count() as i64;

            line_offset += additions - deletions;

            // Add pause between hunks
            self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * HUNK_PAUSE) as u64 });
        }
    }

    /// Generate cursor movement steps from current line to target line
    fn generate_cursor_movement(&mut self, from_line: usize, to_line: usize, distance: usize) -> usize {
        if from_line == to_line {
            return to_line;
        }

        // Determine speed multiplier based on total distance
        let speed_multiplier = if distance <= 5 {
            CURSOR_MOVE_SHORT_MULTIPLIER
        } else if distance <= 20 {
            CURSOR_MOVE_MEDIUM_MULTIPLIER
        } else {
            CURSOR_MOVE_LONG_MULTIPLIER
        };

        // Calculate pause per step
        let pause_per_step = (self.speed_ms as f64 * CURSOR_MOVE_PAUSE * speed_multiplier).max(1.0) as u64;

        if from_line < to_line {
            // Move down
            for line in (from_line + 1)..=to_line {
                self.steps.push(AnimationStep::MoveCursor { line, col: 0 });
                self.steps.push(AnimationStep::Pause { duration_ms: pause_per_step });
            }
        } else {
            // Move up
            for line in (to_line..from_line).rev() {
                self.steps.push(AnimationStep::MoveCursor { line, col: 0 });
                self.steps.push(AnimationStep::Pause { duration_ms: pause_per_step });
            }
        }

        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * CURSOR_MOVE_END_PAUSE) as u64 });
        to_line
    }

    /// Generate animation steps for a diff hunk
    /// Returns (final_cursor_line, final_buffer_line)
    fn generate_steps_for_hunk(
        &mut self,
        hunk: &DiffHunk,
        start_cursor_line: usize,
        start_buffer_line: usize,
    ) -> (usize, usize) {
        // buffer_line tracks the actual line number in the current buffer
        let mut buffer_line = start_buffer_line;
        let mut cursor_line = start_cursor_line;

        for line_change in &hunk.lines {
            match line_change.change_type {
                LineChangeType::Deletion => {
                    // Delete the entire line at current buffer position
                    self.steps.push(AnimationStep::DeleteLine {
                        line: buffer_line,
                    });
                    // Update highlights after line deletion
                    self.steps.push(AnimationStep::UpdateHighlights);
                    self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * DELETE_LINE_PAUSE) as u64 });
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

                    // Update highlights immediately after inserting empty line
                    // This ensures highlights for lines below are shifted correctly
                    self.steps.push(AnimationStep::UpdateHighlights);

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

                    // Update highlights again after line is completely typed
                    self.steps.push(AnimationStep::UpdateHighlights);
                    self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * INSERT_LINE_PAUSE) as u64 });
                }
                LineChangeType::Context => {
                    // Move cursor to next line if needed
                    if buffer_line != cursor_line {
                        self.steps.push(AnimationStep::MoveCursor {
                            line: buffer_line,
                            col: 0,
                        });
                        self.steps.push(AnimationStep::Pause { duration_ms: (self.speed_ms as f64 * CURSOR_MOVE_PAUSE) as u64 });
                    }
                    cursor_line = buffer_line;
                    buffer_line += 1; // Move to next line
                }
            }
        }

        (cursor_line, buffer_line)
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

                // Calculate byte position before insertion
                if self.buffer.editing_line != Some(line) {
                    // Starting edit on new line - calculate insertion byte position
                    let line_str = &self.buffer.lines[line];
                    let byte_pos = line_str.chars().take(col).map(|c| c.len_utf8()).sum();
                    self.buffer.editing_line = Some(line);
                    self.buffer.editing_insert_byte_position = byte_pos;
                    self.buffer.editing_line_byte_offset = 0;
                }

                self.buffer.insert_char(line, col, ch);
                self.buffer.cursor_line = line;
                self.buffer.cursor_col = col + 1;

                // Track editing offset for highlight adjustment
                self.buffer.editing_line_byte_offset += ch.len_utf8() as isize;
            }
            AnimationStep::InsertLine { line, content } => {
                self.active_pane = ActivePane::Editor;
                self.buffer.insert_line(line, content);
                self.buffer.cursor_line = line;
                self.buffer.cursor_col = 0;

                // Reset editing offset for new line (insertion starts at position 0)
                self.buffer.editing_line = Some(line);
                self.buffer.editing_insert_byte_position = 0;
                self.buffer.editing_line_byte_offset = 0;
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
                path,
            } => {
                self.active_pane = ActivePane::Editor;
                // Switch to new file
                self.current_file_index = file_index;
                self.current_file_path = Some(path.clone());
                self.buffer = EditorBuffer::from_content(&content);

                // Update syntax highlighter for new file
                // This will clear language settings if not supported
                self.highlighter.borrow_mut().set_language_from_path(&path);
            }
            AnimationStep::UpdateHighlights => {
                self.active_pane = ActivePane::Editor;
                self.update_buffer_highlights();
                // Reset editing offset after highlight update
                self.buffer.editing_line = None;
                self.buffer.editing_insert_byte_position = 0;
                self.buffer.editing_line_byte_offset = 0;
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
