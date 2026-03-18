use crate::graph::{
    CallForm, EdgeOrigin, EdgeStrength, GraphLayer, ReferenceKind, RelationKind, ResolutionTier,
    ResolvedEdge, SemanticGraph, SymbolKind,
};
use crate::plugins::{RepoContext, RuntimePlugin};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct WordPressHooksPlugin;

impl RuntimePlugin for WordPressHooksPlugin {
    fn id(&self) -> &'static str {
        "wordpress_hooks"
    }

    fn emit_edges(&self, repo: &RepoContext, graph: &SemanticGraph) -> Vec<ResolvedEdge> {
        let symbols_by_id = graph
            .symbols
            .iter()
            .map(|symbol| (symbol.id.clone(), symbol))
            .collect::<HashMap<_, _>>();
        let functions_by_name = global_unique_function_targets(graph);
        let methods_by_owner_and_name = methods_by_owner_and_name(graph);
        let mut source_cache = HashMap::<PathBuf, Vec<String>>::new();
        let mut registrations = HashMap::<String, Vec<HookCallbackTarget>>::new();
        let mut edges = Vec::new();
        let mut emitted = HashSet::<(PathBuf, String, usize, RelationKind)>::new();

        for reference in graph
            .references
            .iter()
            .filter(is_hook_registration_reference)
        {
            let Some(snippet) = source_snippet(
                repo,
                &reference.file_path,
                reference.line,
                &mut source_cache,
            ) else {
                continue;
            };
            let Some((hook_name, callback)) = parse_registration(
                &snippet,
                reference,
                &symbols_by_id,
                &functions_by_name,
                &methods_by_owner_and_name,
            ) else {
                continue;
            };
            registrations
                .entry(hook_name.clone())
                .or_default()
                .push(callback.clone());
            if emitted.insert((
                reference.file_path.clone(),
                callback.symbol_id.clone(),
                reference.line,
                RelationKind::EventSubscribe,
            )) {
                edges.push(
                    ResolvedEdge::new(
                        reference.file_path.clone(),
                        reference.enclosing_symbol_id.clone(),
                        callback.file_path.clone(),
                        callback.symbol_id.clone(),
                        ReferenceKind::Call,
                        ResolutionTier::Global,
                        650,
                        format!("wordpress hook registration `{hook_name}`"),
                        reference.line,
                    )
                    .with_metadata(
                        RelationKind::EventSubscribe,
                        GraphLayer::Framework,
                        EdgeStrength::Dynamic,
                        EdgeOrigin::Plugin,
                    ),
                );
            }
        }

        for reference in graph.references.iter().filter(is_hook_dispatch_reference) {
            let Some(snippet) = source_snippet(
                repo,
                &reference.file_path,
                reference.line,
                &mut source_cache,
            ) else {
                continue;
            };
            let Some(hook_name) = parse_dispatch_hook_name(&snippet) else {
                continue;
            };
            let Some(callbacks) = registrations.get(&hook_name) else {
                continue;
            };
            for callback in callbacks {
                if !emitted.insert((
                    reference.file_path.clone(),
                    callback.symbol_id.clone(),
                    reference.line,
                    RelationKind::EventPublish,
                )) {
                    continue;
                }
                edges.push(
                    ResolvedEdge::new(
                        reference.file_path.clone(),
                        reference.enclosing_symbol_id.clone(),
                        callback.file_path.clone(),
                        callback.symbol_id.clone(),
                        ReferenceKind::Call,
                        ResolutionTier::Global,
                        700,
                        format!("wordpress hook dispatch `{hook_name}`"),
                        reference.line,
                    )
                    .with_metadata(
                        RelationKind::EventPublish,
                        GraphLayer::Runtime,
                        EdgeStrength::Dynamic,
                        EdgeOrigin::Plugin,
                    ),
                );
            }
        }

        edges
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HookCallbackTarget {
    symbol_id: String,
    file_path: PathBuf,
}

fn is_hook_registration_reference(reference: &&crate::graph::SemanticReference) -> bool {
    reference.kind == ReferenceKind::Call
        && reference.call_form == Some(CallForm::Free)
        && matches!(reference.target_name.as_str(), "add_action" | "add_filter")
}

fn is_hook_dispatch_reference(reference: &&crate::graph::SemanticReference) -> bool {
    reference.kind == ReferenceKind::Call
        && reference.call_form == Some(CallForm::Free)
        && matches!(
            reference.target_name.as_str(),
            "do_action" | "apply_filters"
        )
}

fn parse_registration(
    snippet: &str,
    reference: &crate::graph::SemanticReference,
    symbols_by_id: &HashMap<String, &crate::graph::SymbolNode>,
    functions_by_name: &HashMap<String, HookCallbackTarget>,
    methods_by_owner_and_name: &HashMap<(String, String), HookCallbackTarget>,
) -> Option<(String, HookCallbackTarget)> {
    if let Some(captures) = registration_string_callback_regex().captures(snippet) {
        let hook_name = captures.name("hook")?.as_str().to_owned();
        let callback = captures.name("callback")?.as_str();
        let target = if let Some((owner, method)) = callback.split_once("::") {
            methods_by_owner_and_name
                .get(&(owner.to_owned(), method.to_owned()))
                .cloned()
        } else {
            functions_by_name.get(callback).cloned()
        }?;
        return Some((hook_name, target));
    }

    let captures = registration_array_this_callback_regex()
        .captures(snippet)
        .or_else(|| registration_short_array_this_callback_regex().captures(snippet))?;
    let hook_name = captures.name("hook")?.as_str().to_owned();
    if let Some(method) = captures.name("method") {
        let enclosing_symbol_id = reference.enclosing_symbol_id.as_ref()?;
        let owner = symbols_by_id
            .get(enclosing_symbol_id)
            .and_then(|symbol| symbol.owner_type_name.clone())
            .or_else(|| {
                symbols_by_id
                    .get(enclosing_symbol_id)
                    .map(|symbol| symbol.name.clone())
            })?;
        let target = methods_by_owner_and_name
            .get(&(owner, method.as_str().to_owned()))
            .cloned()?;
        return Some((hook_name, target));
    }
    None
}

fn parse_dispatch_hook_name(snippet: &str) -> Option<String> {
    dispatch_hook_regex()
        .captures(snippet)
        .and_then(|captures| captures.name("hook"))
        .map(|value| value.as_str().to_owned())
}

fn source_snippet(
    repo: &RepoContext,
    file_path: &Path,
    line: usize,
    source_cache: &mut HashMap<PathBuf, Vec<String>>,
) -> Option<String> {
    let lines = source_cache
        .entry(file_path.to_path_buf())
        .or_insert_with(|| {
            fs::read_to_string(repo.root().join(file_path))
                .map(|source| source.lines().map(str::to_owned).collect())
                .unwrap_or_default()
        });
    if lines.is_empty() {
        return None;
    }
    let start = line.saturating_sub(1);
    let end = (start + 4).min(lines.len());
    Some(lines[start..end].join(" "))
}

fn global_unique_function_targets(graph: &SemanticGraph) -> HashMap<String, HookCallbackTarget> {
    let mut grouped = HashMap::<String, Vec<HookCallbackTarget>>::new();
    for symbol in graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Function)
    {
        grouped
            .entry(symbol.name.clone())
            .or_default()
            .push(HookCallbackTarget {
                symbol_id: symbol.id.clone(),
                file_path: symbol.file_path.clone(),
            });
    }
    grouped
        .into_iter()
        .filter_map(|(name, mut matches)| {
            if matches.len() == 1 {
                matches.pop().map(|target| (name, target))
            } else {
                None
            }
        })
        .collect()
}

fn methods_by_owner_and_name(
    graph: &SemanticGraph,
) -> HashMap<(String, String), HookCallbackTarget> {
    graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Method)
        .filter_map(|symbol| {
            let owner = symbol.owner_type_name.clone()?;
            Some((
                (owner, symbol.name.clone()),
                HookCallbackTarget {
                    symbol_id: symbol.id.clone(),
                    file_path: symbol.file_path.clone(),
                },
            ))
        })
        .collect()
}

fn registration_string_callback_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"add_(?:action|filter)\s*\(\s*'(?P<hook>[^']+)'\s*,\s*'(?P<callback>[^']+)'")
            .expect("valid wordpress string callback regex")
    })
}

fn registration_array_this_callback_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"add_(?:action|filter)\s*\(\s*'(?P<hook>[^']+)'\s*,\s*array\s*\(\s*\$this\s*,\s*'(?P<method>[^']+)'\s*\)",
        )
        .expect("valid wordpress this-array callback regex")
    })
}

fn registration_short_array_this_callback_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"add_(?:action|filter)\s*\(\s*'(?P<hook>[^']+)'\s*,\s*\[\s*\$this\s*,\s*'(?P<method>[^']+)'\s*\]",
        )
        .expect("valid wordpress short this-array callback regex")
    })
}

fn dispatch_hook_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?:do_action|apply_filters)\s*\(\s*'(?P<hook>[^']+)'")
            .expect("valid wordpress dispatch regex")
    })
}

#[cfg(test)]
mod tests {
    use super::WordPressHooksPlugin;
    use crate::graph::{GraphLayer, RelationKind};
    use crate::ingestion::scan::ScanConfig;
    use crate::plugins::{RepoContext, RuntimePlugin};
    use crate::resolve::resolve_graph;
    use crate::{ingestion::pipeline::analyze_project, parsing::php::parse_php_to_graph};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn emits_hook_edges_for_string_callbacks() {
        let fixture = create_fixture();
        let file_path = fixture.join("src/wp-signup.php");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(
            &file_path,
            r#"<?php
function do_signup_header() {
    do_action( 'signup_header' );
}
add_action( 'wp_head', 'do_signup_header' );
do_action( 'wp_head' );
"#,
        )
        .unwrap();

        let mut graph = parse_php_to_graph(
            PathBuf::from("src/wp-signup.php"),
            &fs::read_to_string(&file_path).unwrap(),
        )
        .unwrap();
        resolve_graph(&mut graph);

        let plugin = WordPressHooksPlugin;
        let edges = plugin.emit_edges(&RepoContext::new(&fixture), &graph);

        assert!(edges.iter().any(|edge| {
            edge.relation_kind == RelationKind::EventSubscribe
                && edge.layer == GraphLayer::Framework
        }));
        assert!(edges.iter().any(|edge| {
            edge.relation_kind == RelationKind::EventPublish && edge.layer == GraphLayer::Runtime
        }));
    }

    #[test]
    fn emits_hook_edges_for_this_array_callbacks_from_wordpress_source_shape() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src/wp-includes")).unwrap();
        fs::write(
            fixture.join("src/wp-includes/class-wp-customize-manager.php"),
            r#"<?php
final class WP_Customize_Manager {
    public function register() {
        add_action( 'customize_register', array( $this, 'register_controls' ) );
        do_action( 'customize_register' );
    }

    public function register_controls() {}
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let hook_edges = analysis
            .semantic_graph
            .resolved_edges
            .iter()
            .filter(|edge| {
                matches!(
                    edge.relation_kind,
                    RelationKind::EventSubscribe | RelationKind::EventPublish
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(hook_edges.len(), 2);
        assert!(hook_edges.iter().any(|edge| edge.line == 4));
        assert!(hook_edges.iter().any(|edge| edge.line == 5));
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-wordpress-plugin-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
