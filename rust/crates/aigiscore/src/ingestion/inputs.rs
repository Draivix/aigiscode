//! One captured value for each configuration input used during an analysis.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use super::pipeline::ProjectAnalysisError;
use super::scan::{scan_repository, ScanConfig, ScanResult};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InputCapture {
    pub files: InputFiles,
    pub base_scan_config: ScanConfig,
}

impl InputCapture {
    pub fn new(base_scan_config: &ScanConfig) -> Self {
        Self {
            files: InputFiles::default(),
            base_scan_config: base_scan_config.clone(),
        }
    }

    pub fn verify(&self, scan: &ScanResult) -> Result<(), ProjectAnalysisError> {
        // Configuration read after the inventory must describe the very bytes
        // inventoried, even if a file changed and changed back between readers.
        for file in &scan.files {
            let path = scan.root.join(&file.relative_path);
            if let Some(bytes) = self.files.files.get(&path) {
                if bytes.as_ref().is_none_or(|bytes| {
                    bytes.len() as u64 != file.size_bytes
                        || super::hash::hash_bytes_xxh3(bytes) != file.content_hash
                }) {
                    return Err(ProjectAnalysisError::InputChanged { path });
                }
            }
        }
        let current = scan_repository(&scan.root, &self.base_scan_config)?;
        if current.root != scan.root
            || current.scope_fingerprint != scan.scope_fingerprint
            || current.scope != scan.scope
            || current.semantic_env != scan.semantic_env
        {
            return Err(ProjectAnalysisError::InputChanged {
                path: scan.root.clone(),
            });
        }
        for (before, after) in scan.files.iter().zip(&current.files) {
            if before.relative_path != after.relative_path
                || before.size_bytes != after.size_bytes
                || before.content_hash != after.content_hash
            {
                return Err(ProjectAnalysisError::InputChanged {
                    path: scan.root.join(&before.relative_path),
                });
            }
        }
        if scan.files.len() != current.files.len() {
            let extra = scan.files.get(current.files.len())
                .or_else(|| current.files.get(scan.files.len()));
            return Err(ProjectAnalysisError::InputChanged {
                path: extra.map(|file| scan.root.join(&file.relative_path))
                    .unwrap_or_else(|| scan.root.clone()),
            });
        }
        // Check configuration after the full inventory walk, closest to the
        // publication boundary, including dependencies outside the scan scope.
        self.files.verify().map_err(|(path, source)| {
            if matches!(source.kind(), io::ErrorKind::Interrupted | io::ErrorKind::NotFound | io::ErrorKind::InvalidData) {
                ProjectAnalysisError::InputChanged { path }
            } else {
                ProjectAnalysisError::ReadFile { path, source }
            }
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InputFiles {
    files: BTreeMap<PathBuf, Option<Arc<[u8]>>>,
    directories: BTreeMap<PathBuf, bool>,
}

impl InputFiles {
    pub fn read(&mut self, path: &Path) -> io::Result<Option<Arc<[u8]>>> {
        let path = absolute_path(path)?;
        if let Some(bytes) = self.files.get(&path) {
            return Ok(bytes.clone());
        }
        let bytes = read_optional(&path)?;
        self.files.insert(path, bytes.clone());
        Ok(bytes)
    }

    pub fn is_dir(&mut self, path: &Path) -> io::Result<bool> {
        let path = absolute_path(path)?;
        if let Some(present) = self.directories.get(&path) {
            return Ok(*present);
        }
        let present = directory_exists(&path)?;
        self.directories.insert(path, present);
        Ok(present)
    }

    pub fn verify(&self) -> Result<(), (PathBuf, io::Error)> {
        for (path, bytes) in &self.files {
            let current = read_optional(path).map_err(|error| (path.clone(), error))?;
            if &current != bytes {
                return Err((path.clone(), changed()));
            }
        }
        for (path, present) in &self.directories {
            let current = directory_exists(path).map_err(|error| (path.clone(), error))?;
            if current != *present {
                return Err((path.clone(), changed()));
            }
        }
        Ok(())
    }
}

fn changed() -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, "analysis input changed during capture")
}

fn directory_exists(path: &Path) -> io::Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn read_optional(path: &Path) -> io::Result<Option<Arc<[u8]>>> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "configuration is not a regular file",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
    fs::read(path).map(|bytes| Some(Arc::from(bytes)))
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    Ok(normalize_relative_path(&if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    }))
}

pub(crate) fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(normalized.components().next_back(), Some(Component::Normal(_))) {
                    normalized.pop();
                } else if !normalized.has_root() {
                    normalized.push("..");
                }
            }
            Component::Normal(segment) => normalized.push(segment),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
