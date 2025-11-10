# Theme Customization Guide

gitlogue provides a rich theming system with 9 beautiful built-in themes and support for custom theme configuration.

## Table of Contents

- [Built-in Themes](#built-in-themes)
- [Using Themes](#using-themes)
- [Theme Configuration](#theme-configuration)
- [Creating Custom Themes](#creating-custom-themes)
- [Theme Structure](#theme-structure)
- [Troubleshooting](#troubleshooting)

## Built-in Themes

gitlogue comes with 9 carefully crafted themes optimized for code readability and visual appeal:

### Dark Themes

- **tokyo-night** (default) - A modern dark theme inspired by Tokyo's neon nights. Balanced contrast with blue and purple accents.
- **dracula** - A vibrant dark theme with rich purples and greens. High contrast for extended viewing.
- **nord** - A cool, arctic-inspired color palette with muted blues and grays. Easy on the eyes for long sessions.
- **solarized-dark** - The legendary Solarized theme (dark variant). Scientifically designed for optimal readability.
- **monokai** - A classic theme with warm browns and oranges. Reminiscent of vintage terminals.
- **one-dark** - Inspired by Atom's One Dark theme. Clean and professional with excellent syntax highlighting.
- **gruvbox** - A retro groove color scheme with warm, earthy tones. Perfect for a cozy coding atmosphere.
- **catppuccin** - A soothing pastel theme (Mocha variant) with soft, gentle colors.

### Light Themes

- **solarized-light** - The popular Solarized theme (light variant). Perfect for well-lit environments.

## Using Themes

### Listing Available Themes

To see all available themes with descriptions:

```bash
gitlogue theme list
```

This command displays:
- Theme names
- Brief descriptions
- Whether they are dark or light themes

### Selecting a Theme

#### Via Command Line

Use the `--theme` option to specify a theme for a single session:

```bash
gitlogue --theme dracula
gitlogue --theme nord
gitlogue --theme solarized-light
```

Examples:
```bash
# Use Dracula theme for current directory
gitlogue --theme dracula

# View a specific commit with Nord theme
gitlogue --commit abc123 --theme nord

# Slower typing with Gruvbox theme
gitlogue --speed 50 --theme gruvbox
```

#### Via Configuration File

> **Note:** Configuration file support is planned for a future release and is not yet available.

Once implemented, you'll be able to set a default theme in `~/.config/gitlogue/config.toml`.

## Creating Custom Themes

> **Note:** Custom theme support is planned for future releases and is not yet available in the current version.

Custom themes will be loaded from the `~/.config/gitlogue/themes/` directory and support both JSON and TOML formats.

### Planned Features

- Load themes from `~/.config/gitlogue/themes/`
- Support for TOML and JSON format
- Hot-reloading during development
- Theme validation and error reporting
- Theme sharing via GitHub gists

## Theme Structure

A gitlogue theme defines colors for all UI components:

### UI Components

- **Background colors**: Left panel (file tree) and right panel (editor) backgrounds
- **Editor colors**: Line numbers, cursor, separators, selection
- **File tree colors**: Status indicators (added, deleted, modified, renamed)
- **Terminal colors**: Command input, output, cursor, prompt
- **Status bar colors**: Commit hash, author, date, message
- **Syntax highlighting colors**: Keywords, types, functions, strings, comments, operators, etc.

### Example Theme File Structure (TOML)

```toml
# ~/.config/gitlogue/themes/my-theme.toml

name = "My Custom Theme"
description = "A beautiful custom theme"
author = "Your Name"
variant = "dark"  # or "light"

[background]
left = { r = 30, g = 34, b = 54 }
right = { r = 26, g = 27, b = 38 }

[editor]
line_number = { r = 86, g = 95, b = 137 }
line_number_cursor = { r = 125, g = 207, b = 255 }
cursor_char_bg = { r = 122, g = 162, b = 247 }
cursor_char_fg = { r = 26, g = 27, b = 38 }
separator = { r = 50, g = 54, b = 74 }

[file_tree]
added = { r = 158, g = 206, b = 106 }
deleted = { r = 247, g = 118, b = 142 }
modified = { r = 255, g = 158, b = 100 }
renamed = { r = 125, g = 207, b = 255 }

[terminal]
command_input = { r = 169, g = 177, b = 214 }
command_output = { r = 192, g = 202, b = 245 }
cursor = { r = 125, g = 207, b = 255 }

[status_bar]
hash = { r = 187, g = 154, b = 247 }
author = { r = 125, g = 207, b = 255 }
date = { r = 255, g = 158, b = 100 }
message = { r = 192, g = 202, b = 245 }

[syntax]
keyword = { r = 187, g = 154, b = 247 }
type = { r = 125, g = 207, b = 255 }
function = { r = 130, g = 170, b = 255 }
string = { r = 158, g = 206, b = 106 }
number = { r = 255, g = 158, b = 100 }
comment = { r = 86, g = 95, b = 137 }
operator = { r = 187, g = 154, b = 247 }
variable = { r = 169, g = 177, b = 214 }
constant = { r = 255, g = 158, b = 100 }
```

### Color Format

Colors are specified as RGB values with components ranging from 0 to 255:

```toml
color_name = { r = 255, g = 100, b = 50 }
```

## Choosing the Right Theme

### For Long Sessions
- **nord** - Easy on the eyes with muted colors
- **gruvbox** - Warm, comfortable color palette
- **catppuccin** - Gentle pastel tones

### For High Contrast
- **dracula** - Vibrant and clear
- **solarized-dark** - Scientifically optimized contrast

### For Presentations
- **tokyo-night** - Modern and professional
- **one-dark** - Clean and clear

### For Bright Environments
- **solarized-light** - Optimized for well-lit spaces

## Troubleshooting

### Theme Not Loading

If your theme isn't applied:

1. Check the theme name is correct:
   ```bash
   gitlogue theme list
   ```

2. Verify the theme name spelling (case-sensitive with hyphens):
   - Use `tokyo-night`, not `tokyo_night` or `TokyoNight`

3. Try specifying the theme explicitly:
   ```bash
   gitlogue --theme tokyo-night
   ```

### Colors Look Wrong

If colors appear incorrect:

1. Check your terminal's color support:
   ```bash
   echo $TERM
   ```

2. Ensure your terminal supports 24-bit colors (truecolor)
3. Try a different terminal emulator if colors still look wrong

## Next Steps

- Explore different themes to find your favorite
- Check the [Usage Guide](usage.md) for more options
- See the [Architecture Overview](ARCHITECTURE.md) to understand theme implementation

## Feedback

Have ideas for new themes? Open an issue or pull request on [GitHub](https://github.com/unhappychoice/gitlogue)!
