use crate::artifacts::{
    build_agent_handoff_artifact, write_project_analysis_artifacts, AgentHandoffArtifact,
    ArtifactPaths,
};
use crate::detectors::dead_code::{DeadCodeCategory, DeadCodeFinding};
use crate::detectors::hardwiring::{HardwiringCategory, HardwiringFinding};
use crate::external::ExternalFinding;
use crate::graph::analysis::{BottleneckFile, CycleClass, CycleFinding};
use crate::ingestion::pipeline::{analyze_project, ProjectAnalysis, ProjectAnalysisError};
use crate::ingestion::scan::ScanConfig;
use crate::kuzu_index::{
    build_dependency_graph_artifact, build_evidence_graph_artifact, default_kuzu_path, query_kuzu,
    schema_reference_markdown, write_semantic_graph_kuzu_artifact, DependencyGraphArtifact,
    EvidenceGraphArtifact, KuzuIndexError,
};
use crate::policy::PolicyLoadError;
use crate::review::{
    load_review_surface, PolicyStatus as ReviewPolicyStatus, ReviewFinding, ReviewFindingFamily,
    ReviewFindingSeverity, ReviewSurface,
};
use crate::surface::{ArchitectureSurface, HotspotFile};
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
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
const HANDOFF_URI: &str = "aigiscode://repo/current/handoff";
const FINDING_TEMPLATE_URI: &str = "aigiscode://repo/current/finding/{finding_id}";
const FINDING_URI_PREFIX: &str = "aigiscode://repo/current/finding/";

#[derive(Debug, Error)]
pub enum McpServerError {
    #[error(transparent)]
    Analysis(#[from] ProjectAnalysisError),
    #[error(transparent)]
    Policy(#[from] PolicyLoadError),
    #[error("failed to write AigisCode artifacts: {0}")]
    WriteArtifacts(#[source] std::io::Error),
    #[error("failed to materialize Kuzu graph artifact: {0}")]
    Kuzu(#[from] KuzuIndexError),
    #[error("failed to start MCP server: {0}")]
    Startup(#[from] rmcp::service::ServerInitializeError),
    #[error("MCP server task failed: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("failed to create Tokio runtime: {0}")]
    Runtime(#[source] std::io::Error),
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
                deterministic_findings: output_dir.join("deterministic-findings.json"),
                external_analysis: output_dir.join("external-analysis.json"),
                architecture_surface: output_dir.join("architecture-surface.json"),
                review_surface: output_dir.join("review-surface.json"),
                agent_handoff: output_dir.join("aigiscode-handoff.json"),
                aigiscode_report: output_dir.join("aigiscode-report.json"),
                output_dir,
            }
        };
        let kuzu_path = if write_artifacts || write_kuzu {
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
            self.resource(HANDOFF_URI)
                .with_description("Agent handoff artifact with top visible findings, feedback-loop metrics, and next recommended actions.")
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
            HANDOFF_URI => Ok((String::from(uri), to_json_pretty(&self.state.handoff)?)),
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
    kuzu_path: Option<PathBuf>,
    dependency_graph: DependencyGraphArtifact,
    evidence_graph: EvidenceGraphArtifact,
    handoff: AgentHandoffArtifact,
    repo_overview: RepoOverviewOutput,
    finding_summaries: Vec<FindingSummaryOutput>,
    finding_details: HashMap<String, FindingDetailOutput>,
    hotspots: Vec<HotspotOutput>,
    bottlenecks: Vec<BottleneckOutput>,
    orphan_files: Vec<String>,
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
        let finding_details = build_finding_details(&analysis, &surface, &review_surface);
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
        let orphan_files = analysis
            .graph_analysis
            .orphan_files
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
        let coverage = CoverageReportOutput::new(&root, &surface, &review_surface);
        let quality = QualityEvaluationOutput::new(&root, &analysis, &surface, &review_surface);
        let handoff = build_agent_handoff_artifact(&analysis, &review_surface);
        let repo_overview = RepoOverviewOutput::new(
            &root,
            &surface,
            &review_surface,
            &artifact_paths,
            kuzu_path.as_deref(),
            &handoff,
            finding_summaries.iter().take(10).cloned().collect(),
        );

        Ok(Self {
            root,
            kuzu_path,
            dependency_graph,
            evidence_graph,
            handoff,
            repo_overview,
            finding_summaries,
            finding_details,
            hotspots,
            bottlenecks,
            orphan_files,
            runtime_entry_candidates,
            strong_cycles,
            total_cycles,
            atlas,
            coverage,
            quality,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum FindingFamilyFilter {
    Graph,
    DeadCode,
    Hardwiring,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum FindingSeverityFilter {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ListFindingsParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    family: Option<FindingFamilyFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    severity: Option<FindingSeverityFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_items: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExplainFindingParams {
    finding_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ShowHotspotsParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_items: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ShowCyclesParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    strong_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_items: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CypherQueryParams {
    query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepoOverviewOutput {
    root: String,
    artifact_files: ArtifactFileOutput,
    overview: OverviewOutput,
    review_summary: ReviewSummaryOutput,
    feedback_loop: FeedbackLoopOutput,
    languages: Vec<LanguageCoverageOutput>,
    top_findings: Vec<FindingSummaryOutput>,
}

impl RepoOverviewOutput {
    fn new(
        root: &str,
        surface: &ArchitectureSurface,
        review_surface: &ReviewSurface,
        artifact_paths: &ArtifactPaths,
        kuzu_path: Option<&Path>,
        handoff: &AgentHandoffArtifact,
        top_findings: Vec<FindingSummaryOutput>,
    ) -> Self {
        Self {
            root: String::from(root),
            artifact_files: ArtifactFileOutput::from_paths(artifact_paths, kuzu_path),
            overview: OverviewOutput::from_surface(surface),
            review_summary: ReviewSummaryOutput::from_review_surface(review_surface),
            feedback_loop: FeedbackLoopOutput::from_handoff(handoff),
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
pub struct ReviewSummaryOutput {
    visible_findings: usize,
    accepted_by_policy: usize,
    suppressed_by_rule: usize,
    unreviewed_findings: usize,
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
    detected_total: usize,
    actionable_visible: usize,
    accepted_by_policy: usize,
    suppressed_by_rule: usize,
    ai_reviewed: usize,
    rules_generated: usize,
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
    output_dir: String,
    deterministic_analysis: String,
    semantic_graph: String,
    dependency_graph: String,
    evidence_graph: String,
    kuzu_graph: Option<String>,
    deterministic_findings: String,
    external_analysis: String,
    architecture_surface: String,
    review_surface: String,
    agent_handoff: String,
    aigiscode_report: String,
}

impl ArtifactFileOutput {
    fn from_paths(paths: &ArtifactPaths, kuzu_path: Option<&Path>) -> Self {
        Self {
            output_dir: display_path(&paths.output_dir),
            deterministic_analysis: display_path(&paths.deterministic_analysis),
            semantic_graph: display_path(&paths.semantic_graph),
            dependency_graph: display_path(&paths.dependency_graph),
            evidence_graph: display_path(&paths.evidence_graph),
            kuzu_graph: kuzu_path.map(display_path),
            deterministic_findings: display_path(&paths.deterministic_findings),
            external_analysis: display_path(&paths.external_analysis),
            architecture_surface: display_path(&paths.architecture_surface),
            review_surface: display_path(&paths.review_surface),
            agent_handoff: display_path(&paths.agent_handoff),
            aigiscode_report: display_path(&paths.aigiscode_report),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OverviewOutput {
    scanned_files: usize,
    analyzed_files: usize,
    symbols: usize,
    references: usize,
    resolved_edges: usize,
    unresolved_reference_sites: usize,
    strong_cycle_count: usize,
    total_cycle_count: usize,
    bottleneck_count: usize,
    orphan_count: usize,
    runtime_entry_count: usize,
    dead_code_count: usize,
    hardwiring_count: usize,
    external_finding_count: usize,
    external_tool_run_count: usize,
    override_edge_count: usize,
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
            external_finding_count: surface.overview.external_finding_count,
            external_tool_run_count: surface.overview.external_tool_run_count,
            override_edge_count: surface.overview.override_edge_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LanguageCoverageOutput {
    language: String,
    file_count: usize,
}

impl LanguageCoverageOutput {
    fn from_language(language: &crate::surface::LanguageCoverage) -> Self {
        Self {
            language: language.language.clone(),
            file_count: language.file_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListFindingsOutput {
    total: usize,
    findings: Vec<FindingSummaryOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindingSummaryOutput {
    id: String,
    family: String,
    severity: String,
    title: String,
    summary: String,
    file_paths: Vec<String>,
    line: Option<usize>,
    languages: Vec<String>,
    policy_status: String,
    is_visible: bool,
}

impl FindingSummaryOutput {
    fn from_review_finding(finding: &ReviewFinding) -> Self {
        Self {
            id: finding.id.clone(),
            family: review_family_label(finding.family),
            severity: review_severity_label(finding.severity),
            title: finding.title.clone(),
            summary: finding.summary.clone(),
            file_paths: finding.file_paths.clone(),
            line: finding.line,
            languages: infer_languages_from_strings(&finding.file_paths),
            policy_status: review_policy_status_label(finding.policy_status),
            is_visible: finding.is_visible,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindingDetailOutput {
    finding: FindingSummaryOutput,
    explanation: String,
    evidence_kind: String,
    related_files: Vec<String>,
    cycle_files: Vec<String>,
    hotspot: Option<HotspotOutput>,
    symbol_id: Option<String>,
    literal_value: Option<String>,
    context: Option<String>,
    resource_uri: String,
    policy_status: String,
    is_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HotspotsOutput {
    hotspots: Vec<HotspotOutput>,
    bottlenecks: Vec<BottleneckOutput>,
    orphan_files: Vec<String>,
    runtime_entry_candidates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HotspotOutput {
    file_path: String,
    language: String,
    inbound_edges: usize,
    outbound_edges: usize,
    finding_count: usize,
    bottleneck_centrality_millis: u32,
    is_orphan: bool,
    is_runtime_entry: bool,
}

impl HotspotOutput {
    fn from_hotspot(hotspot: HotspotFile) -> Self {
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
    file_path: String,
    centrality_millis: u32,
}

impl BottleneckOutput {
    fn from_bottleneck(bottleneck: &BottleneckFile) -> Self {
        Self {
            file_path: display_path(&bottleneck.file_path),
            centrality_millis: bottleneck.centrality_millis,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CyclesOutput {
    strong_cycles: Vec<CycleOutput>,
    total_cycles: Vec<CycleOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CycleOutput {
    size: usize,
    files: Vec<String>,
    cycle_class: String,
    layers: Vec<String>,
    dominant_relations: Vec<String>,
    edge_count: usize,
}

impl CycleOutput {
    fn from_cycle_finding(cycle: &CycleFinding) -> Self {
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
    root: String,
    summary: String,
    dimensions: Vec<QualityDimensionOutput>,
    suspects: Vec<QualitySuspectOutput>,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CypherQueryOutput {
    columns: Vec<String>,
    rows: Vec<serde_json::Map<String, serde_json::Value>>,
    row_count: usize,
}

impl CypherQueryOutput {
    fn from_result(result: crate::kuzu_index::KuzuQueryOutput) -> Self {
        Self {
            columns: result.columns,
            rows: result.rows,
            row_count: result.row_count,
        }
    }
}

impl QualityEvaluationOutput {
    fn new(
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
    key: String,
    label: String,
    severity: String,
    count: usize,
    summary: String,
    supporting_files: Vec<String>,
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
    category: String,
    file_path: String,
    reason: String,
    evidence_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AtlasOutput {
    nodes: Vec<AtlasNodeOutput>,
    edges: Vec<AtlasEdgeOutput>,
}

impl AtlasOutput {
    fn from_surface(surface: &ArchitectureSurface) -> Self {
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
    file_path: String,
    language: String,
    inbound_edges: usize,
    outbound_edges: usize,
    finding_count: usize,
    bottleneck_centrality_millis: u32,
    is_orphan: bool,
    is_runtime_entry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AtlasEdgeOutput {
    source_file_path: String,
    target_file_path: String,
    edge_count: usize,
    kinds: Vec<String>,
    strongest_resolution_tier: String,
    average_confidence_millis: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoverageReportOutput {
    root: String,
    scanned_files: usize,
    analyzed_files: usize,
    unresolved_reference_sites: usize,
    supported_languages: Vec<LanguageCoverageOutput>,
    visible_findings: usize,
    accepted_by_policy: usize,
    suppressed_by_rule: usize,
    external_findings: usize,
    external_tool_runs: usize,
    notes: Vec<String>,
}

impl CoverageReportOutput {
    fn new(root: &str, surface: &ArchitectureSurface, review_surface: &ReviewSurface) -> Self {
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

fn build_finding_details(
    analysis: &ProjectAnalysis,
    surface: &ArchitectureSurface,
    review_surface: &ReviewSurface,
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
                    resource_uri: format!("{FINDING_URI_PREFIX}{id}"),
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
                    resource_uri: format!("{FINDING_URI_PREFIX}{id}"),
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
                    resource_uri: format!("{FINDING_URI_PREFIX}{id}"),
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
                    resource_uri: format!("{FINDING_URI_PREFIX}{id}"),
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
                    resource_uri: format!("{FINDING_URI_PREFIX}{id}"),
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
                    resource_uri: format!("{FINDING_URI_PREFIX}{id}"),
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
        HardwiringCategory::EnvOutsideConfig => String::from(
            "Environment access appears outside a config-like file path.",
        ),
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

fn review_family_label(family: ReviewFindingFamily) -> String {
    match family {
        ReviewFindingFamily::Graph => String::from("graph"),
        ReviewFindingFamily::DeadCode => String::from("dead_code"),
        ReviewFindingFamily::Hardwiring => String::from("hardwiring"),
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

fn family_matches(family: &str, filter: Option<FindingFamilyFilter>) -> bool {
    match filter {
        None => true,
        Some(FindingFamilyFilter::Graph) => family == "graph",
        Some(FindingFamilyFilter::DeadCode) => family == "dead_code",
        Some(FindingFamilyFilter::Hardwiring) => family == "hardwiring",
    }
}

fn severity_matches(severity: &str, filter: Option<FindingSeverityFilter>) -> bool {
    match filter {
        None => true,
        Some(FindingSeverityFilter::High) => severity == "high",
        Some(FindingSeverityFilter::Medium) => severity == "medium",
        Some(FindingSeverityFilter::Low) => severity == "low",
    }
}

fn path_matches(finding: &FindingSummaryOutput, file_path: Option<&str>) -> bool {
    file_path.is_none_or(|file_path| {
        finding
            .file_paths
            .iter()
            .any(|path| path.contains(file_path))
    })
}

fn language_matches(finding: &FindingSummaryOutput, language: Option<&str>) -> bool {
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

fn graph_layer_label(layer: &crate::graph::GraphLayer) -> &'static str {
    match layer {
        crate::graph::GraphLayer::Structural => "structural",
        crate::graph::GraphLayer::Runtime => "runtime",
        crate::graph::GraphLayer::Framework => "framework",
        crate::graph::GraphLayer::PolicyOverlay => "policy_overlay",
    }
}

fn relation_kind_label(kind: &crate::graph::RelationKind) -> &'static str {
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

fn display_path(path: &Path) -> String {
    path.display().to_string()
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
        HANDOFF_URI => "handoff",
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
        HANDOFF_URI => "Agent Handoff",
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

#[cfg(test)]
mod tests {
    use super::{
        AigiscodeMcpServer, CypherQueryParams, ListFindingsParams, ShowHotspotsParams,
        COVERAGE_URI, DEPENDENCY_GRAPH_URI, EVIDENCE_GRAPH_URI, FINDINGS_URI, FINDING_URI_PREFIX,
        GRAPH_SCHEMA_URI, HANDOFF_URI, HOTSPOTS_URI, OVERVIEW_URI,
    };
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

        let server = AigiscodeMcpServer::load(fixture.clone(), None, true, true).unwrap();

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
            .agent_handoff
            .ends_with("aigiscode-handoff.json"));
        assert!(overview.feedback_loop.detected_total >= overview.review_summary.visible_findings);

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

        let server = AigiscodeMcpServer::load(fixture, None, true, true).unwrap();

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
            .any(|resource| resource.raw.uri == HANDOFF_URI));

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

        let (_, handoff_payload) = server.read_resource_payload(HANDOFF_URI).unwrap();
        let handoff_json: Value = serde_json::from_str(&handoff_payload).unwrap();
        assert!(handoff_json["next_steps"].is_array());
        assert!(
            handoff_json["feedback_loop"]["detected_total"]
                .as_u64()
                .unwrap()
                >= 1
        );

        let finding_uri = format!("{FINDING_URI_PREFIX}{finding_id}");
        let (_, finding_payload) = server.read_resource_payload(&finding_uri).unwrap();
        let finding_json: Value = serde_json::from_str(&finding_payload).unwrap();
        assert_eq!(finding_json["finding"]["id"], Value::String(finding_id));

        let cypher = server
            .cypher_query(Parameters(CypherQueryParams {
                query: String::from("MATCH (n:CodeNode) RETURN n.kind AS kind, count(*) AS count ORDER BY count DESC"),
            }))
            .await
            .unwrap()
            .0;
        assert!(cypher.row_count >= 1);
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

        let server = AigiscodeMcpServer::load(fixture, None, true, true).unwrap();
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
}
