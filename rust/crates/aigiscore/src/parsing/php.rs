use super::add_file_module_symbol;
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

    let mut context = PhpContext { file_path, source };
    walk_tree(tree.root_node(), &mut context, &mut graph);
    trace(&format!(
        "php walk complete {}",
        context.file_path.display()
    ));
    Ok(graph)
}

struct PhpContext<'a> {
    file_path: PathBuf,
    source: &'a str,
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
            "class_declaration" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    let name = context.text(name_node);
                    let mut symbol = make_symbol(
                        context,
                        SymbolKind::Class,
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
                    // `if (class_exists(X)) { class Y } else { class Y }` declares
                    // the same class name twice in one file; without line
                    // qualification both bodies (and their methods, which embed
                    // the parent ID) collapse onto one symbol ID.
                    if is_conditionally_declared(current) {
                        symbol.id = format!("{}:L{}", symbol.id, symbol.start_line);
                    }
                    let symbol_id = symbol.id.clone();
                    graph.add_symbol(symbol);
                    record_php_heritage(current, context, graph, Some(symbol_id.as_str()));
                    push_children(&mut stack, current, Some(symbol_id), Some(name));
                    continue;
                }
            }
            "interface_declaration" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    let name = context.text(name_node);
                    let symbol = make_symbol(
                        context,
                        SymbolKind::Interface,
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
                    graph.add_symbol(symbol);
                }
                continue;
            }
            "trait_declaration" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    let name = context.text(name_node);
                    let symbol = make_symbol(
                        context,
                        SymbolKind::Trait,
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
                    graph.add_symbol(symbol);
                }
                continue;
            }
            "enum_declaration" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    let name = context.text(name_node);
                    let symbol = make_symbol(
                        context,
                        SymbolKind::Enum,
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
                    graph.add_symbol(symbol);
                }
                continue;
            }
            "function_definition" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    let name = context.text(name_node);
                    let symbol = make_symbol(
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
                        Visibility::Public,
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

#[allow(clippy::too_many_arguments)]
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
            });
        }
        "scoped_call_expression" => {
            let receiver_name = node.child_by_field_name("scope").map(|n| context.text(n));
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
                receiver_type_name: None,
                call_form: Some(CallForm::Associated),
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
    let Some(target) = node
        .children(&mut node.walk())
        .find(|child| matches!(child.kind(), "name" | "qualified_name"))
    else {
        return;
    };
    graph.add_reference(SemanticReference {
        file_path: context.file_path.clone(),
        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
        kind: ReferenceKind::Call,
        target_name: context.text(target),
        binding_name: None,
        line: context.line(node),
        arity: Some(argument_count(node)),
        receiver_name: None,
        receiver_type_name: None,
        call_form: Some(CallForm::Associated),
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

fn infer_member_receiver_type(
    receiver_node: Node<'_>,
    scope_node: Node<'_>,
    context: &PhpContext<'_>,
    container_type_name: Option<&str>,
) -> Option<String> {
    let mut active_receivers = HashSet::new();
    let mut active_calls = HashSet::new();
    match receiver_node.kind() {
        "variable_name" if context.text(receiver_node) == "$this" => {
            container_type_name.map(str::to_owned)
        }
        "object_creation_expression" => {
            receiver_node
                .children(&mut receiver_node.walk())
                .find_map(|child| {
                    matches!(child.kind(), "name" | "qualified_name").then_some(context.text(child))
                })
        }
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
                    return current.child_by_field_name("right").and_then(|right| {
                        if right.kind() == "object_creation_expression" {
                            return right.children(&mut right.walk()).find_map(|child| {
                                matches!(child.kind(), "name" | "qualified_name")
                                    .then_some(context.text(child))
                            });
                        }
                        infer_call_result_type(
                            right,
                            call_node_owner(current),
                            context,
                            active_receivers,
                            active_calls,
                        )
                    });
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
        "function_call_expression" => node
            .child_by_field_name("function")
            .map(|function| leaf_namespace_name(&context.text(function))),
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
