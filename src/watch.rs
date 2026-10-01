use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::git::GitRepository;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
/// When more commits than this are pending, the oldest ones are dropped.
const MAX_PENDING: usize = 100;
/// Typing speed multiplier reached when the queue is full.
const MAX_SPEED_MULTIPLIER: f64 = 10.0;

/// Watches the repository HEAD and queues newly arrived commits for playback.
pub struct CommitWatcher {
    last_head: Option<String>,
    last_branch: Option<String>,
    /// Branch and its tip recorded when HEAD became detached.
    detached_from: Option<(String, Option<String>)>,
    pending: VecDeque<String>,
    next_poll: Instant,
}

impl CommitWatcher {
    pub fn new(repo: &GitRepository, now: Instant) -> Self {
        Self {
            last_head: repo.head_commit_id(),
            last_branch: repo.head_branch_name(),
            detached_from: None,
            pending: VecDeque::new(),
            next_poll: now + POLL_INTERVAL,
        }
    }

    /// Checks HEAD for new commits once the poll interval has elapsed.
    pub fn poll(&mut self, repo: &GitRepository, now: Instant) {
        if now < self.next_poll {
            return;
        }
        self.next_poll = now + POLL_INTERVAL;
        self.check_head(repo);
    }

    /// Takes the oldest pending commit hash.
    pub fn next_commit(&mut self) -> Option<String> {
        self.pending.pop_front()
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    fn check_head(&mut self, repo: &GitRepository) {
        let Some(head) = repo.head_commit_id() else {
            return;
        };
        let branch_switched = self.update_branch(repo.head_branch_name(), &head);
        if branch_switched || self.last_head.as_deref() == Some(head.as_str()) {
            self.last_head = Some(head);
            return;
        }
        let new_commits = repo
            .commit_ids_between(self.last_head.as_deref(), &head)
            .unwrap_or_else(|_| vec![head.clone()]);
        self.enqueue(new_commits);
        self.last_head = Some(head);
    }

    /// Records the checked-out branch and reports whether HEAD moved without new commits:
    /// a switch to another branch, or a return to an unchanged branch after a detached checkout.
    /// Detached HEAD itself (e.g. mid-rebase) is ignored so rebased commits are still replayed.
    fn update_branch(&mut self, branch: Option<String>, head: &str) -> bool {
        let Some(branch) = branch else {
            self.remember_detached_tip();
            return false;
        };
        let returned_unchanged = self
            .detached_from
            .take()
            .is_some_and(|(detached, tip)| detached == branch && tip.as_deref() == Some(head));
        let switched = self
            .last_branch
            .as_ref()
            .is_some_and(|last| *last != branch);
        self.last_branch = Some(branch);
        switched || returned_unchanged
    }

    fn remember_detached_tip(&mut self) {
        if self.detached_from.is_none() {
            self.detached_from = self
                .last_branch
                .clone()
                .map(|branch| (branch, self.last_head.clone()));
        }
    }

    fn enqueue(&mut self, commits: Vec<String>) {
        self.pending.extend(commits);
        let overflow = self.pending.len().saturating_sub(MAX_PENDING);
        self.pending.drain(..overflow);
    }
}

/// Speeds up typing linearly from 1x (empty queue) to `MAX_SPEED_MULTIPLIER` (full queue).
pub fn adaptive_speed_multiplier(pending: usize) -> f64 {
    let fill = pending.min(MAX_PENDING) as f64 / MAX_PENDING as f64;
    1.0 + (MAX_SPEED_MULTIPLIER - 1.0) * fill
}

#[cfg(test)]
mod tests {
    use super::*;

    fn watcher_with_pending(pending: &[&str]) -> CommitWatcher {
        CommitWatcher {
            last_head: None,
            last_branch: None,
            detached_from: None,
            pending: pending.iter().map(|hash| hash.to_string()).collect(),
            next_poll: Instant::now(),
        }
    }

    fn hashes(count: usize) -> Vec<String> {
        (0..count).map(|i| format!("commit{i}")).collect()
    }

    #[test]
    fn enqueue_keeps_commits_in_arrival_order() {
        let mut watcher = watcher_with_pending(&["a"]);
        watcher.enqueue(vec!["b".to_string(), "c".to_string()]);

        assert_eq!(watcher.next_commit().as_deref(), Some("a"));
        assert_eq!(watcher.next_commit().as_deref(), Some("b"));
        assert_eq!(watcher.next_commit().as_deref(), Some("c"));
        assert_eq!(watcher.next_commit(), None);
    }

    #[test]
    fn enqueue_drops_oldest_commits_when_queue_overflows() {
        let mut watcher = watcher_with_pending(&[]);
        watcher.enqueue(hashes(MAX_PENDING + 2));

        assert_eq!(watcher.pending_len(), MAX_PENDING);
        assert_eq!(watcher.next_commit().as_deref(), Some("commit2"));
    }

    #[test]
    fn update_branch_reports_switch_only_between_named_branches() {
        let mut watcher = watcher_with_pending(&[]);

        assert!(!watcher.update_branch(Some("main".to_string()), "a"));
        assert!(!watcher.update_branch(None, "b"));
        assert!(!watcher.update_branch(Some("main".to_string()), "c"));
        assert!(watcher.update_branch(Some("feature".to_string()), "c"));
    }

    #[test]
    fn update_branch_reports_return_to_unchanged_tip_after_detaching() {
        let mut watcher = watcher_with_pending(&[]);
        watcher.last_branch = Some("main".to_string());
        watcher.last_head = Some("tip".to_string());

        assert!(!watcher.update_branch(None, "ancestor"));
        watcher.last_head = Some("ancestor".to_string());

        assert!(watcher.update_branch(Some("main".to_string()), "tip"));
    }

    #[test]
    fn update_branch_keeps_replaying_when_branch_tip_changed_while_detached() {
        let mut watcher = watcher_with_pending(&[]);
        watcher.last_branch = Some("main".to_string());
        watcher.last_head = Some("tip".to_string());

        assert!(!watcher.update_branch(None, "onto"));

        assert!(!watcher.update_branch(Some("main".to_string()), "rebased"));
    }

    #[test]
    fn adaptive_speed_multiplier_scales_linearly_up_to_full_queue() {
        assert_eq!(adaptive_speed_multiplier(0), 1.0);
        assert_eq!(adaptive_speed_multiplier(MAX_PENDING / 2), 5.5);
        assert_eq!(adaptive_speed_multiplier(MAX_PENDING), MAX_SPEED_MULTIPLIER);
    }
}
