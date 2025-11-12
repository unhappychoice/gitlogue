# gitlogue

<p align="center">
  <img src="docs/assets/demo.gif" alt="gitlogue demo" width="800" />
</p>

A terminal-based Git commit diff animator that brings your code changes to life.

Watch commits replay with realistic typing animations, syntax highlighting, and file tree views - transforming Git history into a cinematic coding experience.

## Installation

### Using Install Script (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/unhappychoice/gitlogue/main/install.sh | bash
```

### Using Homebrew

```bash
brew install unhappychoice/tap/gitlogue
```

### Using Cargo

```bash
cargo install gitlogue
```

### From Source

```bash
git clone https://github.com/unhappychoice/gitlogue.git
cd gitlogue
cargo install --path .
```

See the [Installation Guide](docs/installation.md) for more options and troubleshooting.

## Features

- 🎬 **Animated Commit Replay** - Watch commits being recreated with realistic typing animations
- 🎨 **Syntax Highlighting** - Support for 26 programming languages via tree-sitter
- 🎭 **Multiple Themes** - 9 built-in themes with customization support
- 🌳 **File Tree View** - Directory structure with change statistics
- 📊 **Commit Metadata** - Author, date, message, and change details
- 🎯 **Multiple Modes** - Random commit playback or view a specific commit
- ⚡ **Fast & Lightweight** - Built with Rust for performance

## Usage

### Popular Use Cases

- 🖥️  **Screensaver** - Ambient coding display for your workspace
- 🎓 **Education** - Show how code evolved over time
- 📺 **Presentations** - Live code history replay
- 🎬 **Content Creation** - Record demos with VHS or asciinema
- 🎨 **Desktop Ricing** - Perfect for tiling window manager setups
- 💼 **Look Busy** - Appear productive during meetings (we don't judge!)

### Quick Start

```bash
# Screensaver mode - random commits
gitlogue

# View a specific commit
gitlogue --commit abc123

# Use a different theme
gitlogue --theme dracula

# Adjust typing speed (ms per character)
gitlogue --speed 20

# List available themes
gitlogue theme list

# Set default theme
gitlogue theme set dracula

# Combine options
gitlogue --commit HEAD~5 --theme nord --speed 15
```

### Controls

- `Esc` or `Ctrl+C` - Quit the application

See the [Usage Guide](docs/usage.md) for detailed examples and advanced features.

## Configuration

gitlogue can be configured via `~/.config/gitlogue/config.toml`. You can set default theme, typing speed, and background preferences.

See the [Configuration Guide](docs/configuration.md) for detailed options and examples.

## Supported Languages

Rust, TypeScript, JavaScript, Python, Go, Ruby, Swift, Kotlin, Java, PHP, C#, C, C++, Haskell, Dart, Scala, Clojure, Zig, Elixir, Erlang, HTML, CSS, JSON, Markdown, YAML, XML

## Documentation

- [Installation Guide](docs/installation.md) - Detailed installation instructions for different platforms
- [Usage Guide](docs/usage.md) - Comprehensive usage examples and CLI options
- [Configuration Guide](docs/configuration.md) - Config file options and customization
- [Theme Customization](docs/themes.md) - Theme configuration and customization
- [Contributing Guidelines](docs/CONTRIBUTING.md) - How to contribute to the project
- [Architecture Overview](docs/ARCHITECTURE.md) - Technical architecture and design decisions

## Related Projects

Want to type code yourself? Try [**GitType**](https://github.com/unhappychoice/gittype).

## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](docs/CONTRIBUTING.md) for details on how to get started.

## License

ISC License - see [LICENSE](LICENSE) file for details.

## Author

[@unhappychoice](https://github.com/unhappychoice)
