//! Completion of the configured secondary rules, independent of native parse coverage.

use super::ast_grep::AstGrepSkippedFile;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SecondaryCoverageStatus {
    Complete,
    Incomplete,
    NoInputs,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SecondaryCoverage {
    pub status: SecondaryCoverageStatus,
    pub input_files: usize,
    pub scanned_files: usize,
    pub prefiltered_files: usize,
    pub oversized_files: usize,
    pub unsupported_files: usize,
    pub other_gap_files: usize,
    pub gap_bytes: usize,
    pub gap_files_preview: Vec<AstGrepSkippedFile>,
}

impl SecondaryCoverage {
    /// Complete means no recorded execution gaps in the configured rule scope;
    /// neither prefilter soundness nor absence of defects is established by it.
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            SecondaryCoverageStatus::Complete | SecondaryCoverageStatus::NoInputs
        )
    }

    pub fn from_scan(scanned_files: usize, skipped: &[AstGrepSkippedFile]) -> Self {
        let mut coverage = Self {
            input_files: scanned_files + skipped.len(),
            scanned_files,
            ..Self::default()
        };
        let mut gaps = Vec::new();
        for file in skipped {
            match file.reason.as_str() {
                "no_family_prefilter_hit" => {
                    coverage.prefiltered_files += 1;
                    continue;
                }
                "file_too_large_for_secondary_scan" => coverage.oversized_files += 1,
                "no_rules_for_language" => coverage.unsupported_files += 1,
                _ => coverage.other_gap_files += 1,
            }
            coverage.gap_bytes += file.bytes;
            gaps.push(file);
        }
        coverage.status = if !gaps.is_empty() {
            SecondaryCoverageStatus::Incomplete
        } else if coverage.input_files == 0 {
            SecondaryCoverageStatus::NoInputs
        } else {
            SecondaryCoverageStatus::Complete
        };
        gaps.sort_by(|left, right| {
            right
                .bytes
                .cmp(&left.bytes)
                .then(left.file_path.cmp(&right.file_path))
        });
        coverage.gap_files_preview = gaps.into_iter().take(5).cloned().collect();
        coverage
    }

    pub fn summary(&self) -> String {
        format!(
            "Secondary rule coverage {:?}: {} scanned, {} prefiltered, {} oversized, {} without language rules, {} other gaps. See ast-grep-scan.json; this does not establish absence of defects.",
            self.status, self.scanned_files, self.prefiltered_files, self.oversized_files,
            self.unsupported_files, self.other_gap_files,
        )
    }
}
