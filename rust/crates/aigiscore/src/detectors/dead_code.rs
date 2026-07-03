use crate::graph::{
    ReferenceKind, ResolvedEdge, SemanticGraph, SymbolKind, SymbolNode, Visibility,
};
use crate::identity::{normalized_path, stable_fingerprint};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadCodeCategory {
    UnusedPrivateFunction,
    UnusedImport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeadCodeProofTier {
    Certain,
    #[default]
    Strong,
    Heuristic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadCodeFinding {
    pub category: DeadCodeCategory,
    pub symbol_id: String,
    pub file_path: PathBuf,
    pub name: String,
    pub line: usize,
    #[serde(default)]
    pub proof_tier: DeadCodeProofTier,
    #[serde(default)]
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DeadCodeResult {
    pub findings: Vec<DeadCodeFinding>,
}

pub fn analyze_dead_code(
    graph: &SemanticGraph,
    parsed_sources: &[(PathBuf, String)],
) -> DeadCodeResult {
    let called_symbols = graph
        .resolved_edges
        .iter()
        .filter(|edge| edge.kind == ReferenceKind::Call)
        .map(|edge| edge.target_symbol_id.clone())
        .collect::<HashSet<_>>();

    let sources_by_path = parsed_sources
        .iter()
        .map(|(path, source)| (path.as_path(), source.as_str()))
        .collect::<HashMap<_, _>>();

    let mut findings = graph
        .symbols
        .iter()
        .filter(|symbol| {
            matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method)
                && symbol.visibility == Visibility::Private
                && symbol.name != "main"
                && !is_runtime_magic_method(symbol.file_path.as_path(), &symbol.name)
                && !has_decorator_binding(symbol, graph)
                && !is_test_or_framework_entry_symbol(symbol, &sources_by_path)
                && !called_symbols.contains(&symbol.id)
                && !private_function_used_lexically_in_file(symbol, &sources_by_path)
        })
        .map(|symbol| DeadCodeFinding {
            category: DeadCodeCategory::UnusedPrivateFunction,
            symbol_id: symbol.id.clone(),
            file_path: symbol.file_path.clone(),
            name: symbol.name.clone(),
            line: symbol.start_line,
            proof_tier: dead_code_proof_tier_for_symbol(symbol),
            fingerprint: dead_code_fingerprint(
                DeadCodeCategory::UnusedPrivateFunction,
                &symbol.file_path,
                &symbol.name,
            ),
        })
        .collect::<Vec<_>>();

    let used_import_targets = graph
        .resolved_edges
        .iter()
        .filter(|edge| edge.kind != ReferenceKind::Import)
        .map(|edge| (edge.source_file_path.clone(), edge.target_symbol_id.clone()))
        .collect::<HashSet<_>>();
    let symbols_by_id = graph
        .symbols
        .iter()
        .map(|symbol| {
            (
                symbol.id.clone(),
                (
                    symbol.name.clone(),
                    symbol.kind,
                    symbol.owner_type_name.clone(),
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    // (source file, line) -> resolved edges at that location, in edge order.
    // Linear scans of `resolved_edges` per reference are O(references x edges)
    // and dominate analysis time on large repositories.
    let mut edges_by_location = HashMap::<(&Path, usize), Vec<&ResolvedEdge>>::new();
    for edge in &graph.resolved_edges {
        edges_by_location
            .entry((edge.source_file_path.as_path(), edge.line))
            .or_default()
            .push(edge);
    }
    let mut receiver_targets_by_binding = HashMap::<(PathBuf, String), HashSet<String>>::new();
    for reference in graph
        .references
        .iter()
        .filter(|reference| reference.kind != ReferenceKind::Import)
    {
        let Some(receiver_name) = reference.receiver_name.as_ref() else {
            continue;
        };
        let binding_name = leaf_symbol_name(receiver_name);
        let matching_targets = edges_by_location
            .get(&(reference.file_path.as_path(), reference.line))
            .into_iter()
            .flatten()
            .filter(|edge| edge.kind == reference.kind)
            .filter_map(|edge| symbols_by_id.get(&edge.target_symbol_id))
            .flat_map(|(name, _, owner_type_name)| {
                owner_type_name
                    .iter()
                    .cloned()
                    .chain(std::iter::once(name.clone()))
            });
        receiver_targets_by_binding
            .entry((reference.file_path.clone(), binding_name))
            .or_default()
            .extend(matching_targets);
    }

    for reference in graph
        .references
        .iter()
        .filter(|reference| reference.kind == ReferenceKind::Import)
    {
        if is_package_export_surface(reference.file_path.as_path()) {
            continue;
        }
        // Imports that bind no local name cannot be "unused" — there is no
        // binding to leave unread. In JavaScript/TypeScript these are the
        // side-effect form (`import './x'`) and the dynamic form
        // (`import('./x')` / `require('./x')`), whose target is a module
        // specifier, not a symbol. Falling back to `leaf_symbol_name` here
        // would fabricate a binding from the path (e.g. `./X.vue` -> `vue`,
        // `zone.js` -> `js`) and flag a phantom import. Python and PHP always
        // record an explicit binding name, so this only skips the JS
        // bindingless forms.
        if reference.binding_name.is_none() {
            continue;
        }
        let candidate_edges = edges_by_location
            .get(&(reference.file_path.as_path(), reference.line))
            .map(|edges| {
                edges
                    .iter()
                    .filter(|edge| edge.kind == ReferenceKind::Import)
                    .copied()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let resolved_import = candidate_edges
            .iter()
            .find(|edge| {
                symbols_by_id
                    .get(&edge.target_symbol_id)
                    .map(|(name, _, _)| name == &leaf_symbol_name(&reference.target_name))
                    .unwrap_or(false)
            })
            .copied()
            .or_else(|| {
                reference.binding_name.as_ref().and_then(|binding_name| {
                    candidate_edges
                        .iter()
                        .find(|edge| {
                            symbols_by_id
                                .get(&edge.target_symbol_id)
                                .map(|(name, _, _)| name == binding_name)
                                .unwrap_or(false)
                        })
                        .copied()
                })
            })
            .or_else(|| {
                candidate_edges
                    .iter()
                    .find(|edge| {
                        symbols_by_id
                            .get(&edge.target_symbol_id)
                            .map(|(_, kind, _)| *kind == SymbolKind::Module)
                            .unwrap_or(false)
                    })
                    .copied()
            });
        let Some(resolved_import) = resolved_import else {
            continue;
        };
        let imported_symbol_name = symbols_by_id
            .get(&resolved_import.target_symbol_id)
            .map(|(name, _, _)| name.clone());
        let binding_name = reference
            .binding_name
            .clone()
            .unwrap_or_else(|| leaf_symbol_name(&reference.target_name));
        if used_import_targets.contains(&(
            reference.file_path.clone(),
            resolved_import.target_symbol_id.clone(),
        )) {
            continue;
        }
        if imported_symbol_name
            .as_ref()
            .is_some_and(|imported_symbol_name| {
                receiver_targets_by_binding
                    .get(&(reference.file_path.clone(), binding_name.clone()))
                    .is_some_and(|targets| targets.contains(imported_symbol_name))
            })
        {
            continue;
        }
        // Framework facades, attributes, `instanceof` checks, and type
        // positions often never resolve to graph edges, so a missing resolved
        // edge is not proof an import is unused. Any mention of the binding
        // name outside import-like lines suppresses the finding.
        if sources_by_path
            .get(reference.file_path.as_path())
            .is_some_and(|source| import_name_used_lexically(source, reference.line, &binding_name))
        {
            continue;
        }
        findings.push(DeadCodeFinding {
            category: DeadCodeCategory::UnusedImport,
            symbol_id: resolved_import.target_symbol_id.clone(),
            file_path: reference.file_path.clone(),
            name: binding_name,
            line: reference.line,
            proof_tier: dead_code_proof_tier(DeadCodeCategory::UnusedImport),
            fingerprint: dead_code_fingerprint(
                DeadCodeCategory::UnusedImport,
                &reference.file_path,
                &reference
                    .binding_name
                    .clone()
                    .unwrap_or_else(|| leaf_symbol_name(&reference.target_name)),
            ),
        });
    }

    findings.sort_by(|left, right| {
        left.file_path
            .cmp(&right.file_path)
            .then(left.line.cmp(&right.line))
            .then(left.name.cmp(&right.name))
    });

    DeadCodeResult { findings }
}

fn import_name_used_lexically(source: &str, import_line: usize, name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    source.lines().enumerate().any(|(index, line)| {
        index + 1 != import_line && !is_import_like_line(line) && line_mentions_word(line, name)
    })
}

fn is_import_like_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    // PHP `use` doubles as a trait-use statement, which is a real usage of an
    // imported trait; only namespaced/aliased `use` lines count as imports.
    if let Some(rest) = trimmed.strip_prefix("use ") {
        return rest.contains('\\') || rest.contains(" as ");
    }
    trimmed.starts_with("import ") || trimmed.starts_with("from ")
}

/// A private function is only reachable from its own file (Rust module, PHP
/// class, Python module), so if its name appears as a bare word anywhere else
/// in that same file — outside its own signature line and comments — the graph
/// simply missed the usage (a higher-order reference like `.map(func)`, a
/// macro expansion, or an unresolved same-file call), and it is not dead.
/// This mirrors the import lexical guard: a missing call edge is not proof.
fn private_function_used_lexically_in_file(
    symbol: &SymbolNode,
    sources_by_path: &HashMap<&Path, &str>,
) -> bool {
    if symbol.name.is_empty() {
        return false;
    }
    let Some(source) = sources_by_path.get(symbol.file_path.as_path()) else {
        return false;
    };
    source.lines().enumerate().any(|(index, line)| {
        index + 1 != symbol.start_line
            && !is_pure_comment_line(line)
            && line_mentions_word(line, &symbol.name)
    })
}

fn is_pure_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    // Rust attributes (`#[...]`, `#![...]`) start with `#` but are code, not
    // comments, and frequently reference functions by name in string form
    // (e.g. serde `skip_serializing_if = "is_zero"`, `serialize_with = "..."`).
    // Only a bare `#` line (Python/shell/Ruby comment) counts.
    let is_rust_attribute = trimmed.starts_with("#[") || trimmed.starts_with("#![");
    trimmed.starts_with("//")
        || trimmed.starts_with('*')
        || trimmed.starts_with("/*")
        || (trimmed.starts_with('#') && !is_rust_attribute)
        || trimmed.starts_with("--")
}

fn line_mentions_word(line: &str, name: &str) -> bool {
    let bytes = line.as_bytes();
    let mut start = 0;
    while let Some(position) = line[start..].find(name) {
        let begin = start + position;
        let end = begin + name.len();
        let boundary_before = begin == 0 || !is_word_byte(bytes[begin - 1]);
        let boundary_after = end >= bytes.len() || !is_word_byte(bytes[end]);
        if boundary_before && boundary_after {
            return true;
        }
        let step = name.chars().next().map(char::len_utf8).unwrap_or(1);
        start = begin + step;
    }
    false
}

fn is_word_byte(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn dead_code_fingerprint(category: DeadCodeCategory, file_path: &Path, name: &str) -> String {
    stable_fingerprint(&[
        "dead-code",
        dead_code_category_label(category),
        &normalized_path(file_path),
        name,
    ])
}

fn dead_code_category_label(category: DeadCodeCategory) -> &'static str {
    match category {
        DeadCodeCategory::UnusedPrivateFunction => "unused-private-function",
        DeadCodeCategory::UnusedImport => "unused-import",
    }
}

pub fn dead_code_proof_tier(category: DeadCodeCategory) -> DeadCodeProofTier {
    match category {
        DeadCodeCategory::UnusedImport => DeadCodeProofTier::Certain,
        DeadCodeCategory::UnusedPrivateFunction => DeadCodeProofTier::Strong,
    }
}

fn dead_code_proof_tier_for_symbol(symbol: &SymbolNode) -> DeadCodeProofTier {
    if is_python_file(symbol.file_path.as_path())
        && (is_nested_private_function(symbol) || is_python_accessor_candidate(&symbol.name))
    {
        return DeadCodeProofTier::Heuristic;
    }
    dead_code_proof_tier(DeadCodeCategory::UnusedPrivateFunction)
}

fn is_runtime_magic_method(file_path: &Path, name: &str) -> bool {
    is_python_file(file_path) && name.len() > 4 && name.starts_with("__") && name.ends_with("__")
}

fn is_package_export_surface(file_path: &Path) -> bool {
    is_python_file(file_path)
        && file_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "__init__.py")
}

fn is_python_file(file_path: &Path) -> bool {
    file_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "py")
}

fn is_nested_private_function(symbol: &SymbolNode) -> bool {
    matches!(symbol.kind, SymbolKind::Function) && symbol.parent_symbol_id.is_some()
}

fn is_python_accessor_candidate(name: &str) -> bool {
    name.starts_with("_get_") || name.starts_with("_set_") || name.starts_with("_del_")
}

fn has_decorator_binding(symbol: &SymbolNode, graph: &SemanticGraph) -> bool {
    is_python_file(symbol.file_path.as_path())
        && graph.references.iter().any(|reference| {
            reference.enclosing_symbol_id.as_deref() == Some(symbol.id.as_str())
                && reference.kind == ReferenceKind::Call
                && reference.line < symbol.start_line
        })
}

/// A test / framework entry function is invoked by a test harness or the
/// runtime (via an attribute), never by another symbol in the graph, so a
/// missing call edge is not proof it is dead. Rust `#[cfg(test)]` modules keep
/// tests inline in a source file (they cannot be excluded by directory the way
/// separate test folders can), so without this guard `#[test]` functions are
/// the dominant dead-code false positive on any Rust codebase.
///
/// Detects attribute-annotated entry points by scanning the attribute/comment
/// block immediately above the function: `#[test]`, `#[tokio::test]` and other
/// `::test` async runners, `#[bench]`, common test-macro attributes, and the
/// externally-linked Rust attributes (`no_mangle`, `export_name`, proc-macros).
fn is_test_or_framework_entry_symbol(
    symbol: &SymbolNode,
    sources_by_path: &HashMap<&Path, &str>,
) -> bool {
    let Some(source) = sources_by_path.get(symbol.file_path.as_path()) else {
        return false;
    };
    preceding_attributes_mark_entry_point(source, symbol.start_line)
}

fn preceding_attributes_mark_entry_point(source: &str, start_line: usize) -> bool {
    if start_line == 0 {
        return false;
    }
    let lines: Vec<&str> = source.lines().collect();
    // start_line is 1-indexed; scan upward from the line above the signature.
    let mut index = start_line.saturating_sub(1);
    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim();
        if trimmed.is_empty()
            || trimmed.starts_with("///")
            || trimmed.starts_with("//!")
            || trimmed.starts_with("//")
        {
            continue;
        }
        if let Some(attribute) = trimmed
            .strip_prefix("#[")
            .and_then(|rest| rest.strip_suffix(']'))
            .or_else(|| {
                trimmed
                    .strip_prefix("#![")
                    .and_then(|r| r.strip_suffix(']'))
            })
        {
            if attribute_is_entry_point(attribute) {
                return true;
            }
            // Another (non-entry) attribute or doc line; keep scanning upward.
            continue;
        }
        // Reached real code (or a modifier like `pub`/`async`/`unsafe`) that is
        // not an attribute or comment — the attribute block has ended.
        if matches!(
            trimmed.split_whitespace().next(),
            Some("pub" | "async" | "unsafe" | "const" | "extern" | "fn")
        ) {
            continue;
        }
        break;
    }
    false
}

/// True when a single attribute's head names a test/bench/entry attribute.
/// `attribute` is the inside of `#[...]`, e.g. `test`, `tokio::test`,
/// `test_case(1, 2)`, `cfg_attr(feature = "x", test)`.
fn attribute_is_entry_point(attribute: &str) -> bool {
    // Head token before any `(` argument list, trimmed.
    let head = attribute.split('(').next().unwrap_or(attribute).trim();
    let last_segment = head.rsplit("::").next().unwrap_or(head).trim();
    if matches!(
        last_segment,
        "test"
            | "bench"
            | "rstest"
            | "test_case"
            | "proptest"
            | "quickcheck"
            | "no_mangle"
            | "export_name"
            | "proc_macro"
            | "proc_macro_derive"
            | "proc_macro_attribute"
    ) {
        return true;
    }
    // `#[cfg_attr(..., test)]` and similar wrappers that inject a test attr.
    if head.starts_with("cfg_attr") && attribute.contains("test") {
        return true;
    }
    false
}

fn leaf_symbol_name(name: &str) -> String {
    name.trim_matches(&['{', '}'][..])
        .rsplit("::")
        .next()
        .unwrap_or(name)
        .rsplit('\\')
        .next()
        .unwrap_or(name)
        .rsplit('.')
        .next()
        .unwrap_or(name)
        .rsplit('/')
        .next()
        .unwrap_or(name)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{analyze_dead_code, DeadCodeCategory, DeadCodeProofTier};
    use crate::parsing::javascript::parse_javascript_to_graph;
    use crate::parsing::php::parse_php_to_graph;
    use crate::parsing::python::parse_python_to_graph;
    use crate::parsing::rust::parse_rust_to_graph;
    use crate::parsing::vue::parse_vue_to_graph;
    use crate::resolve::resolve_graph;
    use std::path::PathBuf;

    #[test]
    fn flags_private_rust_functions_without_incoming_calls() {
        let mut graph = parse_rust_to_graph(
            PathBuf::from("src/lib.rs"),
            r#"
fn helper() {}
fn unused() {}

pub fn entry() {
    helper();
}
"#,
        )
        .unwrap();

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert_eq!(result.findings.len(), 1);
        assert_eq!(
            result.findings[0].category,
            DeadCodeCategory::UnusedPrivateFunction
        );
        assert_eq!(result.findings[0].name, "unused");
        assert_eq!(result.findings[0].proof_tier, DeadCodeProofTier::Strong);
    }

    #[test]
    fn rust_test_functions_are_not_flagged_but_real_dead_code_still_is() {
        let path = PathBuf::from("src/lib.rs");
        let source = r#"
fn dead_helper() {}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_supported_agent_adapters() {
        assert_eq!(1 + 1, 2);
    }

    #[tokio::test]
    async fn redacts_sensitive_api_keys_from_error_text() {
        assert!(true);
    }
}
"#;
        let mut graph = parse_rust_to_graph(path.clone(), source).unwrap();
        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[(path, source.to_string())]);

        let names: Vec<&str> = result.findings.iter().map(|f| f.name.as_str()).collect();
        assert!(
            names.contains(&"dead_helper"),
            "genuinely unused private fn must still be flagged: {names:?}"
        );
        assert!(
            !names.contains(&"parses_supported_agent_adapters"),
            "#[test] fn must not be flagged as dead: {names:?}"
        );
        assert!(
            !names.contains(&"redacts_sensitive_api_keys_from_error_text"),
            "#[tokio::test] fn must not be flagged as dead: {names:?}"
        );
    }

    #[test]
    fn lexical_usage_suppresses_unused_import_but_truly_unused_still_reported() {
        let importer_path = PathBuf::from("app/Actions/ImpersonateUserAction.php");
        let importer_source = r#"<?php
namespace App\Actions;

use App\Models\User;
use App\Models\Account;

final class ImpersonateUserAction
{
    public function handle(object $actor): bool
    {
        return $actor instanceof User;
    }
}
"#;
        let mut graph = parse_php_to_graph(importer_path.clone(), importer_source).unwrap();
        let mut imported = parse_php_to_graph(
            PathBuf::from("app/Models/User.php"),
            r#"<?php
namespace App\Models;

final class User {}
"#,
        )
        .unwrap();
        let mut imported_account = parse_php_to_graph(
            PathBuf::from("app/Models/Account.php"),
            r#"<?php
namespace App\Models;

final class Account {}
"#,
        )
        .unwrap();
        graph.files.append(&mut imported.files);
        graph.symbols.append(&mut imported.symbols);
        graph.references.append(&mut imported.references);
        graph.files.append(&mut imported_account.files);
        graph.symbols.append(&mut imported_account.symbols);
        graph.references.append(&mut imported_account.references);

        resolve_graph(&mut graph);
        let parsed_sources = vec![(importer_path, String::from(importer_source))];
        let result = analyze_dead_code(&graph, &parsed_sources);

        // `User` only appears via `instanceof`, which never resolves to an
        // edge; the lexical guard must keep it out of the findings.
        assert!(!result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && finding.name == "User"));
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && finding.name == "Account"));
    }

    #[test]
    fn flags_unused_rust_imports_when_no_resolved_usage_exists() {
        let mut graph = parse_rust_to_graph(
            PathBuf::from("src/lib.rs"),
            r#"
use crate::models::User;
use crate::models::Repo as RepoAlias;

fn helper(user: User) {
    let _typed: User = User {};
}
"#,
        )
        .unwrap();

        let mut imported = parse_rust_to_graph(
            PathBuf::from("src/models.rs"),
            r#"
pub struct User {}
pub struct Repo {}
"#,
        )
        .unwrap();
        graph.files.append(&mut imported.files);
        graph.symbols.append(&mut imported.symbols);
        graph.references.append(&mut imported.references);

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert!(result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && finding.name == "RepoAlias"
                && finding.proof_tier == DeadCodeProofTier::Certain));
        assert!(!result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && finding.name == "User"));
    }

    #[test]
    fn trait_impl_methods_and_attribute_string_refs_are_not_dead() {
        let source = r#"
struct Config {
    value: usize,
}

trait Serialize {
    fn serialize(&self);
}

impl Serialize for Config {
    fn serialize(&self) {}
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

struct Field {
    #[serde(skip_serializing_if = "is_zero")]
    occurrence_index: usize,
}

fn truly_dead_helper() {}
"#;
        let path = PathBuf::from("src/config.rs");
        let mut graph = parse_rust_to_graph(path.clone(), source).unwrap();
        resolve_graph(&mut graph);
        let parsed_sources = vec![(path, String::from(source))];
        let result = analyze_dead_code(&graph, &parsed_sources);

        // Trait-impl method is public contract surface, not dead.
        assert!(
            !result
                .findings
                .iter()
                .any(|finding| finding.name == "serialize"),
            "trait-impl method must not be flagged dead, got: {:?}",
            result.findings
        );
        // `is_zero` is referenced only inside a `#[serde(...)]` attribute
        // string; the lexical backstop must not treat the attribute as a
        // comment.
        assert!(
            !result
                .findings
                .iter()
                .any(|finding| finding.name == "is_zero"),
            "attribute-string function reference must not be flagged dead, got: {:?}",
            result.findings
        );
        // The genuinely unused private function is still reported.
        assert!(
            result.findings.iter().any(|finding| finding.category
                == DeadCodeCategory::UnusedPrivateFunction
                && finding.name == "truly_dead_helper"),
            "real dead code must still be reported, got: {:?}",
            result.findings
        );
    }

    #[test]
    fn dynamic_import_of_module_specifier_is_not_flagged_as_unused_import() {
        // A lazy dynamic import (`() => import('./Widget.vue')`) resolves to a
        // module symbol now that `.vue` files parse, but it binds no local
        // name. Fabricating a binding from the specifier path used to flag a
        // phantom `vue` import; it must not appear in the findings.
        let importer_source = r#"export const registry = {
  widget: () => import('./Widget.vue'),
};
"#;
        let importer_path = PathBuf::from("src/app.ts");
        let mut graph =
            parse_javascript_to_graph(importer_path.clone(), importer_source, true).unwrap();
        let mut widget = parse_vue_to_graph(
            PathBuf::from("src/Widget.vue"),
            r#"<script setup lang="ts">
const label = "widget";
</script>
<template><div>{{ label }}</div></template>
"#,
        )
        .unwrap();
        graph.files.append(&mut widget.files);
        graph.symbols.append(&mut widget.symbols);
        graph.references.append(&mut widget.references);

        resolve_graph(&mut graph);
        let parsed_sources = vec![(importer_path, String::from(importer_source))];
        let result = analyze_dead_code(&graph, &parsed_sources);

        assert!(
            !result
                .findings
                .iter()
                .any(|finding| finding.category == DeadCodeCategory::UnusedImport),
            "dynamic import bound no name and must not be flagged unused, got: {:?}",
            result.findings
        );
    }

    #[test]
    fn does_not_confuse_same_line_default_and_named_imports() {
        let mut graph = parse_javascript_to_graph(
            PathBuf::from("src/app.ts"),
            r#"import DefaultThing, { User } from "./models";
DefaultThing.run();
const user = new User();
const _unused = user;
"#,
            true,
        )
        .unwrap();
        let mut imported = parse_javascript_to_graph(
            PathBuf::from("src/models.ts"),
            r#"export class User {}
export class Service {
  static run() {}
}
"#,
            true,
        )
        .unwrap();
        graph.files.append(&mut imported.files);
        graph.symbols.append(&mut imported.symbols);
        graph.references.append(&mut imported.references);

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert!(!result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && finding.name == "User"));
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && finding.name == "DefaultThing"));
    }

    #[test]
    fn treats_php_static_receiver_imports_as_used() {
        let mut graph = parse_php_to_graph(
            PathBuf::from("app/Actions/PutOrderAction.php"),
            r#"<?php
namespace App\Actions;

use App\Support\EntityRegistry;
use App\Support\FieldLoader;

final class PutOrderAction
{
    public function handle(): void
    {
        EntityRegistry::get('Task');
        FieldLoader::load('Task');
    }
}
"#,
        )
        .unwrap();
        let mut imported = parse_php_to_graph(
            PathBuf::from("app/Support/EntityRegistry.php"),
            r#"<?php
namespace App\Support;

final class EntityRegistry
{
    public static function get(string $entity): void {}
}

final class FieldLoader
{
    public static function load(string $entity): void {}
}
"#,
        )
        .unwrap();
        graph.files.append(&mut imported.files);
        graph.symbols.append(&mut imported.symbols);
        graph.references.append(&mut imported.references);

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert!(!result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && (finding.name == "EntityRegistry" || finding.name == "FieldLoader")));
    }

    #[test]
    fn treats_aliased_php_static_receiver_imports_as_used() {
        let mut graph = parse_php_to_graph(
            PathBuf::from("app/Actions/PutOrderAction.php"),
            r#"<?php
namespace App\Actions;

use App\Support\EntityRegistry as Registry;

final class PutOrderAction
{
    public function handle(): void
    {
        Registry::get('Task');
    }
}
"#,
        )
        .unwrap();
        let mut imported = parse_php_to_graph(
            PathBuf::from("app/Support/EntityRegistry.php"),
            r#"<?php
namespace App\Support;

final class EntityRegistry
{
    public static function get(string $entity): void {}
}
"#,
        )
        .unwrap();
        graph.files.append(&mut imported.files);
        graph.symbols.append(&mut imported.symbols);
        graph.references.append(&mut imported.references);

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert!(!result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && finding.name == "Registry"));
    }

    #[test]
    fn ignores_python_dunder_methods_for_dead_code() {
        let mut graph = parse_python_to_graph(
            PathBuf::from("pkg/config.py"),
            r#"
class Settings:
    def __repr__(self):
        return "Settings"

    def __iter__(self):
        return iter([])

    def _helper(self):
        return 1
"#,
        )
        .unwrap();

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert!(result
            .findings
            .iter()
            .all(|finding| finding.name != "__repr__" && finding.name != "__iter__"));
        assert!(result.findings.iter().any(|finding| finding.category
            == DeadCodeCategory::UnusedPrivateFunction
            && finding.name == "_helper"));
    }

    #[test]
    fn ignores_python_init_reexport_imports_for_dead_code() {
        let mut graph = parse_python_to_graph(
            PathBuf::from("pkg/__init__.py"),
            r#"
from .config import AppConfig
from .registry import apps
"#,
        )
        .unwrap();
        let mut imported = parse_python_to_graph(
            PathBuf::from("pkg/config.py"),
            r#"
class AppConfig:
    pass
"#,
        )
        .unwrap();
        let mut registry = parse_python_to_graph(
            PathBuf::from("pkg/registry.py"),
            r#"
apps = {}
"#,
        )
        .unwrap();
        graph.files.append(&mut imported.files);
        graph.symbols.append(&mut imported.symbols);
        graph.references.append(&mut imported.references);
        graph.files.append(&mut registry.files);
        graph.symbols.append(&mut registry.symbols);
        graph.references.append(&mut registry.references);

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert!(!result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && (finding.name == "AppConfig" || finding.name == "apps")));
    }

    #[test]
    fn downgrades_python_nested_private_functions_to_heuristic() {
        let mut graph = parse_python_to_graph(
            PathBuf::from("pkg/decorators.py"),
            r#"
def outer():
    def _view_wrapper(request):
        return request

    return _view_wrapper
"#,
        )
        .unwrap();

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert!(result
            .findings
            .iter()
            .any(|finding| finding.name == "_view_wrapper"
                && finding.proof_tier == DeadCodeProofTier::Heuristic));
    }

    #[test]
    fn downgrades_python_accessor_style_methods_to_heuristic() {
        let mut graph = parse_python_to_graph(
            PathBuf::from("pkg/tokens.py"),
            r#"
class Token:
    def _get_secret(self):
        return "secret"

    def _set_secret(self, secret):
        self.secret = secret
"#,
        )
        .unwrap();

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert!(result
            .findings
            .iter()
            .any(|finding| finding.name == "_get_secret"
                && finding.proof_tier == DeadCodeProofTier::Heuristic));
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.name == "_set_secret"
                && finding.proof_tier == DeadCodeProofTier::Heuristic));
    }

    #[test]
    fn treats_python_decorator_bound_private_methods_as_live() {
        let mut graph = parse_python_to_graph(
            PathBuf::from("pkg/config.py"),
            r#"
from functools import cached_property

class Settings:
    @cached_property
    def _token(self):
        return "token"

    @property
    def _flag(self):
        return True
"#,
        )
        .unwrap();

        resolve_graph(&mut graph);
        let result = analyze_dead_code(&graph, &[]);

        assert!(!result
            .findings
            .iter()
            .any(|finding| finding.name == "_token" || finding.name == "_flag"));
        assert!(!result
            .findings
            .iter()
            .any(|finding| finding.category == DeadCodeCategory::UnusedImport
                && finding.name == "cached_property"));
    }
}
