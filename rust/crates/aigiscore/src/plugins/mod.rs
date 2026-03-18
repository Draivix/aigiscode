pub mod container;
pub mod queue;
pub mod wordpress;

use crate::graph::SemanticGraph;
use std::path::{Path, PathBuf};

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

pub fn apply_runtime_plugins(repo: &RepoContext, graph: &mut SemanticGraph) {
    for plugin in default_runtime_plugins() {
        for edge in plugin.emit_edges(repo, graph) {
            graph.add_resolved_edge(edge);
        }
    }
}

fn default_runtime_plugins() -> Vec<Box<dyn RuntimePlugin>> {
    vec![
        Box::new(queue::QueueDispatchPlugin),
        Box::new(container::ContainerResolutionPlugin),
        Box::new(wordpress::WordPressHooksPlugin),
    ]
}
