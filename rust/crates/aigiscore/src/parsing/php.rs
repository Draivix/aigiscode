use super::add_file_module_symbol;
mod names;
use crate::graph::{
    CallForm, FileNode, Language, ReferenceKind, SemanticGraph, SemanticReference, SymbolKind,
    SymbolNode, Visibility,
};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use thiserror::Error;
use tree_sitter::{Node, Parser};

#[derive(Debug, Error)]
pub enum PhpParseError {
    #[error("failed to load tree-sitter PHP grammar")]
    Language,
    #[error("tree-sitter returned no parse tree")]
    MissingTree,
}

pub fn parse_php_to_graph(
    file_path: impl Into<PathBuf>,
    source: &str,
) -> Result<SemanticGraph, PhpParseError> {
    let file_path = file_path.into();
    trace(&format!("php parse start {}", file_path.display()));
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_php::LANGUAGE_PHP_ONLY.into())
        .map_err(|_| PhpParseError::Language)?;
    let tree = parser
        .parse(source, None)
        .ok_or(PhpParseError::MissingTree)?;
    trace(&format!("php tree built {}", file_path.display()));

    let mut graph = SemanticGraph::default();
    graph.add_file(FileNode {
        path: file_path.clone(),
        language: Language::Php,
    });
    add_file_module_symbol(&mut graph, &file_path, Language::Php, source);
    super::record_parse_outcome(&mut graph, &file_path, tree.root_node(), "tree-sitter-php");

    let mut context = PhpContext {
        file_path,
        source,
        names: names::PhpNames::new(tree.root_node(), source),
    };
    walk_tree(tree.root_node(), &mut context, &mut graph);
    super::behavior::capture(&mut graph, tree.root_node(), source);
    trace(&format!(
        "php walk complete {}",
        context.file_path.display()
    ));
    Ok(graph)
}

struct PhpContext<'a> {
    file_path: PathBuf,
    source: &'a str,
    names: names::PhpNames,
}

impl<'a> PhpContext<'a> {
    fn text(&self, node: Node<'_>) -> String {
        node.utf8_text(self.source.as_bytes())
            .map(str::to_owned)
            .unwrap_or_default()
    }

    fn line(&self, node: Node<'_>) -> usize {
        node.start_position().row + 1
    }

    fn anonymous_class_name(&self, node: Node<'_>) -> String {
        // Colons cannot occur in a PHP declaration name. Include the file so
        // anonymous types at equal offsets in one namespace remain distinct.
        format!(
            "anonymous:{:032x}:L{}:B{}",
            xxhash_rust::xxh3::xxh3_128(self.file_path.to_string_lossy().as_bytes()),
            self.line(node),
            node.start_byte()
        )
    }

    fn constructed_type(&self, node: Node<'_>) -> Option<String> {
        node.named_children(&mut node.walk())
            .find_map(|child| match child.kind() {
                "name" | "qualified_name" => Some(self.text(child)),
                "anonymous_class" => Some(
                    self.names.declaration(child, &self.anonymous_class_name(child)),
                ),
                _ => None,
            })
    }

    fn symbol_id(&self, kind: SymbolKind, parent: Option<&str>, name: &str) -> String {
        let prefix = match kind {
            SymbolKind::Class => "class",
            SymbolKind::Enum => "enum",
            SymbolKind::Function => "function",
            SymbolKind::Interface => "interface",
            SymbolKind::Method => "method",
            SymbolKind::Trait => "trait",
            _ => "symbol",
        };
        match parent {
            Some(parent) => format!("{prefix}:{}:{parent}:{name}", self.file_path.display()),
            None => format!("{prefix}:{}:{name}", self.file_path.display()),
        }
    }
}

fn walk_tree(node: Node<'_>, context: &mut PhpContext<'_>, graph: &mut SemanticGraph) {
    let mut stack = vec![(node, None::<String>, None::<String>)];
    while let Some((current, container_symbol_id, container_type_name)) = stack.pop() {
        match current.kind() {
            "anonymous_class" => {
                let name = context.anonymous_class_name(current);
                let mut symbol = make_symbol(
                    context,
                    SymbolKind::Class,
                    &name,
                    container_symbol_id.as_deref(),
                    None,
                    None,
                    Visibility::Private,
                    0,
                    0,
                    context.line(current),
                    current.end_position().row + 1,
                );
                symbol.qualified_name = context.names.declaration(current, &name);
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
                record_php_heritage(current, context, graph, Some(&symbol_id));
                // Constructor arguments execute in the enclosing scope. Only
                // the class body owns methods, properties and their `$this`.
                for idx in (0..current.child_count()).rev() {
                    if let Some(child) = current.child(idx as u32) {
                        let in_body = Some(child) == current.child_by_field_name("body");
                        stack.push((
                            child,
                            if in_body {
                                Some(symbol_id.clone())
                            } else {
                                container_symbol_id.clone()
                            },
                            if in_body {
                                Some(name.clone())
                            } else {
                                container_type_name.clone()
                            },
                        ));
                    }
                }
                continue;
            }
            "namespace_use_declaration" => {
                record_use_declaration(current, context, graph, container_symbol_id.as_deref());
            }
            "use_declaration" if container_symbol_id.is_some() => {
                record_trait_use_declaration(
                    current,
                    context,
                    graph,
                    container_symbol_id.as_deref(),
                );
            }
            "class_declaration"
            | "interface_declaration"
            | "trait_declaration"
            | "enum_declaration" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    let name = context.text(name_node);
                    let kind = match current.kind() {
                        "interface_declaration" => SymbolKind::Interface,
                        "trait_declaration" => SymbolKind::Trait,
                        "enum_declaration" => SymbolKind::Enum,
                        _ => SymbolKind::Class,
                    };
                    let mut symbol = make_symbol(
                        context,
                        kind,
                        &name,
                        None,
                        None,
                        None,
                        Visibility::Public,
                        0,
                        0,
                        context.line(name_node),
                        current.end_position().row + 1,
                    );
                    symbol.qualified_name = context.names.declaration(current, &name);
                    if is_conditionally_declared(current)
                        || graph
                            .symbols
                            .iter()
                            .any(|existing| existing.id == symbol.id)
                    {
                        symbol.id = format!(
                            "{}:L{}:B{}",
                            symbol.id,
                            symbol.start_line,
                            current.start_byte()
                        );
                    }
                    let symbol_id = symbol.id.clone();
                    graph.add_symbol(symbol);
                    record_php_heritage(current, context, graph, Some(symbol_id.as_str()));
                    push_children(&mut stack, current, Some(symbol_id), Some(name));
                    continue;
                }
            }
            "function_definition" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    let name = context.text(name_node);
                    let mut symbol = make_symbol(
                        context,
                        SymbolKind::Function,
                        &name,
                        container_symbol_id.as_deref(),
                        container_type_name.as_deref(),
                        function_return_type(current, context),
                        Visibility::Public,
                        parameter_count(current),
                        required_parameter_count(current),
                        context.line(name_node),
                        current.end_position().row + 1,
                    );
                    symbol.qualified_name = context.names.declaration(current, &name);
                    if graph
                        .symbols
                        .iter()
                        .any(|existing| existing.id == symbol.id)
                    {
                        symbol.id = format!(
                            "{}:L{}:B{}",
                            symbol.id,
                            symbol.start_line,
                            current.start_byte()
                        );
                    }
                    let symbol_id = symbol.id.clone();
                    graph.add_symbol(symbol);
                    record_parameter_types(current, context, graph, Some(symbol_id.as_str()));
                    push_children(
                        &mut stack,
                        current,
                        Some(symbol_id),
                        container_type_name.clone(),
                    );
                    continue;
                }
            }
            "method_declaration" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    let name = context.text(name_node);
                    let symbol = make_symbol(
                        context,
                        SymbolKind::Method,
                        &name,
                        container_symbol_id.as_deref(),
                        container_type_name.as_deref(),
                        function_return_type(current, context),
                        php_method_visibility(current, context),
                        parameter_count(current),
                        required_parameter_count(current),
                        context.line(name_node),
                        current.end_position().row + 1,
                    );
                    let symbol_id = symbol.id.clone();
                    graph.add_symbol(symbol);
                    record_parameter_types(current, context, graph, Some(symbol_id.as_str()));
                    push_children(
                        &mut stack,
                        current,
                        Some(symbol_id),
                        container_type_name.clone(),
                    );
                    continue;
                }
            }
            "function_call_expression"
            | "member_call_expression"
            | "nullsafe_member_call_expression"
            | "scoped_call_expression" => {
                record_call(
                    current,
                    context,
                    graph,
                    container_symbol_id.as_deref(),
                    container_type_name.as_deref(),
                );
            }
            "object_creation_expression" => {
                record_constructor_call(current, context, graph, container_symbol_id.as_deref());
            }
            _ => {}
        }

        push_children(
            &mut stack,
            current,
            container_symbol_id,
            container_type_name,
        );
    }
}

fn push_children<'a>(
    stack: &mut Vec<(Node<'a>, Option<String>, Option<String>)>,
    node: Node<'a>,
    container_symbol_id: Option<String>,
    container_type_name: Option<String>,
) {
    for idx in (0..node.child_count()).rev() {
        if let Some(child) = node.child(idx as u32) {
            stack.push((
                child,
                container_symbol_id.clone(),
                container_type_name.clone(),
            ));
        }
    }
}

/// Whether a declaration sits inside a conditional (`if`/`else`) rather than at
/// program or namespace level. PHP allows the same class name to be declared in
/// mutually-exclusive branches (`if (class_exists(...)) { class X } else
/// { class X }`), so a conditional declaration cannot rely on name uniqueness.
fn is_conditionally_declared(node: Node<'_>) -> bool {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        match ancestor.kind() {
            "if_statement" | "else_clause" => return true,
            "program" | "namespace_definition" => return false,
            _ => current = ancestor.parent(),
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn make_symbol(
    context: &PhpContext<'_>,
    kind: SymbolKind,
    name: &str,
    parent_symbol_id: Option<&str>,
    container_type_name: Option<&str>,
    return_type_name: Option<String>,
    visibility: Visibility,
    parameter_count: usize,
    required_parameter_count: usize,
    start_line: usize,
    end_line: usize,
) -> SymbolNode {
    let qualified_name = match container_type_name {
        Some(container) => format!("{container}::{name}"),
        None => name.to_owned(),
    };
    SymbolNode {
        id: context.symbol_id(kind, parent_symbol_id, name),
        file_path: context.file_path.clone(),
        kind,
        name: name.to_owned(),
        qualified_name,
        parent_symbol_id: parent_symbol_id.map(str::to_owned),
        owner_type_name: container_type_name.map(str::to_owned),
        return_type_name,
        visibility,
        parameter_count,
        required_parameter_count,
        start_line,
        end_line,
    }
}

fn record_use_declaration(
    node: Node<'_>,
    context: &PhpContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    for clause in node.children(&mut node.walk()) {
        if clause.kind() != "namespace_use_clause" {
            continue;
        }
        let names = clause
            .children(&mut clause.walk())
            .filter(|child| matches!(child.kind(), "qualified_name" | "name"))
            .map(|child| context.text(child))
            .collect::<Vec<_>>();
        if names.is_empty() {
            continue;
        }
        let target_name = names[0].clone();
        let binding_name = if names.len() > 1 {
            names
                .last()
                .cloned()
                .unwrap_or_else(|| leaf_namespace_name(&target_name))
        } else {
            leaf_namespace_name(&target_name)
        };
        graph.add_reference(SemanticReference {
            file_path: context.file_path.clone(),
            enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
            kind: ReferenceKind::Import,
            target_name,
            binding_name: Some(binding_name),
            line: context.line(clause),
            arity: None,
            receiver_name: None,
            receiver_type_name: None,
            call_form: None,
            class_literal_argument: None,
            class_literal_arguments: Vec::new(),
        });
    }
}

fn record_trait_use_declaration(
    node: Node<'_>,
    context: &PhpContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    for child in node.children(&mut node.walk()) {
        if !matches!(child.kind(), "name" | "qualified_name") {
            continue;
        }
        graph.add_reference(SemanticReference {
            file_path: context.file_path.clone(),
            enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
            kind: ReferenceKind::Type,
            target_name: context.text(child),
            binding_name: None,
            line: context.line(child),
            arity: None,
            receiver_name: None,
            receiver_type_name: None,
            call_form: None,
            class_literal_argument: None,
            class_literal_arguments: Vec::new(),
        });
    }
}

fn record_php_heritage(
    node: Node<'_>,
    context: &PhpContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "base_clause" => {
                if let Some(parent) = child
                    .children(&mut child.walk())
                    .find(|candidate| matches!(candidate.kind(), "name" | "qualified_name"))
                {
                    graph.add_reference(SemanticReference {
                        file_path: context.file_path.clone(),
                        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                        kind: ReferenceKind::Extends,
                        target_name: context.text(parent),
                        binding_name: None,
                        line: context.line(parent),
                        arity: None,
                        receiver_name: None,
                        receiver_type_name: None,
                        call_form: None,
                        class_literal_argument: None,
                        class_literal_arguments: Vec::new(),
                    });
                }
            }
            "class_interface_clause" => {
                for interface in child.children(&mut child.walk()) {
                    if !matches!(interface.kind(), "name" | "qualified_name") {
                        continue;
                    }
                    graph.add_reference(SemanticReference {
                        file_path: context.file_path.clone(),
                        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                        kind: ReferenceKind::Implements,
                        target_name: context.text(interface),
                        binding_name: None,
                        line: context.line(interface),
                        arity: None,
                        receiver_name: None,
                        receiver_type_name: None,
                        call_form: None,
                        class_literal_argument: None,
                        class_literal_arguments: Vec::new(),
                    });
                }
            }
            _ => {}
        }
    }
}

fn record_parameter_types(
    node: Node<'_>,
    context: &PhpContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(parameters) = parameters_node(node) else {
        return;
    };
    for parameter in parameters.children(&mut parameters.walk()) {
        if !is_php_parameter_node(parameter) {
            continue;
        }
        let Some(type_node) = parameter.child_by_field_name("type") else {
            continue;
        };
        graph.add_reference(SemanticReference {
            file_path: context.file_path.clone(),
            enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
            kind: ReferenceKind::Type,
            target_name: context.text(type_node),
            binding_name: None,
            line: context.line(type_node),
            arity: None,
            receiver_name: None,
            receiver_type_name: None,
            call_form: None,
            class_literal_argument: None,
            class_literal_arguments: Vec::new(),
        });
    }
}

fn record_call(
    node: Node<'_>,
    context: &PhpContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
    container_type_name: Option<&str>,
) {
    match node.kind() {
        "function_call_expression" => {
            let Some(function_node) = node.child_by_field_name("function") else {
                return;
            };
            graph.add_reference(SemanticReference {
                file_path: context.file_path.clone(),
                enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                kind: ReferenceKind::Call,
                target_name: context.text(function_node),
                binding_name: None,
                line: context.line(node),
                arity: Some(argument_count(node)),
                receiver_name: None,
                receiver_type_name: None,
                call_form: Some(CallForm::Free),
                class_literal_argument: first_class_literal_argument(node, context),
                class_literal_arguments: class_literal_arguments(node, context),
            });
        }
        "member_call_expression" | "nullsafe_member_call_expression" => {
            let receiver_node = node.child_by_field_name("object");
            let receiver_name = receiver_node.map(|n| context.text(n));
            let receiver_type_name = receiver_node.and_then(|receiver_node| {
                infer_member_receiver_type(
                    receiver_node,
                    call_node_owner(node),
                    context,
                    container_type_name,
                )
            });
            let target_name = node
                .child_by_field_name("name")
                .map(|n| context.text(n))
                .unwrap_or_else(|| context.text(node));
            graph.add_reference(SemanticReference {
                file_path: context.file_path.clone(),
                enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                kind: ReferenceKind::Call,
                target_name,
                binding_name: None,
                line: context.line(node),
                arity: Some(argument_count(node)),
                receiver_name,
                receiver_type_name,
                call_form: Some(CallForm::Member),
                class_literal_argument: first_class_literal_argument(node, context),
                class_literal_arguments: class_literal_arguments(node, context),
            });
        }
        "scoped_call_expression" => {
            let receiver_name = node.child_by_field_name("scope").map(|n| context.text(n));
            let receiver_type_name = receiver_name.as_deref().and_then(|name| {
                if name.eq_ignore_ascii_case("self") {
                    container_type_name.map(|owner| context.names.declaration(node, owner))
                } else {
                    context.names.resolve(node, name)
                }
            });
            let target_name = node
                .child_by_field_name("name")
                .map(|n| context.text(n))
                .unwrap_or_else(|| context.text(node));
            graph.add_reference(SemanticReference {
                file_path: context.file_path.clone(),
                enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                kind: ReferenceKind::Call,
                target_name,
                binding_name: None,
                line: context.line(node),
                arity: Some(argument_count(node)),
                receiver_name,
                receiver_type_name,
                call_form: Some(CallForm::Associated),
                class_literal_argument: first_class_literal_argument(node, context),
                class_literal_arguments: class_literal_arguments(node, context),
            });
        }
        _ => {}
    }
}

fn record_constructor_call(
    node: Node<'_>,
    context: &PhpContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(target_name) = context.constructed_type(node) else {
        return;
    };
    let arguments_owner = node
        .named_children(&mut node.walk())
        .find(|child| child.kind() == "anonymous_class")
        .unwrap_or(node);
    graph.add_reference(SemanticReference {
        file_path: context.file_path.clone(),
        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
        kind: ReferenceKind::Call,
        target_name,
        binding_name: None,
        line: context.line(node),
        arity: Some(argument_count(arguments_owner)),
        receiver_name: None,
        receiver_type_name: None,
        call_form: Some(CallForm::Associated),
        class_literal_argument: None,
        class_literal_arguments: Vec::new(),
    });
}

fn parameter_count(node: Node<'_>) -> usize {
    parameters_node(node)
        .map(|parameters| {
            parameters
                .children(&mut parameters.walk())
                .filter(|child| is_php_parameter_node(*child))
                .count()
        })
        .unwrap_or(0)
}

fn required_parameter_count(node: Node<'_>) -> usize {
    parameters_node(node)
        .map(|parameters| {
            parameters
                .children(&mut parameters.walk())
                .filter(|child| is_php_parameter_node(*child))
                .filter(|child| child.child_by_field_name("default_value").is_none())
                .count()
        })
        .unwrap_or(0)
}

fn parameters_node(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("parameters").or_else(|| {
        node.children(&mut node.walk())
            .find(|child| child.kind().contains("parameters"))
    })
}

fn argument_count(node: Node<'_>) -> usize {
    node.child_by_field_name("arguments")
        .or_else(|| {
            node.named_children(&mut node.walk())
                .find(|child| child.kind() == "arguments")
        })
        .map(|arguments| {
            arguments
                .children(&mut arguments.walk())
                .filter(|child| !matches!(child.kind(), "(" | ")" | "," | "comment"))
                .count()
        })
        .unwrap_or(0)
}

fn call_node_owner(node: Node<'_>) -> Node<'_> {
    let mut current = node;
    while let Some(parent) = current.parent() {
        if matches!(parent.kind(), "function_definition" | "method_declaration") {
            return parent;
        }
        current = parent;
    }
    node
}

fn infer_receiver_type_with_guards(
    scope_node: Node<'_>,
    receiver_name: &str,
    context: &PhpContext<'_>,
    active_receivers: &mut HashSet<String>,
    active_calls: &mut HashSet<usize>,
) -> Option<String> {
    if receiver_name.is_empty() {
        return None;
    }
    if !active_receivers.insert(receiver_name.to_owned()) {
        return None;
    }
    let inferred = collect_receiver_type(
        scope_node,
        receiver_name,
        context,
        active_receivers,
        active_calls,
    );
    active_receivers.remove(receiver_name);
    inferred
}

/// Preserve the dispatch boundary: protected methods can be called by subclasses.
fn php_method_visibility(node: Node<'_>, context: &PhpContext<'_>) -> Visibility {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            return match context.text(child).as_str() {
                "private" => Visibility::Private,
                "protected" => Visibility::Protected,
                _ => Visibility::Public,
            };
        }
    }
    Visibility::Public
}

fn infer_member_receiver_type(
    receiver_node: Node<'_>,
    scope_node: Node<'_>,
    context: &PhpContext<'_>,
    container_type_name: Option<&str>,
) -> Option<String> {
    let mut active_receivers = HashSet::new();
    let mut active_calls = HashSet::new();
    // `$this->prop->method()`: the type lives on the class property, not on
    // any local binding — check it before the generic fallbacks.
    if let Some(property_type) = infer_this_property_type(receiver_node, scope_node, context) {
        return Some(property_type);
    }
    match receiver_node.kind() {
        "variable_name" if context.text(receiver_node) == "$this" => {
            container_type_name.map(|name| context.names.declaration(receiver_node, name))
        }
        "object_creation_expression" => context.constructed_type(receiver_node),
        "function_call_expression"
        | "member_call_expression"
        | "nullsafe_member_call_expression"
        | "scoped_call_expression" => infer_call_result_type(
            receiver_node,
            scope_node,
            context,
            &mut active_receivers,
            &mut active_calls,
        ),
        _ => infer_receiver_type_with_guards(
            scope_node,
            &context.text(receiver_node),
            context,
            &mut active_receivers,
            &mut active_calls,
        ),
    }
}

fn collect_receiver_type(
    node: Node<'_>,
    receiver_name: &str,
    context: &PhpContext<'_>,
    active_receivers: &mut HashSet<String>,
    active_calls: &mut HashSet<usize>,
) -> Option<String> {
    let mut stack = vec![node];
    while let Some(current) = stack.pop() {
        if current.kind() == "anonymous_class" {
            // Its arguments belong to this scope, its declarations do not.
            stack.extend(current.named_children(&mut current.walk())
                .filter(|child| child.kind() == "arguments"));
            continue;
        }
        match current.kind() {
            _ if is_php_parameter_node(current) => {
                let name = current
                    .child_by_field_name("name")
                    .map(|child| context.text(child));
                if name.as_deref() == Some(receiver_name) {
                    return current
                        .child_by_field_name("type")
                        .map(|type_node| context.text(type_node));
                }
            }
            "assignment_expression" => {
                let left = current
                    .child_by_field_name("left")
                    .map(|child| context.text(child));
                if left.as_deref() == Some(receiver_name) {
                    // An explicit `/** @var Foo $x */` docblock is human-stated
                    // intent and wins over inference; when it is absent (or
                    // names a different variable) inference still applies.
                    return docblock_type_for_assignment(current, receiver_name, context).or_else(
                        || {
                            current.child_by_field_name("right").and_then(|right| {
                                if right.kind() == "object_creation_expression" {
                                    return context.constructed_type(right);
                                }
                                infer_call_result_type(
                                    right,
                                    call_node_owner(current),
                                    context,
                                    active_receivers,
                                    active_calls,
                                )
                            })
                        },
                    );
                }
            }
            _ => {}
        }
        for idx in (0..current.child_count()).rev() {
            if let Some(child) = current.child(idx as u32) {
                stack.push(child);
            }
        }
    }
    None
}

fn is_php_parameter_node(node: Node<'_>) -> bool {
    node.child_by_field_name("name").is_some() && node.kind().contains("parameter")
}

/// `$this->prop` receiver: the type is the enclosing class property's
/// declared, promoted, or `@var`-documented type. Constructor promotion
/// (`private readonly Foo $foo`) is the dominant Laravel service pattern.
fn infer_this_property_type(
    receiver_node: Node<'_>,
    scope_node: Node<'_>,
    context: &PhpContext<'_>,
) -> Option<String> {
    if receiver_node.kind() != "member_access_expression" {
        return None;
    }
    let object = receiver_node.child_by_field_name("object")?;
    if context.text(object) != "$this" {
        return None;
    }
    let property_name = receiver_node
        .child_by_field_name("name")
        .map(|name| context.text(name))?;
    let property_var = format!("${property_name}");

    let mut class_node = Some(scope_node);
    while let Some(node) = class_node {
        if matches!(
            node.kind(),
            "class_declaration"
                | "anonymous_class"
                | "enum_declaration"
                | "trait_declaration"
                | "interface_declaration"
        ) {
            break;
        }
        class_node = node.parent();
    }
    let class_node = class_node?;

    let mut stack = vec![class_node];
    while let Some(current) = stack.pop() {
        if current != class_node
            && matches!(current.kind(), "class_declaration" | "anonymous_class")
        {
            continue;
        }
        match current.kind() {
            "property_declaration" => {
                let declares_property = current
                    .children(&mut current.walk())
                    .filter(|child| child.kind() == "property_element")
                    .any(|element| {
                        element
                            .children(&mut element.walk())
                            .filter(|part| part.kind() == "variable_name")
                            .any(|name| context.text(name) == property_var)
                    });
                if declares_property {
                    if let Some(type_node) = current.child_by_field_name("type") {
                        return Some(context.text(type_node));
                    }
                    return docblock_type_for_assignment(current, &property_var, context);
                }
            }
            _ if is_php_parameter_node(current) => {
                // Promoted constructor property: visibility/readonly marker
                // distinguishes it from an ordinary parameter.
                let text = context.text(current);
                let promoted = ["private ", "protected ", "public ", "readonly "]
                    .iter()
                    .any(|marker| text.contains(marker));
                let name = current
                    .child_by_field_name("name")
                    .map(|child| context.text(child));
                if promoted && name.as_deref() == Some(property_var.as_str()) {
                    return current
                        .child_by_field_name("type")
                        .map(|type_node| context.text(type_node));
                }
            }
            _ => {}
        }
        for idx in (0..current.child_count()).rev() {
            if let Some(child) = current.child(idx as u32) {
                stack.push(child);
            }
        }
    }
    None
}

/// Type declared by a `/** @var Foo $x */` docblock immediately above an
/// assignment. The docblock hangs on the wrapping expression statement (or,
/// grammar-dependent, on the assignment itself); only the immediate previous
/// sibling counts — anything looser would guess at human intent.
fn docblock_type_for_assignment(
    assignment: Node<'_>,
    receiver_name: &str,
    context: &PhpContext<'_>,
) -> Option<String> {
    let comment = assignment
        .prev_named_sibling()
        .filter(|sibling| sibling.kind() == "comment")
        .or_else(|| {
            assignment
                .parent()
                .and_then(|parent| parent.prev_named_sibling())
                .filter(|sibling| sibling.kind() == "comment")
        })?;
    parse_var_docblock(&context.text(comment), receiver_name)
}

/// Parse `@var <type> [$name]` from a doc comment. A named variable must match
/// the receiver, otherwise the docblock does not apply (inference still runs).
/// Nullable and union shapes collapse to the first concrete class; array
/// shapes (`Foo[]`) bind nothing — element calls are not member calls on Foo.
fn parse_var_docblock(comment: &str, receiver_name: &str) -> Option<String> {
    let after_var = comment.split_once("@var")?.1;
    let mut tokens = after_var.split_whitespace();
    let raw_type = tokens.next()?;
    let type_name = raw_type
        .trim_start_matches('?')
        .split('|')
        .next()?
        .trim_start_matches('\\');
    if type_name.is_empty()
        || type_name.ends_with("[]")
        || !type_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '\\')
    {
        return None;
    }
    if let Some(variable) = tokens.next().filter(|token| token.starts_with('$')) {
        if variable != receiver_name {
            return None;
        }
    }
    Some(type_name.to_string())
}

fn function_return_type(node: Node<'_>, context: &PhpContext<'_>) -> Option<String> {
    node.child_by_field_name("return_type")
        .map(|type_node| context.text(type_node))
}

fn infer_call_result_type(
    node: Node<'_>,
    scope_node: Node<'_>,
    context: &PhpContext<'_>,
    active_receivers: &mut HashSet<String>,
    active_calls: &mut HashSet<usize>,
) -> Option<String> {
    if !matches!(
        node.kind(),
        "function_call_expression"
            | "member_call_expression"
            | "nullsafe_member_call_expression"
            | "scoped_call_expression"
    ) {
        return None;
    }
    if !active_calls.insert(node.id()) {
        return None;
    }
    let inferred = match node.kind() {
        "function_call_expression" => {
            let function_name = node
                .child_by_field_name("function")
                .map(|function| leaf_namespace_name(&context.text(function)));
            match function_name.as_deref() {
                // Container helpers return the class they are asked to resolve:
                // `$x = app(Foo::class)` types $x as Foo, not as `app`. Ecosystem
                // idiom (Laravel), not a repo-specific heuristic.
                Some("app") | Some("resolve") => {
                    first_class_literal_argument(node, context).or(function_name)
                }
                _ => function_name,
            }
        }
        "member_call_expression" | "nullsafe_member_call_expression" => {
            let receiver_name = node
                .child_by_field_name("object")
                .map(|receiver| context.text(receiver));
            let method_name = node
                .child_by_field_name("name")
                .map(|name| context.text(name));
            // When the inner receiver's type is unknown, a bare method name is
            // NOT a type: returning `whereNull` as the "type" of a chained
            // builder call used to poison receiver resolution downstream.
            // Unknown stays unknown.
            match (receiver_name, method_name) {
                (Some(receiver_name), Some(method_name)) => infer_receiver_type_with_guards(
                    scope_node,
                    &receiver_name,
                    context,
                    active_receivers,
                    active_calls,
                )
                .map(|receiver_type_name| format!("{receiver_type_name}::{method_name}")),
                _ => None,
            }
        }
        "scoped_call_expression" => {
            let receiver_name = node
                .child_by_field_name("scope")
                .map(|scope| leaf_namespace_name(&context.text(scope)));
            let method_name = node
                .child_by_field_name("name")
                .map(|name| context.text(name));
            match (receiver_name, method_name) {
                (Some(receiver_name), Some(method_name)) => {
                    Some(format!("{receiver_name}::{method_name}"))
                }
                (_, method_name) => method_name,
            }
        }
        _ => None,
    };
    active_calls.remove(&node.id());
    inferred
}

fn leaf_namespace_name(value: &str) -> String {
    value
        .rsplit('\\')
        .next()
        .unwrap_or(value)
        .trim_start_matches('\\')
        .to_owned()
}

/// First `Foo::class` argument of a call — the type a container helper
/// (`app()`, `resolve()`) actually returns. Grammar versions vary on whether
/// arguments are wrapped in an `argument` node, so both shapes are accepted.
fn first_class_literal_argument(node: Node<'_>, context: &PhpContext<'_>) -> Option<String> {
    let arguments = node.child_by_field_name("arguments")?;
    let argument = arguments
        .named_children(&mut arguments.walk())
        .find(|child| child.kind() == "argument")?;
    class_literal_argument(argument, node, context)
}

fn class_literal_arguments(node: Node<'_>, context: &PhpContext<'_>) -> Vec<Option<String>> {
    let Some(arguments) = node.child_by_field_name("arguments") else { return Vec::new(); };
    let values = arguments.named_children(&mut arguments.walk()).filter(|child| child.kind() == "argument").take(16)
        .map(|argument| class_literal_argument(argument, node, context)).collect::<Vec<_>>();
    if values.iter().all(Option::is_none) { Vec::new() } else { values }
}

fn class_literal_argument(argument: Node<'_>, node: Node<'_>, context: &PhpContext<'_>) -> Option<String> {
    if argument.child_by_field_name("name").is_some() {
        return None;
    }
    let constant = argument
        .named_children(&mut argument.walk())
        .find(|child| child.kind() != "comment")?;
    if constant.kind() != "class_constant_access_expression" {
        return None;
    }
    let parts = constant
        .named_children(&mut constant.walk())
        .filter(|child| child.kind() != "comment")
        .collect::<Vec<_>>();
    let [name, member] = parts.as_slice() else {
        return None;
    };
    if !matches!(name.kind(), "name" | "qualified_name" | "relative_name")
        || !context.text(*member).eq_ignore_ascii_case("class")
    {
        return None;
    }
    context.names.resolve(node, &context.text(*name))
}

fn trace(message: &str) {
    if env::var_os("AIGISCORE_TRACE").is_some() {
        eprintln!("[aigiscore] {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::parse_php_to_graph;
    use crate::graph::{CallForm, Language, ReferenceKind, SymbolKind};
    use std::path::PathBuf;

    #[test]
    fn anonymous_classes_own_methods_but_not_constructor_arguments() {
        let source = r#"<?php
namespace Domain;
interface Worker { public function work(): void; }
class Repository { public function save(): void {} }
class Host {
    public function provide(): Repository { return new Repository; }
    public function work(): void {}
    public function run(): void {
        $first = new class($this->provide()) implements Worker {
            public function __construct(private Repository $store) {}
            public function work(): void { $this->store->save(); }
            public function again(): void { $this->work(); self::work(); }
        };
        $second = new class implements Worker {
            public function work(): void {}
        };
        $first->work();
        $second->work();
        $this->work();
        $this->store->save();
    }
}
"#;
        let mut graph = parse_php_to_graph("domain/Host.php", source).unwrap();
        assert!(graph.parse_outcomes.iter().all(|outcome| outcome.diagnostics.is_empty()));
        let classes = graph.symbols.iter()
            .filter(|symbol| symbol.kind == SymbolKind::Class && symbol.name.starts_with("anonymous:"))
            .cloned().collect::<Vec<_>>();
        assert_eq!(classes.len(), 2);
        assert_ne!(classes[0].id, classes[1].id);
        let ids = graph.symbols.iter().map(|symbol| &symbol.id)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(ids.len(), graph.symbols.len());
        let method = |owner: &str, name: &str| graph.symbols.iter()
            .find(|symbol| symbol.parent_symbol_id.as_deref() == Some(owner) && symbol.name == name)
            .unwrap().id.clone();
        let host = graph.symbols.iter().find(|symbol| symbol.name == "Host").unwrap();
        let host_work = method(&host.id, "work");
        let run = method(&host.id, "run");
        let first_work = method(&classes[0].id, "work");
        let second_work = method(&classes[1].id, "work");
        assert!(classes.iter().all(|symbol| symbol.parent_symbol_id.as_ref() == Some(&run)));
        let argument_call = graph.references.iter().find(|reference| reference.target_name == "provide").unwrap();
        assert_eq!(argument_call.enclosing_symbol_id.as_ref(), Some(&run));
        assert_eq!(argument_call.receiver_type_name.as_deref(), Some("Domain\\Host"));
        let constructors = graph.references.iter()
            .filter(|reference| classes.iter().any(|symbol| symbol.qualified_name == reference.target_name))
            .collect::<Vec<_>>();
        assert_eq!(constructors.len(), 2);
        assert_eq!(constructors[0].arity, Some(1));
        assert_eq!(constructors[1].arity, Some(0));
        for class in &classes {
            assert!(graph.references.iter().any(|reference| reference.kind == ReferenceKind::Implements
                && reference.enclosing_symbol_id.as_ref() == Some(&class.id)
                && reference.target_name == "Worker"));
        }
        let saves = graph.references.iter().filter(|reference| reference.target_name == "save")
            .collect::<Vec<_>>();
        assert_eq!(saves.len(), 2);
        assert_eq!(saves[0].receiver_type_name.as_deref(), Some("Repository"));
        assert_eq!(saves[1].receiver_type_name, None, "inner promoted property is not a Host property");
        let calls = graph.references.iter().filter(|reference| reference.target_name == "work")
            .map(|reference| (reference.line, reference.receiver_name.clone())).collect::<Vec<_>>();
        crate::resolve::resolve_graph(&mut graph);
        for (line, receiver) in calls {
            let expected = match receiver.as_deref() {
                Some("$first") => &first_work,
                Some("$second") => &second_work,
                Some("$this") if line == 19 => &host_work,
                Some("$this" | "self") => &first_work,
                _ => panic!("unexpected work receiver {receiver:?}"),
            };
            let edges = graph.resolved_edges.iter().filter(|edge| edge.line == line
                && edge.reference_target_name.as_deref() == Some("work")).collect::<Vec<_>>();
            assert!(!edges.is_empty());
            assert!(edges.iter().all(|edge| &edge.target_symbol_id == expected));
        }
        let other = parse_php_to_graph("domain/Other.php", source).unwrap();
        assert!(!other.symbols.iter().any(|symbol| classes.iter()
            .any(|class| class.qualified_name == symbol.qualified_name)));
    }

    #[test]
    fn this_property_receivers_resolve_via_promoted_declared_and_docblock_types() {
        let graph = parse_php_to_graph(
            PathBuf::from("app/Services/EmailService.php"),
            r#"<?php
namespace App\Services;

class EmailService
{
    /** @var LegacySyncManager */
    private $legacy;

    public function __construct(
        private readonly EmailSyncManager $syncManager,
        private AccountRepository $accounts,
    ) {}

    public function sync(): void
    {
        $this->syncManager->queueFullSync($account);
        $this->accounts->findActive();
        $this->legacy->runLegacy();
        $this->dynamic->anything();
    }
}
"#,
        )
        .unwrap();
        let type_of = |target: &str| {
            graph
                .references
                .iter()
                .find(|reference| reference.target_name == target)
                .and_then(|reference| reference.receiver_type_name.as_deref())
        };
        assert_eq!(type_of("queueFullSync"), Some("EmailSyncManager"));
        assert_eq!(type_of("findActive"), Some("AccountRepository"));
        assert_eq!(type_of("runLegacy"), Some("LegacySyncManager"));
        assert_eq!(type_of("anything"), None, "dynamic property stays unknown");
    }

    #[test]
    fn var_docblocks_bind_receiver_types_above_assignments() {
        let graph = parse_php_to_graph(
            PathBuf::from("app/Console/SyncCommand.php"),
            r#"<?php
namespace App\Console;

class SyncCommand
{
    public function handle($repository, $factory, $id): void
    {
        /** @var EmailAccount $account */
        $account = $repository->findAccount($id);
        $account->getEmailAddress();

        /** @var EmailSyncManager */
        $manager = $factory->make();
        $manager->queueFullSync($account);

        /** @var Ignored $notX */
        $x = $repository->findAccount($id);
        $x->getName();

        /** @var EmailAccount[] $accounts */
        $accounts = $repository->all();
        $accounts->count();
    }
}
"#,
        )
        .unwrap();
        let type_of = |target: &str| {
            graph
                .references
                .iter()
                .find(|reference| reference.target_name == target)
                .and_then(|reference| reference.receiver_type_name.as_deref())
        };
        assert_eq!(type_of("getEmailAddress"), Some("EmailAccount"));
        assert_eq!(type_of("queueFullSync"), Some("EmailSyncManager"));
        assert_eq!(
            type_of("getName"),
            None,
            "docblock naming $notX must not bind $x"
        );
        assert_eq!(
            type_of("count"),
            None,
            "array shape Foo[] binds no member type"
        );
    }

    #[test]
    fn container_helper_assignments_type_the_receiver_as_the_resolved_class() {
        let graph = parse_php_to_graph(
            PathBuf::from("app/Console/SyncCommand.php"),
            r#"<?php
namespace App\Console;

use App\Services\EmailSyncManager;

class SyncCommand
{
    public function handle(): void
    {
        $syncManager = app(EmailSyncManager::class);
        $syncManager->queueFullSync($account);

        $made = resolve(EmailSyncManager::class);
        $made->queueFullSync($account);
    }
}
"#,
        )
        .unwrap();
        let receiver_types = graph
            .references
            .iter()
            .filter(|reference| reference.target_name == "queueFullSync")
            .filter_map(|reference| reference.receiver_type_name.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(receiver_types.len(), 2);
        assert!(
            receiver_types
                .iter()
                .all(|receiver_type| *receiver_type == "App\\Services\\EmailSyncManager"),
            "app()/resolve() must bind the ::class argument as the type, got: {receiver_types:?}"
        );
    }

    #[test]
    fn conditionally_declared_duplicate_classes_get_distinct_symbol_ids() {
        let graph = parse_php_to_graph(
            PathBuf::from("app/Providers/TelescopeServiceProvider.php"),
            r#"<?php
if (class_exists(Base::class)) {
    class TelescopeServiceProvider extends Base
    {
        public function register(): void {}
    }
} else {
    class TelescopeServiceProvider extends Fallback
    {
        public function register(): void {}
    }
}
"#,
        )
        .unwrap();

        let class_ids: Vec<&str> = graph
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.kind == SymbolKind::Class && symbol.name == "TelescopeServiceProvider"
            })
            .map(|symbol| symbol.id.as_str())
            .collect();
        assert_eq!(class_ids.len(), 2, "both branch classes extracted");
        assert_ne!(
            class_ids[0], class_ids[1],
            "branch classes must not share an ID"
        );

        let method_ids: Vec<&str> = graph
            .symbols
            .iter()
            .filter(|symbol| symbol.kind == SymbolKind::Method && symbol.name == "register")
            .map(|symbol| symbol.id.as_str())
            .collect();
        assert_eq!(method_ids.len(), 2);
        assert_ne!(
            method_ids[0], method_ids[1],
            "methods embed the parent class ID, so they must diverge too"
        );
    }

    #[test]
    fn preserves_namespace_identity_and_members_of_all_php_containers() {
        let source = r#"<?php
namespace First { class Same { public function run() {} } }
namespace Second { class Same { public function run() {} } }
namespace Third { trait Helpers { private function helper() {} } interface Contract { public function execute(); } enum State { case Open; public function label() {} } }
"#;
        let graph = parse_php_to_graph(PathBuf::from("definitions.php"), source).unwrap();
        for name in [
            "First\\Same",
            "Second\\Same",
            "Third\\Helpers",
            "Third\\Contract",
            "Third\\State",
        ] {
            assert!(graph
                .symbols
                .iter()
                .any(|symbol| symbol.qualified_name == name));
        }
        for name in ["helper", "execute", "label"] {
            assert!(graph
                .symbols
                .iter()
                .any(|symbol| symbol.kind == SymbolKind::Method && symbol.name == name));
        }
        let ids = graph
            .symbols
            .iter()
            .map(|symbol| &symbol.id)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(ids.len(), graph.symbols.len());
    }

    #[test]
    fn parses_php_symbols_and_imports() {
        let graph = parse_php_to_graph(
            PathBuf::from("app/Service.php"),
            r#"<?php
use App\Models\User as U;
class Service extends Base implements Contract {
    public function run(U $user) {
        helper();
        $user->save();
        $this->save();
        new User();
    }
}
function helper() {}
"#,
        )
        .unwrap();

        assert!(graph
            .files
            .iter()
            .any(|file| file.language == Language::Php));
        assert!(graph
            .symbols
            .iter()
            .any(|symbol| symbol.kind == SymbolKind::Class && symbol.name == "Service"));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Import
                && reference.target_name == "App\\Models\\User"
                && reference.binding_name.as_deref() == Some("U")
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Extends && reference.target_name == "Base"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Implements && reference.target_name == "Contract"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "save"
                && reference.receiver_name.as_deref() == Some("$user")
                && reference.receiver_type_name.as_deref() == Some("U")
                && reference.call_form == Some(CallForm::Member)
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "save"
                && reference.call_form == Some(CallForm::Member)
        }));
    }

    #[test]
    fn uses_leaf_binding_name_for_non_aliased_php_imports() {
        let graph = parse_php_to_graph(
            PathBuf::from("app/Action.php"),
            r#"<?php
use App\Entities\_Core\EntityRegistry;

final class Action
{
    public function handle(): void
    {
        EntityRegistry::get('Task');
    }
}
"#,
        )
        .unwrap();

        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Import
                && reference.target_name == "App\\Entities\\_Core\\EntityRegistry"
                && reference.binding_name.as_deref() == Some("EntityRegistry")
        }));
    }

    #[test]
    fn records_php_function_parameter_ranges() {
        let graph = parse_php_to_graph(
            PathBuf::from("app/helpers.php"),
            r#"<?php
function esc_attr( $text ) {}
function translate( $text, $domain = 'default' ) {}
"#,
        )
        .unwrap();

        let esc_attr = graph
            .symbols
            .iter()
            .find(|symbol| symbol.name == "esc_attr")
            .unwrap();
        assert_eq!(esc_attr.parameter_count, 1);
        assert_eq!(esc_attr.required_parameter_count, 1);

        let translate = graph
            .symbols
            .iter()
            .find(|symbol| symbol.name == "translate")
            .unwrap();
        assert_eq!(translate.parameter_count, 2);
        assert_eq!(translate.required_parameter_count, 1);
    }

    #[test]
    fn records_promoted_parameter_types_and_trait_use_as_type_references() {
        let graph = parse_php_to_graph(
            PathBuf::from("app/Console/DemoCommand.php"),
            r#"<?php
use App\Services\ThingManager;
use App\Support\DbalRowAccess;

class DemoCommand {
    use DbalRowAccess;

    public function __construct(
        private readonly ThingManager $thingManager,
    ) {}
}
"#,
        )
        .unwrap();

        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Type && reference.target_name == "ThingManager"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Type && reference.target_name == "DbalRowAccess"
        }));
    }
}
