use crate::graph::{
    EdgeOrigin, EdgeStrength, GraphLayer, ReferenceKind, RelationKind, ResolutionTier,
    ResolvedEdge, SemanticGraph, SymbolKind,
};
use crate::plugins::{RepoContext, RuntimePlugin};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct QueueDispatchPlugin;

impl RuntimePlugin for QueueDispatchPlugin {
    fn id(&self) -> &'static str {
        "queue_dispatch"
    }

    fn emit_edges(&self, _repo: &RepoContext, graph: &SemanticGraph) -> Vec<ResolvedEdge> {
        let symbols_by_id = graph
            .symbols
            .iter()
            .map(|symbol| (symbol.id.clone(), symbol))
            .collect::<HashMap<_, _>>();
        let import_targets = import_targets_by_binding(graph, &symbols_by_id);
        let same_file_symbols = same_file_symbol_targets(graph);
        let mut emitted = HashSet::<(PathBuf, String, usize)>::new();
        let mut edges = Vec::new();

        for reference in graph.references.iter().filter(|reference| {
            reference.kind == ReferenceKind::Call
                && matches!(
                    reference.call_form,
                    Some(crate::graph::CallForm::Associated)
                )
                && is_queue_dispatch_call(&reference.target_name)
        }) {
            let Some(receiver_name) = reference.receiver_name.as_ref() else {
                continue;
            };
            let Some((target_symbol_id, target_file_path)) = resolve_dispatch_target(
                &reference.file_path,
                receiver_name,
                &import_targets,
                &same_file_symbols,
            ) else {
                continue;
            };
            if !emitted.insert((
                reference.file_path.clone(),
                target_symbol_id.clone(),
                reference.line,
            )) {
                continue;
            }
            edges.push(
                ResolvedEdge::new(
                    reference.file_path.clone(),
                    reference.enclosing_symbol_id.clone(),
                    target_file_path,
                    target_symbol_id,
                    ReferenceKind::Call,
                    ResolutionTier::Global,
                    700,
                    format!("runtime queue dispatch via {}", reference.target_name),
                    reference.line,
                )
                .with_metadata(
                    RelationKind::Dispatch,
                    GraphLayer::Runtime,
                    EdgeStrength::Dynamic,
                    EdgeOrigin::Plugin,
                ),
            );
        }

        edges
    }
}

fn is_queue_dispatch_call(target_name: &str) -> bool {
    matches!(
        target_name,
        "dispatch" | "dispatchAfterResponse" | "dispatchSync" | "dispatchNow"
    )
}

fn resolve_dispatch_target(
    file_path: &Path,
    receiver_name: &str,
    import_targets: &HashMap<(PathBuf, String), (String, PathBuf)>,
    same_file_symbols: &HashMap<(PathBuf, String), (String, PathBuf)>,
) -> Option<(String, PathBuf)> {
    let binding_name = leaf_symbol_name(receiver_name);
    import_targets
        .get(&(file_path.to_path_buf(), binding_name.clone()))
        .cloned()
        .or_else(|| {
            same_file_symbols
                .get(&(file_path.to_path_buf(), binding_name))
                .cloned()
        })
}

fn import_targets_by_binding(
    graph: &SemanticGraph,
    symbols_by_id: &HashMap<String, &crate::graph::SymbolNode>,
) -> HashMap<(PathBuf, String), (String, PathBuf)> {
    let mut targets = HashMap::new();

    for reference in graph
        .references
        .iter()
        .filter(|reference| reference.kind == ReferenceKind::Import)
    {
        let binding_name = reference
            .binding_name
            .clone()
            .unwrap_or_else(|| leaf_symbol_name(&reference.target_name));
        let resolved_import = graph.resolved_edges.iter().find(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.source_file_path == reference.file_path
                && edge.line == reference.line
        });
        let Some(resolved_import) = resolved_import else {
            continue;
        };
        let Some(symbol) = symbols_by_id.get(&resolved_import.target_symbol_id) else {
            continue;
        };
        if !matches!(symbol.kind, SymbolKind::Class | SymbolKind::Struct) {
            continue;
        }
        targets.insert(
            (reference.file_path.clone(), binding_name),
            (symbol.id.clone(), symbol.file_path.clone()),
        );
    }

    targets
}

fn same_file_symbol_targets(
    graph: &SemanticGraph,
) -> HashMap<(PathBuf, String), (String, PathBuf)> {
    graph
        .symbols
        .iter()
        .filter(|symbol| matches!(symbol.kind, SymbolKind::Class | SymbolKind::Struct))
        .map(|symbol| {
            (
                (symbol.file_path.clone(), symbol.name.clone()),
                (symbol.id.clone(), symbol.file_path.clone()),
            )
        })
        .collect()
}

fn leaf_symbol_name(name: &str) -> String {
    name.trim_matches(&['{', '}'][..])
        .rsplit("::")
        .next()
        .unwrap_or(name)
        .rsplit('\\')
        .next()
        .unwrap_or(name)
        .rsplit('.')
        .next()
        .unwrap_or(name)
        .rsplit('/')
        .next()
        .unwrap_or(name)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::QueueDispatchPlugin;
    use crate::graph::{GraphLayer, RelationKind};
    use crate::parsing::php::parse_php_to_graph;
    use crate::plugins::{RepoContext, RuntimePlugin};
    use crate::resolve::resolve_graph;
    use std::path::PathBuf;

    #[test]
    fn emits_runtime_dispatch_edges_for_imported_php_jobs() {
        let mut graph = parse_php_to_graph(
            PathBuf::from("app/Services/EmailSyncManager.php"),
            r#"<?php
namespace App\Services;

use App\Jobs\SyncAccountJob;

final class EmailSyncManager
{
    public function run(): void
    {
        SyncAccountJob::dispatch('tenant', 1);
    }
}
"#,
        )
        .unwrap();
        let mut imported = parse_php_to_graph(
            PathBuf::from("app/Jobs/SyncAccountJob.php"),
            r#"<?php
namespace App\Jobs;

final class SyncAccountJob {}
"#,
        )
        .unwrap();
        graph.files.append(&mut imported.files);
        graph.symbols.append(&mut imported.symbols);
        graph.references.append(&mut imported.references);
        resolve_graph(&mut graph);

        let plugin = QueueDispatchPlugin;
        let edges = plugin.emit_edges(&RepoContext::new("."), &graph);

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].layer, GraphLayer::Runtime);
        assert_eq!(edges[0].relation_kind, RelationKind::Dispatch);
        assert_eq!(
            edges[0].target_file_path,
            PathBuf::from("app/Jobs/SyncAccountJob.php")
        );
    }

    #[test]
    fn emits_runtime_dispatch_edges_for_aliased_php_jobs() {
        let mut graph = parse_php_to_graph(
            PathBuf::from("app/Services/EmailSyncManager.php"),
            r#"<?php
namespace App\Services;

use App\Jobs\SyncAccountJob as AccountJob;

final class EmailSyncManager
{
    public function run(): void
    {
        AccountJob::dispatchAfterResponse('tenant', 1);
    }
}
"#,
        )
        .unwrap();
        let mut imported = parse_php_to_graph(
            PathBuf::from("app/Jobs/SyncAccountJob.php"),
            r#"<?php
namespace App\Jobs;

final class SyncAccountJob {}
"#,
        )
        .unwrap();
        graph.files.append(&mut imported.files);
        graph.symbols.append(&mut imported.symbols);
        graph.references.append(&mut imported.references);
        resolve_graph(&mut graph);

        let plugin = QueueDispatchPlugin;
        let edges = plugin.emit_edges(&RepoContext::new("."), &graph);

        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges[0].target_file_path,
            PathBuf::from("app/Jobs/SyncAccountJob.php")
        );
    }
}
