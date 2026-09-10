//! Bounded supplemental evidence for backend orphan candidates, with explicit gaps.

use super::super::is_test_source_path;
use crate::ingestion::scan::{AnalysisBoundaryTruth, AnalysisScope};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

const MAX_SOURCE_BYTES: u64 = 1_048_576;
const MAX_GAP_PREVIEW: usize = 20;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BackendOrphanStatus {
    Complete,
    Incomplete,
    DeferredInputCoverage,
    DeferredBoundary,
    NotApplicable,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupplementalGapReason {
    MissingRoot,
    DirectoryRead,
    EntryRead,
    FileType,
    FileRead,
    Oversized,
    InvalidUtf8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct SupplementalGap {
    pub path: PathBuf,
    pub reason: SupplementalGapReason,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BackendOrphanCoverage {
    pub status: BackendOrphanStatus,
    pub scanned_files: usize,
    pub scanned_bytes: u64,
    pub gap_count: usize,
    pub gaps_preview: Vec<SupplementalGap>,
    pub input_fingerprint: String,
}

impl BackendOrphanCoverage {
    pub fn is_complete(&self) -> bool {
        matches!(self.status, BackendOrphanStatus::Complete | BackendOrphanStatus::NotApplicable)
    }

    pub fn summary(&self) -> String {
        match self.status {
            BackendOrphanStatus::Complete => format!(
                "Backend orphan supplemental coverage complete within the declared sweep: {} files, {} bytes. Orphan findings remain heuristic.",
                self.scanned_files, self.scanned_bytes,
            ),
            BackendOrphanStatus::NotApplicable => String::from("Backend orphan analysis is not applicable: no PHP inputs."),
            BackendOrphanStatus::DeferredInputCoverage => String::from("Backend orphan analysis is deferred because native input coverage is incomplete."),
            BackendOrphanStatus::DeferredBoundary => format!("Backend orphan analysis is deferred at the truncated analysis boundary; {} supplemental files were read within the selected root, but callers may exist outside it.", self.scanned_files),
            BackendOrphanStatus::Incomplete => format!("Backend orphan analysis is deferred: {} supplemental evidence gaps; inspect backend_orphan_coverage.gaps_preview.", self.gap_count),
            BackendOrphanStatus::Unknown => String::from("Backend orphan evidence coverage is unknown; zero findings are not a clean result."),
        }
    }

    fn gap(&mut self, path: &Path, reason: SupplementalGapReason, detail: impl ToString, hash: &mut DefaultHasher) {
        let gap = SupplementalGap { path: path.to_path_buf(), reason, detail: detail.to_string() };
        "gap".hash(hash);
        gap.hash(hash);
        self.gap_count += 1;
        if self.gaps_preview.len() < MAX_GAP_PREVIEW {
            self.gaps_preview.push(gap);
        }
    }
}

pub(super) fn collect(
    root: &Path,
    parsed_sources: &[(PathBuf, String)],
    scope: &AnalysisScope,
    mut visit: impl FnMut(String),
) -> BackendOrphanCoverage {
    const EXTENSIONS: &[&str] = &["php", "json", "yaml", "yml", "xml", "neon", "ini", "sh", "env", "ts", "js"];
    let parsed = parsed_sources.iter().map(|(path, _)| path.as_path()).collect::<HashSet<_>>();
    let mut coverage = BackendOrphanCoverage::default();
    let mut hash = DefaultHasher::new();
    let mut stack = Vec::new();
    if root.as_os_str().is_empty() {
        coverage.gap(root, SupplementalGapReason::MissingRoot, "no supplemental repository root was supplied", &mut hash);
    } else {
        stack.push(root.to_path_buf());
    }
    while let Some(directory) = stack.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                coverage.gap(&directory, SupplementalGapReason::DirectoryRead, error, &mut hash);
                continue;
            }
        };
        let mut ordered = Vec::new();
        for entry in entries {
            match entry {
                Ok(entry) => ordered.push(entry),
                Err(error) => coverage.gap(&directory, SupplementalGapReason::EntryRead, error, &mut hash),
            }
        }
        ordered.sort_by_key(|entry| entry.file_name());
        for entry in ordered {
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap_or(&path);
            if scope.is_generated_path(relative) {
                continue;
            }
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    coverage.gap(&path, SupplementalGapReason::FileType, error, &mut hash);
                    continue;
                }
            };
            if file_type.is_dir() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if !name.starts_with('.') && !matches!(name.as_ref(),
                    "vendor" | "node_modules" | "storage" | "dist" | "build" | "target"
                    | "tests" | "Tests" | "__tests__" | "test" | "Test")
                {
                    stack.push(path);
                }
                continue;
            }
            // Links and special files remain outside the declared sweep.
            if !file_type.is_file() || is_test_source_path(&path)
                || parsed.contains(relative) || parsed.contains(path.as_path())
                || !path.extension().and_then(|extension| extension.to_str()).is_some_and(|extension| EXTENSIONS.contains(&extension))
            {
                continue;
            }
            match read_source(&path) {
                Ok(source) => {
                    "file".hash(&mut hash);
                    relative.hash(&mut hash);
                    source.len().hash(&mut hash);
                    crate::ingestion::hash::hash_bytes_xxh3(source.as_bytes()).0.hash(&mut hash);
                    coverage.scanned_files += 1;
                    coverage.scanned_bytes += source.len() as u64;
                    visit(source);
                }
                Err((reason, detail)) => coverage.gap(&path, reason, detail, &mut hash),
            }
        }
    }
    coverage.input_fingerprint = format!("{:016x}", hash.finish());
    coverage.status = if coverage.gap_count > 0 {
        BackendOrphanStatus::Incomplete
    } else if scope.boundary_truth == AnalysisBoundaryTruth::TruncatedSlice {
        BackendOrphanStatus::DeferredBoundary
    } else {
        BackendOrphanStatus::Complete
    };
    coverage
}

fn read_source(path: &Path) -> Result<String, (SupplementalGapReason, String)> {
    let read = || -> io::Result<Vec<u8>> {
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options.open(path)?;
        if !file.metadata()?.is_file() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "supplemental input is no longer a regular file"));
        }
        let mut bytes = Vec::new();
        // Bound the bytes actually read, including growth after metadata lookup.
        file.take(MAX_SOURCE_BYTES + 1).read_to_end(&mut bytes)?;
        Ok(bytes)
    };
    let bytes = read().map_err(|error| (SupplementalGapReason::FileRead, error.to_string()))?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err((SupplementalGapReason::Oversized, format!("supplemental input exceeds the {MAX_SOURCE_BYTES}-byte limit")));
    }
    String::from_utf8(bytes).map_err(|error| (SupplementalGapReason::InvalidUtf8, error.utf8_error().to_string()))
}
