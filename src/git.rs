use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use gix::bstr::ByteSlice;
use gix::diff::blob::Algorithm;
use gix::object::tree::diff::Change;
use gix::{ObjectId, Repository};
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
    commit_cache: RefCell<Option<Vec<ObjectId>>>,
    // Shared index for both cache-based playback (asc/desc) and range playback.
    // These modes are mutually exclusive based on CLI arguments.
    commit_index: RefCell<usize>,
    commit_range: RefCell<Option<Vec<ObjectId>>>,
}

#[derive(Debug, Clone)]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
}

impl FileStatus {
    pub fn as_str(&self) -> &str {
        match self {
            FileStatus::Added => "A",
            FileStatus::Deleted => "D",
            FileStatus::Modified => "M",
            FileStatus::Renamed => "R",
            FileStatus::Copied => "C",
        }
    }
}

impl FileStatus {
    fn from_change(change: &Change<'_, '_, '_>) -> Self {
        match change {
            Change::Addition { .. } => FileStatus::Added,
            Change::Deletion { .. } => FileStatus::Deleted,
            Change::Modification { .. } => FileStatus::Modified,
            Change::Rewrite { copy: false, .. } => FileStatus::Renamed,
            Change::Rewrite { copy: true, .. } => FileStatus::Copied,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LineChangeType {
    Addition,
    Deletion,
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

struct DiffHunkCollector<'a> {
    input: &'a gix::diff::blob::intern::InternedInput<&'a str>,
    hunks: Vec<DiffHunk>,
    current_hunk: Option<DiffHunk>,
    old_line_no: usize,
    new_line_no: usize,
}

impl<'a> DiffHunkCollector<'a> {
    fn new(input: &'a gix::diff::blob::intern::InternedInput<&'a str>) -> Self {
        Self {
            input,
            hunks: Vec::new(),
            current_hunk: None,
            old_line_no: 1,
            new_line_no: 1,
        }
    }

    fn finish_current_hunk(&mut self) {
        if let Some(hunk) = self.current_hunk.take() {
            self.hunks.push(hunk);
        }
    }
}

impl<'a> gix::diff::blob::Sink for DiffHunkCollector<'a> {
    type Out = Vec<DiffHunk>;

    fn process_change(&mut self, before: std::ops::Range<u32>, after: std::ops::Range<u32>) {
        // Finish previous hunk if it exists
        self.finish_current_hunk();

        let old_start = before.start as usize + 1;
        let new_start = after.start as usize + 1;
        let old_lines = (before.end - before.start) as usize;
        let new_lines = (after.end - after.start) as usize;

        self.old_line_no = old_start;
        self.new_line_no = new_start;

        let mut lines = Vec::new();

        // Process deletions from the before range
        for i in before.start..before.end {
            if let Some(line_token) = self.input.before.get(i as usize) {
                let content = self.input.interner[*line_token].to_string();
                lines.push(LineChange {
                    change_type: LineChangeType::Deletion,
                    content,
                    old_line_no: Some(self.old_line_no),
                    new_line_no: None,
                });
                self.old_line_no += 1;
            }
        }

        // Process additions from the after range
        for i in after.start..after.end {
            if let Some(line_token) = self.input.after.get(i as usize) {
                let content = self.input.interner[*line_token].to_string();
                lines.push(LineChange {
                    change_type: LineChangeType::Addition,
                    content,
                    old_line_no: None,
                    new_line_no: Some(self.new_line_no),
                });
                self.new_line_no += 1;
            }
        }

        self.current_hunk = Some(DiffHunk {
            old_start,
            old_lines,
            new_start,
            new_lines,
            lines,
        });
    }

    fn finish(mut self) -> Self::Out {
        self.finish_current_hunk();
        self.hunks
    }
}

impl GitRepository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = gix::open(path.as_ref()).context("Failed to open Git repository")?;
        Ok(Self {
            repo,
            commit_cache: RefCell::new(None),
            commit_index: RefCell::new(0),
            commit_range: RefCell::new(None),
        })
    }

    pub fn get_commit(&self, hash: &str) -> Result<CommitMetadata> {
        let spec = self
            .repo
            .rev_parse_single(hash)
            .context("Invalid commit hash or commit not found")?;

        let commit_id = spec.object()?.id;
        let commit = self.repo.find_commit(commit_id)?;

        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    pub fn random_commit(&self) -> Result<CommitMetadata> {
        // Check if cache exists, if not populate it
        let mut cache = self.commit_cache.borrow_mut();
        if cache.is_none() {
            let head = self.repo.head_id()?;
            let commits = self.repo.rev_walk([head]).all()?.filter_map(Result::ok);

            let mut candidates = Vec::new();
            for info in commits {
                let Ok(commit) = self.repo.find_commit(info.id) else {
                    continue;
                };
                if commit.parent_ids().count() <= 1 {
                    candidates.push(info.id);
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

    fn parse_commit_range(&self, range: &str) -> Result<Vec<ObjectId>> {
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
            Some(self.repo.rev_parse_single(parts[0])?.object()?.id)
        };

        let end: ObjectId = if parts[1].is_empty() {
            self.repo.head_id()?.into()
        } else {
            self.repo.rev_parse_single(parts[1])?.object()?.id
        };

        // Build list of commits to exclude if start is specified
        // TODO: use `with_hidden()` once that can be trusted.
        let exclude_set: gix::hashtable::HashSet = if let Some(start_oid) = start {
            self.repo
                .rev_walk([start_oid])
                .all()?
                .filter_map(Result::ok)
                .map(|info| info.id)
                .collect()
        } else {
            Default::default()
        };

        let mut commits = Vec::new();
        for info in self.repo.rev_walk([end]).all()?.filter_map(Result::ok) {
            if !exclude_set.contains(&info.id) {
                let Ok(commit) = self.repo.find_commit(info.id) else {
                    continue;
                };
                if commit.parent_ids().count() <= 1 {
                    commits.push(info.id);
                }
            }
        }

        commits.reverse();
        Ok(commits)
    }

    fn populate_cache(&self) -> Result<()> {
        let mut cache = self.commit_cache.borrow_mut();
        if cache.is_none() {
            let head = self.repo.head_id()?;
            let commits = self.repo.rev_walk([head]).all()?.filter_map(Result::ok);

            let mut candidates = Vec::new();
            for info in commits {
                let Ok(commit) = self.repo.find_commit(info.id) else {
                    continue;
                };
                if commit.parent_ids().count() <= 1 {
                    candidates.push(info.id);
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
        commit: &gix::Commit,
    ) -> Result<CommitMetadata> {
        let hash = commit.id.to_string();
        let commit_obj = commit.decode()?;
        let author_sig = commit_obj.author();
        let author_name = author_sig.name.to_str_lossy().into_owned();

        // Parse the time string (format: "seconds timezone") - we only need the seconds
        let timestamp = author_sig.time()?.seconds;
        let date = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);
        let message = commit_obj.message.to_str_lossy().into_owned();

        let changes = Self::extract_changes(repo, commit)?;

        Ok(CommitMetadata {
            hash,
            author: author_name,
            date,
            message,
            changes,
        })
    }

    fn extract_changes(repo: &Repository, commit: &gix::Commit) -> Result<Vec<FileChange>> {
        let commit_obj = commit.decode()?;
        let commit_tree_id = commit_obj.tree();
        let commit_tree = repo.find_tree(commit_tree_id)?;

        let parent_tree = if let Some(parent_id) = commit_obj.parents().next() {
            repo.find_commit(parent_id)?.tree()?
        } else {
            repo.empty_tree()
        };

        let mut changes = Vec::new();
        let algo = repo.diff_algorithm()?;
        parent_tree
            .changes()?
            .for_each_to_obtain_tree(&commit_tree, |change| {
                if change.entry_mode().is_tree() {
                    return anyhow::Ok(gix::object::tree::diff::Action::Continue);
                }
                let path = change.location().to_str_lossy().into_owned();
                let status = FileStatus::from_change(&change);

                let old_path = if let Change::Rewrite {
                    source_location, ..
                } = &change
                {
                    Some(source_location)
                } else {
                    None
                };
                let (old_id, new_id, is_binary) = match &change {
                    Change::Addition { id, .. } => {
                        let oid: ObjectId = id.to_owned().into();
                        (None, Some(oid), Self::is_blob_binary(repo, oid))
                    }
                    Change::Deletion { id, .. } => {
                        let oid: ObjectId = id.to_owned().into();
                        (Some(oid), None, Self::is_blob_binary(repo, oid))
                    }
                    Change::Modification {
                        previous_id: source_id,
                        id,
                        ..
                    }
                    | Change::Rewrite { source_id, id, .. } => {
                        let old_oid: ObjectId = source_id.to_owned().into();
                        let new_oid: ObjectId = id.to_owned().into();
                        let old_binary = Self::is_blob_binary(repo, old_oid);
                        let new_binary = Self::is_blob_binary(repo, new_oid);
                        (Some(old_oid), Some(new_oid), old_binary || new_binary)
                    }
                };

                let old_content =
                    old_id.and_then(|id| Self::get_blob_content(repo, id).ok().flatten());
                let new_content =
                    new_id.and_then(|id| Self::get_blob_content(repo, id).ok().flatten());

                let hunks = if !is_binary {
                    Self::generate_hunks(old_content.as_deref(), new_content.as_deref(), algo)
                } else {
                    Vec::new()
                };

                // Calculate total changed lines
                let total_changed_lines: usize = hunks.iter().flat_map(|hunk| &hunk.lines).count();

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
                    old_path: old_path.map(|path| path.to_str_lossy().into_owned()),
                    status,
                    is_binary,
                    is_excluded,
                    exclusion_reason,
                    old_content,
                    new_content,
                    hunks,
                    diff: String::new(),
                });

                anyhow::Ok(gix::object::tree::diff::Action::Continue)
            })?;

        Ok(changes)
    }

    fn is_blob_binary(repo: &Repository, id: ObjectId) -> bool {
        repo.find_blob(id)
            .ok()
            .map(|blob| {
                let data = blob.data.as_slice();
                data.len() > MAX_BLOB_SIZE || data.contains(&0)
            })
            .unwrap_or(false)
    }

    fn get_blob_content(repo: &Repository, id: ObjectId) -> Result<Option<String>> {
        let blob = repo.find_blob(id)?;
        let data = blob.data.as_slice();

        if data.len() > MAX_BLOB_SIZE || data.contains(&0) {
            Ok(None)
        } else {
            Ok(Some(String::from_utf8_lossy(data).to_string()))
        }
    }

    fn generate_hunks(
        old_content: Option<&str>,
        new_content: Option<&str>,
        algo: Algorithm,
    ) -> Vec<DiffHunk> {
        let old_str = old_content.unwrap_or("");
        let new_str = new_content.unwrap_or("");

        let input = gix::diff::blob::intern::InternedInput::new(old_str, new_str);
        let collector = DiffHunkCollector::new(&input);
        gix::diff::blob::diff(algo, &input, collector)
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

    #[test]
    fn test_user_patterns_integration() {
        // Test all pattern types in one test since OnceLock can only be set once
        let patterns = vec![
            "*.svg".to_string(),
            "*.ipynb".to_string(),
            "dist/**".to_string(),
            "node_modules/**".to_string(),
        ];

        // Only initialize if not already initialized
        let _ = init_ignore_patterns(&patterns);

        // Test file extension patterns
        assert!(should_exclude_file("diagram.svg"));
        assert!(should_exclude_file("path/to/notebook.ipynb"));
        assert!(should_exclude_file("assets/icon.svg"));
        assert!(!should_exclude_file("image.png"));
        assert!(!should_exclude_file("script.py"));

        // Test directory patterns
        assert!(should_exclude_file("dist/bundle.js"));
        assert!(should_exclude_file("dist/css/main.css"));
        assert!(should_exclude_file("node_modules/pkg/index.js"));
        assert!(!should_exclude_file("src/index.js"));
    }

    #[test]
    fn test_empty_patterns() {
        let patterns: Vec<String> = vec![];
        assert!(init_ignore_patterns(&patterns).is_ok());
    }

    #[test]
    fn test_invalid_pattern() {
        let patterns = vec!["[invalid".to_string()];
        assert!(init_ignore_patterns(&patterns).is_err());
    }

    mod generate_hunks {
        use crate::git::GitRepository;
        use gix::diff::blob::Algorithm;

        #[test]
        fn test_generate_hunks_simple_addition() {
            let old = "";
            let new = "line 1\nline 2\nline 3\n";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 1,
                    old_lines: 0,
                    new_start: 1,
                    new_lines: 3,
                    lines: [
                        LineChange {
                            change_type: Addition,
                            content: "line 1",
                            old_line_no: None,
                            new_line_no: Some(
                                1,
                            ),
                        },
                        LineChange {
                            change_type: Addition,
                            content: "line 2",
                            old_line_no: None,
                            new_line_no: Some(
                                2,
                            ),
                        },
                        LineChange {
                            change_type: Addition,
                            content: "line 3",
                            old_line_no: None,
                            new_line_no: Some(
                                3,
                            ),
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_simple_deletion() {
            let old = "line 1\nline 2\nline 3\n";
            let new = "";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 1,
                    old_lines: 3,
                    new_start: 1,
                    new_lines: 0,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 1",
                            old_line_no: Some(
                                1,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Deletion,
                            content: "line 2",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Deletion,
                            content: "line 3",
                            old_line_no: Some(
                                3,
                            ),
                            new_line_no: None,
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_simple_modification() {
            let old = "line 1\nline 2\nline 3\n";
            let new = "line 1\nmodified line 2\nline 3\n";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 2,
                    old_lines: 1,
                    new_start: 2,
                    new_lines: 1,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 2",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Addition,
                            content: "modified line 2",
                            old_line_no: None,
                            new_line_no: Some(
                                2,
                            ),
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_multiple_changes() {
            let old = "line 1\nline 2\nline 3\nline 4\nline 5\n";
            let new = "line 1\nmodified line 2\nline 3\nline 4\nnew line 5\nline 6\n";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 2,
                    old_lines: 1,
                    new_start: 2,
                    new_lines: 1,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 2",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Addition,
                            content: "modified line 2",
                            old_line_no: None,
                            new_line_no: Some(
                                2,
                            ),
                        },
                    ],
                },
                DiffHunk {
                    old_start: 5,
                    old_lines: 1,
                    new_start: 5,
                    new_lines: 2,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 5",
                            old_line_no: Some(
                                5,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Addition,
                            content: "new line 5",
                            old_line_no: None,
                            new_line_no: Some(
                                5,
                            ),
                        },
                        LineChange {
                            change_type: Addition,
                            content: "line 6",
                            old_line_no: None,
                            new_line_no: Some(
                                6,
                            ),
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_addition_in_middle() {
            let old = "line 1\nline 2\nline 3\n";
            let new = "line 1\nline 2\ninserted line\nline 3\n";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 3,
                    old_lines: 0,
                    new_start: 3,
                    new_lines: 1,
                    lines: [
                        LineChange {
                            change_type: Addition,
                            content: "inserted line",
                            old_line_no: None,
                            new_line_no: Some(
                                3,
                            ),
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_deletion_in_middle() {
            let old = "line 1\nline 2\nline 3\nline 4\n";
            let new = "line 1\nline 4\n";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 2,
                    old_lines: 2,
                    new_start: 2,
                    new_lines: 0,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 2",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Deletion,
                            content: "line 3",
                            old_line_no: Some(
                                3,
                            ),
                            new_line_no: None,
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_both_empty() {
            let old = "";
            let new = "";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @"[]");
        }

        #[test]
        fn test_generate_hunks_none_old() {
            let new = "line 1\nline 2\n";
            let hunks = GitRepository::generate_hunks(None, Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 1,
                    old_lines: 0,
                    new_start: 1,
                    new_lines: 2,
                    lines: [
                        LineChange {
                            change_type: Addition,
                            content: "line 1",
                            old_line_no: None,
                            new_line_no: Some(
                                1,
                            ),
                        },
                        LineChange {
                            change_type: Addition,
                            content: "line 2",
                            old_line_no: None,
                            new_line_no: Some(
                                2,
                            ),
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_none_new() {
            let old = "line 1\nline 2\n";
            let hunks = GitRepository::generate_hunks(Some(old), None, Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 1,
                    old_lines: 2,
                    new_start: 1,
                    new_lines: 0,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 1",
                            old_line_no: Some(
                                1,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Deletion,
                            content: "line 2",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_both_none() {
            let hunks = GitRepository::generate_hunks(None, None, Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @"[]");
        }

        #[test]
        fn test_generate_hunks_replace_all() {
            let old = "old line 1\nold line 2\nold line 3\n";
            let new = "new line 1\nnew line 2\nnew line 3\n";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 1,
                    old_lines: 3,
                    new_start: 1,
                    new_lines: 3,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "old line 1",
                            old_line_no: Some(
                                1,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Deletion,
                            content: "old line 2",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Deletion,
                            content: "old line 3",
                            old_line_no: Some(
                                3,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Addition,
                            content: "new line 1",
                            old_line_no: None,
                            new_line_no: Some(
                                1,
                            ),
                        },
                        LineChange {
                            change_type: Addition,
                            content: "new line 2",
                            old_line_no: None,
                            new_line_no: Some(
                                2,
                            ),
                        },
                        LineChange {
                            change_type: Addition,
                            content: "new line 3",
                            old_line_no: None,
                            new_line_no: Some(
                                3,
                            ),
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_mixed_operations() {
            let old = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\n";
            let new = "line 1\nmodified 2\nline 3\nline 5\nline 6\nnew line 7\n";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 2,
                    old_lines: 1,
                    new_start: 2,
                    new_lines: 1,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 2",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Addition,
                            content: "modified 2",
                            old_line_no: None,
                            new_line_no: Some(
                                2,
                            ),
                        },
                    ],
                },
                DiffHunk {
                    old_start: 4,
                    old_lines: 1,
                    new_start: 4,
                    new_lines: 0,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 4",
                            old_line_no: Some(
                                4,
                            ),
                            new_line_no: None,
                        },
                    ],
                },
                DiffHunk {
                    old_start: 7,
                    old_lines: 0,
                    new_start: 6,
                    new_lines: 1,
                    lines: [
                        LineChange {
                            change_type: Addition,
                            content: "new line 7",
                            old_line_no: None,
                            new_line_no: Some(
                                6,
                            ),
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_whitespace_changes() {
            let old = "line 1\nline 2\n";
            let new = "line 1\n  line 2\n";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 2,
                    old_lines: 1,
                    new_start: 2,
                    new_lines: 1,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 2",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Addition,
                            content: "  line 2",
                            old_line_no: None,
                            new_line_no: Some(
                                2,
                            ),
                        },
                    ],
                },
            ]
            "#);
        }

        #[test]
        fn test_generate_hunks_real_code_example() {
            let old = r#"fn main() {
    println!("Hello, world!");
}
"#;
            let new = r#"fn main() {
    let name = "World";
    println!("Hello, {}!", name);
}
"#;
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Myers);

            insta::assert_debug_snapshot!(hunks, @r###"
            [
                DiffHunk {
                    old_start: 2,
                    old_lines: 1,
                    new_start: 2,
                    new_lines: 2,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "    println!(\"Hello, world!\");",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Addition,
                            content: "    let name = \"World\";",
                            old_line_no: None,
                            new_line_no: Some(
                                2,
                            ),
                        },
                        LineChange {
                            change_type: Addition,
                            content: "    println!(\"Hello, {}!\", name);",
                            old_line_no: None,
                            new_line_no: Some(
                                3,
                            ),
                        },
                    ],
                },
            ]
            "###);
        }

        #[test]
        fn test_generate_hunks_histogram_algorithm() {
            let old = "line 1\nline 2\nline 3\n";
            let new = "line 1\nmodified line 2\nline 3\n";
            let hunks = GitRepository::generate_hunks(Some(old), Some(new), Algorithm::Histogram);

            insta::assert_debug_snapshot!(hunks, @r#"
            [
                DiffHunk {
                    old_start: 2,
                    old_lines: 1,
                    new_start: 2,
                    new_lines: 1,
                    lines: [
                        LineChange {
                            change_type: Deletion,
                            content: "line 2",
                            old_line_no: Some(
                                2,
                            ),
                            new_line_no: None,
                        },
                        LineChange {
                            change_type: Addition,
                            content: "modified line 2",
                            old_line_no: None,
                            new_line_no: Some(
                                2,
                            ),
                        },
                    ],
                },
            ]
            "#);
        }
    }
}
