pub mod container;
pub mod queue;
pub mod signals;
pub mod wordpress;

use crate::graph::{ReferenceKind, ResolvedEdge, SemanticGraph, SymbolNode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimePluginDescriptor {
    pub id: &'static str,
    pub description: &'static str,
}

pub trait RuntimePlugin {
    fn id(&self) -> &'static str;
    fn emit_edges(
        &self,
        repo: &RepoContext,
        graph: &SemanticGraph,
    ) -> Vec<crate::graph::ResolvedEdge>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoContext {
    pub root: PathBuf,
}

impl RepoContext {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// First resolved import edge per `(source file, line)`, preserving edge order.
///
/// Runtime plugins resolve import bindings by locating the import edge that
/// matches a reference's file/line. Doing that with a linear scan per
/// reference is O(references x edges) and dominates whole-project analysis on
/// large repositories; this index makes each lookup O(1).
pub(crate) fn first_import_edge_by_location(
    graph: &SemanticGraph,
) -> HashMap<(&Path, usize), &ResolvedEdge> {
    let mut edges_by_location = HashMap::new();
    for edge in graph
        .resolved_edges
        .iter()
        .filter(|edge| edge.kind == ReferenceKind::Import)
    {
        edges_by_location
            .entry((edge.source_file_path.as_path(), edge.line))
            .or_insert(edge);
    }
    edges_by_location
}

/// Strip container/namespace qualifiers and braces from a symbol name,
/// keeping the trailing leaf segment (`Foo\Bar::baz` -> `baz`).
pub(crate) fn leaf_symbol_name(name: &str) -> String {
    name.trim()
        .trim_matches(&['{', '}'][..])
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

/// Map `(file, imported binding name)` to the `(symbol id, defining file)` of
/// the import target, keeping only symbols accepted by `accepts_symbol`.
pub(crate) fn import_targets_by_binding(
    graph: &SemanticGraph,
    symbols_by_id: &HashMap<String, &SymbolNode>,
    accepts_symbol: impl Fn(&SymbolNode) -> bool,
) -> HashMap<(PathBuf, String), (String, PathBuf)> {
    let mut targets = HashMap::new();
    let import_edges_by_location = first_import_edge_by_location(graph);

    for reference in graph
        .references
        .iter()
        .filter(|reference| reference.kind == ReferenceKind::Import)
    {
        let binding_name = reference
            .binding_name
            .clone()
            .unwrap_or_else(|| leaf_symbol_name(&reference.target_name));
        let Some(resolved_import) = import_edges_by_location
            .get(&(reference.file_path.as_path(), reference.line))
            .copied()
        else {
            continue;
        };
        let Some(symbol) = symbols_by_id.get(&resolved_import.target_symbol_id) else {
            continue;
        };
        if !accepts_symbol(symbol) {
            continue;
        }
        targets.insert(
            (reference.file_path.clone(), binding_name),
            (symbol.id.clone(), symbol.file_path.clone()),
        );
    }

    targets
}

pub fn apply_runtime_plugins(repo: &RepoContext, graph: &mut SemanticGraph) {
    for plugin in default_runtime_plugins() {
        for edge in plugin.emit_edges(repo, graph) {
            graph.add_resolved_edge(edge);
        }
    }
}

pub fn built_in_runtime_plugins() -> &'static [RuntimePluginDescriptor] {
    &[
        RuntimePluginDescriptor {
            id: "queue_dispatch",
            description: "Emit runtime dispatch edges for framework-style queued job calls such as Job::dispatch(...).",
        },
        RuntimePluginDescriptor {
            id: "laravel_container",
            description: "Emit framework container-resolution edges for Laravel app()/make()/bound() style dependency lookups.",
        },
        RuntimePluginDescriptor {
            id: "signal_callbacks",
            description: "Emit runtime publish-subscribe edges for generic Signal/connect/send and @receiver(...) callback registration patterns.",
        },
        RuntimePluginDescriptor {
            id: "wordpress_hooks",
            description: "Emit framework publish-subscribe edges for WordPress hook registration and dispatch.",
        },
    ]
}

fn default_runtime_plugins() -> Vec<Box<dyn RuntimePlugin>> {
    vec![
        Box::new(queue::QueueDispatchPlugin),
        Box::new(container::ContainerResolutionPlugin),
        Box::new(signals::SignalCallbacksPlugin),
        Box::new(wordpress::WordPressHooksPlugin),
    ]
}
