use crate::artifacts::{
    AgentHandoffArtifact, ArtifactPaths, ContractValueDelta, ConvergenceAttentionItem,
    ConvergenceContractDelta, ConvergenceFindingDelta, ConvergenceGraphDelta,
    ConvergenceHistoryArtifact, ConvergenceRequiredRadius, ConvergenceStatus, ConvergenceSummary,
    GuardDecisionArtifact, GuardDecisionPressure, GuardDecisionTrigger, GuardTriggerLevel,
    GuardVerdict, GuardianPacket,
};
use crate::contracts::{
    ContractCategorySummary, ContractInventory, ContractSemanticModelPackUsage,
};
use crate::detectors::dead_code::{DeadCodeCategory, DeadCodeFinding};
use crate::detectors::hardwiring::{HardwiringCategory, HardwiringFinding};
use crate::doctrine::{
    load_doctrine_registry, DoctrineCategory, DoctrineDisposition, DoctrineLoadError,
};
use crate::external::ExternalFinding;
use crate::graph::analysis::{BottleneckFile, CycleClass, CycleFinding};
use crate::graph::{GraphLayer, RelationKind};
use crate::ingestion::pipeline::ProjectAnalysis;
use crate::review::{
    PolicyStatus as ReviewPolicyStatus, ReviewFinding, ReviewFindingFamily, ReviewFindingSeverity,
    ReviewSurface,
};
use crate::security::{SecurityCategory, SecurityFinding};
use crate::surface::{ArchitectureSurface, HotspotFile, LanguageCoverage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::Not;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DoctrineRegistryOutput {
    pub version: String,
    pub clauses: Vec<DoctrineClauseOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DoctrineClauseOutput {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub default_disposition: String,
    pub preferred_mechanism: Option<String>,
    pub guidance: Vec<String>,
}

impl DoctrineRegistryOutput {
    pub fn load(root: &Path) -> Result<Self, DoctrineLoadError> {
        Ok(Self::from_registry(&load_doctrine_registry(root)?))
    }

    fn from_registry(registry: &crate::doctrine::DoctrineRegistry) -> Self {
        Self {
            version: registry.version.clone(),
            clauses: registry
                .clauses
                .iter()
                .map(|clause| DoctrineClauseOutput {
                    id: clause.id.clone(),
                    title: clause.title.clone(),
                    description: clause.description.clone(),
                    category: doctrine_category_label(clause.category).to_string(),
                    default_disposition: doctrine_disposition_label(clause.default_disposition)
                        .to_string(),
                    preferred_mechanism: clause.preferred_mechanism.clone(),
                    guidance: clause.guidance.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingFamilyFilter {
    Graph,
    DeadCode,
    Hardwiring,
    Security,
    External,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverityFilter {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ListFindingsParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<FindingFamilyFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<FindingSeverityFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExplainFindingParams {
    pub finding_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ShowHotspotsParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ShowCyclesParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strong_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CypherQueryParams {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoOverviewOutput {
    pub root: String,
    pub artifact_files: ArtifactFileOutput,
    pub overview: OverviewOutput,
    pub contract_inventory: ContractInventoryOutput,
    pub review_summary: ReviewSummaryOutput,
    pub convergence: ConvergenceSummaryOutput,
    pub guard_decision: GuardDecisionOutput,
    pub feedback_loop: FeedbackLoopOutput,
    pub guardian_packets: Vec<GuardianPacketOutput>,
    pub languages: Vec<LanguageCoverageOutput>,
    pub top_findings: Vec<FindingSummaryOutput>,
}

impl RepoOverviewOutput {
    pub fn new(
        root: &str,
        surface: &ArchitectureSurface,
        review_surface: &ReviewSurface,
        artifact_paths: &ArtifactPaths,
        kuzu_path: Option<&Path>,
        handoff: &AgentHandoffArtifact,
        convergence: &ConvergenceOutput,
        guard_decision: &GuardDecisionOutput,
        top_findings: Vec<FindingSummaryOutput>,
    ) -> Self {
        Self {
            root: String::from(root),
            artifact_files: ArtifactFileOutput::from_paths(artifact_paths, kuzu_path),
            overview: OverviewOutput::from_surface(surface),
            contract_inventory: ContractInventoryOutput::from_inventory(
                &surface.contract_inventory,
            ),
            review_summary: ReviewSummaryOutput::from_review_surface(review_surface),
            convergence: convergence.summary.clone(),
            guard_decision: guard_decision.clone(),
            feedback_loop: FeedbackLoopOutput::from_handoff(handoff),
            guardian_packets: handoff
                .guardian_packets
                .iter()
                .map(GuardianPacketOutput::from_packet)
                .collect(),
            languages: surface
                .languages
                .iter()
                .map(LanguageCoverageOutput::from_language)
                .collect(),
            top_findings,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GuardianPacketOutput {
    pub id: String,
    pub priority: String,
    pub focus: String,
    pub primary_target_file: String,
    pub precision: String,
    pub confidence_millis: u16,
    pub summary: String,
    pub target_files: Vec<String>,
    pub primary_anchor: Option<EvidenceAnchorOutput>,
    pub evidence_anchors: Vec<EvidenceAnchorOutput>,
    pub finding_ids: Vec<String>,
    pub context_labels: Vec<String>,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
    pub preferred_mechanism: Option<String>,
    pub obligations: Vec<GuardianObligationOutput>,
    pub suppressibility: GuardianSuppressibilityOutput,
    pub investigation_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GuardianObligationOutput {
    pub action: String,
    pub acceptance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GuardianSuppressibilityOutput {
    pub allowed: bool,
    pub requires_reason: bool,
    pub expiry_required: bool,
}

impl GuardianPacketOutput {
    fn from_packet(packet: &GuardianPacket) -> Self {
        Self {
            id: packet.id.clone(),
            priority: packet.priority.clone(),
            focus: packet.focus.clone(),
            primary_target_file: packet.primary_target_file.clone(),
            precision: packet.precision.clone(),
            confidence_millis: packet.confidence_millis,
            summary: packet.summary.clone(),
            target_files: packet.target_files.clone(),
            primary_anchor: packet
                .primary_anchor
                .as_ref()
                .map(EvidenceAnchorOutput::from_anchor),
            evidence_anchors: packet
                .evidence_anchors
                .iter()
                .map(EvidenceAnchorOutput::from_anchor)
                .collect(),
            finding_ids: packet.finding_ids.clone(),
            context_labels: packet.context_labels.clone(),
            provenance: packet.provenance.clone(),
            doctrine_refs: packet.doctrine_refs.clone(),
            preferred_mechanism: packet.preferred_mechanism.clone(),
            obligations: packet
                .obligations
                .iter()
                .map(|obligation| GuardianObligationOutput {
                    action: obligation.action.clone(),
                    acceptance: obligation.acceptance.clone(),
                })
                .collect(),
            suppressibility: GuardianSuppressibilityOutput {
                allowed: packet.suppressibility.allowed,
                requires_reason: packet.suppressibility.requires_reason,
                expiry_required: packet.suppressibility.expiry_required,
            },
            investigation_questions: packet.investigation_questions.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReviewSummaryOutput {
    pub visible_findings: usize,
    pub accepted_by_policy: usize,
    pub suppressed_by_rule: usize,
    pub unreviewed_findings: usize,
}

impl ReviewSummaryOutput {
    fn from_review_surface(review_surface: &ReviewSurface) -> Self {
        Self {
            visible_findings: review_surface.summary.visible_findings,
            accepted_by_policy: review_surface.summary.accepted_by_policy,
            suppressed_by_rule: review_surface.summary.suppressed_by_rule,
            unreviewed_findings: review_surface.summary.unreviewed_findings,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FeedbackLoopOutput {
    pub detected_total: usize,
    pub actionable_visible: usize,
    pub accepted_by_policy: usize,
    pub suppressed_by_rule: usize,
    pub ai_reviewed: usize,
    pub rules_generated: usize,
}

impl FeedbackLoopOutput {
    fn from_handoff(handoff: &AgentHandoffArtifact) -> Self {
        Self {
            detected_total: handoff.feedback_loop.detected_total,
            actionable_visible: handoff.feedback_loop.actionable_visible,
            accepted_by_policy: handoff.feedback_loop.accepted_by_policy,
            suppressed_by_rule: handoff.feedback_loop.suppressed_by_rule,
            ai_reviewed: handoff.feedback_loop.ai_reviewed,
            rules_generated: handoff.feedback_loop.rules_generated,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactFileOutput {
    pub output_dir: String,
    pub deterministic_analysis: String,
    pub semantic_graph: String,
    pub dependency_graph: String,
    pub evidence_graph: String,
    pub contract_inventory: String,
    pub doctrine_registry: String,
    pub kuzu_graph: Option<String>,
    pub deterministic_findings: String,
    pub external_analysis: String,
    pub architecture_surface: String,
    pub review_surface: String,
    pub convergence_history: String,
    pub guard_decision: String,
    pub agent_handoff: String,
    pub aigiscode_report: String,
    pub aigiscode_report_markdown: String,
}

impl ArtifactFileOutput {
    fn from_paths(paths: &ArtifactPaths, kuzu_path: Option<&Path>) -> Self {
        Self {
            output_dir: display_path(&paths.output_dir),
            deterministic_analysis: display_path(&paths.deterministic_analysis),
            semantic_graph: display_path(&paths.semantic_graph),
            dependency_graph: display_path(&paths.dependency_graph),
            evidence_graph: display_path(&paths.evidence_graph),
            contract_inventory: display_path(&paths.contract_inventory),
            doctrine_registry: display_path(&paths.doctrine_registry),
            kuzu_graph: kuzu_path.map(display_path),
            deterministic_findings: display_path(&paths.deterministic_findings),
            external_analysis: display_path(&paths.external_analysis),
            architecture_surface: display_path(&paths.architecture_surface),
            review_surface: display_path(&paths.review_surface),
            convergence_history: display_path(&paths.convergence_history),
            guard_decision: display_path(&paths.guard_decision),
            agent_handoff: display_path(&paths.agent_handoff),
            aigiscode_report: display_path(&paths.aigiscode_report),
            aigiscode_report_markdown: display_path(&paths.aigiscode_report_markdown),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GuardDecisionOutput {
    pub verdict: String,
    pub confidence_millis: u16,
    pub summary: String,
    pub reasons: Vec<String>,
    pub triggers: Vec<GuardDecisionTriggerOutput>,
    pub doctrine_refs: Vec<String>,
    pub obligations: Vec<GuardianObligationOutput>,
    pub required_radius: ConvergenceRequiredRadiusOutput,
    pub attention_items: Vec<ConvergenceAttentionItemOutput>,
    pub pressure: GuardDecisionPressureOutput,
}

impl GuardDecisionOutput {
    pub fn from_artifact(artifact: &GuardDecisionArtifact) -> Self {
        Self {
            verdict: guard_verdict_label(artifact.verdict),
            confidence_millis: artifact.confidence_millis,
            summary: artifact.summary.clone(),
            reasons: artifact.reasons.clone(),
            triggers: artifact
                .triggers
                .iter()
                .map(GuardDecisionTriggerOutput::from_trigger)
                .collect(),
            doctrine_refs: artifact.doctrine_refs.clone(),
            obligations: artifact
                .obligations
                .iter()
                .map(|obligation| GuardianObligationOutput {
                    action: obligation.action.clone(),
                    acceptance: obligation.acceptance.clone(),
                })
                .collect(),
            required_radius: ConvergenceRequiredRadiusOutput::from_radius(
                &artifact.required_radius,
            ),
            attention_items: artifact
                .attention_items
                .iter()
                .map(ConvergenceAttentionItemOutput::from_item)
                .collect(),
            pressure: GuardDecisionPressureOutput::from_pressure(&artifact.pressure),
        }
    }

    pub fn empty() -> Self {
        Self {
            verdict: String::from("allow"),
            confidence_millis: 0,
            summary: String::from("No guard decision artifact is available."),
            reasons: Vec::new(),
            triggers: Vec::new(),
            doctrine_refs: Vec::new(),
            obligations: Vec::new(),
            required_radius: ConvergenceRequiredRadiusOutput::default(),
            attention_items: Vec::new(),
            pressure: GuardDecisionPressureOutput::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct GuardDecisionPressureOutput {
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
    pub abstraction_sprawl_regression: bool,
}

impl GuardDecisionPressureOutput {
    fn from_pressure(pressure: &GuardDecisionPressure) -> Self {
        Self {
            new_findings: pressure.new_findings,
            worsened_findings: pressure.worsened_findings,
            attention_items: pressure.attention_items,
            exact_or_modeled_attention_items: pressure.exact_or_modeled_attention_items,
            heuristic_attention_items: pressure.heuristic_attention_items,
            required_radius_anchor_files: pressure.required_radius_anchor_files,
            required_radius_one_hop_files: pressure.required_radius_one_hop_files,
            visible_finding_delta: pressure.visible_finding_delta,
            contract_delta_count: pressure.contract_delta_count,
            high_severity_security_regressions: pressure.high_severity_security_regressions,
            cycle_regression: pressure.cycle_regression,
            bottleneck_regression: pressure.bottleneck_regression,
            architectural_smell_regression: pressure.architectural_smell_regression,
            warning_heavy_hotspot_regression: pressure.warning_heavy_hotspot_regression,
            split_identity_model_regression: pressure.split_identity_model_regression,
            compatibility_scar_regression: pressure.compatibility_scar_regression,
            duplicate_mechanism_regression: pressure.duplicate_mechanism_regression,
            sanctioned_path_bypass_regression: pressure.sanctioned_path_bypass_regression,
            abstraction_sprawl_regression: pressure.abstraction_sprawl_regression,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GuardDecisionTriggerOutput {
    pub level: String,
    pub message: String,
    pub precision: String,
    pub confidence_millis: u16,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
}

impl GuardDecisionTriggerOutput {
    fn from_trigger(trigger: &GuardDecisionTrigger) -> Self {
        Self {
            level: guard_trigger_level_label(trigger.level),
            message: trigger.message.clone(),
            precision: trigger.precision.clone(),
            confidence_millis: trigger.confidence_millis,
            provenance: trigger.provenance.clone(),
            doctrine_refs: trigger.doctrine_refs.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceOutput {
    pub root: String,
    pub summary: ConvergenceSummaryOutput,
    pub graph_delta: ConvergenceGraphDeltaOutput,
    pub contract_delta: ConvergenceContractDeltaOutput,
    pub required_investigation_files: Vec<String>,
    pub required_radius: ConvergenceRequiredRadiusOutput,
    pub attention_items: Vec<ConvergenceAttentionItemOutput>,
    pub findings: Vec<ConvergenceFindingOutput>,
}

impl ConvergenceOutput {
    pub fn from_artifact(artifact: &ConvergenceHistoryArtifact) -> Self {
        Self {
            root: artifact.root.clone(),
            summary: ConvergenceSummaryOutput::from_summary(&artifact.summary),
            graph_delta: ConvergenceGraphDeltaOutput::from_graph_delta(&artifact.graph_delta),
            contract_delta: ConvergenceContractDeltaOutput::from_contract_delta(
                &artifact.contract_delta,
            ),
            required_investigation_files: artifact.required_investigation_files.clone(),
            required_radius: ConvergenceRequiredRadiusOutput::from_radius(
                &artifact.required_radius,
            ),
            attention_items: artifact
                .attention_items
                .iter()
                .map(ConvergenceAttentionItemOutput::from_item)
                .collect(),
            findings: artifact
                .findings
                .iter()
                .map(ConvergenceFindingOutput::from_delta)
                .collect(),
        }
    }

    pub fn empty(root: &str) -> Self {
        Self {
            root: String::from(root),
            summary: ConvergenceSummaryOutput {
                current_findings: 0,
                previous_findings: 0,
                new_findings: 0,
                worsened_findings: 0,
                improved_findings: 0,
                unchanged_findings: 0,
                resolved_findings: 0,
            },
            graph_delta: ConvergenceGraphDeltaOutput {
                strong_cycle_delta: 0,
                total_cycle_delta: 0,
                bottleneck_delta: 0,
                architectural_smell_delta: 0,
                warning_heavy_hotspot_delta: 0,
                split_identity_model_delta: 0,
                compatibility_scar_delta: 0,
                duplicate_mechanism_delta: 0,
                sanctioned_path_bypass_delta: 0,
                hand_rolled_parsing_delta: 0,
                abstraction_sprawl_delta: 0,
                visible_finding_delta: 0,
            },
            contract_delta: ConvergenceContractDeltaOutput {
                routes: ContractValueDeltaOutput::default(),
                hooks: ContractValueDeltaOutput::default(),
                registered_keys: ContractValueDeltaOutput::default(),
                symbolic_literals: ContractValueDeltaOutput::default(),
                env_keys: ContractValueDeltaOutput::default(),
                config_keys: ContractValueDeltaOutput::default(),
            },
            required_investigation_files: Vec::new(),
            required_radius: ConvergenceRequiredRadiusOutput::default(),
            attention_items: Vec::new(),
            findings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceSummaryOutput {
    pub current_findings: usize,
    pub previous_findings: usize,
    pub new_findings: usize,
    pub worsened_findings: usize,
    pub improved_findings: usize,
    pub unchanged_findings: usize,
    pub resolved_findings: usize,
}

impl ConvergenceSummaryOutput {
    fn from_summary(summary: &ConvergenceSummary) -> Self {
        Self {
            current_findings: summary.current_findings,
            previous_findings: summary.previous_findings,
            new_findings: summary.new_findings,
            worsened_findings: summary.worsened_findings,
            improved_findings: summary.improved_findings,
            unchanged_findings: summary.unchanged_findings,
            resolved_findings: summary.resolved_findings,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceGraphDeltaOutput {
    pub strong_cycle_delta: isize,
    pub total_cycle_delta: isize,
    pub bottleneck_delta: isize,
    pub architectural_smell_delta: isize,
    pub warning_heavy_hotspot_delta: isize,
    pub split_identity_model_delta: isize,
    pub compatibility_scar_delta: isize,
    pub duplicate_mechanism_delta: isize,
    pub sanctioned_path_bypass_delta: isize,
    pub hand_rolled_parsing_delta: isize,
    pub abstraction_sprawl_delta: isize,
    pub visible_finding_delta: isize,
}

impl ConvergenceGraphDeltaOutput {
    fn from_graph_delta(delta: &ConvergenceGraphDelta) -> Self {
        Self {
            strong_cycle_delta: delta.strong_cycle_delta,
            total_cycle_delta: delta.total_cycle_delta,
            bottleneck_delta: delta.bottleneck_delta,
            architectural_smell_delta: delta.architectural_smell_delta,
            warning_heavy_hotspot_delta: delta.warning_heavy_hotspot_delta,
            split_identity_model_delta: delta.split_identity_model_delta,
            compatibility_scar_delta: delta.compatibility_scar_delta,
            duplicate_mechanism_delta: delta.duplicate_mechanism_delta,
            sanctioned_path_bypass_delta: delta.sanctioned_path_bypass_delta,
            hand_rolled_parsing_delta: delta.hand_rolled_parsing_delta,
            abstraction_sprawl_delta: delta.abstraction_sprawl_delta,
            visible_finding_delta: delta.visible_finding_delta,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceRequiredRadiusOutput {
    pub anchor_files: Vec<String>,
    pub one_hop_files: Vec<String>,
    pub inbound_neighbor_count: usize,
    pub outbound_neighbor_count: usize,
}

impl ConvergenceRequiredRadiusOutput {
    fn from_radius(radius: &ConvergenceRequiredRadius) -> Self {
        Self {
            anchor_files: radius.anchor_files.clone(),
            one_hop_files: radius.one_hop_files.clone(),
            inbound_neighbor_count: radius.inbound_neighbor_count,
            outbound_neighbor_count: radius.outbound_neighbor_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceContractDeltaOutput {
    pub routes: ContractValueDeltaOutput,
    pub hooks: ContractValueDeltaOutput,
    pub registered_keys: ContractValueDeltaOutput,
    pub symbolic_literals: ContractValueDeltaOutput,
    pub env_keys: ContractValueDeltaOutput,
    pub config_keys: ContractValueDeltaOutput,
}

impl ConvergenceContractDeltaOutput {
    fn from_contract_delta(delta: &ConvergenceContractDelta) -> Self {
        Self {
            routes: ContractValueDeltaOutput::from_value_delta(&delta.routes),
            hooks: ContractValueDeltaOutput::from_value_delta(&delta.hooks),
            registered_keys: ContractValueDeltaOutput::from_value_delta(&delta.registered_keys),
            symbolic_literals: ContractValueDeltaOutput::from_value_delta(&delta.symbolic_literals),
            env_keys: ContractValueDeltaOutput::from_value_delta(&delta.env_keys),
            config_keys: ContractValueDeltaOutput::from_value_delta(&delta.config_keys),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ContractValueDeltaOutput {
    pub added_count: usize,
    pub removed_count: usize,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

impl ContractValueDeltaOutput {
    fn from_value_delta(delta: &ContractValueDelta) -> Self {
        Self {
            added_count: delta.added_count,
            removed_count: delta.removed_count,
            added: delta.added.clone(),
            removed: delta.removed.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceAttentionItemOutput {
    pub fingerprint: String,
    pub status: String,
    pub title: String,
    pub family: String,
    pub precision: String,
    pub confidence_millis: u16,
    pub summary: String,
    pub file_paths: Vec<String>,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
    pub obligations: Vec<GuardianObligationOutput>,
}

impl ConvergenceAttentionItemOutput {
    fn from_item(item: &ConvergenceAttentionItem) -> Self {
        Self {
            fingerprint: item.fingerprint.clone(),
            status: convergence_status_label(item.status),
            title: item.title.clone(),
            family: item.family.clone(),
            precision: item.precision.clone(),
            confidence_millis: item.confidence_millis,
            summary: item.summary.clone(),
            file_paths: item.file_paths.clone(),
            provenance: item.provenance.clone(),
            doctrine_refs: item.doctrine_refs.clone(),
            obligations: item
                .obligations
                .iter()
                .map(|obligation| GuardianObligationOutput {
                    action: obligation.action.clone(),
                    acceptance: obligation.acceptance.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceFindingOutput {
    pub fingerprint: String,
    pub current_id: Option<String>,
    pub previous_id: Option<String>,
    pub title: String,
    pub family: String,
    pub status: String,
    pub current_severity: Option<String>,
    pub previous_severity: Option<String>,
    pub current_visible: Option<bool>,
    pub previous_visible: Option<bool>,
    pub file_paths: Vec<String>,
}

impl ConvergenceFindingOutput {
    fn from_delta(delta: &ConvergenceFindingDelta) -> Self {
        Self {
            fingerprint: delta.fingerprint.clone(),
            current_id: delta.current_id.clone(),
            previous_id: delta.previous_id.clone(),
            title: delta.title.clone(),
            family: delta.family.clone(),
            status: convergence_status_label(delta.status),
            current_severity: delta.current_severity.clone(),
            previous_severity: delta.previous_severity.clone(),
            current_visible: delta.current_visible,
            previous_visible: delta.previous_visible,
            file_paths: delta.file_paths.clone(),
        }
    }
}

fn convergence_status_label(status: ConvergenceStatus) -> String {
    match status {
        ConvergenceStatus::New => String::from("new"),
        ConvergenceStatus::Worsened => String::from("worsened"),
        ConvergenceStatus::Improved => String::from("improved"),
        ConvergenceStatus::Unchanged => String::from("unchanged"),
        ConvergenceStatus::Resolved => String::from("resolved"),
    }
}

fn guard_verdict_label(verdict: GuardVerdict) -> String {
    match verdict {
        GuardVerdict::Allow => String::from("allow"),
        GuardVerdict::Warn => String::from("warn"),
        GuardVerdict::Block => String::from("block"),
    }
}

fn guard_trigger_level_label(level: GuardTriggerLevel) -> String {
    match level {
        GuardTriggerLevel::Warn => String::from("warn"),
        GuardTriggerLevel::Block => String::from("block"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OverviewOutput {
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub symbols: usize,
    pub references: usize,
    pub resolved_edges: usize,
    pub unresolved_reference_sites: usize,
    pub strong_cycle_count: usize,
    pub total_cycle_count: usize,
    pub bottleneck_count: usize,
    pub orphan_count: usize,
    pub runtime_entry_count: usize,
    pub dead_code_count: usize,
    pub hardwiring_count: usize,
    pub security_finding_count: usize,
    pub external_finding_count: usize,
    pub external_tool_run_count: usize,
    pub override_edge_count: usize,
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
    pub route_contract_count: usize,
    pub hook_contract_count: usize,
    pub registered_key_count: usize,
    pub symbolic_literal_count: usize,
    pub env_key_count: usize,
    pub config_key_count: usize,
}

impl OverviewOutput {
    fn from_surface(surface: &ArchitectureSurface) -> Self {
        Self {
            scanned_files: surface.overview.scanned_files,
            analyzed_files: surface.overview.analyzed_files,
            symbols: surface.overview.symbols,
            references: surface.overview.references,
            resolved_edges: surface.overview.resolved_edges,
            unresolved_reference_sites: surface.overview.unresolved_reference_sites,
            strong_cycle_count: surface.overview.strong_cycle_count,
            total_cycle_count: surface.overview.total_cycle_count,
            bottleneck_count: surface.overview.bottleneck_count,
            orphan_count: surface.overview.orphan_count,
            runtime_entry_count: surface.overview.runtime_entry_count,
            dead_code_count: surface.overview.dead_code_count,
            hardwiring_count: surface.overview.hardwiring_count,
            security_finding_count: surface.overview.security_finding_count,
            external_finding_count: surface.overview.external_finding_count,
            external_tool_run_count: surface.overview.external_tool_run_count,
            override_edge_count: surface.overview.override_edge_count,
            architectural_smell_count: surface.overview.architectural_smell_count,
            hub_like_dependency_count: surface.overview.hub_like_dependency_count,
            unstable_dependency_count: surface.overview.unstable_dependency_count,
            warning_heavy_hotspot_count: surface.overview.warning_heavy_hotspot_count,
            split_identity_model_count: surface.overview.split_identity_model_count,
            compatibility_scar_count: surface.overview.compatibility_scar_count,
            duplicate_mechanism_count: surface.overview.duplicate_mechanism_count,
            sanctioned_path_bypass_count: surface.overview.sanctioned_path_bypass_count,
            hand_rolled_parsing_count: surface.overview.hand_rolled_parsing_count,
            abstraction_sprawl_count: surface.overview.abstraction_sprawl_count,
            route_contract_count: surface.overview.route_contract_count,
            hook_contract_count: surface.overview.hook_contract_count,
            registered_key_count: surface.overview.registered_key_count,
            symbolic_literal_count: surface.overview.symbolic_literal_count,
            env_key_count: surface.overview.env_key_count,
            config_key_count: surface.overview.config_key_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractInventoryOutput {
    pub summary: ContractInventorySummaryOutput,
    pub semantic_model_packs: Vec<ContractSemanticModelPackUsageOutput>,
    pub routes: Vec<ContractItemOutput>,
    pub hooks: Vec<ContractItemOutput>,
    pub registered_keys: Vec<ContractItemOutput>,
    pub symbolic_literals: Vec<ContractItemOutput>,
    pub env_keys: Vec<ContractItemOutput>,
    pub config_keys: Vec<ContractItemOutput>,
}

impl ContractInventoryOutput {
    pub(crate) fn from_inventory(inventory: &ContractInventory) -> Self {
        Self {
            summary: ContractInventorySummaryOutput::from_summary(&inventory.summary),
            semantic_model_packs: inventory
                .semantic_model_packs
                .iter()
                .map(ContractSemanticModelPackUsageOutput::from_usage)
                .collect(),
            routes: inventory
                .routes
                .iter()
                .map(ContractItemOutput::from_item)
                .collect(),
            hooks: inventory
                .hooks
                .iter()
                .map(ContractItemOutput::from_item)
                .collect(),
            registered_keys: inventory
                .registered_keys
                .iter()
                .map(ContractItemOutput::from_item)
                .collect(),
            symbolic_literals: inventory
                .symbolic_literals
                .iter()
                .map(ContractItemOutput::from_item)
                .collect(),
            env_keys: inventory
                .env_keys
                .iter()
                .map(ContractItemOutput::from_item)
                .collect(),
            config_keys: inventory
                .config_keys
                .iter()
                .map(ContractItemOutput::from_item)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractSemanticModelPackUsageOutput {
    pub id: String,
    pub description: String,
    pub layer: String,
    pub contract_categories: Vec<String>,
}

impl ContractSemanticModelPackUsageOutput {
    fn from_usage(usage: &ContractSemanticModelPackUsage) -> Self {
        Self {
            id: usage.id.clone(),
            description: usage.description.clone(),
            layer: match usage.layer {
                crate::semantic_models::SemanticModelPackLayer::Ecosystem => {
                    String::from("ecosystem")
                }
                crate::semantic_models::SemanticModelPackLayer::Framework => {
                    String::from("framework")
                }
                crate::semantic_models::SemanticModelPackLayer::Library => String::from("library"),
            },
            contract_categories: usage.contract_categories.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractInventorySummaryOutput {
    pub routes: ContractCategorySummaryOutput,
    pub hooks: ContractCategorySummaryOutput,
    pub registered_keys: ContractCategorySummaryOutput,
    pub symbolic_literals: ContractCategorySummaryOutput,
    pub env_keys: ContractCategorySummaryOutput,
    pub config_keys: ContractCategorySummaryOutput,
}

impl ContractInventorySummaryOutput {
    fn from_summary(summary: &crate::contracts::ContractInventorySummary) -> Self {
        Self {
            routes: ContractCategorySummaryOutput::from_summary(&summary.routes),
            hooks: ContractCategorySummaryOutput::from_summary(&summary.hooks),
            registered_keys: ContractCategorySummaryOutput::from_summary(&summary.registered_keys),
            symbolic_literals: ContractCategorySummaryOutput::from_summary(
                &summary.symbolic_literals,
            ),
            env_keys: ContractCategorySummaryOutput::from_summary(&summary.env_keys),
            config_keys: ContractCategorySummaryOutput::from_summary(&summary.config_keys),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractCategorySummaryOutput {
    pub unique_values: usize,
    pub occurrences: usize,
}

impl ContractCategorySummaryOutput {
    fn from_summary(summary: &ContractCategorySummary) -> Self {
        Self {
            unique_values: summary.unique_values,
            occurrences: summary.occurrences,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractItemOutput {
    pub value: String,
    pub count: usize,
    pub locations: Vec<ContractLocationOutput>,
}

impl ContractItemOutput {
    fn from_item(item: &crate::contracts::ContractInventoryItem) -> Self {
        Self {
            value: item.value.clone(),
            count: item.count,
            locations: item
                .locations
                .iter()
                .map(ContractLocationOutput::from_location)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractLocationOutput {
    pub file_path: String,
    pub line: usize,
}

impl ContractLocationOutput {
    fn from_location(location: &crate::contracts::ContractLocation) -> Self {
        Self {
            file_path: display_path(&location.file_path),
            line: location.line,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LanguageCoverageOutput {
    pub language: String,
    pub file_count: usize,
}

impl LanguageCoverageOutput {
    fn from_language(language: &LanguageCoverage) -> Self {
        Self {
            language: language.language.clone(),
            file_count: language.file_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListFindingsOutput {
    pub total: usize,
    pub findings: Vec<FindingSummaryOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceAnchorOutput {
    pub file_path: String,
    pub line: Option<usize>,
    pub label: String,
}

impl EvidenceAnchorOutput {
    fn from_anchor(anchor: &crate::evidence::EvidenceAnchor) -> Self {
        Self {
            file_path: anchor.file_path.display().to_string(),
            line: anchor.line,
            label: anchor.label.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindingSummaryOutput {
    pub id: String,
    pub fingerprint: String,
    pub family: String,
    pub severity: String,
    pub precision: String,
    pub confidence_millis: u16,
    pub title: String,
    pub summary: String,
    pub file_paths: Vec<String>,
    pub line: Option<usize>,
    pub primary_anchor: Option<EvidenceAnchorOutput>,
    pub evidence_anchors: Vec<EvidenceAnchorOutput>,
    pub languages: Vec<String>,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
    pub policy_status: String,
    pub is_visible: bool,
}

impl FindingSummaryOutput {
    pub fn from_review_finding(finding: &ReviewFinding) -> Self {
        Self {
            id: finding.id.clone(),
            fingerprint: finding.fingerprint.clone(),
            family: review_family_label(finding.family),
            severity: review_severity_label(finding.severity),
            precision: finding.precision.clone(),
            confidence_millis: finding.confidence_millis,
            title: finding.title.clone(),
            summary: finding.summary.clone(),
            file_paths: finding.file_paths.clone(),
            line: finding.line,
            primary_anchor: finding
                .primary_anchor
                .as_ref()
                .map(EvidenceAnchorOutput::from_anchor),
            evidence_anchors: finding
                .evidence_anchors
                .iter()
                .map(EvidenceAnchorOutput::from_anchor)
                .collect(),
            languages: infer_languages_from_strings(&finding.file_paths),
            provenance: finding.provenance.clone(),
            doctrine_refs: finding.doctrine_refs.clone(),
            policy_status: review_policy_status_label(finding.policy_status),
            is_visible: finding.is_visible,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindingDetailOutput {
    pub finding: FindingSummaryOutput,
    pub explanation: String,
    pub evidence_kind: String,
    pub related_files: Vec<String>,
    pub cycle_files: Vec<String>,
    pub hotspot: Option<HotspotOutput>,
    pub symbol_id: Option<String>,
    pub literal_value: Option<String>,
    pub context: Option<String>,
    pub resource_uri: String,
    pub policy_status: String,
    pub is_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HotspotsOutput {
    pub hotspots: Vec<HotspotOutput>,
    pub bottlenecks: Vec<BottleneckOutput>,
    pub orphan_files: Vec<String>,
    pub runtime_entry_candidates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HotspotOutput {
    pub file_path: String,
    pub language: String,
    pub inbound_edges: usize,
    pub outbound_edges: usize,
    pub finding_count: usize,
    pub bottleneck_centrality_millis: u32,
    pub is_orphan: bool,
    pub is_runtime_entry: bool,
}

impl HotspotOutput {
    pub fn from_hotspot(hotspot: HotspotFile) -> Self {
        Self {
            file_path: display_path(&hotspot.file_path),
            language: hotspot.language,
            inbound_edges: hotspot.inbound_edges,
            outbound_edges: hotspot.outbound_edges,
            finding_count: hotspot.finding_count,
            bottleneck_centrality_millis: hotspot.bottleneck_centrality_millis,
            is_orphan: hotspot.is_orphan,
            is_runtime_entry: hotspot.is_runtime_entry,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BottleneckOutput {
    pub file_path: String,
    pub centrality_millis: u32,
}

impl BottleneckOutput {
    pub fn from_bottleneck(bottleneck: &BottleneckFile) -> Self {
        Self {
            file_path: display_path(&bottleneck.file_path),
            centrality_millis: bottleneck.centrality_millis,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CyclesOutput {
    pub strong_cycles: Vec<CycleOutput>,
    pub total_cycles: Vec<CycleOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CycleOutput {
    pub size: usize,
    pub files: Vec<String>,
    pub cycle_class: String,
    pub layers: Vec<String>,
    pub dominant_relations: Vec<String>,
    pub edge_count: usize,
}

impl CycleOutput {
    pub fn from_cycle_finding(cycle: &CycleFinding) -> Self {
        Self {
            size: cycle.files.len(),
            files: cycle.files.iter().map(|path| display_path(path)).collect(),
            cycle_class: cycle_class_label(cycle.cycle_class).to_owned(),
            layers: cycle
                .layers
                .iter()
                .map(graph_layer_label)
                .map(str::to_owned)
                .collect(),
            dominant_relations: cycle
                .dominant_relations
                .iter()
                .map(relation_kind_label)
                .map(str::to_owned)
                .collect(),
            edge_count: cycle.edge_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityEvaluationOutput {
    pub root: String,
    pub summary: String,
    pub dimensions: Vec<QualityDimensionOutput>,
    pub suspects: Vec<QualitySuspectOutput>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CypherQueryOutput {
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Map<String, serde_json::Value>>,
    pub row_count: usize,
}

impl CypherQueryOutput {
    pub fn from_result(result: crate::kuzu_index::KuzuQueryOutput) -> Self {
        Self {
            columns: result.columns,
            rows: result.rows,
            row_count: result.row_count,
        }
    }
}

impl QualityEvaluationOutput {
    pub fn new(
        root: &str,
        analysis: &ProjectAnalysis,
        surface: &ArchitectureSurface,
        review_surface: &ReviewSurface,
    ) -> Self {
        let architecture_pressure = surface.overview.strong_cycle_count
            + analysis
                .graph_analysis
                .strong_cycle_findings
                .iter()
                .filter(|cycle| {
                    matches!(
                        cycle.cycle_class,
                        CycleClass::Mixed | CycleClass::ProbableArtifact
                    )
                })
                .count();
        let dead_code_pressure = analysis.dead_code.findings.len();
        let hardwiring_pressure = analysis.hardwiring.findings.len();
        let logic_hotspots = surface
            .hotspots
            .iter()
            .filter(|hotspot| {
                hotspot.finding_count >= 10
                    || hotspot.bottleneck_centrality_millis >= 100
                    || (hotspot.inbound_edges + hotspot.outbound_edges) >= 15
            })
            .count();
        let overengineering_pressure = analysis
            .graph_analysis
            .cycle_findings
            .iter()
            .filter(|cycle| {
                matches!(
                    cycle.cycle_class,
                    CycleClass::Mixed | CycleClass::ProbableArtifact | CycleClass::Framework
                )
            })
            .count();
        let security_pressure = analysis.external_analysis.findings.len()
            + analysis
                .hardwiring
                .findings
                .iter()
                .filter(|finding| {
                    matches!(
                        finding.category,
                        HardwiringCategory::HardcodedNetwork | HardwiringCategory::EnvOutsideConfig
                    )
                })
                .count();

        let dimensions = vec![
            QualityDimensionOutput::new(
                "architecture",
                "Architecture",
                architecture_pressure,
                architecture_pressure_severity(architecture_pressure),
                format!(
                    "{} strong cycles, {} typed cycle findings, {} bottlenecks.",
                    surface.overview.strong_cycle_count,
                    analysis.graph_analysis.cycle_findings.len(),
                    analysis.graph_analysis.bottleneck_files.len()
                ),
                supporting_cycle_files(&analysis.graph_analysis.strong_cycle_findings, 5),
            ),
            QualityDimensionOutput::new(
                "dead_code",
                "Dead Code",
                dead_code_pressure,
                count_severity(dead_code_pressure, 50, 250),
                format!(
                    "{} dead-code findings remain visible to deterministic analysis.",
                    dead_code_pressure
                ),
                top_dead_code_files(&analysis.dead_code.findings, 5),
            ),
            QualityDimensionOutput::new(
                "hardwiring",
                "Hardwiring",
                hardwiring_pressure,
                count_severity(hardwiring_pressure, 30, 150),
                format!(
                    "{} hardwiring findings remain after current suppressions.",
                    hardwiring_pressure
                ),
                top_hardwiring_files(&analysis.hardwiring.findings, 5),
            ),
            QualityDimensionOutput::new(
                "logic_concentration",
                "Logic Concentration",
                logic_hotspots,
                count_severity(logic_hotspots, 3, 8),
                format!(
                    "{} files show elevated coupling/finding concentration.",
                    logic_hotspots
                ),
                surface
                    .hotspots
                    .iter()
                    .take(5)
                    .map(|hotspot| display_path(&hotspot.file_path))
                    .collect(),
            ),
            QualityDimensionOutput::new(
                "overengineering",
                "Overengineering Suspects",
                overengineering_pressure,
                count_severity(overengineering_pressure, 2, 6),
                format!(
                    "{} framework/mixed/probable-artifact cycles suggest abstraction or runtime expansion pressure.",
                    overengineering_pressure
                ),
                supporting_cycle_files(&analysis.graph_analysis.cycle_findings, 5),
            ),
            QualityDimensionOutput::new(
                "security",
                "Security Pressure",
                security_pressure,
                count_severity(security_pressure, 1, 5),
                format!(
                    "{} security-relevant findings across external tools and hardcoded network/env access.",
                    security_pressure
                ),
                security_supporting_files(analysis),
            ),
        ];

        let suspects = quality_suspects(analysis, surface);
        let mut recommendations = Vec::new();
        if architecture_pressure > 0 {
            recommendations.push(String::from(
                "Drill into typed cycle findings first and separate structural cycles from framework/runtime expansion before refactoring.",
            ));
        }
        if dead_code_pressure > 0 {
            recommendations.push(String::from(
                "Sample dead-code hotspots and convert repeated accepted patterns into policy or rules instead of widening detector hacks.",
            ));
        }
        if hardwiring_pressure > 0 {
            recommendations.push(String::from(
                "Review repeated literals and hardcoded network/env access for centralization into configuration, constants, or doctrine.",
            ));
        }
        if security_pressure > 0 {
            recommendations.push(String::from(
                "Treat security pressure as guard-rail work: normalize external findings and hardcoded-network findings into the same remediation workflow.",
            ));
        }
        if recommendations.is_empty() {
            recommendations.push(String::from(
                "Current quality surface is quiet; validate with hotspot and finding sampling before tightening policy.",
            ));
        }

        Self {
            root: String::from(root),
            summary: format!(
                "{} visible findings across {} dimensions; {} remain unreviewed.",
                review_surface.summary.visible_findings,
                dimensions.len(),
                review_surface.summary.unreviewed_findings
            ),
            dimensions,
            suspects,
            recommendations,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityDimensionOutput {
    pub key: String,
    pub label: String,
    pub severity: String,
    pub count: usize,
    pub summary: String,
    pub supporting_files: Vec<String>,
}

impl QualityDimensionOutput {
    fn new(
        key: &str,
        label: &str,
        count: usize,
        severity: &'static str,
        summary: String,
        supporting_files: Vec<String>,
    ) -> Self {
        Self {
            key: String::from(key),
            label: String::from(label),
            severity: String::from(severity),
            count,
            summary,
            supporting_files,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualitySuspectOutput {
    pub category: String,
    pub file_path: String,
    pub reason: String,
    pub evidence_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AtlasOutput {
    pub nodes: Vec<AtlasNodeOutput>,
    pub edges: Vec<AtlasEdgeOutput>,
}

impl AtlasOutput {
    pub fn from_surface(surface: &ArchitectureSurface) -> Self {
        Self {
            nodes: surface
                .atlas
                .nodes
                .iter()
                .map(|node| AtlasNodeOutput {
                    file_path: display_path(&node.file_path),
                    language: node.language.clone(),
                    inbound_edges: node.inbound_edges,
                    outbound_edges: node.outbound_edges,
                    finding_count: node.finding_count,
                    bottleneck_centrality_millis: node.bottleneck_centrality_millis,
                    is_orphan: node.is_orphan,
                    is_runtime_entry: node.is_runtime_entry,
                })
                .collect(),
            edges: surface
                .atlas
                .edges
                .iter()
                .map(|edge| AtlasEdgeOutput {
                    source_file_path: display_path(&edge.source_file_path),
                    target_file_path: display_path(&edge.target_file_path),
                    edge_count: edge.edge_count,
                    kinds: edge.kinds.clone(),
                    strongest_resolution_tier: edge.strongest_resolution_tier.clone(),
                    average_confidence_millis: edge.average_confidence_millis,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AtlasNodeOutput {
    pub file_path: String,
    pub language: String,
    pub inbound_edges: usize,
    pub outbound_edges: usize,
    pub finding_count: usize,
    pub bottleneck_centrality_millis: u32,
    pub is_orphan: bool,
    pub is_runtime_entry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AtlasEdgeOutput {
    pub source_file_path: String,
    pub target_file_path: String,
    pub edge_count: usize,
    pub kinds: Vec<String>,
    pub strongest_resolution_tier: String,
    pub average_confidence_millis: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoverageReportOutput {
    pub root: String,
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub unresolved_reference_sites: usize,
    pub supported_languages: Vec<LanguageCoverageOutput>,
    pub visible_findings: usize,
    pub accepted_by_policy: usize,
    pub suppressed_by_rule: usize,
    pub external_findings: usize,
    pub external_tool_runs: usize,
    pub notes: Vec<String>,
}

impl CoverageReportOutput {
    pub fn new(root: &str, surface: &ArchitectureSurface, review_surface: &ReviewSurface) -> Self {
        let mut notes = Vec::new();
        if surface
            .languages
            .iter()
            .any(|language| language.language == "Unsupported")
        {
            notes.push(String::from(
                "Unsupported files are present in the repository and are reported explicitly.",
            ));
        }
        if surface.overview.unresolved_reference_sites > 0 {
            notes.push(format!(
                "{} unresolved reference sites remain after deterministic resolution.",
                surface.overview.unresolved_reference_sites
            ));
        }
        notes.push(String::from(
            "Detector coverage is currently strongest for graph analysis, unused imports/private functions, and first-pass hardwiring heuristics.",
        ));
        Self {
            root: String::from(root),
            scanned_files: surface.overview.scanned_files,
            analyzed_files: surface.overview.analyzed_files,
            unresolved_reference_sites: surface.overview.unresolved_reference_sites,
            supported_languages: surface
                .languages
                .iter()
                .map(LanguageCoverageOutput::from_language)
                .collect(),
            visible_findings: review_surface.summary.visible_findings,
            accepted_by_policy: review_surface.summary.accepted_by_policy,
            suppressed_by_rule: review_surface.summary.suppressed_by_rule,
            external_findings: surface.overview.external_finding_count,
            external_tool_runs: surface.overview.external_tool_run_count,
            notes,
        }
    }
}

pub fn build_finding_details(
    analysis: &ProjectAnalysis,
    surface: &ArchitectureSurface,
    review_surface: &ReviewSurface,
    finding_uri_prefix: &str,
) -> HashMap<String, FindingDetailOutput> {
    let hotspot_map = surface
        .hotspots
        .iter()
        .cloned()
        .map(|hotspot| {
            (
                display_path(&hotspot.file_path),
                HotspotOutput::from_hotspot(hotspot),
            )
        })
        .collect::<HashMap<_, _>>();
    let summaries = review_surface
        .findings
        .iter()
        .map(|finding| {
            (
                finding.id.clone(),
                FindingSummaryOutput::from_review_finding(finding),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut details = HashMap::new();

    for (index, cycle) in analysis
        .graph_analysis
        .strong_cycle_findings
        .iter()
        .enumerate()
    {
        let id = format!("graph:cycle:{index}");
        if let Some(summary) = summaries.get(&id) {
            details.insert(
                id.clone(),
                FindingDetailOutput {
                    finding: summary.clone(),
                    explanation: format!(
                        "{} cycle across {} files. Dominant relations: {}. Layers: {}.",
                        cycle_class_label(cycle.cycle_class),
                        cycle.files.len(),
                        cycle
                            .dominant_relations
                            .iter()
                            .map(relation_kind_label)
                            .collect::<Vec<_>>()
                            .join(", "),
                        cycle
                            .layers
                            .iter()
                            .map(graph_layer_label)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    evidence_kind: String::from("cycle"),
                    related_files: cycle.files.iter().map(|path| display_path(path)).collect(),
                    cycle_files: cycle.files.iter().map(|path| display_path(path)).collect(),
                    hotspot: None,
                    symbol_id: None,
                    literal_value: None,
                    context: None,
                    resource_uri: format!("{finding_uri_prefix}{id}"),
                    policy_status: summary.policy_status.clone(),
                    is_visible: summary.is_visible,
                },
            );
        }
    }

    for path in &analysis.graph_analysis.orphan_files {
        let id = format!("graph:orphan:{}", path.display());
        if let Some(summary) = summaries.get(&id) {
            let path_display = display_path(path);
            details.insert(
                id.clone(),
                FindingDetailOutput {
                    finding: summary.clone(),
                    explanation: String::from(
                        "This file has outbound dependencies but no inbound structural references from the current deterministic graph.",
                    ),
                    evidence_kind: String::from("orphan_file"),
                    related_files: vec![path_display.clone()],
                    cycle_files: Vec::new(),
                    hotspot: hotspot_map.get(&path_display).cloned(),
                    symbol_id: None,
                    literal_value: None,
                    context: None,
                    resource_uri: format!("{finding_uri_prefix}{id}"),
                    policy_status: summary.policy_status.clone(),
                    is_visible: summary.is_visible,
                },
            );
        }
    }

    for bottleneck in &analysis.graph_analysis.bottleneck_files {
        let id = format!("graph:bottleneck:{}", bottleneck.file_path.display());
        if let Some(summary) = summaries.get(&id) {
            let path_display = display_path(&bottleneck.file_path);
            details.insert(
                id.clone(),
                FindingDetailOutput {
                    finding: summary.clone(),
                    explanation: format!(
                        "This file concentrates dependency flow with betweenness centrality {}.",
                        bottleneck.centrality_millis
                    ),
                    evidence_kind: String::from("bottleneck"),
                    related_files: vec![path_display.clone()],
                    cycle_files: Vec::new(),
                    hotspot: hotspot_map.get(&path_display).cloned(),
                    symbol_id: None,
                    literal_value: None,
                    context: None,
                    resource_uri: format!("{finding_uri_prefix}{id}"),
                    policy_status: summary.policy_status.clone(),
                    is_visible: summary.is_visible,
                },
            );
        }
    }

    for highlight in &surface.highlights {
        if highlight.id.starts_with("architecture:").not() {
            continue;
        }
        if let Some(summary) = summaries.get(&highlight.id) {
            let primary_anchor =
                highlight
                    .primary_anchor
                    .as_ref()
                    .map(|anchor| match anchor.line {
                        Some(line) => format!("{}:{line}", display_path(&anchor.file_path)),
                        None => display_path(&anchor.file_path),
                    });
            let related_files = highlight
                .file_paths
                .iter()
                .map(|path| display_path(path))
                .collect::<Vec<_>>();
            let evidence_anchor_count = highlight.evidence_anchors.len();
            details.insert(
                highlight.id.clone(),
                FindingDetailOutput {
                    finding: summary.clone(),
                    explanation: format!(
                        "{} {}{}",
                        highlight.summary,
                        primary_anchor
                            .as_ref()
                            .map(|anchor| format!("Primary anchor: {anchor}. "))
                            .unwrap_or_default(),
                        if evidence_anchor_count > 0 {
                            format!("{evidence_anchor_count} supporting evidence anchor(s) are attached.")
                        } else {
                            String::from("No additional evidence anchors were attached.")
                        }
                    ),
                    evidence_kind: String::from("architectural_assessment"),
                    related_files: related_files.clone(),
                    cycle_files: Vec::new(),
                    hotspot: highlight
                        .file_paths
                        .first()
                        .map(|path| display_path(path))
                        .and_then(|path| hotspot_map.get(&path).cloned()),
                    symbol_id: None,
                    literal_value: None,
                    context: (!highlight.doctrine_refs.is_empty())
                        .then(|| format!("Doctrine: {}", highlight.doctrine_refs.join(", "))),
                    resource_uri: format!("{finding_uri_prefix}{}", highlight.id),
                    policy_status: summary.policy_status.clone(),
                    is_visible: summary.is_visible,
                },
            );
        }
    }

    for finding in &analysis.dead_code.findings {
        let id = format!(
            "dead-code:{}:{}:{}",
            finding.file_path.display(),
            finding.line,
            finding.name
        );
        if let Some(summary) = summaries.get(&id) {
            let path_display = display_path(&finding.file_path);
            details.insert(
                id.clone(),
                FindingDetailOutput {
                    finding: summary.clone(),
                    explanation: dead_code_explanation(finding),
                    evidence_kind: dead_code_kind(finding.category),
                    related_files: vec![path_display.clone()],
                    cycle_files: Vec::new(),
                    hotspot: hotspot_map.get(&path_display).cloned(),
                    symbol_id: Some(finding.symbol_id.clone()),
                    literal_value: None,
                    context: None,
                    resource_uri: format!("{finding_uri_prefix}{id}"),
                    policy_status: summary.policy_status.clone(),
                    is_visible: summary.is_visible,
                },
            );
        }
    }

    for finding in &analysis.hardwiring.findings {
        let id = format!(
            "hardwiring:{}:{}:{}",
            finding.file_path.display(),
            finding.line,
            finding.value
        );
        if let Some(summary) = summaries.get(&id) {
            let path_display = display_path(&finding.file_path);
            details.insert(
                id.clone(),
                FindingDetailOutput {
                    finding: summary.clone(),
                    explanation: hardwiring_explanation(finding),
                    evidence_kind: hardwiring_kind(finding.category),
                    related_files: vec![path_display.clone()],
                    cycle_files: Vec::new(),
                    hotspot: hotspot_map.get(&path_display).cloned(),
                    symbol_id: None,
                    literal_value: Some(finding.value.clone()),
                    context: Some(finding.context.clone()),
                    resource_uri: format!("{finding_uri_prefix}{id}"),
                    policy_status: summary.policy_status.clone(),
                    is_visible: summary.is_visible,
                },
            );
        }
    }

    for finding in &analysis.security_analysis.findings {
        let id = format!("security:native:{}", finding.fingerprint);
        if let Some(summary) = summaries.get(&id) {
            let path_display = display_path(&finding.file_path);
            details.insert(
                id.clone(),
                FindingDetailOutput {
                    finding: summary.clone(),
                    explanation: security_explanation(finding),
                    evidence_kind: security_kind(finding.category),
                    related_files: vec![path_display.clone()],
                    cycle_files: Vec::new(),
                    hotspot: hotspot_map.get(&path_display).cloned(),
                    symbol_id: None,
                    literal_value: None,
                    context: Some(finding.evidence.clone()),
                    resource_uri: format!("{finding_uri_prefix}{id}"),
                    policy_status: summary.policy_status.clone(),
                    is_visible: summary.is_visible,
                },
            );
        }
    }

    for finding in &analysis.external_analysis.findings {
        let id = format!("external:{}:{}", finding.tool, finding.fingerprint);
        if let Some(summary) = summaries.get(&id) {
            let related_files = finding
                .file_path
                .iter()
                .map(|path| display_path(path))
                .collect::<Vec<_>>();
            let hotspot = finding
                .file_path
                .as_ref()
                .and_then(|path| hotspot_map.get(&display_path(path)).cloned());
            details.insert(
                id.clone(),
                FindingDetailOutput {
                    finding: summary.clone(),
                    explanation: external_explanation(finding),
                    evidence_kind: external_kind(finding),
                    related_files,
                    cycle_files: Vec::new(),
                    hotspot,
                    symbol_id: None,
                    literal_value: None,
                    context: None,
                    resource_uri: format!("{finding_uri_prefix}{id}"),
                    policy_status: summary.policy_status.clone(),
                    is_visible: summary.is_visible,
                },
            );
        }
    }

    details
}

fn dead_code_explanation(finding: &DeadCodeFinding) -> String {
    match finding.category {
        DeadCodeCategory::UnusedPrivateFunction => format!(
            "Private function `{}` has no incoming resolved call edges in the semantic graph.",
            finding.name
        ),
        DeadCodeCategory::UnusedImport => format!(
            "Import `{}` resolves structurally but is not referenced by any non-import edge.",
            finding.name
        ),
    }
}

fn dead_code_kind(category: DeadCodeCategory) -> String {
    match category {
        DeadCodeCategory::UnusedPrivateFunction => String::from("unused_private_function"),
        DeadCodeCategory::UnusedImport => String::from("unused_import"),
    }
}

fn hardwiring_explanation(finding: &HardwiringFinding) -> String {
    match finding.category {
        HardwiringCategory::MagicString => format!(
            "Literal `{}` appears in a direct comparison and looks hardwired.",
            finding.value
        ),
        HardwiringCategory::RepeatedLiteral => format!(
            "Literal `{}` appears repeatedly and may want central configuration or a named constant.",
            finding.value
        ),
        HardwiringCategory::HardcodedNetwork => format!(
            "Network location `{}` is hardcoded in source.",
            finding.value
        ),
        HardwiringCategory::EnvOutsideConfig => {
            String::from("Environment access appears outside a config-like file path.")
        }
    }
}

fn hardwiring_kind(category: HardwiringCategory) -> String {
    match category {
        HardwiringCategory::MagicString => String::from("magic_string"),
        HardwiringCategory::RepeatedLiteral => String::from("repeated_literal"),
        HardwiringCategory::HardcodedNetwork => String::from("hardcoded_network"),
        HardwiringCategory::EnvOutsideConfig => String::from("env_outside_config"),
    }
}

fn external_explanation(finding: &ExternalFinding) -> String {
    match finding.domain.as_str() {
        "security" => format!(
            "{} reported a security-relevant {} finding.",
            finding.tool, finding.category
        ),
        _ => format!(
            "{} reported an external {} finding.",
            finding.tool, finding.category
        ),
    }
}

fn external_kind(finding: &ExternalFinding) -> String {
    finding.category.clone()
}

fn security_explanation(finding: &SecurityFinding) -> String {
    format!(
        "AigisCode detected a native dangerous API pattern in this file: {}",
        finding.message
    )
}

fn security_kind(category: SecurityCategory) -> String {
    match category {
        SecurityCategory::CommandExecution => String::from("dangerous_command_execution"),
        SecurityCategory::CodeInjection => String::from("dangerous_code_injection"),
        SecurityCategory::UnsafeDeserialization => String::from("unsafe_deserialization"),
        SecurityCategory::UnsafeHtmlOutput => String::from("unsafe_html_output"),
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

fn review_policy_status_label(status: ReviewPolicyStatus) -> String {
    match status {
        ReviewPolicyStatus::None => String::from("none"),
        ReviewPolicyStatus::AcceptedByPolicy => String::from("accepted_by_policy"),
        ReviewPolicyStatus::ExcludedByRule => String::from("excluded_by_rule"),
    }
}

pub fn family_matches(family: &str, filter: Option<FindingFamilyFilter>) -> bool {
    match filter {
        None => true,
        Some(FindingFamilyFilter::Graph) => family == "graph",
        Some(FindingFamilyFilter::DeadCode) => family == "dead_code",
        Some(FindingFamilyFilter::Hardwiring) => family == "hardwiring",
        Some(FindingFamilyFilter::Security) => family == "security",
        Some(FindingFamilyFilter::External) => family == "external",
    }
}

pub fn severity_matches(severity: &str, filter: Option<FindingSeverityFilter>) -> bool {
    match filter {
        None => true,
        Some(FindingSeverityFilter::High) => severity == "high",
        Some(FindingSeverityFilter::Medium) => severity == "medium",
        Some(FindingSeverityFilter::Low) => severity == "low",
    }
}

pub fn path_matches(finding: &FindingSummaryOutput, file_path: Option<&str>) -> bool {
    file_path.is_none_or(|file_path| {
        finding
            .file_paths
            .iter()
            .any(|path| path.contains(file_path))
    })
}

pub fn language_matches(finding: &FindingSummaryOutput, language: Option<&str>) -> bool {
    language.is_none_or(|language| {
        let expected = language.to_ascii_lowercase();
        finding.languages.iter().any(|entry| entry == &expected)
    })
}

fn infer_languages_from_strings(paths: &[String]) -> Vec<String> {
    let mut languages = paths
        .iter()
        .filter_map(|path| infer_language_from_path(Path::new(path)))
        .collect::<Vec<_>>();
    languages.sort();
    languages.dedup();
    languages
}

fn infer_language_from_path(path: &Path) -> Option<String> {
    match path.extension().and_then(|value| value.to_str()) {
        Some("rs") => Some(String::from("rust")),
        Some("py") => Some(String::from("python")),
        Some("php") => Some(String::from("php")),
        Some("rb") => Some(String::from("ruby")),
        Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => Some(String::from("javascript")),
        Some("ts") | Some("tsx") => Some(String::from("typescript")),
        Some("vue") => Some(String::from("vue")),
        _ => None,
    }
}

fn quality_suspects(
    analysis: &ProjectAnalysis,
    surface: &ArchitectureSurface,
) -> Vec<QualitySuspectOutput> {
    let mut suspects = Vec::new();

    for cycle in analysis.graph_analysis.strong_cycle_findings.iter().take(3) {
        for file in cycle.files.iter().take(2) {
            suspects.push(QualitySuspectOutput {
                category: String::from("architecture"),
                file_path: display_path(file),
                reason: format!(
                    "{} cycle with {} internal edges.",
                    cycle_class_label(cycle.cycle_class),
                    cycle.edge_count
                ),
                evidence_score: cycle.edge_count as u32,
            });
        }
    }

    for hotspot in surface.hotspots.iter().take(5) {
        if hotspot.finding_count < 5 && hotspot.bottleneck_centrality_millis < 100 {
            continue;
        }
        suspects.push(QualitySuspectOutput {
            category: String::from("logic_concentration"),
            file_path: display_path(&hotspot.file_path),
            reason: format!(
                "{} findings with bottleneck centrality {}.",
                hotspot.finding_count, hotspot.bottleneck_centrality_millis
            ),
            evidence_score: hotspot.finding_count as u32 + hotspot.bottleneck_centrality_millis,
        });
    }

    for finding in analysis
        .hardwiring
        .findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.category,
                HardwiringCategory::HardcodedNetwork | HardwiringCategory::EnvOutsideConfig
            )
        })
        .take(5)
    {
        suspects.push(QualitySuspectOutput {
            category: String::from("security"),
            file_path: display_path(&finding.file_path),
            reason: hardwiring_explanation(finding),
            evidence_score: 1000,
        });
    }

    suspects.sort_by(|left, right| {
        right
            .evidence_score
            .cmp(&left.evidence_score)
            .then(left.file_path.cmp(&right.file_path))
            .then(left.category.cmp(&right.category))
    });
    suspects.truncate(12);
    suspects
}

fn top_dead_code_files(findings: &[DeadCodeFinding], limit: usize) -> Vec<String> {
    top_counted_paths(
        findings
            .iter()
            .map(|finding| finding.file_path.clone())
            .collect::<Vec<_>>(),
        limit,
    )
}

fn top_hardwiring_files(findings: &[HardwiringFinding], limit: usize) -> Vec<String> {
    top_counted_paths(
        findings
            .iter()
            .map(|finding| finding.file_path.clone())
            .collect::<Vec<_>>(),
        limit,
    )
}

fn supporting_cycle_files(cycles: &[CycleFinding], limit: usize) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut files = Vec::new();
    for cycle in cycles {
        for file in &cycle.files {
            let display = display_path(file);
            if seen.insert(display.clone()) {
                files.push(display);
                if files.len() == limit {
                    return files;
                }
            }
        }
    }
    files
}

fn security_supporting_files(analysis: &ProjectAnalysis) -> Vec<String> {
    let mut paths = analysis
        .external_analysis
        .findings
        .iter()
        .filter_map(|finding| finding.file_path.clone())
        .collect::<Vec<_>>();
    paths.extend(
        analysis
            .hardwiring
            .findings
            .iter()
            .filter(|finding| {
                matches!(
                    finding.category,
                    HardwiringCategory::HardcodedNetwork | HardwiringCategory::EnvOutsideConfig
                )
            })
            .map(|finding| finding.file_path.clone()),
    );
    top_counted_paths(paths, 5)
}

fn top_counted_paths(paths: Vec<PathBuf>, limit: usize) -> Vec<String> {
    let mut counts = HashMap::<String, usize>::new();
    for path in paths {
        *counts.entry(display_path(&path)).or_default() += 1;
    }
    let mut ranked = counts.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    ranked
        .into_iter()
        .take(limit)
        .map(|(path, _)| path)
        .collect()
}

fn architecture_pressure_severity(count: usize) -> &'static str {
    count_severity(count, 1, 4)
}

fn count_severity(count: usize, medium_threshold: usize, high_threshold: usize) -> &'static str {
    if count >= high_threshold {
        "high"
    } else if count >= medium_threshold {
        "medium"
    } else {
        "low"
    }
}

fn cycle_class_label(cycle_class: CycleClass) -> &'static str {
    match cycle_class {
        CycleClass::Structural => "Structural",
        CycleClass::Runtime => "Runtime",
        CycleClass::Framework => "Framework",
        CycleClass::PolicyOverlay => "Policy Overlay",
        CycleClass::Mixed => "Mixed",
        CycleClass::ProbableArtifact => "Probable Artifact",
    }
}

fn graph_layer_label(layer: &GraphLayer) -> &'static str {
    match layer {
        GraphLayer::Structural => "structural",
        GraphLayer::Runtime => "runtime",
        GraphLayer::Framework => "framework",
        GraphLayer::PolicyOverlay => "policy_overlay",
    }
}

fn relation_kind_label(kind: &RelationKind) -> &'static str {
    match kind {
        RelationKind::Import => "import",
        RelationKind::Call => "call",
        RelationKind::Dispatch => "dispatch",
        RelationKind::ContainerResolution => "container_resolution",
        RelationKind::EventSubscribe => "event_subscribe",
        RelationKind::EventPublish => "event_publish",
        RelationKind::TypeUse => "type_use",
        RelationKind::Extends => "extends",
        RelationKind::Implements => "implements",
        RelationKind::Overrides => "overrides",
    }
}

fn doctrine_category_label(category: DoctrineCategory) -> &'static str {
    match category {
        DoctrineCategory::Architecture => "architecture",
        DoctrineCategory::ChangeGovernance => "change_governance",
        DoctrineCategory::Configuration => "configuration",
        DoctrineCategory::Maintainability => "maintainability",
        DoctrineCategory::MechanismChoice => "mechanism_choice",
        DoctrineCategory::Security => "security",
    }
}

fn doctrine_disposition_label(disposition: DoctrineDisposition) -> &'static str {
    match disposition {
        DoctrineDisposition::Inform => "inform",
        DoctrineDisposition::Warn => "warn",
        DoctrineDisposition::Block => "block",
    }
}

pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::build_finding_details;
    use crate::assessment::{
        ArchitecturalAssessment, ArchitecturalAssessmentFinding, ArchitecturalAssessmentKind,
    };
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use crate::policy::PolicyBundle;
    use crate::review::build_review_surface;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn build_finding_details_covers_architectural_assessment_ids() {
        let fixture = unique_fixture_dir("mcp-architectural-detail");
        fs::create_dir_all(fixture.join("app/Services/Filter")).unwrap();
        fs::write(
            fixture.join("app/Services/Filter/QueryContractParser.php"),
            br#"<?php
final class QueryContractParser {
    public function parse(string $input): array {
        $parts = explode(':', trim($input));
        return ['left' => $parts[0] ?? '', 'right' => $parts[1] ?? ''];
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Services/Filter/FilterValidator.php"),
            br#"<?php
final class FilterValidator {
    public function validate(string $input): bool {
        return preg_match('/^[a-z:]+$/', trim($input)) === 1;
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Services/Filter/OrderByValidator.php"),
            br#"<?php
final class OrderByValidator {
    public function validate(string $input): bool {
        return preg_match('/^[a-z_]+$/', trim($input)) === 1;
    }
}
"#,
        )
        .unwrap();

        let mut analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        analysis.architectural_assessment = ArchitecturalAssessment {
            findings: vec![ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::HandRolledParsing,
                file_path: PathBuf::from("app/Services/Filter/QueryContractParser.php"),
                related_file_paths: vec![
                    PathBuf::from("app/Services/Filter/FilterValidator.php"),
                    PathBuf::from("app/Services/Filter/OrderByValidator.php"),
                ],
                related_identifiers: vec![
                    String::from("directory:app/services/filter"),
                    String::from("role:parser"),
                    String::from("role:validator"),
                ],
                warning_count: 0,
                warning_weight: 0,
                bottleneck_centrality_millis: 540,
                warning_families: vec![
                    String::from("parsing_role:parser"),
                    String::from("parsing_role:validator"),
                    String::from("concern:custom_parsing"),
                ],
                severity_millis: 1000,
                fingerprint: String::from("architecture|hand-rolled-parsing|fixture"),
            }],
        };
        let surface = analysis.architecture_surface();
        let review_surface = build_review_surface(
            &analysis,
            &surface,
            &PolicyBundle::load(&analysis.root).unwrap(),
        );
        let details = build_finding_details(
            &analysis,
            &surface,
            &review_surface,
            "aigiscode://repo/current/finding/",
        );
        let finding_id = surface
            .highlights
            .iter()
            .find(|highlight| {
                highlight
                    .id
                    .starts_with("architecture:hand-rolled-parsing:")
            })
            .unwrap()
            .id
            .clone();
        let detail = details.get(&finding_id).unwrap();
        assert_eq!(detail.evidence_kind, "architectural_assessment");
        assert!(detail
            .explanation
            .contains("custom parsing or mini-protocol stack"));
    }

    fn unique_fixture_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("aigiscode-{label}-{unique}"))
    }
}
