use crate::graph::{ReferenceKind, SemanticGraph, SymbolKind, Visibility};
use crate::identity::{normalized_path, stable_fingerprint};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadCodeCategory {
    UnusedPrivateFunction,
    UnusedImport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadCodeFinding {
    pub category: DeadCodeCategory,
    pub symbol_id: String,
    pub file_path: PathBuf,
    pub name: String,
    pub line: usize,
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
                && !called_symbols.contains(&symbol.id)
        })
        .map(|symbol| DeadCodeFinding {
            category: DeadCodeCategory::UnusedPrivateFunction,
            symbol_id: symbol.id.clone(),
            file_path: symbol.file_path.clone(),
            name: symbol.name.clone(),
            line: symbol.start_line,
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
    use super::{analyze_dead_code, DeadCodeCategory};
    use crate::parsing::javascript::parse_javascript_to_graph;
    use crate::parsing::php::parse_php_to_graph;
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
                && finding.name == "RepoAlias"));
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
}
