use crate::graph::{
    CallForm, EdgeOrigin, EdgeStrength, GraphLayer, Language, ReferenceKind, RelationKind,
    ResolutionTier, ResolvedEdge, SemanticGraph, SymbolKind,
};
use crate::plugins::{
    import_edges_by_reference, import_targets_by_binding, RepoContext, RuntimePlugin,
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
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
        let import_targets = import_targets_by_binding(graph, &symbols_by_id, |symbol| {
            matches!(
                symbol.kind,
                SymbolKind::Function | SymbolKind::Method | SymbolKind::Class | SymbolKind::Module
            )
        })
        .into_iter()
        .map(|(binding, (symbol_id, file_path))| {
            (
                binding,
                SignalCallbackTarget {
                    symbol_id,
                    file_path,
                },
            )
        })
        .collect::<HashMap<_, _>>();
        let same_file_functions = same_file_function_targets(graph);
        let signal_imports = signal_import_identities(graph);
        let methods_by_owner_and_name = methods_by_owner_and_name(graph);
        let argument_bindings = graph.lexical_bindings.calls.iter()
            .filter(|binding| binding.argument_index == Some(0))
            .map(|binding| (binding.reference_index, &binding.target_symbol_id)).collect::<HashMap<_, _>>();
        let mut registrations = HashMap::<SignalIdentity, Vec<SignalCallbackTarget>>::new();
        let mut edges = Vec::new();
        let mut emitted = HashSet::<(PathBuf, String, usize, RelationKind)>::new();

        scan_receiver_decorators(
            repo,
            graph,
            &same_file_functions,
            &signal_imports,
            &mut registrations,
            &mut edges,
            &mut emitted,
        );

        for (reference_index, reference) in graph.references.iter().enumerate()
            .filter(|(_, reference)| is_signal_connect_reference(reference)) {
            let Some(snippet) = repo.source_snippet(&reference.file_path, reference.line, 2) else {
                continue;
            };
            let Some(first_line) = repo
                .source_lines(&reference.file_path)
                .and_then(|lines| lines.get(reference.line.saturating_sub(1)))
            else {
                continue;
            };
            let Some(captures) = connect_call_regex().captures_iter(&snippet).find(|captures| {
                captures
                    .get(0)
                    .is_some_and(|value| value.start() < first_line.len())
                    && captures.name("receiver").map(|value| value.as_str())
                        == reference.receiver_name.as_deref()
            }) else {
                continue;
            };
            let Some(callback_name) = captures.name("callback").map(|value| value.as_str()) else {
                continue;
            };
            let Some(signal_name) = reference.receiver_name.as_deref() else {
                continue;
            };
            let target = if let Some(binding) = argument_bindings.get(&reference_index) {
                binding.as_ref().and_then(|id| symbols_by_id.get(id)).filter(|symbol| {
                    matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method)
                }).map(|symbol| SignalCallbackTarget { symbol_id: symbol.id.clone(), file_path: symbol.file_path.clone() })
            } else {
                resolve_callback_target(
                reference,
                callback_name,
                &symbols_by_id,
                &import_targets,
                &same_file_functions,
                &methods_by_owner_and_name,
                )
            };
            let Some(target) = target else {
                continue;
            };
            registrations
                .entry(signal_identity(
                    &reference.file_path,
                    signal_name,
                    &signal_imports,
                ))
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
            let Some(signal_name) = reference.receiver_name.as_deref() else {
                continue;
            };
            let identity = signal_identity(&reference.file_path, signal_name, &signal_imports);
            let Some(callbacks) = registrations.get(&identity) else {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SignalOrigin {
    File(PathBuf),
    External(Language, String),
    UnresolvedRelativeImport(PathBuf, String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SignalIdentity {
    origin: SignalOrigin,
    member: String,
}

/// Preserve the imported module and exported binding, including aliases. Local
/// receivers keep their full expression and file instead of a global leaf name.
fn signal_import_identities(
    graph: &SemanticGraph,
) -> HashMap<(PathBuf, String), SignalIdentity> {
    let import_edges = import_edges_by_reference(graph);
    let languages = graph
        .files
        .iter()
        .map(|file| (&file.path, file.language))
        .collect::<HashMap<_, _>>();
    graph
        .references
        .iter()
        .filter(|reference| reference.kind.is_import())
        .filter_map(|reference| {
            let binding = reference.binding_name.as_ref()?;
            let (module, member) = reference
                .target_name
                .split_once("::")
                .unwrap_or((&reference.target_name, ""));
            let origin = if let Some(edge) = import_edges.get(&(
                reference.file_path.as_path(),
                reference.line,
                reference.target_name.as_str(),
            )) {
                SignalOrigin::File(edge.target_file_path.clone())
            } else if module.starts_with('.') || module.starts_with('/') {
                SignalOrigin::UnresolvedRelativeImport(reference.file_path.clone(), module.to_owned())
            } else {
                SignalOrigin::External(*languages.get(&reference.file_path)?, module.to_owned())
            };
            Some((
                (reference.file_path.clone(), binding.clone()),
                SignalIdentity {
                    origin,
                    member: if member == "*" {
                        String::new()
                    } else {
                        member.to_owned()
                    },
                },
            ))
        })
        .collect()
}

fn signal_identity(
    file: &Path,
    receiver: &str,
    imports: &HashMap<(PathBuf, String), SignalIdentity>,
) -> SignalIdentity {
    let (binding, suffix) = receiver.split_once('.').unwrap_or((receiver, ""));
    if let Some(imported) = imports.get(&(file.to_path_buf(), binding.to_owned())) {
        let mut identity = imported.clone();
        if !suffix.is_empty() {
            if !identity.member.is_empty() {
                identity.member.push('.');
            }
            identity.member.push_str(suffix);
        }
        identity
    } else {
        SignalIdentity {
            origin: SignalOrigin::File(file.to_path_buf()),
            member: receiver.to_owned(),
        }
    }
}

fn scan_receiver_decorators(
    repo: &RepoContext,
    graph: &SemanticGraph,
    same_file_functions: &HashMap<(PathBuf, String), SignalCallbackTarget>,
    signal_imports: &HashMap<(PathBuf, String), SignalIdentity>,
    registrations: &mut HashMap<SignalIdentity, Vec<SignalCallbackTarget>>,
    edges: &mut Vec<ResolvedEdge>,
    emitted: &mut HashSet<(PathBuf, String, usize, RelationKind)>,
) {
    for file in graph
        .files
        .iter()
        .filter(|file| file.language == Language::Python)
    {
        let Some(lines) = repo.source_lines(&file.path) else {
            continue;
        };
        let mut index = 0usize;
        while index < lines.len() {
            let Some(captures) = receiver_decorator_regex().captures(lines[index]) else {
                index += 1;
                continue;
            };
            let Some(signal_name) = captures
                .name("signal")
                .map(|value| value.as_str())
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
                .entry(signal_identity(&file.path, signal_name, signal_imports))
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

fn next_decorated_function_name<'a>(lines: &[&'a str], start_index: usize) -> Option<(usize, &'a str)> {
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
    methods_by_owner_and_name: &HashMap<(PathBuf, String, String), SignalCallbackTarget>,
) -> Option<SignalCallbackTarget> {
    if let Some((owner, method)) = callback_name.rsplit_once('.') {
        let (owner_file, owner_name) = if matches!(owner, "self" | "this") {
            let enclosing = symbols_by_id.get(reference.enclosing_symbol_id.as_ref()?)?;
            (
                reference.file_path.clone(),
                enclosing.owner_type_name.clone()?,
            )
        } else if let Some(imported) =
            import_targets.get(&(reference.file_path.clone(), owner.to_owned()))
        {
            let symbol = symbols_by_id.get(&imported.symbol_id)?;
            match symbol.kind {
                SymbolKind::Module => {
                    return same_file_functions
                        .get(&(symbol.file_path.clone(), method.to_owned()))
                        .cloned();
                }
                SymbolKind::Class => (symbol.file_path.clone(), symbol.name.clone()),
                _ => return None,
            }
        } else {
            // Qualified members never fall back to unrelated bare functions.
            let mut owners = symbols_by_id.values().filter(|symbol| {
                symbol.file_path == reference.file_path
                    && symbol.kind == SymbolKind::Class
                    && symbol.name == owner
            });
            let symbol = owners.next()?;
            if owners.next().is_some() {
                return None;
            }
            (symbol.file_path.clone(), symbol.name.clone())
        };
        return methods_by_owner_and_name
            .get(&(owner_file, owner_name, method.to_owned()))
            .cloned();
    }

    import_targets
        .get(&(reference.file_path.clone(), callback_name.to_owned()))
        .filter(|target| {
            symbols_by_id.get(&target.symbol_id).is_some_and(|symbol| {
                matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method)
            })
        })
        .cloned()
        .or_else(|| {
            same_file_functions
                .get(&(reference.file_path.clone(), callback_name.to_owned()))
                .cloned()
        })
}

fn same_file_function_targets(
    graph: &SemanticGraph,
) -> HashMap<(PathBuf, String), SignalCallbackTarget> {
    let scoped = graph.lexical_bindings.scoped_symbol_ids.iter().collect::<HashSet<_>>();
    graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Function && !scoped.contains(&symbol.id))
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

fn methods_by_owner_and_name(
    graph: &SemanticGraph,
) -> HashMap<(PathBuf, String, String), SignalCallbackTarget> {
    let scoped = graph.lexical_bindings.scoped_symbol_ids.iter().collect::<HashSet<_>>();
    graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Method && !scoped.contains(&symbol.id))
        .filter_map(|symbol| {
            let owner = symbol.owner_type_name.clone()?;
            Some((
                (symbol.file_path.clone(), owner, symbol.name.clone()),
                SignalCallbackTarget {
                    symbol_id: symbol.id.clone(),
                    file_path: symbol.file_path.clone(),
                },
            ))
        })
        .collect()
}

fn connect_call_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"\b(?P<receiver>[A-Za-z_][A-Za-z0-9_.]*)\.connect\s*\(\s*(?P<callback>[A-Za-z_][A-Za-z0-9_.]*)\s*[,)]"#,
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
            r#"from app.signals import user_logged_in as login_event, update_last_login

def ready():
    login_event.connect(update_last_login)
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
        fs::write(
            fixture.join("app/other_signals.py"),
            "from django.dispatch import Signal\nuser_logged_in = Signal()\n",
        )
        .unwrap();
        fs::write(
            fixture.join("app/unrelated.py"),
            "from app.other_signals import user_logged_in\nuser_logged_in.send(sender='other')\n",
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
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().all(|edge| edge.target_file_path == PathBuf::from("app/signals.py")));
        assert!(edges.iter().all(|edge| edge.source_file_path != PathBuf::from("app/unrelated.py")));
    }

    #[test]
    fn rejects_oauth_socket_leaf_collisions_and_callback_call_results() {
        let fixture = create_fixture();
        fs::write(
            fixture.join("oauth.ts"),
            "export function connectOAuth() { oauthAdapter.value.connect(oauthRedirectUri.value); }\n",
        )
        .unwrap();
        fs::write(
            fixture.join("voice.ts"),
            "export function sendAudio() { liveSocket.value.send(JSON.stringify({ audio: true })); }\n",
        )
        .unwrap();
        fs::write(
            fixture.join("helper.php"),
            "<?php function value(array $row): string { return ''; }\n",
        )
        .unwrap();
        fs::write(
            fixture.join("local.py"),
            r#"def callback():
    return None
def factory():
    return callback

left.value.connect(factory())
left.value.connect(callback)
right.value.send()
left.value.send()
unknown.connect(unimported)
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("unimported.py"),
            "def unimported():\n    return None\n",
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
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().all(|edge| edge.source_file_path == PathBuf::from("local.py")));
        assert!(edges.iter().all(|edge| edge.target_symbol_id.ends_with(":callback")));
        assert!(edges.iter().any(|edge| {
            edge.relation_kind == RelationKind::EventSubscribe && edge.line == 7
        }));
        assert!(edges.iter().any(|edge| {
            edge.relation_kind == RelationKind::EventPublish && edge.line == 9
        }));
    }

    #[test]
    fn resolves_qualified_callbacks_only_in_the_declared_module_or_class() {
        let fixture = create_fixture();
        fs::write(fixture.join("handlers.py"), "def handle():\n    return None\n").unwrap();
        fs::write(
            fixture.join("worker.py"),
            "class Worker:\n    def handle(self):\n        return None\n",
        )
        .unwrap();
        fs::write(
            fixture.join("unrelated.py"),
            "class Worker:\n    def handle(self):\n        return None\n",
        )
        .unwrap();
        fs::write(
            fixture.join("register.py"),
            r#"import handlers as callbacks
from worker import Worker as ImportedWorker

first.connect(callbacks.handle)
second.connect(ImportedWorker.handle)
third.connect(unknown.handle)
first.send()
second.send()
third.send()
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
        assert_eq!(edges.len(), 4);
        for file in ["handlers.py", "worker.py"] {
            assert_eq!(
                edges.iter().filter(|edge| edge.target_file_path == PathBuf::from(file)).count(),
                2
            );
        }
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
    fn uses_lexical_callback_arguments_without_exposing_other_scopes() {
        let fixture = create_fixture();
        fs::write(fixture.join("signals.ts"), r#"
function setup() {
    const callback = () => 1;
    signal.connect(callback);
    signal.send();
}
function unrelated() { other.connect(callback); other.send(); }
function injected(callback) { injectedSignal.connect(callback); injectedSignal.send(); }
"#).unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let edges = analysis.semantic_graph.resolved_edges.iter().filter(|edge| {
            matches!(edge.relation_kind, RelationKind::EventSubscribe | RelationKind::EventPublish)
        }).collect::<Vec<_>>();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().all(|edge| edge.target_symbol_id.ends_with(":callback")));
        assert!(edges.iter().all(|edge| matches!(edge.line, 4 | 5)));
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
