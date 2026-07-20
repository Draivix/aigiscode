mod contracts;
mod live;
mod watch;

use self::contracts::{
    build_finding_details, confidence_matches, display_path, family_matches, is_corpus_scale_cycle,
    language_matches, path_matches, phase_matches, precision_matches, severity_matches,
    AtlasOutput, BottleneckOutput, BriefHotspotOutput, ConsistencyMode, ContainerDesignOutput,
    ContractInventoryOutput, ConventionClauseOutput, ConventionExemplarOutput, ConventionForOutput,
    ConventionForParams, ConvergenceFindingOutput, ConvergenceOutput, CorpusScaleUnitOutput,
    CoverageReportOutput, CrossLayerConsumerOutput, CycleOutput, CyclesOutput, CypherQueryOutput,
    CypherQueryParams, DoctrineClauseOutput, DoctrineRegistryOutput, ExplainFindingParams,
    FindSymbolOutput, FindSymbolParams, FindingBriefOutput, FindingDetailOutput,
    FindingSummaryOutput, GuardDecisionOutput, HotspotOutput, HotspotsOutput, ImpactRadiusOutput,
    ImpactRadiusParams, ListFindingsOutput, ListFindingsParams, ModuleDesignOutput,
    ModuleDesignParams, ModuleEdgeOutput, PrepareChangeOutput, PrepareChangeParams,
    QualityEvaluationOutput, RepoBriefOutput, RepoOverviewOutput, RepoOverviewParams,
    ReviewRadiusFileOutput, ShowCyclesParams, ShowHotspotsParams, SuppressFindingOutput,
    SuppressFindingParams, SymbolMatchOutput, SymbolUsagesOutput, SymbolUsagesParams,
    UsageSiteOutput, VerifyChangeOutput, VerifyChangeParams,
};
use self::live::LiveState;
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
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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
    watch: bool,
) -> Result<(), McpServerError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(McpServerError::Runtime)?;
    runtime.block_on(async move {
        // Answer `initialize` immediately; build the initial index in the
        // background and publish it through the same live handle the watcher
        // uses. A large repository must not time out the MCP handshake —
        // snapshot-reading tools wait on the first publish instead, and the
        // freshness contract reports the pending index honestly.
        let server = AigiscodeMcpServer::new_pending();
        let live = Arc::clone(&server.live);
        let watch_root = root.clone();
        tokio::spawn(async move {
            let target = live.begin_rebuild().max(1);
            let build_root = root.clone();
            let output_dir = output_dir.clone();
            let built = tokio::task::spawn_blocking(move || {
                build_mcp_state(
                    &build_root,
                    output_dir.as_deref(),
                    write_artifacts,
                    write_kuzu,
                )
            })
            .await;
            match built {
                Ok(Ok(state)) => {
                    live.publish(Some(state), target);
                    eprintln!("aigiscode mcp: initial index published (revision {target})");
                    if watch {
                        watch::spawn_watch(live, watch_root);
                    }
                }
                Ok(Err(error)) => {
                    let message = format!("initial analysis failed: {error}");
                    eprintln!("aigiscode mcp: {message}");
                    live.record_error(message);
                }
                Err(join_error) => {
                    let message = format!("initial analysis task panicked: {join_error}");
                    eprintln!("aigiscode mcp: {message}");
                    live.record_error(message);
                }
            }
        });
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
        let (uri, payload) = self.read_resource_payload(&request.uri).await?;
        Ok(ReadResourceResult::new(vec![ResourceContents::text(
            payload, uri,
        )
        .with_mime_type("application/json")]))
    }
}

pub struct AigiscodeMcpServer {
    live: Arc<LiveState<Option<McpState>>>,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

/// Run the full batch pipeline once and assemble an [`McpState`] snapshot. Shared by the
/// initial server load and the watcher's per-change rebuilds so the two never drift.
fn build_mcp_state(
    root: &Path,
    output_dir: Option<&Path>,
    write_artifacts: bool,
    write_kuzu: bool,
) -> Result<McpState, McpServerError> {
    let analysis = analyze_project(root.to_path_buf(), &ScanConfig::default())?;
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
    McpState::new(analysis, artifact_paths, kuzu_path)
}

impl AigiscodeMcpServer {
    pub fn load(
        root: PathBuf,
        output_dir: Option<&Path>,
        write_artifacts: bool,
        write_kuzu: bool,
    ) -> Result<Self, McpServerError> {
        let state = build_mcp_state(&root, output_dir, write_artifacts, write_kuzu)?;
        Ok(Self::from_state(state))
    }

    fn from_state(state: McpState) -> Self {
        Self {
            live: LiveState::new(Some(state)),
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    /// A server whose initial index has not been built yet: `initialize`
    /// answers immediately, every snapshot-reading tool waits (via `state()`)
    /// until the background initial build publishes revision 1, and the
    /// freshness contract reports the pending state honestly.
    fn new_pending() -> Self {
        Self {
            live: LiveState::new_at(None, 0, true),
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    /// Latest published snapshot + the revision it represents. Awaits the
    /// initial index when the server started pending (bounded; the initial
    /// build failing leaves the daemon degraded and loudly logged rather than
    /// pretending an empty repository).
    async fn state(&self) -> ReadyState {
        // Fast path: already published.
        if self.live.load().snapshot.is_some() {
            return ReadyState(self.live.load());
        }
        // Initial index still building: wait generously, then re-check.
        let _ = self.live.wait_for_revision(1, 15 * 60 * 1000).await;
        ReadyState(self.live.load())
    }

    /// Test-only: mutate the published snapshot in place (clone → mutate → republish),
    /// so tests can inject synthetic state without a full analysis.
    #[cfg(test)]
    fn mutate_state_for_test(&self, mutate: impl FnOnce(&mut McpState)) {
        let current = self.live.load();
        let mut state = current
            .snapshot
            .clone()
            .expect("test server always starts from a built state");
        mutate(&mut state);
        self.live.publish(Some(state), current.revision);
    }
}

/// A published snapshot known to contain a built index.
struct ReadyState(Arc<live::Published<Option<McpState>>>);

impl ReadyState {
    fn snapshot(&self) -> &McpState {
        self.0
            .snapshot
            .as_ref()
            .expect("state() only returns after the initial index is published")
    }

    #[allow(dead_code)]
    fn revision(&self) -> u64 {
        self.0.revision
    }
}

#[tool_router]
#[prompt_router]
impl AigiscodeMcpServer {
    #[tool(
        name = "repo_overview",
        description = "Return repository architecture overview, top findings, artifact locations, \
                       and (under `mcp --watch`) a freshness contract. Accepts optional \
                       min_revision/consistency/wait_ms to wait for the graph to catch up."
    )]
    async fn repo_overview(
        &self,
        Parameters(params): Parameters<RepoOverviewParams>,
    ) -> Json<RepoOverviewOutput> {
        let observed_at_start = self.live.observed();
        let target = match params.consistency {
            ConsistencyMode::WaitUntilIndexed => params.min_revision.unwrap_or(observed_at_start),
            _ => params.min_revision.unwrap_or(0),
        };
        let satisfied = if matches!(params.consistency, ConsistencyMode::WaitUntilIndexed) {
            self.live
                .wait_for_revision(target, params.wait_ms.unwrap_or(0))
                .await
        } else {
            // latest_available / allow_stale: satisfied unless an unmet min_revision was set.
            self.live.load().revision >= target
        };
        // Wait for the initial index if it is still building, then read.
        let state = self.state().await;
        let mut overview = state.snapshot().repo_overview.clone();
        overview.freshness = Some(self.live.freshness(satisfied));
        Json(overview)
    }

    #[tool(
        name = "repo_brief",
        description = "Budgeted orientation brief (target <=3 KB): what this repository is, its \
                       language mix, runtime entries, top hotspots, high-severity pressures, the \
                       current guard verdict, and the doctrine headline. Start here; use \
                       repo_overview only when the full artifact dump is genuinely needed."
    )]
    async fn repo_brief(&self) -> Json<RepoBriefOutput> {
        let state = self.state().await;
        let snapshot = state.snapshot();
        let overview = &snapshot.repo_overview.overview;
        let quality = &snapshot.quality;
        let guard = &snapshot.guard_decision;

        let language_mix = snapshot
            .repo_overview
            .languages
            .iter()
            .filter(|language| language.language != "Unsupported" && language.language != "Data")
            .map(|language| format!("{} {} files", language.language, language.file_count))
            .collect::<Vec<_>>()
            .join(", ");
        let headline = format!(
            "{} analyzed files ({}); {} symbols, {} resolved edges; {} visible findings; guard: {}.",
            overview.analyzed_files,
            language_mix,
            overview.symbols,
            overview.resolved_edges,
            snapshot.finding_summaries.len(),
            guard.verdict,
        );

        let top_pressures = quality
            .dimensions
            .iter()
            .filter(|dimension| dimension.severity == "high")
            .take(3)
            .map(|dimension| format!("{}: {}", dimension.label, dimension.summary))
            .collect::<Vec<_>>();

        let top_hotspots = snapshot
            .hotspots
            .iter()
            .take(3)
            .map(|hotspot| BriefHotspotOutput {
                file_path: hotspot.file_path.clone(),
                finding_count: hotspot.finding_count,
                bottleneck_centrality_millis: hotspot.bottleneck_centrality_millis,
            })
            .collect::<Vec<_>>();

        let block_titles = snapshot
            .doctrine_registry
            .clauses
            .iter()
            .filter(|clause| clause.default_disposition.eq_ignore_ascii_case("block"))
            .map(|clause| clause.title.clone())
            .take(5)
            .collect::<Vec<_>>();
        let doctrine_headline = if block_titles.is_empty() {
            format!(
                "{} doctrine clauses; none block by default.",
                snapshot.doctrine_registry.clauses.len()
            )
        } else {
            format!(
                "{} doctrine clauses; blocking: {}.",
                snapshot.doctrine_registry.clauses.len(),
                block_titles.join("; ")
            )
        };

        Json(RepoBriefOutput {
            root: snapshot.root.clone(),
            headline,
            languages: snapshot.repo_overview.languages.clone(),
            runtime_entries: diversify_by_parent_dir(&snapshot.runtime_entry_candidates, 8),
            top_hotspots,
            top_pressures,
            guard_verdict: guard.verdict.clone(),
            guard_summary: guard.summary.clone(),
            doctrine_headline,
            freshness: Some(self.live.freshness(true)),
        })
    }

    #[tool(
        name = "module_design",
        description = "The architect's read of a module: every class/interface with its public \
                       method signatures (no bodies), plus which other modules it depends on and \
                       which depend on it, heaviest first. Use this to judge design before \
                       reading any implementation."
    )]
    async fn module_design(
        &self,
        Parameters(params): Parameters<ModuleDesignParams>,
    ) -> Json<ModuleDesignOutput> {
        let max_containers = params.max_containers.unwrap_or(30).clamp(1, 100);
        let prefix = params.path.trim().trim_end_matches('/').to_string();
        let state = self.state().await;
        let graph = &state.snapshot().semantic_graph;

        let in_module = |path: &Path| {
            let display = display_path(path);
            display == prefix || display.starts_with(&format!("{prefix}/"))
        };
        let member_files = graph
            .files
            .iter()
            .filter(|file| in_module(&file.path))
            .map(|file| file.path.clone())
            .collect::<HashSet<_>>();

        // Container shells: class-like symbols in member files.
        let container_like = |kind: crate::graph::SymbolKind| {
            matches!(
                kind,
                crate::graph::SymbolKind::Class
                    | crate::graph::SymbolKind::Interface
                    | crate::graph::SymbolKind::Struct
                    | crate::graph::SymbolKind::Enum
                    | crate::graph::SymbolKind::Trait
            )
        };
        let mut containers: HashMap<
            &str,
            (&crate::graph::SymbolNode, Vec<&crate::graph::SymbolNode>),
        > = HashMap::new();
        for symbol in &graph.symbols {
            if container_like(symbol.kind) && member_files.contains(&symbol.file_path) {
                containers
                    .entry(symbol.id.as_str())
                    .or_insert((symbol, Vec::new()));
            }
        }
        for symbol in &graph.symbols {
            if !member_files.contains(&symbol.file_path) {
                continue;
            }
            if let Some(parent) = symbol.parent_symbol_id.as_deref() {
                if let Some(entry) = containers.get_mut(parent) {
                    entry.1.push(symbol);
                }
            }
        }

        // Cross-module edge aggregation + per-container external dependents.
        let mut outbound: HashMap<String, (usize, HashSet<&Path>)> = HashMap::new();
        let mut inbound: HashMap<String, (usize, HashSet<&Path>)> = HashMap::new();
        let mut dependents_by_container: HashMap<&str, HashSet<&Path>> = HashMap::new();
        let symbol_parent: HashMap<&str, Option<&str>> = graph
            .symbols
            .iter()
            .map(|symbol| (symbol.id.as_str(), symbol.parent_symbol_id.as_deref()))
            .collect();
        for edge in &graph.resolved_edges {
            let src_in = member_files.contains(&edge.source_file_path);
            let dst_in = member_files.contains(&edge.target_file_path);
            if src_in && !dst_in {
                let entry = outbound
                    .entry(module_group_of(&edge.target_file_path))
                    .or_default();
                entry.0 += 1;
                entry.1.insert(edge.target_file_path.as_path());
            } else if !src_in && dst_in {
                let entry = inbound
                    .entry(module_group_of(&edge.source_file_path))
                    .or_default();
                entry.0 += 1;
                entry.1.insert(edge.source_file_path.as_path());
                let target = edge.target_symbol_id.as_str();
                let container_id = if containers.contains_key(target) {
                    Some(target)
                } else {
                    symbol_parent.get(target).copied().flatten()
                };
                if let Some(container_id) = container_id {
                    dependents_by_container
                        .entry(container_id)
                        .or_default()
                        .insert(edge.source_file_path.as_path());
                }
            }
        }

        let mut rendered = containers
            .iter()
            .map(|(id, (container, methods))| {
                let public_methods = methods
                    .iter()
                    .filter(|method| method.visibility == crate::graph::Visibility::Public)
                    .collect::<Vec<_>>();
                let mut signatures = public_methods
                    .iter()
                    .map(|method| method_signature(method))
                    .collect::<Vec<_>>();
                signatures.sort();
                let public_method_count = signatures.len();
                signatures.truncate(40);
                ContainerDesignOutput {
                    name: container.name.clone(),
                    kind: symbol_kind_label(container.kind),
                    file_path: display_path(&container.file_path),
                    public_signatures: signatures,
                    method_count: methods.len(),
                    public_method_count,
                    external_dependent_files: dependents_by_container
                        .get(*id)
                        .map(|files| files.len())
                        .unwrap_or(0),
                }
            })
            .collect::<Vec<_>>();
        rendered.sort_by(|a, b| {
            b.external_dependent_files
                .cmp(&a.external_dependent_files)
                .then_with(|| b.public_method_count.cmp(&a.public_method_count))
                .then_with(|| a.name.cmp(&b.name))
        });
        let container_count = rendered.len();
        rendered.truncate(max_containers);

        let to_module_edges = |map: HashMap<String, (usize, HashSet<&Path>)>| {
            let mut edges = map
                .into_iter()
                .map(|(module, (edge_count, files))| ModuleEdgeOutput {
                    module,
                    edge_count,
                    distinct_files: files.len(),
                })
                .collect::<Vec<_>>();
            edges.sort_by(|a, b| {
                b.edge_count
                    .cmp(&a.edge_count)
                    .then_with(|| a.module.cmp(&b.module))
            });
            edges.truncate(15);
            edges
        };

        Json(ModuleDesignOutput {
            path: prefix,
            file_count: member_files.len(),
            container_count,
            truncated: container_count > max_containers,
            containers: rendered,
            outbound_modules: to_module_edges(outbound),
            inbound_modules: to_module_edges(inbound),
            freshness: self.live.actionable_freshness(true),
        })
    }

    #[tool(
        name = "find_symbol",
        description = "Locate symbol definitions by name: kind, owner, file:line, plus inbound \
                       edge/file counts so the agent can judge how load-bearing each definition \
                       is. Exact-name matches rank first, then qualified-name tails, then \
                       case-insensitive substrings."
    )]
    async fn find_symbol(
        &self,
        Parameters(params): Parameters<FindSymbolParams>,
    ) -> Json<FindSymbolOutput> {
        let max_items = params.max_items.unwrap_or(20).clamp(1, 100);
        let query = params.name.trim().to_string();
        let query_lower = query.to_ascii_lowercase();
        let kind_filter = params
            .kind
            .as_deref()
            .map(|kind| kind.trim().to_ascii_lowercase());
        let state = self.state().await;
        let graph = &state.snapshot().semantic_graph;

        let tier_for = |symbol: &crate::graph::SymbolNode| -> Option<u8> {
            if let Some(kind) = kind_filter.as_deref() {
                if symbol_kind_label(symbol.kind) != kind {
                    return None;
                }
            }
            if symbol.name == query || symbol.qualified_name == query {
                return Some(0);
            }
            if symbol.qualified_name.ends_with(&format!("::{query}")) {
                return Some(1);
            }
            if !query_lower.is_empty()
                && (symbol.name.to_ascii_lowercase().contains(&query_lower)
                    || symbol
                        .qualified_name
                        .to_ascii_lowercase()
                        .contains(&query_lower))
            {
                return Some(2);
            }
            None
        };

        let mut ranked = graph
            .symbols
            .iter()
            .filter_map(|symbol| tier_for(symbol).map(|tier| (tier, symbol)))
            .collect::<Vec<_>>();
        ranked.sort_by(|(tier_a, a), (tier_b, b)| {
            tier_a
                .cmp(tier_b)
                .then_with(|| a.file_path.cmp(&b.file_path))
                .then_with(|| a.start_line.cmp(&b.start_line))
                .then_with(|| a.id.cmp(&b.id))
        });
        let total_matches = ranked.len();
        ranked.truncate(max_items);

        let matches = build_symbol_matches(
            graph,
            &ranked.iter().map(|(_, symbol)| *symbol).collect::<Vec<_>>(),
        );

        Json(FindSymbolOutput {
            query,
            total_matches,
            truncated: total_matches > max_items,
            matches,
            freshness: self.live.actionable_freshness(true),
        })
    }

    #[tool(
        name = "symbol_usages",
        description = "Who uses this symbol: inbound resolved references grouped by caller file \
                       with line anchors, heaviest caller first. Accepts a symbol ID from \
                       find_symbol or a bare name; an ambiguous bare name returns the candidate \
                       definitions instead of guessing."
    )]
    async fn symbol_usages(
        &self,
        Parameters(params): Parameters<SymbolUsagesParams>,
    ) -> Json<SymbolUsagesOutput> {
        let max_files = params.max_files.unwrap_or(25).clamp(1, 100);
        let query = params.symbol.trim().to_string();
        let state = self.state().await;
        let graph = &state.snapshot().semantic_graph;
        let freshness = self.live.actionable_freshness(true);

        let target = if let Some(symbol) = graph.symbols.iter().find(|symbol| symbol.id == query) {
            Some(symbol)
        } else {
            let mut named = graph
                .symbols
                .iter()
                .filter(|symbol| symbol.name == query || symbol.qualified_name == query)
                .collect::<Vec<_>>();
            named.sort_by(|a, b| {
                a.file_path
                    .cmp(&b.file_path)
                    .then_with(|| a.start_line.cmp(&b.start_line))
            });
            match named.len() {
                0 => None,
                1 => Some(named[0]),
                _ => {
                    let total_candidates = named.len();
                    named.truncate(20);
                    return Json(SymbolUsagesOutput {
                        symbol_id: None,
                        query,
                        total_edges: 0,
                        distinct_files: 0,
                        truncated: total_candidates > named.len(),
                        usages: Vec::new(),
                        ambiguous_candidates: build_symbol_matches(graph, &named),
                        total_candidates,
                        freshness,
                    });
                }
            }
        };
        let Some(target) = target else {
            return Json(SymbolUsagesOutput {
                symbol_id: None,
                query,
                total_edges: 0,
                distinct_files: 0,
                truncated: false,
                usages: Vec::new(),
                ambiguous_candidates: Vec::new(),
                total_candidates: 0,
                freshness,
            });
        };

        // Group inbound edges by caller file; self-file references are still
        // real usages, so they stay in.
        let mut by_file: std::collections::BTreeMap<String, (usize, Vec<usize>, HashSet<String>)> =
            std::collections::BTreeMap::new();
        let mut total_edges = 0usize;
        for edge in &graph.resolved_edges {
            if edge.target_symbol_id != target.id {
                continue;
            }
            total_edges += 1;
            let entry = by_file
                .entry(display_path(&edge.source_file_path))
                .or_default();
            entry.0 += 1;
            entry.1.push(edge.line);
            entry.2.insert(reference_kind_label(edge.kind));
        }
        let distinct_files = by_file.len();
        let mut usages = by_file
            .into_iter()
            .map(|(file_path, (edge_count, mut lines, kinds))| {
                lines.sort_unstable();
                lines.dedup();
                lines.truncate(20);
                let mut kinds = kinds.into_iter().collect::<Vec<_>>();
                kinds.sort();
                UsageSiteOutput {
                    file_path,
                    edge_count,
                    lines,
                    kinds,
                }
            })
            .collect::<Vec<_>>();
        usages.sort_by(|a, b| {
            b.edge_count
                .cmp(&a.edge_count)
                .then_with(|| a.file_path.cmp(&b.file_path))
        });
        usages.truncate(max_files);

        Json(SymbolUsagesOutput {
            symbol_id: Some(target.id.clone()),
            query,
            total_edges,
            distinct_files,
            truncated: distinct_files > max_files,
            usages,
            ambiguous_candidates: Vec::new(),
            total_candidates: 0,
            freshness,
        })
    }

    #[tool(
        name = "list_findings",
        description = "List findings filtered by review phase (architecture|implementation), family, severity, path, language, precision (certain|modeled|heuristic), and min_confidence (millis). Returns compact briefs sized for triage; call explain_finding(id) for full evidence, anchors, and provenance. `total` reports all matches before the max_items cap."
    )]
    async fn list_findings(
        &self,
        Parameters(params): Parameters<ListFindingsParams>,
    ) -> Json<ListFindingsOutput> {
        let max_items = params.max_items.unwrap_or(100).clamp(1, 500);
        let state = self.state().await;
        let matching = state
            .snapshot()
            .finding_summaries
            .iter()
            .filter(|finding| {
                family_matches(&finding.family, params.family)
                    && phase_matches(&finding.phase, params.phase)
                    && severity_matches(&finding.severity, params.severity)
                    && path_matches(finding, params.file_path.as_deref())
                    && language_matches(finding, params.language.as_deref())
                    && precision_matches(&finding.precision, params.precision.as_deref())
                    && confidence_matches(finding.confidence_millis, params.min_confidence)
            })
            .collect::<Vec<_>>();
        let total = matching.len();
        let findings = matching
            .into_iter()
            .take(max_items)
            .map(FindingBriefOutput::from_summary)
            .collect::<Vec<_>>();
        Json(ListFindingsOutput {
            total,
            truncated: total > findings.len(),
            findings,
        })
    }

    #[tool(
        name = "impact_radius",
        description = "Blast radius before changing a file or symbol: direct and transitive dependents, dependent modules, cross-layer consumers, framework contract wiring, dynamic blind spots, and the concrete review radius."
    )]
    async fn impact_radius(
        &self,
        Parameters(params): Parameters<ImpactRadiusParams>,
    ) -> Result<Json<ImpactRadiusOutput>, String> {
        let max_depth = params.max_depth.unwrap_or(3).clamp(1, 6);
        let state = self.state().await;
        let computation = compute_impact_radius(state.snapshot(), params.target.trim(), max_depth)?;
        Ok(Json(computation.output))
    }

    #[tool(
        name = "prepare_change",
        description = "Pre-edit briefing for a file or symbol in one budgeted call: blast radius (dependents, cross-layer consumers, contract wiring, risk band), findings already flagged inside that radius, test dependents to run after the edit, and doctrine refs in play. Use before editing to know what breaks and what is already under pressure."
    )]
    async fn prepare_change(
        &self,
        Parameters(params): Parameters<PrepareChangeParams>,
    ) -> Result<Json<PrepareChangeOutput>, String> {
        const FINDINGS_CAP: usize = 20;
        const TESTS_CAP: usize = 20;
        const DOCTRINE_CAP: usize = 20;
        let max_depth = params.max_depth.unwrap_or(3).clamp(1, 6);
        let state = self.state().await;
        let snapshot = state.snapshot();
        let computation = compute_impact_radius(snapshot, params.target.trim(), max_depth)?;

        let mut scope: HashSet<&str> = computation
            .dependent_files
            .iter()
            .map(String::as_str)
            .collect();
        scope.insert(computation.output.target_file.as_str());
        let mut in_radius = snapshot
            .finding_summaries
            .iter()
            .filter(|finding| {
                finding
                    .file_paths
                    .iter()
                    .any(|path| scope.contains(path.as_str()))
            })
            .collect::<Vec<_>>();
        in_radius.sort_by(|a, b| {
            severity_rank(&a.severity)
                .cmp(&severity_rank(&b.severity))
                .then(b.confidence_millis.cmp(&a.confidence_millis))
                .then(a.id.cmp(&b.id))
        });
        let findings_total = in_radius.len();
        let findings_in_radius = in_radius
            .iter()
            .take(FINDINGS_CAP)
            .map(|finding| FindingBriefOutput::from_summary(finding))
            .collect::<Vec<_>>();

        let mut doctrine_refs = in_radius
            .iter()
            .flat_map(|finding| finding.doctrine_refs.iter().cloned())
            .collect::<Vec<_>>();
        doctrine_refs.sort();
        doctrine_refs.dedup();
        doctrine_refs.truncate(DOCTRINE_CAP);

        let mut test_dependents = computation
            .dependent_files
            .iter()
            .filter(|path| looks_like_test(path))
            .cloned()
            .collect::<Vec<_>>();
        test_dependents.sort();
        test_dependents.truncate(TESTS_CAP);

        Ok(Json(PrepareChangeOutput {
            impact: computation.output,
            findings_in_radius,
            findings_total,
            findings_truncated: findings_total > FINDINGS_CAP,
            test_dependents,
            doctrine_refs,
            freshness: self.live.actionable_freshness(true),
        }))
    }

    #[tool(
        name = "verify_change",
        description = "Post-edit check scoped to the paths you touched (or, under `mcp --watch` with empty paths, the daemon's observed dirty paths): new/worsened findings in scope versus resolved/improved ones, plus honesty notes when the analysis has not caught up with the edits. Ask about YOUR change, not the whole repo."
    )]
    async fn verify_change(
        &self,
        Parameters(params): Parameters<VerifyChangeParams>,
    ) -> Json<VerifyChangeOutput> {
        const SCOPE_CAP: usize = 50;
        let max_items = params.max_items.unwrap_or(50).clamp(1, 200);
        let state = self.state().await;
        let snapshot = state.snapshot();

        let (raw_scope, scope_source) = if params.paths.is_empty() {
            (self.live.freshness(true).dirty_paths, "daemon_dirty_paths")
        } else {
            (params.paths.clone(), "explicit")
        };
        let root_prefix = format!("{}/", snapshot.root);
        let scope = raw_scope
            .iter()
            .map(|path| {
                path.strip_prefix(&root_prefix)
                    .map(str::to_string)
                    .unwrap_or_else(|| path.clone())
            })
            .collect::<Vec<_>>();

        let in_scope = |finding: &ConvergenceFindingOutput| {
            finding.file_paths.iter().any(|file_path| {
                scope.iter().any(|scope_path| {
                    !scope_path.is_empty()
                        && (file_path.contains(scope_path) || scope_path.contains(file_path))
                })
            })
        };
        let mut regressions: Vec<ConvergenceFindingOutput> = Vec::new();
        let mut fixes: Vec<ConvergenceFindingOutput> = Vec::new();
        for finding in &snapshot.convergence.findings {
            if !in_scope(finding) {
                continue;
            }
            match finding.status.as_str() {
                "new" | "worsened" => regressions.push(finding.clone()),
                "resolved" | "improved" => fixes.push(finding.clone()),
                _ => {}
            }
        }
        regressions.sort_by(|a, b| {
            severity_rank(a.current_severity.as_deref().unwrap_or(""))
                .cmp(&severity_rank(b.current_severity.as_deref().unwrap_or("")))
                .then(a.title.cmp(&b.title))
                .then(a.fingerprint.cmp(&b.fingerprint))
        });
        fixes.sort_by(|a, b| {
            a.title
                .cmp(&b.title)
                .then(a.fingerprint.cmp(&b.fingerprint))
        });
        let regression_count = regressions.len();
        let fix_count = fixes.len();
        let truncated = regression_count > max_items || fix_count > max_items;
        regressions.truncate(max_items);
        fixes.truncate(max_items);

        let mut honesty = Vec::new();
        if scope.is_empty() {
            honesty.push(String::from(
                "scope is empty — pass the paths you edited, or run under `mcp --watch` so the daemon tracks them",
            ));
        }
        if scope_source == "daemon_dirty_paths" && self.live.freshness(true).is_stale {
            honesty.push(String::from(
                "the daemon has observed edits the current analysis does not cover yet — this delta lags your latest changes",
            ));
        }
        honesty.push(String::from(
            "convergence compares the last two analyses; changes never analyzed are invisible to this delta",
        ));

        Json(VerifyChangeOutput {
            scope_paths: scope.into_iter().take(SCOPE_CAP).collect(),
            scope_source: scope_source.to_string(),
            guard_verdict: snapshot.guard_decision.verdict.clone(),
            regressions,
            fixes,
            regression_count,
            fix_count,
            truncated,
            honesty,
            freshness: self.live.actionable_freshness(true),
        })
    }

    #[tool(
        name = "suppress_finding",
        description = "Accept a finding as a sanctioned pattern: writes one exclusion rule (finding type + file pattern + optional symbol + your reason) to .aigiscode/rules.json so future analyses hide it and convergence gets quieter. Detector and orphan findings only — architecture findings need a fix or a doctrine change. Takes effect on the next analysis; under `mcp --watch` that is the next observed source change."
    )]
    async fn suppress_finding(
        &self,
        Parameters(params): Parameters<SuppressFindingParams>,
    ) -> Result<Json<SuppressFindingOutput>, String> {
        let reason = params.reason.trim();
        if reason.is_empty() {
            return Err(String::from(
                "reason is required — rules without a reason decay into mystery policy",
            ));
        }
        let finding_id = params.finding_id.trim();
        let state = self.state().await;
        let snapshot = state.snapshot();
        let finding = snapshot
            .finding_summaries
            .iter()
            .find(|finding| finding.id == finding_id || finding.fingerprint == finding_id)
            .ok_or_else(|| format!("unknown finding: {finding_id}"))?;
        let detail = snapshot.finding_details.get(&finding.id);
        let evidence_kind = detail
            .map(|detail| detail.evidence_kind.as_str())
            .unwrap_or("");
        let (finding_type, symbol_name) = rule_spec_for_finding(finding, evidence_kind, detail)
            .ok_or_else(|| {
                format!(
                    "finding `{finding_id}` ({evidence_kind}) cannot be suppressed by rules — only detector and orphan findings can; architecture findings need a fix or a doctrine change"
                )
            })?;
        let file_pattern =
            finding.file_paths.first().cloned().ok_or_else(|| {
                format!("finding `{finding_id}` has no file path to scope a rule to")
            })?;

        let (rules_path, outcome) = crate::policy::append_exclusion_rule(
            Path::new(&snapshot.root),
            &finding_type,
            &file_pattern,
            symbol_name.as_deref(),
            reason,
        )
        .map_err(|error| error.to_string())?;

        Ok(Json(SuppressFindingOutput {
            finding_id: finding.id.clone(),
            finding_type,
            file_pattern,
            symbol_name,
            rules_path: rules_path.display().to_string(),
            outcome: match outcome {
                crate::policy::AppendRuleOutcome::Appended => String::from("appended"),
                crate::policy::AppendRuleOutcome::AlreadyPresent => String::from("already_present"),
            },
            takes_effect: String::from(
                "next analysis — under `mcp --watch`, the next observed source change triggers it; or run `aigiscode analyze`",
            ),
            honesty: vec![String::from(
                "the finding stays visible in the current snapshot until the next analysis runs",
            )],
            freshness: self.live.actionable_freshness(true),
        }))
    }

    #[tool(
        name = "convention_for",
        description = "The sanctioned way to do something in this repo: doctrine clauses matching your concern (config, persistence, http, dispatch, security, …) with preferred mechanisms and guidance, the declared layer context for the file you plan to touch, and — when the concern maps to a measurable contract — one real in-repo exemplar that does it right with zero visible findings."
    )]
    async fn convention_for(
        &self,
        Parameters(params): Parameters<ConventionForParams>,
    ) -> Json<ConventionForOutput> {
        const CLAUSE_CAP: usize = 3;
        let concern = params.concern.trim().to_string();
        let state = self.state().await;
        let snapshot = state.snapshot();

        let tokens = concern
            .to_ascii_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|token| token.len() >= 3)
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut scored = snapshot
            .doctrine_registry
            .clauses
            .iter()
            .map(|clause| (clause_score(clause, &tokens), clause))
            .filter(|(score, _)| *score > 0)
            .collect::<Vec<_>>();
        scored.sort_by(|(score_a, a), (score_b, b)| {
            score_b.cmp(score_a).then_with(|| a.id.cmp(&b.id))
        });
        let clauses = scored
            .into_iter()
            .take(CLAUSE_CAP)
            .map(|(_, clause)| ConventionClauseOutput {
                id: clause.id.clone(),
                title: clause.title.clone(),
                description: clause.description.clone(),
                default_disposition: clause.default_disposition.clone(),
                preferred_mechanism: clause.preferred_mechanism.clone(),
                guidance: clause.guidance.clone(),
            })
            .collect::<Vec<_>>();

        // Declared layer context for the target path, when a contract exists.
        let (layer, may_depend_on) = params
            .path
            .as_deref()
            .and_then(|path| {
                snapshot
                    .layers
                    .iter()
                    .filter(|layer| {
                        layer.path_prefixes.iter().any(|prefix| {
                            let path = path.trim_end_matches('/');
                            path == prefix || path.starts_with(&format!("{prefix}/"))
                        })
                    })
                    .max_by_key(|layer| {
                        layer
                            .path_prefixes
                            .iter()
                            .map(|prefix| prefix.len())
                            .max()
                            .unwrap_or(0)
                    })
            })
            .map(|layer| (Some(layer.name.clone()), layer.may_depend_on.clone()))
            .unwrap_or((None, Vec::new()));

        // Exemplar: the heaviest clean user of the contract the concern maps to.
        let exemplar_category = exemplar_contract_category(&tokens);
        let exemplar =
            exemplar_category.and_then(|category| pick_convention_exemplar(snapshot, category));

        let mut honesty = Vec::new();
        if clauses.is_empty() {
            honesty.push(String::from(
                "no doctrine clause matched — absence of a clause is not permission, ask or check the registry",
            ));
        }
        if exemplar_category.is_some() && exemplar.is_none() {
            honesty.push(String::from(
                "no clean exemplar found — every file using this contract carries visible findings",
            ));
        }
        if exemplar_category.is_none() {
            honesty.push(String::from(
                "concern does not map to a measurable contract — clauses still apply, exemplar unavailable",
            ));
        }

        Json(ConventionForOutput {
            concern,
            clauses,
            layer,
            may_depend_on,
            exemplar,
            honesty,
            freshness: self.live.actionable_freshness(true),
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
        self.state()
            .await
            .snapshot()
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
                .state()
                .await
                .snapshot()
                .hotspots
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            bottlenecks: self
                .state()
                .await
                .snapshot()
                .bottlenecks
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            orphan_files: self
                .state()
                .await
                .snapshot()
                .orphan_files
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            boundary_truncated_files: self
                .state()
                .await
                .snapshot()
                .boundary_truncated_files
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            runtime_entry_candidates: self
                .state()
                .await
                .snapshot()
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
                .state()
                .await
                .snapshot()
                .strong_cycles
                .iter()
                .take(max_items)
                .cloned()
                .collect(),
            total_cycles: if params.strong_only.unwrap_or(false) {
                Vec::new()
            } else {
                self.state()
                    .await
                    .snapshot()
                    .total_cycles
                    .iter()
                    .take(max_items)
                    .cloned()
                    .collect()
            },
            corpus_scale_units: self.state().await.snapshot().corpus_scale_units.clone(),
        })
    }

    #[tool(
        name = "coverage_report",
        description = "Return language coverage, unresolved-reference pressure, and current parity notes."
    )]
    async fn coverage_report(&self) -> Json<CoverageReportOutput> {
        Json(self.state().await.snapshot().coverage.clone())
    }

    #[tool(
        name = "quality_evaluation",
        description = "Return a structured code-quality audit covering architecture, dead code, hardwiring, logic concentration, overengineering suspects, and security pressure."
    )]
    async fn quality_evaluation(&self) -> Json<QualityEvaluationOutput> {
        Json(self.state().await.snapshot().quality.clone())
    }

    #[tool(
        name = "convergence_report",
        description = "Return graph-connected diff state across runs: summary counts, graph/contract deltas, capped moved-finding list (unchanged deltas summarized, never listed), and capped attention items. Full delta set lives in the convergence resource."
    )]
    async fn convergence_report(&self) -> Json<ConvergenceOutput> {
        Json(self.state().await.snapshot().convergence.budget_capped())
    }

    #[tool(
        name = "guard_decision",
        description = "Return the current doctrine-aware allow/warn/block guard decision derived from diff-local convergence pressure."
    )]
    async fn guard_decision(&self) -> Json<GuardDecisionOutput> {
        Json(self.state().await.snapshot().guard_decision.clone())
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
            self.state()
                .await
                .snapshot()
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
            root: self.state().await.snapshot().graph_packets.root.clone(),
            contract_version: self
                .state()
                .await
                .snapshot()
                .graph_packets
                .contract_version
                .clone(),
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
                &self.state().await.snapshot().semantic_graph,
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
                &self.state().await.snapshot().semantic_graph,
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
        Json(self.state().await.snapshot().repository_topology.clone())
    }

    #[tool(
        name = "cypher_query",
        description = "Execute Cypher against the optional AigisCode Kuzu graph index for deep code-understanding queries."
    )]
    async fn cypher_query(
        &self,
        Parameters(params): Parameters<CypherQueryParams>,
    ) -> Result<Json<CypherQueryOutput>, String> {
        let state = self.state().await;
        let Some(kuzu_path) = state.snapshot().kuzu_path.as_deref() else {
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
                    self.state().await.snapshot().root
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
                    self.state().await.snapshot().root
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
                    self.state().await.snapshot().root
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

    async fn read_resource_payload(&self, uri: &str) -> Result<(String, String), McpError> {
        match uri {
            OVERVIEW_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().repo_overview)?,
            )),
            FINDINGS_URI => {
                let state = self.state().await;
                let summaries = &state.snapshot().finding_summaries;
                Ok((
                    String::from(uri),
                    to_json_pretty(&ListFindingsOutput {
                        total: summaries.len(),
                        truncated: false,
                        findings: summaries
                            .iter()
                            .map(FindingBriefOutput::from_summary)
                            .collect(),
                    })?,
                ))
            }
            ATLAS_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().atlas)?,
            )),
            HOTSPOTS_URI => Ok((
                String::from(uri),
                to_json_pretty(&HotspotsOutput {
                    hotspots: self.state().await.snapshot().hotspots.clone(),
                    bottlenecks: self.state().await.snapshot().bottlenecks.clone(),
                    orphan_files: self.state().await.snapshot().orphan_files.clone(),
                    boundary_truncated_files: self
                        .state()
                        .await
                        .snapshot()
                        .boundary_truncated_files
                        .clone(),
                    runtime_entry_candidates: self
                        .state()
                        .await
                        .snapshot()
                        .runtime_entry_candidates
                        .clone(),
                })?,
            )),
            COVERAGE_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().coverage)?,
            )),
            GRAPH_SCHEMA_URI => Ok((String::from(uri), schema_reference_markdown())),
            DEPENDENCY_GRAPH_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().dependency_graph)?,
            )),
            EVIDENCE_GRAPH_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().evidence_graph)?,
            )),
            CONTRACTS_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().contract_inventory)?,
            )),
            DOCTRINE_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().doctrine_registry)?,
            )),
            HANDOFF_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().handoff)?,
            )),
            CONVERGENCE_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().convergence)?,
            )),
            GUARD_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().guard_decision)?,
            )),
            REPOSITORY_TOPOLOGY_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().repository_topology)?,
            )),
            GRAPH_PACKETS_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().graph_packets)?,
            )),
            QUALITY_URI => Ok((
                String::from(uri),
                to_json_pretty(&self.state().await.snapshot().quality)?,
            )),
            CYCLES_URI => Ok((
                String::from(uri),
                to_json_pretty(&CyclesOutput {
                    strong_cycles: self.state().await.snapshot().strong_cycles.clone(),
                    total_cycles: self.state().await.snapshot().total_cycles.clone(),
                    corpus_scale_units: self.state().await.snapshot().corpus_scale_units.clone(),
                })?,
            )),
            _ if uri.starts_with(FINDING_URI_PREFIX) => {
                let finding_id = uri.trim_start_matches(FINDING_URI_PREFIX);
                let state = self.state().await;
                let detail = state
                    .snapshot()
                    .finding_details
                    .get(finding_id)
                    .ok_or_else(|| {
                        McpError::resource_not_found(
                            format!("unknown finding resource: {uri}"),
                            None,
                        )
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

/// Trailing identifier of a reference target (`Foo::bar` -> `bar`,
/// `ns/mod::name` -> `name`, plain names unchanged).
fn leaf_reference_name(target: &str) -> &str {
    target
        .rsplit("::")
        .next()
        .unwrap_or(target)
        .rsplit(['.', '/'])
        .next()
        .unwrap_or(target)
}

#[derive(Debug, Clone)]
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
    corpus_scale_units: Vec<CorpusScaleUnitOutput>,
    atlas: AtlasOutput,
    coverage: CoverageReportOutput,
    quality: QualityEvaluationOutput,
    layers: Vec<crate::doctrine::LayerContract>,
}

impl McpState {
    fn new(
        analysis: ProjectAnalysis,
        artifact_paths: ArtifactPaths,
        kuzu_path: Option<PathBuf>,
    ) -> Result<Self, McpServerError> {
        let surface = analysis.architecture_surface();
        let layers = crate::doctrine::load_doctrine_registry(&analysis.root)
            .map(|registry| registry.layers)
            .unwrap_or_default();
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
        // Corpus-scale SCCs are topology, not actionable cycles: split them out
        // of both cycle lists once, here, so every consumer sees one truth.
        let analyzed_files = surface.overview.analyzed_files;
        let corpus_scale_units = analysis
            .graph_analysis
            .cycle_findings
            .iter()
            .filter(|cycle| is_corpus_scale_cycle(cycle, analyzed_files))
            .map(|cycle| CorpusScaleUnitOutput::from_cycle_finding(cycle, analyzed_files))
            .collect::<Vec<_>>();
        let strong_cycles = analysis
            .graph_analysis
            .strong_cycle_findings
            .iter()
            .filter(|cycle| !is_corpus_scale_cycle(cycle, analyzed_files))
            .map(CycleOutput::from_cycle_finding)
            .collect::<Vec<_>>();
        let total_cycles = analysis
            .graph_analysis
            .cycle_findings
            .iter()
            .filter(|cycle| !is_corpus_scale_cycle(cycle, analyzed_files))
            .map(CycleOutput::from_cycle_finding)
            .collect::<Vec<_>>();
        let atlas = AtlasOutput::from_surface(&surface);
        let dependency_graph = build_dependency_graph_artifact(&analysis.semantic_graph);
        let evidence_graph = build_evidence_graph_artifact(&analysis.semantic_graph);
        let contract_inventory =
            ContractInventoryOutput::from_inventory(&analysis.contract_inventory);
        let doctrine_registry = DoctrineRegistryOutput::load(&analysis.root)?;
        let coverage =
            CoverageReportOutput::new(&root, &surface, &review_surface, &analysis.semantic_graph);
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
            finding_summaries
                .iter()
                .take(10)
                .map(FindingBriefOutput::from_summary)
                .collect(),
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
            corpus_scale_units,
            atlas,
            coverage,
            quality,
            layers,
        })
    }
}

/// Shared blast-radius computation behind `impact_radius` and `prepare_change`.
struct ImpactComputation {
    output: ImpactRadiusOutput,
    /// Every dependent file inside `max_depth` (display paths, closest first) —
    /// not just the capped review radius. `prepare_change` scopes findings to it.
    dependent_files: Vec<String>,
}

fn compute_impact_radius(
    snapshot: &McpState,
    target: &str,
    max_depth: usize,
) -> Result<ImpactComputation, String> {
    let graph = &snapshot.semantic_graph;

    // Target resolution: exact file path first, then symbol id, then
    // unique symbol name. Ambiguity is an error, never a guess.
    let file_by_display = graph
        .files
        .iter()
        .find(|file| display_path(&file.path) == target)
        .map(|file| file.path.clone());
    let (target_file, target_symbol_id, target_symbol_label) = if let Some(path) = file_by_display {
        (path, None, None)
    } else if let Some(symbol) = graph.symbols.iter().find(|symbol| symbol.id == target) {
        (
            symbol.file_path.clone(),
            Some(symbol.id.clone()),
            Some(format!(
                "{} {}",
                symbol_kind_label(symbol.kind),
                symbol.qualified_name
            )),
        )
    } else {
        let mut matches = graph
            .symbols
            .iter()
            .filter(|symbol| symbol.name == target || symbol.qualified_name == target)
            .collect::<Vec<_>>();
        // A file's Module symbol sharing the name of the container it
        // holds is not real ambiguity — prefer the concrete symbol.
        if matches.len() > 1
            && matches
                .iter()
                .all(|symbol| symbol.file_path == matches[0].file_path)
        {
            matches.retain(|symbol| symbol.kind != crate::graph::SymbolKind::Module);
        }
        match matches.len() {
            0 => {
                return Err(format!(
                    "unknown target: {target} (no file path, symbol id, or symbol name matched)"
                ))
            }
            1 => (
                matches[0].file_path.clone(),
                Some(matches[0].id.clone()),
                Some(format!(
                    "{} {}",
                    symbol_kind_label(matches[0].kind),
                    matches[0].qualified_name
                )),
            ),
            count => {
                let mut candidates = matches
                    .iter()
                    .take(10)
                    .map(|symbol| symbol.id.clone())
                    .collect::<Vec<_>>();
                candidates.sort();
                return Err(format!(
                    "ambiguous target: {count} symbols named {target}; pass one id: {}",
                    candidates.join(", ")
                ));
            }
        }
    };

    // Depth 1: edges into the target (symbol-scoped when a symbol was
    // named — includes edges to its members). Deeper levels: file-level.
    let symbol_parent: HashMap<&str, Option<&str>> = graph
        .symbols
        .iter()
        .map(|symbol| (symbol.id.as_str(), symbol.parent_symbol_id.as_deref()))
        .collect();
    let mut direct_edge_counts: HashMap<&Path, usize> = HashMap::new();
    for edge in &graph.resolved_edges {
        if edge.source_file_path == target_file {
            continue;
        }
        let hits = match target_symbol_id.as_deref() {
            Some(symbol_id) => {
                edge.target_symbol_id == symbol_id
                    || symbol_parent
                        .get(edge.target_symbol_id.as_str())
                        .copied()
                        .flatten()
                        == Some(symbol_id)
            }
            None => edge.target_file_path == target_file,
        };
        if hits {
            *direct_edge_counts
                .entry(edge.source_file_path.as_path())
                .or_default() += 1;
        }
    }

    // Reverse file adjacency for transitive expansion.
    let mut reverse: HashMap<&Path, HashSet<&Path>> = HashMap::new();
    for edge in &graph.resolved_edges {
        if edge.source_file_path != edge.target_file_path {
            reverse
                .entry(edge.target_file_path.as_path())
                .or_default()
                .insert(edge.source_file_path.as_path());
        }
    }
    let mut depth_of: HashMap<&Path, usize> = HashMap::new();
    let mut frontier: Vec<&Path> = direct_edge_counts.keys().copied().collect();
    for path in &frontier {
        depth_of.insert(path, 1);
    }
    let mut depth = 1;
    while depth < max_depth && !frontier.is_empty() {
        depth += 1;
        let mut next = Vec::new();
        for file in frontier {
            if let Some(sources) = reverse.get(file) {
                for source in sources {
                    if *source != target_file.as_path() && !depth_of.contains_key(source) {
                        depth_of.insert(source, depth);
                        next.push(*source);
                    }
                }
            }
        }
        frontier = next;
    }

    let mut dependents_by_depth = depth_of.iter().map(|(p, d)| (*p, *d)).collect::<Vec<_>>();
    dependents_by_depth.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));
    let dependent_files = dependents_by_depth
        .iter()
        .map(|(path, _)| display_path(path))
        .collect::<Vec<_>>();

    // Dependent modules, heaviest first.
    let mut module_counts: HashMap<String, usize> = HashMap::new();
    for path in depth_of.keys() {
        *module_counts.entry(module_group_of(path)).or_default() += 1;
    }
    let mut dependent_modules = module_counts.into_iter().collect::<Vec<_>>();
    dependent_modules.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let dependent_modules = dependent_modules
        .into_iter()
        .take(20)
        .map(|(module, files)| format!("{module} ({files} files)"))
        .collect::<Vec<_>>();

    // Cross-layer consumers under the declared layer contract.
    let layer_of = |path: &Path| -> Option<&str> {
        let display = display_path(path);
        snapshot
            .layers
            .iter()
            .flat_map(|layer| {
                layer
                    .path_prefixes
                    .iter()
                    .map(move |prefix| (layer.name.as_str(), prefix))
            })
            .filter(|(_, prefix)| display == **prefix || display.starts_with(&format!("{prefix}/")))
            .max_by_key(|(_, prefix)| prefix.len())
            .map(|(name, _)| name)
    };
    let target_layer = layer_of(&target_file);
    let mut cross_layer: HashMap<&str, usize> = HashMap::new();
    if let Some(target_layer) = target_layer {
        for path in depth_of.keys() {
            if let Some(consumer_layer) = layer_of(path) {
                if consumer_layer != target_layer {
                    *cross_layer.entry(consumer_layer).or_default() += 1;
                }
            }
        }
    }
    let mut cross_layer_consumers = cross_layer
        .into_iter()
        .map(|(from_layer, files)| CrossLayerConsumerOutput {
            from_layer: from_layer.to_string(),
            files,
        })
        .collect::<Vec<_>>();
    cross_layer_consumers.sort_by(|a, b| b.files.cmp(&a.files));

    // Framework contracts declared in the target file.
    let target_display = display_path(&target_file);
    let mut framework_contract_declarations = Vec::new();
    for (label, items) in [
        ("route", &snapshot.contract_inventory.routes),
        ("hook", &snapshot.contract_inventory.hooks),
        ("channel", &snapshot.contract_inventory.channels),
        (
            "registered_key",
            &snapshot.contract_inventory.registered_keys,
        ),
    ] {
        for item in items {
            if item
                .locations
                .iter()
                .any(|location| location.file_path == target_display)
            {
                framework_contract_declarations.push(format!("{label}:{}", item.value));
            }
        }
    }
    framework_contract_declarations.sort();
    framework_contract_declarations.dedup();
    framework_contract_declarations.truncate(20);

    // Dynamic blind spots: unresolved same-repo references matching names
    // declared in the target file.
    let declared_names = graph
        .symbols
        .iter()
        .filter(|symbol| symbol.file_path == target_file && symbol.name.len() >= 4)
        .map(|symbol| symbol.name.as_str())
        .collect::<HashSet<_>>();
    let resolved_refs = graph
        .resolved_edges
        .iter()
        .filter_map(|edge| {
            edge.reference_target_name
                .as_deref()
                .map(|name| (edge.source_file_path.as_path(), edge.line, name))
        })
        .collect::<HashSet<_>>();
    let unresolved_name_matches = graph
        .references
        .iter()
        .filter(|reference| {
            reference.file_path != target_file
                && matches!(
                    reference.kind,
                    crate::graph::ReferenceKind::Call | crate::graph::ReferenceKind::Type
                )
                && declared_names.contains(leaf_reference_name(&reference.target_name))
                && !resolved_refs.contains(&(
                    reference.file_path.as_path(),
                    reference.line,
                    reference.target_name.as_str(),
                ))
        })
        .count();

    let direct = direct_edge_counts.len();
    let transitive = depth_of.len();
    let risk_band = if transitive > 100 || direct > 50 {
        "high"
    } else if transitive > 20
        || !cross_layer_consumers.is_empty()
        || !framework_contract_declarations.is_empty()
    {
        "medium"
    } else {
        "low"
    };

    let mut review_radius = direct_edge_counts
        .iter()
        .map(|(path, edges)| ReviewRadiusFileOutput {
            file_path: display_path(path),
            edge_count: *edges,
            depth: 1,
        })
        .collect::<Vec<_>>();
    review_radius.sort_by(|a, b| {
        b.edge_count
            .cmp(&a.edge_count)
            .then(a.file_path.cmp(&b.file_path))
    });
    review_radius.truncate(15);

    let mut honesty = Vec::new();
    if !framework_contract_declarations.is_empty() {
        honesty.push(String::from(
            "target declares framework contracts — consumers exist outside the code graph (routes/hooks fire at runtime)",
        ));
    }
    if unresolved_name_matches > 0 {
        honesty.push(format!(
            "{unresolved_name_matches} unresolved same-name references elsewhere may hide additional consumers (dynamic dispatch)"
        ));
    }
    if snapshot.boundary_truncated_files.contains(&target_display) {
        honesty.push(String::from(
            "analysis boundary is truncated around this file — dependents outside the analyzed slice are invisible",
        ));
    }

    Ok(ImpactComputation {
        output: ImpactRadiusOutput {
            target_file: target_display,
            target_symbol: target_symbol_label,
            direct_dependent_files: direct,
            transitive_dependent_files: transitive,
            max_depth,
            dependent_modules,
            cross_layer_consumers,
            framework_contract_declarations,
            unresolved_name_matches,
            risk_band: risk_band.to_string(),
            review_radius,
            honesty,
        },
        dependent_files,
    })
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 3,
    }
}

/// Path heuristic for "this file is a test" — used to tell an agent which
/// dependents exercise the change instead of listing every consumer.
fn looks_like_test(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.starts_with("tests/")
        || lower.contains("/tests/")
        || lower.contains("__tests__")
        || lower.ends_with("_test.rs")
        || lower.ends_with("test.php")
        || lower.contains(".test.")
        || lower.contains(".spec.")
}

/// Map a finding to the exclusion rule the policy engine can enforce for it.
/// Returns `None` for findings the rule engine cannot express (architecture
/// judgments, cycle/topology notes) so the caller refuses honestly instead of
/// writing a rule that never matches.
fn rule_spec_for_finding(
    finding: &FindingSummaryOutput,
    evidence_kind: &str,
    detail: Option<&FindingDetailOutput>,
) -> Option<(String, Option<String>)> {
    match evidence_kind {
        "unused_private_function" | "unused_import" => {
            // dead-code ids are `dead-code:{path}:{line}:{name}`.
            let name = finding
                .id
                .rsplit_once(':')
                .map(|(_, name)| name.to_string());
            Some((String::from(evidence_kind), name))
        }
        "orphan_module" | "orphan_file" => Some((String::from("orphan_file"), None)),
        "magic_string" | "repeated_literal" | "hardcoded_network" | "env_outside_config" => Some((
            String::from(evidence_kind),
            detail.and_then(|detail| detail.literal_value.clone()),
        )),
        "dangerous_command_execution" => Some((
            String::from("security_command_execution"),
            Some(finding.fingerprint.clone()),
        )),
        "dangerous_code_injection" => Some((
            String::from("security_code_injection"),
            Some(finding.fingerprint.clone()),
        )),
        "unsafe_deserialization" | "unsafe_html_output" => Some((
            String::from(evidence_kind),
            Some(finding.fingerprint.clone()),
        )),
        _ if finding.family == "external" => Some((String::from("external"), None)),
        _ => None,
    }
}

/// Score a doctrine clause against concern tokens: id parts weigh most
/// (they are the registry's vocabulary), then title, then description, plus a
/// category affinity for the concerns agents actually ask about. Anything
/// scoring zero is not about the concern and stays out.
fn clause_score(clause: &DoctrineClauseOutput, tokens: &[String]) -> usize {
    let id_parts: Vec<&str> = clause.id.split(|c: char| !c.is_alphanumeric()).collect();
    let title = clause.title.to_ascii_lowercase();
    let description = clause.description.to_ascii_lowercase();
    let mut score = 0;
    for token in tokens {
        if id_parts.iter().any(|part| part == token) {
            score += 3;
        }
        if title.contains(token.as_str()) {
            score += 2;
        }
        if description.contains(token.as_str()) {
            score += 1;
        }
    }
    let affinity = |words: &[&str]| tokens.iter().any(|token| words.contains(&token.as_str()));
    score += match clause.category.as_str() {
        "configuration" if affinity(&["config", "env", "setting", "settings"]) => 2,
        "security" if affinity(&["security", "auth", "token", "password", "crypto"]) => 2,
        "architecture"
            if affinity(&[
                "layer",
                "module",
                "structure",
                "cycle",
                "dependency",
                "architecture",
            ]) =>
        {
            2
        }
        "mechanism_choice"
            if affinity(&[
                "parser",
                "library",
                "mechanism",
                "framework",
                "persistence",
                "orm",
                "database",
                "db",
                "http",
                "client",
            ]) =>
        {
            2
        }
        "maintainability" if affinity(&["complexity", "maintainability", "quality", "clean"]) => 2,
        "change_governance" if affinity(&["change", "release", "migration", "deploy"]) => 2,
        _ => 0,
    };
    score
}

/// The contract-inventory bucket a concern maps to, when it maps to one —
/// exemplars can only be picked where usage is measurable.
fn exemplar_contract_category(tokens: &[String]) -> Option<&'static str> {
    let has = |words: &[&str]| tokens.iter().any(|token| words.contains(&token.as_str()));
    if has(&["env"]) {
        Some("env_keys")
    } else if has(&["config", "setting", "settings"]) {
        Some("config_keys")
    } else if has(&["route", "routes", "http", "api", "endpoint"]) {
        Some("routes")
    } else if has(&["hook", "hooks", "event", "events", "dispatch", "listener"]) {
        Some("hooks")
    } else {
        None
    }
}

/// One exemplar file: the heaviest user of the contract that carries zero
/// visible findings. Heaviest so it shows the *established* pattern, clean so
/// the agent does not copy pressure.
fn pick_convention_exemplar(
    snapshot: &McpState,
    category: &str,
) -> Option<ConventionExemplarOutput> {
    let items = match category {
        "config_keys" => &snapshot.contract_inventory.config_keys,
        "env_keys" => &snapshot.contract_inventory.env_keys,
        "routes" => &snapshot.contract_inventory.routes,
        "hooks" => &snapshot.contract_inventory.hooks,
        _ => return None,
    };
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for item in items {
        for location in &item.locations {
            *counts.entry(location.file_path.as_str()).or_default() += 1;
        }
    }
    let flagged: HashSet<&str> = snapshot
        .finding_summaries
        .iter()
        .flat_map(|finding| finding.file_paths.iter().map(String::as_str))
        .collect();
    let mut candidates = counts
        .into_iter()
        .filter(|(path, _)| !flagged.contains(path))
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    candidates
        .first()
        .map(|(path, count)| ConventionExemplarOutput {
            file_path: (*path).to_string(),
            basis: format!("{count} {category} accesses, zero visible findings"),
        })
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

fn symbol_kind_label(kind: crate::graph::SymbolKind) -> String {
    format!("{kind:?}").to_ascii_lowercase()
}

/// Compact body-free signature: `name(3)`, `name(1..3)`, with `-> Type` when known.
fn method_signature(method: &crate::graph::SymbolNode) -> String {
    let params = if method.required_parameter_count == method.parameter_count {
        format!("{}", method.parameter_count)
    } else {
        format!(
            "{}..{}",
            method.required_parameter_count, method.parameter_count
        )
    };
    match method.return_type_name.as_deref() {
        Some(ret) if !ret.is_empty() => format!("{}({params}) -> {ret}", method.name),
        _ => format!("{}({params})", method.name),
    }
}

/// Grouping key for "which module is this file in": up to the first three
/// directory segments (`app/Modules/Mattermost`, `app/Services/_Core`,
/// `routes`). Purely path-shaped, no framework vocabulary.
fn module_group_of(path: &Path) -> String {
    let display = display_path(path);
    let dirs = display.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
    if dirs.is_empty() {
        return String::from("<root>");
    }
    dirs.split('/').take(3).collect::<Vec<_>>().join("/")
}

fn reference_kind_label(kind: crate::graph::ReferenceKind) -> String {
    format!("{kind:?}").to_ascii_lowercase()
}

/// Build symbol-match rows with inbound edge/file counts computed in one pass
/// over the resolved edges, so a lookup never costs more than one graph scan.
fn build_symbol_matches(
    graph: &crate::graph::SemanticGraph,
    symbols: &[&crate::graph::SymbolNode],
) -> Vec<SymbolMatchOutput> {
    let wanted = symbols
        .iter()
        .map(|symbol| symbol.id.as_str())
        .collect::<HashSet<_>>();
    let mut edge_counts: HashMap<&str, usize> = HashMap::new();
    let mut file_sets: HashMap<&str, HashSet<&Path>> = HashMap::new();
    for edge in &graph.resolved_edges {
        let target = edge.target_symbol_id.as_str();
        if !wanted.contains(target) {
            continue;
        }
        *edge_counts.entry(target).or_default() += 1;
        file_sets
            .entry(target)
            .or_default()
            .insert(edge.source_file_path.as_path());
    }
    symbols
        .iter()
        .map(|symbol| SymbolMatchOutput {
            symbol_id: symbol.id.clone(),
            name: symbol.name.clone(),
            qualified_name: symbol.qualified_name.clone(),
            kind: symbol_kind_label(symbol.kind),
            file_path: display_path(&symbol.file_path),
            start_line: symbol.start_line,
            end_line: symbol.end_line,
            owner_type_name: symbol.owner_type_name.clone(),
            visibility: format!("{:?}", symbol.visibility).to_ascii_lowercase(),
            parameter_count: symbol.parameter_count,
            inbound_edges: edge_counts.get(symbol.id.as_str()).copied().unwrap_or(0),
            inbound_files: file_sets
                .get(symbol.id.as_str())
                .map(|files| files.len())
                .unwrap_or(0),
        })
        .collect()
}

/// Pick up to `cap` entries while preferring directory diversity: first one
/// entry per parent directory (in input order), then fill remaining slots in
/// input order. Keeps an orientation brief from being flooded by one
/// convention-heavy directory (e.g. a migrations folder).
fn diversify_by_parent_dir(entries: &[String], cap: usize) -> Vec<String> {
    let parent_of = |entry: &str| entry.rsplit_once('/').map(|(dir, _)| dir.to_string());
    let mut seen_dirs = HashSet::new();
    let mut picked = Vec::new();
    let mut picked_set = HashSet::new();
    for entry in entries {
        if picked.len() >= cap {
            break;
        }
        if seen_dirs.insert(parent_of(entry)) {
            picked.push(entry.clone());
            picked_set.insert(entry.clone());
        }
    }
    for entry in entries {
        if picked.len() >= cap {
            break;
        }
        if picked_set.insert(entry.clone()) {
            picked.push(entry.clone());
        }
    }
    picked
}

#[cfg(test)]
mod tests {
    use super::{
        AigiscodeMcpServer, CypherQueryParams, FindSymbolParams, ListFindingsParams,
        RepoOverviewParams, ShowHotspotsParams, SymbolUsagesParams, CONTRACTS_URI, CONVERGENCE_URI,
        COVERAGE_URI, DEPENDENCY_GRAPH_URI, DOCTRINE_URI, EVIDENCE_GRAPH_URI, FINDINGS_URI,
        FINDING_URI_PREFIX, GRAPH_PACKETS_URI, GRAPH_SCHEMA_URI, GUARD_URI, HANDOFF_URI,
        HOTSPOTS_URI, OVERVIEW_URI, REPOSITORY_TOPOLOGY_URI,
    };
    use crate::agentic::{
        AgenticPrimaryEvidenceRefs, GraphNeighbor, GraphNeighborDirection, GraphNeighborsParams,
        GraphPacket, GraphPacketArtifact, GraphPacketKind, GraphPacketSummary, GraphTraceParams,
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

        let overview = server
            .repo_overview(Parameters(RepoOverviewParams::default()))
            .await
            .0;
        assert_eq!(overview.root, fixture.display().to_string());
        // The MCP server always carries a live handle; without --watch it is simply the
        // seeded revision 1, not stale, with no dirty paths.
        let freshness = overview.freshness.expect("mcp overview carries freshness");
        assert_eq!(freshness.revision, 1);
        assert_eq!(freshness.indexed_revision, 1);
        assert_eq!(freshness.observed_revision, 1);
        assert!(!freshness.is_stale);
        assert_eq!(freshness.dirty_path_count, 0);
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
        assert_eq!(
            overview.overview.ast_grep_skipped_file_count,
            overview.overview.ast_grep_skipped_files_preview.len()
        );
        if overview.overview.ast_grep_skipped_file_count == 0 {
            assert_eq!(overview.overview.ast_grep_skipped_bytes, 0);
        } else {
            assert!(overview.overview.ast_grep_skipped_bytes > 0);
        }
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
                phase: None,
                family: None,
                severity: None,
                file_path: None,
                language: Some(String::from("rust")),
                precision: None,
                min_confidence: None,
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
    async fn list_findings_returns_budgeted_briefs_with_honest_totals() {
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

        let server =
            AigiscodeMcpServer::load(fixture.clone(), None, true, is_kuzu_available()).unwrap();

        // Honest totals: `total` counts every match before the cap, and the
        // briefs stay compact — evidence and provenance belong to explain_finding.
        let page = server
            .list_findings(Parameters(ListFindingsParams {
                max_items: Some(1),
                ..Default::default()
            }))
            .await
            .0;
        assert!(page.total >= 2, "fixture should yield multiple findings");
        assert_eq!(page.findings.len(), 1);
        assert!(page.truncated);
        let brief = serde_json::to_value(&page.findings[0]).unwrap();
        assert!(brief.get("id").and_then(Value::as_str).is_some());
        assert!(brief.get("file_paths").is_some());
        assert!(brief.get("evidence_anchors").is_none());
        assert!(brief.get("provenance").is_none());
        assert!(brief.get("fingerprint").is_none());

        // Precision and confidence filters actually filter.
        let confident = server
            .list_findings(Parameters(ListFindingsParams {
                min_confidence: Some(1001),
                ..Default::default()
            }))
            .await
            .0;
        assert_eq!(confident.total, 0);
        let heuristic_only = server
            .list_findings(Parameters(ListFindingsParams {
                precision: Some(String::from("heuristic")),
                ..Default::default()
            }))
            .await
            .0;
        assert!(heuristic_only
            .findings
            .iter()
            .all(|finding| finding.precision == "heuristic"));

        // Freshness rides along only when actionable: fresh snapshots omit it,
        // observed edits make it appear.
        let fresh = server
            .find_symbol(Parameters(FindSymbolParams {
                name: String::from("main"),
                kind: None,
                max_items: None,
            }))
            .await
            .0;
        assert!(fresh.freshness.is_none());
        server.live.mark_dirty([(
            fixture.join("src/main.rs"),
            super::live::DirtyKind::Modified,
        )]);
        let stale = server
            .find_symbol(Parameters(FindSymbolParams {
                name: String::from("main"),
                kind: None,
                max_items: None,
            }))
            .await
            .0;
        let freshness = stale
            .freshness
            .expect("stale snapshot must report freshness");
        assert!(freshness.is_stale);
    }

    #[tokio::test]
    async fn prepare_change_briefs_impact_findings_tests_and_doctrine_in_one_call() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::create_dir_all(fixture.join("tests")).unwrap();
        fs::write(
            fixture.join("src/config.rs"),
            br#"pub fn read_mode() -> String {
    std::env::var("APP_MODE").unwrap_or_default()
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod config;
use crate::config::read_mode;

fn main() {
    let mode = read_mode();
    if mode == "draft" {
        let _ = "shared-value";
        let _ = "shared-value";
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("tests/mode_check.rs"),
            br#"#[test]
fn placeholder() {
    assert_eq!(1, 1);
}
"#,
        )
        .unwrap();

        let server =
            AigiscodeMcpServer::load(fixture.clone(), None, true, is_kuzu_available()).unwrap();

        // One call answers what used to take three: blast radius + in-radius
        // findings + test dependents, all budgeted.
        let briefing = server
            .prepare_change(Parameters(super::PrepareChangeParams {
                target: String::from("src/config.rs"),
                max_depth: None,
            }))
            .await
            .unwrap()
            .0;
        assert_eq!(briefing.impact.target_file, "src/config.rs");
        assert!(
            briefing.impact.direct_dependent_files >= 1,
            "main.rs depends on config.rs"
        );
        assert!(
            briefing
                .findings_in_radius
                .iter()
                .all(|finding| !finding.id.is_empty()),
            "in-radius findings must be compact briefs with ids for explain_finding"
        );
        assert_eq!(
            briefing.findings_total,
            briefing
                .findings_in_radius
                .len()
                .max(briefing.findings_total)
        );

        // Unknown targets are errors, never guesses — same contract as impact_radius.
        let unknown = server
            .prepare_change(Parameters(super::PrepareChangeParams {
                target: String::from("no/such/file.rs"),
                max_depth: None,
            }))
            .await;
        assert!(unknown.is_err());
    }

    #[tokio::test]
    async fn verify_change_scopes_convergence_to_the_touched_paths() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn unused() {}

fn main() {
    let _ = "shared-value";
    let _ = "shared-value";
    let _ = "https://api.example.com";
}
"#,
        )
        .unwrap();

        let server =
            AigiscodeMcpServer::load(fixture.clone(), None, true, is_kuzu_available()).unwrap();

        // Explicit scope: first analysis has no baseline, so every finding in
        // scope shows up as new — the honest "you own these now" answer.
        let scoped = server
            .verify_change(Parameters(super::VerifyChangeParams {
                paths: vec![String::from("src/main.rs")],
                max_items: None,
            }))
            .await
            .0;
        assert_eq!(scoped.scope_source, "explicit");
        assert_eq!(scoped.scope_paths, vec![String::from("src/main.rs")]);
        assert!(
            scoped.regression_count >= 1,
            "first-run findings in the touched file must surface as regressions"
        );
        assert_eq!(scoped.regression_count, scoped.regressions.len());

        // Out-of-scope path: nothing should match.
        let other = server
            .verify_change(Parameters(super::VerifyChangeParams {
                paths: vec![String::from("src/elsewhere.rs")],
                max_items: None,
            }))
            .await
            .0;
        assert_eq!(other.regression_count, 0);
        assert_eq!(other.fix_count, 0);

        // Empty explicit scope falls back to daemon dirty paths; with nothing
        // observed there is nothing to verify, and it says so.
        let idle = server
            .verify_change(Parameters(super::VerifyChangeParams::default()))
            .await
            .0;
        assert_eq!(idle.scope_source, "daemon_dirty_paths");
        assert_eq!(idle.regression_count, 0);
        assert!(idle
            .honesty
            .iter()
            .any(|note| note.contains("scope is empty")));

        // Once the daemon observes an edit, the fallback scope picks it up.
        server.live.mark_dirty([(
            fixture.join("src/main.rs"),
            super::live::DirtyKind::Modified,
        )]);
        let dirty = server
            .verify_change(Parameters(super::VerifyChangeParams::default()))
            .await
            .0;
        assert_eq!(dirty.scope_paths, vec![String::from("src/main.rs")]);
        assert!(dirty.regression_count >= 1);
        let freshness = dirty
            .freshness
            .expect("dirty snapshot must report actionable freshness");
        assert!(freshness.is_stale);
    }

    #[tokio::test]
    async fn suppress_finding_writes_a_scoped_reasoned_rule() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn unused() {}

fn main() {
    let _ = "https://api.example.com";
}
"#,
        )
        .unwrap();

        let server =
            AigiscodeMcpServer::load(fixture.clone(), None, true, is_kuzu_available()).unwrap();

        let dead_code = server
            .list_findings(Parameters(ListFindingsParams {
                family: Some(super::contracts::FindingFamilyFilter::DeadCode),
                ..Default::default()
            }))
            .await
            .0;
        let finding = dead_code
            .findings
            .first()
            .expect("fixture must yield a dead-code finding")
            .clone();

        // Reason is mandatory: rules without one decay into mystery policy.
        assert!(server
            .suppress_finding(Parameters(super::SuppressFindingParams {
                finding_id: finding.id.clone(),
                reason: String::new(),
            }))
            .await
            .is_err());

        let suppressed = server
            .suppress_finding(Parameters(super::SuppressFindingParams {
                finding_id: finding.id.clone(),
                reason: String::from("kept for the CLI compatibility surface"),
            }))
            .await
            .unwrap()
            .0;
        assert_eq!(suppressed.outcome, "appended");
        assert_eq!(suppressed.finding_type, "unused_private_function");
        assert_eq!(suppressed.file_pattern, "src/main.rs");
        assert_eq!(suppressed.symbol_name.as_deref(), Some("unused"));

        // The rule landed on disk with its reason, scoped narrowly.
        let rules: Value = serde_json::from_str(
            &fs::read_to_string(fixture.join(".aigiscode/rules.json")).unwrap(),
        )
        .unwrap();
        let rules = rules["rules"].as_array().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["reason"], "kept for the CLI compatibility surface");

        // Same finding again dedups.
        let again = server
            .suppress_finding(Parameters(super::SuppressFindingParams {
                finding_id: finding.id.clone(),
                reason: String::from("duplicate"),
            }))
            .await
            .unwrap()
            .0;
        assert_eq!(again.outcome, "already_present");

        // Unknown findings are errors, never silent no-ops.
        assert!(server
            .suppress_finding(Parameters(super::SuppressFindingParams {
                finding_id: String::from("dead-code:nope.rs:1:ghost"),
                reason: String::from("ghost"),
            }))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn convention_for_returns_doctrine_clauses_and_a_clean_exemplar() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/settings.php"),
            br#"<?php
$name = config('app.name');
$zone = config('app.timezone');
"#,
        )
        .unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}\n").unwrap();

        let server =
            AigiscodeMcpServer::load(fixture.clone(), None, true, is_kuzu_available()).unwrap();

        // A config concern surfaces the configuration doctrine and points at
        // the file already doing it the sanctioned way.
        let answer = server
            .convention_for(Parameters(super::ConventionForParams {
                concern: String::from("config"),
                path: None,
            }))
            .await
            .0;
        assert!(
            answer
                .clauses
                .iter()
                .any(|clause| clause.id == "configuration.coherence"),
            "config concern must match the configuration doctrine, got: {:?}",
            answer
                .clauses
                .iter()
                .map(|clause| clause.id.as_str())
                .collect::<Vec<_>>()
        );
        let exemplar = answer
            .exemplar
            .expect("config maps to the config_keys contract");
        assert_eq!(exemplar.file_path, "src/settings.php");
        assert!(exemplar.basis.contains("config_keys"));

        // A nonsense concern is honestly empty, never a guess.
        let empty = server
            .convention_for(Parameters(super::ConventionForParams {
                concern: String::from("zzzqwx"),
                path: None,
            }))
            .await
            .0;
        assert!(empty.clauses.is_empty());
        assert!(empty
            .honesty
            .iter()
            .any(|note| note.contains("no doctrine clause matched")));
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

        let (_, overview_payload) = server.read_resource_payload(OVERVIEW_URI).await.unwrap();
        let overview_json: Value = serde_json::from_str(&overview_payload).unwrap();
        assert!(overview_json["overview"]["scanned_files"].as_u64().unwrap() >= 1);

        let (_, findings_payload) = server.read_resource_payload(FINDINGS_URI).await.unwrap();
        let findings_json: Value = serde_json::from_str(&findings_payload).unwrap();
        let finding_id = findings_json["findings"][0]["id"]
            .as_str()
            .unwrap()
            .to_owned();

        let (_, hotspot_payload) = server.read_resource_payload(HOTSPOTS_URI).await.unwrap();
        let hotspots_json: Value = serde_json::from_str(&hotspot_payload).unwrap();
        assert!(hotspots_json["hotspots"].is_array());

        let (_, coverage_payload) = server.read_resource_payload(COVERAGE_URI).await.unwrap();
        let coverage_json: Value = serde_json::from_str(&coverage_payload).unwrap();
        assert!(coverage_json["notes"].is_array());

        let (_, schema_payload) = server
            .read_resource_payload(GRAPH_SCHEMA_URI)
            .await
            .unwrap();
        assert!(schema_payload.contains("CodeRelation"));

        let (_, dependency_payload) = server
            .read_resource_payload(DEPENDENCY_GRAPH_URI)
            .await
            .unwrap();
        let dependency_json: Value = serde_json::from_str(&dependency_payload).unwrap();
        assert_eq!(
            dependency_json["view"],
            Value::String(String::from("dependency_view"))
        );
        assert!(dependency_json["edges"].is_array());

        let (_, evidence_payload) = server
            .read_resource_payload(EVIDENCE_GRAPH_URI)
            .await
            .unwrap();
        let evidence_json: Value = serde_json::from_str(&evidence_payload).unwrap();
        assert_eq!(
            evidence_json["view"],
            Value::String(String::from("evidence_view"))
        );
        assert!(evidence_json["edges"].is_array());

        let (_, contracts_payload) = server.read_resource_payload(CONTRACTS_URI).await.unwrap();
        let contracts_json: Value = serde_json::from_str(&contracts_payload).unwrap();
        assert_eq!(contracts_json["summary"]["env_keys"]["unique_values"], 0);

        let (_, doctrine_payload) = server.read_resource_payload(DOCTRINE_URI).await.unwrap();
        let doctrine_json: Value = serde_json::from_str(&doctrine_payload).unwrap();
        assert!(doctrine_json["clauses"].is_array());
        assert_eq!(doctrine_json["version"], "2026-03");

        let (_, handoff_payload) = server.read_resource_payload(HANDOFF_URI).await.unwrap();
        let handoff_json: Value = serde_json::from_str(&handoff_payload).unwrap();
        assert!(handoff_json["next_steps"].is_array());
        assert!(handoff_json["guardian_packets"].is_array());
        assert!(
            handoff_json["feedback_loop"]["detected_total"]
                .as_u64()
                .unwrap()
                >= 1
        );

        let (_, convergence_payload) = server.read_resource_payload(CONVERGENCE_URI).await.unwrap();
        let convergence_json: Value = serde_json::from_str(&convergence_payload).unwrap();
        assert!(convergence_json["summary"].is_object());
        assert!(convergence_json["findings"].is_array());

        let (_, guard_payload) = server.read_resource_payload(GUARD_URI).await.unwrap();
        let guard_json: Value = serde_json::from_str(&guard_payload).unwrap();
        assert!(guard_json["verdict"].as_str().is_some());
        assert!(guard_json["pressure"].is_object());

        let (_, topology_payload) = server
            .read_resource_payload(REPOSITORY_TOPOLOGY_URI)
            .await
            .unwrap();
        let topology_json: Value = serde_json::from_str(&topology_payload).unwrap();
        assert!(topology_json["zones"].is_array());
        assert!(topology_json["summary"]["zone_count"].as_u64().unwrap() >= 1);

        let (_, graph_packets_payload) = server
            .read_resource_payload(GRAPH_PACKETS_URI)
            .await
            .unwrap();
        let graph_packets_json: Value = serde_json::from_str(&graph_packets_payload).unwrap();
        assert!(graph_packets_json["packets"].is_array());

        let finding_uri = format!("{FINDING_URI_PREFIX}{finding_id}");
        let (_, finding_payload) = server.read_resource_payload(&finding_uri).await.unwrap();
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

    #[test]
    fn diversify_by_parent_dir_prefers_one_entry_per_directory_before_filling() {
        let entries = vec![
            "app/migrations/001.php".to_string(),
            "app/migrations/002.php".to_string(),
            "app/migrations/003.php".to_string(),
            "app/routes/api.php".to_string(),
            "app/console/kernel.php".to_string(),
        ];
        let picked = super::diversify_by_parent_dir(&entries, 4);
        assert_eq!(
            picked,
            vec![
                "app/migrations/001.php".to_string(),
                "app/routes/api.php".to_string(),
                "app/console/kernel.php".to_string(),
                "app/migrations/002.php".to_string(),
            ]
        );
        // Cap below the distinct-directory count still respects input order.
        let capped = super::diversify_by_parent_dir(&entries, 2);
        assert_eq!(
            capped,
            vec![
                "app/migrations/001.php".to_string(),
                "app/routes/api.php".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn find_symbol_and_symbol_usages_locate_definitions_and_callers() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            b"mod b;\nmod c;\nfn main() { b::helper(); c::caller(); }\n",
        )
        .unwrap();
        fs::write(fixture.join("src/b.rs"), b"pub fn helper() {}\n").unwrap();
        fs::write(
            fixture.join("src/c.rs"),
            b"use crate::b::helper;\npub fn caller() { helper(); helper(); }\n",
        )
        .unwrap();

        let server = AigiscodeMcpServer::load(fixture, None, true, is_kuzu_available()).unwrap();

        let found = server
            .find_symbol(Parameters(FindSymbolParams {
                name: "helper".to_string(),
                ..Default::default()
            }))
            .await
            .0;
        assert_eq!(found.total_matches, 1);
        let definition = &found.matches[0];
        assert_eq!(definition.kind, "function");
        assert!(definition.file_path.ends_with("src/b.rs"));
        assert!(definition.inbound_files >= 2);
        assert!(definition.inbound_edges >= definition.inbound_files);

        let usages = server
            .symbol_usages(Parameters(SymbolUsagesParams {
                symbol: definition.symbol_id.clone(),
                ..Default::default()
            }))
            .await
            .0;
        assert_eq!(
            usages.symbol_id.as_deref(),
            Some(definition.symbol_id.as_str())
        );
        assert_eq!(usages.distinct_files, definition.inbound_files);
        assert_eq!(usages.total_edges, definition.inbound_edges);
        let caller = usages
            .usages
            .iter()
            .find(|site| site.file_path.ends_with("src/c.rs"))
            .expect("c.rs is a caller");
        assert!(caller.edge_count >= 2);
        assert!(!caller.lines.is_empty());

        // Kind filter that matches nothing stays honest.
        let none = server
            .find_symbol(Parameters(FindSymbolParams {
                name: "helper".to_string(),
                kind: Some("class".to_string()),
                ..Default::default()
            }))
            .await
            .0;
        assert_eq!(none.total_matches, 0);
        assert!(none.matches.is_empty());
    }

    #[tokio::test]
    async fn ambiguous_symbol_usages_return_capped_candidates_not_guesses() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            b"mod a;\nmod b;\nfn main() { a::run(); b::run(); }\n",
        )
        .unwrap();
        fs::write(fixture.join("src/a.rs"), b"pub fn run() {}\n").unwrap();
        fs::write(fixture.join("src/b.rs"), b"pub fn run() {}\n").unwrap();

        let server = AigiscodeMcpServer::load(fixture, None, true, is_kuzu_available()).unwrap();
        let usages = server
            .symbol_usages(Parameters(SymbolUsagesParams {
                symbol: "run".to_string(),
                ..Default::default()
            }))
            .await
            .0;
        assert!(usages.symbol_id.is_none());
        assert!(usages.usages.is_empty());
        assert_eq!(usages.total_candidates, 2);
        assert_eq!(usages.ambiguous_candidates.len(), 2);
    }

    #[tokio::test]
    async fn corpus_scale_scc_reports_as_topology_not_cycle_finding() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        // A 10-file import ring covering the whole corpus: one SCC spanning
        // 100% of analyzed files must be reported as topology, not a cycle.
        for index in 0..10 {
            let next = (index + 1) % 10;
            fs::write(
                fixture.join(format!("src/m{index}.ts")),
                format!("import {{ f{next} }} from './m{next}';\nexport function f{index}() {{ f{next}(); }}\n"),
            )
            .unwrap();
        }

        let server = AigiscodeMcpServer::load(fixture, None, true, is_kuzu_available()).unwrap();
        let cycles = server
            .show_cycles(Parameters(super::ShowCyclesParams::default()))
            .await
            .0;

        assert_eq!(cycles.corpus_scale_units.len(), 1);
        let unit = &cycles.corpus_scale_units[0];
        assert_eq!(unit.size, 10);
        assert!(unit.sample_files.len() <= 10);
        assert!(unit.note.contains("topology"));
        assert!(cycles
            .strong_cycles
            .iter()
            .chain(cycles.total_cycles.iter())
            .all(|cycle| cycle.size < 10));
    }

    #[tokio::test]
    async fn module_design_renders_signatures_and_cross_module_edges() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app/Core")).unwrap();
        fs::create_dir_all(fixture.join("app/Feature")).unwrap();
        fs::write(
            fixture.join("app/Core/Registry.php"),
            br#"<?php
class Registry {
    public function all(): array { return []; }
    public function find(string $key, int $mode = 0): ?string { return null; }
    private function internal(): void {}
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Feature/Consumer.php"),
            br#"<?php
use Core\Registry;
class Consumer {
    public function run(Registry $registry): void { $registry->all(); }
}
"#,
        )
        .unwrap();

        let server = AigiscodeMcpServer::load(fixture, None, true, is_kuzu_available()).unwrap();
        let design = server
            .module_design(Parameters(super::ModuleDesignParams {
                path: String::from("app/Core"),
                max_containers: None,
            }))
            .await
            .0;

        assert_eq!(design.container_count, 1);
        let registry = &design.containers[0];
        assert_eq!(registry.name, "Registry");
        assert_eq!(registry.public_method_count, 2);
        assert_eq!(registry.method_count, 3);
        assert!(registry
            .public_signatures
            .iter()
            .any(|sig| sig == "all(0) -> array"));
        assert!(registry
            .public_signatures
            .iter()
            .any(|sig| sig.starts_with("find(1..2)")));
        // The private method never appears in the public shape.
        assert!(!registry
            .public_signatures
            .iter()
            .any(|sig| sig.starts_with("internal")));
        assert!(design
            .inbound_modules
            .iter()
            .any(|module| module.module == "app/Feature" && module.edge_count >= 1));
    }

    #[tokio::test]
    async fn impact_radius_reports_dependents_and_review_radius() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app/Core")).unwrap();
        fs::create_dir_all(fixture.join("app/Feature")).unwrap();
        fs::create_dir_all(fixture.join("app/Other")).unwrap();
        fs::write(
            fixture.join("app/Core/Registry.php"),
            br#"<?php
class Registry {
    public function find(string $key): ?string { return null; }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Feature/Consumer.php"),
            br#"<?php
use Core\Registry;
class Consumer {
    public function run(Registry $registry): void { $registry->find('a'); $registry->find('b'); }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Other/Indirect.php"),
            br#"<?php
use Feature\Consumer;
class Indirect {
    public function go(Consumer $consumer): void { $consumer->run(new \Core\Registry()); }
}
"#,
        )
        .unwrap();

        let server = AigiscodeMcpServer::load(fixture, None, true, is_kuzu_available()).unwrap();

        let radius = server
            .impact_radius(Parameters(super::ImpactRadiusParams {
                target: String::from("app/Core/Registry.php"),
                max_depth: None,
            }))
            .await
            .unwrap()
            .0;
        assert_eq!(radius.target_file, "app/Core/Registry.php");
        assert!(radius.direct_dependent_files >= 1);
        assert!(radius.transitive_dependent_files >= radius.direct_dependent_files);
        assert!(radius
            .review_radius
            .iter()
            .any(|entry| entry.file_path == "app/Feature/Consumer.php" && entry.edge_count >= 1));

        // Symbol-name targeting resolves without guessing.
        let by_symbol = server
            .impact_radius(Parameters(super::ImpactRadiusParams {
                target: String::from("Registry"),
                max_depth: Some(1),
            }))
            .await
            .unwrap()
            .0;
        assert_eq!(by_symbol.target_file, "app/Core/Registry.php");
        assert!(by_symbol.target_symbol.is_some());

        // Unknown target is an error, not an empty radius.
        assert!(server
            .impact_radius(Parameters(super::ImpactRadiusParams {
                target: String::from("does/not/Exist.php"),
                max_depth: None,
            }))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn repo_brief_stays_within_orientation_budget() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            b"mod b; fn main() { b::helper(); }\n",
        )
        .unwrap();
        fs::write(fixture.join("src/b.rs"), b"pub fn helper() {}\n").unwrap();

        let server = AigiscodeMcpServer::load(fixture, None, true, is_kuzu_available()).unwrap();
        let brief = server.repo_brief().await.0;

        assert!(brief.headline.contains("analyzed files"));
        assert!(!brief.guard_verdict.is_empty());
        assert!(!brief.doctrine_headline.is_empty());
        let freshness = brief.freshness.as_ref().expect("brief carries freshness");
        assert!(!freshness.is_stale);
        let payload = serde_json::to_string(&brief).unwrap();
        assert!(
            payload.len() <= 3 * 1024,
            "repo_brief must stay within the 3 KB orientation budget (got {} bytes)",
            payload.len()
        );
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

        let server = AigiscodeMcpServer::load(fixture, None, true, false).unwrap();
        server.mutate_state_for_test(|state| {
            state.graph_packets = GraphPacketArtifact {
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
                    locations: Vec::new(),
                    evidence_refs: AgenticPrimaryEvidenceRefs::default(),
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
        });

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

    #[tokio::test]
    async fn daemon_freshness_goes_stale_then_fresh_across_a_rebuild() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}\n").unwrap();
        let server = AigiscodeMcpServer::load(fixture.clone(), None, false, false).unwrap();
        let params = || Parameters(RepoOverviewParams::default());

        // Seeded: revision 1, fresh.
        let f = server.repo_overview(params()).await.0.freshness.unwrap();
        assert_eq!(f.indexed_revision, 1);
        assert!(!f.is_stale);

        // Observe a change exactly as the watcher would: now honestly stale.
        server.live.mark_dirty([(
            fixture.join("src/main.rs"),
            super::live::DirtyKind::Modified,
        )]);
        let f = server.repo_overview(params()).await.0.freshness.unwrap();
        assert_eq!(f.observed_revision, 2);
        assert_eq!(f.indexed_revision, 1);
        assert!(f.is_stale);
        assert_eq!(f.dirty_path_count, 1);

        // Rebuild + publish exactly as the watcher's loop does.
        let target = server.live.begin_rebuild();
        let state = super::build_mcp_state(&fixture, None, false, false).unwrap();
        server.live.publish(Some(state), target);

        let f = server.repo_overview(params()).await.0.freshness.unwrap();
        assert_eq!(f.indexed_revision, 2);
        assert_eq!(f.observed_revision, 2);
        assert!(!f.is_stale);
        assert_eq!(f.dirty_path_count, 0);
    }

    #[tokio::test]
    async fn watcher_observes_a_real_file_change_and_republishes() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}\n").unwrap();
        let server = AigiscodeMcpServer::load(fixture.clone(), None, false, false).unwrap();

        super::watch::spawn_watch(std::sync::Arc::clone(&server.live), fixture.clone());
        // Let the native watcher arm before mutating the tree.
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        fs::write(fixture.join("src/main.rs"), b"fn main() { let _x = 1; }\n").unwrap();

        // Poll for the change to be observed (debounce ~300ms) and a rebuilt snapshot to
        // be published (rev >= 2). Generous budget to stay non-flaky under load.
        let mut indexed = 1;
        for _ in 0..120 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            indexed = server.live.load().revision;
            if indexed >= 2 {
                break;
            }
        }
        assert!(
            indexed >= 2,
            "watcher should observe the edit and publish a rebuilt snapshot (indexed={indexed})"
        );
    }
}
