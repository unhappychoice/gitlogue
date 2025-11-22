mod animation;
mod config;
mod git;
mod panes;
mod syntax;
mod theme;
mod ui;
mod widgets;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use config::Config;
use git::GitRepository;
use std::path::{Path, PathBuf};
use theme::Theme;
use ui::UI;

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum PlaybackOrder {
    #[default]
    Random,
    Asc,
    Desc,
}

#[derive(Parser, Debug)]
#[command(
    name = "git-logue",
    version = "0.0.1",
    about = "A Git history screensaver - watch your code rewrite itself",
    long_about = "git-logue is a terminal-based screensaver that replays Git commits as if a ghost developer were typing each change by hand. Characters appear, vanish, and transform with natural pacing and syntax highlighting."
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

    #[arg(long, help = "Display third-party license information")]
    pub license: bool,

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

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle --license flag
    if args.license {
        println!("{}", include_str!("../LICENSE-THIRD-PARTY"));
        return Ok(());
    }

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Commands::Theme { command } => match command {
                ThemeCommands::List => {
                    println!("Available themes:");
                    for theme in Theme::available_themes() {
                        println!("  - {}", theme);
                    }
                    return Ok(());
                }
                ThemeCommands::Set { name } => {
                    // Validate theme exists
                    Theme::load(&name)?;

                    // Load existing config or create new one
                    let mut config = Config::load().unwrap_or_default();
                    config.theme = name.clone();
                    config.save()?;

                    let config_path = Config::config_path()?;
                    println!("Theme set to '{}' in {}", name, config_path.display());
                    return Ok(());
                }
            },
        }
    }

    let repo_path = args.validate()?;
    let repo = GitRepository::open(&repo_path)?;

    let is_commit_specified = args.commit.is_some();
    let is_range_mode = args
        .commit
        .as_ref()
        .map(|c| c.contains(".."))
        .unwrap_or(false);

    // Load config: CLI arguments > config file > defaults
    let config = Config::load()?;

    // Initialize ignore patterns: CLI flags > ignore-file > config
    let mut patterns = config.ignore_patterns.clone();
    if let Some(path) = &args.ignore_file {
        if let Ok(content) = std::fs::read_to_string(path) {
            patterns.extend(
                content
                    .lines()
                    .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                    .map(String::from),
            );
        }
    }
    patterns.extend(args.ignore.clone());
    git::init_ignore_patterns(&patterns).ok();
    let theme_name = args.theme.as_deref().unwrap_or(&config.theme);
    let speed = args.speed.unwrap_or(config.speed);
    let background = args.background.unwrap_or(config.background);
    let mut order = args.order.unwrap_or(match config.order.as_str() {
        "asc" => PlaybackOrder::Asc,
        "desc" => PlaybackOrder::Desc,
        _ => PlaybackOrder::Random,
    });

    // Range mode defaults to asc (chronological) if not explicitly specified
    if is_range_mode && args.order.is_none() {
        order = PlaybackOrder::Asc;
    }

    let loop_playback = args.loop_playback.unwrap_or(config.loop_playback);
    let mut theme = Theme::load(theme_name)?;

    // Apply transparent background if requested
    if !background {
        theme = theme.with_transparent_background();
    }

    // Setup commit range if specified
    if is_range_mode {
        repo.set_commit_range(args.commit.as_ref().unwrap())?;
    }

    // Load initial commit
    let metadata = if is_range_mode {
        match order {
            PlaybackOrder::Random => repo.random_range_commit()?,
            PlaybackOrder::Asc => repo.next_range_commit_asc()?,
            PlaybackOrder::Desc => repo.next_range_commit_desc()?,
        }
    } else if let Some(commit_hash) = &args.commit {
        repo.get_commit(commit_hash)?
    } else {
        match order {
            PlaybackOrder::Random => repo.random_commit()?,
            PlaybackOrder::Asc => repo.next_asc_commit()?,
            PlaybackOrder::Desc => repo.next_desc_commit()?,
        }
    };

    // Create UI with repository reference
    // Range mode always needs repo reference for iteration
    let repo_ref = if is_range_mode {
        Some(&repo)
    } else if is_commit_specified && !loop_playback {
        None
    } else {
        Some(&repo)
    };
    let mut ui = UI::new(
        speed,
        repo_ref,
        theme,
        order,
        loop_playback,
        args.commit.clone(),
        is_range_mode,
    );
    ui.load_commit(metadata);
    ui.run()?;

    Ok(())
}
