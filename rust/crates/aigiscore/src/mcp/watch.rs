//! One index writer, with observation armed before capture and directory watches
//! derived from scan scope. Notifications invalidate immediately; rebuilds coalesce.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

use notify::event::ModifyKind;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::contracts::WatcherStatus;
use super::live::{DirtyKind, LiveState};
use super::McpState;
use crate::ingestion::scan::{watch_directories, ScanConfig};
use crate::resolve::load_resolve_config;

const DEBOUNCE: Duration = Duration::from_millis(300);
const WATCH_RETRY: Duration = Duration::from_secs(2);

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

fn is_ignored(path: &Path, root: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return true;
    };
    if relative.components().next().is_some_and(|part| {
        part.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(".aigiscode")
    }) {
        if relative.components().count() == 1 {
            return false;
        }
        return relative.components().count() != 2
            || !relative
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    ["scan.json", "policy.json", "rules.json", "doctrine.json"]
                        .iter()
                        .any(|control| name.eq_ignore_ascii_case(control))
                });
    }

    // File configuration extends this default set; it cannot remove these entries.
    // Other hidden inputs are observed whenever the effective scan admits them.
    static DEFAULT_IGNORES: OnceLock<HashSet<String>> = OnceLock::new();
    let ignored = DEFAULT_IGNORES.get_or_init(|| ScanConfig::default().ignored_dir_names);
    relative
        .components()
        .any(|part| ignored.contains(part.as_os_str().to_string_lossy().as_ref()))
}

struct InputWatcher {
    _watcher: RecommendedWatcher,
    topology_changed: Arc<AtomicBool>,
}

impl InputWatcher {
    fn arm(
        live: Arc<LiveState<Option<McpState>>>,
        root: &Path,
    ) -> Result<Self, String> {
        let topology_changed = Arc::new(AtomicBool::new(false));
        let observed_topology = Arc::clone(&topology_changed);
        let config_inputs = Arc::new(RwLock::new(HashSet::<PathBuf>::new()));
        let observed_inputs = Arc::clone(&config_inputs);
        let filter_root = root.to_path_buf();
        let mut watcher =
            notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                let event = match result {
                    Ok(event) if !event.need_rescan() => event,
                    result => {
                        let message = match result {
                            Err(error) => error.to_string(),
                            Ok(_) => {
                                String::from("filesystem notification stream requires a rescan")
                            }
                        };
                        observed_topology.store(true, Ordering::Release);
                        live.mark_dirty([(PathBuf::from("."), DirtyKind::Other)]);
                        live.set_watcher_status(WatcherStatus::Failed, Some(message));
                        return;
                    }
                };
                if event.kind.is_access() {
                    return;
                }
                let kind = classify(&event.kind);
                let inputs = observed_inputs.read().unwrap();
                let changes = event
                    .paths
                    .iter()
                    .filter(|path| !is_ignored(path, &filter_root)
                        || inputs.iter().any(|input| input.starts_with(path)))
                    .map(|path| path.strip_prefix(&filter_root).unwrap_or(Path::new(".")))
                    .map(|path| {
                        (
                            if path.as_os_str().is_empty() {
                                PathBuf::from(".")
                            } else {
                                path.to_path_buf()
                            },
                            kind,
                        )
                    })
                    .collect::<Vec<_>>();
                let config_changed = event.paths.iter().any(|path| inputs.iter().any(|input| input.starts_with(path)));
                drop(inputs);
                if changes.is_empty() {
                    return;
                }
                // Invalidate before debounce or rebuild work, including events arriving
                // during a build. A single wake token coalesces without losing paths.
                let topology_event = config_changed || event.kind.is_create()
                    || event.kind.is_remove()
                    || matches!(
                        event.kind,
                        EventKind::Modify(ModifyKind::Name(_)) | EventKind::Any | EventKind::Other
                    )
                    || changes.iter().any(|(path, _)| {
                        path.to_string_lossy()
                            .replace('\\', "/")
                            .eq_ignore_ascii_case(".aigiscode/scan.json")
                    });
                if topology_event {
                    observed_topology.store(true, Ordering::Release);
                }
                live.mark_dirty(changes);
            })
            .map_err(|error| error.to_string())?;

        // Register each parent before descending into its children. No recursive
        // registration of vendor/build trees; empty source directories still count.
        watcher
            .watch(root, RecursiveMode::NonRecursive)
            .map_err(|error| error.to_string())?;
        let mut directories = HashSet::from([root.to_path_buf()]);
        for entry in watch_directories(root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            if entry.path() != root {
                watcher
                    .watch(entry.path(), RecursiveMode::NonRecursive)
                    .map_err(|error| error.to_string())?;
                directories.insert(entry.path().to_path_buf());
            }
        }
        let config_candidates = directories.iter().filter_map(|directory| {
            directory.strip_prefix(root).ok().map(|directory| directory.join("tsconfig.json"))
        }).collect::<Vec<_>>();
        let config = load_resolve_config(root, &config_candidates).map_err(|error| error.to_string())?;
        *config_inputs.write().unwrap() = config.input_paths.iter().cloned().collect();
        for input in config.input_paths {
            // Extended/package configuration may sit in an excluded tree or
            // outside the repo. Watch its nearest existing parent, including
            // absence so later directory/file creation is observed.
            for parent in input.ancestors().skip(1) {
                if directories.contains(parent) { break; }
                match std::fs::metadata(parent) {
                    Ok(metadata) if metadata.is_dir() => {
                        watcher.watch(parent, RecursiveMode::NonRecursive).map_err(|error| error.to_string())?;
                        directories.insert(parent.to_path_buf());
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
                    Err(error) => return Err(error.to_string()),
                    _ => {},
                }
            }
        }
        Ok(Self {
            _watcher: watcher,
            topology_changed,
        })
    }
}

pub(super) fn start_indexer(
    live: Arc<LiveState<Option<McpState>>>,
    root: PathBuf,
    output_dir: Option<PathBuf>,
    write_artifacts: bool,
    write_kuzu: bool,
    watch: bool,
) {
    if watch {
        live.set_watcher_status(WatcherStatus::Starting, None);
        if live.load().snapshot.is_some() {
            live.mark_dirty([(PathBuf::from("."), DirtyKind::Other)]);
        }
    }
    let mut changes = live.subscribe_changes();
    tokio::spawn(async move {
        let root = match root.canonicalize() {
            Ok(root) => root,
            Err(error) => {
                let message = format!("cannot resolve analysis root {}: {error}", root.display());
                if watch {
                    live.set_watcher_status(WatcherStatus::Failed, Some(message.clone()));
                }
                live.record_error(message);
                return;
            }
        };
        // Experimental until the differential regression gate has been approved.
        let incremental_resolution = watch && std::env::var("AIGISCORE_INCREMENTAL_RESOLVE").as_deref() == Ok("1");
        let mut resolver = incremental_resolution.then(crate::resolve::ResolutionCache::default);
        let incremental_scan = watch && std::env::var("AIGISCORE_INCREMENTAL_SCAN").as_deref() == Ok("1");
        let mut scanner = incremental_scan.then(crate::scanners::ast_grep::AstGrepScanCache::default);
        let mut watcher = None::<InputWatcher>;
        let mut immediate = true;
        let mut watch_failed = false;
        loop {
            if !immediate {
                if !watch {
                    break;
                }
                if watch_failed {
                    tokio::select! {
                        _ = tokio::time::sleep(WATCH_RETRY) => {},
                        _ = changes.changed() => {},
                    }
                } else if changes.changed().await.is_err() {
                    break;
                }
                tokio::time::sleep(DEBOUNCE).await;
            }
            changes.borrow_and_update();
            if watch {
                // Keep the previous watches alive until their replacement is armed.
                // This also repairs moved/deleted directories and changed scan scope.
                match InputWatcher::arm(Arc::clone(&live), &root) {
                    Ok(armed) => {
                        watcher = Some(armed);
                        watch_failed = false;
                        live.set_watcher_status(WatcherStatus::Watching, None);
                    }
                    Err(error) => {
                        watch_failed = true;
                        live.mark_dirty([(PathBuf::from("."), DirtyKind::Other)]);
                        live.set_watcher_status(WatcherStatus::Failed, Some(error));
                    }
                }
            }
            let target = live.begin_rebuild().max(1);
            let initial = live.load().snapshot.is_none();
            let build_root = root.clone();
            let build_output = output_dir.clone();
            let mut build_resolver = resolver.take();
            let mut build_scanner = scanner.take();
            let result = tokio::task::spawn_blocking(move || {
                let result = super::build_mcp_state_with_caches(
                    &build_root,
                    build_output.as_deref(),
                    initial && write_artifacts,
                    initial && write_kuzu,
                    build_resolver.as_mut(),
                    build_scanner.as_mut(),
                ).map_err(|error| error.to_string());
                (result, build_resolver, build_scanner)
            })
            .await;
            // A directory/scope event during registration may have introduced an
            // unwatched subtree before target was sampled. Force reconciliation;
            // never publish that capture as proven fresh.
            if watcher
                .as_ref()
                .is_some_and(|watcher| watcher.topology_changed.load(Ordering::Acquire))
            {
                live.mark_dirty([(PathBuf::from("."), DirtyKind::Other)]);
            }
            match result {
                Ok((Ok(state), updated_resolver, updated_scanner)) => {
                    resolver = updated_resolver;
                    scanner = updated_scanner;
                    live.publish(Some(state), target);
                    eprintln!("aigiscode mcp: published revision {target}");
                }
                Ok((Err(message), updated_resolver, updated_scanner)) => {
                    resolver = updated_resolver;
                    scanner = updated_scanner;
                    eprintln!("aigiscode mcp: {message}");
                    live.record_error(message);
                }
                Err(error) => {
                    // The failed worker owned the cache; start fresh after a panic.
                    resolver = incremental_resolution.then(crate::resolve::ResolutionCache::default);
                    scanner = incremental_scan.then(crate::scanners::ast_grep::AstGrepScanCache::default);
                    live.record_error(format!("analysis task failed: {error}"));
                }
            }
            changes.borrow_and_update();
            immediate = !watch_failed && live.observed() > target;
            if !watch {
                break;
            }
        }
    });
}

#[cfg(test)]
pub(super) fn spawn_watch(live: Arc<LiveState<Option<McpState>>>, root: PathBuf) {
    start_indexer(live, root, None, false, false, true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_artifacts_and_vcs_but_keeps_control_and_hidden_inputs() {
        let root = Path::new("/repo");
        assert!(is_ignored(Path::new("/repo/.aigiscode/x.json"), root));
        assert!(is_ignored(Path::new("/repo/.git/HEAD"), root));
        assert!(is_ignored(Path::new("/repo/target/debug/app"), root));
        assert!(is_ignored(Path::new("/repo/node_modules/x/index.js"), root));
        assert!(!is_ignored(Path::new("/repo/.aigiscode/scan.json"), root));
        assert!(!is_ignored(Path::new("/repo/.aigiscode/rules.json"), root));
        assert!(!is_ignored(Path::new("/repo/.hidden/file"), root));
        assert!(!is_ignored(Path::new("/repo/src/main.rs"), root));
    }
}
