use crate::graph::{CallForm, LexicalCallBinding, ReferenceKind, SemanticGraph};
use std::collections::{BTreeSet, HashMap};
use tree_sitter::Node;

#[derive(Default)]
pub(super) struct Bindings {
    declarations: HashMap<(usize, String), Option<String>>,
    scoped_symbols: BTreeSet<String>,
    calls: Vec<(usize, Vec<usize>, Option<String>)>,
    writes: Vec<(String, Vec<usize>)>,
}

fn is_function(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_declaration" | "function_expression" | "arrow_function" | "method_definition")
}

fn is_scope(node: Node<'_>) -> bool {
    is_function(node) || matches!(node.kind(), "program" | "statement_block" | "switch_body" | "for_statement" | "for_in_statement" | "catch_clause")
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
        if is_function(node) {
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

    pub(super) fn call(&mut self, node: Node<'_>, reference_index: usize, source: &str) {
        let scopes = scope_chain(node);
        self.calls.push((reference_index, scopes.clone(), None));
        if let Some(arguments) = node.child_by_field_name("arguments") {
            let first = (0..arguments.named_child_count())
                .filter_map(|index| arguments.named_child(index as u32))
                .find(|child| child.kind() != "comment");
            if let Some(first) = first.filter(|child| child.kind() == "identifier") {
                self.calls.push((reference_index, scopes, Some(first.utf8_text(source.as_bytes()).unwrap_or_default().to_owned())));
            }
        }
    }

    pub(super) fn finish(mut self, graph: &mut SemanticGraph) {
        for (name, scopes) in &self.writes {
            for scope in scopes {
                if let Some(target) = self.declarations.get_mut(&(*scope, name.clone())) {
                    *target = None;
                    break;
                }
            }
        }
        graph.lexical_bindings.scoped_symbol_ids = self.scoped_symbols.into_iter().collect();
        for (reference_index, scopes, argument_name) in self.calls {
            let Some(reference) = graph.references.get(reference_index) else { continue };
            if reference.kind != ReferenceKind::Call
                || (argument_name.is_none() && reference.call_form != Some(CallForm::Free)) {
                continue;
            }
            if let Some(target_symbol_id) = scopes.iter().find_map(|scope| {
                self.declarations.get(&(*scope, argument_name.as_ref().unwrap_or(&reference.target_name).clone()))
            }) {
                graph.lexical_bindings.calls.push(LexicalCallBinding {
                    reference_index,
                    argument_index: argument_name.as_ref().map(|_| 0),
                    target_symbol_id: target_symbol_id.clone(),
                });
            }
        }
    }
}
