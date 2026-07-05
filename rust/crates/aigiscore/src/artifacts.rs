use crate::agentic::{
    build_agentic_review_artifact, build_graph_packet_artifact, build_graph_query_context,
    graph_trace_between_files_with_context, AgenticGraphTrace, AgenticReviewArtifact,
    AgenticSemanticStateFlowKind, GraphPacketArtifact, GraphQueryContext, SemanticStateProofKind,
};
use crate::assessment::ArchitecturalAssessment;
use crate::contracts::ContractInventory;
use crate::detectors::dead_code::DeadCodeResult;
use crate::detectors::hardwiring::HardwiringResult;
use crate::doctrine::{
    load_doctrine_registry, DoctrineDisposition, DoctrineLoadError, DoctrineRegistry,
};
use crate::evidence::EvidenceAnchor;
use crate::external::ExternalAnalysisResult;
use crate::graph::analysis::GraphAnalysis;
use crate::ingestion::pipeline::{PhaseTiming, ProjectAnalysis, SemanticGraphProject};
use crate::kuzu_index::{
    build_dependency_graph_artifact, build_evidence_graph_artifact, DependencyGraphArtifact,
    EvidenceGraphArtifact,
};
use crate::policy::{PolicyBundle, PolicyLoadError};
use crate::review::{
    build_review_surface, ReviewFindingFamily, ReviewFindingSeverity, ReviewSurface,
};
use crate::scanners::ast_grep::{AstGrepFamilyCounts, AstGrepScanResult};
use crate::security::{SecurityAnalysisResult, SecurityContext};
use crate::surface::ArchitectureSurface;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub const DEFAULT_OUTPUT_DIR_NAME: &str = ".aigiscode";
pub const DETERMINISTIC_ANALYSIS_FILE: &str = "deterministic-analysis.json";
pub const SEMANTIC_GRAPH_FILE: &str = "semantic-graph.json";
pub const DEPENDENCY_GRAPH_FILE: &str = "dependency-graph.json";
pub const EVIDENCE_GRAPH_FILE: &str = "evidence-graph.json";
pub const CONTRACT_INVENTORY_FILE: &str = "contract-inventory.json";
pub const DOCTRINE_REGISTRY_FILE: &str = "doctrine-registry.json";
pub const DETERMINISTIC_FINDINGS_FILE: &str = "deterministic-findings.json";
pub const AST_GREP_SCAN_FILE: &str = "ast-grep-scan.json";
pub const EXTERNAL_ANALYSIS_FILE: &str = "external-analysis.json";
pub const ARCHITECTURE_SURFACE_FILE: &str = "architecture-surface.json";
pub const REVIEW_SURFACE_FILE: &str = "review-surface.json";
pub const CONVERGENCE_HISTORY_FILE: &str = "convergence-history.json";
pub const GUARD_DECISION_FILE: &str = "guard-decision.json";
pub const AGENT_HANDOFF_FILE: &str = "aigiscode-handoff.json";
pub const AGENTIC_REVIEW_FILE: &str = "agentic-review.json";
pub const GRAPH_PACKETS_FILE: &str = "graph-packets.json";
pub const REPOSITORY_TOPOLOGY_FILE: &str = "repository-topology.json";
pub const AGENT_REVIEW_JSON_FILE: &str = "agent-review.json";
pub const AGENT_REVIEW_MARKDOWN_FILE: &str = "agent-review.md";
pub const AGENT_OUTPUT_SCHEMA_FILE: &str = "agent-output-schema.json";
pub const AGENT_EXECUTION_EVENTS_FILE: &str = "agent-execution.jsonl";
pub const AGENT_SPIDER_REPORT_FILE: &str = "agent-spider-report.json";
pub const AIGISCODE_REPORT_FILE: &str = "aigiscode-report.json";
pub const AIGISCODE_REPORT_MARKDOWN_FILE: &str = "aigiscode-report.md";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonArtifactStyle {
    Pretty,
    Compact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPaths {
    pub output_dir: PathBuf,
    pub deterministic_analysis: PathBuf,
    pub semantic_graph: PathBuf,
    pub dependency_graph: PathBuf,
    pub evidence_graph: PathBuf,
    pub contract_inventory: PathBuf,
    pub doctrine_registry: PathBuf,
    pub deterministic_findings: PathBuf,
    pub ast_grep_scan: PathBuf,
    pub external_analysis: PathBuf,
    pub architecture_surface: PathBuf,
    pub review_surface: PathBuf,
    pub convergence_history: PathBuf,
    pub guard_decision: PathBuf,
    pub agent_handoff: PathBuf,
    pub agentic_review: PathBuf,
    pub graph_packets: PathBuf,
    pub repository_topology: PathBuf,
    pub aigiscode_report: PathBuf,
    pub aigiscode_report_markdown: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunPaths {
    pub output_dir: PathBuf,
    pub review_json: PathBuf,
    pub review_markdown: PathBuf,
    pub output_schema: PathBuf,
    pub execution_events: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct DeterministicFindingsArtifact<'a> {
    pub root: &'a Path,
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub symbols: usize,
    pub references: usize,
    pub resolved_edges: usize,
    pub graph_analysis: &'a GraphAnalysis,
    pub architectural_assessment: &'a ArchitecturalAssessment,
    pub dead_code: &'a DeadCodeResult,
    pub hardwiring: &'a HardwiringResult,
    pub ast_grep_scan: &'a AstGrepScanResult,
    pub security_analysis: &'a SecurityAnalysisResult,
    pub contract_inventory: &'a ContractInventory,
    pub timings: &'a [PhaseTiming],
}

#[derive(Debug, Serialize)]
pub struct DependencyGraphJsonArtifact<'a> {
    pub root: &'a Path,
    pub dependency_graph: DependencyGraphArtifact,
}

#[derive(Debug, Serialize)]
pub struct EvidenceGraphJsonArtifact<'a> {
    pub root: &'a Path,
    pub evidence_graph: EvidenceGraphArtifact,
}

#[derive(Debug, Serialize)]
pub struct AigiscodeReportArtifact<'a> {
    pub root: &'a Path,
    pub summary: ReportSummary,
    pub feedback_loop: FeedbackLoopSummary,
    pub graph_analysis: &'a GraphAnalysis,
    pub dead_code: &'a DeadCodeResult,
    pub hardwiring: &'a HardwiringResult,
    pub security_analysis: &'a SecurityAnalysisResult,
    pub contract_inventory: &'a ContractInventory,
    pub doctrine_registry: DoctrineRegistry,
    pub external_analysis: &'a ExternalAnalysisResult,
    pub architecture_surface: &'a ArchitectureSurface,
    pub review_surface: &'a ReviewSurface,
    pub convergence_history: &'a ConvergenceHistoryArtifact,
    pub guard_decision: &'a GuardDecisionArtifact,
    pub agent_handoff: &'a AgentHandoffArtifact,
    pub agentic_review: &'a AgenticReviewArtifact,
    pub graph_packets: &'a GraphPacketArtifact,
    pub repository_topology: &'a RepositoryTopologyArtifact,
    pub timings: &'a [PhaseTiming],
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub symbols: usize,
    pub references: usize,
    pub resolved_edges: usize,
    pub strong_cycle_count: usize,
    pub total_cycle_count: usize,
    pub architectural_smell_count: usize,
    pub hub_like_dependency_count: usize,
    pub unstable_dependency_count: usize,
    pub warning_heavy_hotspot_count: usize,
    pub split_identity_model_count: usize,
    pub compatibility_scar_count: usize,
    pub duplicate_mechanism_count: usize,
    pub sanctioned_path_bypass_count: usize,
    pub hand_rolled_parsing_count: usize,
    pub abstraction_sprawl_count: usize,
    pub algorithmic_complexity_hotspot_count: usize,
    pub ast_grep_finding_count: usize,
    pub ast_grep_algorithmic_complexity_count: usize,
    pub ast_grep_security_dangerous_api_count: usize,
    pub ast_grep_framework_misuse_count: usize,
    pub ast_grep_skipped_file_count: usize,
    pub ast_grep_skipped_bytes: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ast_grep_skipped_files_preview: Vec<crate::scanners::ast_grep::AstGrepSkippedFile>,
    pub dead_code_count: usize,
    pub hardwiring_count: usize,
    pub security_finding_count: usize,
    pub declared_route_count: usize,
    pub declared_hook_count: usize,
    pub declared_registered_key_count: usize,
    pub declared_symbolic_literal_count: usize,
    pub declared_env_key_count: usize,
    pub declared_config_key_count: usize,
    pub external_tool_count: usize,
    pub external_finding_count: usize,
    pub visible_findings: usize,
    pub accepted_by_policy: usize,
    pub suppressed_by_rule: usize,
    pub new_findings: usize,
    pub worsened_findings: usize,
    pub improved_findings: usize,
    pub resolved_findings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyArtifact {
    pub root: String,
    pub generated_at_unix_ms: u128,
    pub summary: RepositoryTopologySummary,
    pub manifests: Vec<RepositoryManifest>,
    pub zones: Vec<RepositoryTopologyZone>,
    pub links: Vec<RepositoryTopologyLink>,
    pub runtime_entry_files: Vec<RepositoryTopologyFileRef>,
    pub contract_zones: Vec<RepositoryTopologyContractZone>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologySummary {
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub zone_count: usize,
    pub manifest_count: usize,
    pub link_count: usize,
    pub runtime_entry_count: usize,
    #[serde(default)]
    pub boundary_truncated_count: usize,
    #[serde(default)]
    pub boundary_truth: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_reasons: Vec<String>,
    pub contract_zone_count: usize,
    pub linked_visible_finding_count: usize,
    pub linked_guardian_packet_count: usize,
    pub linked_semantic_state_flow_count: usize,
    pub linked_high_severity_visible_finding_count: usize,
    pub linked_high_priority_guardian_packet_count: usize,
    pub zones_with_cross_zone_pressure: usize,
    pub cross_zone_pressure_link_count: usize,
    pub strongest_cross_zone_relation_count: usize,
    pub baseline_observation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_start: Option<RepositoryTopologyRecommendedSlice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryManifest {
    pub path: String,
    pub zone_path: String,
    pub ecosystem: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyZone {
    pub path: String,
    pub labels: Vec<String>,
    pub manifest_observation: String,
    pub runtime_entry_observation: String,
    #[serde(default)]
    pub boundary_observation: String,
    pub spillover_observation: String,
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub total_size_bytes: u64,
    pub languages: Vec<String>,
    pub runtime_entry_count: usize,
    #[serde(default)]
    pub boundary_truncated_count: usize,
    pub contract_count: usize,
    pub visible_finding_count: usize,
    pub guardian_packet_count: usize,
    pub semantic_state_flow_count: usize,
    pub semantic_state_flow_proof_summary: RepositoryTopologySemanticStateProofSummary,
    pub inbound_cross_zone_link_count: usize,
    pub outbound_cross_zone_link_count: usize,
    pub inbound_cross_zone_relation_count: usize,
    pub outbound_cross_zone_relation_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_visible_finding_severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_guardian_packet_priority: Option<String>,
    pub visible_finding_ids: Vec<String>,
    pub guardian_packet_ids: Vec<String>,
    pub triage_summary: String,
    pub cross_zone_pressure_summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causal_bridges: Vec<RepositoryTopologySupportBridge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triage_next_steps: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triage_steps: Vec<RepositoryTopologyTriageStep>,
    pub finding_status_summary: RepositoryTopologyStatusSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owner_hints: Vec<String>,
    pub owner_hint_confidence: String,
    pub owner_hint_basis: RepositoryTopologyOwnerHintBasis,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub focus_clusters: Vec<RepositoryTopologyFocusCluster>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visible_finding_previews: Vec<RepositoryTopologyFindingPreview>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guardian_packet_previews: Vec<RepositoryTopologyPacketPreview>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_state_flow_previews: Vec<RepositoryTopologyStateFlowPreview>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_zones: Vec<RepositoryTopologyLinkedZone>,
    pub example_files: Vec<String>,
    pub manifest_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyOwnerHintBasis {
    pub sampled_file_count: usize,
    pub observed_file_count: usize,
    pub total_observations: usize,
    pub unique_contributor_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyFindingPreview {
    pub id: String,
    pub severity: String,
    pub family: String,
    pub title: String,
    pub summary: String,
    pub file_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyPacketPreview {
    pub id: String,
    pub priority: String,
    pub focus: String,
    pub summary: String,
    pub primary_target_file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_mechanism: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_anchor: Option<EvidenceAnchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doctrine_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finding_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyStateFlowPreview {
    pub id: String,
    pub label: String,
    pub carrier_type: String,
    pub slot: String,
    pub kind: AgenticSemanticStateFlowKind,
    pub writer_file: String,
    pub reader_file: String,
    pub proof_tier: SemanticStateProofKind,
    pub writer_proof: SemanticStateProofKind,
    pub reader_proof: SemanticStateProofKind,
    pub aggregate_confidence_millis: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyStateFlowRef {
    pub id: String,
    pub label: String,
    pub kind: AgenticSemanticStateFlowKind,
    pub proof_tier: SemanticStateProofKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepositoryTopologySemanticStateProofSummary {
    pub exact_resolved_count: usize,
    pub receiver_typed_count: usize,
    pub heuristic_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyLinkedZone {
    pub path: String,
    pub direction: String,
    pub relation_count: usize,
    pub relation_kinds: Vec<RepositoryTopologyRelationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<RepositoryTopologyCrossZoneExample>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub support_paths: Vec<AgenticGraphTrace>,
    pub linked_visible_finding_count: usize,
    pub linked_guardian_packet_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_visible_finding_severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_guardian_packet_priority: Option<String>,
    pub triage_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyRecommendedSlice {
    pub zone_path: String,
    pub target_file: String,
    pub label: String,
    pub priority: String,
    pub reason: String,
    pub supporting_zone_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepositoryTopologyStatusSummary {
    pub new_count: usize,
    pub worsened_count: usize,
    pub improved_count: usize,
    pub unchanged_count: usize,
    pub resolved_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyTriageStep {
    pub action: String,
    pub priority: String,
    pub target_file: String,
    pub step_kind: String,
    pub freshness: String,
    pub finding_status_summary: RepositoryTopologyStatusSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doctrine_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_state_flow_labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_state_flow_refs: Vec<RepositoryTopologyStateFlowRef>,
    pub semantic_state_flow_proof_summary: RepositoryTopologySemanticStateProofSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causal_bridges: Vec<RepositoryTopologySupportBridge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyFocusCluster {
    pub id: String,
    pub label: String,
    pub primary_target_file: String,
    pub freshness: String,
    pub finding_status_summary: RepositoryTopologyStatusSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_visible_finding_severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_guardian_packet_priority: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visible_finding_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guardian_packet_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub example_files: Vec<String>,
    pub triage_summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_state_flow_labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_state_flow_refs: Vec<RepositoryTopologyStateFlowRef>,
    pub semantic_state_flow_proof_summary: RepositoryTopologySemanticStateProofSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causal_bridges: Vec<RepositoryTopologySupportBridge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyLink {
    pub source_zone: String,
    pub target_zone: String,
    pub relation_count: usize,
    pub relation_kinds: Vec<RepositoryTopologyRelationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<RepositoryTopologyCrossZoneExample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyRelationCount {
    pub relation: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyCrossZoneExample {
    pub source_file: String,
    pub target_file: String,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologySupportBridge {
    pub linked_zone_path: String,
    pub direction: String,
    pub summary: String,
    pub trace_label: String,
    pub source_file: String,
    pub target_file: String,
    pub relation_sequence: Vec<String>,
    pub aggregate_confidence_millis: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyFileRef {
    pub path: String,
    pub zone_path: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyContractZone {
    pub path: String,
    pub route_count: usize,
    pub hook_count: usize,
    pub registered_key_count: usize,
    pub symbolic_literal_count: usize,
    pub env_key_count: usize,
    pub config_key_count: usize,
    pub example_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceHistoryArtifact {
    pub root: String,
    pub summary: ConvergenceSummary,
    pub graph_delta: ConvergenceGraphDelta,
    pub contract_delta: ConvergenceContractDelta,
    pub required_investigation_files: Vec<String>,
    pub required_radius: ConvergenceRequiredRadius,
    pub attention_items: Vec<ConvergenceAttentionItem>,
    pub findings: Vec<ConvergenceFindingDelta>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceSummary {
    pub current_findings: usize,
    pub previous_findings: usize,
    pub new_findings: usize,
    pub worsened_findings: usize,
    pub improved_findings: usize,
    pub unchanged_findings: usize,
    pub resolved_findings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceGraphDelta {
    pub strong_cycle_delta: isize,
    pub total_cycle_delta: isize,
    pub bottleneck_delta: isize,
    pub architectural_smell_delta: isize,
    pub warning_heavy_hotspot_delta: isize,
    pub split_identity_model_delta: isize,
    pub compatibility_scar_delta: isize,
    pub duplicate_mechanism_delta: isize,
    pub sanctioned_path_bypass_delta: isize,
    #[serde(default)]
    pub hand_rolled_parsing_delta: isize,
    #[serde(default)]
    pub abstraction_sprawl_delta: isize,
    #[serde(default)]
    pub algorithmic_complexity_hotspot_delta: isize,
    pub visible_finding_delta: isize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceRequiredRadius {
    pub anchor_files: Vec<String>,
    pub one_hop_files: Vec<String>,
    pub inbound_neighbor_count: usize,
    pub outbound_neighbor_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceContractDelta {
    pub routes: ContractValueDelta,
    pub hooks: ContractValueDelta,
    pub registered_keys: ContractValueDelta,
    pub symbolic_literals: ContractValueDelta,
    pub env_keys: ContractValueDelta,
    pub config_keys: ContractValueDelta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractValueDelta {
    pub added_count: usize,
    pub removed_count: usize,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvergenceStatus {
    New,
    Worsened,
    Improved,
    Unchanged,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceFindingDelta {
    pub fingerprint: String,
    pub current_id: Option<String>,
    pub previous_id: Option<String>,
    pub title: String,
    pub family: String,
    pub status: ConvergenceStatus,
    pub current_severity: Option<String>,
    pub previous_severity: Option<String>,
    pub current_visible: Option<bool>,
    pub previous_visible: Option<bool>,
    pub file_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceAttentionItem {
    pub fingerprint: String,
    pub status: ConvergenceStatus,
    pub title: String,
    pub family: String,
    pub precision: String,
    pub confidence_millis: u16,
    pub summary: String,
    pub file_paths: Vec<String>,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
    pub obligations: Vec<GuardianObligation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardVerdict {
    Allow,
    Warn,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardTriggerLevel {
    Warn,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardDecisionTrigger {
    pub level: GuardTriggerLevel,
    pub message: String,
    pub precision: String,
    pub confidence_millis: u16,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardDecisionPressure {
    pub new_findings: usize,
    pub worsened_findings: usize,
    pub attention_items: usize,
    pub exact_or_modeled_attention_items: usize,
    pub heuristic_attention_items: usize,
    pub required_radius_anchor_files: usize,
    pub required_radius_one_hop_files: usize,
    pub visible_finding_delta: isize,
    pub contract_delta_count: usize,
    pub high_severity_security_regressions: usize,
    pub cycle_regression: bool,
    pub bottleneck_regression: bool,
    pub architectural_smell_regression: bool,
    pub warning_heavy_hotspot_regression: bool,
    pub split_identity_model_regression: bool,
    pub compatibility_scar_regression: bool,
    pub duplicate_mechanism_regression: bool,
    pub sanctioned_path_bypass_regression: bool,
    #[serde(default)]
    pub hand_rolled_parsing_regression: bool,
    #[serde(default)]
    pub abstraction_sprawl_regression: bool,
    #[serde(default)]
    pub algorithmic_complexity_hotspot_regression: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardDecisionArtifact {
    pub root: String,
    pub verdict: GuardVerdict,
    pub confidence_millis: u16,
    pub summary: String,
    pub reasons: Vec<String>,
    pub triggers: Vec<GuardDecisionTrigger>,
    pub doctrine_refs: Vec<String>,
    pub obligations: Vec<GuardianObligation>,
    pub required_radius: ConvergenceRequiredRadius,
    pub attention_items: Vec<ConvergenceAttentionItem>,
    pub pressure: GuardDecisionPressure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FeedbackLoopSummary {
    pub detected_total: usize,
    pub actionable_visible: usize,
    pub accepted_by_policy: usize,
    pub suppressed_by_rule: usize,
    pub ai_reviewed: usize,
    pub rules_generated: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentHandoffArtifact {
    pub root: String,
    pub summary: AgentHandoffSummary,
    pub feedback_loop: FeedbackLoopSummary,
    pub next_steps: Vec<String>,
    pub guardian_packets: Vec<GuardianPacket>,
    pub top_findings: Vec<AgentHandoffFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentHandoffSummary {
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub strong_cycle_count: usize,
    pub bottleneck_count: usize,
    pub architectural_smell_count: usize,
    pub warning_heavy_hotspot_count: usize,
    pub split_identity_model_count: usize,
    pub compatibility_scar_count: usize,
    pub duplicate_mechanism_count: usize,
    pub sanctioned_path_bypass_count: usize,
    pub hand_rolled_parsing_count: usize,
    pub abstraction_sprawl_count: usize,
    pub algorithmic_complexity_hotspot_count: usize,
    pub visible_findings: usize,
    pub dead_code_count: usize,
    pub hardwiring_count: usize,
    pub security_finding_count: usize,
    pub external_finding_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentHandoffFinding {
    pub id: String,
    pub family: String,
    pub phase: String,
    pub severity: String,
    pub title: String,
    pub summary: String,
    pub file_paths: Vec<String>,
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_anchor: Option<EvidenceAnchor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<EvidenceAnchor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GuardianPacket {
    pub id: String,
    pub priority: String,
    pub focus: String,
    pub primary_target_file: String,
    pub precision: String,
    pub confidence_millis: u16,
    pub summary: String,
    pub target_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_anchor: Option<EvidenceAnchor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence_anchors: Vec<EvidenceAnchor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<EvidenceAnchor>,
    pub finding_ids: Vec<String>,
    pub context_labels: Vec<String>,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
    pub preferred_mechanism: Option<String>,
    pub obligations: Vec<GuardianObligation>,
    pub suppressibility: GuardianSuppressibility,
    pub investigation_questions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GuardianObligation {
    pub action: String,
    pub acceptance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GuardianSuppressibility {
    pub allowed: bool,
    pub requires_reason: bool,
    pub expiry_required: bool,
}

pub fn default_output_dir(root: &Path) -> PathBuf {
    root.join(DEFAULT_OUTPUT_DIR_NAME)
}

pub fn default_agent_run_paths(root: &Path, output_dir: Option<&Path>) -> AgentRunPaths {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(root));
    AgentRunPaths {
        review_json: output_dir.join(AGENT_REVIEW_JSON_FILE),
        review_markdown: output_dir.join(AGENT_REVIEW_MARKDOWN_FILE),
        output_schema: output_dir.join(AGENT_OUTPUT_SCHEMA_FILE),
        execution_events: output_dir.join(AGENT_EXECUTION_EVENTS_FILE),
        output_dir,
    }
}

pub fn default_agent_spider_report_path(root: &Path, output_dir: Option<&Path>) -> PathBuf {
    output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(root))
        .join(AGENT_SPIDER_REPORT_FILE)
}

pub fn write_project_analysis_artifacts(
    analysis: &ProjectAnalysis,
    output_dir: Option<&Path>,
) -> io::Result<ArtifactPaths> {
    trace_artifact_step("write.begin", 0);
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&analysis.root));
    fs::create_dir_all(&output_dir)?;
    trace_artifact_step("write.output_dir_ready", 0);

    let paths = ArtifactPaths {
        deterministic_analysis: output_dir.join(DETERMINISTIC_ANALYSIS_FILE),
        semantic_graph: output_dir.join(SEMANTIC_GRAPH_FILE),
        dependency_graph: output_dir.join(DEPENDENCY_GRAPH_FILE),
        evidence_graph: output_dir.join(EVIDENCE_GRAPH_FILE),
        contract_inventory: output_dir.join(CONTRACT_INVENTORY_FILE),
        doctrine_registry: output_dir.join(DOCTRINE_REGISTRY_FILE),
        deterministic_findings: output_dir.join(DETERMINISTIC_FINDINGS_FILE),
        ast_grep_scan: output_dir.join(AST_GREP_SCAN_FILE),
        external_analysis: output_dir.join(EXTERNAL_ANALYSIS_FILE),
        architecture_surface: output_dir.join(ARCHITECTURE_SURFACE_FILE),
        review_surface: output_dir.join(REVIEW_SURFACE_FILE),
        convergence_history: output_dir.join(CONVERGENCE_HISTORY_FILE),
        guard_decision: output_dir.join(GUARD_DECISION_FILE),
        agent_handoff: output_dir.join(AGENT_HANDOFF_FILE),
        agentic_review: output_dir.join(AGENTIC_REVIEW_FILE),
        graph_packets: output_dir.join(GRAPH_PACKETS_FILE),
        repository_topology: output_dir.join(REPOSITORY_TOPOLOGY_FILE),
        aigiscode_report: output_dir.join(AIGISCODE_REPORT_FILE),
        aigiscode_report_markdown: output_dir.join(AIGISCODE_REPORT_MARKDOWN_FILE),
        output_dir,
    };

    let findings = DeterministicFindingsArtifact {
        root: &analysis.root,
        scanned_files: analysis.scan.files.len(),
        analyzed_files: analysis.semantic_graph.files.len(),
        symbols: analysis.semantic_graph.symbols.len(),
        references: analysis.semantic_graph.references.len(),
        resolved_edges: analysis.semantic_graph.resolved_edges.len(),
        graph_analysis: &analysis.graph_analysis,
        architectural_assessment: &analysis.architectural_assessment,
        dead_code: &analysis.dead_code,
        hardwiring: &analysis.hardwiring,
        ast_grep_scan: &analysis.ast_grep_scan,
        security_analysis: &analysis.security_analysis,
        contract_inventory: &analysis.contract_inventory,
        timings: &analysis.timings,
    };
    let surface_started = Instant::now();
    let surface = analysis.architecture_surface();
    trace_artifact_step(
        "architecture_surface.build",
        surface_started.elapsed().as_millis(),
    );
    let dependency_graph = DependencyGraphJsonArtifact {
        root: &analysis.root,
        dependency_graph: build_dependency_graph_artifact(&analysis.semantic_graph),
    };
    let evidence_graph = EvidenceGraphJsonArtifact {
        root: &analysis.root,
        evidence_graph: build_evidence_graph_artifact(&analysis.semantic_graph),
    };
    let previous_architecture_surface =
        read_json_artifact_if_exists::<ArchitectureSurface>(&paths.architecture_surface)?;
    let previous_review_surface =
        read_json_artifact_if_exists::<ReviewSurface>(&paths.review_surface)?;
    let previous_contract_inventory =
        read_json_artifact_if_exists::<ContractInventory>(&paths.contract_inventory)?;
    let policy_bundle = PolicyBundle::load(&analysis.root).map_err(policy_error_to_io)?;
    let doctrine_registry = load_doctrine_registry(&analysis.root).map_err(doctrine_error_to_io)?;
    let review_started = Instant::now();
    let review_surface = build_review_surface(analysis, &surface, &policy_bundle);
    trace_artifact_step("review_surface.build", review_started.elapsed().as_millis());
    let convergence_started = Instant::now();
    let convergence_history = build_convergence_history_artifact(
        &analysis.root,
        &analysis.semantic_graph,
        previous_architecture_surface.as_ref(),
        previous_review_surface.as_ref(),
        previous_contract_inventory.as_ref(),
        &surface,
        &review_surface,
        &analysis.contract_inventory,
        &doctrine_registry,
    );
    trace_artifact_step(
        "convergence.build",
        convergence_started.elapsed().as_millis(),
    );
    let guard_started = Instant::now();
    let guard_decision = build_guard_decision_artifact(&analysis.root, &convergence_history);
    trace_artifact_step("guard.build", guard_started.elapsed().as_millis());
    let feedback_loop = build_feedback_loop_summary(&review_surface);
    let handoff_started = Instant::now();
    let agent_handoff = build_agent_handoff_artifact(analysis, &review_surface, &doctrine_registry);
    trace_artifact_step("agent_handoff.build", handoff_started.elapsed().as_millis());
    let agentic_started = Instant::now();
    let agentic_review = build_agentic_review_artifact(
        analysis,
        &doctrine_registry,
        &agent_handoff,
        &guard_decision,
        &convergence_history,
    );
    trace_artifact_step(
        "agentic_review.build",
        agentic_started.elapsed().as_millis(),
    );
    let graph_packets_started = Instant::now();
    let graph_packets = build_graph_packet_artifact(&agentic_review, analysis);
    trace_artifact_step(
        "graph_packets.build",
        graph_packets_started.elapsed().as_millis(),
    );
    let topology_started = Instant::now();
    let repository_topology = build_repository_topology_artifact(
        analysis,
        Some(&review_surface),
        Some(&agent_handoff),
        Some(&convergence_history),
        Some(&graph_packets),
    );
    trace_artifact_step(
        "repository_topology.build",
        topology_started.elapsed().as_millis(),
    );
    let ast_grep_family_counts = analysis.ast_grep_scan.family_counts();
    let report = AigiscodeReportArtifact {
        root: &analysis.root,
        summary: ReportSummary {
            scanned_files: analysis.scan.files.len(),
            analyzed_files: analysis.semantic_graph.files.len(),
            symbols: analysis.semantic_graph.symbols.len(),
            references: analysis.semantic_graph.references.len(),
            resolved_edges: analysis.semantic_graph.resolved_edges.len(),
            strong_cycle_count: analysis.graph_analysis.strong_circular_dependencies.len(),
            total_cycle_count: analysis.graph_analysis.circular_dependencies.len(),
            architectural_smell_count: analysis.graph_analysis.architectural_smells.len(),
            hub_like_dependency_count: analysis
                .graph_analysis
                .architectural_smells
                .iter()
                .filter(|smell| {
                    smell.kind == crate::graph::analysis::ArchitecturalSmellKind::HubLikeDependency
                })
                .count(),
            unstable_dependency_count: analysis
                .graph_analysis
                .architectural_smells
                .iter()
                .filter(|smell| {
                    smell.kind == crate::graph::analysis::ArchitecturalSmellKind::UnstableDependency
                })
                .count(),
            warning_heavy_hotspot_count: analysis
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::WarningHeavyHotspot),
            split_identity_model_count: analysis
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::SplitIdentityModel),
            compatibility_scar_count: analysis
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::CompatibilityScar),
            duplicate_mechanism_count: analysis
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::DuplicateMechanism),
            sanctioned_path_bypass_count: analysis.architectural_assessment.count_by_kind(
                crate::assessment::ArchitecturalAssessmentKind::SanctionedPathBypass,
            ),
            hand_rolled_parsing_count: analysis
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::HandRolledParsing),
            abstraction_sprawl_count: analysis
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::AbstractionSprawl),
            algorithmic_complexity_hotspot_count: analysis.architectural_assessment.count_by_kind(
                crate::assessment::ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot,
            ),
            ast_grep_finding_count: analysis.ast_grep_scan.findings.len(),
            ast_grep_algorithmic_complexity_count: ast_grep_family_counts.algorithmic_complexity,
            ast_grep_security_dangerous_api_count: ast_grep_family_counts.security_dangerous_api,
            ast_grep_framework_misuse_count: ast_grep_family_counts.framework_misuse,
            ast_grep_skipped_file_count: analysis.ast_grep_scan.skipped_files.len(),
            ast_grep_skipped_bytes: analysis
                .ast_grep_scan
                .skipped_files
                .iter()
                .map(|file| file.bytes)
                .sum(),
            ast_grep_skipped_files_preview: analysis
                .ast_grep_scan
                .skipped_files
                .iter()
                .take(5)
                .cloned()
                .collect(),
            dead_code_count: analysis.dead_code.findings.len(),
            hardwiring_count: analysis.hardwiring.findings.len(),
            security_finding_count: analysis.security_analysis.findings.len(),
            declared_route_count: analysis.contract_inventory.summary.routes.unique_values,
            declared_hook_count: analysis.contract_inventory.summary.hooks.unique_values,
            declared_registered_key_count: analysis
                .contract_inventory
                .summary
                .registered_keys
                .unique_values,
            declared_symbolic_literal_count: analysis
                .contract_inventory
                .summary
                .symbolic_literals
                .unique_values,
            declared_env_key_count: analysis.contract_inventory.summary.env_keys.unique_values,
            declared_config_key_count: analysis
                .contract_inventory
                .summary
                .config_keys
                .unique_values,
            external_tool_count: analysis.external_analysis.tool_runs.len(),
            external_finding_count: analysis.external_analysis.findings.len(),
            visible_findings: review_surface.summary.visible_findings,
            accepted_by_policy: review_surface.summary.accepted_by_policy,
            suppressed_by_rule: review_surface.summary.suppressed_by_rule,
            new_findings: convergence_history.summary.new_findings,
            worsened_findings: convergence_history.summary.worsened_findings,
            improved_findings: convergence_history.summary.improved_findings,
            resolved_findings: convergence_history.summary.resolved_findings,
        },
        feedback_loop: feedback_loop.clone(),
        graph_analysis: &analysis.graph_analysis,
        dead_code: &analysis.dead_code,
        hardwiring: &analysis.hardwiring,
        security_analysis: &analysis.security_analysis,
        contract_inventory: &analysis.contract_inventory,
        doctrine_registry: doctrine_registry.clone(),
        external_analysis: &analysis.external_analysis,
        architecture_surface: &surface,
        review_surface: &review_surface,
        convergence_history: &convergence_history,
        guard_decision: &guard_decision,
        agent_handoff: &agent_handoff,
        agentic_review: &agentic_review,
        graph_packets: &graph_packets,
        repository_topology: &repository_topology,
        timings: &analysis.timings,
    };

    let write_started = Instant::now();
    let report_payload = serialize_json_pretty(&report, &paths.aigiscode_report)?;
    write_json_payload(
        "deterministic_analysis",
        &paths.deterministic_analysis,
        &report_payload,
    )?;
    write_json_with_style(
        "semantic_graph",
        &paths.semantic_graph,
        &analysis.semantic_graph,
        JsonArtifactStyle::Compact,
    )?;
    write_json_with_style(
        "dependency_graph",
        &paths.dependency_graph,
        &dependency_graph,
        JsonArtifactStyle::Compact,
    )?;
    write_json_with_style(
        "evidence_graph",
        &paths.evidence_graph,
        &evidence_graph,
        JsonArtifactStyle::Compact,
    )?;
    write_json(
        "contract_inventory",
        &paths.contract_inventory,
        &analysis.contract_inventory,
    )?;
    write_json(
        "doctrine_registry",
        &paths.doctrine_registry,
        &doctrine_registry,
    )?;
    write_json_with_style(
        "deterministic_findings",
        &paths.deterministic_findings,
        &findings,
        JsonArtifactStyle::Compact,
    )?;
    write_json(
        "ast_grep_scan",
        &paths.ast_grep_scan,
        &analysis.ast_grep_scan,
    )?;
    write_json(
        "external_analysis",
        &paths.external_analysis,
        &analysis.external_analysis,
    )?;
    write_json(
        "architecture_surface",
        &paths.architecture_surface,
        &surface,
    )?;
    write_json("review_surface", &paths.review_surface, &review_surface)?;
    write_json(
        "convergence_history",
        &paths.convergence_history,
        &convergence_history,
    )?;
    write_json("guard_decision", &paths.guard_decision, &guard_decision)?;
    write_json("agent_handoff", &paths.agent_handoff, &agent_handoff)?;
    write_json("agentic_review", &paths.agentic_review, &agentic_review)?;
    write_json("graph_packets", &paths.graph_packets, &graph_packets)?;
    write_json(
        "repository_topology",
        &paths.repository_topology,
        &repository_topology,
    )?;
    write_json_payload("aigiscode_report", &paths.aigiscode_report, &report_payload)?;
    trace_artifact_step("json.write", write_started.elapsed().as_millis());
    let markdown_started = Instant::now();
    write_markdown(
        &paths.aigiscode_report_markdown,
        &build_markdown_report(analysis, &report, &agent_handoff),
    )?;
    trace_artifact_step("markdown.write", markdown_started.elapsed().as_millis());

    Ok(paths)
}

fn trace_artifact_step(step: &str, elapsed_ms: u128) {
    if std::env::var_os("AIGISCORE_TRACE").is_some() {
        eprintln!("[aigiscore] artifact {step} elapsed_ms={elapsed_ms}");
    }
}

pub fn build_repository_topology_artifact(
    analysis: &ProjectAnalysis,
    review_surface: Option<&ReviewSurface>,
    handoff: Option<&AgentHandoffArtifact>,
    convergence_history: Option<&ConvergenceHistoryArtifact>,
    graph_packets: Option<&GraphPacketArtifact>,
) -> RepositoryTopologyArtifact {
    #[derive(Default)]
    struct ZoneAccumulator {
        scanned_files: usize,
        analyzed_files: usize,
        total_size_bytes: u64,
        languages: BTreeSet<String>,
        runtime_entry_count: usize,
        boundary_truncated_count: usize,
        contract_count: usize,
        visible_finding_ids: BTreeSet<String>,
        guardian_packet_ids: BTreeSet<String>,
        example_files: BTreeSet<String>,
        manifest_files: BTreeSet<String>,
    }

    let mut zones = BTreeMap::<String, ZoneAccumulator>::new();
    let mut manifests = Vec::new();
    let graph_context = build_graph_query_context(&analysis.semantic_graph);
    let mut trace_cache = TopologyTraceCache::new();
    let visible_finding_lookup = review_surface
        .map(|surface| {
            surface
                .findings
                .iter()
                .filter(|finding| finding.is_visible)
                .map(|finding| (finding.id.clone(), finding))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let guardian_packet_lookup = handoff
        .map(|artifact| {
            artifact
                .guardian_packets
                .iter()
                .map(|packet| (packet.id.clone(), packet))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let convergence_status_lookup = convergence_history
        .map(|history| build_topology_status_lookup(&history.findings))
        .unwrap_or_default();
    let convergence_finding_status_lookup = convergence_history
        .map(|history| build_topology_finding_status_lookup(&history.findings))
        .unwrap_or_default();
    let convergence_file_status_lookup = convergence_history
        .map(|history| build_topology_file_status_lookup(&history.findings))
        .unwrap_or_default();
    let semantic_state_flow_lookup = graph_packets
        .map(build_topology_state_flow_lookup)
        .unwrap_or_default();
    let mut owner_hint_cache = BTreeMap::<String, Vec<String>>::new();

    for file in &analysis.scan.files {
        let zone_path = topology_zone_path(&file.relative_path);
        let zone = zones.entry(zone_path.clone()).or_default();
        zone.scanned_files += 1;
        zone.total_size_bytes += file.size_bytes;
        zone.example_files
            .insert(file.relative_path.to_string_lossy().replace('\\', "/"));
        if let Some((manifest_path, ecosystem)) = detect_repository_manifest(&file.relative_path) {
            manifests.push(RepositoryManifest {
                path: manifest_path.to_string_lossy().replace('\\', "/"),
                zone_path: zone_path.clone(),
                ecosystem: ecosystem.to_string(),
            });
            zone.manifest_files
                .insert(file.relative_path.to_string_lossy().replace('\\', "/"));
        }
    }

    let file_languages = analysis
        .semantic_graph
        .files
        .iter()
        .map(|file| {
            (
                file.path.clone(),
                graph_language_label(file.language).to_string(),
            )
        })
        .collect::<HashMap<_, _>>();

    for file in &analysis.semantic_graph.files {
        let zone_path = topology_zone_path(&file.path);
        let zone = zones.entry(zone_path).or_default();
        zone.analyzed_files += 1;
        zone.languages
            .insert(graph_language_label(file.language).to_string());
    }

    let mut runtime_entry_file_map = BTreeMap::<String, RepositoryTopologyFileRef>::new();
    for path in &analysis.graph_analysis.runtime_entry_candidates {
        let zone_path = topology_zone_path(path);
        let path_string = path.to_string_lossy().replace('\\', "/");
        runtime_entry_file_map
            .entry(path_string.clone())
            .or_insert_with(|| RepositoryTopologyFileRef {
                path: path_string,
                zone_path,
                language: file_languages.get(path).cloned(),
            });
    }
    for route in &analysis.contract_inventory.routes {
        for location in &route.locations {
            let zone_path = topology_zone_path(&location.file_path);
            let path_string = location.file_path.to_string_lossy().replace('\\', "/");
            runtime_entry_file_map
                .entry(path_string.clone())
                .or_insert_with(|| RepositoryTopologyFileRef {
                    path: path_string,
                    zone_path,
                    language: file_languages.get(&location.file_path).cloned(),
                });
        }
    }
    let runtime_entry_files = runtime_entry_file_map
        .into_values()
        .map(|file_ref| {
            zones
                .entry(file_ref.zone_path.clone())
                .or_default()
                .runtime_entry_count += 1;
            file_ref
        })
        .collect::<Vec<_>>();

    let boundary_truncated_files = crate::surface::effective_boundary_truncated_files(analysis);
    for path in &boundary_truncated_files {
        let zone_path = topology_zone_path(path);
        zones.entry(zone_path).or_default().boundary_truncated_count += 1;
    }

    let contract_zones = build_topology_contract_zones(&analysis.contract_inventory);
    for contract_zone in &contract_zones {
        if let Some(zone) = zones.get_mut(&contract_zone.path) {
            zone.contract_count = contract_zone.route_count
                + contract_zone.hook_count
                + contract_zone.registered_key_count
                + contract_zone.symbolic_literal_count
                + contract_zone.env_key_count
                + contract_zone.config_key_count;
        }
    }

    if let Some(review_surface) = review_surface {
        for finding in review_surface
            .findings
            .iter()
            .filter(|finding| finding.is_visible)
        {
            let affected_zones = finding
                .file_paths
                .iter()
                .map(|file_path| topology_zone_path(Path::new(file_path)))
                .collect::<BTreeSet<_>>();
            for zone_path in affected_zones {
                zones
                    .entry(zone_path)
                    .or_default()
                    .visible_finding_ids
                    .insert(finding.id.clone());
            }
        }
    }

    if let Some(handoff) = handoff {
        for packet in &handoff.guardian_packets {
            let affected_zones = packet
                .target_files
                .iter()
                .chain(std::iter::once(&packet.primary_target_file))
                .map(|file_path| topology_zone_path(Path::new(file_path)))
                .collect::<BTreeSet<_>>();
            for zone_path in affected_zones {
                zones
                    .entry(zone_path)
                    .or_default()
                    .guardian_packet_ids
                    .insert(packet.id.clone());
            }
        }
    }

    #[derive(Default)]
    struct LinkAccumulator {
        counts: BTreeMap<String, usize>,
        examples: BTreeSet<(String, String, String)>,
    }

    let mut link_counts = BTreeMap::<(String, String), LinkAccumulator>::new();
    for edge in &analysis.semantic_graph.resolved_edges {
        let source_zone = topology_zone_path(&edge.source_file_path);
        let target_zone = topology_zone_path(&edge.target_file_path);
        if source_zone == target_zone {
            continue;
        }
        let relation = relation_kind_label(edge.relation_kind).to_string();
        let accumulator = link_counts.entry((source_zone, target_zone)).or_default();
        *accumulator.counts.entry(relation.clone()).or_default() += 1;
        accumulator.examples.insert((
            edge.source_file_path.to_string_lossy().replace('\\', "/"),
            edge.target_file_path.to_string_lossy().replace('\\', "/"),
            relation,
        ));
    }

    let mut links = link_counts
        .into_iter()
        .map(|((source_zone, target_zone), accumulator)| {
            let relation_count = accumulator.counts.values().sum();
            let mut relation_kinds = accumulator
                .counts
                .into_iter()
                .map(|(relation, count)| RepositoryTopologyRelationCount { relation, count })
                .collect::<Vec<_>>();
            relation_kinds.sort_by(|left, right| {
                right
                    .count
                    .cmp(&left.count)
                    .then(left.relation.cmp(&right.relation))
            });
            RepositoryTopologyLink {
                source_zone,
                target_zone,
                relation_count,
                relation_kinds,
                examples: accumulator
                    .examples
                    .into_iter()
                    .take(3)
                    .map(|(source_file, target_file, relation)| {
                        RepositoryTopologyCrossZoneExample {
                            source_file,
                            target_file,
                            relation,
                        }
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>();
    links.sort_by(|left, right| {
        right
            .relation_count
            .cmp(&left.relation_count)
            .then(left.source_zone.cmp(&right.source_zone))
            .then(left.target_zone.cmp(&right.target_zone))
    });
    links.truncate(40);

    let mut zones = zones
        .into_iter()
        .map(|(path, zone)| {
            let visible_finding_ids = zone
                .visible_finding_ids
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>();
            let guardian_packet_ids = zone
                .guardian_packet_ids
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>();
            let mut visible_finding_previews = visible_finding_ids
                .iter()
                .filter_map(|id| visible_finding_lookup.get(id))
                .map(|finding| RepositoryTopologyFindingPreview {
                    id: finding.id.clone(),
                    severity: review_severity_label(finding.severity),
                    family: review_family_label(finding.family).to_string(),
                    title: finding.title.clone(),
                    summary: finding.summary.clone(),
                    file_paths: finding.file_paths.iter().take(3).cloned().collect(),
                    line: finding.line,
                    artifact_refs: vec![
                        String::from(REVIEW_SURFACE_FILE),
                        String::from(ARCHITECTURE_SURFACE_FILE),
                    ],
                })
                .collect::<Vec<_>>();
            visible_finding_previews.sort_by(|left, right| {
                preview_severity_rank(&right.severity)
                    .cmp(&preview_severity_rank(&left.severity))
                    .then(left.id.cmp(&right.id))
            });
            let mut guardian_packet_previews = guardian_packet_ids
                .iter()
                .filter_map(|id| guardian_packet_lookup.get(id))
                .map(|packet| RepositoryTopologyPacketPreview {
                    id: packet.id.clone(),
                    priority: packet.priority.clone(),
                    focus: packet.focus.clone(),
                    summary: packet.summary.clone(),
                    primary_target_file: packet.primary_target_file.clone(),
                    preferred_mechanism: packet.preferred_mechanism.clone(),
                    primary_anchor: packet.primary_anchor.clone(),
                    doctrine_refs: packet.doctrine_refs.iter().take(6).cloned().collect(),
                    finding_ids: packet.finding_ids.iter().take(8).cloned().collect(),
                    artifact_refs: vec![
                        String::from(AGENT_HANDOFF_FILE),
                        String::from(GRAPH_PACKETS_FILE),
                        String::from(AGENTIC_REVIEW_FILE),
                    ],
                })
                .collect::<Vec<_>>();
            guardian_packet_previews.sort_by(|left, right| {
                packet_priority_rank(&right.priority)
                    .cmp(&packet_priority_rank(&left.priority))
                    .then(packet_focus_rank(&right.focus).cmp(&packet_focus_rank(&left.focus)))
                    .then(left.id.cmp(&right.id))
            });
            let example_files = zone.example_files.into_iter().take(5).collect::<Vec<_>>();
            let (owner_hints, owner_hint_confidence, owner_hint_basis) =
                build_topology_owner_hints(&analysis.root, &example_files, &mut owner_hint_cache);
            let focus_clusters = build_topology_focus_clusters(
                &guardian_packet_previews,
                &visible_finding_previews,
                &convergence_finding_status_lookup,
                &convergence_file_status_lookup,
                &semantic_state_flow_lookup,
            );
            let semantic_state_flow_files = example_files
                .iter()
                .cloned()
                .chain(
                    guardian_packet_previews
                        .iter()
                        .map(|packet| packet.primary_target_file.clone()),
                )
                .chain(
                    visible_finding_previews
                        .iter()
                        .flat_map(|finding| finding.file_paths.iter().cloned()),
                )
                .chain(
                    focus_clusters
                        .iter()
                        .map(|cluster| cluster.primary_target_file.clone()),
                )
                .collect::<BTreeSet<_>>();
            let mut all_semantic_state_flow_previews = semantic_state_flow_files
                .iter()
                .flat_map(|file| {
                    semantic_state_flow_lookup
                        .get(file)
                        .into_iter()
                        .flatten()
                        .cloned()
                })
                .collect::<Vec<_>>();
            all_semantic_state_flow_previews.sort_by(|left, right| {
                topology_semantic_state_proof_rank(right.proof_tier)
                    .cmp(&topology_semantic_state_proof_rank(left.proof_tier))
                    .then(
                        right
                            .aggregate_confidence_millis
                            .cmp(&left.aggregate_confidence_millis),
                    )
                    .then(left.label.cmp(&right.label))
                    .then(left.writer_file.cmp(&right.writer_file))
                    .then(left.reader_file.cmp(&right.reader_file))
            });
            all_semantic_state_flow_previews.dedup_by(|left, right| {
                topology_state_flow_key(left) == topology_state_flow_key(right)
            });
            let semantic_state_flow_count = all_semantic_state_flow_previews.len();
            let semantic_state_flow_proof_summary =
                build_semantic_state_proof_summary(&all_semantic_state_flow_previews);
            let semantic_state_flow_previews = all_semantic_state_flow_previews
                .into_iter()
                .take(6)
                .collect::<Vec<_>>();
            let triage_next_steps = build_topology_zone_triage_steps(
                &guardian_packet_previews,
                &visible_finding_previews,
                &semantic_state_flow_previews,
            );
            let triage_steps = build_topology_zone_triage_step_objects(
                &guardian_packet_previews,
                &visible_finding_previews,
                &convergence_finding_status_lookup,
                &convergence_file_status_lookup,
                &semantic_state_flow_lookup,
            );
            let triage_summary = build_topology_zone_triage_summary(
                &path,
                &focus_clusters,
                &guardian_packet_previews,
                &visible_finding_previews,
                &semantic_state_flow_previews,
            );
            let finding_status_summary =
                build_topology_zone_status_summary(&path, &convergence_status_lookup);
            RepositoryTopologyZone {
                labels: classify_topology_zone_labels(
                    &path,
                    zone.manifest_files.is_empty().not(),
                    zone.runtime_entry_count > 0,
                    zone.contract_count > 0,
                    zone.languages
                        .iter()
                        .any(|language| matches!(language.as_str(), "javascript" | "typescript")),
                ),
                manifest_observation: topology_observation_label(
                    zone.manifest_files.is_empty().not(),
                ),
                path,
                runtime_entry_observation: topology_observation_label(zone.runtime_entry_count > 0),
                boundary_observation: topology_observation_label(zone.boundary_truncated_count > 0),
                spillover_observation: String::new(),
                scanned_files: zone.scanned_files,
                analyzed_files: zone.analyzed_files,
                total_size_bytes: zone.total_size_bytes,
                languages: zone.languages.into_iter().collect(),
                runtime_entry_count: zone.runtime_entry_count,
                boundary_truncated_count: zone.boundary_truncated_count,
                contract_count: zone.contract_count,
                visible_finding_count: zone.visible_finding_ids.len(),
                guardian_packet_count: zone.guardian_packet_ids.len(),
                semantic_state_flow_count,
                semantic_state_flow_proof_summary,
                inbound_cross_zone_link_count: 0,
                outbound_cross_zone_link_count: 0,
                inbound_cross_zone_relation_count: 0,
                outbound_cross_zone_relation_count: 0,
                highest_visible_finding_severity: visible_finding_previews
                    .first()
                    .map(|preview| preview.severity.clone()),
                highest_guardian_packet_priority: guardian_packet_previews
                    .first()
                    .map(|preview| preview.priority.clone()),
                visible_finding_ids,
                guardian_packet_ids,
                triage_summary,
                cross_zone_pressure_summary: String::new(),
                causal_bridges: Vec::new(),
                triage_next_steps,
                triage_steps,
                finding_status_summary,
                owner_hints,
                owner_hint_confidence,
                owner_hint_basis,
                focus_clusters,
                visible_finding_previews,
                guardian_packet_previews,
                semantic_state_flow_previews,
                linked_zones: Vec::new(),
                example_files,
                manifest_files: zone.manifest_files.into_iter().collect(),
            }
        })
        .collect::<Vec<_>>();

    #[derive(Clone)]
    struct ZoneLinkPreview {
        visible_finding_count: usize,
        guardian_packet_count: usize,
        highest_visible_finding_severity: Option<String>,
        highest_guardian_packet_priority: Option<String>,
        triage_summary: String,
        representative_file: Option<String>,
    }

    #[derive(Default)]
    struct LinkedZoneAccumulator {
        inbound_relation_count: usize,
        outbound_relation_count: usize,
        relation_kinds: BTreeMap<String, usize>,
        examples: BTreeSet<(String, String, String)>,
    }

    #[derive(Default)]
    struct ZonePressureAccumulator {
        inbound_link_count: usize,
        outbound_link_count: usize,
        inbound_relation_count: usize,
        outbound_relation_count: usize,
        linked_zones: BTreeMap<String, LinkedZoneAccumulator>,
    }

    let zone_preview_lookup = zones
        .iter()
        .map(|zone| {
            (
                zone.path.clone(),
                ZoneLinkPreview {
                    visible_finding_count: zone.visible_finding_count,
                    guardian_packet_count: zone.guardian_packet_count,
                    highest_visible_finding_severity: zone.highest_visible_finding_severity.clone(),
                    highest_guardian_packet_priority: zone.highest_guardian_packet_priority.clone(),
                    triage_summary: zone.triage_summary.clone(),
                    representative_file: zone
                        .focus_clusters
                        .first()
                        .map(|cluster| cluster.primary_target_file.clone())
                        .or_else(|| zone.example_files.first().cloned()),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut zone_pressure = BTreeMap::<String, ZonePressureAccumulator>::new();
    for link in &links {
        {
            let pressure = zone_pressure.entry(link.source_zone.clone()).or_default();
            pressure.outbound_link_count += 1;
            pressure.outbound_relation_count += link.relation_count;
            let linked_zone = pressure
                .linked_zones
                .entry(link.target_zone.clone())
                .or_default();
            linked_zone.outbound_relation_count += link.relation_count;
            for relation in &link.relation_kinds {
                *linked_zone
                    .relation_kinds
                    .entry(relation.relation.clone())
                    .or_default() += relation.count;
            }
            for example in &link.examples {
                linked_zone.examples.insert((
                    example.source_file.clone(),
                    example.target_file.clone(),
                    example.relation.clone(),
                ));
            }
        }
        {
            let pressure = zone_pressure.entry(link.target_zone.clone()).or_default();
            pressure.inbound_link_count += 1;
            pressure.inbound_relation_count += link.relation_count;
            let linked_zone = pressure
                .linked_zones
                .entry(link.source_zone.clone())
                .or_default();
            linked_zone.inbound_relation_count += link.relation_count;
            for relation in &link.relation_kinds {
                *linked_zone
                    .relation_kinds
                    .entry(relation.relation.clone())
                    .or_default() += relation.count;
            }
            for example in &link.examples {
                linked_zone.examples.insert((
                    example.source_file.clone(),
                    example.target_file.clone(),
                    example.relation.clone(),
                ));
            }
        }
    }

    for zone in &mut zones {
        let current_representative_file = zone
            .focus_clusters
            .first()
            .map(|cluster| cluster.primary_target_file.as_str())
            .or_else(|| zone.example_files.first().map(String::as_str));
        if let Some(pressure) = zone_pressure.get(&zone.path) {
            zone.inbound_cross_zone_link_count = pressure.inbound_link_count;
            zone.outbound_cross_zone_link_count = pressure.outbound_link_count;
            zone.inbound_cross_zone_relation_count = pressure.inbound_relation_count;
            zone.outbound_cross_zone_relation_count = pressure.outbound_relation_count;

            let mut linked_zones = pressure
                .linked_zones
                .iter()
                .map(|(linked_zone_path, linked)| {
                    let preview = zone_preview_lookup.get(linked_zone_path);
                    let relation_count =
                        linked.inbound_relation_count + linked.outbound_relation_count;
                    let mut relation_kinds = linked
                        .relation_kinds
                        .iter()
                        .map(|(relation, count)| RepositoryTopologyRelationCount {
                            relation: relation.clone(),
                            count: *count,
                        })
                        .collect::<Vec<_>>();
                    relation_kinds.sort_by(|left, right| {
                        right
                            .count
                            .cmp(&left.count)
                            .then(left.relation.cmp(&right.relation))
                    });
                    relation_kinds.truncate(3);
                    let mut support_paths = build_topology_cross_zone_support_paths(
                        &graph_context,
                        &mut trace_cache,
                        current_representative_file,
                        preview.and_then(|preview| preview.representative_file.as_deref()),
                        topology_link_direction(
                            linked.inbound_relation_count,
                            linked.outbound_relation_count,
                        )
                        .as_str(),
                    );
                    if support_paths.is_empty() {
                        for (source_file, target_file, _) in linked.examples.iter().take(2) {
                            let traces = graph_trace_between_files_cached(
                                &graph_context,
                                &mut trace_cache,
                                source_file,
                                target_file,
                                4,
                                1,
                            );
                            for trace in traces {
                                if support_paths.len() >= 2 {
                                    break;
                                }
                                support_paths.push(trace);
                            }
                            if support_paths.len() >= 2 {
                                break;
                            }
                        }
                    }
                    let mut seen_support_path_labels = BTreeSet::new();
                    support_paths
                        .retain(|trace| seen_support_path_labels.insert(trace.label.clone()));
                    RepositoryTopologyLinkedZone {
                        path: linked_zone_path.clone(),
                        direction: topology_link_direction(
                            linked.inbound_relation_count,
                            linked.outbound_relation_count,
                        ),
                        relation_count,
                        relation_kinds,
                        examples: linked
                            .examples
                            .iter()
                            .take(3)
                            .map(|(source_file, target_file, relation)| {
                                RepositoryTopologyCrossZoneExample {
                                    source_file: source_file.clone(),
                                    target_file: target_file.clone(),
                                    relation: relation.clone(),
                                }
                            })
                            .collect(),
                        support_paths,
                        linked_visible_finding_count: preview
                            .map(|preview| preview.visible_finding_count)
                            .unwrap_or(0),
                        linked_guardian_packet_count: preview
                            .map(|preview| preview.guardian_packet_count)
                            .unwrap_or(0),
                        highest_visible_finding_severity: preview
                            .and_then(|preview| preview.highest_visible_finding_severity.clone()),
                        highest_guardian_packet_priority: preview
                            .and_then(|preview| preview.highest_guardian_packet_priority.clone()),
                        triage_summary: preview
                            .map(|preview| preview.triage_summary.clone())
                            .unwrap_or_else(|| {
                                format!(
                                    "Linked zone `{}` has no direct triage summary.",
                                    linked_zone_path
                                )
                            }),
                    }
                })
                .collect::<Vec<_>>();
            linked_zones.sort_by(|left, right| {
                right
                    .relation_count
                    .cmp(&left.relation_count)
                    .then(
                        packet_priority_rank(
                            right
                                .highest_guardian_packet_priority
                                .as_deref()
                                .unwrap_or("low"),
                        )
                        .cmp(&packet_priority_rank(
                            left.highest_guardian_packet_priority
                                .as_deref()
                                .unwrap_or("low"),
                        )),
                    )
                    .then(left.path.cmp(&right.path))
            });
            linked_zones.truncate(5);
            let causal_bridges = build_topology_support_bridges(&linked_zones, 2);
            zone.cross_zone_pressure_summary =
                build_topology_cross_zone_pressure_summary(&zone.path, &linked_zones);
            zone.triage_summary = enrich_topology_zone_triage_summary_with_bridges(
                &zone.triage_summary,
                &causal_bridges,
            );
            zone.causal_bridges = causal_bridges.clone();
            let mut targeted_bridge_cache =
                BTreeMap::<String, Vec<RepositoryTopologySupportBridge>>::new();
            for step in &mut zone.triage_steps {
                let target_file = step.target_file.clone();
                let bridges = targeted_bridge_cache
                    .entry(target_file.clone())
                    .or_insert_with(|| {
                        build_topology_targeted_support_bridges(
                            &graph_context,
                            &mut trace_cache,
                            target_file.as_str(),
                            &linked_zones,
                            2,
                        )
                    })
                    .clone();
                step.causal_bridges = bridges;
            }
            for cluster in &mut zone.focus_clusters {
                let target_file = cluster.primary_target_file.clone();
                let bridges = targeted_bridge_cache
                    .entry(target_file.clone())
                    .or_insert_with(|| {
                        build_topology_targeted_support_bridges(
                            &graph_context,
                            &mut trace_cache,
                            target_file.as_str(),
                            &linked_zones,
                            2,
                        )
                    })
                    .clone();
                cluster.causal_bridges = bridges;
            }
            zone.linked_zones = linked_zones;
        } else {
            zone.cross_zone_pressure_summary =
                build_topology_cross_zone_pressure_summary(&zone.path, &[]);
        }
    }

    for zone in &mut zones {
        zone.spillover_observation = topology_spillover_observation(
            zone.guardian_packet_count,
            zone.visible_finding_count,
            &zone.linked_zones,
        );
    }

    let recommended_start = build_topology_recommended_start(&zones);

    let linked_visible_finding_count = zones
        .iter()
        .flat_map(|zone| zone.visible_finding_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len();
    let linked_guardian_packet_count = zones
        .iter()
        .flat_map(|zone| zone.guardian_packet_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len();
    let linked_semantic_state_flow_count = graph_packets
        .map(|packets| {
            packets
                .packets
                .iter()
                .flat_map(|packet| packet.semantic_state_flows.iter())
                .map(|flow| {
                    format!(
                        "{}|{}|{}|{}",
                        flow.label, flow.carrier_type, flow.writer.file_path, flow.reader.file_path
                    )
                })
                .collect::<BTreeSet<_>>()
                .len()
        })
        .unwrap_or_default();
    let linked_high_severity_visible_finding_count = zones
        .iter()
        .flat_map(|zone| zone.visible_finding_previews.iter())
        .filter(|preview| preview.severity == "high")
        .map(|preview| preview.id.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let linked_high_priority_guardian_packet_count = zones
        .iter()
        .flat_map(|zone| zone.guardian_packet_previews.iter())
        .filter(|preview| preview.priority == "high")
        .map(|preview| preview.id.clone())
        .collect::<BTreeSet<_>>()
        .len();

    RepositoryTopologyArtifact {
        root: analysis.root.display().to_string(),
        generated_at_unix_ms: topology_generated_at_unix_ms(),
        summary: RepositoryTopologySummary {
            scanned_files: analysis.scan.files.len(),
            analyzed_files: analysis.semantic_graph.files.len(),
            zone_count: zones.len(),
            manifest_count: manifests.len(),
            link_count: links.len(),
            runtime_entry_count: runtime_entry_files.len(),
            boundary_truncated_count: boundary_truncated_files.len(),
            boundary_truth: match analysis.scan.scope.boundary_truth {
                crate::ingestion::scan::AnalysisBoundaryTruth::CompleteRepository => {
                    String::from("complete_repository")
                }
                crate::ingestion::scan::AnalysisBoundaryTruth::TruncatedSlice => {
                    String::from("truncated_slice")
                }
            },
            boundary_reasons: analysis
                .scan
                .scope
                .reasons
                .iter()
                .map(|reason| match reason {
                    crate::ingestion::scan::AnalysisBoundaryReason::CroppedRoot => {
                        String::from("cropped_root")
                    }
                    crate::ingestion::scan::AnalysisBoundaryReason::IncludePathPrefixes => {
                        String::from("include_path_prefixes")
                    }
                })
                .collect(),
            contract_zone_count: contract_zones.len(),
            linked_visible_finding_count,
            linked_guardian_packet_count,
            linked_semantic_state_flow_count,
            linked_high_severity_visible_finding_count,
            linked_high_priority_guardian_packet_count,
            zones_with_cross_zone_pressure: zones
                .iter()
                .filter(|zone| !zone.linked_zones.is_empty())
                .count(),
            cross_zone_pressure_link_count: zones.iter().map(|zone| zone.linked_zones.len()).sum(),
            strongest_cross_zone_relation_count: zones
                .iter()
                .flat_map(|zone| zone.linked_zones.iter())
                .map(|linked_zone| linked_zone.relation_count)
                .max()
                .unwrap_or(0),
            baseline_observation: topology_baseline_observation(convergence_history),
            recommended_start,
        },
        manifests,
        zones,
        links,
        runtime_entry_files,
        contract_zones,
    }
}

pub fn build_feedback_loop_summary(review_surface: &ReviewSurface) -> FeedbackLoopSummary {
    FeedbackLoopSummary {
        detected_total: review_surface.summary.total_findings,
        actionable_visible: review_surface.summary.visible_findings,
        accepted_by_policy: review_surface.summary.accepted_by_policy,
        suppressed_by_rule: review_surface.summary.suppressed_by_rule,
        ai_reviewed: review_surface.summary.ai_reviewed,
        rules_generated: review_surface.summary.rules_generated,
    }
}

fn build_topology_contract_zones(
    inventory: &ContractInventory,
) -> Vec<RepositoryTopologyContractZone> {
    let mut zones = BTreeMap::<String, ContractZoneAccumulator>::new();
    for item in &inventory.routes {
        accumulate_contract_zone(
            &mut zones,
            &item.locations,
            item.count,
            ContractZoneCategory::Route,
        );
    }
    for item in &inventory.hooks {
        accumulate_contract_zone(
            &mut zones,
            &item.locations,
            item.count,
            ContractZoneCategory::Hook,
        );
    }
    for item in &inventory.registered_keys {
        accumulate_contract_zone(
            &mut zones,
            &item.locations,
            item.count,
            ContractZoneCategory::RegisteredKey,
        );
    }
    for item in &inventory.symbolic_literals {
        accumulate_contract_zone(
            &mut zones,
            &item.locations,
            item.count,
            ContractZoneCategory::SymbolicLiteral,
        );
    }
    for item in &inventory.env_keys {
        accumulate_contract_zone(
            &mut zones,
            &item.locations,
            item.count,
            ContractZoneCategory::EnvKey,
        );
    }
    for item in &inventory.config_keys {
        accumulate_contract_zone(
            &mut zones,
            &item.locations,
            item.count,
            ContractZoneCategory::ConfigKey,
        );
    }

    zones
        .into_iter()
        .map(|(path, zone)| RepositoryTopologyContractZone {
            path,
            route_count: zone.route_count,
            hook_count: zone.hook_count,
            registered_key_count: zone.registered_key_count,
            symbolic_literal_count: zone.symbolic_literal_count,
            env_key_count: zone.env_key_count,
            config_key_count: zone.config_key_count,
            example_files: zone.example_files.into_iter().take(5).collect(),
        })
        .collect()
}

#[derive(Default)]
struct ContractZoneAccumulator {
    route_count: usize,
    hook_count: usize,
    registered_key_count: usize,
    symbolic_literal_count: usize,
    env_key_count: usize,
    config_key_count: usize,
    example_files: BTreeSet<String>,
}

enum ContractZoneCategory {
    Route,
    Hook,
    RegisteredKey,
    SymbolicLiteral,
    EnvKey,
    ConfigKey,
}

fn accumulate_contract_zone(
    zones: &mut BTreeMap<String, ContractZoneAccumulator>,
    locations: &[crate::contracts::ContractLocation],
    count: usize,
    category: ContractZoneCategory,
) {
    let affected_zones = locations
        .iter()
        .map(|location| topology_zone_path(&location.file_path))
        .collect::<BTreeSet<_>>();
    for zone_path in affected_zones {
        let zone = zones.entry(zone_path).or_default();
        match category {
            ContractZoneCategory::Route => zone.route_count += count,
            ContractZoneCategory::Hook => zone.hook_count += count,
            ContractZoneCategory::RegisteredKey => zone.registered_key_count += count,
            ContractZoneCategory::SymbolicLiteral => zone.symbolic_literal_count += count,
            ContractZoneCategory::EnvKey => zone.env_key_count += count,
            ContractZoneCategory::ConfigKey => zone.config_key_count += count,
        }
    }
    for location in locations {
        let zone_path = topology_zone_path(&location.file_path);
        zones
            .entry(zone_path)
            .or_default()
            .example_files
            .insert(location.file_path.to_string_lossy().replace('\\', "/"));
    }
}

fn topology_zone_path(path: &Path) -> String {
    let mut components = path.components();
    let Some(first) = components.next() else {
        return String::from(".");
    };
    if components.next().is_none() {
        return String::from(".");
    }
    first.as_os_str().to_string_lossy().into_owned()
}

fn detect_repository_manifest(path: &Path) -> Option<(PathBuf, &'static str)> {
    let file_name = path.file_name()?.to_string_lossy();
    let ecosystem = match file_name.as_ref() {
        "Cargo.toml" => "rust",
        "package.json" | "pnpm-workspace.yaml" | "yarn.lock" => "node",
        "composer.json" => "php",
        "pyproject.toml" => "python",
        "Gemfile" => "ruby",
        _ => return None,
    };
    Some((path.to_path_buf(), ecosystem))
}

fn build_topology_state_flow_lookup(
    graph_packets: &GraphPacketArtifact,
) -> HashMap<String, Vec<RepositoryTopologyStateFlowPreview>> {
    let mut lookup = HashMap::<String, Vec<RepositoryTopologyStateFlowPreview>>::new();
    for packet in &graph_packets.packets {
        if packet.semantic_state_flows.is_empty() {
            continue;
        }
        for flow in &packet.semantic_state_flows {
            let preview = RepositoryTopologyStateFlowPreview {
                id: flow.id.clone(),
                label: flow.label.clone(),
                carrier_type: flow.carrier_type.clone(),
                slot: flow.slot.clone(),
                kind: flow.kind,
                writer_file: flow.writer.file_path.clone(),
                reader_file: flow.reader.file_path.clone(),
                proof_tier: topology_semantic_state_proof_tier(flow),
                writer_proof: flow.writer.proof,
                reader_proof: flow.reader.proof,
                aggregate_confidence_millis: flow.aggregate_confidence_millis,
                artifact_refs: vec![
                    String::from(GRAPH_PACKETS_FILE),
                    String::from(AGENTIC_REVIEW_FILE),
                ],
            };
            let related_files = std::iter::once(packet.primary_file_path.clone())
                .chain(std::iter::once(flow.writer.file_path.clone()))
                .chain(std::iter::once(flow.reader.file_path.clone()))
                .chain(
                    flow.supporting_locations
                        .iter()
                        .map(|location| location.file_path.clone()),
                )
                .collect::<BTreeSet<_>>();
            for file in related_files {
                lookup.entry(file).or_default().push(preview.clone());
            }
        }
    }
    for entry in lookup.values_mut() {
        entry.sort_by(|left, right| {
            topology_semantic_state_proof_rank(right.proof_tier)
                .cmp(&topology_semantic_state_proof_rank(left.proof_tier))
                .then(
                    right
                        .aggregate_confidence_millis
                        .cmp(&left.aggregate_confidence_millis),
                )
                .then(left.label.cmp(&right.label))
                .then(left.writer_file.cmp(&right.writer_file))
                .then(left.reader_file.cmp(&right.reader_file))
        });
        entry.dedup_by(|left, right| {
            topology_state_flow_key(left) == topology_state_flow_key(right)
        });
    }
    lookup
}

fn topology_state_flow_key(preview: &RepositoryTopologyStateFlowPreview) -> String {
    preview.id.clone()
}

fn topology_semantic_state_proof_tier(
    flow: &crate::agentic::AgenticSemanticStateFlow,
) -> SemanticStateProofKind {
    match (flow.writer.proof, flow.reader.proof) {
        (SemanticStateProofKind::ExactResolved, SemanticStateProofKind::ExactResolved) => {
            SemanticStateProofKind::ExactResolved
        }
        (SemanticStateProofKind::Heuristic, _) | (_, SemanticStateProofKind::Heuristic) => {
            SemanticStateProofKind::Heuristic
        }
        _ => SemanticStateProofKind::ReceiverTyped,
    }
}

fn topology_semantic_state_proof_rank(proof: SemanticStateProofKind) -> u8 {
    match proof {
        SemanticStateProofKind::ExactResolved => 2,
        SemanticStateProofKind::ReceiverTyped => 1,
        SemanticStateProofKind::Heuristic => 0,
    }
}

fn topology_semantic_state_proof_label(proof: SemanticStateProofKind) -> &'static str {
    match proof {
        SemanticStateProofKind::ExactResolved => "exact_resolved",
        SemanticStateProofKind::ReceiverTyped => "receiver_typed",
        SemanticStateProofKind::Heuristic => "heuristic",
    }
}

fn topology_semantic_state_kind_label(kind: AgenticSemanticStateFlowKind) -> &'static str {
    match kind {
        AgenticSemanticStateFlowKind::DirectWriteRead => "direct_write_read",
        AgenticSemanticStateFlowKind::Derived => "derived",
        AgenticSemanticStateFlowKind::ResetBeforeRead => "reset_before_read",
    }
}

fn build_semantic_state_proof_summary(
    flows: &[RepositoryTopologyStateFlowPreview],
) -> RepositoryTopologySemanticStateProofSummary {
    let mut summary = RepositoryTopologySemanticStateProofSummary::default();
    for flow in flows {
        match flow.proof_tier {
            SemanticStateProofKind::ExactResolved => summary.exact_resolved_count += 1,
            SemanticStateProofKind::ReceiverTyped => summary.receiver_typed_count += 1,
            SemanticStateProofKind::Heuristic => summary.heuristic_count += 1,
        }
    }
    summary
}

fn build_topology_state_flow_refs(
    flows: &[RepositoryTopologyStateFlowPreview],
    limit: usize,
) -> Vec<RepositoryTopologyStateFlowRef> {
    flows
        .iter()
        .take(limit)
        .map(|flow| RepositoryTopologyStateFlowRef {
            id: flow.id.clone(),
            label: flow.label.clone(),
            kind: flow.kind,
            proof_tier: flow.proof_tier,
        })
        .collect()
}

fn topology_semantic_state_proof_summary_suffix(
    summary: &RepositoryTopologySemanticStateProofSummary,
) -> String {
    let mut parts = Vec::new();
    if summary.exact_resolved_count > 0 {
        parts.push(format!("{} exact_resolved", summary.exact_resolved_count));
    }
    if summary.receiver_typed_count > 0 {
        parts.push(format!("{} receiver_typed", summary.receiver_typed_count));
    }
    if summary.heuristic_count > 0 {
        parts.push(format!("{} heuristic", summary.heuristic_count));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" Semantic-state evidence: {}.", parts.join(", "))
    }
}

fn topology_observation_label(observed: bool) -> String {
    if observed {
        String::from("observed")
    } else {
        String::from("none_observed")
    }
}

fn build_topology_zone_triage_summary(
    zone_path: &str,
    focus_clusters: &[RepositoryTopologyFocusCluster],
    guardian_packet_previews: &[RepositoryTopologyPacketPreview],
    visible_finding_previews: &[RepositoryTopologyFindingPreview],
    semantic_state_flow_previews: &[RepositoryTopologyStateFlowPreview],
) -> String {
    let semantic_state_flow_proof_summary =
        build_semantic_state_proof_summary(semantic_state_flow_previews);
    let semantic_state_suffix =
        topology_semantic_state_proof_summary_suffix(&semantic_state_flow_proof_summary);
    if let Some(cluster) = focus_clusters.first() {
        return format!(
            "Start triage in `{}` with `{}` on `{}`.{}",
            zone_path, cluster.label, cluster.primary_target_file, semantic_state_suffix
        );
    }
    if let Some(packet) = guardian_packet_previews.first() {
        return format!(
            "Start triage in `{}` with `{}` on `{}`.{}",
            zone_path, packet.focus, packet.primary_target_file, semantic_state_suffix
        );
    }
    if let Some(finding) = visible_finding_previews.first() {
        return format!(
            "Start triage in `{}` with `{}` on `{}`.{}",
            zone_path,
            finding.title,
            finding
                .file_paths
                .first()
                .cloned()
                .unwrap_or_else(|| String::from("unknown")),
            semantic_state_suffix
        );
    }
    if let Some(flow) = semantic_state_flow_previews.first() {
        return format!(
            "Start triage in `{}` by tracing `{}` from `{}` to `{}` (`{}` / `{}` proof).",
            zone_path,
            flow.label,
            flow.writer_file,
            flow.reader_file,
            topology_semantic_state_kind_label(flow.kind),
            topology_semantic_state_proof_label(flow.proof_tier)
        );
    }
    format!(
        "No linked triage targets are currently attached to `{}`.",
        zone_path
    )
}

fn build_topology_zone_triage_steps(
    guardian_packet_previews: &[RepositoryTopologyPacketPreview],
    visible_finding_previews: &[RepositoryTopologyFindingPreview],
    semantic_state_flow_previews: &[RepositoryTopologyStateFlowPreview],
) -> Vec<String> {
    let mut steps = guardian_packet_previews
        .iter()
        .take(3)
        .map(|packet| {
            let mechanism = packet
                .preferred_mechanism
                .as_deref()
                .unwrap_or("the sanctioned mechanism");
            format!(
                "Address `{}` in `{}` first and route the slice through `{}`.",
                packet.focus, packet.primary_target_file, mechanism
            )
        })
        .collect::<Vec<_>>();
    if steps.len() < 3 {
        for finding in visible_finding_previews.iter().take(3) {
            if steps.len() >= 3 {
                break;
            }
            let file_hint = finding
                .file_paths
                .first()
                .cloned()
                .unwrap_or_else(|| String::from("unknown"));
            steps.push(format!(
                "Review `{}` in `{}` because the zone still carries a `{}` finding.",
                finding.title, file_hint, finding.severity
            ));
        }
    }
    if steps.len() < 3 {
        for flow in semantic_state_flow_previews.iter().take(3) {
            if steps.len() >= 3 {
                break;
            }
            steps.push(format!(
                "Trace semantic state `{}` from `{}` to `{}` (`{}` / `{}` proof).",
                flow.label,
                flow.writer_file,
                flow.reader_file,
                topology_semantic_state_kind_label(flow.kind),
                topology_semantic_state_proof_label(flow.proof_tier)
            ));
        }
    }
    steps
}

fn topology_link_direction(
    inbound_relation_count: usize,
    outbound_relation_count: usize,
) -> String {
    match (inbound_relation_count > 0, outbound_relation_count > 0) {
        (true, true) => String::from("mixed"),
        (true, false) => String::from("inbound"),
        (false, true) => String::from("outbound"),
        (false, false) => String::from("none"),
    }
}

fn build_topology_cross_zone_pressure_summary(
    zone_path: &str,
    linked_zones: &[RepositoryTopologyLinkedZone],
) -> String {
    let Some(strongest) = linked_zones.first() else {
        return format!(
            "No cross-zone pressure is currently attached to `{}`.",
            zone_path
        );
    };
    let dominant_relations = strongest
        .relation_kinds
        .iter()
        .take(2)
        .map(|relation| relation.relation.as_str())
        .collect::<Vec<_>>();
    let direction_phrase = match strongest.direction.as_str() {
        "inbound" => "from",
        "outbound" => "toward",
        "mixed" => "between",
        _ => "with",
    };
    if strongest.direction == "mixed" {
        return format!(
            "Strongest cross-zone pressure for `{}` is between `{}` and `{}` across {} relations (dominant: {}).",
            zone_path,
            zone_path,
            strongest.path,
            strongest.relation_count,
            dominant_relations.join(", ")
        );
    }
    format!(
        "Strongest cross-zone pressure for `{}` is {} `{}` across {} relations (dominant: {}).",
        zone_path,
        direction_phrase,
        strongest.path,
        strongest.relation_count,
        dominant_relations.join(", ")
    )
}

fn build_topology_support_bridges(
    linked_zones: &[RepositoryTopologyLinkedZone],
    limit: usize,
) -> Vec<RepositoryTopologySupportBridge> {
    let mut bridges = Vec::new();
    for linked_zone in linked_zones {
        let Some(trace) = linked_zone.support_paths.first() else {
            continue;
        };
        let source_file = trace
            .hops
            .first()
            .map(|hop| hop.source_file_path.clone())
            .unwrap_or_else(|| trace.primary_file_path.clone());
        let target_file = trace
            .hops
            .last()
            .map(|hop| hop.target_file_path.clone())
            .or_else(|| trace.supporting_file_path.clone())
            .unwrap_or_else(|| trace.primary_file_path.clone());
        let source_symbol = trace
            .hops
            .first()
            .and_then(|hop| hop.source_symbol_name.as_deref())
            .unwrap_or(source_file.as_str());
        let target_symbol = trace
            .hops
            .last()
            .and_then(|hop| hop.target_symbol_name.as_deref())
            .unwrap_or(target_file.as_str());
        let relation_phrase = if trace.relation_sequence.is_empty() {
            String::from("graph support")
        } else {
            trace.relation_sequence.join(" -> ")
        };
        bridges.push(RepositoryTopologySupportBridge {
            linked_zone_path: linked_zone.path.clone(),
            direction: linked_zone.direction.clone(),
            summary: format!(
                "Bridge into `{}` via `{}` -> `{}` ({})",
                linked_zone.path, source_symbol, target_symbol, relation_phrase
            ),
            trace_label: trace.label.clone(),
            source_file,
            target_file,
            relation_sequence: trace.relation_sequence.clone(),
            aggregate_confidence_millis: trace.aggregate_confidence_millis,
        });
    }
    bridges.sort_by(|left, right| {
        support_bridge_priority(right)
            .cmp(&support_bridge_priority(left))
            .then(
                right
                    .aggregate_confidence_millis
                    .cmp(&left.aggregate_confidence_millis),
            )
            .then(left.linked_zone_path.cmp(&right.linked_zone_path))
    });
    if bridges.len() > limit {
        bridges.truncate(limit);
    }
    bridges
}

type TopologyTraceCache = HashMap<(String, String, usize, usize), Vec<AgenticGraphTrace>>;

fn graph_trace_between_files_cached(
    graph_context: &GraphQueryContext<'_>,
    trace_cache: &mut TopologyTraceCache,
    source_file: &str,
    target_file: &str,
    max_hops: usize,
    max_paths: usize,
) -> Vec<AgenticGraphTrace> {
    let cache_key = (
        source_file.to_string(),
        target_file.to_string(),
        max_hops,
        max_paths,
    );
    if let Some(cached) = trace_cache.get(&cache_key) {
        return cached.clone();
    }
    let traces = graph_trace_between_files_with_context(
        graph_context,
        source_file,
        target_file,
        max_hops,
        max_paths,
    );
    trace_cache.insert(cache_key, traces.clone());
    traces
}

fn build_topology_targeted_support_bridges(
    graph_context: &GraphQueryContext<'_>,
    trace_cache: &mut TopologyTraceCache,
    target_file: &str,
    linked_zones: &[RepositoryTopologyLinkedZone],
    limit: usize,
) -> Vec<RepositoryTopologySupportBridge> {
    let mut bridges = Vec::new();
    for linked_zone in linked_zones {
        let mut candidate_traces = Vec::new();

        for trace in linked_zone.support_paths.iter().take(2) {
            let mut candidate_files = Vec::new();
            if trace.primary_file_path != target_file {
                candidate_files.push(trace.primary_file_path.as_str());
            }
            if let Some(supporting_file_path) = trace.supporting_file_path.as_deref() {
                if supporting_file_path != target_file {
                    candidate_files.push(supporting_file_path);
                }
            }
            for candidate_file in candidate_files {
                let traces = graph_trace_between_files_cached(
                    graph_context,
                    trace_cache,
                    target_file,
                    candidate_file,
                    4,
                    1,
                );
                if !traces.is_empty() {
                    candidate_traces.extend(traces);
                    break;
                }
            }
            if !candidate_traces.is_empty() {
                break;
            }
        }

        if candidate_traces.is_empty() {
            for example in linked_zone.examples.iter().take(3) {
                let candidate_file = if example.source_file == target_file {
                    Some(example.target_file.as_str())
                } else if example.target_file == target_file {
                    Some(example.source_file.as_str())
                } else {
                    Some(example.target_file.as_str())
                };
                let Some(candidate_file) = candidate_file else {
                    continue;
                };
                let traces = graph_trace_between_files_cached(
                    graph_context,
                    trace_cache,
                    target_file,
                    candidate_file,
                    4,
                    1,
                );
                if !traces.is_empty() {
                    candidate_traces.extend(traces);
                    break;
                }
            }
        }

        if candidate_traces.is_empty() {
            continue;
        }

        let trace = candidate_traces.remove(0);
        let source_file = trace
            .hops
            .first()
            .map(|hop| hop.source_file_path.clone())
            .unwrap_or_else(|| trace.primary_file_path.clone());
        let target_file_path = trace
            .hops
            .last()
            .map(|hop| hop.target_file_path.clone())
            .or_else(|| trace.supporting_file_path.clone())
            .unwrap_or_else(|| trace.primary_file_path.clone());
        let source_symbol = trace
            .hops
            .first()
            .and_then(|hop| hop.source_symbol_name.as_deref())
            .unwrap_or(source_file.as_str());
        let target_symbol = trace
            .hops
            .last()
            .and_then(|hop| hop.target_symbol_name.as_deref())
            .unwrap_or(target_file_path.as_str());
        let relation_phrase = if trace.relation_sequence.is_empty() {
            String::from("graph support")
        } else {
            trace.relation_sequence.join(" -> ")
        };
        bridges.push(RepositoryTopologySupportBridge {
            linked_zone_path: linked_zone.path.clone(),
            direction: linked_zone.direction.clone(),
            summary: format!(
                "Bridge into `{}` via `{}` -> `{}` ({})",
                linked_zone.path, source_symbol, target_symbol, relation_phrase
            ),
            trace_label: trace.label.clone(),
            source_file,
            target_file: target_file_path,
            relation_sequence: trace.relation_sequence.clone(),
            aggregate_confidence_millis: trace.aggregate_confidence_millis,
        });
    }

    bridges.sort_by(|left, right| {
        support_bridge_priority(right)
            .cmp(&support_bridge_priority(left))
            .then(
                right
                    .aggregate_confidence_millis
                    .cmp(&left.aggregate_confidence_millis),
            )
            .then(left.linked_zone_path.cmp(&right.linked_zone_path))
    });
    let mut seen = BTreeSet::new();
    bridges.retain(|bridge| {
        seen.insert((bridge.linked_zone_path.clone(), bridge.trace_label.clone()))
    });
    if bridges.len() > limit {
        bridges.truncate(limit);
    }
    bridges
}

fn support_bridge_priority(bridge: &RepositoryTopologySupportBridge) -> (u8, usize) {
    let strongest_relation = bridge
        .relation_sequence
        .iter()
        .map(|relation| support_bridge_relation_rank(relation))
        .max()
        .unwrap_or(0);
    (strongest_relation, bridge.relation_sequence.len())
}

fn support_bridge_relation_rank(relation: &str) -> u8 {
    match relation {
        "Dispatch" => 10,
        "EventPublish" => 9,
        "Call" => 8,
        "ContainerResolution" => 7,
        "EventSubscribe" => 6,
        "Overrides" => 5,
        "Extends" => 4,
        "Implements" => 3,
        "Import" => 2,
        "TypeUse" => 1,
        _ => 0,
    }
}

fn enrich_topology_zone_triage_summary_with_bridges(
    triage_summary: &str,
    causal_bridges: &[RepositoryTopologySupportBridge],
) -> String {
    let Some(strongest_bridge) = causal_bridges.first() else {
        return triage_summary.to_string();
    };
    format!(
        "{} Strongest causal bridge: {}.",
        triage_summary, strongest_bridge.summary
    )
}

fn topology_spillover_observation(
    guardian_packet_count: usize,
    visible_finding_count: usize,
    linked_zones: &[RepositoryTopologyLinkedZone],
) -> String {
    let linked_guardian_packets = linked_zones
        .iter()
        .map(|zone| zone.linked_guardian_packet_count)
        .sum::<usize>();
    if guardian_packet_count > 0 {
        return String::from("locally_actionable");
    }
    if !linked_zones.is_empty() && linked_guardian_packets > 0 {
        return String::from("spillover_heavy");
    }
    if visible_finding_count > 0 {
        return String::from("locally_visible");
    }
    String::from("quiet")
}

fn build_topology_recommended_start(
    zones: &[RepositoryTopologyZone],
) -> Option<RepositoryTopologyRecommendedSlice> {
    zones.iter()
        .filter_map(|zone| {
            let cluster = zone.focus_clusters.first()?;
            let strongest_relation_count = zone
                .linked_zones
                .first()
                .map(|linked_zone| linked_zone.relation_count)
                .unwrap_or(0);
            let score = usize::from(packet_priority_rank(
                zone.highest_guardian_packet_priority
                    .as_deref()
                    .unwrap_or("low"),
            )) * 1_000
                + usize::from(preview_severity_rank(
                    zone.highest_visible_finding_severity
                        .as_deref()
                        .unwrap_or("low"),
                )) * 100
                + zone.guardian_packet_count * 10
                + zone.visible_finding_count
                + strongest_relation_count;
            let priority = cluster
                .highest_guardian_packet_priority
                .clone()
                .or_else(|| cluster.highest_visible_finding_severity.clone())
                .unwrap_or_else(|| String::from("low"));
            Some((
                score,
                RepositoryTopologyRecommendedSlice {
                    zone_path: zone.path.clone(),
                    target_file: cluster.primary_target_file.clone(),
                    label: cluster.label.clone(),
                    priority,
                    reason: format!(
                        "Start here because zone `{}` is the strongest current triage surface, and its lead cluster `{}` in `{}` carries {} visible finding(s), {} guardian packet(s), and strongest cross-zone pressure of {} relations.",
                        zone.path,
                        cluster.label,
                        cluster.primary_target_file,
                        cluster.visible_finding_ids.len(),
                        cluster.guardian_packet_ids.len(),
                        strongest_relation_count
                    ),
                    supporting_zone_count: zone.linked_zones.len(),
                },
            ))
        })
        .max_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then(left.1.target_file.cmp(&right.1.target_file))
        })
        .map(|(_, slice)| slice)
}

fn build_topology_cross_zone_support_paths(
    graph_context: &GraphQueryContext<'_>,
    trace_cache: &mut TopologyTraceCache,
    current_file: Option<&str>,
    linked_file: Option<&str>,
    direction: &str,
) -> Vec<AgenticGraphTrace> {
    let (Some(current_file), Some(linked_file)) = (current_file, linked_file) else {
        return Vec::new();
    };
    if current_file == linked_file {
        return Vec::new();
    }

    match direction {
        "inbound" => {
            let reverse_paths = graph_trace_between_files_cached(
                graph_context,
                trace_cache,
                linked_file,
                current_file,
                4,
                2,
            );
            if reverse_paths.is_empty() {
                graph_trace_between_files_cached(
                    graph_context,
                    trace_cache,
                    current_file,
                    linked_file,
                    4,
                    2,
                )
            } else {
                reverse_paths
            }
        }
        "mixed" => {
            let forward_paths = graph_trace_between_files_cached(
                graph_context,
                trace_cache,
                current_file,
                linked_file,
                4,
                2,
            );
            if forward_paths.is_empty() {
                graph_trace_between_files_cached(
                    graph_context,
                    trace_cache,
                    linked_file,
                    current_file,
                    4,
                    2,
                )
            } else {
                forward_paths
            }
        }
        _ => {
            let forward_paths = graph_trace_between_files_cached(
                graph_context,
                trace_cache,
                current_file,
                linked_file,
                4,
                2,
            );
            if forward_paths.is_empty() {
                graph_trace_between_files_cached(
                    graph_context,
                    trace_cache,
                    linked_file,
                    current_file,
                    4,
                    2,
                )
            } else {
                forward_paths
            }
        }
    }
}

fn build_topology_zone_triage_step_objects(
    guardian_packet_previews: &[RepositoryTopologyPacketPreview],
    visible_finding_previews: &[RepositoryTopologyFindingPreview],
    finding_status_lookup: &HashMap<String, ConvergenceStatus>,
    file_status_lookup: &HashMap<String, Vec<ConvergenceStatus>>,
    semantic_state_flow_lookup: &HashMap<String, Vec<RepositoryTopologyStateFlowPreview>>,
) -> Vec<RepositoryTopologyTriageStep> {
    let mut steps = guardian_packet_previews
        .iter()
        .take(3)
        .map(|packet| {
            let semantic_state_flows_for_target = semantic_state_flow_lookup
                .get(&packet.primary_target_file)
                .cloned()
                .unwrap_or_default();
            let semantic_state_flow_proof_summary =
                build_semantic_state_proof_summary(&semantic_state_flows_for_target);
            let finding_status_summary = build_status_summary_from_finding_ids_or_file(
                &packet.finding_ids,
                Some(packet.primary_target_file.as_str()),
                finding_status_lookup,
                file_status_lookup,
            );
            RepositoryTopologyTriageStep {
                action: format!(
                    "Start with `{}` in `{}` and follow the sanctioned path `{}`.{}",
                    packet.focus,
                    packet.primary_target_file,
                    packet
                        .preferred_mechanism
                        .as_deref()
                        .unwrap_or("the sanctioned mechanism"),
                    topology_semantic_state_proof_summary_suffix(
                        &semantic_state_flow_proof_summary
                    )
                ),
                priority: packet.priority.clone(),
                target_file: packet.primary_target_file.clone(),
                step_kind: String::from("guardian_packet"),
                freshness: topology_freshness_label(&finding_status_summary),
                finding_status_summary,
                packet_id: Some(packet.id.clone()),
                finding_id: None,
                artifact_refs: packet.artifact_refs.clone(),
                doctrine_refs: packet.doctrine_refs.clone(),
                semantic_state_flow_labels: semantic_state_flows_for_target
                    .iter()
                    .take(3)
                    .map(|flow| flow.label.clone())
                    .collect(),
                semantic_state_flow_refs: build_topology_state_flow_refs(
                    &semantic_state_flows_for_target,
                    3,
                ),
                semantic_state_flow_proof_summary,
                causal_bridges: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    if steps.len() < 3 {
        for finding in visible_finding_previews.iter().take(3) {
            if steps.len() >= 3 {
                break;
            }
            let target_file = finding
                .file_paths
                .first()
                .cloned()
                .unwrap_or_else(|| String::from("unknown"));
            let finding_status_summary = build_status_summary_from_finding_ids_or_file(
                std::slice::from_ref(&finding.id),
                Some(target_file.as_str()),
                finding_status_lookup,
                file_status_lookup,
            );
            let semantic_state_flows_for_target = semantic_state_flow_lookup
                .get(&target_file)
                .cloned()
                .unwrap_or_default();
            let semantic_state_flow_proof_summary =
                build_semantic_state_proof_summary(&semantic_state_flows_for_target);
            steps.push(RepositoryTopologyTriageStep {
                action: format!(
                    "Inspect `{}` in `{}` because this zone still carries a `{}` finding.{}",
                    finding.title,
                    target_file,
                    finding.severity,
                    topology_semantic_state_proof_summary_suffix(
                        &semantic_state_flow_proof_summary
                    )
                ),
                priority: finding.severity.clone(),
                target_file: target_file.clone(),
                step_kind: String::from("visible_finding"),
                freshness: topology_freshness_label(&finding_status_summary),
                finding_status_summary,
                packet_id: None,
                finding_id: Some(finding.id.clone()),
                artifact_refs: finding.artifact_refs.clone(),
                doctrine_refs: Vec::new(),
                semantic_state_flow_labels: semantic_state_flows_for_target
                    .iter()
                    .take(3)
                    .map(|flow| flow.label.clone())
                    .collect(),
                semantic_state_flow_refs: build_topology_state_flow_refs(
                    &semantic_state_flows_for_target,
                    3,
                ),
                semantic_state_flow_proof_summary,
                causal_bridges: Vec::new(),
            });
        }
    }
    steps
}

fn build_topology_focus_clusters(
    guardian_packet_previews: &[RepositoryTopologyPacketPreview],
    visible_finding_previews: &[RepositoryTopologyFindingPreview],
    finding_status_lookup: &HashMap<String, ConvergenceStatus>,
    file_status_lookup: &HashMap<String, Vec<ConvergenceStatus>>,
    semantic_state_flow_lookup: &HashMap<String, Vec<RepositoryTopologyStateFlowPreview>>,
) -> Vec<RepositoryTopologyFocusCluster> {
    #[derive(Default)]
    struct FocusAccumulator {
        label: String,
        visible_finding_ids: BTreeSet<String>,
        guardian_packet_ids: BTreeSet<String>,
        example_files: BTreeSet<String>,
        highest_visible_finding_severity: Option<String>,
        highest_guardian_packet_priority: Option<String>,
        triage_summary: Option<String>,
    }

    let mut clusters = BTreeMap::<String, FocusAccumulator>::new();
    for packet in guardian_packet_previews {
        let key = packet.primary_target_file.clone();
        let cluster = clusters.entry(key.clone()).or_default();
        cluster.label = packet.focus.replace('_', " ");
        cluster.guardian_packet_ids.insert(packet.id.clone());
        cluster
            .example_files
            .insert(packet.primary_target_file.clone());
        cluster.highest_guardian_packet_priority = Some(
            cluster
                .highest_guardian_packet_priority
                .as_deref()
                .map(|existing| {
                    if packet_priority_rank(&packet.priority) > packet_priority_rank(existing) {
                        packet.priority.as_str()
                    } else {
                        existing
                    }
                })
                .unwrap_or(packet.priority.as_str())
                .to_string(),
        );
        cluster.triage_summary = Some(packet.summary.clone());
        for finding_id in &packet.finding_ids {
            cluster.visible_finding_ids.insert(finding_id.clone());
        }
    }
    for finding in visible_finding_previews {
        let key = finding
            .file_paths
            .first()
            .cloned()
            .unwrap_or_else(|| String::from("unknown"));
        let cluster = clusters.entry(key.clone()).or_default();
        if cluster.label.is_empty() {
            cluster.label = finding.title.clone();
        }
        cluster.visible_finding_ids.insert(finding.id.clone());
        for file in &finding.file_paths {
            cluster.example_files.insert(file.clone());
        }
        cluster.highest_visible_finding_severity = Some(
            cluster
                .highest_visible_finding_severity
                .as_deref()
                .map(|existing| {
                    if preview_severity_rank(&finding.severity) > preview_severity_rank(existing) {
                        finding.severity.as_str()
                    } else {
                        existing
                    }
                })
                .unwrap_or(finding.severity.as_str())
                .to_string(),
        );
        if cluster.triage_summary.is_none() {
            cluster.triage_summary = Some(finding.summary.clone());
        }
    }

    let mut result = clusters
        .into_iter()
        .map(|(primary_target_file, cluster)| {
            let semantic_state_flows_for_target = semantic_state_flow_lookup
                .get(&primary_target_file)
                .cloned()
                .unwrap_or_default();
            let semantic_state_flow_proof_summary =
                build_semantic_state_proof_summary(&semantic_state_flows_for_target);
            let visible_finding_ids = cluster
                .visible_finding_ids
                .into_iter()
                .take(8)
                .collect::<Vec<_>>();
            let finding_status_summary = build_status_summary_from_finding_ids_or_file(
                &visible_finding_ids,
                Some(primary_target_file.as_str()),
                finding_status_lookup,
                file_status_lookup,
            );
            RepositoryTopologyFocusCluster {
                id: format!("cluster:{primary_target_file}"),
                label: cluster.label,
                primary_target_file: primary_target_file.clone(),
                freshness: topology_freshness_label(&finding_status_summary),
                finding_status_summary,
                highest_visible_finding_severity: cluster.highest_visible_finding_severity,
                highest_guardian_packet_priority: cluster.highest_guardian_packet_priority,
                visible_finding_ids,
                guardian_packet_ids: cluster.guardian_packet_ids.into_iter().take(8).collect(),
                example_files: cluster.example_files.into_iter().take(5).collect(),
                triage_summary: format!(
                    "{}{}",
                    cluster
                        .triage_summary
                        .unwrap_or_else(|| String::from("No triage summary available.")),
                    topology_semantic_state_proof_summary_suffix(
                        &semantic_state_flow_proof_summary
                    )
                ),
                semantic_state_flow_labels: semantic_state_flows_for_target
                    .iter()
                    .take(3)
                    .map(|flow| flow.label.clone())
                    .collect(),
                semantic_state_flow_refs: build_topology_state_flow_refs(
                    &semantic_state_flows_for_target,
                    3,
                ),
                semantic_state_flow_proof_summary,
                causal_bridges: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| {
        packet_priority_rank(
            right
                .highest_guardian_packet_priority
                .as_deref()
                .unwrap_or("low"),
        )
        .cmp(&packet_priority_rank(
            left.highest_guardian_packet_priority
                .as_deref()
                .unwrap_or("low"),
        ))
        .then(
            preview_severity_rank(
                right
                    .highest_visible_finding_severity
                    .as_deref()
                    .unwrap_or("low"),
            )
            .cmp(&preview_severity_rank(
                left.highest_visible_finding_severity
                    .as_deref()
                    .unwrap_or("low"),
            )),
        )
        .then(left.primary_target_file.cmp(&right.primary_target_file))
    });
    result.truncate(5);
    result
}

fn build_topology_status_lookup(
    deltas: &[ConvergenceFindingDelta],
) -> BTreeMap<String, Vec<ConvergenceStatus>> {
    let mut by_zone = BTreeMap::<String, Vec<ConvergenceStatus>>::new();
    for delta in deltas {
        let affected_zones = delta
            .file_paths
            .iter()
            .map(|file_path| topology_zone_path(Path::new(file_path)))
            .collect::<BTreeSet<_>>();
        for zone in affected_zones {
            by_zone.entry(zone).or_default().push(delta.status);
        }
    }
    by_zone
}

fn build_topology_finding_status_lookup(
    deltas: &[ConvergenceFindingDelta],
) -> HashMap<String, ConvergenceStatus> {
    deltas
        .iter()
        .filter_map(|delta| {
            delta
                .current_id
                .as_ref()
                .map(|current_id| (current_id.clone(), delta.status))
        })
        .collect()
}

fn build_topology_file_status_lookup(
    deltas: &[ConvergenceFindingDelta],
) -> HashMap<String, Vec<ConvergenceStatus>> {
    let mut by_file = HashMap::<String, Vec<ConvergenceStatus>>::new();
    for delta in deltas {
        for file_path in &delta.file_paths {
            by_file
                .entry(file_path.clone())
                .or_default()
                .push(delta.status);
        }
    }
    by_file
}

fn build_status_summary_from_finding_ids_or_file(
    finding_ids: &[String],
    target_file: Option<&str>,
    finding_status_lookup: &HashMap<String, ConvergenceStatus>,
    file_status_lookup: &HashMap<String, Vec<ConvergenceStatus>>,
) -> RepositoryTopologyStatusSummary {
    let mut summary = RepositoryTopologyStatusSummary::default();
    let mut saw_status = false;
    for finding_id in finding_ids {
        if let Some(status) = finding_status_lookup.get(finding_id) {
            increment_status_summary(&mut summary, *status);
            saw_status = true;
        }
    }
    if saw_status {
        return summary;
    }
    if let Some(target_file) = target_file {
        for status in file_status_lookup.get(target_file).into_iter().flatten() {
            increment_status_summary(&mut summary, *status);
        }
    }
    summary
}

fn increment_status_summary(
    summary: &mut RepositoryTopologyStatusSummary,
    status: ConvergenceStatus,
) {
    match status {
        ConvergenceStatus::New => summary.new_count += 1,
        ConvergenceStatus::Worsened => summary.worsened_count += 1,
        ConvergenceStatus::Improved => summary.improved_count += 1,
        ConvergenceStatus::Unchanged => summary.unchanged_count += 1,
        ConvergenceStatus::Resolved => summary.resolved_count += 1,
    }
}

fn topology_freshness_label(summary: &RepositoryTopologyStatusSummary) -> String {
    if summary.worsened_count > 0 {
        return String::from("worsened");
    }
    if summary.new_count > 0 {
        return String::from("new");
    }
    if summary.improved_count > 0 {
        return String::from("improved");
    }
    if summary.resolved_count > 0 {
        return String::from("resolved");
    }
    if summary.unchanged_count > 0 {
        return String::from("unchanged");
    }
    String::from("baseline")
}

fn build_topology_zone_status_summary(
    zone_path: &str,
    status_lookup: &BTreeMap<String, Vec<ConvergenceStatus>>,
) -> RepositoryTopologyStatusSummary {
    let mut summary = RepositoryTopologyStatusSummary::default();
    for status in status_lookup.get(zone_path).into_iter().flatten() {
        increment_status_summary(&mut summary, *status);
    }
    summary
}

fn topology_generated_at_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn topology_baseline_observation(
    convergence_history: Option<&ConvergenceHistoryArtifact>,
) -> String {
    match convergence_history {
        Some(history) if history.summary.previous_findings > 0 => String::from("baseline_loaded"),
        Some(_) => String::from("baseline_empty"),
        None => String::from("baseline_unavailable"),
    }
}

fn build_topology_owner_hints(
    root: &Path,
    example_files: &[String],
    cache: &mut BTreeMap<String, Vec<String>>,
) -> (Vec<String>, String, RepositoryTopologyOwnerHintBasis) {
    let mut contributors = BTreeMap::<String, usize>::new();
    let sampled_files = example_files.iter().take(3).cloned().collect::<Vec<_>>();
    let mut observed_file_count = 0usize;
    for file in &sampled_files {
        let authors = if let Some(cached) = cache.get(file) {
            cached.clone()
        } else {
            let output = Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["log", "--format=%an", "--"])
                .arg(file)
                .output();
            let Ok(output) = output else {
                continue;
            };
            if output.status.success().not() {
                continue;
            }
            let authors = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    trimmed.is_empty().not().then(|| trimmed.to_string())
                })
                .collect::<Vec<_>>();
            cache.insert(file.clone(), authors.clone());
            authors
        };
        let mut saw_observation = false;
        for author in authors {
            *contributors.entry(author).or_default() += 1;
            saw_observation = true;
        }
        if saw_observation {
            observed_file_count += 1;
        }
    }

    let mut ranked = contributors.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    let total_observations = ranked.iter().map(|(_, count)| *count).sum::<usize>();
    let unique_contributor_count = ranked.len();
    let top_share = ranked
        .first()
        .map(|(_, count)| (*count as f64) / (total_observations as f64))
        .unwrap_or(0.0);
    let confidence = if total_observations == 0 {
        "none_observed"
    } else if observed_file_count >= 3 && total_observations >= 6 && top_share >= 0.6 {
        "high"
    } else if observed_file_count >= 2 && total_observations >= 3 && top_share >= 0.35 {
        "medium"
    } else {
        "low"
    };
    (
        ranked.into_iter().take(3).map(|(name, _)| name).collect(),
        String::from(confidence),
        RepositoryTopologyOwnerHintBasis {
            sampled_file_count: sampled_files.len(),
            observed_file_count,
            total_observations,
            unique_contributor_count,
        },
    )
}

fn classify_topology_zone_labels(
    zone_path: &str,
    has_manifest: bool,
    has_runtime_entries: bool,
    has_contracts: bool,
    has_frontend_language: bool,
) -> Vec<String> {
    let normalized = zone_path.replace('\\', "/").to_lowercase();
    let mut labels = BTreeSet::new();
    match normalized.as_str() {
        "." => {
            labels.insert(String::from("root"));
        }
        "app" | "src" | "lib" | "services" | "modules" | "entities" => {
            labels.insert(String::from("source"));
        }
        "tests" | "test" | "spec" => {
            labels.insert(String::from("tests"));
        }
        "docs" | "doc" => {
            labels.insert(String::from("docs"));
        }
        "config" | "configs" | "settings" | "routes" => {
            labels.insert(String::from("config"));
        }
        "resources" | "assets" | "frontend" | "ui" | "website" => {
            labels.insert(String::from("ui"));
        }
        "public" | "static" => {
            labels.insert(String::from("public"));
        }
        "database" | "migrations" | "schema" => {
            labels.insert(String::from("data"));
        }
        "bin" | "scripts" | "tools" => {
            labels.insert(String::from("tooling"));
        }
        _ => {}
    }
    if has_manifest {
        labels.insert(String::from("package-boundary"));
    }
    if has_runtime_entries {
        labels.insert(String::from("runtime-entry"));
    }
    if has_contracts {
        labels.insert(String::from("contract-surface"));
    }
    if has_frontend_language {
        labels.insert(String::from("frontend"));
    }
    if labels.is_empty() {
        labels.insert(String::from("module"));
    }
    labels.into_iter().collect()
}

fn graph_language_label(language: crate::graph::Language) -> &'static str {
    match language {
        crate::graph::Language::JavaScript => "javascript",
        crate::graph::Language::Php => "php",
        crate::graph::Language::Python => "python",
        crate::graph::Language::Ruby => "ruby",
        crate::graph::Language::Rust => "rust",
        crate::graph::Language::TypeScript => "typescript",
    }
}

fn relation_kind_label(kind: crate::graph::RelationKind) -> &'static str {
    match kind {
        crate::graph::RelationKind::Import => "import",
        crate::graph::RelationKind::Call => "call",
        crate::graph::RelationKind::Dispatch => "dispatch",
        crate::graph::RelationKind::ContainerResolution => "container_resolution",
        crate::graph::RelationKind::EventSubscribe => "event_subscribe",
        crate::graph::RelationKind::EventPublish => "event_publish",
        crate::graph::RelationKind::TypeUse => "type_use",
        crate::graph::RelationKind::Extends => "extends",
        crate::graph::RelationKind::Implements => "implements",
        crate::graph::RelationKind::Overrides => "overrides",
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_convergence_history_artifact(
    root: &Path,
    semantic_graph: &crate::graph::SemanticGraph,
    previous_architecture_surface: Option<&ArchitectureSurface>,
    previous_review_surface: Option<&ReviewSurface>,
    previous_contract_inventory: Option<&ContractInventory>,
    current_architecture_surface: &ArchitectureSurface,
    current_review_surface: &ReviewSurface,
    current_contract_inventory: &ContractInventory,
    doctrine_registry: &DoctrineRegistry,
) -> ConvergenceHistoryArtifact {
    let previous_findings = previous_review_surface
        .map(|surface| surface.findings.as_slice())
        .unwrap_or(&[]);
    let current_findings = current_review_surface.findings.as_slice();

    let previous_by_fingerprint = previous_findings
        .iter()
        .map(|finding| (finding.fingerprint.clone(), finding))
        .collect::<HashMap<_, _>>();
    let current_by_fingerprint = current_findings
        .iter()
        .map(|finding| (finding.fingerprint.clone(), finding))
        .collect::<HashMap<_, _>>();

    let fingerprints = previous_by_fingerprint
        .keys()
        .chain(current_by_fingerprint.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut findings = Vec::new();
    let mut summary = ConvergenceSummary {
        current_findings: current_findings.len(),
        previous_findings: previous_findings.len(),
        new_findings: 0,
        worsened_findings: 0,
        improved_findings: 0,
        unchanged_findings: 0,
        resolved_findings: 0,
    };

    for fingerprint in fingerprints.iter() {
        let previous = previous_by_fingerprint.get(fingerprint);
        let current = current_by_fingerprint.get(fingerprint);
        let status = classify_convergence_status(previous.copied(), current.copied());
        match status {
            ConvergenceStatus::New => summary.new_findings += 1,
            ConvergenceStatus::Worsened => summary.worsened_findings += 1,
            ConvergenceStatus::Improved => summary.improved_findings += 1,
            ConvergenceStatus::Unchanged => summary.unchanged_findings += 1,
            ConvergenceStatus::Resolved => summary.resolved_findings += 1,
        }
        let template = current.copied().or_else(|| previous.copied());
        let mut file_paths = template
            .map(|finding| finding.file_paths.clone())
            .unwrap_or_default();
        file_paths.sort();
        file_paths.dedup();
        findings.push(ConvergenceFindingDelta {
            fingerprint: fingerprint.clone(),
            current_id: current.map(|finding| finding.id.clone()),
            previous_id: previous.map(|finding| finding.id.clone()),
            title: template
                .map(|finding| finding.title.clone())
                .unwrap_or_default(),
            family: template
                .map(|finding| review_family_label(finding.family))
                .unwrap_or_else(|| String::from("unknown")),
            status,
            current_severity: current.map(|finding| review_severity_label(finding.severity)),
            previous_severity: previous.map(|finding| review_severity_label(finding.severity)),
            current_visible: current.map(|finding| finding.is_visible),
            previous_visible: previous.map(|finding| finding.is_visible),
            file_paths,
        });
    }

    let attention_items =
        build_convergence_attention_items(current_findings, &findings, doctrine_registry);
    let required_investigation_files = attention_items
        .iter()
        .flat_map(|item| item.file_paths.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let required_radius =
        build_convergence_required_radius(semantic_graph, &required_investigation_files);

    findings.sort_by(|left, right| {
        convergence_status_rank(left.status)
            .cmp(&convergence_status_rank(right.status))
            .then(left.title.cmp(&right.title))
            .then(left.fingerprint.cmp(&right.fingerprint))
    });

    let previous_overview = previous_architecture_surface.map(|surface| &surface.overview);
    let current_overview = &current_architecture_surface.overview;

    ConvergenceHistoryArtifact {
        root: root.display().to_string(),
        summary,
        graph_delta: ConvergenceGraphDelta {
            strong_cycle_delta: delta(
                previous_overview.map(|overview| overview.strong_cycle_count),
                current_overview.strong_cycle_count,
            ),
            total_cycle_delta: delta(
                previous_overview.map(|overview| overview.total_cycle_count),
                current_overview.total_cycle_count,
            ),
            bottleneck_delta: delta(
                previous_overview.map(|overview| overview.bottleneck_count),
                current_overview.bottleneck_count,
            ),
            architectural_smell_delta: delta(
                previous_overview.map(|overview| overview.architectural_smell_count),
                current_overview.architectural_smell_count,
            ),
            warning_heavy_hotspot_delta: delta(
                previous_overview.map(|overview| overview.warning_heavy_hotspot_count),
                current_overview.warning_heavy_hotspot_count,
            ),
            split_identity_model_delta: delta(
                previous_overview.map(|overview| overview.split_identity_model_count),
                current_overview.split_identity_model_count,
            ),
            compatibility_scar_delta: delta(
                previous_overview.map(|overview| overview.compatibility_scar_count),
                current_overview.compatibility_scar_count,
            ),
            duplicate_mechanism_delta: delta(
                previous_overview.map(|overview| overview.duplicate_mechanism_count),
                current_overview.duplicate_mechanism_count,
            ),
            sanctioned_path_bypass_delta: delta(
                previous_overview.map(|overview| overview.sanctioned_path_bypass_count),
                current_overview.sanctioned_path_bypass_count,
            ),
            hand_rolled_parsing_delta: delta(
                previous_overview.map(|overview| overview.hand_rolled_parsing_count),
                current_overview.hand_rolled_parsing_count,
            ),
            abstraction_sprawl_delta: delta(
                previous_overview.map(|overview| overview.abstraction_sprawl_count),
                current_overview.abstraction_sprawl_count,
            ),
            algorithmic_complexity_hotspot_delta: delta(
                previous_overview.map(|overview| overview.algorithmic_complexity_hotspot_count),
                current_overview.algorithmic_complexity_hotspot_count,
            ),
            visible_finding_delta: delta(
                previous_review_surface.map(|surface| surface.summary.visible_findings),
                current_review_surface.summary.visible_findings,
            ),
        },
        contract_delta: build_contract_delta(
            previous_contract_inventory,
            current_contract_inventory,
        ),
        required_investigation_files,
        required_radius,
        attention_items,
        findings,
    }
}

fn build_convergence_required_radius(
    semantic_graph: &crate::graph::SemanticGraph,
    anchor_files: &[String],
) -> ConvergenceRequiredRadius {
    let anchor_set = anchor_files.iter().cloned().collect::<BTreeSet<_>>();
    let mut inbound = BTreeSet::new();
    let mut outbound = BTreeSet::new();

    for edge in &semantic_graph.resolved_edges {
        let source = edge.source_file_path.display().to_string();
        let target = edge.target_file_path.display().to_string();

        if anchor_set.contains(&source) && !anchor_set.contains(&target) {
            outbound.insert(target.clone());
        }
        if anchor_set.contains(&target) && !anchor_set.contains(&source) {
            inbound.insert(source);
        }
    }

    let one_hop_files = inbound
        .iter()
        .chain(outbound.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(50)
        .collect::<Vec<_>>();

    ConvergenceRequiredRadius {
        anchor_files: anchor_files.to_vec(),
        one_hop_files,
        inbound_neighbor_count: inbound.len(),
        outbound_neighbor_count: outbound.len(),
    }
}

pub fn build_guard_decision_artifact(
    root: &Path,
    convergence: &ConvergenceHistoryArtifact,
) -> GuardDecisionArtifact {
    let contract_delta_count = [
        &convergence.contract_delta.routes,
        &convergence.contract_delta.hooks,
        &convergence.contract_delta.registered_keys,
        &convergence.contract_delta.symbolic_literals,
        &convergence.contract_delta.env_keys,
        &convergence.contract_delta.config_keys,
    ]
    .into_iter()
    .map(|delta| delta.added_count + delta.removed_count)
    .sum::<usize>();

    let high_severity_security_regressions = convergence
        .findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.status,
                ConvergenceStatus::New | ConvergenceStatus::Worsened
            ) && finding.current_visible == Some(true)
                && finding.family == "security"
                && finding.current_severity.as_deref() == Some("high")
        })
        .count();
    let cycle_regression = convergence.graph_delta.strong_cycle_delta > 0;
    let bottleneck_regression = convergence.graph_delta.bottleneck_delta > 0;
    let architectural_smell_regression = convergence.graph_delta.architectural_smell_delta > 0;
    let warning_heavy_hotspot_regression = convergence.graph_delta.warning_heavy_hotspot_delta > 0;
    let split_identity_model_regression = convergence.graph_delta.split_identity_model_delta > 0;
    let compatibility_scar_regression = convergence.graph_delta.compatibility_scar_delta > 0;
    let duplicate_mechanism_regression = convergence.graph_delta.duplicate_mechanism_delta > 0;
    let sanctioned_path_bypass_regression =
        convergence.graph_delta.sanctioned_path_bypass_delta > 0;
    let hand_rolled_parsing_regression = convergence.graph_delta.hand_rolled_parsing_delta > 0;
    let abstraction_sprawl_regression = convergence.graph_delta.abstraction_sprawl_delta > 0;
    let algorithmic_complexity_hotspot_regression =
        convergence.graph_delta.algorithmic_complexity_hotspot_delta > 0;
    let exact_or_modeled_attention_items = convergence
        .attention_items
        .iter()
        .filter(|item| item.precision != "heuristic")
        .count();
    let heuristic_attention_items = convergence
        .attention_items
        .len()
        .saturating_sub(exact_or_modeled_attention_items);

    let pressure = GuardDecisionPressure {
        new_findings: convergence.summary.new_findings,
        worsened_findings: convergence.summary.worsened_findings,
        attention_items: convergence.attention_items.len(),
        exact_or_modeled_attention_items,
        heuristic_attention_items,
        required_radius_anchor_files: convergence.required_radius.anchor_files.len(),
        required_radius_one_hop_files: convergence.required_radius.one_hop_files.len(),
        visible_finding_delta: convergence.graph_delta.visible_finding_delta,
        contract_delta_count,
        high_severity_security_regressions,
        cycle_regression,
        bottleneck_regression,
        architectural_smell_regression,
        warning_heavy_hotspot_regression,
        split_identity_model_regression,
        compatibility_scar_regression,
        duplicate_mechanism_regression,
        sanctioned_path_bypass_regression,
        hand_rolled_parsing_regression,
        abstraction_sprawl_regression,
        algorithmic_complexity_hotspot_regression,
    };

    let mut reasons = Vec::new();
    let mut triggers = Vec::new();
    let mut doctrine_refs = BTreeSet::from([
        String::from("guardian.change-governance"),
        String::from("guardian.diff-local-judgment"),
    ]);
    let mut obligations = Vec::new();

    if high_severity_security_regressions > 0 {
        let message = format!(
            "{high_severity_security_regressions} new or worsened high-severity security finding(s) are visible."
        );
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.trust-boundaries"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Block,
            message,
            precision: String::from("modeled"),
            confidence_millis: 930,
            provenance: vec![
                String::from("convergence_history"),
                String::from("review_surface"),
            ],
            doctrine_refs: vec![String::from("guardian.trust-boundaries")],
        });
        obligations.push(GuardianObligation {
            action: String::from(
                "Resolve or explicitly justify the new high-severity security regression before accepting the change.",
            ),
            acceptance: String::from(
                "No visible new/worsened high-severity security finding remains in the reviewed slice.",
            ),
        });
    }
    if cycle_regression {
        let message =
            String::from("Strong dependency-cycle pressure increased in the current run.");
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.structural-coherence"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Block,
            message,
            precision: String::from("modeled"),
            confidence_millis: 910,
            provenance: vec![
                String::from("convergence_history"),
                String::from("graph_analysis"),
            ],
            doctrine_refs: vec![String::from("guardian.structural-coherence")],
        });
        obligations.push(GuardianObligation {
            action: String::from(
                "Break or explicitly justify the newly introduced strong cycle before accepting the change.",
            ),
            acceptance: String::from(
                "Strong cycle count no longer regresses relative to the previous run.",
            ),
        });
    }
    if convergence.attention_items.is_empty().not() {
        reasons.push(format!(
            "{} doctrine-backed attention item(s) need review.",
            convergence.attention_items.len()
        ));
        triggers.extend(
            convergence
                .attention_items
                .iter()
                .map(|item| GuardDecisionTrigger {
                    level: if item.precision == "heuristic" {
                        GuardTriggerLevel::Warn
                    } else {
                        GuardTriggerLevel::Block
                    },
                    message: format!("{} [{}]", item.title, item.status_label()),
                    precision: item.precision.clone(),
                    confidence_millis: item.confidence_millis,
                    provenance: item.provenance.clone(),
                    doctrine_refs: item.doctrine_refs.clone(),
                }),
        );
        obligations.extend(
            convergence
                .attention_items
                .iter()
                .flat_map(|item| item.obligations.iter().cloned()),
        );
        doctrine_refs.extend(
            convergence
                .attention_items
                .iter()
                .flat_map(|item| item.doctrine_refs.iter().cloned()),
        );
    }
    if contract_delta_count > 0 {
        let message = format!(
            "{contract_delta_count} contract inventory change(s) were detected in routes/hooks/keys/config surfaces."
        );
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.contract-coherence"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("modeled"),
            confidence_millis: 860,
            provenance: vec![
                String::from("contract_inventory"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.contract-coherence")],
        });
        obligations.push(GuardianObligation {
            action: String::from(
                "Review changed public/runtime contracts and confirm the owning mechanism and callers were updated consistently.",
            ),
            acceptance: String::from(
                "Contract deltas are explained and the affected radius is reviewed or updated.",
            ),
        });
    }
    if architectural_smell_regression {
        let message =
            String::from("Architectural smell count increased relative to the previous run.");
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.architectonic-quality"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("modeled"),
            confidence_millis: 820,
            provenance: vec![
                String::from("graph_analysis"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.architectonic-quality")],
        });
    }
    if bottleneck_regression {
        let message =
            String::from("Bottleneck concentration increased relative to the previous run.");
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.architectonic-quality"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("modeled"),
            confidence_millis: 810,
            provenance: vec![
                String::from("graph_analysis"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.architectonic-quality")],
        });
    }
    if warning_heavy_hotspot_regression {
        let message =
            String::from("Warning-heavy hotspot count increased relative to the previous run.");
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.architectonic-quality"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("modeled"),
            confidence_millis: 790,
            provenance: vec![
                String::from("architectural_assessment"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.architectonic-quality")],
        });
    }
    if split_identity_model_regression {
        let message =
            String::from("Split identity model pressure increased relative to the previous run.");
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.domain-coherence"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("heuristic"),
            confidence_millis: 760,
            provenance: vec![
                String::from("architectural_assessment"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.domain-coherence")],
        });
    }
    if compatibility_scar_regression {
        let message =
            String::from("Compatibility-scar pressure increased relative to the previous run.");
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.domain-coherence"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("heuristic"),
            confidence_millis: 780,
            provenance: vec![
                String::from("architectural_assessment"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.domain-coherence")],
        });
    }
    if duplicate_mechanism_regression {
        let message =
            String::from("Duplicate-mechanism pressure increased relative to the previous run.");
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.mechanism-coherence"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("heuristic"),
            confidence_millis: 800,
            provenance: vec![
                String::from("architectural_assessment"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.mechanism-coherence")],
        });
    }
    if sanctioned_path_bypass_regression {
        let message =
            String::from("Sanctioned-path bypass pressure increased relative to the previous run.");
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.sanctioned-paths"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("heuristic"),
            confidence_millis: 820,
            provenance: vec![
                String::from("architectural_assessment"),
                String::from("hardwiring_detector"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.sanctioned-paths")],
        });
    }
    if abstraction_sprawl_regression {
        let message =
            String::from("Abstraction-sprawl pressure increased relative to the previous run.");
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.overengineering"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("heuristic"),
            confidence_millis: 790,
            provenance: vec![
                String::from("architectural_assessment"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.overengineering")],
        });
    }
    if algorithmic_complexity_hotspot_regression {
        let message = String::from(
            "Algorithmic-complexity hotspot pressure increased relative to the previous run.",
        );
        reasons.push(message.clone());
        doctrine_refs.extend([
            String::from("performance.scaling"),
            String::from("guardian.superlinear-risk"),
        ]);
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("heuristic"),
            confidence_millis: 820,
            provenance: vec![
                String::from("architectural_assessment"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![
                String::from("performance.scaling"),
                String::from("guardian.superlinear-risk"),
            ],
        });
        obligations.push(GuardianObligation {
            action: String::from(
                "Review the new algorithmic-complexity hotspot and justify, batch, cache, hoist, or remove the repeated expensive path.",
            ),
            acceptance: String::from(
                "Algorithmic-complexity hotspot count no longer regresses relative to the previous run, or the new hotspot is explicitly justified.",
            ),
        });
    }
    if hand_rolled_parsing_regression {
        let message = String::from(
            "Homegrown parsing or validation pressure increased relative to the previous run.",
        );
        reasons.push(message.clone());
        doctrine_refs.insert(String::from("guardian.native-vs-library"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("heuristic"),
            confidence_millis: 800,
            provenance: vec![
                String::from("architectural_assessment"),
                String::from("parsed_sources"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![
                String::from("guardian.native-vs-library"),
                String::from("guardian.avoid-homegrown-parser"),
                String::from("guardian.avoid-homegrown-definition-engine"),
                String::from("guardian.avoid-homegrown-scheduler-dsl"),
                String::from("guardian.avoid-homegrown-schema-validation"),
            ],
        });
    }
    if convergence.graph_delta.visible_finding_delta > 0 {
        let message = format!(
            "Visible finding pressure increased by {}.",
            convergence.graph_delta.visible_finding_delta
        );
        reasons.push(message.clone());
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message,
            precision: String::from("modeled"),
            confidence_millis: 780,
            provenance: vec![
                String::from("review_surface"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.change-governance")],
        });
    }

    obligations.sort_by(|left, right| {
        left.action
            .cmp(&right.action)
            .then(left.acceptance.cmp(&right.acceptance))
    });
    obligations.dedup();
    triggers.sort_by(|left, right| {
        guard_trigger_level_rank(left.level)
            .cmp(&guard_trigger_level_rank(right.level))
            .then(right.confidence_millis.cmp(&left.confidence_millis))
            .then(left.message.cmp(&right.message))
    });
    triggers.dedup_by(|left, right| {
        left.level == right.level
            && left.message == right.message
            && left.precision == right.precision
            && left.provenance == right.provenance
    });

    if convergence.required_radius.anchor_files.is_empty().not()
        && (convergence.attention_items.is_empty().not() || contract_delta_count > 0)
    {
        let message = format!(
            "Required review radius is anchored on {} file(s) with {} one-hop neighboring file(s).",
            convergence.required_radius.anchor_files.len(),
            convergence.required_radius.one_hop_files.len()
        );
        doctrine_refs.insert(String::from("guardian.change-radius"));
        triggers.push(GuardDecisionTrigger {
            level: GuardTriggerLevel::Warn,
            message: message.clone(),
            precision: String::from("modeled"),
            confidence_millis: 800,
            provenance: vec![
                String::from("graph_analysis"),
                String::from("convergence_history"),
            ],
            doctrine_refs: vec![String::from("guardian.change-radius")],
        });
        obligations.push(GuardianObligation {
            action: format!(
                "Review the required radius anchored on {} and confirm adjacent callers/dependents were updated consistently.",
                convergence.required_radius.anchor_files.join(", ")
            ),
            acceptance: format!(
                "The guard radius ({}) is reviewed, or updated code covers the {} one-hop neighboring file(s).",
                convergence.required_radius.anchor_files.join(", "),
                convergence.required_radius.one_hop_files.len()
            ),
        });
    }

    obligations.sort_by(|left, right| {
        left.action
            .cmp(&right.action)
            .then(left.acceptance.cmp(&right.acceptance))
    });
    obligations.dedup();

    let block_trigger_count = triggers
        .iter()
        .filter(|trigger| trigger.level == GuardTriggerLevel::Block)
        .count();
    let warn_trigger_count = triggers
        .iter()
        .filter(|trigger| trigger.level == GuardTriggerLevel::Warn)
        .count();
    let max_trigger_confidence = triggers
        .iter()
        .map(|trigger| trigger.confidence_millis)
        .max()
        .unwrap_or(0);

    let (verdict, confidence_millis, summary) = if block_trigger_count > 0 {
        (
            GuardVerdict::Block,
            max_trigger_confidence.max(930),
            String::from(
                "Block: the current diff state introduces or worsens high-risk architectural/security pressure.",
            ),
        )
    } else if warn_trigger_count > 0 {
        (
            GuardVerdict::Warn,
            max_trigger_confidence.max(
                if exact_or_modeled_attention_items > 0 || contract_delta_count > 0 {
                    840
                } else {
                    760
                },
            ),
            if exact_or_modeled_attention_items > 0 || contract_delta_count > 0 {
                String::from(
                    "Warn: the current diff state includes modeled/exact guard pressure that needs focused review.",
                )
            } else {
                String::from(
                    "Warn: the current diff state includes heuristic guard pressure that should be reviewed before it is treated as safe.",
                )
            },
        )
    } else {
        (
            GuardVerdict::Allow,
            980,
            String::from(
                "Allow: no new or worsened diff-local architectural/security pressure was detected.",
            ),
        )
    };

    if obligations.is_empty() && verdict == GuardVerdict::Allow {
        obligations.push(GuardianObligation {
            action: String::from("Proceed with normal review flow."),
            acceptance: String::from(
                "No diff-local architectural or security regression requires extra guard action.",
            ),
        });
    }

    GuardDecisionArtifact {
        root: root.display().to_string(),
        verdict,
        confidence_millis,
        summary,
        reasons,
        triggers,
        doctrine_refs: doctrine_refs.into_iter().collect(),
        obligations,
        required_radius: convergence.required_radius.clone(),
        attention_items: convergence.attention_items.clone(),
        pressure,
    }
}

fn classify_convergence_status(
    previous: Option<&crate::review::ReviewFinding>,
    current: Option<&crate::review::ReviewFinding>,
) -> ConvergenceStatus {
    match (previous, current) {
        (None, Some(_)) => ConvergenceStatus::New,
        (Some(_), None) => ConvergenceStatus::Resolved,
        (Some(previous), Some(current)) => {
            let previous_severity = severity_rank(previous.severity);
            let current_severity = severity_rank(current.severity);
            if current.is_visible && !previous.is_visible {
                ConvergenceStatus::Worsened
            } else if !current.is_visible && previous.is_visible {
                ConvergenceStatus::Improved
            } else if current_severity > previous_severity
                || current.confidence_millis > previous.confidence_millis + 75
            {
                ConvergenceStatus::Worsened
            } else if current_severity < previous_severity
                || previous.confidence_millis > current.confidence_millis + 75
                || current.policy_status != previous.policy_status
            {
                ConvergenceStatus::Improved
            } else {
                ConvergenceStatus::Unchanged
            }
        }
        (None, None) => ConvergenceStatus::Unchanged,
    }
}

fn read_json_artifact_if_exists<T>(path: &Path) -> io::Result<Option<T>>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Ok(None);
    }
    let payload = fs::read(path)?;
    serde_json::from_slice(&payload)
        .map(Some)
        .map_err(io::Error::other)
}

fn delta(previous: Option<usize>, current: usize) -> isize {
    current as isize - previous.unwrap_or_default() as isize
}

fn convergence_status_rank(status: ConvergenceStatus) -> u8 {
    match status {
        ConvergenceStatus::Worsened => 0,
        ConvergenceStatus::New => 1,
        ConvergenceStatus::Improved => 2,
        ConvergenceStatus::Resolved => 3,
        ConvergenceStatus::Unchanged => 4,
    }
}

fn build_contract_delta(
    previous: Option<&ContractInventory>,
    current: &ContractInventory,
) -> ConvergenceContractDelta {
    ConvergenceContractDelta {
        routes: contract_value_delta(
            previous
                .map(|inventory| inventory.routes.as_slice())
                .unwrap_or(&[]),
            &current.routes,
        ),
        hooks: contract_value_delta(
            previous
                .map(|inventory| inventory.hooks.as_slice())
                .unwrap_or(&[]),
            &current.hooks,
        ),
        registered_keys: contract_value_delta(
            previous
                .map(|inventory| inventory.registered_keys.as_slice())
                .unwrap_or(&[]),
            &current.registered_keys,
        ),
        symbolic_literals: contract_value_delta(
            previous
                .map(|inventory| inventory.symbolic_literals.as_slice())
                .unwrap_or(&[]),
            &current.symbolic_literals,
        ),
        env_keys: contract_value_delta(
            previous
                .map(|inventory| inventory.env_keys.as_slice())
                .unwrap_or(&[]),
            &current.env_keys,
        ),
        config_keys: contract_value_delta(
            previous
                .map(|inventory| inventory.config_keys.as_slice())
                .unwrap_or(&[]),
            &current.config_keys,
        ),
    }
}

fn contract_value_delta(
    previous: &[crate::contracts::ContractInventoryItem],
    current: &[crate::contracts::ContractInventoryItem],
) -> ContractValueDelta {
    let previous_values = previous
        .iter()
        .map(|item| item.value.clone())
        .collect::<BTreeSet<_>>();
    let current_values = current
        .iter()
        .map(|item| item.value.clone())
        .collect::<BTreeSet<_>>();
    let added = current_values
        .difference(&previous_values)
        .take(10)
        .cloned()
        .collect::<Vec<_>>();
    let removed = previous_values
        .difference(&current_values)
        .take(10)
        .cloned()
        .collect::<Vec<_>>();

    ContractValueDelta {
        added_count: current_values.difference(&previous_values).count(),
        removed_count: previous_values.difference(&current_values).count(),
        added,
        removed,
    }
}

fn build_convergence_attention_items(
    current_findings: &[crate::review::ReviewFinding],
    deltas: &[ConvergenceFindingDelta],
    doctrine_registry: &DoctrineRegistry,
) -> Vec<ConvergenceAttentionItem> {
    let current_by_fingerprint = current_findings
        .iter()
        .map(|finding| (finding.fingerprint.clone(), finding))
        .collect::<HashMap<_, _>>();

    let mut items = deltas
        .iter()
        .filter(|delta| {
            matches!(
                delta.status,
                ConvergenceStatus::New | ConvergenceStatus::Worsened
            )
        })
        .filter_map(|delta| {
            let finding = current_by_fingerprint.get(&delta.fingerprint)?;
            let focus = convergence_focus(finding);
            let preferred_mechanism = guardian_packet_preferred_mechanism(
                focus,
                &finding.doctrine_refs,
                &[],
                doctrine_registry,
            );
            Some(ConvergenceAttentionItem {
                fingerprint: delta.fingerprint.clone(),
                status: delta.status,
                title: finding.title.clone(),
                family: review_family_label(finding.family),
                precision: finding.precision.clone(),
                confidence_millis: finding.confidence_millis,
                summary: finding.summary.clone(),
                file_paths: finding.file_paths.clone(),
                provenance: finding.provenance.clone(),
                doctrine_refs: finding.doctrine_refs.clone(),
                obligations: guardian_packet_obligations(
                    focus,
                    &finding
                        .file_paths
                        .first()
                        .cloned()
                        .unwrap_or_else(|| String::from("unknown")),
                    preferred_mechanism.as_deref(),
                    &[],
                ),
            })
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        convergence_status_rank(left.status)
            .cmp(&convergence_status_rank(right.status))
            .then(left.title.cmp(&right.title))
            .then(left.fingerprint.cmp(&right.fingerprint))
    });
    items.truncate(10);
    items
}

fn convergence_focus(finding: &crate::review::ReviewFinding) -> &'static str {
    let title = finding.title.to_ascii_lowercase();
    if finding.family == crate::review::ReviewFindingFamily::Security {
        "security_hotspot"
    } else if title.contains("algorithmic complexity") {
        // Without this branch a complexity finding falls into the generic
        // hotspot template while its doctrine clause supplies a complexity
        // mechanism — producing obligations like "collapse competing concerns
        // into `precomputed_index_or_single_pass_flow`".
        "algorithmic_complexity_hotspot"
    } else if title.contains("hand-rolled parsing") {
        "hand_rolled_parsing"
    } else if title.contains("abstraction sprawl") {
        "abstraction_sprawl"
    } else if title.contains("compatibility scar") {
        "compatibility_scar"
    } else if title.contains("split identity model") {
        "split_identity_model"
    } else if title.contains("duplicate mechanism") {
        "duplicate_mechanism"
    } else if title.contains("sanctioned path bypass") {
        "sanctioned_path_bypass"
    } else {
        "warning_heavy_hotspot"
    }
}

impl ConvergenceAttentionItem {
    fn status_label(&self) -> &'static str {
        match self.status {
            ConvergenceStatus::New => "new",
            ConvergenceStatus::Worsened => "worsened",
            ConvergenceStatus::Improved => "improved",
            ConvergenceStatus::Unchanged => "unchanged",
            ConvergenceStatus::Resolved => "resolved",
        }
    }
}

fn guard_trigger_level_rank(level: GuardTriggerLevel) -> u8 {
    match level {
        GuardTriggerLevel::Block => 0,
        GuardTriggerLevel::Warn => 1,
    }
}

fn severity_rank(severity: ReviewFindingSeverity) -> u8 {
    match severity {
        ReviewFindingSeverity::High => 2,
        ReviewFindingSeverity::Medium => 1,
        ReviewFindingSeverity::Low => 0,
    }
}

fn review_family_label(family: ReviewFindingFamily) -> String {
    match family {
        ReviewFindingFamily::Graph => String::from("graph"),
        ReviewFindingFamily::DeadCode => String::from("dead_code"),
        ReviewFindingFamily::Hardwiring => String::from("hardwiring"),
        ReviewFindingFamily::Security => String::from("security"),
        ReviewFindingFamily::External => String::from("external"),
    }
}

fn review_severity_label(severity: ReviewFindingSeverity) -> String {
    match severity {
        ReviewFindingSeverity::High => String::from("high"),
        ReviewFindingSeverity::Medium => String::from("medium"),
        ReviewFindingSeverity::Low => String::from("low"),
    }
}

fn preview_severity_rank(severity: &str) -> u8 {
    match severity {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

pub fn build_agent_handoff_artifact(
    analysis: &ProjectAnalysis,
    review_surface: &ReviewSurface,
    doctrine_registry: &DoctrineRegistry,
) -> AgentHandoffArtifact {
    let feedback_loop = build_feedback_loop_summary(review_surface);
    let visible_findings = review_surface
        .findings
        .iter()
        .filter(|finding| finding.is_visible)
        .collect::<Vec<_>>();
    let high_visible = visible_findings
        .iter()
        .filter(|finding| finding.severity == crate::review::ReviewFindingSeverity::High)
        .count();
    let mut next_steps = Vec::new();

    if analysis
        .graph_analysis
        .strong_circular_dependencies
        .is_empty()
        .not()
    {
        next_steps.push(format!(
            "Break {} strong cycle groups before adding more features.",
            analysis.graph_analysis.strong_circular_dependencies.len()
        ));
    }
    if analysis.graph_analysis.bottleneck_files.is_empty().not() {
        next_steps.push(format!(
            "Refactor the top {} bottleneck files to reduce architectural pressure.",
            analysis.graph_analysis.bottleneck_files.len().min(5)
        ));
    }
    if analysis
        .graph_analysis
        .architectural_smells
        .is_empty()
        .not()
    {
        next_steps.push(format!(
            "Address {} explicit architectural smell findings before they harden into platform debt.",
            analysis.graph_analysis.architectural_smells.len()
        ));
    }
    let warning_hotspot_count = analysis
        .architectural_assessment
        .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::WarningHeavyHotspot);
    let split_identity_count = analysis
        .architectural_assessment
        .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::SplitIdentityModel);
    let compatibility_scar_count = analysis
        .architectural_assessment
        .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::CompatibilityScar);
    let duplicate_mechanism_count = analysis
        .architectural_assessment
        .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::DuplicateMechanism);
    let sanctioned_path_bypass_count = analysis
        .architectural_assessment
        .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::SanctionedPathBypass);
    let hand_rolled_parsing_count = analysis
        .architectural_assessment
        .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::HandRolledParsing);
    let abstraction_sprawl_count = analysis
        .architectural_assessment
        .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::AbstractionSprawl);
    let algorithmic_complexity_hotspot_count = analysis.architectural_assessment.count_by_kind(
        crate::assessment::ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot,
    );
    let guardian_packets = build_guardian_packets(
        analysis,
        review_surface,
        &visible_findings,
        doctrine_registry,
    );
    if warning_hotspot_count > 0 {
        next_steps.push(format!(
            "Reduce {} warning-heavy hotspot files where architectural centrality and detector/security noise are accumulating together.",
            warning_hotspot_count
        ));
    }
    if split_identity_count > 0 {
        next_steps.push(format!(
            "Converge {} split identity model hotspots where the same concept is represented through both object-like and scalar identifier forms.",
            split_identity_count
        ));
    }
    if compatibility_scar_count > 0 {
        next_steps.push(format!(
            "Refactor {} compatibility-scar hotspots where one file is centralizing translation glue for competing domain representations.",
            compatibility_scar_count
        ));
    }
    if duplicate_mechanism_count > 0 {
        next_steps.push(format!(
            "Collapse {} duplicate-mechanism hotspots where the same concern is routed through competing orchestration paths.",
            duplicate_mechanism_count
        ));
    }
    if sanctioned_path_bypass_count > 0 {
        next_steps.push(format!(
            "Refactor {} sanctioned-path bypass hotspots where raw primitives bypass approved configuration or framework pathways.",
            sanctioned_path_bypass_count
        ));
    }
    if abstraction_sprawl_count > 0 {
        next_steps.push(format!(
            "Collapse {} abstraction-sprawl hotspots where one concern is split across too many helper/service/registry/factory-style layers.",
            abstraction_sprawl_count
        ));
    }
    if algorithmic_complexity_hotspot_count > 0 {
        next_steps.push(format!(
            "Reduce {} algorithmic-complexity hotspots where nested iteration, repeated linear scans, sorting, or regex compilation inside loops may create superlinear runtime growth.",
            algorithmic_complexity_hotspot_count
        ));
    }
    if hand_rolled_parsing_count > 0 {
        next_steps.push(format!(
            "Review {} hand-rolled parsing, schema-validation, scheduler-DSL, definition-engine, or contract-stack hotspots and replace custom mini-language, validator/resolver, scheduler/orchestration, schema-walker, or metadata-engine logic with battle-tested native/framework/library mechanisms where possible.",
            hand_rolled_parsing_count
        ));
    }
    if analysis.dead_code.findings.is_empty().not() {
        next_steps.push(format!(
            "Remove or suppress {} dead-code findings after sampling truth.",
            analysis.dead_code.findings.len()
        ));
    }
    if analysis.hardwiring.findings.is_empty().not() {
        next_steps.push(format!(
            "Triage {} hardwiring findings and convert repeated accepted patterns into policy.",
            analysis.hardwiring.findings.len()
        ));
    }
    if analysis.security_analysis.findings.is_empty().not() {
        next_steps.push(format!(
            "Review {} native dangerous-API security findings and prioritize externally reachable sinks first.",
            analysis.security_analysis.findings.len()
        ));
    }
    if analysis.external_analysis.findings.is_empty().not() {
        next_steps.push(format!(
            "Review {} external security findings and feed accepted patterns back into rules.",
            analysis.external_analysis.findings.len()
        ));
    }
    if next_steps.is_empty() {
        next_steps.push(String::from(
            "No major actionable findings remain; keep the current architecture baseline stable.",
        ));
    }

    AgentHandoffArtifact {
        root: analysis.root.display().to_string(),
        summary: AgentHandoffSummary {
            scanned_files: analysis.scan.files.len(),
            analyzed_files: analysis.semantic_graph.files.len(),
            strong_cycle_count: analysis.graph_analysis.strong_circular_dependencies.len(),
            bottleneck_count: analysis.graph_analysis.bottleneck_files.len(),
            architectural_smell_count: analysis.graph_analysis.architectural_smells.len(),
            warning_heavy_hotspot_count: warning_hotspot_count,
            split_identity_model_count: split_identity_count,
            compatibility_scar_count,
            duplicate_mechanism_count,
            sanctioned_path_bypass_count,
            hand_rolled_parsing_count,
            abstraction_sprawl_count,
            algorithmic_complexity_hotspot_count,
            visible_findings: review_surface.summary.visible_findings,
            dead_code_count: analysis.dead_code.findings.len(),
            hardwiring_count: analysis.hardwiring.findings.len(),
            security_finding_count: analysis.security_analysis.findings.len(),
            external_finding_count: analysis.external_analysis.findings.len(),
        },
        feedback_loop,
        next_steps,
        guardian_packets,
        top_findings: {
            let per_phase_cap = if high_visible > 0 { 5 } else { 3 };
            let mut architecture_taken = 0usize;
            let mut implementation_taken = 0usize;
            visible_findings
                .into_iter()
                .filter(|finding| {
                    let taken = match finding.phase {
                        crate::surface::SurfaceFindingPhase::Architecture => {
                            &mut architecture_taken
                        }
                        crate::surface::SurfaceFindingPhase::Implementation => {
                            &mut implementation_taken
                        }
                    };
                    *taken += 1;
                    *taken <= per_phase_cap
                })
                .map(|finding| AgentHandoffFinding {
                    id: finding.id.clone(),
                    family: format!("{:?}", finding.family),
                    phase: review_phase_label(finding.phase).to_string(),
                    severity: format!("{:?}", finding.severity),
                    title: finding.title.clone(),
                    summary: finding.summary.clone(),
                    file_paths: finding.file_paths.clone(),
                    line: finding.line,
                    primary_anchor: finding.primary_anchor.clone(),
                    locations: finding.locations.clone(),
                })
                .collect()
        },
    }
}

fn review_phase_label(phase: crate::surface::SurfaceFindingPhase) -> &'static str {
    match phase {
        crate::surface::SurfaceFindingPhase::Architecture => "architecture",
        crate::surface::SurfaceFindingPhase::Implementation => "implementation",
    }
}

fn build_guardian_packets(
    analysis: &ProjectAnalysis,
    _review_surface: &ReviewSurface,
    visible_findings: &[&crate::review::ReviewFinding],
    doctrine_registry: &DoctrineRegistry,
) -> Vec<GuardianPacket> {
    let mut packets = Vec::new();
    let mut visible_by_file = BTreeMap::<String, Vec<&crate::review::ReviewFinding>>::new();
    for finding in visible_findings {
        for path in &finding.file_paths {
            visible_by_file
                .entry(path.clone())
                .or_default()
                .push(*finding);
        }
    }

    let bottleneck_by_file = analysis
        .graph_analysis
        .bottleneck_files
        .iter()
        .map(|file| (file.file_path.display().to_string(), file.centrality_millis))
        .collect::<HashMap<_, _>>();
    let strong_cycle_files = analysis
        .graph_analysis
        .strong_circular_dependencies
        .iter()
        .flat_map(|group| group.iter().map(|path| path.display().to_string()))
        .collect::<BTreeSet<_>>();
    let compatibility_scar_files = analysis
        .architectural_assessment
        .findings
        .iter()
        .filter(|finding| {
            finding.kind == crate::assessment::ArchitecturalAssessmentKind::CompatibilityScar
        })
        .map(|finding| finding.file_path.display().to_string())
        .collect::<BTreeSet<_>>();

    let mut security_by_file = BTreeMap::<String, Vec<&crate::security::SecurityFinding>>::new();
    for finding in &analysis.security_analysis.findings {
        security_by_file
            .entry(finding.file_path.display().to_string())
            .or_default()
            .push(finding);
    }

    for (file, findings) in &security_by_file {
        let visible_security_ids = findings
            .iter()
            .map(|finding| format!("security:native:{}", finding.fingerprint))
            .filter(|id| visible_findings.iter().any(|visible| visible.id == *id))
            .collect::<Vec<_>>();
        if visible_security_ids.is_empty() {
            continue;
        }

        let contexts = findings
            .iter()
            .flat_map(|finding| finding.contexts.iter().copied())
            .collect::<BTreeSet<_>>();
        let mut context_labels = contexts
            .iter()
            .map(|context| security_context_label(*context))
            .collect::<Vec<_>>();
        if let Some(centrality) = bottleneck_by_file.get(file) {
            context_labels.push(format!("bottleneck:{centrality}"));
        }
        if strong_cycle_files.contains(file) {
            context_labels.push(String::from("strong_cycle"));
        }
        let priority = if findings.iter().any(|finding| {
            finding
                .contexts
                .contains(&SecurityContext::ExternallyReachable)
                || finding
                    .contexts
                    .contains(&SecurityContext::EntryReachableViaGraph)
                || matches!(finding.severity, crate::security::SecuritySeverity::High)
        }) {
            "high"
        } else {
            "medium"
        };
        let doctrine_refs = vec![
            String::from("security.coherence"),
            String::from("guardian.trust-boundaries"),
        ];
        let preferred_mechanism = guardian_packet_preferred_mechanism(
            "security_hotspot",
            &doctrine_refs,
            &context_labels,
            doctrine_registry,
        );
        packets.push(GuardianPacket {
            id: format!("guardian:security:{file}"),
            priority: String::from(priority),
            focus: String::from("security_hotspot"),
            primary_target_file: file.clone(),
            precision: String::from("modeled"),
            confidence_millis: if priority == "high" { 870 } else { 760 },
            summary: format!(
                "{} visible native security findings in {}. Use graph pressure, trust boundaries, and framework posture to choose the canonical mitigation path.",
                visible_security_ids.len(),
                file
            ),
            target_files: vec![file.clone()],
            primary_anchor: findings
                .first()
                .map(|finding| anchor(&finding.file_path, Some(finding.line), "primary")),
            evidence_anchors: findings
                .iter()
                .skip(1)
                .take(3)
                .map(|finding| anchor(&finding.file_path, Some(finding.line), "supporting"))
                .collect(),
            locations: Vec::new(),
            finding_ids: visible_security_ids,
            provenance: vec![
                String::from("native_security"),
                String::from("contract_inventory"),
                String::from("graph_analysis"),
            ],
            doctrine_refs,
            preferred_mechanism: preferred_mechanism.clone(),
            obligations: guardian_packet_obligations(
                "security_hotspot",
                file,
                preferred_mechanism.as_deref(),
                &context_labels,
            ),
            suppressibility: guardian_packet_suppressibility("security_hotspot"),
            investigation_questions: guardian_packet_questions(
                "security_hotspot",
                file,
                &context_labels,
            ),
            context_labels,
        });
    }

    for finding in &analysis.architectural_assessment.findings {
        match finding.kind {
            crate::assessment::ArchitecturalAssessmentKind::WarningHeavyHotspot => {
                let file = finding.file_path.display().to_string();
                let visible = visible_by_file.get(&file).cloned().unwrap_or_default();
                if visible.is_empty() {
                    continue;
                }

                let family_counts = review_family_counts(&visible);
                let hardwiring_count = family_counts.get("hardwiring").copied().unwrap_or_default();
                let non_hardwiring_count = family_counts
                    .iter()
                    .filter(|(family, _)| family.as_str() != "hardwiring")
                    .map(|(_, count)| *count)
                    .sum::<usize>();
                let security_count = family_counts.get("security").copied().unwrap_or_default()
                    + family_counts.get("external").copied().unwrap_or_default();
                if hardwiring_count >= visible.len().saturating_sub(1)
                    || (hardwiring_count > non_hardwiring_count * 2 && security_count == 0)
                {
                    continue;
                }

                let mut context_labels = family_counts
                    .into_iter()
                    .map(|(family, count)| format!("{family}:{count}"))
                    .collect::<Vec<_>>();
                context_labels.push(format!("warning_weight:{}", finding.warning_weight));
                if finding.bottleneck_centrality_millis > 0 {
                    context_labels.push(format!(
                        "bottleneck:{}",
                        finding.bottleneck_centrality_millis
                    ));
                }
                if strong_cycle_files.contains(&file) {
                    context_labels.push(String::from("strong_cycle"));
                }

                let priority = if security_count > 0
                    || strong_cycle_files.contains(&file)
                    || finding.bottleneck_centrality_millis >= 700
                {
                    "high"
                } else {
                    "medium"
                };
                let doctrine_refs = vec![
                    String::from("structural.coherence"),
                    String::from("guardian.centralized-damage"),
                ];
                let preferred_mechanism = guardian_packet_preferred_mechanism(
                    "warning_heavy_hotspot",
                    &doctrine_refs,
                    &context_labels,
                    doctrine_registry,
                );
                packets.push(GuardianPacket {
                    id: format!("guardian:warning-hotspot:{file}"),
                    priority: String::from(priority),
                    focus: String::from("warning_heavy_hotspot"),
                    primary_target_file: file.clone(),
                    precision: String::from("modeled"),
                    confidence_millis: if security_count > 0 { 760 } else { 640 },
                    summary: format!(
                        "{} is a central warning hotspot with {} visible findings across {}. Prioritize canonical simplification instead of local cleanup.",
                        file,
                        visible.len(),
                        finding.warning_families.join(", ")
                    ),
                    target_files: vec![file],
                    primary_anchor: best_effort_anchor_for_file(
                        &finding.file_path,
                        analysis,
                        "primary",
                    ),
                    evidence_anchors: Vec::new(),
                    locations: Vec::new(),
                    finding_ids: condensed_packet_finding_ids(&visible),
                    provenance: vec![
                        String::from("graph_analysis"),
                        String::from("architectural_assessment"),
                        String::from("review_surface"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: preferred_mechanism.clone(),
                    obligations: guardian_packet_obligations(
                        "warning_heavy_hotspot",
                        &finding.file_path.display().to_string(),
                        preferred_mechanism.as_deref(),
                        &context_labels,
                    ),
                    suppressibility: guardian_packet_suppressibility("warning_heavy_hotspot"),
                    investigation_questions: guardian_packet_questions(
                        "warning_heavy_hotspot",
                        &finding.file_path.display().to_string(),
                        &context_labels,
                    ),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::SplitIdentityModel => {
                let primary_file = finding.file_path.display().to_string();
                if compatibility_scar_files.contains(&primary_file) {
                    continue;
                }
                let mut target_files = vec![primary_file.clone()];
                target_files.extend(
                    finding
                        .related_file_paths
                        .iter()
                        .map(|path| path.display().to_string()),
                );
                target_files.sort();
                target_files.dedup();
                let mut context_labels = finding.warning_families.clone();
                context_labels.extend(
                    finding
                        .related_identifiers
                        .iter()
                        .take(4)
                        .map(|identifier| format!("identifier:{identifier}")),
                );
                let finding_id = format!(
                    "architecture:split-identity:{}:{}",
                    primary_file,
                    finding.related_identifiers.join("+")
                );
                let doctrine_refs = vec![
                    String::from("pattern.coherence"),
                    String::from("guardian.single-canonical-representation"),
                ];
                let preferred_mechanism = guardian_packet_preferred_mechanism(
                    "split_identity_model",
                    &doctrine_refs,
                    &context_labels,
                    doctrine_registry,
                );
                packets.push(GuardianPacket {
                    id: format!(
                        "guardian:split-identity:{}",
                        finding.file_path.display()
                    ),
                    priority: if finding.severity_millis >= 700 || target_files.len() >= 3 {
                        String::from("high")
                    } else {
                        String::from("medium")
                    },
                    focus: String::from("split_identity_model"),
                    primary_target_file: primary_file.clone(),
                    precision: String::from("heuristic"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} mixes competing representations of the same domain concept across {} files. Converge to one canonical model before more glue code accumulates.",
                        finding.file_path.display(),
                        target_files.len()
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("parsed_sources"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: preferred_mechanism.clone(),
                    obligations: guardian_packet_obligations(
                        "split_identity_model",
                        &finding.file_path.display().to_string(),
                        preferred_mechanism.as_deref(),
                        &context_labels,
                    ),
                    suppressibility: guardian_packet_suppressibility("split_identity_model"),
                    investigation_questions: guardian_packet_questions(
                        "split_identity_model",
                        &finding.file_path.display().to_string(),
                        &context_labels,
                    ),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::CompatibilityScar => {
                let mut target_files = vec![finding.file_path.display().to_string()];
                target_files.extend(
                    finding
                        .related_file_paths
                        .iter()
                        .map(|path| path.display().to_string()),
                );
                target_files.sort();
                target_files.dedup();
                let mut context_labels = finding.warning_families.clone();
                context_labels.extend(
                    finding
                        .related_identifiers
                        .iter()
                        .take(4)
                        .map(|identifier| format!("identifier:{identifier}")),
                );
                if finding.bottleneck_centrality_millis > 0 {
                    context_labels.push(format!(
                        "bottleneck:{}",
                        finding.bottleneck_centrality_millis
                    ));
                }
                let finding_id = format!(
                    "architecture:compatibility-scar:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                );
                let doctrine_refs = vec![
                    String::from("pattern.coherence"),
                    String::from("guardian.single-canonical-representation"),
                    String::from("guardian.translation-hotspot"),
                ];
                let preferred_mechanism = guardian_packet_preferred_mechanism(
                    "compatibility_scar",
                    &doctrine_refs,
                    &context_labels,
                    doctrine_registry,
                );
                packets.push(GuardianPacket {
                    id: format!(
                        "guardian:compatibility-scar:{}",
                        finding.file_path.display()
                    ),
                    priority: if finding.severity_millis >= 700 || target_files.len() >= 3 {
                        String::from("high")
                    } else {
                        String::from("medium")
                    },
                    focus: String::from("compatibility_scar"),
                    primary_target_file: finding.file_path.display().to_string(),
                    precision: String::from("heuristic"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} is centralizing compatibility or translation glue for {} competing concept families. Collapse this into a canonical model and thinner adapters.",
                        finding.file_path.display(),
                        finding.warning_count
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("parsed_sources"),
                        String::from("graph_analysis"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: preferred_mechanism.clone(),
                    obligations: guardian_packet_obligations(
                        "compatibility_scar",
                        &finding.file_path.display().to_string(),
                        preferred_mechanism.as_deref(),
                        &context_labels,
                    ),
                    suppressibility: guardian_packet_suppressibility("compatibility_scar"),
                    investigation_questions: guardian_packet_questions(
                        "compatibility_scar",
                        &finding.file_path.display().to_string(),
                        &context_labels,
                    ),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::DuplicateMechanism => {
                let mut target_files = vec![finding.file_path.display().to_string()];
                target_files.extend(
                    finding
                        .related_file_paths
                        .iter()
                        .map(|path| path.display().to_string()),
                );
                target_files.sort();
                target_files.dedup();
                let mut context_labels = finding.warning_families.clone();
                context_labels.extend(finding.related_identifiers.iter().cloned());
                if finding.bottleneck_centrality_millis > 0 {
                    context_labels.push(format!(
                        "bottleneck:{}",
                        finding.bottleneck_centrality_millis
                    ));
                }
                let finding_id = format!(
                    "architecture:duplicate-mechanism:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                );
                let doctrine_refs = vec![
                    String::from("mechanism.coherence"),
                    String::from("guardian.single-solution-path"),
                ];
                let preferred_mechanism = guardian_packet_preferred_mechanism(
                    "duplicate_mechanism",
                    &doctrine_refs,
                    &context_labels,
                    doctrine_registry,
                );
                packets.push(GuardianPacket {
                    id: format!(
                        "guardian:duplicate-mechanism:{}",
                        finding.file_path.display()
                    ),
                    priority: if finding.severity_millis >= 700 || target_files.len() >= 3 {
                        String::from("high")
                    } else {
                        String::from("medium")
                    },
                    focus: String::from("duplicate_mechanism"),
                    primary_target_file: finding.file_path.display().to_string(),
                    precision: String::from("heuristic"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} is mixing competing orchestration mechanisms for the same concern. Choose one sanctioned pathway and retire the parallel routes.",
                        finding.file_path.display()
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("parsed_sources"),
                        String::from("graph_analysis"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: preferred_mechanism.clone(),
                    obligations: guardian_packet_obligations(
                        "duplicate_mechanism",
                        &finding.file_path.display().to_string(),
                        preferred_mechanism.as_deref(),
                        &context_labels,
                    ),
                    suppressibility: guardian_packet_suppressibility("duplicate_mechanism"),
                    investigation_questions: guardian_packet_questions(
                        "duplicate_mechanism",
                        &finding.file_path.display().to_string(),
                        &context_labels,
                    ),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::UnwiredFrameworkArtifact => {
                let target_files = vec![finding.file_path.display().to_string()];
                let context_labels = finding.related_identifiers.clone();
                let finding_id = format!(
                    "architecture:unwired-framework-artifact:{}",
                    finding.file_path.display()
                );
                let role = finding
                    .related_identifiers
                    .iter()
                    .find_map(|id| id.strip_prefix("role:"))
                    .unwrap_or("artifact")
                    .to_string();
                let advice = finding
                    .related_identifiers
                    .iter()
                    .find_map(|id| id.strip_prefix("advice:"))
                    .unwrap_or("Wire this artifact into its framework channel or delete it.")
                    .to_string();
                packets.push(GuardianPacket {
                    id: format!(
                        "guardian:unwired-framework-artifact:{}",
                        finding.file_path.display()
                    ),
                    priority: String::from("medium"),
                    focus: String::from("unwired_framework_artifact"),
                    primary_target_file: finding.file_path.display().to_string(),
                    precision: String::from("modeled"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} is shaped like a {role} but no framework channel wires it in — invented, not integrated.",
                        finding.file_path.display()
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("contract_inventory"),
                    ],
                    doctrine_refs: vec![String::from("structural.integration")],
                    preferred_mechanism: Some(String::from("framework_channel_registration")),
                    obligations: vec![GuardianObligation {
                        action: advice,
                        acceptance: String::from(
                            "The artifact is reachable through its framework channel (route, dispatch site, registration), or it no longer exists.",
                        ),
                    }],
                    suppressibility: guardian_packet_suppressibility("unwired_framework_artifact"),
                    investigation_questions: Vec::new(),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::GodClass => {
                let target_files = vec![finding.file_path.display().to_string()];
                let context_labels = finding.related_identifiers.clone();
                let finding_id = format!("architecture:god-class:{}", finding.file_path.display());
                let doctrine_refs = vec![
                    String::from("structural.coherence"),
                    String::from("guardian.architectonic-quality"),
                ];
                packets.push(GuardianPacket {
                    id: format!("guardian:god-class:{}", finding.file_path.display()),
                    priority: if finding.severity_millis >= 700 {
                        String::from("high")
                    } else {
                        String::from("medium")
                    },
                    focus: String::from("god_class"),
                    primary_target_file: finding.file_path.display().to_string(),
                    precision: String::from("modeled"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} exposes {} public methods consumed from {} distinct files; split it along its usage clusters before it accretes further.",
                        finding.file_path.display(),
                        finding.warning_count,
                        finding.warning_weight
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("semantic_graph"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: Some(String::from(
                        "segregated_role_interfaces",
                    )),
                    obligations: vec![GuardianObligation {
                        action: format!(
                            "Group `{}`'s externally-used methods by consumer concern and extract each group behind its own interface or focused class.",
                            finding.file_path.display()
                        ),
                        acceptance: String::from(
                            "No single container both exposes a wide public surface and serves as the direct dependency of a wide consumer set; consumers depend on the narrow interface they actually use.",
                        ),
                    }],
                    suppressibility: guardian_packet_suppressibility("god_class"),
                    investigation_questions: Vec::new(),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::LayerContractViolation => {
                let mut target_files = vec![finding.file_path.display().to_string()];
                target_files.extend(
                    finding
                        .related_file_paths
                        .iter()
                        .map(|path| path.display().to_string()),
                );
                let context_labels = finding.related_identifiers.clone();
                let finding_id = format!(
                    "architecture:layer-contract-violation:{}:{}",
                    finding.file_path.display(),
                    finding
                        .related_file_paths
                        .first()
                        .map(|path| path.display().to_string())
                        .unwrap_or_default()
                );
                let doctrine_refs = vec![
                    String::from("structural.coherence"),
                    String::from("guardian.architectonic-quality"),
                ];
                packets.push(GuardianPacket {
                    id: format!(
                        "guardian:layer-contract-violation:{}",
                        finding.file_path.display()
                    ),
                    priority: String::from("high"),
                    focus: String::from("layer_contract_violation"),
                    primary_target_file: finding.file_path.display().to_string(),
                    precision: String::from("certain"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} depends on {} against the declared layer contract ({} edge instance(s)); invert or remove the wrong-direction dependency.",
                        finding.file_path.display(),
                        finding
                            .related_file_paths
                            .first()
                            .map(|path| path.display().to_string())
                            .unwrap_or_default(),
                        finding.warning_count
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("semantic_graph"),
                        String::from("doctrine_registry"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: Some(String::from("declared_layer_contract")),
                    obligations: vec![GuardianObligation {
                        action: format!(
                            "Invert the dependency from `{}` so the lower layer exposes a contract the higher layer implements or consumes, or move the misplaced code into the layer it actually belongs to.",
                            finding.file_path.display()
                        ),
                        acceptance: String::from(
                            "No resolved edge crosses the declared layer boundary in the forbidden direction.",
                        ),
                    }],
                    suppressibility: guardian_packet_suppressibility("layer_contract_violation"),
                    investigation_questions: Vec::new(),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::SanctionedPathBypass => {
                let target_files = vec![finding.file_path.display().to_string()];
                let mut context_labels = finding.warning_families.clone();
                context_labels.extend(finding.related_identifiers.iter().cloned());
                if finding.bottleneck_centrality_millis > 0 {
                    context_labels.push(format!(
                        "bottleneck:{}",
                        finding.bottleneck_centrality_millis
                    ));
                }
                let finding_id = format!(
                    "architecture:sanctioned-path-bypass:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                );
                let doctrine_refs = vec![
                    String::from("configuration.coherence"),
                    String::from("guardian.sanctioned-paths"),
                ];
                let preferred_mechanism = guardian_packet_preferred_mechanism(
                    "sanctioned_path_bypass",
                    &doctrine_refs,
                    &context_labels,
                    doctrine_registry,
                );
                packets.push(GuardianPacket {
                    id: format!(
                        "guardian:sanctioned-path-bypass:{}",
                        finding.file_path.display()
                    ),
                    priority: if finding.severity_millis >= 700 {
                        String::from("high")
                    } else {
                        String::from("medium")
                    },
                    focus: String::from("sanctioned_path_bypass"),
                    primary_target_file: finding.file_path.display().to_string(),
                    precision: String::from("heuristic"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} bypasses a sanctioned configuration path by mixing raw environment access with an approved configuration access pattern.",
                        finding.file_path.display()
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("hardwiring_detector"),
                        String::from("parsed_sources"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: preferred_mechanism.clone(),
                    obligations: guardian_packet_obligations(
                        "sanctioned_path_bypass",
                        &finding.file_path.display().to_string(),
                        preferred_mechanism.as_deref(),
                        &context_labels,
                    ),
                    suppressibility: guardian_packet_suppressibility("sanctioned_path_bypass"),
                    investigation_questions: guardian_packet_questions(
                        "sanctioned_path_bypass",
                        &finding.file_path.display().to_string(),
                        &context_labels,
                    ),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::AbstractionSprawl => {
                let mut target_files = vec![finding.file_path.display().to_string()];
                target_files.extend(
                    finding
                        .related_file_paths
                        .iter()
                        .map(|path| path.display().to_string()),
                );
                target_files.sort();
                target_files.dedup();
                let mut context_labels = finding.warning_families.clone();
                context_labels.extend(finding.related_identifiers.iter().cloned());
                if finding.bottleneck_centrality_millis > 0 {
                    context_labels.push(format!(
                        "bottleneck:{}",
                        finding.bottleneck_centrality_millis
                    ));
                }
                let finding_id = format!(
                    "architecture:abstraction-sprawl:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                );
                let doctrine_refs = vec![
                    String::from("mechanism.coherence"),
                    String::from("guardian.minimal-mechanism"),
                    String::from("guardian.overengineering"),
                ];
                let preferred_mechanism = guardian_packet_preferred_mechanism(
                    "abstraction_sprawl",
                    &doctrine_refs,
                    &context_labels,
                    doctrine_registry,
                );
                packets.push(GuardianPacket {
                    id: format!(
                        "guardian:abstraction-sprawl:{}",
                        finding.file_path.display()
                    ),
                    priority: if finding.severity_millis >= 700 || target_files.len() >= 3 {
                        String::from("high")
                    } else {
                        String::from("medium")
                    },
                    focus: String::from("abstraction_sprawl"),
                    primary_target_file: finding.file_path.display().to_string(),
                    precision: String::from("heuristic"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} spreads one concern across too many abstraction roles. Collapse the indirection until one primary boundary remains.",
                        finding.file_path.display()
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("parsed_sources"),
                        String::from("graph_analysis"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: preferred_mechanism.clone(),
                    obligations: guardian_packet_obligations(
                        "abstraction_sprawl",
                        &finding.file_path.display().to_string(),
                        preferred_mechanism.as_deref(),
                        &context_labels,
                    ),
                    suppressibility: guardian_packet_suppressibility("abstraction_sprawl"),
                    investigation_questions: guardian_packet_questions(
                        "abstraction_sprawl",
                        &finding.file_path.display().to_string(),
                        &context_labels,
                    ),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::HandRolledParsing => {
                let mut target_files = vec![finding.file_path.display().to_string()];
                target_files.extend(
                    finding
                        .related_file_paths
                        .iter()
                        .map(|path| path.display().to_string()),
                );
                target_files.sort();
                target_files.dedup();
                let mut context_labels = finding.warning_families.clone();
                context_labels.extend(finding.related_identifiers.iter().cloned());
                if finding.bottleneck_centrality_millis > 0 {
                    context_labels.push(format!(
                        "bottleneck:{}",
                        finding.bottleneck_centrality_millis
                    ));
                }
                let finding_id = format!(
                    "architecture:hand-rolled-parsing:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                );
                let is_contract_stack = finding
                    .warning_families
                    .iter()
                    .any(|family| family == "concern:custom_contract_stack");
                let is_schema_validation = finding
                    .warning_families
                    .iter()
                    .any(|family| family == "concern:custom_schema_validation");
                let is_definition_engine = finding
                    .warning_families
                    .iter()
                    .any(|family| family == "concern:custom_definition_engine");
                let is_scheduler_dsl = finding
                    .warning_families
                    .iter()
                    .any(|family| family == "concern:custom_scheduler_dsl");
                let is_filesystem_page_resolution = finding
                    .warning_families
                    .iter()
                    .any(|family| family == "concern:filesystem_page_resolution");
                let is_manifest_backed_policy_engine = finding
                    .warning_families
                    .iter()
                    .any(|family| family == "concern:manifest_backed_policy_engine");
                let doctrine_refs = vec![
                    String::from("guardian.native-vs-library"),
                    String::from(if is_manifest_backed_policy_engine {
                        "guardian.avoid-manifest-backed-policy-engine"
                    } else if is_filesystem_page_resolution {
                        "guardian.avoid-filesystem-page-resolution"
                    } else if is_schema_validation {
                        "guardian.avoid-homegrown-schema-validation"
                    } else if is_scheduler_dsl {
                        "guardian.avoid-homegrown-scheduler-dsl"
                    } else if is_definition_engine {
                        "guardian.avoid-homegrown-definition-engine"
                    } else {
                        "guardian.avoid-homegrown-parser"
                    }),
                    String::from("guardian.overengineering"),
                ];
                let preferred_mechanism = guardian_packet_preferred_mechanism(
                    "hand_rolled_parsing",
                    &doctrine_refs,
                    &context_labels,
                    doctrine_registry,
                );
                packets.push(GuardianPacket {
                    id: format!(
                        "guardian:hand-rolled-parsing:{}",
                        finding.file_path.display()
                    ),
                    priority: if finding.severity_millis >= 700 || target_files.len() >= 3 {
                        String::from("high")
                    } else {
                        String::from("medium")
                    },
                    focus: String::from("hand_rolled_parsing"),
                    primary_target_file: finding.file_path.display().to_string(),
                    precision: String::from("heuristic"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} appears to own a homegrown {}. Verify whether a battle-tested native/framework/library mechanism should replace it.",
                        finding.file_path.display(),
                        if is_manifest_backed_policy_engine {
                            "manifest-backed template policy/runtime engine"
                        } else if is_filesystem_page_resolution {
                            "filesystem-backed page or route resolution layer"
                        } else if is_schema_validation {
                            "schema or validation contract stack"
                        } else if is_scheduler_dsl {
                            "scheduler or job-definition DSL"
                        } else if is_definition_engine {
                            "definition or metadata engine"
                        } else if is_contract_stack {
                            "contract stack"
                        } else {
                            "parsing or mini-protocol stack"
                        }
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("parsed_sources"),
                        String::from("graph_analysis"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: preferred_mechanism.clone(),
                    obligations: guardian_packet_obligations(
                        "hand_rolled_parsing",
                        &finding.file_path.display().to_string(),
                        preferred_mechanism.as_deref(),
                        &context_labels,
                    ),
                    suppressibility: guardian_packet_suppressibility("hand_rolled_parsing"),
                    investigation_questions: guardian_packet_questions(
                        "hand_rolled_parsing",
                        &finding.file_path.display().to_string(),
                        &context_labels,
                    ),
                    context_labels,
                });
            }
            crate::assessment::ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot => {
                let mut target_files = vec![finding.file_path.display().to_string()];
                target_files.extend(
                    finding
                        .related_file_paths
                        .iter()
                        .map(|path| path.display().to_string()),
                );
                target_files.sort();
                target_files.dedup();
                let mut context_labels = finding.warning_families.clone();
                context_labels.extend(finding.related_identifiers.iter().take(4).map(
                    |identifier| {
                        if identifier.starts_with("entry_path:") {
                            identifier.clone()
                        } else {
                            format!("token:{identifier}")
                        }
                    },
                ));
                if finding.pressure_path.len() > 1 {
                    context_labels.push(format!(
                        "pressure_path: {}",
                        finding
                            .pressure_path
                            .iter()
                            .map(|hop| hop.file_path.display().to_string())
                            .collect::<Vec<_>>()
                            .join(" -> ")
                    ));
                }
                if let Some(symbols) = finding
                    .pressure_path
                    .iter()
                    .filter_map(|hop| {
                        hop.source_symbol
                            .as_ref()
                            .zip(hop.target_symbol.as_ref())
                            .map(|(source, target)| format!("{source} -> {target}"))
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .reduce(|left, right| format!("{left} | {right}"))
                {
                    context_labels.push(format!("pressure_path_symbols: {symbols}"));
                }
                if !finding.expensive_operation_flow.is_empty() {
                    context_labels.push(format!(
                        "expensive_operation_flow: {}",
                        finding
                            .expensive_operation_flow
                            .iter()
                            .map(|step| match step.kind {
                                crate::assessment::ArchitecturalComplexityFlowStepKind::PressureHop => {
                                    let mut label = step.file_path.display().to_string();
                                    if let Some(line) = step.line {
                                        label.push(':');
                                        label.push_str(&line.to_string());
                                    }
                                    if let Some(relation) = step.relation_to_next {
                                        label.push(':');
                                        label.push_str(&format!("{relation:?}"));
                                    }
                                    label
                                }
                                crate::assessment::ArchitecturalComplexityFlowStepKind::OperationSite => format!(
                                    "{}:{}:{}",
                                    step.file_path.display(),
                                    step.line.unwrap_or_default(),
                                    step.subtype.as_deref().unwrap_or("operation")
                                ),
                            })
                            .collect::<Vec<_>>()
                            .join(" -> ")
                    ));
                }
                if let Some(symbols) = finding
                    .expensive_operation_flow
                    .iter()
                    .filter_map(|step| {
                        step.source_symbol
                            .as_ref()
                            .zip(step.target_symbol.as_ref())
                            .map(|(source, target)| format!("{source} -> {target}"))
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .reduce(|left, right| format!("{left} | {right}"))
                {
                    context_labels.push(format!("expensive_operation_flow_symbols: {symbols}"));
                }
                if finding.warning_count > 0 {
                    context_labels.push(format!("occurrences:{}", finding.warning_count));
                }
                context_labels.extend(finding.expensive_operation_sites.iter().take(3).map(
                    |site| format!("operation:{}@{}:{}", site.subtype, site.line, site.token),
                ));
                let finding_id = format!(
                    "architecture:algorithmic-complexity:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                );
                let doctrine_refs = vec![
                    String::from("performance.scaling"),
                    String::from("guardian.superlinear-risk"),
                ];
                let preferred_mechanism = guardian_packet_preferred_mechanism(
                    "algorithmic_complexity_hotspot",
                    &doctrine_refs,
                    &context_labels,
                    doctrine_registry,
                );
                packets.push(GuardianPacket {
                    id: format!(
                        "guardian:algorithmic-complexity:{}",
                        finding.file_path.display()
                    ),
                    priority: if finding.severity_millis >= 820 || finding.warning_count >= 2 {
                        String::from("high")
                    } else {
                        String::from("medium")
                    },
                    focus: String::from("algorithmic_complexity_hotspot"),
                    primary_target_file: finding.file_path.display().to_string(),
                    precision: String::from("heuristic"),
                    confidence_millis: finding.severity_millis,
                    summary: format!(
                        "{} contains repeated expensive loop-local work ({}). Reduce the hotspot before it turns into visible scale debt.",
                        finding.file_path.display(),
                        finding.warning_families.join(", ")
                    ),
                    target_files,
                    primary_anchor: best_effort_anchor_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    evidence_anchors: supporting_anchors_for_architectural_assessment(
                        finding,
                        analysis,
                    ),
                    locations: Vec::new(),
                    finding_ids: vec![finding_id],
                    provenance: vec![
                        String::from("architectural_assessment"),
                        String::from("parsed_sources"),
                    ],
                    doctrine_refs,
                    preferred_mechanism: preferred_mechanism.clone(),
                    obligations: guardian_packet_obligations(
                        "algorithmic_complexity_hotspot",
                        &finding.file_path.display().to_string(),
                        preferred_mechanism.as_deref(),
                        &context_labels,
                    ),
                    suppressibility: guardian_packet_suppressibility(
                        "algorithmic_complexity_hotspot",
                    ),
                    investigation_questions: guardian_packet_questions(
                        "algorithmic_complexity_hotspot",
                        &finding.file_path.display().to_string(),
                        &context_labels,
                    ),
                    context_labels,
                });
            }
        }
    }

    for packet in &mut packets {
        if let Some(finding) = best_packet_supporting_finding(packet, visible_findings) {
            packet.primary_anchor = finding.primary_anchor.clone();
            packet.evidence_anchors = finding.evidence_anchors.clone();
            packet.locations = finding.locations.clone();
        } else {
            packet.locations =
                ordered_packet_locations(packet.primary_anchor.as_ref(), &packet.evidence_anchors);
        }
    }

    packets = compact_algorithmic_complexity_packets(packets);

    packets.sort_by(|left, right| {
        packet_priority_rank(&right.priority)
            .cmp(&packet_priority_rank(&left.priority))
            .then(packet_focus_rank(&right.focus).cmp(&packet_focus_rank(&left.focus)))
            .then(left.target_files.cmp(&right.target_files))
            .then(left.focus.cmp(&right.focus))
    });
    select_diverse_guardian_packet_budget(packets, 8)
}

fn compact_algorithmic_complexity_packets(packets: Vec<GuardianPacket>) -> Vec<GuardianPacket> {
    let mut compacted = Vec::with_capacity(packets.len());
    let mut merged_by_file = BTreeMap::<String, GuardianPacket>::new();

    for packet in packets {
        if packet.focus != "algorithmic_complexity_hotspot" {
            compacted.push(packet);
            continue;
        }

        if let Some(existing) = merged_by_file.get_mut(&packet.primary_target_file) {
            merge_algorithmic_complexity_packet(existing, packet);
        } else {
            merged_by_file.insert(packet.primary_target_file.clone(), packet);
        }
    }

    compacted.extend(
        merged_by_file
            .into_values()
            .map(finalize_algorithmic_complexity_packet),
    );
    compacted
}

fn merge_algorithmic_complexity_packet(target: &mut GuardianPacket, packet: GuardianPacket) {
    if packet_priority_rank(&packet.priority) > packet_priority_rank(&target.priority) {
        target.priority = packet.priority;
    }
    if packet.confidence_millis > target.confidence_millis {
        target.confidence_millis = packet.confidence_millis;
    }
    if packet.precision == "exact" || (target.precision != "exact" && packet.precision == "modeled")
    {
        target.precision = packet.precision;
    }
    if target.primary_anchor.is_none() {
        target.primary_anchor = packet.primary_anchor;
    }

    merge_sorted_unique_strings(&mut target.target_files, packet.target_files);
    merge_sorted_unique_anchors(&mut target.evidence_anchors, packet.evidence_anchors);
    merge_sorted_unique_anchors(&mut target.locations, packet.locations);
    merge_sorted_unique_strings(&mut target.finding_ids, packet.finding_ids);
    merge_sorted_unique_strings(&mut target.provenance, packet.provenance);
    merge_sorted_unique_strings(&mut target.doctrine_refs, packet.doctrine_refs);
    merge_sorted_unique_strings(&mut target.context_labels, packet.context_labels);

    if target.preferred_mechanism.is_none() {
        target.preferred_mechanism = packet.preferred_mechanism;
    }
}

fn finalize_algorithmic_complexity_packet(mut packet: GuardianPacket) -> GuardianPacket {
    packet.locations =
        ordered_packet_locations(packet.primary_anchor.as_ref(), &packet.evidence_anchors);
    let total_occurrences = packet
        .context_labels
        .iter()
        .filter_map(|label| label.strip_prefix("occurrences:"))
        .filter_map(|value| value.parse::<usize>().ok())
        .sum::<usize>();
    let mut complexity_labels = packet
        .context_labels
        .iter()
        .filter(|label| label.starts_with("complexity:"))
        .cloned()
        .collect::<Vec<_>>();
    complexity_labels.sort();
    complexity_labels.dedup();

    let mut token_labels = packet
        .context_labels
        .iter()
        .filter(|label| label.starts_with("token:"))
        .cloned()
        .collect::<Vec<_>>();
    token_labels.sort();
    token_labels.dedup();

    let mut other_labels = packet
        .context_labels
        .iter()
        .filter(|label| !label.starts_with("complexity:") && !label.starts_with("token:"))
        .filter(|label| !label.starts_with("occurrences:"))
        .cloned()
        .collect::<Vec<_>>();
    other_labels.sort();
    other_labels.dedup();

    packet.context_labels = complexity_labels.clone();
    packet
        .context_labels
        .extend(token_labels.into_iter().take(6));
    if total_occurrences > 0 {
        packet
            .context_labels
            .push(format!("occurrences:{total_occurrences}"));
    }
    packet.context_labels.extend(other_labels);

    let summary_kinds = complexity_labels
        .iter()
        .map(|label| label.trim_start_matches("complexity:"))
        .collect::<Vec<_>>();
    let entry_pressure = packet
        .context_labels
        .iter()
        .any(|label| label == "pressure:entry_reachable_via_graph");
    let direct_entry = packet
        .context_labels
        .iter()
        .any(|label| label == "pressure:direct_runtime_entry");
    let entry_path = packet
        .context_labels
        .iter()
        .find(|label| label.starts_with("entry_path:"))
        .cloned()
        .or_else(|| {
            packet
                .context_labels
                .iter()
                .find(|label| label.starts_with("pressure_path:"))
                .cloned()
        });
    packet.summary = format!(
        "{} contains repeated expensive loop-local work ({}).{}{} Reduce the hotspot before it turns into visible scale debt.",
        packet.primary_target_file,
        summary_kinds.join(", "),
        if direct_entry {
            " It sits directly on a runtime entry."
        } else if entry_pressure {
            " It is reachable from runtime entry code."
        } else {
            ""
        },
        entry_path
            .map(|path| format!(" {}", path))
            .unwrap_or_default()
    );
    packet.obligations = guardian_packet_obligations(
        "algorithmic_complexity_hotspot",
        &packet.primary_target_file,
        packet.preferred_mechanism.as_deref(),
        &packet.context_labels,
    );
    packet.investigation_questions = guardian_packet_questions(
        "algorithmic_complexity_hotspot",
        &packet.primary_target_file,
        &packet.context_labels,
    );
    packet
}

fn merge_sorted_unique_strings(target: &mut Vec<String>, source: Vec<String>) {
    for value in source {
        if !target.contains(&value) {
            target.push(value);
        }
    }
    target.sort();
    target.dedup();
}

fn merge_sorted_unique_anchors(target: &mut Vec<EvidenceAnchor>, source: Vec<EvidenceAnchor>) {
    for value in source {
        if !target.contains(&value) {
            target.push(value);
        }
    }
    target.sort_by(|left, right| {
        left.file_path
            .cmp(&right.file_path)
            .then(left.line.cmp(&right.line))
            .then(left.label.cmp(&right.label))
    });
    target.dedup();
}

fn ordered_packet_locations(
    primary_anchor: Option<&EvidenceAnchor>,
    evidence_anchors: &[EvidenceAnchor],
) -> Vec<EvidenceAnchor> {
    let mut locations = Vec::new();
    if let Some(anchor) = primary_anchor {
        locations.push(anchor.clone());
    }
    for anchor in evidence_anchors {
        if !locations.contains(anchor) {
            locations.push(anchor.clone());
        }
    }
    locations
}

fn select_diverse_guardian_packet_budget(
    packets: Vec<GuardianPacket>,
    budget: usize,
) -> Vec<GuardianPacket> {
    if packets.len() <= budget {
        return packets;
    }

    let mut selected = Vec::with_capacity(budget);
    let mut overflow = Vec::new();
    let mut seen_focuses = BTreeSet::<String>::new();

    for packet in packets {
        if selected.len() < budget && seen_focuses.insert(packet.focus.clone()) {
            selected.push(packet);
        } else {
            overflow.push(packet);
        }
    }

    if selected.len() < budget {
        selected.extend(overflow.into_iter().take(budget - selected.len()));
    }
    selected
}

fn anchor(file_path: &Path, line: Option<usize>, label: &str) -> EvidenceAnchor {
    EvidenceAnchor {
        file_path: file_path.to_path_buf(),
        line,
        label: String::from(label),
    }
}

fn best_effort_anchor_for_architectural_assessment(
    finding: &crate::assessment::ArchitecturalAssessmentFinding,
    analysis: &ProjectAnalysis,
) -> Option<EvidenceAnchor> {
    let mut tokens = finding
        .related_identifiers
        .iter()
        .filter_map(|identifier| {
            identifier
                .strip_prefix("concept:")
                .or_else(|| identifier.strip_prefix("raw_"))
                .map(String::from)
                .or_else(|| {
                    if identifier.starts_with("role:") {
                        None
                    } else {
                        Some(identifier.clone())
                    }
                })
        })
        .collect::<Vec<_>>();
    tokens.extend(
        finding
            .warning_families
            .iter()
            .filter_map(|family| family.split(':').next_back())
            .map(String::from),
    );
    best_effort_anchor_for_file_with_tokens(&finding.file_path, analysis, "primary", &tokens)
}

fn supporting_anchors_for_architectural_assessment(
    finding: &crate::assessment::ArchitecturalAssessmentFinding,
    analysis: &ProjectAnalysis,
) -> Vec<EvidenceAnchor> {
    finding
        .related_file_paths
        .iter()
        .filter_map(|path| best_effort_anchor_for_file(path, analysis, "supporting"))
        .collect()
}

fn best_effort_anchor_for_file(
    file_path: &Path,
    analysis: &ProjectAnalysis,
    label: &str,
) -> Option<EvidenceAnchor> {
    let mut tokens = Vec::new();
    if let Some(stem) = file_path.file_stem().and_then(|stem| stem.to_str()) {
        tokens.push(stem.to_string());
    }
    best_effort_anchor_for_file_with_tokens(file_path, analysis, label, &tokens)
}

fn best_effort_anchor_for_file_with_tokens(
    file_path: &Path,
    analysis: &ProjectAnalysis,
    label: &str,
    tokens: &[String],
) -> Option<EvidenceAnchor> {
    let content = analysis
        .parsed_sources
        .iter()
        .find_map(|(path, content)| (path == file_path).then_some(content))?;
    let line = anchor_line_for_content(content, tokens);
    Some(anchor(file_path, line, label))
}

fn anchor_line_for_content(content: &str, tokens: &[String]) -> Option<usize> {
    let lowered_tokens = tokens
        .iter()
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    for (index, line) in content.lines().enumerate() {
        let lowered = line.to_ascii_lowercase();
        if lowered_tokens.iter().any(|token| lowered.contains(token)) {
            return Some(index + 1);
        }
    }

    content
        .lines()
        .enumerate()
        .find(|(_, line)| !line.trim().is_empty())
        .map(|(index, _)| index + 1)
}

fn best_packet_supporting_finding<'a>(
    packet: &GuardianPacket,
    visible_findings: &'a [&crate::review::ReviewFinding],
) -> Option<&'a crate::review::ReviewFinding> {
    if let Some(finding) = packet.finding_ids.iter().find_map(|id| {
        visible_findings
            .iter()
            .copied()
            .find(|finding| &finding.id == id)
    }) {
        return Some(finding);
    }

    visible_findings.iter().copied().find(|finding| {
        finding
            .file_paths
            .iter()
            .any(|path| path == &packet.primary_target_file)
    })
}

fn review_family_counts(findings: &[&crate::review::ReviewFinding]) -> BTreeMap<String, usize> {
    let mut family_counts = BTreeMap::<String, usize>::new();
    for finding in findings {
        *family_counts
            .entry(format!("{:?}", finding.family).to_ascii_lowercase())
            .or_insert(0) += 1;
    }
    family_counts
}

fn condensed_packet_finding_ids(findings: &[&crate::review::ReviewFinding]) -> Vec<String> {
    let mut ids = Vec::new();
    for prefix in ["architecture:", "graph:bottleneck:", "graph:cycle:"] {
        for finding in findings {
            if finding.id.starts_with(prefix) && !ids.contains(&finding.id) {
                ids.push(finding.id.clone());
            }
        }
    }
    let mut supporting = findings.iter().collect::<Vec<_>>();
    supporting.sort_by(|left, right| {
        packet_support_rank(right.family)
            .cmp(&packet_support_rank(left.family))
            .then(review_severity_rank(right.severity).cmp(&review_severity_rank(left.severity)))
            .then(left.id.cmp(&right.id))
    });
    for finding in supporting {
        if ids.len() >= 8 {
            break;
        }
        if !ids.contains(&finding.id) {
            ids.push(finding.id.clone());
        }
    }
    ids
}

fn packet_priority_rank(priority: &str) -> u8 {
    match priority {
        "high" => 3,
        "medium" => 2,
        _ => 1,
    }
}

fn packet_support_rank(family: crate::review::ReviewFindingFamily) -> u8 {
    match family {
        crate::review::ReviewFindingFamily::Security => 5,
        crate::review::ReviewFindingFamily::External => 4,
        crate::review::ReviewFindingFamily::Graph => 3,
        crate::review::ReviewFindingFamily::DeadCode => 2,
        crate::review::ReviewFindingFamily::Hardwiring => 1,
    }
}

fn review_severity_rank(severity: crate::review::ReviewFindingSeverity) -> u8 {
    match severity {
        crate::review::ReviewFindingSeverity::High => 3,
        crate::review::ReviewFindingSeverity::Medium => 2,
        crate::review::ReviewFindingSeverity::Low => 1,
    }
}

fn packet_focus_rank(focus: &str) -> u8 {
    match focus {
        "security_hotspot" => 6,
        "hand_rolled_parsing" => 5,
        "algorithmic_complexity_hotspot" => 5,
        "sanctioned_path_bypass" => 4,
        "duplicate_mechanism" => 4,
        "abstraction_sprawl" => 3,
        "compatibility_scar" => 2,
        "split_identity_model" => 1,
        "warning_heavy_hotspot" => 0,
        _ => 0,
    }
}

fn guardian_packet_preferred_mechanism(
    focus: &str,
    doctrine_refs: &[String],
    context_labels: &[String],
    doctrine_registry: &DoctrineRegistry,
) -> Option<String> {
    if let Some(preferred_mechanism) = doctrine_refs
        .iter()
        .enumerate()
        .filter_map(|(index, id)| {
            doctrine_registry.clause(id).and_then(|clause| {
                clause.preferred_mechanism.as_ref().map(|mechanism| {
                    (
                        doctrine_disposition_rank(clause.default_disposition),
                        index,
                        mechanism.clone(),
                    )
                })
            })
        })
        .max_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)))
        .map(|(_, _, mechanism)| mechanism)
    {
        return Some(preferred_mechanism);
    }

    match focus {
        "security_hotspot" => Some(
            if context_labels.iter().any(|label| {
                label == "externally_reachable"
                    || label == "entry_reachable_via_graph"
                    || label == "interactive_execution"
            }) {
                String::from("sanctioned_security_boundary")
            } else {
                String::from("trusted_runtime_wrapper")
            },
        ),
        "warning_heavy_hotspot" => Some(String::from("single_authoritative_service_boundary")),
        "algorithmic_complexity_hotspot" => {
            Some(String::from("precomputed_index_or_single_pass_flow"))
        }
        "abstraction_sprawl" => Some(String::from("single_authoritative_domain_boundary")),
        "hand_rolled_parsing" => Some(String::from("battle_tested_parser_or_native_contract")),
        "duplicate_mechanism" => Some(String::from("single_sanctioned_orchestration_path")),
        "sanctioned_path_bypass" => Some(String::from("single_sanctioned_configuration_path")),
        "split_identity_model" | "compatibility_scar" => {
            Some(String::from("single_canonical_domain_contract"))
        }
        _ => None,
    }
}

fn doctrine_disposition_rank(disposition: DoctrineDisposition) -> u8 {
    match disposition {
        DoctrineDisposition::Block => 3,
        DoctrineDisposition::Warn => 2,
        DoctrineDisposition::Inform => 1,
    }
}

fn guardian_packet_obligations(
    focus: &str,
    primary_file: &str,
    preferred_mechanism: Option<&str>,
    _context_labels: &[String],
) -> Vec<GuardianObligation> {
    match focus {
        "security_hotspot" => vec![
            GuardianObligation {
                action: format!(
                    "Trace how meaningful input reaches `{primary_file}` and remove or isolate the dangerous primitive behind `{}`.",
                    preferred_mechanism.unwrap_or("a sanctioned boundary")
                ),
                acceptance: String::from(
                    "The remaining path either eliminates the primitive or routes it through a reviewed boundary with explicit validation and authorization constraints.",
                ),
            },
            GuardianObligation {
                action: String::from(
                    "Document why this primitive is still required and which caller or runtime surface owns the trust boundary.",
                ),
                acceptance: String::from(
                    "The owning boundary and mitigation path are explicit in code or doctrine instead of depending on ambient behavior.",
                ),
            },
        ],
        "warning_heavy_hotspot" => vec![
            GuardianObligation {
                action: format!(
                    "Collapse the competing concerns around `{primary_file}` into `{}`.",
                    preferred_mechanism.unwrap_or(
                        "one canonical service, adapter, or responsibility boundary",
                    )
                ),
                acceptance: String::from(
                    "The hotspot no longer acts as the place where unrelated architectural concerns accumulate.",
                ),
            },
            GuardianObligation {
                action: String::from(
                    "Remove one-off local fixes that duplicate an existing mechanism or boundary.",
                ),
                acceptance: String::from(
                    "The changed slice uses one authoritative pathway for the concern instead of parallel cleanup logic.",
                ),
            },
        ],
        "algorithmic_complexity_hotspot" => vec![
            GuardianObligation {
                action: format!(
                    "Refactor `{primary_file}` so the risky loop-local work runs through `{}` instead of repeated scans, sorting, parse/decode, file reads, or regex compilation inside loops.",
                    preferred_mechanism.unwrap_or("a precomputed index or single-pass accumulation path")
                ),
                acceptance: String::from(
                    "The changed slice removes the obvious superlinear hotspot or hoists the expensive work out of the loop.",
                ),
            },
            GuardianObligation {
                action: String::from(
                    "Measure the hot path before and after the change so the remaining complexity risk is justified with actual scale assumptions.",
                ),
                acceptance: String::from(
                    "The code or review makes clear why the remaining runtime cost is acceptable for expected input sizes.",
                ),
            },
        ],
        "abstraction_sprawl" => vec![
            GuardianObligation {
                action: format!(
                    "Collapse the helper/service/factory/registry indirection around `{primary_file}` until `{}` owns the concern.",
                    preferred_mechanism.unwrap_or("one primary boundary")
                ),
                acceptance: String::from(
                    "The concern no longer requires multiple abstraction roles to understand or change one flow.",
                ),
            },
            GuardianObligation {
                action: String::from(
                    "Retire decorative abstractions that only rename or forward behavior without protecting a real boundary.",
                ),
                acceptance: String::from(
                    "The remaining abstraction layers each own a distinct boundary or capability instead of stacking incidental wrappers.",
                ),
            },
        ],
        "hand_rolled_parsing" => vec![
            GuardianObligation {
                action: format!(
                    "Audit `{primary_file}` for custom parsing, schema validation, scheduler/orchestration DSLs, definition-engine, validator/resolver, or mini-language logic and route the concern through `{}` if it is a valid sanctioned replacement.",
                    preferred_mechanism.unwrap_or("a battle-tested native/framework/library parser")
                ),
                acceptance: String::from(
                    "The changed slice no longer depends on an unnecessary homegrown parsing, validation, or scheduler/orchestration stack when a sanctioned mechanism already exists.",
                ),
            },
            GuardianObligation {
                action: String::from(
                    "If custom parsing, validation, scheduler, or definition-engine behavior is still required, isolate it behind one narrow boundary and document why a stronger existing mechanism could not be used.",
                ),
                acceptance: String::from(
                    "The remaining parsing, validation, scheduler, or definition logic is small, explicit, and justified instead of spread across validators, resolvers, normalizers, registries, executors, commands, definition services, or helper layers.",
                ),
            },
        ],
        "split_identity_model" => vec![
            GuardianObligation {
                action: format!(
                    "Pick `{}` as the canonical representation for the concept family centered on `{primary_file}`.",
                    preferred_mechanism.unwrap_or("one canonical domain contract")
                ),
                acceptance: String::from(
                    "Callers no longer need to translate between object, id, and alias forms for the same concept.",
                ),
            },
            GuardianObligation {
                action: String::from(
                    "Move unavoidable compatibility handling to a thin boundary adapter.",
                ),
                acceptance: String::from(
                    "Core domain code uses the canonical representation, and compatibility aliases are isolated at the edge.",
                ),
            },
        ],
        "compatibility_scar" => vec![
            GuardianObligation {
                action: format!(
                    "Reduce the normalization and translation load concentrated in `{primary_file}` by migrating callers to `{}`.",
                    preferred_mechanism.unwrap_or("one canonical contract")
                ),
                acceptance: String::from(
                    "The file is no longer a mandatory translation hotspot for multiple competing representations.",
                ),
            },
            GuardianObligation {
                action: String::from(
                    "Keep only the minimum compatibility shim needed for the migration window and make the exit path explicit.",
                ),
                acceptance: String::from(
                    "Legacy aliases or fallback mappings are either removed or clearly temporary with an owner and end state.",
                ),
            },
        ],
        "duplicate_mechanism" => vec![
            GuardianObligation {
                action: format!(
                    "Choose `{}` for the concern centered on `{primary_file}` and route new behavior through it.",
                    preferred_mechanism.unwrap_or("one sanctioned orchestration path")
                ),
                acceptance: String::from(
                    "The concern no longer depends on parallel hooks, listeners, jobs, or direct notification paths for the same responsibility.",
                ),
            },
            GuardianObligation {
                action: String::from(
                    "Retire or isolate the duplicate pathways so the remaining flow is explainable from one primary mechanism.",
                ),
                acceptance: String::from(
                    "Agents and reviewers can point to one authoritative orchestration mechanism instead of reconciling overlapping routes.",
                ),
            },
        ],
        "sanctioned_path_bypass" => vec![
            GuardianObligation {
                action: format!(
                    "Move the raw environment reads in `{primary_file}` behind `{}`.",
                    preferred_mechanism.unwrap_or("the sanctioned configuration path used by the surrounding code")
                ),
                acceptance: String::from(
                    "The changed slice reads configuration through one approved path instead of mixing direct env access with config helpers or settings access.",
                ),
            },
            GuardianObligation {
                action: String::from(
                    "Isolate unavoidable bootstrap-time environment reads to a dedicated configuration boundary.",
                ),
                acceptance: String::from(
                    "Raw environment access no longer leaks into ordinary service, domain, or orchestration code.",
                ),
            },
        ],
        _ => Vec::new(),
    }
}

fn guardian_packet_suppressibility(_focus: &str) -> GuardianSuppressibility {
    GuardianSuppressibility {
        allowed: true,
        requires_reason: true,
        expiry_required: true,
    }
}

fn guardian_packet_questions(
    focus: &str,
    primary_file: &str,
    context_labels: &[String],
) -> Vec<String> {
    match focus {
        "security_hotspot" => {
            let externally_reachable = context_labels
                .iter()
                .any(|label| {
                    label == "externally_reachable" || label == "entry_reachable_via_graph"
                });
            let mut questions = vec![
                format!(
                    "What concrete inputs can reach dangerous primitives in `{primary_file}`, and through which route, hook, signal, or runtime entry?"
                ),
                format!(
                    "What is the canonical mitigation path for `{primary_file}`: safer API, stronger validation, narrower capability, or isolation behind a dedicated service?"
                ),
            ];
            if externally_reachable {
                questions.push(format!(
                    "Is `{primary_file}` protected by an actual trust boundary, or is this sink reachable without sufficient permission/auth checks?"
                ));
            } else {
                questions.push(format!(
                    "Is the dangerous primitive in `{primary_file}` operationally expected, or is it convenience logic that should be redesigned out of the path?"
                ));
            }
            questions
        }
        "algorithmic_complexity_hotspot" => vec![
            format!(
                "Which exact loop or repeated scan in `{primary_file}` is likely to grow superlinearly, and can its data be indexed, cached, or accumulated once instead?"
            ),
            format!(
                "Is the expensive work in `{primary_file}` truly on a small bounded input, or are we relying on optimistic scale assumptions that should be made explicit?"
            ),
        ],
        "compatibility_scar" => vec![
            format!(
                "Which representation in `{primary_file}` should be canonical, and which compatibility aliases or translation paths can be removed?"
            ),
            format!(
                "Why is `{primary_file}` owning this normalization logic instead of a thinner adapter or a single domain boundary?"
            ),
            format!(
                "What callers depend on the legacy forms handled in `{primary_file}`, and can they be migrated to one authoritative contract?"
            ),
        ],
        "abstraction_sprawl" => vec![
            format!(
                "Which abstraction in `{primary_file}` is the real boundary, and which surrounding helpers, managers, registries, or builders are only forwarding or renaming work?"
            ),
            format!(
                "Does the concern around `{primary_file}` truly need this many abstraction roles, or can the flow be simplified into one primary domain/service boundary?"
            ),
        ],
        "hand_rolled_parsing" => vec![
            format!(
                "What exact mini-language, query syntax, schema-validation contract, scheduler/job DSL, definition engine, or validator/resolver flow is `{primary_file}` implementing by hand, and does the framework or an existing library already solve it?"
            ),
            format!(
                "Can the parsing, validation, scheduling, or contract logic in `{primary_file}` be collapsed behind one sanctioned parser, scheduler, validator, metadata contract, or contract boundary instead of being spread across validators, resolvers, normalizers, registries, executors, definition services, or helpers?"
            ),
        ],
        "duplicate_mechanism" => vec![
            format!(
                "Which concern in `{primary_file}` is currently flowing through multiple orchestration mechanisms, and which one should remain authoritative?"
            ),
            format!(
                "Are the extra hooks, listeners, jobs, or direct notification paths in `{primary_file}` true requirements or historical leftovers that can be retired?"
            ),
        ],
        "sanctioned_path_bypass" => vec![
            format!(
                "Why is `{primary_file}` reading raw environment state directly when it also has access to a sanctioned configuration pathway?"
            ),
            format!(
                "Can the raw environment dependency in `{primary_file}` be moved to a bootstrap/config boundary so the rest of the code consumes one canonical configuration contract?"
            ),
        ],
        "split_identity_model" => vec![
            format!(
                "Which identifier or object form handled by `{primary_file}` is the real domain concept, and which competing forms are only legacy or transport baggage?"
            ),
            format!(
                "Can `{primary_file}` be changed so one canonical representation flows through the system instead of converting between parallel forms?"
            ),
        ],
        "warning_heavy_hotspot" => vec![
            format!(
                "Why is `{primary_file}` both central and noisy, and which responsibilities can be split so the graph pressure and finding mix decrease together?"
            ),
            format!(
                "Which findings in `{primary_file}` represent real architecture debt versus accepted framework mechanics that should move into policy or rules?"
            ),
        ],
        _ => Vec::new(),
    }
}

fn security_context_label(context: SecurityContext) -> String {
    match context {
        SecurityContext::ExternallyReachable => String::from("externally_reachable"),
        SecurityContext::EntryReachableViaGraph => String::from("entry_reachable_via_graph"),
        SecurityContext::BoundaryInputInSameFile => String::from("boundary_input_in_same_file"),
        SecurityContext::BoundaryInputReachableViaGraph => {
            String::from("boundary_input_reachable_via_graph")
        }
        SecurityContext::InteractiveExecution => String::from("interactive_execution"),
        SecurityContext::CacheStorage => String::from("cache_storage"),
        SecurityContext::DatabaseTooling => String::from("database_tooling"),
        SecurityContext::MigrationSupport => String::from("migration_support"),
        SecurityContext::DevelopmentRuntime => String::from("development_runtime"),
    }
}

pub fn write_architecture_surface_artifact(
    surface: &ArchitectureSurface,
    root: &Path,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(root));
    fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(ARCHITECTURE_SURFACE_FILE);
    write_json("architecture_surface", &path, surface)?;
    Ok(path)
}

pub fn write_semantic_graph_artifact(
    project: &SemanticGraphProject,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&project.root));
    fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(SEMANTIC_GRAPH_FILE);
    write_json_with_style(
        "semantic_graph",
        &path,
        &project.semantic_graph,
        JsonArtifactStyle::Compact,
    )?;
    Ok(path)
}

pub fn write_dependency_graph_artifact(
    project: &SemanticGraphProject,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&project.root));
    fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(DEPENDENCY_GRAPH_FILE);
    let dependency_graph = DependencyGraphJsonArtifact {
        root: &project.root,
        dependency_graph: build_dependency_graph_artifact(&project.semantic_graph),
    };
    write_json_with_style(
        "dependency_graph",
        &path,
        &dependency_graph,
        JsonArtifactStyle::Compact,
    )?;
    Ok(path)
}

pub fn write_evidence_graph_artifact(
    project: &SemanticGraphProject,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&project.root));
    fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(EVIDENCE_GRAPH_FILE);
    let evidence_graph = EvidenceGraphJsonArtifact {
        root: &project.root,
        evidence_graph: build_evidence_graph_artifact(&project.semantic_graph),
    };
    write_json_with_style(
        "evidence_graph",
        &path,
        &evidence_graph,
        JsonArtifactStyle::Compact,
    )?;
    Ok(path)
}

pub fn write_contract_inventory_artifact(
    analysis: &ProjectAnalysis,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&analysis.root));
    fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(CONTRACT_INVENTORY_FILE);
    write_json("contract_inventory", &path, &analysis.contract_inventory)?;
    Ok(path)
}

fn build_markdown_report(
    analysis: &ProjectAnalysis,
    report: &AigiscodeReportArtifact<'_>,
    handoff: &AgentHandoffArtifact,
) -> String {
    let AstGrepFamilyCounts {
        algorithmic_complexity: ast_grep_algorithmic_complexity_count,
        framework_misuse: ast_grep_framework_misuse_count,
        security_dangerous_api: ast_grep_security_dangerous_api_count,
    } = analysis.ast_grep_scan.family_counts();
    let ast_grep_skipped_preview = if report.summary.ast_grep_skipped_files_preview.is_empty() {
        None
    } else {
        Some(
            report
                .summary
                .ast_grep_skipped_files_preview
                .iter()
                .map(|file| {
                    format!(
                        "{} ({} bytes; {})",
                        file.file_path.display(),
                        file.bytes,
                        file.reason
                    )
                })
                .collect::<Vec<_>>()
                .join(", "),
        )
    };
    let mut lines = vec![
        String::from("# AigisCode Report"),
        String::new(),
        format!("- Root: `{}`", analysis.root.display()),
        format!("- Scanned files: {}", report.summary.scanned_files),
        format!("- Analyzed files: {}", report.summary.analyzed_files),
        format!("- Symbols: {}", report.summary.symbols),
        format!("- Resolved edges: {}", report.summary.resolved_edges),
        format!("- Strong cycles: {}", report.summary.strong_cycle_count),
        format!("- Total cycles: {}", report.summary.total_cycle_count),
        format!(
            "- Architectural smells: {} (hub-like: {}, unstable dependencies: {}, warning-heavy hotspots: {}, split identity models: {}, compatibility scars: {}, duplicate mechanisms: {}, sanctioned-path bypasses: {}, hand-rolled parsing: {}, abstraction sprawl: {})",
            report.summary.architectural_smell_count,
            report.summary.hub_like_dependency_count,
            report.summary.unstable_dependency_count,
            report.summary.warning_heavy_hotspot_count,
            report.summary.split_identity_model_count,
            report.summary.compatibility_scar_count,
            report.summary.duplicate_mechanism_count,
            report.summary.sanctioned_path_bypass_count,
            report.summary.hand_rolled_parsing_count,
            report.summary.abstraction_sprawl_count
        ),
        format!("- Dead code findings: {}", report.summary.dead_code_count),
        format!("- Hardwiring findings: {}", report.summary.hardwiring_count),
        format!(
            "- Native security findings: {}",
            report.summary.security_finding_count
        ),
        format!(
            "- Secondary scanner findings: {} (complexity: {}, security: {}, framework misuse: {}, skipped files: {}, skipped bytes: {})",
            report.summary.ast_grep_finding_count,
            ast_grep_algorithmic_complexity_count,
            ast_grep_security_dangerous_api_count,
            ast_grep_framework_misuse_count,
            report.summary.ast_grep_skipped_file_count,
            report.summary.ast_grep_skipped_bytes
        ),
        ast_grep_skipped_preview
            .map(|preview| format!("- Secondary scanner skipped preview: {}", preview))
            .unwrap_or_default(),
        format!(
            "- External findings: {}",
            report.summary.external_finding_count
        ),
        format!("- Visible findings: {}", report.summary.visible_findings),
        format!(
            "- Accepted by policy: {}",
            report.summary.accepted_by_policy
        ),
        format!(
            "- Suppressed by rule: {}",
            report.summary.suppressed_by_rule
        ),
        format!("- New findings vs previous run: {}", report.summary.new_findings),
        format!(
            "- Worsened findings vs previous run: {}",
            report.summary.worsened_findings
        ),
        format!(
            "- Improved findings vs previous run: {}",
            report.summary.improved_findings
        ),
        format!(
            "- Resolved findings vs previous run: {}",
            report.summary.resolved_findings
        ),
        format!(
            "- Repository topology: {} zones, {} manifests, {} cross-zone links, {} runtime entries",
            report.repository_topology.summary.zone_count,
            report.repository_topology.summary.manifest_count,
            report.repository_topology.summary.link_count,
            report.repository_topology.summary.runtime_entry_count
        ),
        String::new(),
        String::from("## Repository Topology"),
        String::new(),
        format!(
            "- Contract-bearing zones: {}",
            report.repository_topology.summary.contract_zone_count
        ),
        format!(
            "- Top zones: {}",
            report
                .repository_topology
                .zones
                .iter()
                .take(5)
                .map(|zone| format!(
                    "{} [{}]",
                    zone.path,
                    zone.labels.join(", ")
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!(
            "- Top cross-zone links: {}",
            report
                .repository_topology
                .links
                .iter()
                .take(5)
                .map(|link| format!(
                    "{} -> {} ({})",
                    link.source_zone, link.target_zone, link.relation_count
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        String::new(),
        String::from("## Convergence"),
        String::new(),
        format!(
            "- Contract delta: routes +{} / -{}, hooks +{} / -{}, registered keys +{} / -{}",
            report.convergence_history.contract_delta.routes.added_count,
            report.convergence_history.contract_delta.routes.removed_count,
            report.convergence_history.contract_delta.hooks.added_count,
            report.convergence_history.contract_delta.hooks.removed_count,
            report.convergence_history.contract_delta.registered_keys.added_count,
            report.convergence_history.contract_delta.registered_keys.removed_count
        ),
        format!(
            "- Graph delta: strong cycles {:+}, total cycles {:+}, bottlenecks {:+}, architectural smells {:+}, visible findings {:+}",
            report.convergence_history.graph_delta.strong_cycle_delta,
            report.convergence_history.graph_delta.total_cycle_delta,
            report.convergence_history.graph_delta.bottleneck_delta,
            report.convergence_history.graph_delta.architectural_smell_delta,
            report.convergence_history.graph_delta.visible_finding_delta
        ),
        format!(
            "- Attention items: {}",
            report.convergence_history.attention_items.len()
        ),
        format!(
            "- Required radius: anchors {}, one-hop files {}, inbound neighbors {}, outbound neighbors {}",
            report.convergence_history.required_radius.anchor_files.len(),
            report.convergence_history.required_radius.one_hop_files.len(),
            report.convergence_history.required_radius.inbound_neighbor_count,
            report.convergence_history.required_radius.outbound_neighbor_count
        ),
        if report
            .convergence_history
            .required_radius
            .anchor_files
            .is_empty()
        {
            String::from("- Radius anchors: none")
        } else {
            format!(
                "- Radius anchors: {}",
                report
                    .convergence_history
                    .required_radius
                    .anchor_files
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
        String::new(),
        String::from("## Guard Decision"),
        String::new(),
        format!(
            "- Verdict: {:?} (confidence {})",
            report.guard_decision.verdict, report.guard_decision.confidence_millis
        ),
        format!("- Summary: {}", report.guard_decision.summary),
        format!(
            "- Reasons: {}",
            if report.guard_decision.reasons.is_empty() {
                String::from("none")
            } else {
                report.guard_decision.reasons.join("; ")
            }
        ),
        format!(
            "- Triggers: {}",
            if report.guard_decision.triggers.is_empty() {
                String::from("none")
            } else {
                report
                    .guard_decision
                    .triggers
                    .iter()
                    .take(5)
                    .map(|trigger| format!(
                        "{:?}/{}/{}",
                        trigger.level, trigger.precision, trigger.message
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            }
        ),
        format!(
            "- Required radius: {} anchor file(s), {} one-hop neighboring file(s)",
            report.guard_decision.required_radius.anchor_files.len(),
            report.guard_decision.required_radius.one_hop_files.len()
        ),
        String::new(),
        String::from("## Contracts"),
        String::new(),
        format!(
            "- Routes: {} unique / {} occurrences",
            analysis.contract_inventory.summary.routes.unique_values,
            analysis.contract_inventory.summary.routes.occurrences
        ),
        format!(
            "- Hooks: {} unique / {} occurrences",
            analysis.contract_inventory.summary.hooks.unique_values,
            analysis.contract_inventory.summary.hooks.occurrences
        ),
        format!(
            "- Registered keys: {} unique / {} occurrences",
            analysis
                .contract_inventory
                .summary
                .registered_keys
                .unique_values,
            analysis
                .contract_inventory
                .summary
                .registered_keys
                .occurrences
        ),
        format!(
            "- Env keys: {} unique / {} occurrences",
            analysis.contract_inventory.summary.env_keys.unique_values,
            analysis.contract_inventory.summary.env_keys.occurrences
        ),
        format!(
            "- Config keys: {} unique / {} occurrences",
            analysis
                .contract_inventory
                .summary
                .config_keys
                .unique_values,
            analysis.contract_inventory.summary.config_keys.occurrences
        ),
        String::new(),
        String::from("## Feedback Loop"),
        String::new(),
        format!("- Detected total: {}", report.feedback_loop.detected_total),
        format!(
            "- Actionable visible: {}",
            report.feedback_loop.actionable_visible
        ),
        format!(
            "- Accepted by policy: {}",
            report.feedback_loop.accepted_by_policy
        ),
        format!(
            "- Suppressed by rule: {}",
            report.feedback_loop.suppressed_by_rule
        ),
        String::new(),
        String::from("## Next Steps"),
        String::new(),
    ];

    for step in &handoff.next_steps {
        lines.push(format!("- {step}"));
    }

    lines.push(String::new());
    lines.push(String::from("## Guardian Packets"));
    lines.push(String::new());

    if handoff.guardian_packets.is_empty() {
        lines.push(String::from("- No guardian packets."));
    } else {
        for packet in &handoff.guardian_packets {
            lines.push(format!(
                "- [{} / {} / {} / {}] {} (`{}`)",
                packet.focus,
                packet.priority,
                packet.precision,
                packet.confidence_millis,
                packet.summary,
                packet.primary_target_file
            ));
            if let Some(question) = packet.investigation_questions.first() {
                lines.push(format!("  - Investigate: {question}"));
            }
            if let Some(preferred_mechanism) = &packet.preferred_mechanism {
                lines.push(format!("  - Preferred mechanism: {preferred_mechanism}"));
            }
            if packet.doctrine_refs.is_empty().not() {
                lines.push(format!("  - Doctrine: {}", packet.doctrine_refs.join(", ")));
            }
            for obligation in &packet.obligations {
                lines.push(format!("  - Obligation: {}", obligation.action));
                lines.push(format!("    Acceptance: {}", obligation.acceptance));
            }
        }
    }

    lines.push(String::new());
    lines.push(String::from("## Top Visible Findings"));
    lines.push(String::new());

    if handoff.top_findings.is_empty() {
        lines.push(String::from("- No visible findings."));
    } else {
        for (phase, heading, empty_note) in [
            (
                "architecture",
                "### Architecture (judge the structure first)",
                "- No architecture-phase findings.",
            ),
            (
                "implementation",
                "### Implementation (bodies, after the structure is judged)",
                "- No implementation-phase findings in the top slice.",
            ),
        ] {
            lines.push(String::from(heading));
            lines.push(String::new());
            let mut any = false;
            for finding in handoff
                .top_findings
                .iter()
                .filter(|finding| finding.phase == phase)
            {
                any = true;
                let line_suffix = finding
                    .line
                    .map(|line| format!(" line {}", line))
                    .unwrap_or_default();
                let location = finding
                    .file_paths
                    .first()
                    .cloned()
                    .unwrap_or_else(|| String::from("unknown"));
                lines.push(format!(
                    "- [{} / {}] {}: {} (`{}`{})",
                    finding.family,
                    finding.severity,
                    finding.title,
                    finding.summary,
                    location,
                    line_suffix
                ));
            }
            if !any {
                lines.push(String::from(empty_note));
            }
            lines.push(String::new());
        }
    }

    lines.push(String::new());
    lines.push(String::from("## Timings"));
    lines.push(String::new());
    for timing in &analysis.timings {
        lines.push(format!("- {:?}: {} ms", timing.phase, timing.elapsed_ms));
    }

    lines.push(String::new());
    lines.join("\n")
}

fn serialize_json_pretty<T: Serialize>(value: &T, path: &Path) -> io::Result<Vec<u8>> {
    let mut payload = serde_json::to_vec_pretty(value).map_err(|error| {
        io::Error::other(format!("failed to serialize {}: {error}", path.display()))
    })?;
    payload.push(b'\n');
    Ok(payload)
}

fn write_json_payload(label: &str, path: &Path, payload: &[u8]) -> io::Result<()> {
    let started = Instant::now();
    fs::write(path, payload)?;
    trace_artifact_step(
        &format!("json.write.{label}"),
        started.elapsed().as_millis(),
    );
    Ok(())
}

fn write_json<T: Serialize>(label: &str, path: &Path, value: &T) -> io::Result<()> {
    write_json_with_style(label, path, value, JsonArtifactStyle::Pretty)
}

fn write_json_with_style<T: Serialize>(
    label: &str,
    path: &Path,
    value: &T,
    style: JsonArtifactStyle,
) -> io::Result<()> {
    let started = Instant::now();
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    match style {
        JsonArtifactStyle::Pretty => {
            serde_json::to_writer_pretty(&mut writer, value).map_err(|error| {
                io::Error::other(format!("failed to serialize {}: {error}", path.display()))
            })?;
        }
        JsonArtifactStyle::Compact => {
            serde_json::to_writer(&mut writer, value).map_err(|error| {
                io::Error::other(format!("failed to serialize {}: {error}", path.display()))
            })?;
        }
    }
    writer.write_all(b"\n")?;
    writer.flush()?;
    trace_artifact_step(
        &format!("json.write.{label}"),
        started.elapsed().as_millis(),
    );
    Ok(())
}

fn write_markdown(path: &Path, value: &str) -> io::Result<()> {
    let mut data = value.as_bytes().to_vec();
    data.push(b'\n');
    fs::write(path, data)
}

fn policy_error_to_io(error: PolicyLoadError) -> io::Error {
    io::Error::other(error)
}

fn doctrine_error_to_io(error: DoctrineLoadError) -> io::Error {
    io::Error::other(error)
}

#[cfg(test)]
mod tests {
    use super::{
        build_agent_handoff_artifact, build_guard_decision_artifact,
        build_topology_state_flow_lookup, build_topology_support_bridges,
        build_topology_targeted_support_bridges, convergence_focus, default_output_dir,
        select_diverse_guardian_packet_budget, write_architecture_surface_artifact,
        write_contract_inventory_artifact, write_dependency_graph_artifact,
        write_evidence_graph_artifact, write_project_analysis_artifacts,
        write_semantic_graph_artifact, ContractValueDelta, ConvergenceContractDelta,
        ConvergenceGraphDelta, ConvergenceHistoryArtifact, ConvergenceRequiredRadius,
        ConvergenceSummary, GuardVerdict, GuardianObligation, GuardianPacket,
        GuardianSuppressibility, RepositoryTopologyLinkedZone, AGENTIC_REVIEW_FILE,
        AGENT_HANDOFF_FILE, AIGISCODE_REPORT_FILE, AIGISCODE_REPORT_MARKDOWN_FILE,
        ARCHITECTURE_SURFACE_FILE, AST_GREP_SCAN_FILE, CONTRACT_INVENTORY_FILE,
        CONVERGENCE_HISTORY_FILE, DEPENDENCY_GRAPH_FILE, DETERMINISTIC_ANALYSIS_FILE,
        DETERMINISTIC_FINDINGS_FILE, EVIDENCE_GRAPH_FILE, EXTERNAL_ANALYSIS_FILE,
        GRAPH_PACKETS_FILE, GUARD_DECISION_FILE, REPOSITORY_TOPOLOGY_FILE, REVIEW_SURFACE_FILE,
        SEMANTIC_GRAPH_FILE,
    };
    use crate::agentic::{
        AgenticEvidenceLocation, AgenticGraphTrace, AgenticGraphTraceHop, AgenticGraphTraceKind,
        AgenticPrimaryEvidenceRefs, AgenticSemanticStateEvent, AgenticSemanticStateEventRole,
        AgenticSemanticStateFlow, AgenticSemanticStateFlowKind, GraphPacket, GraphPacketArtifact,
        GraphPacketKind, GraphPacketSummary, SemanticStateProofKind,
    };
    use crate::doctrine::{built_in_doctrine_registry, load_doctrine_registry};
    use crate::graph::SemanticGraph;
    use crate::graph::{ReferenceKind, ResolutionTier, ResolvedEdge};
    use crate::ingestion::pipeline::{analyze_project, build_semantic_graph_project};
    use crate::ingestion::scan::ScanConfig;
    use crate::policy::PolicyBundle;
    use crate::review::build_review_surface;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn convergence_focus_routes_complexity_findings_to_the_complexity_template() {
        let finding = |title: &str, family: crate::review::ReviewFindingFamily| {
            crate::review::ReviewFinding {
                id: String::from("finding-1"),
                fingerprint: String::from("fp-1"),
                family,
                phase: crate::surface::SurfaceFindingPhase::default(),
                severity: crate::review::ReviewFindingSeverity::Medium,
                precision: String::from("modeled"),
                confidence_millis: 800,
                title: String::from(title),
                summary: String::new(),
                file_paths: vec![String::from("app/Service.php")],
                line: None,
                primary_anchor: None,
                evidence_anchors: Vec::new(),
                locations: Vec::new(),
                supporting_context: Vec::new(),
                provenance: Vec::new(),
                doctrine_refs: Vec::new(),
                review_status: crate::review::ReviewStatus::Unreviewed,
                policy_status: crate::review::PolicyStatus::None,
                is_visible: true,
            }
        };
        assert_eq!(
            convergence_focus(&finding(
                "Algorithmic complexity hotspot",
                crate::review::ReviewFindingFamily::Graph,
            )),
            "algorithmic_complexity_hotspot"
        );
        assert_eq!(
            convergence_focus(&finding(
                "Repeated literal",
                crate::review::ReviewFindingFamily::Hardwiring,
            )),
            "warning_heavy_hotspot"
        );
    }

    #[test]
    fn topology_support_bridges_prefer_causal_paths_over_import_only_paths() {
        let linked_zones = vec![
            RepositoryTopologyLinkedZone {
                path: String::from("Filter"),
                direction: String::from("outbound"),
                relation_count: 20,
                relation_kinds: Vec::new(),
                examples: Vec::new(),
                support_paths: vec![AgenticGraphTrace {
                    id: String::from("graph-trace|criteria-filter"),
                    label: String::from("Criteria.php -> Filter/Filter.php"),
                    kind: AgenticGraphTraceKind::DirectedSupportPath,
                    primary_file_path: String::from("Criteria.php"),
                    supporting_file_path: Some(String::from("Filter/Filter.php")),
                    aggregate_confidence_millis: 900,
                    relation_sequence: vec![String::from("Import")],
                    truncated: false,
                    hops: vec![AgenticGraphTraceHop {
                        source_file_path: String::from("Criteria.php"),
                        source_symbol_id: None,
                        source_symbol_name: None,
                        target_file_path: String::from("Filter/Filter.php"),
                        target_symbol_id: None,
                        target_symbol_name: Some(String::from("Filter")),
                        relation_kind: String::from("Import"),
                        layer: String::from("structural"),
                        origin: String::from("resolved"),
                        strength: String::from("normal"),
                        resolution_tier: String::from("exact"),
                        line: 11,
                        confidence_millis: 900,
                        reason: String::from("import"),
                    }],
                }],
                linked_visible_finding_count: 0,
                linked_guardian_packet_count: 0,
                highest_visible_finding_severity: None,
                highest_guardian_packet_priority: None,
                triage_summary: String::new(),
            },
            RepositoryTopologyLinkedZone {
                path: String::from("Aggregation"),
                direction: String::from("outbound"),
                relation_count: 10,
                relation_kinds: Vec::new(),
                examples: Vec::new(),
                support_paths: vec![AgenticGraphTrace {
                    id: String::from("graph-trace|criteria-aggregation"),
                    label: String::from("Criteria.php -> Aggregation/Aggregation.php"),
                    kind: AgenticGraphTraceKind::DirectedSupportPath,
                    primary_file_path: String::from("Criteria.php"),
                    supporting_file_path: Some(String::from("Aggregation/Aggregation.php")),
                    aggregate_confidence_millis: 900,
                    relation_sequence: vec![String::from("Call")],
                    truncated: false,
                    hops: vec![AgenticGraphTraceHop {
                        source_file_path: String::from("Criteria.php"),
                        source_symbol_id: None,
                        source_symbol_name: Some(String::from("Criteria::hasEqualsFilter")),
                        target_file_path: String::from("Aggregation/Aggregation.php"),
                        target_symbol_id: None,
                        target_symbol_name: Some(String::from("Aggregation::getField")),
                        relation_kind: String::from("Call"),
                        layer: String::from("structural"),
                        origin: String::from("resolved"),
                        strength: String::from("normal"),
                        resolution_tier: String::from("exact"),
                        line: 19,
                        confidence_millis: 900,
                        reason: String::from("call"),
                    }],
                }],
                linked_visible_finding_count: 0,
                linked_guardian_packet_count: 0,
                highest_visible_finding_severity: None,
                highest_guardian_packet_priority: None,
                triage_summary: String::new(),
            },
        ];

        let bridges = build_topology_support_bridges(&linked_zones, 2);

        assert_eq!(bridges.len(), 2);
        assert_eq!(bridges[0].linked_zone_path, "Aggregation");
        assert_eq!(bridges[0].relation_sequence, vec![String::from("Call")]);
        assert_eq!(bridges[1].linked_zone_path, "Filter");
    }

    #[test]
    fn topology_state_flow_lookup_indexes_all_related_files() {
        let artifact = GraphPacketArtifact {
            root: String::from("/tmp/example"),
            contract_version: String::from("1"),
            summary: GraphPacketSummary {
                total_packets: 1,
                guardian_task_packets: 1,
                fallback_file_packets: 0,
                top_anchor_files: vec![String::from("Primary.php")],
            },
            packets: vec![GraphPacket {
                id: String::from("packet-1"),
                kind: GraphPacketKind::GuardianTask,
                title: String::from("Packet"),
                summary: String::from("Summary"),
                primary_file_path: String::from("Primary.php"),
                primary_anchor: None,
                evidence_anchors: Vec::new(),
                locations: Vec::new(),
                evidence_refs: AgenticPrimaryEvidenceRefs::default(),
                doctrine_refs: Vec::new(),
                preferred_mechanism: None,
                obligations: Vec::new(),
                relation_histogram: Vec::new(),
                neighbors: Vec::new(),
                graph_traces: Vec::new(),
                code_flows: Vec::new(),
                source_sink_paths: Vec::new(),
                semantic_state_flows: vec![AgenticSemanticStateFlow {
                    id: String::from(
                        "semantic-state|Carrier.php|Carrier|slot|direct_write_read|Writer.php|10|Reader.php|20",
                    ),
                    label: String::from("Carrier.slot: write -> read"),
                    carrier_type: String::from("Carrier"),
                    slot: String::from("slot"),
                    kind: AgenticSemanticStateFlowKind::DirectWriteRead,
                    writer: AgenticSemanticStateEvent {
                        file_path: String::from("Writer.php"),
                        symbol_name: Some(String::from("Writer::write")),
                        line: Some(10),
                        api: String::from("write"),
                        role: AgenticSemanticStateEventRole::Writer,
                        proof: SemanticStateProofKind::ExactResolved,
                        value_kind: None,
                        path: None,
                    },
                    reader: AgenticSemanticStateEvent {
                        file_path: String::from("Reader.php"),
                        symbol_name: Some(String::from("Reader::read")),
                        line: Some(20),
                        api: String::from("read"),
                        role: AgenticSemanticStateEventRole::Reader,
                        proof: SemanticStateProofKind::ExactResolved,
                        value_kind: None,
                        path: None,
                    },
                    aggregate_confidence_millis: 710,
                    supporting_locations: vec![AgenticEvidenceLocation {
                        role: String::from("carrier"),
                        file_path: String::from("Carrier.php"),
                        line: Some(5),
                    }],
                }],
            }],
        };

        let lookup = build_topology_state_flow_lookup(&artifact);
        for path in ["Primary.php", "Writer.php", "Reader.php", "Carrier.php"] {
            assert!(
                lookup.contains_key(path),
                "expected topology state-flow lookup to index {path}"
            );
            assert_eq!(lookup[path].len(), 1);
        }
    }

    #[test]
    fn topology_state_flow_lookup_preserves_distinct_shared_labels() {
        let semantic_state_flows = (0..7)
            .map(|index| AgenticSemanticStateFlow {
                id: format!(
                    "semantic-state|Carrier.php|Carrier|slot|direct_write_read|Writer{index}.php|{}|Reader{index}.php|{}",
                    index + 1,
                    index + 11
                ),
                label: String::from("Carrier.slot: write -> read"),
                carrier_type: String::from("Carrier"),
                slot: String::from("slot"),
                kind: AgenticSemanticStateFlowKind::DirectWriteRead,
                writer: AgenticSemanticStateEvent {
                    file_path: format!("Writer{index}.php"),
                    symbol_name: Some(format!("Writer{index}::write")),
                    line: Some(index + 1),
                    api: String::from("write"),
                    role: AgenticSemanticStateEventRole::Writer,
                    proof: SemanticStateProofKind::ExactResolved,
                    value_kind: None,
                    path: None,
                },
                reader: AgenticSemanticStateEvent {
                    file_path: format!("Reader{index}.php"),
                    symbol_name: Some(format!("Reader{index}::read")),
                    line: Some(index + 11),
                    api: String::from("read"),
                    role: AgenticSemanticStateEventRole::Reader,
                    proof: SemanticStateProofKind::ExactResolved,
                    value_kind: None,
                    path: None,
                },
                aggregate_confidence_millis: 600 + index as u16,
                supporting_locations: Vec::new(),
            })
            .collect::<Vec<_>>();
        let artifact = GraphPacketArtifact {
            root: String::from("/tmp/example"),
            contract_version: String::from("1"),
            summary: GraphPacketSummary {
                total_packets: 1,
                guardian_task_packets: 1,
                fallback_file_packets: 0,
                top_anchor_files: vec![String::from("Primary.php")],
            },
            packets: vec![GraphPacket {
                id: String::from("packet-1"),
                kind: GraphPacketKind::GuardianTask,
                title: String::from("Packet"),
                summary: String::from("Summary"),
                primary_file_path: String::from("Primary.php"),
                primary_anchor: None,
                evidence_anchors: Vec::new(),
                locations: Vec::new(),
                evidence_refs: AgenticPrimaryEvidenceRefs::default(),
                doctrine_refs: Vec::new(),
                preferred_mechanism: None,
                obligations: Vec::new(),
                relation_histogram: Vec::new(),
                neighbors: Vec::new(),
                graph_traces: Vec::new(),
                code_flows: Vec::new(),
                source_sink_paths: Vec::new(),
                semantic_state_flows,
            }],
        };

        let lookup = build_topology_state_flow_lookup(&artifact);
        assert_eq!(lookup["Primary.php"].len(), 7);
    }

    #[test]
    fn targeted_topology_support_bridges_follow_the_target_file() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("Parser/QueryStringParser.php"),
            None,
            PathBuf::from("Aggregation/Aggregation.php"),
            String::from("symbol:aggregation"),
            ReferenceKind::Call,
            ResolutionTier::ImportScoped,
            900,
            String::from("query-parser-call"),
            21,
        ));
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("Criteria.php"),
            None,
            PathBuf::from("Aggregation/Aggregation.php"),
            String::from("symbol:aggregation"),
            ReferenceKind::Import,
            ResolutionTier::ImportScoped,
            900,
            String::from("criteria-import"),
            8,
        ));

        let linked_zones = vec![RepositoryTopologyLinkedZone {
            path: String::from("Aggregation"),
            direction: String::from("outbound"),
            relation_count: 10,
            relation_kinds: Vec::new(),
            examples: vec![super::RepositoryTopologyCrossZoneExample {
                source_file: String::from("Parser/QueryStringParser.php"),
                target_file: String::from("Aggregation/Aggregation.php"),
                relation: String::from("call"),
            }],
            support_paths: vec![AgenticGraphTrace {
                id: String::from("graph-trace|criteria-aggregation-example"),
                label: String::from("Criteria.php -> Aggregation/Aggregation.php"),
                kind: AgenticGraphTraceKind::DirectedSupportPath,
                primary_file_path: String::from("Criteria.php"),
                supporting_file_path: Some(String::from("Aggregation/Aggregation.php")),
                aggregate_confidence_millis: 900,
                relation_sequence: vec![String::from("Import")],
                truncated: false,
                hops: vec![AgenticGraphTraceHop {
                    source_file_path: String::from("Criteria.php"),
                    source_symbol_id: None,
                    source_symbol_name: Some(String::from("Criteria")),
                    target_file_path: String::from("Aggregation/Aggregation.php"),
                    target_symbol_id: None,
                    target_symbol_name: Some(String::from("Aggregation::getField")),
                    relation_kind: String::from("Import"),
                    layer: String::from("structural"),
                    origin: String::from("resolved"),
                    strength: String::from("normal"),
                    resolution_tier: String::from("exact"),
                    line: 8,
                    confidence_millis: 900,
                    reason: String::from("import"),
                }],
            }],
            linked_visible_finding_count: 0,
            linked_guardian_packet_count: 0,
            highest_visible_finding_severity: None,
            highest_guardian_packet_priority: None,
            triage_summary: String::new(),
        }];

        let graph_context = crate::agentic::build_graph_query_context(&graph);
        let mut trace_cache = HashMap::new();
        let bridges = build_topology_targeted_support_bridges(
            &graph_context,
            &mut trace_cache,
            "Parser/QueryStringParser.php",
            &linked_zones,
            2,
        );

        assert_eq!(bridges.len(), 1);
        assert_eq!(bridges[0].linked_zone_path, "Aggregation");
        assert_eq!(bridges[0].relation_sequence, vec![String::from("Call")]);
        assert_eq!(bridges[0].source_file, "Parser/QueryStringParser.php");
    }

    #[test]
    fn writes_project_analysis_artifact_family() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod models;
use crate::models::Repo;
use crate::models::User;
fn main() {
    let status = "draft";
    let user = User;
    let _ = user;
    let _ = status;
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/models.rs"),
            b"pub struct User;\npub struct Repo;\n",
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let output_dir = fixture.join("artifacts");
        let paths = write_project_analysis_artifacts(&analysis, Some(&output_dir)).unwrap();

        assert_eq!(paths.output_dir, output_dir);
        assert!(paths
            .deterministic_analysis
            .ends_with(DETERMINISTIC_ANALYSIS_FILE));
        assert!(paths.semantic_graph.ends_with(SEMANTIC_GRAPH_FILE));
        assert!(paths.dependency_graph.ends_with(DEPENDENCY_GRAPH_FILE));
        assert!(paths.evidence_graph.ends_with(EVIDENCE_GRAPH_FILE));
        assert!(paths.contract_inventory.ends_with(CONTRACT_INVENTORY_FILE));
        assert!(paths
            .deterministic_findings
            .ends_with(DETERMINISTIC_FINDINGS_FILE));
        assert!(paths.ast_grep_scan.ends_with(AST_GREP_SCAN_FILE));
        assert!(paths.external_analysis.ends_with(EXTERNAL_ANALYSIS_FILE));
        assert!(paths
            .architecture_surface
            .ends_with(ARCHITECTURE_SURFACE_FILE));
        assert!(paths.review_surface.ends_with(REVIEW_SURFACE_FILE));
        assert!(paths
            .convergence_history
            .ends_with(CONVERGENCE_HISTORY_FILE));
        assert!(paths.guard_decision.ends_with(GUARD_DECISION_FILE));
        assert!(paths.agent_handoff.ends_with(AGENT_HANDOFF_FILE));
        assert!(paths.agentic_review.ends_with(AGENTIC_REVIEW_FILE));
        assert!(paths.graph_packets.ends_with(GRAPH_PACKETS_FILE));
        assert!(paths
            .repository_topology
            .ends_with(REPOSITORY_TOPOLOGY_FILE));
        assert!(paths.aigiscode_report.ends_with(AIGISCODE_REPORT_FILE));
        assert!(paths
            .aigiscode_report_markdown
            .ends_with(AIGISCODE_REPORT_MARKDOWN_FILE));

        let findings_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.deterministic_findings).unwrap())
                .unwrap();
        assert_eq!(findings_payload["scanned_files"], 2);
        assert!(findings_payload["resolved_edges"].as_u64().unwrap() >= 1);
        assert!(findings_payload["hardwiring"]["findings"]
            .as_array()
            .is_some());
        assert!(findings_payload["architectural_assessment"]["findings"]
            .as_array()
            .is_some());
        assert!(
            findings_payload["contract_inventory"]["summary"]["routes"]["unique_values"]
                .as_u64()
                .is_some()
        );
        let ast_grep_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.ast_grep_scan).unwrap()).unwrap();
        assert_eq!(ast_grep_payload["scanner"], Value::from("ast_grep"));

        let report_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.aigiscode_report).unwrap()).unwrap();
        assert_eq!(report_payload["summary"]["scanned_files"], 2);
        assert!(report_payload["summary"]["ast_grep_finding_count"]
            .as_u64()
            .is_some());
        assert!(
            report_payload["summary"]["ast_grep_algorithmic_complexity_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            report_payload["summary"]["ast_grep_security_dangerous_api_count"]
                .as_u64()
                .is_some()
        );
        assert!(report_payload["summary"]["ast_grep_framework_misuse_count"]
            .as_u64()
            .is_some());
        assert!(report_payload["summary"]["ast_grep_skipped_file_count"]
            .as_u64()
            .is_some());
        assert!(report_payload["summary"]["ast_grep_skipped_bytes"]
            .as_u64()
            .is_some());
        assert!(report_payload["summary"]["ast_grep_skipped_files_preview"].is_array());
        assert!(
            report_payload["architecture_surface"]["overview"]["resolved_edges"]
                .as_u64()
                .unwrap()
                >= 1
        );
        assert_eq!(
            report_payload["review_surface"]["summary"]["accepted_by_policy"],
            0
        );
        assert_eq!(
            report_payload["feedback_loop"]["detected_total"]
                .as_u64()
                .unwrap(),
            report_payload["review_surface"]["summary"]["total_findings"]
                .as_u64()
                .unwrap()
        );
        assert!(report_payload["summary"]["warning_heavy_hotspot_count"]
            .as_u64()
            .is_some());
        assert!(report_payload["agent_handoff"]["top_findings"]
            .as_array()
            .is_some());
        assert!(
            report_payload["agentic_review"]["transport"]["recommended_protocol"]
                .as_str()
                .is_some()
        );
        assert!(report_payload["graph_packets"]["packets"]
            .as_array()
            .is_some());
        assert!(report_payload["repository_topology"]["zones"]
            .as_array()
            .is_some());
        assert!(
            report_payload["convergence_history"]["summary"]["new_findings"]
                .as_u64()
                .is_some()
        );
        assert!(report_payload["guard_decision"]["verdict"]
            .as_str()
            .is_some());
        assert!(
            report_payload["contract_inventory"]["summary"]["routes"]["unique_values"]
                .as_u64()
                .is_some()
        );

        let convergence_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.convergence_history).unwrap()).unwrap();
        assert!(convergence_payload["summary"]["new_findings"]
            .as_u64()
            .is_some());
        assert!(convergence_payload["graph_delta"]["strong_cycle_delta"]
            .as_i64()
            .is_some());

        let guard_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.guard_decision).unwrap()).unwrap();
        assert!(guard_payload["verdict"].as_str().is_some());
        assert!(guard_payload["pressure"].is_object());
        assert!(guard_payload["triggers"].as_array().is_some());
        assert!(guard_payload["pressure"]["required_radius_anchor_files"]
            .as_u64()
            .is_some());
        assert!(guard_payload["pressure"]["required_radius_one_hop_files"]
            .as_u64()
            .is_some());

        let topology_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.repository_topology).unwrap()).unwrap();
        assert!(topology_payload["generated_at_unix_ms"].as_u64().is_some());
        assert!(topology_payload["summary"]["zone_count"].as_u64().unwrap() >= 1);
        assert!(
            topology_payload["summary"]["linked_high_severity_visible_finding_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["summary"]["linked_high_priority_guardian_packet_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["summary"]["linked_semantic_state_flow_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["summary"]["zones_with_cross_zone_pressure"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["summary"]["cross_zone_pressure_link_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["summary"]["strongest_cross_zone_relation_count"]
                .as_u64()
                .is_some()
        );
        assert!(topology_payload["summary"]["baseline_observation"].is_string());
        assert!(
            topology_payload["summary"]["recommended_start"].is_object()
                || topology_payload["summary"]["recommended_start"].is_null()
        );
        assert!(topology_payload["zones"].as_array().is_some());
        assert!(topology_payload["runtime_entry_files"].as_array().is_some());
        assert!(topology_payload["zones"][0]["spillover_observation"].is_string());
        assert!(
            topology_payload["zones"][0]["inbound_cross_zone_link_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["outbound_cross_zone_link_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["inbound_cross_zone_relation_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["outbound_cross_zone_relation_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["highest_visible_finding_severity"].is_string()
                || topology_payload["zones"][0]["highest_visible_finding_severity"].is_null()
        );
        assert!(
            topology_payload["zones"][0]["highest_guardian_packet_priority"].is_string()
                || topology_payload["zones"][0]["highest_guardian_packet_priority"].is_null()
        );
        assert!(topology_payload["zones"][0]["semantic_state_flow_count"]
            .as_u64()
            .is_some());
        assert!(
            topology_payload["zones"][0]["semantic_state_flow_proof_summary"]
                ["exact_resolved_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["semantic_state_flow_proof_summary"]
                ["receiver_typed_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["semantic_state_flow_proof_summary"]["heuristic_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["visible_finding_previews"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["visible_finding_previews"].is_null()
        );
        assert!(
            topology_payload["zones"][0]["guardian_packet_previews"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["guardian_packet_previews"].is_null()
        );
        assert!(
            topology_payload["zones"][0]["semantic_state_flow_previews"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["semantic_state_flow_previews"].is_null()
        );
        assert!(topology_payload["zones"][0]["triage_summary"].is_string());
        assert!(topology_payload["zones"][0]["cross_zone_pressure_summary"].is_string());
        assert!(topology_payload["zones"][0]["triage_next_steps"]
            .as_array()
            .is_some());
        assert!(topology_payload["zones"][0]["triage_steps"]
            .as_array()
            .is_some());
        assert!(
            topology_payload["zones"][0]["triage_steps"][0]["freshness"].is_string()
                || topology_payload["zones"][0]["triage_steps"][0].is_null()
        );
        assert!(
            topology_payload["zones"][0]["triage_steps"][0]["finding_status_summary"].is_object()
                || topology_payload["zones"][0]["triage_steps"][0].is_null()
        );
        assert!(topology_payload["zones"][0]["finding_status_summary"].is_object());
        assert!(
            topology_payload["zones"][0]["owner_hints"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["owner_hints"].is_null()
        );
        assert!(topology_payload["zones"][0]["owner_hint_confidence"].is_string());
        assert!(
            topology_payload["zones"][0]["owner_hint_basis"]["sampled_file_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["owner_hint_basis"]["observed_file_count"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["owner_hint_basis"]["total_observations"]
                .as_u64()
                .is_some()
        );
        assert!(
            topology_payload["zones"][0]["owner_hint_basis"]["unique_contributor_count"]
                .as_u64()
                .is_some()
        );
        assert!(topology_payload["zones"][0]["focus_clusters"]
            .as_array()
            .is_some());
        assert!(
            topology_payload["zones"][0]["focus_clusters"][0]["freshness"].is_string()
                || topology_payload["zones"][0]["focus_clusters"][0].is_null()
        );
        assert!(
            topology_payload["zones"][0]["focus_clusters"][0]["finding_status_summary"].is_object()
                || topology_payload["zones"][0]["focus_clusters"][0].is_null()
        );
        assert!(
            topology_payload["zones"][0]["linked_zones"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["linked_zones"].is_null()
        );
        assert!(
            topology_payload["zones"][0]["linked_zones"][0]["examples"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["linked_zones"][0].is_null()
        );
        assert!(
            topology_payload["zones"][0]["linked_zones"][0]["support_paths"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["linked_zones"][0].is_null()
        );
        assert!(
            topology_payload["links"][0]["examples"]
                .as_array()
                .is_some()
                || topology_payload["links"][0].is_null()
        );
        assert!(
            topology_payload["zones"][0]["visible_finding_previews"][0]["artifact_refs"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["visible_finding_previews"][0]["artifact_refs"]
                    .is_null()
        );
        assert!(
            topology_payload["zones"][0]["guardian_packet_previews"][0]["artifact_refs"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["guardian_packet_previews"][0]["artifact_refs"]
                    .is_null()
        );
        assert!(
            topology_payload["zones"][0]["triage_steps"][0]["artifact_refs"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["triage_steps"][0]["artifact_refs"].is_null()
        );
        assert!(
            topology_payload["zones"][0]["triage_steps"][0]["causal_bridges"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["triage_steps"][0]["causal_bridges"].is_null()
        );
        assert!(
            topology_payload["zones"][0]["triage_steps"][0]["semantic_state_flow_labels"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["triage_steps"][0]["semantic_state_flow_labels"]
                    .is_null()
        );
        assert!(
            topology_payload["zones"][0]["triage_steps"][0]["semantic_state_flow_refs"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["triage_steps"][0]["semantic_state_flow_refs"]
                    .is_null()
        );
        assert!(
            topology_payload["zones"][0]["triage_steps"][0]["semantic_state_flow_proof_summary"]
                ["heuristic_count"]
                .as_u64()
                .is_some()
                || topology_payload["zones"][0]["triage_steps"][0].is_null()
        );
        assert!(
            topology_payload["zones"][0]["focus_clusters"][0]["causal_bridges"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["focus_clusters"][0]["causal_bridges"].is_null()
        );
        assert!(
            topology_payload["zones"][0]["focus_clusters"][0]["semantic_state_flow_labels"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["focus_clusters"][0]["semantic_state_flow_labels"]
                    .is_null()
        );
        assert!(
            topology_payload["zones"][0]["focus_clusters"][0]["semantic_state_flow_refs"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["focus_clusters"][0]["semantic_state_flow_refs"]
                    .is_null()
        );
        assert!(
            topology_payload["zones"][0]["focus_clusters"][0]["semantic_state_flow_proof_summary"]
                ["receiver_typed_count"]
                .as_u64()
                .is_some()
                || topology_payload["zones"][0]["focus_clusters"][0].is_null()
        );
        assert!(
            topology_payload["zones"][0]["causal_bridges"]
                .as_array()
                .is_some()
                || topology_payload["zones"][0]["causal_bridges"].is_null()
        );
        assert!(
            topology_payload["zones"][0]["semantic_state_flow_previews"][0]["proof_tier"]
                .is_string()
                || topology_payload["zones"][0]["semantic_state_flow_previews"][0].is_null()
        );
        assert!(
            topology_payload["zones"][0]["semantic_state_flow_previews"][0]["id"].is_string()
                || topology_payload["zones"][0]["semantic_state_flow_previews"][0].is_null()
        );

        let dependency_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.dependency_graph).unwrap()).unwrap();
        assert_eq!(
            dependency_payload["dependency_graph"]["view"],
            Value::String(String::from("dependency_view"))
        );
        assert!(dependency_payload["dependency_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|node| node["kind"] != Value::String(String::from("MODULE"))));

        let evidence_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.evidence_graph).unwrap()).unwrap();
        assert_eq!(
            evidence_payload["evidence_graph"]["view"],
            Value::String(String::from("evidence_view"))
        );
        assert!(evidence_payload["evidence_graph"]["edges"]
            .as_array()
            .is_some());

        let contract_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.contract_inventory).unwrap()).unwrap();
        assert!(contract_payload["summary"]["routes"]["unique_values"]
            .as_u64()
            .is_some());

        let handoff_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.agent_handoff).unwrap()).unwrap();
        assert_eq!(
            handoff_payload["summary"]["scanned_files"]
                .as_u64()
                .unwrap(),
            2
        );
        assert!(handoff_payload["next_steps"].as_array().is_some());
        let guardian_packets = handoff_payload["guardian_packets"].as_array().unwrap();
        if let Some(first_packet) = guardian_packets.first() {
            assert!(first_packet["primary_target_file"].as_str().is_some());
            assert!(first_packet["precision"].as_str().is_some());
            assert!(first_packet["confidence_millis"].as_u64().is_some());
            assert!(first_packet["provenance"].as_array().is_some());
            assert!(first_packet["doctrine_refs"].as_array().is_some());
            assert!(first_packet["obligations"].as_array().is_some());
            assert!(first_packet["suppressibility"].is_object());
        }

        let markdown_report = fs::read_to_string(paths.aigiscode_report_markdown).unwrap();
        assert!(markdown_report.contains("# AigisCode Report"));
        assert!(markdown_report.contains("## Guard Decision"));
        assert!(markdown_report.contains("- Triggers:"));
        assert!(markdown_report.contains("## Guardian Packets"));
        assert!(markdown_report.contains("## Top Visible Findings"));
        assert!(markdown_report.contains("## Timings"));
    }

    #[test]
    fn report_and_surface_propagate_ast_grep_prefilter_skips() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/app.ts"), "export const answer = 42;\n").unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let output_dir = fixture.join("artifacts");
        let paths = write_project_analysis_artifacts(&analysis, Some(&output_dir)).unwrap();

        let ast_grep_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.ast_grep_scan).unwrap()).unwrap();
        assert_eq!(
            ast_grep_payload["skipped_files"][0]["reason"],
            Value::from("no_family_prefilter_hit")
        );

        let report_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.aigiscode_report).unwrap()).unwrap();
        assert_eq!(report_payload["summary"]["ast_grep_skipped_file_count"], 1);
        assert_eq!(report_payload["summary"]["ast_grep_skipped_bytes"], 26);
        assert_eq!(
            report_payload["summary"]["ast_grep_skipped_files_preview"][0]["file_path"],
            Value::from("src/app.ts")
        );

        let surface_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.architecture_surface).unwrap()).unwrap();
        assert_eq!(
            surface_payload["overview"]["ast_grep_skipped_file_count"],
            Value::from(1)
        );
        assert_eq!(
            surface_payload["overview"]["ast_grep_skipped_bytes"],
            Value::from(26)
        );
        assert_eq!(
            surface_payload["overview"]["ast_grep_skipped_files_preview"][0]["reason"],
            Value::from("no_family_prefilter_hit")
        );
    }

    #[test]
    fn convergence_history_tracks_previous_run_deltas() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::create_dir_all(fixture.join("routes")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn main() {
    if status == "draft" {
        let _ = status;
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("routes/web.php"),
            br#"<?php Route::get('/users', 'UserController@index');"#,
        )
        .unwrap();

        let output_dir = fixture.join("artifacts");
        let first = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let first_paths = write_project_analysis_artifacts(&first, Some(&output_dir)).unwrap();
        let first_convergence: Value =
            serde_json::from_str(&fs::read_to_string(&first_paths.convergence_history).unwrap())
                .unwrap();
        assert_eq!(
            first_convergence["summary"]["previous_findings"]
                .as_u64()
                .unwrap(),
            0
        );

        fs::write(
            fixture.join("src/main.rs"),
            br#"fn main() {
    if status == "draft" {
        let _ = status;
    }
    let url = "https://api.example.com";
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("routes/web.php"),
            br#"<?php Route::get('/users', 'UserController@index'); Route::post('/users/create', 'UserController@store');"#,
        )
        .unwrap();

        let second = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let second_paths = write_project_analysis_artifacts(&second, Some(&output_dir)).unwrap();
        let second_convergence: Value =
            serde_json::from_str(&fs::read_to_string(&second_paths.convergence_history).unwrap())
                .unwrap();
        assert!(
            second_convergence["summary"]["new_findings"]
                .as_u64()
                .unwrap()
                >= 1
        );
        assert!(second_convergence["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["status"] == Value::String(String::from("New"))));
        assert_eq!(
            second_convergence["contract_delta"]["routes"]["added_count"]
                .as_u64()
                .unwrap(),
            1
        );
        assert!(second_convergence["required_radius"].is_object());
        assert!(second_convergence["attention_items"].as_array().is_some());
        assert!(second_convergence["attention_items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["precision"].as_str().is_some()
                && item["confidence_millis"].as_u64().is_some()
                && item["provenance"].as_array().is_some()));
        let second_topology: Value =
            serde_json::from_str(&fs::read_to_string(&second_paths.repository_topology).unwrap())
                .unwrap();
        assert!(second_topology["zones"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|zone| zone["triage_steps"].as_array().into_iter().flatten())
            .any(|step| step["freshness"] == Value::String(String::from("new"))));
    }

    #[test]
    fn guard_decision_promotes_architectonic_regressions_into_triggers() {
        let convergence = ConvergenceHistoryArtifact {
            root: String::from("/tmp/example"),
            summary: ConvergenceSummary {
                current_findings: 0,
                previous_findings: 0,
                new_findings: 0,
                worsened_findings: 0,
                improved_findings: 0,
                unchanged_findings: 0,
                resolved_findings: 0,
            },
            graph_delta: ConvergenceGraphDelta {
                strong_cycle_delta: 0,
                total_cycle_delta: 0,
                bottleneck_delta: 0,
                architectural_smell_delta: 0,
                warning_heavy_hotspot_delta: 0,
                abstraction_sprawl_delta: 0,
                hand_rolled_parsing_delta: 0,
                split_identity_model_delta: 1,
                compatibility_scar_delta: 1,
                duplicate_mechanism_delta: 1,
                sanctioned_path_bypass_delta: 0,
                visible_finding_delta: 0,
                algorithmic_complexity_hotspot_delta: 0,
            },
            contract_delta: ConvergenceContractDelta {
                routes: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                hooks: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                registered_keys: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                symbolic_literals: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                env_keys: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                config_keys: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
            },
            required_investigation_files: Vec::new(),
            required_radius: ConvergenceRequiredRadius {
                anchor_files: Vec::new(),
                one_hop_files: Vec::new(),
                inbound_neighbor_count: 0,
                outbound_neighbor_count: 0,
            },
            attention_items: Vec::new(),
            findings: Vec::new(),
        };

        let guard = build_guard_decision_artifact(Path::new("/tmp/example"), &convergence);

        assert_eq!(guard.verdict, GuardVerdict::Warn);
        assert!(guard.pressure.split_identity_model_regression);
        assert!(guard.pressure.compatibility_scar_regression);
        assert!(guard.pressure.duplicate_mechanism_regression);
        let messages = guard
            .triggers
            .iter()
            .map(|trigger| trigger.message.as_str())
            .collect::<Vec<_>>();
        assert!(messages
            .iter()
            .any(|message| message.contains("Split identity model pressure increased")));
        assert!(messages
            .iter()
            .any(|message| message.contains("Compatibility-scar pressure increased")));
        assert!(messages
            .iter()
            .any(|message| message.contains("Duplicate-mechanism pressure increased")));
    }

    #[test]
    fn guard_decision_surfaces_algorithmic_complexity_regressions() {
        let convergence = ConvergenceHistoryArtifact {
            root: String::from("/tmp/example"),
            summary: ConvergenceSummary {
                current_findings: 0,
                previous_findings: 0,
                new_findings: 0,
                worsened_findings: 0,
                improved_findings: 0,
                unchanged_findings: 0,
                resolved_findings: 0,
            },
            graph_delta: ConvergenceGraphDelta {
                strong_cycle_delta: 0,
                total_cycle_delta: 0,
                bottleneck_delta: 0,
                architectural_smell_delta: 0,
                warning_heavy_hotspot_delta: 0,
                abstraction_sprawl_delta: 0,
                hand_rolled_parsing_delta: 0,
                split_identity_model_delta: 0,
                compatibility_scar_delta: 0,
                duplicate_mechanism_delta: 0,
                sanctioned_path_bypass_delta: 0,
                algorithmic_complexity_hotspot_delta: 2,
                visible_finding_delta: 0,
            },
            contract_delta: ConvergenceContractDelta {
                routes: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                hooks: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                registered_keys: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                symbolic_literals: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                env_keys: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                config_keys: ContractValueDelta {
                    added_count: 0,
                    removed_count: 0,
                    added: Vec::new(),
                    removed: Vec::new(),
                },
            },
            required_investigation_files: Vec::new(),
            required_radius: ConvergenceRequiredRadius {
                anchor_files: Vec::new(),
                one_hop_files: Vec::new(),
                inbound_neighbor_count: 0,
                outbound_neighbor_count: 0,
            },
            attention_items: Vec::new(),
            findings: Vec::new(),
        };

        let guard = build_guard_decision_artifact(Path::new("/tmp/example"), &convergence);

        assert_eq!(guard.verdict, GuardVerdict::Warn);
        assert!(guard.pressure.algorithmic_complexity_hotspot_regression);
        assert!(guard.triggers.iter().any(|trigger| trigger
            .message
            .contains("Algorithmic-complexity hotspot pressure increased")));
    }

    #[test]
    fn builds_guardian_packets_from_security_and_split_identity_signals() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app")).unwrap();
        fs::write(
            fixture.join("app/service.py"),
            br#"def run(command, assignedUser, assigned_user_id):
    exec(command)
    return assignedUser or assigned_user_id
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/model.py"),
            br#"class Ticket:
    def getAssignedUserId(self):
        return self.assigned_user_id

    def setAssignedUserId(self, assignedUserId):
        self.assignedUserId = assignedUserId
        return self.assignedUserId
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/consumer.py"),
            br#"def use(assignedUser, assigned_user_id, assignedUserId):
    return assignedUserId or assigned_user_id or assignedUser
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = analysis.architecture_surface();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let doctrine_registry = load_doctrine_registry(&fixture).unwrap();
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine_registry);
        let focuses = handoff
            .guardian_packets
            .iter()
            .map(|packet| packet.focus.as_str())
            .collect::<Vec<_>>();

        assert!(focuses.contains(&"security_hotspot"));
        assert!(focuses.contains(&"split_identity_model"));
        assert!(handoff
            .guardian_packets
            .iter()
            .all(|packet| !packet.investigation_questions.is_empty()));
        assert!(handoff
            .guardian_packets
            .iter()
            .all(|packet| !packet.precision.is_empty()));
        assert!(handoff
            .guardian_packets
            .iter()
            .all(|packet| !packet.provenance.is_empty()));
        assert!(handoff
            .guardian_packets
            .iter()
            .all(|packet| !packet.doctrine_refs.is_empty()));
        assert!(handoff
            .guardian_packets
            .iter()
            .all(|packet| packet.confidence_millis > 0));
        assert!(handoff.guardian_packets.iter().all(|packet| packet
            .preferred_mechanism
            .as_ref()
            .is_some_and(|value| !value.is_empty())));
        assert!(handoff
            .guardian_packets
            .iter()
            .all(|packet| !packet.obligations.is_empty()));
        assert!(handoff
            .guardian_packets
            .iter()
            .all(|packet| packet.suppressibility.allowed
                && packet.suppressibility.requires_reason
                && packet.suppressibility.expiry_required));
        assert!(handoff
            .guardian_packets
            .iter()
            .all(|packet| packet.target_files.contains(&packet.primary_target_file)));
        let split_packet = handoff
            .guardian_packets
            .iter()
            .find(|packet| packet.focus == "split_identity_model")
            .unwrap();
        assert!(split_packet.primary_anchor.is_some());
        let security_packet = handoff
            .guardian_packets
            .iter()
            .find(|packet| packet.focus == "security_hotspot")
            .unwrap();
        assert!(security_packet.primary_anchor.is_some());
    }

    #[test]
    fn guardian_packets_prefer_repo_defined_mechanisms_from_doctrine() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::create_dir_all(fixture.join("app/Services/Filter")).unwrap();
        fs::write(
            fixture.join(".aigiscode/doctrine.json"),
            r#"{
  "version": "repo-override-1",
  "clauses": [
    {
      "id": "guardian.native-vs-library",
      "title": "Repo native versus library",
      "description": "Use the sanctioned query contract parser.",
      "category": "MechanismChoice",
      "default_disposition": "Block",
      "preferred_mechanism": "query_contract_parser",
      "guidance": ["Use the sanctioned query contract parser."]
    }
  ]
}"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Services/Filter/QueryContractParser.php"),
            br#"<?php
final class QueryContractParser {
    public function parse(Request $request): array {
        $filters = json_decode($request->input('filters', '[]'), true);
        $parts = array_map(trim(...), explode(',', (string) $request->query('sort')));
        return $this->parseSort($parts);
    }

    private function parseSort(array $parts): array {
        return $parts;
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Services/Filter/FilterValidator.php"),
            br#"<?php
final class FilterValidator {
    public function validateOperator(string $operator): bool {
        return preg_match('/^[a-z_]+$/', $operator) === 1;
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Services/Filter/FilterDefinitionResolver.php"),
            br#"<?php
final class FilterDefinitionResolver {
    public function resolve(string $name): array {
        $normalized = trim($name);
        return ['key' => substr($normalized, 0, 10)];
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        assert_eq!(
            analysis
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::HandRolledParsing),
            1
        );
        let surface = analysis.architecture_surface();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let doctrine_registry = load_doctrine_registry(&fixture).unwrap();
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine_registry);

        let packet = handoff
            .guardian_packets
            .iter()
            .find(|packet| packet.focus == "hand_rolled_parsing")
            .unwrap();
        assert_eq!(
            packet.preferred_mechanism.as_deref(),
            Some("query_contract_parser")
        );
        assert!(packet
            .obligations
            .iter()
            .any(|obligation| obligation.action.contains("query_contract_parser")));
    }

    #[test]
    fn guardian_packets_keep_native_anchors_for_hand_rolled_parsing() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app/Services/Filter")).unwrap();
        fs::write(
            fixture.join("app/Services/Filter/QueryContractParser.php"),
            br#"<?php
final class QueryContractParser {
    public function parse(Request $request): array {
        $filters = json_decode($request->input('filters', '[]'), true);
        $parts = array_map(trim(...), explode(',', (string) $request->query('sort')));
        return $this->parseSort($parts);
    }

    private function parseSort(array $parts): array {
        return $parts;
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Services/Filter/FilterValidator.php"),
            br#"<?php
final class FilterValidator {
    public function validateOperator(string $operator): bool {
        return preg_match('/^[a-z_]+$/', $operator) === 1;
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Services/Filter/FilterDefinitionResolver.php"),
            br#"<?php
final class FilterDefinitionResolver {
    public function resolve(string $name): array {
        $normalized = trim($name);
        return ['key' => substr($normalized, 0, 10)];
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = analysis.architecture_surface();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let doctrine_registry = load_doctrine_registry(&fixture).unwrap();
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine_registry);

        let packet = handoff
            .guardian_packets
            .iter()
            .find(|packet| packet.focus == "hand_rolled_parsing")
            .unwrap();
        assert_eq!(
            packet
                .primary_anchor
                .as_ref()
                .map(|anchor| anchor.file_path.clone()),
            Some(PathBuf::from("app/Services/Filter/QueryContractParser.php"))
        );
        assert!(packet
            .primary_anchor
            .as_ref()
            .and_then(|anchor| anchor.line)
            .is_some());
        assert!(!packet.evidence_anchors.is_empty());
    }

    #[test]
    fn merges_algorithmic_complexity_packets_by_primary_file() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/hotspot.ts"),
            r#"
                export function scoreRows(rows: string[][], allowList: string[]): string[] {
                    const out: string[] = [];
                    for (const row of rows) {
                        for (const cell of row) {
                            if (allowList.includes(cell)) {
                                out.push(cell);
                            }
                        }
                        out.sort();
                    }
                    return out;
                }
            "#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = analysis.architecture_surface();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let doctrine_registry = load_doctrine_registry(&fixture).unwrap();
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine_registry);

        let packets = handoff
            .guardian_packets
            .iter()
            .filter(|packet| {
                packet.focus == "algorithmic_complexity_hotspot"
                    && packet.primary_target_file == "src/hotspot.ts"
            })
            .collect::<Vec<_>>();
        assert_eq!(packets.len(), 1);
        let packet = packets[0];
        assert!(packet
            .context_labels
            .iter()
            .any(|label| label == "complexity:nested_iteration"));
        assert!(packet
            .context_labels
            .iter()
            .any(|label| label == "complexity:collection_scan_in_loop"));
        assert!(packet
            .context_labels
            .iter()
            .any(|label| label == "complexity:sort_in_loop"));
        assert!(packet.finding_ids.len() >= 2);
        assert!(packet.summary.contains("nested_iteration"));
        assert!(packet.summary.contains("sort_in_loop"));
    }

    #[test]
    fn guardian_packet_budget_keeps_focus_diversity_before_overflow() {
        fn packet(id: &str, focus: &str, priority: &str) -> GuardianPacket {
            GuardianPacket {
                id: id.to_string(),
                priority: priority.to_string(),
                focus: focus.to_string(),
                primary_target_file: format!("{id}.rs"),
                precision: String::from("heuristic"),
                confidence_millis: 700,
                summary: id.to_string(),
                target_files: vec![format!("{id}.rs")],
                primary_anchor: None,
                evidence_anchors: Vec::new(),
                locations: Vec::new(),
                finding_ids: vec![id.to_string()],
                context_labels: Vec::new(),
                provenance: vec![String::from("test")],
                doctrine_refs: vec![String::from("test.doctrine")],
                preferred_mechanism: None,
                obligations: vec![GuardianObligation {
                    action: String::from("act"),
                    acceptance: String::from("accept"),
                }],
                suppressibility: GuardianSuppressibility {
                    allowed: true,
                    requires_reason: true,
                    expiry_required: true,
                },
                investigation_questions: vec![String::from("why")],
            }
        }

        let selected = select_diverse_guardian_packet_budget(
            vec![
                packet("alg-1", "algorithmic_complexity_hotspot", "high"),
                packet("alg-2", "algorithmic_complexity_hotspot", "high"),
                packet("warn-1", "warning_heavy_hotspot", "high"),
                packet("security-1", "security_hotspot", "high"),
            ],
            3,
        );

        let ids = selected
            .iter()
            .map(|packet| packet.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["alg-1", "warn-1", "security-1"]);
    }

    #[test]
    fn hand_rolled_scheduler_packets_prefer_specific_doctrine_mechanism() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app/Services/Settings")).unwrap();
        fs::create_dir_all(fixture.join("app/Services/Jobs")).unwrap();
        fs::create_dir_all(fixture.join("app/Console/Commands")).unwrap();
        fs::write(
            fixture.join("app/Services/Settings/JobRegistry.php"),
            br#"<?php
final class JobRegistry {
    public function getJobs(): array {
        $jobs = config('jobs.jobs', []);
        foreach ($this->moduleRegistry->getEnabledModules() as $module) {
            $jobs = array_merge($jobs, $module['manifest']['jobs'] ?? []);
        }
        return $jobs;
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Services/Jobs/ScheduledJobExecutor.php"),
            br#"<?php
final class ScheduledJobExecutor {
    public function execute(array $config): string {
        $exitCode = Artisan::call((string) ($config['command'] ?? ''));
        dispatch(new SyncTenantJob());
        return (string) $exitCode;
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Console/Commands/ErpScheduledJobsRunCommand.php"),
            br#"<?php
final class ErpScheduledJobsRunCommand {
    public function handle(): int {
        $expression = new CronExpression('* * * * *');
        if ($expression->isDue(now())) {
            Cache::lock('scheduled_job:tenant:sync', 900);
        }
        return 0;
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = analysis.architecture_surface();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let doctrine_registry = built_in_doctrine_registry();
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine_registry);

        let packet = handoff
            .guardian_packets
            .iter()
            .find(|packet| {
                packet.focus == "hand_rolled_parsing"
                    && packet.preferred_mechanism.as_deref() == Some("framework_scheduler_or_queue")
            })
            .expect("expected scheduler dsl guardian packet");
        assert_eq!(
            packet.preferred_mechanism.as_deref(),
            Some("framework_scheduler_or_queue")
        );
        assert!(packet.target_files.iter().any(|file| {
            file == "app/Services/Jobs/ScheduledJobExecutor.php"
                || file == "app/Services/Settings/JobRegistry.php"
                || file == "app/Console/Commands/ErpScheduledJobsRunCommand.php"
        }));
        assert!(packet
            .obligations
            .iter()
            .any(|obligation| obligation.action.contains("framework_scheduler_or_queue")));
    }

    #[test]
    fn defaults_to_repo_aigiscode_directory_for_surface_artifact() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn main() {
    let url = "https://api.example.com";
    let _ = url;
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = analysis.architecture_surface();
        let output_path = write_architecture_surface_artifact(&surface, &fixture, None).unwrap();

        assert_eq!(
            output_path,
            default_output_dir(&fixture).join(ARCHITECTURE_SURFACE_FILE)
        );
        assert!(output_path.exists());
    }

    #[test]
    fn writes_semantic_graph_artifact_without_full_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod models;
use crate::models::User;
fn main() {
    let _ = User {};
}
"#,
        )
        .unwrap();
        fs::write(fixture.join("src/models.rs"), b"pub struct User;\n").unwrap();

        let graph_project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let output_path = write_semantic_graph_artifact(&graph_project, None).unwrap();

        assert_eq!(
            output_path,
            default_output_dir(&fixture).join(SEMANTIC_GRAPH_FILE)
        );
        let payload: Value =
            serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();
        assert!(payload["resolved_edges"].as_array().is_some());
    }

    #[test]
    fn writes_dependency_graph_artifact_without_full_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod models;
use crate::models::User;
fn main() {
    let _ = User {};
}
"#,
        )
        .unwrap();
        fs::write(fixture.join("src/models.rs"), b"pub struct User;\n").unwrap();

        let graph_project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let output_path = write_dependency_graph_artifact(&graph_project, None).unwrap();

        assert_eq!(
            output_path,
            default_output_dir(&fixture).join(DEPENDENCY_GRAPH_FILE)
        );
        let payload: Value =
            serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();
        assert_eq!(
            payload["dependency_graph"]["view"],
            Value::String(String::from("dependency_view"))
        );
        assert!(payload["dependency_graph"]["edges"].as_array().is_some());
    }

    #[test]
    fn writes_evidence_graph_artifact_without_full_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod models;
use crate::models::User;
fn main() {
    let _ = User {};
}
"#,
        )
        .unwrap();
        fs::write(fixture.join("src/models.rs"), b"pub struct User;\n").unwrap();

        let graph_project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let output_path = write_evidence_graph_artifact(&graph_project, None).unwrap();

        assert_eq!(
            output_path,
            default_output_dir(&fixture).join(EVIDENCE_GRAPH_FILE)
        );
        let payload: Value =
            serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();
        assert_eq!(
            payload["evidence_graph"]["view"],
            Value::String(String::from("evidence_view"))
        );
        assert!(payload["evidence_graph"]["edges"].as_array().is_some());
    }

    #[test]
    fn writes_contract_inventory_artifact_from_project_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("routes")).unwrap();
        fs::write(
            fixture.join("routes/web.php"),
            br#"<?php Route::get('/users', 'UserController@index');"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let output_path = write_contract_inventory_artifact(&analysis, None).unwrap();

        assert_eq!(
            output_path,
            default_output_dir(&fixture).join(CONTRACT_INVENTORY_FILE)
        );
        let payload: Value =
            serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();
        assert_eq!(payload["summary"]["routes"]["unique_values"], 1);
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-artifacts-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
