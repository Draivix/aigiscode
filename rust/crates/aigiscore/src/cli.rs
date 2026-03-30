use crate::agent_runtime::{
    parse_agent_adapter_id, run_agent_review, run_agent_spider, AgentRunResult,
    AgentSpiderRunResult,
};
use crate::agentic::build_agentic_review_artifact;
use crate::artifacts::{
    build_agent_handoff_artifact, build_convergence_history_artifact,
    build_guard_decision_artifact, default_output_dir, write_architecture_surface_artifact,
    write_dependency_graph_artifact, write_evidence_graph_artifact,
    write_project_analysis_artifacts, write_semantic_graph_artifact, ArtifactPaths,
    AGENTIC_REVIEW_FILE, AGENT_HANDOFF_FILE, AIGISCODE_REPORT_FILE, AIGISCODE_REPORT_MARKDOWN_FILE,
    ARCHITECTURE_SURFACE_FILE, AST_GREP_SCAN_FILE, CONTRACT_INVENTORY_FILE,
    CONVERGENCE_HISTORY_FILE, DEPENDENCY_GRAPH_FILE, DETERMINISTIC_ANALYSIS_FILE,
    DETERMINISTIC_FINDINGS_FILE, DOCTRINE_REGISTRY_FILE, EVIDENCE_GRAPH_FILE,
    EXTERNAL_ANALYSIS_FILE, GRAPH_PACKETS_FILE, GUARD_DECISION_FILE, REPOSITORY_TOPOLOGY_FILE,
    REVIEW_SURFACE_FILE, SEMANTIC_GRAPH_FILE,
};
use crate::assessment::build_architectural_assessment_with_ast_grep_and_graph;
use crate::doctrine::load_doctrine_registry;
use crate::external::collect_external_analysis;
use crate::ingestion::pipeline::{
    analyze_project, analyze_rust_project, build_semantic_graph_project, PhaseTiming,
    ProjectAnalysis, SemanticGraphProject,
};
use crate::ingestion::scan::ScanConfig;
use crate::kuzu_index::{default_kuzu_path, query_kuzu, write_semantic_graph_kuzu_artifact};
use crate::mcp::run_stdio_server;
use crate::plugins::built_in_runtime_plugins;
use crate::policy::tune::{
    load_or_build_review_surface, suggest_policy_patch, write_policy_suggestion,
};
use crate::policy::PolicyBundle;
use crate::review::build_review_surface;
use crate::semantic_models::built_in_semantic_model_packs;
use serde::Serialize;
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};

struct AgentContext {
    review: crate::agentic::AgenticReviewArtifact,
}

fn build_agent_context(result: &ProjectAnalysis) -> Result<AgentContext, i32> {
    let surface = result.architecture_surface();
    let doctrine = load_doctrine_registry(&result.root).map_err(|error| {
        eprintln!("{error}");
        1
    })?;
    let review_surface = build_review_surface(result, &surface, &PolicyBundle::default());
    let convergence = build_convergence_history_artifact(
        &result.root,
        &result.semantic_graph,
        None,
        None,
        None,
        &surface,
        &review_surface,
        &result.contract_inventory,
        &doctrine,
    );
    let guard = build_guard_decision_artifact(&result.root, &convergence);
    let handoff = build_agent_handoff_artifact(result, &review_surface, &doctrine);
    let review = build_agentic_review_artifact(result, &doctrine, &handoff, &guard, &convergence);
    Ok(AgentContext { review })
}

pub fn run_with_default_stack() -> i32 {
    const STACK_SIZE_BYTES: usize = 256 * 1024 * 1024;
    let handle = std::thread::Builder::new()
        .name(String::from("aigiscore-main"))
        .stack_size(STACK_SIZE_BYTES)
        .spawn(run_from_env)
        .expect("failed to spawn aigiscore main thread");
    handle.join().unwrap_or_else(|_| {
        eprintln!("aigiscore aborted unexpectedly");
        1
    })
}

pub fn run_from_env() -> i32 {
    run(std::env::args().skip(1))
}

pub fn run<I>(args: I) -> i32
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let Some(command) = args.next() else {
        print_usage_and_exit();
    };

    match command.as_str() {
        "analyze" => {
            let (path, options) = parse_path_and_options(args);
            run_project_analysis_command(path, options)
        }
        "report" => {
            let (path, options) = parse_path_and_options(args);
            run_project_analysis_command(path, options)
        }
        "analyze-rust" => {
            let (path, options) = parse_path_and_options(args);
            match analyze_rust_project(path, &ScanConfig::default()) {
                Ok(result) => {
                    let artifact_paths = if options.write_artifacts {
                        match write_project_analysis_artifacts(
                            &result,
                            options.output_dir.as_deref(),
                        ) {
                            Ok(paths) => Some(paths),
                            Err(error) => {
                                eprintln!("{error}");
                                return 1;
                            }
                        }
                    } else {
                        None
                    };
                    let output =
                        build_analysis_command_output(&result, artifact_paths.as_ref(), None);
                    let json = serde_json::to_string_pretty(&output)
                        .expect("failed to serialize analysis summary");
                    println!("{json}");
                    0
                }
                Err(error) => {
                    eprintln!("{error}");
                    1
                }
            }
        }
        "surface" => {
            let (path, options) = parse_path_and_options(args);
            match analyze_project(path.clone(), &ScanConfig::default()) {
                Ok(result) => {
                    let surface = result.architecture_surface();
                    if options.write_artifacts {
                        if let Err(error) = write_architecture_surface_artifact(
                            &surface,
                            &path,
                            options.output_dir.as_deref(),
                        ) {
                            eprintln!("{error}");
                            return 1;
                        }
                    }
                    let json = serde_json::to_string_pretty(&surface)
                        .expect("failed to serialize architecture surface");
                    println!("{json}");
                    0
                }
                Err(error) => {
                    eprintln!("{error}");
                    1
                }
            }
        }
        "graph" => {
            let (path, options) = parse_path_and_options(args);
            run_graph_command(path, options)
        }
        "mcp" => {
            let (path, options) = parse_path_and_options(args);
            match run_stdio_server(
                path,
                options.output_dir,
                options.write_artifacts,
                options.write_kuzu,
            ) {
                Ok(()) => 0,
                Err(error) => {
                    eprintln!("{error}");
                    1
                }
            }
        }
        "cypher" => {
            let (path, query, output_dir) = parse_path_query_and_output_dir(args);
            run_cypher_command(path, query, output_dir)
        }
        "info" => {
            let (path, output_dir) = parse_path_and_output_dir_only(args);
            run_info_command(path, output_dir)
        }
        "agent" => {
            let (path, options) = parse_path_and_options(args);
            run_agent_command(path, options)
        }
        "agent-run" => {
            let (path, options) = parse_agent_run_args(args);
            run_agent_run_command(path, options)
        }
        "agent-spider" => {
            let (path, options) = parse_agent_spider_args(args);
            run_agent_spider_command(path, options)
        }
        "plugins" => run_plugins_command(),
        "tune" => {
            let (path, output_dir) = parse_path_and_output_dir_only(args);
            run_tune_command(path, output_dir)
        }
        "version" | "--version" | "-V" => {
            println!("aigiscode {}", env!("CARGO_PKG_VERSION"));
            0
        }
        _ => print_usage_and_exit(),
    }
}

#[derive(Debug, Default)]
struct ArtifactOptions {
    output_dir: Option<PathBuf>,
    write_artifacts: bool,
    write_kuzu: bool,
    external_tools: Vec<String>,
}

#[derive(Debug)]
struct AgentRunOptions {
    output_dir: Option<PathBuf>,
    adapter: crate::agentic::AgenticAdapterId,
    model: Option<String>,
}

#[derive(Debug)]
struct AgentSpiderOptions {
    output_dir: Option<PathBuf>,
    adapter: crate::agentic::AgenticAdapterId,
    model: Option<String>,
    limit: usize,
}

#[derive(Debug, Serialize)]
struct AnalyzeCommandOutput {
    root: PathBuf,
    artifacts: Option<AnalyzeArtifactOutput>,
    summary: AnalyzeCommandSummary,
    timings: Vec<PhaseTiming>,
}

#[derive(Debug, Serialize)]
struct GraphCommandOutput {
    root: PathBuf,
    artifact: Option<PathBuf>,
    dependency_graph: Option<PathBuf>,
    evidence_graph: Option<PathBuf>,
    kuzu_graph: Option<PathBuf>,
    summary: GraphCommandSummary,
    timings: Vec<PhaseTiming>,
}

#[derive(Debug, Serialize)]
struct GraphCommandSummary {
    scanned_files: usize,
    analyzed_files: usize,
    symbols: usize,
    references: usize,
    resolved_edges: usize,
}

#[derive(Debug, Serialize)]
struct AnalyzeArtifactOutput {
    output_dir: PathBuf,
    deterministic_analysis: PathBuf,
    semantic_graph: PathBuf,
    dependency_graph: PathBuf,
    evidence_graph: PathBuf,
    contract_inventory: PathBuf,
    doctrine_registry: PathBuf,
    kuzu_graph: Option<PathBuf>,
    deterministic_findings: PathBuf,
    ast_grep_scan: PathBuf,
    external_analysis: PathBuf,
    architecture_surface: PathBuf,
    review_surface: PathBuf,
    convergence_history: PathBuf,
    guard_decision: PathBuf,
    agent_handoff: PathBuf,
    agentic_review: PathBuf,
    graph_packets: PathBuf,
    repository_topology: PathBuf,
    aigiscode_report: PathBuf,
    aigiscode_report_markdown: PathBuf,
}

#[derive(Debug, Serialize)]
struct CypherCommandOutput {
    root: PathBuf,
    kuzu_graph: PathBuf,
    columns: Vec<String>,
    rows: Vec<JsonMap<String, JsonValue>>,
    row_count: usize,
}

#[derive(Debug, Serialize)]
struct PluginsCommandOutput {
    built_in_semantic_model_packs: Vec<PluginCatalogEntry>,
    built_in_runtime_plugins: Vec<PluginCatalogEntry>,
}

#[derive(Debug, Serialize)]
struct PluginCatalogEntry {
    id: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct AnalyzeCommandSummary {
    scanned_files: usize,
    analyzed_files: usize,
    symbols: usize,
    references: usize,
    resolved_edges: usize,
    strong_cycle_count: usize,
    total_cycle_count: usize,
    architectural_smell_count: usize,
    warning_heavy_hotspot_count: usize,
    split_identity_model_count: usize,
    compatibility_scar_count: usize,
    duplicate_mechanism_count: usize,
    hand_rolled_parsing_count: usize,
    abstraction_sprawl_count: usize,
    algorithmic_complexity_hotspot_count: usize,
    ast_grep_finding_count: usize,
    ast_grep_algorithmic_complexity_count: usize,
    ast_grep_security_dangerous_api_count: usize,
    ast_grep_framework_misuse_count: usize,
    ast_grep_skipped_file_count: usize,
    ast_grep_skipped_bytes: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ast_grep_skipped_files_preview: Vec<crate::scanners::ast_grep::AstGrepSkippedFile>,
    dead_code_count: usize,
    hardwiring_count: usize,
    security_finding_count: usize,
    external_tool_count: usize,
    external_finding_count: usize,
}

#[derive(Debug, Serialize)]
struct InfoArtifactPresence {
    deterministic_analysis: bool,
    semantic_graph: bool,
    dependency_graph: bool,
    evidence_graph: bool,
    contract_inventory: bool,
    doctrine_registry: bool,
    deterministic_findings: bool,
    ast_grep_scan: bool,
    external_analysis: bool,
    architecture_surface: bool,
    review_surface: bool,
    convergence_history: bool,
    guard_decision: bool,
    agent_handoff: bool,
    agentic_review: bool,
    graph_packets: bool,
    repository_topology: bool,
    aigiscode_report: bool,
    aigiscode_report_markdown: bool,
    kuzu_graph: bool,
}

#[derive(Debug, Serialize)]
struct AgentRunCommandOutput {
    root: PathBuf,
    adapter: String,
    review_json: PathBuf,
    review_markdown: PathBuf,
    output_schema: PathBuf,
    execution_events: PathBuf,
}

#[derive(Debug, Serialize)]
struct AgentSpiderCommandOutput {
    root: PathBuf,
    adapter: String,
    aggregate_report: PathBuf,
    packet_limit: usize,
    completed_packets: usize,
    failed_packets: usize,
}

#[derive(Debug, Serialize)]
struct TuneCommandOutput {
    root: PathBuf,
    suggested_policy_path: PathBuf,
    suggested_policy: JsonValue,
    suggestions: Vec<TuneSuggestionOutput>,
    summary: TuneCommandSummary,
}

#[derive(Debug, Serialize)]
struct TuneSuggestionOutput {
    field: String,
    value: JsonValue,
    reason: String,
}

#[derive(Debug, Serialize)]
struct TuneCommandSummary {
    visible_findings: usize,
    accepted_by_policy: usize,
    suppressed_by_rule: usize,
    runtime_entry_candidates: usize,
    repeated_literal_values: usize,
}

fn parse_path_and_options<I>(args: I) -> (PathBuf, ArtifactOptions)
where
    I: IntoIterator<Item = String>,
{
    let mut path: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut write_artifacts = true;
    let mut write_kuzu = false;
    let mut external_tools = Vec::new();
    let mut args = args.into_iter();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--output-dir" => {
                let Some(dir) = args.next() else {
                    eprintln!("missing value for --output-dir");
                    print_usage_and_exit();
                };
                output_dir = Some(PathBuf::from(dir));
            }
            "--no-write" => {
                write_artifacts = false;
            }
            "--kuzu" => {
                write_kuzu = true;
            }
            "--external-tool" => {
                let Some(tool) = args.next() else {
                    eprintln!("missing value for --external-tool");
                    print_usage_and_exit();
                };
                external_tools.push(tool);
            }
            "--external-tools" => {
                let Some(tools) = args.next() else {
                    eprintln!("missing value for --external-tools");
                    print_usage_and_exit();
                };
                external_tools.push(tools);
            }
            "--help" | "-h" => print_usage_and_exit(),
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                print_usage_and_exit();
            }
            value => {
                if path.is_some() {
                    eprintln!("unexpected argument: {value}");
                    print_usage_and_exit();
                }
                path = Some(PathBuf::from(value));
            }
        }
    }

    let Some(path) = path else {
        eprintln!("missing repository path");
        print_usage_and_exit();
    };

    (
        path,
        ArtifactOptions {
            output_dir,
            write_artifacts,
            write_kuzu,
            external_tools,
        },
    )
}

fn parse_path_query_and_output_dir<I>(args: I) -> (PathBuf, String, Option<PathBuf>)
where
    I: IntoIterator<Item = String>,
{
    let mut path: Option<PathBuf> = None;
    let mut query: Option<String> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut args = args.into_iter();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--output-dir" => {
                let Some(dir) = args.next() else {
                    eprintln!("missing value for --output-dir");
                    print_usage_and_exit();
                };
                output_dir = Some(PathBuf::from(dir));
            }
            "--help" | "-h" => print_usage_and_exit(),
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                print_usage_and_exit();
            }
            value => {
                if path.is_none() {
                    path = Some(PathBuf::from(value));
                } else if query.is_none() {
                    query = Some(String::from(value));
                } else {
                    eprintln!("unexpected argument: {value}");
                    print_usage_and_exit();
                }
            }
        }
    }

    let Some(path) = path else {
        eprintln!("missing repository path");
        print_usage_and_exit();
    };
    let Some(query) = query else {
        eprintln!("missing Cypher query");
        print_usage_and_exit();
    };
    (path, query, output_dir)
}

fn parse_path_and_output_dir_only<I>(args: I) -> (PathBuf, Option<PathBuf>)
where
    I: IntoIterator<Item = String>,
{
    let mut path: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut args = args.into_iter();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--output-dir" => {
                let Some(dir) = args.next() else {
                    eprintln!("missing value for --output-dir");
                    print_usage_and_exit();
                };
                output_dir = Some(PathBuf::from(dir));
            }
            "--help" | "-h" => print_usage_and_exit(),
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                print_usage_and_exit();
            }
            value => {
                if path.is_some() {
                    eprintln!("unexpected argument: {value}");
                    print_usage_and_exit();
                }
                path = Some(PathBuf::from(value));
            }
        }
    }

    let Some(path) = path else {
        eprintln!("missing repository path");
        print_usage_and_exit();
    };

    (path, output_dir)
}

fn parse_agent_run_args<I>(args: I) -> (PathBuf, AgentRunOptions)
where
    I: IntoIterator<Item = String>,
{
    let mut path: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut adapter = crate::agentic::AgenticAdapterId::CodexExecCli;
    let mut model: Option<String> = None;
    let mut args = args.into_iter();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--output-dir" => {
                let Some(dir) = args.next() else {
                    eprintln!("missing value for --output-dir");
                    print_usage_and_exit();
                };
                output_dir = Some(PathBuf::from(dir));
            }
            "--adapter" => {
                let Some(value) = args.next() else {
                    eprintln!("missing value for --adapter");
                    print_usage_and_exit();
                };
                adapter = match parse_agent_adapter_id(&value) {
                    Ok(adapter) => adapter,
                    Err(error) => {
                        eprintln!("{error}");
                        print_usage_and_exit();
                    }
                };
            }
            "--model" => {
                let Some(value) = args.next() else {
                    eprintln!("missing value for --model");
                    print_usage_and_exit();
                };
                model = Some(value);
            }
            "--help" | "-h" => print_usage_and_exit(),
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                print_usage_and_exit();
            }
            value => {
                if path.is_some() {
                    eprintln!("unexpected argument: {value}");
                    print_usage_and_exit();
                }
                path = Some(PathBuf::from(value));
            }
        }
    }

    let Some(path) = path else {
        eprintln!("missing repository path");
        print_usage_and_exit();
    };

    (
        path,
        AgentRunOptions {
            output_dir,
            adapter,
            model,
        },
    )
}

fn parse_agent_spider_args<I>(args: I) -> (PathBuf, AgentSpiderOptions)
where
    I: IntoIterator<Item = String>,
{
    let mut path: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut adapter = crate::agentic::AgenticAdapterId::CodexExecCli;
    let mut model: Option<String> = None;
    let mut limit: usize = 3;
    let mut args = args.into_iter();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--output-dir" => {
                let Some(dir) = args.next() else {
                    eprintln!("missing value for --output-dir");
                    print_usage_and_exit();
                };
                output_dir = Some(PathBuf::from(dir));
            }
            "--adapter" => {
                let Some(value) = args.next() else {
                    eprintln!("missing value for --adapter");
                    print_usage_and_exit();
                };
                adapter = match parse_agent_adapter_id(&value) {
                    Ok(adapter) => adapter,
                    Err(error) => {
                        eprintln!("{error}");
                        print_usage_and_exit();
                    }
                };
            }
            "--model" => {
                let Some(value) = args.next() else {
                    eprintln!("missing value for --model");
                    print_usage_and_exit();
                };
                model = Some(value);
            }
            "--limit" => {
                let Some(value) = args.next() else {
                    eprintln!("missing value for --limit");
                    print_usage_and_exit();
                };
                limit = value.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("invalid --limit value: {value}");
                    print_usage_and_exit();
                });
            }
            "--help" | "-h" => print_usage_and_exit(),
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                print_usage_and_exit();
            }
            value => {
                if path.is_some() {
                    eprintln!("unexpected argument: {value}");
                    print_usage_and_exit();
                }
                path = Some(PathBuf::from(value));
            }
        }
    }

    let Some(path) = path else {
        eprintln!("missing repository path");
        print_usage_and_exit();
    };

    (
        path,
        AgentSpiderOptions {
            output_dir,
            adapter,
            model,
            limit,
        },
    )
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "usage: aigiscode <command> <path> [--output-dir <dir>] [--no-write] [--external-tool <name>]\n\
         commands:\n\
         analyze       run full deterministic analysis and write native artifacts\n\
          report        compatibility alias for analyze that also writes aigiscode-report.json\n\
         analyze-rust  compatibility alias for analyze\n\
         agent         run full analysis and print the graph-backed AI review contract\n\
         agent-run     run the graph-backed AI review through a concrete agent adapter and write reports\n\
         agent-spider  crawl top task packets through a concrete agent adapter and write per-packet reports\n\
         graph         build and optionally write semantic-graph.json, dependency-graph.json, and evidence-graph.json without running detector/report phases\n\
          cypher        materialize/query the optional Kuzu graph index for code understanding\n\
          info          inspect existing Rust-native artifact state for one repository\n\
          plugins       list built-in runtime/framework overlay plugins\n\
          tune          suggest a narrow policy patch from current analysis signals\n\
          surface       emit architecture surface JSON and write architecture-surface.json\n\
          mcp           start the native Rust stdio MCP server for one repository\n\
          version       print CLI version\n\
         graph options:\n\
         --kuzu                    materialize the optional Kuzu graph artifact beside JSON output\n\
         agent-run options:\n\
          --adapter <name>         one of: codex-exec, responses-http, codex-sdk\n\
          --model <model>          override the adapter's default model\n\
         agent-spider options:\n\
          --adapter <name>         one of: codex-exec, responses-http, codex-sdk\n\
          --model <model>          override the adapter's default model\n\
          --limit <n>              crawl only the top N task packets (default: 3)\n\
         external tools:\n\
          --external-tool <name>   repeatable; supported: opengrep, trivy, grype, ruff, gitleaks, pip-audit, osv-scanner, composer-audit, npm-audit, cargo-deny, cargo-clippy\n\
          --external-tools <csv>   comma-separated alias; use 'all' to run every supported adapter"
    );
    std::process::exit(2);
}

fn build_analysis_command_output(
    result: &ProjectAnalysis,
    artifact_paths: Option<&ArtifactPaths>,
    kuzu_graph: Option<PathBuf>,
) -> AnalyzeCommandOutput {
    let ast_grep_family_counts = result.ast_grep_scan.family_counts();
    AnalyzeCommandOutput {
        root: result.root.clone(),
        artifacts: artifact_paths.map(|paths| AnalyzeArtifactOutput {
            output_dir: paths.output_dir.clone(),
            deterministic_analysis: paths.deterministic_analysis.clone(),
            semantic_graph: paths.semantic_graph.clone(),
            dependency_graph: paths.dependency_graph.clone(),
            evidence_graph: paths.evidence_graph.clone(),
            contract_inventory: paths.contract_inventory.clone(),
            doctrine_registry: paths.doctrine_registry.clone(),
            kuzu_graph: kuzu_graph.clone(),
            deterministic_findings: paths.deterministic_findings.clone(),
            ast_grep_scan: paths.ast_grep_scan.clone(),
            external_analysis: paths.external_analysis.clone(),
            architecture_surface: paths.architecture_surface.clone(),
            review_surface: paths.review_surface.clone(),
            convergence_history: paths.convergence_history.clone(),
            guard_decision: paths.guard_decision.clone(),
            agent_handoff: paths.agent_handoff.clone(),
            agentic_review: paths.agentic_review.clone(),
            graph_packets: paths.graph_packets.clone(),
            repository_topology: paths.repository_topology.clone(),
            aigiscode_report: paths.aigiscode_report.clone(),
            aigiscode_report_markdown: paths.aigiscode_report_markdown.clone(),
        }),
        summary: AnalyzeCommandSummary {
            scanned_files: result.scan.files.len(),
            analyzed_files: result.semantic_graph.files.len(),
            symbols: result.semantic_graph.symbols.len(),
            references: result.semantic_graph.references.len(),
            resolved_edges: result.semantic_graph.resolved_edges.len(),
            strong_cycle_count: result.graph_analysis.strong_circular_dependencies.len(),
            total_cycle_count: result.graph_analysis.circular_dependencies.len(),
            architectural_smell_count: result.graph_analysis.architectural_smells.len(),
            warning_heavy_hotspot_count: result
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::WarningHeavyHotspot),
            split_identity_model_count: result
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::SplitIdentityModel),
            compatibility_scar_count: result
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::CompatibilityScar),
            duplicate_mechanism_count: result
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::DuplicateMechanism),
            hand_rolled_parsing_count: result
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::HandRolledParsing),
            abstraction_sprawl_count: result
                .architectural_assessment
                .count_by_kind(crate::assessment::ArchitecturalAssessmentKind::AbstractionSprawl),
            algorithmic_complexity_hotspot_count: result.architectural_assessment.count_by_kind(
                crate::assessment::ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot,
            ),
            ast_grep_finding_count: result.ast_grep_scan.findings.len(),
            ast_grep_algorithmic_complexity_count: ast_grep_family_counts.algorithmic_complexity,
            ast_grep_security_dangerous_api_count: ast_grep_family_counts.security_dangerous_api,
            ast_grep_framework_misuse_count: ast_grep_family_counts.framework_misuse,
            ast_grep_skipped_file_count: result.ast_grep_scan.skipped_files.len(),
            ast_grep_skipped_bytes: result
                .ast_grep_scan
                .skipped_files
                .iter()
                .map(|file| file.bytes)
                .sum(),
            ast_grep_skipped_files_preview: result
                .ast_grep_scan
                .skipped_files
                .iter()
                .take(5)
                .cloned()
                .collect(),
            dead_code_count: result.dead_code.findings.len(),
            hardwiring_count: result.hardwiring.findings.len(),
            security_finding_count: result.security_analysis.findings.len(),
            external_tool_count: result.external_analysis.tool_runs.len(),
            external_finding_count: result.external_analysis.findings.len(),
        },
        timings: result.timings.clone(),
    }
}

fn run_project_analysis_command(path: PathBuf, options: ArtifactOptions) -> i32 {
    if !options.write_artifacts && !options.external_tools.is_empty() {
        eprintln!("external-tool execution requires artifact writing; remove --no-write");
        return 1;
    }
    if !options.write_artifacts && options.write_kuzu {
        eprintln!("--kuzu requires artifact writing; remove --no-write");
        return 1;
    }
    match analyze_project(path, &ScanConfig::default()) {
        Ok(mut result) => {
            trace_cli_step("analysis.returned");
            if !options.external_tools.is_empty() {
                let output_dir = options
                    .output_dir
                    .clone()
                    .unwrap_or_else(|| default_output_dir(&result.root));
                match collect_external_analysis(&result.root, &output_dir, &options.external_tools)
                {
                    Ok(external_analysis) => {
                        result.external_analysis = external_analysis;
                        result.architectural_assessment =
                            build_architectural_assessment_with_ast_grep_and_graph(
                                &result.graph_analysis,
                                &result.dead_code,
                                &result.hardwiring,
                                &result.external_analysis,
                                &result.parsed_sources,
                                &result.ast_grep_scan,
                                Some(&result.semantic_graph),
                            );
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        return 1;
                    }
                }
            }
            let artifact_paths = if options.write_artifacts {
                trace_cli_step("artifacts.start");
                match write_project_analysis_artifacts(&result, options.output_dir.as_deref()) {
                    Ok(paths) => {
                        trace_cli_step("artifacts.done");
                        Some(paths)
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        return 1;
                    }
                }
            } else {
                None
            };
            let kuzu_graph = if options.write_kuzu {
                match write_semantic_graph_kuzu_artifact(
                    &result.root,
                    &result.semantic_graph,
                    options.output_dir.as_deref(),
                ) {
                    Ok(path) => Some(path),
                    Err(error) => {
                        eprintln!("{error}");
                        return 1;
                    }
                }
            } else {
                None
            };
            let output =
                build_analysis_command_output(&result, artifact_paths.as_ref(), kuzu_graph);
            let json = serde_json::to_string_pretty(&output)
                .expect("failed to serialize analysis summary");
            println!("{json}");
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn trace_cli_step(step: &str) {
    if std::env::var_os("AIGISCORE_TRACE").is_none() {
        return;
    }
    eprintln!("[aigiscore] cli {step}");
}

fn run_graph_command(path: PathBuf, options: ArtifactOptions) -> i32 {
    if !options.external_tools.is_empty() {
        eprintln!("external tools are not supported for graph-only runs");
        return 1;
    }
    if !options.write_artifacts && options.write_kuzu {
        eprintln!("--kuzu requires artifact writing; remove --no-write");
        return 1;
    }
    match build_semantic_graph_project(path, &ScanConfig::default()) {
        Ok(result) => {
            let (artifact, dependency_graph, evidence_graph) = if options.write_artifacts {
                match write_semantic_graph_artifact(&result, options.output_dir.as_deref()) {
                    Ok(path) => {
                        let dependency_graph = match write_dependency_graph_artifact(
                            &result,
                            options.output_dir.as_deref(),
                        ) {
                            Ok(path) => Some(path),
                            Err(error) => {
                                eprintln!("{error}");
                                return 1;
                            }
                        };
                        let evidence_graph = match write_evidence_graph_artifact(
                            &result,
                            options.output_dir.as_deref(),
                        ) {
                            Ok(path) => Some(path),
                            Err(error) => {
                                eprintln!("{error}");
                                return 1;
                            }
                        };
                        (Some(path), dependency_graph, evidence_graph)
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        return 1;
                    }
                }
            } else {
                (None, None, None)
            };
            let kuzu_graph = if options.write_kuzu {
                match write_semantic_graph_kuzu_artifact(
                    &result.root,
                    &result.semantic_graph,
                    options.output_dir.as_deref(),
                ) {
                    Ok(path) => Some(path),
                    Err(error) => {
                        eprintln!("{error}");
                        return 1;
                    }
                }
            } else {
                None
            };
            let output = build_graph_command_output(
                &result,
                artifact,
                dependency_graph,
                evidence_graph,
                kuzu_graph,
            );
            let json =
                serde_json::to_string_pretty(&output).expect("failed to serialize graph summary");
            println!("{json}");
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn build_graph_command_output(
    result: &SemanticGraphProject,
    artifact: Option<PathBuf>,
    dependency_graph: Option<PathBuf>,
    evidence_graph: Option<PathBuf>,
    kuzu_graph: Option<PathBuf>,
) -> GraphCommandOutput {
    GraphCommandOutput {
        root: result.root.clone(),
        artifact,
        dependency_graph,
        evidence_graph,
        kuzu_graph,
        summary: GraphCommandSummary {
            scanned_files: result.scan.files.len(),
            analyzed_files: result.semantic_graph.files.len(),
            symbols: result.semantic_graph.symbols.len(),
            references: result.semantic_graph.references.len(),
            resolved_edges: result.semantic_graph.resolved_edges.len(),
        },
        timings: result.timings.clone(),
    }
}

fn run_cypher_command(path: PathBuf, query: String, output_dir: Option<PathBuf>) -> i32 {
    let db_path = default_kuzu_path(&path, output_dir.as_deref());
    let db_path = if db_path.exists() {
        db_path
    } else {
        match build_semantic_graph_project(path.clone(), &ScanConfig::default()) {
            Ok(result) => match write_semantic_graph_kuzu_artifact(
                &result.root,
                &result.semantic_graph,
                output_dir.as_deref(),
            ) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("{error}");
                    return 1;
                }
            },
            Err(error) => {
                eprintln!("{error}");
                return 1;
            }
        }
    };

    match query_kuzu(&db_path, &query) {
        Ok(result) => {
            let output = CypherCommandOutput {
                root: path,
                kuzu_graph: result.db_path,
                columns: result.columns,
                rows: result.rows,
                row_count: result.row_count,
            };
            let json =
                serde_json::to_string_pretty(&output).expect("failed to serialize cypher output");
            println!("{json}");
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn run_info_command(path: PathBuf, output_dir: Option<PathBuf>) -> i32 {
    match build_info_command_output(&path, output_dir.as_deref()) {
        Ok(output) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&output).expect("failed to serialize info output")
            );
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn run_agent_command(path: PathBuf, options: ArtifactOptions) -> i32 {
    if !options.write_artifacts && !options.external_tools.is_empty() {
        eprintln!("external-tool execution requires artifact writing; remove --no-write");
        return 1;
    }
    if !options.write_artifacts && options.write_kuzu {
        eprintln!("--kuzu requires artifact writing; remove --no-write");
        return 1;
    }

    match analyze_project(path, &ScanConfig::default()) {
        Ok(mut result) => {
            if !options.external_tools.is_empty() {
                let output_dir = options
                    .output_dir
                    .clone()
                    .unwrap_or_else(|| default_output_dir(&result.root));
                match collect_external_analysis(&result.root, &output_dir, &options.external_tools)
                {
                    Ok(external_analysis) => {
                        result.external_analysis = external_analysis;
                        result.architectural_assessment =
                            build_architectural_assessment_with_ast_grep_and_graph(
                                &result.graph_analysis,
                                &result.dead_code,
                                &result.hardwiring,
                                &result.external_analysis,
                                &result.parsed_sources,
                                &result.ast_grep_scan,
                                Some(&result.semantic_graph),
                            );
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        return 1;
                    }
                }
            }

            let agentic_review = if options.write_artifacts {
                let artifact_paths = match write_project_analysis_artifacts(
                    &result,
                    options.output_dir.as_deref(),
                ) {
                    Ok(paths) => paths,
                    Err(error) => {
                        eprintln!("{error}");
                        return 1;
                    }
                };
                if options.write_kuzu {
                    if let Err(error) = write_semantic_graph_kuzu_artifact(
                        &result.root,
                        &result.semantic_graph,
                        options.output_dir.as_deref(),
                    ) {
                        eprintln!("{error}");
                        return 1;
                    }
                }
                match fs::read(&artifact_paths.agentic_review) {
                    Ok(payload) => match serde_json::from_slice::<JsonValue>(&payload) {
                        Ok(agentic_review) => agentic_review,
                        Err(error) => {
                            eprintln!(
                                "failed to parse {}: {error}",
                                artifact_paths.agentic_review.display()
                            );
                            return 1;
                        }
                    },
                    Err(error) => {
                        eprintln!(
                            "failed to read {}: {error}",
                            artifact_paths.agentic_review.display()
                        );
                        return 1;
                    }
                }
            } else {
                let context = match build_agent_context(&result) {
                    Ok(context) => context,
                    Err(code) => return code,
                };
                serde_json::to_value(&context.review)
                    .expect("failed to serialize in-memory agentic review")
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&agentic_review)
                    .expect("failed to serialize agentic review output")
            );
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn run_agent_run_command(path: PathBuf, options: AgentRunOptions) -> i32 {
    match analyze_project(path.clone(), &ScanConfig::default()) {
        Ok(result) => {
            if let Err(error) =
                write_project_analysis_artifacts(&result, options.output_dir.as_deref())
            {
                eprintln!("{error}");
                return 1;
            }
            let context = match build_agent_context(&result) {
                Ok(context) => context,
                Err(code) => return code,
            };
            let run_result = match run_agent_review(
                &context.review,
                &result.root,
                options.output_dir.as_deref(),
                options.adapter,
                options.model.as_deref(),
            ) {
                Ok(result) => result,
                Err(error) => {
                    eprintln!("{error}");
                    return 1;
                }
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&build_agent_run_command_output(
                    &result.root,
                    &run_result
                ))
                .expect("failed to serialize agent-run output")
            );
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn run_agent_spider_command(path: PathBuf, options: AgentSpiderOptions) -> i32 {
    match analyze_project(path.clone(), &ScanConfig::default()) {
        Ok(result) => {
            if let Err(error) =
                write_project_analysis_artifacts(&result, options.output_dir.as_deref())
            {
                eprintln!("{error}");
                return 1;
            }
            let context = match build_agent_context(&result) {
                Ok(context) => context,
                Err(code) => return code,
            };
            let spider_result = match run_agent_spider(
                &context.review,
                &result.root,
                options.output_dir.as_deref(),
                options.adapter,
                options.model.as_deref(),
                options.limit,
            ) {
                Ok(result) => result,
                Err(error) => {
                    eprintln!("{error}");
                    return 1;
                }
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&build_agent_spider_command_output(
                    &result.root,
                    &spider_result,
                ))
                .expect("failed to serialize agent-spider output")
            );
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn build_agent_run_command_output(root: &Path, result: &AgentRunResult) -> AgentRunCommandOutput {
    AgentRunCommandOutput {
        root: root.to_path_buf(),
        adapter: format!("{:?}", result.adapter),
        review_json: result.review_json.clone(),
        review_markdown: result.review_markdown.clone(),
        output_schema: result.output_schema.clone(),
        execution_events: result.execution_events.clone(),
    }
}

fn build_agent_spider_command_output(
    root: &Path,
    result: &AgentSpiderRunResult,
) -> AgentSpiderCommandOutput {
    AgentSpiderCommandOutput {
        root: root.to_path_buf(),
        adapter: format!("{:?}", result.adapter),
        aggregate_report: result.aggregate_report.clone(),
        packet_limit: result.packet_limit,
        completed_packets: result.completed_packets,
        failed_packets: result.failed_packets,
    }
}

fn run_plugins_command() -> i32 {
    let output = build_plugins_command_output();
    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("failed to serialize plugins output")
    );
    0
}

fn run_tune_command(path: PathBuf, output_dir: Option<PathBuf>) -> i32 {
    match analyze_project(&path, &ScanConfig::default()) {
        Ok(analysis) => match load_or_build_review_surface(&analysis) {
            Ok(review_surface) => {
                let suggestion = suggest_policy_patch(&analysis, &review_surface);
                let suggested_policy_path =
                    match write_policy_suggestion(&suggestion, output_dir.as_deref()) {
                        Ok(path) => path,
                        Err(error) => {
                            eprintln!("{error}");
                            return 1;
                        }
                    };
                let output = TuneCommandOutput {
                    root: analysis.root.clone(),
                    suggested_policy_path,
                    suggested_policy: suggestion.suggested_policy.clone(),
                    suggestions: suggestion
                        .suggestions
                        .iter()
                        .map(|suggestion| TuneSuggestionOutput {
                            field: suggestion.field.clone(),
                            value: suggestion.value.clone(),
                            reason: suggestion.reason.clone(),
                        })
                        .collect(),
                    summary: TuneCommandSummary {
                        visible_findings: suggestion.summary.visible_findings,
                        accepted_by_policy: suggestion.summary.accepted_by_policy,
                        suppressed_by_rule: suggestion.summary.suppressed_by_rule,
                        runtime_entry_candidates: suggestion.summary.runtime_entry_candidates,
                        repeated_literal_values: suggestion.summary.repeated_literal_values,
                    },
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).expect("failed to serialize tune output")
                );
                0
            }
            Err(error) => {
                eprintln!("{error}");
                1
            }
        },
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn build_plugins_command_output() -> PluginsCommandOutput {
    PluginsCommandOutput {
        built_in_semantic_model_packs: built_in_semantic_model_packs()
            .iter()
            .map(|pack| PluginCatalogEntry {
                id: String::from(pack.id),
                description: String::from(pack.description),
            })
            .collect(),
        built_in_runtime_plugins: built_in_runtime_plugins()
            .iter()
            .map(|plugin| PluginCatalogEntry {
                id: String::from(plugin.id),
                description: String::from(plugin.description),
            })
            .collect(),
    }
}

fn build_info_command_output(root: &Path, output_dir: Option<&Path>) -> Result<JsonValue, String> {
    let artifact_paths = expected_artifact_paths(root, output_dir);
    let kuzu_graph = default_kuzu_path(root, output_dir);
    let report = read_json_if_exists(&artifact_paths.aigiscode_report)?;
    let surface = read_json_if_exists(&artifact_paths.architecture_surface)?;
    let contract_inventory = read_json_if_exists(&artifact_paths.contract_inventory)?;
    let doctrine_registry = read_json_if_exists(&artifact_paths.doctrine_registry)?;
    let ast_grep_scan = read_json_if_exists(&artifact_paths.ast_grep_scan)?;
    let convergence_history = read_json_if_exists(&artifact_paths.convergence_history)?;
    let guard_decision = read_json_if_exists(&artifact_paths.guard_decision)?;
    let agentic_review = read_json_if_exists(&artifact_paths.agentic_review)?;
    let graph_packets = read_json_if_exists(&artifact_paths.graph_packets)?;
    let repository_topology = read_json_if_exists(&artifact_paths.repository_topology)?;

    if report.is_none()
        && surface.is_none()
        && contract_inventory.is_none()
        && doctrine_registry.is_none()
        && ast_grep_scan.is_none()
        && guard_decision.is_none()
        && agentic_review.is_none()
        && graph_packets.is_none()
        && repository_topology.is_none()
    {
        return Err(format!(
            "no analysis artifacts found under {}; run `aigiscode analyze {}` first",
            artifact_paths.output_dir.display(),
            root.display()
        ));
    }

    let hotspots = surface
        .as_ref()
        .and_then(|payload| payload.get("hotspots"))
        .and_then(|value| value.as_array())
        .map(|items| JsonValue::Array(items.iter().take(10).cloned().collect()))
        .unwrap_or(JsonValue::Array(Vec::new()));

    let output = json!({
        "root": root,
        "output_dir": artifact_paths.output_dir,
        "artifacts": InfoArtifactPresence {
            deterministic_analysis: artifact_paths.deterministic_analysis.exists(),
            semantic_graph: artifact_paths.semantic_graph.exists(),
            dependency_graph: artifact_paths.dependency_graph.exists(),
            evidence_graph: artifact_paths.evidence_graph.exists(),
            contract_inventory: artifact_paths.contract_inventory.exists(),
            doctrine_registry: artifact_paths.doctrine_registry.exists(),
            deterministic_findings: artifact_paths.deterministic_findings.exists(),
            ast_grep_scan: artifact_paths.ast_grep_scan.exists(),
            external_analysis: artifact_paths.external_analysis.exists(),
            architecture_surface: artifact_paths.architecture_surface.exists(),
            review_surface: artifact_paths.review_surface.exists(),
            convergence_history: artifact_paths.convergence_history.exists(),
            guard_decision: artifact_paths.guard_decision.exists(),
            agent_handoff: artifact_paths.agent_handoff.exists(),
            agentic_review: artifact_paths.agentic_review.exists(),
            graph_packets: artifact_paths.graph_packets.exists(),
            repository_topology: artifact_paths.repository_topology.exists(),
            aigiscode_report: artifact_paths.aigiscode_report.exists(),
            aigiscode_report_markdown: artifact_paths.aigiscode_report_markdown.exists(),
            kuzu_graph: kuzu_graph.exists(),
        },
        "summary": report.as_ref().and_then(|payload| payload.get("summary")).cloned().unwrap_or(JsonValue::Null),
        "feedback_loop": report.as_ref().and_then(|payload| payload.get("feedback_loop")).cloned().unwrap_or(JsonValue::Null),
        "convergence": convergence_history.as_ref().and_then(|payload| payload.get("summary")).cloned().unwrap_or(JsonValue::Null),
        "guard_decision": guard_decision.unwrap_or(JsonValue::Null),
        "agentic_review": agentic_review.unwrap_or(JsonValue::Null),
        "graph_packets": graph_packets.unwrap_or(JsonValue::Null),
        "repository_topology": repository_topology.unwrap_or(JsonValue::Null),
        "ast_grep_scan": ast_grep_scan.unwrap_or(JsonValue::Null),
        "convergence_attention": convergence_history.as_ref().map(|payload| {
            json!({
                "attention_items_count": payload
                    .get("attention_items")
                    .and_then(|value| value.as_array())
                    .map(|items| items.len())
                    .unwrap_or(0),
                "required_investigation_files": payload
                    .get("required_investigation_files")
                    .cloned()
                    .unwrap_or(JsonValue::Array(Vec::new())),
                "required_radius": payload
                    .get("required_radius")
                    .cloned()
                    .unwrap_or(JsonValue::Null),
            })
        }).unwrap_or(JsonValue::Null),
        "contract_inventory": contract_inventory.as_ref().map(|payload| {
            payload.get("summary").cloned().unwrap_or(JsonValue::Null)
        }).unwrap_or_else(|| {
            report.as_ref()
                .and_then(|payload| payload.get("contract_inventory"))
                .and_then(|value| value.get("summary"))
                .cloned()
                .unwrap_or(JsonValue::Null)
        }),
        "doctrine_registry": doctrine_registry.unwrap_or(JsonValue::Null),
        "languages": surface.as_ref().and_then(|payload| payload.get("languages")).cloned().unwrap_or(JsonValue::Array(Vec::new())),
        "top_hotspots": hotspots,
    });
    Ok(output)
}

fn expected_artifact_paths(root: &Path, output_dir: Option<&Path>) -> ArtifactPaths {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(root));
    ArtifactPaths {
        output_dir: output_dir.clone(),
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
    }
}

fn read_json_if_exists(path: &Path) -> Result<Option<JsonValue>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let payload =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_slice(&payload)
        .map(Some)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{
        build_info_command_output, build_plugins_command_output, run, run_agent_command,
        run_tune_command, ArtifactOptions,
    };
    use crate::artifacts::write_project_analysis_artifacts;
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn plugins_command_lists_built_in_runtime_plugins() {
        let output = build_plugins_command_output();
        assert!(output
            .built_in_semantic_model_packs
            .iter()
            .any(|pack| pack.id == "django_routes"));
        assert!(output
            .built_in_semantic_model_packs
            .iter()
            .any(|pack| pack.id == "wordpress_rest_routes"));
        assert!(output
            .built_in_runtime_plugins
            .iter()
            .any(|plugin| plugin.id == "queue_dispatch"));
        assert!(output
            .built_in_runtime_plugins
            .iter()
            .any(|plugin| plugin.id == "laravel_container"));
        assert!(output
            .built_in_runtime_plugins
            .iter()
            .any(|plugin| plugin.id == "wordpress_hooks"));
    }

    #[test]
    fn info_command_reads_existing_artifact_state() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("routes")).unwrap();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("routes/web.php"),
            br#"<?php Route::get('/users', 'UserController@index'); add_action('init', 'boot');"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/runtime.ts"),
            br#"type Status = 'draft' | 'published'; const mode = process.env.APP_MODE;"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let output_dir = fixture.join(".aigiscode-test");
        write_project_analysis_artifacts(&analysis, Some(&output_dir)).unwrap();

        let output = build_info_command_output(&fixture, Some(&output_dir)).unwrap();
        let summary = output.get("summary").and_then(Value::as_object).unwrap();
        let contract_summary = output
            .get("contract_inventory")
            .and_then(Value::as_object)
            .unwrap();

        assert!(summary.get("scanned_files").is_some());
        assert_eq!(contract_summary["routes"]["unique_values"], Value::from(1));
        assert_eq!(output["artifacts"]["contract_inventory"], Value::Bool(true));
        assert_eq!(output["artifacts"]["ast_grep_scan"], Value::Bool(true));
    }

    #[test]
    fn tune_command_writes_suggested_policy_patch() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/app.ts"), br#"export const ready = true;"#).unwrap();
        fs::write(
            fixture.join("src/index.ts"),
            br#"import { ready } from "./app";
if (!ready) { throw new Error("boot failed"); }"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/main.ts"),
            br#"const a = "alpha"; const b = "alpha";
const c = "beta"; const d = "beta";
const e = "gamma"; const f = "gamma";
const g = "delta"; const h = "delta";
const i = "epsilon"; const j = "epsilon";
const k = "zeta"; const l = "zeta";
const m = "lambda"; const n = "lambda";
const o = "theta"; const p = "theta";
const q = "iota"; const r = "iota";
const s = "kappa"; const t = "kappa";"#,
        )
        .unwrap();

        let output_dir = fixture.join(".aigiscode-out");
        assert_eq!(
            run_tune_command(fixture.clone(), Some(output_dir.clone())),
            0
        );
        let payload: Value = serde_json::from_str(
            &fs::read_to_string(output_dir.join("policy.suggested.json")).unwrap(),
        )
        .unwrap();
        assert!(payload["graph"]["orphan_entry_patterns"].is_array());
        assert_eq!(
            payload["hardwiring"]["repeated_literal_min_occurrences"],
            Value::from(3)
        );
    }

    #[test]
    fn command_dispatch_recognizes_plugins_and_tune() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn main() { let _ = "shared"; let _ = "shared"; }"#,
        )
        .unwrap();

        assert_eq!(run(["plugins"]), 0);
        assert_eq!(run(["tune", fixture.to_string_lossy().as_ref()]), 0);
        assert_eq!(run(["agent", fixture.to_string_lossy().as_ref()]), 0);
    }

    #[test]
    fn agent_command_writes_and_prints_agentic_review() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn main() { let mode = std::env::var("APP_MODE").unwrap_or_default(); println!("{}", mode); }"#,
        )
        .unwrap();

        let output_dir = fixture.join(".aigiscode-agent");
        assert_eq!(
            run_agent_command(
                fixture.clone(),
                ArtifactOptions {
                    output_dir: Some(output_dir.clone()),
                    write_artifacts: true,
                    write_kuzu: false,
                    external_tools: Vec::new(),
                },
            ),
            0
        );

        let payload: Value = serde_json::from_str(
            &fs::read_to_string(output_dir.join("agentic-review.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            payload["transport"]["provider_family"],
            Value::from("openai")
        );
        assert_eq!(
            payload["transport"]["recommended_protocol"],
            Value::from("responses_api")
        );
        assert_eq!(payload["execution"]["provider"], Value::from("openai"));
        assert_eq!(
            payload["execution"]["openai_responses"]["endpoint"],
            Value::from("https://api.openai.com/v1/responses")
        );
        assert_eq!(
            payload["execution"]["report_targets"][0]["file_name"],
            Value::from("agent-review.md")
        );
    }

    #[test]
    fn agent_command_supports_no_write_mode() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn main() { let mode = std::env::var("APP_MODE").unwrap_or_default(); println!("{}", mode); }"#,
        )
        .unwrap();

        assert_eq!(
            run(["agent", fixture.to_string_lossy().as_ref(), "--no-write"]),
            0
        );
        assert!(!fixture.join(".aigiscode").exists());
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-cli-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
