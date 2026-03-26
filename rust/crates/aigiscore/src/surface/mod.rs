use crate::assessment::{ArchitecturalAssessmentFinding, ArchitecturalAssessmentKind};
use crate::contracts::ContractInventory;
use crate::detectors::dead_code::{DeadCodeCategory, DeadCodeFinding, DeadCodeProofTier};
use crate::detectors::hardwiring::{HardwiringCategory, HardwiringFinding};
use crate::evidence::EvidenceAnchor;
use crate::external::{ExternalFinding, ExternalSeverity};
use crate::graph::analysis::{
    ArchitecturalSmell, ArchitecturalSmellKind, BottleneckFile, CycleClass, CycleFinding,
};
use crate::graph::{GraphLayer, Language, ReferenceKind, RelationKind, ResolutionTier};
use crate::identity::{normalized_path, stable_fingerprint};
use crate::ingestion::pipeline::{PhaseTiming, ProjectAnalysis};
use crate::security::{SecurityCategory, SecurityFinding, SecuritySeverity};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

struct IndexedParsedSource<'a> {
    content: &'a str,
    lowered_lines: Vec<String>,
    first_non_empty_line: Option<usize>,
}

struct SurfaceBuildContext<'a> {
    parsed_source_lookup: HashMap<&'a Path, IndexedParsedSource<'a>>,
    component_path_lookup: HashMap<String, PathBuf>,
}

impl<'a> SurfaceBuildContext<'a> {
    fn new(analysis: &'a ProjectAnalysis) -> Self {
        let parsed_source_lookup = analysis
            .parsed_sources
            .iter()
            .map(|(path, content)| {
                let lowered_lines = content
                    .lines()
                    .map(|line| line.to_ascii_lowercase())
                    .collect::<Vec<_>>();
                let first_non_empty_line = content
                    .lines()
                    .enumerate()
                    .find(|(_, line)| !line.trim().is_empty())
                    .map(|(index, _)| index + 1);
                (
                    path.as_path(),
                    IndexedParsedSource {
                        content: content.as_str(),
                        lowered_lines,
                        first_non_empty_line,
                    },
                )
            })
            .collect();
        let mut component_path_lookup = HashMap::new();
        for file in &analysis.semantic_graph.files {
            let path_string = file.path.to_string_lossy().to_string();
            component_path_lookup.insert(path_string, file.path.clone());
            if let Some(name) = file.path.file_name().and_then(|name| name.to_str()) {
                component_path_lookup
                    .entry(name.to_string())
                    .or_insert_with(|| file.path.clone());
            }
        }
        Self {
            parsed_source_lookup,
            component_path_lookup,
        }
    }
}

fn resolve_component_path(component: &str, context: &SurfaceBuildContext<'_>) -> Option<PathBuf> {
    if let Some(path) = context.component_path_lookup.get(component) {
        return Some(path.clone());
    }

    let component = component.trim_matches('/');
    context
        .parsed_source_lookup
        .keys()
        .filter(|path| {
            if component == "." {
                return true;
            }
            path.iter()
                .next()
                .map(|segment| segment.to_string_lossy() == component)
                .unwrap_or(false)
        })
        .map(|path| (*path).to_path_buf())
        .min_by(|left, right| {
            left.components()
                .count()
                .cmp(&right.components().count())
                .then(left.cmp(right))
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureSurface {
    pub root: PathBuf,
    pub overview: SurfaceOverview,
    pub contract_inventory: ContractInventory,
    pub languages: Vec<LanguageCoverage>,
    pub hotspots: Vec<HotspotFile>,
    pub cycle_findings: Vec<SurfaceCycleFinding>,
    pub highlights: Vec<SurfaceFinding>,
    pub atlas: RepositoryAtlas,
    pub timings: Vec<PhaseTiming>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceOverview {
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
    #[serde(default)]
    pub boundary_truncated_count: usize,
    pub runtime_entry_count: usize,
    #[serde(default)]
    pub boundary_truth: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_reasons: Vec<String>,
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
    #[serde(default)]
    pub hand_rolled_parsing_count: usize,
    #[serde(default)]
    pub abstraction_sprawl_count: usize,
    #[serde(default)]
    pub algorithmic_complexity_hotspot_count: usize,
    #[serde(default)]
    pub ast_grep_finding_count: usize,
    #[serde(default)]
    pub ast_grep_algorithmic_complexity_count: usize,
    #[serde(default)]
    pub ast_grep_security_dangerous_api_count: usize,
    #[serde(default)]
    pub ast_grep_framework_misuse_count: usize,
    pub route_contract_count: usize,
    pub hook_contract_count: usize,
    pub registered_key_count: usize,
    pub symbolic_literal_count: usize,
    pub env_key_count: usize,
    pub config_key_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageCoverage {
    pub language: String,
    pub file_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotspotFile {
    pub file_path: PathBuf,
    pub language: String,
    pub inbound_edges: usize,
    pub outbound_edges: usize,
    pub finding_count: usize,
    pub bottleneck_centrality_millis: u32,
    pub is_orphan: bool,
    #[serde(default)]
    pub is_boundary_truncated: bool,
    pub is_runtime_entry: bool,
    #[serde(default)]
    pub boundary_truth: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceCycleFinding {
    pub id: String,
    pub fingerprint: String,
    pub cycle_class: String,
    pub files: Vec<PathBuf>,
    pub layers: Vec<String>,
    pub dominant_relations: Vec<String>,
    pub edge_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceFindingFamily {
    Graph,
    DeadCode,
    Hardwiring,
    Security,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceFindingSeverity {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceFinding {
    pub id: String,
    pub fingerprint: String,
    pub family: SurfaceFindingFamily,
    pub severity: SurfaceFindingSeverity,
    pub precision: String,
    pub confidence_millis: u16,
    pub title: String,
    pub summary: String,
    pub file_paths: Vec<PathBuf>,
    pub line: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_anchor: Option<EvidenceAnchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_anchors: Vec<EvidenceAnchor>,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryAtlas {
    pub nodes: Vec<AtlasNode>,
    pub edges: Vec<AtlasEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlasNode {
    pub file_path: PathBuf,
    pub language: String,
    pub inbound_edges: usize,
    pub outbound_edges: usize,
    pub finding_count: usize,
    pub bottleneck_centrality_millis: u32,
    pub is_orphan: bool,
    #[serde(default)]
    pub is_boundary_truncated: bool,
    pub is_runtime_entry: bool,
    #[serde(default)]
    pub boundary_truth: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlasEdge {
    pub source_file_path: PathBuf,
    pub target_file_path: PathBuf,
    pub edge_count: usize,
    pub kinds: Vec<String>,
    pub strongest_resolution_tier: String,
    pub average_confidence_millis: u16,
}

pub fn effective_runtime_entry_files(analysis: &ProjectAnalysis) -> HashSet<PathBuf> {
    let mut runtime_entry_set = analysis
        .graph_analysis
        .runtime_entry_candidates
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    for route in &analysis.contract_inventory.routes {
        for location in &route.locations {
            runtime_entry_set.insert(location.file_path.clone());
        }
    }
    runtime_entry_set
}

pub fn effective_orphan_files(analysis: &ProjectAnalysis) -> Vec<PathBuf> {
    let runtime_entry_set = effective_runtime_entry_files(analysis);
    analysis
        .graph_analysis
        .orphan_files
        .iter()
        .filter(|path| !runtime_entry_set.contains(*path))
        .cloned()
        .collect()
}

pub fn effective_boundary_truncated_files(analysis: &ProjectAnalysis) -> Vec<PathBuf> {
    let runtime_entry_set = effective_runtime_entry_files(analysis);
    analysis
        .graph_analysis
        .boundary_truncated_files
        .iter()
        .filter(|path| !runtime_entry_set.contains(*path))
        .cloned()
        .collect()
}

pub fn build_architecture_surface(analysis: &ProjectAnalysis) -> ArchitectureSurface {
    let surface_started_at = Instant::now();
    trace_surface_step(
        "enter",
        surface_started_at.elapsed().as_millis(),
        Some(format!(
            "parsed_sources={} files={} resolved_edges={}",
            analysis.parsed_sources.len(),
            analysis.semantic_graph.files.len(),
            analysis.semantic_graph.resolved_edges.len()
        )),
    );
    let context_started_at = Instant::now();
    let context = SurfaceBuildContext::new(analysis);
    trace_surface_step(
        "context.build",
        context_started_at.elapsed().as_millis(),
        None,
    );
    let orphan_files = effective_orphan_files(analysis);
    let orphan_set = orphan_files.iter().cloned().collect::<HashSet<_>>();
    let boundary_truncated_files = effective_boundary_truncated_files(analysis);
    let boundary_truncated_set = boundary_truncated_files
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let runtime_entry_set = effective_runtime_entry_files(analysis);
    let bottlenecks = analysis
        .graph_analysis
        .bottleneck_files
        .iter()
        .map(|file| (file.file_path.clone(), file.centrality_millis))
        .collect::<HashMap<_, _>>();
    let file_languages = collect_file_languages(analysis);
    let inbound_started_at = Instant::now();
    let inbound_outbound = collect_inbound_outbound_edges(analysis);
    trace_surface_step(
        "collect_inbound_outbound_edges",
        inbound_started_at.elapsed().as_millis(),
        Some(format!("files={}", inbound_outbound.len())),
    );
    let counts_started_at = Instant::now();
    let finding_counts = collect_finding_counts(analysis);
    trace_surface_step(
        "collect_finding_counts",
        counts_started_at.elapsed().as_millis(),
        Some(format!("files={}", finding_counts.len())),
    );

    let hotspots_started_at = Instant::now();
    let mut hotspots = analysis
        .semantic_graph
        .files
        .iter()
        .map(|file| {
            let (inbound_edges, outbound_edges) =
                inbound_outbound.get(&file.path).copied().unwrap_or((0, 0));
            HotspotFile {
                file_path: file.path.clone(),
                language: language_label(file.language),
                inbound_edges,
                outbound_edges,
                finding_count: finding_counts.get(&file.path).copied().unwrap_or(0),
                bottleneck_centrality_millis: bottlenecks.get(&file.path).copied().unwrap_or(0),
                is_orphan: orphan_set.contains(&file.path),
                is_boundary_truncated: boundary_truncated_set.contains(&file.path),
                is_runtime_entry: runtime_entry_set.contains(&file.path),
                boundary_truth: boundary_truth_label(analysis).to_string(),
            }
        })
        .collect::<Vec<_>>();
    hotspots.sort_by(|left, right| {
        right
            .finding_count
            .cmp(&left.finding_count)
            .then(
                right
                    .bottleneck_centrality_millis
                    .cmp(&left.bottleneck_centrality_millis),
            )
            .then(
                (right.inbound_edges + right.outbound_edges)
                    .cmp(&(left.inbound_edges + left.outbound_edges)),
            )
            .then(left.file_path.cmp(&right.file_path))
    });
    hotspots.truncate(20);
    trace_surface_step(
        "hotspots.build",
        hotspots_started_at.elapsed().as_millis(),
        Some(format!("count={}", hotspots.len())),
    );

    let atlas_nodes_started_at = Instant::now();
    let atlas_nodes = analysis
        .semantic_graph
        .files
        .iter()
        .map(|file| {
            let (inbound_edges, outbound_edges) =
                inbound_outbound.get(&file.path).copied().unwrap_or((0, 0));
            AtlasNode {
                file_path: file.path.clone(),
                language: language_label(file.language),
                inbound_edges,
                outbound_edges,
                finding_count: finding_counts.get(&file.path).copied().unwrap_or(0),
                bottleneck_centrality_millis: bottlenecks.get(&file.path).copied().unwrap_or(0),
                is_orphan: orphan_set.contains(&file.path),
                is_boundary_truncated: boundary_truncated_set.contains(&file.path),
                is_runtime_entry: runtime_entry_set.contains(&file.path),
                boundary_truth: boundary_truth_label(analysis).to_string(),
            }
        })
        .collect::<Vec<_>>();
    trace_surface_step(
        "atlas.nodes",
        atlas_nodes_started_at.elapsed().as_millis(),
        Some(format!("count={}", atlas_nodes.len())),
    );
    let atlas_edges_started_at = Instant::now();
    let atlas_edges = build_atlas_edges(analysis);
    trace_surface_step(
        "atlas.edges",
        atlas_edges_started_at.elapsed().as_millis(),
        Some(format!("count={}", atlas_edges.len())),
    );
    let highlights_started_at = Instant::now();
    let highlights = build_highlights(analysis, &context);
    trace_surface_step(
        "highlights.build",
        highlights_started_at.elapsed().as_millis(),
        Some(format!("count={}", highlights.len())),
    );
    let ast_grep_family_counts = analysis.ast_grep_scan.family_counts();

    ArchitectureSurface {
        root: analysis.root.clone(),
        overview: SurfaceOverview {
            scanned_files: analysis.scan.files.len(),
            analyzed_files: analysis.semantic_graph.files.len(),
            symbols: analysis.semantic_graph.symbols.len(),
            references: analysis.semantic_graph.references.len(),
            resolved_edges: analysis.semantic_graph.resolved_edges.len(),
            unresolved_reference_sites: unresolved_reference_sites(analysis),
            strong_cycle_count: analysis.graph_analysis.strong_circular_dependencies.len(),
            total_cycle_count: analysis.graph_analysis.circular_dependencies.len(),
            bottleneck_count: analysis.graph_analysis.bottleneck_files.len(),
            orphan_count: orphan_files.len(),
            boundary_truncated_count: boundary_truncated_files.len(),
            runtime_entry_count: runtime_entry_set.len(),
            boundary_truth: boundary_truth_label(analysis).to_string(),
            boundary_reasons: boundary_reason_labels(analysis),
            dead_code_count: analysis.dead_code.findings.len(),
            hardwiring_count: analysis.hardwiring.findings.len(),
            security_finding_count: analysis.security_analysis.findings.len(),
            external_finding_count: analysis.external_analysis.findings.len(),
            external_tool_run_count: analysis.external_analysis.tool_runs.len(),
            override_edge_count: analysis.graph_analysis.override_edges,
            architectural_smell_count: analysis.graph_analysis.architectural_smells.len(),
            hub_like_dependency_count: analysis
                .graph_analysis
                .architectural_smells
                .iter()
                .filter(|smell| smell.kind == ArchitecturalSmellKind::HubLikeDependency)
                .count(),
            unstable_dependency_count: analysis
                .graph_analysis
                .architectural_smells
                .iter()
                .filter(|smell| smell.kind == ArchitecturalSmellKind::UnstableDependency)
                .count(),
            warning_heavy_hotspot_count: analysis
                .architectural_assessment
                .count_by_kind(ArchitecturalAssessmentKind::WarningHeavyHotspot),
            split_identity_model_count: analysis
                .architectural_assessment
                .count_by_kind(ArchitecturalAssessmentKind::SplitIdentityModel),
            compatibility_scar_count: analysis
                .architectural_assessment
                .count_by_kind(ArchitecturalAssessmentKind::CompatibilityScar),
            duplicate_mechanism_count: analysis
                .architectural_assessment
                .count_by_kind(ArchitecturalAssessmentKind::DuplicateMechanism),
            sanctioned_path_bypass_count: analysis
                .architectural_assessment
                .count_by_kind(ArchitecturalAssessmentKind::SanctionedPathBypass),
            hand_rolled_parsing_count: analysis
                .architectural_assessment
                .count_by_kind(ArchitecturalAssessmentKind::HandRolledParsing),
            abstraction_sprawl_count: analysis
                .architectural_assessment
                .count_by_kind(ArchitecturalAssessmentKind::AbstractionSprawl),
            algorithmic_complexity_hotspot_count: analysis
                .architectural_assessment
                .count_by_kind(ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot),
            ast_grep_finding_count: analysis.ast_grep_scan.findings.len(),
            ast_grep_algorithmic_complexity_count: ast_grep_family_counts.algorithmic_complexity,
            ast_grep_security_dangerous_api_count: ast_grep_family_counts.security_dangerous_api,
            ast_grep_framework_misuse_count: ast_grep_family_counts.framework_misuse,
            route_contract_count: analysis.contract_inventory.summary.routes.unique_values,
            hook_contract_count: analysis.contract_inventory.summary.hooks.unique_values,
            registered_key_count: analysis
                .contract_inventory
                .summary
                .registered_keys
                .unique_values,
            symbolic_literal_count: analysis
                .contract_inventory
                .summary
                .symbolic_literals
                .unique_values,
            env_key_count: analysis.contract_inventory.summary.env_keys.unique_values,
            config_key_count: analysis
                .contract_inventory
                .summary
                .config_keys
                .unique_values,
        },
        contract_inventory: analysis.contract_inventory.clone(),
        languages: language_coverage(
            &analysis.scan.files,
            &analysis.semantic_graph.files,
            &file_languages,
        ),
        hotspots,
        cycle_findings: analysis
            .graph_analysis
            .strong_cycle_findings
            .iter()
            .enumerate()
            .map(|(index, cycle)| surface_cycle_finding(index, cycle))
            .collect(),
        highlights,
        atlas: RepositoryAtlas {
            nodes: atlas_nodes,
            edges: atlas_edges,
        },
        timings: analysis.timings.clone(),
    }
}

fn trace_surface_step(step: &str, elapsed_ms: u128, extra: Option<String>) {
    if std::env::var_os("AIGISCORE_TRACE").is_none() {
        return;
    }
    match extra {
        Some(extra) => eprintln!("[aigiscore] surface {step} elapsed_ms={elapsed_ms} {extra}"),
        None => eprintln!("[aigiscore] surface {step} elapsed_ms={elapsed_ms}"),
    }
}

fn language_coverage(
    scanned_files: &[crate::ingestion::scan::ScannedFile],
    analyzed_files: &[crate::graph::FileNode],
    file_languages: &HashMap<PathBuf, String>,
) -> Vec<LanguageCoverage> {
    let mut counts = HashMap::<String, usize>::new();
    for file in scanned_files {
        let language = file_languages
            .get(&file.relative_path)
            .cloned()
            .or_else(|| path_language_label(&file.relative_path))
            .unwrap_or_else(|| String::from("Unsupported"));
        *counts.entry(language).or_default() += 1;
    }
    for file in analyzed_files {
        counts.entry(language_label(file.language)).or_default();
    }
    let mut coverage = counts
        .into_iter()
        .map(|(language, file_count)| LanguageCoverage {
            language,
            file_count,
        })
        .collect::<Vec<_>>();
    coverage.sort_by(|left, right| {
        right
            .file_count
            .cmp(&left.file_count)
            .then(left.language.cmp(&right.language))
    });
    coverage
}

fn collect_file_languages(analysis: &ProjectAnalysis) -> HashMap<PathBuf, String> {
    analysis
        .semantic_graph
        .files
        .iter()
        .map(|file| (file.path.clone(), language_label(file.language)))
        .collect()
}

fn collect_inbound_outbound_edges(analysis: &ProjectAnalysis) -> HashMap<PathBuf, (usize, usize)> {
    let mut counts = HashMap::<PathBuf, (usize, usize)>::new();
    for edge in &analysis.semantic_graph.resolved_edges {
        counts
            .entry(edge.source_file_path.clone())
            .or_insert((0, 0))
            .1 += 1;
        counts
            .entry(edge.target_file_path.clone())
            .or_insert((0, 0))
            .0 += 1;
    }
    counts
}

fn collect_finding_counts(analysis: &ProjectAnalysis) -> HashMap<PathBuf, usize> {
    let mut counts = HashMap::<PathBuf, usize>::new();
    for finding in &analysis.dead_code.findings {
        *counts.entry(finding.file_path.clone()).or_default() += 1;
    }
    for finding in &analysis.hardwiring.findings {
        *counts.entry(finding.file_path.clone()).or_default() += 1;
    }
    for finding in &analysis.security_analysis.findings {
        *counts.entry(finding.file_path.clone()).or_default() += 1;
    }
    for finding in &analysis.external_analysis.findings {
        if let Some(file_path) = &finding.file_path {
            *counts.entry(file_path.clone()).or_default() += 1;
        }
    }
    for path in effective_orphan_files(analysis) {
        *counts.entry(path.clone()).or_default() += 1;
    }
    for file in &analysis.graph_analysis.bottleneck_files {
        *counts.entry(file.file_path.clone()).or_default() += 1;
    }
    counts
}

fn unresolved_reference_sites(analysis: &ProjectAnalysis) -> usize {
    let resolved_sites = analysis
        .semantic_graph
        .resolved_edges
        .iter()
        .map(|edge| {
            (
                edge.source_file_path.clone(),
                edge.kind,
                edge.line,
                edge.source_symbol_id.clone(),
            )
        })
        .collect::<HashSet<_>>();
    analysis
        .semantic_graph
        .references
        .iter()
        .filter(|reference| {
            !resolved_sites.contains(&(
                reference.file_path.clone(),
                reference.kind,
                reference.line,
                reference.enclosing_symbol_id.clone(),
            ))
        })
        .count()
}

fn build_highlights(
    analysis: &ProjectAnalysis,
    context: &SurfaceBuildContext<'_>,
) -> Vec<SurfaceFinding> {
    let mut highlights = Vec::new();
    let orphan_files = effective_orphan_files(analysis);

    for (index, cycle) in analysis
        .graph_analysis
        .strong_cycle_findings
        .iter()
        .enumerate()
    {
        highlights.push(SurfaceFinding {
            id: format!("graph:cycle:{index}"),
            fingerprint: cycle.fingerprint.clone(),
            family: SurfaceFindingFamily::Graph,
            severity: SurfaceFindingSeverity::High,
            precision: String::from("modeled"),
            confidence_millis: 780,
            title: format!(
                "{} cycle across {} files",
                cycle_class_label(cycle.cycle_class),
                cycle.files.len()
            ),
            summary: format!(
                "{}; layers: {}; dominant relations: {}",
                cycle
                    .files
                    .iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(" -> "),
                cycle
                    .layers
                    .iter()
                    .map(graph_layer_label)
                    .collect::<Vec<_>>()
                    .join(", "),
                cycle
                    .dominant_relations
                    .iter()
                    .map(relation_kind_label)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            file_paths: cycle.files.clone(),
            line: None,
            primary_anchor: cycle
                .files
                .first()
                .cloned()
                .map(|file_path| EvidenceAnchor {
                    file_path,
                    line: None,
                    label: String::from("primary"),
                }),
            evidence_anchors: cycle
                .files
                .iter()
                .skip(1)
                .cloned()
                .map(|file_path| EvidenceAnchor {
                    file_path,
                    line: None,
                    label: String::from("cycle-member"),
                })
                .collect(),
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![String::from("structural.coherence")],
        });
    }

    for path in &orphan_files {
        highlights.push(SurfaceFinding {
            id: format!("graph:orphan:{}", path.display()),
            fingerprint: stable_fingerprint(&["graph", "orphan", &normalized_path(path)]),
            family: SurfaceFindingFamily::Graph,
            severity: SurfaceFindingSeverity::Medium,
            precision: String::from("exact"),
            confidence_millis: 900,
            title: String::from("Orphan file"),
            summary: format!(
                "{} has outbound dependencies but no inbound edges",
                path.display()
            ),
            file_paths: vec![path.clone()],
            line: None,
            primary_anchor: Some(EvidenceAnchor {
                file_path: path.clone(),
                line: None,
                label: String::from("primary"),
            }),
            evidence_anchors: Vec::new(),
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![String::from("structural.coherence")],
        });
    }

    for path in &effective_boundary_truncated_files(analysis) {
        highlights.push(SurfaceFinding {
            id: format!("graph:boundary-truncated:{}", path.display()),
            fingerprint: stable_fingerprint(&[
                "graph",
                "boundary-truncated",
                &normalized_path(path),
            ]),
            family: SurfaceFindingFamily::Graph,
            severity: SurfaceFindingSeverity::Low,
            precision: String::from("exact"),
            confidence_millis: 900,
            title: String::from("Boundary-truncated file"),
            summary: format!(
                "{} has outbound dependencies but no inbound edges inside the current scoped analysis",
                path.display()
            ),
            file_paths: vec![path.clone()],
            line: None,
            primary_anchor: Some(EvidenceAnchor {
                file_path: path.clone(),
                line: None,
                label: String::from("primary"),
            }),
            evidence_anchors: Vec::new(),
            provenance: vec![String::from("graph_analysis"), String::from("analysis_scope")],
            doctrine_refs: vec![String::from("guardian.boundary-stability")],
        });
    }

    for bottleneck in analysis.graph_analysis.bottleneck_files.iter().take(10) {
        highlights.push(surface_finding_from_bottleneck(
            bottleneck, analysis, context,
        ));
    }

    for smell in &analysis.graph_analysis.architectural_smells {
        highlights.push(surface_finding_from_architectural_smell(
            smell, analysis, context,
        ));
    }

    for finding in &analysis.architectural_assessment.findings {
        highlights.push(surface_finding_from_architectural_assessment(
            finding, analysis, context,
        ));
    }

    for finding in &analysis.dead_code.findings {
        highlights.push(surface_finding_from_dead_code(finding));
    }

    for finding in &analysis.hardwiring.findings {
        highlights.push(surface_finding_from_hardwiring(finding));
    }
    for finding in &analysis.security_analysis.findings {
        highlights.push(surface_finding_from_security(finding));
    }
    for finding in &analysis.external_analysis.findings {
        highlights.push(surface_finding_from_external(finding));
    }

    highlights.sort_by(|left, right| {
        severity_rank(right.severity)
            .cmp(&severity_rank(left.severity))
            .then(family_rank(left.family).cmp(&family_rank(right.family)))
            .then(left.file_paths.cmp(&right.file_paths))
            .then(left.line.cmp(&right.line))
            .then(left.id.cmp(&right.id))
    });
    highlights
}

fn boundary_truth_label(analysis: &ProjectAnalysis) -> &'static str {
    match analysis.scan.scope.boundary_truth {
        crate::ingestion::scan::AnalysisBoundaryTruth::CompleteRepository => "complete_repository",
        crate::ingestion::scan::AnalysisBoundaryTruth::TruncatedSlice => "truncated_slice",
    }
}

fn boundary_reason_labels(analysis: &ProjectAnalysis) -> Vec<String> {
    analysis
        .scan
        .scope
        .reasons
        .iter()
        .map(|reason| match reason {
            crate::ingestion::scan::AnalysisBoundaryReason::CroppedRoot => "cropped_root",
            crate::ingestion::scan::AnalysisBoundaryReason::IncludePathPrefixes => {
                "include_path_prefixes"
            }
        })
        .map(String::from)
        .collect()
}

fn surface_finding_from_architectural_assessment(
    finding: &ArchitecturalAssessmentFinding,
    analysis: &ProjectAnalysis,
    context: &SurfaceBuildContext<'_>,
) -> SurfaceFinding {
    let primary_anchor =
        best_effort_anchor_for_architectural_assessment(finding, analysis, context);
    let evidence_anchors =
        supporting_anchors_for_architectural_assessment(finding, analysis, context);
    let primary_line = primary_anchor.as_ref().and_then(|anchor| anchor.line);
    match finding.kind {
        ArchitecturalAssessmentKind::WarningHeavyHotspot => SurfaceFinding {
            id: format!("architecture:hotspot:{}", finding.file_path.display()),
            fingerprint: finding.fingerprint.clone(),
            family: SurfaceFindingFamily::Graph,
            severity: SurfaceFindingSeverity::High,
            precision: String::from("modeled"),
            confidence_millis: finding.severity_millis,
            title: String::from("Warning-heavy hotspot"),
            summary: format!(
                "{} is both architecturally central ({}) and accumulating warnings (count {}, weight {}, families: {})",
                finding.file_path.display(),
                finding.bottleneck_centrality_millis,
                finding.warning_count,
                finding.warning_weight,
                finding.warning_families.join(", ")
            ),
            file_paths: vec![finding.file_path.clone()],
            line: primary_line,
            primary_anchor,
            evidence_anchors,
            provenance: vec![
                String::from("graph_analysis"),
                String::from("architectural_assessment"),
            ],
            doctrine_refs: vec![
                String::from("structural.coherence"),
                String::from("guardian.centralized-damage"),
            ],
        },
        ArchitecturalAssessmentKind::SplitIdentityModel => {
            let mut file_paths = vec![finding.file_path.clone()];
            file_paths.extend(finding.related_file_paths.clone());
            SurfaceFinding {
                id: format!(
                    "architecture:split-identity:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                ),
                fingerprint: finding.fingerprint.clone(),
                family: SurfaceFindingFamily::Graph,
                severity: SurfaceFindingSeverity::Medium,
                precision: String::from("heuristic"),
                confidence_millis: finding.severity_millis,
                title: String::from("Split identity model"),
                summary: format!(
                    "{} mixes object-like and scalar identity forms for the same concept (identifiers: {})",
                    finding.file_path.display(),
                    finding.related_identifiers.join(", ")
                ),
                file_paths,
                line: primary_line,
                primary_anchor,
                evidence_anchors,
                provenance: vec![
                    String::from("architectural_assessment"),
                    String::from("parsed_sources"),
                ],
                doctrine_refs: vec![
                    String::from("pattern.coherence"),
                    String::from("guardian.single-canonical-representation"),
                ],
            }
        }
        ArchitecturalAssessmentKind::CompatibilityScar => {
            let mut file_paths = vec![finding.file_path.clone()];
            file_paths.extend(finding.related_file_paths.clone());
            SurfaceFinding {
                id: format!(
                    "architecture:compatibility-scar:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                ),
                fingerprint: finding.fingerprint.clone(),
                family: SurfaceFindingFamily::Graph,
                severity: SurfaceFindingSeverity::High,
                precision: String::from("heuristic"),
                confidence_millis: finding.severity_millis,
                title: String::from("Compatibility scar"),
                summary: format!(
                    "{} centralizes translation or compatibility glue for competing representations (identifiers: {})",
                    finding.file_path.display(),
                    finding.related_identifiers.join(", ")
                ),
                file_paths,
                line: primary_line,
                primary_anchor,
                evidence_anchors,
                provenance: vec![
                    String::from("architectural_assessment"),
                    String::from("parsed_sources"),
                    String::from("graph_analysis"),
                ],
                doctrine_refs: vec![
                    String::from("pattern.coherence"),
                    String::from("guardian.single-canonical-representation"),
                    String::from("guardian.translation-hotspot"),
                ],
            }
        }
        ArchitecturalAssessmentKind::DuplicateMechanism => {
            let mut file_paths = vec![finding.file_path.clone()];
            file_paths.extend(finding.related_file_paths.clone());
            SurfaceFinding {
                id: format!(
                    "architecture:duplicate-mechanism:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                ),
                fingerprint: finding.fingerprint.clone(),
                family: SurfaceFindingFamily::Graph,
                severity: SurfaceFindingSeverity::High,
                precision: String::from("heuristic"),
                confidence_millis: finding.severity_millis,
                title: String::from("Duplicate mechanism"),
                summary: format!(
                    "{} appears to route the same concern through multiple orchestration mechanisms ({})",
                    finding.file_path.display(),
                    finding.warning_families.join(", ")
                ),
                file_paths,
                line: primary_line,
                primary_anchor,
                evidence_anchors,
                provenance: vec![
                    String::from("architectural_assessment"),
                    String::from("parsed_sources"),
                    String::from("graph_analysis"),
                ],
                doctrine_refs: vec![
                    String::from("mechanism.coherence"),
                    String::from("guardian.single-solution-path"),
                ],
            }
        }
        ArchitecturalAssessmentKind::SanctionedPathBypass => {
            let mut file_paths = vec![finding.file_path.clone()];
            file_paths.extend(finding.related_file_paths.clone());
            SurfaceFinding {
                id: format!(
                    "architecture:sanctioned-path-bypass:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                ),
                fingerprint: finding.fingerprint.clone(),
                family: SurfaceFindingFamily::Graph,
                severity: SurfaceFindingSeverity::High,
                precision: String::from("heuristic"),
                confidence_millis: finding.severity_millis,
                title: String::from("Sanctioned path bypass"),
                summary: format!(
                    "{} mixes raw environment access with sanctioned configuration access ({})",
                    finding.file_path.display(),
                    finding.related_identifiers.join(", ")
                ),
                file_paths,
                line: primary_line,
                primary_anchor,
                evidence_anchors,
                provenance: vec![
                    String::from("architectural_assessment"),
                    String::from("hardwiring_detector"),
                    String::from("parsed_sources"),
                ],
                doctrine_refs: vec![
                    String::from("configuration.coherence"),
                    String::from("guardian.sanctioned-paths"),
                ],
            }
        }
        ArchitecturalAssessmentKind::AbstractionSprawl => {
            let mut file_paths = vec![finding.file_path.clone()];
            file_paths.extend(finding.related_file_paths.clone());
            SurfaceFinding {
                id: format!(
                    "architecture:abstraction-sprawl:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                ),
                fingerprint: finding.fingerprint.clone(),
                family: SurfaceFindingFamily::Graph,
                severity: SurfaceFindingSeverity::High,
                precision: String::from("heuristic"),
                confidence_millis: finding.severity_millis,
                title: String::from("Abstraction sprawl"),
                summary: format!(
                    "{} spreads one concern across too many abstraction roles ({})",
                    finding.file_path.display(),
                    finding.warning_families.join(", ")
                ),
                file_paths,
                line: primary_line,
                primary_anchor,
                evidence_anchors,
                provenance: vec![
                    String::from("architectural_assessment"),
                    String::from("parsed_sources"),
                    String::from("graph_analysis"),
                ],
                doctrine_refs: vec![
                    String::from("mechanism.coherence"),
                    String::from("guardian.minimal-mechanism"),
                    String::from("guardian.overengineering"),
                ],
            }
        }
        ArchitecturalAssessmentKind::HandRolledParsing => {
            let mut file_paths = vec![finding.file_path.clone()];
            file_paths.extend(finding.related_file_paths.clone());
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
            let is_filesystem_page_resolution = finding
                .warning_families
                .iter()
                .any(|family| family == "concern:filesystem_page_resolution");
            let is_manifest_backed_policy_engine = finding
                .warning_families
                .iter()
                .any(|family| family == "concern:manifest_backed_policy_engine");
            SurfaceFinding {
                id: format!(
                    "architecture:hand-rolled-parsing:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                ),
                fingerprint: finding.fingerprint.clone(),
                family: SurfaceFindingFamily::Graph,
                severity: SurfaceFindingSeverity::High,
                precision: String::from("heuristic"),
                confidence_millis: finding.severity_millis,
                title: String::from(if is_manifest_backed_policy_engine {
                    "Manifest-backed policy engine"
                } else if is_filesystem_page_resolution {
                    "Filesystem page resolution layer"
                } else if is_schema_validation {
                    "Homegrown schema validation"
                } else if is_scheduler_dsl {
                    "Homegrown scheduler DSL"
                } else if is_definition_engine {
                    "Homegrown definition engine"
                } else {
                    "Hand-rolled parsing"
                }),
                summary: format!(
                    "{} appears to own a custom {} ({})",
                    finding.file_path.display(),
                    if is_manifest_backed_policy_engine {
                        "manifest-backed template policy/runtime engine"
                    } else if is_filesystem_page_resolution {
                        "filesystem-backed page or route resolution layer"
                    } else if is_schema_validation {
                        "schema or validation contract stack"
                    } else if is_scheduler_dsl {
                        "scheduler or job-definition DSL"
                    } else if is_definition_engine {
                        "definition or metadata engine"
                    } else if is_contract_stack {
                        "contract stack"
                    } else {
                        "parsing or mini-protocol stack"
                    },
                    finding.warning_families.join(", ")
                ),
                file_paths,
                line: primary_line,
                primary_anchor,
                evidence_anchors,
                provenance: vec![
                    String::from("architectural_assessment"),
                    String::from("parsed_sources"),
                    String::from("graph_analysis"),
                ],
                doctrine_refs: vec![
                    String::from("pattern.coherence"),
                    String::from(if is_manifest_backed_policy_engine {
                        "guardian.avoid-manifest-backed-policy-engine"
                    } else if is_filesystem_page_resolution {
                        "guardian.avoid-filesystem-page-resolution"
                    } else if is_schema_validation {
                        "guardian.avoid-homegrown-schema-validation"
                    } else if is_scheduler_dsl {
                        "guardian.avoid-homegrown-scheduler-dsl"
                    } else if is_definition_engine {
                        "guardian.avoid-homegrown-definition-engine"
                    } else {
                        "guardian.avoid-homegrown-parser"
                    }),
                    String::from("guardian.native-vs-library"),
                ],
            }
        }
        ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot => {
            let mut file_paths = vec![finding.file_path.clone()];
            file_paths.extend(finding.related_file_paths.clone());
            SurfaceFinding {
                id: format!(
                    "architecture:algorithmic-complexity:{}:{}",
                    finding.file_path.display(),
                    finding.related_identifiers.join("+")
                ),
                fingerprint: finding.fingerprint.clone(),
                family: SurfaceFindingFamily::Graph,
                severity: SurfaceFindingSeverity::High,
                precision: String::from("heuristic"),
                confidence_millis: finding.severity_millis,
                title: String::from("Algorithmic complexity hotspot"),
                summary: format!(
                    "{} contains repeated expensive loop-local work ({}) that is likely to become a scaling hotspot",
                    finding.file_path.display(),
                    finding.warning_families.join(", ")
                ),
                file_paths,
                line: primary_line,
                primary_anchor,
                evidence_anchors,
                provenance: vec![
                    String::from("architectural_assessment"),
                    String::from("parsed_sources"),
                ],
                doctrine_refs: vec![
                    String::from("performance.scaling"),
                    String::from("guardian.superlinear-risk"),
                ],
            }
        }
    }
}

fn surface_finding_from_architectural_smell(
    smell: &ArchitecturalSmell,
    analysis: &ProjectAnalysis,
    context: &SurfaceBuildContext<'_>,
) -> SurfaceFinding {
    let primary_anchor =
        best_effort_anchor_for_component(&smell.subject, analysis, context, "primary");
    let evidence_anchors = smell
        .related_components
        .iter()
        .filter_map(|component| {
            best_effort_anchor_for_component(component, analysis, context, "supporting")
        })
        .collect::<Vec<_>>();
    let file_paths =
        architectural_smell_file_paths(smell, context, primary_anchor.as_ref(), &evidence_anchors);
    match smell.kind {
        ArchitecturalSmellKind::HubLikeDependency => SurfaceFinding {
            id: format!("graph:smell:hub:{}", smell.subject),
            fingerprint: smell.fingerprint.clone(),
            family: SurfaceFindingFamily::Graph,
            severity: SurfaceFindingSeverity::High,
            precision: String::from("modeled"),
            confidence_millis: 820,
            title: String::from("Hub-like dependency"),
            summary: format!(
                "Module `{}` has high incoming and outgoing dependency pressure ({} cross-module links)",
                smell.subject, smell.evidence_count
            ),
            file_paths,
            line: primary_anchor.as_ref().and_then(|anchor| anchor.line),
            primary_anchor,
            evidence_anchors,
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![
                String::from("structural.coherence"),
                String::from("guardian.centralized-damage"),
            ],
        },
        ArchitecturalSmellKind::UnstableDependency => SurfaceFinding {
            id: format!(
                "graph:smell:unstable:{}:{}",
                smell.subject,
                smell.related_components.join("+")
            ),
            fingerprint: smell.fingerprint.clone(),
            family: SurfaceFindingFamily::Graph,
            severity: SurfaceFindingSeverity::High,
            precision: String::from("modeled"),
            confidence_millis: 780,
            title: String::from("Unstable dependency"),
            summary: format!(
                "More stable module `{}` depends on more volatile module(s): {}",
                smell.subject,
                smell.related_components.join(", ")
            ),
            file_paths,
            line: primary_anchor.as_ref().and_then(|anchor| anchor.line),
            primary_anchor,
            evidence_anchors,
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![
                String::from("structural.coherence"),
                String::from("guardian.boundary-stability"),
            ],
        },
    }
}

fn architectural_smell_file_paths(
    smell: &ArchitecturalSmell,
    context: &SurfaceBuildContext<'_>,
    primary_anchor: Option<&EvidenceAnchor>,
    evidence_anchors: &[EvidenceAnchor],
) -> Vec<PathBuf> {
    let mut file_paths = Vec::new();
    let mut push_unique = |path: PathBuf| {
        if !file_paths.contains(&path) {
            file_paths.push(path);
        }
    };

    if let Some(anchor) = primary_anchor {
        push_unique(anchor.file_path.clone());
    }
    if let Some(path) = resolve_component_path(&smell.subject, context) {
        push_unique(path);
    }
    for anchor in evidence_anchors {
        push_unique(anchor.file_path.clone());
    }
    for component in &smell.related_components {
        if let Some(path) = resolve_component_path(component, context) {
            push_unique(path);
        }
    }

    file_paths
}

fn surface_cycle_finding(index: usize, finding: &CycleFinding) -> SurfaceCycleFinding {
    SurfaceCycleFinding {
        id: format!("graph:cycle:{index}"),
        fingerprint: finding.fingerprint.clone(),
        cycle_class: cycle_class_label(finding.cycle_class).to_owned(),
        files: finding.files.clone(),
        layers: finding
            .layers
            .iter()
            .map(|layer| graph_layer_label(layer).to_owned())
            .collect(),
        dominant_relations: finding
            .dominant_relations
            .iter()
            .map(|relation| relation_kind_label(relation).to_owned())
            .collect(),
        edge_count: finding.edge_count,
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

fn relation_kind_label(relation: &RelationKind) -> &'static str {
    match relation {
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

fn surface_finding_from_bottleneck(
    bottleneck: &BottleneckFile,
    analysis: &ProjectAnalysis,
    context: &SurfaceBuildContext<'_>,
) -> SurfaceFinding {
    let primary_anchor =
        best_effort_anchor_for_file(&bottleneck.file_path, analysis, context, "primary");
    SurfaceFinding {
        id: format!("graph:bottleneck:{}", bottleneck.file_path.display()),
        fingerprint: bottleneck.fingerprint.clone(),
        family: SurfaceFindingFamily::Graph,
        severity: SurfaceFindingSeverity::Medium,
        precision: String::from("modeled"),
        confidence_millis: 760,
        title: String::from("Bottleneck file"),
        summary: format!(
            "{} concentrates dependency flow (centrality {})",
            bottleneck.file_path.display(),
            bottleneck.centrality_millis
        ),
        file_paths: vec![bottleneck.file_path.clone()],
        line: primary_anchor.as_ref().and_then(|anchor| anchor.line),
        primary_anchor,
        evidence_anchors: Vec::new(),
        provenance: vec![String::from("graph_analysis")],
        doctrine_refs: vec![
            String::from("structural.coherence"),
            String::from("guardian.centralized-damage"),
        ],
    }
}

fn surface_finding_from_dead_code(finding: &DeadCodeFinding) -> SurfaceFinding {
    let (precision, confidence_millis) = match finding.proof_tier {
        DeadCodeProofTier::Certain => ("exact", 980),
        DeadCodeProofTier::Strong => ("modeled", 900),
        DeadCodeProofTier::Heuristic => ("heuristic", 700),
    };
    SurfaceFinding {
        id: format!(
            "dead-code:{}:{}:{}",
            finding.file_path.display(),
            finding.line,
            finding.name
        ),
        fingerprint: finding.fingerprint.clone(),
        family: SurfaceFindingFamily::DeadCode,
        severity: SurfaceFindingSeverity::Medium,
        precision: String::from(precision),
        confidence_millis,
        title: match finding.category {
            DeadCodeCategory::UnusedPrivateFunction => String::from("Unused private function"),
            DeadCodeCategory::UnusedImport => String::from("Unused import"),
        },
        summary: format!(
            "{} appears unused ({})",
            finding.name,
            dead_code_proof_tier_label(finding.proof_tier)
        ),
        file_paths: vec![finding.file_path.clone()],
        line: Some(finding.line),
        primary_anchor: Some(anchor(&finding.file_path, Some(finding.line), "primary")),
        evidence_anchors: Vec::new(),
        provenance: vec![
            String::from("dead_code_detector"),
            String::from("graph_analysis"),
        ],
        doctrine_refs: vec![String::from("maintainability.remove-dead-code")],
    }
}

fn dead_code_proof_tier_label(tier: DeadCodeProofTier) -> &'static str {
    match tier {
        DeadCodeProofTier::Certain => "certain",
        DeadCodeProofTier::Strong => "strong",
        DeadCodeProofTier::Heuristic => "heuristic",
    }
}

fn surface_finding_from_hardwiring(finding: &HardwiringFinding) -> SurfaceFinding {
    SurfaceFinding {
        id: format!(
            "hardwiring:{}:{}:{}",
            finding.file_path.display(),
            finding.line,
            finding.value
        ),
        fingerprint: finding.fingerprint.clone(),
        family: SurfaceFindingFamily::Hardwiring,
        severity: hardwiring_severity(finding.category),
        precision: String::from("heuristic"),
        confidence_millis: match finding.category {
            HardwiringCategory::HardcodedNetwork | HardwiringCategory::EnvOutsideConfig => 760,
            HardwiringCategory::MagicString | HardwiringCategory::RepeatedLiteral => 620,
        },
        title: match finding.category {
            HardwiringCategory::MagicString => String::from("Magic string"),
            HardwiringCategory::RepeatedLiteral => String::from("Repeated literal"),
            HardwiringCategory::HardcodedNetwork => String::from("Hardcoded network"),
            HardwiringCategory::EnvOutsideConfig => {
                String::from("Environment access outside config")
            }
        },
        summary: format!("{} in `{}`", finding.value, finding.context),
        file_paths: vec![finding.file_path.clone()],
        line: Some(finding.line),
        primary_anchor: Some(anchor(&finding.file_path, Some(finding.line), "primary")),
        evidence_anchors: Vec::new(),
        provenance: vec![String::from("hardwiring_detector")],
        doctrine_refs: vec![String::from("configuration.coherence")],
    }
}

fn hardwiring_severity(category: HardwiringCategory) -> SurfaceFindingSeverity {
    match category {
        HardwiringCategory::HardcodedNetwork | HardwiringCategory::EnvOutsideConfig => {
            SurfaceFindingSeverity::Medium
        }
        HardwiringCategory::MagicString | HardwiringCategory::RepeatedLiteral => {
            SurfaceFindingSeverity::Low
        }
    }
}

fn surface_finding_from_external(finding: &ExternalFinding) -> SurfaceFinding {
    let file_paths = finding
        .file_path
        .clone()
        .map(|path| vec![path])
        .unwrap_or_default();
    let title = format!("{} {}", finding.tool, finding.category.replace('_', " "));
    SurfaceFinding {
        id: format!("external:{}:{}", finding.tool, finding.fingerprint),
        fingerprint: finding.fingerprint.clone(),
        family: SurfaceFindingFamily::External,
        severity: external_severity(finding.severity),
        precision: String::from("imported"),
        confidence_millis: match finding.confidence {
            crate::external::ExternalConfidence::High => 900,
            crate::external::ExternalConfidence::Medium => 700,
            crate::external::ExternalConfidence::Low => 500,
        },
        title,
        summary: finding.message.clone(),
        file_paths,
        line: finding.line,
        primary_anchor: finding
            .file_path
            .as_ref()
            .map(|path| anchor(path, finding.line, "primary")),
        evidence_anchors: Vec::new(),
        provenance: vec![
            String::from("external_tool"),
            format!("tool:{}", finding.tool),
        ],
        doctrine_refs: vec![String::from("security.external-evidence")],
    }
}

fn surface_finding_from_security(finding: &SecurityFinding) -> SurfaceFinding {
    SurfaceFinding {
        id: format!("security:native:{}", finding.fingerprint),
        fingerprint: finding.fingerprint.clone(),
        family: SurfaceFindingFamily::Security,
        severity: security_severity(finding.severity),
        precision: String::from("modeled"),
        confidence_millis: 840,
        title: match finding.category {
            SecurityCategory::CommandExecution => String::from("Dangerous command execution API"),
            SecurityCategory::CodeInjection => String::from("Dangerous code evaluation API"),
            SecurityCategory::UnsafeDeserialization => String::from("Unsafe deserialization API"),
            SecurityCategory::UnsafeHtmlOutput => String::from("Unsafe HTML output API"),
        },
        summary: finding.message.clone(),
        file_paths: vec![finding.file_path.clone()],
        line: Some(finding.line),
        primary_anchor: Some(anchor(&finding.file_path, Some(finding.line), "primary")),
        evidence_anchors: Vec::new(),
        provenance: vec![
            String::from("native_security"),
            String::from("contract_inventory"),
        ],
        doctrine_refs: vec![
            String::from("security.coherence"),
            String::from("guardian.trust-boundaries"),
        ],
    }
}

fn security_severity(severity: SecuritySeverity) -> SurfaceFindingSeverity {
    match severity {
        SecuritySeverity::High => SurfaceFindingSeverity::High,
        SecuritySeverity::Medium => SurfaceFindingSeverity::Medium,
        SecuritySeverity::Low => SurfaceFindingSeverity::Low,
    }
}

fn external_severity(severity: ExternalSeverity) -> SurfaceFindingSeverity {
    match severity {
        ExternalSeverity::High => SurfaceFindingSeverity::High,
        ExternalSeverity::Medium => SurfaceFindingSeverity::Medium,
        ExternalSeverity::Low => SurfaceFindingSeverity::Low,
    }
}

fn anchor(file_path: &Path, line: Option<usize>, label: &str) -> EvidenceAnchor {
    EvidenceAnchor {
        file_path: file_path.to_path_buf(),
        line,
        label: String::from(label),
    }
}

fn best_effort_anchor_for_architectural_assessment(
    finding: &ArchitecturalAssessmentFinding,
    analysis: &ProjectAnalysis,
    context: &SurfaceBuildContext<'_>,
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
    best_effort_anchor_for_file_with_tokens(
        &finding.file_path,
        analysis,
        context,
        "primary",
        &tokens,
    )
}

fn supporting_anchors_for_architectural_assessment(
    finding: &ArchitecturalAssessmentFinding,
    analysis: &ProjectAnalysis,
    context: &SurfaceBuildContext<'_>,
) -> Vec<EvidenceAnchor> {
    finding
        .related_file_paths
        .iter()
        .filter_map(|path| best_effort_anchor_for_file(path, analysis, context, "supporting"))
        .collect()
}

fn best_effort_anchor_for_component(
    component: &str,
    analysis: &ProjectAnalysis,
    context: &SurfaceBuildContext<'_>,
    label: &str,
) -> Option<EvidenceAnchor> {
    let _ = analysis;
    let path = resolve_component_path(component, context)?;
    best_effort_anchor_for_file(&path, analysis, context, label)
}

fn best_effort_anchor_for_file(
    file_path: &Path,
    analysis: &ProjectAnalysis,
    context: &SurfaceBuildContext<'_>,
    label: &str,
) -> Option<EvidenceAnchor> {
    let mut tokens = Vec::new();
    if let Some(stem) = file_path.file_stem().and_then(|stem| stem.to_str()) {
        tokens.push(stem.to_string());
    }
    best_effort_anchor_for_file_with_tokens(file_path, analysis, context, label, &tokens)
}

fn best_effort_anchor_for_file_with_tokens(
    file_path: &Path,
    analysis: &ProjectAnalysis,
    context: &SurfaceBuildContext<'_>,
    label: &str,
    tokens: &[String],
) -> Option<EvidenceAnchor> {
    let _ = analysis;
    let source = context.parsed_source_lookup.get(file_path)?;
    let line = anchor_line_for_content(source, tokens);
    Some(anchor(file_path, line, label))
}

fn anchor_line_for_content(source: &IndexedParsedSource<'_>, tokens: &[String]) -> Option<usize> {
    let lowered_tokens = tokens
        .iter()
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    for (index, lowered) in source.lowered_lines.iter().enumerate() {
        if lowered_tokens.iter().any(|token| lowered.contains(token)) {
            return Some(index + 1);
        }
    }

    source.first_non_empty_line.or_else(|| {
        source
            .content
            .lines()
            .enumerate()
            .find(|(_, line)| !line.trim().is_empty())
            .map(|(index, _)| index + 1)
    })
}

fn build_atlas_edges(analysis: &ProjectAnalysis) -> Vec<AtlasEdge> {
    let mut groups = HashMap::<(PathBuf, PathBuf), AtlasEdgeAccumulator>::new();
    for edge in &analysis.semantic_graph.resolved_edges {
        let key = (edge.source_file_path.clone(), edge.target_file_path.clone());
        let entry = groups.entry(key).or_default();
        entry.edge_count += 1;
        entry.kinds.insert(reference_kind_label(edge.kind));
        entry.strongest_resolution_tier =
            strongest_tier(entry.strongest_resolution_tier, edge.resolution_tier);
        entry.confidence_total += u32::from(edge.confidence_millis);
    }

    let mut edges = groups
        .into_iter()
        .map(|((source_file_path, target_file_path), entry)| {
            let mut kinds = entry.kinds.into_iter().collect::<Vec<_>>();
            kinds.sort();
            AtlasEdge {
                source_file_path,
                target_file_path,
                edge_count: entry.edge_count,
                kinds,
                strongest_resolution_tier: resolution_tier_label(entry.strongest_resolution_tier),
                average_confidence_millis: (entry.confidence_total / entry.edge_count as u32)
                    as u16,
            }
        })
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        left.source_file_path
            .cmp(&right.source_file_path)
            .then(left.target_file_path.cmp(&right.target_file_path))
    });
    edges
}

#[derive(Default)]
struct AtlasEdgeAccumulator {
    edge_count: usize,
    kinds: HashSet<String>,
    strongest_resolution_tier: Option<ResolutionTier>,
    confidence_total: u32,
}

fn strongest_tier(
    existing: Option<ResolutionTier>,
    candidate: ResolutionTier,
) -> Option<ResolutionTier> {
    match existing {
        None => Some(candidate),
        Some(current) if tier_rank(candidate) < tier_rank(current) => Some(candidate),
        Some(current) => Some(current),
    }
}

fn tier_rank(tier: ResolutionTier) -> u8 {
    match tier {
        ResolutionTier::SameFile => 0,
        ResolutionTier::ImportScoped => 1,
        ResolutionTier::Global => 2,
    }
}

fn resolution_tier_label(tier: Option<ResolutionTier>) -> String {
    match tier.unwrap_or(ResolutionTier::Global) {
        ResolutionTier::SameFile => String::from("SameFile"),
        ResolutionTier::ImportScoped => String::from("ImportScoped"),
        ResolutionTier::Global => String::from("Global"),
    }
}

fn severity_rank(severity: SurfaceFindingSeverity) -> u8 {
    match severity {
        SurfaceFindingSeverity::High => 2,
        SurfaceFindingSeverity::Medium => 1,
        SurfaceFindingSeverity::Low => 0,
    }
}

fn family_rank(family: SurfaceFindingFamily) -> u8 {
    match family {
        SurfaceFindingFamily::Graph => 0,
        SurfaceFindingFamily::Security => 1,
        SurfaceFindingFamily::External => 2,
        SurfaceFindingFamily::DeadCode => 3,
        SurfaceFindingFamily::Hardwiring => 4,
    }
}

fn language_label(language: Language) -> String {
    match language {
        Language::JavaScript => String::from("JavaScript"),
        Language::Php => String::from("PHP"),
        Language::Python => String::from("Python"),
        Language::Ruby => String::from("Ruby"),
        Language::Rust => String::from("Rust"),
        Language::TypeScript => String::from("TypeScript"),
    }
}

fn path_language_label(path: &Path) -> Option<String> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => Some(String::from("Rust")),
        Some("js") | Some("jsx") => Some(String::from("JavaScript")),
        Some("ts") | Some("tsx") => Some(String::from("TypeScript")),
        Some("php") | Some("phtml") | Some("php3") | Some("php4") | Some("php5") | Some("php8") => {
            Some(String::from("PHP"))
        }
        Some("py") => Some(String::from("Python")),
        Some("rb") | Some("rake") => Some(String::from("Ruby")),
        _ => None,
    }
}

fn reference_kind_label(kind: ReferenceKind) -> String {
    match kind {
        ReferenceKind::Import => String::from("Import"),
        ReferenceKind::Call => String::from("Call"),
        ReferenceKind::Type => String::from("Type"),
        ReferenceKind::Extends => String::from("Extends"),
        ReferenceKind::Implements => String::from("Implements"),
        ReferenceKind::Overrides => String::from("Overrides"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        architectural_smell_file_paths, best_effort_anchor_for_component,
        build_architecture_surface, surface_finding_from_architectural_smell, SurfaceBuildContext,
    };
    use crate::graph::analysis::{ArchitecturalSmell, ArchitecturalSmellKind};
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builds_an_architecture_surface_from_project_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::create_dir_all(fixture.join("app")).unwrap();

        fs::write(
            fixture.join("src/models.ts"),
            br#"export class User {
  save() {}
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/factory.ts"),
            br#"import { User } from "./models";

export class UserFactory {
  static build(): User {
    return new User();
  }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/service.ts"),
            br#"import { UserFactory } from "./factory";
import { User } from "./models";

function helper() {}
function unused() {}

function run() {
  helper();
  UserFactory.build().save();
  const api = "https://api.example.com";
  const env = process.env.APP_ENV;
  const a = "shared";
  const b = "shared";
  if (env === "prod") {}
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/orphan.py"),
            br#"from src.service import run

def execute():
    run()
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = build_architecture_surface(&analysis);

        assert_eq!(surface.overview.scanned_files, 4);
        assert!(surface.overview.resolved_edges >= 3);
        assert_eq!(surface.overview.route_contract_count, 0);
        assert!(surface
            .languages
            .iter()
            .any(|entry| entry.language == "TypeScript" && entry.file_count == 3));
        assert!(surface
            .hotspots
            .iter()
            .any(|hotspot| hotspot.file_path == Path::new("src/service.ts")));
        assert!(surface
            .atlas
            .edges
            .iter()
            .any(|edge| edge.source_file_path == Path::new("src/service.ts")
                && edge.target_file_path == Path::new("src/factory.ts")));
        assert!(surface
            .highlights
            .iter()
            .all(|finding| !finding.fingerprint.is_empty()));
    }

    #[test]
    fn surface_exposes_contract_inventory_summary() {
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
        let surface = build_architecture_surface(&analysis);

        assert_eq!(surface.overview.route_contract_count, 1);
        assert_eq!(surface.overview.hook_contract_count, 1);
        assert_eq!(surface.overview.env_key_count, 1);
        assert!(surface
            .contract_inventory
            .symbolic_literals
            .iter()
            .any(|item| item.value == "draft"));
    }

    #[test]
    fn cycle_fingerprints_are_stable_across_surface_views() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/a.ts"),
            br#"import { runB } from "./b";
export function runA() { runB(); }
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/b.ts"),
            br#"import { runA } from "./a";
export function runB() { runA(); }
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = build_architecture_surface(&analysis);

        let cycle = surface
            .cycle_findings
            .first()
            .expect("expected cycle finding");
        let highlight = surface
            .highlights
            .iter()
            .find(|finding| finding.id == cycle.id)
            .expect("expected cycle highlight");

        assert_eq!(cycle.fingerprint, highlight.fingerprint);
        assert!(!cycle.fingerprint.is_empty());
        assert_ne!(cycle.fingerprint, cycle.id);
    }

    #[test]
    fn route_declared_php_attribute_controller_is_runtime_entry_not_orphan() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src/Controller")).unwrap();
        fs::write(
            fixture.join("src/Controller/TriggerFlowController.php"),
            br#"<?php
#[Route(path: '/api/_action/trigger-event/{eventName}')]
final class TriggerFlowController {
    public function __invoke(): void {}
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = build_architecture_surface(&analysis);
        let controller_path = Path::new("src/Controller/TriggerFlowController.php");
        let hotspot = surface
            .hotspots
            .iter()
            .find(|hotspot| hotspot.file_path == controller_path)
            .expect("expected controller hotspot");

        assert!(hotspot.is_runtime_entry);
        assert!(!hotspot.is_orphan);
        assert_eq!(surface.overview.route_contract_count, 1);
        assert_eq!(surface.overview.runtime_entry_count, 1);
        assert_eq!(surface.overview.orphan_count, 0);
        assert!(surface
            .highlights
            .iter()
            .all(|finding| finding.id != "graph:orphan:src/Controller/TriggerFlowController.php"));
    }

    #[test]
    fn prefix_filtered_scan_emits_boundary_truncated_truth_instead_of_orphan_debt() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join(".aigiscode/scan.json"),
            br#"{
  "include_path_prefixes": ["src"]
}"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/a.ts"),
            br#"import { runB } from "./b";
export function runA() { runB(); }
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/b.ts"),
            br#"export function runB() {}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = build_architecture_surface(&analysis);

        assert_eq!(surface.overview.boundary_truth, "truncated_slice");
        assert_eq!(surface.overview.boundary_truncated_count, 1);
        assert_eq!(surface.overview.orphan_count, 0);
        let hotspot = surface
            .hotspots
            .iter()
            .find(|hotspot| hotspot.file_path == Path::new("src/a.ts"))
            .expect("expected a.ts hotspot");
        assert!(hotspot.is_boundary_truncated);
        assert!(!hotspot.is_orphan);
        assert!(surface
            .highlights
            .iter()
            .any(|finding| finding.id == "graph:boundary-truncated:src/a.ts"));
        assert!(surface
            .highlights
            .iter()
            .all(|finding| finding.id != "graph:orphan:src/a.ts"));
    }

    #[test]
    fn architectural_smell_surface_finding_carries_subject_and_related_file_paths() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/blocks.php"),
            br#"<?php
function load_blocks() {
    require_once __DIR__ . '/manager.php';
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/manager.php"),
            br#"<?php
function load_manager() {}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let context = SurfaceBuildContext::new(&analysis);
        let smell = ArchitecturalSmell {
            kind: ArchitecturalSmellKind::HubLikeDependency,
            subject: String::from("blocks.php"),
            related_components: vec![String::from("manager.php")],
            evidence_count: 2,
            severity_millis: 820,
            fingerprint: String::from("test-smell"),
        };

        let primary_anchor =
            best_effort_anchor_for_component(&smell.subject, &analysis, &context, "primary");
        let evidence_anchors = smell
            .related_components
            .iter()
            .filter_map(|component| {
                best_effort_anchor_for_component(component, &analysis, &context, "supporting")
            })
            .collect::<Vec<_>>();
        let file_paths = architectural_smell_file_paths(
            &smell,
            &context,
            primary_anchor.as_ref(),
            &evidence_anchors,
        );
        let finding = surface_finding_from_architectural_smell(&smell, &analysis, &context);

        assert_eq!(
            file_paths,
            vec![
                PathBuf::from("src/blocks.php"),
                PathBuf::from("src/manager.php")
            ]
        );
        assert_eq!(finding.file_paths, file_paths);
        assert_eq!(
            finding
                .primary_anchor
                .as_ref()
                .map(|anchor| anchor.file_path.clone()),
            Some(PathBuf::from("src/blocks.php"))
        );
    }

    #[test]
    fn architectural_smell_surface_finding_resolves_top_level_module_labels_to_files() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::create_dir_all(fixture.join("app")).unwrap();
        fs::write(
            fixture.join("src/service.ts"),
            br#"export function load() {}"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/runtime.ts"),
            br#"export function boot() {}"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let context = SurfaceBuildContext::new(&analysis);
        let smell = ArchitecturalSmell {
            kind: ArchitecturalSmellKind::UnstableDependency,
            subject: String::from("src"),
            related_components: vec![String::from("app")],
            evidence_count: 1,
            severity_millis: 780,
            fingerprint: String::from("test-module-smell"),
        };

        let finding = surface_finding_from_architectural_smell(&smell, &analysis, &context);

        assert_eq!(
            finding.file_paths,
            vec![
                PathBuf::from("src/service.ts"),
                PathBuf::from("app/runtime.ts")
            ]
        );
        assert_eq!(
            finding
                .primary_anchor
                .as_ref()
                .map(|anchor| anchor.file_path.clone()),
            Some(PathBuf::from("src/service.ts"))
        );
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aigiscore-surface-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
