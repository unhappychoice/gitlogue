<p align="center">
  <img src="docs/assets/demo.gif" alt="gitlogue" width="820">
</p>

<p align="center">
  <a href="https://crates.io/crates/gitlogue"><img src="https://img.shields.io/crates/v/gitlogue.svg?style=flat-square&color=E06B4B" alt="crates.io"></a>
  <a href="https://github.com/unhappychoice/gitlogue/releases"><img src="https://img.shields.io/github/v/release/unhappychoice/gitlogue?style=flat-square&color=E0C14B&label=release" alt="release"></a>
  <a href="https://github.com/unhappychoice/gitlogue/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/unhappychoice/gitlogue/ci.yml?branch=main&style=flat-square&label=CI" alt="CI"></a>
  <a href="https://codecov.io/gh/unhappychoice/gitlogue"><img src="https://img.shields.io/codecov/c/github/unhappychoice/gitlogue?style=flat-square" alt="codecov"></a>
  <a href="https://github.com/unhappychoice/gitlogue/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/gitlogue.svg?style=flat-square" alt="license"></a>
  <a href="https://terminaltrove.com/gitlogue/"><img src="https://img.shields.io/badge/Terminal_Trove-Tool_of_The_Week-2ea043?style=flat-square" alt="Terminal Trove Tool of The Week"></a>
  <a href="https://hellogithub.com/repository/unhappychoice/gitlogue" target="_blank"><img src="https://abroad.hellogithub.com/v1/widgets/recommend.svg?rid=611a8aecd29d4ac295c64899448f395c&claim_uid=cSxaw6P2C5pfIlE&theme=small" alt="Featured｜HelloGitHub"></a>
</p>

<p align="center">
  <strong>A cinematic Git commit replay tool for the terminal.</strong><br>
  <sub>Turn your Git history into a living, animated story — typing, syntax highlighting, file tree transitions.</sub>
</p>

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
🔍 **Working Tree Diff View** — Visualize staged/unstaged changes before committing
🎨 **Tree-sitter Syntax Highlighting** — 32 languages supported
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

# Watch the repository and replay new commits as they are made
gitlogue --watch

# View staged changes (default)
gitlogue diff

# View unstaged changes instead
gitlogue diff --unstaged

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

## Key Bindings

### Playback

| Key | Action |
|-----|--------|
| `Space` | Toggle play / pause |
| `h` | Step one line backward |
| `l` | Step one line forward |
| `H` (Shift+h) | Step one change backward |
| `L` (Shift+l) | Step one change forward |
| `p` | Previous commit |
| `n` | Next commit |
| `Esc` | Open menu |
| `q` / `Ctrl+c` | Quit |

### Menu

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Select item |
| `Esc` | Close menu |

## Configuration

gitlogue can be configured via `~/.config/gitlogue/config.toml`.  
You can set the default theme, typing speed, and background preferences.

See the [Configuration Guide](docs/configuration.md) for full options and examples.

## Supported Languages

Astro, Bash, C, C#, C++, Clojure, CSS, Dart, Elixir, Erlang, Go, Godot (GDScript / Shader / Resource), Haskell, HTML, Java, JavaScript, JSON, Kotlin, Lua, Markdown, Nix, PHP, Python, Ruby, Rust, Scala, Svelte, Swift, TypeScript, XML, YAML, Zig

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

[@unhappychoice](https://unhappychoice.com)

## Support

If you find this project useful, please consider:

- ⭐️ [Star on GitHub](https://github.com/unhappychoice/gitlogue)
- 🐦 [Share on X](https://x.com/intent/post?text=Your%20Git%20history%20as%20cinema.%20gitlogue%20replays%20commits%20as%20a%20terminal%20animation%20%F0%9F%8E%9E%EF%B8%8F&url=https%3A//github.com/unhappychoice/gitlogue&hashtags=gitlogue,CLI,Rust,git)
- 🦋 [Share on Bluesky](https://bsky.app/intent/compose?text=Your%20Git%20history%20as%20cinema.%20gitlogue%20replays%20commits%20as%20a%20terminal%20animation%20%F0%9F%8E%9E%EF%B8%8F%20%23gitlogue%20%23CLI%20%23Rust%20%23git%20https%3A//github.com/unhappychoice/gitlogue)
- 🧵 [Share on Threads](https://www.threads.net/intent/post?text=Your%20Git%20history%20as%20cinema.%20gitlogue%20replays%20commits%20as%20a%20terminal%20animation%20%F0%9F%8E%9E%EF%B8%8F%20%23gitlogue%20%23CLI%20%23Rust%20%23git%20https%3A//github.com/unhappychoice/gitlogue)
- 💼 [Share on LinkedIn](https://www.linkedin.com/sharing/share-offsite/?url=https%3A//github.com/unhappychoice/gitlogue)
- 📘 [Share on Facebook](https://www.facebook.com/sharer/sharer.php?u=https%3A//github.com/unhappychoice/gitlogue)
- 🟧 [Submit to Hacker News](https://news.ycombinator.com/submitlink?u=https%3A//github.com/unhappychoice/gitlogue&t=Your%20Git%20history%20as%20cinema.%20gitlogue%20replays%20commits%20as%20a%20terminal%20animation%20%F0%9F%8E%9E%EF%B8%8F)
- 💬 Drop it into your Discord server or developer chat
- ✍️ Write about it on your blog or in a newsletter

Every bit of support helps. Thanks!
