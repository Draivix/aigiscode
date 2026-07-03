use crate::graph::{
    CallForm, Language, ReferenceKind, ResolutionTier, ResolvedEdge, SemanticGraph,
    SemanticReference, SymbolKind,
};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolDefinition {
    pub symbol_id: String,
    pub file_path: PathBuf,
    pub kind: SymbolKind,
    pub name: String,
    pub qualified_name: String,
    pub parent_symbol_id: Option<String>,
    pub owner_type_name: Option<String>,
    pub return_type_name: Option<String>,
    pub parameter_count: usize,
    pub required_parameter_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TieredCandidates {
    pub candidates: Vec<SymbolDefinition>,
    pub tier: ResolutionTier,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ReceiverResolution {
    symbol_ids: HashSet<String>,
    type_names: HashSet<String>,
    file_paths: HashSet<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct ResolveConfig {
    tsconfig_paths: Vec<TsPathAlias>,
    composer_psr4: Vec<ComposerPsr4Mapping>,
    python_roots: Vec<PathBuf>,
    ruby_load_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct TsPathAlias {
    pattern: String,
    targets: Vec<String>,
    base_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct ComposerPsr4Mapping {
    prefix: String,
    directories: Vec<PathBuf>,
}

#[derive(Debug, Default)]
pub struct ResolutionContext {
    file_index: HashMap<(PathBuf, String), Vec<SymbolDefinition>>,
    global_index: HashMap<String, Vec<SymbolDefinition>>,
    qualified_index: HashMap<String, Vec<SymbolDefinition>>,
    import_map: HashMap<PathBuf, HashSet<PathBuf>>,
    named_import_map: HashMap<(PathBuf, String), (PathBuf, String)>,
    module_index: HashMap<PathBuf, SymbolDefinition>,
    reference_import_map: HashMap<(PathBuf, usize, String), Vec<PathBuf>>,
    language_map: HashMap<PathBuf, Language>,
}

impl ResolutionContext {
    pub fn from_graph(graph: &SemanticGraph, config: &ResolveConfig) -> Self {
        let mut context = Self::default();

        for file in &graph.files {
            context
                .language_map
                .insert(file.path.clone(), file.language);
        }

        for symbol in &graph.symbols {
            let definition = SymbolDefinition {
                symbol_id: symbol.id.clone(),
                file_path: symbol.file_path.clone(),
                kind: symbol.kind,
                name: symbol.name.clone(),
                qualified_name: symbol.qualified_name.clone(),
                parent_symbol_id: symbol.parent_symbol_id.clone(),
                owner_type_name: symbol.owner_type_name.clone(),
                return_type_name: symbol.return_type_name.clone(),
                parameter_count: symbol.parameter_count,
                required_parameter_count: symbol.required_parameter_count,
            };

            if symbol.kind == SymbolKind::Module {
                context
                    .module_index
                    .insert(symbol.file_path.clone(), definition);
                continue;
            }

            context
                .file_index
                .entry((symbol.file_path.clone(), symbol.name.clone()))
                .or_default()
                .push(definition.clone());
            context
                .global_index
                .entry(symbol.name.clone())
                .or_default()
                .push(definition.clone());
            context
                .qualified_index
                .entry(symbol.qualified_name.clone())
                .or_default()
                .push(definition);
        }

        // Built once: per-reference reconstruction is O(imports x files) and
        // dominated resolve time on large repositories.
        let known_files = graph
            .files
            .iter()
            .map(|file| file.path.clone())
            .collect::<HashSet<_>>();

        for reference in &graph.references {
            if reference.kind != ReferenceKind::Import {
                continue;
            }

            let import_targets =
                resolve_import_paths(reference, &known_files, &context.language_map, config);
            if import_targets.is_empty() {
                continue;
            }

            let import_targets_vec = import_targets.iter().cloned().collect::<Vec<_>>();
            context.reference_import_map.insert(
                (
                    reference.file_path.clone(),
                    reference.line,
                    reference.target_name.clone(),
                ),
                import_targets_vec,
            );

            let import_targets_for_bindings = import_targets.clone();
            context
                .import_map
                .entry(reference.file_path.clone())
                .or_default()
                .extend(import_targets);

            if let Some(binding_name) = &reference.binding_name {
                let exported_name = leaf_symbol_name(&reference.target_name);
                for target_file in import_targets_for_bindings {
                    context.named_import_map.insert(
                        (reference.file_path.clone(), binding_name.clone()),
                        (target_file, exported_name.clone()),
                    );
                }
            }
        }

        context
    }

    pub fn resolve(&self, name: &str, from_file: &Path) -> Option<TieredCandidates> {
        let key = (from_file.to_path_buf(), name.to_owned());
        if let Some(definitions) = self.file_index.get(&key) {
            return Some(TieredCandidates {
                candidates: definitions.clone(),
                tier: ResolutionTier::SameFile,
            });
        }

        if let Some((source_file, exported_name)) = self
            .named_import_map
            .get(&(from_file.to_path_buf(), name.to_owned()))
        {
            let mut named_candidates = self
                .global_index
                .get(exported_name)
                .into_iter()
                .flat_map(|candidates| candidates.iter())
                .filter(|candidate| &candidate.file_path == source_file)
                .cloned()
                .collect::<Vec<_>>();
            if named_candidates.is_empty() {
                if let Some(module_candidate) = self.module_index.get(source_file) {
                    named_candidates.push(module_candidate.clone());
                }
            }
            if !named_candidates.is_empty() {
                return Some(TieredCandidates {
                    candidates: named_candidates,
                    tier: ResolutionTier::ImportScoped,
                });
            }
        }

        let all_candidates = self.global_index.get(name)?.clone();
        if let Some(imported_files) = self.import_map.get(from_file) {
            let imported_candidates = all_candidates
                .iter()
                .filter(|candidate| imported_files.contains(&candidate.file_path))
                .cloned()
                .collect::<Vec<_>>();
            if !imported_candidates.is_empty() {
                return Some(TieredCandidates {
                    candidates: imported_candidates,
                    tier: ResolutionTier::ImportScoped,
                });
            }
        }

        Some(TieredCandidates {
            candidates: all_candidates,
            tier: ResolutionTier::Global,
        })
    }

    fn import_targets_for_reference(&self, reference: &SemanticReference) -> Vec<PathBuf> {
        self.reference_import_map
            .get(&(
                reference.file_path.clone(),
                reference.line,
                reference.target_name.clone(),
            ))
            .cloned()
            .unwrap_or_default()
    }

    fn module_candidates_for_files(&self, files: &[PathBuf]) -> Vec<SymbolDefinition> {
        files
            .iter()
            .filter_map(|file| self.module_index.get(file).cloned())
            .collect()
    }
}

pub fn load_resolve_config(root: &Path) -> ResolveConfig {
    let mut config = ResolveConfig::default();
    config
        .tsconfig_paths
        .extend(load_tsconfig_paths(root, &root.join("tsconfig.json")));
    config
        .tsconfig_paths
        .extend(load_tsconfig_paths(root, &root.join("jsconfig.json")));
    config
        .composer_psr4
        .extend(load_composer_psr4(root, &root.join("composer.json")));
    config.python_roots = discover_python_roots(root);
    config.ruby_load_paths = discover_ruby_load_paths(root);
    config
}

pub fn resolve_graph(graph: &mut SemanticGraph) {
    resolve_graph_with_config(graph, &ResolveConfig::default());
}

pub fn resolve_graph_with_config(graph: &mut SemanticGraph, config: &ResolveConfig) {
    graph.resolved_edges.clear();
    let context = ResolutionContext::from_graph(graph, config);
    let mut occurrence_counters = HashMap::<(PathBuf, usize, String), usize>::new();
    let resolved = graph
        .references
        .iter()
        .filter_map(|reference| {
            let occurrence_key = (
                reference.file_path.clone(),
                reference.line,
                reference.target_name.clone(),
            );
            let occurrence_index = *occurrence_counters
                .entry(occurrence_key)
                .and_modify(|value| *value += 1)
                .or_insert(0);
            resolve_reference(reference, &context).map(|edge| {
                edge.with_reference_identity(reference.target_name.clone(), occurrence_index)
            })
        })
        .collect::<Vec<_>>();

    for edge in resolved {
        graph.add_resolved_edge(edge);
    }
    append_override_edges(graph);
}

fn resolve_reference(
    reference: &SemanticReference,
    context: &ResolutionContext,
) -> Option<ResolvedEdge> {
    if reference.kind == ReferenceKind::Import {
        return resolve_import_reference(reference, context);
    }

    let target_name = leaf_symbol_name(&reference.target_name);
    let mut candidates = context.resolve(&target_name, &reference.file_path)?;
    candidates = filter_candidates(reference, candidates, context);
    pick_edge(reference, candidates)
}

fn resolve_import_reference(
    reference: &SemanticReference,
    context: &ResolutionContext,
) -> Option<ResolvedEdge> {
    let preferred_name = reference
        .binding_name
        .clone()
        .unwrap_or_else(|| leaf_symbol_name(&reference.target_name));
    let import_targets = context.import_targets_for_reference(reference);
    let mut candidates = context
        .resolve(&preferred_name, &reference.file_path)
        .or_else(|| {
            let module_candidates = context.module_candidates_for_files(&import_targets);
            (!module_candidates.is_empty()).then_some(TieredCandidates {
                candidates: module_candidates,
                tier: ResolutionTier::ImportScoped,
            })
        })?;

    if !import_targets.is_empty() {
        let filtered = candidates
            .candidates
            .iter()
            .filter(|candidate| import_targets.contains(&candidate.file_path))
            .cloned()
            .collect::<Vec<_>>();
        if !filtered.is_empty() {
            candidates.candidates = filtered;
        } else {
            let module_candidates = context.module_candidates_for_files(&import_targets);
            if !module_candidates.is_empty() {
                candidates.candidates = module_candidates;
                candidates.tier = ResolutionTier::ImportScoped;
            }
        }
    }

    pick_edge(reference, candidates)
}

fn pick_edge(reference: &SemanticReference, candidates: TieredCandidates) -> Option<ResolvedEdge> {
    if candidates.candidates.is_empty() {
        return None;
    }
    if candidates.tier == ResolutionTier::Global && candidates.candidates.len() != 1 {
        return None;
    }

    let target = candidates.candidates.first()?.clone();
    Some(ResolvedEdge::new(
        reference.file_path.clone(),
        reference.enclosing_symbol_id.clone(),
        target.file_path,
        target.symbol_id,
        reference.kind,
        candidates.tier,
        confidence_millis(candidates.tier),
        resolution_reason(reference.kind, candidates.tier),
        reference.line,
    ))
}

fn filter_candidates(
    reference: &SemanticReference,
    mut candidates: TieredCandidates,
    context: &ResolutionContext,
) -> TieredCandidates {
    if reference.kind == ReferenceKind::Call {
        candidates = prefer_same_language_call_candidates(reference, candidates, context);
        let mut receiver_narrowed = false;
        if matches!(reference.call_form, Some(CallForm::Free)) {
            let free_function_candidates = candidates
                .candidates
                .iter()
                .filter(|candidate| candidate.owner_type_name.is_none())
                .cloned()
                .collect::<Vec<_>>();
            if !free_function_candidates.is_empty() {
                candidates.candidates = free_function_candidates;
            }
        }
        if let Some(receiver_type_name) = &reference.receiver_type_name {
            let receiver_resolution =
                resolve_receiver_type(context, &reference.file_path, receiver_type_name);
            if !receiver_resolution.symbol_ids.is_empty()
                || !receiver_resolution.type_names.is_empty()
                || !receiver_resolution.file_paths.is_empty()
            {
                let direct_owner_filtered = candidates
                    .candidates
                    .iter()
                    .filter(|candidate| {
                        candidate
                            .parent_symbol_id
                            .as_ref()
                            .is_some_and(|parent| receiver_resolution.symbol_ids.contains(parent))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if !direct_owner_filtered.is_empty() {
                    if direct_owner_filtered.len() < candidates.candidates.len() {
                        receiver_narrowed = true;
                    }
                    candidates.candidates = direct_owner_filtered;
                } else {
                    let receiver_filtered = candidates
                        .candidates
                        .iter()
                        .filter(|candidate| {
                            candidate
                                .owner_type_name
                                .as_ref()
                                .is_some_and(|owner_type_name| {
                                    receiver_resolution.type_names.contains(owner_type_name)
                                })
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    if !receiver_filtered.is_empty() {
                        if receiver_filtered.len() < candidates.candidates.len() {
                            receiver_narrowed = true;
                        }
                        candidates.candidates = receiver_filtered;
                    }
                }

                if candidates.candidates.len() > 1 && !receiver_resolution.file_paths.is_empty() {
                    let file_filtered = candidates
                        .candidates
                        .iter()
                        .filter(|candidate| {
                            receiver_resolution
                                .file_paths
                                .contains(&candidate.file_path)
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    if !file_filtered.is_empty() {
                        if file_filtered.len() < candidates.candidates.len() {
                            receiver_narrowed = true;
                        }
                        candidates.candidates = file_filtered;
                    }
                }
            }
        } else if matches!(reference.call_form, Some(CallForm::Associated)) {
            if let Some(receiver_name) = &reference.receiver_name {
                let owner_filtered = candidates
                    .candidates
                    .iter()
                    .filter(|candidate| {
                        candidate.owner_type_name.as_deref() == Some(receiver_name.as_str())
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if !owner_filtered.is_empty() {
                    if owner_filtered.len() < candidates.candidates.len() {
                        receiver_narrowed = true;
                    }
                    candidates.candidates = owner_filtered;
                }
            }
        }

        if let Some(arity) = reference.arity {
            let arity_filtered = candidates
                .candidates
                .iter()
                .filter(|candidate| {
                    candidate.required_parameter_count <= arity
                        && arity <= candidate.parameter_count
                })
                .cloned()
                .collect::<Vec<_>>();
            if !arity_filtered.is_empty() {
                candidates.candidates = arity_filtered;
            }
        }

        if matches!(reference.call_form, Some(CallForm::Member))
            && reference.receiver_type_name.is_none()
            && candidates.tier == ResolutionTier::Global
        {
            candidates.candidates.clear();
        }

        // A call made through an explicit receiver/scope that is *not* a self
        // reference can never resolve to its own enclosing method. Recursion is
        // always written against a self receiver (`$this->m()`, `self::m()`,
        // `static::m()`, Rust `self.m()`), so any other receiver — a collaborator
        // (`$this->schemaCompiler->getSchemaHash()`), a call-chain result
        // (`$this->handler()->findAvailability()`), another variable
        // (`$registry->register()`), or `parent::__construct()` — denotes a
        // different object/class. Same-file name matching would otherwise bind such
        // a call to a same-named enclosing method, fabricating a recursion
        // self-loop. Remove only that self-candidate; genuine recursion (self
        // receiver) and every real cross-method edge are untouched.
        let receiver_is_self = reference
            .receiver_name
            .as_deref()
            .is_some_and(is_self_receiver_name);
        let has_explicit_receiver = reference.receiver_name.is_some()
            && matches!(
                reference.call_form,
                Some(CallForm::Member | CallForm::Associated)
            );
        if has_explicit_receiver && !receiver_is_self {
            if let Some(enclosing) = reference.enclosing_symbol_id.as_deref() {
                candidates
                    .candidates
                    .retain(|candidate| candidate.symbol_id != enclosing);
            }
        }

        if receiver_narrowed && candidates.tier == ResolutionTier::Global {
            candidates.tier = ResolutionTier::ImportScoped;
        }
    }

    candidates
}

/// Whether a member-call receiver expression is a bare self reference, across the
/// languages the resolver serves (`$this` PHP, `this` JS/TS, `self`/`Self` Rust &
/// Python-ish, `static` PHP late static binding). A chained/property receiver such
/// as `$this->collaborator` is deliberately not self — only the exact keyword is.
fn is_self_receiver_name(receiver_name: &str) -> bool {
    matches!(
        receiver_name.trim(),
        "$this" | "this" | "self" | "Self" | "static"
    )
}

fn prefer_same_language_call_candidates(
    reference: &SemanticReference,
    mut candidates: TieredCandidates,
    context: &ResolutionContext,
) -> TieredCandidates {
    let Some(source_language) = context.language_map.get(&reference.file_path).copied() else {
        return candidates;
    };
    let same_language_candidates = candidates
        .candidates
        .iter()
        .filter(|candidate| {
            context
                .language_map
                .get(&candidate.file_path)
                .copied()
                .is_some_and(|candidate_language| candidate_language == source_language)
        })
        .cloned()
        .collect::<Vec<_>>();
    if !same_language_candidates.is_empty()
        && same_language_candidates.len() < candidates.candidates.len()
    {
        candidates.candidates = same_language_candidates;
        if candidates.tier == ResolutionTier::Global {
            candidates.tier = ResolutionTier::ImportScoped;
        }
    }
    candidates
}

fn resolve_receiver_type(
    context: &ResolutionContext,
    from_file: &Path,
    receiver_type_name: &str,
) -> ReceiverResolution {
    let mut resolution = ReceiverResolution::default();
    let mut visited = HashSet::new();
    collect_receiver_type(
        context,
        from_file,
        receiver_type_name,
        &mut visited,
        &mut resolution,
    );
    resolution
}

fn collect_receiver_type(
    context: &ResolutionContext,
    from_file: &Path,
    receiver_type_name: &str,
    visited: &mut HashSet<(PathBuf, String)>,
    resolution: &mut ReceiverResolution,
) {
    for candidate_name in extract_candidate_type_names(receiver_type_name) {
        let visit_key = (from_file.to_path_buf(), candidate_name.clone());
        if !visited.insert(visit_key) {
            continue;
        }

        resolution
            .type_names
            .insert(leaf_symbol_name(&candidate_name));

        let candidates = resolve_receiver_candidates(context, from_file, &candidate_name);
        for candidate in candidates {
            match candidate.kind {
                SymbolKind::Class
                | SymbolKind::Struct
                | SymbolKind::Interface
                | SymbolKind::Enum
                | SymbolKind::Trait => {
                    resolution.symbol_ids.insert(candidate.symbol_id.clone());
                    resolution.file_paths.insert(candidate.file_path.clone());
                    resolution.type_names.insert(candidate.name.clone());
                    resolution
                        .type_names
                        .insert(leaf_symbol_name(&candidate.qualified_name));
                }
                _ => {}
            }

            if let Some(return_type_name) = &candidate.return_type_name {
                if return_type_name.trim() != candidate_name {
                    collect_receiver_type(
                        context,
                        &candidate.file_path,
                        return_type_name,
                        visited,
                        resolution,
                    );
                }
            }
        }
    }
}

fn resolve_receiver_candidates(
    context: &ResolutionContext,
    from_file: &Path,
    receiver_type_name: &str,
) -> Vec<SymbolDefinition> {
    if receiver_type_name.contains("::") {
        let normalized = normalize_qualified_receiver_name(context, from_file, receiver_type_name);
        if let Some(candidates) = context.qualified_index.get(&normalized) {
            return candidates.clone();
        }
    }

    let normalized_name = leaf_symbol_name(receiver_type_name);
    context
        .resolve(&normalized_name, from_file)
        .map(|tiered| tiered.candidates)
        .unwrap_or_default()
}

fn normalize_qualified_receiver_name(
    context: &ResolutionContext,
    from_file: &Path,
    receiver_type_name: &str,
) -> String {
    let Some((owner, member)) = receiver_type_name.split_once("::") else {
        return receiver_type_name.to_owned();
    };
    let normalized_owner = context
        .named_import_map
        .get(&(from_file.to_path_buf(), owner.to_owned()))
        .map(|(_, exported_name)| exported_name.clone())
        .unwrap_or_else(|| leaf_symbol_name(owner));
    format!("{normalized_owner}::{member}")
}

fn extract_candidate_type_names(type_expression: &str) -> Vec<String> {
    let qualified = type_expression.trim();
    if qualified.is_empty() {
        return Vec::new();
    }
    if qualified.contains("::") && !qualified.contains('<') && !qualified.contains('[') {
        return vec![qualified.to_owned()];
    }

    static TYPE_TOKEN_REGEX: OnceLock<Regex> = OnceLock::new();
    let matcher = TYPE_TOKEN_REGEX.get_or_init(|| {
        Regex::new(r"[A-Za-z_\\][A-Za-z0-9_:\\\\]*").expect("valid type token regex")
    });
    let ignored = [
        "Promise",
        "Option",
        "Optional",
        "Result",
        "Array",
        "Vec",
        "List",
        "Dict",
        "Map",
        "Set",
        "Tuple",
        "Iterable",
        "Iterator",
        "Sequence",
        "Mapping",
        "Union",
        "Literal",
        "Awaitable",
        "null",
        "undefined",
        "None",
        "void",
        "never",
        "mixed",
        "any",
        "unknown",
        "string",
        "number",
        "boolean",
        "bool",
        "int",
        "float",
        "str",
        "array",
        "object",
        "callable",
        "resource",
        "true",
        "false",
    ]
    .into_iter()
    .collect::<HashSet<_>>();

    let mut preferred = Vec::new();
    let mut fallback = Vec::new();
    let mut seen = HashSet::new();

    for raw_match in matcher.find_iter(type_expression) {
        let token = raw_match.as_str().trim_start_matches('\\');
        if token.is_empty() {
            continue;
        }
        let leaf = leaf_symbol_name(token);
        if ignored.contains(leaf.as_str()) {
            continue;
        }
        if !seen.insert(token.to_owned()) {
            continue;
        }
        if leaf
            .chars()
            .next()
            .is_some_and(|character| character.is_uppercase())
        {
            preferred.push(token.to_owned());
        } else {
            fallback.push(token.to_owned());
        }
    }

    preferred.extend(fallback);
    if preferred.is_empty() {
        vec![leaf_symbol_name(type_expression)]
    } else {
        preferred
    }
}

fn append_override_edges(graph: &mut SemanticGraph) {
    let extends_by_child = graph
        .resolved_edges
        .iter()
        .filter(|edge| edge.kind == ReferenceKind::Extends)
        .filter_map(|edge| {
            edge.source_symbol_id
                .as_ref()
                .map(|source_id| (source_id.clone(), edge.target_symbol_id.clone()))
        })
        .fold(
            HashMap::<String, Vec<String>>::new(),
            |mut acc, (child, parent)| {
                acc.entry(child).or_default().push(parent);
                acc
            },
        );

    if extends_by_child.is_empty() {
        return;
    }

    let class_symbols = graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Class)
        .map(|symbol| (symbol.id.clone(), symbol))
        .collect::<HashMap<_, _>>();
    let file_languages = graph
        .files
        .iter()
        .map(|file| (file.path.clone(), file.language))
        .collect::<HashMap<_, _>>();
    let methods_by_class = graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Method)
        .filter_map(|symbol| {
            symbol
                .parent_symbol_id
                .as_ref()
                .map(|parent_id| (parent_id.clone(), symbol))
        })
        .fold(
            HashMap::<String, Vec<_>>::new(),
            |mut acc, (parent_id, symbol)| {
                acc.entry(parent_id).or_default().push(symbol);
                acc
            },
        );

    let existing_overrides = graph
        .resolved_edges
        .iter()
        .filter(|edge| edge.kind == ReferenceKind::Overrides)
        .map(|edge| {
            (
                edge.source_symbol_id.clone().unwrap_or_default(),
                edge.target_symbol_id.clone(),
            )
        })
        .collect::<HashSet<_>>();

    let override_edges = graph
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Method)
        .filter_map(|method| {
            let parent_class_id = method.parent_symbol_id.as_ref()?;
            let class_symbol = class_symbols.get(parent_class_id)?;
            let language = file_languages
                .get(&class_symbol.file_path)
                .copied()
                .unwrap_or(Language::Rust);
            let ancestor_method = find_overridden_method(
                parent_class_id,
                &method.name,
                language,
                &extends_by_child,
                &methods_by_class,
            )?;
            let key = (method.id.clone(), ancestor_method.id.clone());
            (!existing_overrides.contains(&key)).then_some(ResolvedEdge::new(
                method.file_path.clone(),
                Some(method.id.clone()),
                ancestor_method.file_path.clone(),
                ancestor_method.id.clone(),
                ReferenceKind::Overrides,
                ResolutionTier::ImportScoped,
                950,
                format!(
                    "override:{}::{}->{}",
                    class_symbol.name, method.name, ancestor_method.qualified_name
                ),
                method.start_line,
            ))
        })
        .collect::<Vec<_>>();

    for edge in override_edges {
        graph.add_resolved_edge(edge);
    }
}

fn find_overridden_method<'a>(
    class_id: &str,
    method_name: &str,
    language: Language,
    extends_by_child: &HashMap<String, Vec<String>>,
    methods_by_class: &'a HashMap<String, Vec<&'a crate::graph::SymbolNode>>,
) -> Option<&'a crate::graph::SymbolNode> {
    let ancestor_order = ancestor_search_order(class_id, language, extends_by_child);

    for candidate_class_id in ancestor_order {
        if let Some(methods) = methods_by_class.get(&candidate_class_id) {
            if let Some(method) = methods.iter().find(|method| method.name == method_name) {
                return Some(method);
            }
        }
    }

    None
}

fn ancestor_search_order(
    class_id: &str,
    language: Language,
    extends_by_child: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    if language == Language::Python {
        let mut cache = HashMap::new();
        if let Some(mro) = c3_linearize(class_id, extends_by_child, &mut cache) {
            return mro;
        }
    }
    gather_ancestors_breadth_first(class_id, extends_by_child)
}

fn gather_ancestors_breadth_first(
    class_id: &str,
    extends_by_child: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let mut order = Vec::new();
    let mut queue = VecDeque::from(extends_by_child.get(class_id).cloned().unwrap_or_default());
    let mut visited = HashSet::new();

    while let Some(candidate_class_id) = queue.pop_front() {
        if !visited.insert(candidate_class_id.clone()) {
            continue;
        }
        order.push(candidate_class_id.clone());
        if let Some(parents) = extends_by_child.get(&candidate_class_id) {
            queue.extend(parents.iter().cloned());
        }
    }

    order
}

fn c3_linearize(
    class_id: &str,
    extends_by_child: &HashMap<String, Vec<String>>,
    cache: &mut HashMap<String, Option<Vec<String>>>,
) -> Option<Vec<String>> {
    if let Some(cached) = cache.get(class_id) {
        return cached.clone();
    }

    let direct_parents = extends_by_child.get(class_id).cloned().unwrap_or_default();
    if direct_parents.is_empty() {
        cache.insert(class_id.to_owned(), Some(Vec::new()));
        return Some(Vec::new());
    }

    let mut sequences = Vec::new();
    for parent_id in &direct_parents {
        let mut parent_linearization = vec![parent_id.clone()];
        parent_linearization.extend(c3_linearize(parent_id, extends_by_child, cache)?);
        sequences.push(parent_linearization);
    }
    sequences.push(direct_parents.clone());

    let mut result = Vec::new();
    while sequences.iter().any(|sequence| !sequence.is_empty()) {
        let candidate = sequences
            .iter()
            .filter(|sequence| !sequence.is_empty())
            .find_map(|sequence| {
                let head = &sequence[0];
                let in_tail = sequences
                    .iter()
                    .any(|other| other.len() > 1 && other.iter().skip(1).any(|item| item == head));
                (!in_tail).then_some(head.clone())
            });
        let Some(candidate) = candidate else {
            cache.insert(class_id.to_owned(), None);
            return None;
        };

        result.push(candidate.clone());
        for sequence in &mut sequences {
            if sequence.first() == Some(&candidate) {
                sequence.remove(0);
            }
        }
    }

    cache.insert(class_id.to_owned(), Some(result.clone()));
    Some(result)
}

fn confidence_millis(tier: ResolutionTier) -> u16 {
    match tier {
        ResolutionTier::SameFile => 950,
        ResolutionTier::ImportScoped => 900,
        ResolutionTier::Global => 500,
    }
}

fn resolution_reason(kind: ReferenceKind, tier: ResolutionTier) -> String {
    let base = match kind {
        ReferenceKind::Import => "import",
        ReferenceKind::Call => "call",
        ReferenceKind::Type => "type",
        ReferenceKind::Extends => "extends",
        ReferenceKind::Implements => "implements",
        ReferenceKind::Overrides => "overrides",
    };
    let tier_name = match tier {
        ResolutionTier::SameFile => "same-file",
        ResolutionTier::ImportScoped => "import-scoped",
        ResolutionTier::Global => "global",
    };
    format!("{base}:{tier_name}")
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

fn resolve_import_paths(
    reference: &SemanticReference,
    known_files: &HashSet<PathBuf>,
    language_map: &HashMap<PathBuf, Language>,
    config: &ResolveConfig,
) -> HashSet<PathBuf> {
    let Some(language) = language_map
        .get(&reference.file_path)
        .copied()
        .or_else(|| infer_language(&reference.file_path))
    else {
        return HashSet::new();
    };

    match language {
        Language::JavaScript | Language::TypeScript => resolve_javascript_import_paths(
            &reference.file_path,
            &reference.target_name,
            &known_files,
            config,
        ),
        Language::Php => resolve_php_import_paths(&reference.target_name, &known_files, config),
        Language::Python => resolve_python_import_paths(
            &reference.file_path,
            &reference.target_name,
            &known_files,
            config,
        ),
        Language::Ruby => resolve_ruby_import_paths(
            &reference.file_path,
            &reference.target_name,
            &known_files,
            config,
        ),
        Language::Rust => {
            resolve_rust_import_paths(&reference.file_path, &reference.target_name, &known_files)
        }
    }
}

fn resolve_rust_import_paths(
    from_file: &Path,
    import_target: &str,
    known_files: &HashSet<PathBuf>,
) -> HashSet<PathBuf> {
    let normalized_target = import_target.trim_matches('{').trim_matches('}').trim();
    let segments = normalized_target
        .split("::")
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return HashSet::new();
    }

    let anchored = normalize_rust_import_segments(from_file, &segments);
    let candidate_prefixes = if anchored.is_empty() {
        Vec::new()
    } else {
        let mut prefixes = Vec::new();
        for len in (1..=anchored.len()).rev() {
            prefixes.push(anchored[..len].join("/"));
        }
        prefixes
    };

    let mut resolved = HashSet::new();
    for prefix in candidate_prefixes {
        for suffix in [".rs", "/mod.rs"] {
            let candidate = PathBuf::from(format!("src/{prefix}{suffix}"));
            if known_files.contains(&candidate) {
                resolved.insert(candidate);
            }
        }
    }
    resolved
}

fn resolve_javascript_import_paths(
    from_file: &Path,
    import_target: &str,
    known_files: &HashSet<PathBuf>,
    config: &ResolveConfig,
) -> HashSet<PathBuf> {
    let module_target = import_target
        .split("::")
        .next()
        .unwrap_or(import_target)
        .trim();
    let mut candidates = Vec::new();

    if module_target.starts_with("./") || module_target.starts_with("../") {
        let base = from_file.parent().unwrap_or_else(|| Path::new(""));
        let normalized = normalize_relative_path(&base.join(module_target));
        candidates.extend(javascript_candidate_paths(&normalized));
    } else if module_target.starts_with('/') {
        let normalized = normalize_relative_path(Path::new(module_target.trim_start_matches('/')));
        candidates.extend(javascript_candidate_paths(&normalized));
    } else {
        let config_matches = resolve_tsconfig_path_alias(module_target, known_files, config);
        if !config_matches.is_empty() {
            return config_matches;
        }
        let normalized = normalize_relative_path(Path::new(module_target));
        candidates.extend(javascript_candidate_paths(&normalized));
        candidates.extend(javascript_candidate_paths(&PathBuf::from(format!(
            "src/{module_target}"
        ))));
    }

    match_candidates(candidates, known_files)
}

fn javascript_candidate_paths(base: &Path) -> Vec<PathBuf> {
    if base.extension().is_some() {
        return vec![normalize_relative_path(base)];
    }

    let mut candidates = Vec::new();
    for extension in ["js", "jsx", "ts", "tsx", "vue"] {
        candidates.push(normalize_relative_path(&base.with_extension(extension)));
        candidates.push(normalize_relative_path(
            &base.join(format!("index.{extension}")),
        ));
    }
    candidates
}

fn resolve_python_import_paths(
    from_file: &Path,
    import_target: &str,
    known_files: &HashSet<PathBuf>,
    config: &ResolveConfig,
) -> HashSet<PathBuf> {
    let module_target = import_target
        .split("::")
        .next()
        .unwrap_or(import_target)
        .trim();
    let leading_dots = module_target.chars().take_while(|ch| *ch == '.').count();
    let remainder = module_target.trim_start_matches('.');

    let mut base = from_file
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf();
    for _ in 1..leading_dots {
        if !base.pop() {
            break;
        }
    }

    let relative = if leading_dots > 0 {
        base
    } else {
        PathBuf::new()
    };
    let mut segments = Vec::new();
    if !remainder.is_empty() {
        segments.extend(remainder.split('.').map(str::to_owned));
    }

    let mut candidates = Vec::new();
    if leading_dots > 0 {
        let mut path = relative;
        for segment in &segments {
            path.push(segment);
        }
        candidates.extend(python_candidate_paths(&path));
    } else {
        for root in &config.python_roots {
            let mut path = root.clone();
            for segment in &segments {
                path.push(segment);
            }
            candidates.extend(python_candidate_paths(&path));
        }
        if config.python_roots.is_empty() {
            let mut path = PathBuf::new();
            for segment in &segments {
                path.push(segment);
            }
            candidates.extend(python_candidate_paths(&path));
        }
    }
    match_candidates(candidates, known_files)
}

fn python_candidate_paths(base: &Path) -> Vec<PathBuf> {
    let normalized = normalize_relative_path(base);
    if normalized.extension() == Some(OsStr::new("py")) {
        return vec![normalized];
    }
    if normalized.as_os_str().is_empty() {
        return Vec::new();
    }
    vec![
        normalized.with_extension("py"),
        normalized.join("__init__.py"),
    ]
}

fn resolve_php_import_paths(
    import_target: &str,
    known_files: &HashSet<PathBuf>,
    config: &ResolveConfig,
) -> HashSet<PathBuf> {
    let normalized = import_target.trim_start_matches('\\').replace('\\', "/");
    let config_matches = resolve_composer_psr4_import(&normalized, known_files, config);
    if !config_matches.is_empty() {
        return config_matches;
    }
    let mut candidates = vec![PathBuf::from(format!("{normalized}.php"))];
    let segments = normalized.split('/').collect::<Vec<_>>();
    for start in 1..segments.len() {
        candidates.push(PathBuf::from(format!(
            "{}.php",
            segments[start..].join("/")
        )));
    }
    let mut resolved = match_candidates(candidates, known_files);
    if resolved.is_empty() {
        resolved.extend(suffix_match_case_insensitive(
            &format!("{normalized}.php"),
            known_files,
        ));
        for start in 1..segments.len() {
            resolved.extend(suffix_match_case_insensitive(
                &format!("{}.php", segments[start..].join("/")),
                known_files,
            ));
        }
    }
    resolved
}

fn resolve_ruby_import_paths(
    from_file: &Path,
    import_target: &str,
    known_files: &HashSet<PathBuf>,
    config: &ResolveConfig,
) -> HashSet<PathBuf> {
    let mut candidates = Vec::new();
    if import_target.starts_with("./") || import_target.starts_with("../") {
        let base = from_file.parent().unwrap_or_else(|| Path::new(""));
        let normalized = normalize_relative_path(&base.join(import_target));
        candidates.push(normalized.with_extension("rb"));
        candidates.push(normalized.join("init.rb"));
    } else {
        for root in &config.ruby_load_paths {
            candidates.push(root.join(format!("{import_target}.rb")));
            candidates.push(root.join(import_target).join("init.rb"));
        }
        if config.ruby_load_paths.is_empty() {
            candidates.push(PathBuf::from(format!("{import_target}.rb")));
        }
    }

    let mut resolved = match_candidates(candidates, known_files);
    if resolved.is_empty() {
        resolved.extend(suffix_match(&format!("{import_target}.rb"), known_files));
    }
    resolved
}

fn normalize_rust_import_segments(from_file: &Path, segments: &[&str]) -> Vec<String> {
    match segments.first().copied() {
        Some("crate") => segments[1..]
            .iter()
            .map(|segment| (*segment).to_owned())
            .collect(),
        Some("self") => {
            let mut base = rust_module_segments_for_file(from_file);
            base.extend(segments[1..].iter().map(|segment| (*segment).to_owned()));
            base
        }
        Some("super") => {
            let mut base = rust_module_segments_for_file(from_file);
            if !base.is_empty() {
                base.pop();
            }
            base.extend(segments[1..].iter().map(|segment| (*segment).to_owned()));
            base
        }
        _ => segments
            .iter()
            .map(|segment| (*segment).to_owned())
            .collect(),
    }
}

fn rust_module_segments_for_file(file_path: &Path) -> Vec<String> {
    let mut segments = file_path
        .iter()
        .map(|segment| segment.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    if matches!(segments.first().map(String::as_str), Some("src")) {
        segments.remove(0);
    }

    match segments.last().map(String::as_str) {
        Some("lib.rs") | Some("main.rs") | Some("mod.rs") => {
            segments.pop();
        }
        Some(last) if last.ends_with(".rs") => {
            let stem = last.trim_end_matches(".rs").to_owned();
            segments.pop();
            segments.push(stem);
        }
        _ => {}
    }

    segments
}

fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(segment) => normalized.push(segment),
            _ => {}
        }
    }
    normalized
}

fn relativize_to_root(path: &Path, root: &Path) -> PathBuf {
    let relative = path.strip_prefix(root).unwrap_or(path);
    normalize_relative_path(relative)
}

fn match_candidates(candidates: Vec<PathBuf>, known_files: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    candidates
        .into_iter()
        .map(|path| normalize_relative_path(&path))
        .filter(|path| known_files.contains(path))
        .collect()
}

fn suffix_match(suffix: &str, known_files: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    let normalized_suffix = suffix.replace('\\', "/");
    known_files
        .iter()
        .filter(|path| {
            let normalized = path.to_string_lossy().replace('\\', "/");
            normalized == normalized_suffix
                || normalized.ends_with(&format!("/{normalized_suffix}"))
        })
        .cloned()
        .collect()
}

fn suffix_match_case_insensitive(suffix: &str, known_files: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    let normalized_suffix = suffix.replace('\\', "/").to_lowercase();
    known_files
        .iter()
        .filter(|path| {
            let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
            normalized == normalized_suffix
                || normalized.ends_with(&format!("/{normalized_suffix}"))
        })
        .cloned()
        .collect()
}

fn load_tsconfig_paths(root: &Path, path: &Path) -> Vec<TsPathAlias> {
    let Ok(source) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<Value>(&source) else {
        return Vec::new();
    };
    let Some(compiler_options) = json.get("compilerOptions") else {
        return Vec::new();
    };
    let Some(paths) = compiler_options.get("paths").and_then(Value::as_object) else {
        return Vec::new();
    };

    let config_dir = path.parent().unwrap_or_else(|| Path::new(""));
    let base_dir = compiler_options
        .get("baseUrl")
        .and_then(Value::as_str)
        .map(|base_url| relativize_to_root(&config_dir.join(base_url), root))
        .unwrap_or_else(|| relativize_to_root(config_dir, root));

    paths
        .iter()
        .filter_map(|(pattern, targets)| {
            let target_values = match targets {
                Value::Array(values) => values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
                Value::String(value) => vec![value.clone()],
                _ => Vec::new(),
            };
            (!target_values.is_empty()).then_some(TsPathAlias {
                pattern: pattern.clone(),
                targets: target_values,
                base_dir: base_dir.clone(),
            })
        })
        .collect()
}

fn load_composer_psr4(root: &Path, path: &Path) -> Vec<ComposerPsr4Mapping> {
    let Ok(source) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<Value>(&source) else {
        return Vec::new();
    };
    let Some(psr4) = json
        .get("autoload")
        .and_then(|autoload| autoload.get("psr-4"))
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };

    let root_dir = path.parent().unwrap_or_else(|| Path::new(""));
    let mut mappings = psr4
        .iter()
        .filter_map(|(prefix, directories)| {
            let dirs = match directories {
                Value::Array(values) => values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(|value| relativize_to_root(&root_dir.join(value), root))
                    .collect::<Vec<_>>(),
                Value::String(value) => vec![relativize_to_root(&root_dir.join(value), root)],
                _ => Vec::new(),
            };
            (!dirs.is_empty()).then_some(ComposerPsr4Mapping {
                prefix: prefix.clone(),
                directories: dirs,
            })
        })
        .collect::<Vec<_>>();
    mappings.sort_by(|left, right| right.prefix.len().cmp(&left.prefix.len()));
    mappings
}

fn resolve_tsconfig_path_alias(
    import_target: &str,
    known_files: &HashSet<PathBuf>,
    config: &ResolveConfig,
) -> HashSet<PathBuf> {
    for alias in &config.tsconfig_paths {
        if let Some(wildcard) = match_ts_path_pattern(&alias.pattern, import_target) {
            let candidates = alias
                .targets
                .iter()
                .flat_map(|target| {
                    let substituted = apply_ts_path_target(target, wildcard);
                    javascript_candidate_paths(&normalize_relative_path(
                        &alias.base_dir.join(substituted),
                    ))
                })
                .collect::<Vec<_>>();
            let resolved = match_candidates(candidates, known_files);
            if !resolved.is_empty() {
                return resolved;
            }
        }
    }
    HashSet::new()
}

fn match_ts_path_pattern<'a>(pattern: &'a str, import_target: &'a str) -> Option<Option<&'a str>> {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        if import_target.starts_with(prefix) && import_target.ends_with(suffix) {
            let wildcard = &import_target[prefix.len()..import_target.len() - suffix.len()];
            return Some(Some(wildcard));
        }
        return None;
    }
    (pattern == import_target).then_some(None)
}

fn apply_ts_path_target(target: &str, wildcard: Option<&str>) -> String {
    match wildcard {
        Some(wildcard) => target.replace('*', wildcard),
        None => target.to_owned(),
    }
}

fn resolve_composer_psr4_import(
    normalized_import_target: &str,
    known_files: &HashSet<PathBuf>,
    config: &ResolveConfig,
) -> HashSet<PathBuf> {
    let namespaced = normalized_import_target.replace('/', "\\");
    for mapping in &config.composer_psr4 {
        if !namespaced.starts_with(&mapping.prefix) {
            continue;
        }
        let remainder = namespaced[mapping.prefix.len()..]
            .trim_start_matches('\\')
            .replace('\\', "/");
        let candidates = mapping
            .directories
            .iter()
            .map(|directory| normalize_relative_path(&directory.join(format!("{remainder}.php"))))
            .collect::<Vec<_>>();
        let resolved = match_candidates(candidates, known_files);
        if !resolved.is_empty() {
            return resolved;
        }
    }
    HashSet::new()
}

fn discover_python_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::new()];
    for candidate in ["src", "app"] {
        if root.join(candidate).is_dir() {
            roots.push(PathBuf::from(candidate));
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn discover_ruby_load_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for candidate in ["lib", "app"] {
        if root.join(candidate).is_dir() {
            paths.push(PathBuf::from(candidate));
        }
    }
    paths
}

fn infer_language(path: &Path) -> Option<Language> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("js" | "jsx") => Some(Language::JavaScript),
        Some("ts" | "tsx" | "vue") => Some(Language::TypeScript),
        Some("php" | "phtml" | "php3" | "php4" | "php5" | "php8") => Some(Language::Php),
        Some("py") => Some(Language::Python),
        Some("rb" | "rake") => Some(Language::Ruby),
        Some("rs") => Some(Language::Rust),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        load_resolve_config, resolve_graph, resolve_graph_with_config, ComposerPsr4Mapping,
        ResolutionContext, ResolutionTier, ResolveConfig, TsPathAlias,
    };
    use crate::graph::{
        CallForm, FileNode, Language, ReferenceKind, SemanticGraph, SemanticReference, SymbolKind,
        SymbolNode, Visibility,
    };
    use crate::parsing::javascript::parse_javascript_to_graph;
    use crate::parsing::php::parse_php_to_graph;
    use crate::parsing::python::parse_python_to_graph;
    use crate::parsing::ruby::parse_ruby_to_graph;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolves_same_file_before_import_scoped_and_global() {
        let graph = fixture_graph();
        let context = ResolutionContext::from_graph(&graph, &ResolveConfig::default());

        let resolved = context
            .resolve("helper", PathBuf::from("src/main.rs").as_path())
            .unwrap();

        assert_eq!(resolved.tier, ResolutionTier::SameFile);
        assert_eq!(resolved.candidates.len(), 1);
        assert_eq!(
            resolved.candidates[0].file_path,
            PathBuf::from("src/main.rs")
        );
    }

    #[test]
    fn resolves_import_scoped_symbol_from_use_target() {
        let graph = fixture_graph();
        let context = ResolutionContext::from_graph(&graph, &ResolveConfig::default());

        let resolved = context
            .resolve("User", PathBuf::from("src/main.rs").as_path())
            .unwrap();

        assert_eq!(resolved.tier, ResolutionTier::ImportScoped);
        assert_eq!(resolved.candidates.len(), 1);
        assert_eq!(
            resolved.candidates[0].file_path,
            PathBuf::from("src/models.rs")
        );
    }

    #[test]
    fn resolves_references_into_edges_and_refuses_ambiguous_global() {
        let mut graph = fixture_graph();
        graph.symbols.push(SymbolNode {
            id: String::from("function:src/other.rs:helper"),
            file_path: PathBuf::from("src/other.rs"),
            kind: SymbolKind::Function,
            name: String::from("helper"),
            qualified_name: String::from("helper"),
            parent_symbol_id: None,
            owner_type_name: None,
            return_type_name: None,
            visibility: Visibility::Private,
            parameter_count: 0,
            required_parameter_count: 0,
            start_line: 1,
            end_line: 1,
        });

        resolve_graph(&mut graph);

        assert!(graph.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("src/models.rs")
                && edge.resolution_tier == ResolutionTier::ImportScoped
        }));
        assert!(graph.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("src/main.rs")
                && edge.resolution_tier == ResolutionTier::SameFile
        }));
        assert!(!graph
            .resolved_edges
            .iter()
            .any(|edge| edge.target_symbol_id == "function:src/other.rs:helper"));
    }

    #[test]
    fn drops_non_self_receiver_call_self_loops_but_keeps_recursion() {
        let mut graph = SemanticGraph::default();
        graph.symbols.push(SymbolNode {
            id: String::from("method:app/Provider.php:Provider:getHash"),
            file_path: PathBuf::from("app/Provider.php"),
            kind: SymbolKind::Method,
            name: String::from("getHash"),
            qualified_name: String::from("Provider::getHash"),
            parent_symbol_id: Some(String::from("class:app/Provider.php:Provider")),
            owner_type_name: Some(String::from("Provider")),
            return_type_name: None,
            visibility: Visibility::Private,
            parameter_count: 0,
            required_parameter_count: 0,
            start_line: 1,
            end_line: 8,
        });
        let self_id = "method:app/Provider.php:Provider:getHash";
        // False self-loop: `$this->compiler->getHash()` inside getHash() — the
        // receiver is a collaborator whose type is unknown, not the enclosing class.
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("app/Provider.php"),
            enclosing_symbol_id: Some(String::from(self_id)),
            kind: ReferenceKind::Call,
            target_name: String::from("getHash"),
            binding_name: None,
            line: 3,
            arity: Some(0),
            receiver_name: Some(String::from("$this->compiler")),
            receiver_type_name: None,
            call_form: Some(CallForm::Member),
        });
        // Real recursion: `$this->getHash()` inside getHash().
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("app/Provider.php"),
            enclosing_symbol_id: Some(String::from(self_id)),
            kind: ReferenceKind::Call,
            target_name: String::from("getHash"),
            binding_name: None,
            line: 5,
            arity: Some(0),
            receiver_name: Some(String::from("$this")),
            receiver_type_name: None,
            call_form: Some(CallForm::Member),
        });

        resolve_graph(&mut graph);

        let self_loops = graph
            .resolved_edges
            .iter()
            .filter(|edge| {
                edge.source_symbol_id.as_deref() == Some(self_id)
                    && edge.target_symbol_id == self_id
            })
            .collect::<Vec<_>>();
        assert_eq!(
            self_loops.len(),
            1,
            "only the $this recursion should survive as a self-loop, got {self_loops:?}"
        );
        assert_eq!(self_loops[0].line, 5, "the surviving self-loop is the recursion");
    }

    #[test]
    fn resolves_member_calls_by_receiver_type_and_arity() {
        let mut graph = SemanticGraph::default();
        graph.symbols.push(SymbolNode {
            id: String::from("method:src/user.rs:save"),
            file_path: PathBuf::from("src/user.rs"),
            kind: SymbolKind::Method,
            name: String::from("save"),
            qualified_name: String::from("User::save"),
            parent_symbol_id: Some(String::from("struct:src/user.rs:User")),
            owner_type_name: Some(String::from("User")),
            return_type_name: None,
            visibility: Visibility::Private,
            parameter_count: 1,
            required_parameter_count: 1,
            start_line: 1,
            end_line: 1,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("method:src/repo.rs:save"),
            file_path: PathBuf::from("src/repo.rs"),
            kind: SymbolKind::Method,
            name: String::from("save"),
            qualified_name: String::from("Repo::save"),
            parent_symbol_id: Some(String::from("struct:src/repo.rs:Repo")),
            owner_type_name: Some(String::from("Repo")),
            return_type_name: None,
            visibility: Visibility::Private,
            parameter_count: 0,
            required_parameter_count: 0,
            start_line: 1,
            end_line: 1,
        });
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("src/main.rs"),
            enclosing_symbol_id: Some(String::from("function:src/main.rs:process")),
            kind: ReferenceKind::Call,
            target_name: String::from("save"),
            binding_name: None,
            line: 2,
            arity: Some(1),
            receiver_name: Some(String::from("user")),
            receiver_type_name: Some(String::from("User")),
            call_form: Some(CallForm::Member),
        });

        resolve_graph(&mut graph);

        assert_eq!(graph.resolved_edges.len(), 1);
        assert_eq!(
            graph.resolved_edges[0].target_file_path,
            PathBuf::from("src/user.rs")
        );
        assert_eq!(
            graph.resolved_edges[0].target_symbol_id,
            "method:src/user.rs:save"
        );
    }

    #[test]
    fn resolves_calls_when_arity_matches_optional_parameters() {
        let mut graph = SemanticGraph::default();
        graph.files.push(crate::graph::FileNode {
            path: PathBuf::from("src/main.rs"),
            language: Language::Rust,
        });
        graph.files.push(crate::graph::FileNode {
            path: PathBuf::from("src/i18n.rs"),
            language: Language::Rust,
        });
        graph.files.push(crate::graph::FileNode {
            path: PathBuf::from("src/theme.rs"),
            language: Language::Rust,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("function:src/i18n.rs:translate"),
            file_path: PathBuf::from("src/i18n.rs"),
            kind: SymbolKind::Function,
            name: String::from("translate"),
            qualified_name: String::from("translate"),
            parent_symbol_id: None,
            owner_type_name: None,
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 2,
            required_parameter_count: 1,
            start_line: 1,
            end_line: 1,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("method:src/theme.rs:translate"),
            file_path: PathBuf::from("src/theme.rs"),
            kind: SymbolKind::Method,
            name: String::from("translate"),
            qualified_name: String::from("Theme::translate"),
            parent_symbol_id: Some(String::from("class:src/theme.rs:Theme")),
            owner_type_name: Some(String::from("Theme")),
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 1,
            required_parameter_count: 1,
            start_line: 1,
            end_line: 1,
        });
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("src/main.rs"),
            enclosing_symbol_id: Some(String::from("function:src/main.rs:run")),
            kind: ReferenceKind::Call,
            target_name: String::from("translate"),
            binding_name: None,
            line: 12,
            arity: Some(1),
            receiver_name: None,
            receiver_type_name: None,
            call_form: Some(CallForm::Free),
        });

        resolve_graph(&mut graph);

        assert_eq!(graph.resolved_edges.len(), 1);
        assert_eq!(
            graph.resolved_edges[0].target_symbol_id,
            "function:src/i18n.rs:translate"
        );
    }

    #[test]
    fn prefers_same_language_call_targets_over_cross_language_global_matches() {
        let mut graph = SemanticGraph::default();
        graph.files.push(crate::graph::FileNode {
            path: PathBuf::from("Gruntfile.js"),
            language: Language::JavaScript,
        });
        graph.files.push(crate::graph::FileNode {
            path: PathBuf::from("src/js/config.js"),
            language: Language::JavaScript,
        });
        graph.files.push(crate::graph::FileNode {
            path: PathBuf::from("src/wp-includes/interactivity-api/class-wp-interactivity-api.php"),
            language: Language::Php,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("function:src/js/config.js:config"),
            file_path: PathBuf::from("src/js/config.js"),
            kind: SymbolKind::Function,
            name: String::from("config"),
            qualified_name: String::from("config"),
            parent_symbol_id: None,
            owner_type_name: None,
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 1,
            required_parameter_count: 1,
            start_line: 1,
            end_line: 1,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("method:src/wp-includes/interactivity-api/class-wp-interactivity-api.php:config"),
            file_path: PathBuf::from("src/wp-includes/interactivity-api/class-wp-interactivity-api.php"),
            kind: SymbolKind::Method,
            name: String::from("config"),
            qualified_name: String::from("WP_Interactivity_API::config"),
            parent_symbol_id: Some(String::from(
                "class:src/wp-includes/interactivity-api/class-wp-interactivity-api.php:WP_Interactivity_API",
            )),
            owner_type_name: Some(String::from("WP_Interactivity_API")),
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 1,
            required_parameter_count: 1,
            start_line: 10,
            end_line: 12,
        });
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("Gruntfile.js"),
            enclosing_symbol_id: None,
            kind: ReferenceKind::Call,
            target_name: String::from("config"),
            binding_name: None,
            line: 42,
            arity: Some(1),
            receiver_name: None,
            receiver_type_name: None,
            call_form: Some(CallForm::Free),
        });

        resolve_graph(&mut graph);

        assert_eq!(graph.resolved_edges.len(), 1);
        assert_eq!(
            graph.resolved_edges[0].target_symbol_id,
            "function:src/js/config.js:config"
        );
    }

    #[test]
    fn refuses_global_member_call_resolution_without_receiver_type() {
        let mut graph = SemanticGraph::default();
        graph.files.push(crate::graph::FileNode {
            path: PathBuf::from("Gruntfile.js"),
            language: Language::JavaScript,
        });
        graph.files.push(crate::graph::FileNode {
            path: PathBuf::from("src/wp-includes/interactivity-api/class-wp-interactivity-api.php"),
            language: Language::Php,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("method:src/wp-includes/interactivity-api/class-wp-interactivity-api.php:config"),
            file_path: PathBuf::from("src/wp-includes/interactivity-api/class-wp-interactivity-api.php"),
            kind: SymbolKind::Method,
            name: String::from("config"),
            qualified_name: String::from("WP_Interactivity_API::config"),
            parent_symbol_id: Some(String::from(
                "class:src/wp-includes/interactivity-api/class-wp-interactivity-api.php:WP_Interactivity_API",
            )),
            owner_type_name: Some(String::from("WP_Interactivity_API")),
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 1,
            required_parameter_count: 1,
            start_line: 10,
            end_line: 12,
        });
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("Gruntfile.js"),
            enclosing_symbol_id: None,
            kind: ReferenceKind::Call,
            target_name: String::from("config"),
            binding_name: None,
            line: 42,
            arity: Some(1),
            receiver_name: Some(String::from("grunt")),
            receiver_type_name: None,
            call_form: Some(CallForm::Member),
        });

        resolve_graph(&mut graph);

        assert!(graph.resolved_edges.is_empty());
    }

    #[test]
    fn resolves_alias_import_bindings_by_local_name() {
        let mut graph = SemanticGraph::default();
        graph.files.push(FileNode {
            path: PathBuf::from("src/models.rs"),
            language: Language::Rust,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("struct:src/models.rs:User"),
            file_path: PathBuf::from("src/models.rs"),
            kind: SymbolKind::Struct,
            name: String::from("User"),
            qualified_name: String::from("User"),
            parent_symbol_id: None,
            owner_type_name: None,
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 0,
            required_parameter_count: 0,
            start_line: 1,
            end_line: 1,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("module:src/models.rs"),
            file_path: PathBuf::from("src/models.rs"),
            kind: SymbolKind::Module,
            name: String::from("models"),
            qualified_name: String::from("models"),
            parent_symbol_id: None,
            owner_type_name: None,
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 0,
            required_parameter_count: 0,
            start_line: 1,
            end_line: 1,
        });
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("src/main.rs"),
            enclosing_symbol_id: None,
            kind: ReferenceKind::Import,
            target_name: String::from("crate::models::User"),
            binding_name: Some(String::from("U")),
            line: 1,
            arity: None,
            receiver_name: None,
            receiver_type_name: None,
            call_form: None,
        });

        let context = ResolutionContext::from_graph(&graph, &ResolveConfig::default());
        let resolved = context
            .resolve("U", PathBuf::from("src/main.rs").as_path())
            .unwrap();

        assert_eq!(resolved.tier, ResolutionTier::ImportScoped);
        assert_eq!(resolved.candidates.len(), 1);
        assert_eq!(resolved.candidates[0].name, "User");
        assert_eq!(
            resolved.candidates[0].file_path,
            PathBuf::from("src/models.rs")
        );
    }

    #[test]
    fn resolves_member_calls_using_imported_type_aliases() {
        let mut service = parse_python_to_graph(
            PathBuf::from("app/service.py"),
            r#"from .models import User as U
from .repo import Repo

def run(user: U):
    user.save()
"#,
        )
        .unwrap();
        let mut models = parse_python_to_graph(
            PathBuf::from("app/models.py"),
            r#"class User:
    def save(self):
        pass
"#,
        )
        .unwrap();
        let mut repo = parse_python_to_graph(
            PathBuf::from("app/repo.py"),
            r#"class Repo:
    def save(self):
        pass
"#,
        )
        .unwrap();

        service.files.append(&mut models.files);
        service.files.append(&mut repo.files);
        service.symbols.append(&mut models.symbols);
        service.symbols.append(&mut repo.symbols);
        service.references.append(&mut models.references);
        service.references.append(&mut repo.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/models.py")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(!service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/repo.py")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_typescript_member_calls_using_factory_return_types() {
        let mut service = parse_javascript_to_graph(
            PathBuf::from("src/service.ts"),
            r#"import { buildUser } from "./factory";

function run() {
  const user = buildUser();
  user.save();
}
"#,
            true,
        )
        .unwrap();
        let mut factory = parse_javascript_to_graph(
            PathBuf::from("src/factory.ts"),
            r#"import { User } from "./models";

export function buildUser(): User {
  return new User();
}
"#,
            true,
        )
        .unwrap();
        let mut models = parse_javascript_to_graph(
            PathBuf::from("src/models.ts"),
            r#"export class User {
  save() {}
}
"#,
            true,
        )
        .unwrap();
        let mut repo = parse_javascript_to_graph(
            PathBuf::from("src/repo.ts"),
            r#"export class Repo {
  save() {}
}
"#,
            true,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.files.append(&mut repo.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.symbols.append(&mut repo.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);
        service.references.append(&mut repo.references);

        resolve_graph(&mut service);

        let save_edge = service.resolved_edges.iter().find(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("src/models.ts")
                && edge.target_symbol_id.contains("save")
        });
        assert!(save_edge.is_some());
        assert_eq!(
            save_edge.unwrap().resolution_tier,
            ResolutionTier::ImportScoped
        );
        assert!(!service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("src/repo.ts")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_typescript_member_calls_using_aliased_factory_types() {
        let mut service = parse_javascript_to_graph(
            PathBuf::from("src/service.ts"),
            r#"import { UserFactory as UF } from "./factory";

function run(factory: UF) {
  const user = factory.buildUser();
  user.save();
}
"#,
            true,
        )
        .unwrap();
        let mut factory = parse_javascript_to_graph(
            PathBuf::from("src/factory.ts"),
            r#"import { User } from "./models";

export class UserFactory {
  buildUser(): User {
    return new User();
  }
}
"#,
            true,
        )
        .unwrap();
        let mut models = parse_javascript_to_graph(
            PathBuf::from("src/models.ts"),
            r#"export class User {
  save() {}
}
"#,
            true,
        )
        .unwrap();
        let mut repo = parse_javascript_to_graph(
            PathBuf::from("src/repo.ts"),
            r#"export class Repo {
  save() {}
}
"#,
            true,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.files.append(&mut repo.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.symbols.append(&mut repo.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);
        service.references.append(&mut repo.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("src/models.ts")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(!service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("src/repo.ts")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_typescript_chained_factory_member_calls() {
        let mut service = parse_javascript_to_graph(
            PathBuf::from("src/service.ts"),
            r#"import { buildUser } from "./factory";

function run() {
  buildUser().save();
}
"#,
            true,
        )
        .unwrap();
        let mut factory = parse_javascript_to_graph(
            PathBuf::from("src/factory.ts"),
            r#"import { User } from "./models";

export function buildUser(): User {
  return new User();
}
"#,
            true,
        )
        .unwrap();
        let mut models = parse_javascript_to_graph(
            PathBuf::from("src/models.ts"),
            r#"export class User {
  save() {}
}
"#,
            true,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("src/models.ts")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_typescript_static_factory_member_calls() {
        let mut service = parse_javascript_to_graph(
            PathBuf::from("src/service.ts"),
            r#"import { UserFactory as UF } from "./factory";

function run() {
  UF.buildUser().save();
}
"#,
            true,
        )
        .unwrap();
        let mut factory = parse_javascript_to_graph(
            PathBuf::from("src/factory.ts"),
            r#"import { User } from "./models";

export class UserFactory {
  static buildUser(): User {
    return new User();
  }
}
"#,
            true,
        )
        .unwrap();
        let mut models = parse_javascript_to_graph(
            PathBuf::from("src/models.ts"),
            r#"export class User {
  save() {}
}
"#,
            true,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("src/models.ts")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_typescript_awaited_promise_return_types() {
        let mut service = parse_javascript_to_graph(
            PathBuf::from("src/service.ts"),
            r#"import { buildUser } from "./factory";

async function run() {
  const user = await buildUser();
  user.save();
}
"#,
            true,
        )
        .unwrap();
        let mut factory = parse_javascript_to_graph(
            PathBuf::from("src/factory.ts"),
            r#"import { User } from "./models";

export function buildUser(): Promise<User> {
  return Promise.resolve(new User());
}
"#,
            true,
        )
        .unwrap();
        let mut models = parse_javascript_to_graph(
            PathBuf::from("src/models.ts"),
            r#"export class User {
  save() {}
}
"#,
            true,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("src/models.ts")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_python_member_calls_using_factory_return_types() {
        let mut service = parse_python_to_graph(
            PathBuf::from("app/service.py"),
            r#"from .factory import build_user

def run():
    user = build_user()
    user.save()
"#,
        )
        .unwrap();
        let mut factory = parse_python_to_graph(
            PathBuf::from("app/factory.py"),
            r#"from .models import User

def build_user() -> User:
    return User()
"#,
        )
        .unwrap();
        let mut models = parse_python_to_graph(
            PathBuf::from("app/models.py"),
            r#"class User:
    def save(self):
        pass
"#,
        )
        .unwrap();
        let mut repo = parse_python_to_graph(
            PathBuf::from("app/repo.py"),
            r#"class Repo:
    def save(self):
        pass
"#,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.files.append(&mut repo.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.symbols.append(&mut repo.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);
        service.references.append(&mut repo.references);

        resolve_graph(&mut service);

        let save_edge = service.resolved_edges.iter().find(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/models.py")
                && edge.target_symbol_id.contains("save")
        });
        assert!(save_edge.is_some());
        assert_eq!(
            save_edge.unwrap().resolution_tier,
            ResolutionTier::ImportScoped
        );
        assert!(!service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/repo.py")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_python_chained_factory_member_calls() {
        let mut service = parse_python_to_graph(
            PathBuf::from("app/service.py"),
            r#"from .factory import build_user

def run():
    build_user().save()
"#,
        )
        .unwrap();
        let mut factory = parse_python_to_graph(
            PathBuf::from("app/factory.py"),
            r#"from .models import User

def build_user() -> User:
    return User()
"#,
        )
        .unwrap();
        let mut models = parse_python_to_graph(
            PathBuf::from("app/models.py"),
            r#"class User:
    def save(self):
        pass
"#,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/models.py")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_python_static_factory_member_calls() {
        let mut service = parse_python_to_graph(
            PathBuf::from("app/service.py"),
            r#"from .factory import UserFactory as UF

def run():
    UF.build_user().save()
"#,
        )
        .unwrap();
        let mut factory = parse_python_to_graph(
            PathBuf::from("app/factory.py"),
            r#"from .models import User

class UserFactory:
    @staticmethod
    def build_user() -> User:
        return User()
"#,
        )
        .unwrap();
        let mut models = parse_python_to_graph(
            PathBuf::from("app/models.py"),
            r#"class User:
    def save(self):
        pass
"#,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/models.py")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_python_optional_return_types() {
        let mut service = parse_python_to_graph(
            PathBuf::from("app/service.py"),
            r#"from .factory import build_user

def run():
    user = build_user()
    user.save()
"#,
        )
        .unwrap();
        let mut factory = parse_python_to_graph(
            PathBuf::from("app/factory.py"),
            r#"from typing import Optional
from .models import User

def build_user() -> Optional[User]:
    return User()
"#,
        )
        .unwrap();
        let mut models = parse_python_to_graph(
            PathBuf::from("app/models.py"),
            r#"class User:
    def save(self):
        pass
"#,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/models.py")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_php_member_calls_using_factory_return_types() {
        let mut service = parse_php_to_graph(
            PathBuf::from("app/Service.php"),
            r#"<?php
use App\Factories\UserFactory;

function run(UserFactory $factory) {
    $user = $factory->makeUser();
    $user->save();
}
"#,
        )
        .unwrap();
        let mut factory = parse_php_to_graph(
            PathBuf::from("app/Factories/UserFactory.php"),
            r#"<?php
namespace App\Factories;

use App\Models\User;

class UserFactory {
    public function makeUser(): User {
        return new User();
    }
}
"#,
        )
        .unwrap();
        let mut models = parse_php_to_graph(
            PathBuf::from("app/Models/User.php"),
            r#"<?php
namespace App\Models;

class User {
    public function save() {}
}
"#,
        )
        .unwrap();
        let mut repo = parse_php_to_graph(
            PathBuf::from("app/Repo.php"),
            r#"<?php
class Repo {
    public function save() {}
}
"#,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.files.append(&mut repo.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.symbols.append(&mut repo.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);
        service.references.append(&mut repo.references);

        resolve_graph(&mut service);

        let save_edge = service.resolved_edges.iter().find(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/Models/User.php")
                && edge.target_symbol_id.contains("save")
        });
        assert!(save_edge.is_some());
        assert_eq!(
            save_edge.unwrap().resolution_tier,
            ResolutionTier::ImportScoped
        );
        assert!(!service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/Repo.php")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_php_member_calls_using_aliased_factory_types() {
        let mut service = parse_php_to_graph(
            PathBuf::from("app/Service.php"),
            r#"<?php
use App\Factories\UserFactory as UF;

function run(UF $factory) {
    $user = $factory->makeUser();
    $user->save();
}
"#,
        )
        .unwrap();
        let mut factory = parse_php_to_graph(
            PathBuf::from("app/Factories/UserFactory.php"),
            r#"<?php
namespace App\Factories;

use App\Models\User;

class UserFactory {
    public function makeUser(): User {
        return new User();
    }
}
"#,
        )
        .unwrap();
        let mut models = parse_php_to_graph(
            PathBuf::from("app/Models/User.php"),
            r#"<?php
namespace App\Models;

class User {
    public function save() {}
}
"#,
        )
        .unwrap();
        let mut repo = parse_php_to_graph(
            PathBuf::from("app/Repo.php"),
            r#"<?php
class Repo {
    public function save() {}
}
"#,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.files.append(&mut repo.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.symbols.append(&mut repo.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);
        service.references.append(&mut repo.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/Models/User.php")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(!service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/Repo.php")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_php_chained_factory_member_calls() {
        let mut service = parse_php_to_graph(
            PathBuf::from("app/Service.php"),
            r#"<?php
use App\Factories\UserFactory;

function run(UserFactory $factory) {
    $factory->makeUser()->save();
}
"#,
        )
        .unwrap();
        let mut factory = parse_php_to_graph(
            PathBuf::from("app/Factories/UserFactory.php"),
            r#"<?php
namespace App\Factories;

use App\Models\User;

class UserFactory {
    public function makeUser(): User {
        return new User();
    }
}
"#,
        )
        .unwrap();
        let mut models = parse_php_to_graph(
            PathBuf::from("app/Models/User.php"),
            r#"<?php
namespace App\Models;

class User {
    public function save() {}
}
"#,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/Models/User.php")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_php_nullable_return_types() {
        let mut service = parse_php_to_graph(
            PathBuf::from("app/Service.php"),
            r#"<?php
use App\Factories\UserFactory;

function run(UserFactory $factory) {
    $user = $factory->makeUser();
    $user?->save();
}
"#,
        )
        .unwrap();
        let mut factory = parse_php_to_graph(
            PathBuf::from("app/Factories/UserFactory.php"),
            r#"<?php
namespace App\Factories;

use App\Models\User;

class UserFactory {
    public function makeUser(): ?User {
        return new User();
    }
}
"#,
        )
        .unwrap();
        let mut models = parse_php_to_graph(
            PathBuf::from("app/Models/User.php"),
            r#"<?php
namespace App\Models;

class User {
    public function save() {}
}
"#,
        )
        .unwrap();

        service.files.append(&mut factory.files);
        service.files.append(&mut models.files);
        service.symbols.append(&mut factory.symbols);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut factory.references);
        service.references.append(&mut models.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/Models/User.php")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_ruby_member_calls_after_constructor_assignment() {
        let mut service = parse_ruby_to_graph(
            PathBuf::from("app/service.rb"),
            r#"def run
  user = User.new
  user.save
end
"#,
        )
        .unwrap();
        let mut user = parse_ruby_to_graph(
            PathBuf::from("app/models/user.rb"),
            r#"class User
  def save
  end
end
"#,
        )
        .unwrap();
        let mut repo = parse_ruby_to_graph(
            PathBuf::from("app/repo.rb"),
            r#"class Repo
  def save
  end
end
"#,
        )
        .unwrap();

        service.files.append(&mut user.files);
        service.files.append(&mut repo.files);
        service.symbols.append(&mut user.symbols);
        service.symbols.append(&mut repo.symbols);
        service.references.append(&mut user.references);
        service.references.append(&mut repo.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/models/user.rb")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(!service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/repo.rb")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn resolves_ruby_chained_constructor_member_calls() {
        let mut service = parse_ruby_to_graph(
            PathBuf::from("app/service.rb"),
            r#"def run
  User.new.save
end
"#,
        )
        .unwrap();
        let mut user = parse_ruby_to_graph(
            PathBuf::from("app/models/user.rb"),
            r#"class User
  def save
  end
end
"#,
        )
        .unwrap();

        service.files.append(&mut user.files);
        service.symbols.append(&mut user.symbols);
        service.references.append(&mut user.references);

        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.target_file_path == Path::new("app/models/user.rb")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn emits_override_edges_for_inherited_methods() {
        let mut graph = parse_python_to_graph(
            PathBuf::from("app/service.py"),
            r#"class Base:
    def save(self):
        pass

class Service(Base):
    def save(self):
        pass
"#,
        )
        .unwrap();

        resolve_graph(&mut graph);

        assert!(graph.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Overrides
                && edge.source_symbol_id.as_deref()
                    == Some("method:app/service.py:class:app/service.py:Service:save")
                && edge.target_symbol_id == "method:app/service.py:class:app/service.py:Base:save"
        }));
    }

    #[test]
    fn uses_python_mro_order_for_override_targets() {
        let mut graph = parse_python_to_graph(
            PathBuf::from("app/service.py"),
            r#"class A:
    def save(self):
        pass

class B(A):
    pass

class C(A):
    def save(self):
        pass

class D(B, C):
    def save(self):
        pass
"#,
        )
        .unwrap();

        resolve_graph(&mut graph);

        assert!(graph.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Overrides
                && edge.source_symbol_id.as_deref()
                    == Some("method:app/service.py:class:app/service.py:D:save")
                && edge.target_symbol_id == "method:app/service.py:class:app/service.py:C:save"
        }));
        assert!(!graph.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Overrides
                && edge.source_symbol_id.as_deref()
                    == Some("method:app/service.py:class:app/service.py:D:save")
                && edge.target_symbol_id == "method:app/service.py:class:app/service.py:A:save"
        }));
    }

    #[test]
    fn resolves_typescript_relative_imports_with_module_fallback() {
        let mut app = parse_javascript_to_graph(
            PathBuf::from("src/app.ts"),
            r#"import DefaultThing, { User } from "./models";
DefaultThing.run();
const user = new User();
"#,
            true,
        )
        .unwrap();
        let mut models = parse_javascript_to_graph(
            PathBuf::from("src/models.ts"),
            r#"export class User {}
export class Service {
  static run() {}
}"#,
            true,
        )
        .unwrap();

        app.files.append(&mut models.files);
        app.symbols.append(&mut models.symbols);
        app.references.append(&mut models.references);
        resolve_graph(&mut app);

        assert!(app.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("src/models.ts")
                && edge.target_symbol_id == "module:src/models.ts"
        }));
        assert!(app.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call && edge.target_file_path == Path::new("src/models.ts")
        }));
    }

    #[test]
    fn resolves_typescript_tsconfig_path_aliases() {
        let mut app = parse_javascript_to_graph(
            PathBuf::from("src/app.ts"),
            r#"import { User } from "@domain/models";
const user = new User();
const _unused = user;
"#,
            true,
        )
        .unwrap();
        let mut models = parse_javascript_to_graph(
            PathBuf::from("packages/domain/models.ts"),
            r#"export class User {}"#,
            true,
        )
        .unwrap();

        app.files.append(&mut models.files);
        app.symbols.append(&mut models.symbols);
        app.references.append(&mut models.references);

        let config = ResolveConfig {
            tsconfig_paths: vec![TsPathAlias {
                pattern: String::from("@domain/*"),
                targets: vec![String::from("packages/domain/*")],
                base_dir: PathBuf::new(),
            }],
            composer_psr4: Vec::new(),
            python_roots: Vec::new(),
            ruby_load_paths: Vec::new(),
        };
        resolve_graph_with_config(&mut app, &config);

        assert!(app.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("packages/domain/models.ts")
        }));
    }

    #[test]
    fn resolves_python_relative_imports() {
        let mut service = parse_python_to_graph(
            PathBuf::from("app/service.py"),
            r#"from .models import User

def run(user: User):
    return user
"#,
        )
        .unwrap();
        let mut models = parse_python_to_graph(
            PathBuf::from("app/models.py"),
            r#"class User:
    pass
"#,
        )
        .unwrap();

        service.files.append(&mut models.files);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut models.references);
        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("app/models.py")
        }));
    }

    #[test]
    fn resolves_python_absolute_imports_from_src_root() {
        let mut service = parse_python_to_graph(
            PathBuf::from("src/pkg/service.py"),
            r#"from domain.models import User

def run(user: User):
    return user
"#,
        )
        .unwrap();
        let mut models = parse_python_to_graph(
            PathBuf::from("src/domain/models.py"),
            r#"class User:
    pass
"#,
        )
        .unwrap();

        service.files.append(&mut models.files);
        service.symbols.append(&mut models.symbols);
        service.references.append(&mut models.references);

        let config = ResolveConfig {
            tsconfig_paths: Vec::new(),
            composer_psr4: Vec::new(),
            python_roots: vec![PathBuf::from("src")],
            ruby_load_paths: Vec::new(),
        };
        resolve_graph_with_config(&mut service, &config);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("src/domain/models.py")
        }));
    }

    #[test]
    fn resolves_php_namespace_imports_by_suffix() {
        let mut service = parse_php_to_graph(
            PathBuf::from("app/Service.php"),
            r#"<?php
use App\Models\User;
"#,
        )
        .unwrap();
        let mut user = parse_php_to_graph(
            PathBuf::from("app/Models/User.php"),
            r#"<?php class User {}"#,
        )
        .unwrap();

        service.files.append(&mut user.files);
        service.symbols.append(&mut user.symbols);
        service.references.append(&mut user.references);
        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("app/Models/User.php")
        }));
    }

    #[test]
    fn resolves_php_imports_with_composer_psr4_mapping() {
        let mut service = parse_php_to_graph(
            PathBuf::from("app/Service.php"),
            r#"<?php
use Acme\Models\User;
"#,
        )
        .unwrap();
        let mut user = parse_php_to_graph(
            PathBuf::from("app/Models/User.php"),
            r#"<?php class User {}"#,
        )
        .unwrap();

        service.files.append(&mut user.files);
        service.symbols.append(&mut user.symbols);
        service.references.append(&mut user.references);

        let config = ResolveConfig {
            tsconfig_paths: Vec::new(),
            composer_psr4: vec![ComposerPsr4Mapping {
                prefix: String::from("Acme\\"),
                directories: vec![PathBuf::from("app")],
            }],
            python_roots: Vec::new(),
            ruby_load_paths: Vec::new(),
        };
        resolve_graph_with_config(&mut service, &config);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("app/Models/User.php")
        }));
    }

    #[test]
    fn resolves_ruby_require_relative_to_module_file() {
        let mut service = parse_ruby_to_graph(
            PathBuf::from("app/service.rb"),
            r#"require_relative "./models/user"
"#,
        )
        .unwrap();
        let mut user = parse_ruby_to_graph(
            PathBuf::from("app/models/user.rb"),
            r#"class User
end
"#,
        )
        .unwrap();

        service.files.append(&mut user.files);
        service.symbols.append(&mut user.symbols);
        service.references.append(&mut user.references);
        resolve_graph(&mut service);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("app/models/user.rb")
                && edge.target_symbol_id == "module:app/models/user.rb"
        }));
    }

    #[test]
    fn resolves_ruby_require_from_load_path() {
        let mut service = parse_ruby_to_graph(
            PathBuf::from("app/service.rb"),
            r#"require "support/user"
"#,
        )
        .unwrap();
        let mut user = parse_ruby_to_graph(
            PathBuf::from("lib/support/user.rb"),
            r#"class User
end
"#,
        )
        .unwrap();

        service.files.append(&mut user.files);
        service.symbols.append(&mut user.symbols);
        service.references.append(&mut user.references);

        let config = ResolveConfig {
            tsconfig_paths: Vec::new(),
            composer_psr4: Vec::new(),
            python_roots: Vec::new(),
            ruby_load_paths: vec![PathBuf::from("lib")],
        };
        resolve_graph_with_config(&mut service, &config);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("lib/support/user.rb")
                && edge.target_symbol_id == "module:lib/support/user.rb"
        }));
    }

    #[test]
    fn loads_tsconfig_and_composer_mappings_from_disk() {
        let fixture = create_fixture();
        fs::write(
            fixture.join("tsconfig.json"),
            br#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@domain/*": ["packages/domain/*"]
    }
  }
}"#,
        )
        .unwrap();
        fs::write(
            fixture.join("composer.json"),
            br#"{
  "autoload": {
    "psr-4": {
      "Acme\\": "app/"
    }
  }
}"#,
        )
        .unwrap();

        let config = load_resolve_config(&fixture);

        assert_eq!(config.tsconfig_paths.len(), 1);
        assert_eq!(config.composer_psr4.len(), 1);
        assert!(!config.python_roots.is_empty());
    }

    fn fixture_graph() -> SemanticGraph {
        let mut graph = SemanticGraph::default();
        graph.files.push(FileNode {
            path: PathBuf::from("src/main.rs"),
            language: Language::Rust,
        });
        graph.files.push(FileNode {
            path: PathBuf::from("src/models.rs"),
            language: Language::Rust,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("module:src/main.rs"),
            file_path: PathBuf::from("src/main.rs"),
            kind: SymbolKind::Module,
            name: String::from("main"),
            qualified_name: String::from("main"),
            parent_symbol_id: None,
            owner_type_name: None,
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 0,
            required_parameter_count: 0,
            start_line: 1,
            end_line: 1,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("module:src/models.rs"),
            file_path: PathBuf::from("src/models.rs"),
            kind: SymbolKind::Module,
            name: String::from("models"),
            qualified_name: String::from("models"),
            parent_symbol_id: None,
            owner_type_name: None,
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 0,
            required_parameter_count: 0,
            start_line: 1,
            end_line: 1,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("function:src/main.rs:helper"),
            file_path: PathBuf::from("src/main.rs"),
            kind: SymbolKind::Function,
            name: String::from("helper"),
            qualified_name: String::from("helper"),
            parent_symbol_id: None,
            owner_type_name: None,
            return_type_name: None,
            visibility: Visibility::Private,
            parameter_count: 0,
            required_parameter_count: 0,
            start_line: 1,
            end_line: 1,
        });
        graph.symbols.push(SymbolNode {
            id: String::from("struct:src/models.rs:User"),
            file_path: PathBuf::from("src/models.rs"),
            kind: SymbolKind::Struct,
            name: String::from("User"),
            qualified_name: String::from("User"),
            parent_symbol_id: None,
            owner_type_name: None,
            return_type_name: None,
            visibility: Visibility::Public,
            parameter_count: 0,
            required_parameter_count: 0,
            start_line: 1,
            end_line: 1,
        });
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("src/main.rs"),
            enclosing_symbol_id: None,
            kind: ReferenceKind::Import,
            target_name: String::from("crate::models::User"),
            binding_name: Some(String::from("User")),
            line: 1,
            arity: None,
            receiver_name: None,
            receiver_type_name: None,
            call_form: None,
        });
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("src/main.rs"),
            enclosing_symbol_id: Some(String::from("function:src/main.rs:entry")),
            kind: ReferenceKind::Call,
            target_name: String::from("helper"),
            binding_name: None,
            line: 2,
            arity: Some(0),
            receiver_name: None,
            receiver_type_name: None,
            call_form: Some(CallForm::Free),
        });
        graph
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aigiscore-resolve-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
