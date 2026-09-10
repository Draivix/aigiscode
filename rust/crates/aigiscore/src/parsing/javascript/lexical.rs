use crate::graph::{CallForm, LexicalCallBinding, LexicalReferenceBinding, ReferenceKind, SemanticGraph, SymbolKind, SymbolNode};
use std::collections::{BTreeSet, HashMap};
use tree_sitter::Node;

#[derive(Default)]
pub(super) struct Bindings {
    declarations: HashMap<(usize, String), Option<String>>,
    scoped_symbols: BTreeSet<String>,
    calls: Vec<(usize, Vec<usize>, Option<String>)>,
    writes: Vec<(String, Vec<usize>)>,
    instances: HashMap<(usize, String), (String, Vec<usize>)>,
    invalid_instances: BTreeSet<(usize, String)>,
    receivers: HashMap<usize, Receiver>,
    methods: HashMap<(String, String, bool), Option<String>>,
    method_owners: HashMap<usize, (String, bool)>,
    types: HashMap<(usize, String), Option<String>>,
    named_references: Vec<(usize, Vec<usize>, bool)>,
}

enum Receiver {
    Name(String),
    Constructed(String),
    This(usize),
}

fn is_static(node: Node<'_>) -> bool {
    (0..node.child_count()).filter_map(|index| node.child(index as u32))
        .any(|child| child.kind() == "static")
}

fn is_function(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_declaration" | "function_expression" | "arrow_function" | "method_definition")
}

fn is_scope(node: Node<'_>) -> bool {
    is_function(node) || matches!(node.kind(), "program" | "statement_block" | "switch_body" | "for_statement" | "for_in_statement" | "catch_clause" | "class_declaration" | "abstract_class_declaration" | "interface_declaration" | "type_alias_declaration")
}

fn declaration_scope(node: Node<'_>, function_scoped: bool) -> Option<Node<'_>> {
    let mut parent = node.parent();
    while let Some(scope) = parent {
        if (function_scoped && (is_function(scope) || scope.kind() == "program"))
            || (!function_scoped && is_scope(scope))
        {
            return Some(scope);
        }
        parent = scope.parent();
    }
    None
}

pub(super) fn scoped_declaration(node: Node<'_>) -> bool {
    declaration_scope(node, false).is_some_and(|scope| scope.kind() != "program")
}

fn pattern_names<'a>(pattern: Node<'_>, source: &'a str, names: &mut Vec<&'a str>) {
        match pattern.kind() {
            "identifier" | "shorthand_property_identifier_pattern" => {
                names.push(pattern.utf8_text(source.as_bytes()).unwrap_or_default());
            }
            "assignment_pattern" | "object_assignment_pattern" => {
                if let Some(left) = pattern.child_by_field_name("left") {
                    pattern_names(left, source, names);
                }
            }
            "pair_pattern" => {
                if let Some(value) = pattern.child_by_field_name("value") {
                    pattern_names(value, source, names);
                }
            }
            "required_parameter" | "optional_parameter" => {
                if let Some(name) = pattern.child_by_field_name("pattern").or_else(|| pattern.child_by_field_name("name")) {
                    pattern_names(name, source, names);
                }
            }
            "formal_parameters" | "object_pattern" | "array_pattern" | "rest_pattern" | "parenthesized_expression" => {
                for index in 0..pattern.named_child_count() {
                    if let Some(child) = pattern.named_child(index as u32) {
                        pattern_names(child, source, names);
                    }
                }
            }
            _ => {}
        }
    }


fn scope_chain(node: Node<'_>) -> Vec<usize> {
        let mut scopes = Vec::new();
        let mut parent = node.parent();
        while let Some(scope) = parent {
            if is_scope(scope) {
                scopes.push(scope.id());
            }
            parent = scope.parent();
        }
    scopes
}

impl Bindings {
    fn declare(&mut self, scope: Node<'_>, name: &str, target: Option<&str>) {
        let value = target.map(str::to_owned);
        self.declarations.entry((scope.id(), name.to_owned()))
            .and_modify(|existing| {
                if *existing != value {
                    *existing = None;
                }
            })
            .or_insert(value);
    }

    fn pattern(&mut self, scope: Node<'_>, pattern: Node<'_>, source: &str) {
        let mut names = Vec::new();
        pattern_names(pattern, source, &mut names);
        for name in names {
            self.declare(scope, name, None);
        }
    }

    pub(super) fn function_scope(&mut self, node: Node<'_>, source: &str, target: Option<&str>) {
        if let Some(parameters) = node.child_by_field_name("parameters").or_else(|| node.child_by_field_name("parameter")) {
            self.pattern(node, parameters, source);
        }
        // A named function expression's self-name exists only in its own body.
        if node.kind() == "function_expression" {
            if let Some(name) = node.child_by_field_name("name") {
                self.declare(node, name.utf8_text(source.as_bytes()).unwrap_or_default(), target);
            }
        }
    }

    pub(super) fn observe_scope(&mut self, node: Node<'_>, source: &str) {
        if matches!(node.kind(), "type_alias_declaration" | "type_parameter") {
            if let (Some(scope), Some(name)) = (declaration_scope(node, false), node.child_by_field_name("name")) {
                self.types.insert((scope.id(), name.utf8_text(source.as_bytes()).unwrap_or_default().to_owned()), None);
            }
        } else if is_function(node) {
            self.function_scope(node, source, None);
        } else if matches!(node.kind(), "assignment_expression" | "augmented_assignment_expression" | "update_expression") {
            if let Some(left) = node.child_by_field_name("left").or_else(|| node.child_by_field_name("argument")) {
                self.write(node, left, source);
            }
        } else if node.kind() == "for_in_statement" {
            if let Some(left) = node.child_by_field_name("left") {
                let kind = (0..node.child_count()).filter_map(|index| node.child(index as u32))
                    .find(|child| matches!(child.kind(), "var" | "let" | "const" | "using"));
                if let Some(kind) = kind {
                    let scope = if kind.kind() == "var" { declaration_scope(node, true).unwrap_or(node) } else { node };
                    self.pattern(scope, left, source);
                } else {
                    self.write(node, left, source);
                }
            }
        } else if node.kind() == "catch_clause" {
            if let Some(parameter) = node.child_by_field_name("parameter") {
                self.pattern(node, parameter, source);
            }
        }
    }

    fn write(&mut self, node: Node<'_>, pattern: Node<'_>, source: &str) {
        let mut names = Vec::new();
        pattern_names(pattern, source, &mut names);
        let scopes = scope_chain(node);
        for name in names {
            self.writes.push((name.to_owned(), scopes.clone()));
        }
    }

    pub(super) fn variable(&mut self, node: Node<'_>, source: &str, target: Option<&str>) {
        let function_scoped = node.parent().is_some_and(|parent| parent.kind() == "variable_declaration");
        let Some(scope) = declaration_scope(node, function_scoped) else { return };
        let Some(name) = node.child_by_field_name("name") else { return };
        if name.kind() == "identifier" {
            let key = (scope.id(), name.utf8_text(source.as_bytes()).unwrap_or_default().to_owned());
            if self.declarations.contains_key(&key) {
                if self.instances.remove(&key).is_some() {
                    self.invalid_instances.insert(key.clone());
                }
            } else if let Some(constructor) = node.child_by_field_name("value")
                .map(super::unwrap_function_value)
                .filter(|value| value.kind() == "new_expression")
                .and_then(|value| value.child_by_field_name("constructor"))
                .filter(|constructor| matches!(constructor.kind(), "identifier" | "type_identifier"))
            {
                self.instances.insert(key, (constructor.utf8_text(source.as_bytes()).unwrap_or_default().to_owned(), scope_chain(node)));
            }
            self.declare(scope, name.utf8_text(source.as_bytes()).unwrap_or_default(), target);
            if scope.kind() != "program" {
                self.scoped_symbols.extend(target.map(str::to_owned));
            }
        } else {
            self.pattern(scope, name, source);
        }
    }

    pub(super) fn function_declaration(&mut self, node: Node<'_>, source: &str, target: &str) {
        let Some(scope) = declaration_scope(node, false) else { return };
        if let Some(name) = node.child_by_field_name("name") {
            self.declare(scope, name.utf8_text(source.as_bytes()).unwrap_or_default(), Some(target));
            if scope.kind() != "program" {
                self.scoped_symbols.insert(target.to_owned());
            }
        }
    }

    pub(super) fn type_declaration(&mut self, node: Node<'_>, symbol: &SymbolNode) {
        let Some(scope) = declaration_scope(node, false) else { return };
        self.types.entry((scope.id(), symbol.name.clone()))
            .and_modify(|target| *target = None)
            .or_insert_with(|| Some(symbol.id.clone()));
        if scope.kind() != "program" {
            self.scoped_symbols.insert(symbol.id.clone());
        }
        if symbol.kind == SymbolKind::Class {
            // The class body's self-name is independent of an outer reassignment.
            self.types.insert((node.id(), symbol.name.clone()), Some(symbol.id.clone()));
            self.declare(node, &symbol.name, Some(&symbol.id));
        }
    }

    pub(super) fn named_reference(&mut self, node: Node<'_>, reference_index: usize, value_namespace: bool) {
        self.named_references.push((reference_index, scope_chain(node), value_namespace));
    }

    pub(super) fn call(&mut self, node: Node<'_>, reference_index: usize, source: &str) {
        let scopes = scope_chain(node);
        self.calls.push((reference_index, scopes.clone(), None));
        if let Some(receiver) = node.child_by_field_name("function")
            .filter(|function| function.kind() == "member_expression")
            .and_then(|function| function.child_by_field_name("object"))
            .map(super::unwrap_function_value)
        {
            let receiver = match receiver.kind() {
                "identifier" => Some(Receiver::Name(receiver.utf8_text(source.as_bytes()).unwrap_or_default().to_owned())),
                "new_expression" => receiver.child_by_field_name("constructor")
                    .filter(|constructor| matches!(constructor.kind(), "identifier" | "type_identifier"))
                    .map(|constructor| Receiver::Constructed(constructor.utf8_text(source.as_bytes()).unwrap_or_default().to_owned())),
                "this" => {
                    let mut parent = node.parent();
                    let mut owner = None;
                    while let Some(scope) = parent {
                        if is_function(scope) && scope.kind() != "arrow_function" {
                            owner = Some(Receiver::This(scope.id()));
                            break;
                        }
                        parent = scope.parent();
                    }
                    owner
                }
                _ => None,
            };
            if let Some(receiver) = receiver {
                self.receivers.insert(reference_index, receiver);
            }
        }
        if let Some(arguments) = node.child_by_field_name("arguments") {
            let first = (0..arguments.named_child_count())
                .filter_map(|index| arguments.named_child(index as u32))
                .find(|child| child.kind() != "comment");
            if let Some(first) = first.filter(|child| child.kind() == "identifier") {
                self.calls.push((reference_index, scopes, Some(first.utf8_text(source.as_bytes()).unwrap_or_default().to_owned())));
            }
        }
    }

    pub(super) fn method(&mut self, node: Node<'_>, symbol: &SymbolNode) {
        let Some(parent) = symbol.parent_symbol_id.as_ref() else { return };
        let is_static = is_static(node);
        let accessor = (0..node.child_count()).filter_map(|index| node.child(index as u32))
            .any(|child| matches!(child.kind(), "get" | "set"));
        self.method_owners.insert(node.id(), (parent.clone(), is_static));
        self.methods.entry((parent.clone(), symbol.name.clone(), is_static))
            .and_modify(|target| *target = None)
            .or_insert_with(|| (!accessor).then(|| symbol.id.clone()));
    }

    fn binding_key(&self, name: &str, scopes: &[usize]) -> Option<(usize, String)> {
        scopes.iter().map(|scope| (*scope, name.to_owned()))
            .find(|key| self.declarations.contains_key(key))
    }

    fn class_binding(&self, name: &str, scopes: &[usize], kinds: &HashMap<&str, SymbolKind>) -> Option<String> {
        let key = self.binding_key(name, scopes)?;
        let id = self.declarations.get(&key)?.as_ref()?;
        (kinds.get(id.as_str()) == Some(&SymbolKind::Class)).then(|| id.clone())
    }

    fn member_binding(&self, receiver: &Receiver, name: &str, receiver_type: Option<&str>, scopes: &[usize], kinds: &HashMap<&str, SymbolKind>) -> Option<Option<String>> {
        let (owner, is_static) = match receiver {
            Receiver::Name(receiver) => {
                let key = self.binding_key(receiver, scopes)?;
                if self.invalid_instances.contains(&key) {
                    return Some(None);
                }
                if let Some(owner) = self.class_binding(receiver, scopes, kinds) {
                    (owner, true)
                } else {
                    let Some((constructor, construction_scopes)) = self.instances.get(&key) else {
                        // An existing type hint may name a scoped class, but an
                        // unsupported value/alias must not revive a global guess.
                        let scoped_type = receiver_type.and_then(|name| scopes.iter()
                            .find_map(|scope| self.types.get(&(*scope, name.to_owned()))))
                            .and_then(|target| target.as_ref())
                            .is_some_and(|target| self.scoped_symbols.contains(target));
                        return scoped_type.then_some(None);
                    };
                    let Some(owner) = self.class_binding(constructor, construction_scopes, kinds) else {
                        return self.binding_key(constructor, construction_scopes).map(|_| None);
                    };
                    (owner, false)
                }
            }
            Receiver::Constructed(constructor) => {
                let Some(owner) = self.class_binding(constructor, scopes, kinds) else {
                    return self.binding_key(constructor, scopes).map(|_| None);
                };
                (owner, false)
            }
            Receiver::This(method) => {
                let Some(owner) = self.method_owners.get(method) else { return Some(None) };
                owner.clone()
            }
        };
        // Keep this proof bounded to scoped classes. Other receiver types retain
        // the resolver's existing import, inheritance and return-type handling.
        if !self.scoped_symbols.contains(&owner) {
            return None;
        }
        Some(self.methods.get(&(owner, name.to_owned(), is_static)).cloned().flatten())
    }

    pub(super) fn finish(mut self, graph: &mut SemanticGraph) {
        for (name, scopes) in &self.writes {
            for scope in scopes {
                if let Some(target) = self.declarations.get_mut(&(*scope, name.clone())) {
                    *target = None;
                    let key = (*scope, name.clone());
                    if self.instances.remove(&key).is_some() {
                        self.invalid_instances.insert(key);
                    }
                    break;
                }
            }
        }
        // Parser order places containers before their children. Descendants of
        // a local class must not re-enter the global indexes through method names.
        for symbol in &graph.symbols {
            if symbol.parent_symbol_id.as_ref().is_some_and(|parent| self.scoped_symbols.contains(parent)) {
                self.scoped_symbols.insert(symbol.id.clone());
            }
        }
        let kinds = graph.symbols.iter().map(|symbol| (symbol.id.as_str(), symbol.kind)).collect::<HashMap<_, _>>();
        for (reference_index, scopes, value_namespace) in &self.named_references {
            let Some(reference) = graph.references.get(*reference_index) else { continue };
            let declarations = if *value_namespace { &self.declarations } else { &self.types };
            if let Some(target) = scopes.iter().find_map(|scope| declarations.get(&(*scope, reference.target_name.clone()))) {
                graph.lexical_bindings.named_references.push(LexicalReferenceBinding {
                    reference_index: *reference_index,
                    target_symbol_id: target.clone(),
                });
            }
        }
        for (reference_index, scopes, argument_name) in std::mem::take(&mut self.calls) {
            let Some(reference) = graph.references.get(reference_index) else { continue };
            if reference.kind != ReferenceKind::Call {
                continue;
            }
            let target = if argument_name.is_none() && reference.call_form == Some(CallForm::Member) {
                self.receivers.get(&reference_index)
                    .and_then(|receiver| self.member_binding(receiver, &reference.target_name, reference.receiver_type_name.as_deref(), &scopes, &kinds))
            } else if argument_name.is_some() || matches!(reference.call_form, Some(CallForm::Free | CallForm::Associated)) {
                self.binding_key(argument_name.as_ref().unwrap_or(&reference.target_name), &scopes)
                    .and_then(|key| self.declarations.get(&key))
                    .map(|target| target.as_ref().filter(|id| {
                        let kind = kinds.get(id.as_str());
                        if argument_name.is_none() && reference.call_form == Some(CallForm::Associated) {
                            kind == Some(&SymbolKind::Class)
                        } else {
                            kind == Some(&SymbolKind::Function)
                        }
                    }).cloned())
            } else {
                None
            };
            if let Some(target_symbol_id) = target {
                graph.lexical_bindings.calls.push(LexicalCallBinding {
                    reference_index,
                    argument_index: argument_name.as_ref().map(|_| 0),
                    target_symbol_id,
                });
            }
        }
        graph.lexical_bindings.scoped_symbol_ids = self.scoped_symbols.into_iter().collect();
    }
}
