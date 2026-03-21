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
use crate::security::{SecurityAnalysisResult, SecurityContext};
use crate::surface::ArchitectureSurface;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io;
use std::ops::Not;
use std::path::{Path, PathBuf};

pub const DEFAULT_OUTPUT_DIR_NAME: &str = ".aigiscode";
pub const DETERMINISTIC_ANALYSIS_FILE: &str = "deterministic-analysis.json";
pub const SEMANTIC_GRAPH_FILE: &str = "semantic-graph.json";
pub const DEPENDENCY_GRAPH_FILE: &str = "dependency-graph.json";
pub const EVIDENCE_GRAPH_FILE: &str = "evidence-graph.json";
pub const CONTRACT_INVENTORY_FILE: &str = "contract-inventory.json";
pub const DOCTRINE_REGISTRY_FILE: &str = "doctrine-registry.json";
pub const DETERMINISTIC_FINDINGS_FILE: &str = "deterministic-findings.json";
pub const EXTERNAL_ANALYSIS_FILE: &str = "external-analysis.json";
pub const ARCHITECTURE_SURFACE_FILE: &str = "architecture-surface.json";
pub const REVIEW_SURFACE_FILE: &str = "review-surface.json";
pub const CONVERGENCE_HISTORY_FILE: &str = "convergence-history.json";
pub const GUARD_DECISION_FILE: &str = "guard-decision.json";
pub const AGENT_HANDOFF_FILE: &str = "aigiscode-handoff.json";
pub const AIGISCODE_REPORT_FILE: &str = "aigiscode-report.json";
pub const AIGISCODE_REPORT_MARKDOWN_FILE: &str = "aigiscode-report.md";

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
    pub external_analysis: PathBuf,
    pub architecture_surface: PathBuf,
    pub review_surface: PathBuf,
    pub convergence_history: PathBuf,
    pub guard_decision: PathBuf,
    pub agent_handoff: PathBuf,
    pub aigiscode_report: PathBuf,
    pub aigiscode_report_markdown: PathBuf,
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
    pub severity: String,
    pub title: String,
    pub summary: String,
    pub file_paths: Vec<String>,
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_anchor: Option<EvidenceAnchor>,
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
    pub finding_ids: Vec<String>,
    pub context_labels: Vec<String>,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
    pub preferred_mechanism: Option<String>,
    pub obligations: Vec<GuardianObligation>,
    pub suppressibility: GuardianSuppressibility,
    pub investigation_questions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

pub fn write_project_analysis_artifacts(
    analysis: &ProjectAnalysis,
    output_dir: Option<&Path>,
) -> io::Result<ArtifactPaths> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&analysis.root));
    fs::create_dir_all(&output_dir)?;

    let paths = ArtifactPaths {
        deterministic_analysis: output_dir.join(DETERMINISTIC_ANALYSIS_FILE),
        semantic_graph: output_dir.join(SEMANTIC_GRAPH_FILE),
        dependency_graph: output_dir.join(DEPENDENCY_GRAPH_FILE),
        evidence_graph: output_dir.join(EVIDENCE_GRAPH_FILE),
        contract_inventory: output_dir.join(CONTRACT_INVENTORY_FILE),
        doctrine_registry: output_dir.join(DOCTRINE_REGISTRY_FILE),
        deterministic_findings: output_dir.join(DETERMINISTIC_FINDINGS_FILE),
        external_analysis: output_dir.join(EXTERNAL_ANALYSIS_FILE),
        architecture_surface: output_dir.join(ARCHITECTURE_SURFACE_FILE),
        review_surface: output_dir.join(REVIEW_SURFACE_FILE),
        convergence_history: output_dir.join(CONVERGENCE_HISTORY_FILE),
        guard_decision: output_dir.join(GUARD_DECISION_FILE),
        agent_handoff: output_dir.join(AGENT_HANDOFF_FILE),
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
        security_analysis: &analysis.security_analysis,
        contract_inventory: &analysis.contract_inventory,
        timings: &analysis.timings,
    };
    let surface = analysis.architecture_surface();
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
    let review_surface = build_review_surface(analysis, &surface, &policy_bundle);
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
    let guard_decision = build_guard_decision_artifact(&analysis.root, &convergence_history);
    let feedback_loop = build_feedback_loop_summary(&review_surface);
    let agent_handoff = build_agent_handoff_artifact(analysis, &review_surface, &doctrine_registry);
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
        timings: &analysis.timings,
    };

    write_json(&paths.deterministic_analysis, &report)?;
    write_json(&paths.semantic_graph, &analysis.semantic_graph)?;
    write_json(&paths.dependency_graph, &dependency_graph)?;
    write_json(&paths.evidence_graph, &evidence_graph)?;
    write_json(&paths.contract_inventory, &analysis.contract_inventory)?;
    write_json(&paths.doctrine_registry, &doctrine_registry)?;
    write_json(&paths.deterministic_findings, &findings)?;
    write_json(&paths.external_analysis, &analysis.external_analysis)?;
    write_json(&paths.architecture_surface, &surface)?;
    write_json(&paths.review_surface, &review_surface)?;
    write_json(&paths.convergence_history, &convergence_history)?;
    write_json(&paths.guard_decision, &guard_decision)?;
    write_json(&paths.agent_handoff, &agent_handoff)?;
    write_json(&paths.aigiscode_report, &report)?;
    write_markdown(
        &paths.aigiscode_report_markdown,
        &build_markdown_report(analysis, &report, &agent_handoff),
    )?;

    Ok(paths)
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
            visible_findings: review_surface.summary.visible_findings,
            dead_code_count: analysis.dead_code.findings.len(),
            hardwiring_count: analysis.hardwiring.findings.len(),
            security_finding_count: analysis.security_analysis.findings.len(),
            external_finding_count: analysis.external_analysis.findings.len(),
        },
        feedback_loop,
        next_steps,
        guardian_packets,
        top_findings: visible_findings
            .into_iter()
            .take(if high_visible > 0 { 8 } else { 5 })
            .map(|finding| AgentHandoffFinding {
                id: finding.id.clone(),
                family: format!("{:?}", finding.family),
                severity: format!("{:?}", finding.severity),
                title: finding.title.clone(),
                summary: finding.summary.clone(),
                file_paths: finding.file_paths.clone(),
                line: finding.line,
                primary_anchor: finding.primary_anchor.clone(),
            })
            .collect(),
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
                &file,
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
                let doctrine_refs = vec![
                    String::from("guardian.native-vs-library"),
                    String::from(if is_schema_validation {
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
                        if is_schema_validation {
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
        }
    }

    for packet in &mut packets {
        if let Some(finding) = best_packet_supporting_finding(packet, visible_findings) {
            packet.primary_anchor = finding.primary_anchor.clone();
            packet.evidence_anchors = finding.evidence_anchors.clone();
        }
    }

    packets.sort_by(|left, right| {
        packet_priority_rank(&right.priority)
            .cmp(&packet_priority_rank(&left.priority))
            .then(packet_focus_rank(&right.focus).cmp(&packet_focus_rank(&left.focus)))
            .then(left.target_files.cmp(&right.target_files))
            .then(left.focus.cmp(&right.focus))
    });
    packets.truncate(8);
    packets
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
            if context_labels
                .iter()
                .any(|label| label == "externally_reachable" || label == "interactive_execution")
            {
                String::from("sanctioned_security_boundary")
            } else {
                String::from("trusted_runtime_wrapper")
            },
        ),
        "warning_heavy_hotspot" => Some(String::from("single_authoritative_service_boundary")),
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
                .any(|label| label == "externally_reachable");
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
    write_json(&path, surface)?;
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
    write_json(&path, &project.semantic_graph)?;
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
    write_json(&path, &dependency_graph)?;
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
    write_json(&path, &evidence_graph)?;
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
    write_json(&path, &analysis.contract_inventory)?;
    Ok(path)
}

fn build_markdown_report(
    analysis: &ProjectAnalysis,
    report: &AigiscodeReportArtifact<'_>,
    handoff: &AgentHandoffArtifact,
) -> String {
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
        for finding in &handoff.top_findings {
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

fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let payload = serde_json::to_vec_pretty(value).map_err(|error| {
        io::Error::other(format!("failed to serialize {}: {error}", path.display()))
    })?;
    let mut data = payload;
    data.push(b'\n');
    fs::write(path, data)
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
        build_agent_handoff_artifact, build_guard_decision_artifact, default_output_dir,
        write_architecture_surface_artifact, write_contract_inventory_artifact,
        write_dependency_graph_artifact, write_evidence_graph_artifact,
        write_project_analysis_artifacts, write_semantic_graph_artifact, ContractValueDelta,
        ConvergenceContractDelta, ConvergenceGraphDelta, ConvergenceHistoryArtifact,
        ConvergenceRequiredRadius, ConvergenceSummary, GuardVerdict, AGENT_HANDOFF_FILE,
        AIGISCODE_REPORT_FILE, AIGISCODE_REPORT_MARKDOWN_FILE, ARCHITECTURE_SURFACE_FILE,
        CONTRACT_INVENTORY_FILE, CONVERGENCE_HISTORY_FILE, DEPENDENCY_GRAPH_FILE,
        DETERMINISTIC_ANALYSIS_FILE, DETERMINISTIC_FINDINGS_FILE, EVIDENCE_GRAPH_FILE,
        EXTERNAL_ANALYSIS_FILE, GUARD_DECISION_FILE, REVIEW_SURFACE_FILE, SEMANTIC_GRAPH_FILE,
    };
    use crate::doctrine::{built_in_doctrine_registry, load_doctrine_registry};
    use crate::ingestion::pipeline::{analyze_project, build_semantic_graph_project};
    use crate::ingestion::scan::ScanConfig;
    use crate::policy::PolicyBundle;
    use crate::review::build_review_surface;
    use serde_json::Value;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_project_analysis_artifact_family() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod models;
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
        fs::write(fixture.join("src/models.rs"), b"pub struct User;\n").unwrap();

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

        let report_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.aigiscode_report).unwrap()).unwrap();
        assert_eq!(report_payload["summary"]["scanned_files"], 2);
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
