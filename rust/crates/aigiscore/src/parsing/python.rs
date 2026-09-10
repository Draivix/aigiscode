use super::add_file_module_symbol;
use crate::graph::{
    CallForm, FileNode, Language, ReferenceKind, SemanticGraph, SemanticReference, SymbolKind,
    SymbolNode, Visibility,
};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::vec::Vec;
use thiserror::Error;
use tree_sitter::{Node, Parser};

mod lexical;

#[derive(Debug, Error)]
pub enum PythonParseError {
    #[error("failed to load tree-sitter Python grammar")]
    Language,
    #[error("tree-sitter returned no parse tree")]
    MissingTree,
}

pub fn parse_python_to_graph(
    file_path: impl Into<PathBuf>,
    source: &str,
) -> Result<SemanticGraph, PythonParseError> {
    let file_path = file_path.into();
    trace(&format!("python parse start {}", file_path.display()));
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .map_err(|_| PythonParseError::Language)?;
    let tree = parser
        .parse(source, None)
        .ok_or(PythonParseError::MissingTree)?;
    trace(&format!("python tree built {}", file_path.display()));

    let mut graph = SemanticGraph::default();
    graph.add_file(FileNode {
        path: file_path.clone(),
        language: Language::Python,
    });
    add_file_module_symbol(&mut graph, &file_path, Language::Python, source);
    super::record_parse_outcome(&mut graph, &file_path, tree.root_node(), "tree-sitter-python");

    let mut context = PythonContext { file_path, source };
    walk_tree(tree.root_node(), &mut context, &mut graph);
    super::behavior::capture(&mut graph, tree.root_node(), source);
    trace(&format!(
        "python walk complete {}",
        context.file_path.display()
    ));
    Ok(graph)
}

struct PythonContext<'a> {
    file_path: PathBuf,
    source: &'a str,
}

impl<'a> PythonContext<'a> {
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
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            _ => "symbol",
        };
        match parent {
            Some(parent) => format!("{prefix}:{}:{parent}:{name}", self.file_path.display()),
            None => format!("{prefix}:{}:{name}", self.file_path.display()),
        }
    }
}

fn walk_tree(node: Node<'_>, context: &mut PythonContext<'_>, graph: &mut SemanticGraph) {
    let mut bindings = lexical::Bindings::default();
    let mut stack = vec![(node, None::<String>, None::<String>)];
    while let Some((current, container_symbol_id, container_type_name)) = stack.pop() {
        bindings.observe(current, context.source);
        let first_reference = graph.references.len();
        match current.kind() {
            "import_from_statement" => {
                record_import_from(current, context, graph, container_symbol_id.as_deref());
            }
            "import_statement" => {
                record_import(current, context, graph, container_symbol_id.as_deref());
            }
            "class_definition" => {
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
                    let local = lexical::is_local_class(current);
                    if local {
                        let scope = format!("{}@{}:{}", container_symbol_id.as_deref().unwrap_or("scope"), context.line(current), current.start_position().column);
                        symbol.id = context.symbol_id(SymbolKind::Class, Some(&scope), &name);
                        symbol.parent_symbol_id = container_symbol_id.clone();
                    }
                    bindings.class(current, &symbol, local);
                    let symbol_id = symbol.id.clone();
                    graph.add_symbol(symbol);
                    record_superclasses(current, context, graph, Some(symbol_id.as_str()));
                    bindings.references(current, first_reference..graph.references.len());
                    push_children(&mut stack, current, Some(symbol_id), Some(name));
                    continue;
                }
            }
            "function_definition" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    let name = context.text(name_node);
                    let kind = if container_type_name.is_some() {
                        SymbolKind::Method
                    } else {
                        SymbolKind::Function
                    };
                    let visibility = if name.starts_with('_') && name != "__init__" {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    };
                    let symbol = make_symbol(
                        context,
                        kind,
                        &name,
                        container_symbol_id.as_deref(),
                        container_type_name.as_deref(),
                        function_return_type(current, context),
                        visibility,
                        parameter_count(current),
                        parameter_count(current),
                        context.line(name_node),
                        current.end_position().row + 1,
                    );
                    let symbol_id = symbol.id.clone();
                    graph.add_symbol(symbol);
                    record_decorators(current, context, graph, Some(symbol_id.as_str()));
                    record_parameter_types(current, context, graph, Some(symbol_id.as_str()));
                    bindings.references(current, first_reference..graph.references.len());
                    push_children(
                        &mut stack,
                        current,
                        Some(symbol_id),
                        container_type_name.clone(),
                    );
                    continue;
                }
            }
            "call" => {
                record_call(
                    current,
                    context,
                    graph,
                    container_symbol_id.as_deref(),
                    container_type_name.as_deref(),
                );
            }
            _ => {}
        }
        bindings.references(current, first_reference..graph.references.len());
        push_children(
            &mut stack,
            current,
            container_symbol_id,
            container_type_name,
        );
    }
    bindings.finish(graph);
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
fn make_symbol(
    context: &PythonContext<'_>,
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

fn record_import_from(
    node: Node<'_>,
    context: &PythonContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(module_node) = node.child_by_field_name("module_name") else {
        return;
    };
    let module_name = context.text(module_node);
    let mut after_import_keyword = false;
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            let kind = child.kind();
            if kind == "import" {
                after_import_keyword = true;
                continue;
            }
            if !after_import_keyword {
                continue;
            }
            match kind {
                "aliased_import" => {
                    let imported_name = child.child(0).map(|n| context.text(n)).unwrap_or_default();
                    let binding_name = child
                        .child(child.child_count().saturating_sub(1) as u32)
                        .map(|n| context.text(n))
                        .unwrap_or_else(|| imported_name.clone());
                    graph.add_reference(SemanticReference {
                        file_path: context.file_path.clone(),
                        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                        kind: ReferenceKind::Import,
                        target_name: format!("{module_name}::{imported_name}"),
                        binding_name: Some(binding_name),
                        line: context.line(child),
                        arity: None,
                        receiver_name: None,
                        receiver_type_name: None,
                        call_form: None,
                        class_literal_argument: None,
                        class_literal_arguments: Vec::new(),
                    });
                }
                "dotted_name" | "identifier" => {
                    let imported_name = context.text(child);
                    graph.add_reference(SemanticReference {
                        file_path: context.file_path.clone(),
                        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                        kind: ReferenceKind::Import,
                        target_name: format!("{module_name}::{imported_name}"),
                        binding_name: Some(last_dotted_segment(&imported_name)),
                        line: context.line(child),
                        arity: None,
                        receiver_name: None,
                        receiver_type_name: None,
                        call_form: None,
                        class_literal_argument: None,
                        class_literal_arguments: Vec::new(),
                    });
                }
                _ => {}
            }
        }
    }
}

fn record_import(
    node: Node<'_>,
    context: &PythonContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            match child.kind() {
                "aliased_import" => {
                    let imported_name = child.child(0).map(|n| context.text(n)).unwrap_or_default();
                    let binding_name = child
                        .child(child.child_count().saturating_sub(1) as u32)
                        .map(|n| context.text(n))
                        .unwrap_or_else(|| last_dotted_segment(&imported_name));
                    graph.add_reference(SemanticReference {
                        file_path: context.file_path.clone(),
                        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                        kind: ReferenceKind::Import,
                        target_name: imported_name,
                        binding_name: Some(binding_name),
                        line: context.line(child),
                        arity: None,
                        receiver_name: None,
                        receiver_type_name: None,
                        call_form: None,
                        class_literal_argument: None,
                        class_literal_arguments: Vec::new(),
                    });
                }
                "dotted_name" => {
                    let imported_name = context.text(child);
                    graph.add_reference(SemanticReference {
                        file_path: context.file_path.clone(),
                        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                        kind: ReferenceKind::Import,
                        target_name: imported_name.clone(),
                        binding_name: Some(last_dotted_segment(&imported_name)),
                        line: context.line(child),
                        arity: None,
                        receiver_name: None,
                        receiver_type_name: None,
                        call_form: None,
                        class_literal_argument: None,
                        class_literal_arguments: Vec::new(),
                    });
                }
                _ => {}
            }
        }
    }
}

fn record_superclasses(
    node: Node<'_>,
    context: &PythonContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    if let Some(superclasses) = node.child_by_field_name("superclasses") {
        for idx in 0..superclasses.child_count() {
            if let Some(child) = superclasses.child(idx as u32) {
                if child.kind() == "identifier" {
                    graph.add_reference(SemanticReference {
                        file_path: context.file_path.clone(),
                        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                        kind: ReferenceKind::Extends,
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
        }
    }
}

fn record_parameter_types(
    node: Node<'_>,
    context: &PythonContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    if let Some(parameters) = node.child_by_field_name("parameters") {
        for idx in 0..parameters.child_count() {
            if let Some(child) = parameters.child(idx as u32) {
                if child.kind() == "typed_parameter" {
                    if let Some(type_node) = child.child_by_field_name("type") {
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
            }
        }
    }
}

fn record_call(
    node: Node<'_>,
    context: &PythonContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
    container_type_name: Option<&str>,
) {
    let Some(function_node) = node.child_by_field_name("function") else {
        return;
    };
    match function_node.kind() {
        "identifier" => {
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
                class_literal_argument: None,
                class_literal_arguments: Vec::new(),
            });
        }
        "attribute" => {
            let receiver_node = function_node.child_by_field_name("object");
            let receiver_name = receiver_node.map(|n| context.text(n));
            let receiver_type_name = receiver_node.and_then(|receiver_node| {
                infer_member_receiver_type(
                    receiver_node,
                    call_node_owner(node),
                    context,
                    container_type_name,
                )
            });
            let target_name = function_node
                .children(&mut function_node.walk())
                .last()
                .map(|n| context.text(n))
                .unwrap_or_else(|| context.text(function_node));
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
                class_literal_argument: None,
                class_literal_arguments: Vec::new(),
            });
        }
        _ => {}
    }
}

fn record_decorators(
    node: Node<'_>,
    context: &PythonContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(decorated_definition) = node
        .parent()
        .filter(|parent| parent.kind() == "decorated_definition")
    else {
        return;
    };
    for idx in 0..decorated_definition.named_child_count() {
        let Some(child) = decorated_definition.named_child(idx as u32) else {
            continue;
        };
        if child.kind() != "decorator" {
            continue;
        }
        let Some(target_node) = child.named_child(0) else {
            continue;
        };
        match target_node.kind() {
            "identifier" => {
                graph.add_reference(SemanticReference {
                    file_path: context.file_path.clone(),
                    enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                    kind: ReferenceKind::Call,
                    target_name: context.text(target_node),
                    binding_name: None,
                    line: context.line(child),
                    arity: Some(0),
                    receiver_name: None,
                    receiver_type_name: None,
                    call_form: Some(CallForm::Free),
                    class_literal_argument: None,
                    class_literal_arguments: Vec::new(),
                });
            }
            "attribute" => {
                let receiver_node = target_node.child_by_field_name("object");
                let receiver_name = receiver_node.map(|n| context.text(n));
                let receiver_type_name = receiver_node.and_then(|receiver_node| {
                    infer_member_receiver_type(
                        receiver_node,
                        call_node_owner(target_node),
                        context,
                        None,
                    )
                });
                let target_name = target_node
                    .children(&mut target_node.walk())
                    .last()
                    .map(|n| context.text(n))
                    .unwrap_or_else(|| context.text(target_node));
                graph.add_reference(SemanticReference {
                    file_path: context.file_path.clone(),
                    enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                    kind: ReferenceKind::Call,
                    target_name,
                    binding_name: None,
                    line: context.line(child),
                    arity: Some(0),
                    receiver_name,
                    receiver_type_name,
                    call_form: Some(CallForm::Member),
                    class_literal_argument: None,
                    class_literal_arguments: Vec::new(),
                });
            }
            "call" => {
                if let Some(function_node) = target_node.child_by_field_name("function") {
                    match function_node.kind() {
                        "identifier" => {
                            graph.add_reference(SemanticReference {
                                file_path: context.file_path.clone(),
                                enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                                kind: ReferenceKind::Call,
                                target_name: context.text(function_node),
                                binding_name: None,
                                line: context.line(child),
                                arity: Some(argument_count(target_node)),
                                receiver_name: None,
                                receiver_type_name: None,
                                call_form: Some(CallForm::Free),
                                class_literal_argument: None,
                                class_literal_arguments: Vec::new(),
                            });
                        }
                        "attribute" => {
                            let receiver_node = function_node.child_by_field_name("object");
                            let receiver_name = receiver_node.map(|n| context.text(n));
                            let receiver_type_name = receiver_node.and_then(|receiver_node| {
                                infer_member_receiver_type(
                                    receiver_node,
                                    call_node_owner(target_node),
                                    context,
                                    None,
                                )
                            });
                            let target_name = function_node
                                .children(&mut function_node.walk())
                                .last()
                                .map(|n| context.text(n))
                                .unwrap_or_else(|| context.text(function_node));
                            graph.add_reference(SemanticReference {
                                file_path: context.file_path.clone(),
                                enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                                kind: ReferenceKind::Call,
                                target_name,
                                binding_name: None,
                                line: context.line(child),
                                arity: Some(argument_count(target_node)),
                                receiver_name,
                                receiver_type_name,
                                call_form: Some(CallForm::Member),
                                class_literal_argument: None,
                                class_literal_arguments: Vec::new(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn parameter_count(node: Node<'_>) -> usize {
    node.child_by_field_name("parameters")
        .map(|parameters| {
            parameters
                .children(&mut parameters.walk())
                .filter(|child| {
                    matches!(
                        child.kind(),
                        "identifier" | "typed_parameter" | "default_parameter"
                    )
                })
                .count()
        })
        .unwrap_or(0)
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
        if matches!(parent.kind(), "function_definition" | "lambda") {
            return parent;
        }
        current = parent;
    }
    node
}

fn infer_receiver_type_with_guards(
    scope_node: Node<'_>,
    receiver_name: &str,
    context: &PythonContext<'_>,
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
    context: &PythonContext<'_>,
    container_type_name: Option<&str>,
) -> Option<String> {
    let mut active_receivers = HashSet::new();
    let mut active_calls = HashSet::new();
    match receiver_node.kind() {
        "identifier" if context.text(receiver_node) == "self" => {
            container_type_name.map(str::to_owned)
        }
        "call" => infer_call_result_type(
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
    scope_node: Node<'_>,
    receiver_name: &str,
    context: &PythonContext<'_>,
    active_receivers: &mut HashSet<String>,
    active_calls: &mut HashSet<usize>,
) -> Option<String> {
    let mut stack = vec![scope_node];
    while let Some(node) = stack.pop() {
        let inferred = match node.kind() {
            "typed_parameter" => {
                let name = node
                    .child_by_field_name("name")
                    .or_else(|| {
                        node.children(&mut node.walk())
                            .find(|child| child.kind() == "identifier")
                    })
                    .map(|child| context.text(child));
                if name.as_deref() == Some(receiver_name) {
                    node.child_by_field_name("type")
                        .map(|type_node| context.text(type_node))
                } else {
                    None
                }
            }
            "assignment" => {
                let left = node
                    .child_by_field_name("left")
                    .map(|child| context.text(child));
                if left.as_deref() == Some(receiver_name) {
                    node.child_by_field_name("type")
                        .map(|type_node| context.text(type_node))
                        .or_else(|| {
                            node.child_by_field_name("right").and_then(|right| {
                                if right.kind() != "call" {
                                    return None;
                                }
                                infer_call_result_type(
                                    right,
                                    call_node_owner(node),
                                    context,
                                    active_receivers,
                                    active_calls,
                                )
                            })
                        })
                } else {
                    None
                }
            }
            _ => None,
        };
        if inferred.is_some() {
            return inferred;
        }
        for idx in (0..node.child_count()).rev() {
            if let Some(child) = node.child(idx as u32) {
                stack.push(child);
            }
        }
    }
    None
}

fn function_return_type(node: Node<'_>, context: &PythonContext<'_>) -> Option<String> {
    node.child_by_field_name("return_type")
        .map(|type_node| context.text(type_node))
        .filter(|value| !value.is_empty())
}

fn infer_call_result_type(
    node: Node<'_>,
    scope_node: Node<'_>,
    context: &PythonContext<'_>,
    active_receivers: &mut HashSet<String>,
    active_calls: &mut HashSet<usize>,
) -> Option<String> {
    if node.kind() != "call" {
        return None;
    }
    if !active_calls.insert(node.id()) {
        return None;
    }
    let function = node.child_by_field_name("function")?;
    let inferred = match function.kind() {
        "identifier" => Some(context.text(function)),
        "attribute" => {
            let receiver_name = function
                .child_by_field_name("object")
                .map(|receiver| context.text(receiver));
            let method_name = function
                .children(&mut function.walk())
                .last()
                .map(|child| context.text(child));
            match (receiver_name, method_name) {
                (Some(receiver_name), Some(method_name)) => {
                    let receiver_type_name = infer_receiver_type_with_guards(
                        scope_node,
                        &receiver_name,
                        context,
                        active_receivers,
                        active_calls,
                    );
                    receiver_type_name
                        .or_else(|| static_receiver_type_name(function, context))
                        .map(|receiver_type_name| format!("{receiver_type_name}::{method_name}"))
                        .or(Some(method_name))
                }
                (_, method_name) => method_name,
            }
        }
        _ => {
            let candidate = leaf_identifier(function, context);
            (!candidate.is_empty()).then_some(candidate)
        }
    };
    active_calls.remove(&node.id());
    inferred
}

fn static_receiver_type_name(
    attribute_node: Node<'_>,
    context: &PythonContext<'_>,
) -> Option<String> {
    let receiver_node = attribute_node.child_by_field_name("object")?;
    let candidate = leaf_identifier(receiver_node, context);
    candidate
        .chars()
        .next()
        .filter(|character| character.is_uppercase())?;
    Some(candidate)
}

fn leaf_identifier(node: Node<'_>, context: &PythonContext<'_>) -> String {
    let mut stack = vec![node];
    while let Some(current) = stack.pop() {
        if matches!(current.kind(), "identifier" | "type") {
            return context.text(current);
        }
        for idx in (0..current.child_count()).rev() {
            if let Some(child) = current.child(idx as u32) {
                stack.push(child);
            }
        }
    }
    String::new()
}

fn last_dotted_segment(value: &str) -> String {
    value.rsplit('.').next().unwrap_or(value).to_owned()
}

fn trace(message: &str) {
    if env::var_os("AIGISCORE_TRACE").is_some() {
        eprintln!("[aigiscore-python] {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::parse_python_to_graph;
    use crate::graph::{CallForm, Language, ReferenceKind, SymbolKind, Visibility};
    use std::path::{Path, PathBuf};

    #[test]
    fn parses_python_symbols_and_imports() {
        let graph = parse_python_to_graph(
            PathBuf::from("app/service.py"),
            r#"from .models import User as U, Repo
import app.services.helper as helper_mod
class Service(Base):
    def run(self, user: U):
        helper()
        user.save()
        self.save()
def helper():
    pass
"#,
        )
        .unwrap();

        assert!(graph
            .files
            .iter()
            .any(|file| file.language == Language::Python));
        assert!(graph
            .symbols
            .iter()
            .any(|symbol| symbol.kind == SymbolKind::Class && symbol.name == "Service"));
        assert!(graph
            .symbols
            .iter()
            .any(|symbol| symbol.kind == SymbolKind::Method
                && symbol.qualified_name == "Service::run"));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Import
                && reference.target_name == ".models::User"
                && reference.binding_name.as_deref() == Some("U")
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Extends && reference.target_name == "Base"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Type && reference.target_name == "U"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "save"
                && reference.receiver_name.as_deref() == Some("user")
                && reference.receiver_type_name.as_deref() == Some("U")
                && reference.call_form == Some(CallForm::Member)
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "save"
                && reference.call_form == Some(CallForm::Member)
        }));
        assert!(graph.symbols.iter().any(|symbol| {
            symbol.kind == SymbolKind::Method
                && symbol.name == "run"
                && symbol.visibility == Visibility::Public
        }));
    }

    #[test]
    fn local_class_constructors_keep_identity_and_mask_foreign_names() {
        let source = r#"
def first():
    class Local: pass
    value = Local()
    def closure():
        nonlocal Local
        return Local()
def second():
    class Local: pass
    return Local()
def third():
    class Local: pass
    return Local()
def outside():
    return Local()
def parameter(Local):
    return Local()
def global_reader():
    global Local
    return Local()
def changed():
    class Changed: pass
    def replace():
        nonlocal Changed
        Changed = callback
    return Changed()
"#;
        let mut graph = parse_python_to_graph("local.py", source).unwrap();
        graph.append(parse_python_to_graph("foreign.py", "class Local: pass\nclass Changed: pass\n").unwrap());
        crate::resolve::resolve_graph(&mut graph);
        let symbols = graph.symbols.iter().map(|symbol| (symbol.id.as_str(), symbol)).collect::<std::collections::HashMap<_, _>>();
        assert_eq!(symbols.len(), graph.symbols.len());
        let calls = graph.resolved_edges.iter().filter(|edge| edge.source_file_path == Path::new("local.py")
            && edge.reference_target_name.as_deref() == Some("Local") && edge.kind == ReferenceKind::Call).collect::<Vec<_>>();
        assert_eq!(calls.len(), 4);
        assert!(calls.iter().all(|edge| edge.target_file_path == Path::new("local.py") && edge.reason == "call:lexical-binding"));
        let mut owners = calls.iter().map(|edge| {
            let class = symbols[edge.target_symbol_id.as_str()];
            symbols[class.parent_symbol_id.as_deref().unwrap()].name.as_str()
        }).collect::<Vec<_>>();
        owners.sort_unstable();
        assert_eq!(owners, ["first", "first", "second", "third"]);
        assert!(!graph.resolved_edges.iter().any(|edge| edge.reference_target_name.as_deref() == Some("Changed")
            && edge.kind == ReferenceKind::Call));
    }

    #[test]
    fn local_class_lookup_skips_method_namespaces_and_default_parameter_scopes() {
        let source = r#"
class Local: pass
def factory():
    class Local: pass
    def with_default(Local=Local()): pass
    def typed(value: Local): pass
    class Holder:
        class Local: pass
        made = Local()
        def method(self):
            return Local()
    def global_reader():
        global Local
        return Local()
    values = [Local() for Local in values]
    class Child(Local): pass
"#;
        let mut graph = parse_python_to_graph("scopes.py", source).unwrap();
        crate::resolve::resolve_graph(&mut graph);
        let symbols = graph.symbols.iter().map(|symbol| (symbol.id.as_str(), symbol)).collect::<std::collections::HashMap<_, _>>();
        let line = |text: &str| source.lines().position(|value| value.contains(text)).unwrap() + 1;
        for (reference_line, expected_parent) in [(line("with_default"), Some("factory")), (line("def typed"), Some("factory")),
            (line("made ="), Some("Holder")), (line("def method") + 1, Some("factory")),
            (line("global Local") + 1, None), (line("class Child"), Some("factory"))] {
            let edge = graph.resolved_edges.iter().find(|edge| edge.line == reference_line
                && edge.reference_target_name.as_deref() == Some("Local")).unwrap();
            let target = symbols[edge.target_symbol_id.as_str()];
            let parent = target.parent_symbol_id.as_deref().map(|id| symbols[id].name.as_str());
            assert_eq!(parent, expected_parent);
        }
        assert!(!graph.resolved_edges.iter().any(|edge| edge.line == line("values = [")));
    }

    #[test]
    fn parses_deeply_nested_python_without_recursive_walker_overflow() {
        let depth = 6000;
        let source = format!("value = {}1{}\n", "(".repeat(depth), ")".repeat(depth));

        let graph = parse_python_to_graph(PathBuf::from("app/deep.py"), &source).unwrap();

        assert!(graph
            .files
            .iter()
            .any(|file| file.path == Path::new("app/deep.py")));
    }
}
