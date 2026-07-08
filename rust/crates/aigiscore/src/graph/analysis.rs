use crate::graph::{
    EdgeStrength, GraphLayer, ReferenceKind, RelationKind, ResolvedEdge, SemanticGraph,
};
use crate::identity::{normalized_path, stable_fingerprint};
use crate::ingestion::scan::{AnalysisBoundaryTruth, AnalysisScope};
use petgraph::algo::kosaraju_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{EdgeRef, NodeIndexable};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GraphAnalysis {
    pub circular_dependencies: Vec<Vec<PathBuf>>,
    pub strong_circular_dependencies: Vec<Vec<PathBuf>>,
    pub cycle_findings: Vec<CycleFinding>,
    pub strong_cycle_findings: Vec<CycleFinding>,
    pub architectural_smells: Vec<ArchitecturalSmell>,
    pub coupling_metrics: Vec<CouplingMetric>,
    pub bottleneck_files: Vec<BottleneckFile>,
    #[serde(default)]
    pub zero_inbound_candidate_files: Vec<PathBuf>,
    pub orphan_files: Vec<PathBuf>,
    #[serde(default)]
    pub boundary_truncated_files: Vec<PathBuf>,
    pub runtime_entry_candidates: Vec<PathBuf>,
    #[serde(default)]
    pub boundary_truth: AnalysisBoundaryTruth,
    pub node_count: usize,
    pub edge_count: usize,
    pub density_millis: u32,
    pub override_edges: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleClass {
    Structural,
    Runtime,
    Framework,
    PolicyOverlay,
    Mixed,
    ProbableArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleFinding {
    pub files: Vec<PathBuf>,
    pub cycle_class: CycleClass,
    pub layers: Vec<GraphLayer>,
    pub dominant_relations: Vec<RelationKind>,
    pub edge_count: usize,
    #[serde(default)]
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouplingMetric {
    pub module: String,
    pub afferent: usize,
    pub efferent: usize,
    pub instability_millis: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArchitecturalSmellKind {
    HubLikeDependency,
    UnstableDependency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitecturalSmell {
    pub kind: ArchitecturalSmellKind,
    pub subject: String,
    pub related_components: Vec<String>,
    #[serde(default)]
    pub relation_previews: Vec<String>,
    pub evidence_count: usize,
    pub severity_millis: u16,
    #[serde(default)]
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BottleneckFile {
    pub file_path: PathBuf,
    pub centrality_millis: u32,
    #[serde(default)]
    pub fingerprint: String,
}

pub fn analyze_semantic_graph(
    graph: &SemanticGraph,
    analysis_scope: &AnalysisScope,
) -> GraphAnalysis {
    let file_graph_started = Instant::now();
    let file_graph = build_file_dependency_graph(graph.resolved_edges.iter(), |edge| {
        matches!(
            edge.kind,
            ReferenceKind::Import
                | ReferenceKind::Call
                | ReferenceKind::Type
                | ReferenceKind::Extends
                | ReferenceKind::Implements
        )
    });
    trace(&format!(
        "graph.file_graph elapsed_ms={}",
        file_graph_started.elapsed().as_millis()
    ));

    let strong_graph_started = Instant::now();
    let strong_graph = build_file_dependency_graph(graph.resolved_edges.iter(), |edge| {
        matches!(
            edge.kind,
            ReferenceKind::Import
                | ReferenceKind::Call
                | ReferenceKind::Extends
                | ReferenceKind::Implements
        ) && edge.strength != EdgeStrength::Inferred
    });
    trace(&format!(
        "graph.strong_graph elapsed_ms={}",
        strong_graph_started.elapsed().as_millis()
    ));

    let orphan_started = Instant::now();
    let (
        zero_inbound_candidate_files,
        orphan_files,
        boundary_truncated_files,
        runtime_entry_candidates,
    ) = find_orphan_files(&file_graph, analysis_scope);
    trace(&format!(
        "graph.orphans elapsed_ms={}",
        orphan_started.elapsed().as_millis()
    ));
    let cycles_started = Instant::now();
    let circular_dependencies = find_cycles(&file_graph);
    trace(&format!(
        "graph.cycles elapsed_ms={}",
        cycles_started.elapsed().as_millis()
    ));
    let strong_cycles_started = Instant::now();
    let strong_circular_dependencies = find_cycles(&strong_graph);
    trace(&format!(
        "graph.strong_cycles elapsed_ms={}",
        strong_cycles_started.elapsed().as_millis()
    ));
    let cycle_findings_started = Instant::now();
    let cycle_findings = classify_cycles(graph, &circular_dependencies);
    trace(&format!(
        "graph.cycle_findings elapsed_ms={}",
        cycle_findings_started.elapsed().as_millis()
    ));
    let strong_cycle_findings_started = Instant::now();
    let strong_cycle_findings = classify_cycles(graph, &strong_circular_dependencies);
    trace(&format!(
        "graph.strong_cycle_findings elapsed_ms={}",
        strong_cycle_findings_started.elapsed().as_millis()
    ));

    let coupling_started = Instant::now();
    let coupling_metrics = calculate_coupling(&file_graph);
    trace(&format!(
        "graph.coupling elapsed_ms={}",
        coupling_started.elapsed().as_millis()
    ));
    let smells_started = Instant::now();
    let architectural_smells = detect_architectural_smells(&file_graph, &coupling_metrics);
    trace(&format!(
        "graph.architectural_smells elapsed_ms={}",
        smells_started.elapsed().as_millis()
    ));
    let bottlenecks_started = Instant::now();
    let bottleneck_files = find_bottlenecks(&file_graph, 20);
    trace(&format!(
        "graph.bottlenecks elapsed_ms={}",
        bottlenecks_started.elapsed().as_millis()
    ));

    GraphAnalysis {
        circular_dependencies,
        strong_circular_dependencies,
        cycle_findings,
        strong_cycle_findings,
        architectural_smells,
        coupling_metrics,
        bottleneck_files,
        zero_inbound_candidate_files,
        orphan_files,
        boundary_truncated_files,
        runtime_entry_candidates,
        boundary_truth: analysis_scope.boundary_truth,
        node_count: file_graph.node_count(),
        edge_count: file_graph.edge_count(),
        density_millis: density_millis(&file_graph),
        override_edges: graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.kind == ReferenceKind::Overrides)
            .count(),
    }
}

fn trace(message: &str) {
    if env::var_os("AIGISCORE_TRACE").is_some() {
        eprintln!("[aigiscore] {message}");
    }
}

fn detect_architectural_smells(
    graph: &DiGraph<PathBuf, ()>,
    coupling_metrics: &[CouplingMetric],
) -> Vec<ArchitecturalSmell> {
    let module_dependency_counts = build_module_dependency_counts(graph);
    let mut smells = detect_hub_like_dependencies(coupling_metrics, &module_dependency_counts);
    smells.extend(detect_unstable_dependencies(
        coupling_metrics,
        &module_dependency_counts,
    ));
    smells.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(left.subject.cmp(&right.subject))
            .then(left.kind.cmp(&right.kind))
    });
    smells
}

fn detect_hub_like_dependencies(
    coupling_metrics: &[CouplingMetric],
    module_dependency_counts: &HashMap<(String, String), usize>,
) -> Vec<ArchitecturalSmell> {
    let mut totals = coupling_metrics
        .iter()
        .filter(|metric| metric.afferent >= 2 && metric.efferent >= 2)
        .map(|metric| metric.afferent + metric.efferent)
        .collect::<Vec<_>>();
    if totals.is_empty() {
        return Vec::new();
    }
    totals.sort_unstable();
    let threshold_index = ((totals.len() as f64 * 0.8).floor() as usize).min(totals.len() - 1);
    let threshold = totals[threshold_index].max(6);
    let max_total = *totals.last().unwrap_or(&threshold);

    let mut smells = coupling_metrics
        .iter()
        .filter_map(|metric| {
            let total = metric.afferent + metric.efferent;
            if metric.afferent < 2 || metric.efferent < 2 || total < threshold {
                return None;
            }
            let severity = ((total as f64 / max_total as f64) * 1000.0).round() as u16;
            let (related_components, relation_previews) =
                supporting_neighbors_for_module(&metric.module, module_dependency_counts, 3, 2);
            Some(ArchitecturalSmell {
                kind: ArchitecturalSmellKind::HubLikeDependency,
                subject: metric.module.clone(),
                related_components,
                relation_previews,
                evidence_count: total,
                severity_millis: severity,
                fingerprint: String::new(),
            })
        })
        .collect::<Vec<_>>();
    for smell in &mut smells {
        smell.fingerprint = architectural_smell_fingerprint(smell);
    }
    smells.sort_by(|left, right| left.subject.cmp(&right.subject));
    smells
}

fn detect_unstable_dependencies(
    coupling_metrics: &[CouplingMetric],
    module_dependency_counts: &HashMap<(String, String), usize>,
) -> Vec<ArchitecturalSmell> {
    let instability_by_module = coupling_metrics
        .iter()
        .map(|metric| (metric.module.clone(), metric.instability_millis))
        .collect::<HashMap<_, _>>();
    let afferent_by_module = coupling_metrics
        .iter()
        .map(|metric| (metric.module.clone(), metric.afferent))
        .collect::<HashMap<_, _>>();

    let module_dependencies = module_dependency_counts
        .keys()
        .cloned()
        .collect::<HashSet<_>>();

    let mut smells = module_dependencies
        .into_iter()
        .filter_map(|(source_module, target_module)| {
            let source_instability = *instability_by_module.get(&source_module)?;
            let target_instability = *instability_by_module.get(&target_module)?;
            let source_afferent = *afferent_by_module.get(&source_module).unwrap_or(&0);
            if source_afferent < 2 || source_instability + 250 > target_instability {
                return None;
            }
            let evidence_count = *module_dependency_counts
                .get(&(source_module.clone(), target_module.clone()))
                .unwrap_or(&1);
            Some(ArchitecturalSmell {
                kind: ArchitecturalSmellKind::UnstableDependency,
                subject: source_module.clone(),
                related_components: vec![target_module.clone()],
                relation_previews: vec![format!(
                    "{} -> {} ({} cross-module links)",
                    source_module, target_module, evidence_count
                )],
                evidence_count,
                severity_millis: target_instability - source_instability,
                fingerprint: String::new(),
            })
        })
        .collect::<Vec<_>>();
    for smell in &mut smells {
        smell.fingerprint = architectural_smell_fingerprint(smell);
    }
    smells.sort_by(|left, right| left.subject.cmp(&right.subject));
    smells
}

fn build_module_dependency_counts(
    graph: &DiGraph<PathBuf, ()>,
) -> HashMap<(String, String), usize> {
    let mut counts = HashMap::new();
    for edge in graph.raw_edges() {
        let Some(source) = graph.node_weight(edge.source()) else {
            continue;
        };
        let Some(target) = graph.node_weight(edge.target()) else {
            continue;
        };
        let source_module = top_level_module(source);
        let target_module = top_level_module(target);
        if source_module == target_module {
            continue;
        }
        *counts.entry((source_module, target_module)).or_insert(0) += 1;
    }
    counts
}

fn supporting_neighbors_for_module(
    subject: &str,
    module_dependency_counts: &HashMap<(String, String), usize>,
    component_limit: usize,
    preview_limit: usize,
) -> (Vec<String>, Vec<String>) {
    let mut neighbor_counts = HashMap::<String, (usize, usize)>::new();
    for ((source, target), count) in module_dependency_counts {
        if source == subject {
            neighbor_counts
                .entry(target.clone())
                .and_modify(|(outbound, _)| *outbound += *count)
                .or_insert((*count, 0));
        }
        if target == subject {
            neighbor_counts
                .entry(source.clone())
                .and_modify(|(_, inbound)| *inbound += *count)
                .or_insert((0, *count));
        }
    }
    let mut ranked = neighbor_counts.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        let left_total = left.1 .0 + left.1 .1;
        let right_total = right.1 .0 + right.1 .1;
        right_total.cmp(&left_total).then(left.0.cmp(&right.0))
    });
    let related_components = ranked
        .iter()
        .take(component_limit)
        .map(|(neighbor, _)| neighbor.clone())
        .collect::<Vec<_>>();
    let relation_previews = ranked
        .into_iter()
        .take(preview_limit)
        .map(
            |(neighbor, (outbound, inbound))| match (outbound > 0, inbound > 0) {
                (true, true) => format!(
                    "{} <-> {} ({} cross-module links)",
                    subject,
                    neighbor,
                    outbound + inbound
                ),
                (true, false) => {
                    format!(
                        "{} -> {} ({} cross-module links)",
                        subject, neighbor, outbound
                    )
                }
                (false, true) => {
                    format!(
                        "{} -> {} ({} cross-module links)",
                        neighbor, subject, inbound
                    )
                }
                (false, false) => format!("{} <> {} (0 cross-module links)", subject, neighbor),
            },
        )
        .collect::<Vec<_>>();
    (related_components, relation_previews)
}

fn build_file_dependency_graph<'a, I, F>(edges: I, include: F) -> DiGraph<PathBuf, ()>
where
    I: IntoIterator<Item = &'a ResolvedEdge>,
    F: Fn(&ResolvedEdge) -> bool,
{
    trace("graph.build_file_dependency_graph start");
    let mut graph = DiGraph::<PathBuf, ()>::new();
    let mut indices: HashMap<PathBuf, NodeIndex> = HashMap::new();
    let mut seen_edges: HashSet<(NodeIndex, NodeIndex)> = HashSet::new();
    let mut included_edges = 0usize;

    for edge in edges {
        if !include(edge) {
            continue;
        }
        included_edges += 1;
        if included_edges == 1 || included_edges % 100 == 0 {
            trace(&format!(
                "graph.build_file_dependency_graph progress included_edges={included_edges} nodes={} edges={}",
                graph.node_count(),
                graph.edge_count()
            ));
        }
        let source = *indices
            .entry(edge.source_file_path.clone())
            .or_insert_with(|| graph.add_node(edge.source_file_path.clone()));
        let target = *indices
            .entry(edge.target_file_path.clone())
            .or_insert_with(|| graph.add_node(edge.target_file_path.clone()));

        if source != target && seen_edges.insert((source, target)) {
            graph.add_edge(source, target, ());
        }
    }

    trace(&format!(
        "graph.build_file_dependency_graph done included_edges={included_edges} nodes={} edges={}",
        graph.node_count(),
        graph.edge_count()
    ));

    graph
}

fn find_cycles(graph: &DiGraph<PathBuf, ()>) -> Vec<Vec<PathBuf>> {
    let mut cycles = kosaraju_scc(graph)
        .into_iter()
        .filter(|component| component.len() > 1)
        .map(|component| {
            let mut paths: Vec<PathBuf> = component
                .into_iter()
                .filter_map(|index| graph.node_weight(index).cloned())
                .collect();
            paths.sort();
            paths
        })
        .collect::<Vec<_>>();
    cycles.sort();
    cycles
}

fn classify_cycles(graph: &SemanticGraph, cycles: &[Vec<PathBuf>]) -> Vec<CycleFinding> {
    cycles
        .iter()
        .map(|files| {
            let file_set = files.iter().cloned().collect::<HashSet<_>>();
            let component_edges = graph
                .resolved_edges
                .iter()
                .filter(|edge| {
                    file_set.contains(&edge.source_file_path)
                        && file_set.contains(&edge.target_file_path)
                        && matches!(
                            edge.kind,
                            ReferenceKind::Import
                                | ReferenceKind::Call
                                | ReferenceKind::Type
                                | ReferenceKind::Extends
                                | ReferenceKind::Implements
                        )
                })
                .collect::<Vec<_>>();
            let mut layers = component_edges
                .iter()
                .map(|edge| edge.layer)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            layers.sort_by_key(layer_rank);

            let mut relation_counts = HashMap::<RelationKind, usize>::new();
            for edge in &component_edges {
                *relation_counts.entry(edge.relation_kind).or_default() += 1;
            }
            let mut dominant_relations = relation_counts.into_iter().collect::<Vec<_>>();
            dominant_relations
                .sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
            let dominant_relations = dominant_relations
                .into_iter()
                .map(|(relation, _)| relation)
                .take(3)
                .collect::<Vec<_>>();

            let mut finding = CycleFinding {
                files: files.clone(),
                cycle_class: classify_cycle_class(files, &layers, &component_edges),
                layers,
                dominant_relations,
                edge_count: component_edges.len(),
                fingerprint: String::new(),
            };
            finding.fingerprint = cycle_fingerprint(&finding);
            finding
        })
        .collect()
}

fn classify_cycle_class(
    files: &[PathBuf],
    layers: &[GraphLayer],
    component_edges: &[&ResolvedEdge],
) -> CycleClass {
    // A big knot is only "probably the analyzer's fault" when its cyclicity
    // rests on low-confidence edges. If a strongly-connected core covering
    // most of the component survives on high-confidence edges alone, the
    // knot is proven real regardless of size — labeling it an artifact
    // would teach users to distrust true findings.
    if files.len() >= 8 && !high_confidence_core_covers_component(files, component_edges) {
        return CycleClass::ProbableArtifact;
    }

    if layers.len() > 1 {
        return CycleClass::Mixed;
    }

    match layers.first().copied().unwrap_or(GraphLayer::Structural) {
        GraphLayer::Structural => CycleClass::Structural,
        GraphLayer::Runtime => CycleClass::Runtime,
        GraphLayer::Framework => CycleClass::Framework,
        GraphLayer::PolicyOverlay => CycleClass::PolicyOverlay,
    }
}

const HIGH_CONFIDENCE_EDGE_MILLIS: u16 = 900;

/// True when the component stays strongly connected across most of its files
/// (>= 60%) using only high-confidence edges.
fn high_confidence_core_covers_component(
    files: &[PathBuf],
    component_edges: &[&ResolvedEdge],
) -> bool {
    let mut graph = DiGraph::<PathBuf, ()>::new();
    let mut indices = HashMap::<PathBuf, _>::new();
    let mut seen_edges = HashSet::new();
    for edge in component_edges
        .iter()
        .filter(|edge| edge.confidence_millis >= HIGH_CONFIDENCE_EDGE_MILLIS)
    {
        let source = *indices
            .entry(edge.source_file_path.clone())
            .or_insert_with(|| graph.add_node(edge.source_file_path.clone()));
        let target = *indices
            .entry(edge.target_file_path.clone())
            .or_insert_with(|| graph.add_node(edge.target_file_path.clone()));
        if source != target && seen_edges.insert((source, target)) {
            graph.add_edge(source, target, ());
        }
    }
    let largest_core = kosaraju_scc(&graph)
        .into_iter()
        .map(|component| component.len())
        .max()
        .unwrap_or(0);
    largest_core * 10 >= files.len() * 6
}

fn layer_rank(layer: &GraphLayer) -> u8 {
    match layer {
        GraphLayer::Structural => 0,
        GraphLayer::Runtime => 1,
        GraphLayer::Framework => 2,
        GraphLayer::PolicyOverlay => 3,
    }
}

fn calculate_coupling(graph: &DiGraph<PathBuf, ()>) -> Vec<CouplingMetric> {
    let mut module_in: HashMap<String, HashSet<String>> = HashMap::new();
    let mut module_out: HashMap<String, HashSet<String>> = HashMap::new();

    for node in graph.node_weights() {
        let module = top_level_module(node);
        module_in.entry(module.clone()).or_default();
        module_out.entry(module).or_default();
    }

    for edge in graph.raw_edges() {
        let source = graph
            .node_weight(edge.source())
            .expect("missing source node");
        let target = graph
            .node_weight(edge.target())
            .expect("missing target node");
        let source_module = top_level_module(source);
        let target_module = top_level_module(target);
        if source_module != target_module {
            module_out
                .entry(source_module.clone())
                .or_default()
                .insert(target_module.clone());
            module_in
                .entry(target_module)
                .or_default()
                .insert(source_module);
        }
    }

    let mut metrics = module_in
        .keys()
        .chain(module_out.keys())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|module| {
            let afferent = module_in.get(&module).map(HashSet::len).unwrap_or(0);
            let efferent = module_out.get(&module).map(HashSet::len).unwrap_or(0);
            let instability = if afferent + efferent == 0 {
                0
            } else {
                ((efferent as f64 / (afferent + efferent) as f64) * 1000.0).round() as u16
            };
            CouplingMetric {
                module,
                afferent,
                efferent,
                instability_millis: instability,
            }
        })
        .collect::<Vec<_>>();
    metrics.sort_by(|left, right| left.module.cmp(&right.module));
    metrics
}

fn top_level_module(path: &Path) -> String {
    path.iter()
        .next()
        .map(|segment| segment.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("."))
}

const ENTRY_POINT_PATTERNS: [&str; 13] = [
    "/main.",
    "/lib.",
    "/mod.",
    "/index.",
    "/__init__.",
    "/config/",
    "/controllers/",
    "/Controllers/",
    "/routes/",
    "/migrations/",
    "/seeders/",
    "/factories/",
    "/tests/",
];

fn find_orphan_files(
    graph: &DiGraph<PathBuf, ()>,
    analysis_scope: &AnalysisScope,
) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut zero_inbound_candidates = Vec::new();
    let mut orphans = Vec::new();
    let mut boundary_truncated = Vec::new();
    let mut runtime_entry_candidates = Vec::new();

    for node_index in graph.node_indices() {
        let in_degree = graph
            .edges_directed(node_index, petgraph::Direction::Incoming)
            .count();
        let out_degree = graph
            .edges_directed(node_index, petgraph::Direction::Outgoing)
            .count();
        if in_degree != 0 || out_degree == 0 {
            continue;
        }
        let Some(path) = graph.node_weight(node_index).cloned() else {
            continue;
        };
        zero_inbound_candidates.push(path.clone());
        if is_default_entry_point(&path) {
            runtime_entry_candidates.push(path);
        } else if analysis_scope.boundary_truth == AnalysisBoundaryTruth::TruncatedSlice {
            boundary_truncated.push(path);
        } else {
            orphans.push(path);
        }
    }

    zero_inbound_candidates.sort();
    orphans.sort();
    boundary_truncated.sort();
    runtime_entry_candidates.sort();
    (
        zero_inbound_candidates,
        orphans,
        boundary_truncated,
        runtime_entry_candidates,
    )
}

fn is_default_entry_point(path: &Path) -> bool {
    let normalized = format!("/{}", path.to_string_lossy().replace('\\', "/"));
    ENTRY_POINT_PATTERNS
        .iter()
        .any(|pattern| normalized.contains(pattern))
}

fn find_bottlenecks(graph: &DiGraph<PathBuf, ()>, top_n: usize) -> Vec<BottleneckFile> {
    if graph.node_count() < 3 {
        return Vec::new();
    }

    let centrality = brandes_betweenness_centrality(graph);
    let mut ranked = centrality
        .into_iter()
        .filter_map(|(index, score)| {
            (score > 0.0).then(|| {
                graph.node_weight(index).cloned().map(|file_path| {
                    let mut finding = BottleneckFile {
                        file_path,
                        centrality_millis: (score * 1000.0).round() as u32,
                        fingerprint: String::new(),
                    };
                    finding.fingerprint = bottleneck_fingerprint(&finding.file_path);
                    finding
                })
            })?
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| {
        right
            .centrality_millis
            .cmp(&left.centrality_millis)
            .then_with(|| left.file_path.cmp(&right.file_path))
    });
    ranked.truncate(top_n);
    ranked
}

fn brandes_betweenness_centrality(graph: &DiGraph<PathBuf, ()>) -> HashMap<NodeIndex, f64> {
    let node_bound = graph.node_bound();
    let mut centrality = vec![0.0_f64; node_bound];

    for source in graph.node_indices() {
        let mut stack = Vec::new();
        let mut predecessors = vec![Vec::<NodeIndex>::new(); node_bound];
        let mut sigma = vec![0.0_f64; node_bound];
        let mut distance = vec![-1_i32; node_bound];

        sigma[source.index()] = 1.0;
        distance[source.index()] = 0;

        let mut queue = VecDeque::from([source]);
        while let Some(vertex) = queue.pop_front() {
            stack.push(vertex);
            let vertex_distance = distance[vertex.index()];
            for edge in graph.edges(vertex) {
                let neighbor = edge.target();
                let neighbor_index = neighbor.index();
                if distance[neighbor_index] < 0 {
                    queue.push_back(neighbor);
                    distance[neighbor_index] = vertex_distance + 1;
                }
                if distance[neighbor_index] == vertex_distance + 1 {
                    sigma[neighbor_index] += sigma[vertex.index()];
                    predecessors[neighbor_index].push(vertex);
                }
            }
        }

        let mut dependency = vec![0.0_f64; node_bound];

        while let Some(vertex) = stack.pop() {
            let vertex_index = vertex.index();
            let sigma_vertex = sigma[vertex_index];
            if sigma_vertex == 0.0 {
                continue;
            }
            let dependency_vertex = dependency[vertex_index];
            let predecessors_for_vertex = std::mem::take(&mut predecessors[vertex_index]);
            for predecessor in predecessors_for_vertex {
                let predecessor_index = predecessor.index();
                let sigma_predecessor = sigma[predecessor_index];
                let contribution = (sigma_predecessor / sigma_vertex) * (1.0 + dependency_vertex);
                dependency[predecessor_index] += contribution;
            }
            if vertex != source {
                centrality[vertex_index] += dependency_vertex;
            }
        }
    }

    graph
        .node_indices()
        .map(|index| (index, centrality[index.index()]))
        .collect()
}

fn density_millis(graph: &DiGraph<PathBuf, ()>) -> u32 {
    let node_count = graph.node_count();
    if node_count < 2 {
        return 0;
    }
    let possible_edges = (node_count * (node_count - 1)) as f64;
    ((graph.edge_count() as f64 / possible_edges) * 1000.0).round() as u32
}

fn cycle_fingerprint(finding: &CycleFinding) -> String {
    let file_parts = sorted_paths(&finding.files);
    let mut layer_parts = finding
        .layers
        .iter()
        .map(|layer| graph_layer_label(*layer))
        .collect::<Vec<_>>();
    layer_parts.sort();
    let mut relation_parts = finding
        .dominant_relations
        .iter()
        .map(|relation| relation_kind_label(*relation))
        .collect::<Vec<_>>();
    relation_parts.sort();
    let edge_count = finding.edge_count.to_string();
    let mut parts = vec!["graph", "cycle", cycle_class_label(finding.cycle_class)];
    parts.extend(file_parts.iter().map(String::as_str));
    parts.extend(layer_parts);
    parts.extend(relation_parts);
    parts.push(edge_count.as_str());
    stable_fingerprint(&parts)
}

fn architectural_smell_fingerprint(smell: &ArchitecturalSmell) -> String {
    let kind = match smell.kind {
        ArchitecturalSmellKind::HubLikeDependency => "hub-like-dependency",
        ArchitecturalSmellKind::UnstableDependency => "unstable-dependency",
    };
    let mut related = smell.related_components.clone();
    related.sort();
    related.dedup();
    let mut parts = vec!["graph", "smell", kind, smell.subject.as_str()];
    parts.extend(related.iter().map(String::as_str));
    stable_fingerprint(&parts)
}

fn bottleneck_fingerprint(path: &Path) -> String {
    stable_fingerprint(&["graph", "bottleneck", &normalized_path(path)])
}

fn cycle_class_label(cycle_class: CycleClass) -> &'static str {
    match cycle_class {
        CycleClass::Structural => "structural",
        CycleClass::Runtime => "runtime",
        CycleClass::Framework => "framework",
        CycleClass::PolicyOverlay => "policy-overlay",
        CycleClass::Mixed => "mixed",
        CycleClass::ProbableArtifact => "probable-artifact",
    }
}

fn graph_layer_label(layer: GraphLayer) -> &'static str {
    match layer {
        GraphLayer::Structural => "structural",
        GraphLayer::Runtime => "runtime",
        GraphLayer::Framework => "framework",
        GraphLayer::PolicyOverlay => "policy-overlay",
    }
}

fn relation_kind_label(kind: RelationKind) -> &'static str {
    match kind {
        RelationKind::Import => "import",
        RelationKind::Call => "call",
        RelationKind::Dispatch => "dispatch",
        RelationKind::ContainerResolution => "container-resolution",
        RelationKind::EventSubscribe => "event-subscribe",
        RelationKind::EventPublish => "event-publish",
        RelationKind::TypeUse => "type-use",
        RelationKind::Extends => "extends",
        RelationKind::Implements => "implements",
        RelationKind::Overrides => "overrides",
    }
}

fn sorted_paths(paths: &[PathBuf]) -> Vec<String> {
    let mut parts = paths
        .iter()
        .map(|path| normalized_path(path))
        .collect::<Vec<_>>();
    parts.sort();
    parts.dedup();
    parts
}

#[cfg(test)]
mod tests {
    use super::{analyze_semantic_graph, ArchitecturalSmellKind};
    use crate::graph::{
        EdgeOrigin, EdgeStrength, GraphLayer, ReferenceKind, RelationKind, ResolutionTier,
        ResolvedEdge, SemanticGraph,
    };
    use crate::ingestion::scan::{AnalysisBoundaryReason, AnalysisBoundaryTruth, AnalysisScope};
    use std::path::PathBuf;

    #[test]
    fn finds_cycles_from_resolved_file_edges() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.rs", "src/b.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/b.rs", "src/a.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/b.rs", "src/c.rs", ReferenceKind::Call));

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());

        assert_eq!(
            analysis.circular_dependencies,
            vec![vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")]]
        );
        assert_eq!(
            analysis.strong_circular_dependencies,
            vec![vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")]]
        );
        assert_eq!(analysis.strong_cycle_findings.len(), 1);
        assert_eq!(
            analysis.strong_cycle_findings[0].cycle_class,
            super::CycleClass::Structural
        );
        assert_eq!(analysis.coupling_metrics.len(), 1);
        assert_eq!(analysis.coupling_metrics[0].module, "src");
        assert_eq!(analysis.node_count, 3);
        assert_eq!(analysis.edge_count, 3);
        assert_eq!(analysis.override_edges, 0);
    }

    #[test]
    fn calculates_module_coupling_metrics() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.rs", "domain/b.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/a.rs", "infra/c.rs", ReferenceKind::Call));
        graph.add_resolved_edge(edge("infra/c.rs", "domain/b.rs", ReferenceKind::Import));

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());
        let modules = analysis
            .coupling_metrics
            .iter()
            .map(|metric| (metric.module.clone(), metric.afferent, metric.efferent))
            .collect::<Vec<_>>();

        assert!(modules.contains(&(String::from("src"), 0, 2)));
        assert!(modules.contains(&(String::from("domain"), 2, 0)));
        assert!(modules.contains(&(String::from("infra"), 1, 1)));
        assert_eq!(analysis.override_edges, 0);
    }

    #[test]
    fn detects_hub_like_dependency_smells() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("domain/a.rs", "core/hub.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("infra/b.rs", "core/hub.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("ui/c.rs", "core/hub.rs", ReferenceKind::Call));
        graph.add_resolved_edge(edge("core/hub.rs", "domain/a.rs", ReferenceKind::Call));
        graph.add_resolved_edge(edge("core/hub.rs", "infra/b.rs", ReferenceKind::Call));
        graph.add_resolved_edge(edge("core/hub.rs", "ui/c.rs", ReferenceKind::Import));

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());

        assert!(analysis.architectural_smells.iter().any(|smell| {
            smell.kind == ArchitecturalSmellKind::HubLikeDependency
                && smell.subject == "core"
                && !smell.related_components.is_empty()
                && !smell.relation_previews.is_empty()
        }));
    }

    #[test]
    fn detects_unstable_dependency_smells() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge(
            "app/main.rs",
            "domain/service.rs",
            ReferenceKind::Import,
        ));
        graph.add_resolved_edge(edge(
            "infra/cache.rs",
            "domain/service.rs",
            ReferenceKind::Import,
        ));
        graph.add_resolved_edge(edge(
            "domain/service.rs",
            "ui/panel.rs",
            ReferenceKind::Import,
        ));
        graph.add_resolved_edge(edge("ui/panel.rs", "infra/cache.rs", ReferenceKind::Call));
        graph.add_resolved_edge(edge("ui/panel.rs", "app/main.rs", ReferenceKind::Call));

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());

        assert!(analysis.architectural_smells.iter().any(|smell| {
            smell.kind == ArchitecturalSmellKind::UnstableDependency
                && smell.subject == "domain"
                && smell.related_components == vec![String::from("ui")]
                && smell.relation_previews
                    == vec![String::from("domain -> ui (1 cross-module links)")]
        }));
    }

    #[test]
    fn counts_override_edges_without_affecting_dependency_cycles() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.py", "src/base.py", ReferenceKind::Extends));
        graph.add_resolved_edge(edge("src/a.py", "src/base.py", ReferenceKind::Overrides));

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());

        assert!(analysis.circular_dependencies.is_empty());
        assert_eq!(analysis.override_edges, 1);
    }

    #[test]
    fn classifies_mixed_cycles_from_non_structural_layers() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.rs", "src/b.rs", ReferenceKind::Import));
        graph.add_resolved_edge(
            edge("src/b.rs", "src/a.rs", ReferenceKind::Call).with_metadata(
                RelationKind::Call,
                GraphLayer::Runtime,
                EdgeStrength::Dynamic,
                EdgeOrigin::Plugin,
            ),
        );

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());

        assert_eq!(analysis.strong_cycle_findings.len(), 1);
        assert_eq!(
            analysis.strong_cycle_findings[0].cycle_class,
            super::CycleClass::Mixed
        );
        assert_eq!(analysis.strong_cycle_findings[0].layers.len(), 2);
    }

    #[test]
    fn excludes_type_only_cycles_from_strong_cycles() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.rs", "src/b.rs", ReferenceKind::Type));
        graph.add_resolved_edge(edge("src/b.rs", "src/a.rs", ReferenceKind::Type));

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());

        assert_eq!(
            analysis.circular_dependencies,
            vec![vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")]]
        );
        assert!(analysis.strong_circular_dependencies.is_empty());
        assert!(analysis.strong_cycle_findings.is_empty());
    }

    #[test]
    fn detects_orphans_runtime_entries_and_bottlenecks() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.rs", "src/b.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/b.rs", "src/c.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/main.rs", "src/b.rs", ReferenceKind::Call));

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());

        assert!(analysis.orphan_files.contains(&PathBuf::from("src/a.rs")));
        assert!(analysis
            .runtime_entry_candidates
            .contains(&PathBuf::from("src/main.rs")));
        assert!(!analysis.bottleneck_files.is_empty());
        assert_eq!(
            analysis.bottleneck_files[0].file_path,
            PathBuf::from("src/b.rs")
        );
        assert!(analysis.bottleneck_files[0].centrality_millis > 0);
        assert_eq!(analysis.node_count, 4);
        assert_eq!(analysis.edge_count, 3);
        assert!(analysis.density_millis > 0);
    }

    #[test]
    fn treats_zero_inbound_files_as_boundary_truncated_in_scoped_analysis() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.rs", "src/b.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/main.rs", "src/b.rs", ReferenceKind::Call));

        let analysis = analyze_semantic_graph(
            &graph,
            &AnalysisScope {
                repository_root: Some(PathBuf::from("/tmp/repo")),
                boundary_truth: AnalysisBoundaryTruth::TruncatedSlice,
                reasons: vec![AnalysisBoundaryReason::CroppedRoot],
                include_path_prefixes: Vec::new(),
            },
        );

        assert!(analysis.orphan_files.is_empty());
        assert!(analysis
            .boundary_truncated_files
            .contains(&PathBuf::from("src/a.rs")));
        assert_eq!(
            analysis.boundary_truth,
            AnalysisBoundaryTruth::TruncatedSlice
        );
    }

    fn edge(source: &str, target: &str, kind: ReferenceKind) -> ResolvedEdge {
        ResolvedEdge::new(
            PathBuf::from(source),
            None,
            PathBuf::from(target),
            format!("symbol:{target}"),
            kind,
            ResolutionTier::ImportScoped,
            900,
            String::from("test"),
            1,
        )
    }

    fn low_confidence_framework_edge(source: &str, target: &str) -> ResolvedEdge {
        let mut edge = edge(source, target, ReferenceKind::Call);
        edge.resolution_tier = ResolutionTier::Global;
        edge.confidence_millis = 650;
        edge.layer = GraphLayer::Framework;
        edge.relation_kind = RelationKind::ContainerResolution;
        edge
    }

    // Ten files in a ring. With high-confidence edges closing the ring the
    // knot is proven real (Mixed), never an artifact — even at suspicious
    // size with mixed layers.
    #[test]
    fn big_cycle_on_high_confidence_edges_is_not_an_artifact() {
        let mut graph = SemanticGraph::default();
        let files = (0..10).map(|i| format!("src/f{i}.php")).collect::<Vec<_>>();
        for pair in files.windows(2) {
            graph.add_resolved_edge(edge(&pair[0], &pair[1], ReferenceKind::Call));
        }
        graph.add_resolved_edge(edge(&files[9], &files[0], ReferenceKind::Call));
        // One extra low-confidence framework edge mixes the layers without
        // being load-bearing for the cycle.
        graph.add_resolved_edge(low_confidence_framework_edge(&files[0], &files[5]));

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());

        assert_eq!(analysis.strong_cycle_findings.len(), 1);
        assert_eq!(
            analysis.strong_cycle_findings[0].cycle_class,
            super::CycleClass::Mixed
        );
    }

    // Same ring, but the closing edge only exists at low confidence: the
    // cyclicity itself is unproven, so the artifact suspicion stands.
    #[test]
    fn big_cycle_closed_only_by_low_confidence_edges_is_probable_artifact() {
        let mut graph = SemanticGraph::default();
        let files = (0..10).map(|i| format!("src/f{i}.php")).collect::<Vec<_>>();
        for pair in files.windows(2) {
            graph.add_resolved_edge(edge(&pair[0], &pair[1], ReferenceKind::Call));
        }
        graph.add_resolved_edge(low_confidence_framework_edge(&files[9], &files[0]));

        let analysis = analyze_semantic_graph(&graph, &AnalysisScope::default());

        assert_eq!(analysis.strong_cycle_findings.len(), 1);
        assert_eq!(
            analysis.strong_cycle_findings[0].cycle_class,
            super::CycleClass::ProbableArtifact
        );
    }
}
