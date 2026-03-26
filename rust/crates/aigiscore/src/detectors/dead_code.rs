use crate::graph::{ReferenceKind, SemanticGraph, SymbolKind, SymbolNode, Visibility};
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

pub fn analyze_dead_code(graph: &SemanticGraph) -> DeadCodeResult {
    let called_symbols = graph
        .resolved_edges
        .iter()
        .filter(|edge| edge.kind == ReferenceKind::Call)
        .map(|edge| edge.target_symbol_id.clone())
        .collect::<HashSet<_>>();

    let mut findings = graph
        .symbols
        .iter()
        .filter(|symbol| {
            matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method)
                && symbol.visibility == Visibility::Private
                && symbol.name != "main"
                && !is_runtime_magic_method(symbol.file_path.as_path(), &symbol.name)
                && !has_decorator_binding(symbol, graph)
                && !called_symbols.contains(&symbol.id)
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
        let matching_targets = graph
            .resolved_edges
            .iter()
            .filter(|edge| {
                edge.kind == reference.kind
                    && edge.source_file_path == reference.file_path
                    && edge.line == reference.line
            })
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
        let candidate_edges = graph
            .resolved_edges
            .iter()
            .filter(|edge| {
                edge.kind == ReferenceKind::Import
                    && edge.source_file_path == reference.file_path
                    && edge.line == reference.line
            })
            .collect::<Vec<_>>();
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
        let result = analyze_dead_code(&graph);

        assert_eq!(result.findings.len(), 1);
        assert_eq!(
            result.findings[0].category,
            DeadCodeCategory::UnusedPrivateFunction
        );
        assert_eq!(result.findings[0].name, "unused");
        assert_eq!(result.findings[0].proof_tier, DeadCodeProofTier::Strong);
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
        let result = analyze_dead_code(&graph);

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
        let result = analyze_dead_code(&graph);

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
        let result = analyze_dead_code(&graph);

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
        let result = analyze_dead_code(&graph);

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
        let result = analyze_dead_code(&graph);

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
        let result = analyze_dead_code(&graph);

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
        let result = analyze_dead_code(&graph);

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
        let result = analyze_dead_code(&graph);

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
        let result = analyze_dead_code(&graph);

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
