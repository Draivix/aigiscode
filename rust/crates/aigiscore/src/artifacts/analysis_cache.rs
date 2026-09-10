//! Read the existing native findings contract through its manifest hash.

use crate::assessment::ArchitecturalAssessment;
use crate::contracts::ContractInventory;
use crate::coverage::InputCoverage;
use crate::detectors::{dead_code::DeadCodeResult, hardwiring::HardwiringResult};
use crate::graph::analysis::GraphAnalysis;
use crate::ingestion::pipeline::SemanticGraphProject;
use crate::scanners::ast_grep::AstGrepScanResult;
use crate::security::SecurityAnalysisResult;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Deserialize)]
pub(crate) struct CachedDeterministicFindings {
    input_coverage: InputCoverage,
    scanned_files: usize,
    analyzed_files: usize,
    symbols: usize,
    references: usize,
    resolved_edges: usize,
    pub graph_analysis: GraphAnalysis,
    pub architectural_assessment: ArchitecturalAssessment,
    pub dead_code: DeadCodeResult,
    pub hardwiring: HardwiringResult,
    pub ast_grep_scan: AstGrepScanResult,
    pub security_analysis: SecurityAnalysisResult,
    pub contract_inventory: ContractInventory,
}

impl CachedDeterministicFindings {
    pub fn load(directory: &Path, expected_hash: &str, project: &SemanticGraphProject) -> Option<Self> {
        if expected_hash.len() != 16 {
            return None;
        }
        let file = File::open(directory.join(super::DETERMINISTIC_FINDINGS_FILE)).ok()?;
        // Parse and fingerprint one stream, including trailing bytes. The caller
        // has already verified the source/configuration and semantic graph.
        let mut reader = BufReader::new(crate::ingestion::hash::HashingIo::new(file));
        let cached: Self = serde_json::from_reader(&mut reader).ok()?;
        if format!("{:016x}", reader.get_ref().content_hash().0) != expected_hash
            || cached.scanned_files != project.scan.files.len()
            || cached.analyzed_files != project.semantic_graph.files.len()
            || cached.symbols != project.semantic_graph.symbols.len()
            || cached.references != project.semantic_graph.references.len()
            || cached.resolved_edges != project.semantic_graph.resolved_edges.len()
            || cached.input_coverage != project.semantic_graph.input_coverage()
        {
            return None;
        }
        Some(cached)
    }
}
