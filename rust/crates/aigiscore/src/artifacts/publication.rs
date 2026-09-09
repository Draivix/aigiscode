//! Complete analytical generations with a single commit point. Flat files are
//! compatibility views; readers needing a family pin `current-generation.json`.

use super::{atomic, ArtifactPaths};
use crate::ingestion::hash::hash_file_xxh3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const POINTER: &str = "current-generation.json";
pub(super) const SEAL: &str = "generation-manifest.json";
const GENERATIONS: &str = ".generations";
const LOCK: &str = ".publication.lock";
const VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct Commit {
    version: u32,
    generation: String,
    manifest_xxh3: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Member {
    bytes: u64,
    xxh3: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    version: u32,
    generation: String,
    artifacts: BTreeMap<String, Member>,
}

/// Keeps a cooperative legacy read lock alive. Published generations never change
/// under native writers and do not hold a reader lock or block the next publisher.
pub struct ArtifactSnapshot {
    pub directory: PathBuf,
    pub generation: Option<String>,
    pub identity: Option<super::SnapshotIdentity>,
    _legacy_lease: Option<File>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PublishedArtifactStatus {
    pub directory: PathBuf,
    pub generation: Option<String>,
    /// None for legacy/no exported snapshot. This compares captured source, scan and configuration inputs with
    /// the served index. It does not assert equality of derived reports or baseline context.
    pub inputs_match_index: Option<bool>,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn control_bytes(path: &Path) -> io::Result<Option<Vec<u8>>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !file.metadata()?.is_file() {
        return Err(invalid("publication control input is not a regular file"));
    }
    let mut bytes = Vec::new();
    file.take(65_537).read_to_end(&mut bytes)?;
    if bytes.len() > 65_536 {
        return Err(invalid("publication control input exceeds 64 KiB"));
    }
    Ok(Some(bytes))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
}

fn checked_generation(
    directory: PathBuf,
    expected: Option<&Commit>,
) -> io::Result<ArtifactSnapshot> {
    if !fs::symlink_metadata(&directory)?.is_dir() {
        return Err(invalid(
            "artifact generation must be a directory, not a symlink",
        ));
    }
    let bytes = control_bytes(&directory.join(SEAL))?
        .ok_or_else(|| invalid("artifact generation has no completion manifest"))?;
    if expected.is_some_and(|commit| {
        commit.manifest_xxh3 != format!("{:016x}", xxhash_rust::xxh3::xxh3_64(&bytes))
    }) {
        return Err(invalid(
            "artifact generation manifest does not match the commit marker",
        ));
    }
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(invalid_json)?;
    if manifest.version != VERSION
        || !valid_id(&manifest.generation)
        || expected.is_some_and(|commit| commit.generation != manifest.generation)
    {
        return Err(invalid("unsupported or inconsistent artifact generation"));
    }
    // Check every member, not just the baseline trio. No new generation is accepted
    // when even a report or secondary-scanner member is missing or damaged.
    let paths = ArtifactPaths::in_directory(directory.clone());
    if manifest.artifacts.len() != paths.members().len() {
        return Err(invalid(
            "artifact generation has an incomplete member inventory",
        ));
    }
    for path in paths.members() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let member = manifest
            .artifacts
            .get(name)
            .ok_or_else(|| invalid(format!("artifact generation is missing {name}")))?;
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_file()
            || metadata.len() != member.bytes
            || format!("{:016x}", hash_file_xxh3(path)?.0) != member.xxh3
        {
            return Err(invalid(format!(
                "artifact generation member failed verification: {name}"
            )));
        }
    }
    let identity = super::read_json_artifact_if_exists::<super::ScanManifest>(
        &directory.join(super::SCAN_MANIFEST_FILE),
    )?
    .and_then(|manifest| manifest.snapshot_identity);
    Ok(ArtifactSnapshot {
        directory,
        generation: Some(manifest.generation),
        identity,
        _legacy_lease: None,
    })
}

fn invalid_json(error: serde_json::Error) -> io::Error {
    invalid(format!("invalid publication control JSON: {error}"))
}

fn committed(root: &Path) -> io::Result<Option<ArtifactSnapshot>> {
    let Some(bytes) = control_bytes(&root.join(POINTER))? else {
        return Ok(None);
    };
    let commit: Commit = serde_json::from_slice(&bytes).map_err(invalid_json)?;
    if commit.version != VERSION || !valid_id(&commit.generation) {
        return Err(invalid("invalid artifact generation commit marker"));
    }
    let generations = root.join(GENERATIONS);
    if !fs::symlink_metadata(&generations)?.is_dir() {
        return Err(invalid("artifact generation store must not be a symlink"));
    }
    checked_generation(generations.join(&commit.generation), Some(&commit)).map(Some)
}

impl ArtifactSnapshot {
    pub fn pin(root: &Path) -> io::Result<Option<Self>> {
        if let Some(snapshot) = committed(root)? {
            return Ok(Some(snapshot));
        }
        if control_bytes(&root.join(SEAL))?.is_some() {
            return checked_generation(root.to_path_buf(), None).map(Some);
        }
        let lease = match File::open(root.join(LOCK)) {
            Ok(file) => {
                fs4::FileExt::lock_shared(&file)?;
                Some(file)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        // A first publication may have finished while the shared lock waited.
        if let Some(snapshot) = committed(root)? {
            return Ok(Some(snapshot));
        }
        if root.join(GENERATIONS).try_exists()? {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "no complete artifact generation has been committed",
            ));
        }
        match fs::metadata(root) {
            Ok(metadata) if metadata.is_dir() => Ok(Some(Self {
                directory: root.to_path_buf(),
                generation: None,
                identity: None,
                _legacy_lease: lease,
            })),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
            Ok(_) => Err(invalid("artifact output root is not a directory")),
        }
    }
}

pub(super) struct Publication {
    pub directory: PathBuf,
    pub baseline_directory: PathBuf,
    root: PathBuf,
    id: String,
    committed: bool,
    _writer_lease: File,
}

impl Publication {
    pub fn begin(root: &Path) -> io::Result<Self> {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        fs::create_dir_all(root)?;
        if root.join(SEAL).try_exists()? {
            return Err(invalid(
                "cannot publish into an immutable artifact generation",
            ));
        }
        let lease = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join(LOCK))?;
        fs4::FileExt::lock(&lease)?;
        // Under the writer lock, legacy recovery can inspect its existing sealed
        // baseline trio even after an interrupted first publication.
        let baseline_directory = committed(root)?
            .map(|snapshot| snapshot.directory)
            .unwrap_or_else(|| root.to_path_buf());
        let store = root.join(GENERATIONS);
        fs::create_dir_all(&store)?;
        if !fs::symlink_metadata(&store)?.is_dir() {
            return Err(invalid("artifact generation store must not be a symlink"));
        }
        for _ in 0..128 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos();
            let id = format!(
                "{now:x}-{:x}-{:x}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            );
            let directory = store.join(&id);
            let builder = fs::DirBuilder::new();
            #[cfg(unix)]
            let builder = {
                use std::os::unix::fs::DirBuilderExt;
                let mut builder = builder;
                builder.mode(0o700);
                builder
            };
            match builder.create(&directory) {
                Ok(()) => {
                    return Ok(Self {
                        directory,
                        baseline_directory,
                        root: root.to_path_buf(),
                        id,
                        committed: false,
                        _writer_lease: lease,
                    })
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "artifact generation namespace exhausted",
        ))
    }

    pub fn publish(mut self, paths: &ArtifactPaths) -> io::Result<()> {
        let mut artifacts = BTreeMap::new();
        for path in paths.members() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| invalid("invalid artifact member name"))?;
            // Preserve pre-existing per-file privacy before making the new directory
            // traversable to the same readers as the output root.
            match fs::symlink_metadata(self.root.join(name)) {
                Ok(metadata) if metadata.is_file() => {
                    fs::set_permissions(path, metadata.permissions())?;
                    File::open(path)?.sync_all()?;
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
            artifacts.insert(
                name.to_owned(),
                Member {
                    bytes: fs::metadata(path)?.len(),
                    xxh3: format!("{:016x}", hash_file_xxh3(path)?.0),
                },
            );
        }
        let manifest = Manifest {
            version: VERSION,
            generation: self.id.clone(),
            artifacts,
        };
        let payload = serde_json::to_vec_pretty(&manifest).map_err(invalid_json)?;
        atomic::write(&self.directory.join(SEAL), |writer| {
            writer.write_all(&payload)
        })?;
        fs::set_permissions(&self.directory, fs::metadata(&self.root)?.permissions())?;
        sync_directory(&self.directory)?;
        sync_directory(&self.root.join(GENERATIONS))?;
        // Copies keep legacy paths writable without changing immutable generation
        // inodes. They are compatibility views, never the multi-file commit point.
        for path in paths.members() {
            let target = self.root.join(
                path.file_name()
                    .ok_or_else(|| invalid("invalid artifact member path"))?,
            );
            atomic::write(&target, |writer| {
                io::copy(&mut File::open(path)?, writer).map(|_| ())
            })?;
        }
        let commit = Commit {
            version: VERSION,
            generation: self.id.clone(),
            manifest_xxh3: format!("{:016x}", xxhash_rust::xxh3::xxh3_64(&payload)),
        };
        atomic::write(&self.root.join(POINTER), |writer| {
            serde_json::to_writer_pretty(writer, &commit).map_err(io::Error::other)
        })?;
        // Once the pointer rename succeeded, a sync failure must not remove the
        // generation a concurrent reader may already have pinned.
        self.committed = true;
        sync_directory(&self.root)
    }
}

fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

impl Drop for Publication {
    fn drop(&mut self) {
        if !self.committed {
            if let Err(error) = fs::remove_dir_all(&self.directory) {
                eprintln!("unpublished artifact generation cleanup failed: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::write_project_analysis_artifacts;
    use crate::ingestion::{pipeline::analyze_project, scan::ScanConfig};

    fn project() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("aigiscode-publication-{}-{id}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.rs"), "fn main() {}\n").unwrap();
        root
    }

    #[test]
    fn old_pins_survive_publication_and_compatibility_edits() {
        let root = project();
        let output = root.join(".aigiscode");
        let analysis = analyze_project(&root, &ScanConfig::default()).unwrap();
        let first = write_project_analysis_artifacts(&analysis, None).unwrap();
        let pinned = ArtifactSnapshot::pin(&output).unwrap().unwrap();
        let bytes = fs::read(&first.aigiscode_report).unwrap();
        fs::write(root.join("main.rs"), "fn main() { let _value = 1; }\n").unwrap();
        let next = analyze_project(&root, &ScanConfig::default()).unwrap();
        let second = write_project_analysis_artifacts(&next, None).unwrap();
        assert_ne!(first.output_dir, second.output_dir);
        assert_eq!(pinned.directory, first.output_dir);
        assert_eq!(fs::read(&first.aigiscode_report).unwrap(), bytes);
        assert_eq!(
            ArtifactSnapshot::pin(&output).unwrap().unwrap().directory,
            second.output_dir
        );
        fs::write(
            output.join("semantic-graph.json"),
            "damaged flat compatibility copy",
        )
        .unwrap();
        assert!(ArtifactSnapshot::pin(&output).is_ok());
        assert!(atomic::write(&second.semantic_graph, |writer| writer.write_all(b"{}")).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_compatibility_refresh_preserves_the_committed_family() {
        let root = project();
        let output = root.join(".aigiscode");
        let analysis = analyze_project(&root, &ScanConfig::default()).unwrap();
        let first = write_project_analysis_artifacts(&analysis, None).unwrap();
        let marker = fs::read(output.join(POINTER)).unwrap();
        fs::remove_file(output.join("review-surface.json")).unwrap();
        fs::create_dir(output.join("review-surface.json")).unwrap();
        assert!(write_project_analysis_artifacts(&analysis, None).is_err());
        assert_eq!(fs::read(output.join(POINTER)).unwrap(), marker);
        assert_eq!(
            ArtifactSnapshot::pin(&output).unwrap().unwrap().directory,
            first.output_dir
        );
        assert_eq!(fs::read_dir(output.join(GENERATIONS)).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_commits_and_corrupt_members_never_fall_back_to_flat_files() {
        let root = project();
        let output = root.join(".aigiscode");
        let analysis = analyze_project(&root, &ScanConfig::default()).unwrap();
        let paths = write_project_analysis_artifacts(&analysis, None).unwrap();
        let marker = fs::read(output.join(POINTER)).unwrap();
        fs::write(
            output.join(POINTER),
            br#"{"version":1,"generation":"../escape","manifest_xxh3":"0"}"#,
        )
        .unwrap();
        assert!(ArtifactSnapshot::pin(&output).is_err());
        fs::write(output.join(POINTER), marker).unwrap();
        fs::write(paths.review_surface, b"{}").unwrap();
        assert!(ArtifactSnapshot::pin(&output).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
