use super::add_file_module_symbol;
use crate::graph::{
    CallForm, FileNode, Language, ReferenceKind, SemanticGraph, SemanticReference, SymbolKind,
    SymbolNode, Visibility,
};
use std::path::PathBuf;
use thiserror::Error;
use tree_sitter::{Node, Parser};

#[derive(Debug, Error)]
pub enum RubyParseError {
    #[error("failed to load tree-sitter Ruby grammar")]
    Language,
    #[error("tree-sitter returned no parse tree")]
    MissingTree,
}

pub fn parse_ruby_to_graph(
    file_path: impl Into<PathBuf>,
    source: &str,
) -> Result<SemanticGraph, RubyParseError> {
    let file_path = file_path.into();
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_ruby::LANGUAGE.into())
        .map_err(|_| RubyParseError::Language)?;
    let tree = parser
        .parse(source, None)
        .ok_or(RubyParseError::MissingTree)?;

    let mut graph = SemanticGraph::default();
    graph.add_file(FileNode {
        path: file_path.clone(),
        language: Language::Ruby,
    });
    add_file_module_symbol(&mut graph, &file_path, Language::Ruby, source);

    let mut context = RubyContext { file_path, source };
    walk_node(tree.root_node(), &mut context, &mut graph, None, None);
    Ok(graph)
}

struct RubyContext<'a> {
    file_path: PathBuf,
    source: &'a str,
}

impl<'a> RubyContext<'a> {
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
            SymbolKind::Method => "method",
            SymbolKind::Module => "module",
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
    context: &mut RubyContext<'_>,
    graph: &mut SemanticGraph,
    container_symbol_id: Option<&str>,
    container_type_name: Option<&str>,
) {
    match node.kind() {
        "module" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = context.text(name_node);
                let symbol = make_symbol(
                    context,
                    SymbolKind::Module,
                    &name,
                    container_symbol_id,
                    container_type_name,
                    0,
                    0,
                    context.line(name_node),
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
        "class" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = context.text(name_node);
                let symbol = make_symbol(
                    context,
                    SymbolKind::Class,
                    &name,
                    container_symbol_id,
                    container_type_name,
                    0,
                    0,
                    context.line(name_node),
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
                record_superclass(node, context, graph, Some(symbol_id.as_str()));
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
        "method" | "singleton_method" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = context.text(name_node);
                let symbol = make_symbol(
                    context,
                    SymbolKind::Method,
                    &name,
                    container_symbol_id,
                    container_type_name,
                    parameter_count(node),
                    parameter_count(node),
                    context.line(name_node),
                    node.end_position().row + 1,
                );
                let symbol_id = symbol.id.clone();
                graph.add_symbol(symbol);
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
        "call" => {
            record_call(
                node,
                context,
                graph,
                container_symbol_id,
                container_type_name,
            );
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
    context: &mut RubyContext<'_>,
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
    context: &RubyContext<'_>,
    kind: SymbolKind,
    name: &str,
    parent_symbol_id: Option<&str>,
    container_type_name: Option<&str>,
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
        return_type_name: None,
        visibility: Visibility::Public,
        parameter_count,
        required_parameter_count,
        start_line,
        end_line,
    }
}

fn record_superclass(
    node: Node<'_>,
    context: &RubyContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
) {
    let Some(superclass) = node.child_by_field_name("superclass") else {
        return;
    };
    let Some(target) = superclass
        .children(&mut superclass.walk())
        .find(|child| child.kind() == "constant")
    else {
        return;
    };
    graph.add_reference(SemanticReference {
        file_path: context.file_path.clone(),
        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
        kind: ReferenceKind::Extends,
        target_name: context.text(target),
        binding_name: None,
        line: context.line(target),
        arity: None,
        receiver_name: None,
        receiver_type_name: None,
        call_form: None,
    });
}

fn record_call(
    node: Node<'_>,
    context: &RubyContext<'_>,
    graph: &mut SemanticGraph,
    enclosing_symbol_id: Option<&str>,
    container_type_name: Option<&str>,
) {
    let method_node = node.child_by_field_name("method");
    let arguments_node = node.child_by_field_name("arguments");
    let receiver_node = node.child_by_field_name("receiver");
    let Some(method_node) = method_node else {
        return;
    };
    let method_name = context.text(method_node);

    if matches!(method_name.as_str(), "require" | "require_relative") {
        let Some(arguments_node) = arguments_node else {
            return;
        };
        if let Some(path_node) = arguments_node
            .children(&mut arguments_node.walk())
            .find(|child| matches!(child.kind(), "string" | "simple_symbol"))
        {
            let mut import_path = context
                .text(path_node)
                .trim_matches(&['"', '\'', ':'][..])
                .to_owned();
            if method_name == "require_relative" && !import_path.starts_with("./") {
                import_path = format!("./{import_path}");
            }
            graph.add_reference(SemanticReference {
                file_path: context.file_path.clone(),
                enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                kind: ReferenceKind::Import,
                target_name: import_path.clone(),
                binding_name: Some(
                    import_path
                        .rsplit('/')
                        .next()
                        .unwrap_or(&import_path)
                        .to_owned(),
                ),
                line: context.line(node),
                arity: None,
                receiver_name: None,
                receiver_type_name: None,
                call_form: None,
            });
        }
        return;
    }

    if matches!(method_name.as_str(), "include" | "extend" | "prepend") {
        if let Some(arguments_node) = arguments_node {
            for argument in arguments_node.children(&mut arguments_node.walk()) {
                if argument.kind() != "constant" {
                    continue;
                }
                graph.add_reference(SemanticReference {
                    file_path: context.file_path.clone(),
                    enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
                    kind: ReferenceKind::Implements,
                    target_name: context.text(argument),
                    binding_name: None,
                    line: context.line(argument),
                    arity: None,
                    receiver_name: None,
                    receiver_type_name: None,
                    call_form: None,
                });
            }
        }
        return;
    }

    let receiver_name = receiver_node.map(|node| context.text(node));
    let receiver_type_name = receiver_node.and_then(|receiver_node| {
        infer_member_receiver_type(
            receiver_node,
            call_node_owner(node),
            context,
            container_type_name,
        )
    });
    graph.add_reference(SemanticReference {
        file_path: context.file_path.clone(),
        enclosing_symbol_id: enclosing_symbol_id.map(str::to_owned),
        kind: ReferenceKind::Call,
        target_name: method_name,
        binding_name: None,
        line: context.line(node),
        arity: Some(argument_count(node)),
        receiver_name,
        receiver_type_name,
        call_form: Some(if receiver_node.is_some() {
            CallForm::Member
        } else {
            CallForm::Free
        }),
    });
}

fn call_node_owner(node: Node<'_>) -> Node<'_> {
    let mut current = node;
    while let Some(parent) = current.parent() {
        if matches!(parent.kind(), "method" | "singleton_method") {
            return parent;
        }
        current = parent;
    }
    node
}

fn infer_receiver_type(
    scope_node: Node<'_>,
    receiver_name: &str,
    context: &RubyContext<'_>,
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
    context: &RubyContext<'_>,
    container_type_name: Option<&str>,
) -> Option<String> {
    match receiver_node.kind() {
        "self" => container_type_name.map(str::to_owned),
        "call" => infer_assignment_type(receiver_node, context),
        _ => infer_receiver_type(scope_node, &context.text(receiver_node), context),
    }
}

fn collect_receiver_type(
    node: Node<'_>,
    receiver_name: &str,
    context: &RubyContext<'_>,
    inferred: &mut Option<String>,
) {
    if inferred.is_some() {
        return;
    }
    if matches!(node.kind(), "assignment" | "command_assignment") {
        let left = node
            .child_by_field_name("left")
            .map(|child| context.text(child));
        if left.as_deref() == Some(receiver_name) {
            *inferred = node
                .child_by_field_name("right")
                .and_then(|right| infer_assignment_type(right, context));
            return;
        }
    }
    for idx in 0..node.child_count() {
        if let Some(child) = node.child(idx as u32) {
            collect_receiver_type(child, receiver_name, context, inferred);
        }
    }
}

fn infer_assignment_type(node: Node<'_>, context: &RubyContext<'_>) -> Option<String> {
    match node.kind() {
        "call" => {
            let method = node
                .child_by_field_name("method")
                .map(|child| context.text(child));
            let receiver = node
                .child_by_field_name("receiver")
                .map(|child| context.text(child));
            if method.as_deref() == Some("new") {
                return receiver.map(|value| leaf_ruby_constant(&value));
            }
            method
        }
        _ => None,
    }
}

fn leaf_ruby_constant(value: &str) -> String {
    value.rsplit("::").next().unwrap_or(value).to_owned()
}

fn parameter_count(node: Node<'_>) -> usize {
    node.child_by_field_name("parameters")
        .map(|parameters| parameters.named_child_count())
        .unwrap_or(0)
}

fn argument_count(node: Node<'_>) -> usize {
    node.child_by_field_name("arguments")
        .map(|arguments| arguments.named_child_count())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::parse_ruby_to_graph;
    use crate::graph::{CallForm, Language, ReferenceKind, SymbolKind};
    use std::path::PathBuf;

    #[test]
    fn parses_ruby_symbols_imports_and_mixins() {
        let graph = parse_ruby_to_graph(
            PathBuf::from("app/service.rb"),
            r#"require_relative "./models/user"
class Service < Base
  include Persisted

  def run(user)
    helper
    self.save(user)
  end
end
"#,
        )
        .unwrap();

        assert!(graph
            .files
            .iter()
            .any(|file| file.language == Language::Ruby));
        assert!(graph
            .symbols
            .iter()
            .any(|symbol| symbol.kind == SymbolKind::Class && symbol.name == "Service"));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Import && reference.target_name == "./models/user"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Extends && reference.target_name == "Base"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Implements && reference.target_name == "Persisted"
        }));
        assert!(graph.references.iter().any(|reference| {
            reference.kind == ReferenceKind::Call
                && reference.target_name == "save"
                && reference.call_form == Some(CallForm::Member)
        }));
    }
}
