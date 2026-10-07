mod animation;
mod config;
mod git;
mod panes;
mod syntax;
mod theme;
mod ui;
mod watch;
mod widgets;

use animation::SpeedRule;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use config::Config;
use git::{DiffMode, GitRepository};
use std::path::{Path, PathBuf};
use theme::Theme;
use ui::UI;
use watch::CommitWatcher;

/// Defines the order in which commits are played back during animation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum PlaybackOrder {
    #[default]
    Random,
    Asc,
    Desc,
}

#[derive(Parser, Debug)]
#[command(
    name = "gitlogue",
    version,
    about = "A Git history screensaver - watch your code rewrite itself",
    long_about = "gitlogue is a terminal-based screensaver that replays Git commits as if a ghost developer were typing each change by hand. Characters appear, vanish, and transform with natural pacing and syntax highlighting."
)]
pub struct Args {
    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "Path to Git repository (defaults to current directory)"
    )]
    pub path: Option<PathBuf>,

    #[arg(
        short,
        long,
        value_name = "HASH_OR_RANGE",
        help = "Replay a specific commit or commit range (e.g., HEAD~5..HEAD or abc123..)"
    )]
    pub commit: Option<String>,

    #[arg(
        short,
        long,
        value_name = "MS",
        help = "Typing speed in milliseconds per character (overrides config file)"
    )]
    pub speed: Option<u64>,

    #[arg(
        short,
        long,
        value_name = "NAME",
        help = "Theme to use (overrides config file)"
    )]
    pub theme: Option<String>,

    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        value_name = "BOOL",
        help = "Show background colors (use --background=false for transparent background, overrides config file)"
    )]
    pub background: Option<bool>,

    #[arg(
        long,
        value_enum,
        value_name = "ORDER",
        help = "Commit playback order (overrides config file)"
    )]
    pub order: Option<PlaybackOrder>,

    #[arg(
        long = "loop",
        num_args = 0..=1,
        default_missing_value = "true",
        value_name = "BOOL",
        help = "Loop the animation continuously (useful with --commit for commit ranges)"
    )]
    pub loop_playback: Option<bool>,

    #[arg(
        short = 'w',
        long,
        conflicts_with_all = ["commit", "order", "loop_playback", "author", "before", "after"],
        help = "Watch the repository and replay new commits as they are made"
    )]
    pub watch: bool,

    #[arg(long, help = "Display third-party license information")]
    pub license: bool,

    #[arg(
        short = 'a',
        long,
        value_name = "PATTERN",
        value_parser = |s: &str| if s.trim().is_empty() {
            Err("Author pattern cannot be empty".to_string())
        } else {
            Ok(s.to_string())
        },
        help = "Filter commits by author name or email (partial match, case-insensitive)"
    )]
    pub author: Option<String>,

    #[arg(
        long,
        value_name = "DATE",
        help = "Show commits before this date (e.g., '2024-01-01', '1 week ago', 'yesterday')"
    )]
    pub before: Option<String>,

    #[arg(
        long,
        value_name = "DATE",
        help = "Show commits after this date (e.g., '2024-01-01', '1 week ago', 'yesterday')"
    )]
    pub after: Option<String>,

    #[arg(
        short = 'i',
        long = "ignore",
        value_name = "PATTERN",
        action = clap::ArgAction::Append,
        help = "Ignore files matching pattern (gitignore syntax, can be specified multiple times)"
    )]
    pub ignore: Vec<String>,

    #[arg(
        long = "ignore-file",
        value_name = "PATH",
        help = "Path to file containing ignore patterns (one per line, like .gitignore)"
    )]
    pub ignore_file: Option<PathBuf>,

    #[arg(
        long = "speed-rule",
        value_name = "PATTERN:MS",
        action = clap::ArgAction::Append,
        help = "Set typing speed for files matching pattern (e.g., '*.java:50', '*.xml:5'). Can be specified multiple times."
    )]
    pub speed_rule: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Theme management commands
    Theme {
        #[command(subcommand)]
        command: ThemeCommands,
    },
    /// Show staged working tree changes (use --unstaged for unstaged changes)
    Diff {
        #[arg(long, help = "Show unstaged changes instead of staged")]
        unstaged: bool,

        #[arg(
            short,
            long,
            value_name = "MS",
            help = "Typing speed in milliseconds per character"
        )]
        speed: Option<u64>,

        #[arg(short, long, value_name = "NAME", help = "Theme to use")]
        theme: Option<String>,

        #[arg(long, num_args = 0..=1, default_missing_value = "true", value_name = "BOOL",
              help = "Show background colors (use --background=false for transparent)")]
        background: Option<bool>,

        #[arg(long = "loop", num_args = 0..=1, default_missing_value = "true", value_name = "BOOL",
              help = "Loop the animation continuously")]
        loop_playback: Option<bool>,

        #[arg(short = 'i', long = "ignore", value_name = "PATTERN", action = clap::ArgAction::Append,
              help = "Ignore files matching pattern (gitignore syntax)")]
        ignore: Vec<String>,

        #[arg(long = "speed-rule", value_name = "PATTERN:MS", action = clap::ArgAction::Append,
              help = "Set typing speed for files matching pattern (e.g., '*.java:50')")]
        speed_rule: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ThemeCommands {
    /// List all available themes
    List,
    /// Set default theme in config file
    Set {
        #[arg(value_name = "NAME", help = "Theme name to set as default")]
        name: String,
    },
}

impl Args {
    /// Validates the command-line arguments and returns the Git repository path.
    pub fn validate(&self) -> Result<PathBuf> {
        let start_path = self.path.clone().unwrap_or_else(|| PathBuf::from("."));

        if !start_path.exists() {
            anyhow::bail!("Path does not exist: {}", start_path.display());
        }

        let canonical_path = start_path
            .canonicalize()
            .context("Failed to resolve path")?;

        let repo_path = Self::find_git_root(&canonical_path).ok_or_else(|| {
            anyhow::anyhow!(
                "Not a Git repository: {} (or any parent directories)",
                start_path.display()
            )
        })?;

        Ok(repo_path)
    }

    fn find_git_root(start_path: &Path) -> Option<PathBuf> {
        let mut current = if start_path.is_file() {
            start_path.parent()?.to_path_buf()
        } else {
            start_path.to_path_buf()
        };

        loop {
            if current.join(".git").exists() {
                return Some(current);
            }
            if !current.pop() {
                return None;
            }
        }
    }
}

fn is_range_mode(commit: Option<&str>) -> bool {
    commit.is_some_and(|spec| spec.contains(".."))
}

fn resolve_order(
    cli_order: Option<PlaybackOrder>,
    config_order: &str,
    is_range_mode: bool,
    is_filtered: bool,
) -> PlaybackOrder {
    let order = cli_order.unwrap_or(match config_order {
        "asc" => PlaybackOrder::Asc,
        "desc" => PlaybackOrder::Desc,
        _ => PlaybackOrder::Random,
    });

    if (is_range_mode || is_filtered) && cli_order.is_none() {
        PlaybackOrder::Asc
    } else {
        order
    }
}

fn collect_ignore_patterns(
    config_patterns: &[String],
    ignore_file: Option<&Path>,
    cli_patterns: &[String],
) -> Result<Vec<String>> {
    let mut patterns = config_patterns.to_vec();

    if let Some(path) = ignore_file {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read ignore file: {}", path.display()))?;
        patterns.extend(
            content
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .map(String::from),
        );
    }

    patterns.extend(cli_patterns.iter().cloned());
    Ok(patterns)
}

fn parse_speed_rules(cli_rules: &[String], config_rules: &[String]) -> Vec<SpeedRule> {
    cli_rules
        .iter()
        .chain(config_rules.iter())
        .filter_map(|rule| {
            SpeedRule::parse(rule).or_else(|| {
                eprintln!("Warning: Invalid speed rule '{}', skipping", rule);
                None
            })
        })
        .collect()
}

fn load_initial_commit(
    repo: &GitRepository,
    commit: Option<&str>,
    order: PlaybackOrder,
) -> Result<git::CommitMetadata> {
    if is_range_mode(commit) {
        repo.set_commit_range(commit.expect("range mode requires a commit spec"))?;
        return match order {
            PlaybackOrder::Random => repo.random_range_commit(),
            PlaybackOrder::Asc => repo.next_range_commit_asc(),
            PlaybackOrder::Desc => repo.next_range_commit_desc(),
        };
    }

    if let Some(commit_hash) = commit {
        return repo.get_commit(commit_hash);
    }

    match order {
        PlaybackOrder::Random => repo.random_commit(),
        PlaybackOrder::Asc => repo.next_asc_commit(),
        PlaybackOrder::Desc => repo.next_desc_commit(),
    }
}

fn should_keep_repo_ref(
    is_range_mode: bool,
    is_filtered: bool,
    is_commit_specified: bool,
    loop_playback: bool,
) -> bool {
    is_range_mode || is_filtered || !is_commit_specified || loop_playback
}

struct RuntimeOptions {
    speed: u64,
    theme: Theme,
    loop_playback: bool,
    speed_rules: Vec<SpeedRule>,
}

struct DiffCommandOptions<'a> {
    unstaged: bool,
    speed: Option<u64>,
    theme: Option<&'a str>,
    background: Option<bool>,
    loop_playback: Option<bool>,
    ignore: &'a [String],
    speed_rule: &'a [String],
}

struct DiffPlaybackPlan {
    mode: DiffMode,
    metadata: git::CommitMetadata,
    runtime: RuntimeOptions,
}

struct CommitPlaybackPlan {
    metadata: git::CommitMetadata,
    runtime: RuntimeOptions,
    order: PlaybackOrder,
    is_range_mode: bool,
    keep_repo_ref: bool,
}

fn load_theme_with_background(
    cli_theme: Option<&str>,
    cli_background: Option<bool>,
    config: &Config,
) -> Result<Theme> {
    let theme = Theme::load(cli_theme.unwrap_or(&config.theme))?;

    Ok(if cli_background.unwrap_or(config.background) {
        theme
    } else {
        theme.with_transparent_background()
    })
}

fn resolve_runtime_options(
    cli_speed: Option<u64>,
    cli_theme: Option<&str>,
    cli_background: Option<bool>,
    cli_loop_playback: Option<bool>,
    cli_speed_rules: &[String],
    config: &Config,
    loop_default: bool,
) -> Result<RuntimeOptions> {
    Ok(RuntimeOptions {
        speed: cli_speed.unwrap_or(config.speed),
        theme: load_theme_with_background(cli_theme, cli_background, config)?,
        loop_playback: cli_loop_playback.unwrap_or(loop_default),
        speed_rules: parse_speed_rules(cli_speed_rules, &config.speed_rules),
    })
}

fn has_commit_filters(args: &Args) -> bool {
    args.author.is_some() || args.before.is_some() || args.after.is_some()
}

fn apply_commit_filters(repo: &mut GitRepository, args: &Args) -> Result<()> {
    if args.author.is_some() {
        repo.set_author_filter(args.author.clone());
    }
    if let Some(ref before_str) = args.before {
        repo.set_before_filter(Some(git::parse_date(before_str)?));
    }
    if let Some(ref after_str) = args.after {
        repo.set_after_filter(Some(git::parse_date(after_str)?));
    }
    Ok(())
}

fn initialize_ignore_patterns(patterns: &[String]) -> Result<()> {
    match git::init_ignore_patterns(patterns) {
        Err(error) if error.to_string() == "User patterns already initialized" => Ok(()),
        result => result,
    }
}

fn prepare_commit_playback(
    repo: &mut GitRepository,
    args: &Args,
    config: &Config,
) -> Result<CommitPlaybackPlan> {
    apply_commit_filters(repo, args)?;

    let is_range_mode = is_range_mode(args.commit.as_deref());
    let is_filtered = has_commit_filters(args);
    let patterns = collect_ignore_patterns(
        &config.ignore_patterns,
        args.ignore_file.as_deref(),
        &args.ignore,
    )?;
    initialize_ignore_patterns(&patterns)?;
    let order = resolve_order(args.order, &config.order, is_range_mode, is_filtered);
    let runtime = resolve_runtime_options(
        args.speed,
        args.theme.as_deref(),
        args.background,
        args.loop_playback,
        &args.speed_rule,
        config,
        config.loop_playback,
    )?;
    let metadata = load_initial_commit(repo, args.commit.as_deref(), order)?;
    let keep_repo_ref = should_keep_repo_ref(
        is_range_mode,
        is_filtered,
        args.commit.is_some(),
        runtime.loop_playback,
    );

    Ok(CommitPlaybackPlan {
        metadata,
        runtime,
        order,
        is_range_mode,
        keep_repo_ref,
    })
}

fn prepare_diff_playback(
    repo: &GitRepository,
    options: DiffCommandOptions<'_>,
    config: &Config,
) -> Result<Option<DiffPlaybackPlan>> {
    let mode = if options.unstaged {
        DiffMode::Unstaged
    } else {
        DiffMode::Staged
    };
    let patterns = collect_ignore_patterns(&config.ignore_patterns, None, options.ignore)?;
    initialize_ignore_patterns(&patterns)?;
    let metadata = repo.get_working_tree_diff(mode)?;
    if metadata.changes.is_empty() {
        return Ok(None);
    }
    let runtime = resolve_runtime_options(
        options.speed,
        options.theme,
        options.background,
        options.loop_playback,
        options.speed_rule,
        config,
        false,
    )?;
    Ok(Some(DiffPlaybackPlan {
        mode,
        metadata,
        runtime,
    }))
}

fn run_watch_mode(repo: &GitRepository, args: &Args, config: &Config) -> Result<()> {
    let patterns = collect_ignore_patterns(
        &config.ignore_patterns,
        args.ignore_file.as_deref(),
        &args.ignore,
    )?;
    initialize_ignore_patterns(&patterns)?;
    let RuntimeOptions {
        speed,
        theme,
        speed_rules,
        ..
    } = resolve_runtime_options(
        args.speed,
        args.theme.as_deref(),
        args.background,
        None,
        &args.speed_rule,
        config,
        false,
    )?;
    let mut ui = UI::new(
        speed,
        Some(repo),
        theme,
        PlaybackOrder::Asc,
        false,
        None,
        false,
        speed_rules,
    );
    ui.enable_watch(CommitWatcher::new(repo, std::time::Instant::now()));
    ui.run()
}

fn format_theme_list() -> String {
    std::iter::once("Available themes:".to_string())
        .chain(
            Theme::available_themes()
                .into_iter()
                .map(|theme| format!("  - {theme}")),
        )
        .collect::<Vec<_>>()
        .join("\n")
}

fn persist_theme_selection(name: &str) -> Result<String> {
    Theme::load(name)?;

    let mut config = Config::load().unwrap_or_default();
    config.theme = name.to_string();
    config.save()?;

    Config::config_path().map(|path| format!("Theme set to '{name}' in {}", path.display()))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle --license flag
    if args.license {
        println!("{}", include_str!("../LICENSE-THIRD-PARTY"));
        return Ok(());
    }

    // Handle subcommands
    if let Some(ref command) = args.command {
        match command {
            Commands::Theme { command } => match command {
                ThemeCommands::List => {
                    println!("{}", format_theme_list());
                    return Ok(());
                }
                ThemeCommands::Set { name } => {
                    println!("{}", persist_theme_selection(name)?);
                    return Ok(());
                }
            },
            Commands::Diff {
                unstaged,
                speed,
                theme,
                background,
                loop_playback,
                ignore,
                speed_rule,
            } => {
                let repo_path = args.validate()?;
                let repo = GitRepository::open(&repo_path)?;
                let config = Config::load()?;
                let Some(plan) = prepare_diff_playback(
                    &repo,
                    DiffCommandOptions {
                        unstaged: *unstaged,
                        speed: *speed,
                        theme: theme.as_deref(),
                        background: *background,
                        loop_playback: *loop_playback,
                        ignore,
                        speed_rule,
                    },
                    &config,
                )?
                else {
                    println!("No changes to display");
                    return Ok(());
                };
                let DiffPlaybackPlan {
                    mode,
                    metadata,
                    runtime,
                } = plan;
                let RuntimeOptions {
                    speed,
                    theme,
                    loop_playback,
                    speed_rules,
                } = runtime;

                // Create UI - pass repo ref only if looping (to refresh diff)
                let repo_ref = loop_playback.then_some(&repo);
                let mut ui = UI::new(
                    speed,
                    repo_ref,
                    theme,
                    PlaybackOrder::Asc,
                    loop_playback,
                    None,
                    false,
                    speed_rules,
                );
                ui.set_diff_mode(Some(mode));
                ui.load_commit(metadata);
                ui.run()?;

                return Ok(());
            }
        }
    }

    let repo_path = args.validate()?;
    let mut repo = GitRepository::open(&repo_path)?;
    let config = Config::load()?;
    if args.watch {
        return run_watch_mode(&repo, &args, &config);
    }
    let CommitPlaybackPlan {
        metadata,
        runtime,
        order,
        is_range_mode,
        keep_repo_ref,
    } = prepare_commit_playback(&mut repo, &args, &config)?;
    let RuntimeOptions {
        speed,
        theme,
        loop_playback,
        speed_rules,
    } = runtime;
    let repo_ref = keep_repo_ref.then_some(&repo);
    let mut ui = UI::new(
        speed,
        repo_ref,
        theme,
        order,
        loop_playback,
        args.commit.clone(),
        is_range_mode,
        speed_rules,
    );
    ui.load_commit(metadata);
    ui.run()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature, Time};
    use ratatui::style::Color;
    use std::env;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering as CounterOrdering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestRepo {
        path: PathBuf,
        repo: Repository,
    }

    struct TempHome {
        _override: config::test_home::Guard,
        path: PathBuf,
    }

    impl TestRepo {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique_id = format!(
                "{}_{}_{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                COUNTER.fetch_add(1, CounterOrdering::SeqCst)
            );
            let path = std::env::temp_dir().join(format!("gitlogue_main_test_{unique_id}"));
            fs::create_dir_all(&path).unwrap();

            let repo = Repository::init(&path).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test User").unwrap();
            config.set_str("user.email", "test@example.com").unwrap();

            Self { path, repo }
        }

        fn commit_file(
            &self,
            relative_path: &str,
            content: &str,
            message: &str,
            timestamp: i64,
        ) -> String {
            self.commit_file_with_author(
                relative_path,
                content,
                "Test User",
                "test@example.com",
                timestamp,
                message,
            )
        }

        fn commit_file_with_author(
            &self,
            relative_path: &str,
            content: &str,
            author_name: &str,
            author_email: &str,
            timestamp: i64,
            message: &str,
        ) -> String {
            let file_path = self.path.join(relative_path);
            file_path
                .parent()
                .map(fs::create_dir_all)
                .transpose()
                .unwrap();
            fs::write(&file_path, content).unwrap();

            let mut index = self.repo.index().unwrap();
            index.add_path(Path::new(relative_path)).unwrap();
            index.write().unwrap();

            let tree_id = index.write_tree().unwrap();
            let tree = self.repo.find_tree(tree_id).unwrap();
            let signature =
                Signature::new(author_name, author_email, &Time::new(timestamp, 0)).unwrap();
            let parent = self
                .repo
                .head()
                .ok()
                .and_then(|head| head.peel_to_commit().ok());

            let oid = match parent.as_ref() {
                Some(parent_commit) => self.repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[parent_commit],
                ),
                None => self
                    .repo
                    .commit(Some("HEAD"), &signature, &signature, message, &tree, &[]),
            }
            .unwrap();

            oid.to_string()
        }
    }

    impl TempHome {
        fn new() -> Result<Self> {
            let path = env::temp_dir().join(format!(
                "gitlogue-main-home-{}-{}",
                std::process::id(),
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
            ));

            fs::create_dir_all(&path)?;

            let _override = config::test_home::Guard::new(&path);
            Ok(Self { _override, path })
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    impl Drop for TempHome {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn sample_config() -> Config {
        Config {
            theme: "tokyo-night".to_string(),
            speed: 12,
            background: false,
            order: "desc".to_string(),
            loop_playback: true,
            ignore_patterns: vec!["dist/**".to_string()],
            speed_rules: vec!["*.md:40".to_string()],
        }
    }

    fn test_args(path: Option<PathBuf>) -> Args {
        Args {
            path,
            commit: None,
            speed: None,
            theme: None,
            background: None,
            order: None,
            loop_playback: None,
            watch: false,
            license: false,
            author: None,
            before: None,
            after: None,
            ignore: Vec::new(),
            ignore_file: None,
            speed_rule: Vec::new(),
            command: None,
        }
    }

    #[test]
    fn validate_resolves_git_root_from_nested_files() {
        let test_repo = TestRepo::new();
        let file_path = test_repo.path.join("src/main.rs");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, "fn main() {}\n").unwrap();

        let args = test_args(Some(file_path));

        assert_eq!(args.validate().unwrap(), test_repo.path);
    }

    #[test]
    fn validate_rejects_missing_and_non_repo_paths() {
        let missing = test_args(Some(std::env::temp_dir().join("gitlogue_missing_repo")));
        let missing_error = missing.validate().unwrap_err().to_string();
        assert!(missing_error.contains("Path does not exist"));

        let dir = TestRepo::new();
        let non_repo = dir.path.join("plain-dir");
        fs::create_dir_all(&non_repo).unwrap();
        fs::remove_dir_all(dir.path.join(".git")).unwrap();

        let error = test_args(Some(non_repo))
            .validate()
            .unwrap_err()
            .to_string();
        assert!(error.contains("Not a Git repository"));
    }

    #[test]
    fn resolve_order_defaults_filtered_modes_to_asc_without_overriding_cli() {
        assert_eq!(
            resolve_order(None, "desc", false, false),
            PlaybackOrder::Desc
        );
        assert_eq!(
            resolve_order(None, "random", true, false),
            PlaybackOrder::Asc
        );
        assert_eq!(
            resolve_order(Some(PlaybackOrder::Desc), "asc", true, true),
            PlaybackOrder::Desc
        );
    }

    #[test]
    fn collect_ignore_patterns_merges_config_file_and_cli_sources() {
        let test_repo = TestRepo::new();
        let ignore_file = test_repo.path.join(".gitlogueignore");
        fs::write(&ignore_file, "*.tmp\n\n# comment\ncoverage/**\n").unwrap();

        let patterns = collect_ignore_patterns(
            &["dist/**".to_string()],
            Some(ignore_file.as_path()),
            &["*.png".to_string()],
        )
        .unwrap();

        assert_eq!(
            patterns,
            vec![
                "dist/**".to_string(),
                "*.tmp".to_string(),
                "coverage/**".to_string(),
                "*.png".to_string()
            ]
        );
    }

    #[test]
    fn parse_speed_rules_keeps_valid_rules_in_priority_order() {
        let rules = parse_speed_rules(
            &["src/**/*.rs:5".to_string(), "invalid".to_string()],
            &["README.md:40".to_string()],
        );

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].speed_ms, 5);
        assert!(rules[0].matches("src/main.rs"));
        assert_eq!(rules[1].speed_ms, 40);
        assert!(rules[1].matches("README.md"));
    }

    #[test]
    fn load_initial_commit_supports_commit_specs_and_each_playback_order() {
        let test_repo = TestRepo::new();
        let oldest = test_repo.commit_file("history.txt", "one\n", "first", 1_700_000_000);
        let middle = test_repo.commit_file("history.txt", "two\n", "second", 1_700_000_100);
        let newest = test_repo.commit_file("history.txt", "three\n", "third", 1_700_000_200);

        assert_eq!(
            load_initial_commit(
                &GitRepository::open(&test_repo.path).unwrap(),
                None,
                PlaybackOrder::Asc,
            )
            .unwrap()
            .hash,
            oldest
        );
        assert_eq!(
            load_initial_commit(
                &GitRepository::open(&test_repo.path).unwrap(),
                Some(newest.as_str()),
                PlaybackOrder::Random,
            )
            .unwrap()
            .hash,
            newest
        );
        assert_eq!(
            load_initial_commit(
                &GitRepository::open(&test_repo.path).unwrap(),
                Some("HEAD~2..HEAD"),
                PlaybackOrder::Asc,
            )
            .unwrap()
            .hash,
            middle
        );
        assert_eq!(
            load_initial_commit(
                &GitRepository::open(&test_repo.path).unwrap(),
                Some("HEAD~2..HEAD"),
                PlaybackOrder::Desc,
            )
            .unwrap()
            .hash,
            newest
        );

        let single_commit_repo = TestRepo::new();
        let only = single_commit_repo.commit_file("solo.txt", "one\n", "only", 1_700_010_000);
        let single_repo = GitRepository::open(&single_commit_repo.path).unwrap();

        assert_eq!(
            load_initial_commit(&single_repo, None, PlaybackOrder::Random)
                .unwrap()
                .hash,
            only
        );
        assert_eq!(
            load_initial_commit(
                &GitRepository::open(&test_repo.path).unwrap(),
                None,
                PlaybackOrder::Desc,
            )
            .unwrap()
            .hash,
            newest
        );
        assert_eq!(
            load_initial_commit(
                &GitRepository::open(&test_repo.path).unwrap(),
                Some("HEAD~1..HEAD"),
                PlaybackOrder::Random,
            )
            .unwrap()
            .hash,
            newest
        );
    }

    #[test]
    fn should_keep_repo_ref_only_drops_one_shot_commit_playback() {
        assert!(should_keep_repo_ref(true, false, true, false));
        assert!(should_keep_repo_ref(false, true, true, false));
        assert!(should_keep_repo_ref(false, false, false, false));
        assert!(should_keep_repo_ref(false, false, true, true));
        assert!(!should_keep_repo_ref(false, false, true, false));
    }

    #[test]
    fn resolve_runtime_options_merges_cli_overrides_and_loop_defaults() {
        let config = sample_config();
        let diff_runtime =
            resolve_runtime_options(None, None, None, None, &[], &config, false).unwrap();

        assert_eq!(diff_runtime.speed, config.speed);
        assert!(!diff_runtime.loop_playback);
        assert_eq!(diff_runtime.theme.background_left, Color::Reset);
        assert_eq!(diff_runtime.theme.background_right, Color::Reset);
        assert_eq!(diff_runtime.speed_rules.len(), 1);
        assert!(diff_runtime.speed_rules[0].matches("README.md"));

        let cli_rules = vec!["src/**/*.rs:5".to_string(), "invalid".to_string()];
        let cli_runtime = resolve_runtime_options(
            Some(7),
            Some("nord"),
            Some(true),
            Some(true),
            &cli_rules,
            &config,
            false,
        )
        .unwrap();

        let nord = Theme::load("nord").unwrap();

        assert_eq!(cli_runtime.speed, 7);
        assert!(cli_runtime.loop_playback);
        assert_eq!(cli_runtime.theme.background_left, nord.background_left);
        assert_eq!(cli_runtime.theme.background_right, nord.background_right);
        assert_eq!(cli_runtime.speed_rules.len(), 2);
        assert!(cli_runtime.speed_rules[0].matches("src/main.rs"));
        assert!(cli_runtime.speed_rules[1].matches("README.md"));
    }

    #[test]
    fn format_theme_list_includes_header_and_every_theme() {
        let output = format_theme_list();

        assert!(output.starts_with("Available themes:"));
        assert!(Theme::available_themes()
            .into_iter()
            .all(|theme| output.contains(&format!("  - {theme}"))));
    }

    #[test]
    fn persist_theme_selection_saves_theme_under_temp_home() -> Result<()> {
        let temp_home = TempHome::new()?;
        let message = persist_theme_selection("nord")?;
        let saved = Config::load()?;

        assert_eq!(saved.theme, "nord");
        assert!(message.contains("Theme set to 'nord'"));
        assert!(message.contains(&temp_home.path.display().to_string()));

        Ok(())
    }

    #[test]
    fn prepare_commit_playback_applies_filters_and_forces_repo_iteration() -> Result<()> {
        let test_repo = TestRepo::new();
        test_repo.commit_file_with_author(
            "history.txt",
            "one\n",
            "Alice",
            "alice@example.com",
            1_704_067_200,
            "first",
        );
        let bob = test_repo.commit_file_with_author(
            "history.txt",
            "two\n",
            "Bob",
            "bob@example.com",
            1_704_153_600,
            "second",
        );
        let mut repo = GitRepository::open(&test_repo.path)?;
        let mut args = test_args(None);
        args.author = Some("BOB@EXAMPLE.COM".to_string());
        args.after = Some("2024-01-01T12:00:00Z".to_string());
        args.before = Some("2024-01-02T12:00:00Z".to_string());
        let mut config = sample_config();
        config.loop_playback = false;

        let plan = prepare_commit_playback(&mut repo, &args, &config)?;

        assert_eq!(plan.metadata.hash, bob);
        assert_eq!(plan.order, PlaybackOrder::Asc);
        assert!(!plan.is_range_mode);
        assert!(plan.keep_repo_ref);
        assert_eq!(plan.runtime.speed, 12);
        assert!(!plan.runtime.loop_playback);

        Ok(())
    }

    #[test]
    fn prepare_diff_playback_returns_none_for_clean_tree() -> Result<()> {
        let test_repo = TestRepo::new();
        test_repo.commit_file("src/lib.rs", "fn clean() {}\n", "clean", 1);
        let repo = GitRepository::open(&test_repo.path)?;
        let mut config = sample_config();
        config.ignore_patterns.clear();
        let empty = Vec::<String>::new();

        let plan = prepare_diff_playback(
            &repo,
            DiffCommandOptions {
                unstaged: false,
                speed: None,
                theme: None,
                background: None,
                loop_playback: None,
                ignore: &empty,
                speed_rule: &empty,
            },
            &config,
        )?;

        assert!(plan.is_none());

        Ok(())
    }

    #[test]
    fn prepare_diff_playback_reads_staged_changes_and_runtime_overrides() -> Result<()> {
        let test_repo = TestRepo::new();
        test_repo.commit_file("src/lib.rs", "fn clean() {}\n", "clean", 1);
        test_repo.commit_file("README.md", "before\n", "docs", 2);
        fs::write(test_repo.path.join("src/lib.rs"), "fn staged() {}\n")?;
        let mut index = test_repo.repo.index()?;
        index.add_path(Path::new("src/lib.rs"))?;
        index.write()?;
        let repo = GitRepository::open(&test_repo.path)?;
        let mut config = sample_config();
        config.ignore_patterns.clear();
        let cli_rules = vec!["src/**/*.rs:5".to_string()];
        let empty = Vec::<String>::new();
        let nord = Theme::load("nord")?;

        let plan = prepare_diff_playback(
            &repo,
            DiffCommandOptions {
                unstaged: false,
                speed: Some(7),
                theme: Some("nord"),
                background: Some(true),
                loop_playback: Some(true),
                ignore: &empty,
                speed_rule: &cli_rules,
            },
            &config,
        )?
        .expect("staged changes should produce a playback plan");

        assert_eq!(plan.mode, DiffMode::Staged);
        assert_eq!(plan.metadata.hash, "working-tree");
        assert_eq!(plan.metadata.message, "Staged changes");
        assert_eq!(plan.runtime.speed, 7);
        assert!(plan.runtime.loop_playback);
        assert_eq!(plan.runtime.theme.background_left, nord.background_left);
        assert_eq!(plan.runtime.theme.background_right, nord.background_right);
        assert_eq!(plan.runtime.speed_rules.len(), 2);
        assert!(plan.runtime.speed_rules[0].matches("src/main.rs"));
        assert!(plan.runtime.speed_rules[1].matches("README.md"));

        Ok(())
    }

    #[test]
    fn prepare_diff_playback_reads_unstaged_changes() -> Result<()> {
        let test_repo = TestRepo::new();
        test_repo.commit_file("src/lib.rs", "fn clean() {}\n", "clean", 1);
        fs::write(test_repo.path.join("src/lib.rs"), "fn unstaged() {}\n")?;
        let repo = GitRepository::open(&test_repo.path)?;
        let mut config = sample_config();
        config.ignore_patterns.clear();
        let empty = Vec::<String>::new();

        let plan = prepare_diff_playback(
            &repo,
            DiffCommandOptions {
                unstaged: true,
                speed: None,
                theme: None,
                background: None,
                loop_playback: None,
                ignore: &empty,
                speed_rule: &empty,
            },
            &config,
        )?
        .expect("unstaged changes should produce a playback plan");

        assert_eq!(plan.mode, DiffMode::Unstaged);
        assert_eq!(plan.metadata.hash, "working-tree");
        assert_eq!(plan.metadata.message, "Unstaged changes");
        assert!(!plan.runtime.loop_playback);

        Ok(())
    }

    #[test]
    fn args_parser_supports_bool_flags_and_rejects_blank_author() {
        let args = Args::try_parse_from([
            "gitlogue",
            "--background=false",
            "--loop",
            "--ignore",
            "*.tmp",
            "--ignore",
            "dist/**",
            "--speed-rule",
            "*.rs:5",
        ])
        .unwrap();

        assert_eq!(args.background, Some(false));
        assert_eq!(args.loop_playback, Some(true));
        assert_eq!(
            args.ignore,
            vec!["*.tmp".to_string(), "dist/**".to_string()]
        );
        assert_eq!(args.speed_rule, vec!["*.rs:5".to_string()]);

        let error = Args::try_parse_from(["gitlogue", "--author", "   "])
            .unwrap_err()
            .to_string();

        assert!(error.contains("Author pattern cannot be empty"));
    }

    #[test]
    fn watch_flag_parses_and_rejects_commit_selection_options() {
        let args = Args::try_parse_from(["gitlogue", "--watch", "--speed", "5"]).unwrap();
        assert!(args.watch);
        assert_eq!(args.speed, Some(5));

        let error = Args::try_parse_from(["gitlogue", "--watch", "--commit", "HEAD"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("cannot be used with"));
    }
}
