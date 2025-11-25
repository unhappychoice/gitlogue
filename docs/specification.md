# gitlogue Specification (v0.1.0)

## Overview

`gitlogue` is a **Git history screensaver** — a terminal-based ambient experience where code rewrites itself over time.
Commits are replayed as if a ghost developer were typing each change by hand:
characters appear, vanish, and transform with natural pacing and syntax highlighting.

This first milestone focuses on building the **random commit playback** and **single-commit mode**,
with animated typing and a live editor-style display.

## Core Features for v0.1.0

### 1. Runtime Behavior

- **Command-line entrypoint:**
  ```bash
  gitlogue [--path <repo>] [--commit <hash>] [--speed <ms_per_char>]
  ```

- **Modes:**
  - No `--commit`: randomly select commits from the repository and loop indefinitely
  - With `--commit <hash>`: replay a specific commit once

- Launches immediately into playback (no interactivity)
- Exit gracefully on any key press or Ctrl+C

### 2. Commit Playback

- Use [`git2`](https://docs.rs/git2/latest/git2/) to load commit data and diffs
- For each commit (or the single specified one):
  - Compare file tree with previous state
  - For each changed file:
    - Load previous version into a virtual "editor buffer"
    - Apply diff hunks **as typing animation**:
      - inserted lines appear character by character
      - deleted lines are erased with simulated backspaces
      - modified lines rewrite in place
    - Apply **syntax highlighting** dynamically via `tree-sitter`
  - Display commit metadata (`author`, `date`, `message`) in a status bar
  - Small random pauses between edits to mimic natural typing flow

### 3. Visual Presentation (using `ratatui`)

- **Full-screen layout:**
  - **Editor pane:** animated code playback
  - **Status bar:** commit hash, author, date, and message
- Typing cursor blinks during active sequences
- Code colors update in real time through `tree-sitter` tokens
- Smooth scroll when edits exceed screen height

### 4. Random Commit Loop

- Default mode continuously selects random commits (excluding merges if possible)
- After each commit playback, short pause (1–3 seconds), then pick another
- Designed as a perpetual screensaver — no fixed end state

### 5. Configuration

Optional `.gitloguerc`:
```toml
[display]
speed = 30          # milliseconds per character
theme = "night"
highlight = true
cursor = true
```

## Stretch Goals (Optional)

- Author-based session transitions (`$ ssh alice@repo`)
- Subtle typing sound effects or background hum
- Branch-aware playback (random across branches)
- Asciinema / video export mode

## Tech Stack

- **Language:** Rust
- **Libraries:**
  - `git2` — history and diff extraction
  - `ratatui` — terminal rendering
  - `tree-sitter` — syntax highlighting
  - `chrono` — timestamps
  - `rand` — randomized commit selection & timing
  - `serde` — config parsing
  - `clap` — CLI argument parsing

## Milestone Criteria

✅ Random commit playback loop works
✅ Single-commit playback via `--commit`
✅ Animated typing for insert/delete/modify
✅ Syntax highlighting live-updates
✅ Runs indefinitely like a real terminal screensaver

## Implementation Breakdown

See the following issues for implementation tasks:

- #2: Setup project structure and dependencies
- #3: Implement CLI argument parsing
- #4: Implement Git repository and commit loading
- #5: Implement diff parsing and change extraction
- #6: Build basic ratatui UI layout
- #7: Implement typing animation engine
- #8: Implement syntax highlighting with tree-sitter
- #9: Implement commit playback orchestration
- #10: Implement input handling and exit mechanism
- #11: Implement configuration file support
