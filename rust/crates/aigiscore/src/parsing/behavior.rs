//! Bounded body facts captured from the parser's existing tree. They describe
//! source structure and never claim runtime equivalence or effect execution.

use crate::graph::{SemanticGraph, SymbolKind, SymbolNode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;
use tree_sitter::Node;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BehaviorParameter {
    pub name: String,
    pub type_hint: Option<String>,
    pub has_default: bool,
    #[serde(default)]
    pub is_receiver: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BehaviorExpression {
    pub line: usize,
    pub end_line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    /// Variable names and string contents are normalized. The fingerprint
    /// distinguishes exact source expressions without persisting their text.
    pub shape: String,
    pub fingerprint: String,
    pub shape_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BehaviorCall {
    pub target: String,
    pub receiver_shape: Option<String>,
    pub expression: BehaviorExpression,
    pub argument_shapes: Vec<String>,
    pub conditional: bool,
    pub arguments_truncated: bool,
    #[serde(default)]
    pub guards: Vec<BehaviorGuard>,
    #[serde(default)]
    pub guards_truncated: bool,
    #[serde(default)]
    pub inside_loop: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BranchArm { Consequent, Alternative, Condition, Unknown }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BehaviorGuard {
    /// Index into the enclosing FunctionBehavior.branches vector.
    pub branch: usize,
    pub arm: BranchArm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FunctionBehavior {
    pub symbol_id: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub end_line: usize,
    pub parameters: Vec<BehaviorParameter>,
    pub read_selectors: Vec<String>,
    pub writes: Vec<BehaviorExpression>,
    pub branches: Vec<BehaviorExpression>,
    pub returns: Vec<BehaviorExpression>,
    pub calls: Vec<BehaviorCall>,
    pub throws: usize,
    pub catches: usize,
    pub loops: usize,
    pub nested_callables: usize,
    pub statement_count: usize,
    pub delegates_directly: bool,
    pub token_count: usize,
    pub structural_fingerprints: Vec<String>,
    pub complete: bool,
    pub test_guarded: bool,
    #[serde(default)]
    pub has_decorators_or_attributes: Option<bool>,
    #[serde(default)]
    pub has_calling_convention_modifier: Option<bool>,
    pub truncated: bool,
}

pub(super) fn capture(graph: &mut SemanticGraph, root: Node<'_>, source: &str) {
    let mut symbols = HashMap::<(&str, usize), Vec<&SymbolNode>>::new();
    for symbol in &graph.symbols {
        if matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method) {
            symbols.entry((&symbol.name, symbol.start_line)).or_default().push(symbol);
        }
    }
    let mut captured = HashSet::new();
    let mut pending = vec![root];
    let mut behaviors = Vec::new();
    while let Some(node) = pending.pop() {
        if callable(node.kind()) {
            let declaration = node.parent().filter(|parent| matches!(parent.kind(), "variable_declarator" | "pair" | "assignment_expression"));
            let name = node.child_by_field_name("name").or_else(|| declaration.and_then(|parent| {
                parent.child_by_field_name("name").or_else(|| parent.child_by_field_name("key")).or_else(|| parent.child_by_field_name("left"))
            }));
            if let (Some(name), Some(body)) = (name, node.child_by_field_name("body")) {
                let name_line = name.start_position().row + 1;
                let name = text(name, source).trim_matches(['\'', '"']);
                let candidates = symbols.get(&(name, node.start_position().row + 1))
                    .or_else(|| symbols.get(&(name, name_line)))
                    .or_else(|| declaration.and_then(|parent| symbols.get(&(name, parent.start_position().row + 1))));
                if let Some(candidates) = candidates {
                    if candidates.len() == 1 && captured.insert(candidates[0].id.clone()) {
                        behaviors.push(capture_body(candidates[0], node, body, source));
                    }
                }
            }
        }
        pending.extend(node.named_children(&mut node.walk()));
    }
    behaviors.sort_by(|left, right| left.symbol_id.cmp(&right.symbol_id));
    graph.function_behaviors = behaviors;
}

fn capture_body(symbol: &SymbolNode, declaration: Node<'_>, body: Node<'_>, source: &str) -> FunctionBehavior {
    let mut decorated = has_decorators_or_attributes(declaration);
    let calling_convention = calling_convention_modifier(declaration, body);
    let mut parameter_bindings_complete = true;
    let mut parameters_truncated = false;
    let parameters = declaration.child_by_field_name("parameters").map(|parameters| {
        parameters_truncated = parameters.named_child_count() > 64;
        parameters.named_children(&mut parameters.walk()).filter(|node| !comment(node.kind())).take(64).enumerate().map(|(index, node)| {
            decorated |= node.named_children(&mut node.walk()).any(|child| matches!(child.kind(), "decorator" | "attribute_list" | "attribute_item"));
            let name = node.child_by_field_name("name").or_else(|| node.child_by_field_name("pattern"))
                .or_else(|| (node.kind() == "self_parameter").then(|| node.named_children(&mut node.walk()).find(|child| child.kind() == "self")).flatten())
                .or_else(|| node.named_child(0)).unwrap_or(node);
            parameter_bindings_complete &= matches!(name.kind(), "identifier" | "name" | "variable_name" | "self" | "self_parameter");
            parameters_truncated |= text(name, source).len() > 128
                || node.child_by_field_name("type").is_some_and(|node| text(node, source).len() > 128);
            BehaviorParameter {
                name: bounded(text(name, source), 128),
                type_hint: node.child_by_field_name("type").map(|node| bounded(text(node, source), 128)),
                has_default: node.child_by_field_name("default_value").or_else(|| node.child_by_field_name("value")).is_some(),
                is_receiver: node.kind() == "self_parameter" || (index == 0 && symbol.kind == SymbolKind::Method
                    && declaration.kind() == "function_definition" && !decorated),
            }
        }).collect::<Vec<_>>()
    }).unwrap_or_default();
    let parameter_names = parameters.iter().enumerate().map(|(index, parameter)| (parameter.name.trim_start_matches('$'), index))
        .collect::<HashMap<_, _>>();
    let mut fact = FunctionBehavior {
        symbol_id: symbol.id.clone(), file_path: symbol.file_path.clone(),
        line: symbol.start_line, end_line: symbol.end_line,
        parameters: parameters.clone(), read_selectors: Vec::new(), writes: Vec::new(), branches: Vec::new(),
        returns: Vec::new(), calls: Vec::new(), throws: 0, catches: 0, loops: 0, nested_callables: 0,
        statement_count: body.named_child_count(), delegates_directly: directly_delegates(body),
        token_count: 0, structural_fingerprints: Vec::new(),
        complete: !declaration.has_error() && parameter_bindings_complete,
        test_guarded: test_guarded(declaration, source), truncated: parameters_truncated,
        has_decorators_or_attributes: Some(decorated),
        has_calling_convention_modifier: calling_convention,
    };
    let mut selectors = BTreeSet::new();
    let mut tokens = Vec::new();
    let mut branch_indices = HashMap::new();
    let mut pending = vec![(body, false)];
    let mut visited = 0usize;
    while let Some((node, conditional)) = pending.pop() {
        visited += 1;
        if visited > 8192 { fact.truncated = true; break; }
        if comment(node.kind()) { continue; }
        if node.is_named() && callable(node.kind()) { fact.nested_callables += 1; continue; }
        let branch = node.is_named() && is_branch(node.kind());
        if branch {
            let condition = node.child_by_field_name("condition").or_else(|| node.child_by_field_name("value")).unwrap_or(node);
            if fact.branches.len() < 32 { branch_indices.insert(node.id(), fact.branches.len()); }
            push_bounded(&mut fact.branches, expression(condition, source, &parameter_names), &mut fact.truncated);
        }
        if node.is_named() && matches!(node.kind(), "return_statement" | "return_expression" | "return") {
            push_bounded(&mut fact.returns, expression(node, source, &parameter_names), &mut fact.truncated);
        }
        if matches!(node.kind(), "assignment_expression" | "augmented_assignment_expression" | "assignment" | "augmented_assignment") {
            if let Some(left) = node.child_by_field_name("left") {
                // Local variable assignments do not establish shared state ownership.
                if matches!(left.kind(), "member_access_expression" | "member_expression" | "attribute" | "field_expression" | "subscript_expression" | "subscript" | "element_reference") {
                    push_bounded(&mut fact.writes, expression(node, source, &parameter_names), &mut fact.truncated);
                }
            }
        }
        if is_call(node.kind()) {
            if let Some(target) = call_target(node, source) {
                let arguments = node.child_by_field_name("arguments");
                let shapes = arguments.map(|arguments| arguments.named_children(&mut arguments.walk())
                    .take(16).map(|argument| shape(argument, source, &parameter_names)).collect::<Vec<_>>()).unwrap_or_default();
                let arguments_truncated = arguments.is_some_and(|arguments| arguments.named_child_count() > 16)
                    || shapes.iter().any(|(_, truncated)| *truncated);
                let argument_shapes = shapes.into_iter().map(|(shape, _)| shape).collect();
                let receiver = node.child_by_field_name("object").or_else(|| node.child_by_field_name("scope"))
                    .or_else(|| node.child_by_field_name("receiver"))
                    .or_else(|| node.child_by_field_name("function").and_then(|function| function.child_by_field_name("object")
                        .or_else(|| function.child_by_field_name("value"))));
                let receiver_shape = receiver.map(|receiver| shape(receiver, source, &parameter_names).0);
                let (guards, guards_truncated, inside_loop) = call_guards(node, declaration, &branch_indices);
                push_bounded(&mut fact.calls, BehaviorCall {
                    target, receiver_shape, expression: expression(node, source, &parameter_names), argument_shapes,
                    conditional: conditional || optional_call(node), arguments_truncated,
                    guards, guards_truncated, inside_loop,
                }, &mut fact.truncated);
            }
        }
        if let Some(selector) = selector(node, source) { selectors.insert(selector); }
        fact.throws += usize::from(node.is_named() && matches!(node.kind(), "throw_expression" | "throw_statement" | "raise_statement"));
        fact.catches += usize::from(node.is_named() && matches!(node.kind(), "catch_clause" | "except_clause" | "rescue"));
        fact.loops += usize::from(node.is_named() && is_loop(node.kind()));
        if atomic_token(node.kind()) || node.child_count() == 0 {
            tokens.push(token(node, source, &parameter_names));
        } else {
            for index in (0..node.child_count()).rev() {
                if let Some(child) = node.child(index as u32) {
                    let short_circuit_operand = short_circuit(node, source)
                        && node.child_by_field_name("right").is_none_or(|right| right.id() == child.id());
                    pending.push((child, conditional || branch || is_loop(node.kind()) || short_circuit_operand
                        || optional_call(node) || matches!(node.kind(), "catch_clause" | "except_clause" | "rescue" | "rescue_modifier" | "assert_statement")));
                }
            }
        }
    }
    fact.token_count = tokens.len();
    fact.structural_fingerprints = tokens.windows(4).map(|tokens| xxhash_rust::xxh3::xxh3_64(tokens.join(" ").as_bytes()))
        .collect::<BTreeSet<_>>().into_iter().take(32).map(|hash| format!("{hash:016x}")).collect();
    if selectors.len() > 32 { fact.truncated = true; }
    fact.read_selectors = selectors.into_iter().take(32).collect();
    if let Some(tail) = implicit_return(declaration, body, source) {
        push_bounded(&mut fact.returns, expression(tail, source, &parameter_names), &mut fact.truncated);
    }
    fact.truncated |= calling_convention.is_none() || fact.branches.iter().chain(&fact.returns).chain(&fact.writes).any(|expression| expression.shape_truncated)
        || fact.calls.iter().any(|call| call.expression.shape_truncated || call.arguments_truncated || call.guards_truncated);
    fact
}

fn call_guards(mut child: Node<'_>, declaration: Node<'_>, branches: &HashMap<usize, usize>) -> (Vec<BehaviorGuard>, bool, bool) {
    let mut guards = Vec::new();
    let mut truncated = false;
    let mut inside_loop = false;
    while let Some(parent) = child.parent().filter(|parent| parent.id() != declaration.id()) {
        inside_loop |= is_loop(parent.kind());
        if is_branch(parent.kind()) {
            if let Some(index) = branches.get(&parent.id()) {
                let field_matches = |field| parent.child_by_field_name(field).is_some_and(|node| node.id() == child.id());
                let arm = if field_matches("condition") || field_matches("value") { BranchArm::Condition }
                    else if field_matches("alternative") || matches!(child.kind(), "else_clause" | "else_if_clause" | "elif_clause" | "elsif") { BranchArm::Alternative }
                    else if field_matches("consequence") || field_matches("body") { BranchArm::Consequent }
                    else { BranchArm::Unknown };
                if guards.len() < 8 { guards.push(BehaviorGuard { branch: *index, arm }); }
                else { truncated = true; }
            } else { truncated = true; }
        }
        child = parent;
    }
    (guards, truncated, inside_loop)
}

fn short_circuit(node: Node<'_>, source: &str) -> bool {
    let operator = node.child_by_field_name("operator").map(|operator| text(operator, source));
    operator.is_some_and(|operator| matches!(operator, "&&" | "||" | "??" | "and" | "or" | "&&=" | "||=" | "??="))
}

fn optional_call(node: Node<'_>) -> bool {
    node.kind().starts_with("nullsafe_") || node.child_by_field_name("optional_chain").is_some()
        || node.child_by_field_name("function").is_some_and(|function| function.child_by_field_name("optional_chain").is_some())
}

fn push_bounded<T>(items: &mut Vec<T>, item: T, truncated: &mut bool) {
    if items.len() < 32 { items.push(item); } else { *truncated = true; }
}

fn expression(node: Node<'_>, source: &str, parameters: &HashMap<&str, usize>) -> BehaviorExpression {
    let (shape, shape_truncated) = shape(node, source, parameters);
    BehaviorExpression {
        line: node.start_position().row + 1, end_line: node.end_position().row + 1,
        start_byte: node.start_byte(), end_byte: node.end_byte(), shape, shape_truncated,
        fingerprint: format!("{:016x}", xxhash_rust::xxh3::xxh3_64(text(node, source).as_bytes())),
    }
}

fn shape(root: Node<'_>, source: &str, parameters: &HashMap<&str, usize>) -> (String, bool) {
    let mut pending = vec![root];
    let mut tokens = Vec::new();
    let mut truncated = false;
    while let Some(node) = pending.pop() {
        if tokens.len() == 64 { truncated = true; break; }
        if comment(node.kind()) { continue; }
        if atomic_token(node.kind()) || node.child_count() == 0 { tokens.push(token(node, source, parameters)); }
        else {
            for index in (0..node.child_count()).rev() {
                if let Some(child) = node.child(index as u32) { pending.push(child); }
            }
        }
    }
    let value = tokens.join(" ");
    (bounded(&value, 256), truncated || value.len() > 256)
}

fn token(node: Node<'_>, source: &str, parameters: &HashMap<&str, usize>) -> String {
    let value = text(node, source);
    if node.kind() == "variable_name" && value == "$this" { return "this".into(); }
    if string_literal(node.kind()) || node.kind() == "simple_symbol" {
        if let Some(parent) = node.parent().filter(|parent| indexed_selector(parent.kind())) {
            if let Some(key) = selector(parent, source) { return format!("key:{key}"); }
        }
        return "string".into();
    }
    if matches!(node.kind(), "variable_name" | "identifier" | "name") {
        if let Some(index) = parameters.get(value.trim_start_matches('$')) { return format!("input{index}"); }
        if node.kind() == "variable_name" { return "local".into(); }
    }
    bounded(value, 64)
}

fn selector(node: Node<'_>, source: &str) -> Option<String> {
    let selected = match node.kind() {
        "member_access_expression" | "nullsafe_member_access_expression" => node.child_by_field_name("name"),
        "member_expression" => node.child_by_field_name("property"),
        "attribute" => node.child_by_field_name("attribute"),
        "field_expression" => node.child_by_field_name("field"),
        "subscript_expression" | "subscript" | "element_reference" => node.child_by_field_name("index")
            .or_else(|| node.child_by_field_name("subscript"))
            .or_else(|| node.named_child(node.named_child_count().checked_sub(1)? as u32)),
        _ => None,
    }?;
    let selected = if selected.kind() == "argument_list" && selected.named_child_count() == 1 {
        selected.named_child(0)?
    } else { selected };
    if indexed_selector(node.kind()) && !string_literal(selected.kind())
        && !matches!(selected.kind(), "simple_symbol" | "integer" | "integer_literal") { return None; }
    let value = text(selected, source).trim_matches(['\'', '"', '`']).trim_start_matches(':');
    (!value.is_empty() && value.len() <= 64 && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'))
        .then(|| value.to_owned())
}

fn indexed_selector(kind: &str) -> bool { matches!(kind, "subscript_expression" | "subscript" | "element_reference") }

fn call_target(node: Node<'_>, source: &str) -> Option<String> {
    let target = node.child_by_field_name("name").or_else(|| node.child_by_field_name("method"))
        .or_else(|| node.child_by_field_name("function"))?;
    let target = target.child_by_field_name("property").or_else(|| target.child_by_field_name("field"))
        .or_else(|| target.child_by_field_name("attribute")).unwrap_or(target);
    Some(bounded(text(target, source), 128))
}

fn is_call(kind: &str) -> bool {
    matches!(kind, "function_call_expression" | "member_call_expression" | "nullsafe_member_call_expression" | "scoped_call_expression" | "call_expression" | "call" | "method_call_expression")
}

fn directly_delegates(body: Node<'_>) -> bool {
    if is_call(body.kind()) { return true; }
    if body.named_child_count() != 1 { return false; }
    let Some(node) = body.named_child(0) else { return false; };
    is_call(node.kind()) || (matches!(node.kind(), "return_statement" | "return_expression" | "return" | "expression_statement")
        && node.named_child_count() == 1 && node.named_child(0).is_some_and(|child| is_call(child.kind())))
}

fn implicit_return<'a>(declaration: Node<'a>, body: Node<'a>, source: &str) -> Option<Node<'a>> {
    if declaration.kind() == "arrow_function" && body.kind() != "statement_block" { return Some(body); }
    if !matches!(declaration.kind(), "function_item" | "method" | "singleton_method") { return None; }
    let tail = body.named_children(&mut body.walk()).filter(|node| !comment(node.kind())).last()?;
    if matches!(tail.kind(), "return_statement" | "return_expression" | "return" | "let_declaration") { return None; }
    if declaration.kind() == "function_item" && text(tail, source).trim_end().ends_with(';') { return None; }
    Some(tail)
}

fn test_guarded(mut node: Node<'_>, source: &str) -> bool {
    loop {
        if matches!(node.kind(), "function_item" | "mod_item") {
            let mut previous = node.prev_named_sibling();
            while let Some(attribute) = previous.filter(|attribute| attribute.kind() == "attribute_item" || comment(attribute.kind())) {
                let value = text(attribute, source).replace(' ', "");
                if attribute.kind() == "attribute_item" && (value.contains("#[test]") || value.contains("cfg(test)")) { return true; }
                previous = attribute.prev_named_sibling();
            }
        }
        let Some(parent) = node.parent() else { return false; };
        node = parent;
    }
}

fn has_decorators_or_attributes(mut node: Node<'_>) -> bool {
    loop {
        if node.kind() == "decorated_definition" { return true; }
        if callable(node.kind()) || matches!(node.kind(), "class_declaration" | "class_definition" | "interface_declaration"
            | "trait_declaration" | "enum_declaration" | "impl_item" | "struct_item" | "enum_item" | "trait_item" | "mod_item") {
            if node.named_children(&mut node.walk()).any(|child|
                matches!(child.kind(), "decorator" | "attribute_list" | "attribute_item" | "attributes")) { return true; }
            let mut previous = node.prev_named_sibling();
            while let Some(comment_node) = previous.filter(|sibling| comment(sibling.kind())) { previous = comment_node.prev_named_sibling(); }
            if previous.is_some_and(|previous| previous.kind() == "attribute_item") { return true; }
        }
        let Some(parent) = node.parent() else { return false; };
        node = parent;
    }
}

fn calling_convention_modifier(declaration: Node<'_>, body: Node<'_>) -> Option<bool> {
    let mut pending = vec![declaration];
    let mut visited = 0;
    while let Some(node) = pending.pop() {
        if node.id() == body.id() || node.start_byte() >= body.start_byte() { continue; }
        visited += 1;
        if visited > 1024 { return None; }
        if matches!(node.kind(), "async" | "unsafe" | "extern" | "const" | "reference_modifier" | "rest_pattern")
            || node.kind().contains("variadic") || node.kind().contains("splat") || node.kind().starts_with("generator_function") { return Some(true); }
        pending.extend(node.children(&mut node.walk()));
    }
    Some(false)
}

fn is_loop(kind: &str) -> bool {
    matches!(kind, "for_statement" | "for_expression" | "foreach_statement" | "while_statement" | "while_expression" | "for" | "while" | "loop_expression")
}

fn is_branch(kind: &str) -> bool {
    matches!(kind, "if_statement" | "if_expression" | "elif_clause" | "else_if_clause" | "elsif" | "conditional_expression" | "conditional_operator" | "switch_statement" | "match_expression" | "match_statement" | "case_clause" | "if" | "unless" | "case" | "when")
}

fn callable(kind: &str) -> bool {
    matches!(kind, "function_item" | "function_declaration" | "function_definition" | "method_declaration" | "method_definition" | "method" | "singleton_method" | "arrow_function" | "function_expression" | "generator_function_declaration" | "generator_function" | "lambda" | "anonymous_function" | "anonymous_function_creation_expression" | "closure_expression" | "lambda_expression")
}

fn comment(kind: &str) -> bool { kind.contains("comment") }
fn string_literal(kind: &str) -> bool { matches!(kind, "string" | "string_literal" | "encapsed_string" | "template_string" | "raw_string_literal" | "heredoc" | "nowdoc") }
fn atomic_token(kind: &str) -> bool { kind == "variable_name" || string_literal(kind) }
fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str { source.get(node.byte_range()).unwrap_or_default() }
fn bounded(value: &str, limit: usize) -> String {
    let mut end = value.len().min(limit);
    while !value.is_char_boundary(end) { end -= 1; }
    value[..end].to_owned()
}
