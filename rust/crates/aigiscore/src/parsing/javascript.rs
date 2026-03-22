use super::add_file_module_symbol;
use crate::graph::{
    CallForm, FileNode, Language, ReferenceKind, SemanticGraph, SemanticReference, SymbolKind,
    SymbolNode, Visibility,
};
use std::path::PathBuf;
use thiserror::Error;
use tree_sitter::{Node, Parser};

#[derive(Debug, Error)]
pub enum JavaScriptParseError {
    #[error("failed to load tree-sitter JavaScript/TypeScript grammar")]
    Language,
    #[error("tree-sitter returned no parse tree")]
    MissingTree,
}

pub fn parse_javascript_to_graph(
    file_path: impl Into<PathBuf>,
    source: &str,
    is_typescript: bool,
) -> Result<SemanticGraph, JavaScriptParseError> {
    let file_path = file_path.into();
    let mut parser = Parser::new();
    let language = if is_typescript {
        Language::TypeScript
    } else {
        Language::JavaScript
    };
    let tree_sitter_language = if is_typescript {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT
    } else {
        tree_sitter_javascript::LANGUAGE
    };
    parser
        .set_language(&tree_sitter_language.into())
        .map_err(|_| JavaScriptParseError::Language)?;

    let tree = parser
        .parse(source, None)
        .ok_or(JavaScriptParseError::MissingTree)?;
    let root = tree.root_node();

    let mut graph = SemanticGraph::default();
    graph.add_file(FileNode {
        path: file_path.clone(),
        language,
    });
    add_file_module_symbol(&mut graph, &file_path, language, source);

    let mut context = JavaScriptContext { file_path, source };
    walk_node(root, &mut context, &mut graph, None, None);
    Ok(graph)
}

struct JavaScriptContext<'a> {
    file_path: PathBuf,
    source: &'a str,
}

impl<'a> JavaScriptContext<'a> {
    fn text(&self, node: Node<'_>) -> String {
        node.utf8_text(self.source.as_bytes())
            .map(str::to_owned)
            .unwrap_or_default()
    }

    fn line(&self, node: Node<'_>) -> usize {
        node.start_position().row + 1
    }

    fn string_value(&self, node: Node<'_>) -> String {
        self.text(node)
            .trim_matches(&['"', '\'', '`'][..])
            .to_owned()
    }

    fn symbol_id(&self, kind: SymbolKind, parent: Option<&str>, name: &str) -> String {
        let prefix = match kind {
            SymbolKind::Class => "class",
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Interface => "interface",
            _ => "symbol",
        };
        match parent {
            Some(parent) => format!("{prefix}:{}:{parent}:{name}", self.file_path.display()),
            None => format!("{prefix}:{}:{name}", self.file_path.display()),
        }
    }
}

fn walk_node(
    node: Node<'_>,
    context: &mut JavaScriptContext<'_>,
    graph: &mut SemanticGraph,
    container_symbol_id: Option<&str>,
    container_type_name: Option<&str>,
) {
    match node.kind() {
        "import_statement" => {
            record_import_statement(node, context, graph, container_symbol_id);
        }
        "class_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = context.text(name_node);
                let symbol = make_symbol(
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
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
                record_js_heritage(node, context, graph, Some(symbol_id.as_str()));
                walk_children(
                    node,
                    context,
                    graph,
                    Some(symbol_id.as_str()),
                    Some(name.as_str()),
                );
            }
            return;
        }
        "function_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = context.text(name_node);
                let symbol = make_symbol(
                    context,
                    SymbolKind::Function,
                    &name,
                    container_symbol_id,
                    container_type_name,
                    function_return_type(node, context),
                    Visibility::Public,
                    parameter_count(node),
                    parameter_count(node),
                    context.line(name_node),
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
                record_js_parameter_types(node, context, graph, Some(symbol_id.as_str()));
                walk_children(
                    node,
                    context,
                    graph,
                    Some(symbol_id.as_str()),
                    container_type_name,
                );
            }
            return;
        }
        "method_definition" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = context.text(name_node);
                let symbol = make_symbol(
                    context,
                    SymbolKind::Method,
                    &name,
                    container_symbol_id,
                    container_type_name,
                    function_return_type(node, context),
                    Visibility::Public,
                    parameter_count(node),
                    parameter_count(node),
                    context.line(name_node),
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
                record_js_parameter_types(node, context, graph, Some(symbol_id.as_str()));
                walk_children(
                    node,
                    context,
                    graph,
                    Some(symbol_id.as_str()),
                    container_type_name,
                );
            }
            return;
        }
        "call_expression" => {
            record_call(node, context, graph, container_symbol_id);
        }
        "new_expression" => {
            record_constructor_call(node, context, graph, container_symbol_id);
        }
        _ => {}
    }

    walk_children(
        node,
        context,
        graph,
        container_symbol_id,
        container_type_name,
    );
}

fn walk_children(
    node: Node<'_>,
    context: &mut JavaScriptContext<'_>,
    graph: &mut SemanticGraph,
    container_symbol_id: Option<&str>,
    container_type_name: Option<&str>,
) {
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            walk_node(
                child,
                context,
                graph,
                container_symbol_id,
                container_type_name,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn make_symbol(
    context: &JavaScriptContext<'_>,
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

fn record_import_statement(
    node: Node<'_>,
    context: &JavaScriptContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(source_node) = node.child_by_field_name("source") else {
        return;
    };
    let import_source = context.string_value(source_node);
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            if child.kind() == "import_clause" {
                record_import_clause(child, &import_source, context, graph, enclosing_symbol_id);
            }
        }
    }
    if !node
        .children(&mut node.walk())
        .any(|child| child.kind() == "import_clause")
    {
        graph.add_reference(SemanticReference {
            file_path: context.file_path.clone(),
            enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
            kind: ReferenceKind::Import,
            target_name: import_source,
            binding_name: None,
            line: context.line(node),
            arity: None,
            receiver_name: None,
            receiver_type_name: None,
            call_form: None,
        });
    }
}

fn record_import_clause(
    clause: Node<'_>,
    import_source: &str,
    context: &JavaScriptContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    for idx in 0..clause.child_count() {
        if let Some(child) = clause.child(idx as u32) {
            match child.kind() {
                "identifier" => {
                    graph.add_reference(SemanticReference {
                        file_path: context.file_path.clone(),
                        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                        kind: ReferenceKind::Import,
                        target_name: format!("{import_source}::default"),
                        binding_name: Some(context.text(child)),
                        line: context.line(child),
                        arity: None,
                        receiver_name: None,
                        receiver_type_name: None,
                        call_form: None,
                    });
                }
                "named_imports" => {
                    for item_index in 0..child.child_count() {
                        if let Some(specifier) = child.child(item_index as u32) {
                            if specifier.kind() != "import_specifier" {
                                continue;
                            }
                            let imported_name = specifier
                                .child_by_field_name("name")
                                .map(|node| context.text(node))
                                .unwrap_or_default();
                            let binding_name = specifier
                                .child_by_field_name("alias")
                                .map(|node| context.text(node))
                                .unwrap_or_else(|| imported_name.clone());
                            graph.add_reference(SemanticReference {
                                file_path: context.file_path.clone(),
                                enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                                kind: ReferenceKind::Import,
                                target_name: format!("{import_source}::{imported_name}"),
                                binding_name: Some(binding_name),
                                line: context.line(specifier),
                                arity: None,
                                receiver_name: None,
                                receiver_type_name: None,
                                call_form: None,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn record_js_heritage(
    node: Node<'_>,
    context: &JavaScriptContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            if matches!(child.kind(), "class_heritage" | "extends_clause") {
                let parent = find_first_heritage_target(child);
                if let Some(parent) = parent {
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
        }
    }
}

fn find_first_heritage_target(node: Node<'_>) -> Option<Node<'_>> {
    if matches!(
        node.kind(),
        "identifier" | "type_identifier" | "nested_identifier" | "member_expression"
    ) {
        return Some(node);
    }
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            if let Some(found) = find_first_heritage_target(child) {
                return Some(found);
            }
        }
    }
    None
}

fn record_js_parameter_types(
    node: Node<'_>,
    context: &JavaScriptContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return;
    };
    for idx in 0..parameters.child_count() {
        if let Some(parameter) = parameters.child(idx as u32) {
            let type_node = parameter.child_by_field_name("type");
            if let Some(type_annotation) = type_node {
                let type_name = type_text(type_annotation, context);
                if let Some(type_name) = type_name {
                    graph.add_reference(SemanticReference {
                        file_path: context.file_path.clone(),
                        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                        kind: ReferenceKind::Type,
                        target_name: type_name,
                        binding_name: None,
                        line: context.line(type_annotation),
                        arity: None,
                        receiver_name: None,
                        receiver_type_name: None,
                        call_form: None,
                    });
                }
            }
        }
    }
}

fn record_call(
    node: Node<'_>,
    context: &JavaScriptContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
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
            });
        }
        "member_expression" => {
            let receiver_node = function_node.child_by_field_name("object");
            let receiver_name = receiver_node.map(|n| context.text(n));
            let receiver_type_name = receiver_node.and_then(|receiver_node| {
                infer_member_receiver_type(
                    receiver_node,
                    call_node_owner(node),
                    context,
                    graph,
                    enclosing_symbol_id,
                )
            });
            let target_name = function_node
                .child_by_field_name("property")
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
            });
        }
        _ => {}
    }
}

fn record_constructor_call(
    node: Node<'_>,
    context: &JavaScriptContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let target = node
        .children(&mut node.walk())
        .find(|child| matches!(child.kind(), "identifier" | "type_identifier"));
    if let Some(target) = target {
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
}

fn parameter_count(node: Node<'_>) -> usize {
    node.child_by_field_name("parameters")
        .map(|parameters| {
            parameters
                .children(&mut parameters.walk())
                .filter(|child| {
                    matches!(
                        child.kind(),
                        "required_parameter"
                            | "optional_parameter"
                            | "identifier"
                            | "assignment_pattern"
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
        if matches!(
            parent.kind(),
            "function_declaration" | "method_definition" | "arrow_function" | "function_expression"
        ) {
            return parent;
        }
        current = parent;
    }
    node
}

fn infer_receiver_type(
    scope_node: Node<'_>,
    receiver_name: &str,
    context: &JavaScriptContext<'_>,
) -> Option<String> {
    if receiver_name.is_empty() {
        return None;
    }
    let mut inferred = None;
    collect_receiver_type(scope_node, receiver_name, context, &mut inferred);
    inferred
}

fn infer_member_receiver_type(
    receiver_node: Node<'_>,
    scope_node: Node<'_>,
    context: &JavaScriptContext<'_>,
    graph: &SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) -> Option<String> {
    match receiver_node.kind() {
        "this" => enclosing_symbol_id
            .and_then(|id| graph.symbols.iter().find(|symbol| symbol.id == id))
            .and_then(|symbol| symbol.owner_type_name.clone()),
        "new_expression" => receiver_node
            .child_by_field_name("constructor")
            .and_then(|constructor| leaf_identifier(constructor, context)),
        "call_expression" | "await_expression" => {
            infer_call_result_type(receiver_node, scope_node, context)
        }
        _ => infer_receiver_type(scope_node, &context.text(receiver_node), context),
    }
}

fn collect_receiver_type(
    node: Node<'_>,
    receiver_name: &str,
    context: &JavaScriptContext<'_>,
    inferred: &mut Option<String>,
) {
    if inferred.is_some() {
        return;
    }
    match node.kind() {
        "required_parameter" | "optional_parameter" => {
            let name = node
                .child_by_field_name("pattern")
                .or_else(|| node.child_by_field_name("name"))
                .map(|child| context.text(child));
            if name.as_deref() == Some(receiver_name) {
                *inferred = node
                    .child_by_field_name("type")
                    .and_then(|type_node| type_text(type_node, context));
                return;
            }
        }
        "variable_declarator" => {
            let name = node
                .child_by_field_name("name")
                .map(|child| context.text(child));
            if name.as_deref() == Some(receiver_name) {
                *inferred = node
                    .child_by_field_name("type")
                    .and_then(|type_node| type_text(type_node, context))
                    .or_else(|| {
                        node.child_by_field_name("value").and_then(|value| {
                            if value.kind() == "new_expression" {
                                value
                                    .child_by_field_name("constructor")
                                    .and_then(|constructor| leaf_identifier(constructor, context))
                            } else if value.kind() == "call_expression" {
                                infer_call_result_type(value, call_node_owner(node), context)
                            } else if value.kind() == "await_expression" {
                                value.named_child(0).and_then(|child| {
                                    infer_call_result_type(child, call_node_owner(node), context)
                                })
                            } else {
                                None
                            }
                        })
                    });
                return;
            }
        }
        _ => {}
    }
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            collect_receiver_type(child, receiver_name, context, inferred);
        }
    }
}

fn function_return_type(node: Node<'_>, context: &JavaScriptContext<'_>) -> Option<String> {
    node.child_by_field_name("return_type")
        .and_then(|type_node| type_text(type_node, context))
}

fn infer_call_result_type(
    node: Node<'_>,
    scope_node: Node<'_>,
    context: &JavaScriptContext<'_>,
) -> Option<String> {
    if node.kind() == "await_expression" {
        return node
            .named_child(0)
            .and_then(|child| infer_call_result_type(child, scope_node, context));
    }
    if node.kind() != "call_expression" {
        return None;
    }
    let function_node = node.child_by_field_name("function")?;
    match function_node.kind() {
        "identifier" | "type_identifier" => Some(context.text(function_node)),
        "member_expression" => {
            let receiver_name = function_node
                .child_by_field_name("object")
                .map(|receiver| context.text(receiver));
            let method_name = function_node
                .child_by_field_name("property")
                .map(|property| context.text(property))
                .or_else(|| leaf_identifier(function_node, context));
            match (receiver_name, method_name) {
                (Some(receiver_name), Some(method_name)) => {
                    let receiver_type_name =
                        infer_receiver_type(scope_node, &receiver_name, context);
                    receiver_type_name
                        .or_else(|| static_receiver_type_name(function_node, context))
                        .map(|receiver_type_name| format!("{receiver_type_name}::{method_name}"))
                        .or(Some(method_name))
                }
                (_, method_name) => method_name,
            }
        }
        _ => leaf_identifier(function_node, context),
    }
}

fn static_receiver_type_name(
    member_expression: Node<'_>,
    context: &JavaScriptContext<'_>,
) -> Option<String> {
    let receiver_node = member_expression.child_by_field_name("object")?;
    let candidate = leaf_identifier(receiver_node, context)?;
    candidate
        .chars()
        .next()
        .filter(|character| character.is_uppercase())?;
    Some(candidate)
}

fn type_text(node: Node<'_>, context: &JavaScriptContext<'_>) -> Option<String> {
    let text = context.text(node).trim_start_matches(':').trim().to_owned();
    (!text.is_empty()).then_some(text)
}

fn leaf_identifier(node: Node<'_>, context: &JavaScriptContext<'_>) -> Option<String> {
    if matches!(
        node.kind(),
        "identifier" | "type_identifier" | "predefined_type"
    ) {
        return Some(context.text(node).trim_start_matches(':').trim().to_owned());
    }
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            if let Some(found) = leaf_identifier(child, context) {
                return Some(found);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_javascript_to_graph;
    use crate::graph::{CallForm, Language, ReferenceKind, SymbolKind};
    use std::path::PathBuf;

    #[test]
    fn parses_typescript_symbols_and_imports() {
        let graph = parse_javascript_to_graph(
            PathBuf::from("src/app.ts"),
            r#"import DefaultThing, { User as U, Repo } from "./models";
class Service extends Base { run(user: User) { helper(); user.save(); this.save(); new Repo(); } }
function helper() {}"#,
            true,
        )
        .unwrap();

        assert!(graph
            .files
            .iter()
            .any(|file| file.language == Language::TypeScript));
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
                && reference.target_name == "./models::User"
                && reference.binding_name.as_deref() == Some("U")
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Extends && reference.target_name == "Base"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Type && reference.target_name == "User"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "save"
                && reference.receiver_name.as_deref() == Some("user")
                && reference.receiver_type_name.as_deref() == Some("User")
                && reference.call_form == Some(CallForm::Member)
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "save"
                && reference.call_form == Some(CallForm::Member)
        }));
    }
}
