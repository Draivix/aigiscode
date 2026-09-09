//! Input coverage is evidence about what was parsed, not a claim of semantic soundness.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParseDiagnosticKind {
    ErrorNode,
    MissingNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParseDiagnostic {
    pub kind: ParseDiagnosticKind,
    pub node_kind: String,
    pub start_line: usize,
    /// One-based UTF-8 byte column. End coordinates are exclusive.
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParseScope {
    #[default]
    Source,
    VueScriptOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParseOutcome {
    pub file_path: PathBuf,
    pub parser: String,
    pub scope: ParseScope,
    /// Recovery can mean invalid source or a grammar limitation; it proves neither by itself.
    pub required_recovery: bool,
    pub diagnostic_count: usize,
    pub diagnostics: Vec<ParseDiagnostic>,
    pub diagnostics_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UnsupportedSource {
    pub file_path: PathBuf,
    pub extension: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InputCoverageStatus {
    Complete,
    Incomplete,
    NoSupportedSources,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InputCoverage {
    pub status: InputCoverageStatus,
    pub supported_source_files: usize,
    pub parsed_source_files: usize,
    pub recovered_source_files: usize,
    pub scope_limited_files: usize,
    pub files_without_parse_evidence: usize,
    pub unsupported_source_files: usize,
    /// Scanned inputs outside the supported or recognized unsupported source inventory.
    pub other_input_files: usize,
    pub diagnostic_count: usize,
    pub parse_issues_preview: Vec<ParseOutcome>,
    pub unsupported_sources_preview: Vec<UnsupportedSource>,
    pub deferred_checks: Vec<String>,
}

impl InputCoverage {
    pub fn is_complete(&self) -> bool {
        self.status == InputCoverageStatus::Complete
    }

    pub fn summarize<'a>(
        files: impl Iterator<Item = &'a PathBuf>,
        outcomes: &[ParseOutcome],
        unsupported: &[UnsupportedSource],
        other_input_files: usize,
    ) -> Self {
        let by_path = outcomes
            .iter()
            .map(|outcome| (&outcome.file_path, outcome))
            .collect::<HashMap<_, _>>();
        let mut coverage = Self {
            unsupported_source_files: unsupported.len(),
            unsupported_sources_preview: unsupported.iter().take(50).cloned().collect(),
            other_input_files,
            ..Self::default()
        };
        for path in files {
            coverage.supported_source_files += 1;
            let Some(outcome) = by_path.get(path) else {
                coverage.files_without_parse_evidence += 1;
                continue;
            };
            coverage.parsed_source_files += 1;
            coverage.recovered_source_files += usize::from(outcome.required_recovery);
            coverage.scope_limited_files += usize::from(outcome.scope != ParseScope::Source);
            coverage.diagnostic_count += outcome.diagnostic_count;
            if (outcome.required_recovery || outcome.scope != ParseScope::Source)
                && coverage.parse_issues_preview.len() < 5
            {
                let mut preview = (*outcome).clone();
                preview.diagnostics.truncate(3);
                preview.diagnostics_truncated =
                    preview.diagnostic_count > preview.diagnostics.len();
                coverage.parse_issues_preview.push(preview);
            }
        }
        coverage.status = if coverage.unsupported_source_files > 0
            || coverage.recovered_source_files > 0
            || coverage.scope_limited_files > 0
            || coverage.files_without_parse_evidence > 0
        {
            InputCoverageStatus::Incomplete
        } else if coverage.supported_source_files == 0 {
            InputCoverageStatus::NoSupportedSources
        } else {
            InputCoverageStatus::Complete
        };
        if !coverage.is_complete() {
            coverage.deferred_checks = [
                "unused_imports",
                "unused_private_functions",
                "orphan_modules",
                "confirmed_orphan_files",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect();
        }
        coverage
    }
}

/// Recognized source languages without a native parser. Assets and configuration
/// are counted separately; this list does not assert that every unknown extension is data.
pub fn unsupported_source(path: &Path) -> Option<UnsupportedSource> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    matches!(
        extension.as_str(),
        "c" | "h"
            | "cc"
            | "cpp"
            | "cxx"
            | "hpp"
            | "cs"
            | "go"
            | "java"
            | "kt"
            | "kts"
            | "scala"
            | "swift"
            | "dart"
            | "ex"
            | "exs"
            | "erl"
            | "hrl"
            | "hs"
            | "lhs"
            | "lua"
            | "pl"
            | "pm"
            | "r"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "ps1"
            | "sql"
            | "svelte"
            | "astro"
            | "clj"
            | "cljs"
            | "cljc"
            | "groovy"
            | "vbs"
            | "vb"
            | "fs"
            | "fsx"
            | "jl"
            | "m"
            | "mm"
            | "coffee"
            | "elm"
            | "ml"
            | "mli"
            | "zig"
            | "sol"
    )
    .then(|| UnsupportedSource {
        file_path: path.to_path_buf(),
        extension,
    })
}
