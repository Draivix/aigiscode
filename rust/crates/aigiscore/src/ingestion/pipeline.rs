use crate::assessment::{build_architectural_assessment_full, ArchitecturalAssessment};
use crate::contracts::{build_contract_inventory, ContractInventory};
use crate::detectors::dead_code::{analyze_dead_code_scoped, DeadCodeResult};
use crate::detectors::hardwiring::{analyze_hardwiring_with_contracts, HardwiringResult};
use crate::doctrine::{load_doctrine_registry_with_inputs, DoctrineLoadError, DoctrineRegistry};
use crate::external::ExternalAnalysisResult;
use crate::graph::analysis::{analyze_semantic_graph, GraphAnalysis};
use crate::graph::SemanticGraph;
use crate::ingestion::inputs::InputCapture;
use crate::ingestion::scan::{scan_repository, scan_repository_with_inputs, ScanConfig, ScanError, ScanResult, ScannedFile};
use crate::ingestion::structure::{build_structure_graph, StructureGraph};
use crate::parsing::{is_supported_source_file, parse_source_file, ParseFileError};
use crate::plugins::{apply_runtime_plugins, RepoContext};
use crate::policy::{PolicyBundle, PolicyLoadError};
use crate::resolve::{load_resolve_config_with_inputs, resolve_graph_with_config, ResolveConfigError, ResolutionCache, ResolutionWork};
use crate::scanners::ast_grep::{run_ast_grep_scan_with_cache, AstGrepScanCache, AstGrepScanResult, AstGrepScanWork};
use crate::security::{analyze_security_findings_with_ast_grep_and_graph, SecurityAnalysisResult};
use crate::surface::{build_architecture_surface, ArchitectureSurface};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngestionPhase {
    Scan,
    Structure,
    Parse,
    Resolve,
    Analyze,
    LoadGraph,
    LoadAnalysis,
    VerifyInputs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseTiming {
    pub phase: IngestionPhase,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestionPipelineResult {
    pub root: PathBuf,
    pub scan: ScanResult,
    pub structure: StructureGraph,
    pub timings: Vec<PhaseTiming>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticGraphProject {
    pub root: PathBuf,
    pub scan: ScanResult,
    pub structure: StructureGraph,
    pub semantic_graph: SemanticGraph,
    #[serde(default)]
    pub resolve_config_xxh3: String,
    pub timings: Vec<PhaseTiming>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_work: Option<ResolutionWork>,
    #[serde(skip)]
    pub parsed_sources: Vec<(PathBuf, String)>,
    #[serde(skip)]
    capture: InputCapture,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Clone))]
pub struct ProjectAnalysis {
    pub root: PathBuf,
    pub scan: ScanResult,
    pub structure: StructureGraph,
    pub semantic_graph: SemanticGraph,
    pub graph_analysis: GraphAnalysis,
    pub architectural_assessment: ArchitecturalAssessment,
    #[serde(skip)]
    doctrine_registry: DoctrineRegistry,
    #[serde(skip)]
    policy_bundle: PolicyBundle,
    #[serde(skip)]
    capture: InputCapture,
    #[serde(skip)]
    pub(crate) resolve_config_xxh3: String,
    pub contract_inventory: ContractInventory,
    pub dead_code: DeadCodeResult,
    pub hardwiring: HardwiringResult,
    #[serde(default, skip_serializing_if = "SecurityAnalysisResult::is_empty")]
    pub security_analysis: SecurityAnalysisResult,
    #[serde(default, skip_serializing_if = "ExternalAnalysisResult::is_empty")]
    pub external_analysis: ExternalAnalysisResult,
    #[serde(default, skip_serializing_if = "AstGrepScanResult::is_empty")]
    pub ast_grep_scan: AstGrepScanResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ast_grep_work: Option<AstGrepScanWork>,
    pub timings: Vec<PhaseTiming>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_work: Option<ResolutionWork>,
    #[serde(skip)]
    pub parsed_sources: Vec<(PathBuf, String)>,
}

impl ProjectAnalysis {
    pub(crate) fn verify_inputs(&self) -> Result<(), ProjectAnalysisError> {
        self.capture.verify(&self.scan)?;
        verify_supplemental(&self.dead_code, &self.parsed_sources, &self.scan)
    }

    pub fn doctrine_registry(&self) -> &DoctrineRegistry {
        &self.doctrine_registry
    }

    pub fn policy_bundle(&self) -> &PolicyBundle {
        &self.policy_bundle
    }

    pub fn architecture_surface(&self) -> ArchitectureSurface {
        build_architecture_surface(self)
    }

    /// Add external evidence without discarding the captured architectural doctrine.
    pub fn set_external_analysis(&mut self, external: ExternalAnalysisResult) {
        self.external_analysis = external;
        self.architectural_assessment = build_architectural_assessment_full(
            &self.graph_analysis,
            &self.dead_code,
            &self.hardwiring,
            &self.external_analysis,
            &self.parsed_sources,
            &self.ast_grep_scan,
            Some(&self.semantic_graph),
            &self.doctrine_registry.layers,
        );
    }
}

#[derive(Debug, Error)]
pub enum ProjectAnalysisError {
    #[error("analysis input changed during capture: {path}; retry on stable inputs")]
    InputChanged { path: PathBuf },
    #[error("failed to pin analysis artifacts: {0}")]
    Artifacts(std::io::Error),
    #[error(transparent)]
    Scan(#[from] ScanError),
    #[error(transparent)]
    Doctrine(#[from] DoctrineLoadError),
    #[error(transparent)]
    Policy(#[from] PolicyLoadError),
    #[error(transparent)]
    ResolveConfig(#[from] ResolveConfigError),
    #[error("failed to read source file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse source file {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: ParseFileError,
    },
}

pub type RustProjectAnalysis = ProjectAnalysis;
pub type RustProjectAnalysisError = ProjectAnalysisError;

pub fn run_ingestion_pipeline(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
) -> Result<IngestionPipelineResult, ScanError> {
    let root = root.into();

    let scan_started = Instant::now();
    let scan = scan_repository(&root, scan_config)?;
    let scan_elapsed = scan_started.elapsed().as_millis();

    let structure_started = Instant::now();
    let structure = build_structure_graph(&scan.files);
    let structure_elapsed = structure_started.elapsed().as_millis();

    Ok(IngestionPipelineResult {
        root,
        scan,
        structure,
        timings: vec![
            PhaseTiming {
                phase: IngestionPhase::Scan,
                elapsed_ms: scan_elapsed,
            },
            PhaseTiming {
                phase: IngestionPhase::Structure,
                elapsed_ms: structure_elapsed,
            },
        ],
    })
}

pub fn analyze_project(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
) -> Result<ProjectAnalysis, ProjectAnalysisError> {
    analyze_project_with_caches(root, scan_config, None, None)
}

pub(crate) fn analyze_project_with_caches(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
    resolver: Option<&mut ResolutionCache>,
    scanner: Option<&mut AstGrepScanCache>,
) -> Result<ProjectAnalysis, ProjectAnalysisError> {
    let graph_project = build_semantic_graph_project_with_resolver(root, scan_config, resolver)?;
    finish_project_analysis(graph_project, scanner, None)
}

/// Opt-in fast load (driven by `AIGISCORE_FAST_LOAD=1` at the call site):
/// skips Parse+Resolve when the scan manifest proves the analyzed file set
/// and contents are unchanged and the resolver-config fingerprint matches.
/// Returns `Ok(None)` on any doubt — fast-load may decline, never lie.
pub fn analyze_project_fast_load(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
    output_dir: Option<&Path>,
) -> Result<Option<ProjectAnalysis>, ProjectAnalysisError> {
    let root = root.into();
    let output_dir = output_dir.map(Path::to_path_buf)
        .unwrap_or_else(|| root.join(crate::artifacts::DEFAULT_OUTPUT_DIR_NAME));
    let Some(snapshot) = crate::artifacts::ArtifactSnapshot::pin(&output_dir)
        .map_err(ProjectAnalysisError::Artifacts)? else {
        return Ok(None);
    };
    analyze_project_fast_load_pinned(&root, scan_config, &snapshot.directory, None)
}

pub(crate) fn analyze_project_fast_load_pinned(
    root: &Path,
    scan_config: &ScanConfig,
    output_dir: &Path,
    scanner: Option<&mut AstGrepScanCache>,
) -> Result<Option<ProjectAnalysis>, ProjectAnalysisError> {
    let Some((graph_project, manifest)) = try_fast_load_graph_project(root, scan_config, Some(output_dir))? else {
        return Ok(None);
    };
    Ok(Some(finish_project_analysis(graph_project, scanner, Some((output_dir, &manifest)))?))
}

fn try_fast_load_graph_project(
    root: &Path,
    scan_config: &ScanConfig,
    output_dir: Option<&Path>,
) -> Result<Option<(SemanticGraphProject, crate::artifacts::ScanManifest)>, ProjectAnalysisError> {
    let scan_started = Instant::now();
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join(crate::artifacts::DEFAULT_OUTPUT_DIR_NAME));
    let manifest: crate::artifacts::ScanManifest =
        match fs::read_to_string(output_dir.join(crate::artifacts::SCAN_MANIFEST_FILE))
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
        {
            Some(manifest) => manifest,
            None => return Ok(None),
        };
    if manifest.aigiscode_version != env!("CARGO_PKG_VERSION")
        || manifest.semantic_revision != crate::artifacts::SEMANTIC_REVISION
        || manifest.semantic_graph_xxh3.len() != 16
    {
        return Ok(None);
    }

    let mut capture = InputCapture::new(scan_config);
    let scan = scan_repository_with_inputs(root, scan_config, &mut capture.files)?;
    if manifest.snapshot_identity.as_ref().is_none_or(|identity| {
        identity.scope_fingerprint != scan.scope_fingerprint
            || identity.root != scan.root.display().to_string()
            || identity.engine_fingerprint != env!("AIGISCODE_ENGINE_FINGERPRINT")
            || identity.input_inventory_fingerprint != scan.input_fingerprint()
            || identity.semantic_env_fingerprint != format!("{:032x}", scan.semantic_env.fingerprint.0)
    }) {
        return Ok(None);
    }
    let supported = scan
        .files
        .iter()
        .filter(|file| is_supported_source_file(&file.relative_path))
        .collect::<Vec<_>>();
    if supported.len() != manifest.files.len() {
        return Ok(None);
    }
    let resolve_config = load_resolve_config_with_inputs(
        root,
        &supported.iter().map(|file| file.relative_path.clone()).collect::<Vec<_>>(),
        &mut capture.files,
    )?;
    if manifest.resolve_config_xxh3 != resolve_config.fingerprint {
        return Ok(None);
    }
    let expected: HashMap<&str, &str> = manifest
        .files
        .iter()
        .map(|entry| (entry.path.as_str(), entry.xxh3.as_str()))
        .collect();
    // Reads are parallel; results merge in scan order and any mismatch
    // declines the whole fast-load.
    enum FastLoadFile {
        Match(PathBuf, String),
        Mismatch,
    }
    let loaded = supported
        .par_iter()
        .map(|file| {
            let display = file.relative_path.display().to_string();
            let Some(expected_hash) = expected.get(display.as_str()) else {
                return Ok(FastLoadFile::Mismatch);
            };
            let source = read_scanned_source(root, file)?;
            if format!("{:016x}", xxhash_rust::xxh3::xxh3_64(source.as_bytes())) != *expected_hash {
                return Ok(FastLoadFile::Mismatch);
            }
            Ok(FastLoadFile::Match(file.relative_path.clone(), source))
        })
        .collect::<Result<Vec<_>, ProjectAnalysisError>>()?;
    let mut parsed_sources = Vec::with_capacity(loaded.len());
    for item in loaded {
        match item {
            FastLoadFile::Match(path, source) => parsed_sources.push((path, source)),
            FastLoadFile::Mismatch => return Ok(None),
        }
    }

    let scan_elapsed = scan_started.elapsed().as_millis();
    let graph_load_started = Instant::now();
    let file = match fs::File::open(output_dir.join(crate::artifacts::SEMANTIC_GRAPH_FILE)) {
        Ok(file) => file,
        Err(_) => return Ok(None),
    };
    // Decode and hash the same stream. A second open would race publication,
    // while a whole-file String doubles peak memory for large cached graphs.
    let mut reader = BufReader::new(crate::ingestion::hash::HashingIo::new(file));
    let mut semantic_graph: SemanticGraph = match serde_json::from_reader(&mut reader) {
        Ok(graph) => graph,
        Err(_) => return Ok(None),
    };
    if format!("{:016x}", reader.get_ref().content_hash().0) != manifest.semantic_graph_xxh3 {
        return Ok(None);
    }
    update_input_inventory(&mut semantic_graph, &scan);
    let graph_load_elapsed = graph_load_started.elapsed().as_millis();
    let structure_started = Instant::now();
    let structure = build_structure_graph(&scan.files);
    Ok(Some((SemanticGraphProject {
        root: root.to_path_buf(),
        scan,
        structure,
        semantic_graph,
        resolve_config_xxh3: resolve_config.fingerprint,
        resolution_work: None,
        parsed_sources,
        capture,
        timings: vec![
            PhaseTiming {
                phase: IngestionPhase::Scan,
                elapsed_ms: scan_elapsed,
            },
            PhaseTiming {
                phase: IngestionPhase::LoadGraph,
                elapsed_ms: graph_load_elapsed,
            },
            PhaseTiming {
                phase: IngestionPhase::Structure,
                elapsed_ms: structure_started.elapsed().as_millis(),
            },
        ],
    }, manifest)))
}

fn finish_project_analysis(
    mut graph_project: SemanticGraphProject,
    scanner: Option<&mut AstGrepScanCache>,
    cached: Option<(&Path, &crate::artifacts::ScanManifest)>,
) -> Result<ProjectAnalysis, ProjectAnalysisError> {
    let analyze_started = Instant::now();
    let doctrine_registry = load_doctrine_registry_with_inputs(&graph_project.scan.root, &mut graph_project.capture.files)?;
    let policy_bundle = PolicyBundle::load_with_inputs(&graph_project.scan.root, &mut graph_project.capture.files)?;
    if let Some((directory, manifest)) = cached {
        let configuration_matches = manifest.snapshot_identity.as_ref().is_some_and(|identity| {
            identity.external_tools.is_empty() && identity.external_checks_complete
                && identity.assessment_config_fingerprint == crate::artifacts::SnapshotIdentity::assessment_config_fingerprint(&policy_bundle, &doctrine_registry)
        });
        if let Some(findings) = manifest.deterministic_findings_xxh3.as_deref()
            .filter(|_| configuration_matches)
            .and_then(|hash| crate::artifacts::CachedDeterministicFindings::load(directory, hash, &graph_project))
        {
            // Dead-code evidence can include a supplemental sweep outside the
            // parsed slice. Re-evaluate it rather than assuming the scan manifest
            // fingerprints those additional files.
            let dead_code = analyze_dead_code_scoped(&graph_project.semantic_graph, &graph_project.parsed_sources,
                &findings.contract_inventory, &graph_project.scan.root, &graph_project.scan.scope);
            if dead_code == findings.dead_code {
                let mut timings = graph_project.timings;
                let elapsed_ms = analyze_started.elapsed().as_millis();
                timings.push(PhaseTiming { phase: IngestionPhase::LoadAnalysis, elapsed_ms });
                trace(&format!("fast_load.native_analysis restored elapsed_ms={elapsed_ms}"));
                verify_capture(&graph_project.capture, &graph_project.scan, &mut timings, Some((&dead_code, &graph_project.parsed_sources)))?;
                return Ok(ProjectAnalysis {
                    root: graph_project.root,
                    scan: graph_project.scan,
                    structure: graph_project.structure,
                    semantic_graph: graph_project.semantic_graph,
                    resolve_config_xxh3: graph_project.resolve_config_xxh3,
                    resolution_work: graph_project.resolution_work,
                    parsed_sources: graph_project.parsed_sources,
                    graph_analysis: findings.graph_analysis,
                    architectural_assessment: findings.architectural_assessment,
                    contract_inventory: findings.contract_inventory,
                    dead_code,
                    hardwiring: findings.hardwiring,
                    security_analysis: findings.security_analysis,
                    ast_grep_scan: findings.ast_grep_scan,
                    external_analysis: ExternalAnalysisResult::default(),
                    ast_grep_work: None,
                    doctrine_registry,
                    policy_bundle,
                    capture: graph_project.capture,
                    timings,
                });
            }
        }
        trace("fast_load.native_analysis declined; recomputing native analysis");
    }
    let SemanticGraphProject {
        root,
        scan,
        structure,
        semantic_graph,
        resolve_config_xxh3,
        resolution_work,
        mut timings,
        parsed_sources,
        capture,
    } = graph_project;

    trace("analyze start");
    let graph_started = Instant::now();
    let graph_analysis = analyze_semantic_graph(&semantic_graph, &scan.scope);
    trace(&format!(
        "analyze.graph_analysis elapsed_ms={}",
        graph_started.elapsed().as_millis()
    ));

    let contract_started = Instant::now();
    let contract_inventory = build_contract_inventory(&parsed_sources);
    trace(&format!(
        "analyze.contract_inventory elapsed_ms={}",
        contract_started.elapsed().as_millis()
    ));
    let contract_lookup = contract_inventory.lookup();

    let dead_code_started = Instant::now();
    let dead_code = analyze_dead_code_scoped(
        &semantic_graph,
        &parsed_sources,
        &contract_inventory,
        &scan.root,
        &scan.scope,
    );
    trace(&format!(
        "analyze.dead_code elapsed_ms={}",
        dead_code_started.elapsed().as_millis()
    ));

    let hardwiring_started = Instant::now();
    let hardwiring = analyze_hardwiring_with_contracts(&parsed_sources, &contract_lookup);
    trace(&format!(
        "analyze.hardwiring elapsed_ms={}",
        hardwiring_started.elapsed().as_millis()
    ));

    let ast_grep_started = Instant::now();
    let incremental_scan = scanner.is_some();
    let (ast_grep_scan, scan_work) = run_ast_grep_scan_with_cache(&parsed_sources, scanner);
    let ast_grep_work = incremental_scan.then_some(scan_work);
    trace(&format!(
        "analyze.ast_grep elapsed_ms={}",
        ast_grep_started.elapsed().as_millis()
    ));

    let security_started = Instant::now();
    let security_analysis = analyze_security_findings_with_ast_grep_and_graph(
        &parsed_sources,
        &contract_inventory,
        &graph_analysis.runtime_entry_candidates,
        &ast_grep_scan,
        Some(&semantic_graph),
    );
    trace(&format!(
        "analyze.security elapsed_ms={}",
        security_started.elapsed().as_millis()
    ));

    let assessment_started = Instant::now();
    let architectural_assessment = build_architectural_assessment_full(
        &graph_analysis,
        &dead_code,
        &hardwiring,
        &ExternalAnalysisResult::default(),
        &parsed_sources,
        &ast_grep_scan,
        Some(&semantic_graph),
        &doctrine_registry.layers,
    );
    trace(&format!(
        "analyze.architectural_assessment elapsed_ms={}",
        assessment_started.elapsed().as_millis()
    ));
    let analyze_elapsed = analyze_started.elapsed().as_millis();
    trace(&format!(
        "analyze complete cycles={} contracts={} dead_code={} hardwiring={} elapsed_ms={analyze_elapsed}",
        graph_analysis.strong_circular_dependencies.len(),
        contract_inventory.summary.routes.unique_values
            + contract_inventory.summary.hooks.unique_values
            + contract_inventory.summary.registered_keys.unique_values
            + contract_inventory.summary.symbolic_literals.unique_values
            + contract_inventory.summary.env_keys.unique_values
            + contract_inventory.summary.config_keys.unique_values,
        dead_code.findings.len(),
        hardwiring.findings.len()
    ));
    timings.push(PhaseTiming {
        phase: IngestionPhase::Analyze,
        elapsed_ms: analyze_elapsed,
    });

    verify_capture(&capture, &scan, &mut timings, Some((&dead_code, &parsed_sources)))?;
    Ok(ProjectAnalysis {
        root,
        scan,
        structure,
        semantic_graph,
        graph_analysis,
        architectural_assessment,
        doctrine_registry,
        policy_bundle,
        capture,
        resolve_config_xxh3,
        contract_inventory,
        dead_code,
        hardwiring,
        security_analysis,
        external_analysis: ExternalAnalysisResult::default(),
        ast_grep_scan,
        ast_grep_work,
        resolution_work,
        timings,
        parsed_sources,
    })
}

pub fn analyze_rust_project(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
) -> Result<ProjectAnalysis, ProjectAnalysisError> {
    analyze_project(root, scan_config)
}

pub fn build_semantic_graph_project(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
) -> Result<SemanticGraphProject, ProjectAnalysisError> {
    let mut project = build_semantic_graph_project_with_resolver(root, scan_config, None)?;
    verify_capture(&project.capture, &project.scan, &mut project.timings, None)?;
    Ok(project)
}

pub(crate) fn build_semantic_graph_project_with_resolver(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
    resolver: Option<&mut ResolutionCache>,
) -> Result<SemanticGraphProject, ProjectAnalysisError> {
    let root = root.into();
    trace(&format!("analyze_project start {}", root.display()));

    let scan_started = Instant::now();
    let mut capture = InputCapture::new(scan_config);
    let scan = scan_repository_with_inputs(&root, scan_config, &mut capture.files)?;
    let scan_elapsed = scan_started.elapsed().as_millis();
    trace(&format!(
        "scan complete files={} elapsed_ms={scan_elapsed}",
        scan.files.len()
    ));

    let structure_started = Instant::now();
    let structure = build_structure_graph(&scan.files);
    let structure_elapsed = structure_started.elapsed().as_millis();
    trace(&format!(
        "structure complete nodes={} edges={} elapsed_ms={structure_elapsed}",
        structure.nodes.len(),
        structure.contains_edges.len()
    ));

    let parse_started = Instant::now();
    // Per-file parsing is embarrassingly parallel; results are collected in
    // scan order and merged sequentially, so the graph stays byte-identical
    // to a single-threaded run (Hard Rule: same repo, same bytes).
    let supported_files = scan
        .files
        .iter()
        .filter(|file| is_supported_source_file(&file.relative_path))
        .collect::<Vec<_>>();
    let parsed_results = supported_files
        .par_iter()
        .map(|file| {
            let absolute_path = root.join(&file.relative_path);
            let source = read_scanned_source(&root, file)?;
            let parsed =
                parse_source_file(file.relative_path.clone(), &source).map_err(|source| {
                    ProjectAnalysisError::Parse {
                        path: absolute_path.clone(),
                        source,
                    }
                })?;
            Ok((file.relative_path.clone(), source, parsed))
        })
        .collect::<Result<Vec<_>, ProjectAnalysisError>>()?;
    let mut semantic_graph = SemanticGraph::default();
    let mut parsed_sources = Vec::new();
    for (relative_path, source, parsed) in parsed_results {
        semantic_graph.append(parsed);
        parsed_sources.push((relative_path, source));
    }
    let parse_elapsed = parse_started.elapsed().as_millis();
    update_input_inventory(&mut semantic_graph, &scan);
    trace(&format!(
        "parse complete semantic_files={} symbols={} references={} elapsed_ms={parse_elapsed}",
        semantic_graph.files.len(),
        semantic_graph.symbols.len(),
        semantic_graph.references.len()
    ));

    let resolve_started = Instant::now();
    let resolve_config = load_resolve_config_with_inputs(
        &root,
        &semantic_graph.files.iter().map(|file| file.path.clone()).collect::<Vec<_>>(),
        &mut capture.files,
    )?;
    trace("resolve start");
    let resolution_work = match resolver {
        Some(resolver) => Some(resolver.resolve(&mut semantic_graph, &resolve_config)),
        None => {
            resolve_graph_with_config(&mut semantic_graph, &resolve_config);
            None
        }
    };
    let resolve_elapsed = resolve_started.elapsed().as_millis();
    trace(&format!(
        "resolve complete resolved_edges={} elapsed_ms={resolve_elapsed}",
        semantic_graph.resolved_edges.len()
    ));
    let plugins_started = Instant::now();
    apply_runtime_plugins(&RepoContext::new(root.clone(), &parsed_sources), &mut semantic_graph);
    semantic_graph.downgrade_recovered_edges();
    trace(&format!(
        "runtime plugins complete resolved_edges={} elapsed_ms={}",
        semantic_graph.resolved_edges.len(),
        plugins_started.elapsed().as_millis()
    ));

    let timings = vec![
        PhaseTiming { phase: IngestionPhase::Scan, elapsed_ms: scan_elapsed },
        PhaseTiming { phase: IngestionPhase::Structure, elapsed_ms: structure_elapsed },
        PhaseTiming { phase: IngestionPhase::Parse, elapsed_ms: parse_elapsed },
        PhaseTiming { phase: IngestionPhase::Resolve, elapsed_ms: resolve_elapsed },
    ];
    Ok(SemanticGraphProject {
        root,
        scan,
        structure,
        semantic_graph,
        resolve_config_xxh3: resolve_config.fingerprint,
        resolution_work,
        parsed_sources,
        capture,
        timings,
    })
}

fn verify_capture(
    capture: &InputCapture,
    scan: &ScanResult,
    timings: &mut Vec<PhaseTiming>,
    supplemental: Option<(&DeadCodeResult, &[(PathBuf, String)])>,
) -> Result<(), ProjectAnalysisError> {
    let started = Instant::now();
    capture.verify(scan)?;
    if let Some((dead_code, sources)) = supplemental {
        verify_supplemental(dead_code, sources, scan)?;
    }
    let elapsed_ms = started.elapsed().as_millis();
    trace(&format!("capture verified elapsed_ms={elapsed_ms}"));
    timings.push(PhaseTiming { phase: IngestionPhase::VerifyInputs, elapsed_ms });
    Ok(())
}

fn verify_supplemental(dead_code: &DeadCodeResult, sources: &[(PathBuf, String)], scan: &ScanResult) -> Result<(), ProjectAnalysisError> {
    if dead_code.supplemental_inputs_match(&scan.root, sources, &scan.scope) {
        Ok(())
    } else {
        Err(ProjectAnalysisError::InputChanged { path: scan.root.clone() })
    }
}

fn read_scanned_source(root: &Path, file: &ScannedFile) -> Result<String, ProjectAnalysisError> {
    let path = root.join(&file.relative_path);
    let source = fs::read_to_string(&path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            ProjectAnalysisError::InputChanged { path: path.clone() }
        } else {
            ProjectAnalysisError::ReadFile { path: path.clone(), source }
        }
    })?;
    if source.len() as u64 != file.size_bytes
        || crate::ingestion::hash::hash_bytes_xxh3(source.as_bytes()) != file.content_hash
    {
        return Err(ProjectAnalysisError::InputChanged { path });
    }
    Ok(source)
}

fn update_input_inventory(graph: &mut SemanticGraph, scan: &ScanResult) {
    graph.unsupported_sources = scan.files.iter()
        .filter(|file| !is_supported_source_file(&file.relative_path))
        .filter_map(|file| crate::coverage::unsupported_source(&file.relative_path))
        .collect();
    graph.other_input_files = scan.files.len().saturating_sub(
        graph.files.len() + graph.unsupported_sources.len(),
    );
}

fn trace(message: &str) {
    if env::var_os("AIGISCORE_TRACE").is_some() {
        eprintln!("[aigiscore] {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze_project, analyze_rust_project, run_ingestion_pipeline, IngestionPhase};
    use crate::graph::{GraphLayer, RelationKind};
    use crate::ingestion::scan::ScanConfig;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolver_uses_the_configuration_bytes_captured_by_scan() {
        let fixture = create_fixture();
        for directory in ["src/v1", "src/v2"] {
            fs::create_dir_all(fixture.join(directory)).unwrap();
        }
        let old = r#"{"compilerOptions":{"paths":{"@api/*":["src/v1/*"]}}}"#;
        let new = r#"{"compilerOptions":{"paths":{"@api/*":["src/v2/*"]}}}"#;
        fs::write(fixture.join("tsconfig.json"), old).unwrap();
        let sources = [
            ("src/main.ts", "import { User } from '@api/user'; export const user = new User();"),
            ("src/v1/user.ts", "export class User {}"),
            ("src/v2/user.ts", "export class User {}"),
        ];
        let mut graph = crate::graph::SemanticGraph::default();
        for (path, source) in sources {
            fs::write(fixture.join(path), source).unwrap();
            graph.append(crate::parsing::parse_source_file(PathBuf::from(path), source).unwrap());
        }
        let files = sources.iter().map(|(path, _)| PathBuf::from(path)).collect::<Vec<_>>();
        let mut capture = super::InputCapture::new(&ScanConfig::default());
        let scan = super::scan_repository_with_inputs(&fixture, &ScanConfig::default(), &mut capture.files).unwrap();
        // The first resolver read happens after the edit, but the scan already
        // captured this configuration for the semantic environment.
        fs::write(fixture.join("tsconfig.json"), new).unwrap();
        let captured = crate::resolve::load_resolve_config_with_inputs(&fixture, &files, &mut capture.files).unwrap();
        crate::resolve::resolve_graph_with_config(&mut graph, &captured);
        let constructor_target = |graph: &crate::graph::SemanticGraph| graph.resolved_edges.iter()
            .find(|edge| edge.source_file_path == Path::new("src/main.ts")
                && edge.kind == crate::graph::ReferenceKind::Call)
            .map(|edge| edge.target_file_path.clone());
        assert_eq!(constructor_target(&graph), Some(PathBuf::from("src/v1/user.ts")));
        assert!(matches!(capture.verify(&scan), Err(super::ProjectAnalysisError::InputChanged { .. })));
        let current = crate::resolve::load_resolve_config(&fixture, &files).unwrap();
        crate::resolve::resolve_graph_with_config(&mut graph, &current);
        assert_eq!(constructor_target(&graph), Some(PathBuf::from("src/v2/user.ts")));
        assert_ne!(captured.fingerprint, current.fingerprint);
    }

    #[test]
    fn completed_capture_rejects_data_inventory_and_absent_configuration_changes() {
        let fixture = create_fixture();
        fs::write(fixture.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(fixture.join("data.json"), "{\"value\":1}\n").unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        analysis.verify_inputs().unwrap();
        fs::write(fixture.join("data.json"), "{\"value\":2}\n").unwrap();
        assert!(matches!(analysis.verify_inputs(), Err(super::ProjectAnalysisError::InputChanged { .. })));
        fs::write(fixture.join("data.json"), "{\"value\":1}\n").unwrap();
        fs::write(fixture.join("new.rs"), "pub fn new() {}\n").unwrap();
        assert!(matches!(analysis.verify_inputs(), Err(super::ProjectAnalysisError::InputChanged { .. })));
        fs::remove_file(fixture.join("new.rs")).unwrap();
        fs::remove_file(fixture.join("data.json")).unwrap();
        assert!(matches!(analysis.verify_inputs(), Err(super::ProjectAnalysisError::InputChanged { .. })));
        fs::write(fixture.join("data.json"), "{\"value\":1}\n").unwrap();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::write(fixture.join(".aigiscode/policy.json"), "{}").unwrap();
        assert!(matches!(analysis.verify_inputs(), Err(super::ProjectAnalysisError::InputChanged { .. })));
        fs::remove_file(fixture.join(".aigiscode/policy.json")).unwrap();
        // An empty resolver root changes meaning without adding an admitted file.
        fs::create_dir(fixture.join("lib")).unwrap();
        assert!(matches!(analysis.verify_inputs(), Err(super::ProjectAnalysisError::InputChanged { .. })));
        fs::remove_dir(fixture.join("lib")).unwrap();
        fs::create_dir(fixture.join("target")).unwrap();
        fs::write(fixture.join("target/generated.rs"), "fn ignored() {}\n").unwrap();
        analysis.verify_inputs().unwrap();
    }

    #[test]
    fn source_reads_reject_changes_since_scan_including_same_size_edits_and_removal() {
        let fixture = create_fixture();
        let path = fixture.join("main.rs");
        fs::write(&path, "fn main() { aa(); }\n").unwrap();
        let scan = crate::ingestion::scan::scan_repository(&fixture, &ScanConfig::default()).unwrap();
        let file = scan.files.iter().find(|file| file.relative_path == Path::new("main.rs")).unwrap();
        assert!(super::read_scanned_source(&fixture, file).is_ok());
        fs::write(&path, "fn main() { bb(); }\n").unwrap();
        assert!(matches!(super::read_scanned_source(&fixture, file), Err(super::ProjectAnalysisError::InputChanged { .. })));
        fs::remove_file(&path).unwrap();
        assert!(matches!(super::read_scanned_source(&fixture, file), Err(super::ProjectAnalysisError::InputChanged { .. })));
    }

    #[test]
    fn fast_load_requires_data_inventory_and_hidden_semantic_environment_identity() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".cargo")).unwrap();
        fs::write(fixture.join(".cargo/config.toml"), "[build]\nincremental = true\n").unwrap();
        fs::write(fixture.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(fixture.join("data.json"), "{\"value\":1}\n").unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        crate::artifacts::write_project_analysis_artifacts(&analysis, None).unwrap();
        assert!(super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None).unwrap().is_some());
        fs::write(fixture.join("data.json"), "{\"value\":2}\n").unwrap();
        assert!(super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None).unwrap().is_none());
        fs::write(fixture.join("data.json"), "{\"value\":1}\n").unwrap();
        fs::write(fixture.join("new.json"), "{}\n").unwrap();
        assert!(super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None).unwrap().is_none());
        fs::remove_file(fixture.join("new.json")).unwrap();
        fs::remove_file(fixture.join("data.json")).unwrap();
        assert!(super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None).unwrap().is_none());
        fs::write(fixture.join("data.json"), "{\"value\":1}\n").unwrap();
        assert!(super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None).unwrap().is_some());
        fs::write(fixture.join(".cargo/config.toml"), "[build]\nincremental = false\n").unwrap();
        assert!(super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None).unwrap().is_none());
    }

    #[test]
    fn external_evidence_preserves_captured_layer_contracts() {
        use crate::assessment::ArchitecturalAssessmentKind::LayerContractViolation;
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::create_dir_all(fixture.join("domain")).unwrap();
        fs::create_dir_all(fixture.join("runtime")).unwrap();
        fs::write(
            fixture.join(".aigiscode/doctrine.json"),
            r#"{"layers":[
                {"name":"domain","path_prefixes":["domain"]},
                {"name":"runtime","path_prefixes":["runtime"],"may_depend_on":["domain"]}
            ]}"#,
        )
        .unwrap();
        fs::write(fixture.join("domain/model.ts"), "import '../runtime/io';").unwrap();
        fs::write(fixture.join("runtime/io.ts"), "export const io = 1;").unwrap();
        let mut analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let before = analysis
            .architectural_assessment
            .findings
            .iter()
            .filter(|finding| finding.kind == LayerContractViolation)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(before.len(), 1);
        let doctrine_path = fixture.join(".aigiscode/doctrine.json");
        let captured_doctrine = fs::read(&doctrine_path).unwrap();
        crate::artifacts::write_project_analysis_artifacts(&analysis, None).unwrap();
        let publication_path = fixture.join(".aigiscode/current-generation.json");
        let publication_before = fs::read(&publication_path).unwrap();
        // A run uses its captured doctrine even if the file changes afterward.
        fs::write(fixture.join(".aigiscode/doctrine.json"), "{}").unwrap();
        let external = crate::external::collect_external_analysis(
            &fixture,
            &fixture.join(".aigiscode"),
            &[String::from("unsupported")],
        )
        .unwrap();
        assert_eq!(external.tool_runs.len(), 1);
        analysis.set_external_analysis(external);
        let after = analysis
            .architectural_assessment
            .findings
            .iter()
            .filter(|finding| finding.kind == LayerContractViolation)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(before, after);
        assert_eq!(analysis.external_analysis.tool_runs.len(), 1);
        let error = crate::artifacts::write_project_analysis_artifacts(&analysis, None).unwrap_err();
        assert!(matches!(error.get_ref().and_then(|error| error.downcast_ref::<super::ProjectAnalysisError>()),
            Some(super::ProjectAnalysisError::InputChanged { .. })));
        assert_eq!(fs::read(publication_path).unwrap(), publication_before);
        fs::write(doctrine_path, captured_doctrine).unwrap();
        let paths = crate::artifacts::write_project_analysis_artifacts(&analysis, None).unwrap();
        let manifest: crate::artifacts::ScanManifest = serde_json::from_slice(&fs::read(paths.scan_manifest).unwrap()).unwrap();
        assert!(manifest.deterministic_findings_xxh3.is_none());
    }

    #[test]
    fn invalid_doctrine_is_an_analysis_error_before_artifact_writing() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        let path = fixture.join(".aigiscode/doctrine.json");
        fs::write(&path, "{broken").unwrap();
        assert!(matches!(
            analyze_project(&fixture, &ScanConfig::default()),
            Err(super::ProjectAnalysisError::Doctrine(_))
        ));
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(matches!(
            analyze_project(&fixture, &ScanConfig::default()),
            Err(super::ProjectAnalysisError::Doctrine(_))
        ));
    }

    #[test]
    fn fast_load_round_trips_unchanged_tree_and_declines_on_change() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/models.rs"),
            b"pub struct User {}\nimpl User { pub fn save(&self) {} }\n",
        )
        .unwrap();
        fs::write(fixture.join("src/main.rs"), b"fn main() {}\n").unwrap();
        fs::write(fixture.join("src/types.ts"), "export interface Item {}\n").unwrap();
        fs::write(
            fixture.join("src/reader.ts"),
            "import type { Item } from './types';\n",
        )
        .unwrap();
        fs::write(
            fixture.join("src/service.php"),
            "<?php namespace Example; class Service {} app(Service::class);",
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        crate::artifacts::write_project_analysis_artifacts(&analysis, None).unwrap();
        // Keep the legacy-flat cache contract covered alongside publication tests.
        fs::remove_file(fixture.join(".aigiscode/current-generation.json")).unwrap();
        fs::remove_dir_all(fixture.join(".aigiscode/.generations")).unwrap();

        // Unchanged tree: fast load succeeds and carries the same graph.
        let loaded = super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None)
            .unwrap()
            .expect("unchanged tree must fast-load");
        assert_eq!(
            loaded.semantic_graph.symbols.len(),
            analysis.semantic_graph.symbols.len()
        );
        assert_eq!(
            loaded.semantic_graph.resolved_edges.len(),
            analysis.semantic_graph.resolved_edges.len()
        );
        assert_eq!(loaded.semantic_graph, analysis.semantic_graph);
        assert_eq!(loaded.graph_analysis, analysis.graph_analysis);
        assert_eq!(loaded.architectural_assessment, analysis.architectural_assessment);
        assert_eq!(loaded.contract_inventory, analysis.contract_inventory);
        assert_eq!(loaded.dead_code, analysis.dead_code);
        assert_eq!(loaded.hardwiring, analysis.hardwiring);
        assert_eq!(loaded.ast_grep_scan, analysis.ast_grep_scan);
        assert_eq!(loaded.security_analysis, analysis.security_analysis);
        assert!(loaded.timings.iter().any(|timing| timing.phase == IngestionPhase::LoadAnalysis));
        assert!(!loaded.timings.iter().any(|timing| timing.phase == IngestionPhase::Analyze));

        // Valid JSON with unrelated counts must not stand in for this graph,
        // even if its bytes are named by a legacy manifest.
        let findings_path = fixture.join(".aigiscode/deterministic-findings.json");
        let original_findings = fs::read(&findings_path).unwrap();
        let manifest_path = fixture.join(".aigiscode/scan-manifest.json");
        let original_manifest = fs::read(&manifest_path).unwrap();
        let mut findings: serde_json::Value = serde_json::from_slice(&original_findings).unwrap();
        findings["scanned_files"] = serde_json::json!(0);
        let bytes = serde_json::to_vec(&findings).unwrap();
        let mut manifest: serde_json::Value = serde_json::from_slice(&original_manifest).unwrap();
        manifest["deterministic_findings_xxh3"] = serde_json::json!(format!("{:016x}", xxhash_rust::xxh3::xxh3_64(&bytes)));
        fs::write(&findings_path, bytes).unwrap();
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let recomputed = super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None).unwrap().unwrap();
        assert!(recomputed.timings.iter().any(|timing| timing.phase == IngestionPhase::Analyze));
        assert_eq!(recomputed.architectural_assessment, analysis.architectural_assessment);
        fs::write(&findings_path, original_findings).unwrap();
        fs::write(&manifest_path, original_manifest).unwrap();

        // A valid but unrelated graph must not be accepted for unchanged files.
        let graph_path = fixture.join(".aigiscode/semantic-graph.json");
        let original_graph = fs::read(&graph_path).unwrap();
        let mut unrelated: serde_json::Value = serde_json::from_slice(&original_graph).unwrap();
        unrelated["symbols"] = serde_json::json!([]);
        fs::write(&graph_path, serde_json::to_vec(&unrelated).unwrap()).unwrap();
        assert!(
            super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None)
                .unwrap()
                .is_none()
        );
        fs::write(&graph_path, original_graph).unwrap();

        // Identical files are insufficient after analyzer semantics change.
        let manifest_path = fixture.join(".aigiscode/scan-manifest.json");
        let original_manifest = fs::read_to_string(&manifest_path).unwrap();
        let mut old_manifest: serde_json::Value = serde_json::from_str(&original_manifest).unwrap();
        old_manifest
            .as_object_mut()
            .unwrap()
            .remove("semantic_revision");
        fs::write(&manifest_path, serde_json::to_vec(&old_manifest).unwrap()).unwrap();
        assert!(
            super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None)
                .unwrap()
                .is_none()
        );
        old_manifest["semantic_revision"] =
            serde_json::json!(crate::artifacts::SEMANTIC_REVISION + 1);
        fs::write(&manifest_path, serde_json::to_vec(&old_manifest).unwrap()).unwrap();
        assert!(
            super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None)
                .unwrap()
                .is_none()
        );
        fs::write(&manifest_path, original_manifest).unwrap();

        // Any content change must decline, never serve the stale graph.
        fs::write(fixture.join("src/main.rs"), b"fn main() { changed(); }\n").unwrap();
        assert!(
            super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None)
                .unwrap()
                .is_none()
        );

        // A deleted manifest also declines cleanly.
        let _ = fs::remove_file(fixture.join(".aigiscode/scan-manifest.json"));
        assert!(
            super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn fast_load_uses_only_the_selected_artifact_directory() {
        let fixture = create_fixture();
        fs::write(fixture.join("main.rs"), "fn main() {}\n").unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let output = fixture.join(".custom-artifacts");
        crate::artifacts::write_project_analysis_artifacts(&analysis, Some(&output)).unwrap();

        let loaded = super::analyze_project_fast_load(
            &fixture,
            &ScanConfig::default(),
            Some(&output),
        )
        .unwrap()
        .expect("custom artifact directory must fast-load");
        assert_eq!(loaded.semantic_graph, analysis.semantic_graph);
        assert!(super::analyze_project_fast_load(&fixture, &ScanConfig::default(), None)
            .unwrap()
            .is_none());

        crate::artifacts::write_project_analysis_artifacts(&analysis, None).unwrap();
        fs::remove_dir_all(&output).unwrap();
        assert!(super::analyze_project_fast_load(
            &fixture,
            &ScanConfig::default(),
            Some(&output),
        )
        .unwrap()
        .is_none(), "missing custom cache must not silently select the default cache");
    }

    #[test]
    fn generated_exclusions_apply_to_parser_and_supplemental_reachability() {
        let fixture = create_fixture();
        for directory in ["app", "cache", "bootstrap", ".aigiscode"] {
            fs::create_dir_all(fixture.join(directory)).unwrap();
        }
        fs::write(
            fixture.join("app/UnwiredService.php"),
            "<?php namespace App; class UnwiredService {}",
        )
        .unwrap();
        fs::write(
            fixture.join("app/LiveService.php"),
            "<?php namespace App; class LiveService {}",
        )
        .unwrap();
        fs::write(
            fixture.join("cache/FileIndex.php"),
            "<?php return ['UnwiredService'];",
        )
        .unwrap();
        fs::write(
            fixture.join("bootstrap/entry.php"),
            "<?php new \\App\\LiveService();",
        )
        .unwrap();
        fs::write(
            fixture.join(".aigiscode/scan.json"),
            r#"{
            "include_path_prefixes": ["app"],
            "generated_path_prefixes": ["./cache"]
        }"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            let external = create_fixture();
            fs::write(
                external.join("entry.php"),
                "<?php new \\App\\UnwiredService();",
            )
            .unwrap();
            std::os::unix::fs::symlink(
                external.join("entry.php"),
                fixture.join("bootstrap/external.php"),
            )
            .unwrap();
        }
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let orphans = analysis
            .dead_code
            .findings
            .iter()
            .filter(|finding| {
                finding.category == crate::detectors::dead_code::DeadCodeCategory::OrphanModule
            })
            .map(|finding| finding.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(orphans, vec!["UnwiredService"]);
        assert_eq!(analysis.semantic_graph.files.len(), 2);
        assert_eq!(
            analysis
                .architecture_surface()
                .overview
                .generated_path_prefixes,
            vec![PathBuf::from("cache")]
        );
        for prefix in ["", ".", "..", "../outside", "/outside"] {
            let config = ScanConfig {
                generated_path_prefixes: vec![PathBuf::from(prefix)],
                ..ScanConfig::default()
            };
            assert!(matches!(
                crate::ingestion::scan::scan_repository(&fixture, &config),
                Err(crate::ingestion::scan::ScanError::InvalidGeneratedPrefix(_))
            ));
        }
    }

    #[test]
    fn runs_scan_and_structure_as_explicit_phases() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src/core")).unwrap();
        fs::write(fixture.join("src/core/lib.rs"), b"pub fn demo() {}").unwrap();

        let result = run_ingestion_pipeline(&fixture, &ScanConfig::default()).unwrap();

        assert_eq!(result.scan.files.len(), 1);
        assert!(result
            .structure
            .nodes
            .iter()
            .any(|node| node.path == Path::new("src/core/lib.rs")));
        assert_eq!(result.timings.len(), 2);
        assert_eq!(result.timings[0].phase, IngestionPhase::Scan);
        assert_eq!(result.timings[1].phase, IngestionPhase::Structure);
    }

    #[test]
    fn analyzes_a_small_rust_project_end_to_end() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/models.rs"),
            b"pub struct User {}\nimpl User { pub fn save(&self) {} }\n",
        )
        .unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"use crate::models::User;

fn helper() {}
fn unused() {}

fn main() {
    let user = User {};
    let api = "https://api.example.com";
    helper();
    user.save();
    let _ = api;
}
"#,
        )
        .unwrap();

        let result = analyze_rust_project(&fixture, &ScanConfig::default()).unwrap();

        assert_eq!(result.scan.files.len(), 2);
        assert!(count_edges_to(&result, Path::new("src/models.rs")) >= 1);
        assert!(result
            .dead_code
            .findings
            .iter()
            .any(|finding| finding.name == "unused"));
        assert!(result
            .hardwiring
            .findings
            .iter()
            .any(|finding| finding.value == "https://api.example.com"));
        assert_eq!(result.timings.len(), 6);
        assert_eq!(result.timings[2].phase, IngestionPhase::Parse);
        assert_eq!(result.timings[3].phase, IngestionPhase::Resolve);
        assert_eq!(result.timings[4].phase, IngestionPhase::Analyze);
        assert_eq!(result.timings[5].phase, IngestionPhase::VerifyInputs);
    }

    #[test]
    fn analyzes_a_small_multi_language_project_end_to_end() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::create_dir_all(fixture.join("app/Models")).unwrap();
        fs::create_dir_all(fixture.join("app/models")).unwrap();

        fs::write(
            fixture.join("src/app.ts"),
            br#"import DefaultThing, { User } from "./models";
DefaultThing.run();
const user = new User();
const _unused = user;
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/models.ts"),
            br#"export class User {}
export class Service {
  static run() {}
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/service.py"),
            br#"from .models import User

def run(user: User):
    return user
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/models.py"),
            br#"class User:
    pass
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Service.php"),
            br#"<?php
use App\Models\User;
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Models/User.php"),
            br#"<?php namespace App\Models; class User {}"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/service.rb"),
            br#"require_relative "./models/user"
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/models/user.rb"),
            br#"class User
end
"#,
        )
        .unwrap();

        let result = analyze_project(&fixture, &ScanConfig::default()).unwrap();

        assert_eq!(result.scan.files.len(), 8);
        assert!(result
            .semantic_graph
            .files
            .iter()
            .any(|file| file.path == Path::new("src/app.ts")));
        assert!(count_edges_to(&result, Path::new("src/models.ts")) >= 1);
        assert!(count_edges_to(&result, Path::new("app/models.py")) >= 1);
        assert!(count_edges_to(&result, Path::new("app/Models/User.php")) >= 1);
        assert!(count_edges_to(&result, Path::new("app/models/user.rb")) >= 1);
        assert_eq!(result.timings.len(), 6);
    }

    #[test]
    fn analyzes_cross_file_factory_receivers_end_to_end() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::create_dir_all(fixture.join("app/Factories")).unwrap();
        fs::create_dir_all(fixture.join("app/Models")).unwrap();
        fs::create_dir_all(fixture.join("app")).unwrap();

        fs::write(
            fixture.join("src/service.ts"),
            br#"import { UserFactory as UF } from "./factory";

function run() {
  UF.buildUser().save();
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/factory.ts"),
            br#"import { User } from "./models";

export class UserFactory {
  static buildUser(): User {
    return new User();
  }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/models.ts"),
            br#"export class User {
  save() {}
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/service.py"),
            br#"from .factory import UserFactory as UF

def run():
    UF.build_user().save()
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/factory.py"),
            br#"from .models import User

class UserFactory:
    @staticmethod
    def build_user() -> User:
        return User()
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/models.py"),
            br#"class User:
    def save(self):
        pass
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Service.php"),
            br#"<?php
use App\Factories\UserFactory as UF;

function run() {
    UF::makeUser()->save();
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Factories/UserFactory.php"),
            br#"<?php
namespace App\Factories;

use App\Models\User;

class UserFactory {
    public static function makeUser(): User {
        return new User();
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Models/User.php"),
            br#"<?php
namespace App\Models;

class User {
    public function save() {}
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/service.rb"),
            br#"require_relative "./user"

def run
  User.new.save
end
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/user.rb"),
            br#"class User
  def save
  end
end
"#,
        )
        .unwrap();

        let result = analyze_project(&fixture, &ScanConfig::default()).unwrap();

        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.target_file_path == Path::new("src/models.ts")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.target_file_path == Path::new("app/models.py")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.target_file_path == Path::new("app/Models/User.php")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.target_file_path == Path::new("app/user.rb")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn applies_runtime_queue_plugins_during_project_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app/Services")).unwrap();
        fs::create_dir_all(fixture.join("app/Jobs")).unwrap();

        fs::write(
            fixture.join("app/Services/EmailSyncManager.php"),
            br#"<?php
namespace App\Services;

use App\Jobs\SyncAccountJob;

final class EmailSyncManager
{
    public static function notifyJob(): void {}

    public function run(): void
    {
        SyncAccountJob::dispatch('tenant', 1);
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Jobs/SyncAccountJob.php"),
            br#"<?php
namespace App\Jobs;

use App\Services\EmailSyncManager;

final class SyncAccountJob
{
    public function handle(): void
    {
        EmailSyncManager::notifyJob();
    }
}
"#,
        )
        .unwrap();

        let result = analyze_project(&fixture, &ScanConfig::default()).unwrap();

        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.relation_kind == RelationKind::Dispatch
                && edge.layer == GraphLayer::Runtime
                && edge.target_file_path == Path::new("app/Jobs/SyncAccountJob.php")
        }));
        assert!(result
            .graph_analysis
            .strong_cycle_findings
            .iter()
            .any(|finding| finding.cycle_class == crate::graph::analysis::CycleClass::Mixed));
    }

    #[test]
    fn builds_contract_inventory_during_project_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("routes")).unwrap();
        fs::create_dir_all(fixture.join("src")).unwrap();

        fs::write(
            fixture.join("routes/web.php"),
            br#"<?php
Route::get('/users', 'UserController@index');
add_action('init', 'boot_users');
config('mail.driver');
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/runtime.ts"),
            br#"type Status = 'draft' | 'published';
const mode = process.env.APP_MODE;
"#,
        )
        .unwrap();

        let result = analyze_project(&fixture, &ScanConfig::default()).unwrap();

        assert_eq!(result.contract_inventory.summary.routes.unique_values, 1);
        assert_eq!(result.contract_inventory.summary.hooks.unique_values, 1);
        assert_eq!(
            result.contract_inventory.summary.config_keys.unique_values,
            1
        );
        assert_eq!(result.contract_inventory.summary.env_keys.unique_values, 1);
        assert!(result
            .contract_inventory
            .symbolic_literals
            .iter()
            .any(|item| item.value == "draft"));
    }

    fn count_edges_to(result: &super::ProjectAnalysis, target: &Path) -> usize {
        result
            .semantic_graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.target_file_path == target)
            .count()
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aigiscore-pipeline-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
