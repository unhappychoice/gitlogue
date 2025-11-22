use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Commit as Git2Commit, Delta, DiffOptions, Oid, Repository};
use globset::{Glob, GlobSet, GlobSetBuilder};
use rand::Rng;
use std::cell::RefCell;
use std::path::Path;
use std::sync::OnceLock;

// Thread-safe global pattern matcher for user-defined ignore patterns
static USER_PATTERNS: OnceLock<GlobSet> = OnceLock::new();

// Maximum blob size to read (500KB)
const MAX_BLOB_SIZE: usize = 500 * 1024;

// Maximum number of changed lines per file to animate
// Files with more changes will be skipped to prevent performance issues
const MAX_CHANGE_LINES: usize = 2000;

// Files to exclude from diff animation (lock files and generated files)
const EXCLUDED_FILES: &[&str] = &[
    // JavaScript/Node.js
    "yarn.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "bun.lock",
    "bun.lockb",
    // Rust
    "Cargo.lock",
    // Ruby
    "Gemfile.lock",
    // Python
    "poetry.lock",
    "Pipfile.lock",
    // PHP
    "composer.lock",
    // Go
    "go.sum",
    // Swift
    "Package.resolved",
    // Dart/Flutter
    "pubspec.lock",
    // .NET/C#
    "packages.lock.json",
    "project.assets.json",
    // Elixir
    "mix.lock",
    // Java/Gradle
    "gradle.lockfile",
    "buildscript-gradle.lockfile",
    // Scala
    "build.sbt.lock",
    // Bazel
    "MODULE.bazel.lock",
];

// File patterns to exclude from diff animation
const EXCLUDED_PATTERNS: &[&str] = &[
    // Minified files
    ".min.js",
    ".min.css",
    // Bundled files
    ".bundle.js",
    ".bundle.css",
    // Source maps
    ".js.map",
    ".css.map",
    ".d.ts.map",
    // Test snapshots
    ".snap",
    "__snapshots__",
];

/// Initialize user-defined ignore patterns (call once at startup)
pub fn init_ignore_patterns(patterns: &[String]) -> Result<()> {
    if patterns.is_empty() {
        return Ok(());
    }

    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        let glob =
            Glob::new(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?;
        builder.add(glob);
    }

    let globset = builder.build().context("Failed to build glob set")?;

    USER_PATTERNS
        .set(globset)
        .map_err(|_| anyhow::anyhow!("User patterns already initialized"))?;

    Ok(())
}

/// Check if a file should be excluded from diff animation
pub fn should_exclude_file(path: &str) -> bool {
    // Check user-defined patterns first
    if let Some(patterns) = USER_PATTERNS.get() {
        if patterns.is_match(path) {
            return true;
        }
    }

    let filename = path.rsplit('/').next().unwrap_or(path);

    // Check if it's a lock file
    if EXCLUDED_FILES.contains(&filename) {
        return true;
    }

    // Check if it matches excluded patterns
    for pattern in EXCLUDED_PATTERNS {
        if filename.ends_with(pattern) || path.contains(pattern) {
            return true;
        }
    }

    false
}

pub struct GitRepository {
    repo: Repository,
    commit_cache: RefCell<Option<Vec<Oid>>>,
    // Shared index for both cache-based playback (asc/desc) and range playback.
    // These modes are mutually exclusive based on CLI arguments.
    commit_index: RefCell<usize>,
    commit_range: RefCell<Option<Vec<Oid>>>,
}

#[derive(Debug, Clone)]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Unmodified,
}

impl FileStatus {
    pub fn as_str(&self) -> &str {
        match self {
            FileStatus::Added => "A",
            FileStatus::Deleted => "D",
            FileStatus::Modified => "M",
            FileStatus::Renamed => "R",
            FileStatus::Copied => "C",
            FileStatus::Unmodified => "U",
        }
    }
}

impl From<Delta> for FileStatus {
    fn from(delta: Delta) -> Self {
        match delta {
            Delta::Added => FileStatus::Added,
            Delta::Deleted => FileStatus::Deleted,
            Delta::Modified => FileStatus::Modified,
            Delta::Renamed => FileStatus::Renamed,
            Delta::Copied => FileStatus::Copied,
            Delta::Unmodified => FileStatus::Unmodified,
            _ => FileStatus::Modified,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LineChangeType {
    Addition,
    Deletion,
    Context,
}

#[derive(Debug, Clone)]
pub struct LineChange {
    pub change_type: LineChangeType,
    pub content: String,
    #[allow(dead_code)]
    pub old_line_no: Option<usize>,
    #[allow(dead_code)]
    pub new_line_no: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    #[allow(dead_code)]
    pub old_lines: usize,
    #[allow(dead_code)]
    pub new_start: usize,
    #[allow(dead_code)]
    pub new_lines: usize,
    pub lines: Vec<LineChange>,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    #[allow(dead_code)]
    pub old_path: Option<String>,
    pub status: FileStatus,
    #[allow(dead_code)]
    pub is_binary: bool,
    pub is_excluded: bool,
    pub exclusion_reason: Option<String>,
    pub old_content: Option<String>,
    #[allow(dead_code)]
    pub new_content: Option<String>,
    pub hunks: Vec<DiffHunk>,
    #[allow(dead_code)]
    pub diff: String,
}

#[derive(Debug, Clone)]
pub struct CommitMetadata {
    pub hash: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub changes: Vec<FileChange>,
}

impl CommitMetadata {
    /// Returns indices sorted in FileTree display order (directory -> filename)
    pub fn sorted_file_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.changes.len()).collect();
        indices.sort_by_key(|&index| {
            let path = &self.changes[index].path;
            let parts: Vec<&str> = path.split('/').collect();

            if parts.len() == 1 {
                // Root level file: ("", filename)
                (String::new(), path.clone())
            } else {
                // File in directory: (directory, filename)
                let dir = parts[..parts.len() - 1].join("/");
                let filename = parts[parts.len() - 1].to_string();
                (dir, filename)
            }
        });
        indices
    }
}

impl GitRepository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path).context("Failed to open Git repository")?;
        Ok(Self {
            repo,
            commit_cache: RefCell::new(None),
            commit_index: RefCell::new(0),
            commit_range: RefCell::new(None),
        })
    }

    pub fn get_commit(&self, hash: &str) -> Result<CommitMetadata> {
        let obj = self
            .repo
            .revparse_single(hash)
            .context("Invalid commit hash or commit not found")?;

        let commit = obj.peel_to_commit().context("Object is not a commit")?;

        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    pub fn random_commit(&self) -> Result<CommitMetadata> {
        // Check if cache exists, if not populate it
        let mut cache = self.commit_cache.borrow_mut();
        if cache.is_none() {
            let mut revwalk = self.repo.revwalk()?;
            revwalk.push_head()?;

            let mut candidates = Vec::new();
            for oid in revwalk.filter_map(|oid| oid.ok()) {
                if let Ok(commit) = self.repo.find_commit(oid) {
                    if commit.parent_count() <= 1 {
                        candidates.push(oid);
                    }
                }
            }

            if candidates.is_empty() {
                anyhow::bail!("No non-merge commits found in repository");
            }

            *cache = Some(candidates);
        }

        let candidates = cache.as_ref().unwrap();
        let selected_oid = candidates
            .get(rand::rng().random_range(0..candidates.len()))
            .context("Failed to select random commit")?;

        let commit = self.repo.find_commit(*selected_oid)?;
        drop(cache); // Release the borrow before calling extract_metadata_with_changes
        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    pub fn next_asc_commit(&self) -> Result<CommitMetadata> {
        self.populate_cache()?;

        let cache = self.commit_cache.borrow();
        let candidates = cache.as_ref().unwrap();
        let mut index = self.commit_index.borrow_mut();

        if candidates.is_empty() {
            anyhow::bail!("No non-merge commits found in repository");
        }

        if *index >= candidates.len() {
            anyhow::bail!("All commits have been played");
        }

        // Asc order: oldest first (reverse of cache order)
        let asc_index = candidates.len() - 1 - *index;
        let selected_oid = candidates
            .get(asc_index)
            .context("Failed to select commit")?;

        *index += 1;

        let commit = self.repo.find_commit(*selected_oid)?;
        drop(index);
        drop(cache);
        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    pub fn next_desc_commit(&self) -> Result<CommitMetadata> {
        self.populate_cache()?;

        let cache = self.commit_cache.borrow();
        let candidates = cache.as_ref().unwrap();
        let mut index = self.commit_index.borrow_mut();

        if candidates.is_empty() {
            anyhow::bail!("No non-merge commits found in repository");
        }

        if *index >= candidates.len() {
            anyhow::bail!("All commits have been played");
        }

        // Desc order: newest first (same as cache order)
        let selected_oid = candidates.get(*index).context("Failed to select commit")?;

        *index += 1;

        let commit = self.repo.find_commit(*selected_oid)?;
        drop(index);
        drop(cache);
        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    pub fn reset_index(&self) {
        *self.commit_index.borrow_mut() = 0;
    }

    pub fn set_commit_range(&self, range: &str) -> Result<()> {
        let commits = self.parse_commit_range(range)?;
        *self.commit_range.borrow_mut() = Some(commits);
        *self.commit_index.borrow_mut() = 0;
        Ok(())
    }

    pub fn next_range_commit_asc(&self) -> Result<CommitMetadata> {
        let range = self.commit_range.borrow();
        let commits = range.as_ref().context("Commit range not set")?;
        let mut index = self.commit_index.borrow_mut();

        if commits.is_empty() {
            anyhow::bail!("No commits in range");
        }

        if *index >= commits.len() {
            anyhow::bail!("All commits in range have been played");
        }

        let selected_oid = commits.get(*index).context("Failed to select commit")?;
        *index += 1;

        let commit = self.repo.find_commit(*selected_oid)?;
        drop(index);
        drop(range);
        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    pub fn next_range_commit_desc(&self) -> Result<CommitMetadata> {
        let range = self.commit_range.borrow();
        let commits = range.as_ref().context("Commit range not set")?;
        let mut index = self.commit_index.borrow_mut();

        if commits.is_empty() {
            anyhow::bail!("No commits in range");
        }

        if *index >= commits.len() {
            anyhow::bail!("All commits in range have been played");
        }

        // Desc order: newest first (reverse of asc)
        let desc_index = commits.len() - 1 - *index;
        let selected_oid = commits.get(desc_index).context("Failed to select commit")?;
        *index += 1;

        let commit = self.repo.find_commit(*selected_oid)?;
        drop(index);
        drop(range);
        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    pub fn random_range_commit(&self) -> Result<CommitMetadata> {
        let range = self.commit_range.borrow();
        let commits = range.as_ref().context("Commit range not set")?;

        if commits.is_empty() {
            anyhow::bail!("No commits in range");
        }

        let selected_oid = commits
            .get(rand::rng().random_range(0..commits.len()))
            .context("Failed to select random commit")?;

        let commit = self.repo.find_commit(*selected_oid)?;
        drop(range);
        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    fn parse_commit_range(&self, range: &str) -> Result<Vec<Oid>> {
        // Reject symmetric difference operator (not supported)
        if range.contains("...") {
            anyhow::bail!(
                "Symmetric difference operator '...' is not supported. Use '..' instead (e.g., 'HEAD~5..HEAD')"
            );
        }

        if !range.contains("..") {
            anyhow::bail!(
                "Invalid range format: {}. Use formats like 'HEAD~5..HEAD' or 'abc123..'",
                range
            );
        }

        let parts: Vec<&str> = range.split("..").collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid range format: {}", range);
        }

        let start = if parts[0].is_empty() {
            None
        } else {
            Some(self.repo.revparse_single(parts[0])?.id())
        };

        let end = if parts[1].is_empty() {
            self.repo.head()?.peel_to_commit()?.id()
        } else {
            self.repo.revparse_single(parts[1])?.id()
        };

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(end)?;

        if let Some(start_oid) = start {
            revwalk.hide(start_oid)?;
        }

        let mut commits = Vec::new();
        for oid in revwalk.filter_map(|oid| oid.ok()) {
            if let Ok(commit) = self.repo.find_commit(oid) {
                if commit.parent_count() <= 1 {
                    commits.push(oid);
                }
            }
        }

        commits.reverse();
        Ok(commits)
    }

    fn populate_cache(&self) -> Result<()> {
        let mut cache = self.commit_cache.borrow_mut();
        if cache.is_none() {
            let mut revwalk = self.repo.revwalk()?;
            revwalk.push_head()?;

            let mut candidates = Vec::new();
            for oid in revwalk.filter_map(|oid| oid.ok()) {
                if let Ok(commit) = self.repo.find_commit(oid) {
                    if commit.parent_count() <= 1 {
                        candidates.push(oid);
                    }
                }
            }

            if candidates.is_empty() {
                anyhow::bail!("No non-merge commits found in repository");
            }

            *cache = Some(candidates);
        }
        Ok(())
    }

    fn extract_metadata_with_changes(
        repo: &Repository,
        commit: &Git2Commit,
    ) -> Result<CommitMetadata> {
        let hash = commit.id().to_string();
        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let timestamp = author.when().seconds();
        let date = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);
        let message = commit.message().unwrap_or("").trim().to_string();

        let changes = Self::extract_changes(repo, commit)?;

        Ok(CommitMetadata {
            hash,
            author: author_name,
            date,
            message,
            changes,
        })
    }

    fn extract_changes(repo: &Repository, commit: &Git2Commit) -> Result<Vec<FileChange>> {
        let commit_tree = commit.tree().context("Failed to get commit tree")?;
        let parent_tree = if commit.parent_count() > 0 {
            match commit.parent(0).and_then(|p| p.tree()) {
                Ok(tree) => Some(tree),
                Err(_) => return Ok(Vec::new()), // Skip if parent tree unavailable
            }
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        diff_opts.context_lines(3);

        let diff = match repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&commit_tree),
            Some(&mut diff_opts),
        ) {
            Ok(d) => d,
            Err(_) => return Ok(Vec::new()), // Skip if diff fails
        };

        let mut changes = Vec::new();

        for i in 0..diff.deltas().len() {
            let delta = diff.get_delta(i).unwrap();
            let status = FileStatus::from(delta.status());

            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .and_then(|p| p.to_str())
                .unwrap_or("unknown")
                .to_string();

            let old_path = if delta.status() == Delta::Renamed {
                delta
                    .old_file()
                    .path()
                    .and_then(|p| p.to_str())
                    .map(String::from)
            } else {
                None
            };

            let is_binary = delta.new_file().is_binary() || delta.old_file().is_binary();

            let old_content = if let Some(parent_tree) = parent_tree.as_ref() {
                if let Some(old_file_path) = delta.old_file().path() {
                    parent_tree
                        .get_path(old_file_path)
                        .ok()
                        .and_then(|entry| repo.find_blob(entry.id()).ok())
                        .and_then(|blob| {
                            if !blob.is_binary() && blob.size() <= MAX_BLOB_SIZE {
                                Some(String::from_utf8_lossy(blob.content()).to_string())
                            } else {
                                None
                            }
                        })
                } else {
                    None
                }
            } else {
                None
            };

            let new_content = if let Some(new_file_path) = delta.new_file().path() {
                commit_tree
                    .get_path(new_file_path)
                    .ok()
                    .and_then(|entry| repo.find_blob(entry.id()).ok())
                    .and_then(|blob| {
                        if !blob.is_binary() && blob.size() <= MAX_BLOB_SIZE {
                            Some(String::from_utf8_lossy(blob.content()).to_string())
                        } else {
                            None
                        }
                    })
            } else {
                None
            };

            let mut hunks = Vec::new();
            let mut diff_text = String::new();

            if let Ok(Some(mut patch)) = git2::Patch::from_diff(&diff, i) {
                if let Ok(patch_str) = patch.to_buf() {
                    diff_text = String::from_utf8_lossy(patch_str.as_ref()).to_string();
                }

                if !is_binary {
                    for hunk_idx in 0..patch.num_hunks() {
                        if let Ok((hunk, _hunk_lines)) = patch.hunk(hunk_idx) {
                            let mut lines = Vec::new();
                            let num_lines = patch.num_lines_in_hunk(hunk_idx).unwrap_or(0);

                            let mut old_line_no = hunk.old_start() as usize;
                            let mut new_line_no = hunk.new_start() as usize;

                            for line_idx in 0..num_lines {
                                if let Ok(line) = patch.line_in_hunk(hunk_idx, line_idx) {
                                    let content =
                                        String::from_utf8_lossy(line.content()).to_string();
                                    let origin = line.origin();

                                    let (change_type, old_no, new_no) = match origin {
                                        '+' => {
                                            let no = new_line_no;
                                            new_line_no += 1;
                                            (LineChangeType::Addition, None, Some(no))
                                        }
                                        '-' => {
                                            let no = old_line_no;
                                            old_line_no += 1;
                                            (LineChangeType::Deletion, Some(no), None)
                                        }
                                        _ => {
                                            let old_no = old_line_no;
                                            let new_no = new_line_no;
                                            old_line_no += 1;
                                            new_line_no += 1;
                                            (LineChangeType::Context, Some(old_no), Some(new_no))
                                        }
                                    };

                                    lines.push(LineChange {
                                        change_type,
                                        content,
                                        old_line_no: old_no,
                                        new_line_no: new_no,
                                    });
                                }
                            }

                            hunks.push(DiffHunk {
                                old_start: hunk.old_start() as usize,
                                old_lines: hunk.old_lines() as usize,
                                new_start: hunk.new_start() as usize,
                                new_lines: hunk.new_lines() as usize,
                                lines,
                            });
                        }
                    }
                }
            }

            // Calculate total changed lines (additions + deletions)
            let total_changed_lines: usize = hunks
                .iter()
                .flat_map(|hunk| &hunk.lines)
                .filter(|line| !matches!(line.change_type, LineChangeType::Context))
                .count();

            // Determine exclusion reason
            let (is_excluded, exclusion_reason) = if should_exclude_file(&path) {
                (true, Some("lock/generated file".to_string()))
            } else if total_changed_lines > MAX_CHANGE_LINES {
                (
                    true,
                    Some(format!("too many changes ({} lines)", total_changed_lines)),
                )
            } else {
                (false, None)
            };

            changes.push(FileChange {
                path,
                old_path,
                status,
                is_binary,
                is_excluded,
                exclusion_reason,
                old_content,
                new_content,
                hunks,
                diff: diff_text,
            });
        }

        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_exclude_lock_files() {
        // JavaScript/Node.js
        assert!(should_exclude_file("package-lock.json"));
        assert!(should_exclude_file("yarn.lock"));
        assert!(should_exclude_file("pnpm-lock.yaml"));
        // Rust
        assert!(should_exclude_file("Cargo.lock"));
        // Ruby
        assert!(should_exclude_file("Gemfile.lock"));
        // Python
        assert!(should_exclude_file("poetry.lock"));
        assert!(should_exclude_file("Pipfile.lock"));
        // PHP
        assert!(should_exclude_file("composer.lock"));
        // Go
        assert!(should_exclude_file("go.sum"));
        // Swift
        assert!(should_exclude_file("Package.resolved"));
        // Dart/Flutter
        assert!(should_exclude_file("pubspec.lock"));
        // .NET/C#
        assert!(should_exclude_file("packages.lock.json"));
        assert!(should_exclude_file("project.assets.json"));
        // Elixir
        assert!(should_exclude_file("mix.lock"));
        // Java/Gradle
        assert!(should_exclude_file("gradle.lockfile"));
        assert!(should_exclude_file("buildscript-gradle.lockfile"));
        // Scala
        assert!(should_exclude_file("build.sbt.lock"));
        // Bazel
        assert!(should_exclude_file("MODULE.bazel.lock"));
    }

    #[test]
    fn test_should_exclude_lock_files_with_path() {
        assert!(should_exclude_file("path/to/package-lock.json"));
        assert!(should_exclude_file("src/Cargo.lock"));
        assert!(should_exclude_file("frontend/yarn.lock"));
    }

    #[test]
    fn test_should_exclude_minified_files() {
        assert!(should_exclude_file("bundle.min.js"));
        assert!(should_exclude_file("app.min.css"));
        assert!(should_exclude_file("vendor.bundle.js"));
        assert!(should_exclude_file("styles.bundle.css"));
        // Source maps
        assert!(should_exclude_file("app.js.map"));
        assert!(should_exclude_file("styles.css.map"));
        assert!(should_exclude_file("types.d.ts.map"));
    }

    #[test]
    fn test_should_exclude_minified_files_with_path() {
        assert!(should_exclude_file("dist/bundle.min.js"));
        assert!(should_exclude_file("public/assets/app.min.css"));
    }

    #[test]
    fn test_should_not_exclude_normal_files() {
        assert!(!should_exclude_file("src/main.rs"));
        assert!(!should_exclude_file("package.json"));
        assert!(!should_exclude_file("Cargo.toml"));
        assert!(!should_exclude_file("app.js"));
        assert!(!should_exclude_file("styles.css"));
        assert!(!should_exclude_file("lock.txt"));
        assert!(!should_exclude_file("minify.rs"));
    }

    #[test]
    fn test_should_exclude_snapshot_files() {
        assert!(should_exclude_file("component.test.ts.snap"));
        assert!(should_exclude_file("tests/__snapshots__/test.snap"));
        assert!(should_exclude_file("__snapshots__/component.snap"));
        assert!(should_exclude_file("src/__snapshots__/app.test.js.snap"));
    }
}
