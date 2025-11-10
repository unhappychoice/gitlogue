# Installation Guide

This guide provides detailed instructions for installing gitlogue on different platforms.

## Prerequisites

- Git must be installed on your system
- For building from source: Rust 1.70 or later

## Installation Methods

### Method 1: Using Cargo (Recommended)

The easiest way to install gitlogue is using Cargo, Rust's package manager:

```bash
cargo install gitlogue
```

This will download, compile, and install the latest version of gitlogue from [crates.io](https://crates.io).

### Method 2: From Source

If you want to build the latest development version or contribute to the project:

1. Clone the repository:
   ```bash
   git clone https://github.com/unhappychoice/gitlogue.git
   cd gitlogue
   ```

2. Build and install:
   ```bash
   cargo install --path .
   ```

   Or just build for development:
   ```bash
   cargo build --release
   ```

   The binary will be located at `target/release/gitlogue`.

### Method 3: Download Pre-built Binaries (Coming Soon)

Pre-built binaries for macOS, Linux, and Windows will be available in the [Releases](https://github.com/unhappychoice/gitlogue/releases) section.

## Platform-Specific Notes

### macOS

On macOS, you may need to install Xcode Command Line Tools if you haven't already:

```bash
xcode-select --install
```

### Linux

Make sure you have the build essentials installed:

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install build-essential git
```

**Fedora/RHEL:**
```bash
sudo dnf install gcc git
```

**Arch Linux:**
```bash
sudo pacman -S base-devel git
```

### Windows

1. Install [Rust](https://www.rust-lang.org/tools/install) using rustup
2. Install [Git for Windows](https://git-scm.com/download/win)
3. Follow the Cargo installation method above

## Verifying Installation

After installation, verify that gitlogue is correctly installed:

```bash
gitlogue --help
```

You should see the help message with available commands and options.

## Updating

### Cargo Installation

To update to the latest version:

```bash
cargo install gitlogue
```

Cargo will automatically update to the newest version.

### Source Installation

```bash
cd gitlogue
git pull origin main
cargo install --path .
```

## Uninstalling

To remove gitlogue from your system:

```bash
cargo uninstall gitlogue
```

## Configuration

Configuration file support is planned for future releases. See the [Usage Guide](usage.md) for available command-line options.

## Troubleshooting

### Command Not Found

If you get a "command not found" error after installation, make sure Cargo's bin directory is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this line to your shell configuration file (`~/.bashrc`, `~/.zshrc`, etc.) to make it permanent.

### Build Errors

If you encounter build errors:

1. Ensure you have the latest version of Rust:
   ```bash
   rustup update
   ```

2. Clean the build directory and try again:
   ```bash
   cargo clean
   cargo build --release
   ```

### Permission Errors

On Linux/macOS, if you encounter permission errors during installation, you may need to use `sudo`:

```bash
sudo cargo install gitlogue
```

However, it's generally recommended to use cargo without sudo and ensure your user has proper permissions.

## Next Steps

- Read the [Usage Guide](usage.md) to learn how to use gitlogue
- Explore [Theme Customization](themes.md) to personalize your experience
- Check out the [Contributing Guidelines](CONTRIBUTING.md) if you want to contribute

## Getting Help

If you encounter issues during installation:

- Check the [GitHub Issues](https://github.com/unhappychoice/gitlogue/issues) for known problems
- Open a new issue if your problem isn't already reported
- Include your OS, Rust version (`rustc --version`), and error messages
