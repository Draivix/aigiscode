//! Agent edit receipts invalidate before waiting for the filesystem hint stream.

use super::contracts::{RecordChangedPathsOutput, WatcherStatus};
use super::live::DirtyKind;
use super::AigiscodeMcpServer;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

const MAX_EDIT_PATHS: usize = 128;
const MAX_PATH_BYTES: usize = 4096;

pub(super) fn path_in_scope(file: &str, scope: &str) -> bool {
    !file.is_empty()
        && !scope.is_empty()
        && (scope == "." || Path::new(file).starts_with(scope) || Path::new(scope).starts_with(file))
}

fn normalize_paths(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    if paths.is_empty() || paths.len() > MAX_EDIT_PATHS {
        return Err(format!(
            "paths must contain between 1 and {MAX_EDIT_PATHS} entries"
        ));
    }
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for raw in paths {
        if raw.trim().is_empty() || raw.len() > MAX_PATH_BYTES || raw.contains('\0') {
            return Err(String::from(
                "paths must be nonempty and at most 4096 bytes, without NUL",
            ));
        }
        let portable = raw.replace('\\', "/");
        // Reject Windows drive paths even on a Unix server.
        if portable.as_bytes().get(1) == Some(&b':')
            && portable.as_bytes()[0].is_ascii_alphabetic()
        {
            return Err(String::from(
                "paths must be repository-relative, without a drive prefix",
            ));
        }
        let mut path = PathBuf::new();
        for component in Path::new(&portable).components() {
            match component {
                Component::Normal(part) => path.push(part),
                Component::CurDir => {}
                _ => {
                    return Err(String::from(
                        "paths must be repository-relative, without traversal",
                    ));
                }
            }
        }
        if path.as_os_str().is_empty() {
            path.push(".");
        }
        if seen.insert(path.clone()) {
            normalized.push(path);
        }
    }
    Ok(normalized)
}

impl AigiscodeMcpServer {
    pub(super) fn record_edits(&self, paths: &[String]) -> Result<RecordChangedPathsOutput, String> {
        // Validate the entire batch before changing revision or dirty paths. No
        // existence check: removals, new files and not-yet-watched directories count.
        let paths = normalize_paths(paths)?;
        if !self.live.has_index_writer()
            || !matches!(
                self.live.freshness(true).watcher,
                WatcherStatus::Starting | WatcherStatus::Watching | WatcherStatus::Failed
            )
        {
            return Err(String::from(
                "record_changed_paths requires an active mcp --watch indexer",
            ));
        }
        let min_revision = self
            .live
            .mark_dirty(paths.iter().cloned().map(|path| (path, DirtyKind::Other)));
        Ok(RecordChangedPathsOutput {
            min_revision,
            paths: paths
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            freshness: self.live.freshness(true),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_paths_preserve_boundaries_and_deduplicate_normalized_names() {
        let paths = ["./src/Panel.vue", "src\\Panel.vue", "deleted/file.ts", ".", "src/část A.ts"]
            .map(String::from);
        assert_eq!(normalize_paths(&paths).unwrap(), [
            "src/Panel.vue", "deleted/file.ts", ".", "src/část A.ts",
        ].map(PathBuf::from));
        assert!(path_in_scope("src/Panel.vue", "src"));
        assert!(path_in_scope("src/Panel.vue", "."));
        assert!(!path_in_scope("src/Panel.vue", "Panel.vue"));
        assert!(!path_in_scope("src/Panel.vue", "src/Panel"));
        assert!(!path_in_scope("src/Panel.vue", ""));
    }

    #[test]
    fn invalid_edit_batches_do_not_partially_invalidate_the_index() {
        let server = AigiscodeMcpServer::new_pending();
        let initial = server.live.observed();
        for invalid in ["", "   ", "../outside", "src/../outside", "/absolute", "C:\\outside", "\\\\server\\share", "src/\0bad"] {
            let paths = [String::from("valid.rs"), String::from(invalid)];
            assert!(normalize_paths(&paths).is_err());
            assert!(server.record_edits(&paths).is_err());
            assert_eq!(server.live.observed(), initial);
            assert_eq!(server.live.freshness(true).dirty_path_count, 0);
        }
        assert!(normalize_paths(&[]).is_err());
        assert!(normalize_paths(&vec![String::from("a"); MAX_EDIT_PATHS + 1]).is_err());
        assert!(normalize_paths(&["a".repeat(MAX_PATH_BYTES + 1)]).is_err());
    }

    #[test]
    fn a_one_shot_server_does_not_acknowledge_an_unavailable_rebuild() {
        let server = AigiscodeMcpServer::new_pending();
        let initial = server.live.observed();
        let error = server.record_edits(&[String::from("src/new.rs")]).unwrap_err();
        assert!(error.contains("active mcp --watch"));
        assert_eq!(server.live.observed(), initial);
    }
}
