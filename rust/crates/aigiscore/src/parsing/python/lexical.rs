use crate::graph::{CallForm, LexicalCallBinding, LexicalReferenceBinding, ReferenceKind, SemanticGraph, SymbolNode};
use std::collections::{BTreeSet, HashMap};
use std::ops::Range;
use tree_sitter::Node;

#[derive(Clone, PartialEq, Eq)]
enum Value {
    Class(String),
    Imported,
    Unknown,
}

#[derive(Default)]
pub(super) struct Bindings<'tree> {
    declarations: HashMap<(usize, String), Value>,
    directives: HashMap<(usize, String), bool>,
    scopes: HashMap<usize, Node<'tree>>,
    references: Vec<(usize, Node<'tree>)>,
    local_names: BTreeSet<String>,
    scoped_symbols: BTreeSet<String>,
}

fn function(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_definition" | "lambda")
}

fn comprehension(node: Node<'_>) -> bool {
    matches!(node.kind(), "list_comprehension" | "set_comprehension" | "dictionary_comprehension" | "generator_expression")
}

pub(super) fn is_local_class(node: Node<'_>) -> bool {
    let mut parent = node.parent();
    while let Some(scope) = parent {
        if function(scope) {
            return true;
        }
        parent = scope.parent();
    }
    false
}

// Defaults, decorators and base expressions execute outside the new body.
// A class namespace does not extend into nested functions or classes.
fn scope_chain(mut node: Node<'_>) -> Vec<Node<'_>> {
    let mut scopes = Vec::new();
    while let Some(parent) = node.parent() {
        if parent.kind() == "module" || comprehension(parent)
            || ((function(parent) || parent.kind() == "class_definition")
                && parent.child_by_field_name("body") == Some(node)
                && (parent.kind() != "class_definition" || scopes.is_empty()))
        {
            scopes.push(parent);
        }
        node = parent;
    }
    scopes
}

fn pattern_names(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        match node.kind() {
            "identifier" => names.push(node.utf8_text(source.as_bytes()).unwrap_or_default().to_owned()),
            "attribute" | "subscript" | "type" | "string" | "concatenated_string" => {}
            "default_parameter" | "typed_default_parameter" => {
                if let Some(name) = node.child_by_field_name("name") {
                    pending.push(name);
                }
            }
            "typed_parameter" => {
                for child in node.named_children(&mut node.walk()).filter(|child| child.kind() != "type") {
                    pending.push(child);
                }
            }
            "dotted_name" => {
                let text = node.utf8_text(source.as_bytes()).unwrap_or_default();
                if !text.contains('.') {
                    names.push(text.to_owned());
                }
            }
            "class_pattern" => {
                for child in node.named_children(&mut node.walk()).filter(|child| child.kind() != "dotted_name") {
                    pending.push(child);
                }
            }
            _ => {
                for child in node.named_children(&mut node.walk()) {
                    pending.push(child);
                }
            }
        }
    }
}

impl<'tree> Bindings<'tree> {
    fn declare(&mut self, scope: Node<'tree>, name: String, value: Value) {
        self.scopes.insert(scope.id(), scope);
        self.declarations.entry((scope.id(), name))
            .and_modify(|existing| {
                if *existing != value {
                    *existing = Value::Unknown;
                }
            })
            .or_insert(value);
    }

    fn pattern(&mut self, scope: Node<'tree>, pattern: Node<'_>, source: &str) {
        let mut names = Vec::new();
        pattern_names(pattern, source, &mut names);
        for name in names {
            self.declare(scope, name, Value::Unknown);
        }
    }

    pub(super) fn observe(&mut self, node: Node<'tree>, source: &str) {
        if function(node) {
            if let Some(parameters) = node.child_by_field_name("parameters") {
                self.pattern(node, parameters, source);
            }
        }
        if !matches!(node.kind(), "function_definition" | "assignment" | "augmented_assignment" | "for_statement" | "for_in_clause" | "named_expression" | "as_pattern" | "delete_statement" | "case_pattern" | "global_statement" | "nonlocal_statement" | "import_statement" | "import_from_statement") {
            return;
        }
        let Some(scope) = scope_chain(node).first().copied() else { return };
        match node.kind() {
            "function_definition" => {
                if let Some(name) = node.child_by_field_name("name") {
                    self.pattern(scope, name, source);
                }
            }
            "assignment" | "augmented_assignment" | "for_statement" | "for_in_clause" => {
                if let Some(left) = node.child_by_field_name("left") {
                    self.pattern(scope, left, source);
                }
            }
            "named_expression" => {
                if let (Some(scope), Some(name)) = (scope_chain(node).into_iter().find(|scope| !comprehension(*scope)), node.child_by_field_name("name")) {
                    self.pattern(scope, name, source);
                }
            }
            "as_pattern" => {
                if let Some(alias) = node.child_by_field_name("alias") {
                    self.pattern(scope, alias, source);
                }
            }
            "delete_statement" | "case_pattern" => self.pattern(scope, node, source),
            "global_statement" | "nonlocal_statement" => {
                self.scopes.insert(scope.id(), scope);
                for name in node.named_children(&mut node.walk()).filter(|child| child.kind() == "identifier") {
                    self.directives.insert((scope.id(), name.utf8_text(source.as_bytes()).unwrap_or_default().to_owned()), node.kind() == "global_statement");
                }
            }
            "import_statement" | "import_from_statement" => {
                let mut cursor = node.walk();
                for imported in node.children_by_field_name("name", &mut cursor) {
                    let name = if imported.kind() == "aliased_import" {
                        imported.child_by_field_name("alias").map(|alias| alias.utf8_text(source.as_bytes()).unwrap_or_default().to_owned())
                    } else {
                        Some(imported.utf8_text(source.as_bytes()).unwrap_or_default().split('.').next().unwrap_or_default().to_owned())
                    };
                    if let Some(name) = name {
                        self.declare(scope, name, if scope.kind() == "module" { Value::Imported } else { Value::Unknown });
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn class(&mut self, node: Node<'tree>, symbol: &SymbolNode, local: bool) {
        if let Some(scope) = scope_chain(node).first().copied() {
            let decorated = node.parent().is_some_and(|parent| parent.kind() == "decorated_definition");
            self.declare(scope, symbol.name.clone(), if decorated { Value::Unknown } else { Value::Class(symbol.id.clone()) });
        }
        if local {
            self.local_names.insert(symbol.name.clone());
            self.scoped_symbols.insert(symbol.id.clone());
        }
    }

    pub(super) fn references(&mut self, node: Node<'tree>, indexes: Range<usize>) {
        self.references.extend(indexes.map(|index| (index, node)));
    }

    fn lookup(&self, node: Node<'_>, name: &str) -> Option<&Value> {
        let scopes = scope_chain(node);
        if scopes.iter().any(|scope| comprehension(*scope)) {
            return Some(&Value::Unknown);
        }
        let mut nonlocal = false;
        for scope in &scopes {
            if nonlocal && !function(*scope) {
                continue;
            }
            let key = (scope.id(), name.to_owned());
            match self.directives.get(&key) {
                Some(true) => return Some(scopes.last().and_then(|module| self.declarations.get(&(module.id(), name.to_owned()))).unwrap_or(&Value::Unknown)),
                Some(false) => { nonlocal = true; continue; }
                None => {}
            }
            if let Some(value) = self.declarations.get(&key) {
                return Some(value);
            }
        }
        nonlocal.then_some(&Value::Unknown)
    }

    pub(super) fn finish(mut self, graph: &mut SemanticGraph) {
        if self.local_names.is_empty() {
            return;
        }
        // Writes through a directive invalidate the destination. A directive
        // alone redirects reads and does not change the referenced binding.
        for ((scope_id, name), global) in &self.directives {
            if !self.declarations.contains_key(&(*scope_id, name.clone())) {
                continue;
            }
            let Some(scope) = self.scopes.get(scope_id) else { continue };
            let ancestors = scope_chain(*scope);
            let target = if *global {
                ancestors.last().copied()
            } else {
                ancestors.into_iter().find(|parent| function(*parent)
                    && self.declarations.contains_key(&(parent.id(), name.clone())))
            };
            if let Some(target) = target {
                self.declarations.insert((target.id(), name.clone()), Value::Unknown);
            }
        }
        for symbol in &graph.symbols {
            if symbol.parent_symbol_id.as_ref().is_some_and(|parent| self.scoped_symbols.contains(parent)) {
                self.scoped_symbols.insert(symbol.id.clone());
            }
        }
        for (index, node) in &self.references {
            let reference = &graph.references[*index];
            let named = matches!(reference.kind, ReferenceKind::Type | ReferenceKind::Extends);
            let free = reference.kind == ReferenceKind::Call && reference.call_form == Some(CallForm::Free);
            if (named || free) && self.local_names.contains(&reference.target_name) {
                let target = match self.lookup(*node, &reference.target_name) {
                    Some(Value::Class(id)) => Some(id.clone()),
                    Some(Value::Unknown) | None => None,
                    Some(Value::Imported) => continue,
                };
                if free {
                    graph.lexical_bindings.calls.push(LexicalCallBinding { reference_index: *index, argument_index: None, target_symbol_id: target });
                } else {
                    graph.lexical_bindings.named_references.push(LexicalReferenceBinding { reference_index: *index, target_symbol_id: target });
                }
            } else if reference.kind == ReferenceKind::Call && reference.call_form == Some(CallForm::Member)
                && reference.receiver_type_name.as_ref().is_some_and(|name| self.local_names.contains(name)
                    && match self.lookup(*node, name) {
                        Some(Value::Class(id)) => self.scoped_symbols.contains(id),
                        Some(Value::Imported) => false,
                        Some(Value::Unknown) | None => true,
                    })
            {
                // The existing string-based receiver inference cannot prove
                // which local class an alias/instance carries across scopes.
                graph.lexical_bindings.calls.push(LexicalCallBinding { reference_index: *index, argument_index: None, target_symbol_id: None });
            }
        }
        graph.lexical_bindings.scoped_symbol_ids.extend(self.scoped_symbols);
    }
}
