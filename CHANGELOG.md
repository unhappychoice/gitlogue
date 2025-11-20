# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-11-20

### ✨ Features

- feat: support --order option with commit ranges ([89da1c5](https://github.com/unhappychoice/gitlogue/commit/89da1c5))
- feat: add commit range option ([96f6b4a](https://github.com/unhappychoice/gitlogue/commit/96f6b4a))
- feat: enable SIGTERM and SIGHUP handling in ctrlc crate ([64f170c](https://github.com/unhappychoice/gitlogue/commit/64f170c))
- feat: add Ctrl+C and q key support for quitting application ([78cb6d2](https://github.com/unhappychoice/gitlogue/commit/78cb6d2))

### 🐛 Bug Fixes

- fix: detect git repository from subdirectories ([5ee8605](https://github.com/unhappychoice/gitlogue/commit/5ee8605))

### 📝 Other Changes

- chore: bump version to v0.3.0 ([28e97f1](https://github.com/unhappychoice/gitlogue/commit/28e97f1))
- style: apply cargo fmt ([00c07e6](https://github.com/unhappychoice/gitlogue/commit/00c07e6))
- refactor: reject symmetric difference operator in commit range ([4ccc02d](https://github.com/unhappychoice/gitlogue/commit/4ccc02d))
- docs: update documentation for commit range feature ([1dd81be](https://github.com/unhappychoice/gitlogue/commit/1dd81be))
- chore(deps): bump clap from 4.5.52 to 4.5.53 ([5afa869](https://github.com/unhappychoice/gitlogue/commit/5afa869))
- Add 'bun.lockb' to ignored files list ([01d86ee](https://github.com/unhappychoice/gitlogue/commit/01d86ee))
- Add bun.lock to ignored files list ([d701fc4](https://github.com/unhappychoice/gitlogue/commit/d701fc4))
- docs: add instructions for installing on Arch Linux ([57a2c6e](https://github.com/unhappychoice/gitlogue/commit/57a2c6e))


## [0.2.0] - 2025-11-19

### ✨ Features

- feat: add --loop flag for continuous animation playback ([21c86db](https://github.com/unhappychoice/gitlogue/commit/21c86db))
- feat: add --order flag for commit playback order ([5a45a60](https://github.com/unhappychoice/gitlogue/commit/5a45a60))
- feat: add syntax highlighting for shell scripts ([76f68e2](https://github.com/unhappychoice/gitlogue/commit/76f68e2))
- feat: add OGP image generator and social preview ([0b3d187](https://github.com/unhappychoice/gitlogue/commit/0b3d187))

### 🐛 Bug Fixes

- fix: asc/desc order finishes after all commits played ([fe32bbf](https://github.com/unhappychoice/gitlogue/commit/fe32bbf))
- fix: use ~/.config for config path on all platforms ([b9c18e8](https://github.com/unhappychoice/gitlogue/commit/b9c18e8))
- fix(deps): update tree-sitter-yaml API usage for 0.7 compatibility ([ce47173](https://github.com/unhappychoice/gitlogue/commit/ce47173))
- fix(deps): update rand API usage for 0.9 compatibility ([aaf6a98](https://github.com/unhappychoice/gitlogue/commit/aaf6a98))

### 📝 Other Changes

- chore: bump version to v0.2.0 ([e18f250](https://github.com/unhappychoice/gitlogue/commit/e18f250))
- docs: add --loop option documentation ([0e03086](https://github.com/unhappychoice/gitlogue/commit/0e03086))
- docs: add --order option documentation ([db237d9](https://github.com/unhappychoice/gitlogue/commit/db237d9))
- docs: add Terminal Trove Tool of The Week badge ([8fbd92b](https://github.com/unhappychoice/gitlogue/commit/8fbd92b))
- chore: add CODEOWNERS file ([fddb7fb](https://github.com/unhappychoice/gitlogue/commit/fddb7fb))
- chore(deps): update tree-sitter-bash to v0.25 ([aa16451](https://github.com/unhappychoice/gitlogue/commit/aa16451))
- chore(deps): add tree-sitter-bash dependency ([bf7c3c4](https://github.com/unhappychoice/gitlogue/commit/bf7c3c4))
- chore(deps): bump tree-sitter-yaml from 0.6.1 to 0.7.2 ([5165e90](https://github.com/unhappychoice/gitlogue/commit/5165e90))
- chore(deps): bump rand from 0.8.5 to 0.9.2 ([2c902a3](https://github.com/unhappychoice/gitlogue/commit/2c902a3))
- chore(deps): bump toml from 0.8.23 to 0.9.8 ([3a4c730](https://github.com/unhappychoice/gitlogue/commit/3a4c730))
- chore(deps): bump dirs from 5.0.1 to 6.0.0 ([b8a86ed](https://github.com/unhappychoice/gitlogue/commit/b8a86ed))
- chore(deps): bump git2 from 0.19.0 to 0.20.2 ([db56c9f](https://github.com/unhappychoice/gitlogue/commit/db56c9f))
- chore(deps): bump tree-sitter-json from 0.23.0 to 0.24.8 ([0e51963](https://github.com/unhappychoice/gitlogue/commit/0e51963))
- chore(deps): bump clap from 4.5.51 to 4.5.52 ([8e04f01](https://github.com/unhappychoice/gitlogue/commit/8e04f01))
- chore(deps): bump crossterm from 0.28.1 to 0.29.0 ([89b4523](https://github.com/unhappychoice/gitlogue/commit/89b4523))
- chore(deps): bump tree-sitter-md from 0.3.2 to 0.5.1 ([f28e754](https://github.com/unhappychoice/gitlogue/commit/f28e754))
- chore(deps): bump toml_edit from 0.22.27 to 0.23.7 ([93cba7f](https://github.com/unhappychoice/gitlogue/commit/93cba7f))
- chore(deps): bump tree-sitter-css from 0.23.2 to 0.25.0 ([f177622](https://github.com/unhappychoice/gitlogue/commit/f177622))
- chore(deps): bump unicode-width from 0.1.14 to 0.2.0 ([f0c292e](https://github.com/unhappychoice/gitlogue/commit/f0c292e))
- chore: add dependabot configuration for Cargo dependencies ([720b887](https://github.com/unhappychoice/gitlogue/commit/720b887))
- docs: add OLED burn-in warning for screensaver mode ([e310e04](https://github.com/unhappychoice/gitlogue/commit/e310e04))
- refactor: increase OGP image padding for better spacing ([944839d](https://github.com/unhappychoice/gitlogue/commit/944839d))


## [0.1.0] - 2025-11-13

### 📝 Other Changes

- chore: bump version to v0.1.0 ([71b65d6](https://github.com/unhappychoice/gitlogue/commit/71b65d6))
- docs: add screensaver integration examples for Hyprland, Sway, i3, and X11 ([e31b6a4](https://github.com/unhappychoice/gitlogue/commit/e31b6a4))
- docs: expand Related Projects section with terminal screensavers ([047d7ca](https://github.com/unhappychoice/gitlogue/commit/047d7ca))
- Revise README for improved clarity and style ([ed8af4a](https://github.com/unhappychoice/gitlogue/commit/ed8af4a))


## [0.0.5] - 2025-11-12

### 🐛 Bug Fixes

- fix: include LICENSE-THIRD-PARTY in package for --license flag ([8b4b3f6](https://github.com/unhappychoice/gitlogue/commit/8b4b3f6))

### 📝 Other Changes

- chore: bump version to v0.0.5 ([f42ea87](https://github.com/unhappychoice/gitlogue/commit/f42ea87))


## [0.0.4] - 2025-11-12

### 🐛 Bug Fixes

- fix: reduce package size for crates.io by excluding unnecessary files ([9417aac](https://github.com/unhappychoice/gitlogue/commit/9417aac))

### 📝 Other Changes

- chore: bump version to v0.0.4 ([8df1167](https://github.com/unhappychoice/gitlogue/commit/8df1167))


## [0.0.3] - 2025-11-12

### 🐛 Bug Fixes

- fix: use vendored OpenSSL and libgit2 for cross-platform builds ([371338d](https://github.com/unhappychoice/gitlogue/commit/371338d))

### 📝 Other Changes

- chore: bump version to v0.0.3 ([74b0a5b](https://github.com/unhappychoice/gitlogue/commit/74b0a5b))


## [0.0.2] - 2025-11-12

### ✨ Features

- feat: add --license flag to display third-party licenses ([624b0d7](https://github.com/unhappychoice/gitlogue/commit/624b0d7))
- feat: add third-party license tracking ([5b7e078](https://github.com/unhappychoice/gitlogue/commit/5b7e078))
- feat: add Homebrew formula template ([ab80a69](https://github.com/unhappychoice/gitlogue/commit/ab80a69))
- feat: add installation script ([fd0d92b](https://github.com/unhappychoice/gitlogue/commit/fd0d92b))
- feat: add theme set command and config merging ([5a072e2](https://github.com/unhappychoice/gitlogue/commit/5a072e2))
- feat: implement config file with comment preservation ([9d32ccc](https://github.com/unhappychoice/gitlogue/commit/9d32ccc))
- feat: add toml_edit dependency for config comment preservation ([a044363](https://github.com/unhappychoice/gitlogue/commit/a044363))
- feat: add 6 new themes and sort themes alphabetically ([a4d2d6a](https://github.com/unhappychoice/gitlogue/commit/a4d2d6a))
- feat: add --background option for transparent background support ([3d4d78c](https://github.com/unhappychoice/gitlogue/commit/3d4d78c))
- feat: add SelectableParagraph widget with character-boundary wrapping ([a63b08a](https://github.com/unhappychoice/gitlogue/commit/a63b08a))
- feat: extend FileTree background to full width and fix rendering issues ([63a490c](https://github.com/unhappychoice/gitlogue/commit/63a490c))
- feat: update UI to use FileTree caching and unicode width ([5d9c685](https://github.com/unhappychoice/gitlogue/commit/5d9c685))
- feat: improve animation scroll with unicode width support ([7875c23](https://github.com/unhappychoice/gitlogue/commit/7875c23))
- feat: add caching and auto-scroll to FileTree ([e16e6d8](https://github.com/unhappychoice/gitlogue/commit/e16e6d8))
- feat: add sorted file indices method to CommitMetadata ([a2f53f6](https://github.com/unhappychoice/gitlogue/commit/a2f53f6))
- feat: add unicode-width dependency for proper text display width calculation ([131609f](https://github.com/unhappychoice/gitlogue/commit/131609f))
- feat: add exclusion for large files and snapshots ([c9d197f](https://github.com/unhappychoice/gitlogue/commit/c9d197f))
- feat: add exclusion for large files and snapshots ([9dcb567](https://github.com/unhappychoice/gitlogue/commit/9dcb567))
- feat: skip editor animation for renamed/moved files ([5470911](https://github.com/unhappychoice/gitlogue/commit/5470911))
- feat: skip editor animation for deleted files ([be7a325](https://github.com/unhappychoice/gitlogue/commit/be7a325))
- feat: add theme subcommand and configuration loading ([9c80186](https://github.com/unhappychoice/gitlogue/commit/9c80186))
- feat: add 8 built-in themes and theme loading system ([7da0532](https://github.com/unhappychoice/gitlogue/commit/7da0532))
- feat: add config module for theme management ([b1ba337](https://github.com/unhappychoice/gitlogue/commit/b1ba337))
- feat: add dirs dependency for config file support ([c8360bc](https://github.com/unhappychoice/gitlogue/commit/c8360bc))
- feat: add GitHub Actions CI/CD pipeline ([a2f74c1](https://github.com/unhappychoice/gitlogue/commit/a2f74c1))
- feat: enhance editor UI with distance-based opacity and cursor highlighting ([726cb3e](https://github.com/unhappychoice/gitlogue/commit/726cb3e))
- feat: add file dialog animation and eased cursor movement ([2440e8c](https://github.com/unhappychoice/gitlogue/commit/2440e8c))
- feat: add background colors and padding to all panes ([360a3d1](https://github.com/unhappychoice/gitlogue/commit/360a3d1))
- feat: add centralized Tokyo Night theme system ([543f6b0](https://github.com/unhappychoice/gitlogue/commit/543f6b0))
- feat: implement frame rate limiting and batch animation steps ([8927a6b](https://github.com/unhappychoice/gitlogue/commit/8927a6b))
- feat: exclude lock files and generated files from diff animation ([4e12cfe](https://github.com/unhappychoice/gitlogue/commit/4e12cfe))
- feat: implement input handling and exit mechanism ([f16f674](https://github.com/unhappychoice/gitlogue/commit/f16f674))
- feat(syntax): implement tree-sitter syntax highlighting for 26 languages ([8a3b1c3](https://github.com/unhappychoice/gitlogue/commit/8a3b1c3))
- feat(ui): enhance file tree with directory grouping and change stats ([49161be](https://github.com/unhappychoice/gitlogue/commit/49161be))
- feat(animation): make cursor movement faster than typing ([5e1b9cc](https://github.com/unhappychoice/gitlogue/commit/5e1b9cc))
- feat(animation): add random variation to typing speed ([9bd2fc8](https://github.com/unhappychoice/gitlogue/commit/9bd2fc8))
- feat(ui): auto-reload with random commits ([13f8267](https://github.com/unhappychoice/gitlogue/commit/13f8267))
- feat(ui): show cursor in active pane only ([d065558](https://github.com/unhappychoice/gitlogue/commit/d065558))
- feat(editor): add line numbers with highlighting ([c11bdad](https://github.com/unhappychoice/gitlogue/commit/c11bdad))
- feat(terminal): add file open and individual git add commands ([04fcae3](https://github.com/unhappychoice/gitlogue/commit/04fcae3))
- feat(terminal): add character-by-character typing for commands ([053783b](https://github.com/unhappychoice/gitlogue/commit/053783b))
- feat(terminal): add git command animation simulation ([35178e5](https://github.com/unhappychoice/gitlogue/commit/35178e5))
- feat(animation): animate cursor movement line by line ([24059e1](https://github.com/unhappychoice/gitlogue/commit/24059e1))
- feat(animation): add cursor movement between hunks ([b14432d](https://github.com/unhappychoice/gitlogue/commit/b14432d))
- feat(animation): add multi-file support ([7d36d1a](https://github.com/unhappychoice/gitlogue/commit/7d36d1a))
- feat(animation): add auto-scroll to keep cursor centered ([f76c1df](https://github.com/unhappychoice/gitlogue/commit/f76c1df))
- feat(animation): implement typing animation engine ([2064f5e](https://github.com/unhappychoice/gitlogue/commit/2064f5e))
- feat: reduce terminal pane height to 20% ([40d1407](https://github.com/unhappychoice/gitlogue/commit/40d1407))
- feat: implement basic ratatui UI layout ([991f0e9](https://github.com/unhappychoice/gitlogue/commit/991f0e9))
- feat: add full file content extraction for animation ([7f5db95](https://github.com/unhappychoice/gitlogue/commit/7f5db95))
- feat: implement structured diff parsing for animation (#5) ([a5bb886](https://github.com/unhappychoice/gitlogue/commit/a5bb886))
- feat: add file changes and diff extraction ([8696dc4](https://github.com/unhappychoice/gitlogue/commit/8696dc4))
- feat: implement Git repository and commit loading ([2b0c03d](https://github.com/unhappychoice/gitlogue/commit/2b0c03d))
- feat: implement CLI argument parsing ([2841866](https://github.com/unhappychoice/gitlogue/commit/2841866))
- feat: setup project structure and dependencies ([559f44e](https://github.com/unhappychoice/gitlogue/commit/559f44e))

### 🐛 Bug Fixes

- fix: track Cargo.lock for binary crate ([c639c2b](https://github.com/unhappychoice/gitlogue/commit/c639c2b))
- fix: prevent panic when area is narrower than padding ([b8b1f56](https://github.com/unhappychoice/gitlogue/commit/b8b1f56))
- fix: add auto-scroll to SelectableParagraph ([b31df27](https://github.com/unhappychoice/gitlogue/commit/b31df27))
- fix: correctly fill background to right edge when lines wrap ([3c9731f](https://github.com/unhappychoice/gitlogue/commit/3c9731f))
- fix: invalidate FileTree cache on content width changes ([2544422](https://github.com/unhappychoice/gitlogue/commit/2544422))
- fix: correct cursor line background fill with unicode width ([e56de82](https://github.com/unhappychoice/gitlogue/commit/e56de82))
- fix: correct viewport height calculation to match actual layout ([0704580](https://github.com/unhappychoice/gitlogue/commit/0704580))
- fix: remove go.mod from excluded files ([e906143](https://github.com/unhappychoice/gitlogue/commit/e906143))
- fix: correct byte offset calculation for CRLF line endings ([a1f6d22](https://github.com/unhappychoice/gitlogue/commit/a1f6d22))
- fix(syntax): improve markdown heading highlighting ([ac35d59](https://github.com/unhappychoice/gitlogue/commit/ac35d59))
- fix(animation): prevent infinite loop on new file additions ([5a1677a](https://github.com/unhappychoice/gitlogue/commit/5a1677a))
- fix(animation): convert Git 1-indexed line numbers to 0-indexed array indices ([761443a](https://github.com/unhappychoice/gitlogue/commit/761443a))
- fix(animation): correct line offset tracking across multiple hunks ([0d18444](https://github.com/unhappychoice/gitlogue/commit/0d18444))
- fix(animation): correct line number tracking in buffer ([cff3064](https://github.com/unhappychoice/gitlogue/commit/cff3064))
- fix(animation): start with empty editor before opening files ([c331621](https://github.com/unhappychoice/gitlogue/commit/c331621))
- fix(animation): handle UTF-8 character indices correctly ([36ea3bb](https://github.com/unhappychoice/gitlogue/commit/36ea3bb))

### 📝 Other Changes

- chore: bump version to v0.0.2 ([202a411](https://github.com/unhappychoice/gitlogue/commit/202a411))
- docs: update installation guide with new methods ([da9c4ff](https://github.com/unhappychoice/gitlogue/commit/da9c4ff))
- chore: set initial version to 0.0.1 ([4d81819](https://github.com/unhappychoice/gitlogue/commit/4d81819))
- docs: add installation methods to README ([1ac5d90](https://github.com/unhappychoice/gitlogue/commit/1ac5d90))
- ci: add automated release workflow ([4714f72](https://github.com/unhappychoice/gitlogue/commit/4714f72))
- docs: simplify GitType link description ([92a968c](https://github.com/unhappychoice/gitlogue/commit/92a968c))
- docs: add link to GitType in Related Projects section ([d7e4227](https://github.com/unhappychoice/gitlogue/commit/d7e4227))
- docs: remove milestone link from README ([4754146](https://github.com/unhappychoice/gitlogue/commit/4754146))
- docs: remove duplicate theme set command from Configuration ([3a62bb4](https://github.com/unhappychoice/gitlogue/commit/3a62bb4))
- docs: move Features section after Installation ([ce09a42](https://github.com/unhappychoice/gitlogue/commit/ce09a42))
- docs: simplify README configuration section ([2d2c6da](https://github.com/unhappychoice/gitlogue/commit/2d2c6da))
- docs: add detailed configuration guide ([88fcfcc](https://github.com/unhappychoice/gitlogue/commit/88fcfcc))
- docs: add configuration section to README ([c33404d](https://github.com/unhappychoice/gitlogue/commit/c33404d))
- refactor: remove old monolithic theme.rs file ([5e0a7be](https://github.com/unhappychoice/gitlogue/commit/5e0a7be))
- refactor: reorganize theme module into separate files ([fa6a5c7](https://github.com/unhappychoice/gitlogue/commit/fa6a5c7))
- style: format code with cargo fmt ([a852cdb](https://github.com/unhappychoice/gitlogue/commit/a852cdb))
- refactor: remove unnecessary wrap calculations from FileTree ([a6849f7](https://github.com/unhappychoice/gitlogue/commit/a6849f7))
- refactor: clean up FileTree code ([8da95db](https://github.com/unhappychoice/gitlogue/commit/8da95db))
- refactor: migrate Editor to SelectableParagraph with dim effect ([85810f0](https://github.com/unhappychoice/gitlogue/commit/85810f0))
- refactor: migrate FileTree to SelectableParagraph with dim effect ([93cc98f](https://github.com/unhappychoice/gitlogue/commit/93cc98f))
- refactor: migrate StatusBar and Terminal to SelectableParagraph ([3e2114c](https://github.com/unhappychoice/gitlogue/commit/3e2114c))
- refactor: use match expression for file status branching ([14c99ca](https://github.com/unhappychoice/gitlogue/commit/14c99ca))
- docs: add demo.gif converted from demo.mp4 ([c028d78](https://github.com/unhappychoice/gitlogue/commit/c028d78))
- docs: add architecture overview documentation ([c0625dd](https://github.com/unhappychoice/gitlogue/commit/c0625dd))
- docs: add contributing guidelines ([f1feb94](https://github.com/unhappychoice/gitlogue/commit/f1feb94))
- docs: add comprehensive usage guide with advanced examples ([50dcd17](https://github.com/unhappychoice/gitlogue/commit/50dcd17))
- docs: add comprehensive installation guide ([a6d790b](https://github.com/unhappychoice/gitlogue/commit/a6d790b))
- docs: enhance theme documentation with detailed guides ([6b8b072](https://github.com/unhappychoice/gitlogue/commit/6b8b072))
- docs: restructure README and update project description ([c8debd9](https://github.com/unhappychoice/gitlogue/commit/c8debd9))
- docs: add theme documentation and update README ([6661a11](https://github.com/unhappychoice/gitlogue/commit/6661a11))
- refactor: accept theme as parameter in UI constructor ([3cbe74d](https://github.com/unhappychoice/gitlogue/commit/3cbe74d))
- Apply suggestion from @coderabbitai[bot] ([c8a1143](https://github.com/unhappychoice/gitlogue/commit/c8a1143))
- perf: optimize char byte offset calculation from O(n²) to O(n) ([4131a78](https://github.com/unhappychoice/gitlogue/commit/4131a78))
- perf: optimize syntax highlighting performance ([0c6f38d](https://github.com/unhappychoice/gitlogue/commit/0c6f38d))
- chore: add ctrlc dependency for signal handling ([6db2f34](https://github.com/unhappychoice/gitlogue/commit/6db2f34))
- docs: add README and ISC LICENSE ([ad8fe8e](https://github.com/unhappychoice/gitlogue/commit/ad8fe8e))
- chore: apply cargo fmt and fix clippy warnings ([3c84db5](https://github.com/unhappychoice/gitlogue/commit/3c84db5))
- refactor: rename project from git-logue to gitlogue ([47d594e](https://github.com/unhappychoice/gitlogue/commit/47d594e))
- refactor(ui): preserve UI instance across commits and cleanup unused code ([c3abee5](https://github.com/unhappychoice/gitlogue/commit/c3abee5))
- refactor(animation): make all durations relative to typing speed ([d22e437](https://github.com/unhappychoice/gitlogue/commit/d22e437))
- refactor(ui): split UI into modular pane structure ([51662c7](https://github.com/unhappychoice/gitlogue/commit/51662c7))
- docs: add project specification ([7e8e4b5](https://github.com/unhappychoice/gitlogue/commit/7e8e4b5))


