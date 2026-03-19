use crate::assessment::{ArchitecturalAssessmentFinding, ArchitecturalAssessmentKind};
use crate::contracts::ContractInventory;
use crate::detectors::dead_code::{DeadCodeCategory, DeadCodeFinding};
use crate::detectors::hardwiring::{HardwiringCategory, HardwiringFinding};
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
    pub is_runtime_entry: bool,
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
    pub is_runtime_entry: bool,
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

pub fn build_architecture_surface(analysis: &ProjectAnalysis) -> ArchitectureSurface {
    let orphan_set = analysis
        .graph_analysis
        .orphan_files
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let runtime_entry_set = analysis
        .graph_analysis
        .runtime_entry_candidates
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let bottlenecks = analysis
        .graph_analysis
        .bottleneck_files
        .iter()
        .map(|file| (file.file_path.clone(), file.centrality_millis))
        .collect::<HashMap<_, _>>();
    let file_languages = collect_file_languages(analysis);
    let inbound_outbound = collect_inbound_outbound_edges(analysis);
    let finding_counts = collect_finding_counts(analysis);

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
                is_runtime_entry: runtime_entry_set.contains(&file.path),
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
                is_runtime_entry: runtime_entry_set.contains(&file.path),
            }
        })
        .collect::<Vec<_>>();
    let atlas_edges = build_atlas_edges(analysis);

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
            orphan_count: analysis.graph_analysis.orphan_files.len(),
            runtime_entry_count: analysis.graph_analysis.runtime_entry_candidates.len(),
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
        highlights: build_highlights(analysis),
        atlas: RepositoryAtlas {
            nodes: atlas_nodes,
            edges: atlas_edges,
        },
        timings: analysis.timings.clone(),
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
    for path in &analysis.graph_analysis.orphan_files {
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

fn build_highlights(analysis: &ProjectAnalysis) -> Vec<SurfaceFinding> {
    let mut highlights = Vec::new();

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
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![String::from("structural.coherence")],
        });
    }

    for path in &analysis.graph_analysis.orphan_files {
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
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![String::from("structural.coherence")],
        });
    }

    for bottleneck in analysis.graph_analysis.bottleneck_files.iter().take(10) {
        highlights.push(surface_finding_from_bottleneck(bottleneck));
    }

    for smell in &analysis.graph_analysis.architectural_smells {
        highlights.push(surface_finding_from_architectural_smell(smell));
    }

    for finding in &analysis.architectural_assessment.findings {
        highlights.push(surface_finding_from_architectural_assessment(finding));
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

fn surface_finding_from_architectural_assessment(
    finding: &ArchitecturalAssessmentFinding,
) -> SurfaceFinding {
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
            line: None,
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
                line: None,
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
                line: None,
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
                line: None,
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
                line: None,
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
    }
}

fn surface_finding_from_architectural_smell(smell: &ArchitecturalSmell) -> SurfaceFinding {
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
            file_paths: Vec::new(),
            line: None,
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
            file_paths: Vec::new(),
            line: None,
            provenance: vec![String::from("graph_analysis")],
            doctrine_refs: vec![
                String::from("structural.coherence"),
                String::from("guardian.boundary-stability"),
            ],
        },
    }
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

fn surface_finding_from_bottleneck(bottleneck: &BottleneckFile) -> SurfaceFinding {
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
        line: None,
        provenance: vec![String::from("graph_analysis")],
        doctrine_refs: vec![
            String::from("structural.coherence"),
            String::from("guardian.centralized-damage"),
        ],
    }
}

fn surface_finding_from_dead_code(finding: &DeadCodeFinding) -> SurfaceFinding {
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
        precision: String::from("exact"),
        confidence_millis: 900,
        title: match finding.category {
            DeadCodeCategory::UnusedPrivateFunction => String::from("Unused private function"),
            DeadCodeCategory::UnusedImport => String::from("Unused import"),
        },
        summary: format!("{} appears unused", finding.name),
        file_paths: vec![finding.file_path.clone()],
        line: Some(finding.line),
        provenance: vec![
            String::from("dead_code_detector"),
            String::from("graph_analysis"),
        ],
        doctrine_refs: vec![String::from("maintainability.remove-dead-code")],
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
    use super::build_architecture_surface;
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use std::fs;
    use std::path::PathBuf;
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
            .any(|hotspot| hotspot.file_path == PathBuf::from("src/service.ts")));
        assert!(surface.atlas.edges.iter().any(|edge| edge.source_file_path
            == PathBuf::from("src/service.ts")
            && edge.target_file_path == PathBuf::from("src/factory.ts")));
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
