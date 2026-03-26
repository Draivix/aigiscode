use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

pub mod analysis;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    JavaScript,
    Php,
    Python,
    Ruby,
    Rust,
    TypeScript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Class,
    Function,
    Interface,
    Method,
    Struct,
    Enum,
    Trait,
    Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferenceKind {
    Import,
    Call,
    Type,
    Extends,
    Implements,
    Overrides,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallForm {
    Free,
    Member,
    Associated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GraphLayer {
    #[serde(alias = "Structural")]
    Structural,
    #[serde(alias = "Runtime")]
    Runtime,
    #[serde(alias = "Framework")]
    Framework,
    #[serde(alias = "PolicyOverlay")]
    PolicyOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EdgeStrength {
    #[serde(alias = "Hard")]
    Hard,
    #[serde(alias = "Inferred")]
    Inferred,
    #[serde(alias = "Dynamic")]
    Dynamic,
    #[serde(alias = "Synthetic")]
    Synthetic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EdgeOrigin {
    #[serde(alias = "Resolver")]
    Resolver,
    #[serde(alias = "Plugin")]
    Plugin,
    #[serde(alias = "Policy")]
    Policy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RelationKind {
    #[serde(alias = "Import")]
    Import,
    #[serde(alias = "Call")]
    Call,
    #[serde(alias = "Dispatch")]
    Dispatch,
    #[serde(alias = "ContainerResolution")]
    ContainerResolution,
    #[serde(alias = "EventSubscribe")]
    EventSubscribe,
    #[serde(alias = "EventPublish")]
    EventPublish,
    #[serde(alias = "TypeUse")]
    TypeUse,
    #[serde(alias = "Extends")]
    Extends,
    #[serde(alias = "Implements")]
    Implements,
    #[serde(alias = "Overrides")]
    Overrides,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileNode {
    pub path: PathBuf,
    pub language: Language,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolNode {
    pub id: String,
    pub file_path: PathBuf,
    pub kind: SymbolKind,
    pub name: String,
    pub qualified_name: String,
    pub parent_symbol_id: Option<String>,
    pub owner_type_name: Option<String>,
    pub return_type_name: Option<String>,
    pub visibility: Visibility,
    pub parameter_count: usize,
    #[serde(default)]
    pub required_parameter_count: usize,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticReference {
    pub file_path: PathBuf,
    pub enclosing_symbol_id: Option<String>,
    pub kind: ReferenceKind,
    pub target_name: String,
    pub binding_name: Option<String>,
    pub line: usize,
    pub arity: Option<usize>,
    pub receiver_name: Option<String>,
    pub receiver_type_name: Option<String>,
    pub call_form: Option<CallForm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionTier {
    SameFile,
    ImportScoped,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedEdge {
    pub source_file_path: PathBuf,
    pub source_symbol_id: Option<String>,
    pub target_file_path: PathBuf,
    pub target_symbol_id: String,
    pub reference_target_name: Option<String>,
    pub kind: ReferenceKind,
    pub relation_kind: RelationKind,
    pub layer: GraphLayer,
    pub strength: EdgeStrength,
    pub origin: EdgeOrigin,
    pub resolution_tier: ResolutionTier,
    pub confidence_millis: u16,
    pub reason: String,
    pub line: usize,
    pub occurrence_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SemanticGraph {
    pub files: Vec<FileNode>,
    pub symbols: Vec<SymbolNode>,
    pub references: Vec<SemanticReference>,
    pub resolved_edges: Vec<ResolvedEdge>,
}

impl SemanticGraph {
    pub fn add_file(&mut self, file: FileNode) {
        self.files.push(file);
    }

    pub fn add_symbol(&mut self, symbol: SymbolNode) {
        self.symbols.push(symbol);
    }

    pub fn add_reference(&mut self, reference: SemanticReference) {
        self.references.push(reference);
    }

    pub fn add_resolved_edge(&mut self, edge: ResolvedEdge) {
        self.resolved_edges.push(edge.normalized());
    }

    pub fn structural_view(&self) -> Vec<&ResolvedEdge> {
        self.resolved_edges
            .iter()
            .filter(|edge| edge.layer == GraphLayer::Structural)
            .collect()
    }

    pub fn runtime_view(&self) -> Vec<&ResolvedEdge> {
        self.resolved_edges
            .iter()
            .filter(|edge| {
                matches!(
                    edge.layer,
                    GraphLayer::Runtime | GraphLayer::Framework | GraphLayer::PolicyOverlay
                )
            })
            .collect()
    }

    pub fn mixed_view(&self) -> Vec<&ResolvedEdge> {
        self.resolved_edges.iter().collect()
    }
}

impl RelationKind {
    pub fn from_reference_kind(kind: ReferenceKind) -> Self {
        match kind {
            ReferenceKind::Import => Self::Import,
            ReferenceKind::Call => Self::Call,
            ReferenceKind::Type => Self::TypeUse,
            ReferenceKind::Extends => Self::Extends,
            ReferenceKind::Implements => Self::Implements,
            ReferenceKind::Overrides => Self::Overrides,
        }
    }
}

impl EdgeStrength {
    pub fn from_resolution_tier(tier: ResolutionTier) -> Self {
        match tier {
            ResolutionTier::SameFile | ResolutionTier::ImportScoped => Self::Hard,
            ResolutionTier::Global => Self::Inferred,
        }
    }
}

impl ResolvedEdge {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_file_path: PathBuf,
        source_symbol_id: Option<String>,
        target_file_path: PathBuf,
        target_symbol_id: String,
        kind: ReferenceKind,
        resolution_tier: ResolutionTier,
        confidence_millis: u16,
        reason: String,
        line: usize,
    ) -> Self {
        Self {
            source_file_path,
            source_symbol_id,
            target_file_path,
            target_symbol_id,
            reference_target_name: None,
            kind,
            relation_kind: RelationKind::from_reference_kind(kind),
            layer: GraphLayer::Structural,
            strength: EdgeStrength::from_resolution_tier(resolution_tier),
            origin: EdgeOrigin::Resolver,
            resolution_tier,
            confidence_millis,
            reason,
            line,
            occurrence_index: 0,
        }
    }

    pub fn with_reference_identity(mut self, target_name: String, occurrence_index: usize) -> Self {
        self.reference_target_name = Some(target_name);
        self.occurrence_index = occurrence_index;
        self
    }

    pub fn with_metadata(
        mut self,
        relation_kind: RelationKind,
        layer: GraphLayer,
        strength: EdgeStrength,
        origin: EdgeOrigin,
    ) -> Self {
        self.relation_kind = relation_kind;
        self.layer = layer;
        self.strength = strength;
        self.origin = origin;
        self
    }

    pub fn normalized(mut self) -> Self {
        if self.origin == EdgeOrigin::Resolver {
            self.relation_kind = RelationKind::from_reference_kind(self.kind);
        }
        self
    }
}

#[derive(Debug, Deserialize)]
struct ResolvedEdgeSerde {
    source_file_path: PathBuf,
    source_symbol_id: Option<String>,
    target_file_path: PathBuf,
    target_symbol_id: String,
    reference_target_name: Option<String>,
    kind: ReferenceKind,
    relation_kind: Option<RelationKind>,
    layer: Option<GraphLayer>,
    strength: Option<EdgeStrength>,
    origin: Option<EdgeOrigin>,
    resolution_tier: ResolutionTier,
    confidence_millis: u16,
    reason: String,
    line: usize,
    occurrence_index: Option<usize>,
}

impl<'de> Deserialize<'de> for ResolvedEdge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = ResolvedEdgeSerde::deserialize(deserializer)?;
        Ok(Self {
            source_file_path: raw.source_file_path,
            source_symbol_id: raw.source_symbol_id,
            target_file_path: raw.target_file_path,
            target_symbol_id: raw.target_symbol_id,
            reference_target_name: raw.reference_target_name,
            kind: raw.kind,
            relation_kind: raw
                .relation_kind
                .unwrap_or_else(|| RelationKind::from_reference_kind(raw.kind)),
            layer: raw.layer.unwrap_or(GraphLayer::Structural),
            strength: raw
                .strength
                .unwrap_or_else(|| EdgeStrength::from_resolution_tier(raw.resolution_tier)),
            origin: raw.origin.unwrap_or(EdgeOrigin::Resolver),
            resolution_tier: raw.resolution_tier,
            confidence_millis: raw.confidence_millis,
            reason: raw.reason,
            line: raw.line,
            occurrence_index: raw.occurrence_index.unwrap_or(0),
        }
        .normalized())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EdgeOrigin, EdgeStrength, GraphLayer, ReferenceKind, RelationKind, ResolutionTier,
        ResolvedEdge, SemanticGraph,
    };
    use serde_json::json;
    use std::path::PathBuf;

    #[test]
    fn exposes_structural_runtime_and_mixed_edge_views() {
        let mut graph = SemanticGraph::default();
        graph.add_resolved_edge(ResolvedEdge::new(
            PathBuf::from("src/a.rs"),
            None,
            PathBuf::from("src/b.rs"),
            String::from("symbol:src/b.rs"),
            ReferenceKind::Import,
            ResolutionTier::ImportScoped,
            900,
            String::from("test"),
            1,
        ));
        graph.add_resolved_edge(
            ResolvedEdge::new(
                PathBuf::from("src/a.rs"),
                None,
                PathBuf::from("src/jobs.rs"),
                String::from("symbol:src/jobs.rs"),
                ReferenceKind::Call,
                ResolutionTier::Global,
                700,
                String::from("plugin"),
                2,
            )
            .with_metadata(
                RelationKind::Call,
                GraphLayer::Runtime,
                EdgeStrength::Dynamic,
                EdgeOrigin::Plugin,
            ),
        );
        graph.add_resolved_edge(
            ResolvedEdge::new(
                PathBuf::from("src/a.rs"),
                None,
                PathBuf::from("src/policy.rs"),
                String::from("symbol:src/policy.rs"),
                ReferenceKind::Import,
                ResolutionTier::Global,
                650,
                String::from("policy"),
                3,
            )
            .with_metadata(
                RelationKind::Import,
                GraphLayer::PolicyOverlay,
                EdgeStrength::Synthetic,
                EdgeOrigin::Policy,
            ),
        );

        assert_eq!(graph.structural_view().len(), 1);
        assert_eq!(graph.runtime_view().len(), 2);
        assert_eq!(graph.mixed_view().len(), 3);
    }

    #[test]
    fn deserializes_older_edges_by_reconstructing_metadata() {
        let edge: ResolvedEdge = serde_json::from_value(json!({
            "source_file_path": "src/a.rs",
            "source_symbol_id": null,
            "target_file_path": "src/b.rs",
            "target_symbol_id": "symbol:src/b.rs",
            "kind": "Call",
            "resolution_tier": "Global",
            "confidence_millis": 750,
            "reason": "legacy",
            "line": 7
        }))
        .unwrap();

        assert_eq!(edge.relation_kind, RelationKind::Call);
        assert_eq!(edge.layer, GraphLayer::Structural);
        assert_eq!(edge.strength, EdgeStrength::Inferred);
        assert_eq!(edge.origin, EdgeOrigin::Resolver);
    }
}
