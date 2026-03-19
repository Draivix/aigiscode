pub mod container;
pub mod queue;
pub mod signals;
pub mod wordpress;

use crate::graph::SemanticGraph;
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
