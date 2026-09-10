use super::McpState;
use crate::assessment::behavior::ImplementationComparison;
use crate::assessment::wiring::{ExecutionPathAssessment, WiringSymbol};
use crate::graph::{SemanticGraph, SymbolNode};
use crate::parsing::behavior::FunctionBehavior;
use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ImplementationContextParams {
    /// One to eight exact symbol IDs, qualified names or unambiguous names.
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(super) struct ImplementationContextOutput {
    pub source_snapshot_id: String,
    pub input_coverage: crate::coverage::InputCoverage,
    pub selected: Vec<WiringSymbol>,
    pub execution_paths: ExecutionPathAssessment,
    pub body_count: usize,
    pub body_previews: Vec<FunctionBehavior>,
    pub bodies_truncated: bool,
    pub comparison_count: usize,
    pub comparisons: Vec<ImplementationComparison>,
    pub comparisons_truncated: bool,
}

pub(super) fn resolve_targets<'a>(graph: &'a SemanticGraph, targets: &[String]) -> Result<Vec<&'a SymbolNode>, McpError> {
    if targets.is_empty() || targets.len() > 8 || targets.iter().any(|target| target.trim().is_empty() || target.len() > 4096) {
        return Err(McpError::invalid_params("targets must contain one to eight nonempty symbol IDs or names (at most 4096 bytes each)", None));
    }
    let mut selected = Vec::new();
    for target in targets {
        let implementation = |symbol: &&SymbolNode| matches!(symbol.kind, crate::graph::SymbolKind::Function | crate::graph::SymbolKind::Method
            | crate::graph::SymbolKind::Class | crate::graph::SymbolKind::Interface | crate::graph::SymbolKind::Struct | crate::graph::SymbolKind::Trait | crate::graph::SymbolKind::Enum);
        let exact_id = graph.symbols.iter().filter(implementation).find(|symbol| symbol.id == *target);
        let matches = exact_id.into_iter().chain(graph.symbols.iter().filter(implementation).filter(|symbol| exact_id.is_none()
            && (symbol.name == *target || symbol.qualified_name == *target))).collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(McpError::invalid_params(format!("target {target:?} has {} matches; use an exact ID from find_symbol", matches.len()),
                Some(serde_json::json!({"candidate_ids": matches.iter().take(8).map(|symbol| &symbol.id).collect::<Vec<_>>()}))));
        }
        if !selected.iter().any(|symbol: &&SymbolNode| symbol.id == matches[0].id) { selected.push(matches[0]); }
    }
    Ok(selected)
}

pub(super) fn implementation_context(snapshot: &McpState, params: ImplementationContextParams) -> Result<ImplementationContextOutput, McpError> {
    let analysis = &snapshot.analysis;
    let graph = &analysis.semantic_graph;
    let selected = resolve_targets(graph, &params.targets)?;
    let ids = selected.iter().map(|symbol| symbol.id.clone()).collect::<Vec<_>>();
    let members = graph.symbols.iter().filter(|symbol| ids.contains(&symbol.id) || symbol.parent_symbol_id.as_ref().is_some_and(|id| ids.contains(id)))
        .map(|symbol| symbol.id.as_str()).collect::<HashSet<_>>();
    let bodies = graph.function_behaviors.iter().filter(|body| members.contains(body.symbol_id.as_str())).collect::<Vec<_>>();
    let comparisons = analysis.architectural_assessment.behavior.comparisons.iter().filter(|comparison|
        members.contains(comparison.left.symbol_id.as_str()) || members.contains(comparison.right.symbol_id.as_str())
        || comparison.related_implementations.iter().any(|implementation| members.contains(implementation.symbol_id.as_str())))
        .collect::<Vec<_>>();
    Ok(ImplementationContextOutput {
        source_snapshot_id: crate::agentic::review_snapshot_id(analysis),
        input_coverage: graph.input_coverage(), selected: selected.into_iter().map(Into::into).collect(),
        execution_paths: crate::assessment::wiring::assess(graph, &analysis.contract_inventory, &ids),
        body_count: bodies.len(), bodies_truncated: bodies.len() > 32, body_previews: bodies.into_iter().take(32).cloned().collect(),
        comparison_count: comparisons.len(), comparisons_truncated: comparisons.len() > 16, comparisons: comparisons.into_iter().take(16).cloned().collect(),
    })
}
