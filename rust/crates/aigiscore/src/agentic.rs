use crate::artifacts::{
    AgentHandoffArtifact, AgentHandoffFinding, ConvergenceHistoryArtifact, ConvergenceStatus,
    GuardDecisionArtifact, GuardVerdict, GuardianObligation, GuardianPacket,
};
use crate::doctrine::DoctrineRegistry;
use crate::evidence::EvidenceAnchor;
use crate::graph::{ReferenceKind, RelationKind, ResolvedEdge, SemanticGraph, SymbolNode};
use crate::identity::stable_fingerprint;
use crate::ingestion::pipeline::ProjectAnalysis;
use regex::Regex;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgenticReviewArtifact {
    pub root: String,
    pub contract_version: String,
    pub transport: AgenticTransportContract,
    pub execution: AgenticExecutionContract,
    pub graph_priority: AgenticGraphPriority,
    pub summary: AgenticReviewSummary,
    pub diff_summary: AgenticDiffSummary,
    pub context_artifacts: Vec<AgenticContextArtifact>,
    pub system_prompt: String,
    pub user_prompt: String,
    pub task_packets: Vec<AgenticTaskPacket>,
    pub guardian_packets: Vec<GuardianPacket>,
    pub top_findings: Vec<AgentHandoffFinding>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphPacketArtifact {
    pub root: String,
    pub contract_version: String,
    pub summary: GraphPacketSummary,
    pub packets: Vec<GraphPacket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphPacketSummary {
    pub total_packets: usize,
    pub guardian_task_packets: usize,
    pub fallback_file_packets: usize,
    pub top_anchor_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphPacket {
    pub id: String,
    pub kind: GraphPacketKind,
    pub title: String,
    pub summary: String,
    pub primary_file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_anchor: Option<EvidenceAnchor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence_anchors: Vec<EvidenceAnchor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<EvidenceAnchor>,
    pub evidence_refs: AgenticPrimaryEvidenceRefs,
    pub doctrine_refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_mechanism: Option<String>,
    pub obligations: Vec<GuardianObligation>,
    pub relation_histogram: Vec<GraphRelationCount>,
    pub neighbors: Vec<GraphNeighbor>,
    pub graph_traces: Vec<AgenticGraphTrace>,
    pub code_flows: Vec<AgenticCodeFlow>,
    pub source_sink_paths: Vec<AgenticSourceSinkPath>,
    pub semantic_state_flows: Vec<AgenticSemanticStateFlow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GraphPacketKind {
    GuardianTask,
    FocusFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphRelationCount {
    pub relation_kind: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphNeighbor {
    pub file_path: String,
    pub direction: GraphNeighborDirection,
    pub edge_count: usize,
    pub aggregate_confidence_millis: u16,
    pub relation_histogram: Vec<GraphRelationCount>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GraphNeighborDirection {
    Outbound,
    Inbound,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticTransportContract {
    pub provider_family: String,
    pub recommended_protocol: String,
    pub recommended_auth: String,
    pub recommended_default_model: String,
    pub recommended_coding_models: Vec<String>,
    pub recommended_tool_runtime: String,
    pub supports_background_responses: bool,
    pub shell_tool_supported: bool,
    pub browser_oauth_supported_as_primary: bool,
    pub official_rust_sdk_documented: bool,
    pub official_codex_sdk_strategy: String,
    pub implementation_guidance: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticExecutionContract {
    pub provider: String,
    pub delivery_mode: String,
    pub auth_env_var: String,
    pub preferred_local_adapter: AgenticAdapterId,
    pub preferred_service_adapter: AgenticAdapterId,
    pub adapters: Vec<AgenticAdapterPlan>,
    pub report_targets: Vec<AgenticReportTarget>,
    pub structured_output: AgenticStructuredOutputContract,
    pub openai_responses: AgenticOpenAiExecutionPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgenticAdapterId {
    CodexExecCli,
    OpenAiResponsesHttp,
    CodexSdkTypeScript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgenticAuthMode {
    ApiKey,
    ChatGptOAuth,
    SavedCliAuth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgenticAdapterRuntime {
    LocalCli,
    RustHttp,
    TypeScriptSidecar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticAdapterPlan {
    pub id: AgenticAdapterId,
    pub runtime: AgenticAdapterRuntime,
    pub auth_modes: Vec<AgenticAuthMode>,
    pub supports_structured_output: bool,
    pub supports_background: bool,
    pub purpose: String,
    pub invocation: AgenticAdapterInvocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgenticAdapterInvocation {
    CodexExecCli {
        binary: String,
        subcommand: String,
        default_model: String,
        schema_flag: String,
        output_file_flag: String,
        json_events_flag: String,
        default_sandbox: String,
        required_context_artifacts: Vec<String>,
    },
    OpenAiResponsesHttp {
        endpoint: String,
        method: String,
        model: String,
        reasoning_effort: String,
        background: bool,
        tool_profile: String,
        tool_recommendations: Vec<String>,
        required_context_artifacts: Vec<String>,
    },
    CodexSdkTypeScript {
        package_name: String,
        node_runtime: String,
        default_model: String,
        transport_bridge: String,
        required_context_artifacts: Vec<String>,
        auth_note: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticReportTarget {
    pub file_name: String,
    pub format: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticStructuredOutputContract {
    pub schema_name: String,
    pub schema_version: String,
    pub must_cover_task_packets: Vec<String>,
    pub required_markdown_sections: Vec<String>,
    pub json_schema: JsonValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticOpenAiExecutionPlan {
    pub endpoint: String,
    pub method: String,
    pub model: String,
    pub reasoning_effort: String,
    pub background: bool,
    pub instructions: String,
    pub input_messages: Vec<AgenticExecutionMessage>,
    pub tool_profile: String,
    pub tool_recommendations: Vec<String>,
    pub required_context_artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticExecutionMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticGraphPriority {
    pub architecture_source: String,
    pub evidence_source: String,
    pub runtime_contract_source: String,
    pub doctrine_source: String,
    pub guard_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticReviewSummary {
    pub guard_verdict: String,
    pub visible_findings: usize,
    pub guardian_packet_count: usize,
    pub top_focus_files: Vec<String>,
    pub doctrine_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticDiffSummary {
    pub new_findings: usize,
    pub worsened_findings: usize,
    pub improved_findings: usize,
    pub resolved_findings: usize,
    pub attention_items: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticContextArtifact {
    pub file: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticTaskPacket {
    pub id: String,
    pub status: String,
    pub priority: String,
    pub focus: String,
    pub title: String,
    pub summary: String,
    pub primary_target_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_anchor: Option<EvidenceAnchor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence_anchors: Vec<EvidenceAnchor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<EvidenceAnchor>,
    pub evidence_refs: AgenticPrimaryEvidenceRefs,
    pub doctrine_refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_mechanism: Option<String>,
    pub obligations: Vec<GuardianObligation>,
    pub required_artifacts: Vec<String>,
    pub evidence_chain: AgenticEvidenceChain,
    pub review_radius_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticEvidenceChain {
    pub claim: String,
    pub artifact_refs: Vec<String>,
    pub locations: Vec<AgenticEvidenceLocation>,
    pub graph_traces: Vec<AgenticGraphTrace>,
    pub code_flows: Vec<AgenticCodeFlow>,
    pub source_sink_paths: Vec<AgenticSourceSinkPath>,
    pub semantic_state_flows: Vec<AgenticSemanticStateFlow>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticPrimaryEvidenceRefs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_graph_trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_code_flow_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_source_sink_path_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_semantic_state_flow_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticEvidenceLocation {
    pub role: String,
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticGraphTrace {
    pub id: String,
    pub label: String,
    pub kind: AgenticGraphTraceKind,
    pub primary_file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supporting_file_path: Option<String>,
    pub aggregate_confidence_millis: u16,
    pub relation_sequence: Vec<String>,
    pub truncated: bool,
    pub hops: Vec<AgenticGraphTraceHop>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgenticGraphTraceKind {
    DirectedSupportPath,
    ReverseSupportPath,
    ContextualSupportPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticGraphTraceHop {
    pub source_file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_symbol_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_symbol_name: Option<String>,
    pub target_file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_symbol_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_symbol_name: Option<String>,
    pub relation_kind: String,
    pub layer: String,
    pub origin: String,
    pub strength: String,
    pub resolution_tier: String,
    pub line: usize,
    pub confidence_millis: u16,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticCodeFlow {
    pub id: String,
    pub graph_trace_id: String,
    pub label: String,
    pub kind: AgenticCodeFlowKind,
    pub trace_kind: AgenticGraphTraceKind,
    pub entry_file_path: String,
    pub exit_file_path: String,
    pub source: AgenticPathEndpoint,
    pub sink: AgenticPathEndpoint,
    pub aggregate_confidence_millis: u16,
    pub relation_sequence: Vec<String>,
    pub truncated: bool,
    pub supporting_locations: Vec<AgenticEvidenceLocation>,
    pub steps: Vec<AgenticCodeFlowStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgenticCodeFlowKind {
    ForwardPropagation,
    BackwardPropagation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticCodeFlowStep {
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_to_next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer_to_next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_to_next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength_to_next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_tier_to_next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_millis_to_next: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticSourceSinkPath {
    pub id: String,
    pub graph_trace_id: String,
    pub code_flow_id: String,
    pub label: String,
    pub kind: AgenticSourceSinkKind,
    pub source: AgenticPathEndpoint,
    pub sink: AgenticPathEndpoint,
    pub aggregate_confidence_millis: u16,
    pub relation_sequence: Vec<String>,
    pub truncated: bool,
    pub supporting_locations: Vec<AgenticEvidenceLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticSemanticStateFlow {
    pub id: String,
    pub label: String,
    pub carrier_type: String,
    pub slot: String,
    pub kind: AgenticSemanticStateFlowKind,
    pub writer: AgenticSemanticStateEvent,
    pub reader: AgenticSemanticStateEvent,
    pub aggregate_confidence_millis: u16,
    pub supporting_locations: Vec<AgenticEvidenceLocation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgenticSemanticStateFlowKind {
    DirectWriteRead,
    Derived,
    ResetBeforeRead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticSemanticStateEvent {
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    pub api: String,
    pub role: AgenticSemanticStateEventRole,
    pub proof: SemanticStateProofKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgenticSemanticStateEventRole {
    Writer,
    Reader,
    Reset,
    Deriver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgenticSourceSinkKind {
    PrimaryToSupporting,
    SupportingToPrimary,
    Contextual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticPathEndpoint {
    pub role: String,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticStructuredReviewResponse {
    pub verdict: String,
    pub summary: String,
    pub claims: Vec<AgenticStructuredClaim>,
    pub next_actions: Vec<String>,
    pub report_markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticStructuredClaim {
    pub task_packet_id: String,
    pub title: String,
    pub severity: String,
    pub why_now: String,
    pub recommended_action: String,
    pub evidence_locations: Vec<AgenticStructuredEvidenceLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgenticStructuredEvidenceLocation {
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ListGraphPacketsParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphNeighborsParams {
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphTraceParams {
    pub start_file_path: String,
    pub goal_file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_hops: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_paths: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphNeighborsOutput {
    pub file_path: String,
    pub neighbors: Vec<GraphNeighbor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphTraceOutput {
    pub start_file_path: String,
    pub goal_file_path: String,
    pub paths: Vec<AgenticGraphTrace>,
}

pub fn focus_agentic_review_artifact(
    review: &AgenticReviewArtifact,
    packet_id: &str,
) -> Option<AgenticReviewArtifact> {
    let packet = review
        .task_packets
        .iter()
        .find(|packet| packet.id == packet_id)?
        .clone();
    let mut narrowed = review.clone();
    narrowed.user_prompt = format!(
        "{}\n\nFocus this run on exactly one task packet: {}. Primary target file: {}. Do not widen scope beyond the provided packet unless needed to explain supporting evidence or doctrine obligations.",
        review.user_prompt, packet.id, packet.primary_target_file
    );
    narrowed.summary.top_focus_files = vec![packet.primary_target_file.clone()];
    narrowed.summary.guardian_packet_count = usize::from(
        review
            .guardian_packets
            .iter()
            .any(|guardian| guardian.id == packet.id),
    );
    narrowed.task_packets = vec![packet.clone()];
    narrowed.guardian_packets = review
        .guardian_packets
        .iter()
        .filter(|guardian| guardian.id == packet.id)
        .cloned()
        .collect();
    narrowed.top_findings = review
        .top_findings
        .iter()
        .filter(|finding| {
            finding
                .file_paths
                .iter()
                .any(|path| path == &packet.primary_target_file)
        })
        .cloned()
        .collect();
    narrowed.next_steps = packet
        .obligations
        .iter()
        .map(|obligation| format!("{} -> {}", obligation.action, obligation.acceptance))
        .collect();
    narrowed.execution.structured_output.must_cover_task_packets = vec![packet.id];
    Some(narrowed)
}

pub fn build_agentic_review_artifact(
    analysis: &ProjectAnalysis,
    doctrine_registry: &DoctrineRegistry,
    handoff: &AgentHandoffArtifact,
    guard_decision: &GuardDecisionArtifact,
    convergence: &ConvergenceHistoryArtifact,
) -> AgenticReviewArtifact {
    let review_started_at = Instant::now();
    trace_agentic_step(
        "review.enter",
        0,
        Some(format!(
            "guardian_packets={} top_findings={}",
            handoff.guardian_packets.len(),
            handoff.top_findings.len()
        )),
    );
    let context = AgenticAnalysisContext::new(analysis);
    let mut focus_files = handoff
        .guardian_packets
        .iter()
        .flat_map(|packet| packet.target_files.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if focus_files.is_empty() {
        focus_files = handoff
            .top_findings
            .iter()
            .flat_map(|finding| finding.file_paths.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
    }
    focus_files.truncate(8);
    let doctrine_refs = handoff
        .guardian_packets
        .iter()
        .flat_map(|packet| packet.doctrine_refs.iter().cloned())
        .chain(guard_decision.doctrine_refs.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(10)
        .collect::<Vec<_>>();
    let system_prompt =
        build_system_prompt(doctrine_registry, &guard_decision.verdict, &focus_files);
    let task_packets_started_at = Instant::now();
    let task_packets = build_task_packets(analysis, handoff, guard_decision, convergence, &context);
    trace_agentic_step(
        "task_packets.build",
        task_packets_started_at.elapsed().as_millis(),
        Some(format!("count={}", task_packets.len())),
    );
    let user_prompt = build_user_prompt(
        analysis,
        handoff,
        guard_decision,
        convergence,
        &focus_files,
        &doctrine_refs,
        &task_packets,
    );
    let context_artifacts = vec![
        AgenticContextArtifact {
            file: String::from("dependency-graph.json"),
            role: String::from("low-noise architecture graph"),
        },
        AgenticContextArtifact {
            file: String::from("evidence-graph.json"),
            role: String::from("detailed call-site and runtime evidence"),
        },
        AgenticContextArtifact {
            file: String::from("contract-inventory.json"),
            role: String::from("declared routes, hooks, config keys, and runtime contracts"),
        },
        AgenticContextArtifact {
            file: String::from("doctrine-registry.json"),
            role: String::from("sanctioned mechanisms and guard doctrine"),
        },
        AgenticContextArtifact {
            file: String::from("architecture-surface.json"),
            role: String::from("architecture-facing summary and anchored highlights"),
        },
        AgenticContextArtifact {
            file: String::from("review-surface.json"),
            role: String::from("visible findings and policy/rule overlays"),
        },
        AgenticContextArtifact {
            file: String::from("guard-decision.json"),
            role: String::from("current allow/warn/block judgment"),
        },
        AgenticContextArtifact {
            file: String::from("aigiscode-handoff.json"),
            role: String::from("guardian packets and next-step claims"),
        },
        AgenticContextArtifact {
            file: String::from("graph-packets.json"),
            role: String::from("bounded graph neighborhoods for top packets and focus files"),
        },
        AgenticContextArtifact {
            file: String::from("repository-topology.json"),
            role: String::from("flatter zone-level topology, triage starts, and causal bridges"),
        },
    ];
    let execution = build_execution_contract(&system_prompt, &user_prompt, &task_packets);
    trace_agentic_step(
        "review.ready",
        review_started_at.elapsed().as_millis(),
        Some(format!("task_packets={}", task_packets.len())),
    );

    AgenticReviewArtifact {
        root: analysis.root.display().to_string(),
        contract_version: String::from("2026-03-28"),
        transport: AgenticTransportContract {
            provider_family: String::from("openai"),
            recommended_protocol: String::from("responses_api"),
            recommended_auth: String::from("api_key"),
            recommended_default_model: String::from("gpt-5.4"),
            recommended_coding_models: vec![
                String::from("gpt-5.3-codex"),
                String::from("gpt-5.2-codex"),
                String::from("gpt-5.1-codex-max"),
            ],
            recommended_tool_runtime: String::from("responses_api_shell_tool"),
            supports_background_responses: true,
            shell_tool_supported: true,
            browser_oauth_supported_as_primary: false,
            official_rust_sdk_documented: false,
            official_codex_sdk_strategy: String::from("optional_typescript_sidecar"),
            implementation_guidance: vec![
                String::from(
                    "Treat the graph artifacts as the source of truth and keep the provider integration behind a typed Rust adapter.",
                ),
                String::from(
                    "Do not make browser-only Codex OAuth the primary product contract; use a direct API boundary when automation must be reliable.",
                ),
                String::from(
                    "Default to gpt-5.4 for broad code-and-reasoning workflows; use Codex-tuned models only when the task is primarily coding-specific.",
                ),
                String::from(
                    "Use Responses API background runs plus the shell tool for long-running agent loops that must inspect, edit, test, and report on real repositories.",
                ),
                String::from(
                    "If a provider SDK is unavailable in Rust, keep the request/response contract native in Rust and send HTTP requests directly.",
                ),
                String::from(
                    "If the official Codex SDK becomes necessary for local-agent control semantics, isolate it behind a thin TypeScript sidecar instead of leaking Node into the product core.",
                ),
            ],
        },
        execution,
        graph_priority: AgenticGraphPriority {
            architecture_source: String::from("dependency-graph.json"),
            evidence_source: String::from("evidence-graph.json"),
            runtime_contract_source: String::from("contract-inventory.json"),
            doctrine_source: String::from("doctrine-registry.json"),
            guard_source: String::from("guard-decision.json"),
        },
        summary: AgenticReviewSummary {
            guard_verdict: guard_verdict_label(guard_decision.verdict),
            visible_findings: handoff.summary.visible_findings,
            guardian_packet_count: handoff.guardian_packets.len(),
            top_focus_files: focus_files.clone(),
            doctrine_refs,
        },
        diff_summary: AgenticDiffSummary {
            new_findings: convergence.summary.new_findings,
            worsened_findings: convergence.summary.worsened_findings,
            improved_findings: convergence.summary.improved_findings,
            resolved_findings: convergence.summary.resolved_findings,
            attention_items: convergence.attention_items.len(),
        },
        context_artifacts,
        system_prompt,
        user_prompt,
        task_packets,
        guardian_packets: handoff.guardian_packets.clone(),
        top_findings: handoff.top_findings.clone(),
        next_steps: handoff.next_steps.clone(),
    }
}

pub fn build_graph_packet_artifact(
    review: &AgenticReviewArtifact,
    analysis: &ProjectAnalysis,
) -> GraphPacketArtifact {
    let context = AgenticAnalysisContext::new(analysis);
    let mut packets = review
        .task_packets
        .iter()
        .map(|packet| GraphPacket {
            id: packet.id.clone(),
            kind: GraphPacketKind::GuardianTask,
            title: packet.title.clone(),
            summary: packet.summary.clone(),
            primary_file_path: packet.primary_target_file.clone(),
            primary_anchor: packet.primary_anchor.clone(),
            evidence_anchors: packet.evidence_anchors.clone(),
            locations: packet.locations.clone(),
            evidence_refs: packet.evidence_refs.clone(),
            doctrine_refs: packet.doctrine_refs.clone(),
            preferred_mechanism: packet.preferred_mechanism.clone(),
            obligations: packet.obligations.clone(),
            relation_histogram: relation_histogram_for_files(
                &context.graph,
                std::iter::once(packet.primary_target_file.as_str())
                    .chain(
                        packet
                            .evidence_anchors
                            .iter()
                            .map(|anchor| anchor.file_path.to_str().unwrap_or_default()),
                    )
                    .filter(|path| !path.is_empty()),
            ),
            neighbors: graph_neighbors_for_file_with_context(
                &context.graph,
                &packet.primary_target_file,
                8,
            ),
            graph_traces: packet.evidence_chain.graph_traces.clone(),
            code_flows: packet.evidence_chain.code_flows.clone(),
            source_sink_paths: packet.evidence_chain.source_sink_paths.clone(),
            semantic_state_flows: packet.evidence_chain.semantic_state_flows.clone(),
        })
        .collect::<Vec<_>>();

    if packets.is_empty() {
        let fallback_files = if review.summary.top_focus_files.is_empty() {
            analysis
                .semantic_graph
                .files
                .iter()
                .map(|file| file.path.display().to_string())
                .take(5)
                .collect::<Vec<_>>()
        } else {
            review
                .summary
                .top_focus_files
                .iter()
                .take(5)
                .cloned()
                .collect()
        };
        for file_path in fallback_files {
            let neighbors = graph_neighbors_for_file_with_context(&context.graph, &file_path, 8);
            let (graph_traces, code_flows, source_sink_paths, semantic_state_flows) =
                build_focus_file_graph_evidence_with_context(&context, &file_path, &neighbors);
            packets.push(GraphPacket {
                id: format!("focus-file:{file_path}"),
                kind: GraphPacketKind::FocusFile,
                title: format!("Focused graph packet for {file_path}"),
                summary: format!(
                    "Bounded repository neighborhood for {file_path} derived from top-focus files."
                ),
                primary_file_path: file_path.clone(),
                primary_anchor: None,
                evidence_anchors: Vec::new(),
                locations: Vec::new(),
                evidence_refs: build_primary_evidence_refs(
                    &graph_traces,
                    &code_flows,
                    &source_sink_paths,
                    &semantic_state_flows,
                ),
                doctrine_refs: review.summary.doctrine_refs.clone(),
                preferred_mechanism: None,
                obligations: Vec::new(),
                relation_histogram: relation_histogram_for_files(
                    &context.graph,
                    std::iter::once(file_path.as_str()),
                ),
                neighbors,
                graph_traces,
                code_flows,
                source_sink_paths,
                semantic_state_flows,
            });
        }
    }

    let top_anchor_files = packets
        .iter()
        .map(|packet| packet.primary_file_path.clone())
        .take(8)
        .collect::<Vec<_>>();

    GraphPacketArtifact {
        root: review.root.clone(),
        contract_version: review.contract_version.clone(),
        summary: GraphPacketSummary {
            total_packets: packets.len(),
            guardian_task_packets: packets
                .iter()
                .filter(|packet| packet.kind == GraphPacketKind::GuardianTask)
                .count(),
            fallback_file_packets: packets
                .iter()
                .filter(|packet| packet.kind == GraphPacketKind::FocusFile)
                .count(),
            top_anchor_files,
        },
        packets,
    }
}

#[cfg(test)]
fn build_focus_file_graph_evidence(
    semantic_graph: &SemanticGraph,
    source_lookup: &HashMap<String, &str>,
    primary_file: &str,
    neighbors: &[GraphNeighbor],
) -> (
    Vec<AgenticGraphTrace>,
    Vec<AgenticCodeFlow>,
    Vec<AgenticSourceSinkPath>,
    Vec<AgenticSemanticStateFlow>,
) {
    let context = AgenticAnalysisContext::from_source_lookup(semantic_graph, source_lookup.clone());
    build_focus_file_graph_evidence_with_context(&context, primary_file, neighbors)
}

fn build_focus_file_graph_evidence_with_context(
    context: &AgenticAnalysisContext<'_>,
    primary_file: &str,
    neighbors: &[GraphNeighbor],
) -> (
    Vec<AgenticGraphTrace>,
    Vec<AgenticCodeFlow>,
    Vec<AgenticSourceSinkPath>,
    Vec<AgenticSemanticStateFlow>,
) {
    let mut traces = Vec::new();
    let mut seen_labels = BTreeSet::new();
    for neighbor in neighbors.iter().take(3) {
        if neighbor.file_path == primary_file {
            continue;
        }
        let mut candidate_traces = graph_trace_between_files_with_context(
            &context.graph,
            primary_file,
            &neighbor.file_path,
            4,
            1,
        );
        if candidate_traces.is_empty() {
            candidate_traces = graph_trace_between_files_with_context(
                &context.graph,
                &neighbor.file_path,
                primary_file,
                4,
                1,
            );
        }
        for trace in candidate_traces {
            if trace.hops.is_empty() {
                continue;
            }
            let first_source = trace
                .hops
                .first()
                .map(|hop| hop.source_file_path.as_str())
                .unwrap_or_default();
            let last_target = trace
                .hops
                .last()
                .map(|hop| hop.target_file_path.as_str())
                .unwrap_or_default();
            if first_source == last_target {
                continue;
            }
            if seen_labels.insert(trace.label.clone()) {
                traces.push(trace);
            }
        }
    }
    let code_flows = build_code_flows(&traces);
    let locations = std::iter::once(AgenticEvidenceLocation {
        role: String::from("primary"),
        file_path: primary_file.to_owned(),
        line: None,
    })
    .chain(
        neighbors
            .iter()
            .take(3)
            .map(|neighbor| AgenticEvidenceLocation {
                role: String::from("supporting"),
                file_path: neighbor.file_path.clone(),
                line: None,
            }),
    )
    .collect::<Vec<_>>();
    let source_sink_paths = build_source_sink_paths(&traces, &code_flows, &locations);
    let candidate_files = std::iter::once(primary_file.to_owned())
        .chain(
            neighbors
                .iter()
                .take(3)
                .map(|neighbor| neighbor.file_path.clone()),
        )
        .chain(traces.iter().flat_map(trace_file_paths))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut semantic_state_flows =
        build_semantic_state_flows_with_context(primary_file, &candidate_files, context);
    if semantic_state_flows.is_empty() && context.graph.semantic_graph.files.len() <= 128 {
        let broader_candidate_files = context
            .graph
            .semantic_graph
            .files
            .iter()
            .map(|file| file.path.display().to_string())
            .collect::<Vec<_>>();
        semantic_state_flows = build_semantic_state_flows_with_context(
            primary_file,
            &broader_candidate_files,
            context,
        );
    }
    (traces, code_flows, source_sink_paths, semantic_state_flows)
}

struct AgenticAnalysisContext<'a> {
    source_lookup: HashMap<String, &'a str>,
    graph: GraphQueryContext<'a>,
    file_symbols: HashMap<String, Vec<&'a SymbolNode>>,
    semantic_state_call_site_index: HashMap<String, Vec<SemanticStateCallSite>>,
    semantic_state_carrier_cache: RefCell<HashMap<String, Option<SemanticStateCarrier>>>,
    semantic_state_invocation_cache:
        RefCell<HashMap<(String, String), Vec<SemanticStateInvocation>>>,
    semantic_state_flow_cache: RefCell<HashMap<(String, String), Vec<AgenticSemanticStateFlow>>>,
}

impl<'a> AgenticAnalysisContext<'a> {
    fn from_source_lookup(
        semantic_graph: &'a SemanticGraph,
        source_lookup: HashMap<String, &'a str>,
    ) -> Self {
        Self {
            source_lookup,
            graph: GraphQueryContext::new(semantic_graph),
            file_symbols: build_file_symbol_index(semantic_graph),
            semantic_state_call_site_index: build_semantic_state_call_site_index(semantic_graph),
            semantic_state_carrier_cache: RefCell::new(HashMap::new()),
            semantic_state_invocation_cache: RefCell::new(HashMap::new()),
            semantic_state_flow_cache: RefCell::new(HashMap::new()),
        }
    }

    fn from_graph_and_sources(
        semantic_graph: &'a SemanticGraph,
        parsed_sources: &'a [(std::path::PathBuf, String)],
    ) -> Self {
        Self::from_source_lookup(semantic_graph, build_source_lookup(parsed_sources))
    }

    fn new(analysis: &'a ProjectAnalysis) -> Self {
        Self::from_graph_and_sources(&analysis.semantic_graph, &analysis.parsed_sources)
    }
}

pub struct GraphQueryContext<'a> {
    semantic_graph: &'a SemanticGraph,
    symbol_lookup: HashMap<&'a str, &'a SymbolNode>,
    outbound_edges: HashMap<String, Vec<&'a ResolvedEdge>>,
    incident_edges: HashMap<String, Vec<&'a ResolvedEdge>>,
    trace_cache: RefCell<HashMap<(String, String, usize, usize), Vec<AgenticGraphTrace>>>,
}

impl<'a> GraphQueryContext<'a> {
    pub fn new(semantic_graph: &'a SemanticGraph) -> Self {
        let symbol_lookup = build_symbol_lookup(semantic_graph);
        let mut outbound_edges = HashMap::<String, Vec<&'a ResolvedEdge>>::new();
        let mut incident_edges = HashMap::<String, Vec<&'a ResolvedEdge>>::new();
        for edge in &semantic_graph.resolved_edges {
            let source = edge.source_file_path.display().to_string();
            let target = edge.target_file_path.display().to_string();
            outbound_edges.entry(source.clone()).or_default().push(edge);
            incident_edges.entry(source.clone()).or_default().push(edge);
            if target != source {
                incident_edges.entry(target).or_default().push(edge);
            }
        }
        for edges in outbound_edges.values_mut() {
            edges.sort_by(|left, right| {
                edge_path_priority(right)
                    .cmp(&edge_path_priority(left))
                    .then(left.line.cmp(&right.line))
                    .then(left.target_file_path.cmp(&right.target_file_path))
            });
        }
        Self {
            semantic_graph,
            symbol_lookup,
            outbound_edges,
            incident_edges,
            trace_cache: RefCell::new(HashMap::new()),
        }
    }
}

pub fn build_graph_query_context(semantic_graph: &SemanticGraph) -> GraphQueryContext<'_> {
    GraphQueryContext::new(semantic_graph)
}

fn build_execution_contract(
    system_prompt: &str,
    user_prompt: &str,
    task_packets: &[AgenticTaskPacket],
) -> AgenticExecutionContract {
    let required_context_artifacts = vec![
        String::from("dependency-graph.json"),
        String::from("evidence-graph.json"),
        String::from("contract-inventory.json"),
        String::from("doctrine-registry.json"),
        String::from("guard-decision.json"),
        String::from("aigiscode-handoff.json"),
        String::from("graph-packets.json"),
        String::from("repository-topology.json"),
    ];
    let must_cover_task_packets = task_packets
        .iter()
        .take(12)
        .map(|packet| packet.id.clone())
        .collect::<Vec<_>>();
    let report_targets = vec![
        AgenticReportTarget {
            file_name: String::from("agent-review.md"),
            format: String::from("markdown"),
            purpose: String::from("Human-readable architectonic and security review"),
        },
        AgenticReportTarget {
            file_name: String::from("agent-review.json"),
            format: String::from("json"),
            purpose: String::from("Structured claim/evidence/obligation bundle"),
        },
    ];
    let structured_output = AgenticStructuredOutputContract {
        schema_name: String::from("aigiscode_agentic_review_response"),
        schema_version: String::from("2026-03-28"),
        must_cover_task_packets,
        required_markdown_sections: vec![
            String::from("Verdict"),
            String::from("Top Claims"),
            String::from("Evidence"),
            String::from("Obligations"),
            String::from("Next Actions"),
        ],
        json_schema: normalize_codex_output_schema(
            serde_json::to_value(schema_for!(AgenticStructuredReviewResponse))
                .expect("failed to serialize agentic structured response schema"),
        ),
    };

    AgenticExecutionContract {
        provider: String::from("openai"),
        delivery_mode: String::from("background_responses_job"),
        auth_env_var: String::from("OPENAI_API_KEY"),
        preferred_local_adapter: AgenticAdapterId::CodexExecCli,
        preferred_service_adapter: AgenticAdapterId::OpenAiResponsesHttp,
        adapters: vec![
            AgenticAdapterPlan {
                id: AgenticAdapterId::CodexExecCli,
                runtime: AgenticAdapterRuntime::LocalCli,
                auth_modes: vec![AgenticAuthMode::ApiKey, AgenticAuthMode::SavedCliAuth],
                supports_structured_output: true,
                supports_background: false,
                purpose: String::from(
                    "Best current local operator adapter when Codex CLI auth or CODEX_API_KEY is available and you want a real non-interactive coding agent to inspect the repository and write reports.",
                ),
                invocation: AgenticAdapterInvocation::CodexExecCli {
                    binary: String::from("codex"),
                    subcommand: String::from("exec"),
                    default_model: String::from("gpt-5.3-codex"),
                    schema_flag: String::from("--output-schema"),
                    output_file_flag: String::from("--output-last-message"),
                    json_events_flag: String::from("--json"),
                    default_sandbox: String::from("read-only"),
                    required_context_artifacts: required_context_artifacts.clone(),
                },
            },
            AgenticAdapterPlan {
                id: AgenticAdapterId::OpenAiResponsesHttp,
                runtime: AgenticAdapterRuntime::RustHttp,
                auth_modes: vec![AgenticAuthMode::ApiKey],
                supports_structured_output: true,
                supports_background: true,
                purpose: String::from(
                    "Best backend/service adapter for stable product automation behind a typed Rust boundary.",
                ),
                invocation: AgenticAdapterInvocation::OpenAiResponsesHttp {
                    endpoint: String::from("https://api.openai.com/v1/responses"),
                    method: String::from("POST"),
                    model: String::from("gpt-5.4"),
                    reasoning_effort: String::from("medium"),
                    background: true,
                    tool_profile: String::from("graph_backed_repository_review"),
                    tool_recommendations: vec![
                        String::from("shell"),
                        String::from("apply_patch"),
                    ],
                    required_context_artifacts: required_context_artifacts.clone(),
                },
            },
            AgenticAdapterPlan {
                id: AgenticAdapterId::CodexSdkTypeScript,
                runtime: AgenticAdapterRuntime::TypeScriptSidecar,
                auth_modes: vec![AgenticAuthMode::ApiKey, AgenticAuthMode::ChatGptOAuth],
                supports_structured_output: true,
                supports_background: true,
                purpose: String::from(
                    "Optional thin sidecar when the official TypeScript Codex SDK is required for local-agent control semantics.",
                ),
                invocation: AgenticAdapterInvocation::CodexSdkTypeScript {
                    package_name: String::from("@openai/codex-sdk"),
                    node_runtime: String::from("node18_plus"),
                    default_model: String::from("gpt-5.3-codex"),
                    transport_bridge: String::from("thin_typescript_sidecar_over_stdio_or_jsonl"),
                    required_context_artifacts: required_context_artifacts.clone(),
                    auth_note: String::from(
                        "The official Codex SDK is TypeScript-first. Keep auth/session handling in the sidecar and keep graphing, doctrine, and report validation in Rust.",
                    ),
                },
            },
        ],
        report_targets,
        structured_output,
        openai_responses: AgenticOpenAiExecutionPlan {
            endpoint: String::from("https://api.openai.com/v1/responses"),
            method: String::from("POST"),
            model: String::from("gpt-5.4"),
            reasoning_effort: String::from("medium"),
            background: true,
            instructions: system_prompt.to_owned(),
            input_messages: vec![AgenticExecutionMessage {
                role: String::from("user"),
                content: user_prompt.to_owned(),
            }],
            tool_profile: String::from("graph_backed_repository_review"),
            tool_recommendations: vec![String::from("shell"), String::from("apply_patch")],
            required_context_artifacts,
        },
    }
}

fn normalize_codex_output_schema(mut schema: JsonValue) -> JsonValue {
    normalize_codex_output_schema_value(&mut schema);
    schema
}

fn normalize_codex_output_schema_value(value: &mut JsonValue) {
    match value {
        JsonValue::Object(map) => {
            let is_object_type = match map.get("type") {
                Some(JsonValue::String(kind)) => kind == "object",
                Some(JsonValue::Array(kinds)) => kinds
                    .iter()
                    .any(|kind| matches!(kind, JsonValue::String(name) if name == "object")),
                _ => false,
            };
            if is_object_type && !map.contains_key("additionalProperties") {
                map.insert(String::from("additionalProperties"), JsonValue::Bool(false));
            }
            if is_object_type {
                if let Some(JsonValue::Object(properties)) = map.get("properties") {
                    let mut required = properties.keys().cloned().collect::<Vec<_>>();
                    required.sort();
                    map.insert(
                        String::from("required"),
                        JsonValue::Array(required.into_iter().map(JsonValue::String).collect()),
                    );
                }
            }
            for value in map.values_mut() {
                normalize_codex_output_schema_value(value);
            }
        }
        JsonValue::Array(values) => {
            for value in values {
                normalize_codex_output_schema_value(value);
            }
        }
        _ => {}
    }
}

fn build_system_prompt(
    doctrine_registry: &DoctrineRegistry,
    verdict: &GuardVerdict,
    focus_files: &[String],
) -> String {
    let doctrine_count = doctrine_registry.clauses.len();
    let focus_line = if focus_files.is_empty() {
        String::from("No focus files were preselected; use the graph and guard state to find the right slice.")
    } else {
        format!("Start from these focus files: {}.", focus_files.join(", "))
    };
    format!(
        "You are AigisCode's graph-backed architectural reviewer. Treat dependency-graph.json as low-noise architecture truth, evidence-graph.json as detailed proof, contract-inventory.json as runtime/public contract truth, doctrine-registry.json as sanctioned mechanism doctrine, and guard-decision.json as the current governance state. Prefer graph-backed claims over file-local guesses, do not invent new framework paths when doctrine already names a sanctioned mechanism, and keep recommendations diff-local and architecture-aware. Current guard verdict: {}. Doctrine clauses available: {}. {}",
        guard_verdict_label(*verdict),
        doctrine_count,
        focus_line
    )
}

fn build_user_prompt(
    analysis: &ProjectAnalysis,
    handoff: &AgentHandoffArtifact,
    guard_decision: &GuardDecisionArtifact,
    convergence: &ConvergenceHistoryArtifact,
    focus_files: &[String],
    doctrine_refs: &[String],
    task_packets: &[AgenticTaskPacket],
) -> String {
    let task_summaries = task_packets
        .iter()
        .take(3)
        .map(|packet| {
            format!(
                "{} [{}]: {}",
                packet.primary_target_file, packet.status, packet.summary
            )
        })
        .collect::<Vec<_>>();
    let packet_block = if task_summaries.is_empty() {
        String::from("No task packets are present.")
    } else {
        task_summaries.join(" | ")
    };
    let doctrine_line = if doctrine_refs.is_empty() {
        String::from("No doctrine refs were selected.")
    } else {
        format!("Relevant doctrine refs: {}.", doctrine_refs.join(", "))
    };
    let focus_line = if focus_files.is_empty() {
        String::from("No focus files were preselected.")
    } else {
        format!("Focus files: {}.", focus_files.join(", "))
    };
    format!(
        "Review this repository as an end-to-end architectural analyzer, not a file-by-file linter. Repository summary: {} analyzed files, {} resolved graph edges, {} strong cycle groups, {} architectural smells, {} visible review findings. Diff summary: {} new, {} worsened, {} improved, {} resolved. Guard verdict: {}. {} {} Top graph-backed task packets: {} Next steps already identified: {}",
        analysis.semantic_graph.files.len(),
        analysis.semantic_graph.resolved_edges.len(),
        analysis.graph_analysis.strong_circular_dependencies.len(),
        analysis.graph_analysis.architectural_smells.len(),
        handoff.summary.visible_findings,
        convergence.summary.new_findings,
        convergence.summary.worsened_findings,
        convergence.summary.improved_findings,
        convergence.summary.resolved_findings,
        guard_verdict_label(guard_decision.verdict),
        focus_line,
        doctrine_line,
        packet_block,
        handoff.next_steps.join(" | ")
    )
}

fn build_task_packets(
    _analysis: &ProjectAnalysis,
    handoff: &AgentHandoffArtifact,
    guard_decision: &GuardDecisionArtifact,
    convergence: &ConvergenceHistoryArtifact,
    context: &AgenticAnalysisContext<'_>,
) -> Vec<AgenticTaskPacket> {
    let mut packets = handoff
        .guardian_packets
        .iter()
        .map(|packet| {
            let packet_started_at = Instant::now();
            let status = packet_status(packet, convergence);
            let review_radius_files = guard_decision
                .required_radius
                .anchor_files
                .iter()
                .filter(|path| packet.target_files.contains(*path))
                .take(8)
                .cloned()
                .collect::<Vec<_>>();
            let required_artifacts = vec![
                String::from("dependency-graph.json"),
                String::from("evidence-graph.json"),
                String::from("contract-inventory.json"),
                String::from("doctrine-registry.json"),
                String::from("guard-decision.json"),
                String::from("aigiscode-handoff.json"),
            ];
            let evidence_chain = build_evidence_chain(packet, &required_artifacts, context);
            let task_packet = AgenticTaskPacket {
                id: packet.id.clone(),
                status,
                priority: packet.priority.clone(),
                focus: packet.focus.clone(),
                title: packet.summary.clone(),
                summary: packet.summary.clone(),
                primary_target_file: packet.primary_target_file.clone(),
                primary_anchor: packet.primary_anchor.clone(),
                evidence_anchors: packet.evidence_anchors.clone(),
                locations: packet.locations.clone(),
                evidence_refs: build_primary_evidence_refs(
                    &evidence_chain.graph_traces,
                    &evidence_chain.code_flows,
                    &evidence_chain.source_sink_paths,
                    &evidence_chain.semantic_state_flows,
                ),
                doctrine_refs: packet.doctrine_refs.clone(),
                preferred_mechanism: packet.preferred_mechanism.clone(),
                obligations: packet.obligations.clone(),
                required_artifacts: required_artifacts.clone(),
                evidence_chain,
                review_radius_files,
            };
            trace_agentic_step(
                "task_packet.build",
                packet_started_at.elapsed().as_millis(),
                Some(format!(
                    "id={} file={}",
                    task_packet.id, task_packet.primary_target_file
                )),
            );
            task_packet
        })
        .collect::<Vec<_>>();
    packets.sort_by(|left, right| {
        task_status_rank(&left.status)
            .cmp(&task_status_rank(&right.status))
            .then(left.priority.cmp(&right.priority))
            .then(left.primary_target_file.cmp(&right.primary_target_file))
    });
    packets
}

fn trace_agentic_step(step: &str, elapsed_ms: u128, extra: Option<String>) {
    if std::env::var_os("AIGISCORE_TRACE").is_none() {
        return;
    }
    match extra {
        Some(extra) => eprintln!("[aigiscore] agentic {step} elapsed_ms={elapsed_ms} {extra}"),
        None => eprintln!("[aigiscore] agentic {step} elapsed_ms={elapsed_ms}"),
    }
}

fn build_evidence_chain(
    packet: &GuardianPacket,
    required_artifacts: &[String],
    context: &AgenticAnalysisContext<'_>,
) -> AgenticEvidenceChain {
    let evidence_started_at = Instant::now();
    let mut locations = Vec::new();
    if let Some(anchor) = &packet.primary_anchor {
        locations.push(AgenticEvidenceLocation {
            role: String::from("primary"),
            file_path: anchor.file_path.display().to_string(),
            line: anchor.line,
        });
    }
    locations.extend(
        packet
            .evidence_anchors
            .iter()
            .map(|anchor| AgenticEvidenceLocation {
                role: anchor.label.clone(),
                file_path: anchor.file_path.display().to_string(),
                line: anchor.line,
            }),
    );
    if locations.is_empty() {
        locations.extend(
            packet
                .target_files
                .iter()
                .take(4)
                .map(|path| AgenticEvidenceLocation {
                    role: String::from("context"),
                    file_path: path.clone(),
                    line: None,
                }),
        );
    }

    let traces_started_at = Instant::now();
    let graph_traces = build_graph_traces_with_context(packet, &context.graph);
    trace_agentic_step(
        "evidence.graph_traces",
        traces_started_at.elapsed().as_millis(),
        Some(format!("id={} count={}", packet.id, graph_traces.len())),
    );

    let flows_started_at = Instant::now();
    let code_flows = build_code_flows(&graph_traces);
    trace_agentic_step(
        "evidence.code_flows",
        flows_started_at.elapsed().as_millis(),
        Some(format!("id={} count={}", packet.id, code_flows.len())),
    );
    let source_sink_started_at = Instant::now();
    let source_sink_paths = build_source_sink_paths(&graph_traces, &code_flows, &locations);
    trace_agentic_step(
        "evidence.source_sink_paths",
        source_sink_started_at.elapsed().as_millis(),
        Some(format!(
            "id={} count={}",
            packet.id,
            source_sink_paths.len()
        )),
    );
    let candidate_files = packet
        .target_files
        .iter()
        .cloned()
        .chain(locations.iter().map(|location| location.file_path.clone()))
        .chain(graph_traces.iter().flat_map(trace_file_paths))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let semantic_state_started_at = Instant::now();
    let semantic_state_flows = if should_build_semantic_state_flows(packet, candidate_files.len()) {
        build_semantic_state_flows_with_context(
            &packet.primary_target_file,
            &candidate_files,
            context,
        )
    } else {
        Vec::new()
    };
    trace_agentic_step(
        "evidence.semantic_state_flows",
        semantic_state_started_at.elapsed().as_millis(),
        Some(format!(
            "id={} candidates={} count={}",
            packet.id,
            candidate_files.len(),
            semantic_state_flows.len()
        )),
    );
    trace_agentic_step(
        "evidence.total",
        evidence_started_at.elapsed().as_millis(),
        Some(format!("id={}", packet.id)),
    );

    AgenticEvidenceChain {
        claim: packet.summary.clone(),
        artifact_refs: required_artifacts.to_vec(),
        locations,
        source_sink_paths,
        code_flows,
        graph_traces,
        semantic_state_flows,
    }
}

fn build_primary_evidence_refs(
    graph_traces: &[AgenticGraphTrace],
    code_flows: &[AgenticCodeFlow],
    source_sink_paths: &[AgenticSourceSinkPath],
    semantic_state_flows: &[AgenticSemanticStateFlow],
) -> AgenticPrimaryEvidenceRefs {
    AgenticPrimaryEvidenceRefs {
        primary_graph_trace_id: graph_traces.first().map(|trace| trace.id.clone()),
        primary_code_flow_id: code_flows.first().map(|flow| flow.id.clone()),
        primary_source_sink_path_id: source_sink_paths.first().map(|path| path.id.clone()),
        primary_semantic_state_flow_id: semantic_state_flows.first().map(|flow| flow.id.clone()),
    }
}

fn packet_supports_semantic_state_flows(packet: &GuardianPacket) -> bool {
    packet.focus == "hand_rolled_parsing"
}

fn should_build_semantic_state_flows(packet: &GuardianPacket, candidate_file_count: usize) -> bool {
    packet_supports_semantic_state_flows(packet) && candidate_file_count <= 16
}

#[cfg(test)]
fn build_graph_traces(
    packet: &GuardianPacket,
    semantic_graph: &SemanticGraph,
) -> Vec<AgenticGraphTrace> {
    let context = GraphQueryContext::new(semantic_graph);
    build_graph_traces_with_context(packet, &context)
}

fn build_graph_traces_with_context(
    packet: &GuardianPacket,
    graph_context: &GraphQueryContext<'_>,
) -> Vec<AgenticGraphTrace> {
    let primary = packet.primary_target_file.as_str();
    let target_files = packet.target_files.iter().cloned().collect::<HashSet<_>>();
    let mut traces = Vec::new();
    let mut seen_labels = HashSet::new();
    let search_profile = graph_trace_search_profile(packet);

    for target in packet
        .target_files
        .iter()
        .filter(|target| target.as_str() != primary)
        .take(search_profile.target_limit)
    {
        let directed_paths = directed_graph_traces(
            graph_context,
            primary,
            target,
            search_profile.max_hops,
            search_profile.path_limit,
        );
        if !directed_paths.is_empty() {
            for trace in directed_paths {
                if seen_labels.insert(trace.label.clone()) {
                    traces.push(trace);
                }
            }
            continue;
        }

        let reverse_paths = directed_graph_traces(
            graph_context,
            target,
            primary,
            search_profile.max_hops,
            search_profile.path_limit,
        );
        for trace in reverse_paths.into_iter().map(|mut trace| {
            trace.kind = AgenticGraphTraceKind::ReverseSupportPath;
            trace.primary_file_path = primary.to_owned();
            trace.supporting_file_path = Some(target.clone());
            trace.id = graph_trace_id_for_hops(
                AgenticGraphTraceKind::ReverseSupportPath,
                primary,
                target.as_str(),
                &trace.hops,
            );
            trace
        }) {
            if seen_labels.insert(trace.label.clone()) {
                traces.push(trace);
            }
        }
    }

    if traces.is_empty() {
        for edge in strongest_context_edges_with_context(graph_context, primary, &target_files, 3) {
            let source = edge.source_file_path.display().to_string();
            let target = edge.target_file_path.display().to_string();
            let label = format!("{source} -> {target}");
            if seen_labels.insert(label.clone()) {
                traces.push(AgenticGraphTrace {
                    id: graph_trace_id_for_edges(
                        AgenticGraphTraceKind::ContextualSupportPath,
                        primary,
                        if source == primary { &target } else { &source },
                        &[edge],
                    ),
                    label,
                    kind: AgenticGraphTraceKind::ContextualSupportPath,
                    primary_file_path: primary.to_owned(),
                    supporting_file_path: if source == primary {
                        Some(target.clone())
                    } else {
                        Some(source.clone())
                    },
                    aggregate_confidence_millis: edge.confidence_millis,
                    relation_sequence: vec![format!("{:?}", edge.relation_kind)],
                    truncated: false,
                    hops: build_trace_hops(&[edge], &graph_context.symbol_lookup),
                });
            }
        }
    }

    traces
}

struct GraphTraceSearchProfile {
    max_hops: usize,
    path_limit: usize,
    target_limit: usize,
}

fn graph_trace_search_profile(packet: &GuardianPacket) -> GraphTraceSearchProfile {
    let broad_packet = packet.target_files.len() > 4
        || matches!(
            packet.focus.as_str(),
            "duplicate_mechanism" | "warning_heavy_hotspot"
        );
    GraphTraceSearchProfile {
        max_hops: 5,
        path_limit: if broad_packet { 1 } else { 2 },
        target_limit: if packet.target_files.len() > 8 { 2 } else { 3 },
    }
}

fn build_code_flows(traces: &[AgenticGraphTrace]) -> Vec<AgenticCodeFlow> {
    traces
        .iter()
        .filter(|trace| !trace.hops.is_empty())
        .map(|trace| {
            let mut steps = Vec::new();
            for (index, hop) in trace.hops.iter().enumerate() {
                let next_hop = trace.hops.get(index + 1);
                if index == 0 {
                    steps.push(AgenticCodeFlowStep {
                        file_path: hop.source_file_path.clone(),
                        symbol_name: hop.source_symbol_name.clone(),
                        line: Some(hop.line),
                        relation_to_next: Some(hop.relation_kind.clone()),
                        layer_to_next: Some(hop.layer.clone()),
                        origin_to_next: Some(hop.origin.clone()),
                        strength_to_next: Some(hop.strength.clone()),
                        resolution_tier_to_next: Some(hop.resolution_tier.clone()),
                        confidence_millis_to_next: Some(hop.confidence_millis),
                        next_file_path: Some(hop.target_file_path.clone()),
                    });
                }
                steps.push(AgenticCodeFlowStep {
                    file_path: hop.target_file_path.clone(),
                    symbol_name: hop.target_symbol_name.clone(),
                    line: next_hop.map(|next_hop| next_hop.line).or(Some(hop.line)),
                    relation_to_next: next_hop.map(|next_hop| next_hop.relation_kind.clone()),
                    layer_to_next: next_hop.map(|next_hop| next_hop.layer.clone()),
                    origin_to_next: next_hop.map(|next_hop| next_hop.origin.clone()),
                    strength_to_next: next_hop.map(|next_hop| next_hop.strength.clone()),
                    resolution_tier_to_next: next_hop
                        .map(|next_hop| next_hop.resolution_tier.clone()),
                    confidence_millis_to_next: next_hop.map(|next_hop| next_hop.confidence_millis),
                    next_file_path: next_hop.map(|next_hop| next_hop.target_file_path.clone()),
                });
            }
            let first_hop = trace.hops.first().expect("non-empty trace hops");
            let last_hop = trace.hops.last().expect("non-empty trace hops");
            let source = AgenticPathEndpoint {
                role: source_role(trace.kind).to_owned(),
                file_path: first_hop.source_file_path.clone(),
                line: Some(first_hop.line),
                symbol_name: first_hop.source_symbol_name.clone(),
            };
            let sink = AgenticPathEndpoint {
                role: sink_role(trace.kind).to_owned(),
                file_path: last_hop.target_file_path.clone(),
                line: Some(last_hop.line),
                symbol_name: last_hop.target_symbol_name.clone(),
            };
            let supporting_locations = steps
                .iter()
                .skip(1)
                .take(steps.len().saturating_sub(2))
                .map(|step| AgenticEvidenceLocation {
                    role: String::from("supporting_flow"),
                    file_path: step.file_path.clone(),
                    line: step.line,
                })
                .collect::<Vec<_>>();
            AgenticCodeFlow {
                id: code_flow_id(trace),
                graph_trace_id: trace.id.clone(),
                label: trace.label.clone(),
                kind: match trace.kind {
                    AgenticGraphTraceKind::DirectedSupportPath => {
                        AgenticCodeFlowKind::ForwardPropagation
                    }
                    AgenticGraphTraceKind::ReverseSupportPath
                    | AgenticGraphTraceKind::ContextualSupportPath => {
                        AgenticCodeFlowKind::BackwardPropagation
                    }
                },
                trace_kind: trace.kind,
                entry_file_path: source.file_path.clone(),
                exit_file_path: sink.file_path.clone(),
                source,
                sink,
                aggregate_confidence_millis: trace.aggregate_confidence_millis,
                relation_sequence: trace.relation_sequence.clone(),
                truncated: trace.truncated,
                supporting_locations,
                steps,
            }
        })
        .collect()
}

fn build_source_sink_paths(
    traces: &[AgenticGraphTrace],
    code_flows: &[AgenticCodeFlow],
    locations: &[AgenticEvidenceLocation],
) -> Vec<AgenticSourceSinkPath> {
    let line_lookup = locations
        .iter()
        .filter_map(|location| {
            location
                .line
                .map(|line| (location.file_path.as_str(), line))
        })
        .collect::<HashMap<_, _>>();

    traces
        .iter()
        .zip(code_flows.iter())
        .filter_map(|(trace, flow)| {
            let first_hop = trace.hops.first()?;
            let last_hop = trace.hops.last()?;
            let source = AgenticPathEndpoint {
                role: flow.source.role.clone(),
                file_path: flow.source.file_path.clone(),
                line: flow
                    .source
                    .line
                    .or_else(|| line_lookup.get(flow.source.file_path.as_str()).copied())
                    .or(Some(first_hop.line)),
                symbol_name: flow.source.symbol_name.clone(),
            };
            let sink = AgenticPathEndpoint {
                role: flow.sink.role.clone(),
                file_path: flow.sink.file_path.clone(),
                line: flow
                    .sink
                    .line
                    .or_else(|| line_lookup.get(flow.sink.file_path.as_str()).copied())
                    .or(Some(last_hop.line)),
                symbol_name: flow.sink.symbol_name.clone(),
            };
            Some(AgenticSourceSinkPath {
                id: source_sink_path_id(trace, flow),
                graph_trace_id: trace.id.clone(),
                code_flow_id: flow.id.clone(),
                label: trace.label.clone(),
                kind: match trace.kind {
                    AgenticGraphTraceKind::DirectedSupportPath => {
                        AgenticSourceSinkKind::PrimaryToSupporting
                    }
                    AgenticGraphTraceKind::ReverseSupportPath => {
                        AgenticSourceSinkKind::SupportingToPrimary
                    }
                    AgenticGraphTraceKind::ContextualSupportPath => {
                        AgenticSourceSinkKind::Contextual
                    }
                },
                source,
                sink,
                aggregate_confidence_millis: trace.aggregate_confidence_millis,
                relation_sequence: trace.relation_sequence.clone(),
                truncated: trace.truncated,
                supporting_locations: flow.supporting_locations.clone(),
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SemanticStateApiRole {
    Writer,
    Reader,
    Reset,
    Deriver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SemanticStateSlotSpec {
    Fixed(String),
    KeyedArgument { container: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticStateApiMethod {
    api_name: String,
    slot_spec: SemanticStateSlotSpec,
    role: SemanticStateApiRole,
    line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticStateCarrier {
    file_path: String,
    carrier_type: String,
    is_trait: bool,
    trait_owner_type_names: BTreeSet<String>,
    methods: Vec<SemanticStateApiMethod>,
    method_symbol_ids: BTreeMap<String, BTreeSet<String>>,
    methods_by_symbol_id: BTreeMap<String, SemanticStateApiMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticStateInvocation {
    file_path: String,
    symbol_id: Option<String>,
    symbol_name: Option<String>,
    receiver_name: Option<String>,
    occurrence_index: usize,
    line: usize,
    api_name: String,
    slot: String,
    slot_origin: SemanticStateSlotOrigin,
    role: SemanticStateApiRole,
    carrier_type: String,
    proof_kind: SemanticStateProofKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticStateSlotOrigin {
    Fixed,
    KeyedArgument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticStateCallSite {
    file_path: String,
    enclosing_symbol_id: Option<String>,
    line: usize,
    occurrence_index: usize,
    target_name: String,
    receiver_name: Option<String>,
    normalized_receiver_name: Option<String>,
    receiver_type_name: Option<String>,
    resolved_target_symbol_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HeuristicSemanticStateCallOccurrence {
    receiver_name: Option<String>,
    occurrence_index: usize,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SemanticStateProofKind {
    ExactResolved,
    ReceiverTyped,
    Heuristic,
}

fn build_source_lookup<'a>(
    parsed_sources: &'a [(std::path::PathBuf, String)],
) -> HashMap<String, &'a str> {
    parsed_sources
        .iter()
        .map(|(path, source)| (path.display().to_string(), source.as_str()))
        .collect()
}

fn trace_file_paths(trace: &AgenticGraphTrace) -> impl Iterator<Item = String> + '_ {
    trace
        .hops
        .iter()
        .flat_map(|hop| [hop.source_file_path.clone(), hop.target_file_path.clone()])
}

#[cfg(test)]
fn build_semantic_state_flows(
    primary_file: &str,
    candidate_files: &[String],
    semantic_graph: &SemanticGraph,
    source_lookup: &HashMap<String, &str>,
) -> Vec<AgenticSemanticStateFlow> {
    let context = AgenticAnalysisContext::from_source_lookup(semantic_graph, source_lookup.clone());
    build_semantic_state_flows_with_context(primary_file, candidate_files, &context)
}

fn build_semantic_state_flows_with_context(
    primary_file: &str,
    candidate_files: &[String],
    context: &AgenticAnalysisContext<'_>,
) -> Vec<AgenticSemanticStateFlow> {
    let candidate_set = candidate_files.iter().cloned().collect::<BTreeSet<_>>();
    let cache_key = (
        primary_file.to_owned(),
        candidate_set.iter().cloned().collect::<Vec<_>>().join("|"),
    );
    if let Some(cached) = context.semantic_state_flow_cache.borrow().get(&cache_key) {
        return cached.clone();
    }
    let carriers = candidate_set
        .iter()
        .filter_map(|file_path| cached_semantic_state_carrier(file_path, context))
        .collect::<Vec<_>>();
    if carriers.is_empty() {
        return Vec::new();
    }

    let mut flows = Vec::new();
    let mut seen = BTreeSet::new();

    for carrier in carriers {
        let invocations = candidate_set
            .iter()
            .flat_map(|file_path| {
                cached_semantic_state_invocations_for_file(&carrier, file_path, context)
            })
            .collect::<Vec<_>>();
        if invocations.is_empty() {
            continue;
        }
        let mut slots = invocations
            .iter()
            .map(|invocation| invocation.slot.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        slots.sort();

        for slot in slots {
            let slot_invocations = invocations
                .iter()
                .filter(|invocation| invocation.slot == slot)
                .cloned()
                .collect::<Vec<_>>();
            let mut writers = slot_invocations
                .iter()
                .filter(|invocation| matches!(invocation.role, SemanticStateApiRole::Writer))
                .cloned()
                .collect::<Vec<_>>();
            let mut readers = slot_invocations
                .iter()
                .filter(|invocation| {
                    matches!(
                        invocation.role,
                        SemanticStateApiRole::Reader | SemanticStateApiRole::Deriver
                    )
                })
                .cloned()
                .collect::<Vec<_>>();
            let mut resets = slot_invocations
                .iter()
                .filter(|invocation| matches!(invocation.role, SemanticStateApiRole::Reset))
                .cloned()
                .collect::<Vec<_>>();

            writers.sort_by_key(|invocation| state_event_rank(invocation, primary_file));
            readers.sort_by_key(|invocation| state_event_rank(invocation, primary_file));
            resets.sort_by_key(|invocation| state_event_rank(invocation, primary_file));

            if let Some((writer, reader)) =
                select_semantic_state_pair(&writers, &readers, primary_file)
            {
                let flow = semantic_state_flow_from_pair(
                    &carrier,
                    &slot,
                    &writer,
                    &reader,
                    semantic_flow_kind_for_pair(&writer, &reader),
                );
                if seen.insert(semantic_state_flow_key(&flow)) {
                    flows.push(flow);
                }
            }
            if let Some((reset, reader)) =
                select_semantic_state_pair(&resets, &readers, primary_file)
            {
                let flow = semantic_state_flow_from_pair(
                    &carrier,
                    &slot,
                    &reset,
                    &reader,
                    AgenticSemanticStateFlowKind::ResetBeforeRead,
                );
                if seen.insert(semantic_state_flow_key(&flow)) {
                    flows.push(flow);
                }
            }
        }
    }

    flows.sort_by(|left, right| {
        right
            .aggregate_confidence_millis
            .cmp(&left.aggregate_confidence_millis)
            .then(left.carrier_type.cmp(&right.carrier_type))
            .then(left.slot.cmp(&right.slot))
            .then(left.label.cmp(&right.label))
    });
    flows.truncate(6);
    context
        .semantic_state_flow_cache
        .borrow_mut()
        .insert(cache_key, flows.clone());
    flows
}

fn cached_semantic_state_carrier(
    file_path: &str,
    context: &AgenticAnalysisContext<'_>,
) -> Option<SemanticStateCarrier> {
    if let Some(cached) = context.semantic_state_carrier_cache.borrow().get(file_path) {
        return cached.clone();
    }
    let computed = context.source_lookup.get(file_path).and_then(|source| {
        build_semantic_state_carrier(file_path, source, context.graph.semantic_graph)
    });
    context
        .semantic_state_carrier_cache
        .borrow_mut()
        .insert(file_path.to_owned(), computed.clone());
    computed
}

fn cached_semantic_state_invocations_for_file(
    carrier: &SemanticStateCarrier,
    file_path: &str,
    context: &AgenticAnalysisContext<'_>,
) -> Vec<SemanticStateInvocation> {
    let key = (carrier.file_path.clone(), file_path.to_owned());
    if let Some(cached) = context.semantic_state_invocation_cache.borrow().get(&key) {
        return cached.clone();
    }

    let source = context.source_lookup.get(file_path).copied();
    let mut file_invocations = collect_graph_backed_semantic_state_invocations(
        carrier,
        file_path,
        source,
        &context.file_symbols,
        &context.semantic_state_call_site_index,
    );
    if let Some(source) = source {
        let heuristic_invocations = collect_heuristic_semantic_state_invocations(
            carrier,
            file_path,
            source,
            &context.file_symbols,
            &context.semantic_state_call_site_index,
        );
        let mut seen_graph_invocations = file_invocations
            .iter()
            .map(semantic_state_invocation_key)
            .collect::<BTreeSet<_>>();
        for invocation in heuristic_invocations {
            if seen_graph_invocations.insert(semantic_state_invocation_key(&invocation)) {
                file_invocations.push(invocation);
            }
        }
    }
    context
        .semantic_state_invocation_cache
        .borrow_mut()
        .insert(key, file_invocations.clone());
    file_invocations
}

fn semantic_state_flow_key(flow: &AgenticSemanticStateFlow) -> String {
    flow.id.clone()
}

fn semantic_state_flow_id(
    carrier: &SemanticStateCarrier,
    slot: &str,
    writer: &SemanticStateInvocation,
    reader: &SemanticStateInvocation,
    kind: AgenticSemanticStateFlowKind,
) -> String {
    format!(
        "semantic-state|{}|{}|{}|{}|{}|{}|{}|{}",
        carrier.file_path,
        carrier.carrier_type,
        slot,
        semantic_state_flow_kind_key(kind),
        writer.file_path,
        writer.line,
        reader.file_path,
        reader.line,
    )
}

fn semantic_state_flow_kind_key(kind: AgenticSemanticStateFlowKind) -> &'static str {
    match kind {
        AgenticSemanticStateFlowKind::DirectWriteRead => "direct_write_read",
        AgenticSemanticStateFlowKind::Derived => "derived",
        AgenticSemanticStateFlowKind::ResetBeforeRead => "reset_before_read",
    }
}

fn semantic_state_flow_from_pair(
    carrier: &SemanticStateCarrier,
    slot: &str,
    writer: &SemanticStateInvocation,
    reader: &SemanticStateInvocation,
    kind: AgenticSemanticStateFlowKind,
) -> AgenticSemanticStateFlow {
    let aggregate_confidence_millis = semantic_state_confidence(writer, reader, &carrier.file_path);
    let id = semantic_state_flow_id(carrier, slot, writer, reader, kind);
    let mut seen_supporting = BTreeSet::new();
    let supporting_locations = [
        (
            String::from("carrier"),
            carrier.file_path.clone(),
            carrier
                .methods
                .iter()
                .find(|method| semantic_state_method_matches_slot(method, slot))
                .map(|method| method.line),
        ),
        (
            String::from("writer"),
            writer.file_path.clone(),
            Some(writer.line),
        ),
        (
            String::from("reader"),
            reader.file_path.clone(),
            Some(reader.line),
        ),
    ]
    .into_iter()
    .filter(|(role, file_path, line)| {
        seen_supporting.insert((role.clone(), file_path.clone(), *line))
    })
    .map(|(role, file_path, line)| AgenticEvidenceLocation {
        role,
        file_path,
        line,
    })
    .collect::<Vec<_>>();

    AgenticSemanticStateFlow {
        id,
        label: format!(
            "{}.{}: {} -> {}",
            carrier.carrier_type, slot, writer.api_name, reader.api_name
        ),
        carrier_type: carrier.carrier_type.clone(),
        slot: slot.to_owned(),
        kind,
        writer: AgenticSemanticStateEvent {
            file_path: writer.file_path.clone(),
            symbol_name: writer.symbol_name.clone(),
            line: Some(writer.line),
            api: writer.api_name.clone(),
            role: semantic_event_role(writer),
            proof: writer.proof_kind,
            value_kind: Some(slot.to_owned()),
            path: Some(slot.to_owned()),
        },
        reader: AgenticSemanticStateEvent {
            file_path: reader.file_path.clone(),
            symbol_name: reader.symbol_name.clone(),
            line: Some(reader.line),
            api: reader.api_name.clone(),
            role: semantic_event_role(reader),
            proof: reader.proof_kind,
            value_kind: Some(slot.to_owned()),
            path: Some(slot.to_owned()),
        },
        aggregate_confidence_millis,
        supporting_locations,
    }
}

fn semantic_state_declared_slot(method: &SemanticStateApiMethod) -> String {
    match &method.slot_spec {
        SemanticStateSlotSpec::Fixed(slot) => slot.clone(),
        SemanticStateSlotSpec::KeyedArgument { container } => container.clone(),
    }
}

fn semantic_state_method_matches_slot(method: &SemanticStateApiMethod, slot: &str) -> bool {
    match &method.slot_spec {
        SemanticStateSlotSpec::Fixed(declared_slot) => declared_slot == slot,
        SemanticStateSlotSpec::KeyedArgument { container } => {
            slot == container || slot.starts_with(&format!("{container}["))
        }
    }
}

fn semantic_event_role(invocation: &SemanticStateInvocation) -> AgenticSemanticStateEventRole {
    match invocation.role {
        SemanticStateApiRole::Writer => AgenticSemanticStateEventRole::Writer,
        SemanticStateApiRole::Reader => AgenticSemanticStateEventRole::Reader,
        SemanticStateApiRole::Reset => AgenticSemanticStateEventRole::Reset,
        SemanticStateApiRole::Deriver => AgenticSemanticStateEventRole::Deriver,
    }
}

fn semantic_flow_kind_for_pair(
    _writer: &SemanticStateInvocation,
    reader: &SemanticStateInvocation,
) -> AgenticSemanticStateFlowKind {
    if matches!(reader.role, SemanticStateApiRole::Deriver) {
        AgenticSemanticStateFlowKind::Derived
    } else {
        AgenticSemanticStateFlowKind::DirectWriteRead
    }
}

fn semantic_state_confidence(
    writer: &SemanticStateInvocation,
    reader: &SemanticStateInvocation,
    carrier_file: &str,
) -> u16 {
    let writer_outside = writer.file_path != carrier_file;
    let reader_outside = reader.file_path != carrier_file;
    let mut confidence = 560;
    confidence +=
        match (writer.proof_kind, reader.proof_kind) {
            (SemanticStateProofKind::ExactResolved, SemanticStateProofKind::ExactResolved) => 180,
            (SemanticStateProofKind::ExactResolved, _)
            | (_, SemanticStateProofKind::ExactResolved) => 140,
            (SemanticStateProofKind::ReceiverTyped, SemanticStateProofKind::ReceiverTyped) => 120,
            (SemanticStateProofKind::ReceiverTyped, _)
            | (_, SemanticStateProofKind::ReceiverTyped) => 90,
            _ => 40,
        };
    if !writer_outside || !reader_outside {
        confidence += 80;
    }
    if writer.file_path == reader.file_path {
        confidence += 40;
    } else {
        confidence += 20;
    }
    confidence.min(780)
}

fn state_event_rank(
    invocation: &SemanticStateInvocation,
    primary_file: &str,
) -> (u8, usize, String) {
    let primary_rank = if invocation.file_path == primary_file {
        0
    } else {
        1
    };
    let role_rank = match invocation.role {
        SemanticStateApiRole::Writer => 0,
        SemanticStateApiRole::Reader => 1,
        SemanticStateApiRole::Deriver => 2,
        SemanticStateApiRole::Reset => 3,
    };
    let proof_rank = match invocation.proof_kind {
        SemanticStateProofKind::ExactResolved => 0,
        SemanticStateProofKind::ReceiverTyped => 1,
        SemanticStateProofKind::Heuristic => 2,
    };
    (
        primary_rank + proof_rank + role_rank,
        invocation.line,
        invocation.file_path.clone(),
    )
}

fn select_semantic_state_pair(
    writers: &[SemanticStateInvocation],
    readers: &[SemanticStateInvocation],
    primary_file: &str,
) -> Option<(SemanticStateInvocation, SemanticStateInvocation)> {
    writers
        .iter()
        .flat_map(|writer| readers.iter().map(move |reader| (writer, reader)))
        .min_by_key(|(writer, reader)| semantic_state_pair_rank(writer, reader, primary_file))
        .map(|(writer, reader)| (writer.clone(), reader.clone()))
}

fn semantic_state_pair_rank(
    writer: &SemanticStateInvocation,
    reader: &SemanticStateInvocation,
    primary_file: &str,
) -> (u8, u8, u8, usize, u8, usize, usize, String, String) {
    let proof_rank = match (writer.proof_kind, reader.proof_kind) {
        (SemanticStateProofKind::ExactResolved, SemanticStateProofKind::ExactResolved) => 0,
        (SemanticStateProofKind::ExactResolved, SemanticStateProofKind::ReceiverTyped)
        | (SemanticStateProofKind::ReceiverTyped, SemanticStateProofKind::ExactResolved) => 1,
        (SemanticStateProofKind::ReceiverTyped, SemanticStateProofKind::ReceiverTyped) => 2,
        (SemanticStateProofKind::ExactResolved, SemanticStateProofKind::Heuristic)
        | (SemanticStateProofKind::Heuristic, SemanticStateProofKind::ExactResolved) => 3,
        (SemanticStateProofKind::ReceiverTyped, SemanticStateProofKind::Heuristic)
        | (SemanticStateProofKind::Heuristic, SemanticStateProofKind::ReceiverTyped) => 4,
        (SemanticStateProofKind::Heuristic, SemanticStateProofKind::Heuristic) => 5,
    };
    let same_symbol_rank = if writer.symbol_id.is_some() && writer.symbol_id == reader.symbol_id {
        0
    } else if writer.file_path == reader.file_path {
        1
    } else {
        2
    };
    let receiver_rank = if writer.file_path == reader.file_path
        && writer.symbol_id.is_some()
        && writer.symbol_id == reader.symbol_id
    {
        match (
            writer.receiver_name.as_deref(),
            reader.receiver_name.as_deref(),
        ) {
            (Some(left), Some(right)) if left == right => 0,
            (Some(_), Some(_)) => 2,
            _ => 1,
        }
    } else {
        1
    };
    let line_distance = if writer.file_path == reader.file_path {
        writer.line.abs_diff(reader.line)
    } else {
        usize::MAX / 4
    };
    let primary_penalty = [
        writer.file_path.as_str() != primary_file,
        reader.file_path.as_str() != primary_file,
    ]
    .into_iter()
    .filter(|outside| *outside)
    .count() as u8;
    (
        same_symbol_rank,
        receiver_rank,
        proof_rank,
        line_distance,
        primary_penalty,
        writer.line,
        reader.line,
        writer.file_path.clone(),
        reader.file_path.clone(),
    )
}

fn build_semantic_state_carrier(
    file_path: &str,
    source: &str,
    semantic_graph: &SemanticGraph,
) -> Option<SemanticStateCarrier> {
    let methods = extract_semantic_state_methods(source);
    let distinct_slots = methods
        .iter()
        .map(semantic_state_declared_slot)
        .collect::<BTreeSet<_>>();
    let has_writer = methods.iter().any(|method| {
        matches!(
            method.role,
            SemanticStateApiRole::Writer | SemanticStateApiRole::Reset
        )
    });
    let has_reader = methods.iter().any(|method| {
        matches!(
            method.role,
            SemanticStateApiRole::Reader | SemanticStateApiRole::Deriver
        )
    });
    if methods.len() < 4 || distinct_slots.len() < 2 || !has_writer || !has_reader {
        return None;
    }

    Some(SemanticStateCarrier {
        file_path: file_path.to_owned(),
        carrier_type: carrier_type_for_file(semantic_graph, file_path, &methods),
        is_trait: carrier_symbol_for_file(semantic_graph, file_path, &methods)
            .is_some_and(|symbol| symbol.kind == crate::graph::SymbolKind::Trait),
        trait_owner_type_names: trait_owner_type_names_for_file(
            semantic_graph,
            file_path,
            &methods,
        ),
        methods_by_symbol_id: build_semantic_state_method_lookup(
            semantic_graph,
            file_path,
            &methods,
        ),
        method_symbol_ids: build_semantic_state_method_symbol_ids(
            semantic_graph,
            file_path,
            &methods,
        ),
        methods,
    })
}

fn build_semantic_state_method_lookup(
    semantic_graph: &SemanticGraph,
    file_path: &str,
    methods: &[SemanticStateApiMethod],
) -> BTreeMap<String, SemanticStateApiMethod> {
    let mut lookup = BTreeMap::<String, SemanticStateApiMethod>::new();
    for method in methods {
        for symbol in semantic_graph.symbols.iter().filter(|symbol| {
            symbol.file_path.display().to_string() == file_path
                && symbol.kind == crate::graph::SymbolKind::Method
                && symbol.name == method.api_name
                && symbol.start_line <= method.line
                && method.line <= symbol.end_line
        }) {
            lookup.insert(symbol.id.clone(), method.clone());
        }
    }
    lookup
}

fn build_semantic_state_call_site_index(
    semantic_graph: &SemanticGraph,
) -> HashMap<String, Vec<SemanticStateCallSite>> {
    let symbol_by_id = semantic_graph
        .symbols
        .iter()
        .map(|symbol| (symbol.id.clone(), symbol))
        .collect::<HashMap<_, _>>();
    let mut resolved_targets_by_occurrence =
        HashMap::<(String, usize, String, usize), BTreeSet<String>>::new();
    for edge in &semantic_graph.resolved_edges {
        if edge.kind != ReferenceKind::Call {
            continue;
        }
        let Some(reference_target_name) = edge.reference_target_name.as_ref() else {
            continue;
        };
        resolved_targets_by_occurrence
            .entry((
                edge.source_file_path.display().to_string(),
                edge.line,
                reference_target_name.clone(),
                edge.occurrence_index,
            ))
            .or_default()
            .insert(edge.target_symbol_id.clone());
    }

    let mut index = HashMap::<String, Vec<SemanticStateCallSite>>::new();
    let mut occurrence_counters = HashMap::<(String, usize, String), usize>::new();
    for reference in &semantic_graph.references {
        if reference.kind != ReferenceKind::Call {
            continue;
        }
        let file_path = reference.file_path.display().to_string();
        let occurrence_key = (
            file_path.clone(),
            reference.line,
            reference.target_name.clone(),
        );
        let occurrence_index = *occurrence_counters
            .entry(occurrence_key)
            .and_modify(|value| *value += 1)
            .or_insert(0);
        index
            .entry(file_path.clone())
            .or_default()
            .push(SemanticStateCallSite {
                file_path,
                enclosing_symbol_id: reference.enclosing_symbol_id.clone(),
                line: reference.line,
                occurrence_index,
                target_name: reference.target_name.clone(),
                receiver_name: reference.receiver_name.clone(),
                normalized_receiver_name: reference
                    .receiver_name
                    .as_deref()
                    .map(normalize_semantic_state_receiver_name),
                receiver_type_name: reference.receiver_type_name.clone(),
                resolved_target_symbol_ids: resolved_targets_by_occurrence
                    .get(&(
                        reference.file_path.display().to_string(),
                        reference.line,
                        reference.target_name.clone(),
                        occurrence_index,
                    ))
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|symbol_id| {
                        let Some(symbol) = symbol_by_id.get(symbol_id) else {
                            return false;
                        };
                        if symbol.name != reference.target_name {
                            return false;
                        }
                        match (
                            reference.receiver_type_name.as_deref(),
                            symbol.owner_type_name.as_deref(),
                        ) {
                            (Some(receiver_type_name), Some(owner_type_name)) => {
                                receiver_type_name == owner_type_name
                            }
                            _ => true,
                        }
                    })
                    .collect(),
            });
    }

    index
}

fn collect_graph_backed_semantic_state_invocations(
    carrier: &SemanticStateCarrier,
    file_path: &str,
    source: Option<&str>,
    file_symbols: &HashMap<String, Vec<&SymbolNode>>,
    call_site_index: &HashMap<String, Vec<SemanticStateCallSite>>,
) -> Vec<SemanticStateInvocation> {
    let mut invocations = Vec::new();
    let mut seen = BTreeSet::new();
    let Some(call_sites) = call_site_index.get(file_path) else {
        return invocations;
    };

    for call_site in call_sites {
        let exact_method = call_site
            .resolved_target_symbol_ids
            .iter()
            .find_map(|target_symbol_id| carrier.methods_by_symbol_id.get(target_symbol_id));
        let typed_method = carrier.methods.iter().find(|method| {
            call_site.target_name == method.api_name
                && call_site.receiver_type_name.as_deref() == Some(carrier.carrier_type.as_str())
        });
        let trait_backed_method = carrier.methods.iter().find(|method| {
            carrier.is_trait
                && call_site.resolved_target_symbol_ids.is_empty()
                && call_site.target_name == method.api_name
                && call_site
                    .receiver_type_name
                    .as_ref()
                    .is_some_and(|receiver_type_name| {
                        carrier.trait_owner_type_names.contains(receiver_type_name)
                            || carrier
                                .trait_owner_type_names
                                .contains(&semantic_state_leaf_type_name(receiver_type_name))
                    })
        });
        let (method, proof_kind) = if let Some(method) = exact_method {
            (method, SemanticStateProofKind::ExactResolved)
        } else if let Some(method) = typed_method {
            (method, SemanticStateProofKind::ReceiverTyped)
        } else if let Some(method) = trait_backed_method {
            (method, SemanticStateProofKind::ReceiverTyped)
        } else {
            continue;
        };

        let (slot, slot_origin) = semantic_state_invocation_slot(
            method,
            source,
            call_site.line,
            call_site.receiver_name.as_deref(),
        );
        let key = (
            file_path.to_owned(),
            call_site.line,
            call_site.occurrence_index,
            method.api_name.clone(),
            slot.clone(),
            call_site
                .receiver_name
                .as_deref()
                .map(normalize_semantic_state_receiver_name),
        );
        if !seen.insert(key) {
            continue;
        }
        let enclosing_symbol = call_site
            .enclosing_symbol_id
            .as_deref()
            .and_then(|symbol_id| {
                file_symbols
                    .get(file_path)?
                    .iter()
                    .find(|symbol| symbol.id == symbol_id)
                    .copied()
            })
            .or_else(|| enclosing_symbol(file_symbols, file_path, call_site.line));
        invocations.push(SemanticStateInvocation {
            file_path: file_path.to_owned(),
            symbol_id: enclosing_symbol.map(|symbol| symbol.id.clone()),
            symbol_name: enclosing_symbol.map(|symbol| symbol.name.clone()),
            receiver_name: call_site
                .receiver_name
                .as_deref()
                .map(normalize_semantic_state_receiver_name),
            occurrence_index: call_site.occurrence_index,
            line: call_site.line,
            api_name: method.api_name.clone(),
            slot,
            slot_origin,
            role: method.role.clone(),
            carrier_type: carrier.carrier_type.clone(),
            proof_kind,
        });
    }

    invocations
}

fn collect_heuristic_semantic_state_invocations(
    carrier: &SemanticStateCarrier,
    file_path: &str,
    source: &str,
    file_symbols: &HashMap<String, Vec<&SymbolNode>>,
    call_site_index: &HashMap<String, Vec<SemanticStateCallSite>>,
) -> Vec<SemanticStateInvocation> {
    let mut invocations = Vec::new();
    let mut seen = BTreeSet::new();
    for method in &carrier.methods {
        for (index, line) in source.lines().enumerate() {
            let line_number = index + 1;
            for occurrence in
                extract_heuristic_semantic_state_call_occurrences(line, &method.api_name)
            {
                if heuristic_semantic_state_call_conflicts_with_graph(
                    carrier,
                    file_path,
                    line_number,
                    occurrence.occurrence_index,
                    occurrence.receiver_name.as_deref(),
                    &method.api_name,
                    call_site_index,
                ) {
                    continue;
                }
                let (slot, slot_origin) = semantic_state_invocation_slot(
                    method,
                    Some(source),
                    line_number,
                    occurrence.receiver_name.as_deref(),
                );
                let key = (
                    file_path.to_owned(),
                    line_number,
                    occurrence.occurrence_index,
                    method.api_name.clone(),
                    slot.clone(),
                    occurrence
                        .receiver_name
                        .as_deref()
                        .map(normalize_semantic_state_receiver_name),
                );
                if !seen.insert(key) {
                    continue;
                }
                let enclosing_symbol = enclosing_symbol(file_symbols, file_path, line_number);
                invocations.push(SemanticStateInvocation {
                    file_path: file_path.to_owned(),
                    symbol_id: enclosing_symbol.map(|symbol| symbol.id.clone()),
                    symbol_name: enclosing_symbol.map(|symbol| symbol.name.clone()),
                    receiver_name: occurrence
                        .receiver_name
                        .as_deref()
                        .map(normalize_semantic_state_receiver_name),
                    occurrence_index: occurrence.occurrence_index,
                    line: line_number,
                    api_name: method.api_name.clone(),
                    slot,
                    slot_origin,
                    role: method.role.clone(),
                    carrier_type: carrier.carrier_type.clone(),
                    proof_kind: SemanticStateProofKind::Heuristic,
                });
            }
        }
    }
    if file_path == carrier.file_path {
        return invocations;
    }

    let keyed_invocation_keys = invocations
        .iter()
        .filter(|invocation| {
            invocation.slot_origin == SemanticStateSlotOrigin::KeyedArgument
                && !invocation.slot.contains("[unknown]")
        })
        .map(|invocation| {
            (
                invocation.symbol_id.clone(),
                invocation.line,
                invocation.api_name.clone(),
                invocation.slot.clone(),
            )
        })
        .collect::<BTreeSet<_>>();

    let allowed_symbol_ids = invocations
        .iter()
        .fold(
            BTreeMap::<Option<String>, BTreeSet<String>>::new(),
            |mut grouped, invocation| {
                grouped
                    .entry(invocation.symbol_id.clone())
                    .or_default()
                    .insert(invocation.api_name.clone());
                grouped
            },
        )
        .into_iter()
        .filter_map(|(symbol_id, apis)| (apis.len() >= 2).then_some(symbol_id))
        .collect::<BTreeSet<_>>();

    invocations
        .into_iter()
        .filter(|invocation| {
            allowed_symbol_ids.contains(&invocation.symbol_id)
                || keyed_invocation_keys.contains(&(
                    invocation.symbol_id.clone(),
                    invocation.line,
                    invocation.api_name.clone(),
                    invocation.slot.clone(),
                ))
        })
        .collect()
}

fn semantic_state_invocation_key(
    invocation: &SemanticStateInvocation,
) -> (String, usize, usize, String, String, Option<String>) {
    (
        invocation.file_path.clone(),
        invocation.line,
        invocation.occurrence_index,
        invocation.api_name.clone(),
        invocation.slot.clone(),
        invocation.receiver_name.clone(),
    )
}

fn semantic_state_invocation_slot(
    method: &SemanticStateApiMethod,
    source: Option<&str>,
    line_number: usize,
    receiver_name: Option<&str>,
) -> (String, SemanticStateSlotOrigin) {
    match &method.slot_spec {
        SemanticStateSlotSpec::Fixed(slot) => (slot.clone(), SemanticStateSlotOrigin::Fixed),
        SemanticStateSlotSpec::KeyedArgument { container } => {
            let key_expression = source
                .and_then(|source| source.lines().nth(line_number.saturating_sub(1)))
                .and_then(|line_text| {
                    extract_semantic_state_call_key_expression(
                        line_text,
                        &method.api_name,
                        receiver_name,
                    )
                })
                .and_then(|key_expression| {
                    source.map(|source| {
                        resolve_semantic_state_key_expression(source, line_number, &key_expression)
                    })
                })
                .unwrap_or_else(|| String::from("unknown"));
            (
                format!("{container}[{key_expression}]"),
                SemanticStateSlotOrigin::KeyedArgument,
            )
        }
    }
}

fn extract_semantic_state_call_key_expression(
    line_text: &str,
    api_name: &str,
    receiver_name: Option<&str>,
) -> Option<String> {
    let receiver_pattern = receiver_name
        .map(normalize_semantic_state_receiver_name)
        .map(|receiver_name| format!(r"\$?{}", regex::escape(&receiver_name)))
        .unwrap_or_else(|| String::from(r"\$?[A-Za-z_][A-Za-z0-9_]*"));
    let pattern = Regex::new(&format!(
        r"{receiver_pattern}\s*(?:->|\.)\s*{}\s*\(\s*(?P<arg>[^,\)]*)",
        regex::escape(api_name)
    ))
    .expect("semantic state key extraction regex should compile");
    pattern
        .captures(line_text)
        .and_then(|captures| captures.name("arg"))
        .map(|capture| normalize_semantic_state_key_expression(capture.as_str()))
        .filter(|expression| !expression.is_empty())
}

fn resolve_semantic_state_key_expression(
    source: &str,
    line_number: usize,
    expression: &str,
) -> String {
    let mut current = expression.to_owned();
    for _ in 0..2 {
        let Some(variable_name) = current.strip_prefix('$').filter(|candidate| {
            candidate
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        }) else {
            break;
        };
        let Some(resolved) =
            find_semantic_state_local_assignment(source, line_number, variable_name)
        else {
            break;
        };
        current = resolved;
    }
    current
}

fn find_semantic_state_local_assignment(
    source: &str,
    line_number: usize,
    variable_name: &str,
) -> Option<String> {
    let pattern = Regex::new(&format!(
        r"\${}\s*=\s*(?P<expr>.+?);(?:\s*//.*)?$",
        regex::escape(variable_name)
    ))
    .expect("semantic state assignment regex should compile");
    let lines = source.lines().collect::<Vec<_>>();
    let start_index = line_number.saturating_sub(1);
    let lower_bound = start_index.saturating_sub(8);
    for index in (lower_bound..start_index).rev() {
        let line = lines.get(index)?.trim();
        if line.is_empty() || line == "{" || line == "}" {
            continue;
        }
        if line.starts_with("if ")
            || line.starts_with("foreach ")
            || line.starts_with("for ")
            || line.starts_with("while ")
            || line.starts_with("switch ")
            || line.starts_with("function ")
        {
            break;
        }
        let Some(captures) = pattern.captures(line) else {
            continue;
        };
        let expression = captures
            .name("expr")
            .map(|capture| normalize_semantic_state_key_expression(capture.as_str()))
            .unwrap_or_default();
        if !expression.is_empty() {
            return Some(expression);
        }
    }
    None
}

fn normalize_semantic_state_key_expression(expression: &str) -> String {
    let expression = expression.trim();
    if expression.is_empty() {
        return String::new();
    }
    let expression = expression
        .split_once("//")
        .map(|(before, _)| before)
        .unwrap_or(expression);
    expression
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
}

fn heuristic_semantic_state_call_conflicts_with_graph(
    carrier: &SemanticStateCarrier,
    file_path: &str,
    line: usize,
    occurrence_index: usize,
    receiver_name: Option<&str>,
    api_name: &str,
    call_site_index: &HashMap<String, Vec<SemanticStateCallSite>>,
) -> bool {
    let Some(call_sites) = call_site_index.get(file_path) else {
        return false;
    };
    let normalized_receiver_name = receiver_name.map(normalize_semantic_state_receiver_name);
    let mut saw_matching_call_site = false;
    let mut saw_untyped_or_ambiguous_call_site = false;
    for call_site in call_sites.iter().filter(|call_site| {
        call_site.line == line
            && call_site.target_name == api_name
            && call_site.occurrence_index == occurrence_index
    }) {
        if normalized_receiver_name.is_some()
            && call_site.normalized_receiver_name.as_deref() != normalized_receiver_name.as_deref()
        {
            continue;
        }
        saw_matching_call_site = true;
        if call_site.receiver_type_name.as_deref() == Some(carrier.carrier_type.as_str()) {
            return false;
        }
        if carrier.is_trait && call_site.receiver_type_name.is_some() {
            return false;
        }
        if call_site
            .resolved_target_symbol_ids
            .iter()
            .any(|symbol_id| carrier.methods_by_symbol_id.contains_key(symbol_id))
        {
            return false;
        }
        if call_site.receiver_type_name.is_some()
            || !call_site.resolved_target_symbol_ids.is_empty()
        {
            continue;
        }
        saw_untyped_or_ambiguous_call_site = true;
    }
    saw_matching_call_site && !saw_untyped_or_ambiguous_call_site
}

fn extract_heuristic_semantic_state_call_occurrences(
    line_text: &str,
    api_name: &str,
) -> Vec<HeuristicSemanticStateCallOccurrence> {
    let pattern = Regex::new(&format!(
        r"(?P<receiver>\$?[A-Za-z_][A-Za-z0-9_]*)\s*(?:->|\.)\s*{}\s*\(",
        regex::escape(api_name)
    ))
    .expect("semantic state receiver regex should compile");
    pattern
        .captures_iter(line_text)
        .enumerate()
        .map(
            |(occurrence_index, captures)| HeuristicSemanticStateCallOccurrence {
                receiver_name: captures
                    .name("receiver")
                    .map(|receiver| receiver.as_str().to_owned()),
                occurrence_index,
            },
        )
        .collect()
}

fn normalize_semantic_state_receiver_name(receiver_name: &str) -> String {
    receiver_name.trim_start_matches('$').to_owned()
}

fn extract_semantic_state_methods(source: &str) -> Vec<SemanticStateApiMethod> {
    let method_pattern = Regex::new(
        r"\bfunction\s+((?:add|set|get|has|reset|build|compile)[A-Z][A-Za-z0-9_]*)\s*\(([^)]*)\)",
    )
    .expect("semantic state method regex should compile");
    let mut methods = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let Some(captures) = method_pattern.captures(line) else {
            continue;
        };
        let Some(method_name) = captures.get(1).map(|capture| capture.as_str()) else {
            continue;
        };
        let params = captures
            .get(2)
            .map(|capture| capture.as_str())
            .unwrap_or_default();
        let Some((role, slot_spec)) = classify_semantic_state_method(method_name, params) else {
            continue;
        };
        methods.push(SemanticStateApiMethod {
            api_name: method_name.to_owned(),
            slot_spec,
            role,
            line: index + 1,
        });
    }
    methods
}

fn classify_semantic_state_method(
    method_name: &str,
    params: &str,
) -> Option<(SemanticStateApiRole, SemanticStateSlotSpec)> {
    let prefixes = [
        ("add", SemanticStateApiRole::Writer),
        ("set", SemanticStateApiRole::Writer),
        ("get", SemanticStateApiRole::Reader),
        ("has", SemanticStateApiRole::Reader),
        ("reset", SemanticStateApiRole::Reset),
        ("build", SemanticStateApiRole::Deriver),
        ("compile", SemanticStateApiRole::Deriver),
    ];
    let (prefix, role) = prefixes
        .into_iter()
        .find(|(prefix, _)| method_name.starts_with(prefix))?;
    let suffix = method_name.strip_prefix(prefix)?;
    if suffix.is_empty() {
        return None;
    }
    if let Some(container) = classify_keyed_semantic_state_container(suffix, params) {
        return Some((
            role.clone(),
            SemanticStateSlotSpec::KeyedArgument { container },
        ));
    }
    Some((
        role.clone(),
        SemanticStateSlotSpec::Fixed(semantic_slot_name(prefix, suffix)),
    ))
}

fn classify_keyed_semantic_state_container(suffix: &str, params: &str) -> Option<String> {
    let container = lower_camel_case(suffix);
    let allowed_containers = [
        "customField",
        "dynamicColumn",
        "attribute",
        "property",
        "entry",
        "item",
        "option",
        "setting",
        "field",
        "column",
        "metadata",
    ];
    if !allowed_containers.contains(&container.as_str()) {
        return None;
    }
    let first_param_name = params
        .split(',')
        .next()
        .map(extract_semantic_state_param_name)
        .filter(|param_name| !param_name.is_empty())?;
    let allowed_param_names = [
        "name",
        "key",
        "path",
        "field",
        "column",
        "slot",
        "entry",
        "item",
        "property",
        "attribute",
        "option",
        "setting",
        "metadata",
        "meta",
    ];
    allowed_param_names
        .contains(&first_param_name.as_str())
        .then_some(container)
}

fn extract_semantic_state_param_name(param: &str) -> String {
    let normalized = param.trim();
    let normalized = normalized
        .strip_prefix('?')
        .unwrap_or(normalized)
        .split('=')
        .next()
        .unwrap_or(normalized)
        .trim();
    normalized
        .split_whitespace()
        .last()
        .unwrap_or(normalized)
        .trim_start_matches('&')
        .trim_start_matches('$')
        .to_owned()
}

fn semantic_slot_name(prefix: &str, suffix: &str) -> String {
    let lowered = lower_camel_case(suffix);
    let normalized = match prefix {
        "add" => singularize_slot(pluralize_slot_if_item(lowered)),
        _ => singularize_slot(lowered),
    };
    normalized
}

fn pluralize_slot_if_item(slot: String) -> String {
    match slot.as_str() {
        "term" | "sorting" => slot,
        _ if slot.ends_with('s') => slot,
        _ if slot.ends_with('y') => format!("{}ies", &slot[..slot.len() - 1]),
        _ => format!("{slot}s"),
    }
}

fn lower_camel_case(input: &str) -> String {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_ascii_lowercase(),
        chars.collect::<String>()
    )
}

fn singularize_slot(slot: String) -> String {
    if slot.ends_with("ies") && slot.len() > 3 {
        format!("{}y", &slot[..slot.len() - 3])
    } else if slot.ends_with('s') && !slot.ends_with("ss") && slot.len() > 1 {
        slot[..slot.len() - 1].to_owned()
    } else {
        slot
    }
}

fn carrier_type_for_file(
    semantic_graph: &SemanticGraph,
    file_path: &str,
    methods: &[SemanticStateApiMethod],
) -> String {
    carrier_symbol_for_file(semantic_graph, file_path, methods)
        .map(|symbol| symbol.name.clone())
        .unwrap_or_else(|| {
            std::path::Path::new(file_path)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(file_path)
                .to_owned()
        })
}

fn carrier_symbol_for_file<'a>(
    semantic_graph: &'a SemanticGraph,
    file_path: &str,
    methods: &[SemanticStateApiMethod],
) -> Option<&'a SymbolNode> {
    semantic_graph
        .symbols
        .iter()
        .filter(|symbol| {
            symbol.file_path.display().to_string() == file_path
                && matches!(
                    symbol.kind,
                    crate::graph::SymbolKind::Class
                        | crate::graph::SymbolKind::Struct
                        | crate::graph::SymbolKind::Trait
                )
                && methods
                    .iter()
                    .filter(|method| {
                        symbol.start_line <= method.line && method.line <= symbol.end_line
                    })
                    .count()
                    > 0
        })
        .max_by_key(|symbol| {
            methods
                .iter()
                .filter(|method| symbol.start_line <= method.line && method.line <= symbol.end_line)
                .count()
        })
}

fn trait_owner_type_names_for_file(
    semantic_graph: &SemanticGraph,
    file_path: &str,
    methods: &[SemanticStateApiMethod],
) -> BTreeSet<String> {
    let Some(carrier_symbol) = carrier_symbol_for_file(semantic_graph, file_path, methods) else {
        return BTreeSet::new();
    };
    if carrier_symbol.kind != crate::graph::SymbolKind::Trait {
        return BTreeSet::new();
    }

    let symbol_by_id = semantic_graph
        .symbols
        .iter()
        .map(|symbol| (symbol.id.as_str(), symbol))
        .collect::<HashMap<_, _>>();
    semantic_graph
        .resolved_edges
        .iter()
        .filter(|edge| {
            edge.kind == ReferenceKind::Type && edge.target_symbol_id == carrier_symbol.id
        })
        .filter_map(|edge| edge.source_symbol_id.as_deref())
        .filter_map(|source_symbol_id| symbol_by_id.get(source_symbol_id).copied())
        .flat_map(|symbol| {
            [
                Some(symbol.name.clone()),
                Some(symbol.qualified_name.clone()),
                Some(semantic_state_leaf_type_name(&symbol.qualified_name)),
            ]
        })
        .flatten()
        .collect()
}

fn semantic_state_leaf_type_name(type_name: &str) -> String {
    type_name
        .rsplit(['\\', ':'])
        .find(|part| !part.is_empty())
        .unwrap_or(type_name)
        .to_owned()
}

fn build_semantic_state_method_symbol_ids(
    semantic_graph: &SemanticGraph,
    file_path: &str,
    methods: &[SemanticStateApiMethod],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut ids = BTreeMap::<String, BTreeSet<String>>::new();
    for method in methods {
        let matching_ids = semantic_graph
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.file_path.display().to_string() == file_path
                    && symbol.kind == crate::graph::SymbolKind::Method
                    && symbol.name == method.api_name
                    && symbol.start_line <= method.line
                    && method.line <= symbol.end_line
            })
            .map(|symbol| symbol.id.clone())
            .collect::<BTreeSet<_>>();
        if !matching_ids.is_empty() {
            ids.insert(method.api_name.clone(), matching_ids);
        }
    }
    ids
}

fn build_file_symbol_index<'a>(
    semantic_graph: &'a SemanticGraph,
) -> HashMap<String, Vec<&'a SymbolNode>> {
    let mut index = HashMap::<String, Vec<&'a SymbolNode>>::new();
    for symbol in &semantic_graph.symbols {
        index
            .entry(symbol.file_path.display().to_string())
            .or_default()
            .push(symbol);
    }
    index
}

fn enclosing_symbol<'a>(
    file_symbols: &'a HashMap<String, Vec<&'a SymbolNode>>,
    file_path: &str,
    line: usize,
) -> Option<&'a SymbolNode> {
    file_symbols
        .get(file_path)?
        .iter()
        .filter(|symbol| symbol.start_line <= line && line <= symbol.end_line)
        .min_by_key(|symbol| symbol.end_line.saturating_sub(symbol.start_line))
        .copied()
}

pub fn graph_neighbors_for_file(
    semantic_graph: &SemanticGraph,
    file_path: &str,
    max_items: usize,
) -> Vec<GraphNeighbor> {
    let context = GraphQueryContext::new(semantic_graph);
    graph_neighbors_for_file_with_context(&context, file_path, max_items)
}

pub fn graph_neighbors_for_file_with_context(
    graph_context: &GraphQueryContext<'_>,
    file_path: &str,
    max_items: usize,
) -> Vec<GraphNeighbor> {
    let mut grouped = HashMap::<String, Vec<&ResolvedEdge>>::new();
    for edge in graph_context
        .incident_edges
        .get(file_path)
        .into_iter()
        .flatten()
    {
        let source = edge.source_file_path.display().to_string();
        let target = edge.target_file_path.display().to_string();
        let neighbor = if source == file_path { target } else { source };
        grouped.entry(neighbor).or_default().push(edge);
    }

    let mut neighbors = grouped
        .into_iter()
        .map(|(neighbor, edges)| {
            let outbound = edges
                .iter()
                .any(|edge| edge.source_file_path.display().to_string() == file_path);
            let inbound = edges
                .iter()
                .any(|edge| edge.target_file_path.display().to_string() == file_path);
            let direction = match (outbound, inbound) {
                (true, true) => GraphNeighborDirection::Mixed,
                (true, false) => GraphNeighborDirection::Outbound,
                (false, true) => GraphNeighborDirection::Inbound,
                (false, false) => GraphNeighborDirection::Mixed,
            };
            GraphNeighbor {
                file_path: neighbor,
                direction,
                edge_count: edges.len(),
                aggregate_confidence_millis: aggregate_path_confidence(&edges),
                relation_histogram: relation_histogram(edges.iter().copied()),
            }
        })
        .collect::<Vec<_>>();

    neighbors.sort_by(|left, right| {
        right
            .edge_count
            .cmp(&left.edge_count)
            .then(
                right
                    .aggregate_confidence_millis
                    .cmp(&left.aggregate_confidence_millis),
            )
            .then(left.file_path.cmp(&right.file_path))
    });
    neighbors.truncate(max_items);
    neighbors
}

pub fn graph_trace_between_files(
    semantic_graph: &SemanticGraph,
    start_file_path: &str,
    goal_file_path: &str,
    max_hops: usize,
    max_paths: usize,
) -> Vec<AgenticGraphTrace> {
    let context = GraphQueryContext::new(semantic_graph);
    graph_trace_between_files_with_context(
        &context,
        start_file_path,
        goal_file_path,
        max_hops,
        max_paths,
    )
}

pub fn graph_trace_between_files_with_context(
    graph_context: &GraphQueryContext<'_>,
    start_file_path: &str,
    goal_file_path: &str,
    max_hops: usize,
    max_paths: usize,
) -> Vec<AgenticGraphTrace> {
    directed_graph_traces(
        graph_context,
        start_file_path,
        goal_file_path,
        max_hops,
        max_paths,
    )
}

fn directed_graph_traces(
    graph_context: &GraphQueryContext<'_>,
    start_file_path: &str,
    goal_file_path: &str,
    max_hops: usize,
    max_paths: usize,
) -> Vec<AgenticGraphTrace> {
    let key = (
        start_file_path.to_owned(),
        goal_file_path.to_owned(),
        max_hops,
        max_paths,
    );
    if let Some(cached) = graph_context.trace_cache.borrow().get(&key) {
        return cached.clone();
    }

    let paths = if max_paths <= 1 {
        find_shortest_directed_file_path(graph_context, start_file_path, goal_file_path, max_hops)
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        find_directed_file_paths(
            graph_context,
            start_file_path,
            goal_file_path,
            max_hops,
            max_paths,
        )
    };

    let traces = paths
        .into_iter()
        .enumerate()
        .map(|(index, path)| AgenticGraphTrace {
            id: graph_trace_id_for_edges(
                AgenticGraphTraceKind::DirectedSupportPath,
                start_file_path,
                goal_file_path,
                &path,
            ),
            label: trace_label(start_file_path, goal_file_path, index),
            kind: AgenticGraphTraceKind::DirectedSupportPath,
            primary_file_path: start_file_path.to_owned(),
            supporting_file_path: Some(goal_file_path.to_owned()),
            aggregate_confidence_millis: aggregate_path_confidence(&path),
            relation_sequence: relation_sequence(&path),
            truncated: path.len() >= max_hops,
            hops: build_trace_hops(&path, &graph_context.symbol_lookup),
        })
        .collect::<Vec<_>>();
    graph_context
        .trace_cache
        .borrow_mut()
        .insert(key, traces.clone());
    traces
}

fn find_shortest_directed_file_path<'a>(
    graph_context: &'a GraphQueryContext<'a>,
    start: &str,
    goal: &str,
    max_hops: usize,
) -> Option<Vec<&'a ResolvedEdge>> {
    if start == goal {
        return Some(Vec::new());
    }

    let mut queue = VecDeque::from([(start.to_owned(), 0usize)]);
    let mut visited = HashSet::from([start.to_owned()]);
    let mut predecessors = HashMap::<String, (String, &'a ResolvedEdge)>::new();

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_hops {
            continue;
        }
        let Some(candidate_edges) = graph_context.outbound_edges.get(&current) else {
            continue;
        };
        for edge in candidate_edges {
            let next = edge.target_file_path.display().to_string();
            if !visited.insert(next.clone()) {
                continue;
            }
            predecessors.insert(next.clone(), (current.clone(), edge));
            if next == goal {
                let mut path = Vec::<&ResolvedEdge>::new();
                let mut cursor = next;
                while let Some((previous, edge)) = predecessors.get(&cursor) {
                    path.push(*edge);
                    cursor = previous.clone();
                    if cursor == start {
                        path.reverse();
                        return Some(path);
                    }
                }
                return None;
            }
            queue.push_back((next, depth + 1));
        }
    }

    None
}

fn source_role(kind: AgenticGraphTraceKind) -> &'static str {
    match kind {
        AgenticGraphTraceKind::DirectedSupportPath => "primary",
        AgenticGraphTraceKind::ReverseSupportPath => "supporting",
        AgenticGraphTraceKind::ContextualSupportPath => "context",
    }
}

fn sink_role(kind: AgenticGraphTraceKind) -> &'static str {
    match kind {
        AgenticGraphTraceKind::DirectedSupportPath => "supporting",
        AgenticGraphTraceKind::ReverseSupportPath => "primary",
        AgenticGraphTraceKind::ContextualSupportPath => "context",
    }
}

fn trace_label(start: &str, goal: &str, index: usize) -> String {
    if index == 0 {
        format!("{start} -> {goal}")
    } else {
        format!("{start} -> {goal} [alt {}]", index + 1)
    }
}

fn graph_trace_id_for_edges(
    kind: AgenticGraphTraceKind,
    primary_file_path: &str,
    supporting_file_path: &str,
    path: &[&ResolvedEdge],
) -> String {
    let mut owned_parts = vec![
        graph_trace_kind_id_label(kind).to_string(),
        primary_file_path.to_string(),
        supporting_file_path.to_string(),
    ];
    for edge in path {
        owned_parts.push(path_id_component(&edge.source_file_path));
        owned_parts.push(edge.line.to_string());
        owned_parts.push(path_id_component(&edge.target_file_path));
        owned_parts.push(edge.target_symbol_id.clone());
        owned_parts.push(relation_kind_id_label(edge.relation_kind).to_string());
    }
    let parts = owned_parts.iter().map(String::as_str).collect::<Vec<_>>();
    format!("graph-trace|{}", stable_fingerprint(&parts))
}

fn graph_trace_id_for_hops(
    kind: AgenticGraphTraceKind,
    primary_file_path: &str,
    supporting_file_path: &str,
    hops: &[AgenticGraphTraceHop],
) -> String {
    let mut owned_parts = vec![
        graph_trace_kind_id_label(kind).to_string(),
        primary_file_path.to_string(),
        supporting_file_path.to_string(),
    ];
    for hop in hops {
        owned_parts.push(hop.source_file_path.clone());
        owned_parts.push(hop.line.to_string());
        owned_parts.push(hop.target_file_path.clone());
        owned_parts.push(hop.target_symbol_id.clone().unwrap_or_default());
        owned_parts.push(hop.relation_kind.clone());
    }
    let parts = owned_parts.iter().map(String::as_str).collect::<Vec<_>>();
    format!("graph-trace|{}", stable_fingerprint(&parts))
}

fn path_id_component(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

fn graph_trace_kind_id_label(kind: AgenticGraphTraceKind) -> &'static str {
    match kind {
        AgenticGraphTraceKind::DirectedSupportPath => "directed",
        AgenticGraphTraceKind::ReverseSupportPath => "reverse",
        AgenticGraphTraceKind::ContextualSupportPath => "contextual",
    }
}

fn relation_kind_id_label(kind: RelationKind) -> &'static str {
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

fn code_flow_id(trace: &AgenticGraphTrace) -> String {
    format!(
        "code-flow|{}",
        stable_fingerprint(&[trace.id.as_str(), "v1"])
    )
}

fn source_sink_path_id(trace: &AgenticGraphTrace, flow: &AgenticCodeFlow) -> String {
    format!(
        "source-sink|{}",
        stable_fingerprint(&[trace.id.as_str(), flow.id.as_str(), "v1"])
    )
}

fn aggregate_path_confidence(path: &[&ResolvedEdge]) -> u16 {
    if path.is_empty() {
        return 0;
    }
    let total = path
        .iter()
        .map(|edge| usize::from(edge.confidence_millis))
        .sum::<usize>();
    (total / path.len()) as u16
}

fn relation_sequence(path: &[&ResolvedEdge]) -> Vec<String> {
    path.iter()
        .map(|edge| format!("{:?}", edge.relation_kind))
        .collect()
}

fn relation_histogram<'a>(
    edges: impl Iterator<Item = &'a ResolvedEdge>,
) -> Vec<GraphRelationCount> {
    let mut counts = HashMap::<String, usize>::new();
    for edge in edges {
        *counts
            .entry(format!("{:?}", edge.relation_kind))
            .or_insert(0) += 1;
    }
    let mut histogram = counts
        .into_iter()
        .map(|(relation_kind, count)| GraphRelationCount {
            relation_kind,
            count,
        })
        .collect::<Vec<_>>();
    histogram.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.relation_kind.cmp(&right.relation_kind))
    });
    histogram
}

fn relation_histogram_for_files<'a>(
    graph_context: &'a GraphQueryContext<'a>,
    file_paths: impl Iterator<Item = &'a str>,
) -> Vec<GraphRelationCount> {
    let targets = file_paths
        .filter(|path| !path.is_empty())
        .map(String::from)
        .collect::<HashSet<_>>();
    let mut seen = BTreeSet::<(String, usize, String, String)>::new();
    relation_histogram(
        targets
            .iter()
            .flat_map(|path| {
                graph_context
                    .incident_edges
                    .get(path)
                    .into_iter()
                    .flatten()
                    .copied()
            })
            .filter(|edge| {
                let source = edge.source_file_path.display().to_string();
                let target = edge.target_file_path.display().to_string();
                targets.contains(&source) || targets.contains(&target)
            })
            .filter(|edge| {
                seen.insert((
                    edge.source_file_path.display().to_string(),
                    edge.line,
                    edge.target_file_path.display().to_string(),
                    edge.target_symbol_id.clone(),
                ))
            }),
    )
}

fn build_symbol_lookup(semantic_graph: &SemanticGraph) -> HashMap<&str, &SymbolNode> {
    semantic_graph
        .symbols
        .iter()
        .map(|symbol| (symbol.id.as_str(), symbol))
        .collect()
}

fn build_trace_hops(
    path: &[&ResolvedEdge],
    symbol_lookup: &HashMap<&str, &SymbolNode>,
) -> Vec<AgenticGraphTraceHop> {
    path.iter()
        .map(|edge| {
            let source_symbol = edge
                .source_symbol_id
                .as_deref()
                .and_then(|id| symbol_lookup.get(id))
                .copied();
            let target_symbol = symbol_lookup.get(edge.target_symbol_id.as_str()).copied();
            AgenticGraphTraceHop {
                source_file_path: edge.source_file_path.display().to_string(),
                source_symbol_id: edge.source_symbol_id.clone(),
                source_symbol_name: source_symbol.map(|symbol| symbol.qualified_name.clone()),
                target_file_path: edge.target_file_path.display().to_string(),
                target_symbol_id: Some(edge.target_symbol_id.clone()),
                target_symbol_name: target_symbol.map(|symbol| symbol.qualified_name.clone()),
                relation_kind: format!("{:?}", edge.relation_kind),
                layer: format!("{:?}", edge.layer),
                origin: format!("{:?}", edge.origin),
                strength: format!("{:?}", edge.strength),
                resolution_tier: format!("{:?}", edge.resolution_tier),
                line: edge.line,
                confidence_millis: edge.confidence_millis,
                reason: edge.reason.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
fn strongest_context_edges<'a>(
    semantic_graph: &'a SemanticGraph,
    primary: &str,
    target_files: &HashSet<String>,
    limit: usize,
) -> Vec<&'a ResolvedEdge> {
    let mut edges = semantic_graph
        .resolved_edges
        .iter()
        .filter(|edge| {
            let source = edge.source_file_path.display().to_string();
            let target = edge.target_file_path.display().to_string();
            if source == target {
                return false;
            }
            (source == primary || target == primary)
                && (target_files.is_empty()
                    || target_files.contains(&source)
                    || target_files.contains(&target))
        })
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        right
            .confidence_millis
            .cmp(&left.confidence_millis)
            .then(left.line.cmp(&right.line))
            .then(left.source_file_path.cmp(&right.source_file_path))
            .then(left.target_file_path.cmp(&right.target_file_path))
    });
    edges.truncate(limit);
    edges
}

fn strongest_context_edges_with_context<'a>(
    graph_context: &'a GraphQueryContext<'a>,
    primary: &str,
    target_files: &HashSet<String>,
    limit: usize,
) -> Vec<&'a ResolvedEdge> {
    let mut edges = graph_context
        .incident_edges
        .get(primary)
        .into_iter()
        .flatten()
        .copied()
        .filter(|edge| {
            let source = edge.source_file_path.display().to_string();
            let target = edge.target_file_path.display().to_string();
            if source == target {
                return false;
            }
            (source == primary || target == primary)
                && (target_files.is_empty()
                    || target_files.contains(&source)
                    || target_files.contains(&target))
        })
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        right
            .confidence_millis
            .cmp(&left.confidence_millis)
            .then(left.line.cmp(&right.line))
            .then(left.source_file_path.cmp(&right.source_file_path))
            .then(left.target_file_path.cmp(&right.target_file_path))
    });
    edges.truncate(limit);
    edges
}

fn find_directed_file_paths<'a>(
    graph_context: &'a GraphQueryContext<'a>,
    start: &str,
    goal: &str,
    max_hops: usize,
    path_limit: usize,
) -> Vec<Vec<&'a ResolvedEdge>> {
    const MAX_BRANCHES_PER_NODE: usize = 12;
    let mut paths = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((
        start.to_owned(),
        Vec::<&ResolvedEdge>::new(),
        HashSet::from([start.to_owned()]),
    ));

    while let Some((current, path, visited)) = queue.pop_front() {
        if current == goal {
            paths.push(path);
            if paths.len() >= path_limit {
                break;
            }
            continue;
        }
        if path.len() >= max_hops {
            continue;
        }
        let Some(candidate_edges) = graph_context.outbound_edges.get(&current) else {
            continue;
        };
        for edge in candidate_edges.iter().take(MAX_BRANCHES_PER_NODE) {
            let next = edge.target_file_path.display().to_string();
            if visited.contains(&next) {
                continue;
            }
            let mut next_path = path.clone();
            next_path.push(edge);
            let mut next_visited = visited.clone();
            next_visited.insert(next.clone());
            queue.push_back((next, next_path, next_visited));
        }
    }

    paths
}

fn edge_path_priority(edge: &ResolvedEdge) -> (u8, u16, u8) {
    (
        relation_path_rank(edge.relation_kind),
        edge.confidence_millis,
        layer_path_rank(edge.layer),
    )
}

fn relation_path_rank(kind: RelationKind) -> u8 {
    match kind {
        RelationKind::Dispatch => 10,
        RelationKind::EventPublish => 9,
        RelationKind::Call => 8,
        RelationKind::ContainerResolution => 7,
        RelationKind::EventSubscribe => 6,
        RelationKind::Overrides => 5,
        RelationKind::Extends => 4,
        RelationKind::Implements => 3,
        RelationKind::Import => 2,
        RelationKind::TypeUse => 1,
    }
}

fn layer_path_rank(layer: crate::graph::GraphLayer) -> u8 {
    match layer {
        crate::graph::GraphLayer::Runtime => 4,
        crate::graph::GraphLayer::Framework => 3,
        crate::graph::GraphLayer::PolicyOverlay => 2,
        crate::graph::GraphLayer::Structural => 1,
    }
}

fn packet_status(packet: &GuardianPacket, convergence: &ConvergenceHistoryArtifact) -> String {
    convergence
        .attention_items
        .iter()
        .filter(|item| {
            item.file_paths.iter().any(|path| {
                packet.target_files.contains(path) || *path == packet.primary_target_file
            })
        })
        .map(|item| convergence_status_label(item.status))
        .min_by_key(|status| task_status_rank(status))
        .or_else(|| {
            convergence
                .findings
                .iter()
                .filter(|finding| {
                    finding.file_paths.iter().any(|path| {
                        packet.target_files.contains(path) || *path == packet.primary_target_file
                    })
                })
                .map(|finding| convergence_status_label(finding.status))
                .min_by_key(|status| task_status_rank(status))
        })
        .unwrap_or_else(|| String::from("context"))
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

fn task_status_rank(status: &str) -> u8 {
    match status {
        "new" => 0,
        "worsened" => 1,
        "improved" => 2,
        "unchanged" => 3,
        "resolved" => 4,
        _ => 5,
    }
}

fn guard_verdict_label(verdict: GuardVerdict) -> String {
    match verdict {
        GuardVerdict::Allow => String::from("Allow"),
        GuardVerdict::Warn => String::from("Warn"),
        GuardVerdict::Block => String::from("Block"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_agentic_review_artifact, build_code_flows, build_execution_contract,
        build_focus_file_graph_evidence, build_graph_packet_artifact, build_graph_traces,
        build_semantic_state_flows, build_source_lookup, build_source_sink_paths,
        focus_agentic_review_artifact, graph_neighbors_for_file, graph_trace_between_files,
        packet_supports_semantic_state_flows, should_build_semantic_state_flows,
        strongest_context_edges, AgenticContextArtifact, AgenticDiffSummary, AgenticEvidenceChain,
        AgenticEvidenceLocation, AgenticGraphPriority, AgenticGraphTrace, AgenticGraphTraceHop,
        AgenticGraphTraceKind, AgenticPrimaryEvidenceRefs, AgenticReviewArtifact,
        AgenticReviewSummary, AgenticTaskPacket, AgenticTransportContract, GraphPacketKind,
        SemanticStateProofKind,
    };
    use crate::artifacts::{
        build_agent_handoff_artifact, build_convergence_history_artifact,
        build_guard_decision_artifact, AgentHandoffArtifact, AgentHandoffFinding, GuardianPacket,
        GuardianSuppressibility,
    };
    use crate::doctrine::built_in_doctrine_registry;
    use crate::graph::{ReferenceKind, ResolutionTier, ResolvedEdge, SemanticGraph};
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use crate::policy::PolicyBundle;
    use crate::review::build_review_surface;
    use crate::review::{
        PolicyStatus, ReviewFinding, ReviewFindingFamily, ReviewFindingSeverity, ReviewStatus,
    };
    use crate::surface::build_architecture_surface;
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builds_agentic_review_artifact_from_graph_and_guard_state() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            b"fn main() { let mode = std::env::var(\"APP_MODE\").unwrap_or_default(); println!(\"{}\", mode); }\n",
        )
        .unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = build_architecture_surface(&analysis);
        let doctrine = built_in_doctrine_registry();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine);
        let convergence = crate::artifacts::build_convergence_history_artifact(
            &analysis.root,
            &analysis.semantic_graph,
            None,
            None,
            None,
            &surface,
            &review_surface,
            &analysis.contract_inventory,
            &doctrine,
        );
        let guard = build_guard_decision_artifact(&analysis.root, &convergence);

        let artifact =
            build_agentic_review_artifact(&analysis, &doctrine, &handoff, &guard, &convergence);
        assert_eq!(artifact.transport.provider_family, "openai");
        assert_eq!(artifact.transport.recommended_protocol, "responses_api");
        assert_eq!(artifact.execution.provider, "openai");
        assert_eq!(
            artifact.execution.openai_responses.endpoint,
            "https://api.openai.com/v1/responses"
        );
        assert_eq!(
            artifact.execution.structured_output.schema_name,
            "aigiscode_agentic_review_response"
        );
        assert_eq!(artifact.contract_version, "2026-03-28");
        assert_eq!(
            artifact.execution.structured_output.schema_version,
            "2026-03-28"
        );
        assert!(artifact
            .execution
            .report_targets
            .iter()
            .any(|target| target.file_name == "agent-review.md"));
        assert!(artifact.task_packets.iter().all(|packet| {
            packet.evidence_refs.primary_graph_trace_id
                == packet
                    .evidence_chain
                    .graph_traces
                    .first()
                    .map(|trace| trace.id.clone())
                && packet.evidence_refs.primary_code_flow_id
                    == packet
                        .evidence_chain
                        .code_flows
                        .first()
                        .map(|flow| flow.id.clone())
                && packet.evidence_refs.primary_source_sink_path_id
                    == packet
                        .evidence_chain
                        .source_sink_paths
                        .first()
                        .map(|path| path.id.clone())
        }));
        assert_eq!(
            artifact.graph_priority.architecture_source,
            "dependency-graph.json"
        );
        assert!(artifact
            .context_artifacts
            .iter()
            .any(|artifact| artifact.file == "aigiscode-handoff.json"));
        assert!(artifact.system_prompt.contains("dependency-graph.json"));
        if let Some(packet) = artifact.task_packets.first() {
            assert!(packet
                .evidence_chain
                .artifact_refs
                .iter()
                .any(|artifact| artifact == "evidence-graph.json"));
        }
    }

    #[test]
    fn falls_back_to_top_findings_for_focus_files_when_guardian_packets_are_empty() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}\n").unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let doctrine = built_in_doctrine_registry();
        let handoff = AgentHandoffArtifact {
            root: analysis.root.display().to_string(),
            summary: crate::artifacts::AgentHandoffSummary {
                scanned_files: 1,
                analyzed_files: 1,
                strong_cycle_count: 0,
                bottleneck_count: 0,
                architectural_smell_count: 0,
                warning_heavy_hotspot_count: 0,
                split_identity_model_count: 0,
                compatibility_scar_count: 0,
                duplicate_mechanism_count: 0,
                sanctioned_path_bypass_count: 0,
                hand_rolled_parsing_count: 0,
                abstraction_sprawl_count: 0,
                algorithmic_complexity_hotspot_count: 0,
                visible_findings: 1,
                dead_code_count: 0,
                hardwiring_count: 0,
                security_finding_count: 0,
                external_finding_count: 0,
            },
            feedback_loop: crate::artifacts::FeedbackLoopSummary {
                detected_total: 1,
                actionable_visible: 1,
                accepted_by_policy: 0,
                suppressed_by_rule: 0,
                ai_reviewed: 0,
                rules_generated: 0,
            },
            next_steps: vec![String::from("Inspect src/main.rs")],
            guardian_packets: Vec::new(),
            top_findings: vec![AgentHandoffFinding {
                id: String::from("finding-1"),
                family: String::from("graph"),
                severity: String::from("high"),
                title: String::from("Example finding"),
                summary: String::from("Bounded fallback finding"),
                file_paths: vec![String::from("src/main.rs")],
                line: Some(1),
                primary_anchor: None,
                locations: Vec::new(),
            }],
        };
        let review_surface = crate::review::ReviewSurface {
            root: analysis.root.display().to_string(),
            summary: crate::review::ReviewSummary {
                total_findings: 1,
                visible_findings: 1,
                unreviewed_findings: 1,
                accepted_by_policy: 0,
                suppressed_by_rule: 0,
                ai_reviewed: 0,
                rules_generated: 0,
            },
            findings: vec![ReviewFinding {
                id: String::from("finding-1"),
                fingerprint: String::from("finding-1"),
                family: ReviewFindingFamily::Graph,
                severity: ReviewFindingSeverity::High,
                title: String::from("Example finding"),
                summary: String::from("Bounded fallback finding"),
                file_paths: vec![String::from("src/main.rs")],
                line: Some(1),
                primary_anchor: None,
                evidence_anchors: Vec::new(),
                locations: Vec::new(),
                supporting_context: Vec::new(),
                precision: String::from("modeled"),
                confidence_millis: 800,
                provenance: vec![String::from("graph_analysis")],
                doctrine_refs: vec![String::from("guardian.architectonic-quality")],
                review_status: ReviewStatus::Unreviewed,
                policy_status: PolicyStatus::None,
                is_visible: true,
            }],
        };
        let convergence = crate::artifacts::build_convergence_history_artifact(
            &analysis.root,
            &analysis.semantic_graph,
            None,
            None,
            None,
            &analysis.architecture_surface(),
            &review_surface,
            &analysis.contract_inventory,
            &doctrine,
        );
        let guard = build_guard_decision_artifact(&analysis.root, &convergence);

        let artifact =
            build_agentic_review_artifact(&analysis, &doctrine, &handoff, &guard, &convergence);
        assert_eq!(
            artifact.summary.top_focus_files,
            vec![String::from("src/main.rs")]
        );
        assert!(artifact.system_prompt.contains("src/main.rs"));
    }

    #[test]
    fn codex_output_schema_disallows_additional_properties() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}\n").unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = build_architecture_surface(&analysis);
        let doctrine = built_in_doctrine_registry();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine);
        let convergence = crate::artifacts::build_convergence_history_artifact(
            &analysis.root,
            &analysis.semantic_graph,
            None,
            None,
            None,
            &surface,
            &review_surface,
            &analysis.contract_inventory,
            &doctrine,
        );
        let guard = build_guard_decision_artifact(&analysis.root, &convergence);

        let artifact =
            build_agentic_review_artifact(&analysis, &doctrine, &handoff, &guard, &convergence);
        assert_eq!(
            artifact.execution.structured_output.json_schema["additionalProperties"],
            serde_json::Value::Bool(false)
        );
        assert_eq!(
            artifact.execution.structured_output.json_schema["$defs"]["AgenticStructuredClaim"]
                ["additionalProperties"],
            serde_json::Value::Bool(false)
        );
        assert_eq!(
            artifact.execution.structured_output.json_schema["$defs"]
                ["AgenticStructuredEvidenceLocation"]["required"],
            serde_json::json!(["file_path", "line"])
        );
    }

    #[test]
    fn builds_graph_traces_for_packet_targets() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/lib.rs"), b"pub fn helper() {}\n").unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            b"mod lib;\nfn main() { lib::helper(); }\n",
        )
        .unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let packet = GuardianPacket {
            id: String::from("guardian:test-trace"),
            priority: String::from("high"),
            focus: String::from("architecture"),
            primary_target_file: String::from("src/main.rs"),
            precision: String::from("modeled"),
            confidence_millis: 800,
            summary: String::from("Trace packet"),
            target_files: vec![String::from("src/main.rs"), String::from("src/lib.rs")],
            primary_anchor: None,
            evidence_anchors: Vec::new(),
            locations: Vec::new(),
            finding_ids: vec![String::from("finding-1")],
            context_labels: Vec::new(),
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![String::from("guardian.test")],
            preferred_mechanism: None,
            obligations: Vec::new(),
            suppressibility: GuardianSuppressibility {
                allowed: true,
                requires_reason: true,
                expiry_required: true,
            },
            investigation_questions: Vec::new(),
        };
        let traces = build_graph_traces(&packet, &analysis.semantic_graph);
        assert!(!traces.is_empty());
        assert!(traces
            .iter()
            .any(|trace| trace.kind == AgenticGraphTraceKind::DirectedSupportPath));
        assert!(traces
            .iter()
            .flat_map(|trace| trace.hops.iter())
            .any(|hop| {
                hop.target_file_path == "src/lib.rs" || hop.source_file_path == "src/lib.rs"
            }));
        assert!(traces
            .iter()
            .flat_map(|trace| trace.hops.iter())
            .any(|hop| hop.target_symbol_name.is_some() || hop.source_symbol_name.is_some()));
        let code_flows = build_code_flows(&traces);
        assert!(!code_flows.is_empty());
        assert!(code_flows.iter().any(|flow| {
            flow.steps
                .iter()
                .any(|step| step.file_path == "src/lib.rs" || step.file_path == "src/main.rs")
        }));
        assert!(code_flows
            .iter()
            .all(|flow| !flow.source.file_path.is_empty()));
        assert!(code_flows
            .iter()
            .all(|flow| !flow.sink.file_path.is_empty()));
        assert!(traces.iter().all(|trace| !trace.id.is_empty()));
        assert!(code_flows
            .iter()
            .all(|flow| !flow.id.is_empty() && !flow.graph_trace_id.is_empty()));
        assert!(code_flows
            .iter()
            .all(|flow| { traces.iter().any(|trace| trace.id == flow.graph_trace_id) }));
        assert!(code_flows
            .iter()
            .flat_map(|flow| flow.steps.iter())
            .any(|step| step.layer_to_next.is_some() || step.origin_to_next.is_some()));
        let source_sink_paths = build_source_sink_paths(
            &traces,
            &code_flows,
            &[
                AgenticEvidenceLocation {
                    role: String::from("primary"),
                    file_path: String::from("src/main.rs"),
                    line: Some(2),
                },
                AgenticEvidenceLocation {
                    role: String::from("support"),
                    file_path: String::from("src/lib.rs"),
                    line: Some(1),
                },
            ],
        );
        assert!(!source_sink_paths.is_empty());
        assert!(!source_sink_paths[0].id.is_empty());
        assert_eq!(
            source_sink_paths[0].graph_trace_id,
            code_flows[0].graph_trace_id
        );
        assert_eq!(source_sink_paths[0].code_flow_id, code_flows[0].id);
        assert_eq!(source_sink_paths[0].source.file_path, "src/main.rs");
        assert_eq!(source_sink_paths[0].source.line, Some(2));
        assert_eq!(
            source_sink_paths[0].sink.file_path,
            code_flows[0].sink.file_path
        );
        assert_eq!(source_sink_paths[0].sink.line, code_flows[0].sink.line);
        assert_eq!(
            source_sink_paths[0].supporting_locations,
            code_flows[0].supporting_locations
        );
    }

    #[test]
    fn code_flows_use_next_hop_line_for_intermediate_target_steps() {
        let traces = vec![AgenticGraphTrace {
            id: String::from("graph-trace|example"),
            label: String::from("a.rs -> c.rs"),
            kind: AgenticGraphTraceKind::DirectedSupportPath,
            primary_file_path: String::from("a.rs"),
            supporting_file_path: Some(String::from("c.rs")),
            aggregate_confidence_millis: 900,
            relation_sequence: vec![String::from("Call"), String::from("Import")],
            truncated: false,
            hops: vec![
                AgenticGraphTraceHop {
                    source_file_path: String::from("a.rs"),
                    source_symbol_id: Some(String::from("function:a.rs:start")),
                    source_symbol_name: Some(String::from("start")),
                    target_file_path: String::from("b.rs"),
                    target_symbol_id: Some(String::from("function:b.rs:mid")),
                    target_symbol_name: Some(String::from("mid")),
                    relation_kind: String::from("Call"),
                    layer: String::from("Structural"),
                    origin: String::from("Resolver"),
                    strength: String::from("Hard"),
                    resolution_tier: String::from("SameFile"),
                    line: 11,
                    confidence_millis: 900,
                    reason: String::from("call"),
                },
                AgenticGraphTraceHop {
                    source_file_path: String::from("b.rs"),
                    source_symbol_id: Some(String::from("function:b.rs:mid")),
                    source_symbol_name: Some(String::from("mid")),
                    target_file_path: String::from("c.rs"),
                    target_symbol_id: Some(String::from("function:c.rs:end")),
                    target_symbol_name: Some(String::from("end")),
                    relation_kind: String::from("Import"),
                    layer: String::from("Structural"),
                    origin: String::from("Resolver"),
                    strength: String::from("Hard"),
                    resolution_tier: String::from("ImportScoped"),
                    line: 27,
                    confidence_millis: 850,
                    reason: String::from("import"),
                },
            ],
        }];

        let flows = build_code_flows(&traces);
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].steps.len(), 3);
        assert_eq!(flows[0].steps[1].file_path, "b.rs");
        assert_eq!(flows[0].steps[1].line, Some(27));
        assert_eq!(flows[0].supporting_locations.len(), 1);
        assert_eq!(flows[0].supporting_locations[0].file_path, "b.rs");
        assert_eq!(flows[0].supporting_locations[0].line, Some(27));
    }

    #[test]
    fn builds_graph_packet_artifact_from_task_packets() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/main.rs"),
            br#"mod service;
fn main() { service::run(); }"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/service.rs"),
            br#"pub fn run() { helper(); }
fn helper() {}"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let doctrine = built_in_doctrine_registry();
        let surface = analysis.architecture_surface();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let convergence = build_convergence_history_artifact(
            &analysis.root,
            &analysis.semantic_graph,
            None,
            None,
            None,
            &surface,
            &review_surface,
            &analysis.contract_inventory,
            &doctrine,
        );
        let guard = build_guard_decision_artifact(&analysis.root, &convergence);
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine);
        let review =
            build_agentic_review_artifact(&analysis, &doctrine, &handoff, &guard, &convergence);

        let packets = build_graph_packet_artifact(&review, &analysis);
        assert!(packets.summary.total_packets >= 1);
        assert!(!packets.packets.is_empty());
        assert!(packets.packets.iter().any(|packet| {
            packet.evidence_refs.primary_graph_trace_id.is_some()
                || packet.evidence_refs.primary_code_flow_id.is_some()
                || packet.evidence_refs.primary_source_sink_path_id.is_some()
        }));
        assert!(packets.packets[0].neighbors.iter().all(|neighbor| {
            !neighbor.file_path.is_empty() && !neighbor.relation_histogram.is_empty()
        }));
    }

    #[test]
    fn fallback_focus_file_graph_packets_carry_traces_from_neighbors() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/main.rs"),
            br#"mod service;
fn main() { service::run(); }"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/service.rs"),
            br#"pub fn run() { helper(); }
fn helper() {}"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let doctrine = built_in_doctrine_registry();
        let surface = analysis.architecture_surface();
        let review_surface = build_review_surface(&analysis, &surface, &PolicyBundle::default());
        let convergence = build_convergence_history_artifact(
            &analysis.root,
            &analysis.semantic_graph,
            None,
            None,
            None,
            &surface,
            &review_surface,
            &analysis.contract_inventory,
            &doctrine,
        );
        let guard = build_guard_decision_artifact(&analysis.root, &convergence);
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface, &doctrine);
        let mut review =
            build_agentic_review_artifact(&analysis, &doctrine, &handoff, &guard, &convergence);
        review.task_packets.clear();
        review.summary.top_focus_files = vec![String::from("src/main.rs")];

        let packets = build_graph_packet_artifact(&review, &analysis);
        let focus_packet = packets
            .packets
            .iter()
            .find(|packet| packet.kind == GraphPacketKind::FocusFile)
            .expect("expected a fallback focus-file packet");

        assert!(!focus_packet.neighbors.is_empty());
        assert!(!focus_packet.graph_traces.is_empty());
        assert!(!focus_packet.code_flows.is_empty());
        assert!(!focus_packet.source_sink_paths.is_empty());
        assert!(focus_packet
            .code_flows
            .iter()
            .all(|flow| !flow.source.file_path.is_empty() && !flow.sink.file_path.is_empty()));
    }

    #[test]
    fn focus_file_graph_evidence_recovers_reverse_paths_and_skips_self_traces() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/support.rs"),
            None,
            PathBuf::from("src/main.rs"),
            String::from("symbol:src/main.rs"),
            ReferenceKind::Call,
            ResolutionTier::ImportScoped,
            910,
            String::from("support-to-main"),
            12,
        ));

        let neighbors = graph_neighbors_for_file(&graph, "src/main.rs", 8);
        let source_lookup = HashMap::new();
        let (traces, code_flows, source_sink_paths, semantic_state_flows) =
            build_focus_file_graph_evidence(&graph, &source_lookup, "src/main.rs", &neighbors);

        assert!(!traces.is_empty());
        assert!(traces
            .iter()
            .any(|trace| trace.label.contains("src/support.rs -> src/main.rs")));
        assert!(traces.iter().all(|trace| {
            let first_source = trace
                .hops
                .first()
                .map(|hop| hop.source_file_path.as_str())
                .unwrap_or_default();
            let last_target = trace
                .hops
                .last()
                .map(|hop| hop.target_file_path.as_str())
                .unwrap_or_default();
            !trace.hops.is_empty() && first_source != last_target
        }));
        assert!(!code_flows.is_empty());
        assert!(!source_sink_paths.is_empty());
        assert!(semantic_state_flows.is_empty());
    }

    #[test]
    fn builds_reverse_graph_traces_when_support_flows_back_to_primary() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/support.rs"),
            None,
            PathBuf::from("src/main.rs"),
            String::from("symbol:src/main.rs"),
            ReferenceKind::Call,
            ResolutionTier::ImportScoped,
            910,
            String::from("support-to-main"),
            12,
        ));
        let packet = GuardianPacket {
            id: String::from("guardian:test-reverse-trace"),
            priority: String::from("high"),
            focus: String::from("architecture"),
            primary_target_file: String::from("src/main.rs"),
            precision: String::from("modeled"),
            confidence_millis: 850,
            summary: String::from("Reverse trace packet"),
            target_files: vec![String::from("src/main.rs"), String::from("src/support.rs")],
            primary_anchor: None,
            evidence_anchors: Vec::new(),
            locations: Vec::new(),
            finding_ids: vec![String::from("finding-2")],
            context_labels: Vec::new(),
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![String::from("guardian.test")],
            preferred_mechanism: None,
            obligations: Vec::new(),
            suppressibility: GuardianSuppressibility {
                allowed: true,
                requires_reason: true,
                expiry_required: true,
            },
            investigation_questions: Vec::new(),
        };

        let traces = build_graph_traces(&packet, &graph);

        assert!(traces
            .iter()
            .any(|trace| trace.kind == AgenticGraphTraceKind::ReverseSupportPath));
    }

    #[test]
    fn builds_alternate_graph_traces_when_multiple_paths_exist() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/a.rs"),
            None,
            PathBuf::from("src/b.rs"),
            String::from("symbol:src/b.rs"),
            ReferenceKind::Call,
            ResolutionTier::ImportScoped,
            900,
            String::from("a-b"),
            10,
        ));
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/b.rs"),
            None,
            PathBuf::from("src/goal.rs"),
            String::from("symbol:src/goal.rs"),
            ReferenceKind::Type,
            ResolutionTier::ImportScoped,
            880,
            String::from("b-goal"),
            11,
        ));
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/a.rs"),
            None,
            PathBuf::from("src/c.rs"),
            String::from("symbol:src/c.rs"),
            ReferenceKind::Call,
            ResolutionTier::ImportScoped,
            870,
            String::from("a-c"),
            12,
        ));
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/c.rs"),
            None,
            PathBuf::from("src/goal.rs"),
            String::from("symbol:src/goal.rs"),
            ReferenceKind::Type,
            ResolutionTier::ImportScoped,
            860,
            String::from("c-goal"),
            13,
        ));

        let packet = GuardianPacket {
            id: String::from("guardian:test-alt-trace"),
            priority: String::from("high"),
            focus: String::from("architecture"),
            primary_target_file: String::from("src/a.rs"),
            precision: String::from("modeled"),
            confidence_millis: 900,
            summary: String::from("Alternate trace packet"),
            target_files: vec![String::from("src/a.rs"), String::from("src/goal.rs")],
            primary_anchor: None,
            evidence_anchors: Vec::new(),
            locations: Vec::new(),
            finding_ids: vec![String::from("finding-3")],
            context_labels: Vec::new(),
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![String::from("guardian.test")],
            preferred_mechanism: None,
            obligations: Vec::new(),
            suppressibility: GuardianSuppressibility {
                allowed: true,
                requires_reason: true,
                expiry_required: true,
            },
            investigation_questions: Vec::new(),
        };

        let traces = build_graph_traces(&packet, &graph);

        assert!(traces.len() >= 2);
        assert!(traces.iter().any(|trace| trace.label.contains("[alt 2]")));
    }

    #[test]
    fn broad_duplicate_mechanism_packets_limit_alternate_path_search() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/a.rs"),
            None,
            PathBuf::from("src/b.rs"),
            String::from("symbol:src/b.rs"),
            ReferenceKind::Call,
            ResolutionTier::ImportScoped,
            900,
            String::from("a-b"),
            10,
        ));
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/b.rs"),
            None,
            PathBuf::from("src/goal.rs"),
            String::from("symbol:src/goal.rs"),
            ReferenceKind::Type,
            ResolutionTier::ImportScoped,
            880,
            String::from("b-goal"),
            11,
        ));
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/a.rs"),
            None,
            PathBuf::from("src/c.rs"),
            String::from("symbol:src/c.rs"),
            ReferenceKind::Call,
            ResolutionTier::ImportScoped,
            870,
            String::from("a-c"),
            12,
        ));
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/c.rs"),
            None,
            PathBuf::from("src/goal.rs"),
            String::from("symbol:src/goal.rs"),
            ReferenceKind::Type,
            ResolutionTier::ImportScoped,
            860,
            String::from("c-goal"),
            13,
        ));

        let packet = GuardianPacket {
            id: String::from("guardian:test-broad-duplicate"),
            priority: String::from("high"),
            focus: String::from("duplicate_mechanism"),
            primary_target_file: String::from("src/a.rs"),
            precision: String::from("modeled"),
            confidence_millis: 900,
            summary: String::from("Broad duplicate mechanism packet"),
            target_files: vec![
                String::from("src/a.rs"),
                String::from("src/goal.rs"),
                String::from("src/x.rs"),
                String::from("src/y.rs"),
                String::from("src/z.rs"),
            ],
            primary_anchor: None,
            evidence_anchors: Vec::new(),
            locations: Vec::new(),
            finding_ids: vec![String::from("finding-4")],
            context_labels: Vec::new(),
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![String::from("guardian.test")],
            preferred_mechanism: None,
            obligations: Vec::new(),
            suppressibility: GuardianSuppressibility {
                allowed: true,
                requires_reason: true,
                expiry_required: true,
            },
            investigation_questions: Vec::new(),
        };

        let traces = build_graph_traces(&packet, &graph);

        assert_eq!(traces.len(), 1);
        assert!(traces.iter().all(|trace| !trace.label.contains("[alt 2]")));
    }

    #[test]
    fn sanctioned_path_bypass_packets_skip_semantic_state_flows() {
        let packet = GuardianPacket {
            id: String::from("guardian:test-sanctioned-broad"),
            priority: String::from("high"),
            focus: String::from("sanctioned_path_bypass"),
            primary_target_file: String::from("src/artifacts.rs"),
            precision: String::from("heuristic"),
            confidence_millis: 800,
            summary: String::from("Broad sanctioned packet"),
            target_files: vec![
                String::from("src/artifacts.rs"),
                String::from("src/cli.rs"),
                String::from("src/agentic.rs"),
                String::from("src/doctrine/mod.rs"),
                String::from("src/mcp/mod.rs"),
            ],
            primary_anchor: None,
            evidence_anchors: Vec::new(),
            locations: Vec::new(),
            finding_ids: vec![String::from("finding-5")],
            context_labels: Vec::new(),
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![String::from("guardian.test")],
            preferred_mechanism: None,
            obligations: Vec::new(),
            suppressibility: GuardianSuppressibility {
                allowed: true,
                requires_reason: true,
                expiry_required: true,
            },
            investigation_questions: Vec::new(),
        };

        assert!(!packet_supports_semantic_state_flows(&packet));
        assert!(!should_build_semantic_state_flows(&packet, 4));
        assert!(!should_build_semantic_state_flows(&packet, 15));
    }

    #[test]
    fn graph_trace_prefers_more_causal_edges_over_plain_imports() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/a.rs"),
            None,
            PathBuf::from("src/b.rs"),
            String::from("symbol:src/b.rs:import"),
            ReferenceKind::Import,
            ResolutionTier::ImportScoped,
            900,
            String::from("import:import-scoped"),
            4,
        ));
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/a.rs"),
            None,
            PathBuf::from("src/b.rs"),
            String::from("symbol:src/b.rs:call"),
            ReferenceKind::Call,
            ResolutionTier::ImportScoped,
            850,
            String::from("call:import-scoped"),
            8,
        ));

        let traces = graph_trace_between_files(&graph, "src/a.rs", "src/b.rs", 3, 1);

        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].relation_sequence, vec![String::from("Call")]);
        assert_eq!(traces[0].hops[0].relation_kind, "Call");
    }

    #[test]
    fn strongest_context_edges_skip_same_file_self_paths() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/main.rs"),
            Some(String::from("symbol:src/main.rs:self")),
            PathBuf::from("src/main.rs"),
            String::from("symbol:src/main.rs:self-target"),
            ReferenceKind::Type,
            ResolutionTier::SameFile,
            950,
            String::from("type:same-file"),
            12,
        ));
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/main.rs"),
            None,
            PathBuf::from("src/support.rs"),
            String::from("symbol:src/support.rs"),
            ReferenceKind::Import,
            ResolutionTier::ImportScoped,
            900,
            String::from("import:import-scoped"),
            4,
        ));

        let target_files =
            HashSet::from([String::from("src/main.rs"), String::from("src/support.rs")]);
        let edges = strongest_context_edges(&graph, "src/main.rs", &target_files, 3);

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target_file_path, PathBuf::from("src/support.rs"));
    }

    #[test]
    fn builds_semantic_state_flows_for_shopware_like_carrier_usage() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/Criteria.php"),
            br#"<?php
class Criteria {
    public function addFilter($filter) {}
    public function getFilters() { return []; }
    public function setTerm($term) {}
    public function getTerm() { return ''; }
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/RequestCriteriaBuilder.php"),
            br#"<?php
class RequestCriteriaBuilder {
    public function build($criteria) {
        $criteria->addFilter('active');
        $criteria->setTerm('search');
    }
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/CriteriaQueryBuilder.php"),
            br#"<?php
class CriteriaQueryBuilder {
    public function run($criteria) {
        $criteria->getFilters();
        $criteria->getTerm();
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let source_lookup = build_source_lookup(&analysis.parsed_sources);
        let flows = build_semantic_state_flows(
            "src/Criteria.php",
            &[
                String::from("src/Criteria.php"),
                String::from("src/RequestCriteriaBuilder.php"),
                String::from("src/CriteriaQueryBuilder.php"),
            ],
            &analysis.semantic_graph,
            &source_lookup,
        );

        assert!(!flows.is_empty());
        assert!(flows.iter().any(|flow| {
            flow.carrier_type == "Criteria"
                && flow.slot == "filter"
                && flow.writer.file_path == "src/RequestCriteriaBuilder.php"
                && flow.reader.file_path == "src/CriteriaQueryBuilder.php"
        }));
        assert!(flows.iter().any(|flow| flow.slot == "term"));
    }

    #[test]
    fn semantic_state_flows_merge_heuristic_calls_into_mixed_resolution_files() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/Criteria.php"),
            br#"<?php
class Criteria {
    public function addFilter($filter) {}
    public function getFilters() { return []; }
    public function setTerm($term) {}
    public function getTerm() { return ''; }
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/MixedConsumer.php"),
            br#"<?php
class MixedConsumer {
    public function run(Criteria $criteria) {
        $criteria->getTerm();
        $alias = $criteria;
        $alias->setTerm('search');
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let source_lookup = build_source_lookup(&analysis.parsed_sources);
        let flows = build_semantic_state_flows(
            "src/Criteria.php",
            &[
                String::from("src/Criteria.php"),
                String::from("src/MixedConsumer.php"),
            ],
            &analysis.semantic_graph,
            &source_lookup,
        );

        assert!(flows.iter().any(|flow| {
            flow.carrier_type == "Criteria"
                && flow.slot == "term"
                && flow.writer.file_path == "src/MixedConsumer.php"
                && flow.reader.file_path == "src/MixedConsumer.php"
        }));
    }

    #[test]
    fn builds_semantic_state_flows_for_keyed_carrier_arguments() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/Entity.php"),
            br#"<?php
class Entity {
    public function setCustomField(string $name, mixed $value) {}
    public function getCustomField(string $name, mixed $default = null) {}
    public function setDynamicColumn(string $name, mixed $value) {}
    public function getDynamicColumn(string $name) {}
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/Writer.php"),
            br#"<?php
class Writer {
    public function populate(Entity $entity, string $relationName) {
        $entity->setCustomField('_'.$relationName, null);
        $entity->setDynamicColumn('status', 'open');
    }
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/Reader.php"),
            br#"<?php
class Reader {
    public function inspect(Entity $entity, string $relationName) {
        $entity->getCustomField('_'.$relationName);
        $entity->getDynamicColumn('status');
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let source_lookup = build_source_lookup(&analysis.parsed_sources);
        let flows = build_semantic_state_flows(
            "src/Entity.php",
            &[
                String::from("src/Entity.php"),
                String::from("src/Writer.php"),
                String::from("src/Reader.php"),
            ],
            &analysis.semantic_graph,
            &source_lookup,
        );

        assert!(flows.iter().any(|flow| {
            flow.carrier_type == "Entity"
                && flow.slot == "customField['_'.$relationName]"
                && flow.writer.file_path == "src/Writer.php"
                && flow.reader.file_path == "src/Reader.php"
        }));
        assert!(flows
            .iter()
            .any(|flow| flow.slot == "dynamicColumn['status']"));
    }

    #[test]
    fn builds_semantic_state_flows_for_keyed_carrier_indirect_reader_keys() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/HasJsonbStorage.php"),
            br#"<?php
trait HasJsonbStorage {
    public function setCustomField(string $name, mixed $value) {}
    public function getCustomField(string $name, mixed $default = null) {}
    public function setDynamicColumn(string $name, mixed $value) {}
    public function getDynamicColumn(string $name) {}
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/RelationBatchLoader.php"),
            br#"<?php
class RelationBatchLoader {
    public function load($entity, $relationName, $parent) {
        $entity->setCustomField('_'.$relationName, $parent);
    }
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/GetEntityTool.php"),
            br#"<?php
class GetEntityTool {
    public function run($entity, $relationName) {
        $relationKey = '_'.$relationName;
        return $entity->getCustomField($relationKey);
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let source_lookup = build_source_lookup(&analysis.parsed_sources);
        let flows = build_semantic_state_flows(
            "src/HasJsonbStorage.php",
            &[
                String::from("src/HasJsonbStorage.php"),
                String::from("src/RelationBatchLoader.php"),
                String::from("src/GetEntityTool.php"),
            ],
            &analysis.semantic_graph,
            &source_lookup,
        );

        assert!(flows.iter().any(|flow| {
            flow.slot == "customField['_'.$relationName]"
                && flow.writer.file_path == "src/RelationBatchLoader.php"
                && flow.reader.file_path == "src/GetEntityTool.php"
        }));
    }

    #[test]
    fn trait_backed_carrier_flows_upgrade_to_receiver_typed_when_trait_use_resolves() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/CarrierTrait.php"),
            br#"<?php
trait CarrierTrait {
    public function setCustomField(string $name, mixed $value) {}
    public function getCustomField(string $name, mixed $default = null) {}
    public function setDynamicColumn(string $name, mixed $value) {}
    public function getDynamicColumn(string $name) {}
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/Entity.php"),
            br#"<?php
class Entity {
    use CarrierTrait;
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/Writer.php"),
            br#"<?php
class Writer {
    public function run(Entity $entity, string $relationName, mixed $parent) {
        $entity->setCustomField('_'.$relationName, $parent);
    }
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/Reader.php"),
            br#"<?php
class Reader {
    public function run(Entity $entity, string $relationName) {
        $relationKey = '_'.$relationName;
        return $entity->getCustomField($relationKey);
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let source_lookup = build_source_lookup(&analysis.parsed_sources);
        let flows = build_semantic_state_flows(
            "src/CarrierTrait.php",
            &[
                String::from("src/CarrierTrait.php"),
                String::from("src/Entity.php"),
                String::from("src/Writer.php"),
                String::from("src/Reader.php"),
            ],
            &analysis.semantic_graph,
            &source_lookup,
        );

        assert!(flows.iter().any(|flow| {
            flow.slot == "customField['_'.$relationName]"
                && matches!(flow.writer.proof, SemanticStateProofKind::ReceiverTyped)
                && matches!(flow.reader.proof, SemanticStateProofKind::ReceiverTyped)
        }));
    }

    #[test]
    fn semantic_state_flows_keep_same_line_occurrences_distinct() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/Criteria.php"),
            br#"<?php
class Criteria {
    public function addFilter($filter) {}
    public function getFilters() { return []; }
    public function setTerm($term) {}
    public function getTerm() { return ''; }
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/MixedConsumer.php"),
            br#"<?php
class MixedConsumer {
    public function run(Criteria $typed, $alias) {
        $typed->setTerm('ignored'); $alias->setTerm('search');
        return $alias->getTerm();
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let source_lookup = build_source_lookup(&analysis.parsed_sources);
        let flows = build_semantic_state_flows(
            "src/Criteria.php",
            &[
                String::from("src/Criteria.php"),
                String::from("src/MixedConsumer.php"),
            ],
            &analysis.semantic_graph,
            &source_lookup,
        );

        assert!(flows.iter().any(|flow| {
            flow.slot == "term"
                && flow.writer.file_path == "src/MixedConsumer.php"
                && flow.reader.file_path == "src/MixedConsumer.php"
                && matches!(flow.writer.proof, SemanticStateProofKind::Heuristic)
        }));
    }

    #[test]
    fn semantic_state_flows_skip_thin_non_carrier_files() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/UserService.php"),
            br#"<?php
class UserService {
    public function getUser() {}
    public function setUser($user) {}
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/UserController.php"),
            br#"<?php
class UserController {
    public function show($service) {
        $service->getUser();
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let source_lookup = build_source_lookup(&analysis.parsed_sources);
        let flows = build_semantic_state_flows(
            "src/UserService.php",
            &[
                String::from("src/UserService.php"),
                String::from("src/UserController.php"),
            ],
            &analysis.semantic_graph,
            &source_lookup,
        );

        assert!(flows.iter().all(|flow| flow.carrier_type != "Criteria"));
    }

    #[test]
    fn semantic_state_flows_skip_wrong_receiver_type_matches() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/Criteria.php"),
            br#"<?php
class Criteria {
    public function addFilter($filter) {}
    public function setLimit($limit) {}
    public function getFilters() {}
    public function getLimit() {}
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/OtherService.php"),
            br#"<?php
class OtherService {
    public function addFilter($filter) {}
    public function setLimit($limit) {}
    public function getFilters() {}
    public function getLimit() {}
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/WrongReader.php"),
            br#"<?php
class WrongReader {
    public function inspect(OtherService $other) {
        $other->getFilters();
        $other->getLimit();
    }
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/WrongWriter.php"),
            br#"<?php
class WrongWriter {
    public function populate(OtherService $other) {
        $other->addFilter('x');
        $other->setLimit(10);
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let source_lookup = build_source_lookup(&analysis.parsed_sources);
        let flows = build_semantic_state_flows(
            "src/Criteria.php",
            &[
                String::from("src/Criteria.php"),
                String::from("src/WrongWriter.php"),
                String::from("src/WrongReader.php"),
                String::from("src/OtherService.php"),
            ],
            &analysis.semantic_graph,
            &source_lookup,
        );

        assert!(flows.iter().all(|flow| flow.carrier_type != "Criteria"));
    }

    #[test]
    fn semantic_state_flows_prefer_same_symbol_pairs_over_earlier_unrelated_reads() {
        let fixture = create_fixture();
        std::fs::create_dir_all(fixture.join("src")).unwrap();
        std::fs::write(
            fixture.join("src/Criteria.php"),
            br#"<?php
class Criteria {
    public function addFilter($filter) {}
    public function getFilters() {}
    public function setLimit($limit) {}
    public function getLimit() {}
}
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.join("src/Consumer.php"),
            br#"<?php
class Consumer {
    public function unrelated(Criteria $criteria) {
        $criteria->getFilters();
    }

    public function paired(Criteria $criteria) {
        $criteria->addFilter('x');
        $criteria->getFilters();
    }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let source_lookup = build_source_lookup(&analysis.parsed_sources);
        let flows = build_semantic_state_flows(
            "src/Criteria.php",
            &[
                String::from("src/Criteria.php"),
                String::from("src/Consumer.php"),
            ],
            &analysis.semantic_graph,
            &source_lookup,
        );

        let filter_flow = flows
            .iter()
            .find(|flow| flow.carrier_type == "Criteria" && flow.slot == "filter")
            .expect("expected filter semantic state flow");

        assert_eq!(filter_flow.writer.symbol_name.as_deref(), Some("paired"));
        assert_eq!(filter_flow.reader.symbol_name.as_deref(), Some("paired"));
        assert!(matches!(
            filter_flow.writer.proof,
            SemanticStateProofKind::ReceiverTyped | SemanticStateProofKind::ExactResolved
        ));
        assert!(matches!(
            filter_flow.reader.proof,
            SemanticStateProofKind::ReceiverTyped | SemanticStateProofKind::ExactResolved
        ));
    }

    #[test]
    fn focuses_review_on_single_task_packet() {
        let artifact = AgenticReviewArtifact {
            root: String::from("/tmp/example"),
            contract_version: String::from("2026-03-28"),
            transport: AgenticTransportContract {
                provider_family: String::from("openai"),
                recommended_protocol: String::from("responses_api"),
                recommended_auth: String::from("api_key"),
                recommended_default_model: String::from("gpt-5.4"),
                recommended_coding_models: vec![String::from("gpt-5.3-codex")],
                recommended_tool_runtime: String::from("responses_api_shell_tool"),
                supports_background_responses: true,
                shell_tool_supported: true,
                browser_oauth_supported_as_primary: false,
                official_rust_sdk_documented: false,
                official_codex_sdk_strategy: String::from("optional_typescript_sidecar"),
                implementation_guidance: vec![String::from("Keep graph artifacts primary.")],
            },
            execution: build_execution_contract(
                "system prompt",
                "user prompt",
                &[AgenticTaskPacket {
                    id: String::from("guardian:test"),
                    status: String::from("new"),
                    priority: String::from("high"),
                    focus: String::from("architecture"),
                    title: String::from("Test packet"),
                    summary: String::from("Test packet"),
                    primary_target_file: String::from("src/main.rs"),
                    primary_anchor: None,
                    evidence_anchors: Vec::new(),
                    locations: Vec::new(),
                    evidence_refs: AgenticPrimaryEvidenceRefs::default(),
                    doctrine_refs: vec![String::from("guardian.test")],
                    preferred_mechanism: Some(String::from("preferred_path")),
                    obligations: Vec::new(),
                    required_artifacts: vec![String::from("dependency-graph.json")],
                    evidence_chain: AgenticEvidenceChain {
                        claim: String::from("Test packet"),
                        artifact_refs: vec![String::from("dependency-graph.json")],
                        locations: vec![AgenticEvidenceLocation {
                            role: String::from("primary"),
                            file_path: String::from("src/main.rs"),
                            line: Some(1),
                        }],
                        source_sink_paths: Vec::new(),
                        code_flows: Vec::new(),
                        graph_traces: Vec::new(),
                        semantic_state_flows: Vec::new(),
                    },
                    review_radius_files: vec![String::from("src/main.rs")],
                }],
            ),
            graph_priority: AgenticGraphPriority {
                architecture_source: String::from("dependency-graph.json"),
                evidence_source: String::from("evidence-graph.json"),
                runtime_contract_source: String::from("contract-inventory.json"),
                doctrine_source: String::from("doctrine-registry.json"),
                guard_source: String::from("guard-decision.json"),
            },
            summary: AgenticReviewSummary {
                guard_verdict: String::from("Warn"),
                visible_findings: 1,
                guardian_packet_count: 1,
                top_focus_files: vec![String::from("src/main.rs")],
                doctrine_refs: vec![String::from("guardian.test")],
            },
            diff_summary: AgenticDiffSummary {
                new_findings: 1,
                worsened_findings: 0,
                improved_findings: 0,
                resolved_findings: 0,
                attention_items: 1,
            },
            context_artifacts: vec![AgenticContextArtifact {
                file: String::from("dependency-graph.json"),
                role: String::from("graph"),
            }],
            system_prompt: String::from("system prompt"),
            user_prompt: String::from("user prompt"),
            task_packets: vec![AgenticTaskPacket {
                id: String::from("guardian:test"),
                status: String::from("new"),
                priority: String::from("high"),
                focus: String::from("architecture"),
                title: String::from("Test packet"),
                summary: String::from("Test packet"),
                primary_target_file: String::from("src/main.rs"),
                primary_anchor: None,
                evidence_anchors: Vec::new(),
                locations: Vec::new(),
                evidence_refs: AgenticPrimaryEvidenceRefs::default(),
                doctrine_refs: vec![String::from("guardian.test")],
                preferred_mechanism: Some(String::from("preferred_path")),
                obligations: Vec::new(),
                required_artifacts: vec![String::from("dependency-graph.json")],
                evidence_chain: AgenticEvidenceChain {
                    claim: String::from("Test packet"),
                    artifact_refs: vec![String::from("dependency-graph.json")],
                    locations: vec![AgenticEvidenceLocation {
                        role: String::from("primary"),
                        file_path: String::from("src/main.rs"),
                        line: Some(1),
                    }],
                    source_sink_paths: Vec::new(),
                    code_flows: Vec::new(),
                    graph_traces: Vec::new(),
                    semantic_state_flows: Vec::new(),
                },
                review_radius_files: vec![String::from("src/main.rs")],
            }],
            guardian_packets: Vec::new(),
            top_findings: vec![AgentHandoffFinding {
                id: String::from("finding-1"),
                family: String::from("graph"),
                severity: String::from("high"),
                title: String::from("Example finding"),
                summary: String::from("Bounded fallback finding"),
                file_paths: vec![String::from("src/main.rs")],
                line: Some(1),
                primary_anchor: None,
                locations: Vec::new(),
            }],
            next_steps: vec![String::from("Inspect src/main.rs")],
        };
        let focused = focus_agentic_review_artifact(&artifact, "guardian:test").unwrap();
        assert_eq!(focused.task_packets.len(), 1);
        assert_eq!(focused.task_packets[0].id, "guardian:test");
        assert_eq!(
            focused.execution.structured_output.must_cover_task_packets,
            vec![String::from("guardian:test")]
        );
        assert!(focused.user_prompt.contains("guardian:test"));
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscode-agentic-fixture-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
