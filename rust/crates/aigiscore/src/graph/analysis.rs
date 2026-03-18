use crate::graph::{GraphLayer, ReferenceKind, RelationKind, ResolvedEdge, SemanticGraph};
use petgraph::algo::kosaraju_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GraphAnalysis {
    pub circular_dependencies: Vec<Vec<PathBuf>>,
    pub strong_circular_dependencies: Vec<Vec<PathBuf>>,
    pub cycle_findings: Vec<CycleFinding>,
    pub strong_cycle_findings: Vec<CycleFinding>,
    pub coupling_metrics: Vec<CouplingMetric>,
    pub bottleneck_files: Vec<BottleneckFile>,
    pub orphan_files: Vec<PathBuf>,
    pub runtime_entry_candidates: Vec<PathBuf>,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouplingMetric {
    pub module: String,
    pub afferent: usize,
    pub efferent: usize,
    pub instability_millis: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BottleneckFile {
    pub file_path: PathBuf,
    pub centrality_millis: u32,
}

pub fn analyze_semantic_graph(graph: &SemanticGraph) -> GraphAnalysis {
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
    let strong_graph = build_file_dependency_graph(graph.resolved_edges.iter(), |edge| {
        matches!(
            edge.kind,
            ReferenceKind::Import
                | ReferenceKind::Call
                | ReferenceKind::Type
                | ReferenceKind::Extends
                | ReferenceKind::Implements
        )
    });

    let (orphan_files, runtime_entry_candidates) = find_orphan_files(&file_graph);
    let circular_dependencies = find_cycles(&file_graph);
    let strong_circular_dependencies = find_cycles(&strong_graph);
    let cycle_findings = classify_cycles(graph, &circular_dependencies);
    let strong_cycle_findings = classify_cycles(graph, &strong_circular_dependencies);

    GraphAnalysis {
        circular_dependencies,
        strong_circular_dependencies,
        cycle_findings,
        strong_cycle_findings,
        coupling_metrics: calculate_coupling(&file_graph),
        bottleneck_files: find_bottlenecks(&file_graph, 20),
        orphan_files,
        runtime_entry_candidates,
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

fn build_file_dependency_graph<'a, I, F>(edges: I, include: F) -> DiGraph<PathBuf, ()>
where
    I: IntoIterator<Item = &'a ResolvedEdge>,
    F: Fn(&ResolvedEdge) -> bool,
{
    let mut graph = DiGraph::<PathBuf, ()>::new();
    let mut indices: HashMap<PathBuf, NodeIndex> = HashMap::new();

    for edge in edges {
        if !include(edge) {
            continue;
        }
        let source = *indices
            .entry(edge.source_file_path.clone())
            .or_insert_with(|| graph.add_node(edge.source_file_path.clone()));
        let target = *indices
            .entry(edge.target_file_path.clone())
            .or_insert_with(|| graph.add_node(edge.target_file_path.clone()));

        if source != target && graph.find_edge(source, target).is_none() {
            graph.add_edge(source, target, ());
        }
    }

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

            CycleFinding {
                files: files.clone(),
                cycle_class: classify_cycle_class(files, &layers, &dominant_relations),
                layers,
                dominant_relations,
                edge_count: component_edges.len(),
            }
        })
        .collect()
}

fn classify_cycle_class(
    files: &[PathBuf],
    layers: &[GraphLayer],
    dominant_relations: &[RelationKind],
) -> CycleClass {
    if layers.len() > 1 {
        if files.len() >= 8 {
            return CycleClass::ProbableArtifact;
        }
        return CycleClass::Mixed;
    }

    match layers.first().copied().unwrap_or(GraphLayer::Structural) {
        GraphLayer::Structural => CycleClass::Structural,
        GraphLayer::Runtime => CycleClass::Runtime,
        GraphLayer::Framework => {
            if files.len() >= 8
                && dominant_relations
                    .iter()
                    .all(|relation| matches!(relation, RelationKind::Call | RelationKind::Import))
            {
                CycleClass::ProbableArtifact
            } else {
                CycleClass::Framework
            }
        }
        GraphLayer::PolicyOverlay => CycleClass::PolicyOverlay,
    }
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

fn top_level_module(path: &PathBuf) -> String {
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

fn find_orphan_files(graph: &DiGraph<PathBuf, ()>) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut orphans = Vec::new();
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
        if is_default_entry_point(&path) {
            runtime_entry_candidates.push(path);
        } else {
            orphans.push(path);
        }
    }

    orphans.sort();
    runtime_entry_candidates.sort();
    (orphans, runtime_entry_candidates)
}

fn is_default_entry_point(path: &PathBuf) -> bool {
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
                graph
                    .node_weight(index)
                    .cloned()
                    .map(|file_path| BottleneckFile {
                        file_path,
                        centrality_millis: (score * 1000.0).round() as u32,
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
    let mut centrality = graph
        .node_indices()
        .map(|index| (index, 0.0))
        .collect::<HashMap<_, _>>();

    for source in graph.node_indices() {
        let mut stack = Vec::new();
        let mut predecessors = graph
            .node_indices()
            .map(|index| (index, Vec::<NodeIndex>::new()))
            .collect::<HashMap<_, _>>();
        let mut sigma = graph
            .node_indices()
            .map(|index| (index, 0.0))
            .collect::<HashMap<_, _>>();
        let mut distance = graph
            .node_indices()
            .map(|index| (index, -1_i32))
            .collect::<HashMap<_, _>>();

        sigma.insert(source, 1.0);
        distance.insert(source, 0);

        let mut queue = std::collections::VecDeque::from([source]);
        while let Some(vertex) = queue.pop_front() {
            stack.push(vertex);
            let vertex_distance = *distance.get(&vertex).unwrap_or(&-1);
            for edge in graph.edges(vertex) {
                let neighbor = edge.target();
                if *distance.get(&neighbor).unwrap_or(&-1) < 0 {
                    queue.push_back(neighbor);
                    distance.insert(neighbor, vertex_distance + 1);
                }
                if *distance.get(&neighbor).unwrap_or(&-1) == vertex_distance + 1 {
                    let sigma_vertex = *sigma.get(&vertex).unwrap_or(&0.0);
                    sigma
                        .entry(neighbor)
                        .and_modify(|value| *value += sigma_vertex)
                        .or_insert(sigma_vertex);
                    predecessors.entry(neighbor).or_default().push(vertex);
                }
            }
        }

        let mut dependency = graph
            .node_indices()
            .map(|index| (index, 0.0))
            .collect::<HashMap<_, _>>();

        while let Some(vertex) = stack.pop() {
            let sigma_vertex = *sigma.get(&vertex).unwrap_or(&0.0);
            if sigma_vertex == 0.0 {
                continue;
            }
            let dependency_vertex = *dependency.get(&vertex).unwrap_or(&0.0);
            let predecessors_for_vertex = predecessors.remove(&vertex).unwrap_or_default();
            for predecessor in predecessors_for_vertex {
                let sigma_predecessor = *sigma.get(&predecessor).unwrap_or(&0.0);
                let contribution = (sigma_predecessor / sigma_vertex) * (1.0 + dependency_vertex);
                dependency
                    .entry(predecessor)
                    .and_modify(|value| *value += contribution)
                    .or_insert(contribution);
            }
            if vertex != source {
                centrality
                    .entry(vertex)
                    .and_modify(|value| *value += dependency_vertex)
                    .or_insert(dependency_vertex);
            }
        }
    }

    centrality
}

fn density_millis(graph: &DiGraph<PathBuf, ()>) -> u32 {
    let node_count = graph.node_count();
    if node_count < 2 {
        return 0;
    }
    let possible_edges = (node_count * (node_count - 1)) as f64;
    ((graph.edge_count() as f64 / possible_edges) * 1000.0).round() as u32
}

#[cfg(test)]
mod tests {
    use super::analyze_semantic_graph;
    use crate::graph::{
        EdgeOrigin, EdgeStrength, GraphLayer, ReferenceKind, RelationKind, ResolutionTier,
        ResolvedEdge, SemanticGraph,
    };
    use std::path::PathBuf;

    #[test]
    fn finds_cycles_from_resolved_file_edges() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.rs", "src/b.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/b.rs", "src/a.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/b.rs", "src/c.rs", ReferenceKind::Call));

        let analysis = analyze_semantic_graph(&graph);

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

        let analysis = analyze_semantic_graph(&graph);
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
    fn counts_override_edges_without_affecting_dependency_cycles() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.py", "src/base.py", ReferenceKind::Extends));
        graph.add_resolved_edge(edge("src/a.py", "src/base.py", ReferenceKind::Overrides));

        let analysis = analyze_semantic_graph(&graph);

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

        let analysis = analyze_semantic_graph(&graph);

        assert_eq!(analysis.strong_cycle_findings.len(), 1);
        assert_eq!(
            analysis.strong_cycle_findings[0].cycle_class,
            super::CycleClass::Mixed
        );
        assert_eq!(analysis.strong_cycle_findings[0].layers.len(), 2);
    }

    #[test]
    fn detects_orphans_runtime_entries_and_bottlenecks() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(edge("src/a.rs", "src/b.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/b.rs", "src/c.rs", ReferenceKind::Import));
        graph.add_resolved_edge(edge("src/main.rs", "src/b.rs", ReferenceKind::Call));

        let analysis = analyze_semantic_graph(&graph);

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
}
