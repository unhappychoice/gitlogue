use crate::git::{CommitMetadata, DiffHunk, FileChange, FileStatus, LineChangeType};
use crate::syntax::Highlighter;
use rand::Rng;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthStr;

// Duration multipliers relative to typing speed
const CURSOR_MOVE_PAUSE: f64 = 0.5; // Cursor movement between lines (base speed)
const CURSOR_MOVE_SHORT_MULTIPLIER: f64 = 1.0; // Speed for short distances (1-50 lines)
const CURSOR_MOVE_MEDIUM_MULTIPLIER: f64 = 0.3; // Speed for medium distances (51-200 lines)
const CURSOR_MOVE_LONG_MULTIPLIER: f64 = 0.05; // Speed for long distances (201+ lines)
const MAX_SCROLL_STEPS: usize = 60; // Maximum animation steps for any scroll distance
const MIN_LOG_STEPS: usize = 50; // Minimum steps for logarithmic scaling (aligned with SHORT threshold)
const LOG_SCALE_FACTOR: f64 = 8.0; // Scaling factor for logarithmic step calculation
const DELETE_LINE_PAUSE: f64 = 10.0; // After deleting a line
const INSERT_LINE_PAUSE: f64 = 6.7; // After inserting a line
const HUNK_PAUSE: f64 = 50.0; // Between hunks
const CHECKOUT_PAUSE: f64 = 16.7; // After git checkout command
const CHECKOUT_OUTPUT_PAUSE: f64 = 33.3; // After git checkout output
const OPEN_FILE_FIRST_PAUSE: f64 = 33.3; // Before opening first file
const OPEN_FILE_PAUSE: f64 = 50.0; // Before opening subsequent files
const OPEN_CMD_PAUSE: f64 = 16.7; // After open command
const FILE_SWITCH_PAUSE: f64 = 26.7; // After switching file
const GIT_ADD_PAUSE: f64 = 33.3; // Before git add
const GIT_ADD_CMD_PAUSE: f64 = 16.7; // After git add command
const GIT_COMMIT_PAUSE: f64 = 26.7; // After git commit command
const COMMIT_OUTPUT_PAUSE: f64 = 33.3; // After commit output
const GIT_PUSH_PAUSE: f64 = 16.7; // After git push command
const PUSH_OUTPUT_PAUSE: f64 = 10.0; // Between push output lines
const PUSH_FINAL_PAUSE: f64 = 66.7; // After final push output

/// Represents the current state of the editor buffer
#[derive(Debug, Clone)]
pub struct EditorBuffer {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub cached_highlights: Vec<crate::syntax::HighlightSpan>,
    /// Pre-calculated highlights for old and new content
    pub old_highlights: Vec<crate::syntax::HighlightSpan>,
    pub new_highlights: Vec<crate::syntax::HighlightSpan>,
    /// Store old and new content for byte offset calculation
    pub old_content_lines: Vec<String>,
    pub new_content_lines: Vec<String>,
    /// Pre-calculated byte offsets for each line (handles CRLF correctly)
    pub old_content_line_offsets: Vec<usize>,
    pub new_content_line_offsets: Vec<usize>,
}

impl EditorBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            cached_highlights: Vec::new(),
            old_highlights: Vec::new(),
            new_highlights: Vec::new(),
            old_content_lines: Vec::new(),
            new_content_lines: Vec::new(),
            old_content_line_offsets: Vec::new(),
            new_content_line_offsets: Vec::new(),
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
            old_highlights: Vec::new(),
            new_highlights: Vec::new(),
            old_content_lines: Vec::new(),
            new_content_lines: Vec::new(),
            old_content_line_offsets: Vec::new(),
            new_content_line_offsets: Vec::new(),
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
    InsertChar {
        line: usize,
        col: usize,
        ch: char,
    },
    InsertLine {
        line: usize,
        content: String,
    },
    DeleteLine {
        line: usize,
    },
    MoveCursor {
        line: usize,
        col: usize,
    },
    Pause {
        duration_ms: u64,
    },
    SwitchFile {
        file_index: usize,
        old_content: String,
        new_content: String,
        path: String,
    },
    OpenFileDialogStart,
    DialogTypeChar {
        ch: char,
    },
    TerminalPrompt,
    TerminalTypeChar {
        ch: char,
    },
    TerminalOutput {
        text: String,
    },
    ResetState,
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
    content_width: usize,
    pub current_file_index: usize,
    pub current_file_path: Option<String>,
    pub terminal_lines: Vec<String>,
    pub active_pane: ActivePane,
    pub highlighter: RefCell<Highlighter>,
    /// Track cumulative line offset from old_content (insertions - deletions)
    pub line_offset: isize,
    /// Target frames per second for rendering
    #[allow(dead_code)]
    target_fps: u64,
    /// Frame interval in milliseconds (calculated from target_fps)
    frame_interval_ms: u64,
    /// Last frame render time
    last_frame: Instant,
    /// Dialog title (e.g., "Open File...")
    pub dialog_title: Option<String>,
    /// Text being typed in the dialog
    pub dialog_typing_text: String,
    /// Current metadata being displayed
    current_metadata: Option<CommitMetadata>,
    /// Pending metadata to be applied on ResetState
    pending_metadata: Option<CommitMetadata>,
}

impl AnimationEngine {
    pub fn new(speed_ms: u64) -> Self {
        let target_fps: u64 = 120;
        let frame_interval_ms = 1000 / target_fps;
        let now = Instant::now();
        Self {
            buffer: EditorBuffer::new(),
            state: AnimationState::Idle,
            steps: Vec::new(),
            current_step: 0,
            last_update: now,
            speed_ms,
            next_step_delay: speed_ms,
            pause_until: None,
            cursor_visible: true,
            cursor_blink_timer: now,
            viewport_height: 20, // Default, will be updated from UI
            content_width: 80,   // Default, will be updated from UI
            current_file_index: 0,
            current_file_path: None,
            terminal_lines: Vec::new(),
            active_pane: ActivePane::Terminal, // Start with terminal (git checkout)
            highlighter: RefCell::new(Highlighter::new()),
            line_offset: 0,
            target_fps,
            frame_interval_ms,
            last_frame: now,
            dialog_title: None,
            dialog_typing_text: String::new(),
            current_metadata: None,
            pending_metadata: None,
        }
    }

    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
    }

    pub fn set_content_width(&mut self, width: usize) {
        self.content_width = width;
    }

    /// Get the current metadata being displayed
    pub fn current_metadata(&self) -> Option<&CommitMetadata> {
        self.current_metadata.as_ref()
    }

    fn calculate_line_offsets(content: &str) -> Vec<usize> {
        std::iter::once(0)
            .chain(content.bytes().enumerate().filter_map(|(i, b)| {
                if b == b'\n' {
                    Some(i + 1)
                } else {
                    None
                }
            }))
            .collect()
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
        // Store pending metadata to be applied on ResetState
        self.pending_metadata = Some(metadata.clone());

        self.steps.clear();
        self.current_step = 0;
        self.state = AnimationState::Playing;
        self.last_update = Instant::now();
        self.pause_until = None;

        // Time travel to commit date
        let parent_hash = format!("{}^", &metadata.hash[..7]);
        let datetime_str = metadata.date.format("%Y-%m-%d %H:%M:%S").to_string();
        self.add_terminal_command(&format!("time-travel {}", datetime_str));
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * CHECKOUT_PAUSE) as u64,
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "⚡ Initializing temporal displacement field...".to_string(),
        });
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * CHECKOUT_OUTPUT_PAUSE * 0.5) as u64,
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "✨ Warping through spacetime...".to_string(),
        });
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * CHECKOUT_OUTPUT_PAUSE * 0.5) as u64,
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!("🕰️  Arrived at {}", datetime_str),
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!(
                "📍 Location: commit {} by {}",
                &metadata.hash[..7],
                metadata.author
            ),
        });
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * CHECKOUT_OUTPUT_PAUSE) as u64,
        });

        // Apply new metadata after time-travel animation
        self.steps.push(AnimationStep::ResetState);

        // Sort file changes to match FileTree display order (directory -> filename)
        let sorted_indices = metadata.sorted_file_indices();

        // Process all file changes in sorted order
        for &index in &sorted_indices {
            let change = &metadata.changes[index];
            match (change.is_excluded, &change.status) {
                // Skip excluded files (lock files and generated files)
                (true, _) => {
                    // Switch to the excluded file to show in file tree
                    let old_content = change.old_content.clone().unwrap_or_default();
                    let new_content = change.new_content.clone().unwrap_or_default();
                    self.steps.push(AnimationStep::SwitchFile {
                        file_index: index,
                        old_content,
                        new_content,
                        path: change.path.clone(),
                    });

                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * OPEN_FILE_PAUSE) as u64,
                    });
                    let reason = change
                        .exclusion_reason
                        .as_deref()
                        .unwrap_or("excluded file");
                    self.steps.push(AnimationStep::TerminalOutput {
                        text: format!("📦 {} (skipped - {})", change.path, reason),
                    });
                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * OPEN_CMD_PAUSE) as u64,
                    });
                }
                // For deleted files, skip editor animation and only run rm + git add
                (false, FileStatus::Deleted) => {
                    // Switch to the deleted file to show in file tree
                    let old_content = change.old_content.clone().unwrap_or_default();
                    self.steps.push(AnimationStep::SwitchFile {
                        file_index: index,
                        old_content,
                        new_content: String::new(),
                        path: change.path.clone(),
                    });

                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * GIT_ADD_PAUSE) as u64,
                    });
                    self.add_terminal_command(&format!("rm {}", change.path));
                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * GIT_ADD_CMD_PAUSE) as u64,
                    });
                    self.add_terminal_command(&format!("git add {}", change.path));
                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * GIT_ADD_CMD_PAUSE) as u64,
                    });
                }
                // For renamed/moved files, skip editor animation and only run mv + git add
                (false, FileStatus::Renamed) => {
                    // Switch to the renamed file to show in file tree
                    let old_content = change.old_content.clone().unwrap_or_default();
                    let new_content = change.new_content.clone().unwrap_or_default();
                    self.steps.push(AnimationStep::SwitchFile {
                        file_index: index,
                        old_content,
                        new_content,
                        path: change.path.clone(),
                    });

                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * GIT_ADD_PAUSE) as u64,
                    });
                    if let Some(old_path) = &change.old_path {
                        self.add_terminal_command(&format!("mv {} {}", old_path, change.path));
                        self.steps.push(AnimationStep::Pause {
                            duration_ms: (self.speed_ms as f64 * GIT_ADD_CMD_PAUSE) as u64,
                        });
                    }
                    self.add_terminal_command(&format!("git add {}", change.path));
                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * GIT_ADD_CMD_PAUSE) as u64,
                    });
                }
                // Normal files (Added, Modified, etc.) - full editor animation
                (false, _) => {
                    // Open file in editor
                    if index == 0 {
                        self.steps.push(AnimationStep::Pause {
                            duration_ms: (self.speed_ms as f64 * OPEN_FILE_FIRST_PAUSE) as u64,
                        });
                    } else {
                        self.steps.push(AnimationStep::Pause {
                            duration_ms: (self.speed_ms as f64 * OPEN_FILE_PAUSE) as u64,
                        });
                    }
                    // Show "Open File..." dialog and type the file path
                    self.steps.push(AnimationStep::OpenFileDialogStart);
                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * 5.0) as u64,
                    });

                    // Type each character of the file path
                    for ch in change.path.chars() {
                        self.steps.push(AnimationStep::DialogTypeChar { ch });
                    }

                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * OPEN_CMD_PAUSE) as u64,
                    });

                    // Add file switch step with both old and new content
                    let old_content = change.old_content.clone().unwrap_or_default();
                    let new_content = change.new_content.clone().unwrap_or_default();
                    self.steps.push(AnimationStep::SwitchFile {
                        file_index: index,
                        old_content,
                        new_content,
                        path: change.path.clone(),
                    });

                    // Add pause before starting file animation
                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * FILE_SWITCH_PAUSE) as u64,
                    });

                    // Generate animation steps for this file
                    self.generate_steps_for_file(change);

                    // Git add this file after editing
                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * GIT_ADD_PAUSE) as u64,
                    });
                    self.add_terminal_command(&format!("git add {}", change.path));
                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * GIT_ADD_CMD_PAUSE) as u64,
                    });
                }
            }
        }

        // Git commit
        let commit_message = metadata.message.lines().next().unwrap_or("Update");
        self.add_terminal_command(&format!("git commit -m \"{}\"", commit_message));
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * GIT_COMMIT_PAUSE) as u64,
        });
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
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * COMMIT_OUTPUT_PAUSE) as u64,
        });

        // Git push
        self.add_terminal_command("git push origin main");
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * GIT_PUSH_PAUSE) as u64,
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "🚀 Launching code into the cloud...".to_string(),
        });
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * PUSH_OUTPUT_PAUSE) as u64,
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "📦 Compressing digital dreams: 100% (5/5)".to_string(),
        });
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * PUSH_OUTPUT_PAUSE) as u64,
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "✍️  Signing with invisible ink: done.".to_string(),
        });
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * GIT_PUSH_PAUSE) as u64,
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: "📡 Beaming to origin/main via satellite...".to_string(),
        });
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * PUSH_OUTPUT_PAUSE) as u64,
        });
        self.steps.push(AnimationStep::TerminalOutput {
            text: format!(
                "   {}..{} ✨ SUCCESS",
                &parent_hash[..7],
                &metadata.hash[..7]
            ),
        });
        self.steps.push(AnimationStep::Pause {
            duration_ms: (self.speed_ms as f64 * PUSH_FINAL_PAUSE) as u64,
        });

        // Start with empty editor (no file opened yet)
        self.buffer = EditorBuffer::new();
    }

    /// Generate animation steps for a file change
    fn generate_steps_for_file(&mut self, change: &FileChange) {
        let mut current_cursor_line = 0;
        let mut line_offset = 0i64; // Track how buffer lines differ from old file

        // Parse old_content into lines for indentation calculation during cursor movement
        let old_lines: Vec<&str> = change
            .old_content
            .as_ref()
            .map(|c| c.lines().collect())
            .unwrap_or_default();

        // Process each hunk
        for hunk in &change.hunks {
            // Calculate target line in current buffer
            // hunk.old_start is 1-indexed (Git line numbers start at 1)
            // We need to convert to 0-indexed and adjust by how many lines we've added/removed
            let target_line = ((hunk.old_start as i64) - 1 + line_offset).max(0) as usize;

            // Calculate distance for speed adjustment
            let distance = target_line.abs_diff(current_cursor_line);

            current_cursor_line = self.generate_cursor_movement(
                current_cursor_line,
                target_line,
                distance,
                &old_lines,
            );

            let (final_cursor_line, _final_buffer_line) =
                self.generate_steps_for_hunk(hunk, current_cursor_line, target_line);

            current_cursor_line = final_cursor_line;

            // Update offset based on changes in this hunk
            // Count additions and deletions to update the offset
            let additions = hunk
                .lines
                .iter()
                .filter(|l| matches!(l.change_type, LineChangeType::Addition))
                .count() as i64;
            let deletions = hunk
                .lines
                .iter()
                .filter(|l| matches!(l.change_type, LineChangeType::Deletion))
                .count() as i64;

            line_offset += additions - deletions;

            // Add pause between hunks
            self.steps.push(AnimationStep::Pause {
                duration_ms: (self.speed_ms as f64 * HUNK_PAUSE) as u64,
            });
        }
    }

    /// Generate cursor movement steps from current line to target line
    fn generate_cursor_movement(
        &mut self,
        from_line: usize,
        to_line: usize,
        distance: usize,
        lines: &[&str],
    ) -> usize {
        if from_line == to_line {
            return to_line;
        }

        // Determine base speed multiplier based on total distance
        let base_speed_multiplier = if distance <= 50 {
            CURSOR_MOVE_SHORT_MULTIPLIER
        } else if distance <= 200 {
            CURSOR_MOVE_MEDIUM_MULTIPLIER
        } else {
            CURSOR_MOVE_LONG_MULTIPLIER
        };

        // Limit total animation steps for performance
        // For very long distances, use fewer steps with larger jumps
        // Threshold aligned with SHORT distance category (50) to ensure monotonicity
        let num_steps = if distance <= MIN_LOG_STEPS {
            distance // Show every line for short distances
        } else {
            // Scale steps logarithmically for longer distances
            // This ensures smooth animation while limiting total steps
            let log_steps = (distance as f64).ln() * LOG_SCALE_FACTOR;
            (log_steps as usize).clamp(MIN_LOG_STEPS, MAX_SCROLL_STEPS)
        };

        let mut positions = Vec::with_capacity(num_steps + 1);

        for i in 0..=num_steps {
            let t = i as f64 / num_steps as f64;
            let eased = self.ease_in_out_cubic(t);
            let line_progress = (eased * distance as f64).round() as usize;

            let actual_line = if from_line < to_line {
                from_line + line_progress
            } else {
                from_line - line_progress
            };

            // Avoid duplicate positions
            if positions.is_empty() || positions.last() != Some(&actual_line) {
                positions.push(actual_line);
            }
        }

        // Generate movement steps
        let base_pause =
            (self.speed_ms as f64 * CURSOR_MOVE_PAUSE * base_speed_multiplier).max(1.0) as u64;

        for line in positions {
            if line != from_line {
                // Calculate indentation (first non-whitespace character position)
                let col = lines
                    .get(line)
                    .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
                    .unwrap_or(0);
                self.steps.push(AnimationStep::MoveCursor { line, col });
                self.steps.push(AnimationStep::Pause {
                    duration_ms: base_pause,
                });
            }
        }

        to_line
    }

    /// Ease-in-out cubic easing function
    /// Starts slow, accelerates in middle, ends slow
    fn ease_in_out_cubic(&self, t: f64) -> f64 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
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
                    self.steps
                        .push(AnimationStep::DeleteLine { line: buffer_line });
                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * DELETE_LINE_PAUSE) as u64,
                    });
                    cursor_line = buffer_line;
                    // After deletion, buffer_line stays the same
                    // (the next line moves up to this position)
                }
                LineChangeType::Addition => {
                    let content = &line_change.content;
                    let indentation_len = content.chars().take_while(|c| c.is_whitespace()).count();

                    // Insert line with indentation already included
                    let indentation: String = content.chars().take(indentation_len).collect();
                    self.steps.push(AnimationStep::InsertLine {
                        line: buffer_line,
                        content: indentation,
                    });

                    // Type each character after the indentation
                    for (i, ch) in content.chars().skip(indentation_len).enumerate() {
                        self.steps.push(AnimationStep::InsertChar {
                            line: buffer_line,
                            col: indentation_len + i,
                            ch,
                        });
                    }

                    cursor_line = buffer_line;
                    buffer_line += 1; // Move to next line after insertion

                    self.steps.push(AnimationStep::Pause {
                        duration_ms: (self.speed_ms as f64 * INSERT_LINE_PAUSE) as u64,
                    });
                }
                LineChangeType::Context => {
                    // Move cursor to next line if needed
                    if buffer_line != cursor_line {
                        // Calculate indentation (first non-whitespace character position)
                        let col = line_change
                            .content
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .count();
                        self.steps.push(AnimationStep::MoveCursor {
                            line: buffer_line,
                            col,
                        });
                        self.steps.push(AnimationStep::Pause {
                            duration_ms: (self.speed_ms as f64 * CURSOR_MOVE_PAUSE) as u64,
                        });
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
        self.update_cursor_blink();

        if self.is_paused() {
            return true;
        }

        if self.state != AnimationState::Playing {
            return false;
        }

        let now = Instant::now();
        if !self.should_render_frame(now) {
            return false;
        }

        let executed = self.execute_batch_steps(now);

        if self.current_step >= self.steps.len() {
            self.state = AnimationState::Finished;
        }

        executed
    }

    fn update_cursor_blink(&mut self) {
        if self.cursor_blink_timer.elapsed() >= Duration::from_millis(500) {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink_timer = Instant::now();
        }
    }

    fn is_paused(&mut self) -> bool {
        if let Some(pause_until) = self.pause_until {
            if Instant::now() < pause_until {
                return true;
            }
            self.pause_until = None;
        }
        false
    }

    fn should_render_frame(&self, now: Instant) -> bool {
        now.duration_since(self.last_frame) >= Duration::from_millis(self.frame_interval_ms)
    }

    fn execute_batch_steps(&mut self, frame_start: Instant) -> bool {
        let mut accumulated_delay = 0u64;
        let mut executed_any = false;

        while self.current_step < self.steps.len() {
            if !self.can_execute_step(executed_any, accumulated_delay) {
                break;
            }

            let step_delay = self.next_step_delay;
            let step = self.steps[self.current_step].clone();

            self.execute_step(step);
            self.current_step += 1;
            executed_any = true;
            accumulated_delay += step_delay;
        }

        if executed_any {
            self.last_update = Instant::now();
            self.last_frame = frame_start;
        }

        executed_any
    }

    fn can_execute_step(&self, executed_any: bool, accumulated_delay: u64) -> bool {
        // First step: check if enough time has elapsed since last step
        if !executed_any {
            return self.last_update.elapsed() >= Duration::from_millis(self.next_step_delay);
        }

        // Subsequent steps: check if they fit within frame budget
        accumulated_delay + self.next_step_delay <= self.frame_interval_ms
    }

    fn execute_step(&mut self, step: AnimationStep) {
        // Calculate delay for next step with randomization for typing steps
        let mut rng = rand::rng();
        self.next_step_delay = match &step {
            AnimationStep::InsertChar { .. } | AnimationStep::TerminalTypeChar { .. } => {
                // Add 70-130% variation to typing speed
                let variation = rng.random_range(0.7..=1.3);
                ((self.speed_ms as f64) * variation) as u64
            }
            AnimationStep::DialogTypeChar { .. } => {
                // Dialog typing is slower (2x speed with variation)
                let variation = rng.random_range(0.7..=1.3);
                ((self.speed_ms as f64) * 2.0 * variation) as u64
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
            AnimationStep::InsertLine { line, content } => {
                self.active_pane = ActivePane::Editor;
                let content_len = content.chars().count();
                self.buffer.insert_line(line, content);
                self.buffer.cursor_line = line;
                self.buffer.cursor_col = content_len;

                // Track line offset for old_highlights mapping
                self.line_offset += 1;
            }
            AnimationStep::DeleteLine { line } => {
                self.active_pane = ActivePane::Editor;
                self.buffer.delete_line(line);
                self.buffer.cursor_line = line;
                // Set cursor to first non-whitespace position of the line that moved up
                self.buffer.cursor_col = self
                    .buffer
                    .lines
                    .get(line)
                    .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
                    .unwrap_or(0);

                // Track line offset for old_highlights mapping
                self.line_offset -= 1;
            }
            AnimationStep::MoveCursor { line, col } => {
                self.active_pane = ActivePane::Editor;
                self.buffer.cursor_line = line;
                self.buffer.cursor_col = col;
            }
            AnimationStep::Pause { duration_ms } => {
                self.pause_until = Some(Instant::now() + Duration::from_millis(duration_ms));
            }
            AnimationStep::OpenFileDialogStart => {
                self.dialog_typing_text = String::new();
                self.dialog_title = Some("Open File...".to_string());
            }
            AnimationStep::DialogTypeChar { ch } => {
                self.dialog_typing_text.push(ch);
            }
            AnimationStep::SwitchFile {
                file_index,
                old_content,
                new_content,
                path,
            } => {
                self.active_pane = ActivePane::Editor;
                // Clear dialog when file is actually switched
                self.dialog_title = None;
                self.dialog_typing_text = String::new();
                // Switch to new file
                self.current_file_index = file_index;
                self.current_file_path = Some(path.clone());
                self.buffer = EditorBuffer::from_content(&old_content);

                // Update syntax highlighter for new file
                // This will clear language settings if not supported
                self.highlighter.borrow_mut().set_language_from_path(&path);

                // Pre-calculate highlights for both old and new content
                self.buffer.old_highlights = self.highlighter.borrow_mut().highlight(&old_content);
                self.buffer.new_highlights = self.highlighter.borrow_mut().highlight(&new_content);

                // Store content lines for byte offset calculation
                self.buffer.old_content_lines = if old_content.is_empty() {
                    vec![String::new()]
                } else {
                    old_content.lines().map(|s| s.to_string()).collect()
                };
                self.buffer.new_content_lines = if new_content.is_empty() {
                    vec![String::new()]
                } else {
                    new_content.lines().map(|s| s.to_string()).collect()
                };

                // Pre-calculate line byte offsets (handles CRLF correctly)
                self.buffer.old_content_line_offsets = Self::calculate_line_offsets(&old_content);
                self.buffer.new_content_line_offsets = Self::calculate_line_offsets(&new_content);

                // Initialize cached_highlights with old_highlights
                self.buffer.cached_highlights = self.buffer.old_highlights.clone();

                // Reset line offset
                self.line_offset = 0;
            }
            AnimationStep::TerminalPrompt => {
                self.active_pane = ActivePane::Terminal;
                // Start a new command line with prompt
                self.terminal_lines.push("~ ".to_string());
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
            AnimationStep::ResetState => {
                // Apply pending metadata and reset UI state after time-travel animation
                if let Some(metadata) = self.pending_metadata.take() {
                    self.current_metadata = Some(metadata);
                }
                self.current_file_index = 0;
                // Keep terminal_lines to preserve time-travel command and output
                self.buffer = EditorBuffer::new();
                self.current_file_path = None;
                self.active_pane = ActivePane::Terminal;
            }
        }

        // Update scroll to keep cursor centered
        self.update_scroll();
    }

    fn calculate_line_display_height(&self, line: &str) -> usize {
        if self.content_width == 0 {
            return 1;
        }

        // Calculate text area width (excluding line numbers, padding, etc.)
        let line_num_width = format!("{}", self.buffer.lines.len()).len().max(3);
        let left_padding = 2;
        let line_num_and_space = line_num_width + 1;
        let separator = 2;
        let right_padding = 2;
        let fixed_width = left_padding + line_num_and_space + separator + right_padding;

        let text_width = self.content_width.saturating_sub(fixed_width);
        if text_width == 0 {
            return 1;
        }

        // Calculate how many lines this text will take when wrapped (using display width)
        let display_width = line.width();
        display_width.div_ceil(text_width).max(1)
    }

    fn update_scroll(&mut self) {
        if self.viewport_height == 0 {
            return;
        }

        let cursor_line = self.buffer.cursor_line;

        // Calculate display line positions for each logical line
        let mut display_line_positions = Vec::with_capacity(self.buffer.lines.len());
        let mut current_display_line = 0;

        for line in &self.buffer.lines {
            display_line_positions.push(current_display_line);
            current_display_line += self.calculate_line_display_height(line);
        }

        let total_display_lines = current_display_line;
        let cursor_display_line = display_line_positions
            .get(cursor_line)
            .copied()
            .unwrap_or(0);

        // Calculate target scroll position (in display lines)
        let half_viewport = self.viewport_height / 2;
        let target_display_offset = if cursor_display_line < half_viewport {
            0
        } else if cursor_display_line + half_viewport >= total_display_lines {
            total_display_lines.saturating_sub(self.viewport_height)
        } else {
            cursor_display_line.saturating_sub(half_viewport)
        };

        // Find the logical line that corresponds to the target display offset
        let mut logical_offset = 0;
        for (line_idx, &display_pos) in display_line_positions.iter().enumerate() {
            if display_pos >= target_display_offset {
                logical_offset = line_idx;
                break;
            }
        }

        self.buffer.scroll_offset = logical_offset;
    }

    pub fn is_finished(&self) -> bool {
        self.state == AnimationState::Finished
    }
}
