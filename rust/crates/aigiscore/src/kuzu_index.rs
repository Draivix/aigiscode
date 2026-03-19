use crate::artifacts::default_output_dir;
use crate::graph::{
    EdgeOrigin, EdgeStrength, FileNode, GraphLayer, ReferenceKind, RelationKind, ResolutionTier,
    SemanticGraph, SymbolKind, Visibility,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const KUZU_DB_NAME: &str = "graph.kuzu";

const NODE_TABLE_NAME: &str = "CodeNode";
const REL_TABLE_NAME: &str = "CodeRelation";

const NODE_PATH_ENV: &str = "AIGISCODE_NODE_PATH";
const NODE_MODULES_ENV: &str = "AIGISCODE_NODE_MODULES";
const NODE_HELPER_RELATIVE_PATH: &str = "tools/kuzu_bridge.mjs";

#[derive(Debug, Error)]
pub enum KuzuIndexError {
    #[error("failed to prepare Kuzu artifact directory {path}: {source}")]
    PrepareDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to clean previous Kuzu database {path}: {source}")]
    CleanupDb {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to create Kuzu temp export directory {path}: {source}")]
    TempDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write Kuzu CSV {path}: {source}")]
    WriteCsv {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to run Node for Kuzu integration: {0}")]
    NodeSpawn(#[source] std::io::Error),
    #[error("Kuzu helper failed: {0}")]
    HelperFailure(String),
    #[error("failed to parse Kuzu helper output: {0}")]
    ParseHelper(#[from] serde_json::Error),
    #[error("could not locate a usable Node Kuzu module path; set {NODE_PATH_ENV} or keep ../nexus/gitnexus/node_modules available")]
    MissingNodePath,
    #[error("failed to locate Kuzu bridge script at {0}")]
    MissingHelper(PathBuf),
}

#[derive(Debug, Clone, Serialize)]
pub struct KuzuQueryOutput {
    pub db_path: PathBuf,
    pub columns: Vec<String>,
    pub rows: Vec<JsonMap<String, JsonValue>>,
    pub row_count: usize,
}

#[derive(Debug, Deserialize)]
struct HelperQueryOutput {
    columns: Vec<String>,
    rows: Vec<JsonMap<String, JsonValue>>,
    row_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyGraphArtifact {
    pub view: &'static str,
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceGraphArtifact {
    pub view: &'static str,
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<EvidenceEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub qualified_name: String,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_symbol_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_parameter_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub type_name: String,
    pub reference_kind: String,
    pub relation_kind: String,
    pub layer: String,
    pub strength: String,
    pub origin: String,
    pub resolution_tier: String,
    pub confidence_millis: u16,
    pub reason: String,
    pub line: usize,
    pub occurrence_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_symbol_id: Option<String>,
    pub target_symbol_id: String,
    pub source_file_path: String,
    pub target_file_path: String,
    pub type_name: String,
    pub reference_kind: String,
    pub relation_kind: String,
    pub layer: String,
    pub strength: String,
    pub origin: String,
    pub resolution_tier: String,
    pub confidence_millis: u16,
    pub reason: String,
    pub line: usize,
}

pub fn default_kuzu_path(root: &Path, output_dir: Option<&Path>) -> PathBuf {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(root));
    output_dir.join(KUZU_DB_NAME)
}

pub fn write_semantic_graph_kuzu_artifact(
    root: &Path,
    graph: &SemanticGraph,
    output_dir: Option<&Path>,
) -> Result<PathBuf, KuzuIndexError> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(root));
    fs::create_dir_all(&output_dir).map_err(|source| KuzuIndexError::PrepareDir {
        path: output_dir.clone(),
        source,
    })?;
    let db_path = output_dir.join(KUZU_DB_NAME);
    cleanup_db_path(&db_path)?;

    let temp_dir = unique_temp_dir(&output_dir);
    fs::create_dir_all(&temp_dir).map_err(|source| KuzuIndexError::TempDir {
        path: temp_dir.clone(),
        source,
    })?;
    let node_csv = temp_dir.join("nodes.csv");
    let rel_csv = temp_dir.join("relations.csv");
    write_nodes_csv(&node_csv, graph)?;
    write_relations_csv(&rel_csv, graph)?;

    run_bridge_command(&[
        String::from("materialize"),
        db_path.display().to_string(),
        node_csv.display().to_string(),
        rel_csv.display().to_string(),
    ])?;

    let _ = fs::remove_file(&node_csv);
    let _ = fs::remove_file(&rel_csv);
    let _ = fs::remove_dir(&temp_dir);

    Ok(db_path)
}

pub fn build_dependency_graph_artifact(graph: &SemanticGraph) -> DependencyGraphArtifact {
    let nodes = export_graph_nodes(graph);
    let symbol_lookup = graph
        .symbols
        .iter()
        .map(|symbol| (symbol.id.clone(), (symbol.kind, symbol.file_path.clone())))
        .collect::<HashMap<_, _>>();
    let edges = aggregate_dependency_edges(graph, &symbol_lookup)
        .into_iter()
        .map(|aggregate| DependencyEdge {
            from: aggregate.from,
            to: aggregate.to,
            type_name: aggregate.type_name,
            reference_kind: aggregate.reference_kind,
            relation_kind: aggregate.relation_kind,
            layer: aggregate.layer,
            strength: aggregate.strength,
            origin: aggregate.origin,
            resolution_tier: aggregate.resolution_tier,
            confidence_millis: aggregate.confidence_millis,
            reason: aggregate.reason,
            line: aggregate.line,
            occurrence_count: aggregate.occurrence_count,
        })
        .collect::<Vec<_>>();

    DependencyGraphArtifact {
        view: "dependency_view",
        nodes,
        edges,
    }
}

pub fn build_evidence_graph_artifact(graph: &SemanticGraph) -> EvidenceGraphArtifact {
    let nodes = export_graph_nodes(graph);
    let symbol_lookup = graph
        .symbols
        .iter()
        .map(|symbol| (symbol.id.clone(), (symbol.kind, symbol.file_path.clone())))
        .collect::<HashMap<_, _>>();
    let mut edges = graph
        .resolved_edges
        .iter()
        .enumerate()
        .map(|(index, edge)| EvidenceEdge {
            id: format!(
                "{}:{}:{}:{}:{}",
                display_path(&edge.source_file_path),
                edge.line,
                relation_type_name(edge.relation_kind),
                edge.target_symbol_id,
                index
            ),
            from: export_node_id(
                edge.source_symbol_id.as_deref(),
                &edge.source_file_path,
                &symbol_lookup,
            ),
            to: export_node_id(
                Some(edge.target_symbol_id.as_str()),
                &edge.target_file_path,
                &symbol_lookup,
            ),
            source_symbol_id: edge.source_symbol_id.clone(),
            target_symbol_id: edge.target_symbol_id.clone(),
            source_file_path: display_path(&edge.source_file_path),
            target_file_path: display_path(&edge.target_file_path),
            type_name: relation_type_name(edge.relation_kind),
            reference_kind: reference_kind_name(edge.kind),
            relation_kind: relation_kind_name(edge.relation_kind),
            layer: layer_name(edge.layer),
            strength: strength_name(edge.strength),
            origin: origin_name(edge.origin),
            resolution_tier: resolution_tier_name(edge.resolution_tier),
            confidence_millis: edge.confidence_millis,
            reason: edge.reason.clone(),
            line: edge.line,
        })
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then(left.to.cmp(&right.to))
            .then(left.type_name.cmp(&right.type_name))
            .then(left.line.cmp(&right.line))
            .then(left.id.cmp(&right.id))
    });

    EvidenceGraphArtifact {
        view: "evidence_view",
        nodes,
        edges,
    }
}

pub fn query_kuzu(db_path: &Path, cypher: &str) -> Result<KuzuQueryOutput, KuzuIndexError> {
    let stdout = run_bridge_command(&[
        String::from("query"),
        db_path.display().to_string(),
        String::from(cypher),
    ])?;
    let parsed: HelperQueryOutput = serde_json::from_str(&stdout)?;
    Ok(KuzuQueryOutput {
        db_path: db_path.to_path_buf(),
        columns: parsed.columns,
        rows: parsed.rows,
        row_count: parsed.row_count,
    })
}

pub fn schema_reference_markdown() -> String {
    [
        "AigisCode Kuzu schema:\n",
        &format!("- Single node table: `{NODE_TABLE_NAME}`\n"),
        &format!("- Single relationship table: `{REL_TABLE_NAME}`\n\n"),
        "Node columns:\n",
        "- `id`, `kind`, `name`, `qualifiedName`, `filePath`, `language`, `parentSymbolId`, `ownerTypeName`, `returnTypeName`, `visibility`, `parameterCount`, `requiredParameterCount`, `startLine`, `endLine`\n\n",
        "Relationship columns:\n",
        "- `type`, `referenceKind`, `relationKind`, `layer`, `strength`, `origin`, `resolutionTier`, `confidenceMillis`, `reason`, `line`, `occurrenceCount`\n\n",
        "Export shape:\n",
        "- This Kuzu artifact is the normalized `dependency_view`, not the raw evidence graph. Synthetic `MODULE` nodes and `CONTAINS` edges are omitted, module-targeted edges are remapped to file nodes, and repeated dependencies are collapsed with `occurrenceCount` while JSON artifacts retain raw call-site evidence.\n\n",
        "Runtime requirement:\n",
        &format!(
            "- The helper looks for the Node `kuzu` package via `{NODE_PATH_ENV}` or `../nexus/gitnexus/node_modules`.\n\n"
        ),
        "Useful queries:\n",
        "```cypher\n",
        "MATCH (n:CodeNode) RETURN n.kind, count(*) AS count ORDER BY count DESC;\n",
        "MATCH ()-[r:CodeRelation]->() RETURN r.type AS type, count(*) AS count ORDER BY count DESC;\n",
        "MATCH (a:CodeNode)-[r:CodeRelation {type: 'CALL'}]->(b:CodeNode) RETURN a.name, b.name LIMIT 25;\n",
        "MATCH (a:CodeNode)-[r:CodeRelation]->(b:CodeNode)\n",
        "WHERE r.layer IN ['runtime', 'framework']\n",
        "RETURN a.filePath, r.type, b.filePath LIMIT 25;\n",
        "```\n",
    ]
    .concat()
}

fn run_bridge_command(args: &[String]) -> Result<String, KuzuIndexError> {
    let helper_path = helper_script_path()?;
    let node_path = discover_node_path()?;
    for attempt in 0..5 {
        let output = Command::new("node")
            .arg(&helper_path)
            .args(args)
            .env("NODE_PATH", &node_path)
            .env(NODE_MODULES_ENV, &node_path)
            .output()
            .map_err(KuzuIndexError::NodeSpawn)?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
        }
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        if detail.contains("Could not set lock on file") && attempt < 4 {
            thread::sleep(std::time::Duration::from_millis(150 * (attempt + 1) as u64));
            continue;
        }
        return Err(KuzuIndexError::HelperFailure(detail));
    }
    Err(KuzuIndexError::HelperFailure(String::from(
        "Kuzu helper failed after retries",
    )))
}

fn discover_node_path() -> Result<String, KuzuIndexError> {
    if let Ok(value) = env::var(NODE_PATH_ENV) {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    for candidate in [
        repo_root().join("node_modules"),
        repo_root().join("../nexus/gitnexus/node_modules"),
    ] {
        if candidate.exists() {
            return Ok(candidate.display().to_string());
        }
    }
    Err(KuzuIndexError::MissingNodePath)
}

fn helper_script_path() -> Result<PathBuf, KuzuIndexError> {
    let path = repo_root().join(NODE_HELPER_RELATIVE_PATH);
    if path.exists() {
        Ok(path)
    } else {
        Err(KuzuIndexError::MissingHelper(path))
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

fn cleanup_db_path(db_path: &Path) -> Result<(), KuzuIndexError> {
    if let Ok(metadata) = fs::metadata(db_path) {
        if metadata.is_dir() {
            fs::remove_dir_all(db_path).map_err(|source| KuzuIndexError::CleanupDb {
                path: db_path.to_path_buf(),
                source,
            })?;
        } else {
            fs::remove_file(db_path).map_err(|source| KuzuIndexError::CleanupDb {
                path: db_path.to_path_buf(),
                source,
            })?;
        }
    }

    for suffix in [".wal", ".lock"] {
        let sidecar = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        if sidecar.exists() {
            fs::remove_file(&sidecar).map_err(|source| KuzuIndexError::CleanupDb {
                path: sidecar,
                source,
            })?;
        }
    }
    Ok(())
}

fn unique_temp_dir(output_dir: &Path) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    output_dir.join(format!(".kuzu-export-{millis}-{}", std::process::id()))
}

fn write_nodes_csv(path: &Path, graph: &SemanticGraph) -> Result<(), KuzuIndexError> {
    let nodes = export_graph_nodes(graph);
    let mut lines = Vec::with_capacity(nodes.len() + 1);
    lines.push(String::from(
        "\"id\",\"kind\",\"name\",\"qualifiedName\",\"filePath\",\"language\",\"parentSymbolId\",\"ownerTypeName\",\"returnTypeName\",\"visibility\",\"parameterCount\",\"requiredParameterCount\",\"startLine\",\"endLine\"",
    ));
    for node in nodes {
        lines.push(csv_row([
            node.id,
            node.kind,
            node.name,
            node.qualified_name,
            node.file_path,
            node.language.unwrap_or_default(),
            node.parent_symbol_id.unwrap_or_default(),
            node.owner_type_name.unwrap_or_default(),
            node.return_type_name.unwrap_or_default(),
            node.visibility.unwrap_or_default(),
            node.parameter_count.unwrap_or_default().to_string(),
            node.required_parameter_count
                .unwrap_or_default()
                .to_string(),
            node.start_line.unwrap_or_default().to_string(),
            node.end_line.unwrap_or_default().to_string(),
        ]));
    }

    fs::write(path, lines.join("\n")).map_err(|source| KuzuIndexError::WriteCsv {
        path: path.to_path_buf(),
        source,
    })
}

fn export_graph_nodes(graph: &SemanticGraph) -> Vec<DependencyNode> {
    let mut nodes = Vec::with_capacity(graph.files.len() + graph.symbols.len());
    let mut seen = HashSet::new();

    for file in &graph.files {
        let id = file_node_id(file);
        if seen.insert(id.clone()) {
            nodes.push(DependencyNode {
                id,
                kind: String::from("FILE"),
                name: file_name(&file.path),
                qualified_name: display_path(&file.path),
                file_path: display_path(&file.path),
                language: Some(language_name(file)),
                parent_symbol_id: None,
                owner_type_name: None,
                return_type_name: None,
                visibility: None,
                parameter_count: Some(0),
                required_parameter_count: Some(0),
                start_line: Some(0),
                end_line: Some(0),
            });
        }
    }

    for symbol in &graph.symbols {
        if symbol.kind == SymbolKind::Module {
            continue;
        }
        if seen.insert(symbol.id.clone()) {
            nodes.push(DependencyNode {
                id: symbol.id.clone(),
                kind: symbol_kind_name(symbol.kind),
                name: symbol.name.clone(),
                qualified_name: symbol.qualified_name.clone(),
                file_path: display_path(&symbol.file_path),
                language: None,
                parent_symbol_id: symbol.parent_symbol_id.clone(),
                owner_type_name: symbol.owner_type_name.clone(),
                return_type_name: symbol.return_type_name.clone(),
                visibility: Some(visibility_name(symbol.visibility)),
                parameter_count: Some(symbol.parameter_count),
                required_parameter_count: Some(symbol.required_parameter_count),
                start_line: Some(symbol.start_line),
                end_line: Some(symbol.end_line),
            });
        }
    }

    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    nodes
}

fn write_relations_csv(path: &Path, graph: &SemanticGraph) -> Result<(), KuzuIndexError> {
    let mut lines = Vec::with_capacity(graph.resolved_edges.len() + 1);
    lines.push(String::from(
        "\"from\",\"to\",\"type\",\"referenceKind\",\"relationKind\",\"layer\",\"strength\",\"origin\",\"resolutionTier\",\"confidenceMillis\",\"reason\",\"line\",\"occurrenceCount\"",
    ));

    let symbol_lookup = graph
        .symbols
        .iter()
        .map(|symbol| (symbol.id.clone(), (symbol.kind, symbol.file_path.clone())))
        .collect::<HashMap<_, _>>();

    for aggregate in aggregate_dependency_edges(graph, &symbol_lookup) {
        lines.push(csv_row([
            aggregate.from,
            aggregate.to,
            aggregate.type_name,
            aggregate.reference_kind,
            aggregate.relation_kind,
            aggregate.layer,
            aggregate.strength,
            aggregate.origin,
            aggregate.resolution_tier,
            aggregate.confidence_millis.to_string(),
            aggregate.reason,
            aggregate.line.to_string(),
            aggregate.occurrence_count.to_string(),
        ]));
    }

    fs::write(path, lines.join("\n")).map_err(|source| KuzuIndexError::WriteCsv {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DependencyEdgeKey {
    from: String,
    to: String,
    type_name: String,
    reference_kind: String,
    relation_kind: String,
    layer: String,
}

#[derive(Debug, Clone)]
struct DependencyEdgeAggregate {
    from: String,
    to: String,
    type_name: String,
    reference_kind: String,
    relation_kind: String,
    layer: String,
    strength: String,
    origin: String,
    resolution_tier: String,
    confidence_millis: u16,
    reason: String,
    line: usize,
    occurrence_count: usize,
}

fn aggregate_dependency_edges(
    graph: &SemanticGraph,
    symbol_lookup: &HashMap<String, (SymbolKind, PathBuf)>,
) -> Vec<DependencyEdgeAggregate> {
    let mut aggregated = HashMap::<DependencyEdgeKey, DependencyEdgeAggregate>::new();

    for edge in &graph.resolved_edges {
        let from = export_node_id(
            edge.source_symbol_id.as_deref(),
            &edge.source_file_path,
            symbol_lookup,
        );
        let to = export_node_id(
            Some(edge.target_symbol_id.as_str()),
            &edge.target_file_path,
            symbol_lookup,
        );
        let (type_name, relation_kind) = dependency_relation_names(edge, symbol_lookup);
        let key = DependencyEdgeKey {
            from: from.clone(),
            to: to.clone(),
            type_name: type_name.clone(),
            reference_kind: reference_kind_name(edge.kind),
            relation_kind: relation_kind.clone(),
            layer: layer_name(edge.layer),
        };
        let aggregate = aggregated
            .entry(key.clone())
            .or_insert_with(|| DependencyEdgeAggregate {
                from,
                to,
                type_name,
                reference_kind: key.reference_kind.clone(),
                relation_kind,
                layer: key.layer.clone(),
                strength: strength_name(edge.strength),
                origin: origin_name(edge.origin),
                resolution_tier: resolution_tier_name(edge.resolution_tier),
                confidence_millis: edge.confidence_millis,
                reason: edge.reason.clone(),
                line: edge.line,
                occurrence_count: 0,
            });
        aggregate.occurrence_count += 1;
        if edge.line < aggregate.line {
            aggregate.line = edge.line;
        }
        if edge.confidence_millis > aggregate.confidence_millis {
            aggregate.confidence_millis = edge.confidence_millis;
        }
        if strength_rank(edge.strength) > strength_rank_name(&aggregate.strength) {
            aggregate.strength = strength_name(edge.strength);
        }
        if tier_rank(edge.resolution_tier) > tier_rank_name(&aggregate.resolution_tier) {
            aggregate.resolution_tier = resolution_tier_name(edge.resolution_tier);
        }
        if edge.reason != aggregate.reason && !aggregate.reason.contains(edge.reason.as_str()) {
            aggregate.reason.push_str(" | ");
            aggregate.reason.push_str(&edge.reason);
        }
        if aggregate.origin != origin_name(edge.origin) && aggregate.origin != "mixed" {
            aggregate.origin = String::from("mixed");
        }
    }

    let mut values = aggregated.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then(left.to.cmp(&right.to))
            .then(left.type_name.cmp(&right.type_name))
    });
    values
}

fn dependency_relation_names(
    edge: &crate::graph::ResolvedEdge,
    symbol_lookup: &HashMap<String, (SymbolKind, PathBuf)>,
) -> (String, String) {
    let target_kind = symbol_lookup
        .get(&edge.target_symbol_id)
        .map(|(kind, _)| *kind);
    if edge.relation_kind == RelationKind::Call
        && edge.kind == ReferenceKind::Call
        && matches!(target_kind, Some(SymbolKind::Class | SymbolKind::Struct))
    {
        return (String::from("INSTANTIATE"), String::from("instantiate"));
    }
    if edge.relation_kind == RelationKind::Call
        && edge.kind == ReferenceKind::Call
        && edge.source_symbol_id.is_none()
    {
        return (
            String::from("TOP_LEVEL_CALL"),
            String::from("top_level_call"),
        );
    }
    (
        relation_type_name(edge.relation_kind),
        relation_kind_name(edge.relation_kind),
    )
}

fn export_node_id(
    symbol_id: Option<&str>,
    file_path: &Path,
    symbol_lookup: &HashMap<String, (SymbolKind, PathBuf)>,
) -> String {
    let Some(symbol_id) = symbol_id else {
        return file_node_id_from_path(file_path);
    };
    match symbol_lookup.get(symbol_id) {
        Some((SymbolKind::Module, module_file_path)) => file_node_id_from_path(module_file_path),
        Some(_) => String::from(symbol_id),
        None => file_node_id_from_path(file_path),
    }
}

fn csv_row<const N: usize>(values: [String; N]) -> String {
    values
        .into_iter()
        .map(|value| format!("\"{}\"", value.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(",")
}

fn file_node_id(file: &FileNode) -> String {
    file_node_id_from_path(&file.path)
}

fn file_node_id_from_path(path: &Path) -> String {
    format!("file:{}", display_path(path))
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(String::from)
        .unwrap_or_else(|| display_path(path))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn language_name(file: &FileNode) -> String {
    format!("{:?}", file.language).to_ascii_uppercase()
}

fn symbol_kind_name(kind: SymbolKind) -> String {
    format!("{kind:?}").to_ascii_uppercase()
}

fn visibility_name(visibility: Visibility) -> String {
    format!("{visibility:?}").to_ascii_lowercase()
}

fn reference_kind_name(kind: ReferenceKind) -> String {
    format!("{kind:?}").to_ascii_uppercase()
}

fn relation_kind_name(kind: RelationKind) -> String {
    format!("{kind:?}").to_ascii_lowercase()
}

fn relation_type_name(kind: RelationKind) -> String {
    format!("{kind:?}").to_ascii_uppercase()
}

fn layer_name(layer: GraphLayer) -> String {
    match layer {
        GraphLayer::Structural => String::from("structural"),
        GraphLayer::Runtime => String::from("runtime"),
        GraphLayer::Framework => String::from("framework"),
        GraphLayer::PolicyOverlay => String::from("policy_overlay"),
    }
}

fn strength_name(strength: EdgeStrength) -> String {
    match strength {
        EdgeStrength::Hard => String::from("hard"),
        EdgeStrength::Inferred => String::from("inferred"),
        EdgeStrength::Dynamic => String::from("dynamic"),
        EdgeStrength::Synthetic => String::from("synthetic"),
    }
}

fn strength_rank(strength: EdgeStrength) -> u8 {
    match strength {
        EdgeStrength::Synthetic => 0,
        EdgeStrength::Dynamic => 1,
        EdgeStrength::Inferred => 2,
        EdgeStrength::Hard => 3,
    }
}

fn strength_rank_name(value: &str) -> u8 {
    match value {
        "synthetic" => 0,
        "dynamic" => 1,
        "inferred" => 2,
        "hard" => 3,
        _ => 0,
    }
}

fn origin_name(origin: EdgeOrigin) -> String {
    match origin {
        EdgeOrigin::Resolver => String::from("resolver"),
        EdgeOrigin::Plugin => String::from("plugin"),
        EdgeOrigin::Policy => String::from("policy"),
    }
}

fn resolution_tier_name(tier: ResolutionTier) -> String {
    match tier {
        ResolutionTier::SameFile => String::from("same_file"),
        ResolutionTier::ImportScoped => String::from("import_scoped"),
        ResolutionTier::Global => String::from("global"),
    }
}

fn tier_rank(tier: ResolutionTier) -> u8 {
    match tier {
        ResolutionTier::Global => 0,
        ResolutionTier::ImportScoped => 1,
        ResolutionTier::SameFile => 2,
    }
}

fn tier_rank_name(value: &str) -> u8 {
    match value {
        "global" => 0,
        "import_scoped" => 1,
        "same_file" => 2,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_dependency_graph_artifact, build_evidence_graph_artifact, query_kuzu,
        write_semantic_graph_kuzu_artifact,
    };
    use crate::ingestion::pipeline::build_semantic_graph_project;
    use crate::ingestion::scan::ScanConfig;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn materializes_semantic_graph_and_answers_cypher() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            "mod models;\nuse crate::models::User;\nfn main() { let _ = User; }\n",
        )
        .unwrap();
        fs::write(fixture.join("src/models.rs"), "pub struct User;\n").unwrap();

        let project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let db_path =
            write_semantic_graph_kuzu_artifact(&project.root, &project.semantic_graph, None)
                .unwrap();

        let node_counts = query_kuzu(
            &db_path,
            "MATCH (n:CodeNode) RETURN n.kind AS kind, count(*) AS count ORDER BY count DESC",
        )
        .unwrap();
        assert!(!node_counts.rows.is_empty());
        assert!(!node_counts.rows.iter().any(|row| {
            row.get("kind") == Some(&serde_json::Value::String(String::from("MODULE")))
        }));

        let edge_counts = query_kuzu(
            &db_path,
            "MATCH ()-[r:CodeRelation]->() RETURN r.type AS type, count(*) AS count ORDER BY count DESC",
        )
        .unwrap();
        assert!(!edge_counts.rows.iter().any(|row| {
            row.get("type") == Some(&serde_json::Value::String(String::from("CONTAINS")))
        }));

        let import_occurrences = query_kuzu(
            &db_path,
            "MATCH ()-[r:CodeRelation {type: 'IMPORT'}]->() RETURN r.occurrenceCount AS occurrences LIMIT 1",
        )
        .unwrap();
        assert_eq!(import_occurrences.row_count, 1);
        assert!(import_occurrences.rows[0]
            .get("occurrences")
            .is_some_and(|value| value.as_i64().unwrap_or_default() >= 1));
    }

    #[test]
    fn graph_projection_artifacts_are_stable_and_exclude_module_nodes() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            "mod models;\nuse crate::models::User;\nfn main() { let _ = User; let _ = User; }\n",
        )
        .unwrap();
        fs::write(fixture.join("src/models.rs"), "pub struct User;\n").unwrap();

        let project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let dependency_graph = build_dependency_graph_artifact(&project.semantic_graph);
        let evidence_graph = build_evidence_graph_artifact(&project.semantic_graph);

        assert_eq!(dependency_graph.view, "dependency_view");
        assert_eq!(evidence_graph.view, "evidence_view");
        assert!(dependency_graph
            .nodes
            .windows(2)
            .all(|window| window[0].id <= window[1].id));
        assert!(evidence_graph
            .nodes
            .windows(2)
            .all(|window| window[0].id <= window[1].id));
        assert!(dependency_graph
            .nodes
            .iter()
            .all(|node| node.kind != "MODULE"));
        assert!(evidence_graph
            .nodes
            .iter()
            .all(|node| node.kind != "MODULE"));
        assert!(dependency_graph.edges.windows(2).all(|window| (
            window[0].from.as_str(),
            window[0].to.as_str(),
            window[0].type_name.as_str()
        ) <= (
            window[1].from.as_str(),
            window[1].to.as_str(),
            window[1].type_name.as_str()
        )));
        assert!(evidence_graph.edges.windows(2).all(|window| {
            (
                window[0].from.as_str(),
                window[0].to.as_str(),
                window[0].type_name.as_str(),
                window[0].line,
                window[0].id.as_str(),
            ) <= (
                window[1].from.as_str(),
                window[1].to.as_str(),
                window[1].type_name.as_str(),
                window[1].line,
                window[1].id.as_str(),
            )
        }));
        assert!(evidence_graph.edges.len() >= dependency_graph.edges.len());
    }

    #[test]
    fn dependency_view_projects_constructor_calls_as_instantiate_edges() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/index.php"),
            "<?php\nclass User {}\nfunction build_user(): User { return new User(); }\n",
        )
        .unwrap();

        let project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let dependency_graph = build_dependency_graph_artifact(&project.semantic_graph);
        let evidence_graph = build_evidence_graph_artifact(&project.semantic_graph);

        assert!(dependency_graph
            .edges
            .iter()
            .any(|edge| edge.type_name == "INSTANTIATE" && edge.relation_kind == "instantiate"));
        assert!(evidence_graph
            .edges
            .iter()
            .any(|edge| edge.type_name == "CALL" && edge.relation_kind == "call"));
    }

    #[test]
    fn dependency_view_projects_file_scope_calls_as_top_level_edges() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/index.php"),
            "<?php\nhelper();\nfunction helper() {}\n",
        )
        .unwrap();

        let project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let dependency_graph = build_dependency_graph_artifact(&project.semantic_graph);
        let evidence_graph = build_evidence_graph_artifact(&project.semantic_graph);

        assert!(dependency_graph.edges.iter().any(|edge| {
            edge.type_name == "TOP_LEVEL_CALL" && edge.relation_kind == "top_level_call"
        }));
        assert!(evidence_graph
            .edges
            .iter()
            .any(|edge| edge.type_name == "CALL" && edge.relation_kind == "call"));
    }

    fn create_fixture() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "aigiscode-kuzu-test-{nanos}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
}
