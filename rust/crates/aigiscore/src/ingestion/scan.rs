use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};
use xxhash_rust::xxh3::Xxh3;

use crate::ingestion::hash::hash_file_xxh3;
use crate::revision::{ContentHash, FileContentKey, FileMtime, SemanticEnvFingerprint};

const SCAN_FILE: &str = ".aigiscode/scan.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanConfig {
    pub include_path_prefixes: Vec<PathBuf>,
    pub ignored_dir_names: HashSet<String>,
    pub ignored_path_prefixes: Vec<PathBuf>,
    pub skip_hidden: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            include_path_prefixes: Vec::new(),
            ignored_dir_names: HashSet::from([
                String::from("vendor"),
                String::from("node_modules"),
                String::from("storage"),
                String::from("__pycache__"),
                String::from("tmp"),
                String::from("dist"),
                String::from("build"),
                String::from("target"),
                String::from("coverage"),
            ]),
            ignored_path_prefixes: vec![
                PathBuf::from("public/build"),
                PathBuf::from("public/vendor"),
            ],
            skip_hidden: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisBoundaryTruth {
    #[default]
    CompleteRepository,
    TruncatedSlice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisBoundaryReason {
    CroppedRoot,
    IncludePathPrefixes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AnalysisScope {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_root: Option<PathBuf>,
    #[serde(default)]
    pub boundary_truth: AnalysisBoundaryTruth,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<AnalysisBoundaryReason>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_path_prefixes: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScannedFile {
    pub relative_path: PathBuf,
    pub size_bytes: u64,
    /// Best-effort filesystem mtime — diagnostic / future stable-read heuristic, NOT
    /// identity. `None` when the platform/file could not report it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<FileMtime>,
    /// Non-cryptographic content identity (xxh3-64). `serde(default)` keeps older
    /// `scan.json` artifacts (which lacked this field) deserializable.
    #[serde(default)]
    pub content_hash: ContentHash,
}

impl ScannedFile {
    /// The `(path, content_hash)` identity every derived fact will be tagged with.
    #[must_use]
    pub fn file_key(&self) -> FileContentKey {
        FileContentKey::new(self.relative_path.clone(), self.content_hash)
    }
}

/// One config input to the semantic-environment fingerprint (a file whose change can
/// alter code meaning without altering any source text — `Cargo.toml`, lockfiles, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticEnvInput {
    pub relative_path: PathBuf,
    pub exists: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<ContentHash>,
}

/// Fingerprint of the build/workspace configuration for a scan. Advances the semantic-env
/// revision independently of source-content changes.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SemanticEnvSnapshot {
    pub fingerprint: SemanticEnvFingerprint,
    #[serde(default)]
    pub inputs: Vec<SemanticEnvInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScanSummary {
    pub scanned_files: usize,
    pub skipped_dirs: usize,
    pub skipped_hidden_dirs: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub root: PathBuf,
    pub files: Vec<ScannedFile>,
    pub summary: ScanSummary,
    #[serde(default)]
    pub scope: AnalysisScope,
    /// Fingerprint of build/workspace config files that can change code meaning without
    /// changing source text. `serde(default)` keeps older artifacts deserializable.
    #[serde(default)]
    pub semantic_env: SemanticEnvSnapshot,
}

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("repository root does not exist: {0}")]
    MissingRoot(PathBuf),
    #[error("repository root is not a directory: {0}")]
    RootIsNotDirectory(PathBuf),
    #[error("failed to read metadata for {path}: {source}")]
    Metadata {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("walkdir failed for {path}: {source}")]
    Walk {
        path: PathBuf,
        #[source]
        source: walkdir::Error,
    },
    #[error("failed to read scan config {path}: {source}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse scan config {path}: {source}")]
    ParseConfig {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub fn scan_repository(
    root: impl Into<PathBuf>,
    config: &ScanConfig,
) -> Result<ScanResult, ScanError> {
    let root = root.into();
    if !root.exists() {
        return Err(ScanError::MissingRoot(root));
    }
    if !root.is_dir() {
        return Err(ScanError::RootIsNotDirectory(root));
    }
    let effective_config = load_scan_config(&root, config)?;

    let mut files = Vec::new();
    let skipped_dirs = Cell::new(0usize);
    let skipped_hidden_dirs = Cell::new(0usize);

    let walker = WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            should_visit(
                entry,
                &root,
                &effective_config,
                &skipped_dirs,
                &skipped_hidden_dirs,
            )
        });

    for entry in walker {
        let entry = entry.map_err(|source| ScanError::Walk {
            path: root.clone(),
            source,
        })?;

        if !entry.file_type().is_file() {
            continue;
        }

        let relative_path = relative_path(&root, entry.path());
        if !matches_include_prefixes(&relative_path, &effective_config.include_path_prefixes) {
            continue;
        }
        let metadata = fs::metadata(entry.path()).map_err(|source| ScanError::Metadata {
            path: entry.path().to_path_buf(),
            source,
        })?;

        // Content identity for the change substrate. Reading every file is intentional
        // extra I/O in Phase 1 — correctness/contract wiring matters more than speed, and
        // the pipeline re-reads these files to parse them anyway.
        let content_hash = hash_file_xxh3(entry.path()).map_err(|source| ScanError::Metadata {
            path: entry.path().to_path_buf(),
            source,
        })?;
        let modified_at = metadata.modified().ok().map(FileMtime::from_system_time);

        files.push(ScannedFile {
            relative_path,
            size_bytes: metadata.len(),
            modified_at,
            content_hash,
        });
    }

    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    let summary = ScanSummary {
        scanned_files: files.len(),
        skipped_dirs: skipped_dirs.get(),
        skipped_hidden_dirs: skipped_hidden_dirs.get(),
    };
    let scope = build_analysis_scope(&root, &effective_config);
    let semantic_env = compute_semantic_env(&root);

    Ok(ScanResult {
        root,
        files,
        summary,
        scope,
        semantic_env,
    })
}

/// Config files whose change can alter code meaning without altering source text. Scanned
/// at the analysis root; conservative and language-agnostic for Phase 1.
const SEMANTIC_ENV_CANDIDATES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "rust-toolchain",
    ".cargo/config.toml",
    ".cargo/config",
    "tsconfig.json",
    "jsconfig.json",
    "package.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "composer.json",
    "composer.lock",
    "pyproject.toml",
    "poetry.lock",
    "requirements.txt",
    "Gemfile",
    "Gemfile.lock",
    "go.mod",
    "go.sum",
];

/// Fingerprint the semantic environment: for each candidate config file, fold its path,
/// existence, size, and content hash into one xxh3-128 digest. Deterministic and ordered;
/// mtimes are deliberately excluded (they are not identity).
pub fn compute_semantic_env(root: &Path) -> SemanticEnvSnapshot {
    let mut hasher = Xxh3::new();
    let mut inputs = Vec::with_capacity(SEMANTIC_ENV_CANDIDATES.len());
    for rel in SEMANTIC_ENV_CANDIDATES {
        let abs = root.join(rel);
        hasher.update(rel.as_bytes());
        match fs::metadata(&abs) {
            Ok(meta) if meta.is_file() => {
                let hash = hash_file_xxh3(&abs).unwrap_or(ContentHash(0));
                hasher.update(&[1]);
                hasher.update(&meta.len().to_le_bytes());
                hasher.update(&hash.0.to_le_bytes());
                inputs.push(SemanticEnvInput {
                    relative_path: PathBuf::from(rel),
                    exists: true,
                    size_bytes: Some(meta.len()),
                    content_hash: Some(hash),
                });
            }
            _ => {
                hasher.update(&[0]);
                inputs.push(SemanticEnvInput {
                    relative_path: PathBuf::from(rel),
                    exists: false,
                    size_bytes: None,
                    content_hash: None,
                });
            }
        }
    }
    SemanticEnvSnapshot {
        fingerprint: SemanticEnvFingerprint(hasher.digest128()),
        inputs,
    }
}

fn build_analysis_scope(root: &Path, config: &ScanConfig) -> AnalysisScope {
    let repository_root = detect_repository_root(root);
    let mut reasons = Vec::new();
    if repository_root
        .as_ref()
        .is_some_and(|repo_root| repo_root != root)
    {
        reasons.push(AnalysisBoundaryReason::CroppedRoot);
    }
    if !config.include_path_prefixes.is_empty() {
        reasons.push(AnalysisBoundaryReason::IncludePathPrefixes);
    }
    let boundary_truth = if reasons.is_empty() {
        AnalysisBoundaryTruth::CompleteRepository
    } else {
        AnalysisBoundaryTruth::TruncatedSlice
    };
    AnalysisScope {
        repository_root,
        boundary_truth,
        reasons,
        include_path_prefixes: config.include_path_prefixes.clone(),
    }
}

fn detect_repository_root(root: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn should_visit(
    entry: &DirEntry,
    root: &Path,
    config: &ScanConfig,
    skipped_dirs: &Cell<usize>,
    skipped_hidden_dirs: &Cell<usize>,
) -> bool {
    if entry.depth() == 0 {
        return true;
    }

    if !entry.file_type().is_dir() {
        return true;
    }

    let name = entry.file_name().to_string_lossy();
    if config.skip_hidden && name.starts_with('.') {
        skipped_hidden_dirs.set(skipped_hidden_dirs.get() + 1);
        return false;
    }

    if config.ignored_dir_names.contains(name.as_ref()) {
        skipped_dirs.set(skipped_dirs.get() + 1);
        return false;
    }

    let relative = relative_path(root, entry.path());
    if !overlaps_include_prefixes(&relative, &config.include_path_prefixes) {
        skipped_dirs.set(skipped_dirs.get() + 1);
        return false;
    }
    if config
        .ignored_path_prefixes
        .iter()
        .any(|prefix| relative.starts_with(prefix))
    {
        skipped_dirs.set(skipped_dirs.get() + 1);
        return false;
    }

    true
}

fn load_scan_config(root: &Path, base: &ScanConfig) -> Result<ScanConfig, ScanError> {
    let path = root.join(SCAN_FILE);
    let mut config = base.clone();
    let file = match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str::<ScanConfigFile>(&content).map_err(|source| {
            ScanError::ParseConfig {
                path: path.clone(),
                source,
            }
        })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(config),
        Err(source) => {
            return Err(ScanError::ReadConfig {
                path: path.clone(),
                source,
            })
        }
    };

    if !file.include_path_prefixes.is_empty() {
        config.include_path_prefixes = file.include_path_prefixes;
    }
    config.ignored_dir_names.extend(file.ignored_dir_names);
    config
        .ignored_path_prefixes
        .extend(file.ignored_path_prefixes);
    if let Some(skip_hidden) = file.skip_hidden {
        config.skip_hidden = skip_hidden;
    }

    Ok(config)
}

#[derive(Debug, Default, Deserialize)]
struct ScanConfigFile {
    #[serde(default)]
    include_path_prefixes: Vec<PathBuf>,
    #[serde(default)]
    ignored_dir_names: Vec<String>,
    #[serde(default)]
    ignored_path_prefixes: Vec<PathBuf>,
    #[serde(default)]
    skip_hidden: Option<bool>,
}

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(normalize_component)
        .collect()
}

fn normalize_component(component: Component<'_>) -> Option<PathBuf> {
    match component {
        Component::Normal(segment) => Some(PathBuf::from(segment)),
        _ => None,
    }
}

fn matches_include_prefixes(path: &Path, include_path_prefixes: &[PathBuf]) -> bool {
    include_path_prefixes.is_empty()
        || include_path_prefixes
            .iter()
            .any(|prefix| path.starts_with(prefix))
}

fn overlaps_include_prefixes(path: &Path, include_path_prefixes: &[PathBuf]) -> bool {
    include_path_prefixes.is_empty()
        || include_path_prefixes
            .iter()
            .any(|prefix| path.starts_with(prefix) || prefix.starts_with(path))
}

#[cfg(test)]
mod tests {
    use super::{compute_semantic_env, scan_repository, AnalysisBoundaryTruth, ScanConfig};
    use crate::revision::ContentHash;
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn scans_files_and_captures_content_identity() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src/nested")).unwrap();
        fs::write(fixture.join("src/main.py"), b"print('hello')").unwrap();
        fs::write(fixture.join("src/nested/lib.rs"), b"fn main() {}").unwrap();

        let result = scan_repository(&fixture, &ScanConfig::default()).unwrap();

        let paths: Vec<PathBuf> = result
            .files
            .iter()
            .map(|file| file.relative_path.clone())
            .collect();
        assert_eq!(
            paths,
            vec![
                PathBuf::from("src/main.py"),
                PathBuf::from("src/nested/lib.rs")
            ]
        );
        // Every scanned file gets a non-zero content identity and a file_key mirroring it.
        for file in &result.files {
            assert_ne!(file.content_hash, ContentHash(0));
            assert_eq!(file.file_key().content_hash, file.content_hash);
            assert_eq!(file.file_key().relative_path, file.relative_path);
        }
    }

    #[test]
    fn content_hash_is_stable_and_content_sensitive() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/main.py"), b"print('hello')").unwrap();

        let first = scan_repository(&fixture, &ScanConfig::default()).unwrap();
        let hash_first = first.files[0].content_hash;

        // Re-scan without changes: same hash.
        let again = scan_repository(&fixture, &ScanConfig::default()).unwrap();
        assert_eq!(again.files[0].content_hash, hash_first);

        // Change the content: hash changes, path stays.
        fs::write(fixture.join("src/main.py"), b"print('world')").unwrap();
        let changed = scan_repository(&fixture, &ScanConfig::default()).unwrap();
        assert_eq!(changed.files[0].relative_path, PathBuf::from("src/main.py"));
        assert_ne!(changed.files[0].content_hash, hash_first);
    }

    #[test]
    fn semantic_env_fingerprint_tracks_config_not_source() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("Cargo.toml"), b"[package]\nname = \"x\"\n").unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}").unwrap();

        let base = compute_semantic_env(&fixture);
        assert_ne!(base.fingerprint, Default::default());

        // Changing a source file must NOT move the semantic-env fingerprint.
        fs::write(fixture.join("src/main.rs"), b"fn main() { let _ = 1; }").unwrap();
        assert_eq!(compute_semantic_env(&fixture).fingerprint, base.fingerprint);

        // Changing Cargo.toml MUST move it.
        fs::write(fixture.join("Cargo.toml"), b"[package]\nname = \"y\"\n").unwrap();
        assert_ne!(compute_semantic_env(&fixture).fingerprint, base.fingerprint);
    }

    #[test]
    fn respects_ignored_directory_names_and_prefixes() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("vendor/pkg")).unwrap();
        fs::create_dir_all(fixture.join("public/build")).unwrap();
        fs::create_dir_all(fixture.join("app")).unwrap();
        fs::write(fixture.join("vendor/pkg/a.php"), b"<?php").unwrap();
        fs::write(fixture.join("public/build/app.js"), b"console.log(1);").unwrap();
        fs::write(fixture.join("app/main.py"), b"print('ok')").unwrap();

        let config = ScanConfig {
            include_path_prefixes: Vec::new(),
            ignored_dir_names: HashSet::from([String::from("vendor")]),
            ignored_path_prefixes: vec![PathBuf::from("public/build")],
            skip_hidden: true,
        };

        let result = scan_repository(&fixture, &config).unwrap();
        let paths: Vec<PathBuf> = result
            .files
            .into_iter()
            .map(|file| file.relative_path)
            .collect();

        assert_eq!(paths, vec![PathBuf::from("app/main.py")]);
        assert_eq!(result.summary.skipped_dirs, 2);
    }

    #[test]
    fn default_config_skips_dependency_and_build_artifacts() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::create_dir_all(fixture.join("node_modules/react")).unwrap();
        fs::create_dir_all(fixture.join("target/debug")).unwrap();
        fs::create_dir_all(fixture.join("public/build")).unwrap();
        fs::create_dir_all(fixture.join("dist/assets")).unwrap();
        fs::write(fixture.join("src/main.ts"), b"export const ok = true;").unwrap();
        fs::write(
            fixture.join("node_modules/react/index.js"),
            b"module.exports = {};",
        )
        .unwrap();
        fs::write(fixture.join("target/debug/app"), b"binary").unwrap();
        fs::write(fixture.join("public/build/app.js"), b"console.log(1);").unwrap();
        fs::write(fixture.join("dist/assets/app.js"), b"console.log(2);").unwrap();

        let result = scan_repository(&fixture, &ScanConfig::default()).unwrap();
        let paths: Vec<PathBuf> = result
            .files
            .into_iter()
            .map(|file| file.relative_path)
            .collect();

        assert_eq!(paths, vec![PathBuf::from("src/main.ts")]);
        assert_eq!(result.summary.skipped_dirs, 4);
    }

    #[test]
    fn respects_repo_local_include_path_prefixes_from_aigiscode_scan_config() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::create_dir_all(fixture.join("app")).unwrap();
        fs::create_dir_all(fixture.join("resources/js")).unwrap();
        fs::create_dir_all(fixture.join("tests")).unwrap();
        fs::write(
            fixture.join(".aigiscode/scan.json"),
            br#"{
  "include_path_prefixes": ["app", "resources"]
}"#,
        )
        .unwrap();
        fs::write(fixture.join("app/main.php"), b"<?php echo 'ok';").unwrap();
        fs::write(
            fixture.join("resources/js/app.ts"),
            b"export const app = true;",
        )
        .unwrap();
        fs::write(fixture.join("tests/app_test.php"), b"<?php").unwrap();

        let result = scan_repository(&fixture, &ScanConfig::default()).unwrap();
        let paths: Vec<PathBuf> = result
            .files
            .into_iter()
            .map(|file| file.relative_path)
            .collect();

        assert_eq!(
            paths,
            vec![
                PathBuf::from("app/main.php"),
                PathBuf::from("resources/js/app.ts")
            ]
        );
        assert_eq!(
            result.scope.boundary_truth,
            AnalysisBoundaryTruth::TruncatedSlice
        );
    }

    #[test]
    fn marks_nested_git_subtree_scans_as_truncated() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src/module")).unwrap();
        fs::write(fixture.join("src/module/main.rs"), b"fn main() {}").unwrap();
        let init = Command::new("git")
            .arg("-C")
            .arg(&fixture)
            .args(["init", "-q"])
            .status()
            .expect("git init should run in test fixture");
        assert!(init.success(), "git init should succeed");

        let result = scan_repository(fixture.join("src/module"), &ScanConfig::default()).unwrap();

        assert_eq!(
            result.scope.boundary_truth,
            AnalysisBoundaryTruth::TruncatedSlice
        );
        assert!(result.scope.repository_root.is_some());
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aigiscore-scan-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
