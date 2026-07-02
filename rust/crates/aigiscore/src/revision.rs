//! Content-addressable change substrate for online (incremental) analysis.
//!
//! Phase 1 of the online code-graph work (see `docs/ONLINE_CODE_GRAPH_ARCHITECTURE.md`)
//! only needs these identities to *exist and be threaded*, not to drive incremental
//! recomputation yet. They make "unchanged since last run" representable, which the
//! previous `ScannedFile { relative_path, size_bytes }` model could not express.
//!
//! Nothing here is cryptographic: [`ContentHash`]/[`SemanticEnvFingerprint`] are
//! non-cryptographic identities for local change detection only — never treat them as
//! collision-proof or security-sensitive.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Monotonic counter for observed/indexed repository state. Advanced by the daemon as
/// filesystem changes are observed and as snapshots are published.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct RepoRevision(pub u64);

impl RepoRevision {
    pub const ZERO: Self = Self(0);

    /// Saturating successor — a revision counter must never wrap.
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl fmt::Display for RepoRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Monotonic counter for the semantic environment (build/workspace config that can change
/// a file's meaning without changing its text). Advances only when the environment
/// fingerprint changes, independently of source-content revisions.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct SemanticEnvRevision(pub u64);

impl SemanticEnvRevision {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// Non-cryptographic content identity of a single file (xxh3-64).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentHash(pub u64);

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Stable lowercase hex for logs/contracts.
        write!(f, "{:016x}", self.0)
    }
}

/// Non-cryptographic fingerprint of the whole semantic environment (xxh3-128).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SemanticEnvFingerprint(pub u128);

impl fmt::Display for SemanticEnvFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

/// Stable identity of a file as `(path, content_hash)` — the key every derived fact will
/// eventually be tagged with so unchanged files can be reused across runs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct FileContentKey {
    pub relative_path: PathBuf,
    pub content_hash: ContentHash,
}

impl FileContentKey {
    pub fn new(relative_path: impl Into<PathBuf>, content_hash: ContentHash) -> Self {
        Self {
            relative_path: relative_path.into(),
            content_hash,
        }
    }
}

/// Filesystem modification time. Diagnostic / future stable-read heuristic only — it is
/// **not** identity (`ContentHash` is). Kept separate so nothing mistakes it for one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMtime {
    pub unix_seconds: i64,
    pub nanos: u32,
}

impl FileMtime {
    /// Convert a `SystemTime` to a `FileMtime`, tolerating pre-epoch times.
    pub fn from_system_time(time: std::time::SystemTime) -> Self {
        match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(delta) => Self {
                unix_seconds: delta.as_secs() as i64,
                nanos: delta.subsec_nanos(),
            },
            Err(err) => {
                // Time before the epoch: record a negative second offset.
                let delta = err.duration();
                Self {
                    unix_seconds: -(delta.as_secs() as i64),
                    nanos: delta.subsec_nanos(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revisions_advance_and_saturate() {
        assert_eq!(RepoRevision::ZERO.next(), RepoRevision(1));
        assert_eq!(RepoRevision(u64::MAX).next(), RepoRevision(u64::MAX));
        assert!(RepoRevision(2) > RepoRevision(1));
    }

    #[test]
    fn hashes_render_as_fixed_width_hex() {
        assert_eq!(ContentHash(0xab).to_string(), "00000000000000ab");
        assert_eq!(
            SemanticEnvFingerprint(0xff).to_string(),
            "000000000000000000000000000000ff"
        );
    }

    #[test]
    fn file_key_pairs_path_and_hash() {
        let key = FileContentKey::new("src/main.rs", ContentHash(7));
        assert_eq!(key.content_hash, ContentHash(7));
        assert_eq!(key.relative_path.to_str(), Some("src/main.rs"));
    }
}
