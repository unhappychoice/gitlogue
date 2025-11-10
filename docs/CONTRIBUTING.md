# Contributing to gitlogue

Thank you for your interest in contributing to gitlogue! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)
- [Project Structure](#project-structure)

## Code of Conduct

This project follows a simple code of conduct:

- Be respectful and considerate in all interactions
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Respect different viewpoints and experiences

## Getting Started

### Prerequisites

- Git
- Rust 1.70 or later
- A GitHub account
- Familiarity with Rust programming

### First Steps

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/gitlogue.git
   cd gitlogue
   ```

3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/unhappychoice/gitlogue.git
   ```

4. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Building the Project

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run the project
cargo run

# Run with arguments
cargo run -- --theme dracula
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Running Examples

```bash
# Run the syntax highlighter test
cargo run --example test_highlighter
```

### Code Formatting

```bash
# Format code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check
```

### Linting

```bash
# Run clippy for linting
cargo clippy

# Run clippy with all features
cargo clippy --all-features -- -D warnings
```

## Making Changes

### Branch Naming

Use descriptive branch names with prefixes:

- `feature/` - New features (e.g., `feature/custom-themes`)
- `fix/` - Bug fixes (e.g., `fix/crash-on-empty-repo`)
- `docs/` - Documentation changes (e.g., `docs/improve-readme`)
- `refactor/` - Code refactoring (e.g., `refactor/theme-loading`)
- `test/` - Test additions or improvements (e.g., `test/add-integration-tests`)

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(themes): add catppuccin theme

Add the Catppuccin Mocha theme variant with carefully chosen colors
for optimal readability.

Closes #42
```

```
fix(animation): prevent crash on large commits

Fixed a panic that occurred when processing commits with more than
1000 changed files by adding proper bounds checking.

Fixes #38
```

### Keeping Your Fork Updated

```bash
# Fetch upstream changes
git fetch upstream

# Merge upstream main into your branch
git checkout main
git merge upstream/main

# Push updates to your fork
git push origin main
```

## Coding Standards

### Rust Style Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format code
- Address all `cargo clippy` warnings
- Write idiomatic Rust code

### Code Organization

- Keep functions small and focused (aim for 10-15 lines)
- Use meaningful variable and function names
- Avoid deep nesting (max 3-4 levels)
- Prefer composition over inheritance
- Use modules to organize related functionality

### Documentation

- Add doc comments for public APIs:
  ```rust
  /// Loads a theme by name from built-in themes.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the theme to load
  ///
  /// # Returns
  ///
  /// Returns `Some(Theme)` if found, `None` otherwise
  pub fn load_theme(name: &str) -> Option<Theme> {
      // implementation
  }
  ```

- Include examples in doc comments when helpful
- Update documentation when changing APIs

### Error Handling

- Use `Result` and `?` operator for recoverable errors
- Use `panic!` only for unrecoverable errors
- Provide meaningful error messages
- Use `anyhow` for application-level errors

### Performance

- Avoid unnecessary allocations
- Use iterators instead of loops when appropriate
- Profile before optimizing
- Document performance considerations

## Testing

### Writing Tests

- Add unit tests for new functionality
- Add integration tests for user-facing features
- Aim for good code coverage
- Test edge cases and error conditions

Example unit test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_loading() {
        let theme = load_theme("dracula");
        assert!(theme.is_some());

        let theme = load_theme("nonexistent");
        assert!(theme.is_none());
    }
}
```

### Running Tests Locally

```bash
# Run all tests
cargo test

# Run specific test module
cargo test theme::tests

# Run with verbose output
cargo test -- --nocapture
```

## Submitting Changes

### Before Submitting

1. **Ensure all tests pass**:
   ```bash
   cargo test
   ```

2. **Format your code**:
   ```bash
   cargo fmt
   ```

3. **Run clippy**:
   ```bash
   cargo clippy
   ```

4. **Update documentation** if needed

5. **Add tests** for new functionality

### Creating a Pull Request

1. **Push your changes** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Open a pull request** on GitHub

3. **Fill out the PR template** with:
   - Clear description of changes
   - Related issue numbers
   - Testing performed
   - Screenshots for UI changes

4. **Wait for review** and address feedback

### Pull Request Guidelines

- Keep PRs focused on a single feature or fix
- Include tests for new functionality
- Update documentation as needed
- Respond to review comments promptly
- Rebase on main if conflicts arise

## Reporting Bugs

### Before Reporting

1. **Check existing issues** to avoid duplicates
2. **Try the latest version** to see if it's fixed
3. **Gather information** about the bug

### Bug Report Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Run command '...'
2. Observe error '...'

**Expected behavior**
What you expected to happen.

**Environment:**
- OS: [e.g., macOS 13.0, Ubuntu 22.04]
- Rust version: [e.g., 1.70.0]
- gitlogue version: [e.g., 0.1.0]

**Additional context**
Any other relevant information.
```

## Suggesting Features

### Feature Request Template

```markdown
**Feature Description**
A clear description of the feature.

**Use Case**
Explain why this feature would be useful.

**Proposed Implementation**
If you have ideas about how to implement it.

**Alternatives**
Any alternative solutions you've considered.
```

## Project Structure

```
gitlogue/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root
│   ├── animation.rs      # Animation engine
│   ├── config.rs         # Configuration handling
│   ├── git.rs            # Git operations
│   ├── theme.rs          # Theme system
│   ├── ui.rs             # Main UI coordinator
│   ├── panes/            # UI components
│   │   ├── mod.rs
│   │   ├── editor.rs     # Code editor pane
│   │   ├── file_tree.rs  # File tree pane
│   │   ├── status_bar.rs # Status bar
│   │   └── terminal.rs   # Terminal pane
│   └── syntax/           # Syntax highlighting
│       ├── mod.rs
│       └── languages/    # Language parsers
├── docs/                 # Documentation
├── examples/             # Example programs
└── tests/                # Integration tests
```

### Key Modules

- **animation**: Handles typing animation and timing
- **git**: Git repository operations and diff parsing
- **theme**: Theme loading and management
- **ui**: Ratatui-based terminal UI
- **panes**: Individual UI components
- **syntax**: Tree-sitter syntax highlighting

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Chat**: Join our community (coming soon)
- **Issues**: Search existing issues or open a new one

## Recognition

Contributors will be:
- Listed in the project's contributors page
- Mentioned in release notes for significant contributions
- Given credit in commit messages

## License

By contributing to gitlogue, you agree that your contributions will be licensed under the ISC License.

---

Thank you for contributing to gitlogue! Your efforts help make this project better for everyone.
