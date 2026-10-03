# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.0] - 2026-10-03

### ✨ Features

- feat: return to waiting screen after watch playback finishes ([784f6e5](https://github.com/unhappychoice/gitlogue/commit/784f6e5))
- feat: add watch mode to replay new commits as they are made ([9cfdddf](https://github.com/unhappychoice/gitlogue/commit/9cfdddf))

### 🐛 Bug Fixes

- fix: respect pause and detached checkouts in watch mode ([3b7f372](https://github.com/unhappychoice/gitlogue/commit/3b7f372))
- fix: redraw when UI state changes without animation progress ([8a7158b](https://github.com/unhappychoice/gitlogue/commit/8a7158b))
- fix(ci): fail Homebrew SHA step on HTTP errors ([9a27f93](https://github.com/unhappychoice/gitlogue/commit/9a27f93))

### 📝 Other Changes

- chore: bump version to v0.12.0 ([06a19b7](https://github.com/unhappychoice/gitlogue/commit/06a19b7))
- refactor: show waiting screen instead of startup HEAD in watch mode ([f89347f](https://github.com/unhappychoice/gitlogue/commit/f89347f))
- chore(deps): bump rand from 0.10.2 to 0.10.3 ([c2d782c](https://github.com/unhappychoice/gitlogue/commit/c2d782c))
- chore(deps): bump toml_edit ([76d7e99](https://github.com/unhappychoice/gitlogue/commit/76d7e99))
- chore(deps): bump toml from 1.1.5+spec-1.1.0 to 1.1.6+spec-1.1.0 ([0b34718](https://github.com/unhappychoice/gitlogue/commit/0b34718))
- chore(deps): bump dirs from 6.0.0 to 7.0.0 ([bdcffbc](https://github.com/unhappychoice/gitlogue/commit/bdcffbc))
- chore(deps): bump toml from 1.1.4+spec-1.1.0 to 1.1.5+spec-1.1.0 ([59dc7b3](https://github.com/unhappychoice/gitlogue/commit/59dc7b3))
- chore(deps): bump chrono-english from 0.2.0 to 0.2.1 ([971f97b](https://github.com/unhappychoice/gitlogue/commit/971f97b))
- chore: update flake.nix hashes for v0.11.0 ([4868df2](https://github.com/unhappychoice/gitlogue/commit/4868df2))


## [0.11.0] - 2026-08-30

### ✨ Features

- feat(syntax): add Godot file highlighting ([2aeb59b](https://github.com/unhappychoice/gitlogue/commit/2aeb59b))

### 🐛 Bug Fixes

- fix(syntax): narrow GDScript parameter highlights ([7362948](https://github.com/unhappychoice/gitlogue/commit/7362948))

### 📝 Other Changes

- chore: bump version to v0.11.0 ([c463184](https://github.com/unhappychoice/gitlogue/commit/c463184))
- chore(nix): update nixpkgs to fix crate downloads (#234) ([e20bba9](https://github.com/unhappychoice/gitlogue/commit/e20bba9))
- docs: refresh supported language lists ([b2991eb](https://github.com/unhappychoice/gitlogue/commit/b2991eb))
- chore(deps): bump chrono-english from 0.1.8 to 0.2.0 (#233) ([4c2a482](https://github.com/unhappychoice/gitlogue/commit/4c2a482))
- docs: count Godot as one supported language ([afe9928](https://github.com/unhappychoice/gitlogue/commit/afe9928))
- docs: clarify supported language-family count ([5ad3092](https://github.com/unhappychoice/gitlogue/commit/5ad3092))
- chore(deps): bump tree-sitter-scala from 0.26.0 to 0.26.2 ([2a4fd89](https://github.com/unhappychoice/gitlogue/commit/2a4fd89))
- chore(ci): use personal git identity for automated commits ([8280084](https://github.com/unhappychoice/gitlogue/commit/8280084))
- chore(deps): bump globset from 0.4.19 to 0.4.20 ([6e47f16](https://github.com/unhappychoice/gitlogue/commit/6e47f16))
- chore(deps): bump toml from 1.1.3+spec-1.1.0 to 1.1.4+spec-1.1.0 ([a282dd3](https://github.com/unhappychoice/gitlogue/commit/a282dd3))
- chore(deps): bump tree-sitter-erlang from 0.19.0 to 0.20.0 ([1fd9569](https://github.com/unhappychoice/gitlogue/commit/1fd9569))
- chore: update flake.nix hashes for v0.10.0 ([3fb5c9a](https://github.com/unhappychoice/gitlogue/commit/3fb5c9a))


## [0.10.0] - 2026-07-25

### ✨ Features

- feat(git): detect renames in diff extraction and preserve old_path ([782e28f](https://github.com/unhappychoice/gitlogue/commit/782e28f))

### 🐛 Bug Fixes

- fix(ci): bump cachix/install-nix-action to v31 in release.yml ([f9c046f](https://github.com/unhappychoice/gitlogue/commit/f9c046f))
- fix(ci): bump cachix/install-nix-action from v27 to v31 ([d52afc8](https://github.com/unhappychoice/gitlogue/commit/d52afc8))
- fix(ci): bump rust-cache prefix-key to invalidate stale cache ([e2e1fbf](https://github.com/unhappychoice/gitlogue/commit/e2e1fbf))
- fix: address CodeRabbit review findings on rename detection and ignore handling ([2f50404](https://github.com/unhappychoice/gitlogue/commit/2f50404))

### 📝 Other Changes

- chore: bump version to v0.10.0 ([28a2f04](https://github.com/unhappychoice/gitlogue/commit/28a2f04))
- chore(deps): bump clap from 4.6.2 to 4.6.3 ([cb16226](https://github.com/unhappychoice/gitlogue/commit/cb16226))
- chore(deps): bump serde from 1.0.228 to 1.0.229 ([bfa2e8f](https://github.com/unhappychoice/gitlogue/commit/bfa2e8f))
- chore(deps): bump anyhow from 1.0.103 to 1.0.104 ([59a5273](https://github.com/unhappychoice/gitlogue/commit/59a5273))
- chore(deps): bump globset from 0.4.18 to 0.4.19 ([3c39bb3](https://github.com/unhappychoice/gitlogue/commit/3c39bb3))
- chore(deps): bump clap from 4.6.1 to 4.6.2 ([4cd1f8c](https://github.com/unhappychoice/gitlogue/commit/4cd1f8c))
- chore(deps): bump toml_edit ([8f39d64](https://github.com/unhappychoice/gitlogue/commit/8f39d64))
- chore(deps): bump toml from 1.1.2+spec-1.1.0 to 1.1.3+spec-1.1.0 ([8506de7](https://github.com/unhappychoice/gitlogue/commit/8506de7))
- chore(deps): bump rand from 0.10.1 to 0.10.2 ([a7bfa2e](https://github.com/unhappychoice/gitlogue/commit/a7bfa2e))
- chore(deps): bump anyhow from 1.0.102 to 1.0.103 ([b800f56](https://github.com/unhappychoice/gitlogue/commit/b800f56))
- chore(deps): bump ratatui from 0.30.1 to 0.30.2 ([127f59a](https://github.com/unhappychoice/gitlogue/commit/127f59a))
- chore(deps): bump ratatui from 0.30.0 to 0.30.1 ([450f68d](https://github.com/unhappychoice/gitlogue/commit/450f68d))
- chore(deps): bump tree-sitter-erlang from 0.18.0 to 0.19.0 ([09d4546](https://github.com/unhappychoice/gitlogue/commit/09d4546))
- chore(deps): bump chrono from 0.4.44 to 0.4.45 ([b885f16](https://github.com/unhappychoice/gitlogue/commit/b885f16))
- chore(deps): bump tree-sitter-swift from 0.7.2 to 0.7.3 ([c055f30](https://github.com/unhappychoice/gitlogue/commit/c055f30))
- chore(deps): bump toml_edit ([99d5aa3](https://github.com/unhappychoice/gitlogue/commit/99d5aa3))
- chore(deps): bump tree-sitter-erlang from 0.17.0 to 0.18.0 ([4029ebc](https://github.com/unhappychoice/gitlogue/commit/4029ebc))
- chore(deps): bump git2 from 0.20.4 to 0.21.0 ([43ac447](https://github.com/unhappychoice/gitlogue/commit/43ac447))
- docs: add HelloGitHub featured badge ([e0642e1](https://github.com/unhappychoice/gitlogue/commit/e0642e1))
- chore(deps): bump tree-sitter-erlang from 0.16.0 to 0.17.0 ([a4a8066](https://github.com/unhappychoice/gitlogue/commit/a4a8066))
- docs: add Discord and blog mentions to Support section ([c525869](https://github.com/unhappychoice/gitlogue/commit/c525869))
- docs: add general hashtags to share links ([2bb9984](https://github.com/unhappychoice/gitlogue/commit/2bb9984))
- docs: add Support section with star and share links to README ([bd3b1c9](https://github.com/unhappychoice/gitlogue/commit/bd3b1c9))
- ci: merge test job into coverage run ([50321ea](https://github.com/unhappychoice/gitlogue/commit/50321ea))
- ci: add codecov coverage measurement ([4ab7bef](https://github.com/unhappychoice/gitlogue/commit/4ab7bef))
- chore(deps): bump tree-sitter-swift from 0.7.1 to 0.7.2 ([74e94cf](https://github.com/unhappychoice/gitlogue/commit/74e94cf))
- docs(readme): add hero section with centered demo, badge row, and tagline ([460fa22](https://github.com/unhappychoice/gitlogue/commit/460fa22))
- chore: remove accidentally committed llvm-cov report file ([914462b](https://github.com/unhappychoice/gitlogue/commit/914462b))
- refactor(config): replace HOME env mutation in TempHome with injected override ([0d5c3e4](https://github.com/unhappychoice/gitlogue/commit/0d5c3e4))
- test: address Clippy lint and version drift in About overlay assertion ([d49df10](https://github.com/unhappychoice/gitlogue/commit/d49df10))
- refactor(syntax): treat unreachable defensive branches in bundled tree-sitter setup as internal invariants ([3a8dbcf](https://github.com/unhappychoice/gitlogue/commit/3a8dbcf))
- refactor(ui): restructure event loop for direct unit testing and add branch tests ([7d39094](https://github.com/unhappychoice/gitlogue/commit/7d39094))
- refactor(ui): extract external signal handling into testable helpers and add regression tests ([63bc1d9](https://github.com/unhappychoice/gitlogue/commit/63bc1d9))
- test(syntax): cover two failure branches in syntax/mod.rs ([ac29450](https://github.com/unhappychoice/gitlogue/commit/ac29450))
- refactor(ui): extract terminal restore and external signal shutdown into testable helpers ([c122991](https://github.com/unhappychoice/gitlogue/commit/c122991))
- test(cli): add integration test for invalid theme CLI error case ([9e2c80d](https://github.com/unhappychoice/gitlogue/commit/9e2c80d))
- test(git): cover date filter miss and empty effective commit range error branches ([f468d34](https://github.com/unhappychoice/gitlogue/commit/f468d34))
- test(syntax): cover non-UTF-8 extension guard in get_language ([c4e4234](https://github.com/unhappychoice/gitlogue/commit/c4e4234))
- refactor(ui): extract run loop helpers for redraw, signal flag, and event dispatch with unit tests ([7c7429f](https://github.com/unhappychoice/gitlogue/commit/7c7429f))
- test(cli): add PTY-based integration tests for normal and staged-diff playback shutdown ([07668ea](https://github.com/unhappychoice/gitlogue/commit/07668ea))
- test(animation): fill AnimationEngine boundary branches and clean up unreachable ones ([d570a26](https://github.com/unhappychoice/gitlogue/commit/d570a26))
- test(git): add regression tests for add/delete commits and excluded lockfile diffs ([2e41a23](https://github.com/unhappychoice/gitlogue/commit/2e41a23))
- test(git): add regression tests for large-diff commit and working-tree boundary cases ([97f9d9f](https://github.com/unhappychoice/gitlogue/commit/97f9d9f))
- refactor(syntax): tidy unreachable language registry fallback into enum and cover SelectableParagraph zero-size render ([59c1771](https://github.com/unhappychoice/gitlogue/commit/59c1771))
- test(panes,widgets): cover unreached render branches ([fba60e6](https://github.com/unhappychoice/gitlogue/commit/fba60e6))
- test(git): cover defensive branches and open-ended commit range ([3473841](https://github.com/unhappychoice/gitlogue/commit/3473841))
- test(ui): add tests for pure state transition helpers ([ec11314](https://github.com/unhappychoice/gitlogue/commit/ec11314))
- test(syntax): cover token mapping and injection guard paths in highlighter ([bb14b87](https://github.com/unhappychoice/gitlogue/commit/bb14b87))
- test(cli): cover normal and staged-diff playback startup paths via non-interactive integration tests ([cb3a23c](https://github.com/unhappychoice/gitlogue/commit/cb3a23c))
- test(syntax): cover highlighter fallback and caching branches ([ba4942e](https://github.com/unhappychoice/gitlogue/commit/ba4942e))
- test(animation): directly test uncovered AnimationEngine helpers ([a6cb9c6](https://github.com/unhappychoice/gitlogue/commit/a6cb9c6))
- test(config): add regression tests for filesystem error paths ([4eddc44](https://github.com/unhappychoice/gitlogue/commit/4eddc44))
- test(main): extract normal playback prep into helpers and cover uncovered branches ([8caee7d](https://github.com/unhappychoice/gitlogue/commit/8caee7d))
- test(cli): cover non-interactive CLI branches with integration tests ([407bba6](https://github.com/unhappychoice/gitlogue/commit/407bba6))
- test(ui): extract playback state transitions into helpers and add regression tests ([deb44e9](https://github.com/unhappychoice/gitlogue/commit/deb44e9))
- test(config): cover compatibility, error paths, and empty array serialization ([74ae357](https://github.com/unhappychoice/gitlogue/commit/74ae357))
- test(ui): extract keyboard input branches into helpers and add regression tests ([4c3be65](https://github.com/unhappychoice/gitlogue/commit/4c3be65))
- test(main): extract diff subcommand prep into helper and cover staged/unstaged/empty diff ([952960b](https://github.com/unhappychoice/gitlogue/commit/952960b))
- refactor(main): make CLI/runtime helpers and theme path testable ([4b8c82b](https://github.com/unhappychoice/gitlogue/commit/4b8c82b))
- test(widgets): cover SelectableParagraph boundary branches and clean up unreachable guards ([ad2e09d](https://github.com/unhappychoice/gitlogue/commit/ad2e09d))
- test(ui): add unit tests for uncovered UI branches ([e2b84f6](https://github.com/unhappychoice/gitlogue/commit/e2b84f6))
- test(animation): cover special branches and manual checkpoint in AnimationEngine ([ff8d76c](https://github.com/unhappychoice/gitlogue/commit/ff8d76c))
- refactor(main): extract CLI resolution into testable helpers and add unit tests ([95b3510](https://github.com/unhappychoice/gitlogue/commit/95b3510))
- test(git): add unit tests for commit history selection logic ([e5e8d7b](https://github.com/unhappychoice/gitlogue/commit/e5e8d7b))
- test(ui): add unit tests for state transitions and overlay rendering ([576e964](https://github.com/unhappychoice/gitlogue/commit/576e964))
- test(syntax): add unit tests for syntax highlighter core ([cc8de91](https://github.com/unhappychoice/gitlogue/commit/cc8de91))
- test(panes): add unit tests for EditorPane render and highlight ([927a544](https://github.com/unhappychoice/gitlogue/commit/927a544))
- test(panes): add render tests for StatusBarPane and TerminalPane ([9341abf](https://github.com/unhappychoice/gitlogue/commit/9341abf))
- test(panes): cover FileTreePane tree build, cache update, and selected-row render ([b6687cc](https://github.com/unhappychoice/gitlogue/commit/b6687cc))
- test(syntax): add unit tests for language resolution logic ([b811a7c](https://github.com/unhappychoice/gitlogue/commit/b811a7c))
- test(theme): add unit tests for theme resolution and transparent background ([ee674a2](https://github.com/unhappychoice/gitlogue/commit/ee674a2))
- test(widgets): cover SelectableParagraph wrap, dim, and render branches ([0e284e0](https://github.com/unhappychoice/gitlogue/commit/0e284e0))
- test(config): add save/load tests and fix loop setting round-trip ([94cef3a](https://github.com/unhappychoice/gitlogue/commit/94cef3a))
- test(animation): add unit tests for core logic in animation.rs ([2f1b642](https://github.com/unhappychoice/gitlogue/commit/2f1b642))
- chore(deps): bump tree-sitter-dart from 0.1.0 to 0.2.0 ([39ddfa4](https://github.com/unhappychoice/gitlogue/commit/39ddfa4))
- chore(deps): bump tree-sitter-erlang from 0.15.0 to 0.16.0 ([2bcf755](https://github.com/unhappychoice/gitlogue/commit/2bcf755))
- chore(deps): bump tree-sitter-c from 0.24.1 to 0.24.2 ([099dc82](https://github.com/unhappychoice/gitlogue/commit/099dc82))
- chore(deps): bump clap from 4.6.0 to 4.6.1 ([70dbfa2](https://github.com/unhappychoice/gitlogue/commit/70dbfa2))
- chore(deps): bump tree-sitter-scala from 0.25.0 to 0.26.0 ([82f92b8](https://github.com/unhappychoice/gitlogue/commit/82f92b8))
- chore: update flake.nix hashes for v0.9.0 ([7800337](https://github.com/unhappychoice/gitlogue/commit/7800337))


## [0.9.0] - 2026-04-20

### ✨ Features

- feat: add Nix syntax highlighting support ([025aeb6](https://github.com/unhappychoice/gitlogue/commit/025aeb6))
- feat(syntax): support tree-sitter language injections (#192) ([e76953e](https://github.com/unhappychoice/gitlogue/commit/e76953e))
- feat: add Astro syntax highlighting support (#190) ([f309314](https://github.com/unhappychoice/gitlogue/commit/f309314))

### 🐛 Bug Fixes

- fix(animation): collapse nested if into match arm guard ([431c029](https://github.com/unhappychoice/gitlogue/commit/431c029))
- fix(syntax): update dart language loading for tree-sitter-dart 0.1.0 ([5b7f03b](https://github.com/unhappychoice/gitlogue/commit/5b7f03b))

### 📝 Other Changes

- chore: bump version to v0.9.0 ([bbbf7ac](https://github.com/unhappychoice/gitlogue/commit/bbbf7ac))
- chore(deps): bump tree-sitter-c-sharp from 0.23.1 to 0.23.5 ([31c5efb](https://github.com/unhappychoice/gitlogue/commit/31c5efb))
- chore(deps): bump rand from 0.10.0 to 0.10.1 ([58f6d85](https://github.com/unhappychoice/gitlogue/commit/58f6d85))
- chore(deps): bump toml_edit ([de7786f](https://github.com/unhappychoice/gitlogue/commit/de7786f))
- chore(deps): bump toml_edit from 0.25.9+spec-1.1.0 to 0.25.10+spec-1.1.0 ([c7a4a91](https://github.com/unhappychoice/gitlogue/commit/c7a4a91))
- chore(deps): bump toml_edit from 0.25.6+spec-1.1.0 to 0.25.9+spec-1.1.0 ([ac2e65c](https://github.com/unhappychoice/gitlogue/commit/ac2e65c))
- chore(deps): bump toml from 1.1.1+spec-1.1.0 to 1.1.2+spec-1.1.0 ([ec38319](https://github.com/unhappychoice/gitlogue/commit/ec38319))
- chore(deps): bump toml from 1.1.0+spec-1.1.0 to 1.1.1+spec-1.1.0 ([b2a3b3e](https://github.com/unhappychoice/gitlogue/commit/b2a3b3e))
- chore(deps): bump tree-sitter-rust from 0.24.1 to 0.24.2 ([0598f62](https://github.com/unhappychoice/gitlogue/commit/0598f62))
- docs: add author section with website link ([6a2af0c](https://github.com/unhappychoice/gitlogue/commit/6a2af0c))
- chore(deps): bump toml_edit from 0.25.5+spec-1.1.0 to 0.25.6+spec-1.1.0 ([659d1ce](https://github.com/unhappychoice/gitlogue/commit/659d1ce))
- chore(deps): bump toml from 1.0.7+spec-1.1.0 to 1.1.0+spec-1.1.0 ([7063ca2](https://github.com/unhappychoice/gitlogue/commit/7063ca2))
- chore(deps): bump tree-sitter-rust from 0.24.0 to 0.24.1 ([854441d](https://github.com/unhappychoice/gitlogue/commit/854441d))
- chore(deps): bump toml from 1.0.6+spec-1.1.0 to 1.0.7+spec-1.1.0 ([fc5e347](https://github.com/unhappychoice/gitlogue/commit/fc5e347))
- chore(deps): bump toml_edit from 0.25.4+spec-1.1.0 to 0.25.5+spec-1.1.0 ([fa2966d](https://github.com/unhappychoice/gitlogue/commit/fa2966d))
- chore(deps): bump clap from 4.5.60 to 4.6.0 ([de9f025](https://github.com/unhappychoice/gitlogue/commit/de9f025))
- chore(deps): bump tree-sitter-scala from 0.24.0 to 0.25.0 ([6092e78](https://github.com/unhappychoice/gitlogue/commit/6092e78))
- chore(deps): bump tree-sitter-dart from 0.0.4 to 0.1.0 ([4b3ca69](https://github.com/unhappychoice/gitlogue/commit/4b3ca69))
- chore(deps): bump toml from 1.0.4+spec-1.1.0 to 1.0.6+spec-1.1.0 ([5b24318](https://github.com/unhappychoice/gitlogue/commit/5b24318))
- chore(deps): bump toml from 1.0.3+spec-1.1.0 to 1.0.4+spec-1.1.0 ([70798a0](https://github.com/unhappychoice/gitlogue/commit/70798a0))
- chore(deps): bump toml_edit from 0.25.3+spec-1.1.0 to 0.25.4+spec-1.1.0 ([90e9666](https://github.com/unhappychoice/gitlogue/commit/90e9666))
- chore(deps): bump tree-sitter-elixir from 0.3.4 to 0.3.5 ([5a6c0d6](https://github.com/unhappychoice/gitlogue/commit/5a6c0d6))
- chore(deps): bump tree-sitter-lua from 0.4.1 to 0.5.0 ([a896219](https://github.com/unhappychoice/gitlogue/commit/a896219))
- chore(deps): bump tree-sitter-md from 0.5.2 to 0.5.3 ([fbabef0](https://github.com/unhappychoice/gitlogue/commit/fbabef0))
- chore(deps): bump chrono from 0.4.43 to 0.4.44 ([9ac6540](https://github.com/unhappychoice/gitlogue/commit/9ac6540))
- chore(deps): bump anyhow from 1.0.101 to 1.0.102 ([a001134](https://github.com/unhappychoice/gitlogue/commit/a001134))
- chore(deps): bump clap from 4.5.59 to 4.5.60 ([2b1c679](https://github.com/unhappychoice/gitlogue/commit/2b1c679))
- chore(deps): bump toml_edit from 0.25.2+spec-1.1.0 to 0.25.3+spec-1.1.0 ([b0363b6](https://github.com/unhappychoice/gitlogue/commit/b0363b6))
- chore(deps): bump toml from 1.0.2+spec-1.1.0 to 1.0.3+spec-1.1.0 ([3204a68](https://github.com/unhappychoice/gitlogue/commit/3204a68))
- chore(deps): bump toml_edit from 0.25.1+spec-1.1.0 to 0.25.2+spec-1.1.0 ([6c7976f](https://github.com/unhappychoice/gitlogue/commit/6c7976f))
- chore(deps): bump toml from 1.0.1+spec-1.1.0 to 1.0.2+spec-1.1.0 ([7155da2](https://github.com/unhappychoice/gitlogue/commit/7155da2))
- chore(deps): bump clap from 4.5.58 to 4.5.59 ([8a66e7f](https://github.com/unhappychoice/gitlogue/commit/8a66e7f))
- chore(deps): bump toml from 1.0.0+spec-1.1.0 to 1.0.1+spec-1.1.0 ([5a65afb](https://github.com/unhappychoice/gitlogue/commit/5a65afb))
- chore(deps): bump toml_edit from 0.25.0+spec-1.1.0 to 0.25.1+spec-1.1.0 ([a489869](https://github.com/unhappychoice/gitlogue/commit/a489869))
- docs: add key bindings section to README ([df36cc4](https://github.com/unhappychoice/gitlogue/commit/df36cc4))
- chore: update flake.nix hashes for v0.8.0 ([9d20227](https://github.com/unhappychoice/gitlogue/commit/9d20227))


## [0.8.0] - 2026-02-12

### 🐛 Bug Fixes

- fix: address CodeRabbit review issues ([a8a9208](https://github.com/unhappychoice/gitlogue/commit/a8a9208))
- fix: clear dialog area before rendering to prevent text bleed-through ([75c1266](https://github.com/unhappychoice/gitlogue/commit/75c1266))

### 📝 Other Changes

- chore: bump version to v0.8.0 ([1cec5c2](https://github.com/unhappychoice/gitlogue/commit/1cec5c2))
- refactor: improve playback controls UX ([d8bff84](https://github.com/unhappychoice/gitlogue/commit/d8bff84))
- added playback controlls ([2506b99](https://github.com/unhappychoice/gitlogue/commit/2506b99))
- chore(deps): bump rand from 0.9.2 to 0.10.0 (#147) ([4c951d8](https://github.com/unhappychoice/gitlogue/commit/4c951d8))
- chore(deps): bump toml from 0.9.12+spec-1.1.0 to 1.0.0+spec-1.1.0 ([d6bdbfe](https://github.com/unhappychoice/gitlogue/commit/d6bdbfe))
- chore(deps): bump clap from 4.5.57 to 4.5.58 ([11a9b36](https://github.com/unhappychoice/gitlogue/commit/11a9b36))
- chore(deps): bump toml_edit from 0.24.1+spec-1.1.0 to 0.25.0+spec-1.1.0 ([c3e6793](https://github.com/unhappychoice/gitlogue/commit/c3e6793))
- chore(deps): bump toml from 0.9.11+spec-1.1.0 to 0.9.12+spec-1.1.0 ([c3b9514](https://github.com/unhappychoice/gitlogue/commit/c3b9514))
- chore(deps): bump toml_edit from 0.24.0+spec-1.1.0 to 0.24.1+spec-1.1.0 ([6100f9c](https://github.com/unhappychoice/gitlogue/commit/6100f9c))
- chore(deps): bump ctrlc from 3.5.1 to 3.5.2 ([8636309](https://github.com/unhappychoice/gitlogue/commit/8636309))
- chore(deps): bump anyhow from 1.0.100 to 1.0.101 ([74edf44](https://github.com/unhappychoice/gitlogue/commit/74edf44))
- chore(deps): bump clap from 4.5.56 to 4.5.57 ([a6cc996](https://github.com/unhappychoice/gitlogue/commit/a6cc996))
- chore(deps): bump git2 from 0.20.3 to 0.20.4 ([02689a7](https://github.com/unhappychoice/gitlogue/commit/02689a7))
- chore(deps): bump clap from 4.5.54 to 4.5.56 ([de1279e](https://github.com/unhappychoice/gitlogue/commit/de1279e))
- chore(deps): bump unicode-width from 0.2.0 to 0.2.2 ([e2b4adb](https://github.com/unhappychoice/gitlogue/commit/e2b4adb))
- chore: update flake.nix hashes for v0.7.0 ([9154fe1](https://github.com/unhappychoice/gitlogue/commit/9154fe1))


## [0.7.0] - 2026-01-16

### ✨ Features

- feat(themes): add fluorite theme ([48f0e11](https://github.com/unhappychoice/gitlogue/commit/48f0e11))
- feat: add uv.lock to the excluded files ([ca92dc0](https://github.com/unhappychoice/gitlogue/commit/ca92dc0))
- feat: add diff subcommand to visualize working tree changes (#123) ([7805317](https://github.com/unhappychoice/gitlogue/commit/7805317))

### 🐛 Bug Fixes

- fix(themes): apply cargo fmt and fix alphabetical order ([e5ab941](https://github.com/unhappychoice/gitlogue/commit/e5ab941))
- fix(themes): apply 'coderabbit' suggested style changes ([aa4545e](https://github.com/unhappychoice/gitlogue/commit/aa4545e))
- fix(themes): add missing comma ([d2bf885](https://github.com/unhappychoice/gitlogue/commit/d2bf885))

### 📝 Other Changes

- chore: bump version to v0.7.0 ([c7ff630](https://github.com/unhappychoice/gitlogue/commit/c7ff630))
- chore(deps): bump chrono from 0.4.42 to 0.4.43 ([7ec94ad](https://github.com/unhappychoice/gitlogue/commit/7ec94ad))
- chore(deps): bump toml from 0.9.10+spec-1.1.0 to 0.9.11+spec-1.1.0 ([b79396f](https://github.com/unhappychoice/gitlogue/commit/b79396f))
- chore(deps): bump tree-sitter-md from 0.5.1 to 0.5.2 ([45b031e](https://github.com/unhappychoice/gitlogue/commit/45b031e))
- chore(deps): bump clap from 4.5.53 to 4.5.54 ([12f034e](https://github.com/unhappychoice/gitlogue/commit/12f034e))
- chore(deps): bump tree-sitter-lua from 0.2.0 to 0.4.1 ([5db6d54](https://github.com/unhappychoice/gitlogue/commit/5db6d54))
- chore(deps): bump ratatui from 0.29.0 to 0.30.0 ([9de4c9b](https://github.com/unhappychoice/gitlogue/commit/9de4c9b))
- chore(deps): bump toml from 0.9.8 to 0.9.10+spec-1.1.0 ([a033d22](https://github.com/unhappychoice/gitlogue/commit/a033d22))
- chore(deps): bump toml_edit from 0.23.9 to 0.24.0+spec-1.1.0 ([a4a6e9c](https://github.com/unhappychoice/gitlogue/commit/a4a6e9c))
- chore: update flake.nix hashes for v0.6.0 ([7545577](https://github.com/unhappychoice/gitlogue/commit/7545577))


## [0.6.0] - 2025-12-08

### ✨ Features

- feat(theme): add Telemetry theme ([2270d72](https://github.com/unhappychoice/gitlogue/commit/2270d72))
- feat: add speed_rules support in config file ([5f1621f](https://github.com/unhappychoice/gitlogue/commit/5f1621f))
- feat: add file-specific speed rules with --speed-rule option ([d8ba787](https://github.com/unhappychoice/gitlogue/commit/d8ba787))

### 🐛 Bug Fixes

- fix: allow sub-base-speed pauses to work correctly ([5e3a6ef](https://github.com/unhappychoice/gitlogue/commit/5e3a6ef))
- fix: apply speed rules to all pause durations ([4a008e6](https://github.com/unhappychoice/gitlogue/commit/4a008e6))

### 📝 Other Changes

- chore: bump version to v0.6.0 ([4e065c1](https://github.com/unhappychoice/gitlogue/commit/4e065c1))
- docs: add telemetry theme to themes documentation ([bb5ebbe](https://github.com/unhappychoice/gitlogue/commit/bb5ebbe))
- chore(deps): bump git2 from 0.20.2 to 0.20.3 ([28cb068](https://github.com/unhappychoice/gitlogue/commit/28cb068))
- chore(deps): bump toml_edit from 0.23.7 to 0.23.9 ([d12df6a](https://github.com/unhappychoice/gitlogue/commit/d12df6a))
- style: fix fmt and clippy warnings ([1d594d6](https://github.com/unhappychoice/gitlogue/commit/1d594d6))
- docs: add speed-rule documentation ([63092ce](https://github.com/unhappychoice/gitlogue/commit/63092ce))
- chore(deps): bump tree-sitter-bash from 0.25.0 to 0.25.1 ([ce09a6a](https://github.com/unhappychoice/gitlogue/commit/ce09a6a))
- docs: add docstrings to improve coverage ([4007761](https://github.com/unhappychoice/gitlogue/commit/4007761))
- chore: update flake.nix hashes for v0.5.0 ([8e7173b](https://github.com/unhappychoice/gitlogue/commit/8e7173b))


## [0.5.0] - 2025-11-30

### ✨ Features

- feat: add Svelte syntax highlighting support ([3655e09](https://github.com/unhappychoice/gitlogue/commit/3655e09))
- feat: add Lua syntax highlighting support ([76e7c6a](https://github.com/unhappychoice/gitlogue/commit/76e7c6a))
- feat: add date filtering options for commit history ([da7fc17](https://github.com/unhappychoice/gitlogue/commit/da7fc17))

### 🐛 Bug Fixes

- fix: invalidate old cache by changing prefix-key ([52e803c](https://github.com/unhappychoice/gitlogue/commit/52e803c))
- fix: update glibc version requirement to 2.35 ([c67e5a1](https://github.com/unhappychoice/gitlogue/commit/c67e5a1))
- fix: use ubuntu-22.04 for Linux builds and remove cache from release ([90f7ae8](https://github.com/unhappychoice/gitlogue/commit/90f7ae8))
- fix: update fzf and VHS examples for better compatibility (#109) ([84f77c5](https://github.com/unhappychoice/gitlogue/commit/84f77c5))
- fix: return error instead of fallback for invalid commit timestamp ([dde2914](https://github.com/unhappychoice/gitlogue/commit/dde2914))
- fix(docs): correct fzf integration example ([ca748da](https://github.com/unhappychoice/gitlogue/commit/ca748da))
- fix: Correct date filter comparison logic in matches_date_filter function to be fully identical to Git ([c643eea](https://github.com/unhappychoice/gitlogue/commit/c643eea))
- fix: Fix format error ([de3fb5d](https://github.com/unhappychoice/gitlogue/commit/de3fb5d))

### 📝 Other Changes

- chore: bump version to v0.5.0 ([907efec](https://github.com/unhappychoice/gitlogue/commit/907efec))
- docs: update supported languages list ([6708fe9](https://github.com/unhappychoice/gitlogue/commit/6708fe9))
- Update README to reflect new brew instructions (#111) ([bcb19f8](https://github.com/unhappychoice/gitlogue/commit/bcb19f8))
- refactor: sort use statements consistently across files ([6f212b9](https://github.com/unhappychoice/gitlogue/commit/6f212b9))
- refactor: sort TokenType enum and match arms alphabetically ([01db07e](https://github.com/unhappychoice/gitlogue/commit/01db07e))
- docs: expand Nix installation instructions ([d63814a](https://github.com/unhappychoice/gitlogue/commit/d63814a))
- ci: add flake.lock and update release workflow to maintain it ([3dac4ec](https://github.com/unhappychoice/gitlogue/commit/3dac4ec))
- docs: add Nix installation method ([eebf770](https://github.com/unhappychoice/gitlogue/commit/eebf770))
- chore: update flake.nix hashes for v0.4.1 ([ebdf0ee](https://github.com/unhappychoice/gitlogue/commit/ebdf0ee))


## [0.4.1] - 2025-11-25

### ✨ Features

- feat: add glibc version check to install script ([6cfece8](https://github.com/unhappychoice/gitlogue/commit/6cfece8))

### 🐛 Bug Fixes

- fix: use ubuntu-latest for Linux release builds ([2eb6d85](https://github.com/unhappychoice/gitlogue/commit/2eb6d85))

### 📝 Other Changes

- chore: bump version to v0.4.1 ([2ff5ace](https://github.com/unhappychoice/gitlogue/commit/2ff5ace))
- docs: add troubleshooting for glibc version errors ([77553a3](https://github.com/unhappychoice/gitlogue/commit/77553a3))
- chore: update flake.nix hashes for v0.4.0 ([5467d70](https://github.com/unhappychoice/gitlogue/commit/5467d70))


## [0.4.0] - 2025-11-25

### ✨ Features

- feat: add author filter option for commit display ([cfda74d](https://github.com/unhappychoice/gitlogue/commit/cfda74d))
- feat: move cursor to first non-whitespace position during scroll ([8cf6a46](https://github.com/unhappychoice/gitlogue/commit/8cf6a46))
- feat: skip cursor movement to indentation ([9ef01e2](https://github.com/unhappychoice/gitlogue/commit/9ef01e2))
- feat: introduced the pattern matching for ignoring parameters ([5350424](https://github.com/unhappychoice/gitlogue/commit/5350424))
- feat: add MODULE.bazel.lock to excluded files ([420c57a](https://github.com/unhappychoice/gitlogue/commit/420c57a))

### 🐛 Bug Fixes

- fix: default to asc order when --author is specified ([72ebcee](https://github.com/unhappychoice/gitlogue/commit/72ebcee))
- fix: add validation for author filter input to prevent empty patterns ([80e54c0](https://github.com/unhappychoice/gitlogue/commit/80e54c0))
- fix: add perl to nativeBuildInputs for openssl-sys build ([f3d7672](https://github.com/unhappychoice/gitlogue/commit/f3d7672))
- fix: update cli name and version ([7a07511](https://github.com/unhappychoice/gitlogue/commit/7a07511))
- fix: add other typescript extensions (#84) ([a2a614d](https://github.com/unhappychoice/gitlogue/commit/a2a614d))

### 📝 Other Changes

- chore: bump version to v0.4.0 ([868bc89](https://github.com/unhappychoice/gitlogue/commit/868bc89))
- docs: add behavior notes for --author filtering ([12cf5a1](https://github.com/unhappychoice/gitlogue/commit/12cf5a1))
- refactor: extract magic numbers and fix step discontinuity ([e1624aa](https://github.com/unhappychoice/gitlogue/commit/e1624aa))
- perf: improve scrolling speed in large files ([f39f158](https://github.com/unhappychoice/gitlogue/commit/f39f158))
- Use last supported LTS for glibc version in Linux builds & use arm runners (#98) ([cb45144](https://github.com/unhappychoice/gitlogue/commit/cb45144))
- refactor: use Cargo.toml version in CLI ([e330c16](https://github.com/unhappychoice/gitlogue/commit/e330c16))
- created a flake for nixos users (#92) ([3f24f29](https://github.com/unhappychoice/gitlogue/commit/3f24f29))
- test: update ignore patterns test to use SVG instead of PNG ([2c83a6f](https://github.com/unhappychoice/gitlogue/commit/2c83a6f))
- docs: add documentation for ignore patterns feature ([caa25ef](https://github.com/unhappychoice/gitlogue/commit/caa25ef))
- test: verifying working omission of patterns ([25a260b](https://github.com/unhappychoice/gitlogue/commit/25a260b))
- chore: adding dependency of package ([b122734](https://github.com/unhappychoice/gitlogue/commit/b122734))
- updating the dependency and config ([6a6d8d5](https://github.com/unhappychoice/gitlogue/commit/6a6d8d5))


## [0.3.0] - 2025-11-20

### ✨ Features

- feat: support --order option with commit ranges ([89da1c5](https://github.com/unhappychoice/gitlogue/commit/89da1c5))
- feat: add commit range option ([96f6b4a](https://github.com/unhappychoice/gitlogue/commit/96f6b4a))
- feat: enable SIGTERM and SIGHUP handling in ctrlc crate ([64f170c](https://github.com/unhappychoice/gitlogue/commit/64f170c))
- feat: add Ctrl+C and q key support for quitting application ([78cb6d2](https://github.com/unhappychoice/gitlogue/commit/78cb6d2))

### 🐛 Bug Fixes

- fix: detect git repository from subdirectories ([5ee8605](https://github.com/unhappychoice/gitlogue/commit/5ee8605))

### 📝 Other Changes

- chore: bump version to v0.3.0 ([1f68771](https://github.com/unhappychoice/gitlogue/commit/1f68771))
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


