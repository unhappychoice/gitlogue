# Configuration Guide

gitlogue supports configuration via a TOML file located at `~/.config/gitlogue/config.toml`.

All configuration options are optional. CLI arguments take precedence over config file values.

## Quick Start

Set your default theme using the `theme set` command:

```bash
gitlogue theme set dracula
```

Or manually create/edit the config file:

```bash
# Create config directory
mkdir -p ~/.config/gitlogue

# Edit config file
vim ~/.config/gitlogue/config.toml
```

## Configuration File Format

The config file uses TOML format. Here's a complete example:

```toml
# gitlogue configuration file
# All settings are optional and will use defaults if not specified

# Theme to use for syntax highlighting
theme = "dracula"

# Typing speed in milliseconds per character
speed = 50

# Show background colors (set to false for transparent background)
background = true

# Commit playback order: random, asc, or desc
order = "random"

# Loop the animation continuously
loop = false

# Ignore patterns (gitignore syntax)
# Examples: ["*.ipynb", "poetry.lock", "docs/api/**"]
ignore_patterns = []

# Speed rules for different file types (pattern:milliseconds)
# Examples: ["*.java:50", "*.xml:5", "*.rs:30"]
speed_rules = []
```

## Configuration Options

### `theme`

Theme name for syntax highlighting.

- **Type**: String
- **Default**: `"tokyo-night"`
- **Example**: `theme = "dracula"`

See all available themes:

```bash
gitlogue theme list
```

Available themes: ayu-dark, catppuccin, dracula, everforest, github-dark, gruvbox, material, monokai, night-owl, nord, one-dark, rose-pine, solarized-dark, solarized-light, tokyo-night

### `speed`

Typing speed in milliseconds per character. Lower values = faster typing animation.

- **Type**: Integer
- **Default**: `30`
- **Range**: Typically 10-100
- **Example**: `speed = 20`

### `background`

Whether to show background colors in themes.

- **Type**: Boolean
- **Default**: `true`
- **Example**: `background = false`

Set to `false` for transparent background (useful for terminal transparency).

### `order`

Commit playback order.

- **Type**: String
- **Default**: `"random"`
- **Example**: `order = "asc"`

Available orders:
- `random` - Randomly selects commits (default)
- `asc` - Replays commits from oldest to newest
- `desc` - Replays commits from newest to oldest

### `loop`

Whether to loop the animation continuously.

- **Type**: Boolean
- **Default**: `false`
- **Example**: `loop = true`

When enabled, the animation will repeat indefinitely after completing. Especially useful with specific commits for demonstrations and ambient displays.

### `ignore_patterns`

List of patterns for files to ignore during animation.

- **Type**: Array of strings
- **Default**: `[]` (empty array)
- **Example**: `ignore_patterns = ["*.ipynb", "poetry.lock"]`

Pattern syntax (gitignore-style):
- `*.ipynb` - All Jupyter notebook files anywhere in the repository
- `poetry.lock` - Poetry lock file in repository root
- `docs/api/**` - All files under docs/api directory

**Note**: Binary files (images, videos, etc.) are already automatically excluded and don't need to be specified here.

Common use cases:
```toml
# Ignore Jupyter notebooks (JSON format, hard to read as text diff)
ignore_patterns = ["*.ipynb"]

# Ignore lock files that are tracked in git
ignore_patterns = ["poetry.lock", "Cargo.lock", "yarn.lock"]

# Ignore generated documentation
ignore_patterns = ["docs/api/**"]

# Combine multiple patterns
ignore_patterns = [
    "*.ipynb",
    "poetry.lock",
    "**/*.test.js",
    "**/__snapshots__/**"
]
```

Note: These patterns are additive with CLI `--ignore` flags and `--ignore-file` options. The ignore order is:
1. Config file `ignore_patterns`
2. `--ignore-file` patterns
3. CLI `--ignore` flags (highest priority)

### `speed_rules`

List of file-specific typing speed rules. Each rule specifies a glob pattern and speed in milliseconds.

- **Type**: Array of strings
- **Default**: `[]` (empty array)
- **Format**: `"PATTERN:MILLISECONDS"`
- **Example**: `speed_rules = ["*.java:50", "*.xml:5"]`

The first matching rule wins. Files that don't match any rule use the base `speed` setting.

Common use cases:
```toml
# Slow down for important source files, speed through configs
speed_rules = [
    "*.java:50",     # 50ms per char for Java (slower, more readable)
    "*.rs:40",       # 40ms per char for Rust
    "*.xml:5",       # 5ms per char for XML (fast)
    "*.json:5",      # 5ms per char for JSON (fast)
    "*.yaml:10",     # 10ms per char for YAML
]

# Focus on specific directories
speed_rules = [
    "src/**/*.ts:30",    # Slower for source TypeScript
    "tests/**/*.ts:10",  # Faster for test files
]
```

Note: CLI `--speed-rule` flags take priority over config file rules. Rules are evaluated in order (CLI first, then config).

## Configuration Priority

Settings are applied in the following order (highest priority first):

1. **CLI arguments** - Command-line flags override everything
   ```bash
   gitlogue --theme nord --speed 20 --background=false --order asc --loop --speed-rule "*.rs:50"
   ```

2. **Configuration file** - Values from `~/.config/gitlogue/config.toml`

3. **Default values** - Built-in defaults if nothing else is specified

### Example

Given this config file:

```toml
theme = "dracula"
speed = 50
```

Running this command:

```bash
gitlogue --speed 20
```

Will use:
- `theme = "dracula"` (from config)
- `speed = 20` (from CLI, overrides config)
- `background = true` (default value)

## Managing Configuration

### Using the `theme set` Command

The `theme set` command updates your config file while preserving comments:

```bash
# Set theme to nord
gitlogue theme set nord

# Your config file comments are preserved
cat ~/.config/gitlogue/config.toml
# Output:
# My custom comment here
theme = "nord"
speed = 50
```

### Manual Editing

You can also edit the config file directly:

```bash
vim ~/.config/gitlogue/config.toml
```

Add your own comments for documentation:

```toml
# I prefer darker themes for night coding
theme = "tokyo-night"

# Slower speed for demos
speed = 50

# Transparent background for my terminal setup
background = false
```

## Comment Preservation

gitlogue uses `toml_edit` to preserve all comments when updating the config file via commands:

- Block comments above settings
- Inline comments after values
- Custom user comments
- Formatting and whitespace

This means you can document your preferences in the config file without losing them when using `theme set`.

## Troubleshooting

### Config file not found

If the config file doesn't exist, gitlogue will use default values. You can create it:

```bash
mkdir -p ~/.config/gitlogue
cat > ~/.config/gitlogue/config.toml << 'EOF'
theme = "tokyo-night"
speed = 30
background = true
order = "random"
loop = false
ignore_patterns = []
speed_rules = []
EOF
```

Or use `theme set` to create it automatically:

```bash
gitlogue theme set tokyo-night
```

### Invalid TOML syntax

If your config file has syntax errors, gitlogue will show an error message. Validate your TOML:

```bash
# Check syntax
cat ~/.config/gitlogue/config.toml
```

Common mistakes:
- Missing quotes around string values: `theme = dracula` ❌ → `theme = "dracula"` ✅
- Wrong boolean syntax: `background = True` ❌ → `background = true` ✅

### Theme not found

If you set an invalid theme name:

```bash
gitlogue theme set invalid-theme
# Error: Unknown theme: invalid-theme
# Available themes: ayu-dark, catppuccin, dracula, ...
```

The config file won't be updated. Use `gitlogue theme list` to see available themes.

## Related

- [Theme Customization](themes.md) - Creating custom themes
- [Usage Guide](usage.md) - CLI options and examples
