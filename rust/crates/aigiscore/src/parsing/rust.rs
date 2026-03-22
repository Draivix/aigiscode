use super::add_file_module_symbol;
use crate::graph::{
    CallForm, FileNode, Language, ReferenceKind, SemanticGraph, SemanticReference, SymbolKind,
    SymbolNode, Visibility,
};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tree_sitter::{Node, Parser};

#[derive(Debug, Error)]
pub enum RustParseError {
    #[error("failed to load tree-sitter Rust grammar")]
    Language,
    #[error("tree-sitter returned no parse tree")]
    MissingTree,
}

pub fn parse_rust_to_graph(
    file_path: impl Into<PathBuf>,
    source: &str,
) -> Result<SemanticGraph, RustParseError> {
    let file_path = file_path.into();
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|_| RustParseError::Language)?;

    let tree = parser
        .parse(source, None)
        .ok_or(RustParseError::MissingTree)?;
    let root = tree.root_node();

    let mut graph = SemanticGraph::default();
    graph.add_file(FileNode {
        path: file_path.clone(),
        language: Language::Rust,
    });
    add_file_module_symbol(&mut graph, &file_path, Language::Rust, source);

    let mut context = RustExtractionContext::new(file_path, source);
    walk_node(root, &mut context, &mut graph, None, None);

    Ok(graph)
}

struct RustExtractionContext<'a> {
    file_path: PathBuf,
    source: &'a str,
}

impl<'a> RustExtractionContext<'a> {
    fn new(file_path: PathBuf, source: &'a str) -> Self {
        Self { file_path, source }
    }

    fn text(&self, node: Node<'_>) -> String {
        node.utf8_text(self.source.as_bytes())
            .map(str::to_owned)
            .unwrap_or_default()
    }

    fn line(&self, node: Node<'_>) -> usize {
        node.start_position().row + 1
    }

    fn symbol_id(&self, kind: SymbolKind, parent_symbol_id: Option<&str>, name: &str) -> String {
        let prefix = symbol_kind_label(kind);
        match parent_symbol_id {
            Some(parent) => format!("{prefix}:{}:{parent}:{name}", self.file_path.display()),
            None => format!("{prefix}:{}:{name}", self.file_path.display()),
        }
    }

    fn parameter_count(&self, node: Node<'_>) -> usize {
        let Some(parameters) = node.child_by_field_name("parameters") else {
            return 0;
        };
        let mut count = 0;
        for idx in 0..parameters.child_count() {
            if let Some(child) = parameters.child(idx as u32) {
                if child.kind() == "parameter" {
                    count += 1;
                }
            }
        }
        count
    }
}

fn walk_node(
    node: Node<'_>,
    context: &mut RustExtractionContext<'_>,
    graph: &mut SemanticGraph,
    container_symbol_id: Option<&str>,
    container_type_name: Option<&str>,
) {
    match node.kind() {
        "use_declaration" => {
            record_import(node, context, graph, container_symbol_id);
        }
        "struct_item" => {
            if let Some(type_node) = node.child_by_field_name("name") {
                let name = context.text(type_node);
                let symbol = make_symbol(
                    context,
                    SymbolKind::Struct,
                    &name,
                    None,
                    None,
                    None,
                    visibility_for(node),
                    0,
                    0,
                    type_node.start_position().row + 1,
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
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
        "trait_item" => {
            if let Some(type_node) = node.child_by_field_name("name") {
                let name = context.text(type_node);
                let symbol = make_symbol(
                    context,
                    SymbolKind::Trait,
                    &name,
                    None,
                    None,
                    None,
                    visibility_for(node),
                    0,
                    0,
                    type_node.start_position().row + 1,
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
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
        "enum_item" => {
            if let Some(type_node) = node.child_by_field_name("name") {
                let name = context.text(type_node);
                let symbol = make_symbol(
                    context,
                    SymbolKind::Enum,
                    &name,
                    None,
                    None,
                    None,
                    visibility_for(node),
                    0,
                    0,
                    type_node.start_position().row + 1,
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
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
        "function_item" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = context.text(name_node);
                let kind = if container_type_name.is_some() {
                    SymbolKind::Method
                } else {
                    SymbolKind::Function
                };
                let symbol = make_symbol(
                    context,
                    kind,
                    &name,
                    container_symbol_id,
                    container_type_name,
                    function_return_type(node, context),
                    visibility_for(node),
                    context.parameter_count(node),
                    context.parameter_count(node),
                    name_node.start_position().row + 1,
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
                record_parameter_type_references(node, context, graph, Some(symbol_id.as_str()));
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
        "function_signature_item" => {
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
                    context.parameter_count(node),
                    context.parameter_count(node),
                    name_node.start_position().row + 1,
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
                record_parameter_type_references(node, context, graph, Some(symbol_id.as_str()));
            }
            return;
        }
        "let_declaration" => {
            record_let_type_reference(node, context, graph, container_symbol_id);
        }
        "impl_item" => {
            record_impl_relationships(node, context, graph, container_symbol_id);
            let impl_type_name = node
                .child_by_field_name("type")
                .map(|name_node| context.text(name_node))
                .unwrap_or_default();
            walk_children(
                node,
                context,
                graph,
                container_symbol_id,
                (!impl_type_name.is_empty()).then_some(impl_type_name.as_str()),
            );
            return;
        }
        "call_expression" => {
            record_call(node, context, graph, container_symbol_id);
        }
        "struct_expression" => {
            record_struct_expression(node, context, graph, container_symbol_id);
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
    context: &mut RustExtractionContext<'_>,
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
    context: &RustExtractionContext<'_>,
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

fn function_return_type(node: Node<'_>, context: &RustExtractionContext<'_>) -> Option<String> {
    node.child_by_field_name("return_type")
        .map(|type_node| context.text(type_node))
}

fn record_import(
    node: Node<'_>,
    context: &RustExtractionContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(argument) = node.child_by_field_name("argument") else {
        return;
    };
    let bindings = expand_use_tree(argument, context, None);
    for (target_name, binding_name) in bindings {
        graph.add_reference(SemanticReference {
            file_path: context.file_path.clone(),
            enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
            kind: ReferenceKind::Import,
            target_name,
            binding_name: Some(binding_name),
            line: context.line(node),
            arity: None,
            receiver_name: None,
            receiver_type_name: None,
            call_form: None,
        });
    }
}

fn record_impl_relationships(
    node: Node<'_>,
    context: &RustExtractionContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let trait_name = node.child_by_field_name("trait").map(|n| context.text(n));

    if let Some(target_trait) = trait_name {
        graph.add_reference(SemanticReference {
            file_path: context.file_path.clone(),
            enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
            kind: ReferenceKind::Implements,
            target_name: target_trait,
            binding_name: None,
            line: context.line(node),
            arity: None,
            receiver_name: None,
            receiver_type_name: None,
            call_form: None,
        });
    }
}

fn record_parameter_type_references(
    node: Node<'_>,
    context: &RustExtractionContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return;
    };
    for idx in 0..parameters.child_count() {
        if let Some(parameter) = parameters.child(idx as u32) {
            if parameter.kind() != "parameter" {
                continue;
            }
            if let Some(type_node) = parameter.child_by_field_name("type") {
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
    }
}

fn record_let_type_reference(
    node: Node<'_>,
    context: &RustExtractionContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    if let Some(type_node) = node.child_by_field_name("type") {
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
    context: &RustExtractionContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(function_node) = node.child_by_field_name("function") else {
        return;
    };

    let (target_name, call_form) = match function_node.kind() {
        "identifier" => (context.text(function_node), CallForm::Free),
        "field_expression" => {
            let receiver_name = function_node
                .child_by_field_name("value")
                .map(|n| context.text(n))
                .unwrap_or_default();
            let name = function_node
                .child_by_field_name("field")
                .map(|n| context.text(n))
                .unwrap_or_else(|| context.text(function_node));
            let receiver_type_name =
                infer_receiver_type(call_node_owner(node), &receiver_name, context);
            graph.add_reference(SemanticReference {
                file_path: context.file_path.clone(),
                enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                kind: ReferenceKind::Call,
                target_name: name,
                binding_name: None,
                line: context.line(node),
                arity: Some(argument_count(node)),
                receiver_name: Some(receiver_name),
                receiver_type_name,
                call_form: Some(CallForm::Member),
            });
            return;
        }
        "scoped_identifier" => {
            let receiver_name = function_node
                .child_by_field_name("path")
                .map(|n| context.text(n))
                .or_else(|| function_node.child(0).map(|n| context.text(n)));
            let name = function_node
                .child_by_field_name("name")
                .map(|n| context.text(n))
                .unwrap_or_else(|| leaf_name(&context.text(function_node)));
            graph.add_reference(SemanticReference {
                file_path: context.file_path.clone(),
                enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                kind: ReferenceKind::Call,
                target_name: name,
                binding_name: None,
                line: context.line(node),
                arity: Some(argument_count(node)),
                receiver_name,
                receiver_type_name: None,
                call_form: Some(CallForm::Associated),
            });
            return;
        }
        _ => return,
    };

    graph.add_reference(SemanticReference {
        file_path: context.file_path.clone(),
        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
        kind: ReferenceKind::Call,
        target_name,
        binding_name: None,
        line: context.line(node),
        arity: Some(argument_count(node)),
        receiver_name: None,
        receiver_type_name: None,
        call_form: Some(call_form),
    });
}

fn record_struct_expression(
    node: Node<'_>,
    context: &RustExtractionContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let type_node = (0..node.child_count()).find_map(|idx| {
        let child = node.child(idx as u32)?;
        matches!(child.kind(), "type_identifier" | "scoped_type_identifier").then_some(child)
    });
    let Some(type_node) = type_node else {
        return;
    };

    let target_name = context.text(type_node);
    if target_name == "Self" {
        return;
    }

    graph.add_reference(SemanticReference {
        file_path: context.file_path.clone(),
        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
        kind: ReferenceKind::Call,
        target_name,
        binding_name: None,
        line: context.line(node),
        arity: Some(0),
        receiver_name: None,
        receiver_type_name: None,
        call_form: Some(CallForm::Associated),
    });
}

fn expand_use_tree(
    node: Node<'_>,
    context: &RustExtractionContext<'_>,
    prefix: Option<String>,
) -> Vec<(String, String)> {
    match node.kind() {
        "identifier" => {
            let name = context.text(node);
            let target = append_segment(prefix, &name);
            vec![(target, name)]
        }
        "scoped_identifier" => {
            let target = context.text(node);
            let binding = leaf_name(&target);
            vec![(append_path(prefix, &target), binding)]
        }
        "use_as_clause" => {
            let original = node
                .child(0)
                .map(|child| context.text(child))
                .unwrap_or_default();
            let alias = node
                .child(node.child_count().saturating_sub(1) as u32)
                .map(|child| context.text(child))
                .unwrap_or_else(|| leaf_name(&original));
            vec![(append_path(prefix, &original), alias)]
        }
        "scoped_use_list" => {
            let base = node
                .child(0)
                .map(|child| context.text(child))
                .unwrap_or_default();
            let merged_prefix = Some(append_path(prefix, &base));
            let mut results = Vec::new();
            for idx in 0..node.child_count() {
                if let Some(child) = node.child(idx as u32) {
                    if child.kind() == "use_list" {
                        results.extend(expand_use_tree(child, context, merged_prefix.clone()));
                    }
                }
            }
            results
        }
        "use_list" => {
            let mut results = Vec::new();
            for idx in 0..node.child_count() {
                if let Some(child) = node.child(idx as u32) {
                    match child.kind() {
                        "identifier" | "scoped_identifier" | "use_as_clause"
                        | "scoped_use_list" => {
                            results.extend(expand_use_tree(child, context, prefix.clone()));
                        }
                        _ => {}
                    }
                }
            }
            results
        }
        _ => Vec::new(),
    }
}

fn append_segment(prefix: Option<String>, segment: &str) -> String {
    match prefix {
        Some(prefix) if !prefix.is_empty() => format!("{prefix}::{segment}"),
        _ => segment.to_owned(),
    }
}

fn append_path(prefix: Option<String>, path: &str) -> String {
    match prefix {
        Some(prefix) if !prefix.is_empty() => format!("{prefix}::{path}"),
        _ => path.to_owned(),
    }
}

fn argument_count(node: Node<'_>) -> usize {
    let Some(arguments) = node.child_by_field_name("arguments") else {
        return 0;
    };
    let mut count = 0;
    for idx in 0..arguments.child_count() {
        if let Some(child) = arguments.child(idx as u32) {
            let kind = child.kind();
            if !matches!(kind, "(" | ")" | ",") {
                count += 1;
            }
        }
    }
    count
}

fn call_node_owner(node: Node<'_>) -> Node<'_> {
    let mut current = node;
    while let Some(parent) = current.parent() {
        if matches!(
            parent.kind(),
            "function_item" | "function_signature_item" | "closure_expression"
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
    context: &RustExtractionContext<'_>,
) -> Option<String> {
    if receiver_name.is_empty() {
        return None;
    }
    let mut inferred = None;
    collect_receiver_type(scope_node, receiver_name, context, &mut inferred);
    inferred
}

fn collect_receiver_type(
    node: Node<'_>,
    receiver_name: &str,
    context: &RustExtractionContext<'_>,
    inferred: &mut Option<String>,
) {
    if inferred.is_some() {
        return;
    }
    if node.kind() == "let_declaration" {
        let binding_name = node
            .child_by_field_name("pattern")
            .and_then(|pattern| find_first_identifier(pattern, context));
        if binding_name.as_deref() == Some(receiver_name) {
            *inferred = node
                .child_by_field_name("type")
                .map(|n| context.text(n))
                .or_else(|| {
                    node.child_by_field_name("value").and_then(|value| {
                        if value.kind() == "struct_expression" {
                            (0..value.child_count()).find_map(|idx| {
                                let child = value.child(idx as u32)?;
                                matches!(child.kind(), "type_identifier" | "scoped_type_identifier")
                                    .then_some(context.text(child))
                            })
                        } else {
                            None
                        }
                    })
                });
            return;
        }
    }
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            collect_receiver_type(child, receiver_name, context, inferred);
        }
    }
}

fn find_first_identifier(node: Node<'_>, context: &RustExtractionContext<'_>) -> Option<String> {
    if node.kind() == "identifier" {
        return Some(context.text(node));
    }
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            if let Some(identifier) = find_first_identifier(child, context) {
                return Some(identifier);
            }
        }
    }
    None
}

fn leaf_name(name: &str) -> String {
    name.rsplit("::").next().unwrap_or(name).to_owned()
}

fn visibility_for(node: Node<'_>) -> Visibility {
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            if child.kind() == "visibility_modifier" {
                return Visibility::Public;
            }
        }
    }
    Visibility::Private
}

fn symbol_kind_label(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Class => "class",
        SymbolKind::Function => "function",
        SymbolKind::Interface => "interface",
        SymbolKind::Method => "method",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Trait => "trait",
        SymbolKind::Module => "module",
    }
}

pub fn parse_rust_file(path: impl AsRef<Path>) -> Result<SemanticGraph, RustParseError> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path).map_err(|_| RustParseError::MissingTree)?;
    parse_rust_to_graph(path.to_path_buf(), &source)
}

#[cfg(test)]
mod tests {
    use super::parse_rust_to_graph;
    use crate::graph::{CallForm, ReferenceKind, SymbolKind, Visibility};
    use std::path::PathBuf;

    #[test]
    fn extracts_rust_symbols_and_references() {
        let graph = parse_rust_to_graph(
            PathBuf::from("src/lib.rs"),
            r#"
use crate::models::User;

pub struct Account {}

trait Drawable {
    fn draw(&self);
}

impl Account {
    pub fn load(user: User) -> Self {
        helper();
        Self {}
    }
}

impl Drawable for Account {
    fn draw(&self) {
        helper();
        let _value = Account {};
        Account::build();
    }
}

fn helper() {}
"#,
        )
        .unwrap();

        assert!(graph.symbols.iter().any(|symbol| {
            symbol.kind == SymbolKind::Struct
                && symbol.name == "Account"
                && symbol.visibility == Visibility::Public
        }));
        assert!(graph.symbols.iter().any(|symbol| {
            symbol.kind == SymbolKind::Method
                && symbol.qualified_name == "Account::load"
                && symbol.visibility == Visibility::Public
        }));
        assert!(graph
            .symbols
            .iter()
            .any(|symbol| { symbol.kind == SymbolKind::Trait && symbol.name == "Drawable" }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Import
                && reference.target_name == "crate::models::User"
                && reference.binding_name.as_deref() == Some("User")
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Implements && reference.target_name == "Drawable"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "helper"
                && reference.call_form == Some(CallForm::Free)
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "build"
                && reference.call_form == Some(CallForm::Associated)
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "Account"
                && reference.call_form == Some(CallForm::Associated)
        }));
    }

    #[test]
    fn expands_grouped_and_aliased_use_trees() {
        let graph = parse_rust_to_graph(
            PathBuf::from("src/lib.rs"),
            r#"
use crate::{models::{User as U, Repo}, services::Auth};
"#,
        )
        .unwrap();

        let imports = graph
            .references
            .iter()
            .filter(|reference| reference.kind == ReferenceKind::Import)
            .collect::<Vec<_>>();

        assert_eq!(imports.len(), 3);
        assert!(imports.iter().any(|reference| {
            reference.target_name == "crate::models::User"
                && reference.binding_name.as_deref() == Some("U")
        }));
        assert!(imports.iter().any(|reference| {
            reference.target_name == "crate::models::Repo"
                && reference.binding_name.as_deref() == Some("Repo")
        }));
        assert!(imports.iter().any(|reference| {
            reference.target_name == "crate::services::Auth"
                && reference.binding_name.as_deref() == Some("Auth")
        }));
    }
}
