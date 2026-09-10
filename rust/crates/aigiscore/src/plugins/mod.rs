pub mod container;
pub mod queue;
pub mod signals;
pub mod wordpress;

use crate::graph::{ResolvedEdge, SemanticGraph, SymbolNode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    fn emit_registrations(&self, _repo: &RepoContext, _graph: &SemanticGraph) -> Vec<crate::graph::RuntimeRegistration> {
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoContext<'a> {
    pub root: PathBuf,
    sources: HashMap<&'a Path, CapturedSource<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedSource<'a> {
    text: &'a str,
    lines: OnceLock<Vec<&'a str>>,
}

impl<'a> RepoContext<'a> {
    /// Borrow the exact source bytes used to build the graph; never reload disk.
    pub fn new(root: impl Into<PathBuf>, sources: &'a [(PathBuf, String)]) -> Self {
        Self {
            root: root.into(),
            sources: sources
                .iter()
                .map(|(path, text)| {
                    (
                        path.as_path(),
                        CapturedSource {
                            text,
                            lines: OnceLock::new(),
                        },
                    )
                })
                .collect(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn source_lines(&self, path: &Path) -> Option<&[&'a str]> {
        let source = self.sources.get(path)?;
        Some(source.lines.get_or_init(|| source.text.lines().collect()))
    }

    pub(crate) fn source_snippet(
        &self,
        path: &Path,
        line: usize,
        context_after: usize,
    ) -> Option<String> {
        let lines = self.source_lines(path)?;
        let start = line.checked_sub(1)?;
        lines.get(start)?;
        let end = start
            .saturating_add(context_after)
            .saturating_add(1)
            .min(lines.len());
        Some(lines[start..end].join(" "))
    }
}

/// Resolved import edges keyed by their original reference, not just a line.
///
/// Runtime plugins resolve import bindings by locating the import edge that
/// matches a reference's file/line. Doing that with a linear scan per
/// reference is O(references x edges) and dominates whole-project analysis on
/// large repositories; this index makes each lookup O(1).
pub(crate) fn import_edges_by_reference(
    graph: &SemanticGraph,
) -> HashMap<(&Path, usize, &str), &ResolvedEdge> {
    let mut edges_by_location = HashMap::new();
    for edge in graph
        .resolved_edges
        .iter()
        .filter(|edge| edge.kind.is_import())
        .filter(|edge| edge.strength != crate::graph::EdgeStrength::Inferred)
    {
        let Some(target_name) = edge.reference_target_name.as_deref() else {
            continue;
        };
        edges_by_location
            .entry((edge.source_file_path.as_path(), edge.line, target_name))
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
/// Class/struct symbols addressable by bare name from their own file — the
/// same-file tier of plugin target resolution, shared by the queue and
/// container plugins.
pub(crate) fn same_file_symbol_targets(
    graph: &SemanticGraph,
) -> HashMap<(PathBuf, String), (String, PathBuf)> {
    graph
        .symbols
        .iter()
        .filter(|symbol| {
            matches!(
                symbol.kind,
                crate::graph::SymbolKind::Class | crate::graph::SymbolKind::Struct
            )
        })
        .map(|symbol| {
            (
                (symbol.file_path.clone(), symbol.name.clone()),
                (symbol.id.clone(), symbol.file_path.clone()),
            )
        })
        .collect()
}

pub(crate) fn import_targets_by_binding(
    graph: &SemanticGraph,
    symbols_by_id: &HashMap<String, &SymbolNode>,
    accepts_symbol: impl Fn(&SymbolNode) -> bool,
) -> HashMap<(PathBuf, String), (String, PathBuf)> {
    let mut targets = HashMap::new();
    let import_edges_by_location = import_edges_by_reference(graph);

    for reference in graph
        .references
        .iter()
        .filter(|reference| reference.kind.is_import())
    {
        let binding_name = reference
            .binding_name
            .clone()
            .unwrap_or_else(|| leaf_symbol_name(&reference.target_name));
        let Some(resolved_import) = import_edges_by_location
            .get(&(
                reference.file_path.as_path(),
                reference.line,
                reference.target_name.as_str(),
            ))
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
    graph.runtime_registrations.clear();
    for plugin in default_runtime_plugins() {
        graph.runtime_registrations.extend(plugin.emit_registrations(repo, graph));
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

#[cfg(test)]
mod tests {
    use super::RepoContext;
    use std::path::{Path, PathBuf};

    #[test]
    fn captured_snippets_handle_crlf_and_invalid_ranges() {
        let path = Path::new("src/hooks.py");
        let sources = vec![
            (path.to_path_buf(), "first\r\nsecond\nthird\n".to_owned()),
            (PathBuf::from("empty.py"), String::new()),
        ];
        let repo = RepoContext::new(".", &sources);
        assert_eq!(repo.source_snippet(path, 1, 1).as_deref(), Some("first second"));
        assert_eq!(
            repo.source_snippet(path, 2, usize::MAX).as_deref(),
            Some("second third")
        );
        for line in [0, 4, usize::MAX] {
            assert_eq!(repo.source_snippet(path, line, 3), None);
        }
        assert_eq!(repo.source_snippet(Path::new("empty.py"), 1, 0), None);
        assert_eq!(repo.source_lines(Path::new("missing.py")), None);
    }
}
