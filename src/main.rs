mod git;

use anyhow::{Context, Result};
use clap::Parser;
use git::GitRepository;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "git-logue",
    version = "0.1.0",
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
        value_name = "HASH",
        help = "Replay a specific commit (single-commit mode)"
    )]
    pub commit: Option<String>,

    #[arg(
        short,
        long,
        value_name = "MS",
        default_value = "30",
        help = "Typing speed in milliseconds per character"
    )]
    pub speed: u64,
}

impl Args {
    pub fn validate(&self) -> Result<PathBuf> {
        let repo_path = self
            .path
            .clone()
            .unwrap_or_else(|| PathBuf::from("."));

        if !repo_path.exists() {
            anyhow::bail!("Path does not exist: {}", repo_path.display());
        }

        let git_dir = repo_path.join(".git");
        if !git_dir.exists() {
            anyhow::bail!(
                "Not a Git repository: {} (or any parent directories)",
                repo_path.display()
            );
        }

        repo_path
            .canonicalize()
            .context("Failed to resolve repository path")
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let repo_path = args.validate()?;

    println!("git-logue v0.1.0");
    println!("Repository: {}", repo_path.display());
    println!("Speed: {}ms per character", args.speed);
    println!();

    let repo = GitRepository::open(&repo_path)?;

    if let Some(commit_hash) = &args.commit {
        println!("Single-commit mode: {}", commit_hash);
        let metadata = repo.get_commit(commit_hash)?;
        print_commit_info(&metadata);
    } else {
        println!("Random commit loop mode");
        let metadata = repo.random_commit()?;
        println!("Selected random commit:");
        print_commit_info(&metadata);
    }

    Ok(())
}

fn print_commit_info(metadata: &git::CommitMetadata) {
    println!("  Hash:    {}", metadata.hash);
    println!("  Author:  {}", metadata.author);
    println!("  Date:    {}", metadata.date.format("%Y-%m-%d %H:%M:%S"));
    println!("  Message: {}", metadata.message);
    println!();
    println!("  Files changed: {}", metadata.changes.len());
    for change in &metadata.changes {
        let mut display_path = format!("{} {}", change.status.as_str(), change.path);
        if let Some(old_path) = &change.old_path {
            display_path = format!("{} {} -> {}", change.status.as_str(), old_path, change.path);
        }
        if change.is_binary {
            display_path.push_str(" (binary)");
        }
        println!("    {}", display_path);
    }

    if !metadata.changes.is_empty() {
        println!();
        println!("  File contents:");
        for change in &metadata.changes {
            println!("    File: {}", change.path);
            if change.is_binary {
                println!("      (binary file)");
                continue;
            }

            if let Some(old) = &change.old_content {
                let lines = old.lines().count();
                println!("      Old content: {} lines", lines);
            } else {
                println!("      Old content: (none - new file)");
            }

            if let Some(new) = &change.new_content {
                let lines = new.lines().count();
                println!("      New content: {} lines", lines);
            } else {
                println!("      New content: (none - deleted file)");
            }
        }

        println!();
        println!("  Structured changes:");
        for change in &metadata.changes {
            println!("    File: {}", change.path);
            if change.is_binary {
                println!("      (binary file)");
                continue;
            }

            println!("      Hunks: {}", change.hunks.len());
            for (i, hunk) in change.hunks.iter().enumerate() {
                println!(
                    "        Hunk #{}: @@ -{},{} +{},{} @@",
                    i + 1,
                    hunk.old_start,
                    hunk.old_lines,
                    hunk.new_start,
                    hunk.new_lines
                );

                let additions = hunk
                    .lines
                    .iter()
                    .filter(|l| matches!(l.change_type, git::LineChangeType::Addition))
                    .count();
                let deletions = hunk
                    .lines
                    .iter()
                    .filter(|l| matches!(l.change_type, git::LineChangeType::Deletion))
                    .count();

                println!(
                    "          Lines: {} total (+{} -{} context:{})",
                    hunk.lines.len(),
                    additions,
                    deletions,
                    hunk.lines.len() - additions - deletions
                );
            }
        }

        println!();
        println!("  Raw diff:");
        for change in &metadata.changes {
            if !change.diff.is_empty() {
                println!("{}", change.diff);
            }
        }
    }
}
