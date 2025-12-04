# gitlogue

<a title="This tool is Tool of The Week on Terminal Trove, The $HOME of all things in the terminal" href="https://terminaltrove.com/gitlogue/"><img src="https://cdn.terminaltrove.com/media/badges/tool_of_the_week/svg/terminal_trove_tool_of_the_week_green_on_black_bg.svg" alt="Terminal Trove Tool of The Week" height="48" /></a>

<p align="center">
  <img src="docs/assets/demo.gif" alt="gitlogue demo" style="max-width: 100%; width: 800px;" />
</p>

A cinematic Git commit replay tool for the terminal, turning your Git history into a living, animated story.

Watch commits unfold with realistic typing animations, syntax highlighting, and file tree transitions, transforming code changes into a visual experience.

## Installation

### Using Install Script (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/unhappychoice/gitlogue/main/install.sh | bash
```

### Using Homebrew

```bash
brew install gitlogue
```

### Using Cargo

```bash
cargo install gitlogue
```

### On Arch Linux

```bash
pacman -S gitlogue
```

### Using Nix

```bash
# Run directly without installation
nix run github:unhappychoice/gitlogue

# Or install to your profile
nix profile install github:unhappychoice/gitlogue

# For flake-based configurations, add to your inputs:
# inputs.gitlogue.url = "github:unhappychoice/gitlogue";
# Then use: inputs.gitlogue.packages.${system}.default
```

### From Source

```bash
git clone https://github.com/unhappychoice/gitlogue.git
cd gitlogue
cargo install --path .
```

See the [Installation Guide](docs/installation.md) for more options and troubleshooting.

## Features

🎬 **Commit Replay as Animation** — Realistic typing, cursor movement, deletions, and file operations  
🎨 **Tree-sitter Syntax Highlighting** — 29 languages supported  
🌳 **Project File Tree** — Directory structure with change statistics  
🖥️ **Screensaver Mode** — Endless random commit playback  
🎭 **Themes** — 9 built-in themes + full customization support  
⚡ **Fast & Lightweight** — Built with Rust for performance  

## Usage

### Popular Use Cases

🖥️  **Screensaver** — Ambient coding display for your workspace  
🎓 **Education** — Visualize how code evolved over time  
📺 **Presentations** — Replay real commit histories live  
🎬 **Content Creation** — Record demos with VHS or asciinema  
🎨 **Desktop Ricing** — A living decoration for your terminal  
💼 **Look Busy Mode** — Appear productive during meetings

> [!WARNING]
> **Not a True Screensaver** — gitlogue does not include traditional screensaver functions like power management or screen blanking. It's purely a visual display tool.
>
> **OLED Burn-in Risk** — Static elements (like the editor background and border lines) may cause burn-in on OLED displays over extended periods. LCD displays are generally safe from this issue.

### Quick Start

```bash
# Start the cinematic screensaver
gitlogue

# View a specific commit
gitlogue --commit abc123

# Replay a range of commits
gitlogue --commit HEAD~5..HEAD

# Replay commits in chronological order (oldest first)
gitlogue --order asc

# Loop a specific commit continuously
gitlogue --commit abc123 --loop

# Loop through a commit range
gitlogue --commit HEAD~10..HEAD --loop

# Filter commits by author or email (case-insensitive partial match)
gitlogue --author "john"

# Filter commits by date
gitlogue --after "2024-01-01"
gitlogue --before "1 week ago"
gitlogue --after "2024-06-01" --before "2024-07-01"

# Use a different theme
gitlogue --theme dracula

# Adjust typing speed (ms per character)
gitlogue --speed 20

# Set different speeds for different file types
gitlogue --speed-rule "*.java:50" --speed-rule "*.xml:5"

# Ignore specific file patterns (e.g., notebooks, lock files)
gitlogue --ignore "*.ipynb" --ignore "poetry.lock"

# Use an ignore file
gitlogue --ignore-file .gitlogue-ignore

# List available themes
gitlogue theme list

# Set default theme
gitlogue theme set dracula

# Combine options
gitlogue --commit HEAD~5 --author "john" --theme nord --speed 15 --ignore "*.ipynb"
```

## Configuration

gitlogue can be configured via `~/.config/gitlogue/config.toml`.  
You can set the default theme, typing speed, and background preferences.

See the [Configuration Guide](docs/configuration.md) for full options and examples.

## Supported Languages

Bash, C, C#, C++, Clojure, CSS, Dart, Elixir, Erlang, Go, Haskell, HTML, Java, JavaScript, JSON, Kotlin, Lua, Markdown, PHP, Python, Ruby, Rust, Scala, Svelte, Swift, TypeScript, XML, YAML, Zig

## Documentation

[Installation Guide](docs/installation.md)  
[Usage Guide](docs/usage.md)  
[Configuration Guide](docs/configuration.md)  
[Theme Customization](docs/themes.md)  
[Contributing Guidelines](docs/CONTRIBUTING.md)  
[Architecture Overview](docs/ARCHITECTURE.md)

## Related Projects

### Git Visualization & Coding

- [**GitType**](https://github.com/unhappychoice/gittype) - A CLI code-typing game that turns your source code into typing challenges

### Terminal Screensavers

- [**tarts**](https://github.com/oiwn/tarts) - Collection of terminal screensavers in Rust (Matrix, Game of Life, Boids, 3D effects, and more)
- [**cbonsai**](https://gitlab.com/jallbrit/cbonsai) - Grow beautiful bonsai trees in your terminal
- [**asciiquarium**](https://github.com/cmatsuoka/asciiquarium) - Enjoy the mysteries of the sea from your terminal
- [**cmatrix**](https://github.com/abishekvashok/cmatrix) - The Matrix screensaver effect for your terminal
- [**pipes.sh**](https://github.com/pipeseroni/pipes.sh) - Animated pipes flowing through your terminal

## Contributing

Contributions are welcome.  
See the [Contributing Guidelines](docs/CONTRIBUTING.md) for details.

## License

ISC License. See [LICENSE](LICENSE) for details.

## Author

[@unhappychoice](https://github.com/unhappychoice)
