use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannedFile {
    pub relative_path: PathBuf,
    pub size_bytes: u64,
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

        files.push(ScannedFile {
            relative_path,
            size_bytes: metadata.len(),
        });
    }

    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    let summary = ScanSummary {
        scanned_files: files.len(),
        skipped_dirs: skipped_dirs.get(),
        skipped_hidden_dirs: skipped_hidden_dirs.get(),
    };

    Ok(ScanResult {
        root,
        files,
        summary,
    })
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
    use super::{scan_repository, ScanConfig};
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn scans_files_without_reading_contents() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src/nested")).unwrap();
        fs::write(fixture.join("src/main.py"), b"print('hello')").unwrap();
        fs::write(fixture.join("src/nested/lib.rs"), b"fn main() {}").unwrap();

        let result = scan_repository(&fixture, &ScanConfig::default()).unwrap();

        let paths: Vec<PathBuf> = result
            .files
            .into_iter()
            .map(|file| file.relative_path)
            .collect();
        assert_eq!(
            paths,
            vec![
                PathBuf::from("src/main.py"),
                PathBuf::from("src/nested/lib.rs")
            ]
        );
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
