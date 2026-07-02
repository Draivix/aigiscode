//! Filesystem watcher + coalescing rebuild loop for the online MCP daemon.
//!
//! Phase 1 is deliberately "dumb but correct": on any coalesced change the whole project
//! is re-analyzed and the resulting snapshot is atomically published. There is no
//! incremental resolution yet — the value here is *liveness + honest freshness*, not
//! speed. The watcher is a hint stream feeding a dirty set, never a transaction log.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use notify_debouncer_full::new_debouncer;
use notify_debouncer_full::notify::event::EventKind;
use notify_debouncer_full::notify::RecursiveMode;
use notify_debouncer_full::DebounceEventResult;

use super::live::{DirtyKind, LiveState};
use super::McpState;

/// Trailing debounce window the native watcher coalesces raw events into.
const DEBOUNCE_MS: u64 = 300;

/// Directory names never worth reacting to (build/artifact/vcs/dependency dirs). Includes
/// `.aigiscode` so the daemon's own artifact writes can never retrigger it.
const IGNORED_DIR_NAMES: &[&str] = &[
    ".aigiscode",
    ".git",
    "target",
    "node_modules",
    "vendor",
    "dist",
    "build",
    "storage",
    "coverage",
    "__pycache__",
    "tmp",
];

fn classify(kind: &EventKind) -> DirtyKind {
    if kind.is_create() {
        DirtyKind::Created
    } else if kind.is_remove() {
        DirtyKind::Deleted
    } else if kind.is_modify() {
        DirtyKind::Modified
    } else {
        DirtyKind::Other
    }
}

/// A path is ignored if any component under the watched root is a hidden dir or a known
/// build/artifact directory — keeps vendored trees out of scope and avoids self-trigger.
fn is_ignored(path: &Path, root: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        IGNORED_DIR_NAMES.contains(&name.as_ref()) || name.starts_with('.')
    })
}

/// Start watching `root`; each coalesced change re-analyzes the project and publishes a
/// fresh snapshot into `live`. Returns once the watcher is armed; the rebuild loop runs on
/// a spawned task for the daemon's lifetime. Failures to arm the watcher are logged (the
/// server still serves the initial snapshot) rather than fatal.
pub(super) fn spawn_watch(live: Arc<LiveState<McpState>>, root: PathBuf) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<(PathBuf, DirtyKind)>>();

    let filter_root = root.clone();
    let debouncer = new_debouncer(
        Duration::from_millis(DEBOUNCE_MS),
        None,
        move |result: DebounceEventResult| {
            let Ok(events) = result else { return };
            let mut changes: Vec<(PathBuf, DirtyKind)> = Vec::new();
            let mut seen: HashSet<PathBuf> = HashSet::new();
            for event in events {
                if event.event.kind.is_access() {
                    continue; // reads never change content
                }
                if std::env::var_os("AIGISCODE_WATCH_DEBUG").is_some() {
                    eprintln!(
                        "aigiscode watch DEBUG: kind={:?} paths={:?}",
                        event.event.kind, event.event.paths
                    );
                }
                let kind = classify(&event.event.kind);
                for path in &event.event.paths {
                    if is_ignored(path, &filter_root) {
                        continue;
                    }
                    if seen.insert(path.clone()) {
                        changes.push((path.clone(), kind));
                    }
                }
            }
            if !changes.is_empty() {
                let _ = tx.send(changes);
            }
        },
    );

    let mut debouncer = match debouncer {
        Ok(debouncer) => debouncer,
        Err(err) => {
            eprintln!("aigiscode watch: failed to start filesystem watcher: {err}");
            return;
        }
    };
    if let Err(err) = debouncer.watch(&root, RecursiveMode::Recursive) {
        eprintln!("aigiscode watch: failed to watch {}: {err}", root.display());
        return;
    }
    eprintln!("aigiscode watch: watching {} for changes", root.display());

    let rebuild_root = root;
    tokio::spawn(async move {
        // Keep the debouncer alive for the whole task; dropping it stops watching.
        let _debouncer = debouncer;
        while let Some(first) = rx.recv().await {
            live.mark_dirty(first);
            while let Ok(more) = rx.try_recv() {
                live.mark_dirty(more);
            }
            // Rebuild to the latest observed revision, repeating if edits land mid-build.
            loop {
                let target = live.begin_rebuild();
                let root_for_build = rebuild_root.clone();
                let result = tokio::task::spawn_blocking(move || {
                    // Daemon rebuilds never write artifacts (avoids self-trigger; the
                    // watcher ignores `.aigiscode` regardless) and skip Kuzu.
                    super::build_mcp_state(&root_for_build, None, false, false)
                        .map_err(|err| err.to_string())
                })
                .await;
                match result {
                    Ok(Ok(state)) => {
                        live.publish(state, target);
                        eprintln!("aigiscode watch: published revision {target}");
                    }
                    Ok(Err(message)) => {
                        eprintln!("aigiscode watch: rebuild failed: {message}");
                        live.record_error(message);
                    }
                    Err(join_err) => {
                        let message = format!("rebuild task panicked: {join_err}");
                        eprintln!("aigiscode watch: {message}");
                        live.record_error(message);
                    }
                }
                // Absorb any changes observed during the rebuild, then decide if the
                // freshly published revision already covers everything.
                while let Ok(more) = rx.try_recv() {
                    live.mark_dirty(more);
                }
                if live.observed() <= target {
                    break;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_artifact_and_vcs_and_hidden_dirs() {
        let root = Path::new("/repo");
        assert!(is_ignored(Path::new("/repo/.aigiscode/x.json"), root));
        assert!(is_ignored(Path::new("/repo/.git/HEAD"), root));
        assert!(is_ignored(Path::new("/repo/target/debug/app"), root));
        assert!(is_ignored(Path::new("/repo/node_modules/x/index.js"), root));
        assert!(is_ignored(Path::new("/repo/.hidden/file"), root));
        assert!(!is_ignored(Path::new("/repo/src/main.rs"), root));
        assert!(!is_ignored(
            Path::new("/repo/app/Http/Controller.php"),
            root
        ));
    }
}
