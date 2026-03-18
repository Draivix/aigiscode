use crate::artifacts::{
    default_output_dir, write_architecture_surface_artifact, write_dependency_graph_artifact,
    write_evidence_graph_artifact, write_project_analysis_artifacts, write_semantic_graph_artifact,
    ArtifactPaths,
};
use crate::external::collect_external_analysis;
use crate::ingestion::pipeline::{
    analyze_project, analyze_rust_project, build_semantic_graph_project, PhaseTiming,
    ProjectAnalysis, SemanticGraphProject,
};
use crate::ingestion::scan::ScanConfig;
use crate::kuzu_index::{default_kuzu_path, query_kuzu, write_semantic_graph_kuzu_artifact};
use crate::mcp::run_stdio_server;
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::PathBuf;

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
    kuzu_graph: Option<PathBuf>,
    deterministic_findings: PathBuf,
    external_analysis: PathBuf,
    architecture_surface: PathBuf,
    review_surface: PathBuf,
    agent_handoff: PathBuf,
    aigiscode_report: PathBuf,
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
struct AnalyzeCommandSummary {
    scanned_files: usize,
    analyzed_files: usize,
    symbols: usize,
    references: usize,
    resolved_edges: usize,
    strong_cycle_count: usize,
    total_cycle_count: usize,
    dead_code_count: usize,
    hardwiring_count: usize,
    external_tool_count: usize,
    external_finding_count: usize,
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

fn print_usage_and_exit() -> ! {
    eprintln!(
        "usage: aigiscode <command> <path> [--output-dir <dir>] [--no-write] [--external-tool <name>]\n\
         commands:\n\
         analyze       run full deterministic analysis and write native artifacts\n\
          report        compatibility alias for analyze that also writes aigiscode-report.json\n\
         analyze-rust  compatibility alias for analyze\n\
          graph         build and optionally write semantic-graph.json, dependency-graph.json, and evidence-graph.json without running detector/report phases\n\
          cypher        materialize/query the optional Kuzu graph index for code understanding\n\
          surface       emit architecture surface JSON and write architecture-surface.json\n\
          mcp           start the native Rust stdio MCP server for one repository\n\
          version       print CLI version\n\
         graph options:\n\
          --kuzu                    materialize the optional Kuzu graph artifact beside JSON output\n\
         external tools:\n\
          --external-tool <name>   repeatable; supported: ruff, gitleaks, pip-audit, osv-scanner, composer-audit, npm-audit, cargo-deny, cargo-clippy\n\
          --external-tools <csv>   comma-separated alias; use 'all' to run every supported adapter"
    );
    std::process::exit(2);
}

fn build_analysis_command_output(
    result: &ProjectAnalysis,
    artifact_paths: Option<&ArtifactPaths>,
    kuzu_graph: Option<PathBuf>,
) -> AnalyzeCommandOutput {
    AnalyzeCommandOutput {
        root: result.root.clone(),
        artifacts: artifact_paths.map(|paths| AnalyzeArtifactOutput {
            output_dir: paths.output_dir.clone(),
            deterministic_analysis: paths.deterministic_analysis.clone(),
            semantic_graph: paths.semantic_graph.clone(),
            dependency_graph: paths.dependency_graph.clone(),
            evidence_graph: paths.evidence_graph.clone(),
            kuzu_graph: kuzu_graph.clone(),
            deterministic_findings: paths.deterministic_findings.clone(),
            external_analysis: paths.external_analysis.clone(),
            architecture_surface: paths.architecture_surface.clone(),
            review_surface: paths.review_surface.clone(),
            agent_handoff: paths.agent_handoff.clone(),
            aigiscode_report: paths.aigiscode_report.clone(),
        }),
        summary: AnalyzeCommandSummary {
            scanned_files: result.scan.files.len(),
            analyzed_files: result.semantic_graph.files.len(),
            symbols: result.semantic_graph.symbols.len(),
            references: result.semantic_graph.references.len(),
            resolved_edges: result.semantic_graph.resolved_edges.len(),
            strong_cycle_count: result.graph_analysis.strong_circular_dependencies.len(),
            total_cycle_count: result.graph_analysis.circular_dependencies.len(),
            dead_code_count: result.dead_code.findings.len(),
            hardwiring_count: result.hardwiring.findings.len(),
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
            if !options.external_tools.is_empty() {
                let output_dir = options
                    .output_dir
                    .clone()
                    .unwrap_or_else(|| default_output_dir(&result.root));
                match collect_external_analysis(&result.root, &output_dir, &options.external_tools)
                {
                    Ok(external_analysis) => {
                        result.external_analysis = external_analysis;
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        return 1;
                    }
                }
            }
            let artifact_paths = if options.write_artifacts {
                match write_project_analysis_artifacts(&result, options.output_dir.as_deref()) {
                    Ok(paths) => Some(paths),
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
