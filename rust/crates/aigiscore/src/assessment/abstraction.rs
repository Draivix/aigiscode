//! Abstraction evidence comes from captured bodies and resolved delegation.
//! Public contracts and independent reasons to change still require review.

use super::wiring::WiringSymbol;
use super::{ArchitecturalAssessmentFinding, ArchitecturalAssessmentKind};
use crate::evidence::EvidenceAnchor;
use crate::graph::{ReferenceKind, ResolvedEdge, SemanticGraph, SymbolKind, SymbolNode, Visibility};
use crate::parsing::behavior::{BehaviorExpression, FunctionBehavior};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AbstractionBoundary {
    PublicOrProtectedContract,
    DeclaredOverride,
    OwnerDeclaresContract,
    DecoratorOrAttribute,
    CallingConvention,
    DefaultArguments,
    Branching,
    CapturedMutation,
    ErrorHandling,
    Iteration,
    DeferredExecution,
    Composition,
    NonForwardingBody,
    UnverifiedForwardingContract,
    IncompleteBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MethodResponsibility {
    pub method: WiringSymbol,
    pub boundary_evidence: Vec<AbstractionBoundary>,
    pub read_selectors: Vec<String>,
    pub branch_conditions: Vec<BehaviorExpression>,
    pub captured_writes: Vec<BehaviorExpression>,
    pub return_expressions: Vec<BehaviorExpression>,
    pub calls_without_strong_resolution: usize,
    pub callee_count: usize,
    pub callees: Vec<WiringSymbol>,
    pub dependency_owners: Vec<WiringSymbol>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DelegationChain {
    pub path: Vec<WiringSymbol>,
    pub terminal_boundary: Vec<AbstractionBoundary>,
    pub cycle: bool,
    pub truncated: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AbstractionProfile {
    pub method_count: usize,
    pub methods: Vec<MethodResponsibility>,
    pub methods_truncated: bool,
    pub body_evidence_complete: bool,
    pub delegation_chains: Vec<DelegationChain>,
    pub missing_evidence: Vec<String>,
}

pub(crate) struct AbstractionContext<'a> {
    graph: &'a SemanticGraph,
    symbols: HashMap<&'a str, &'a SymbolNode>,
    bodies: HashMap<&'a str, &'a FunctionBehavior>,
    calls: HashMap<&'a str, Vec<&'a ResolvedEdge>>,
    overrides: HashSet<&'a str>,
    contract_owners: HashSet<&'a str>,
    forwarding: HashMap<&'a str, &'a SymbolNode>,
}

impl<'a> AbstractionContext<'a> {
    pub(crate) fn new(graph: &'a SemanticGraph) -> Self {
        let mut calls = HashMap::<&str, Vec<&ResolvedEdge>>::new();
        let mut overrides = HashSet::new();
        let mut contract_owners = graph.symbols.iter().filter(|symbol| matches!(symbol.kind, SymbolKind::Interface | SymbolKind::Trait))
            .map(|symbol| symbol.id.as_str()).collect::<HashSet<_>>();
        contract_owners.extend(graph.references.iter().filter(|reference| reference.kind == ReferenceKind::Implements)
            .filter_map(|reference| reference.enclosing_symbol_id.as_deref()));
        for edge in &graph.resolved_edges {
            if let Some(source) = edge.source_symbol_id.as_deref() {
                if edge.kind == ReferenceKind::Call { calls.entry(source).or_default().push(edge); }
                if edge.kind == ReferenceKind::Overrides { overrides.insert(source); }
                if edge.kind == ReferenceKind::Implements { contract_owners.insert(source); }
            }
            if edge.kind == ReferenceKind::Overrides { overrides.insert(edge.target_symbol_id.as_str()); }
        }
        let mut context = Self {
            graph,
            symbols: graph.symbols.iter().map(|symbol| (symbol.id.as_str(), symbol)).collect(),
            bodies: graph.function_behaviors.iter().map(|body| (body.symbol_id.as_str(), body)).collect(),
            calls, overrides, contract_owners, forwarding: HashMap::new(),
        };
        context.forwarding = graph.symbols.iter().filter_map(|symbol| context.forward_target(symbol).map(|target| (symbol.id.as_str(), target))).collect();
        context
    }

    fn forward_target(&self, symbol: &SymbolNode) -> Option<&'a SymbolNode> {
        if symbol.visibility != Visibility::Private || self.overrides.contains(symbol.id.as_str()) { return None; }
        if symbol.parent_symbol_id.as_deref().is_some_and(|owner| self.contract_owners.contains(owner)) { return None; }
        let body = self.bodies.get(symbol.id.as_str())?;
        if !body.complete || body.truncated || body.test_guarded || !body.delegates_directly
            || body.has_decorators_or_attributes != Some(false)
            || body.has_calling_convention_modifier != Some(false)
            || !body.branches.is_empty() || !body.writes.is_empty() || body.throws + body.catches + body.loops + body.nested_callables > 0
            || body.calls.len() != 1 || body.parameters.iter().any(|parameter| parameter.has_default) { return None; }
        if body.calls[0].argument_shapes != forwarded_arguments(body) { return None; }
        let edges = self.calls.get(symbol.id.as_str())?;
        let target_ids = edges.iter().filter(|edge| edge.confidence_millis >= 900 && edge.line == body.calls[0].expression.line)
            .map(|edge| edge.target_symbol_id.as_str()).collect::<BTreeSet<_>>();
        if target_ids.len() != 1 || edges.iter().any(|edge| edge.confidence_millis < 900) { return None; }
        let target = self.symbols.get(*target_ids.first()?).copied()?;
        if !matches!(target.kind, SymbolKind::Function | SymbolKind::Method) || symbol.return_type_name != target.return_type_name { return None; }
        let returns_value = body.returns.len() == 1;
        let returns_void = matches!(symbol.return_type_name.as_deref(), Some("void" | "()"));
        (returns_value || returns_void).then_some(target)
    }

    fn boundaries(&self, symbol: &SymbolNode) -> Vec<AbstractionBoundary> {
        let mut boundaries = Vec::new();
        if symbol.visibility != Visibility::Private { boundaries.push(AbstractionBoundary::PublicOrProtectedContract); }
        if self.overrides.contains(symbol.id.as_str()) { boundaries.push(AbstractionBoundary::DeclaredOverride); }
        if symbol.parent_symbol_id.as_deref().is_some_and(|owner| self.contract_owners.contains(owner)) { boundaries.push(AbstractionBoundary::OwnerDeclaresContract); }
        let Some(body) = self.bodies.get(symbol.id.as_str()) else { boundaries.push(AbstractionBoundary::IncompleteBody); return boundaries; };
        if !body.complete || body.truncated || body.has_decorators_or_attributes.is_none() || body.has_calling_convention_modifier.is_none() { boundaries.push(AbstractionBoundary::IncompleteBody); }
        if body.has_decorators_or_attributes == Some(true) { boundaries.push(AbstractionBoundary::DecoratorOrAttribute); }
        if body.has_calling_convention_modifier == Some(true) { boundaries.push(AbstractionBoundary::CallingConvention); }
        if body.parameters.iter().any(|parameter| parameter.has_default) { boundaries.push(AbstractionBoundary::DefaultArguments); }
        if !body.branches.is_empty() { boundaries.push(AbstractionBoundary::Branching); }
        if !body.writes.is_empty() { boundaries.push(AbstractionBoundary::CapturedMutation); }
        if body.throws + body.catches > 0 { boundaries.push(AbstractionBoundary::ErrorHandling); }
        if body.loops > 0 { boundaries.push(AbstractionBoundary::Iteration); }
        if body.nested_callables > 0 { boundaries.push(AbstractionBoundary::DeferredExecution); }
        if body.calls.len() > 1 { boundaries.push(AbstractionBoundary::Composition); }
        if !body.delegates_directly || body.calls.first().is_some_and(|call|
            call.argument_shapes != forwarded_arguments(body)) {
            boundaries.push(AbstractionBoundary::NonForwardingBody);
        }
        if body.delegates_directly && symbol.visibility == Visibility::Private && !self.forwarding.contains_key(symbol.id.as_str()) {
            boundaries.push(AbstractionBoundary::UnverifiedForwardingContract);
        }
        boundaries
    }

    fn chain(&self, symbol: &'a SymbolNode) -> Option<DelegationChain> {
        self.forwarding.get(symbol.id.as_str())?;
        let mut path = vec![symbol.into()];
        let mut visited = HashSet::from([symbol.id.as_str()]);
        let mut current = symbol;
        let mut cycle = false;
        let mut truncated = false;
        while let Some(next) = self.forwarding.get(current.id.as_str()).copied() {
            if path.len() == 8 { truncated = true; break; }
            path.push(next.into());
            current = next;
            if !visited.insert(next.id.as_str()) { cycle = true; break; }
        }
        Some(DelegationChain { path, terminal_boundary: self.boundaries(current), cycle, truncated })
    }

    pub(crate) fn profile(&self, symbol: &'a SymbolNode) -> AbstractionProfile {
        let mut members = self.graph.symbols.iter().filter(|member| (member.id == symbol.id || member.parent_symbol_id.as_deref() == Some(symbol.id.as_str()))
            && matches!(member.kind, SymbolKind::Function | SymbolKind::Method)).collect::<Vec<_>>();
        members.sort_by_key(|member| (member.visibility == Visibility::Private, member.start_line, member.id.as_str()));
        let body_evidence_complete = !members.is_empty() && members.iter().all(|member|
            self.bodies.get(member.id.as_str()).is_some_and(|body| body.complete && !body.truncated && body.has_decorators_or_attributes.is_some() && body.has_calling_convention_modifier.is_some()));
        let mut methods = Vec::new();
        let mut delegation_chains = Vec::new();
        for member in members.iter().take(32) {
            let body = self.bodies.get(member.id.as_str()).copied();
            let targets = self.calls.get(member.id.as_str()).into_iter().flatten()
                .filter(|edge| edge.confidence_millis >= 900)
                .filter_map(|edge| self.symbols.get(edge.target_symbol_id.as_str()).copied()).map(|target| (target.id.as_str(), target)).collect::<BTreeMap<_, _>>();
            let owners = targets.values().filter_map(|target| target.parent_symbol_id.as_deref().and_then(|id| self.symbols.get(id).copied()).or(Some(*target)))
                .filter(|owner| owner.id != symbol.id).map(|owner| (owner.id.as_str(), owner)).collect::<BTreeMap<_, _>>();
            methods.push(MethodResponsibility {
                method: (*member).into(), boundary_evidence: self.boundaries(member),
                read_selectors: body.map(|body| body.read_selectors.clone()).unwrap_or_default(),
                branch_conditions: body.map(|body| body.branches.iter().take(8).cloned().collect()).unwrap_or_default(),
                captured_writes: body.map(|body| body.writes.iter().take(8).cloned().collect()).unwrap_or_default(),
                return_expressions: body.map(|body| body.returns.iter().take(8).cloned().collect()).unwrap_or_default(),
                calls_without_strong_resolution: body.map(|body| body.calls.iter().filter(|call| {
                    let targets = self.calls.get(member.id.as_str()).into_iter().flatten().filter(|edge|
                        edge.confidence_millis >= 900 && edge.line == call.expression.line
                            && (edge.reference_target_name.as_deref() == Some(call.target.as_str())
                                || self.symbols.get(edge.target_symbol_id.as_str()).is_some_and(|symbol| symbol.name == call.target)))
                        .map(|edge| edge.target_symbol_id.as_str()).collect::<BTreeSet<_>>();
                    targets.len() != 1
                }).count()).unwrap_or(0),
                callee_count: targets.len(), callees: targets.values().take(16).map(|target| (*target).into()).collect(),
                dependency_owners: owners.values().take(16).map(|owner| (*owner).into()).collect(),
                truncated: targets.len() > 16 || owners.len() > 16 || body.is_some_and(|body| body.truncated || body.branches.len() > 8 || body.writes.len() > 8 || body.returns.len() > 8),
            });
            if let Some(chain) = self.chain(member) { delegation_chains.push(chain); }
        }
        AbstractionProfile { method_count: members.len(), methods, methods_truncated: members.len() > 32, body_evidence_complete, delegation_chains,
            missing_evidence: vec![
                "Assign independent reasons to change from method behavior, consumers and existing dependency owners; different callees alone do not justify splitting".into(),
                "Before inlining, preserve receiver dispatch, declared contracts, defaults, state/lifecycle ownership and consumer behavior; captured transparency is a review candidate".into(),
                "Captured writes may affect local or shared data; absence of a captured write does not prove absence of mutable state".into(),
            ] }
    }
}

fn forwarded_arguments(body: &FunctionBehavior) -> Vec<String> {
    body.parameters.iter().enumerate().filter(|(_, parameter)| !parameter.is_receiver).map(|(index, _)| format!("input{index}")).collect()
}

pub(super) fn is_plain_field_accessor(body: &FunctionBehavior) -> bool {
    if !body.complete || body.truncated || body.has_decorators_or_attributes != Some(false) || body.has_calling_convention_modifier != Some(false) || body.statement_count != 1
        || !body.calls.is_empty() || !body.branches.is_empty() || body.throws + body.catches + body.loops + body.nested_callables > 0
        || body.read_selectors.len() != 1 || body.parameters.iter().any(|parameter| parameter.has_default) { return false; }
    let field = &body.read_selectors[0];
    let inputs = forwarded_arguments(body);
    for receiver in ["this ->", "this .", "input0 .", "self ."] {
        if receiver == "input0 ." && !body.parameters.first().is_some_and(|parameter| parameter.is_receiver) { continue; }
        let access = format!("{receiver} {field}");
        if inputs.is_empty() && body.writes.is_empty() && body.returns.len() == 1
            && [access.clone(), format!("return {access}"), format!("return {access} ;")].contains(&body.returns[0].shape) { return true; }
        if inputs.len() == 1 && body.writes.len() == 1 && body.returns.is_empty()
            && body.writes[0].shape == format!("{access} = {}", inputs[0]) { return true; }
    }
    false
}

pub(super) fn findings(graph: &SemanticGraph) -> Vec<ArchitecturalAssessmentFinding> {
    let context = AbstractionContext::new(graph);
    let inner = context.forwarding.values().map(|target| target.id.as_str()).collect::<HashSet<_>>();
    let mut findings = Vec::new();
    for symbol in &graph.symbols {
        if inner.contains(symbol.id.as_str()) || crate::ingestion::sources::is_test_source_path(&symbol.file_path) { continue; }
        let Some(chain) = context.chain(symbol) else { continue; };
        if chain.path.len() < 3 || chain.cycle || chain.truncated { continue; }
        findings.push(ArchitecturalAssessmentFinding {
            kind: ArchitecturalAssessmentKind::AbstractionSprawl, behavior_comparison_id: None,
            file_path: symbol.file_path.clone(), related_file_paths: chain.path.iter().map(|step| step.file_path.clone())
                .filter(|path| path != &symbol.file_path).collect::<BTreeSet<_>>().into_iter().collect(),
            related_identifiers: chain.path.iter().map(|step| step.symbol_id.clone()).collect(),
            evidence_anchors: chain.path.iter().map(|step| EvidenceAnchor { file_path: step.file_path.clone(), line: Some(step.line), label: "delegation_chain".into() }).collect(),
            warning_count: chain.path.len() - 1, warning_weight: chain.path.len() - 1, bottleneck_centrality_millis: 0,
            warning_families: vec!["private_argument_forwarding".into()], severity_millis: 650,
            pressure_path: Vec::new(), expensive_operation_sites: Vec::new(), expensive_operation_flow: Vec::new(), fingerprint: String::new(),
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::{findings, AbstractionBoundary, AbstractionContext};

    #[test]
    fn private_forwarding_chain_is_evidence_but_public_and_transforming_layers_remain() {
        let mut graph = crate::parsing::php::parse_php_to_graph("Calculator.php", "<?php\nclass Calculator {\n public function run(int $value): int { return $this->first($value); }\n private function first(int $value): int { return $this->second($value); }\n private function second(int $value): int { return $this->calculate($value); }\n private function calculate(int $value): int { return $value + 1; }\n}\n").unwrap();
        crate::resolve::resolve_graph(&mut graph);
        let findings = findings(&graph);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].related_identifiers[0].ends_with(":first"));
        let context = AbstractionContext::new(&graph);
        let container = graph.symbols.iter().find(|symbol| symbol.name == "Calculator" && symbol.kind == crate::graph::SymbolKind::Class).unwrap();
        let profile = context.profile(container);
        assert!(profile.methods.iter().find(|method| method.method.name.ends_with("::run")).unwrap().boundary_evidence.contains(&AbstractionBoundary::PublicOrProtectedContract));
        assert!(profile.methods.iter().find(|method| method.method.name.ends_with("::calculate")).unwrap().boundary_evidence.contains(&AbstractionBoundary::NonForwardingBody));
    }

    #[test]
    fn attributes_and_variadic_binding_preserve_their_boundaries() {
        let mut graph = crate::parsing::php::parse_php_to_graph("Boundaries.php", "<?php\n#[Attribute] class CachePolicy {}\nclass Decorated {\n #[CachePolicy]\n private function first(array $input): array { return $this->second($input); }\n private function second(array $input): array { return $this->terminal($input); }\n private function terminal(array $input): array { return $input; }\n}\nclass Variadic {\n private function first(...$input): array { return $this->second($input); }\n private function second(array $input): array { return $this->terminal($input); }\n private function terminal(array $input): array { return $input; }\n}\nclass Plain {\n private function first(array $input): array { return $this->second($input); }\n private function second(array $input): array { return $this->terminal($input); }\n private function terminal(array $input): array { return $input; }\n}\n").unwrap();
        crate::resolve::resolve_graph(&mut graph);
        let findings = findings(&graph);
        assert_eq!(findings.len(), 1, "unrelated attributes must not contaminate the entire file");
        assert!(findings[0].related_identifiers[0].contains(":Plain:first"));
        let context = AbstractionContext::new(&graph);
        for (name, boundary) in [("Decorated", AbstractionBoundary::DecoratorOrAttribute), ("Variadic", AbstractionBoundary::CallingConvention)] {
            let symbol = graph.symbols.iter().find(|symbol| symbol.name == name && symbol.kind == crate::graph::SymbolKind::Class).unwrap();
            let profile = context.profile(symbol);
            let first = profile.methods.iter().find(|method| method.method.name.ends_with("::first")).unwrap();
            assert!(first.boundary_evidence.contains(&boundary));
        }
    }
}
