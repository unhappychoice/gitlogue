use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Commit as Git2Commit, Delta, DiffOptions, Oid, Repository};
use rand::seq::SliceRandom;
use std::path::Path;

pub struct GitRepository {
    repo: Repository,
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
    pub old_line_no: Option<usize>,
    pub new_line_no: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<LineChange>,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub old_path: Option<String>,
    pub status: FileStatus,
    pub is_binary: bool,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub hunks: Vec<DiffHunk>,
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

impl GitRepository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path)
            .context("Failed to open Git repository")?;
        Ok(Self { repo })
    }

    pub fn get_commit(&self, hash: &str) -> Result<CommitMetadata> {
        let obj = self.repo
            .revparse_single(hash)
            .context("Invalid commit hash or commit not found")?;

        let commit = obj
            .peel_to_commit()
            .context("Object is not a commit")?;

        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    pub fn random_commit(&self) -> Result<CommitMetadata> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        let non_merge_commits: Vec<Oid> = revwalk
            .filter_map(|oid| oid.ok())
            .filter(|oid| {
                self.repo
                    .find_commit(*oid)
                    .map(|c| c.parent_count() <= 1)
                    .unwrap_or(false)
            })
            .collect();

        if non_merge_commits.is_empty() {
            anyhow::bail!("No non-merge commits found in repository");
        }

        let oid = non_merge_commits
            .choose(&mut rand::thread_rng())
            .context("Failed to select random commit")?;

        let commit = self.repo.find_commit(*oid)?;
        Self::extract_metadata_with_changes(&self.repo, &commit)
    }

    fn extract_metadata_with_changes(
        repo: &Repository,
        commit: &Git2Commit,
    ) -> Result<CommitMetadata> {
        let hash = commit.id().to_string();
        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let timestamp = author.when().seconds();
        let date = DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| Utc::now());
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
        let commit_tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        diff_opts.context_lines(3);

        let diff = repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&commit_tree),
            Some(&mut diff_opts),
        )?;

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
                delta.old_file().path().and_then(|p| p.to_str()).map(String::from)
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
                            if !blob.is_binary() {
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
                        if !blob.is_binary() {
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
                                    let content = String::from_utf8_lossy(line.content()).to_string();
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

            changes.push(FileChange {
                path,
                old_path,
                status,
                is_binary,
                old_content,
                new_content,
                hunks,
                diff: diff_text,
            });
        }

        Ok(changes)
    }
}
