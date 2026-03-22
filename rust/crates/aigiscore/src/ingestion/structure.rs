use crate::ingestion::scan::ScannedFile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureNodeKind {
    Directory,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureNode {
    pub path: PathBuf,
    pub kind: StructureNodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainsEdge {
    pub parent: PathBuf,
    pub child: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StructureGraph {
    pub nodes: Vec<StructureNode>,
    pub contains_edges: Vec<ContainsEdge>,
}

pub fn build_structure_graph(files: &[ScannedFile]) -> StructureGraph {
    let mut directory_nodes = BTreeSet::new();
    let mut file_nodes = BTreeSet::new();
    let mut edges = BTreeSet::new();

    directory_nodes.insert(PathBuf::new());

    for file in files {
        let mut current = PathBuf::new();
        let components: Vec<_> = file.relative_path.components().collect();

        for component in components.iter().take(components.len().saturating_sub(1)) {
            current.push(component.as_os_str());
            directory_nodes.insert(current.clone());
        }

        file_nodes.insert(file.relative_path.clone());

        let mut parent = PathBuf::new();
        for component in &components {
            let mut child = parent.clone();
            child.push(component.as_os_str());
            edges.insert((parent.clone(), child.clone()));
            parent = child;
        }
    }

    let mut nodes = Vec::with_capacity(directory_nodes.len() + file_nodes.len());
    nodes.extend(directory_nodes.into_iter().map(|path| StructureNode {
        path,
        kind: StructureNodeKind::Directory,
    }));
    nodes.extend(file_nodes.into_iter().map(|path| StructureNode {
        path,
        kind: StructureNodeKind::File,
    }));

    let contains_edges = edges
        .into_iter()
        .map(|(parent, child)| ContainsEdge { parent, child })
        .collect();

    StructureGraph {
        nodes,
        contains_edges,
    }
}

pub fn parent_path(path: &Path) -> Option<PathBuf> {
    path.parent().map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::{build_structure_graph, parent_path, StructureNodeKind};
    use crate::ingestion::scan::ScannedFile;
    use std::path::{Path, PathBuf};

    #[test]
    fn creates_directory_and_file_nodes() {
        let graph = build_structure_graph(&[
            ScannedFile {
                relative_path: PathBuf::from("src/main.py"),
                size_bytes: 10,
            },
            ScannedFile {
                relative_path: PathBuf::from("src/core/utils.rs"),
                size_bytes: 20,
            },
        ]);

        assert!(graph
            .nodes
            .iter()
            .any(|node| node.path == Path::new("") && node.kind == StructureNodeKind::Directory));
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.path == Path::new("src")
                    && node.kind == StructureNodeKind::Directory)
        );
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.path == Path::new("src/core")
                && node.kind == StructureNodeKind::Directory));
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.path == Path::new("src/main.py")
                && node.kind == StructureNodeKind::File));
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.path == Path::new("src/core/utils.rs")
                && node.kind == StructureNodeKind::File));
    }

    #[test]
    fn emits_contains_edges_for_each_path_segment() {
        let graph = build_structure_graph(&[ScannedFile {
            relative_path: PathBuf::from("src/core/utils.rs"),
            size_bytes: 20,
        }]);

        assert!(graph
            .contains_edges
            .iter()
            .any(|edge| edge.parent == Path::new("") && edge.child == Path::new("src")));
        assert!(graph
            .contains_edges
            .iter()
            .any(|edge| edge.parent == Path::new("src") && edge.child == Path::new("src/core")));
        assert!(graph
            .contains_edges
            .iter()
            .any(|edge| edge.parent == Path::new("src/core")
                && edge.child == Path::new("src/core/utils.rs")));
    }

    #[test]
    fn returns_parent_path_for_nested_paths() {
        assert_eq!(
            parent_path(PathBuf::from("src/core/utils.rs").as_path()),
            Some(PathBuf::from("src/core"))
        );
        assert_eq!(
            parent_path(PathBuf::from("src").as_path()),
            Some(PathBuf::new())
        );
    }
}
