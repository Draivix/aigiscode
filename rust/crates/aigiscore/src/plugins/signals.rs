use crate::graph::{
    CallForm, EdgeOrigin, EdgeStrength, GraphLayer, Language, ReferenceKind, RelationKind,
    ResolutionTier, ResolvedEdge, SemanticGraph, SymbolKind,
};
use crate::plugins::{RepoContext, RuntimePlugin};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct SignalCallbacksPlugin;

impl RuntimePlugin for SignalCallbacksPlugin {
    fn id(&self) -> &'static str {
        "signal_callbacks"
    }

    fn emit_edges(&self, repo: &RepoContext, graph: &SemanticGraph) -> Vec<ResolvedEdge> {
        let symbols_by_id = graph
            .symbols
            .iter()
            .map(|symbol| (symbol.id.clone(), symbol))
            .collect::<HashMap<_, _>>();
        let import_targets = import_targets_by_binding(graph, &symbols_by_id);
        let same_file_functions = same_file_function_targets(graph);
        let global_unique_functions = global_unique_function_targets(graph);
        let methods_by_owner_and_name = methods_by_owner_and_name(graph);
        let mut source_cache = HashMap::<PathBuf, Vec<String>>::new();
        let mut registrations = HashMap::<String, Vec<SignalCallbackTarget>>::new();
        let mut edges = Vec::new();
        let mut emitted = HashSet::<(PathBuf, String, usize, RelationKind)>::new();

        scan_receiver_decorators(
            repo,
            graph,
            &same_file_functions,
            &mut source_cache,
            &mut registrations,
            &mut edges,
            &mut emitted,
        );

        for reference in graph.references.iter().filter(is_signal_connect_reference) {
            let Some(snippet) = source_snippet(
                repo,
                &reference.file_path,
                reference.line,
                &mut source_cache,
                2,
            ) else {
                continue;
            };
            let Some(captures) = connect_call_regex().captures(&snippet) else {
                continue;
            };
            let Some(callback_name) = captures.name("callback").map(|value| value.as_str()) else {
                continue;
            };
            let Some(signal_name) = reference
                .receiver_name
                .as_deref()
                .map(normalize_symbol_name)
            else {
                continue;
            };
            let Some(target) = resolve_callback_target(
                reference,
                callback_name,
                &symbols_by_id,
                &import_targets,
                &same_file_functions,
                &global_unique_functions,
                &methods_by_owner_and_name,
            ) else {
                continue;
            };
            registrations
                .entry(signal_name.clone())
                .or_default()
                .push(target.clone());
            emit_signal_edge(
                &mut edges,
                &mut emitted,
                reference.file_path.clone(),
                reference.enclosing_symbol_id.clone(),
                target,
                reference.line,
                RelationKind::EventSubscribe,
                GraphLayer::Framework,
                format!("signal callback registration `{signal_name}`"),
            );
        }

        for reference in graph.references.iter().filter(is_signal_send_reference) {
            let Some(signal_name) = reference
                .receiver_name
                .as_deref()
                .map(normalize_symbol_name)
            else {
                continue;
            };
            let Some(callbacks) = registrations.get(&signal_name) else {
                continue;
            };
            for callback in callbacks {
                emit_signal_edge(
                    &mut edges,
                    &mut emitted,
                    reference.file_path.clone(),
                    reference.enclosing_symbol_id.clone(),
                    callback.clone(),
                    reference.line,
                    RelationKind::EventPublish,
                    GraphLayer::Runtime,
                    format!("signal dispatch `{signal_name}`"),
                );
            }
        }

        edges
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SignalCallbackTarget {
    symbol_id: String,
    file_path: PathBuf,
}

fn scan_receiver_decorators(
    repo: &RepoContext,
    graph: &SemanticGraph,
    same_file_functions: &HashMap<(PathBuf, String), SignalCallbackTarget>,
    source_cache: &mut HashMap<PathBuf, Vec<String>>,
    registrations: &mut HashMap<String, Vec<SignalCallbackTarget>>,
    edges: &mut Vec<ResolvedEdge>,
    emitted: &mut HashSet<(PathBuf, String, usize, RelationKind)>,
) {
    for file in graph
        .files
        .iter()
        .filter(|file| file.language == Language::Python)
    {
        let Some(lines) = source_lines(repo, &file.path, source_cache) else {
            continue;
        };
        let mut index = 0usize;
        while index < lines.len() {
            let Some(captures) = receiver_decorator_regex().captures(&lines[index]) else {
                index += 1;
                continue;
            };
            let Some(signal_name) = captures
                .name("signal")
                .map(|value| normalize_symbol_name(value.as_str()))
            else {
                index += 1;
                continue;
            };
            let Some((line_number, function_name)) =
                next_decorated_function_name(lines, index.saturating_add(1))
            else {
                index += 1;
                continue;
            };
            let Some(target) = same_file_functions
                .get(&(file.path.clone(), function_name.to_owned()))
                .cloned()
            else {
                index += 1;
                continue;
            };
            registrations
                .entry(signal_name.clone())
                .or_default()
                .push(target.clone());
            emit_signal_edge(
                edges,
                emitted,
                file.path.clone(),
                None,
                target,
                index + 1,
                RelationKind::EventSubscribe,
                GraphLayer::Framework,
                format!("signal callback registration `{signal_name}`"),
            );
            index = line_number.saturating_sub(1);
        }
    }
}

fn next_decorated_function_name(lines: &[String], start_index: usize) -> Option<(usize, &str)> {
    let mut index = start_index;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.is_empty() || trimmed.starts_with('@') {
            index += 1;
            continue;
        }
        let captures = python_function_definition_regex().captures(trimmed)?;
        let name = captures.name("name")?.as_str();
        return Some((index + 1, name));
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn emit_signal_edge(
    edges: &mut Vec<ResolvedEdge>,
    emitted: &mut HashSet<(PathBuf, String, usize, RelationKind)>,
    source_file_path: PathBuf,
    source_symbol_id: Option<String>,
    target: SignalCallbackTarget,
    line: usize,
    relation_kind: RelationKind,
    layer: GraphLayer,
    reason: String,
) {
    if !emitted.insert((
        source_file_path.clone(),
        target.symbol_id.clone(),
        line,
        relation_kind,
    )) {
        return;
    }

    edges.push(
        ResolvedEdge::new(
            source_file_path,
            source_symbol_id,
            target.file_path,
            target.symbol_id,
            ReferenceKind::Call,
            ResolutionTier::Global,
            650,
            reason,
            line,
        )
        .with_metadata(
            relation_kind,
            layer,
            EdgeStrength::Dynamic,
            EdgeOrigin::Plugin,
        ),
    );
}

fn is_signal_connect_reference(reference: &&crate::graph::SemanticReference) -> bool {
    reference.kind == ReferenceKind::Call
        && reference.call_form == Some(CallForm::Member)
        && reference.target_name == "connect"
        && reference.receiver_name.is_some()
}

fn is_signal_send_reference(reference: &&crate::graph::SemanticReference) -> bool {
    reference.kind == ReferenceKind::Call
        && reference.call_form == Some(CallForm::Member)
        && matches!(reference.target_name.as_str(), "send" | "send_robust")
        && reference.receiver_name.is_some()
}

fn resolve_callback_target(
    reference: &crate::graph::SemanticReference,
    callback_name: &str,
    symbols_by_id: &HashMap<String, &crate::graph::SymbolNode>,
    import_targets: &HashMap<(PathBuf, String), SignalCallbackTarget>,
    same_file_functions: &HashMap<(PathBuf, String), SignalCallbackTarget>,
    global_unique_functions: &HashMap<String, SignalCallbackTarget>,
    methods_by_owner_and_name: &HashMap<(String, String), SignalCallbackTarget>,
) -> Option<SignalCallbackTarget> {
    if let Some((owner, method)) = callback_name.rsplit_once('.') {
        let owner_name = if owner == "self" {
            reference
                .enclosing_symbol_id
                .as_ref()
                .and_then(|id| symbols_by_id.get(id))
                .and_then(|symbol| symbol.owner_type_name.as_ref().or(Some(&symbol.name)))
                .cloned()
        } else {
            Some(normalize_symbol_name(owner))
        }?;
        if let Some(target) = methods_by_owner_and_name
            .get(&(owner_name, method.to_owned()))
            .cloned()
        {
            return Some(target);
        }
    }

    let binding_name = normalize_symbol_name(callback_name);
    import_targets
        .get(&(reference.file_path.clone(), binding_name.clone()))
        .cloned()
        .or_else(|| {
            same_file_functions
                .get(&(reference.file_path.clone(), binding_name.clone()))
                .cloned()
        })
        .or_else(|| global_unique_functions.get(&binding_name).cloned())
}

fn import_targets_by_binding(
    graph: &SemanticGraph,
    symbols_by_id: &HashMap<String, &crate::graph::SymbolNode>,
) -> HashMap<(PathBuf, String), SignalCallbackTarget> {
    let mut targets = HashMap::new();

    for reference in graph
        .references
        .iter()
        .filter(|reference| reference.kind == ReferenceKind::Import)
    {
        let binding_name = reference
            .binding_name
            .clone()
            .unwrap_or_else(|| normalize_symbol_name(&reference.target_name));
        let resolved_import = graph.resolved_edges.iter().find(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.source_file_path == reference.file_path
                && edge.line == reference.line
        });
        let Some(resolved_import) = resolved_import else {
            continue;
        };
        let Some(symbol) = symbols_by_id.get(&resolved_import.target_symbol_id) else {
            continue;
        };
        if !matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method) {
            continue;
        }
        targets.insert(
            (reference.file_path.clone(), binding_name),
            SignalCallbackTarget {
                symbol_id: symbol.id.clone(),
                file_path: symbol.file_path.clone(),
            },
        );
    }

    targets
}

fn same_file_function_targets(
    graph: &SemanticGraph,
) -> HashMap<(PathBuf, String), SignalCallbackTarget> {
    graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Function)
        .map(|symbol| {
            (
                (symbol.file_path.clone(), symbol.name.clone()),
                SignalCallbackTarget {
                    symbol_id: symbol.id.clone(),
                    file_path: symbol.file_path.clone(),
                },
            )
        })
        .collect()
}

fn global_unique_function_targets(graph: &SemanticGraph) -> HashMap<String, SignalCallbackTarget> {
    let mut grouped = HashMap::<String, Vec<SignalCallbackTarget>>::new();
    for symbol in graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Function)
    {
        grouped
            .entry(symbol.name.clone())
            .or_default()
            .push(SignalCallbackTarget {
                symbol_id: symbol.id.clone(),
                file_path: symbol.file_path.clone(),
            });
    }

    grouped
        .into_iter()
        .filter_map(|(name, mut matches)| {
            if matches.len() == 1 {
                Some((name, matches.remove(0)))
            } else {
                None
            }
        })
        .collect()
}

fn methods_by_owner_and_name(
    graph: &SemanticGraph,
) -> HashMap<(String, String), SignalCallbackTarget> {
    graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Method)
        .filter_map(|symbol| {
            let owner = symbol.owner_type_name.clone()?;
            Some((
                (owner, symbol.name.clone()),
                SignalCallbackTarget {
                    symbol_id: symbol.id.clone(),
                    file_path: symbol.file_path.clone(),
                },
            ))
        })
        .collect()
}

fn source_lines<'a>(
    repo: &RepoContext,
    file_path: &Path,
    source_cache: &'a mut HashMap<PathBuf, Vec<String>>,
) -> Option<&'a Vec<String>> {
    let lines = source_cache
        .entry(file_path.to_path_buf())
        .or_insert_with(|| {
            fs::read_to_string(repo.root().join(file_path))
                .map(|source| source.lines().map(str::to_owned).collect())
                .unwrap_or_default()
        });
    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

fn source_snippet(
    repo: &RepoContext,
    file_path: &Path,
    line: usize,
    source_cache: &mut HashMap<PathBuf, Vec<String>>,
    context_after: usize,
) -> Option<String> {
    let lines = source_lines(repo, file_path, source_cache)?;
    let start = line.saturating_sub(1);
    let end = (start + context_after + 1).min(lines.len());
    Some(lines[start..end].join(" "))
}

fn normalize_symbol_name(name: &str) -> String {
    name.trim()
        .trim_matches(&['{', '}'][..])
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

fn connect_call_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"\b[A-Za-z_][A-Za-z0-9_.]*\.connect\s*\(\s*(?P<callback>[A-Za-z_][A-Za-z0-9_.]*)"#,
        )
        .expect("valid signal connect regex")
    })
}

fn receiver_decorator_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"^\s*@receiver\(\s*(?P<signal>[A-Za-z_][A-Za-z0-9_.]*)"#)
            .expect("valid signal receiver regex")
    })
}

fn python_function_definition_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"^(?:async\s+def|def)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\("#)
            .expect("valid python function definition regex")
    })
}

#[cfg(test)]
mod tests {
    use super::SignalCallbacksPlugin;
    use crate::graph::{GraphLayer, RelationKind};
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn emits_runtime_edges_for_python_signal_connect_and_send() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app")).unwrap();
        fs::write(
            fixture.join("app/signals.py"),
            r#"from django.dispatch import Signal

user_logged_in = Signal()

def update_last_login(**kwargs):
    return None
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/apps.py"),
            r#"from app.signals import user_logged_in, update_last_login

def ready():
    user_logged_in.connect(update_last_login)
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/runtime.py"),
            r#"from app.signals import user_logged_in

def dispatch_login():
    user_logged_in.send(sender="demo")
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let edges = analysis
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

        assert!(edges.iter().any(|edge| {
            edge.relation_kind == RelationKind::EventSubscribe
                && edge.layer == GraphLayer::Framework
        }));
        assert!(edges.iter().any(|edge| {
            edge.relation_kind == RelationKind::EventPublish && edge.layer == GraphLayer::Runtime
        }));
    }

    #[test]
    fn emits_runtime_edges_for_python_receiver_decorators() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app")).unwrap();
        fs::write(
            fixture.join("app/hashers.py"),
            r#"from django.core.signals import setting_changed
from django.dispatch import receiver

@receiver(setting_changed)
def reset_hashers(**kwargs):
    return None
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let edges = analysis
            .semantic_graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.relation_kind == RelationKind::EventSubscribe)
            .collect::<Vec<_>>();

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].layer, GraphLayer::Framework);
    }

    #[test]
    fn plugin_descriptor_is_constructible() {
        let _plugin = SignalCallbacksPlugin;
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-signal-plugin-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
