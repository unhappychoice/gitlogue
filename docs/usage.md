# Usage Guide

This guide covers all the features and options available in gitlogue.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Command-Line Options](#command-line-options)
- [Subcommands](#subcommands)
- [Configuration File](#configuration-file)
- [Keyboard Controls](#keyboard-controls)
- [Use Cases](#use-cases)
- [Tips and Tricks](#tips-and-tricks)

## Basic Usage

### Screensaver Mode (Default)

The simplest way to use gitlogue is to navigate to any Git repository and run:

```bash
cd /path/to/your/git/repo
gitlogue
```

This will start the screensaver mode, which:
- Randomly selects commits from the repository
- Replays them with typing animations
- Continues indefinitely until you press a key or `Ctrl+C`

### View a Specific Commit

To replay a specific commit:

```bash
gitlogue --commit a1b2c3d
```

Replace `a1b2c3d` with the commit hash you want to view. The commit hash can be:
- Full hash (40 characters)
- Short hash (7+ characters)
- Any valid Git reference (tag, branch name, etc.)

## Command-Line Options

### `--commit <HASH_OR_RANGE>`

Display a specific commit or commit range instead of random playback.

**Single commit:**
```bash
gitlogue --commit abc123
gitlogue --commit HEAD~5
gitlogue --commit v0.1.0
```

**Commit range:**
```bash
# Replay commits from HEAD~5 to HEAD
gitlogue --commit HEAD~5..HEAD

# Replay commits from a specific hash to HEAD
gitlogue --commit abc123..

# Replay commits from a specific hash to another
gitlogue --commit abc123..def456

# Open-ended range from beginning
gitlogue --commit ..HEAD~10
```

When using commit ranges:
- Commits are replayed in chronological order (oldest to newest)
- Each commit is displayed with full animation before moving to the next
- Merge commits are automatically excluded
- Use `--loop` to replay the range continuously

### `--author <PATTERN>` / `-a <PATTERN>`

Filter commits by author name or email address. The filter performs a case-insensitive partial match against both the author's name and email.

```bash
# Filter by name
gitlogue --author "John"
gitlogue -a "jane"

# Filter by email
gitlogue --author "@example.com"
gitlogue -a "john.doe@"

# Loop your own commits
gitlogue --author "alice" --loop
```

When using author filtering:
- Commits are replayed in chronological order (oldest to newest) by default
- Playback stops after all matching commits are shown
- Use `--loop` to replay continuously
- Use `--order` to change playback order

### `--theme <NAME>`

Select a theme for the UI.

```bash
gitlogue --theme dracula
gitlogue --theme nord
gitlogue --theme solarized-light
```

Available themes:
- `tokyo-night` (default)
- `dracula`
- `nord`
- `solarized-dark`
- `solarized-light`
- `monokai`
- `one-dark`
- `gruvbox`
- `catppuccin`

See the [Theme Customization Guide](themes.md) for more details.

### `--speed <MS>`

Set the typing speed in milliseconds per character. Default is 30ms.

```bash
gitlogue --speed 10   # Faster typing
gitlogue --speed 50   # Slower typing
gitlogue --speed 100  # Very slow typing
```

Lower values = faster typing. Recommended range: 10-100ms.

### `--order <ORDER>`

Set the commit playback order.

```bash
gitlogue --order asc     # Oldest first (chronological)
gitlogue --order desc    # Newest first (reverse chronological)
gitlogue --order random  # Random order (default)
```

Available orders:
- `random` (default) - Randomly selects commits
- `asc` - Replays commits from oldest to newest
- `desc` - Replays commits from newest to oldest

### `--loop`

Enable continuous looping of the animation.

```bash
gitlogue --loop                           # Loop random commits continuously
gitlogue --commit abc123 --loop           # Loop a specific commit
gitlogue --commit HEAD~5 --loop           # Loop the commit 5 before HEAD
gitlogue --commit HEAD~10..HEAD --loop    # Loop through a commit range
```

This is especially useful when viewing a specific commit or commit range and you want it to replay indefinitely, perfect for:
- Demonstrations and presentations
- Continuous display screens
- Desktop ricing and ambience
- Educational replays of feature development

### `--help`

Display help information:

```bash
gitlogue --help
```

### `--version`

Display version information:

```bash
gitlogue --version
```

### `--ignore <PATTERN>` / `-i <PATTERN>`

Ignore files matching patterns. Can be specified multiple times.

```bash
# Ignore Jupyter notebooks (JSON format, hard to read)
gitlogue --ignore "*.ipynb"

# Ignore multiple patterns
gitlogue -i "*.ipynb" -i "poetry.lock"

# Ignore generated documentation
gitlogue --ignore "docs/api/**"
```

Pattern syntax (gitignore-style):
- `*.ipynb` - All Jupyter notebook files anywhere in the repository
- `poetry.lock` - Poetry lock file in repository root
- `docs/api/**` - All files under docs/api directory

**Note**: Binary files (images, videos, etc.) are already automatically excluded by gitlogue and don't need to be specified here.

This is useful for:
- Skipping Jupyter notebooks (`.ipynb`) - JSON format that clutters the display
- Hiding large generated/lock files that are tracked in git
- Excluding minified JavaScript bundles
- Focusing on source code by hiding test snapshots or generated docs

### `--ignore-file <PATH>`

Read ignore patterns from a file (one pattern per line, gitignore-style syntax).

```bash
# Create an ignore file
cat > .gitlogue-ignore << EOF
# Jupyter notebooks
*.ipynb

# Lock files
poetry.lock
Cargo.lock

# Generated documentation
docs/api/**
EOF

# Use the ignore file
gitlogue --ignore-file .gitlogue-ignore
```

The file format:
- One pattern per line
- Empty lines are ignored
- Lines starting with `#` are treated as comments

You can combine multiple ignore sources - they are additive:

```bash
# Config file + ignore file + CLI flags
gitlogue --ignore-file .gitlogue-ignore -i "*.md"
```

## Subcommands

### `theme list`

List all available themes with their descriptions:

```bash
gitlogue theme list
```

This displays:
- Theme name
- Brief description
- Preview of the color scheme (coming soon)

## Keyboard Controls

While gitlogue is running:

- `Esc` - Quit the application
- `Ctrl+C` - Quit the application

## Use Cases

### 1. Screensaver

Run gitlogue on a display to show your coding activity:

```bash
gitlogue
```

Perfect for:
- Showcasing your project at conferences
- Office displays
- Developer portfolio presentations

### 2. Code Review Replay

Review how a specific feature was implemented:

```bash
# Find the merge commit
git log --oneline --merges | head -5

# Replay the commit
gitlogue --commit abc123

# Replay a series of commits from a feature branch
gitlogue --commit feature-start..feature-end
```

### 3. Educational Demonstrations

Show students or team members how code evolved:

```bash
# Show a specific refactoring commit
gitlogue --commit refactor-commit-hash --speed 20

# Demonstrate a feature development from start to finish
gitlogue --commit feature-start..feature-end --speed 15

# Show the last week of changes
git log --since="1 week ago" --format="%H" | head -1 | xargs -I {} gitlogue --commit {}..HEAD
```

### 4. Project History Visualization

Explore the history of a project:

```bash
cd popular-open-source-project
gitlogue
```

Watch the legacy being created - see how great projects evolved, one commit at a time, with syntax highlighting for all supported languages.

### 5. Look Busy Mode

gitlogue can be useful when you need to appear busy or demonstrate coding activity:

```bash
# Run on a second monitor or during presentations
gitlogue --speed 20 --theme dracula

# Use with an interesting open-source project
cd ~/Projects/famous-project
gitlogue --theme gruvbox
```

Perfect for:
- Making your workspace look active during meetings
- Background activity for coding livestreams
- Demonstrating "work in progress" during presentations
- Impressing non-technical stakeholders

**Pro tip**: Use `--speed 20` for faster, more impressive-looking typing, or `--speed 50` for a more relaxed pace.

## Tips and Tricks

### Use with Specific Branches

First checkout the branch, then run gitlogue:

```bash
git checkout feature-branch
gitlogue
```

gitlogue will only show commits from the current branch's history.

### Filter Commits by Author

Use the `--author` option to filter commits:

```bash
# Show commits from a specific author
gitlogue --author "Alice"

# Match by email
gitlogue --author "alice@example.com"

# Combine with other options
gitlogue --author "Bob" --theme dracula --speed 20
```

### Combine with Terminal Recording

Record a gitlogue session using various tools:

#### Using asciinema

```bash
asciinema rec gitlogue-demo.cast
gitlogue --commit abc123
# Press Esc to stop
exit
```

#### Using VHS

[VHS](https://github.com/charmbracelet/vhs) is a tool for generating terminal GIFs and videos. Here's how to create professional demos:

**Installation:**
```bash
# macOS
brew install vhs

# Linux (go install)
go install github.com/charmbracelet/vhs@latest
```

**Create a tape file** (`gitlogue-demo.tape`):

```tape
# VHS documentation: https://github.com/charmbracelet/vhs
Output gitlogue-demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800
Set Theme "Dracula"

Type "cd my-project"
Enter
Sleep 500ms

Type "gitlogue --theme dracula --commit abc123 && echo 'Finished'"
Enter

# Wait for gitlogue to complete
Wait /Finished/
Sleep 1s
```

**Run the tape:**
```bash
vhs gitlogue-demo.tape
```

This will generate `gitlogue-demo.gif` with your session automatically recorded and stopped when gitlogue exits.

### fzf Integration

Integrate gitlogue with [fzf](https://github.com/junegunn/fzf) for interactive commit selection.

Add this function to your `~/.bashrc` or `~/.zshrc`:

```bash
# Browse commits and launch gitlogue on selection
glf() {
  git log --oneline --color=always "$@" | \
    fzf --ansi \
        --no-sort \
        --preview 'git show --stat --color=always {1}' \
        --preview-window=right:60% \
        --bind 'enter:become(gitlogue --commit {1})'
}
```

Usage:

```bash
# Browse all commits
glf

# Browse commits from specific author
glf --author="Alice"

# Browse commits in date range
glf --after="2 weeks ago"
```

#### Advanced fzf Menu

Create a full interactive menu:

```bash
# Add to your shell config
gitlogue-menu() {
  local choice
  choice=$(echo -e "Random commits\nSpecific commit\nBy author\nBy date range\nTheme selection" | \
           fzf --prompt="gitlogue> " --height=40% --reverse)

  case "$choice" in
    "Random commits")
      gitlogue
      ;;
    "Specific commit")
      git log --oneline | \
        fzf --prompt="Select commit> " \
            --bind 'enter:become(gitlogue --commit {1})'
      ;;
    "By author")
      local author=$(git log --format='%an' | sort -u | fzf --prompt="Select author> ")
      [ -n "$author" ] && gitlogue --author "$author"
      ;;
    "By date range")
      local after=$(echo -e "1 day ago\n1 week ago\n2 weeks ago\n1 month ago" | fzf --prompt="After> ")
      [ -n "$after" ] && gitlogue --after "$after"
      ;;
    "Theme selection")
      local theme=$(gitlogue theme list | tail -n +2 | sed 's/^  - //' | fzf --prompt="Select theme> ")
      [ -n "$theme" ] && gitlogue --theme "$theme"
      ;;
  esac
}
```

Then use:

```bash
gitlogue-menu
```

### Screensaver Integration

gitlogue can be integrated with idle daemons to automatically start when your system is idle.

#### Hyprland with hypridle

Add to `~/.config/hypr/hypridle.conf`:

```bash
general {
    lock_cmd = pidof gitlogue || alacritty --class gitlogue-screensaver -e gitlogue
    before_sleep_cmd = loginctl lock-session
    after_sleep_cmd = hyprctl dispatch dpms on
}

listener {
    timeout = 300  # 5 minutes
    on-timeout = loginctl lock-session
}

listener {
    timeout = 600  # 10 minutes
    on-timeout = hyprctl dispatch dpms off
    on-resume = hyprctl dispatch dpms on
}
```

And in `~/.config/hypr/hyprland.conf`:

```bash
# Window rules for gitlogue screensaver
windowrulev2 = float,class:^(gitlogue-screensaver)$
windowrulev2 = fullscreen,class:^(gitlogue-screensaver)$
windowrulev2 = noblur,class:^(gitlogue-screensaver)$

# Start hypridle
exec-once = hypridle
```

Exit with Esc or any key to unlock.

#### Sway with swayidle

Add to `~/.config/sway/config`:

```bash
# Screensaver with swayidle
exec swayidle -w \
    timeout 300 'alacritty --class gitlogue-screensaver -e gitlogue' \
    timeout 600 'swaymsg "output * dpms off"' \
         resume 'swaymsg "output * dpms on"' \
    before-sleep 'swaylock -f'

# Window rules for gitlogue screensaver
for_window [app_id="gitlogue-screensaver"] fullscreen enable, floating enable
```

#### i3 with xautolock

Add to `~/.config/i3/config`:

```bash
# Screensaver with xautolock
exec --no-startup-id xautolock -time 5 -locker 'alacritty --class gitlogue-screensaver -e gitlogue'

# Window rules for gitlogue screensaver
for_window [class="gitlogue-screensaver"] fullscreen enable, floating enable
```

#### X11 with xidlehook

For more advanced idle detection:

```bash
# Install xidlehook
# Arch: pacman -S xidlehook
# Ubuntu: cargo install xidlehook

# Add to your WM startup or ~/.xinitrc
xidlehook \
  --not-when-fullscreen \
  --not-when-audio \
  --timer 300 \
    'alacritty --class gitlogue-screensaver -e gitlogue' \
    'pkill gitlogue' \
  --timer 600 \
    'xset dpms force off' \
    'xset dpms force on'
```

#### Standalone Screensaver Script

For any setup, create a simple script:

```bash
#!/bin/bash
# ~/.local/bin/gitlogue-screensaver

# Pick a random repository from your projects
REPOS=(
    ~/Projects/my-project
    ~/Projects/another-project
    ~/work/important-repo
)
REPO="${REPOS[$RANDOM % ${#REPOS[@]}]}"

# Launch gitlogue in fullscreen
cd "$REPO"
alacritty --class gitlogue-screensaver --title "Screensaver" \
    -o "window.opacity=0.95" \
    -e gitlogue --theme tokyo-night --speed 20
```

Make it executable:

```bash
chmod +x ~/.local/bin/gitlogue-screensaver
```

Use with any idle daemon by calling `gitlogue-screensaver` instead of `alacritty -e gitlogue`.

**Pro tips**:
- Use `--speed 15-20` for more dynamic screensaver effect
- Match the theme with your desktop environment
- Consider using `--background=false` for transparent terminals
- Add multiple repositories to randomly cycle through different projects

### Desktop Ricing

gitlogue is perfect for r/unixporn-style desktop customization and tiling window manager setups.

#### i3wm / Sway Configuration

Add to your i3/Sway config (`~/.config/i3/config` or `~/.config/sway/config`):

```bash
# Launch gitlogue in a floating window
for_window [title="gitlogue"] floating enable, resize set 1200 800, move position center

# Keybinding to launch gitlogue
bindsym $mod+g exec alacritty -t "gitlogue" -e gitlogue --theme tokyo-night

# Auto-start gitlogue on a specific workspace
exec --no-startup-id "i3-msg 'workspace 10; exec alacritty -e gitlogue'"
```

#### tmux Integration

Add a dedicated tmux window for gitlogue:

```bash
# ~/.tmux.conf
# Bind key to open gitlogue in new window
bind-key G new-window -n "gitlogue" "gitlogue --theme nord"
```

Or create a tmux session layout:

```bash
#!/bin/bash
# create-dev-session.sh

SESSION="dev"

tmux new-session -d -s $SESSION -n "code"
tmux send-keys -t $SESSION:0 "nvim" C-m

tmux new-window -t $SESSION:1 -n "terminal"
tmux send-keys -t $SESSION:1 "cd ~/Projects" C-m

tmux new-window -t $SESSION:2 -n "gitlogue"
tmux send-keys -t $SESSION:2 "gitlogue --theme catppuccin" C-m

tmux attach-session -t $SESSION
```

#### Conky Integration

Display gitlogue stats in Conky:

```lua
-- ~/.config/conky/conky.conf
conky.config = {
    -- ... your config
}

conky.text = [[
${color}Git Activity:
${execpi 300 cd ~/Projects/my-project && git log --oneline | head -5}

${color}Run: gitlogue to view
]]
```

#### Waybar Module

Add a custom waybar module for launching gitlogue:

```json
// ~/.config/waybar/config
{
  "custom/gitlogue": {
    "format": "  ",
    "tooltip": true,
    "tooltip-format": "Launch gitlogue",
    "on-click": "alacritty -e gitlogue --theme tokyo-night"
  }
}
```

#### Polybar Module

```ini
; ~/.config/polybar/config
[module/gitlogue]
type = custom/script
exec = echo ""
click-left = alacritty -e gitlogue &
format-foreground = #7aa2f7
format-padding = 1
```

#### Wallpaper Engine Alternative

Use gitlogue as a dynamic wallpaper:

```bash
#!/bin/bash
# gitlogue-wallpaper.sh

# Launch gitlogue in fullscreen on external monitor
export DISPLAY=:0.1
alacritty --fullscreen -e gitlogue --theme gruvbox --speed 25
```

Add to your window manager startup:

```bash
# For systems with multiple monitors
exec_always --no-startup-id ~/scripts/gitlogue-wallpaper.sh
```

#### Screenshots for r/unixporn

Perfect setup for impressive screenshots:

```bash
# Terminal: Alacritty with custom theme
# Font: JetBrains Mono Nerd Font
# WM: i3-gaps with 20px gaps
# Bar: Polybar

# Launch gitlogue with matching theme
gitlogue --theme nord --speed 20

# Take screenshot with scrot/grim
scrot ~/Pictures/rice-$(date +%Y%m%d-%H%M%S).png
```

**Pro tips for desktop ricing**:
- Match gitlogue theme with your terminal/WM theme
- Use `--speed 15-20` for more dynamic screenshots
- Consider transparency in terminal emulator for layered effects
- Combine with neofetch/pfetch in split panes

## Supported Languages

gitlogue provides syntax highlighting for 26 programming languages:

- **Systems**: Rust, C, C++, Zig
- **Web**: TypeScript, JavaScript, HTML, CSS
- **Backend**: Python, Go, Ruby, PHP, Java, C#, Kotlin, Swift
- **Functional**: Haskell, Scala, Clojure, Elixir, Erlang
- **Markup/Data**: Markdown, JSON, YAML, XML, Dart

The appropriate highlighter is automatically selected based on file extensions.

## Troubleshooting

### No Commits Displayed

If gitlogue shows no commits:

1. Ensure you're in a Git repository:
   ```bash
   git status
   ```

2. Check that the repository has commits:
   ```bash
   git log
   ```

3. Verify the current branch has history:
   ```bash
   git log --oneline
   ```

### Performance Issues

If gitlogue is slow:

1. Try increasing the speed value:
   ```bash
   gitlogue --speed 50
   ```

2. Check commit size:
   ```bash
   git show --stat <commit-hash>
   ```

Very large commits (1000+ files) may take longer to process.

### Theme Not Applied

If a theme isn't working:

1. List available themes:
   ```bash
   gitlogue theme list
   ```

2. Verify the theme name is correct (case-sensitive with hyphens):
   - Use `tokyo-night`, not `tokyo_night` or `TokyoNight`

3. Try specifying the theme explicitly:
   ```bash
   gitlogue --theme tokyo-night
   ```

## Next Steps

- Explore [Theme Customization](themes.md) to personalize the look
- Read the [Architecture Overview](ARCHITECTURE.md) to understand how gitlogue works
- Check out the [Contributing Guidelines](CONTRIBUTING.md) to contribute

## Feedback

Have suggestions for improving gitlogue? Open an issue on [GitHub](https://github.com/unhappychoice/gitlogue/issues)!
