mod contracts;

use self::contracts::{
    build_finding_details, display_path, family_matches, language_matches, path_matches,
    severity_matches, AtlasOutput, BottleneckOutput, ContractInventoryOutput, ConvergenceOutput,
    CoverageReportOutput, CycleOutput, CyclesOutput, CypherQueryOutput, CypherQueryParams,
    DoctrineRegistryOutput, ExplainFindingParams, FindingDetailOutput, FindingSummaryOutput,
    GuardDecisionOutput, HotspotOutput, HotspotsOutput, ListFindingsOutput, ListFindingsParams,
    QualityEvaluationOutput, RepoOverviewOutput, ShowCyclesParams, ShowHotspotsParams,
};
use crate::agentic::{
    build_graph_packet_artifact, graph_neighbors_for_file, graph_trace_between_files,
    GraphNeighborsOutput, GraphNeighborsParams, GraphPacketArtifact, GraphTraceOutput,
    GraphTraceParams, ListGraphPacketsParams,
};
use crate::artifacts::{
    build_agent_handoff_artifact, write_project_analysis_artifacts, AgentHandoffArtifact,
    ArtifactPaths, ConvergenceHistoryArtifact, GuardDecisionArtifact, RepositoryTopologyArtifact,
};
use crate::doctrine::{load_doctrine_registry, DoctrineLoadError};
use crate::ingestion::pipeline::{analyze_project, ProjectAnalysis, ProjectAnalysisError};
use crate::ingestion::scan::ScanConfig;
use crate::kuzu_index::{
    build_dependency_graph_artifact, build_evidence_graph_artifact, default_kuzu_path, query_kuzu,
    schema_reference_markdown, write_semantic_graph_kuzu_artifact, DependencyGraphArtifact,
    EvidenceGraphArtifact, KuzuIndexError,
};
use crate::policy::PolicyLoadError;
use crate::review::load_review_surface;
use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsResult,
    ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParams, PromptMessage,
    PromptMessageRole, RawResource, RawResourceTemplate, ReadResourceRequestParams,
    ReadResourceResult, ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{model::AnnotateAble, prompt_handler, tool_handler};
use rmcp::{
    prompt, prompt_router, tool, tool_router, ErrorData as McpError, Json, RoleServer,
    ServerHandler, ServiceExt,
};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const OVERVIEW_URI: &str = "aigiscode://repo/current/overview";
const FINDINGS_URI: &str = "aigiscode://repo/current/findings";
const ATLAS_URI: &str = "aigiscode://repo/current/atlas";
const HOTSPOTS_URI: &str = "aigiscode://repo/current/hotspots";
const COVERAGE_URI: &str = "aigiscode://repo/current/coverage";
const CYCLES_URI: &str = "aigiscode://repo/current/cycles";
const QUALITY_URI: &str = "aigiscode://repo/current/quality";
const GRAPH_SCHEMA_URI: &str = "aigiscode://repo/current/graph-schema";
const DEPENDENCY_GRAPH_URI: &str = "aigiscode://repo/current/dependency-graph";
const EVIDENCE_GRAPH_URI: &str = "aigiscode://repo/current/evidence-graph";
const CONTRACTS_URI: &str = "aigiscode://repo/current/contracts";
const DOCTRINE_URI: &str = "aigiscode://repo/current/doctrine";
const HANDOFF_URI: &str = "aigiscode://repo/current/handoff";
const CONVERGENCE_URI: &str = "aigiscode://repo/current/convergence";
const GUARD_URI: &str = "aigiscode://repo/current/guard";
const GRAPH_PACKETS_URI: &str = "aigiscode://repo/current/graph-packets";
const REPOSITORY_TOPOLOGY_URI: &str = "aigiscode://repo/current/repository-topology";
const FINDING_TEMPLATE_URI: &str = "aigiscode://repo/current/finding/{finding_id}";
const FINDING_URI_PREFIX: &str = "aigiscode://repo/current/finding/";

#[derive(Debug, Error)]
pub enum McpServerError {
    #[error(transparent)]
    Analysis(#[from] ProjectAnalysisError),
    #[error(transparent)]
    Policy(#[from] PolicyLoadError),
    #[error(transparent)]
    Doctrine(#[from] DoctrineLoadError),
    #[error("failed to write AigisCode artifacts: {0}")]
    WriteArtifacts(#[source] std::io::Error),
    #[error("failed to materialize Kuzu graph artifact: {0}")]
    Kuzu(#[from] KuzuIndexError),
    #[error("failed to start MCP server: {0}")]
    Startup(Box<rmcp::service::ServerInitializeError>),
    #[error("MCP server task failed: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("failed to create Tokio runtime: {0}")]
    Runtime(#[source] std::io::Error),
}

impl From<rmcp::service::ServerInitializeError> for McpServerError {
    fn from(err: rmcp::service::ServerInitializeError) -> Self {
        Self::Startup(Box::new(err))
    }
}

pub fn run_stdio_server(
    root: PathBuf,
    output_dir: Option<PathBuf>,
    write_artifacts: bool,
    write_kuzu: bool,
) -> Result<(), McpServerError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(McpServerError::Runtime)?;
    runtime.block_on(async move {
        let server =
            AigiscodeMcpServer::load(root, output_dir.as_deref(), write_artifacts, write_kuzu)?;
        server
            .serve(rmcp::transport::stdio())
            .await?
            .waiting()
            .await?;
        Ok(())
    })
}

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
impl ServerHandler for AigiscodeMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
        )
        .with_server_info(Implementation::new("aigiscode", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "AigisCode provides single-repo architectural analysis over native Rust artifacts. \
             Start with repo_overview, then drill into findings, hotspots, cycles, and coverage.",
        )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(self.resource_catalog()))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![
            RawResourceTemplate::new(FINDING_TEMPLATE_URI, "finding")
                .with_title("Finding Detail")
                .with_description("Structured detail for one finding by MCP finding id.")
                .with_mime_type("application/json")
                .no_annotation(),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let (uri, payload) = self.read_resource_payload(&request.uri)?;
        Ok(ReadResourceResult::new(vec![ResourceContents::text(
            payload, uri,
        )
        .with_mime_type("application/json")]))
    }
}

pub struct AigiscodeMcpServer {
    state: McpState,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

impl AigiscodeMcpServer {
    pub fn load(
        root: PathBuf,
        output_dir: Option<&Path>,
        write_artifacts: bool,
        write_kuzu: bool,
    ) -> Result<Self, McpServerError> {
        let analysis = analyze_project(root, &ScanConfig::default())?;
        let artifact_paths = if write_artifacts {
            write_project_analysis_artifacts(&analysis, output_dir)
                .map_err(McpServerError::WriteArtifacts)?
        } else {
            let output_dir = output_dir
                .map(Path::to_path_buf)
                .unwrap_or_else(|| analysis.root.join(".aigiscode"));
            ArtifactPaths {
                deterministic_analysis: output_dir.join("deterministic-analysis.json"),
                semantic_graph: output_dir.join("semantic-graph.json"),
                dependency_graph: output_dir.join("dependency-graph.json"),
                evidence_graph: output_dir.join("evidence-graph.json"),
                contract_inventory: output_dir.join("contract-inventory.json"),
                doctrine_registry: output_dir.join("doctrine-registry.json"),
                deterministic_findings: output_dir.join("deterministic-findings.json"),
                ast_grep_scan: output_dir.join("ast-grep-scan.json"),
                external_analysis: output_dir.join("external-analysis.json"),
                architecture_surface: output_dir.join("architecture-surface.json"),
                review_surface: output_dir.join("review-surface.json"),
                convergence_history: output_dir.join("convergence-history.json"),
                guard_decision: output_dir.join("guard-decision.json"),
                agent_handoff: output_dir.join("aigiscode-handoff.json"),
                agentic_review: output_dir.join("agentic-review.json"),
                graph_packets: output_dir.join("graph-packets.json"),
                repository_topology: output_dir.join("repository-topology.json"),
                aigiscode_report: output_dir.join("aigiscode-report.json"),
                aigiscode_report_markdown: output_dir.join("aigiscode-report.md"),
                output_dir,
            }
        };
        let kuzu_path = if write_kuzu {
            Some(write_semantic_graph_kuzu_artifact(
                &analysis.root,
                &analysis.semantic_graph,
                output_dir,
            )?)
        } else {
            let candidate = default_kuzu_path(&analysis.root, output_dir);
            candidate.exists().then_some(candidate)
        };
        Self::new(analysis, artifact_paths, kuzu_path)
    }

    fn new(
        analysis: ProjectAnalysis,
        artifact_paths: ArtifactPaths,
        kuzu_path: Option<PathBuf>,
    ) -> Result<Self, McpServerError> {
        let state = McpState::new(analysis, artifact_paths, kuzu_path)?;
        Ok(Self {
            state,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        })
    }
}

#[tool_router]
#[prompt_router]
impl AigiscodeMcpServer {
    #[tool(
        name = "repo_overview",
        description = "Return repository architecture overview, top findings, and artifact locations."
    )]
    async fn repo_overview(&self) -> Json<RepoOverviewOutput> {
        Json(self.state.repo_overview.clone())
    }

    #[tool(
        name = "list_findings",
        description = "List architecture findings filtered by family, severity, path, and language."
    )]
    async fn list_findings(
        &self,
        Parameters(params): Parameters<ListFindingsParams>,
    ) -> Json<ListFindingsOutput> {
        let max_items = params.max_items.unwrap_or(100).clamp(1, 500);
        let findings = self
            .state
            .finding_summaries
            .iter()
            .filter(|finding| {
                family_matches(&finding.family, params.family)
                    && severity_matches(&finding.severity, params.severity)
                    && path_matches(finding, params.file_path.as_deref())
                    && language_matches(finding, params.language.as_deref())
            })
            .take(max_items)
            .cloned()
            .collect::<Vec<_>>();
        Json(ListFindingsOutput {
            total: findings.len(),
            findings,
        })
    }

    #[tool(
        name = "explain_finding",
        description = "Return structured detail and evidence for a single AigisCode finding id."
    )]
    async fn explain_finding(
        &self,
        Parameters(params): Parameters<ExplainFindingParams>,
    ) -> Result<Json<FindingDetailOutput>, String> {
        self.state
            .finding_details
            .get(&params.finding_id)
            .cloned()
            .map(Json)
            .ok_or_else(|| format!("unknown finding id: {}", params.finding_id))
    }

    #[tool(
        name = "show_hotspots",
        description = "Return hotspot files ranked by findings, coupling, and bottleneck pressure."
    )]
    async fn show_hotspots(
        &self,
        Parameters(params): Parameters<ShowHotspotsParams>,
    ) -> Json<HotspotsOutput> {
        let max_items = params.max_items.unwrap_or(20).clamp(1, 100);
        Json(HotspotsOutput {
            hotspots: self
                .state
                .hotspots
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            bottlenecks: self
                .state
                .bottlenecks
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            orphan_files: self
                .state
                .orphan_files
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            boundary_truncated_files: self
                .state
                .boundary_truncated_files
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            runtime_entry_candidates: self
                .state
                .runtime_entry_candidates
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
        })
    }

    #[tool(
        name = "show_cycles",
        description = "Return strong cycles and all dependency cycles from the current analysis run."
    )]
    async fn show_cycles(
        &self,
        Parameters(params): Parameters<ShowCyclesParams>,
    ) -> Json<CyclesOutput> {
        let max_items = params.max_items.unwrap_or(25).clamp(1, 100);
        Json(CyclesOutput {
            strong_cycles: self
                .state
                .strong_cycles
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            total_cycles: if params.strong_only.unwrap_or(false) {
                Vec::new()
            } else {
                self.state
                    .total_cycles
                    .iter()
                    .take(max_items)
                    .cloned()
                    .collect()
            },
        })
    }

    #[tool(
        name = "coverage_report",
        description = "Return language coverage, unresolved-reference pressure, and current parity notes."
    )]
    async fn coverage_report(&self) -> Json<CoverageReportOutput> {
        Json(self.state.coverage.clone())
    }

    #[tool(
        name = "quality_evaluation",
        description = "Return a structured code-quality audit covering architecture, dead code, hardwiring, logic concentration, overengineering suspects, and security pressure."
    )]
    async fn quality_evaluation(&self) -> Json<QualityEvaluationOutput> {
        Json(self.state.quality.clone())
    }

    #[tool(
        name = "convergence_report",
        description = "Return graph-connected diff state across runs, including new/worsened/resolved findings, contract deltas, and current attention items."
    )]
    async fn convergence_report(&self) -> Json<ConvergenceOutput> {
        Json(self.state.convergence.clone())
    }

    #[tool(
        name = "guard_decision",
        description = "Return the current doctrine-aware allow/warn/block guard decision derived from diff-local convergence pressure."
    )]
    async fn guard_decision(&self) -> Json<GuardDecisionOutput> {
        Json(self.state.guard_decision.clone())
    }

    #[tool(
        name = "list_graph_packets",
        description = "Return bounded doctrine-aware graph packets filtered by packet id or file path."
    )]
    async fn list_graph_packets(
        &self,
        Parameters(params): Parameters<ListGraphPacketsParams>,
    ) -> Json<GraphPacketArtifact> {
        let max_items = params.max_items.unwrap_or(25).clamp(1, 200);
        let mut packets =
            self.state
                .graph_packets
                .packets
                .iter()
                .filter(|packet| {
                    params
                        .packet_id
                        .as_deref()
                        .is_none_or(|packet_id| packet.id == packet_id)
                        && params.file_path.as_deref().is_none_or(|file_path| {
                            packet.primary_file_path == file_path
                                || packet.primary_anchor.as_ref().is_some_and(|anchor| {
                                    anchor.file_path.display().to_string() == file_path
                                })
                                || packet
                                    .neighbors
                                    .iter()
                                    .any(|neighbor| neighbor.file_path == file_path)
                                || packet.evidence_anchors.iter().any(|anchor| {
                                    anchor.file_path.display().to_string() == file_path
                                })
                                || packet
                                    .graph_traces
                                    .iter()
                                    .flat_map(|trace| trace.hops.iter())
                                    .any(|hop| {
                                        hop.source_file_path == file_path
                                            || hop.target_file_path == file_path
                                    })
                                || packet
                                    .code_flows
                                    .iter()
                                    .flat_map(|flow| flow.steps.iter())
                                    .any(|step| step.file_path == file_path)
                                || packet.source_sink_paths.iter().any(|path| {
                                    path.source.file_path == file_path
                                        || path.sink.file_path == file_path
                                        || path
                                            .supporting_locations
                                            .iter()
                                            .any(|location| location.file_path == file_path)
                                })
                                || packet.semantic_state_flows.iter().any(|flow| {
                                    flow.writer.file_path == file_path
                                        || flow.reader.file_path == file_path
                                        || flow
                                            .supporting_locations
                                            .iter()
                                            .any(|location| location.file_path == file_path)
                                })
                        })
                })
                .take(max_items)
                .cloned()
                .collect::<Vec<_>>();
        let summary = crate::agentic::GraphPacketSummary {
            total_packets: packets.len(),
            guardian_task_packets: packets
                .iter()
                .filter(|packet| packet.kind == crate::agentic::GraphPacketKind::GuardianTask)
                .count(),
            fallback_file_packets: packets
                .iter()
                .filter(|packet| packet.kind == crate::agentic::GraphPacketKind::FocusFile)
                .count(),
            top_anchor_files: packets
                .iter()
                .map(|packet| packet.primary_file_path.clone())
                .take(8)
                .collect(),
        };
        Json(GraphPacketArtifact {
            root: self.state.graph_packets.root.clone(),
            contract_version: self.state.graph_packets.contract_version.clone(),
            summary,
            packets: std::mem::take(&mut packets),
        })
    }

    #[tool(
        name = "graph_neighbors",
        description = "Return bounded graph neighbors for one file path from the current semantic graph."
    )]
    async fn graph_neighbors(
        &self,
        Parameters(params): Parameters<GraphNeighborsParams>,
    ) -> Json<GraphNeighborsOutput> {
        let max_items = params.max_items.unwrap_or(12).clamp(1, 100);
        Json(GraphNeighborsOutput {
            file_path: params.file_path.clone(),
            neighbors: graph_neighbors_for_file(
                &self.state.semantic_graph,
                &params.file_path,
                max_items,
            ),
        })
    }

    #[tool(
        name = "graph_trace",
        description = "Return bounded typed graph paths between two file paths from the current semantic graph."
    )]
    async fn graph_trace(
        &self,
        Parameters(params): Parameters<GraphTraceParams>,
    ) -> Json<GraphTraceOutput> {
        Json(GraphTraceOutput {
            start_file_path: params.start_file_path.clone(),
            goal_file_path: params.goal_file_path.clone(),
            paths: graph_trace_between_files(
                &self.state.semantic_graph,
                &params.start_file_path,
                &params.goal_file_path,
                params.max_hops.unwrap_or(5).clamp(1, 12),
                params.max_paths.unwrap_or(3).clamp(1, 12),
            ),
        })
    }

    #[tool(
        name = "repository_topology",
        description = "Return a flatter repository topology over zones, manifests, runtime entries, and cross-zone links."
    )]
    async fn repository_topology(&self) -> Json<RepositoryTopologyArtifact> {
        Json(self.state.repository_topology.clone())
    }

    #[tool(
        name = "cypher_query",
        description = "Execute Cypher against the optional AigisCode Kuzu graph index for deep code-understanding queries."
    )]
    async fn cypher_query(
        &self,
        Parameters(params): Parameters<CypherQueryParams>,
    ) -> Result<Json<CypherQueryOutput>, String> {
        let Some(kuzu_path) = self.state.kuzu_path.as_deref() else {
            return Err(String::from(
                "Kuzu graph index is not available for this MCP session.",
            ));
        };
        query_kuzu(kuzu_path, &params.query)
            .map(CypherQueryOutput::from_result)
            .map(Json)
            .map_err(|error| error.to_string())
    }

    #[prompt(
        name = "triage_repo",
        description = "Guide an agent through high-signal triage of the current repository."
    )]
    async fn triage_repo(&self) -> GetPromptResult {
        GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "Triage repository {}. Start with the overview, then inspect high-severity \
                     findings and the busiest hotspots first.",
                    self.state.root
                ),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(OVERVIEW_URI).no_annotation(),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(FINDINGS_URI).no_annotation(),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(HOTSPOTS_URI).no_annotation(),
            ),
        ])
        .with_description("Start architectural triage from the native Rust artifact family.")
    }

    #[prompt(
        name = "generate_architecture_brief",
        description = "Generate a concise architecture brief grounded in hotspots, cycles, and atlas context."
    )]
    async fn generate_architecture_brief(&self) -> GetPromptResult {
        GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "Write an architecture brief for {}. Summarize the dominant structural risks, \
                     the most coupled files, the cycle pressure, and where coverage is still partial.",
                    self.state.root
                ),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(OVERVIEW_URI).no_annotation(),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(CYCLES_URI).no_annotation(),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(ATLAS_URI).no_annotation(),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(COVERAGE_URI).no_annotation(),
            ),
        ])
        .with_description("Build an explainable architecture summary from Rust-native artifacts.")
    }

    #[prompt(
        name = "audit_code_quality",
        description = "Generate a quality audit focused on architectural flaws, misplaced logic, dead code, overengineering, and security pressure."
    )]
    async fn audit_code_quality(&self) -> GetPromptResult {
        GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "Audit code quality for {}. Focus on architectural flaws, dead code pressure, hardwiring, logic concentration, overengineering suspects, and security pressure. Use the structured quality report and then drill into supporting findings and hotspots.",
                    self.state.root
                ),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(QUALITY_URI).no_annotation(),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(FINDINGS_URI).no_annotation(),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(HOTSPOTS_URI).no_annotation(),
            ),
            PromptMessage::new_resource_link(
                PromptMessageRole::User,
                self.resource(CYCLES_URI).no_annotation(),
            ),
        ])
        .with_description("Build a structured quality audit from Rust-native findings, hotspots, and cycle classification.")
    }
}

impl AigiscodeMcpServer {
    fn resource_catalog(&self) -> Vec<rmcp::model::Resource> {
        vec![
            self.resource(OVERVIEW_URI)
                .with_description("Repository overview, artifact paths, and top findings.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(FINDINGS_URI)
                .with_description("All current findings emitted by the Rust analysis run.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(ATLAS_URI)
                .with_description("Repository atlas nodes and edges for graph visualization.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(HOTSPOTS_URI)
                .with_description("Hotspot files, bottlenecks, orphans, and runtime entries.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(COVERAGE_URI)
                .with_description("Coverage and trust surface for the current analysis run.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(GRAPH_SCHEMA_URI)
                .with_description("Kuzu graph schema and example Cypher queries for deep code understanding.")
                .with_mime_type("text/markdown")
                .no_annotation(),
            self.resource(DEPENDENCY_GRAPH_URI)
                .with_description("Normalized dependency-view graph artifact for parity checks, impact analysis, and low-noise architecture queries.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(EVIDENCE_GRAPH_URI)
                .with_description("Raw evidence-view graph artifact with call-site multiplicity, lines, confidence, and runtime/plugin metadata.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(CONTRACTS_URI)
                .with_description("Declared runtime contract inventory for routes, hooks, env keys, config keys, and symbolic literals.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(DOCTRINE_URI)
                .with_description("Machine-readable guardian doctrine registry with built-in and repo-owned clauses.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(HANDOFF_URI)
                .with_description("Agent handoff artifact with top visible findings, feedback-loop metrics, and next recommended actions.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(CONVERGENCE_URI)
                .with_description("Graph-connected history state across runs with finding deltas, contract deltas, and current attention items.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(GUARD_URI)
                .with_description("Doctrine-aware allow/warn/block decision derived from diff-local convergence pressure.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(REPOSITORY_TOPOLOGY_URI)
                .with_description("Flatter repository topology over zones, manifests, runtime entries, and cross-zone links.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(GRAPH_PACKETS_URI)
                .with_description("Bounded doctrine-aware graph packets for agent and reviewer navigation.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(QUALITY_URI)
                .with_description("Structured code-quality audit for architecture, dead code, overengineering, and security pressure.")
                .with_mime_type("application/json")
                .no_annotation(),
            self.resource(CYCLES_URI)
                .with_description("Strong and total file dependency cycles.")
                .with_mime_type("application/json")
                .no_annotation(),
        ]
    }

    fn resource(&self, uri: &str) -> RawResource {
        RawResource::new(uri, resource_name(uri)).with_title(resource_title(uri))
    }

    fn read_resource_payload(&self, uri: &str) -> Result<(String, String), McpError> {
        match uri {
            OVERVIEW_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state.repo_overview)?,
            )),
            FINDINGS_URI => Ok((
                String::from(uri),
                to_json_pretty(&ListFindingsOutput {
                    total: self.state.finding_summaries.len(),
                    findings: self.state.finding_summaries.clone(),
                })?,
            )),
            ATLAS_URI => Ok((String::from(uri), to_json_pretty(&self.state.atlas)?)),
            HOTSPOTS_URI => Ok((
                String::from(uri),
                to_json_pretty(&HotspotsOutput {
                    hotspots: self.state.hotspots.clone(),
                    bottlenecks: self.state.bottlenecks.clone(),
                    orphan_files: self.state.orphan_files.clone(),
                    boundary_truncated_files: self.state.boundary_truncated_files.clone(),
                    runtime_entry_candidates: self.state.runtime_entry_candidates.clone(),
                })?,
            )),
            COVERAGE_URI => Ok((String::from(uri), to_json_pretty(&self.state.coverage)?)),
            GRAPH_SCHEMA_URI => Ok((String::from(uri), schema_reference_markdown())),
            DEPENDENCY_GRAPH_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state.dependency_graph)?,
            )),
            EVIDENCE_GRAPH_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state.evidence_graph)?,
            )),
            CONTRACTS_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state.contract_inventory)?,
            )),
            DOCTRINE_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state.doctrine_registry)?,
            )),
            HANDOFF_URI => Ok((String::from(uri), to_json_pretty(&self.state.handoff)?)),
            CONVERGENCE_URI => Ok((String::from(uri), to_json_pretty(&self.state.convergence)?)),
            GUARD_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state.guard_decision)?,
            )),
            REPOSITORY_TOPOLOGY_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state.repository_topology)?,
            )),
            GRAPH_PACKETS_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state.graph_packets)?,
            )),
            QUALITY_URI => Ok((String::from(uri), to_json_pretty(&self.state.quality)?)),
            CYCLES_URI => Ok((
                String::from(uri),
                to_json_pretty(&CyclesOutput {
                    strong_cycles: self.state.strong_cycles.clone(),
                    total_cycles: self.state.total_cycles.clone(),
                })?,
            )),
            _ if uri.starts_with(FINDING_URI_PREFIX) => {
                let finding_id = uri.trim_start_matches(FINDING_URI_PREFIX);
                let detail = self.state.finding_details.get(finding_id).ok_or_else(|| {
                    McpError::resource_not_found(format!("unknown finding resource: {uri}"), None)
                })?;
                Ok((String::from(uri), to_json_pretty(detail)?))
            }
            _ => Err(McpError::resource_not_found(
                format!("unsupported resource uri: {uri}"),
                None,
            )),
        }
    }
}

#[derive(Debug)]
struct McpState {
    root: String,
    semantic_graph: crate::graph::SemanticGraph,
    kuzu_path: Option<PathBuf>,
    dependency_graph: DependencyGraphArtifact,
    evidence_graph: EvidenceGraphArtifact,
    contract_inventory: ContractInventoryOutput,
    doctrine_registry: DoctrineRegistryOutput,
    handoff: AgentHandoffArtifact,
    graph_packets: GraphPacketArtifact,
    repository_topology: RepositoryTopologyArtifact,
    convergence: ConvergenceOutput,
    guard_decision: GuardDecisionOutput,
    repo_overview: RepoOverviewOutput,
    finding_summaries: Vec<FindingSummaryOutput>,
    finding_details: HashMap<String, FindingDetailOutput>,
    hotspots: Vec<HotspotOutput>,
    bottlenecks: Vec<BottleneckOutput>,
    orphan_files: Vec<String>,
    boundary_truncated_files: Vec<String>,
    runtime_entry_candidates: Vec<String>,
    strong_cycles: Vec<CycleOutput>,
    total_cycles: Vec<CycleOutput>,
    atlas: AtlasOutput,
    coverage: CoverageReportOutput,
    quality: QualityEvaluationOutput,
}

impl McpState {
    fn new(
        analysis: ProjectAnalysis,
        artifact_paths: ArtifactPaths,
        kuzu_path: Option<PathBuf>,
    ) -> Result<Self, McpServerError> {
        let surface = analysis.architecture_surface();
        let root = display_path(&analysis.root);
        let review_surface = load_review_surface(&analysis)?;
        let finding_summaries = review_surface
            .findings
            .iter()
            .filter(|finding| finding.is_visible)
            .map(FindingSummaryOutput::from_review_finding)
            .collect::<Vec<_>>();
        let finding_details =
            build_finding_details(&analysis, &surface, &review_surface, FINDING_URI_PREFIX);
        let hotspots = surface
            .hotspots
            .iter()
            .cloned()
            .map(HotspotOutput::from_hotspot)
            .collect::<Vec<_>>();
        let bottlenecks = analysis
            .graph_analysis
            .bottleneck_files
            .iter()
            .map(BottleneckOutput::from_bottleneck)
            .collect::<Vec<_>>();
        let orphan_files = crate::surface::effective_orphan_files(&analysis)
            .iter()
            .map(|path| display_path(path))
            .collect::<Vec<_>>();
        let boundary_truncated_files =
            crate::surface::effective_boundary_truncated_files(&analysis)
                .iter()
                .map(|path| display_path(path))
                .collect::<Vec<_>>();
        let runtime_entry_candidates = analysis
            .graph_analysis
            .runtime_entry_candidates
            .iter()
            .map(|path| display_path(path))
            .collect::<Vec<_>>();
        let strong_cycles = analysis
            .graph_analysis
            .strong_cycle_findings
            .iter()
            .map(CycleOutput::from_cycle_finding)
            .collect::<Vec<_>>();
        let total_cycles = analysis
            .graph_analysis
            .cycle_findings
            .iter()
            .map(CycleOutput::from_cycle_finding)
            .collect::<Vec<_>>();
        let atlas = AtlasOutput::from_surface(&surface);
        let dependency_graph = build_dependency_graph_artifact(&analysis.semantic_graph);
        let evidence_graph = build_evidence_graph_artifact(&analysis.semantic_graph);
        let contract_inventory =
            ContractInventoryOutput::from_inventory(&analysis.contract_inventory);
        let doctrine_registry = DoctrineRegistryOutput::load(&analysis.root)?;
        let coverage = CoverageReportOutput::new(&root, &surface, &review_surface);
        let quality = QualityEvaluationOutput::new(&root, &analysis, &surface, &review_surface);
        let doctrine_registry_native =
            load_doctrine_registry(&analysis.root).map_err(McpServerError::Doctrine)?;
        let handoff =
            build_agent_handoff_artifact(&analysis, &review_surface, &doctrine_registry_native);
        let convergence_artifact =
            read_json_artifact::<ConvergenceHistoryArtifact>(&artifact_paths.convergence_history)
                .unwrap_or_else(|| {
                    crate::artifacts::build_convergence_history_artifact(
                        &analysis.root,
                        &analysis.semantic_graph,
                        None,
                        None,
                        None,
                        &surface,
                        &review_surface,
                        &analysis.contract_inventory,
                        &doctrine_registry_native,
                    )
                });
        let guard_decision_artifact =
            read_json_artifact::<GuardDecisionArtifact>(&artifact_paths.guard_decision)
                .unwrap_or_else(|| {
                    crate::artifacts::build_guard_decision_artifact(
                        &analysis.root,
                        &convergence_artifact,
                    )
                });
        let agentic_review = crate::agentic::build_agentic_review_artifact(
            &analysis,
            &doctrine_registry_native,
            &handoff,
            &guard_decision_artifact,
            &convergence_artifact,
        );
        let graph_packets = build_graph_packet_artifact(&agentic_review, &analysis);
        let repository_topology = crate::artifacts::build_repository_topology_artifact(
            &analysis,
            Some(&review_surface),
            Some(&handoff),
            Some(&convergence_artifact),
            Some(&graph_packets),
        );
        let convergence = ConvergenceOutput::from_artifact(&convergence_artifact);
        let guard_decision = GuardDecisionOutput::from_artifact(&guard_decision_artifact);
        let repo_overview = RepoOverviewOutput::new(
            &root,
            &surface,
            &review_surface,
            &artifact_paths,
            kuzu_path.as_deref(),
            &handoff,
            &convergence,
            &guard_decision,
            finding_summaries.iter().take(10).cloned().collect(),
        );

        Ok(Self {
            root,
            semantic_graph: analysis.semantic_graph,
            kuzu_path,
            dependency_graph,
            evidence_graph,
            contract_inventory,
            doctrine_registry,
            handoff,
            graph_packets,
            repository_topology,
            convergence,
            guard_decision,
            repo_overview,
            finding_summaries,
            finding_details,
            hotspots,
            bottlenecks,
            orphan_files,
            boundary_truncated_files,
            runtime_entry_candidates,
            strong_cycles,
            total_cycles,
            atlas,
            coverage,
            quality,
        })
    }
}

fn resource_name(uri: &str) -> &'static str {
    match uri {
        OVERVIEW_URI => "overview",
        FINDINGS_URI => "findings",
        ATLAS_URI => "atlas",
        HOTSPOTS_URI => "hotspots",
        COVERAGE_URI => "coverage",
        GRAPH_SCHEMA_URI => "graph-schema",
        DEPENDENCY_GRAPH_URI => "dependency-graph",
        EVIDENCE_GRAPH_URI => "evidence-graph",
        CONTRACTS_URI => "contracts",
        DOCTRINE_URI => "doctrine",
        HANDOFF_URI => "handoff",
        CONVERGENCE_URI => "convergence",
        GUARD_URI => "guard",
        GRAPH_PACKETS_URI => "graph-packets",
        REPOSITORY_TOPOLOGY_URI => "repository-topology",
        QUALITY_URI => "quality",
        CYCLES_URI => "cycles",
        _ => "resource",
    }
}

fn resource_title(uri: &str) -> &'static str {
    match uri {
        OVERVIEW_URI => "Repository Overview",
        FINDINGS_URI => "Findings",
        ATLAS_URI => "Repository Atlas",
        HOTSPOTS_URI => "Hotspots",
        COVERAGE_URI => "Coverage Report",
        GRAPH_SCHEMA_URI => "Graph Schema",
        DEPENDENCY_GRAPH_URI => "Dependency Graph",
        EVIDENCE_GRAPH_URI => "Evidence Graph",
        CONTRACTS_URI => "Contract Inventory",
        DOCTRINE_URI => "Doctrine Registry",
        HANDOFF_URI => "Agent Handoff",
        CONVERGENCE_URI => "Convergence Report",
        GUARD_URI => "Guard Decision",
        GRAPH_PACKETS_URI => "Graph Packets",
        REPOSITORY_TOPOLOGY_URI => "Repository Topology",
        QUALITY_URI => "Quality Evaluation",
        CYCLES_URI => "Cycle Report",
        _ => "AigisCode Resource",
    }
}

fn to_json_pretty<T: Serialize>(value: &T) -> Result<String, McpError> {
    serde_json::to_string_pretty(value).map_err(|error| {
        McpError::internal_error(format!("failed to serialize MCP payload: {error}"), None)
    })
}

fn read_json_artifact<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    let payload = fs::read(path).ok()?;
    serde_json::from_slice(&payload).ok()
}

#[cfg(test)]
mod tests {
    use super::{
        AigiscodeMcpServer, CypherQueryParams, ListFindingsParams, ShowHotspotsParams,
        CONTRACTS_URI, CONVERGENCE_URI, COVERAGE_URI, DEPENDENCY_GRAPH_URI, DOCTRINE_URI,
        EVIDENCE_GRAPH_URI, FINDINGS_URI, FINDING_URI_PREFIX, GRAPH_PACKETS_URI, GRAPH_SCHEMA_URI,
        GUARD_URI, HANDOFF_URI, HOTSPOTS_URI, OVERVIEW_URI, REPOSITORY_TOPOLOGY_URI,
    };
    use crate::agentic::{
        GraphNeighbor, GraphNeighborDirection, GraphNeighborsParams, GraphPacket,
        GraphPacketArtifact, GraphPacketKind, GraphPacketSummary, GraphTraceParams,
        ListGraphPacketsParams,
    };
    use crate::evidence::EvidenceAnchor;
    use crate::kuzu_index::is_kuzu_available;
    use rmcp::handler::server::wrapper::Parameters;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn repo_overview_and_findings_tools_expose_structured_results() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod config;
use crate::config::read_mode;

fn unused() {}

fn main() {
    let mode = read_mode();
    if mode == "draft" {
        let _ = "shared-value";
        let _ = "shared-value";
    }
    let items = vec![1, 2, 3];
    for left in &items {
        for right in &items {
            let _ = left + right;
        }
    }
    let _ = "https://api.example.com";
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/config.rs"),
            br#"pub fn read_mode() -> String {
    std::env::var("APP_MODE").unwrap_or_default()
}
"#,
        )
        .unwrap();

        let server =
            AigiscodeMcpServer::load(fixture.clone(), None, true, is_kuzu_available()).unwrap();

        let overview = server.repo_overview().await.0;
        assert_eq!(overview.root, fixture.display().to_string());
        assert!(overview.overview.dead_code_count >= 1);
        assert!(overview
            .artifact_files
            .aigiscode_report
            .ends_with("aigiscode-report.json"));
        assert!(overview
            .artifact_files
            .evidence_graph
            .ends_with("evidence-graph.json"));
        assert!(overview
            .artifact_files
            .contract_inventory
            .ends_with("contract-inventory.json"));
        assert!(overview
            .artifact_files
            .agent_handoff
            .ends_with("aigiscode-handoff.json"));
        assert!(overview
            .artifact_files
            .convergence_history
            .ends_with("convergence-history.json"));
        assert!(overview
            .artifact_files
            .guard_decision
            .ends_with("guard-decision.json"));
        assert!(overview.feedback_loop.detected_total >= overview.review_summary.visible_findings);
        assert_eq!(
            overview.contract_inventory.summary.env_keys.unique_values,
            1
        );
        assert!(overview.overview.algorithmic_complexity_hotspot_count >= 1);
        assert!(overview.overview.ast_grep_finding_count >= 1);
        assert_eq!(
            overview.overview.ast_grep_finding_count,
            overview.overview.ast_grep_algorithmic_complexity_count
                + overview.overview.ast_grep_security_dangerous_api_count
                + overview.overview.ast_grep_framework_misuse_count
        );
        assert!(overview.convergence.current_findings >= overview.review_summary.visible_findings);
        assert!(!overview.guard_decision.verdict.is_empty());
        assert_eq!(
            overview
                .guard_decision
                .pressure
                .exact_or_modeled_attention_items
                + overview.guard_decision.pressure.heuristic_attention_items,
            overview.guard_decision.pressure.attention_items
        );
        assert_eq!(
            overview
                .guard_decision
                .pressure
                .required_radius_anchor_files,
            overview.guard_decision.required_radius.anchor_files.len()
        );
        assert!(overview.guard_decision.triggers.iter().all(|trigger| {
            !trigger.level.is_empty()
                && !trigger.message.is_empty()
                && !trigger.precision.is_empty()
                && !trigger.provenance.is_empty()
        }));

        let findings = server
            .list_findings(Parameters(ListFindingsParams {
                family: None,
                severity: None,
                file_path: None,
                language: Some(String::from("rust")),
                max_items: Some(20),
            }))
            .await
            .0;
        assert!(findings.total >= 2);
        assert!(findings
            .findings
            .iter()
            .any(|finding| finding.id.starts_with("dead-code:")));

        let detail = server
            .explain_finding(Parameters(super::ExplainFindingParams {
                finding_id: findings.findings[0].id.clone(),
            }))
            .await
            .unwrap()
            .0;
        assert!(detail.resource_uri.starts_with(FINDING_URI_PREFIX));
    }

    #[tokio::test]
    async fn resources_cover_overview_findings_hotspots_and_individual_finding() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/lib.rs"),
            br#"fn orphan() {
    let _ = "shared-value";
    let _ = "shared-value";
}
"#,
        )
        .unwrap();

        let server = AigiscodeMcpServer::load(fixture, None, true, is_kuzu_available()).unwrap();

        let resources = server.resource_catalog();
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == OVERVIEW_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == FINDINGS_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == HOTSPOTS_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == GRAPH_SCHEMA_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == DEPENDENCY_GRAPH_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == EVIDENCE_GRAPH_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == CONTRACTS_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == DOCTRINE_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == HANDOFF_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == CONVERGENCE_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == GUARD_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == REPOSITORY_TOPOLOGY_URI));
        assert!(resources
            .iter()
            .any(|resource| resource.raw.uri == GRAPH_PACKETS_URI));

        let (_, overview_payload) = server.read_resource_payload(OVERVIEW_URI).unwrap();
        let overview_json: Value = serde_json::from_str(&overview_payload).unwrap();
        assert!(overview_json["overview"]["scanned_files"].as_u64().unwrap() >= 1);

        let (_, findings_payload) = server.read_resource_payload(FINDINGS_URI).unwrap();
        let findings_json: Value = serde_json::from_str(&findings_payload).unwrap();
        let finding_id = findings_json["findings"][0]["id"]
            .as_str()
            .unwrap()
            .to_owned();

        let (_, hotspot_payload) = server.read_resource_payload(HOTSPOTS_URI).unwrap();
        let hotspots_json: Value = serde_json::from_str(&hotspot_payload).unwrap();
        assert!(hotspots_json["hotspots"].is_array());

        let (_, coverage_payload) = server.read_resource_payload(COVERAGE_URI).unwrap();
        let coverage_json: Value = serde_json::from_str(&coverage_payload).unwrap();
        assert!(coverage_json["notes"].is_array());

        let (_, schema_payload) = server.read_resource_payload(GRAPH_SCHEMA_URI).unwrap();
        assert!(schema_payload.contains("CodeRelation"));

        let (_, dependency_payload) = server.read_resource_payload(DEPENDENCY_GRAPH_URI).unwrap();
        let dependency_json: Value = serde_json::from_str(&dependency_payload).unwrap();
        assert_eq!(
            dependency_json["view"],
            Value::String(String::from("dependency_view"))
        );
        assert!(dependency_json["edges"].is_array());

        let (_, evidence_payload) = server.read_resource_payload(EVIDENCE_GRAPH_URI).unwrap();
        let evidence_json: Value = serde_json::from_str(&evidence_payload).unwrap();
        assert_eq!(
            evidence_json["view"],
            Value::String(String::from("evidence_view"))
        );
        assert!(evidence_json["edges"].is_array());

        let (_, contracts_payload) = server.read_resource_payload(CONTRACTS_URI).unwrap();
        let contracts_json: Value = serde_json::from_str(&contracts_payload).unwrap();
        assert_eq!(contracts_json["summary"]["env_keys"]["unique_values"], 0);

        let (_, doctrine_payload) = server.read_resource_payload(DOCTRINE_URI).unwrap();
        let doctrine_json: Value = serde_json::from_str(&doctrine_payload).unwrap();
        assert!(doctrine_json["clauses"].is_array());
        assert_eq!(doctrine_json["version"], "2026-03");

        let (_, handoff_payload) = server.read_resource_payload(HANDOFF_URI).unwrap();
        let handoff_json: Value = serde_json::from_str(&handoff_payload).unwrap();
        assert!(handoff_json["next_steps"].is_array());
        assert!(handoff_json["guardian_packets"].is_array());
        assert!(
            handoff_json["feedback_loop"]["detected_total"]
                .as_u64()
                .unwrap()
                >= 1
        );

        let (_, convergence_payload) = server.read_resource_payload(CONVERGENCE_URI).unwrap();
        let convergence_json: Value = serde_json::from_str(&convergence_payload).unwrap();
        assert!(convergence_json["summary"].is_object());
        assert!(convergence_json["findings"].is_array());

        let (_, guard_payload) = server.read_resource_payload(GUARD_URI).unwrap();
        let guard_json: Value = serde_json::from_str(&guard_payload).unwrap();
        assert!(guard_json["verdict"].as_str().is_some());
        assert!(guard_json["pressure"].is_object());

        let (_, topology_payload) = server
            .read_resource_payload(REPOSITORY_TOPOLOGY_URI)
            .unwrap();
        let topology_json: Value = serde_json::from_str(&topology_payload).unwrap();
        assert!(topology_json["zones"].is_array());
        assert!(topology_json["summary"]["zone_count"].as_u64().unwrap() >= 1);

        let (_, graph_packets_payload) = server.read_resource_payload(GRAPH_PACKETS_URI).unwrap();
        let graph_packets_json: Value = serde_json::from_str(&graph_packets_payload).unwrap();
        assert!(graph_packets_json["packets"].is_array());

        let finding_uri = format!("{FINDING_URI_PREFIX}{finding_id}");
        let (_, finding_payload) = server.read_resource_payload(&finding_uri).unwrap();
        let finding_json: Value = serde_json::from_str(&finding_payload).unwrap();
        assert_eq!(finding_json["finding"]["id"], Value::String(finding_id));

        if is_kuzu_available() {
            let cypher = server
                .cypher_query(Parameters(CypherQueryParams {
                    query: String::from("MATCH (n:CodeNode) RETURN n.kind AS kind, count(*) AS count ORDER BY count DESC"),
                }))
                .await
                .unwrap()
                .0;
            assert!(cypher.row_count >= 1);
        }
    }

    #[tokio::test]
    async fn show_hotspots_respects_limit() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/a.rs"),
            b"mod b; fn main() { b::helper(); }\n",
        )
        .unwrap();
        fs::write(fixture.join("src/b.rs"), b"pub fn helper() {}\n").unwrap();

        let server = AigiscodeMcpServer::load(fixture, None, true, is_kuzu_available()).unwrap();
        let output = server
            .show_hotspots(Parameters(ShowHotspotsParams { max_items: Some(1) }))
            .await
            .0;

        assert_eq!(output.hotspots.len(), 1);
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-mcp-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[tokio::test]
    async fn graph_packet_tools_return_structured_neighbors_and_traces() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod service;
fn main() { service::run(); }"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/service.rs"),
            br#"pub fn run() { helper(); }
fn helper() {}"#,
        )
        .unwrap();

        let server = AigiscodeMcpServer::load(fixture, None, true, false).unwrap();

        let packets = server
            .list_graph_packets(Parameters(ListGraphPacketsParams {
                packet_id: None,
                file_path: None,
                max_items: Some(10),
            }))
            .await
            .0;
        assert!(packets.summary.total_packets >= 1);
        assert!(!packets.packets.is_empty());

        let topology = server.repository_topology().await.0;
        assert!(topology.summary.zone_count >= 1);
        assert!(!topology.zones.is_empty());

        let primary_file = packets.packets[0].primary_file_path.clone();
        let neighbors = server
            .graph_neighbors(Parameters(GraphNeighborsParams {
                file_path: primary_file.clone(),
                max_items: Some(8),
            }))
            .await
            .0;
        assert_eq!(neighbors.file_path, primary_file);
        assert!(!neighbors.neighbors.is_empty());

        let trace = server
            .graph_trace(Parameters(GraphTraceParams {
                start_file_path: String::from("src/main.rs"),
                goal_file_path: String::from("src/service.rs"),
                max_hops: Some(4),
                max_paths: Some(2),
            }))
            .await
            .0;
        assert!(!trace.paths.is_empty());
        assert_eq!(trace.paths[0].primary_file_path, "src/main.rs");
    }

    #[tokio::test]
    async fn list_graph_packets_matches_primary_anchor_file_paths() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}\n").unwrap();

        let mut server = AigiscodeMcpServer::load(fixture, None, true, false).unwrap();
        server.state.graph_packets = GraphPacketArtifact {
            root: String::from("/tmp/example"),
            contract_version: String::from("1"),
            summary: GraphPacketSummary {
                total_packets: 1,
                guardian_task_packets: 0,
                fallback_file_packets: 1,
                top_anchor_files: vec![String::from("src/primary.rs")],
            },
            packets: vec![GraphPacket {
                id: String::from("packet-1"),
                kind: GraphPacketKind::FocusFile,
                title: String::from("Packet"),
                summary: String::from("Summary"),
                primary_file_path: String::from("src/primary.rs"),
                primary_anchor: Some(EvidenceAnchor {
                    file_path: PathBuf::from("src/anchored.rs"),
                    line: Some(7),
                    label: String::from("anchored"),
                }),
                evidence_anchors: Vec::new(),
                doctrine_refs: Vec::new(),
                preferred_mechanism: None,
                obligations: Vec::new(),
                relation_histogram: Vec::new(),
                neighbors: vec![GraphNeighbor {
                    file_path: String::from("src/neighbor.rs"),
                    direction: GraphNeighborDirection::Outbound,
                    edge_count: 1,
                    aggregate_confidence_millis: 700,
                    relation_histogram: Vec::new(),
                }],
                graph_traces: Vec::new(),
                code_flows: Vec::new(),
                source_sink_paths: Vec::new(),
                semantic_state_flows: Vec::new(),
            }],
        };

        let packets = server
            .list_graph_packets(Parameters(ListGraphPacketsParams {
                packet_id: None,
                file_path: Some(String::from("src/anchored.rs")),
                max_items: Some(10),
            }))
            .await
            .0;

        assert_eq!(packets.summary.total_packets, 1);
        assert_eq!(packets.packets[0].id, "packet-1");
    }
}
