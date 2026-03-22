use crate::artifacts::{
    AgentHandoffArtifact, AgentHandoffFinding, ConvergenceHistoryArtifact, ConvergenceStatus,
    GuardDecisionArtifact, GuardVerdict, GuardianObligation, GuardianPacket,
};
use crate::doctrine::DoctrineRegistry;
use crate::evidence::EvidenceAnchor;
use crate::graph::{ResolvedEdge, SemanticGraph, SymbolNode};
use crate::ingestion::pipeline::ProjectAnalysis;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub doctrine_refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_mechanism: Option<String>,
    pub obligations: Vec<GuardianObligation>,
    pub required_artifacts: Vec<String>,
    pub evidence_chain: AgenticEvidenceChain,
    pub review_radius_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticEvidenceChain {
    pub claim: String,
    pub artifact_refs: Vec<String>,
    pub locations: Vec<AgenticEvidenceLocation>,
    pub graph_traces: Vec<AgenticGraphTrace>,
    pub code_flows: Vec<AgenticCodeFlow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticEvidenceLocation {
    pub role: String,
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticGraphTrace {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgenticGraphTraceKind {
    DirectedSupportPath,
    ReverseSupportPath,
    ContextualSupportPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticCodeFlow {
    pub label: String,
    pub kind: AgenticCodeFlowKind,
    pub entry_file_path: String,
    pub exit_file_path: String,
    pub aggregate_confidence_millis: u16,
    pub relation_sequence: Vec<String>,
    pub truncated: bool,
    pub steps: Vec<AgenticCodeFlowStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgenticCodeFlowKind {
    ForwardPropagation,
    BackwardPropagation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgenticCodeFlowStep {
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_to_next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_file_path: Option<String>,
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
    let user_prompt = build_user_prompt(
        analysis,
        handoff,
        guard_decision,
        convergence,
        &focus_files,
        &doctrine_refs,
    );
    let task_packets = build_task_packets(analysis, handoff, guard_decision, convergence);
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
    ];
    let execution = build_execution_contract(&system_prompt, &user_prompt, &task_packets);

    AgenticReviewArtifact {
        root: analysis.root.display().to_string(),
        contract_version: String::from("2026-03-22"),
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
        schema_version: String::from("2026-03-22"),
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
) -> String {
    let task_summaries = build_task_packets(analysis, handoff, guard_decision, convergence)
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
    analysis: &ProjectAnalysis,
    handoff: &AgentHandoffArtifact,
    guard_decision: &GuardDecisionArtifact,
    convergence: &ConvergenceHistoryArtifact,
) -> Vec<AgenticTaskPacket> {
    let mut packets = handoff
        .guardian_packets
        .iter()
        .map(|packet| {
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
            AgenticTaskPacket {
                id: packet.id.clone(),
                status,
                priority: packet.priority.clone(),
                focus: packet.focus.clone(),
                title: packet.summary.clone(),
                summary: packet.summary.clone(),
                primary_target_file: packet.primary_target_file.clone(),
                primary_anchor: packet.primary_anchor.clone(),
                evidence_anchors: packet.evidence_anchors.clone(),
                doctrine_refs: packet.doctrine_refs.clone(),
                preferred_mechanism: packet.preferred_mechanism.clone(),
                obligations: packet.obligations.clone(),
                required_artifacts: required_artifacts.clone(),
                evidence_chain: build_evidence_chain(
                    packet,
                    &required_artifacts,
                    &analysis.semantic_graph,
                ),
                review_radius_files,
            }
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

fn build_evidence_chain(
    packet: &GuardianPacket,
    required_artifacts: &[String],
    semantic_graph: &SemanticGraph,
) -> AgenticEvidenceChain {
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

    let graph_traces = build_graph_traces(packet, semantic_graph);

    AgenticEvidenceChain {
        claim: packet.summary.clone(),
        artifact_refs: required_artifacts.to_vec(),
        locations,
        code_flows: build_code_flows(&graph_traces),
        graph_traces,
    }
}

fn build_graph_traces(
    packet: &GuardianPacket,
    semantic_graph: &SemanticGraph,
) -> Vec<AgenticGraphTrace> {
    let primary = packet.primary_target_file.as_str();
    let target_files = packet.target_files.iter().cloned().collect::<HashSet<_>>();
    let symbol_lookup = build_symbol_lookup(semantic_graph);
    let mut traces = Vec::new();
    let mut seen_labels = HashSet::new();
    let max_hops = 5;
    let path_limit = 2;

    for target in packet
        .target_files
        .iter()
        .filter(|target| target.as_str() != primary)
        .take(3)
    {
        let directed_paths =
            find_directed_file_paths(semantic_graph, primary, target, max_hops, path_limit);
        if !directed_paths.is_empty() {
            for (index, path) in directed_paths.iter().enumerate() {
                let label = trace_label(primary, target, index);
                if seen_labels.insert(label.clone()) {
                    traces.push(AgenticGraphTrace {
                        label,
                        kind: AgenticGraphTraceKind::DirectedSupportPath,
                        primary_file_path: primary.to_owned(),
                        supporting_file_path: Some(target.clone()),
                        aggregate_confidence_millis: aggregate_path_confidence(path),
                        relation_sequence: relation_sequence(path),
                        truncated: false,
                        hops: build_trace_hops(path, &symbol_lookup),
                    });
                }
            }
            continue;
        }

        let reverse_paths =
            find_directed_file_paths(semantic_graph, target, primary, max_hops, path_limit);
        for (index, path) in reverse_paths.iter().enumerate() {
            let label = trace_label(target, primary, index);
            if seen_labels.insert(label.clone()) {
                traces.push(AgenticGraphTrace {
                    label,
                    kind: AgenticGraphTraceKind::ReverseSupportPath,
                    primary_file_path: primary.to_owned(),
                    supporting_file_path: Some(target.clone()),
                    aggregate_confidence_millis: aggregate_path_confidence(path),
                    relation_sequence: relation_sequence(path),
                    truncated: false,
                    hops: build_trace_hops(path, &symbol_lookup),
                });
            }
        }
    }

    if traces.is_empty() {
        for edge in strongest_context_edges(semantic_graph, primary, &target_files, 3) {
            let source = edge.source_file_path.display().to_string();
            let target = edge.target_file_path.display().to_string();
            let label = format!("{source} -> {target}");
            if seen_labels.insert(label.clone()) {
                traces.push(AgenticGraphTrace {
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
                    hops: build_trace_hops(&[edge], &symbol_lookup),
                });
            }
        }
    }

    traces
}

fn build_code_flows(traces: &[AgenticGraphTrace]) -> Vec<AgenticCodeFlow> {
    traces
        .iter()
        .filter(|trace| !trace.hops.is_empty())
        .map(|trace| {
            let mut steps = Vec::new();
            for (index, hop) in trace.hops.iter().enumerate() {
                if index == 0 {
                    steps.push(AgenticCodeFlowStep {
                        file_path: hop.source_file_path.clone(),
                        symbol_name: hop.source_symbol_name.clone(),
                        line: Some(hop.line),
                        relation_to_next: Some(hop.relation_kind.clone()),
                        next_file_path: Some(hop.target_file_path.clone()),
                    });
                }
                steps.push(AgenticCodeFlowStep {
                    file_path: hop.target_file_path.clone(),
                    symbol_name: hop.target_symbol_name.clone(),
                    line: Some(hop.line),
                    relation_to_next: trace
                        .hops
                        .get(index + 1)
                        .map(|next_hop| next_hop.relation_kind.clone()),
                    next_file_path: trace
                        .hops
                        .get(index + 1)
                        .map(|next_hop| next_hop.target_file_path.clone()),
                });
            }
            AgenticCodeFlow {
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
                entry_file_path: trace
                    .hops
                    .first()
                    .map(|hop| hop.source_file_path.clone())
                    .unwrap_or_else(|| trace.primary_file_path.clone()),
                exit_file_path: trace
                    .hops
                    .last()
                    .map(|hop| hop.target_file_path.clone())
                    .unwrap_or_else(|| trace.primary_file_path.clone()),
                aggregate_confidence_millis: trace.aggregate_confidence_millis,
                relation_sequence: trace.relation_sequence.clone(),
                truncated: trace.truncated,
                steps,
            }
        })
        .collect()
}

fn trace_label(start: &str, goal: &str, index: usize) -> String {
    if index == 0 {
        format!("{start} -> {goal}")
    } else {
        format!("{start} -> {goal} [alt {}]", index + 1)
    }
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
    semantic_graph: &'a SemanticGraph,
    start: &str,
    goal: &str,
    max_hops: usize,
    path_limit: usize,
) -> Vec<Vec<&'a ResolvedEdge>> {
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
        for edge in semantic_graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.source_file_path.display().to_string() == current)
        {
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
        build_graph_traces, focus_agentic_review_artifact, AgenticContextArtifact,
        AgenticDiffSummary, AgenticEvidenceChain, AgenticEvidenceLocation, AgenticGraphPriority,
        AgenticGraphTraceKind, AgenticReviewArtifact, AgenticReviewSummary, AgenticTaskPacket,
        AgenticTransportContract,
    };
    use crate::artifacts::{
        build_agent_handoff_artifact, build_guard_decision_artifact, AgentHandoffArtifact,
        AgentHandoffFinding, GuardianPacket, GuardianSuppressibility,
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
        assert!(artifact
            .execution
            .report_targets
            .iter()
            .any(|target| target.file_name == "agent-review.md"));
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
    fn focuses_review_on_single_task_packet() {
        let artifact = AgenticReviewArtifact {
            root: String::from("/tmp/example"),
            contract_version: String::from("2026-03-22"),
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
                        code_flows: Vec::new(),
                        graph_traces: Vec::new(),
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
                    code_flows: Vec::new(),
                    graph_traces: Vec::new(),
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
