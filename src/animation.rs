use std::cell::RefCell;
use std::time::{Duration, Instant};

use globset::{Glob, GlobMatcher};
use rand::RngExt;
use std::collections::VecDeque;
use unicode_width::UnicodeWidthStr;

use crate::git::{CommitMetadata, DiffHunk, FileChange, FileStatus, LineChangeType};
use crate::syntax::Highlighter;

/// A rule that specifies typing speed for files matching a glob pattern
#[derive(Debug, Clone)]
pub struct SpeedRule {
    pub matcher: GlobMatcher,
    pub speed_ms: u64,
}

impl SpeedRule {
    /// Parse a speed rule from string format "PATTERN:SPEED_MS"
    /// Example: "*.java:50" or "src/**/*.rs:30"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.rsplitn(2, ':').collect();
        if parts.len() != 2 {
            return None;
        }
        let speed_ms = parts[0].parse::<u64>().ok()?;
        let pattern_str = parts[1];
        let glob = Glob::new(pattern_str).ok()?;
        let matcher = glob.compile_matcher();
        Some(Self { matcher, speed_ms })
    }

    /// Check if a file path matches this rule
    pub fn matches(&self, path: &str) -> bool {
        self.matcher.is_match(path)
    }
}

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

const MAX_LINE_CHECKPOINTS: usize = 200;
const MAX_CHANGE_CHECKPOINTS: usize = 64;

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
    /// Creates a new empty editor buffer with default values.
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

    /// Creates an editor buffer initialized with the given content.
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

    /// Inserts a character at the specified line and column position.
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

    /// Inserts a new line with the given content at the specified position.
    pub fn insert_line(&mut self, line: usize, content: String) {
        if line > self.lines.len() {
            self.lines.resize(line, String::new());
        }
        self.lines.insert(line, content);
    }

    /// Deletes the line at the specified position.
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
        multiplier: f64,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepMode {
    Line,
    Change,
}

#[derive(Clone)]
struct ManualCheckpoint {
    step_index: usize,
    buffer: EditorBuffer,
    current_file_index: usize,
    current_file_path: Option<String>,
    terminal_lines: Vec<String>,
    active_pane: ActivePane,
    line_offset: isize,
    dialog_title: Option<String>,
    dialog_typing_text: String,
    speed_ms: u64,
}

impl ManualCheckpoint {
    fn new(engine: &AnimationEngine) -> Self {
        let resume_step = engine
            .current_step
            .saturating_add(1)
            .min(engine.steps.len());
        Self {
            step_index: resume_step,
            buffer: engine.buffer.clone(),
            current_file_index: engine.current_file_index,
            current_file_path: engine.current_file_path.clone(),
            terminal_lines: engine.terminal_lines.clone(),
            active_pane: engine.active_pane.clone(),
            line_offset: engine.line_offset,
            dialog_title: engine.dialog_title.clone(),
            dialog_typing_text: engine.dialog_typing_text.clone(),
            speed_ms: engine.speed_ms,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum CheckpointKind {
    Line,
    Change,
}

/// Main animation engine
pub struct AnimationEngine {
    pub buffer: EditorBuffer,
    pub state: AnimationState,
    steps: Vec<AnimationStep>,
    current_step: usize,
    last_update: Instant,
    speed_ms: u64,
    base_speed_ms: u64,
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
    /// Speed rules for different file patterns
    speed_rules: Vec<SpeedRule>,
    speed_multiplier: f64,
    paused: bool,
    line_checkpoints: VecDeque<ManualCheckpoint>,
    change_checkpoints: VecDeque<ManualCheckpoint>,
}

impl AnimationEngine {
    /// Creates a new animation engine with the specified typing speed.
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
            base_speed_ms: speed_ms,
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
            speed_rules: Vec::new(),
            speed_multiplier: 1.0,
            paused: false,
            line_checkpoints: VecDeque::new(),
            change_checkpoints: VecDeque::new(),
        }
    }

    /// Pause the animation playback.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume animation playback from the current position.
    pub fn resume(&mut self) {
        if self.paused {
            self.paused = false;
            let now = Instant::now();
            self.last_update = now;
            self.last_frame = now;
        }
    }

    /// Execute animation steps manually until the next boundary for the given mode.
    pub fn manual_step(&mut self, mode: StepMode) -> bool {
        if self.state != AnimationState::Playing {
            return false;
        }

        if self.current_step >= self.steps.len() {
            self.state = AnimationState::Finished;
            return false;
        }

        self.pause_until = None;
        let mut executed = false;

        while self.current_step < self.steps.len() {
            let step = self.steps[self.current_step].clone();
            self.execute_step(step.clone());
            self.current_step += 1;
            executed = true;

            if self.current_step >= self.steps.len() {
                self.state = AnimationState::Finished;
            }

            if Self::is_boundary_step(&step, mode) {
                break;
            }
        }

        if executed {
            let now = Instant::now();
            self.last_update = now;
            self.last_frame = now;
        }

        executed
    }

    pub fn restore_line_checkpoint(&mut self) -> bool {
        if self.line_checkpoints.len() < 2 {
            return false;
        }
        self.line_checkpoints.pop_back();
        let snapshot = self
            .line_checkpoints
            .back()
            .cloned()
            .expect("line checkpoint should exist after len guard");
        self.apply_checkpoint(snapshot);
        true
    }

    pub fn restore_change_checkpoint(&mut self) -> bool {
        if self.change_checkpoints.len() < 2 {
            return false;
        }
        self.change_checkpoints.pop_back();
        let snapshot = self
            .change_checkpoints
            .back()
            .cloned()
            .expect("change checkpoint should exist after len guard");
        self.apply_checkpoint(snapshot);
        true
    }

    fn apply_checkpoint(&mut self, snapshot: ManualCheckpoint) {
        self.current_step = snapshot.step_index;
        self.buffer = snapshot.buffer;
        self.current_file_index = snapshot.current_file_index;
        self.current_file_path = snapshot.current_file_path;
        self.terminal_lines = snapshot.terminal_lines;
        self.active_pane = snapshot.active_pane;
        self.line_offset = snapshot.line_offset;
        self.dialog_title = snapshot.dialog_title;
        self.dialog_typing_text = snapshot.dialog_typing_text;
        self.speed_ms = snapshot.speed_ms;
        self.pause_until = None;
        self.paused = true;
        self.state = AnimationState::Playing;
    }

    fn is_boundary_step(step: &AnimationStep, mode: StepMode) -> bool {
        match mode {
            StepMode::Line => matches!(
                step,
                AnimationStep::Pause { .. }
                    | AnimationStep::SwitchFile { .. }
                    | AnimationStep::TerminalPrompt
                    | AnimationStep::TerminalOutput { .. }
                    | AnimationStep::ResetState
            ),
            StepMode::Change => match step {
                AnimationStep::SwitchFile { .. }
                | AnimationStep::TerminalPrompt
                | AnimationStep::TerminalOutput { .. }
                | AnimationStep::ResetState => true,
                AnimationStep::Pause { multiplier } => Self::is_change_pause(*multiplier),
                _ => false,
            },
        }
    }

    fn handle_step_checkpoint(&mut self, step: &AnimationStep) {
        match step {
            AnimationStep::ResetState => {
                self.clear_checkpoints();
                self.record_checkpoint(CheckpointKind::Change);
                self.record_checkpoint(CheckpointKind::Line);
            }
            AnimationStep::SwitchFile { .. } => {
                self.line_checkpoints.clear();
                self.record_checkpoint(CheckpointKind::Change);
                self.record_checkpoint(CheckpointKind::Line);
            }
            AnimationStep::Pause { multiplier } if self.active_pane == ActivePane::Editor => {
                self.record_checkpoint(CheckpointKind::Line);
                if Self::is_change_pause(*multiplier) {
                    self.record_checkpoint(CheckpointKind::Change);
                }
            }
            _ => {}
        }
    }

    fn is_change_pause(multiplier: f64) -> bool {
        (multiplier - HUNK_PAUSE).abs() < f64::EPSILON
    }

    fn record_checkpoint(&mut self, kind: CheckpointKind) {
        if self.current_step == 0 {
            return;
        }

        let snapshot = ManualCheckpoint::new(self);
        match kind {
            CheckpointKind::Line => {
                if self
                    .line_checkpoints
                    .back()
                    .map(|c| c.step_index == snapshot.step_index)
                    .unwrap_or(false)
                {
                    return;
                }
                self.line_checkpoints.push_back(snapshot);
                if self.line_checkpoints.len() > MAX_LINE_CHECKPOINTS {
                    self.line_checkpoints.pop_front();
                }
            }
            CheckpointKind::Change => {
                if self
                    .change_checkpoints
                    .back()
                    .map(|c| c.step_index == snapshot.step_index)
                    .unwrap_or(false)
                {
                    return;
                }
                self.change_checkpoints.push_back(snapshot);
                if self.change_checkpoints.len() > MAX_CHANGE_CHECKPOINTS {
                    self.change_checkpoints.pop_front();
                }
            }
        }
    }

    fn clear_checkpoints(&mut self) {
        self.line_checkpoints.clear();
        self.change_checkpoints.clear();
    }

    /// Set speed rules for file-specific typing speeds
    pub fn set_speed_rules(&mut self, rules: Vec<SpeedRule>) {
        self.speed_rules = rules;
    }

    /// Speed up typing by `multiplier` (applied from the next file onward)
    pub fn set_speed_multiplier(&mut self, multiplier: f64) {
        self.speed_multiplier = multiplier.max(1.0);
    }

    /// Get the speed for a given file path based on speed rules
    /// Returns the first matching rule's speed, or the base speed if no match
    fn get_speed_for_file(&self, path: &str) -> u64 {
        let speed = self
            .speed_rules
            .iter()
            .find(|rule| rule.matches(path))
            .map_or(self.base_speed_ms, |rule| rule.speed_ms);
        (speed as f64 / self.speed_multiplier) as u64
    }

    /// Sets the viewport height for scroll calculations.
    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
    }

    /// Sets the content width for line wrapping calculations.
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

        // Check if this is a working tree diff (not a real commit)
        let is_working_tree = metadata.hash == "working-tree";

        if is_working_tree {
            // Simplified intro for working tree diffs
            self.add_terminal_command("git diff --stat");
            self.steps.push(AnimationStep::Pause {
                multiplier: CHECKOUT_PAUSE,
            });
            self.steps.push(AnimationStep::TerminalOutput {
                text: format!("📝 {}", metadata.message),
            });
            self.steps.push(AnimationStep::TerminalOutput {
                text: format!(
                    "📁 {} file{} changed",
                    metadata.changes.len(),
                    if metadata.changes.len() == 1 { "" } else { "s" }
                ),
            });
            self.steps.push(AnimationStep::Pause {
                multiplier: CHECKOUT_OUTPUT_PAUSE,
            });
        } else {
            // Time travel to commit date
            let datetime_str = metadata.date.format("%Y-%m-%d %H:%M:%S").to_string();
            self.add_terminal_command(&format!("time-travel {}", datetime_str));
            self.steps.push(AnimationStep::Pause {
                multiplier: CHECKOUT_PAUSE,
            });
            self.steps.push(AnimationStep::TerminalOutput {
                text: "⚡ Initializing temporal displacement field...".to_string(),
            });
            self.steps.push(AnimationStep::Pause {
                multiplier: CHECKOUT_OUTPUT_PAUSE * 0.5,
            });
            self.steps.push(AnimationStep::TerminalOutput {
                text: "✨ Warping through spacetime...".to_string(),
            });
            self.steps.push(AnimationStep::Pause {
                multiplier: CHECKOUT_OUTPUT_PAUSE * 0.5,
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
                multiplier: CHECKOUT_OUTPUT_PAUSE,
            });
        }

        // Apply new metadata after intro animation
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
                        multiplier: OPEN_FILE_PAUSE,
                    });
                    let reason = change
                        .exclusion_reason
                        .as_deref()
                        .unwrap_or("excluded file");
                    self.steps.push(AnimationStep::TerminalOutput {
                        text: format!("📦 {} (skipped - {})", change.path, reason),
                    });
                    self.steps.push(AnimationStep::Pause {
                        multiplier: OPEN_CMD_PAUSE,
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
                        multiplier: GIT_ADD_PAUSE,
                    });
                    self.add_terminal_command(&format!("rm {}", change.path));
                    self.steps.push(AnimationStep::Pause {
                        multiplier: GIT_ADD_CMD_PAUSE,
                    });
                    self.add_terminal_command(&format!("git add {}", change.path));
                    self.steps.push(AnimationStep::Pause {
                        multiplier: GIT_ADD_CMD_PAUSE,
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
                        multiplier: GIT_ADD_PAUSE,
                    });
                    if let Some(old_path) = &change.old_path {
                        self.add_terminal_command(&format!("mv {} {}", old_path, change.path));
                        self.steps.push(AnimationStep::Pause {
                            multiplier: GIT_ADD_CMD_PAUSE,
                        });
                    }
                    self.add_terminal_command(&format!("git add {}", change.path));
                    self.steps.push(AnimationStep::Pause {
                        multiplier: GIT_ADD_CMD_PAUSE,
                    });
                }
                // Normal files (Added, Modified, etc.) - full editor animation
                (false, _) => {
                    // Open file in editor
                    if index == 0 {
                        self.steps.push(AnimationStep::Pause {
                            multiplier: OPEN_FILE_FIRST_PAUSE,
                        });
                    } else {
                        self.steps.push(AnimationStep::Pause {
                            multiplier: OPEN_FILE_PAUSE,
                        });
                    }
                    // Show "Open File..." dialog and type the file path
                    self.steps.push(AnimationStep::OpenFileDialogStart);
                    self.steps.push(AnimationStep::Pause { multiplier: 5.0 });

                    // Type each character of the file path
                    for ch in change.path.chars() {
                        self.steps.push(AnimationStep::DialogTypeChar { ch });
                    }

                    self.steps.push(AnimationStep::Pause {
                        multiplier: OPEN_CMD_PAUSE,
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
                        multiplier: FILE_SWITCH_PAUSE,
                    });

                    // Generate animation steps for this file
                    self.generate_steps_for_file(change);

                    // Git add this file after editing
                    self.steps.push(AnimationStep::Pause {
                        multiplier: GIT_ADD_PAUSE,
                    });
                    self.add_terminal_command(&format!("git add {}", change.path));
                    self.steps.push(AnimationStep::Pause {
                        multiplier: GIT_ADD_CMD_PAUSE,
                    });
                }
            }
        }

        // Skip git commit/push animation for working tree diffs
        if is_working_tree {
            // Just add a final pause for working tree mode
            self.steps.push(AnimationStep::Pause {
                multiplier: PUSH_FINAL_PAUSE,
            });
        } else {
            // Git commit
            let parent_hash = format!("{}^", &metadata.hash[..7]);
            let commit_message = metadata.message.lines().next().unwrap_or("Update");
            self.add_terminal_command(&format!("git commit -m \"{}\"", commit_message));
            self.steps.push(AnimationStep::Pause {
                multiplier: GIT_COMMIT_PAUSE,
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
                multiplier: COMMIT_OUTPUT_PAUSE,
            });

            // Git push
            self.add_terminal_command("git push origin main");
            self.steps.push(AnimationStep::Pause {
                multiplier: GIT_PUSH_PAUSE,
            });
            self.steps.push(AnimationStep::TerminalOutput {
                text: "🚀 Launching code into the cloud...".to_string(),
            });
            self.steps.push(AnimationStep::Pause {
                multiplier: PUSH_OUTPUT_PAUSE,
            });
            self.steps.push(AnimationStep::TerminalOutput {
                text: "📦 Compressing digital dreams: 100% (5/5)".to_string(),
            });
            self.steps.push(AnimationStep::Pause {
                multiplier: PUSH_OUTPUT_PAUSE,
            });
            self.steps.push(AnimationStep::TerminalOutput {
                text: "✍️  Signing with invisible ink: done.".to_string(),
            });
            self.steps.push(AnimationStep::Pause {
                multiplier: GIT_PUSH_PAUSE,
            });
            self.steps.push(AnimationStep::TerminalOutput {
                text: "📡 Beaming to origin/main via satellite...".to_string(),
            });
            self.steps.push(AnimationStep::Pause {
                multiplier: PUSH_OUTPUT_PAUSE,
            });
            self.steps.push(AnimationStep::TerminalOutput {
                text: format!(
                    "   {}..{} ✨ SUCCESS",
                    &parent_hash[..7],
                    &metadata.hash[..7]
                ),
            });
            self.steps.push(AnimationStep::Pause {
                multiplier: PUSH_FINAL_PAUSE,
            });
        }

        // Start with empty editor (no file opened yet)
        self.buffer = EditorBuffer::new();
        self.clear_checkpoints();
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
                multiplier: HUNK_PAUSE,
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
        let pause_multiplier = (CURSOR_MOVE_PAUSE * base_speed_multiplier).max(0.01);

        for line in positions {
            if line != from_line {
                // Calculate indentation (first non-whitespace character position)
                let col = lines
                    .get(line)
                    .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
                    .unwrap_or(0);
                self.steps.push(AnimationStep::MoveCursor { line, col });
                self.steps.push(AnimationStep::Pause {
                    multiplier: pause_multiplier,
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
                        multiplier: DELETE_LINE_PAUSE,
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
                        multiplier: INSERT_LINE_PAUSE,
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
                            multiplier: CURSOR_MOVE_PAUSE,
                        });
                    }
                    cursor_line = buffer_line;
                    buffer_line += 1; // Move to next line
                }
            }
        }

        (cursor_line, buffer_line)
    }

    /// Updates animation state and returns true if display needs refresh.
    pub fn tick(&mut self) -> bool {
        self.update_cursor_blink();

        if self.paused {
            return true;
        }

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
        let step_clone = step.clone();
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
            AnimationStep::Pause { .. } => {
                // Pause timing is driven by `pause_until`; don't add extra delay
                0
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
            AnimationStep::Pause { multiplier } => {
                let duration_ms = (self.speed_ms as f64 * multiplier) as u64;
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

                // Update typing speed based on file-specific rules
                self.speed_ms = self.get_speed_for_file(&path);

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

        self.handle_step_checkpoint(&step_clone);

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

    /// Returns true if the animation has completed.
    pub fn is_finished(&self) -> bool {
        self.state == AnimationState::Finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn speed_rule(rule: &str) -> SpeedRule {
        SpeedRule::parse(rule).expect("valid speed rule")
    }

    fn switch_file(path: &str, old_content: &str, new_content: &str) -> AnimationStep {
        AnimationStep::SwitchFile {
            file_index: 1,
            old_content: old_content.to_string(),
            new_content: new_content.to_string(),
            path: path.to_string(),
        }
    }

    fn line_change(change_type: LineChangeType, content: &str) -> crate::git::LineChange {
        crate::git::LineChange {
            change_type,
            content: content.to_string(),
            old_line_no: None,
            new_line_no: None,
        }
    }

    fn hunk(old_start: usize, lines: Vec<crate::git::LineChange>) -> DiffHunk {
        DiffHunk {
            old_start,
            old_lines: lines.len(),
            new_start: old_start,
            new_lines: lines.len(),
            lines,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn file_change(
        path: &str,
        old_path: Option<&str>,
        status: FileStatus,
        is_excluded: bool,
        exclusion_reason: Option<&str>,
        old_content: Option<&str>,
        new_content: Option<&str>,
        hunks: Vec<DiffHunk>,
    ) -> FileChange {
        FileChange {
            path: path.to_string(),
            old_path: old_path.map(str::to_string),
            status,
            is_binary: false,
            is_excluded,
            exclusion_reason: exclusion_reason.map(str::to_string),
            old_content: old_content.map(str::to_string),
            new_content: new_content.map(str::to_string),
            hunks,
            diff: String::new(),
        }
    }

    fn metadata(hash: &str, message: &str, changes: Vec<FileChange>) -> CommitMetadata {
        CommitMetadata {
            hash: hash.to_string(),
            author: "Author".to_string(),
            date: DateTime::parse_from_rfc3339("2024-01-02T03:04:05Z")
                .unwrap()
                .with_timezone(&Utc),
            message: message.to_string(),
            changes,
        }
    }

    fn terminal_commands(steps: &[AnimationStep]) -> Vec<String> {
        let (mut commands, current) = steps.iter().fold(
            (Vec::new(), None::<String>),
            |(mut commands, current), step| match step {
                AnimationStep::TerminalPrompt => {
                    if let Some(command) = current {
                        commands.push(command);
                    }
                    (commands, Some(String::new()))
                }
                AnimationStep::TerminalTypeChar { ch } => {
                    let mut command = current.unwrap_or_default();
                    command.push(*ch);
                    (commands, Some(command))
                }
                _ => (commands, current),
            },
        );

        if let Some(command) = current {
            commands.push(command);
        }

        commands
    }

    fn terminal_outputs(steps: &[AnimationStep]) -> Vec<String> {
        steps
            .iter()
            .filter_map(|step| match step {
                AnimationStep::TerminalOutput { text } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn speed_rule_uses_last_colon_as_separator() {
        let rule = speed_rule("docs/v1:guide/*.md:45");

        assert_eq!(rule.speed_ms, 45);
        assert!(rule.matches("docs/v1:guide/readme.md"));
        assert!(!rule.matches("docs/v1/guide/readme.md"));
    }

    #[test]
    fn speed_rule_rejects_invalid_input() {
        assert!(SpeedRule::parse("*.rs").is_none());
        assert!(SpeedRule::parse("*.rs:not-a-number").is_none());
        assert!(SpeedRule::parse("[*.rs:30").is_none());
    }

    #[test]
    fn editor_buffer_inserts_using_character_index() {
        let mut buffer = EditorBuffer::from_content("a界c");

        buffer.insert_char(0, 2, 'B');

        assert_eq!(buffer.lines, vec!["a界Bc"]);
    }

    #[test]
    fn editor_buffer_insert_char_resizes_missing_lines() {
        let mut buffer = EditorBuffer::new();

        buffer.insert_char(2, 0, 'x');

        assert_eq!(
            buffer.lines,
            vec![String::new(), String::new(), "x".to_string()]
        );
    }

    #[test]
    fn editor_buffer_resizes_and_stays_non_empty() {
        let mut buffer = EditorBuffer::new();

        buffer.insert_line(2, "tail".to_string());
        buffer.delete_line(2);
        buffer.delete_line(1);
        buffer.delete_line(0);

        assert_eq!(buffer.lines, vec![String::new()]);
    }

    #[test]
    fn manual_step_returns_false_when_not_playing() {
        let mut engine = AnimationEngine::new(30);

        assert!(!engine.manual_step(StepMode::Line));
        assert_eq!(engine.state, AnimationState::Idle);
    }

    #[test]
    fn manual_step_marks_finished_when_no_steps_remain() {
        let mut engine = AnimationEngine::new(30);
        engine.state = AnimationState::Playing;

        assert!(!engine.manual_step(StepMode::Change));
        assert_eq!(engine.state, AnimationState::Finished);
    }

    #[test]
    fn switch_file_updates_editor_state_and_offsets() {
        let mut engine = AnimationEngine::new(30);
        engine.set_speed_rules(vec![speed_rule("src/**/*.rs:5")]);
        engine.dialog_title = Some("Open File...".to_string());
        engine.dialog_typing_text = "src/lib.rs".to_string();
        engine.active_pane = ActivePane::Terminal;

        engine.execute_step(switch_file("src/lib.rs", "old\r\nline", "new\nline"));

        assert_eq!(engine.active_pane, ActivePane::Editor);
        assert_eq!(engine.current_file_index, 1);
        assert_eq!(engine.current_file_path.as_deref(), Some("src/lib.rs"));
        assert_eq!(engine.speed_ms, 5);
        assert_eq!(engine.buffer.lines, vec!["old", "line"]);
        assert_eq!(engine.buffer.old_content_line_offsets, vec![0, 5]);
        assert_eq!(engine.buffer.new_content_line_offsets, vec![0, 4]);
        assert_eq!(engine.dialog_title, None);
        assert!(engine.dialog_typing_text.is_empty());
    }

    #[test]
    fn load_commit_generates_special_file_sequences_for_regular_commits() {
        let mut engine = AnimationEngine::new(30);
        let commit = metadata(
            "abcdef1234567890",
            "Refine animation\n\nbody",
            vec![
                file_change(
                    "Cargo.lock",
                    None,
                    FileStatus::Modified,
                    true,
                    Some("lock file"),
                    Some("old"),
                    Some("new"),
                    vec![],
                ),
                file_change(
                    "src/deleted.rs",
                    None,
                    FileStatus::Deleted,
                    false,
                    None,
                    Some("obsolete();\n"),
                    None,
                    vec![],
                ),
                file_change(
                    "src/renamed.rs",
                    Some("src/original.rs"),
                    FileStatus::Renamed,
                    false,
                    None,
                    Some("before\n"),
                    Some("after\n"),
                    vec![],
                ),
                file_change(
                    "src/lib.rs",
                    None,
                    FileStatus::Modified,
                    false,
                    None,
                    Some("fn main() {\n}\n"),
                    Some("fn main() {\n    println!(\"hi\");\n}\n"),
                    vec![hunk(
                        1,
                        vec![
                            line_change(LineChangeType::Context, "fn main() {"),
                            line_change(LineChangeType::Addition, "    println!(\"hi\");"),
                            line_change(LineChangeType::Context, "}"),
                        ],
                    )],
                ),
            ],
        );

        engine.load_commit(&commit);

        let commands = terminal_commands(&engine.steps);
        let outputs = terminal_outputs(&engine.steps);
        let switched_paths = engine
            .steps
            .iter()
            .filter_map(|step| match step {
                AnimationStep::SwitchFile { path, .. } => Some(path.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            commands,
            vec![
                "time-travel 2024-01-02 03:04:05".to_string(),
                "rm src/deleted.rs".to_string(),
                "git add src/deleted.rs".to_string(),
                "git add src/lib.rs".to_string(),
                "mv src/original.rs src/renamed.rs".to_string(),
                "git add src/renamed.rs".to_string(),
                "git commit -m \"Refine animation\"".to_string(),
                "git push origin main".to_string(),
            ]
        );
        assert_eq!(
            switched_paths,
            vec![
                "Cargo.lock".to_string(),
                "src/deleted.rs".to_string(),
                "src/lib.rs".to_string(),
                "src/renamed.rs".to_string(),
            ]
        );
        assert_eq!(
            engine
                .steps
                .iter()
                .filter(|step| matches!(step, AnimationStep::OpenFileDialogStart))
                .count(),
            1
        );
        assert!(engine.steps.iter().any(|step| {
            matches!(
                step,
                AnimationStep::InsertLine { line, content }
                    if *line == 1 && content == "    "
            )
        }));
        assert!(outputs
            .iter()
            .any(|text| text.contains("Cargo.lock (skipped - lock file)")));
        assert!(outputs
            .iter()
            .any(|text| text.contains("Location: commit abcdef1 by Author")));
        assert!(outputs.iter().any(|text| text.contains("Refine animation")));
        assert!(outputs.iter().any(|text| text.contains("✨ SUCCESS")));
        assert!(
            matches!(engine.pending_metadata.as_ref(), Some(pending) if pending.hash == commit.hash)
        );
        assert_eq!(engine.buffer.lines, vec![String::new()]);
    }

    #[test]
    fn load_commit_uses_working_tree_intro_without_commit_or_push_commands() {
        let mut engine = AnimationEngine::new(30);
        let diff = metadata(
            "working-tree",
            "Staged changes",
            vec![file_change(
                "src/draft.rs",
                None,
                FileStatus::Modified,
                false,
                None,
                Some("fn main() {}\n"),
                Some("fn main() {\n    println!(\"draft\");\n}\n"),
                vec![hunk(
                    1,
                    vec![
                        line_change(LineChangeType::Context, "fn main() {"),
                        line_change(LineChangeType::Addition, "    println!(\"draft\");"),
                        line_change(LineChangeType::Context, "}"),
                    ],
                )],
            )],
        );

        engine.load_commit(&diff);

        let commands = terminal_commands(&engine.steps);
        let outputs = terminal_outputs(&engine.steps);

        assert_eq!(
            commands.first().map(String::as_str),
            Some("git diff --stat")
        );
        assert!(commands
            .iter()
            .any(|command| command == "git add src/draft.rs"));
        assert!(!commands
            .iter()
            .any(|command| command.starts_with("git commit -m")));
        assert!(!commands
            .iter()
            .any(|command| command == "git push origin main"));
        assert!(outputs.iter().any(|text| text == "📝 Staged changes"));
        assert!(outputs.iter().any(|text| text == "📁 1 file changed"));
    }

    #[test]
    fn manual_step_records_and_restores_line_and_change_checkpoints() {
        let mut engine = AnimationEngine::new(30);
        engine.state = AnimationState::Playing;
        engine.steps = vec![
            AnimationStep::ResetState,
            switch_file("src/lib.rs", "seed", "seed!\nbranch"),
            AnimationStep::InsertChar {
                line: 0,
                col: 4,
                ch: '!',
            },
            AnimationStep::Pause {
                multiplier: INSERT_LINE_PAUSE,
            },
            AnimationStep::InsertLine {
                line: 1,
                content: "branch".to_string(),
            },
            AnimationStep::Pause {
                multiplier: HUNK_PAUSE,
            },
        ];

        assert!(engine.manual_step(StepMode::Line));
        assert!(engine.manual_step(StepMode::Change));
        assert_eq!(engine.buffer.lines, vec!["seed".to_string()]);

        assert!(engine.manual_step(StepMode::Line));
        assert_eq!(engine.buffer.lines, vec!["seed!".to_string()]);
        assert_eq!(engine.line_checkpoints.len(), 2);
        assert_eq!(engine.change_checkpoints.len(), 1);

        assert!(engine.manual_step(StepMode::Change));
        assert_eq!(
            engine.buffer.lines,
            vec!["seed!".to_string(), "branch".to_string()]
        );
        assert_eq!(engine.line_checkpoints.len(), 3);
        assert_eq!(engine.change_checkpoints.len(), 2);

        assert!(engine.restore_line_checkpoint());
        assert_eq!(engine.buffer.lines, vec!["seed!".to_string()]);
        assert_eq!(engine.current_step, 4);
        assert_eq!(engine.state, AnimationState::Playing);
        assert!(engine.paused);

        assert!(engine.restore_change_checkpoint());
        assert_eq!(engine.buffer.lines, vec!["seed".to_string()]);
        assert_eq!(engine.current_step, 2);
        assert!(matches!(
            engine.current_file_path.as_deref(),
            Some("src/lib.rs")
        ));
    }

    #[test]
    fn record_checkpoint_deduplicates_and_evicts_oldest_snapshots() {
        let checkpoint_steps = || {
            (0..=(MAX_LINE_CHECKPOINTS + 2))
                .map(|_| AnimationStep::TerminalPrompt)
                .collect::<Vec<_>>()
        };
        let mut dedupe_engine = AnimationEngine::new(30);
        dedupe_engine.steps = checkpoint_steps();
        dedupe_engine.current_step = 3;
        dedupe_engine.record_checkpoint(CheckpointKind::Line);
        dedupe_engine.record_checkpoint(CheckpointKind::Line);
        dedupe_engine.record_checkpoint(CheckpointKind::Change);
        dedupe_engine.record_checkpoint(CheckpointKind::Change);
        assert_eq!(dedupe_engine.line_checkpoints.len(), 1);
        assert_eq!(dedupe_engine.change_checkpoints.len(), 1);

        let mut line_engine = AnimationEngine::new(30);
        line_engine.steps = checkpoint_steps();
        (1..=(MAX_LINE_CHECKPOINTS + 1)).for_each(|step| {
            line_engine.current_step = step;
            line_engine.record_checkpoint(CheckpointKind::Line);
        });
        assert_eq!(line_engine.line_checkpoints.len(), MAX_LINE_CHECKPOINTS);
        assert_eq!(
            line_engine.line_checkpoints.front().map(|c| c.step_index),
            Some(3)
        );

        let mut change_engine = AnimationEngine::new(30);
        change_engine.steps = checkpoint_steps();
        (1..=(MAX_CHANGE_CHECKPOINTS + 1)).for_each(|step| {
            change_engine.current_step = step;
            change_engine.record_checkpoint(CheckpointKind::Change);
        });
        assert_eq!(
            change_engine.change_checkpoints.len(),
            MAX_CHANGE_CHECKPOINTS
        );
        assert_eq!(
            change_engine
                .change_checkpoints
                .front()
                .map(|c| c.step_index),
            Some(3)
        );
    }

    #[test]
    fn switch_file_uses_default_speed_and_empty_content_fallbacks() {
        let mut engine = AnimationEngine::new(42);
        engine.set_speed_rules(vec![speed_rule("*.rs:5")]);

        engine.execute_step(switch_file("notes.txt", "", ""));

        assert_eq!(engine.speed_ms, 42);
        assert_eq!(engine.buffer.lines, vec![String::new()]);
        assert_eq!(engine.buffer.old_content_lines, vec![String::new()]);
        assert_eq!(engine.buffer.new_content_lines, vec![String::new()]);
        assert_eq!(engine.buffer.old_content_line_offsets, vec![0]);
        assert_eq!(engine.buffer.new_content_line_offsets, vec![0]);
        assert!(engine.buffer.cached_highlights.is_empty());
    }

    #[test]
    fn generate_cursor_movement_covers_short_medium_and_long_distances() {
        let mut short_engine = AnimationEngine::new(20);

        assert_eq!(short_engine.generate_cursor_movement(2, 2, 0, &[]), 2);
        assert!(short_engine.steps.is_empty());

        let short_lines = ["root", " one", "  two"];
        assert_eq!(
            short_engine.generate_cursor_movement(0, 2, 2, &short_lines),
            2
        );
        assert!(matches!(
            short_engine.steps.first(),
            Some(AnimationStep::MoveCursor { line: 1, col: 1 })
        ));
        assert!(matches!(
            short_engine.steps.get(2),
            Some(AnimationStep::MoveCursor { line: 2, col: 2 })
        ));
        assert!(short_engine.steps.iter().all(|step| match step {
            AnimationStep::Pause { multiplier } => {
                (*multiplier - CURSOR_MOVE_PAUSE * CURSOR_MOVE_SHORT_MULTIPLIER).abs()
                    < f64::EPSILON
            }
            _ => true,
        }));

        let medium_lines = (0..=60)
            .map(|line| format!("{}line {line}", " ".repeat(line % 4)))
            .collect::<Vec<_>>();
        let medium_refs = medium_lines.iter().map(String::as_str).collect::<Vec<_>>();
        let mut medium_engine = AnimationEngine::new(20);
        assert_eq!(
            medium_engine.generate_cursor_movement(60, 0, 60, &medium_refs),
            0
        );
        let medium_moves = medium_engine
            .steps
            .iter()
            .filter_map(|step| match step {
                AnimationStep::MoveCursor { line, .. } => Some(*line),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(medium_moves.len() < 60);
        assert_eq!(medium_moves.last(), Some(&0));
        assert!(medium_moves.windows(2).all(|pair| pair[0] > pair[1]));
        assert!(medium_engine.steps.iter().all(|step| match step {
            AnimationStep::Pause { multiplier } => {
                (*multiplier - CURSOR_MOVE_PAUSE * CURSOR_MOVE_MEDIUM_MULTIPLIER).abs()
                    < f64::EPSILON
            }
            _ => true,
        }));

        let long_lines = (0..=250)
            .map(|line| format!("{}item {line}", " ".repeat(line % 3)))
            .collect::<Vec<_>>();
        let long_refs = long_lines.iter().map(String::as_str).collect::<Vec<_>>();
        let mut long_engine = AnimationEngine::new(20);
        assert_eq!(
            long_engine.generate_cursor_movement(250, 0, 250, &long_refs),
            0
        );
        let long_moves = long_engine
            .steps
            .iter()
            .filter_map(|step| match step {
                AnimationStep::MoveCursor { line, .. } => Some(*line),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(long_moves.len() <= MAX_SCROLL_STEPS);
        assert_eq!(long_moves.last(), Some(&0));
        assert!(long_engine.steps.iter().all(|step| match step {
            AnimationStep::Pause { multiplier } => {
                (*multiplier - CURSOR_MOVE_PAUSE * CURSOR_MOVE_LONG_MULTIPLIER).abs() < f64::EPSILON
            }
            _ => true,
        }));
    }

    #[test]
    fn generate_steps_for_hunk_handles_context_deletion_and_addition() {
        let mut engine = AnimationEngine::new(30);
        let hunk = hunk(
            1,
            vec![
                line_change(LineChangeType::Context, "  keep"),
                line_change(LineChangeType::Deletion, "removed"),
                line_change(LineChangeType::Addition, "    added();"),
                line_change(LineChangeType::Context, "tail"),
            ],
        );

        assert_eq!(engine.generate_steps_for_hunk(&hunk, 0, 2), (4, 5));
        assert!(matches!(
            engine.steps.first(),
            Some(AnimationStep::MoveCursor { line: 2, col: 2 })
        ));
        assert!(matches!(
            engine.steps.get(2),
            Some(AnimationStep::DeleteLine { line: 3 })
        ));
        assert!(matches!(
            engine.steps.get(4),
            Some(AnimationStep::InsertLine { line: 3, content }) if content == "    "
        ));
        assert!(matches!(
            engine.steps.get(5),
            Some(AnimationStep::InsertChar {
                line: 3,
                col: 4,
                ch: 'a'
            })
        ));
        assert!(matches!(
            engine.steps.get(13),
            Some(AnimationStep::Pause { multiplier })
                if (*multiplier - INSERT_LINE_PAUSE).abs() < f64::EPSILON
        ));
        assert!(matches!(
            engine.steps.last(),
            Some(AnimationStep::Pause { multiplier })
                if (*multiplier - CURSOR_MOVE_PAUSE).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn execute_step_updates_dialog_terminal_and_editor_cursor_state() {
        let mut engine = AnimationEngine::new(30);

        engine.execute_step(AnimationStep::OpenFileDialogStart);
        engine.execute_step(AnimationStep::DialogTypeChar { ch: 's' });
        engine.execute_step(AnimationStep::TerminalPrompt);
        engine.execute_step(AnimationStep::TerminalTypeChar { ch: 'g' });
        engine.buffer = EditorBuffer::from_content("line\n  kept");
        engine.execute_step(AnimationStep::DeleteLine { line: 0 });
        engine.execute_step(AnimationStep::MoveCursor { line: 0, col: 2 });

        assert_eq!(engine.dialog_title.as_deref(), Some("Open File..."));
        assert_eq!(engine.dialog_typing_text, "s");
        assert_eq!(engine.terminal_lines, vec!["~ g".to_string()]);
        assert_eq!(engine.buffer.lines, vec!["  kept".to_string()]);
        assert_eq!(engine.buffer.cursor_line, 0);
        assert_eq!(engine.buffer.cursor_col, 2);
        assert_eq!(engine.line_offset, -1);
        assert_eq!(engine.active_pane, ActivePane::Editor);
    }

    #[test]
    fn tick_honors_manual_pause_timed_pause_and_finished_state() {
        let mut engine = AnimationEngine::new(1);
        engine.state = AnimationState::Playing;
        engine.pending_metadata = Some(metadata("abcdef1", "Tick", vec![]));
        engine.steps = vec![
            AnimationStep::ResetState,
            AnimationStep::Pause { multiplier: 1.0 },
            AnimationStep::TerminalOutput {
                text: "done".to_string(),
            },
        ];
        engine.frame_interval_ms = 0;
        engine.next_step_delay = 0;
        engine.last_update = Instant::now() - Duration::from_millis(10);
        engine.last_frame = Instant::now() - Duration::from_millis(10);
        engine.cursor_blink_timer = Instant::now() - Duration::from_millis(600);

        assert!(engine.tick());
        assert!(!engine.cursor_visible);
        assert_eq!(engine.current_step, 1);
        assert!(matches!(
            engine.current_metadata.as_ref(),
            Some(current) if current.hash == "abcdef1"
        ));

        engine.pause();
        assert!(engine.tick());
        assert_eq!(engine.current_step, 1);

        engine.resume();
        engine.last_update = Instant::now() - Duration::from_millis(10);
        engine.last_frame = Instant::now() - Duration::from_millis(10);
        assert!(engine.tick());
        assert_eq!(engine.current_step, 2);
        assert!(engine.pause_until.is_some());

        assert!(engine.tick());
        assert_eq!(engine.current_step, 2);

        engine.pause_until = Some(Instant::now() - Duration::from_millis(1));
        engine.last_update = Instant::now() - Duration::from_millis(10);
        engine.last_frame = Instant::now() - Duration::from_millis(10);
        assert!(engine.tick());
        assert_eq!(engine.terminal_lines, vec!["done".to_string()]);
        assert_eq!(engine.state, AnimationState::Finished);

        assert!(!engine.tick());
    }

    #[test]
    fn tick_waits_for_next_frame_and_setters_update_dimensions() {
        let mut engine = AnimationEngine::new(30);

        engine.set_content_width(0);
        engine.set_viewport_height(7);
        assert_eq!(engine.content_width, 0);
        assert_eq!(engine.viewport_height, 7);
        assert_eq!(engine.calculate_line_display_height("wrapped"), 1);

        engine.state = AnimationState::Playing;
        engine.frame_interval_ms = 1_000;
        engine.last_frame = Instant::now();
        engine.cursor_blink_timer = Instant::now();

        assert!(!engine.tick());
        assert_eq!(engine.state, AnimationState::Playing);
    }

    #[test]
    fn update_scroll_handles_zero_dimensions_middle_and_bottom_alignment() {
        let mut engine = AnimationEngine::new(30);

        assert_eq!(engine.calculate_line_display_height("wrapped"), 1);

        engine.content_width = 6;
        assert_eq!(engine.calculate_line_display_height("wrapped"), 1);

        engine.buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "line 5".to_string(),
        ];
        engine.viewport_height = 0;
        engine.buffer.cursor_line = 4;
        engine.update_scroll();
        assert_eq!(engine.buffer.scroll_offset, 0);

        engine.content_width = 30;
        engine.viewport_height = 2;
        engine.buffer.cursor_line = 3;
        engine.update_scroll();
        assert_eq!(engine.buffer.scroll_offset, 2);

        engine.buffer.cursor_line = 5;
        engine.update_scroll();
        assert_eq!(engine.buffer.scroll_offset, 4);
    }
}
