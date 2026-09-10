//! Source-backed comparison candidates. Similar syntax proposes a review;
//! differing contracts and shared primitives are retained as distinct evidence.

use super::{ArchitecturalAssessmentFinding, ArchitecturalAssessmentKind};
use crate::evidence::EvidenceAnchor;
use crate::graph::{ReferenceKind, SemanticGraph, SymbolKind, SymbolNode};
use crate::identity::stable_fingerprint;
use crate::parsing::behavior::FunctionBehavior;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BehaviorCoverageStatus {
    Complete,
    Partial,
    #[default]
    Unavailable,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BehaviorAssessment {
    #[serde(flatten)]
    pub coverage: BehaviorComparisonCoverage,
    pub comparisons: Vec<ImplementationComparison>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct BehaviorComparisonCoverage {
    pub capture_status: BehaviorCoverageStatus,
    pub function_symbols: usize,
    pub captured_bodies: usize,
    pub usable_bodies: usize,
    pub pairs_considered: usize,
    pub candidate_search_truncated: bool,
    pub repeated_decision_pairs: usize,
    pub different_contract_pairs: usize,
    pub shared_primitive_pairs: usize,
    pub cross_language_pairs: usize,
    pub unverified_contract_pairs: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationComparisonKind {
    RepeatedDecisionCandidate,
    CrossLanguageContractCandidate,
    DifferentDeclaredContracts,
    UnverifiedInputContract,
    SharedPrimitive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ComparedImplementation {
    pub symbol_id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub parameter_types: Vec<Option<String>>,
    pub return_type: Option<String>,
    /// Resolved, non-test call sites in this analyzed graph. Zero is not proof
    /// that the application cannot load or dispatch the implementation.
    pub captured_consumer_count: usize,
    pub consumer_files: Vec<PathBuf>,
    pub consumers_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImplementationComparison {
    pub id: String,
    pub kind: ImplementationComparisonKind,
    pub left: ComparedImplementation,
    pub right: ComparedImplementation,
    pub related_implementations: Vec<ComparedImplementation>,
    pub related_implementations_truncated: bool,
    pub similarity_millis: u16,
    pub shared_selectors: Vec<String>,
    pub left_only_selectors: Vec<String>,
    pub right_only_selectors: Vec<String>,
    pub branch_shapes_differ: bool,
    pub return_shapes_differ: bool,
    pub source_expressions_differ: bool,
    pub error_handling_differ: bool,
    pub write_shapes_differ: bool,
    pub left_only_calls: Vec<String>,
    pub right_only_calls: Vec<String>,
    pub missing_evidence: Vec<String>,
}

pub(super) fn assess(graph: &SemanticGraph) -> BehaviorAssessment {
    let symbols = graph.symbols.iter().map(|symbol| (symbol.id.as_str(), symbol)).collect::<HashMap<_, _>>();
    let languages = graph.files.iter().map(|file| (file.path.as_path(), file.language)).collect::<HashMap<_, _>>();
    let function_symbols = graph.symbols.iter().filter(|symbol| matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method)).count();
    let bodies = graph.function_behaviors.iter().filter(|body| body.complete && !body.truncated && !body.test_guarded)
        .filter(|body| !crate::ingestion::sources::is_test_source_path(&body.file_path))
        .filter(|body| symbols.contains_key(body.symbol_id.as_str())).collect::<Vec<_>>();
    let mut coverage = BehaviorComparisonCoverage {
        capture_status: if graph.function_behaviors.is_empty() { BehaviorCoverageStatus::Unavailable }
            else if graph.function_behaviors.len() == function_symbols && graph.function_behaviors.iter().all(|body| body.complete && !body.truncated) {
                BehaviorCoverageStatus::Complete
            } else { BehaviorCoverageStatus::Partial },
        function_symbols, captured_bodies: graph.function_behaviors.len(), usable_bodies: bodies.len(),
        ..BehaviorComparisonCoverage::default()
    };
    let mut comparisons = Vec::new();
    let mut consumers = HashMap::<&str, Vec<PathBuf>>::new();
    let test_symbols = graph.function_behaviors.iter().filter(|body| body.test_guarded)
        .map(|body| body.symbol_id.as_str()).collect::<HashSet<_>>();
    for edge in graph.resolved_edges.iter().filter(|edge| edge.kind == ReferenceKind::Call
        && edge.source_symbol_id.as_deref() != Some(edge.target_symbol_id.as_str())
        && !edge.source_symbol_id.as_deref().is_some_and(|id| test_symbols.contains(id))
        && !crate::ingestion::sources::is_test_source_path(&edge.source_file_path)) {
        consumers.entry(edge.target_symbol_id.as_str()).or_default().push(edge.source_file_path.clone());
    }
    let mut buckets = BTreeMap::<String, Vec<usize>>::new();
    for (index, body) in bodies.iter().enumerate() {
        if body.parameters.is_empty() || body.token_count < 8 { continue; }
        let symbol = symbols[body.symbol_id.as_str()];
        let words = decision_name(&symbol.name);
        if words.len() >= 2 { buckets.entry(format!("name:{}", words.join("_"))).or_default().push(index); }
        for selector in &body.read_selectors { buckets.entry(format!("selector:{selector}")).or_default().push(index); }
        for fingerprint in body.structural_fingerprints.iter().take(8) {
            buckets.entry(format!("shape:{fingerprint}")).or_default().push(index);
        }
    }
    let mut pairs = BTreeSet::new();
    let mut bucket_order = buckets.iter().filter(|(_, members)| members.len() > 1).collect::<Vec<_>>();
    bucket_order.sort_by_key(|(key, members)| (
        if key.starts_with("name:") { 0 } else if key.starts_with("selector:") { 1 } else { 2 },
        members.len(), key.as_str(),
    ));
    for (_, members) in bucket_order {
        if members.len() > 64 { coverage.candidate_search_truncated = true; continue; }
        for (position, left) in members.iter().enumerate() {
            for right in &members[position + 1..] {
                if pairs.len() >= 50_000 { coverage.candidate_search_truncated = true; break; }
                pairs.insert(((*left).min(*right), (*left).max(*right)));
            }
        }
    }
    for (left, right) in pairs {
        let (left, right) = (bodies[left], bodies[right]);
        if left.symbol_id == right.symbol_id || languages.get(left.file_path.as_path()).is_none()
            || languages.get(right.file_path.as_path()).is_none() { continue; }
        let cross_language = languages.get(left.file_path.as_path()) != languages.get(right.file_path.as_path());
        coverage.pairs_considered += 1;
        let shared_selectors = intersection(&left.read_selectors, &right.read_selectors);
        let union_size = left.structural_fingerprints.iter().chain(&right.structural_fingerprints).collect::<BTreeSet<_>>().len();
        let shared = left.structural_fingerprints.iter().filter(|hash| right.structural_fingerprints.contains(hash)).count();
        let similarity = if union_size == 0 { 0 } else { (1000 * shared / union_size) as u16 };
        let (left_symbol, right_symbol) = (symbols[left.symbol_id.as_str()], symbols[right.symbol_id.as_str()]);
        let left_name = decision_name(&left_symbol.name);
        let same_name = left_name.len() >= 2 && left_name == decision_name(&right_symbol.name);
        let left_calls = call_labels(left);
        let right_calls = call_labels(right);
        let primitive = left.delegates_directly && right.delegates_directly && left.calls.len() == 1 && right.calls.len() == 1 && left_calls == right_calls;
        let decision = !left.branches.is_empty() && !right.branches.is_empty() && shared_selectors.len() >= 2;
        if cross_language && (!decision || !same_name) { continue; }
        if !primitive && (!decision || (!same_name && similarity < 350)) { continue; }
        let left_types = left.parameters.iter().map(|parameter| parameter.type_hint.clone()).collect::<Vec<_>>();
        let right_types = right.parameters.iter().map(|parameter| parameter.type_hint.clone()).collect::<Vec<_>>();
        let known_mismatch = left_types.len() != right_types.len()
            || left_types.iter().zip(&right_types).any(|(left, right)| left.is_some() && right.is_some() && left != right)
            || (left_symbol.return_type_name.is_some() && right_symbol.return_type_name.is_some() && left_symbol.return_type_name != right_symbol.return_type_name);
        let unverified_contract = !same_name && (left.read_selectors != right.read_selectors
            || left_types.iter().chain(&right_types).all(Option::is_none));
        let kind = if cross_language { ImplementationComparisonKind::CrossLanguageContractCandidate }
            else if primitive { ImplementationComparisonKind::SharedPrimitive }
            else if known_mismatch { ImplementationComparisonKind::DifferentDeclaredContracts }
            else if unverified_contract { ImplementationComparisonKind::UnverifiedInputContract }
            else { ImplementationComparisonKind::RepeatedDecisionCandidate };
        let id = stable_fingerprint(&["behavior-comparison", &left.symbol_id, &right.symbol_id]);
        match kind {
            ImplementationComparisonKind::RepeatedDecisionCandidate => coverage.repeated_decision_pairs += 1,
            ImplementationComparisonKind::DifferentDeclaredContracts => coverage.different_contract_pairs += 1,
            ImplementationComparisonKind::SharedPrimitive => coverage.shared_primitive_pairs += 1,
            ImplementationComparisonKind::CrossLanguageContractCandidate => coverage.cross_language_pairs += 1,
            ImplementationComparisonKind::UnverifiedInputContract => coverage.unverified_contract_pairs += 1,
        }
        comparisons.push(ImplementationComparison {
            id, kind,
            left: implementation(left, left_symbol, &consumers), right: implementation(right, right_symbol, &consumers),
            related_implementations: Vec::new(), related_implementations_truncated: false,
            similarity_millis: similarity,
            shared_selectors,
            left_only_selectors: difference(&left.read_selectors, &right.read_selectors),
            right_only_selectors: difference(&right.read_selectors, &left.read_selectors),
            branch_shapes_differ: left.branches.iter().map(|branch| &branch.shape).collect::<Vec<_>>() != right.branches.iter().map(|branch| &branch.shape).collect::<Vec<_>>(),
            return_shapes_differ: left.returns.iter().map(|value| &value.shape).collect::<Vec<_>>() != right.returns.iter().map(|value| &value.shape).collect::<Vec<_>>(),
            source_expressions_differ: left.branches.iter().chain(&left.returns).map(|value| &value.fingerprint).collect::<Vec<_>>() != right.branches.iter().chain(&right.returns).map(|value| &value.fingerprint).collect::<Vec<_>>(),
            error_handling_differ: (left.throws, left.catches) != (right.throws, right.catches),
            write_shapes_differ: left.writes.iter().map(|write| &write.shape).collect::<Vec<_>>() != right.writes.iter().map(|write| &write.shape).collect::<Vec<_>>(),
            left_only_calls: difference(&left_calls, &right_calls), right_only_calls: difference(&right_calls, &left_calls),
            missing_evidence: vec!["Compare the same concrete inputs, defaults, errors and effects before asserting semantic equivalence or choosing a surviving owner".into(),
                "Captured consumers do not establish complete runtime wiring; preserve deliberate adapters and public contracts".into()],
        });
    }
    comparisons.sort_by(|left, right| {
        comparison_priority(right).cmp(&comparison_priority(left))
            .then(right.similarity_millis.cmp(&left.similarity_millis)).then(left.id.cmp(&right.id))
    });
    // Keep the evidence corpus larger than an agent's preview. A small global
    // preview discards rare disagreements behind common provider families.
    if comparisons.len() > 2048 { coverage.candidate_search_truncated = true; comparisons.truncate(2048); }
    for comparison in &mut comparisons {
        if comparison.kind != ImplementationComparisonKind::RepeatedDecisionCandidate { continue; }
        let selectors = comparison.shared_selectors.iter().chain(&comparison.left_only_selectors)
            .chain(&comparison.right_only_selectors).cloned().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        let comparison_language = languages.get(comparison.left.file_path.as_path());
        let mut related = bodies.iter().filter(|body| body.read_selectors == selectors && !body.branches.is_empty()
            && body.symbol_id != comparison.left.symbol_id && body.symbol_id != comparison.right.symbol_id
            && languages.get(body.file_path.as_path()) == comparison_language)
            .filter(|body| body.parameters.iter().map(|parameter| &parameter.type_hint).eq(comparison.left.parameter_types.iter())
                && symbols[body.symbol_id.as_str()].return_type_name == comparison.left.return_type)
            .map(|body| implementation(body, symbols[body.symbol_id.as_str()], &consumers)).collect::<Vec<_>>();
        related.sort_by(|left, right| (left.captured_consumer_count > 0).cmp(&(right.captured_consumer_count > 0)).then(left.symbol_id.cmp(&right.symbol_id)));
        comparison.related_implementations_truncated = related.len() > 4;
        comparison.related_implementations = related.into_iter().take(4).collect();
    }
    BehaviorAssessment { coverage, comparisons }
}

fn comparison_priority(comparison: &ImplementationComparison) -> usize {
    usize::from(matches!(comparison.kind, ImplementationComparisonKind::RepeatedDecisionCandidate | ImplementationComparisonKind::CrossLanguageContractCandidate)) * 4
        + usize::from(!comparison.left_only_selectors.is_empty() || !comparison.right_only_selectors.is_empty()) * 2
        + usize::from(comparison.branch_shapes_differ || comparison.return_shapes_differ)
}

fn decision_name(name: &str) -> Vec<String> {
    super::split_identifier_words(name).into_iter()
        .filter(|word| !matches!(word.as_str(), "get" | "set" | "resolve" | "extract" | "find" | "build"))
        .collect()
}

fn implementation(body: &FunctionBehavior, symbol: &SymbolNode, consumers: &HashMap<&str, Vec<PathBuf>>) -> ComparedImplementation {
    let calls = consumers.get(body.symbol_id.as_str()).cloned().unwrap_or_default();
    let files = calls.iter().cloned().collect::<BTreeSet<_>>();
    ComparedImplementation {
        symbol_id: body.symbol_id.clone(), name: symbol.qualified_name.clone(), file_path: body.file_path.clone(), line: body.line,
        parameter_types: body.parameters.iter().map(|parameter| parameter.type_hint.clone()).collect(),
        return_type: symbol.return_type_name.clone(), captured_consumer_count: calls.len(),
        consumers_truncated: files.len() > 8, consumer_files: files.into_iter().take(8).collect(),
    }
}

fn call_labels(body: &FunctionBehavior) -> Vec<String> {
    body.calls.iter().map(|call| match &call.receiver_shape {
        Some(receiver) => format!("{receiver}::{}", call.target), None => call.target.clone(),
    }).collect::<BTreeSet<_>>().into_iter().collect()
}

fn intersection(left: &[String], right: &[String]) -> Vec<String> { left.iter().filter(|value| right.contains(value)).cloned().collect() }
fn difference(left: &[String], right: &[String]) -> Vec<String> { left.iter().filter(|value| !right.contains(value)).cloned().collect() }

pub(super) fn findings(assessment: &BehaviorAssessment) -> Vec<ArchitecturalAssessmentFinding> {
    assessment.comparisons.iter().filter(|comparison| matches!(comparison.kind, ImplementationComparisonKind::RepeatedDecisionCandidate | ImplementationComparisonKind::CrossLanguageContractCandidate))
        .map(|comparison| ArchitecturalAssessmentFinding {
            behavior_comparison_id: Some(comparison.id.clone()),
            kind: ArchitecturalAssessmentKind::DuplicateMechanism,
            file_path: comparison.left.file_path.clone(), related_file_paths: std::iter::once(comparison.right.file_path.clone())
                .chain(comparison.related_implementations.iter().map(|implementation| implementation.file_path.clone()))
                .filter(|path| path != &comparison.left.file_path).collect::<BTreeSet<_>>().into_iter().collect(),
            related_identifiers: vec![comparison.left.symbol_id.clone(), comparison.right.symbol_id.clone()],
            evidence_anchors: [
                EvidenceAnchor { file_path: comparison.left.file_path.clone(), line: Some(comparison.left.line), label: "implementation_left".into() },
                EvidenceAnchor { file_path: comparison.right.file_path.clone(), line: Some(comparison.right.line), label: "implementation_right".into() },
            ].into_iter().chain(comparison.related_implementations.iter().map(|implementation| EvidenceAnchor {
                file_path: implementation.file_path.clone(), line: Some(implementation.line), label: "related_implementation".into(),
            })).collect(),
            warning_count: 2, warning_weight: 2, bottleneck_centrality_millis: 0,
            warning_families: vec!["repeated_input_decision".into()], severity_millis: 720,
            pressure_path: Vec::new(), expensive_operation_sites: Vec::new(), expensive_operation_flow: Vec::new(), fingerprint: String::new(),
        }).collect()
}

#[cfg(test)]
mod tests {
    use super::{assess, findings, ImplementationComparisonKind};
    use crate::graph::SemanticGraph;

    fn php_graph(sources: &[(&str, &str)]) -> SemanticGraph {
        let mut graph = SemanticGraph::default();
        for (path, source) in sources {
            graph.append(crate::parsing::php::parse_php_to_graph(*path, source).unwrap());
        }
        crate::resolve::resolve_graph(&mut graph);
        graph
    }

    #[test]
    fn compares_actor_keys_and_branch_contracts_from_real_parsed_bodies() {
        let graph = php_graph(&[
            ("Mapper.php", "<?php\nclass Mapper {\n public function resolveActorId(array $context): ?int {\n  if (isset($context['actorId'])) { return (int) $context['actorId']; }\n  $user = $context['user'] ?? null;\n  return $user ? $user->id : null;\n }\n}\n"),
            ("Notifications.php", "<?php\nclass Notifications {\n public function resolveActorId(array $context): ?int {\n  $user = $context['user'] ?? null;\n  if (!$user || !isset($user->id)) { return null; }\n  return (int) $user->id;\n }\n}\n"),
        ]);
        let assessment = assess(&graph);
        let comparison = assessment.comparisons.iter().find(|comparison| comparison.kind == ImplementationComparisonKind::RepeatedDecisionCandidate).unwrap();
        assert_eq!(comparison.shared_selectors, ["id", "user"]);
        assert_eq!(comparison.left_only_selectors, ["actorId"]);
        assert!(comparison.branch_shapes_differ);
        assert!(comparison.return_shapes_differ);
        assert!(comparison.source_expressions_differ);
        assert_eq!(comparison.left.parameter_types, vec![Some("array".into())]);
        assert_eq!(findings(&assessment).len(), 1);
        let mapper = graph.function_behaviors.iter().find(|body| body.file_path.to_str() == Some("Mapper.php")).unwrap();
        assert_eq!(mapper.returns.len(), 2, "return keywords must not duplicate statement facts");
    }

    #[test]
    fn retains_adapters_with_distinct_declared_inputs() {
        let graph = php_graph(&[
            ("First.php", "<?php\nclass First {\n public function normalizeContext(FirstSchema $input): mixed {\n  if ($input->id === null) { return $input->user; }\n  return $input->id;\n }\n}\n"),
            ("Second.php", "<?php\nclass Second {\n public function normalizeContext(SecondSchema $input): mixed {\n  if ($input->id === null) { return $input->user; }\n  return $input->id;\n }\n}\n"),
        ]);
        let assessment = assess(&graph);
        assert!(assessment.comparisons.iter().any(|comparison| comparison.kind == ImplementationComparisonKind::DifferentDeclaredContracts));
        assert!(findings(&assessment).is_empty());
    }

    #[test]
    fn shared_normalization_primitive_is_not_a_business_rule_duplicate() {
        let graph = php_graph(&[
            ("First.php", "<?php\nfunction normalizeLabel(string $value): string { return strtolower($value); }\n"),
            ("Second.php", "<?php\nfunction normalizeCode(string $input): string { return strtolower($input); }\n"),
        ]);
        let assessment = assess(&graph);
        assert!(assessment.comparisons.iter().any(|comparison| comparison.kind == ImplementationComparisonKind::SharedPrimitive));
        assert!(findings(&assessment).is_empty());
    }

    #[test]
    fn different_array_schemas_require_input_contract_review() {
        let graph = php_graph(&[
            ("Fields.php", "<?php\nclass Fields {\n public function isRelationTo(array $definition, string $target): bool {\n  if ($definition['type'] !== 'relation') { return false; }\n  if ($definition['entity'] !== $target) { return false; }\n  return in_array($definition['relation'], ['many', 'one']);\n }\n public function isRelationDefinitionTo(array $definition, string $target): bool {\n  if ($definition['entity'] !== $target) { return false; }\n  return in_array($definition['type'], ['many', 'one']);\n }\n}\n"),
        ]);
        let assessment = assess(&graph);
        assert!(assessment.comparisons.iter().any(|comparison| comparison.kind == ImplementationComparisonKind::UnverifiedInputContract));
        assert!(findings(&assessment).is_empty());
    }

    #[test]
    fn bounded_capture_does_not_turn_missing_body_detail_into_equivalence() {
        let calls = "consume($context['user'], $context['id']);\n".repeat(40);
        let source = format!("<?php\nfunction resolveActorId(array $context): void {{\n{calls}}}\n");
        let graph = php_graph(&[("Large.php", &source)]);
        let body = graph.function_behaviors.iter().find(|body| body.symbol_id.contains("resolveActorId")).unwrap();
        assert!(body.truncated);
        let assessment = assess(&graph);
        assert_eq!(assessment.coverage.usable_bodies, 0);
        assert!(assessment.comparisons.is_empty());
    }
}
