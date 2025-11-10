# Architecture Overview

This document provides a comprehensive overview of gitlogue's architecture, design decisions, and implementation details.

## Table of Contents

- [High-Level Architecture](#high-level-architecture)
- [Core Components](#core-components)
- [Data Flow](#data-flow)
- [Key Design Decisions](#key-design-decisions)
- [Module Details](#module-details)
- [Performance Considerations](#performance-considerations)
- [Future Enhancements](#future-enhancements)

## High-Level Architecture

gitlogue is built as a terminal-based application using Rust. The architecture follows a modular design with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────┐
│                      main.rs                             │
│              (CLI Argument Parsing)                      │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                       UI                                 │
│          (Terminal UI Coordinator)                       │
│                                                          │
│  ┌────────────┬──────────────┬──────────────────────┐  │
│  │ FileTree   │   Editor     │      Terminal        │  │
│  │   Pane     │   Pane       │       Pane           │  │
│  └────────────┴──────────────┴──────────────────────┘  │
│  ┌──────────────────────────────────────────────────┐  │
│  │              Status Bar Pane                     │  │
│  └──────────────────────────────────────────────────┘  │
└──────────┬──────────────────┬───────────────┬──────────┘
           │                  │               │
           ▼                  ▼               ▼
┌──────────────────┐ ┌────────────────┐ ┌─────────────┐
│   Animation      │ │     Theme      │ │    Git      │
│    Engine        │ │    System      │ │  Repository │
└──────┬───────────┘ └────────────────┘ └─────────────┘
       │
       ▼
┌──────────────────┐
│     Syntax       │
│  Highlighting    │
│  (tree-sitter)   │
└──────────────────┘
```

## Core Components

### 1. CLI Interface (`main.rs`)

**Responsibility**: Parse command-line arguments and initialize the application.

**Key Features**:
- Argument parsing using `clap`
- Repository path validation
- Subcommand handling (e.g., `theme list`)
- Configuration loading and theme selection
- Initial commit loading

**Flow**:
1. Parse CLI arguments
2. Validate repository path
3. Load configuration
4. Resolve theme (CLI > config > default)
5. Load initial commit
6. Initialize and run UI

### 2. UI Coordinator (`ui.rs`)

**Responsibility**: Manage the terminal interface and coordinate between panes.

**Key Components**:
- Terminal setup and cleanup (raw mode, alternate screen)
- Event loop for keyboard input
- Layout management using `ratatui`
- State machine for animation flow
- Signal handling (Ctrl+C)

**States**:
- `Playing`: Animation in progress
- `WaitingForNext`: Pause between commits
- `Finished`: Animation complete (single commit mode)

**Layout Structure**:
```
┌────────────────────────────────────────┐
│            Status Bar                  │
├──────────────┬─────────────────────────┤
│              │                         │
│  File Tree   │      Editor             │
│              │                         │
│              │                         │
├──────────────┴─────────────────────────┤
│            Terminal                    │
└────────────────────────────────────────┘
```

### 3. Animation Engine (`animation.rs`)

**Responsibility**: Control the typing animation and edit sequence.

**Key Features**:
- State machine for animation steps
- Character-by-character typing simulation
- Line insertion and deletion
- Cursor movement with realistic timing
- Git command simulation (checkout, add, commit, push)

**Animation States**:
1. `Checkout`: Display git checkout command
2. `OpeningFile`: Show file opening
3. `MovingCursor`: Animate cursor movement
4. `Typing`: Character-by-character typing
5. `DeletingLine`: Line removal
6. `InsertingLine`: Line insertion
7. `WaitingBetweenHunks`: Pause between changes
8. `GitAdd`: Git add command
9. `GitCommit`: Git commit command
10. `GitPush`: Git push command
11. `Finished`: Complete

**Timing Configuration**:
The engine uses carefully tuned timing multipliers to create realistic animation:
- Cursor movement: Varies by distance (short/medium/long)
- Line deletion: 10× base speed
- Line insertion: 6.7× base speed
- Hunk transitions: 50× base speed
- Git commands: 16.7-66.7× base speed

### 4. Git Repository (`git.rs`)

**Responsibility**: Interface with Git repositories and extract commit data.

**Key Features**:
- Repository opening and validation
- Commit retrieval (random or specific)
- Diff parsing and hunk extraction
- File content loading
- Change detection (added/deleted/modified files)

**Excluded Files**:
- Lock files (package-lock.json, Cargo.lock, etc.)
- Minified files (.min.js, .min.css)
- Source maps (.js.map)
- Bundled files (.bundle.js)

**Performance Optimizations**:
- Commit caching to avoid repeated traversal
- Maximum blob size limit (500KB)
- File exclusion patterns

### 5. Syntax Highlighting (`syntax/`)

**Responsibility**: Provide syntax highlighting for code using tree-sitter.

**Supported Languages** (26 total):
- Systems: Rust, C, C++, Zig
- Web: TypeScript, JavaScript, HTML, CSS
- Backend: Python, Go, Ruby, PHP, Java, C#, Kotlin, Swift
- Functional: Haskell, Scala, Clojure, Elixir, Erlang
- Data: JSON, YAML, XML, Markdown, Dart

**Architecture**:
- Language detection by file extension
- Modular parser system (one module per language)
- Token-based highlighting with theme colors
- Highlight caching for performance

**Token Types**:
- Keywords, types, functions, variables
- Strings, numbers, comments
- Operators, punctuation
- Special tokens (imports, attributes, etc.)

### 6. Theme System (`theme.rs`)

**Responsibility**: Manage color schemes and UI styling.

**Built-in Themes** (9 total):
- tokyo-night (default)
- dracula
- nord
- solarized-dark/light
- monokai
- one-dark
- gruvbox
- catppuccin

**Theme Structure**:
```rust
pub struct Theme {
    pub background: BackgroundColors,
    pub editor: EditorColors,
    pub file_tree: FileTreeColors,
    pub terminal: TerminalColors,
    pub status_bar: StatusBarColors,
    pub syntax: SyntaxColors,
}
```

**Color Components**:
- Background colors (left/right panels)
- Editor elements (line numbers, cursor, separator)
- File tree status indicators
- Terminal UI elements
- Status bar sections
- Syntax highlighting tokens

### 7. Panes (`panes/`)

Individual UI components that render specific sections:

#### Editor Pane (`editor.rs`)
- Displays code with line numbers
- Shows cursor position
- Applies syntax highlighting
- Handles scrolling

#### File Tree Pane (`file_tree.rs`)
- Shows directory structure
- Displays file change status (added/deleted/modified)
- Highlights current file
- Shows change statistics

#### Terminal Pane (`terminal.rs`)
- Displays git command input
- Shows command output
- Simulates terminal session

#### Status Bar Pane (`status_bar.rs`)
- Shows commit hash
- Displays author and date
- Shows commit message

### 8. Configuration (`config.rs`)

**Responsibility**: Load and manage user configuration.

**Status**: Configuration file support is planned but not yet fully implemented.

**Planned Configuration File**: `~/.config/gitlogue/config.toml`

**Planned Settings**:
```toml
theme = "dracula"    # Default theme
speed = 30           # Typing speed (ms/char)
```

**Planned Priority**:
1. CLI arguments (highest)
2. Configuration file
3. Built-in defaults (lowest)

## Data Flow

### Startup Flow

```
1. Parse CLI arguments
2. Validate repository path
3. Open Git repository
4. Load configuration file
5. Resolve theme selection
6. Load initial commit
7. Initialize UI with theme
8. Start animation
```

### Animation Loop

```
1. UI receives commit metadata
2. Animation engine processes changes:
   a. Show git checkout
   b. For each file:
      - Open file
      - For each hunk:
        * Move cursor to position
        * Type new characters
        * Delete removed lines
        * Insert new lines
   c. Show git add
   d. Show git commit
   e. Show git push
3. For random mode:
   - Load next random commit
   - Repeat from step 2
4. For single commit mode:
   - Mark as finished
   - Wait for exit
```

### Event Handling

```
1. Terminal captures keyboard events
2. Any key press sets exit flag
3. UI checks exit flag each frame
4. On exit: cleanup and restore terminal
```

## Key Design Decisions

### 1. Rust as the Language

**Reasons**:
- Performance: Fast enough for smooth animations
- Safety: Memory safety without garbage collection
- Ecosystem: Excellent libraries (ratatui, tree-sitter, git2)
- Binary distribution: Single executable, no runtime needed

### 2. Ratatui for UI

**Reasons**:
- Modern terminal UI framework
- Efficient rendering (only updates changed cells)
- Layout system for responsive design
- Active community and good documentation

### 3. Tree-sitter for Syntax Highlighting

**Reasons**:
- Fast and incremental parsing
- Support for many languages
- Accurate syntax trees
- Battle-tested (used by editors like Neovim, Atom)

### 4. Git2 for Repository Access

**Reasons**:
- Low-level access to Git internals
- No external Git binary required
- Efficient commit traversal
- Comprehensive API

### 5. State Machine for Animation

**Reasons**:
- Clear animation flow
- Easy to add new states
- Pauseable and resumable
- Debugging friendly

### 6. Commit Caching

**Reasons**:
- Avoid repeated repository traversal
- Faster random commit selection
- Lazy initialization (only when needed)

## Module Details

### Module Dependency Graph

```
main.rs
  ├─> ui.rs
  │    ├─> animation.rs
  │    │    ├─> syntax/
  │    │    └─> git.rs
  │    ├─> panes/
  │    │    ├─> editor.rs
  │    │    ├─> file_tree.rs
  │    │    ├─> terminal.rs
  │    │    └─> status_bar.rs
  │    └─> theme.rs
  ├─> git.rs
  ├─> config.rs
  └─> theme.rs
```

### Key Data Structures

#### CommitMetadata
```rust
pub struct CommitMetadata {
    pub hash: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub files: Vec<FileChange>,
}
```

#### FileChange
```rust
pub struct FileChange {
    pub path: String,
    pub status: ChangeStatus,
    pub old_content: String,
    pub new_content: String,
    pub hunks: Vec<DiffHunk>,
}
```

#### DiffHunk
```rust
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub changes: Vec<LineChange>,
}
```

#### EditorBuffer
```rust
pub struct EditorBuffer {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub cached_highlights: Vec<HighlightSpan>,
}
```

## Performance Considerations

### 1. Efficient Rendering

- Only redraw changed regions
- Use ratatui's diffing algorithm
- Minimize terminal write operations

### 2. Syntax Highlighting Caching

- Pre-calculate highlights for old/new content
- Store highlights in editor buffer
- Reuse highlights across frames

### 3. Commit Filtering

- Exclude merge commits (optional)
- Skip large commits (>1000 files)
- Filter out lock files and generated files

### 4. Memory Management

- Limit blob size (500KB max)
- Clear cached highlights when switching files
- Lazy load commit list

### 5. Animation Timing

- Use monotonic time for accuracy
- Avoid busy-waiting with event polling
- Configurable frame rate

## Testing Strategy

### Unit Tests

- Animation state transitions
- Git diff parsing
- Theme loading
- Syntax highlighting

### Integration Tests

- Full commit playback
- UI rendering
- Configuration loading
- Theme switching

### Manual Testing

- Visual verification of animations
- Theme appearance
- Performance with large commits
- Error handling

## Future Enhancements

### Planned Features

1. **Custom Themes**
   - Load themes from `~/.config/gitlogue/themes/`
   - Theme validation and error reporting
   - Hot-reloading during development

2. **Advanced Playback Control**
   - Pause/resume animation
   - Speed adjustment on-the-fly
   - Skip to next commit

3. **Statistics Display**
   - Lines added/removed
   - File count
   - Commit time distribution

4. **Export Capabilities**
   - Record to asciinema format
   - Generate GIF/video
   - Export to HTML

5. **Branch Visualization**
   - Show branch names
   - Display merge commits
   - Branch switching animation

6. **Author-based Filtering**
   - Filter by author name
   - Author avatar/profile display
   - Contribution statistics

### Architecture Changes

1. **Plugin System**
   - Custom animation engines
   - Language support extensions
   - Theme providers

2. **Configuration Extensions**
   - Per-repository settings
   - Project-specific themes
   - Animation presets

3. **Performance Improvements**
   - Multi-threaded syntax highlighting
   - Incremental diff parsing
   - GPU acceleration (experimental)

## Contributing

For information on contributing to gitlogue, see the [Contributing Guidelines](CONTRIBUTING.md).

### Architecture Guidelines

When making changes:

1. **Maintain separation of concerns**
   - Keep modules focused and independent
   - Use clear interfaces between components

2. **Preserve animation quality**
   - Test timing changes visually
   - Maintain realistic feel

3. **Follow Rust best practices**
   - Use type safety
   - Avoid unsafe code unless necessary
   - Document public APIs

4. **Consider performance**
   - Profile before optimizing
   - Measure impact of changes
   - Avoid premature optimization

## References

- [Ratatui Documentation](https://docs.rs/ratatui/)
- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [Git2-rs Documentation](https://docs.rs/git2/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

This architecture document is maintained alongside the codebase. If you notice discrepancies or have suggestions for improvements, please open an issue or pull request.
