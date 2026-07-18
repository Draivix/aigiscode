//! Live, revisioned state for the online MCP daemon.
//!
//! The load-bearing invariant (Phase 1 of `docs/ONLINE_CODE_GRAPH_ARCHITECTURE.md`):
//! a reader always gets the latest *published* snapshot immediately, but is told —
//! via [`Freshness`] — when the daemon has *observed* newer changes than that snapshot
//! represents. Never lie about freshness; over-report staleness rather than under-report.
//!
//! Generic over the snapshot type `S` so the revision/staleness logic is unit-testable
//! without constructing a full `McpState`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arc_swap::ArcSwap;
use tokio::sync::watch;

use super::contracts::Freshness;

/// Max dirty paths embedded in a [`Freshness`] response (count is always exact).
const DIRTY_SAMPLE_CAP: usize = 50;

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// How a watched path most recently changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DirtyKind {
    Created,
    Modified,
    Deleted,
    Other,
}

#[derive(Debug, Clone, Copy)]
struct DirtyInfo {
    last_observed: u64,
    #[allow(dead_code)]
    kind: DirtyKind,
}

/// A published snapshot paired with the revision + generation time it represents, so a
/// reader gets the snapshot and its true revision atomically.
pub(super) struct Published<S> {
    pub revision: u64,
    pub generated_at_ms: u64,
    pub snapshot: S,
}

#[derive(Debug)]
struct LiveMeta {
    observed: u64,
    dirty: BTreeMap<PathBuf, DirtyInfo>,
    rebuilding: bool,
    last_error: Option<String>,
}

/// Shared live state: an atomically-swappable published snapshot plus observed-change
/// bookkeeping. Cloneable `Arc` handle is shared between the MCP server (readers) and the
/// watcher/rebuild task (the single writer).
pub(super) struct LiveState<S> {
    current: ArcSwap<Published<S>>,
    meta: Mutex<LiveMeta>,
    /// Latest observed revision, readable without taking the meta lock (used by the
    /// rebuild loop to decide whether another pass is needed).
    observed_atomic: AtomicU64,
    /// Fires with the indexed revision on every successful publish, so waiters can block
    /// until a target revision is indexed.
    published_tx: watch::Sender<u64>,
}

impl<S> LiveState<S> {
    /// Seed the live state with an initial snapshot at revision 1.
    pub(super) fn new(initial: S) -> Arc<Self> {
        Self::new_at(initial, 1, false)
    }

    /// Seed the live state at an explicit revision. `revision 0` +
    /// `rebuilding = true` is the pending-initial-index state: the daemon can
    /// answer `initialize` immediately while the first analysis runs, and the
    /// freshness contract reports the truth (nothing indexed yet).
    pub(super) fn new_at(initial: S, revision: u64, rebuilding: bool) -> Arc<Self> {
        let (published_tx, _rx) = watch::channel(revision);
        Arc::new(Self {
            current: ArcSwap::from_pointee(Published {
                revision,
                generated_at_ms: now_unix_ms(),
                snapshot: initial,
            }),
            meta: Mutex::new(LiveMeta {
                observed: revision.max(1),
                dirty: BTreeMap::new(),
                rebuilding,
                last_error: None,
            }),
            observed_atomic: AtomicU64::new(revision.max(1)),
            published_tx,
        })
    }

    /// Latest published snapshot + its revision (single atomic load).
    pub(super) fn load(&self) -> Arc<Published<S>> {
        self.current.load_full()
    }

    /// Highest observed revision (lock-free).
    pub(super) fn observed(&self) -> u64 {
        self.observed_atomic.load(Ordering::Acquire)
    }

    /// Record observed filesystem changes; advances and returns the observed revision.
    pub(super) fn mark_dirty(
        &self,
        changes: impl IntoIterator<Item = (PathBuf, DirtyKind)>,
    ) -> u64 {
        let mut meta = self.meta.lock().unwrap();
        let next = meta.observed.saturating_add(1);
        meta.observed = next;
        for (path, kind) in changes {
            meta.dirty.insert(
                path,
                DirtyInfo {
                    last_observed: next,
                    kind,
                },
            );
        }
        self.observed_atomic.store(next, Ordering::Release);
        next
    }

    /// Mark a rebuild as in flight; returns the target revision it will represent (the
    /// observed revision at the moment the rebuild starts).
    pub(super) fn begin_rebuild(&self) -> u64 {
        let mut meta = self.meta.lock().unwrap();
        meta.rebuilding = true;
        meta.observed
    }

    /// Atomically publish a rebuilt snapshot for `target`. Stores the snapshot first, then
    /// updates bookkeeping, so a reader interleaving the two only ever *over*-reports
    /// staleness. Dirty paths changed *after* the rebuild started (observed > target) are
    /// retained so freshness stays honest under edits-during-rebuild.
    pub(super) fn publish(&self, snapshot: S, target: u64) {
        self.current.store(Arc::new(Published {
            revision: target,
            generated_at_ms: now_unix_ms(),
            snapshot,
        }));
        {
            let mut meta = self.meta.lock().unwrap();
            meta.rebuilding = false;
            meta.last_error = None;
            meta.dirty.retain(|_, info| info.last_observed > target);
        }
        let _ = self.published_tx.send(target);
    }

    /// Record a rebuild failure without replacing the published snapshot (a stale-but-good
    /// snapshot is better than none; dirty paths are kept so the next change retriggers).
    pub(super) fn record_error(&self, message: String) {
        let mut meta = self.meta.lock().unwrap();
        meta.rebuilding = false;
        meta.last_error = Some(message);
    }

    /// Build the freshness contract for the current published snapshot. `consistency_satisfied`
    /// reflects whether a requested `min_revision`/wait was met by the caller.
    pub(super) fn freshness(&self, consistency_satisfied: bool) -> Freshness {
        // Lock meta first, then load the snapshot: any interleaving publish then makes the
        // snapshot look *newer* than the meta, which can only over-report staleness — safe.
        let meta = self.meta.lock().unwrap();
        let published = self.load();
        let indexed = published.revision;
        let observed = meta.observed;
        Freshness {
            revision: indexed,
            indexed_revision: indexed,
            observed_revision: observed,
            is_stale: observed > indexed || !consistency_satisfied,
            rebuilding: meta.rebuilding,
            consistency_satisfied,
            dirty_path_count: meta.dirty.len(),
            dirty_paths: meta
                .dirty
                .keys()
                .take(DIRTY_SAMPLE_CAP)
                .map(|path| path.display().to_string())
                .collect(),
            generated_at_unix_ms: published.generated_at_ms,
        }
    }

    /// Freshness only when it changes what the caller should do — the snapshot is
    /// stale or a rebuild is in flight. `None` means "fresh", so per-call tool
    /// responses stay lean; `repo_overview`/`repo_brief` remain the always-on
    /// freshness surfaces. Never under-report: any doubt yields `Some`.
    pub(super) fn actionable_freshness(&self, consistency_satisfied: bool) -> Option<Freshness> {
        let freshness = self.freshness(consistency_satisfied);
        if freshness.is_stale || freshness.rebuilding {
            Some(freshness)
        } else {
            None
        }
    }

    /// Block (up to `wait_ms`) until `indexed_revision >= target`. Returns whether the
    /// target was reached. `wait_ms == 0` is a pure non-blocking check.
    pub(super) async fn wait_for_revision(&self, target: u64, wait_ms: u64) -> bool {
        if self.load().revision >= target {
            return true;
        }
        if wait_ms == 0 {
            return false;
        }
        let mut rx = self.published_tx.subscribe();
        let _ = tokio::time::timeout(Duration::from_millis(wait_ms), async {
            loop {
                if self.load().revision >= target {
                    return;
                }
                if rx.changed().await.is_err() {
                    return;
                }
            }
        })
        .await;
        self.load().revision >= target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn fresh_after_seed_then_stale_after_change() {
        let live = LiveState::new("v1".to_string());
        let f = live.freshness(true);
        assert_eq!(f.indexed_revision, 1);
        assert_eq!(f.observed_revision, 1);
        assert!(!f.is_stale);
        assert_eq!(f.dirty_path_count, 0);

        live.mark_dirty([(p("src/a.rs"), DirtyKind::Modified)]);
        let f = live.freshness(true);
        assert_eq!(f.indexed_revision, 1);
        assert_eq!(f.observed_revision, 2);
        assert!(f.is_stale, "observed>indexed must report stale");
        assert_eq!(f.dirty_path_count, 1);
        assert!(f.dirty_paths.iter().any(|d| d.contains("a.rs")));
    }

    #[test]
    fn query_during_rebuild_reports_stale_then_fresh_after_publish() {
        let live = LiveState::new("v1".to_string());
        live.mark_dirty([(p("a"), DirtyKind::Modified)]);
        let target = live.begin_rebuild();
        assert_eq!(target, 2);

        // Mid-rebuild: still serving snapshot rev 1, honestly stale + rebuilding.
        let f = live.freshness(true);
        assert_eq!(f.indexed_revision, 1);
        assert!(f.is_stale);
        assert!(f.rebuilding);
        assert_eq!(live.load().snapshot, "v1");

        live.publish("v2".to_string(), target);
        let f = live.freshness(true);
        assert_eq!(f.indexed_revision, 2);
        assert_eq!(f.observed_revision, 2);
        assert!(!f.is_stale);
        assert!(!f.rebuilding);
        assert_eq!(f.dirty_path_count, 0);
        assert_eq!(live.load().snapshot, "v2");
    }

    #[test]
    fn changes_during_rebuild_keep_repo_stale_after_publish() {
        // The most important Phase 1 test: publishing a completed batch must NOT claim
        // freshness when newer edits arrived mid-build.
        let live = LiveState::new("v1".to_string());
        live.mark_dirty([(p("a"), DirtyKind::Modified)]); // observed 2
        let target = live.begin_rebuild(); // target 2
        live.mark_dirty([(p("b"), DirtyKind::Modified)]); // observed 3 (during rebuild)

        live.publish("v2".to_string(), target); // publishes rev 2
        let f = live.freshness(true);
        assert_eq!(f.indexed_revision, 2);
        assert_eq!(f.observed_revision, 3);
        assert!(f.is_stale, "edit during rebuild must keep repo stale");
        assert_eq!(f.dirty_path_count, 1);
        assert!(f.dirty_paths.iter().any(|d| d == "b"));
        assert!(
            !f.dirty_paths.iter().any(|d| d == "a"),
            "a was covered by rev 2"
        );

        // A follow-up rebuild to the latest observed clears it.
        let target = live.begin_rebuild(); // target 3
        live.publish("v3".to_string(), target);
        let f = live.freshness(true);
        assert_eq!(f.indexed_revision, 3);
        assert_eq!(f.observed_revision, 3);
        assert!(!f.is_stale);
        assert_eq!(f.dirty_path_count, 0);
    }

    #[test]
    fn min_revision_not_met_is_reported_unsatisfied() {
        let live = LiveState::new("v1".to_string());
        // consistency_satisfied=false must force is_stale even at rev parity.
        let f = live.freshness(false);
        assert!(!f.consistency_satisfied);
        assert!(f.is_stale);
    }

    #[test]
    fn actionable_freshness_only_appears_when_the_caller_must_act() {
        let live = LiveState::new("v1".to_string());
        assert!(
            live.actionable_freshness(true).is_none(),
            "fresh + satisfied must stay silent to keep per-call responses lean"
        );
        assert!(
            live.actionable_freshness(false).is_some(),
            "unmet consistency is actionable"
        );
        live.mark_dirty([(p("a"), DirtyKind::Modified)]);
        let freshness = live
            .actionable_freshness(true)
            .expect("dirty snapshot is actionable");
        assert!(freshness.is_stale);
    }

    #[tokio::test]
    async fn wait_for_revision_times_out_then_succeeds() {
        let live = LiveState::new("v1".to_string());
        live.mark_dirty([(p("a"), DirtyKind::Modified)]);
        let target = live.begin_rebuild();
        // Not published yet -> a short wait must time out and report unmet.
        assert!(!live.wait_for_revision(target, 30).await);

        live.publish("v2".to_string(), target);
        // Already indexed -> immediate success.
        assert!(live.wait_for_revision(target, 30).await);
    }
}
