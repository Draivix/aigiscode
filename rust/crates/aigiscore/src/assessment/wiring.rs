//! Captured wiring for explicit implementations. These are source observations,
//! not a claim that two paths execute or that an unreferenced contract is dead.

use crate::contracts::ContractInventory;
use crate::graph::{ReferenceKind, RelationKind, SemanticGraph, SymbolNode};
use crate::parsing::behavior::{BehaviorCall, BehaviorExpression, BranchArm, FunctionBehavior};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WiringSymbol {
    pub symbol_id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
}

impl From<&SymbolNode> for WiringSymbol {
    fn from(symbol: &SymbolNode) -> Self {
        Self { symbol_id: symbol.id.clone(), name: symbol.qualified_name.clone(), file_path: symbol.file_path.clone(), line: symbol.start_line }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WiringSite {
    pub file_path: PathBuf,
    pub line: usize,
    pub source_symbol_id: Option<String>,
    pub target_symbol_id: String,
    pub kind: ReferenceKind,
    pub relation: RelationKind,
    pub confidence_millis: u16,
    /// None when the call cannot be matched uniquely to a captured body site.
    pub syntactically_conditional: Option<bool>,
    pub guard_conditions: Vec<CallGuardEvidence>,
    pub inside_loop: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CallGuardEvidence {
    pub condition: BehaviorExpression,
    pub arm: BranchArm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContractNameMention {
    pub category: String,
    pub value: String,
    pub file_path: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImplementationWiring {
    pub implementation: WiringSymbol,
    pub owner: Option<WiringSymbol>,
    pub production_call_count: usize,
    pub production_calls: Vec<WiringSite>,
    pub test_call_count: usize,
    pub non_call_reference_count: usize,
    pub non_call_references: Vec<WiringSite>,
    pub implemented_contracts: Vec<WiringSymbol>,
    pub declared_implementors: Vec<WiringSymbol>,
    pub registrations: Vec<crate::graph::RuntimeRegistration>,
    /// Name matches alone do not establish registration of this implementation.
    pub contract_name_mentions: Vec<ContractNameMention>,
    pub body_count: usize,
    pub incomplete_body_count: usize,
    pub nested_callable_count: usize,
    pub captured_write_count: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PathRelationshipKind {
    BothHaveCapturedConsumers,
    SharedCallerWithConditionalSites,
    SharedCallerWithExclusiveBranches,
    SharedCallerWithUnconditionalSites,
    SharedCallerWithUnknownControlFlow,
    OneSideWithoutCapturedConsumers,
    NeitherSideHasCapturedConsumers,
    DeclaredContractAlternatives,
    ContractAndImplementation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PathRelationship {
    pub left_symbol_id: String,
    pub right_symbol_id: String,
    pub kind: PathRelationshipKind,
    pub shared_caller_ids: Vec<String>,
    pub shared_contract_ids: Vec<String>,
    pub shared_registration_types: Vec<String>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPathAssessment {
    pub input_parse_complete: bool,
    pub selection_truncated: bool,
    pub implementations: Vec<ImplementationWiring>,
    pub relationships: Vec<PathRelationship>,
    pub missing_evidence: Vec<String>,
}

pub fn assess(graph: &SemanticGraph, inventory: &ContractInventory, ids: &[String]) -> ExecutionPathAssessment {
    let symbols = graph.symbols.iter().map(|symbol| (symbol.id.as_str(), symbol)).collect::<HashMap<_, _>>();
    let bodies = graph.function_behaviors.iter().map(|body| (body.symbol_id.as_str(), body)).collect::<HashMap<_, _>>();
    let mut implementations = Vec::new();
    for id in ids {
        let Some(symbol) = symbols.get(id.as_str()).copied() else { continue; };
        let owner = symbol.parent_symbol_id.as_deref().and_then(|parent| symbols.get(parent).copied()).unwrap_or(symbol);
        let members = graph.symbols.iter().filter(|candidate| candidate.id == *id || candidate.parent_symbol_id.as_ref() == Some(id))
            .map(|candidate| candidate.id.as_str()).collect::<HashSet<_>>();
        let mut production_calls = Vec::new();
        let mut non_call_references = Vec::new();
        let mut test_call_count = 0;
        let mut implemented_contracts = BTreeSet::new();
        let mut declared_implementors = BTreeSet::new();
        for edge in &graph.resolved_edges {
            if edge.kind == ReferenceKind::Implements {
                if edge.source_symbol_id.as_deref() == Some(owner.id.as_str()) { implemented_contracts.insert(edge.target_symbol_id.as_str()); }
                if edge.target_symbol_id == owner.id {
                    if let Some(source) = edge.source_symbol_id.as_deref() { declared_implementors.insert(source); }
                }
            }
            if !members.contains(edge.target_symbol_id.as_str()) || edge.source_symbol_id.as_deref().is_some_and(|source| members.contains(source)) { continue; }
            let test_source = crate::ingestion::sources::is_test_source_path(&edge.source_file_path)
                || edge.source_symbol_id.as_deref().and_then(|source| bodies.get(source)).is_some_and(|body| body.test_guarded);
            if test_source {
                test_call_count += usize::from(edge.kind == ReferenceKind::Call);
                continue;
            }
            let body = edge.source_symbol_id.as_deref().and_then(|source| bodies.get(source)).copied();
            let call = body.and_then(|body| captured_call(body, edge.line, symbols.get(edge.target_symbol_id.as_str()).copied()));
            let site = WiringSite {
                file_path: edge.source_file_path.clone(), line: edge.line, source_symbol_id: edge.source_symbol_id.clone(),
                target_symbol_id: edge.target_symbol_id.clone(), kind: edge.kind, relation: edge.relation_kind,
                confidence_millis: edge.confidence_millis,
                syntactically_conditional: call.map(|call| call.conditional), inside_loop: call.map(|call| call.inside_loop),
                guard_conditions: call.into_iter().flat_map(|call| &call.guards).filter_map(|guard|
                    body.and_then(|body| body.branches.get(guard.branch)).map(|condition| CallGuardEvidence { condition: condition.clone(), arm: guard.arm })).collect(),
            };
            if edge.kind == ReferenceKind::Call { production_calls.push(site); }
            else { non_call_references.push(site); }
        }
        production_calls.sort_by_key(|site| (site.file_path.clone(), site.line, site.target_symbol_id.clone()));
        non_call_references.sort_by_key(|site| (site.file_path.clone(), site.line, site.target_symbol_id.clone()));
        let production_call_count = production_calls.len();
        let non_call_reference_count = non_call_references.len();
        let mut registrations = graph.runtime_registrations.iter().filter(|registration|
            registration.contract_type.eq_ignore_ascii_case(&owner.qualified_name)
                || registration.implementation_type.as_ref().is_some_and(|name| name.eq_ignore_ascii_case(&owner.qualified_name)))
            .cloned().collect::<Vec<_>>();
        let mut contract_name_mentions = Vec::new();
        for (category, items) in [("registered_key", &inventory.registered_keys), ("hook", &inventory.hooks), ("route", &inventory.routes)] {
            for item in items.iter().filter(|item| item.value == owner.name || item.value == owner.qualified_name || item.value.ends_with(&format!("\\{}", owner.name))) {
                for location in &item.locations {
                    contract_name_mentions.push(ContractNameMention { category: category.into(), value: item.value.clone(), file_path: location.file_path.clone(), line: location.line });
                }
            }
        }
        let owned_bodies = members.iter().filter_map(|id| bodies.get(id).copied()).collect::<Vec<_>>();
        let truncated = production_call_count > 32 || non_call_reference_count > 32 || registrations.len() > 16 || contract_name_mentions.len() > 16
            || implemented_contracts.len() > 16 || declared_implementors.len() > 16;
        production_calls.truncate(32);
        non_call_references.truncate(32);
        registrations.truncate(16);
        contract_name_mentions.truncate(16);
        implementations.push(ImplementationWiring {
            implementation: symbol.into(), owner: (owner.id != symbol.id).then(|| owner.into()), production_call_count, production_calls, test_call_count,
            non_call_reference_count, non_call_references,
            implemented_contracts: implemented_contracts.into_iter().take(16).filter_map(|id| symbols.get(id).copied()).map(Into::into).collect(),
            declared_implementors: declared_implementors.into_iter().take(16).filter_map(|id| symbols.get(id).copied()).map(Into::into).collect(),
            registrations, contract_name_mentions, body_count: owned_bodies.len(), incomplete_body_count: owned_bodies.iter().filter(|body| !body.complete || body.truncated).count(),
            nested_callable_count: owned_bodies.iter().map(|body| body.nested_callables).sum(),
            captured_write_count: owned_bodies.iter().map(|body| body.writes.len()).sum(), truncated,
        });
    }
    let mut relationships = Vec::new();
    for (position, left) in implementations.iter().enumerate() {
        for right in &implementations[position + 1..] {
            let left_owner = left.owner.as_ref().unwrap_or(&left.implementation);
            let right_owner = right.owner.as_ref().unwrap_or(&right.implementation);
            let left_callers = left.production_calls.iter().filter_map(|site| site.source_symbol_id.as_ref()).collect::<BTreeSet<_>>();
            let right_callers = right.production_calls.iter().filter_map(|site| site.source_symbol_id.as_ref()).collect::<BTreeSet<_>>();
            let shared = left_callers.intersection(&right_callers).copied().collect::<BTreeSet<_>>();
            let contracts = left.implemented_contracts.iter().filter(|contract| left_owner.symbol_id != right_owner.symbol_id
                && right.implemented_contracts.iter().any(|other| other.symbol_id == contract.symbol_id))
                .map(|contract| contract.symbol_id.clone()).collect::<Vec<_>>();
            let shared_registration_types = left.registrations.iter().filter(|registration| left_owner.symbol_id != right_owner.symbol_id
                && registration.implementation_type.as_ref().is_some_and(|name| name.eq_ignore_ascii_case(&left_owner.name))
                && right.registrations.iter().any(|other| registration.contract_type.eq_ignore_ascii_case(&other.contract_type)
                    && other.implementation_type.as_ref().is_some_and(|name| name.eq_ignore_ascii_case(&right_owner.name))))
                .map(|registration| registration.contract_type.clone())
                .collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
            let contract_link = left.implemented_contracts.iter().any(|contract| contract.symbol_id == right_owner.symbol_id)
                || right.implemented_contracts.iter().any(|contract| contract.symbol_id == left_owner.symbol_id)
                || left.registrations.iter().chain(&right.registrations).any(|registration|
                    (registration.contract_type.eq_ignore_ascii_case(&left_owner.name) && registration.implementation_type.as_ref().is_some_and(|name| name.eq_ignore_ascii_case(&right_owner.name)))
                    || (registration.contract_type.eq_ignore_ascii_case(&right_owner.name) && registration.implementation_type.as_ref().is_some_and(|name| name.eq_ignore_ascii_case(&left_owner.name))));
            let kind = if contract_link { PathRelationshipKind::ContractAndImplementation }
                else if !contracts.is_empty() || !shared_registration_types.is_empty() { PathRelationshipKind::DeclaredContractAlternatives }
                else if !shared.is_empty() {
                    let sites = left.production_calls.iter().chain(&right.production_calls)
                        .filter(|site| site.source_symbol_id.as_ref().is_some_and(|id| shared.contains(id))).collect::<Vec<_>>();
                    if shared.iter().all(|caller| {
                        let left_sites = left.production_calls.iter().filter(|site| site.source_symbol_id.as_ref() == Some(*caller));
                        left_sites.clone().all(|left_site| right.production_calls.iter().filter(|site| site.source_symbol_id.as_ref() == Some(*caller))
                            .all(|right_site| exclusive_branches(left_site, right_site)))
                    }) { PathRelationshipKind::SharedCallerWithExclusiveBranches }
                    else if sites.iter().any(|site| site.syntactically_conditional == Some(true)) { PathRelationshipKind::SharedCallerWithConditionalSites }
                    else if sites.iter().all(|site| site.syntactically_conditional == Some(false)) { PathRelationshipKind::SharedCallerWithUnconditionalSites }
                    else { PathRelationshipKind::SharedCallerWithUnknownControlFlow }
                } else if left.production_call_count > 0 && right.production_call_count > 0 { PathRelationshipKind::BothHaveCapturedConsumers }
                else if left.production_call_count > 0 || right.production_call_count > 0 { PathRelationshipKind::OneSideWithoutCapturedConsumers }
                else { PathRelationshipKind::NeitherSideHasCapturedConsumers };
            relationships.push(PathRelationship { left_symbol_id: left.implementation.symbol_id.clone(), right_symbol_id: right.implementation.symbol_id.clone(), kind,
                shared_caller_ids: shared.into_iter().take(16).cloned().collect(), shared_contract_ids: contracts,
                shared_registration_types,
                truncated: left.truncated || right.truncated || left_callers.intersection(&right_callers).count() > 16 });
        }
    }
    ExecutionPathAssessment { input_parse_complete: graph.input_coverage().is_complete(), selection_truncated: false, implementations, relationships, missing_evidence: vec![
        "Captured call sites do not establish that both paths execute for the same input; inspect selector branches, early exits and dispatch".into(),
        "No captured consumer is not a deletion proof: verify registrations, excluded sources and intended replacement wiring".into(),
        "Preserve provider alternatives, version boundaries, transaction/retry ownership and public contracts; state retirement conditions before migration".into(),
        "Runtime registrations retain their model and parse coverage; contract-name mentions are weaker evidence, and missing matches do not establish absent registration".into(),
    ] }
}

fn captured_call<'a>(body: &'a FunctionBehavior, line: usize, target: Option<&SymbolNode>) -> Option<&'a BehaviorCall> {
    if !body.complete || body.truncated { return None; }
    let target = target?;
    let mut calls = body.calls.iter().filter(|call| call.expression.line == line && call.target == target.name);
    let call = calls.next()?;
    if calls.next().is_some() { return None; }
    Some(call)
}

fn exclusive_branches(left: &WiringSite, right: &WiringSite) -> bool {
    if left.inside_loop != Some(false) || right.inside_loop != Some(false) { return false; }
    left.guard_conditions.iter().any(|left| right.guard_conditions.iter().any(|right|
        left.condition.start_byte == right.condition.start_byte && matches!((left.arm, right.arm),
            (BranchArm::Consequent, BranchArm::Alternative) | (BranchArm::Alternative, BranchArm::Consequent))))
}

#[cfg(test)]
mod tests {
    use super::{assess, PathRelationshipKind};
    use crate::contracts::ContractInventory;

    #[test]
    fn conditional_call_sites_do_not_become_a_both_paths_execute_claim() {
        let mut graph = crate::parsing::php::parse_php_to_graph("Paths.php", "<?php\nfunction oldPath(array $input): int { return 1; }\nfunction newPath(array $input): int { return 2; }\nfunction dispatch(array $input): int {\n if ($input['version'] === 1) { return oldPath($input); }\n return newPath($input);\n}\n").unwrap();
        crate::resolve::resolve_graph(&mut graph);
        let ids = graph.symbols.iter().filter(|symbol| matches!(symbol.name.as_str(), "oldPath" | "newPath"))
            .map(|symbol| symbol.id.clone()).collect::<Vec<_>>();
        let result = assess(&graph, &ContractInventory::default(), &ids);
        assert_eq!(result.implementations.len(), 2);
        assert!(result.implementations.iter().all(|implementation| implementation.production_call_count == 1));
        assert_eq!(result.relationships[0].kind, PathRelationshipKind::SharedCallerWithConditionalSites);
        assert!(!result.missing_evidence.is_empty());
    }

    #[test]
    fn unreferenced_replacement_remains_a_wiring_question() {
        let mut graph = crate::parsing::php::parse_php_to_graph("Replacement.php", "<?php\nclass Original { public static function run(): int { return 1; } }\nclass Replacement { public static function run(): int { return 2; } }\nfunction dispatch(): int { return Original::run(); }\n").unwrap();
        crate::resolve::resolve_graph(&mut graph);
        let ids = graph.symbols.iter().filter(|symbol| matches!(symbol.name.as_str(), "Original" | "Replacement"))
            .map(|symbol| symbol.id.clone()).collect::<Vec<_>>();
        let result = assess(&graph, &ContractInventory::default(), &ids);
        assert_eq!(result.relationships[0].kind, PathRelationshipKind::OneSideWithoutCapturedConsumers);
        assert!(result.implementations.iter().any(|implementation| implementation.implementation.name == "Replacement" && implementation.production_call_count == 0));
    }

    #[test]
    fn exclusive_branches_are_qualified_by_loop_iteration() {
        for (loop_open, loop_close, expected) in [
            ("", "", PathRelationshipKind::SharedCallerWithExclusiveBranches),
            ("foreach ($inputs as $input) {", "}", PathRelationshipKind::SharedCallerWithConditionalSites),
        ] {
            let source = format!("<?php\nfunction oldPath(array $input): void {{}}\nfunction newPath(array $input): void {{}}\nfunction dispatch(array $inputs, array $input): void {{\n{loop_open}\n if ($input['version'] === 1) {{ oldPath($input); }}\n else {{ newPath($input); }}\n{loop_close}\n}}\n");
            let mut graph = crate::parsing::php::parse_php_to_graph("Paths.php", &source).unwrap();
            crate::resolve::resolve_graph(&mut graph);
            let ids = graph.symbols.iter().filter(|symbol| matches!(symbol.name.as_str(), "oldPath" | "newPath"))
                .map(|symbol| symbol.id.clone()).collect::<Vec<_>>();
            let result = assess(&graph, &ContractInventory::default(), &ids);
            assert_eq!(result.relationships[0].kind, expected);
            assert!(result.implementations.iter().flat_map(|implementation| &implementation.production_calls).all(|site| !site.guard_conditions.is_empty()));
        }
    }

    #[test]
    fn deferred_callbacks_are_not_captured_as_immediate_body_calls() {
        let graph = crate::parsing::php::parse_php_to_graph("Publication.php", "<?php\nfunction schedule($transaction): void {\n $transaction->afterCommit(function (): void { publish(); });\n}\n").unwrap();
        let body = graph.function_behaviors.iter().find(|body| body.symbol_id.ends_with(":schedule")).unwrap();
        assert_eq!(body.nested_callables, 1);
        assert!(body.calls.iter().any(|call| call.target == "afterCommit"));
        assert!(!body.calls.iter().any(|call| call.target == "publish"));
    }
}
