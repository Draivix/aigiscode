use crate::graph::{
    CallForm, Language, ReferenceKind, ResolutionTier, ResolvedEdge, SemanticGraph,
    SemanticReference, SymbolKind,
};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use crate::ingestion::inputs::normalize_relative_path;
use std::sync::OnceLock;

mod tsconfig;
pub use tsconfig::ResolveConfigError;
mod cache;
pub use cache::{ResolutionCache, ResolutionWork};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
pub struct TieredCandidates<'a> {
    pub candidates: Vec<&'a SymbolDefinition>,
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
    ts_projects: Vec<tsconfig::TsProject>,
    pub(crate) fingerprint: String,
    pub(crate) input_paths: Vec<PathBuf>,
    composer_psr4: Vec<ComposerPsr4Mapping>,
    python_roots: Vec<PathBuf>,
    ruby_load_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    declared_imports: HashSet<(PathBuf, String)>,
    module_index: HashMap<PathBuf, SymbolDefinition>,
    declared_module_bindings: HashMap<(PathBuf, String), PathBuf>,
    reference_import_map: HashMap<(PathBuf, usize, String), Vec<PathBuf>>,
    language_map: HashMap<PathBuf, Language>,
    lexical_symbols: HashMap<String, SymbolDefinition>,
    lexical_calls: HashMap<(PathBuf, usize, String, usize), Option<String>>,
}

impl ResolutionContext {
    pub fn from_graph(graph: &SemanticGraph, config: &ResolveConfig) -> Self {
        let mut context = Self::default();

        for file in &graph.files {
            context
                .language_map
                .insert(file.path.clone(), file.language);
        }

        // A function nested inside another function/method is scope-local:
        // no code outside its enclosing scope can name it, so it must never
        // be a cross-file (global-tier) resolution target. Same-file
        // resolution keeps it (closures calling siblings).
        let function_like_ids = graph
            .symbols
            .iter()
            .filter(|symbol| matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method))
            .map(|symbol| symbol.id.as_str())
            .collect::<HashSet<_>>();
        let scoped_symbols = graph.lexical_bindings.scoped_symbol_ids.iter().collect::<HashSet<_>>();
        let bound_symbols = graph.lexical_bindings.calls.iter()
            .filter_map(|binding| binding.target_symbol_id.as_ref())
            .chain(graph.lexical_bindings.named_references.iter().filter_map(|binding| binding.target_symbol_id.as_ref()))
            .collect::<HashSet<_>>();

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
            if scoped_symbols.contains(&symbol.id) || bound_symbols.contains(&symbol.id) {
                context.lexical_symbols.insert(symbol.id.clone(), definition.clone());
            }
            // Parser-scoped JS/TS declarations are available only through a
            // lexical binding, never through a file/global name guess.
            if scoped_symbols.contains(&symbol.id) {
                continue;
            }

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
            let scope_local = matches!(symbol.kind, SymbolKind::Function)
                && symbol
                    .parent_symbol_id
                    .as_deref()
                    .is_some_and(|parent| function_like_ids.contains(parent));
            if !scope_local {
                context
                    .global_index
                    .entry(symbol.name.clone())
                    .or_default()
                    .push(definition.clone());
            }
            context
                .qualified_index
                .entry(
                    if context.language_map.get(&symbol.file_path) == Some(&Language::Php) {
                        symbol.qualified_name.to_ascii_lowercase()
                    } else {
                        symbol.qualified_name.clone()
                    },
                )
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

        let bindings = graph.lexical_bindings.calls.iter()
            .filter(|binding| binding.argument_index.is_none())
            .map(|binding| (binding.reference_index, &binding.target_symbol_id))
            .chain(graph.lexical_bindings.named_references.iter().map(|binding| (binding.reference_index, &binding.target_symbol_id)))
            .collect::<HashMap<_, _>>();
        let binding_keys = bindings.keys().filter_map(|index| graph.references.get(*index))
            .map(|reference| (reference.file_path.as_path(), reference.line, reference.target_name.as_str()))
            .collect::<HashSet<_>>();
        let mut binding_occurrences = HashMap::new();
        for (index, reference) in graph.references.iter().enumerate() {
            let key = (reference.file_path.as_path(), reference.line, reference.target_name.as_str());
            if binding_keys.contains(&key) {
                let occurrence = *binding_occurrences.entry(key).and_modify(|value| *value += 1).or_insert(0);
                if let Some(target) = bindings.get(&index) {
                    context.lexical_calls.insert(
                        (reference.file_path.clone(), reference.line, reference.target_name.clone(), occurrence),
                        (*target).clone(),
                    );
                }
            }
            if !reference.kind.is_import() {
                continue;
            }
            if let Some(binding) = &reference.binding_name {
                context.declared_imports.insert((reference.file_path.clone(), binding.clone()));
            }

            let import_targets =
                if context.language_map.get(&reference.file_path) == Some(&Language::Php) {
                    let candidates = context.php_qualified_candidates(&reference.target_name);
                    if candidates.len() == 1 {
                        candidates
                            .into_iter()
                            .map(|candidate| candidate.file_path.clone())
                            .collect()
                    } else {
                        HashSet::new()
                    }
                } else {
                    resolve_import_paths(reference, &known_files, &context.language_map, config)
                };
            if import_targets.is_empty() {
                continue;
            }

            // `resolve_import_paths` returns a HashSet whose iteration order is
            // random per process. Everything ordering-sensitive downstream —
            // `pick_edge` taking the first module candidate, the named-import
            // map's last-insert-wins — must see a deterministic order, or the
            // same input resolves `../types` to `types.ts` on one run and
            // `types/index.ts` on the next.
            let mut import_targets_vec = import_targets.iter().cloned().collect::<Vec<_>>();
            import_targets_vec.sort();
            // Rust `mod child;` has a parser-owned `self::child` import fact.
            // Its local name can differ from the file's crate-root identity
            // (notably `mod lib;` in a binary beside `src/lib.rs`).
            if context.language_map.get(&reference.file_path) == Some(&Language::Rust)
                && reference.binding_name.is_none()
                && import_targets_vec.len() == 1
            {
                if let Some(name) = reference
                    .target_name
                    .strip_prefix("self::")
                    .filter(|name| !name.contains("::") && *name != "*")
                {
                    context.declared_module_bindings.insert(
                        (reference.file_path.clone(), name.to_owned()),
                        import_targets_vec[0].clone(),
                    );
                }
            }
            context.reference_import_map.insert(
                (
                    reference.file_path.clone(),
                    reference.line,
                    reference.target_name.clone(),
                ),
                import_targets_vec.clone(),
            );

            context
                .import_map
                .entry(reference.file_path.clone())
                .or_default()
                .extend(import_targets);

            if let Some(binding_name) = &reference.binding_name {
                let exported_name =
                    if context.language_map.get(&reference.file_path) == Some(&Language::Php) {
                        reference.target_name.trim_start_matches('\\').to_owned()
                    } else {
                        leaf_symbol_name(&reference.target_name)
                    };
                for target_file in import_targets_vec {
                    context.named_import_map.insert(
                        (reference.file_path.clone(), binding_name.clone()),
                        (target_file, exported_name.clone()),
                    );
                }
            }
        }

        context
    }

    pub fn resolve(&self, name: &str, from_file: &Path) -> Option<TieredCandidates<'_>> {
        let key = (from_file.to_path_buf(), name.to_owned());
        if let Some(definitions) = self.file_index.get(&key) {
            return Some(TieredCandidates {
                candidates: definitions.iter().collect(),
                tier: ResolutionTier::SameFile,
            });
        }

        if let Some((source_file, exported_name)) = self
            .named_import_map
            .get(&(from_file.to_path_buf(), name.to_owned()))
        {
            let lookup_name = if self.language_map.get(from_file) == Some(&Language::Php) {
                exported_name.to_ascii_lowercase()
            } else {
                exported_name.clone()
            };
            let mut named_candidates = self
                .qualified_index
                .get(&lookup_name)
                .or_else(|| self.global_index.get(exported_name))
                .into_iter()
                .flat_map(|candidates| candidates.iter())
                .filter(|candidate| &candidate.file_path == source_file)
                .collect::<Vec<_>>();
            if named_candidates.is_empty() {
                if let Some(module_candidate) = self.module_index.get(source_file) {
                    named_candidates.push(module_candidate);
                }
            }
            if !named_candidates.is_empty() {
                return Some(TieredCandidates {
                    candidates: named_candidates,
                    tier: ResolutionTier::ImportScoped,
                });
            }
        }

        if self.declared_imports.contains(&(from_file.to_path_buf(), name.to_owned())) {
            return None;
        }
        let all_candidates = self.global_index.get(name)?;
        if let Some(imported_files) = self.import_map.get(from_file) {
            let imported_candidates = all_candidates
                .iter()
                .filter(|candidate| imported_files.contains(&candidate.file_path))
                .collect::<Vec<_>>();
            if !imported_candidates.is_empty() {
                return Some(TieredCandidates {
                    candidates: imported_candidates,
                    tier: ResolutionTier::ImportScoped,
                });
            }
        }

        Some(TieredCandidates {
            candidates: all_candidates.iter().collect(),
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

    fn php_qualified_candidates(&self, name: &str) -> Vec<&SymbolDefinition> {
        self.qualified_index
            .get(&name.trim_start_matches('\\').to_ascii_lowercase())
            .into_iter()
            .flatten()
            .filter(|candidate| self.language_map.get(&candidate.file_path) == Some(&Language::Php))
            .collect()
    }

    fn module_candidates_for_files(&self, files: &[PathBuf]) -> Vec<&SymbolDefinition> {
        files
            .iter()
            .filter_map(|file| self.module_index.get(file))
            .collect()
    }
}

pub fn load_resolve_config(root: &Path, files: &[PathBuf]) -> Result<ResolveConfig, ResolveConfigError> {
    load_resolve_config_with_inputs(root, files, &mut crate::ingestion::inputs::InputFiles::default())
}

pub(crate) fn load_resolve_config_with_inputs(
    root: &Path,
    files: &[PathBuf],
    inputs: &mut crate::ingestion::inputs::InputFiles,
) -> Result<ResolveConfig, ResolveConfigError> {
    let root = root.canonicalize().map_err(|error| ResolveConfigError {
        path: root.to_path_buf(), message: error.to_string(),
    })?;
    let mut reader = tsconfig::ConfigReader::new(inputs);
    let mut config = ResolveConfig::default();
    config.ts_projects = reader.load_projects(&root, files)?;
    let composer_path = root.join("composer.json");
    if let Some(bytes) = reader.read(&composer_path)? {
        let json = serde_json::from_slice(&bytes).map_err(|error| ResolveConfigError {
            path: composer_path.clone(), message: error.to_string(),
        })?;
        config.composer_psr4 = load_composer_psr4(&root, &composer_path, &json);
    }
    config.python_roots = discover_roots(&root, &["src", "app"], &mut reader)?;
    config.python_roots.insert(0, PathBuf::new());
    config.python_roots.sort();
    config.ruby_load_paths = discover_roots(&root, &["lib", "app"], &mut reader)?;
    config.fingerprint = reader.fingerprint();
    config.input_paths = reader.input_paths();
    Ok(config)
}

pub fn resolve_graph(graph: &mut SemanticGraph) {
    resolve_graph_with_config(graph, &ResolveConfig::default());
}

pub fn resolve_graph_with_config(graph: &mut SemanticGraph, config: &ResolveConfig) {
    graph.resolved_edges.clear();
    let context = ResolutionContext::from_graph(graph, config);
    let resolved = resolve_references(graph.references.iter(), &context);
    for (_, edge) in resolved {
        graph.add_resolved_edge(edge);
    }
    append_override_edges(graph);
}

fn resolve_references<'a>(
    references: impl Iterator<Item = &'a SemanticReference>,
    context: &ResolutionContext,
) -> Vec<(usize, ResolvedEdge)> {
    let mut occurrence_counters = HashMap::<(PathBuf, usize, String), usize>::new();
    references
        .enumerate()
        .filter_map(|(index, reference)| {
            let occurrence_key = (
                reference.file_path.clone(),
                reference.line,
                reference.target_name.clone(),
            );
            let occurrence_index = *occurrence_counters
                .entry(occurrence_key)
                .and_modify(|value| *value += 1)
                .or_insert(0);
            resolve_reference(reference, context, occurrence_index).map(|edge| {
                (index, edge.with_reference_identity(reference.target_name.clone(), occurrence_index))
            })
        })
        .collect()
}

fn resolve_reference(
    reference: &SemanticReference,
    context: &ResolutionContext,
    occurrence_index: usize,
) -> Option<ResolvedEdge> {
    if reference.kind.is_import() {
        return resolve_import_reference(reference, context);
    }
    if let Some(binding) = context.lexical_calls.get(&(
        reference.file_path.clone(), reference.line, reference.target_name.clone(), occurrence_index,
    )) {
        let symbol = context.lexical_symbols.get(binding.as_ref()?)?;
        return pick_edge(reference, TieredCandidates {
            candidates: vec![symbol],
            tier: ResolutionTier::SameFile,
        }).map(|mut edge| {
            edge.reason = if reference.kind == ReferenceKind::Call {
                "call:lexical-binding"
            } else {
                "reference:lexical-binding"
            }.to_owned();
            edge
        });
    }

    // A type reference whose leaf is a language primitive / pseudo type (`float`,
    // `int`, `string`, `void`, `self`, ...) can never denote a user-defined
    // symbol, even when a method or function happens to share that name. Without
    // this guard `private function float(...)` absorbs every `: float` type-use
    // in its own file and fabricates a self-loop edge.
    if reference.kind == ReferenceKind::Type
        && is_primitive_type_name(&leaf_symbol_name(&reference.target_name))
    {
        return None;
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
    if context.language_map.get(&reference.file_path) == Some(&Language::Php) {
        let candidates = context.php_qualified_candidates(&reference.target_name);
        if candidates.len() != 1 {
            return None;
        }
        return pick_edge(
            reference,
            TieredCandidates {
                candidates,
                tier: ResolutionTier::ImportScoped,
            },
        );
    }
    let preferred_name = reference
        .binding_name
        .clone()
        .unwrap_or_else(|| leaf_symbol_name(&reference.target_name));
    let import_targets = context.import_targets_for_reference(reference);
    let source_language = context.language_map.get(&reference.file_path)?;
    let import_targets = import_targets
        .into_iter()
        .filter(|path| context.language_map.get(path).is_some_and(|language| {
            same_language_family(*source_language, *language)
        }))
        .collect::<Vec<_>>();
    if import_targets.is_empty() {
        return None;
    }
    let exported_name = leaf_symbol_name(&reference.target_name);
    let mut definitions = import_targets
        .iter()
        .flat_map(|path| {
            context.file_index.get(&(path.clone(), exported_name.clone()))
                .or_else(|| context.file_index.get(&(path.clone(), preferred_name.clone())))
                .into_iter().flatten()
        })
        .collect::<Vec<_>>();
    if definitions.is_empty() {
        definitions = context.module_candidates_for_files(&import_targets);
    }
    pick_edge(reference, TieredCandidates {
        candidates: definitions,
        tier: ResolutionTier::ImportScoped,
    })
}

fn pick_edge(reference: &SemanticReference, candidates: TieredCandidates<'_>) -> Option<ResolvedEdge> {
    if candidates.candidates.is_empty() {
        return None;
    }
    if candidates.tier == ResolutionTier::Global && candidates.candidates.len() != 1 {
        return None;
    }

    let target = candidates.candidates.first()?;
    Some(ResolvedEdge::new(
        reference.file_path.clone(),
        reference.enclosing_symbol_id.clone(),
        target.file_path.clone(),
        target.symbol_id.clone(),
        reference.kind,
        candidates.tier,
        confidence_millis(candidates.tier),
        resolution_reason(reference.kind, candidates.tier),
        reference.line,
    ))
}

fn filter_candidates<'a>(
    reference: &SemanticReference,
    mut candidates: TieredCandidates<'a>,
    context: &'a ResolutionContext,
) -> TieredCandidates<'a> {
    // A static reference can never cross a language family: a TS `extends
    // Error` / `new Error()` / `SomeService` type-use must not bind to a PHP
    // class of the same name elsewhere in the repo. Vue SFC scripts parse as
    // JS/TS, so legitimate Vue->TS edges stay within one family and survive.
    if let Some(source_language) = context.language_map.get(&reference.file_path).copied() {
        candidates.candidates.retain(|candidate| {
            context
                .language_map
                .get(&candidate.file_path)
                .copied()
                .is_none_or(|candidate_language| {
                    same_language_family(source_language, candidate_language)
                })
        });
        // A global-tier (no import, no same-file definition) inheritance match
        // on a builtin supertype name is the language builtin, not a same-named
        // user class from an unrelated namespace.
        if candidates.tier == ResolutionTier::Global
            && matches!(
                reference.kind,
                ReferenceKind::Extends | ReferenceKind::Implements
            )
            && is_builtin_supertype_name(source_language, &leaf_symbol_name(&reference.target_name))
        {
            candidates.candidates.clear();
        }
    }
    if reference.kind == ReferenceKind::Call {
        candidates = prefer_same_language_call_candidates(reference, candidates, context);
        let mut receiver_narrowed = false;
        if matches!(reference.call_form, Some(CallForm::Free)) {
            // A single-character callee (`t(...)`, `h(...)`, `_(...)`) is a
            // local alias convention (i18n, hyperscript, gettext). With no
            // same-file or import evidence, binding it repo-wide is a guess
            // that fabricates cross-module edges — resolve to nothing.
            if candidates.tier == ResolutionTier::Global
                && leaf_symbol_name(&reference.target_name).chars().count() <= 1
            {
                candidates.candidates.clear();
                return candidates;
            }
            let free_function_candidates = candidates
                .candidates
                .iter()
                .filter(|candidate| candidate.owner_type_name.is_none())
                .cloned()
                .collect::<Vec<_>>();
            if !free_function_candidates.is_empty() {
                candidates.candidates = free_function_candidates;
            } else if candidates.tier == ResolutionTier::Global
                && !reference_language_allows_bare_method_calls(reference, context)
            {
                // A bare call can never invoke an instance method in PHP,
                // JS/TS, or Python — `config(...)` binding to some
                // controller's `config` method fabricates a cross-module
                // edge. Ruby allows receiverless instance calls, so it keeps
                // the method candidates.
                candidates.candidates.clear();
            }
        }
        if let Some(receiver_type_name) = &reference.receiver_type_name {
            let receiver_resolution =
                resolve_receiver_type(context, &reference.file_path, receiver_type_name);
            if receiver_type_name.contains('\\') && receiver_resolution.symbol_ids.is_empty() {
                candidates.candidates.clear();
                return candidates;
            }
            if !receiver_resolution.symbol_ids.is_empty()
                || !receiver_resolution.type_names.is_empty()
                || !receiver_resolution.file_paths.is_empty()
            {
                // A non-empty receiver-consistent filter result is positive
                // proof that the call target belongs to the resolved receiver
                // type — even when it doesn't shrink the list (single global
                // candidate that IS the receiver's method). `receiver_narrowed`
                // is that proof flag, not a strictly-shrank flag.
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
                    receiver_narrowed = true;
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
                        receiver_narrowed = true;
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
                let receiver = resolve_receiver_type(context, &reference.file_path, receiver_name);
                let owner_filtered = candidates
                    .candidates
                    .iter()
                    .filter(|candidate| {
                        is_rust_module_function_candidate(reference, candidate, context)
                            || candidate
                                .parent_symbol_id
                                .as_ref()
                                .is_some_and(|id| receiver.symbol_ids.contains(id))
                            || candidate
                                .owner_type_name
                                .as_ref()
                                .is_some_and(|name| receiver.type_names.contains(name))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if !owner_filtered.is_empty() {
                    // Owner match on the scope name is receiver proof even for a
                    // single candidate (see the member-call comment above).
                    receiver_narrowed = true;
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

        // Global-tier name matching is indefensible for a call written through an
        // explicit non-self receiver unless the receiver was positively resolved
        // to a repo type that narrowed the candidates. Otherwise `Log::warning(...)`
        // (vendor facade) binds to any repo method named `warning`, and a chained
        // `->where(...)->update([...])` binds to any `update` — fabricating
        // cross-module edges that manufacture giant false SCCs. An honest
        // unresolved site beats a confident wrong edge.
        // Self receivers are included: `$this->helper()` whose method lives
        // nowhere in the receiver's resolvable type universe (same file,
        // inheritance chain) must stay unresolved instead of binding to an
        // unrelated class's same-named method. Genuine recursion and
        // same-class calls resolve at SameFile tier before this runs;
        // inherited methods resolve through receiver narrowing.
        let explicit_receiver = matches!(
            reference.call_form,
            Some(CallForm::Member | CallForm::Associated)
        ) && reference.receiver_name.is_some();
        if explicit_receiver
            && !receiver_narrowed
            && (candidates.tier == ResolutionTier::Global
                || !reference
                    .receiver_name
                    .as_deref()
                    .is_some_and(is_self_receiver_name))
        {
            // Import visibility is not evidence of the receiver's identity.
            // An unknown $operation->isNoop() must not bind an imported writer's
            // isNoop(), nor a different class in the same source file. Keep only
            // actual module-qualified free functions (e.g. Python module.run()).
            let module_files = reference
                .receiver_name
                .as_deref()
                .and_then(|name| context.resolve(name, &reference.file_path))
                .map(|resolved| {
                    resolved
                        .candidates
                        .into_iter()
                        .filter(|candidate| candidate.kind == SymbolKind::Module)
                        .map(|candidate| candidate.file_path.clone())
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();
            candidates.candidates.retain(|candidate| {
                candidate.kind == SymbolKind::Function
                    && candidate.owner_type_name.is_none()
                    && module_files.contains(&candidate.file_path)
            });
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

/// Ruby permits receiverless instance-method calls (`helper` inside a class),
/// so bare-call resolution may legitimately bind methods there. PHP, JS/TS,
/// and Python require an explicit receiver for instance methods.
/// Two languages that can statically reference each other's symbols. JS and TS
/// share one module ecosystem (and Vue SFC scripts parse as one of them), so
/// they form a single family; every other language only references itself.
fn same_language_family(left: Language, right: Language) -> bool {
    fn family(language: Language) -> u8 {
        match language {
            Language::JavaScript | Language::TypeScript => 0,
            Language::Php => 1,
            Language::Python => 2,
            Language::Ruby => 3,
            Language::Rust => 4,
        }
    }
    family(left) == family(right)
}

/// Builtin/stdlib supertypes commonly extended without an import. At global
/// tier these names denote the language builtin — binding them to a same-named
/// user class in an unrelated corner of the repo fabricates cross-module (and
/// with the language gate above, formerly cross-language) inheritance edges.
fn is_builtin_supertype_name(language: Language, leaf: &str) -> bool {
    match language {
        Language::JavaScript | Language::TypeScript => matches!(
            leaf,
            "Error"
                | "TypeError"
                | "RangeError"
                | "SyntaxError"
                | "EvalError"
                | "ReferenceError"
                | "URIError"
                | "AggregateError"
                | "Object"
                | "Array"
                | "Map"
                | "Set"
                | "WeakMap"
                | "WeakSet"
                | "Promise"
                | "Event"
                | "EventTarget"
                | "CustomEvent"
                | "Element"
                | "HTMLElement"
                | "Node"
        ),
        Language::Php => matches!(
            leaf,
            "Exception"
                | "Error"
                | "TypeError"
                | "ValueError"
                | "RuntimeException"
                | "InvalidArgumentException"
                | "LogicException"
                | "DomainException"
                | "OutOfRangeException"
                | "ArrayObject"
                | "ArrayIterator"
                | "stdClass"
        ),
        Language::Python => matches!(
            leaf,
            "Exception"
                | "BaseException"
                | "ValueError"
                | "TypeError"
                | "RuntimeError"
                | "KeyError"
                | "AttributeError"
                | "NotImplementedError"
                | "object"
        ),
        Language::Ruby => matches!(leaf, "StandardError" | "RuntimeError" | "Exception"),
        Language::Rust => false,
    }
}

fn is_rust_module_function_candidate(
    reference: &SemanticReference,
    candidate: &SymbolDefinition,
    context: &ResolutionContext,
) -> bool {
    if context.language_map.get(&reference.file_path) != Some(&Language::Rust)
        || candidate.kind != SymbolKind::Function
        || candidate.owner_type_name.is_some()
        || rust_source_root(&reference.file_path) != rust_source_root(&candidate.file_path)
    {
        return false;
    }
    let Some(receiver) = reference.receiver_name.as_deref() else {
        return false;
    };
    if context
        .declared_module_bindings
        .get(&(reference.file_path.clone(), receiver.to_owned()))
        == Some(&candidate.file_path)
    {
        return true;
    }
    if reference.file_path != candidate.file_path
        && !context
            .import_map
            .get(&reference.file_path)
            .is_some_and(|files| files.contains(&candidate.file_path))
    {
        return false;
    }
    let receiver_segments = receiver.split("::").collect::<Vec<_>>();
    normalize_rust_import_segments(&reference.file_path, &receiver_segments)
        == rust_module_segments_for_file(&candidate.file_path)
}

fn reference_language_allows_bare_method_calls(
    reference: &SemanticReference,
    context: &ResolutionContext,
) -> bool {
    context
        .language_map
        .get(&reference.file_path)
        .copied()
        .is_some_and(|language| language == Language::Ruby)
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

fn prefer_same_language_call_candidates<'a>(
    reference: &SemanticReference,
    mut candidates: TieredCandidates<'a>,
    context: &'a ResolutionContext,
) -> TieredCandidates<'a> {
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
        // Language preference is candidate hygiene, NOT resolution evidence:
        // dropping same-named candidates from other languages must not promote
        // a global name-match to import-scoped confidence. That promotion let
        // `->where(...)->update([...])` bind to an arbitrary same-language
        // `update` method at confidence 900 whenever another language also
        // defined `update`.
        candidates.candidates = same_language_candidates;
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

fn resolve_receiver_candidates<'a>(
    context: &'a ResolutionContext,
    from_file: &Path,
    receiver_type_name: &str,
) -> Vec<&'a SymbolDefinition> {
    if let Some((owner, member)) = receiver_type_name
        .split_once("::")
        .filter(|(owner, _)| owner.contains('\\'))
    {
        let owners = context.php_qualified_candidates(owner);
        return context
            .global_index
            .get(member)
            .into_iter()
            .flatten()
            .filter(|candidate| {
                candidate
                    .parent_symbol_id
                    .as_ref()
                    .is_some_and(|parent| owners.iter().any(|owner| &owner.symbol_id == parent))
            })
            .collect();
    }
    if receiver_type_name.contains('\\') && !receiver_type_name.contains("::") {
        return context.php_qualified_candidates(receiver_type_name);
    }
    if receiver_type_name.contains("::") {
        let normalized = normalize_qualified_receiver_name(context, from_file, receiver_type_name);
        if normalized != receiver_type_name && normalized.contains('\\') {
            return resolve_receiver_candidates(context, from_file, &normalized);
        }
        let normalized = if context.language_map.get(from_file) == Some(&Language::Php) {
            normalized.to_ascii_lowercase()
        } else {
            normalized
        };
        if let Some(candidates) = context.qualified_index.get(&normalized) {
            return candidates.iter().collect();
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

/// Language primitive / pseudo type names, across every language AigisCode
/// parses, that can never denote a user-defined symbol. These are reserved or
/// built-in type words: a class cannot be named `float`/`int`/`string`, and a
/// type token `self`/`static`/`void` is always the language builtin. Compared
/// case-insensitively so boxed spellings (`String`, `Object`) collapse too.
const PRIMITIVE_TYPE_NAMES: &[&str] = &[
    "null",
    "undefined",
    "none",
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
    "integer",
    "float",
    "double",
    "str",
    "array",
    "object",
    "callable",
    "iterable",
    "resource",
    "true",
    "false",
    "self",
    "static",
    "parent",
    "this",
];

pub(crate) fn is_primitive_type_name(leaf: &str) -> bool {
    let normalized = leaf.trim().to_ascii_lowercase();
    PRIMITIVE_TYPE_NAMES.contains(&normalized.as_str())
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
    // Container / generic wrapper names whose leaf token is a stdlib construct,
    // not a user domain type. Language primitives are handled separately by
    // `is_primitive_type_name` so the two lists have a single source of truth.
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
        if ignored.contains(leaf.as_str()) || is_primitive_type_name(&leaf) {
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
        ReferenceKind::TypeImport => "type-import",
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

    let mut targets = match language {
        Language::JavaScript | Language::TypeScript => resolve_javascript_import_paths(
            &reference.file_path,
            &reference.target_name,
            known_files,
            config,
        ),
        Language::Php => resolve_php_import_paths(&reference.target_name, known_files, config),
        Language::Python => resolve_python_import_paths(
            &reference.file_path,
            &reference.target_name,
            known_files,
            config,
        ),
        Language::Ruby => resolve_ruby_import_paths(
            &reference.file_path,
            &reference.target_name,
            known_files,
            config,
        ),
        Language::Rust => {
            resolve_rust_import_paths(&reference.file_path, &reference.target_name, known_files)
        }
    };

    // A file never imports itself. Fuzzy basename fallbacks (e.g. a vendor
    // `Laravel\Octane\Octane` use-statement whose tail segment `Octane.php`
    // suffix-matches the local `config/octane.php` that declares it) can
    // otherwise fabricate a self-import edge the codebase never contains.
    targets.remove(&reference.file_path);
    targets
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
            let candidate = rust_source_root(from_file).join(format!("{prefix}{suffix}"));
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
        return resolve_tsconfig_path_alias(from_file, module_target, known_files, config);
    }
    candidates.into_iter().find(|path| known_files.contains(path)).into_iter().collect()
}

fn javascript_candidate_paths(base: &Path) -> Vec<PathBuf> {
    if let Some(extension) = base.extension().and_then(OsStr::to_str) {
        let replacements: &[&str] = match extension {
            "js" => &["ts", "tsx", "d.ts", "js", "jsx"],
            "jsx" => &["tsx", "d.ts", "jsx"],
            "mjs" => &["mts", "d.mts", "mjs"],
            "cjs" => &["cts", "d.cts", "cjs"],
            _ => return vec![normalize_relative_path(base)],
        };
        return replacements.iter().map(|extension| normalize_relative_path(&base.with_extension(extension))).collect();
    }

    let mut candidates = Vec::new();
    for extension in ["ts", "tsx", "d.ts", "js", "jsx", "vue"] {
        candidates.push(normalize_relative_path(&base.with_extension(extension)));
    }
    for extension in ["ts", "tsx", "d.ts", "js", "jsx", "vue"] {
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
    // A namespaced import (`Illuminate\Support\Facades\Auth`) must keep at
    // least its trailing directory+file pair (`Facades/Auth.php`) when the
    // namespace is flattened for fuzzy matching. Allowing the bare leaf lets
    // every vendor class whose last segment collides with a repo file stem
    // (`Auth` vs `routes/auth.php`) fabricate an import edge — one such edge
    // chained the auth controllers and the entity core into a single false SCC.
    // Single-segment imports (`use Auth;`) keep leaf matching.
    let last_fuzzy_start = if segments.len() >= 2 {
        segments.len() - 1
    } else {
        segments.len()
    };
    for start in 1..last_fuzzy_start {
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
        for start in 1..last_fuzzy_start {
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

fn rust_source_root(file_path: &Path) -> &Path {
    file_path
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "src"))
        .unwrap_or(Path::new("src"))
}

fn rust_module_segments_for_file(file_path: &Path) -> Vec<String> {
    let mut segments = file_path
        .strip_prefix(rust_source_root(file_path))
        .unwrap_or(file_path)
        .iter()
        .map(|segment| segment.to_string_lossy().to_string())
        .collect::<Vec<_>>();

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

fn load_composer_psr4(root: &Path, path: &Path, json: &Value) -> Vec<ComposerPsr4Mapping> {
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
    from_file: &Path,
    import_target: &str,
    known_files: &HashSet<PathBuf>,
    config: &ResolveConfig,
) -> HashSet<PathBuf> {
    let mut projects = config.ts_projects.iter().filter(|project| project.contains(from_file)).collect::<Vec<_>>();
    if projects.is_empty() {
        // Include/exclude select root files; an imported dependency can still
        // belong to the nearest project even when it was not a root file.
        projects.extend(config.ts_projects.iter().filter(|project| from_file.starts_with(&project.directory)));
    }
    let depth = projects.iter().map(|project| project.directory.components().count()).max();
    let mut projects = projects.into_iter().filter(|project| Some(project.directory.components().count()) == depth);
    let project = projects.next();
    if let Some(selected) = project {
        // A file can belong to multiple configured projects. Do not pick a
        // conflicting interpretation by traversal order.
        if projects.any(|other| other.aliases != selected.aliases || other.base_url != selected.base_url) {
            return HashSet::new();
        }
    }
    let aliases = project.map_or(config.tsconfig_paths.as_slice(), |project| project.aliases.as_slice());
    let selected = aliases.iter().filter_map(|alias| {
        match_ts_path_pattern(&alias.pattern, import_target).map(|wildcard| (alias, wildcard))
    }).max_by_key(|(alias, wildcard)| (wildcard.is_none(), alias.pattern.find('*').unwrap_or(alias.pattern.len())));
    if let Some((alias, wildcard)) = selected {
        for target in &alias.targets {
            let substituted = apply_ts_path_target(target, wildcard);
            if let Some(path) = javascript_candidate_paths(&normalize_relative_path(&alias.base_dir.join(substituted)))
                .into_iter().find(|path| known_files.contains(path)) {
                return HashSet::from([path]);
            }
        }
    }
    project.and_then(|project| project.base_url.as_ref()).and_then(|base| {
        javascript_candidate_paths(&normalize_relative_path(&base.join(import_target)))
            .into_iter().find(|path| known_files.contains(path))
    }).into_iter().collect()
}

fn match_ts_path_pattern<'a>(pattern: &'a str, import_target: &'a str) -> Option<Option<&'a str>> {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        if import_target.len() >= prefix.len() + suffix.len()
            && import_target.starts_with(prefix) && import_target.ends_with(suffix) {
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

fn discover_roots(
    root: &Path,
    candidates: &[&str],
    reader: &mut tsconfig::ConfigReader<'_>,
) -> Result<Vec<PathBuf>, ResolveConfigError> {
    let mut paths = Vec::new();
    for candidate in candidates {
        if reader.is_dir(&root.join(candidate))? {
            paths.push(PathBuf::from(candidate));
        }
    }
    Ok(paths)
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
        load_resolve_config, resolve_graph, resolve_graph_with_config, resolve_php_import_paths,
        ComposerPsr4Mapping, HashSet, ResolutionContext, ResolutionTier, ResolveConfig,
        TsPathAlias,
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
    fn resolves_bound_arrows_in_their_lexical_scope_and_preserves_imports() {
        let mut graph = parse_javascript_to_graph(PathBuf::from("popup.ts"), r#"
import { format } from './lib';
export function setup() {
    const copyText = async (text: string) => format(text);
    const child = () => copyText('nested');
    copyText('direct');
    child();
    const onlyLocal = () => 1;
    onlyLocal();
}
function parameters(copyText: (text: string) => void) { copyText('unknown'); }
function sibling() { onlyLocal(); }
function shadowImport() { const format = () => 'private'; return format(); }
"#, true).unwrap();
        graph.append(parse_javascript_to_graph(PathBuf::from("lib.ts"),
            "export const format = (text: string) => text; format('module');", true).unwrap());
        graph.append(parse_javascript_to_graph(PathBuf::from("foreign.ts"),
            "export function copyText(text: string) { return text; }", true).unwrap());
        resolve_graph(&mut graph);
        let copy_calls = graph.resolved_edges.iter().filter(|edge| {
            edge.reference_target_name.as_deref() == Some("copyText") && edge.kind == ReferenceKind::Call
        }).collect::<Vec<_>>();
        assert_eq!(copy_calls.len(), 2);
        assert!(copy_calls.iter().all(|edge| {
            edge.target_file_path == Path::new("popup.ts") && edge.reason == "call:lexical-binding"
                && edge.strength == crate::graph::EdgeStrength::Hard
        }));
        assert_eq!(graph.resolved_edges.iter().filter(|edge| {
            edge.reference_target_name.as_deref() == Some("onlyLocal") && edge.kind == ReferenceKind::Call
        }).count(), 1);
        assert!(graph.resolved_edges.iter().any(|edge| {
            edge.source_file_path == Path::new("popup.ts") && edge.target_file_path == Path::new("lib.ts")
                && edge.reference_target_name.as_deref() == Some("format") && edge.kind == ReferenceKind::Call
                && edge.resolution_tier == ResolutionTier::ImportScoped
        }));
        // Appending lib.ts must rebase its call binding to its own reference.
        assert!(graph.resolved_edges.iter().any(|edge| {
            edge.source_file_path == Path::new("lib.ts") && edge.target_file_path == Path::new("lib.ts")
                && edge.reason == "call:lexical-binding"
        }));
    }

    #[test]
    fn preserves_same_line_block_identity_and_masks_reassigned_values() {
        let mut graph = parse_javascript_to_graph(PathBuf::from("scope.js"), r#"
function blocks() { { const local = () => 1; local(); } { const local = () => 2; local(); } local(); }
const recur = function internal(n) { return n ? internal(n - 1) : 0; };
recur(1); internal(1);
let changed = () => 1;
changed = unknown;
changed();
function masked({ recur }) { recur(); }
const single = value => value;
single(1);
const loop = () => 1;
for (const loop of values) loop();
loop();
"#, false).unwrap();
        resolve_graph(&mut graph);
        let local_calls = graph.resolved_edges.iter().filter(|edge| {
            edge.reference_target_name.as_deref() == Some("local")
        }).collect::<Vec<_>>();
        assert_eq!(local_calls.len(), 2);
        assert_ne!(local_calls[0].target_symbol_id, local_calls[1].target_symbol_id);
        assert_eq!(local_calls[0].occurrence_index, 0);
        assert_eq!(local_calls[1].occurrence_index, 1);
        let internal = graph.resolved_edges.iter().filter(|edge| {
            edge.reference_target_name.as_deref() == Some("internal")
        }).collect::<Vec<_>>();
        assert_eq!(internal.len(), 1);
        assert!(internal[0].target_symbol_id.ends_with(":recur"));
        assert_eq!(graph.resolved_edges.iter().filter(|edge| edge.reference_target_name.as_deref() == Some("recur")).count(), 1);
        assert_eq!(graph.resolved_edges.iter().filter(|edge| edge.reference_target_name.as_deref() == Some("loop")).count(), 1);
        assert!(!graph.resolved_edges.iter().any(|edge| edge.reference_target_name.as_deref() == Some("changed")));
        assert!(graph.lexical_bindings.calls.iter().any(|binding| {
            binding.argument_index.is_none() && binding.target_symbol_id.is_none()
                && graph.references[binding.reference_index].target_name == "changed"
        }));
        assert_eq!(graph.symbols.iter().find(|symbol| symbol.name == "single").unwrap().parameter_count, 1);
    }

    #[test]
    fn resolves_local_classes_without_crossing_scope_or_receiver_kind() {
        let source = r#"
function first() {
    class Local {
        run() { this.own(); const arrow = () => this.own(); arrow(); }
        own() {}
        static build() { this.make(); }
        static make() {}
    }
    const instance = new Local();
    instance.run();
    new Local().own();
    Local.build();
    Local.run();
    instance.build();
    function typed(value: Local) {}
    function generic<Local>(value: Local) {}
    class Child extends Local {}
    let changed = new Local();
    changed = unknown;
    changed.run();
    { class Local { own() {} } const nested = new Local(); nested.own(); instance.own(); }
}
function second() { class Local { own() {} } new Local().own(); }
function outside() { new Local(); }
function shadow(Local: unknown) { new Local(); }
"#;
        let mut graph = parse_javascript_to_graph(PathBuf::from("classes.ts"), source, true).unwrap();
        graph.append(parse_javascript_to_graph(PathBuf::from("other.ts"),
            "function foreign() { new Local(); }", true).unwrap());
        resolve_graph(&mut graph);
        let symbols = graph.symbols.iter().map(|symbol| (symbol.id.as_str(), symbol)).collect::<std::collections::HashMap<_, _>>();
        assert_eq!(symbols.len(), graph.symbols.len());
        let first = graph.symbols.iter().find(|symbol| symbol.name == "first").unwrap();
        let first_class = graph.symbols.iter().find(|symbol| symbol.name == "Local"
            && symbol.parent_symbol_id.as_deref() == Some(first.id.as_str())
            && symbol.end_line > symbol.start_line).unwrap();
        let line = |text: &str| source.lines().position(|value| value.contains(text)).unwrap() + 1;
        for text in ["instance.run();", "new Local().own();", "Local.build();"] {
            let edge = graph.resolved_edges.iter().find(|edge| edge.source_file_path == Path::new("classes.ts")
                && edge.line == line(text) && symbols[edge.target_symbol_id.as_str()].kind == SymbolKind::Method).unwrap();
            assert_eq!(symbols[edge.target_symbol_id.as_str()].parent_symbol_id.as_deref(), Some(first_class.id.as_str()));
            assert_eq!(edge.reason, "call:lexical-binding");
        }
        for text in ["Local.run();", "instance.build();", "changed.run();", "function outside()", "function shadow("] {
            assert!(!graph.resolved_edges.iter().any(|edge| edge.source_file_path == Path::new("classes.ts")
                && edge.line == line(text) && edge.kind == ReferenceKind::Call));
        }
        assert!(!graph.resolved_edges.iter().any(|edge| edge.source_file_path == Path::new("other.ts")));
        let nested_line = line("const nested");
        let owners = graph.resolved_edges.iter().filter(|edge| edge.line == nested_line
            && edge.reference_target_name.as_deref() == Some("own"))
            .map(|edge| symbols[edge.target_symbol_id.as_str()].parent_symbol_id.clone()).collect::<std::collections::HashSet<_>>();
        assert_eq!(owners.len(), 2);
        for text in ["function typed", "class Child extends"] {
            assert!(graph.resolved_edges.iter().any(|edge| edge.line == line(text)
                && edge.target_symbol_id == first_class.id && edge.reason == "reference:lexical-binding"));
        }
        assert!(!graph.resolved_edges.iter().any(|edge| edge.line == line("function generic")));
        assert!(graph.lexical_bindings.named_references.iter().all(|binding| {
            graph.references[binding.reference_index].file_path == Path::new("classes.ts")
        }));
    }

    #[test]
    fn unknown_local_instances_do_not_rebind_to_a_foreign_class() {
        let mut graph = parse_javascript_to_graph(PathBuf::from("local.ts"), r#"
function setup() {
    class Local { run() {} }
    let changed = new Local();
    changed = unknown;
    changed.run();
    function typed(value: Local) { value.run(); }
    function dynamic() { this.run(); }
}
"#, true).unwrap();
        let offset = graph.references.len();
        graph.append(parse_javascript_to_graph(PathBuf::from("foreign.ts"),
            "class Local { run() {} } type Alias<Local> = Local[]; function typed(value: Local) {}", true).unwrap());
        resolve_graph(&mut graph);
        assert!(!graph.resolved_edges.iter().any(|edge| edge.source_file_path == Path::new("local.ts")
            && edge.reference_target_name.as_deref() == Some("run")));
        assert!(graph.lexical_bindings.named_references.iter().any(|binding| {
            binding.reference_index >= offset
                && graph.references[binding.reference_index].file_path == Path::new("foreign.ts")
                && binding.target_symbol_id.as_ref().is_some_and(|id| id.contains("foreign.ts"))
        }));
    }

    #[test]
    fn rust_module_calls_require_the_matching_imported_module() {
        let mut graph = crate::graph::SemanticGraph::default();
        for (path, source) in [
            (
                "crates/app/src/main.rs",
                "mod one; mod two; fn main() { one::helper(); two::helper(); unknown::helper(); }",
            ),
            ("crates/app/src/one.rs", "pub fn helper() {}"),
            ("crates/app/src/two.rs", "pub fn other() {}"),
            ("crates/other/src/one.rs", "pub fn helper() {}"),
        ] {
            let parsed =
                crate::parsing::rust::parse_rust_to_graph(PathBuf::from(path), source).unwrap();
            graph.files.extend(parsed.files);
            graph.symbols.extend(parsed.symbols);
            graph.references.extend(parsed.references);
        }
        resolve_graph(&mut graph);
        let calls = graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.kind == ReferenceKind::Call)
            .collect::<Vec<_>>();
        assert_eq!(calls.len(), 1, "{calls:#?}");
        assert_eq!(
            calls[0].target_file_path,
            Path::new("crates/app/src/one.rs")
        );
    }

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

    // A TS `extends Error` names the JS builtin — it must not bind to a PHP
    // class named `Error` elsewhere in the repo (cross-language inheritance
    // is impossible), nor to any global-tier same-language user class.
    #[test]
    fn builtin_supertype_extends_never_resolves_cross_language_or_globally() {
        let mut graph = parse_javascript_to_graph(
            PathBuf::from("resources/js/e2ee.ts"),
            "export class KeyMismatchError extends Error {\n  constructor() { super(); }\n}\n",
            true,
        )
        .unwrap();
        let mut php = parse_php_to_graph(
            PathBuf::from("app/Support/Error.php"),
            "<?php\nnamespace App\\Support;\n\nfinal class Error\n{\n}\n",
        )
        .unwrap();
        graph.files.append(&mut php.files);
        graph.symbols.append(&mut php.symbols);
        graph.references.append(&mut php.references);

        resolve_graph(&mut graph);

        assert!(!graph.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Extends
                && edge.target_file_path == Path::new("app/Support/Error.php")
        }));
    }

    // `new Error(...)` in TS is a constructor call to the JS builtin — it must
    // not resolve to a PHP class named `Error` (cross-family), yet a real
    // Vue(TS)->TS import of a user class must still resolve.
    #[test]
    fn constructor_calls_do_not_cross_language_families() {
        let mut graph = parse_javascript_to_graph(
            PathBuf::from("resources/js/keyStore.ts"),
            "export function fail(): void {\n  throw new Error('boom');\n}\n",
            true,
        )
        .unwrap();
        let mut php = parse_php_to_graph(
            PathBuf::from("app/Modules/Pohoda/Support/Error.php"),
            "<?php\nnamespace App\\Modules\\Pohoda\\Support;\n\nclass Error\n{\n    public function __construct(string $m = '') {}\n}\n",
        )
        .unwrap();
        graph.files.append(&mut php.files);
        graph.symbols.append(&mut php.symbols);
        graph.references.append(&mut php.references);

        resolve_graph(&mut graph);

        assert!(!graph.resolved_edges.iter().any(|edge| {
            edge.target_file_path == Path::new("app/Modules/Pohoda/Support/Error.php")
                && edge.source_file_path == Path::new("resources/js/keyStore.ts")
        }));
    }

    // A destructured i18n alias `t(...)` must not bind repo-wide to a nested
    // helper `function t()` in a foreign module: nested functions are
    // scope-local (excluded from the global tier) and one-char callees carry
    // no global-binding evidence at all.
    #[test]
    fn one_char_free_calls_and_nested_functions_never_bind_globally() {
        let mut graph = parse_javascript_to_graph(
            PathBuf::from("app/Modules/WarehouseMobile/resources/js/Index.vue.ts"),
            "const { t } = useTranslation();\nexport function label(): string {\n  return t('warehouse.title');\n}\n",
            true,
        )
        .unwrap();
        let mut other = parse_javascript_to_graph(
            PathBuf::from("app/Modules/Website/resources/js/fullPreviewOverlay.ts"),
            "export function overlay(): void {\n  function t(key: string): string {\n    return key;\n  }\n  t('x');\n}\n",
            true,
        )
        .unwrap();
        graph.files.append(&mut other.files);
        graph.symbols.append(&mut other.symbols);
        graph.references.append(&mut other.references);

        resolve_graph(&mut graph);

        assert!(!graph.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call
                && edge.source_file_path
                    == Path::new("app/Modules/WarehouseMobile/resources/js/Index.vue.ts")
                && edge.target_file_path
                    == Path::new("app/Modules/Website/resources/js/fullPreviewOverlay.ts")
        }));
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
            class_literal_argument: None,
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
            class_literal_argument: None,
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
        assert_eq!(
            self_loops[0].line, 5,
            "the surviving self-loop is the recursion"
        );
    }

    #[test]
    fn test_stubs_are_not_hard_production_dependencies() {
        let mut graph = SemanticGraph::default();
        for (path, source) in [
            ("app/Work.php", "<?php namespace App; use Vendor\\Connection; class Work { public function run(Connection $db) { $db->query(); } }"),
            ("tests/Connection.php", "<?php namespace Vendor; use App\\Work; class Connection { public function query() {} }"),
        ] {
            let parsed = parse_php_to_graph(PathBuf::from(path), source).unwrap();
            graph.files.extend(parsed.files);
            graph.symbols.extend(parsed.symbols);
            graph.references.extend(parsed.references);
        }
        resolve_graph(&mut graph);
        let candidates = graph
            .resolved_edges
            .iter()
            .filter(|edge| {
                edge.source_file_path == Path::new("app/Work.php")
                    && edge.target_file_path == Path::new("tests/Connection.php")
            })
            .collect::<Vec<_>>();
        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|edge| edge.strength == crate::graph::EdgeStrength::Inferred
                && edge.reason.contains("production visibility is unproven")));
        let analysis = crate::graph::analysis::analyze_semantic_graph(
            &graph,
            &crate::ingestion::scan::AnalysisScope::default(),
        );
        assert_eq!(analysis.cycle_findings.len(), 1);
        assert!(analysis.strong_cycle_findings.is_empty());
    }

    #[test]
    fn php_imports_use_declared_identity_and_keep_same_line_bindings_distinct() {
        let mut graph = SemanticGraph::default();
        for (path, source) in [
            ("src/caller.php", r#"<?php namespace App;
use Domain\Worker as Known; use Vendor\Worker as Missing; use domain\worker as LowerCase; use Domain\Other as Other;
function run(Known $known, Missing $missing) {
    $known->work(); $missing->work(); app(Missing::class)->work(); LowerCase::build();
}"#),
            ("odd/layout.php", "<?php namespace Domain; class Worker { public function work() {} public static function build() {} }"),
            ("elsewhere/other.php", "<?php namespace Domain; class Other {}"),
            ("decoy.php", "<?php namespace VendorMock; class Worker { public function work() {} }"),
        ] {
            let parsed = parse_php_to_graph(PathBuf::from(path), source).unwrap();
            graph.files.extend(parsed.files);
            graph.symbols.extend(parsed.symbols);
            graph.references.extend(parsed.references);
        }
        resolve_graph(&mut graph);
        let imports = graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.kind.is_import())
            .collect::<Vec<_>>();
        assert_eq!(imports.len(), 3);
        let calls = graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.kind == ReferenceKind::Call)
            .collect::<Vec<_>>();
        assert_eq!(calls.len(), 2);
        assert!(calls
            .iter()
            .all(|edge| edge.target_file_path == Path::new("odd/layout.php")));
        let symbols = graph
            .symbols
            .iter()
            .map(|symbol| (symbol.id.clone(), symbol))
            .collect();
        let bindings = crate::plugins::import_targets_by_binding(&graph, &symbols, |symbol| {
            symbol.kind == SymbolKind::Class
        });
        let binding =
            |name: &str| bindings.get(&(PathBuf::from("src/caller.php"), name.to_owned()));
        assert_eq!(binding("Known").unwrap().1, PathBuf::from("odd/layout.php"));
        assert_eq!(
            binding("Other").unwrap().1,
            PathBuf::from("elsewhere/other.php")
        );
        assert!(binding("Missing").is_none());
    }

    #[test]
    fn type_only_import_closes_only_the_type_dependency_graph() {
        let mut graph = SemanticGraph::default();
        for (path, source) in [
            (
                "src/a.ts",
                "import type { B } from './b'; export const value = 1; export type A = B;",
            ),
            (
                "src/b.ts",
                "import { value } from './a'; export interface B {} export const result = value;",
            ),
        ] {
            let parsed = parse_javascript_to_graph(PathBuf::from(path), source, true).unwrap();
            graph.files.extend(parsed.files);
            graph.symbols.extend(parsed.symbols);
            graph.references.extend(parsed.references);
        }
        resolve_graph(&mut graph);
        assert!(graph
            .resolved_edges
            .iter()
            .any(|edge| edge.kind == ReferenceKind::TypeImport
                && edge.target_file_path == Path::new("src/b.ts")));
        let analysis = crate::graph::analysis::analyze_semantic_graph(
            &graph,
            &crate::ingestion::scan::AnalysisScope::default(),
        );
        assert_eq!(analysis.cycle_findings.len(), 1);
        assert!(analysis.strong_cycle_findings.is_empty());
    }

    #[test]
    fn imports_and_same_file_methods_do_not_prove_an_unknown_receiver() {
        let sources = [
            (
                "app/Writer.php",
                r#"<?php namespace App;
class Writer {
    public function isNoop(?string $existing, array $additions): bool { return false; }
    public static function create(): void {}
}"#,
            ),
            (
                "app/Runner.php",
                r#"<?php namespace App;
use App\Writer as W;
class Runner {
    public function run($operation, W $writer): void {
        $operation->isNoop();
        $writer->isNoop(null, []);
        W::create();
        $operation->localHelper();
        $this->localHelper();
    }
    private function localHelper(): void {}
}"#,
            ),
        ];
        let mut graph = SemanticGraph::default();
        for (path, source) in sources {
            let parsed =
                crate::parsing::php::parse_php_to_graph(PathBuf::from(path), source).unwrap();
            graph.files.extend(parsed.files);
            graph.symbols.extend(parsed.symbols);
            graph.references.extend(parsed.references);
        }
        resolve_graph(&mut graph);
        let call_lines = graph
            .resolved_edges
            .iter()
            .filter(|edge| {
                edge.source_file_path == Path::new("app/Runner.php")
                    && edge.kind == ReferenceKind::Call
            })
            .map(|edge| edge.line)
            .collect::<Vec<_>>();
        assert_eq!(call_lines, vec![6, 7, 9]);
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
            class_literal_argument: None,
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
            class_literal_argument: None,
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
            class_literal_argument: None,
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
            class_literal_argument: None,
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
            class_literal_argument: None,
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
            ..ResolveConfig::default()
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
            ..ResolveConfig::default()
        };
        resolve_graph_with_config(&mut service, &config);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("src/domain/models.py")
        }));
    }

    #[test]
    fn resolves_php_namespace_imports_by_declared_identity() {
        let mut service = parse_php_to_graph(
            PathBuf::from("app/Service.php"),
            r#"<?php
use App\Models\User;
"#,
        )
        .unwrap();
        let mut user = parse_php_to_graph(
            PathBuf::from("app/Models/User.php"),
            r#"<?php namespace App\Models; class User {}"#,
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
            r#"<?php namespace Acme\Models; class User {}"#,
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
            ..ResolveConfig::default()
        };
        resolve_graph_with_config(&mut service, &config);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("app/Models/User.php")
        }));
    }

    #[test]
    fn type_reference_to_language_primitive_does_not_resolve_to_same_named_method() {
        let mut service = parse_php_to_graph(
            PathBuf::from("app/Factory.php"),
            r#"<?php
class Factory {
    private function float(mixed $value, float $default): float {
        return is_numeric($value) ? (float) $value : $default;
    }
    private function scale(float $value): float {
        return $this->float($value, 1.0);
    }
}
"#,
        )
        .unwrap();
        resolve_graph(&mut service);

        assert!(
            !service.resolved_edges.iter().any(|edge| {
                edge.kind == ReferenceKind::Type && edge.target_symbol_id.ends_with(":float")
            }),
            "primitive type `float` must not resolve to the same-named method: {:?}",
            service.resolved_edges
        );
        // The genuine recursive call still resolves as a Call edge.
        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Call && edge.target_symbol_id.ends_with(":float")
        }));
    }

    #[test]
    fn ambiguous_module_imports_resolve_deterministically() {
        // `./types` matches both `types.ts` and `types/index.ts`. The winner
        // must be stable across runs (lexicographically first target), not
        // whichever a HashSet happens to yield first.
        let build = || {
            let mut app = parse_javascript_to_graph(
                PathBuf::from("src/app.ts"),
                "import { Message } from './types';\n",
                true,
            )
            .unwrap();
            let mut flat = parse_javascript_to_graph(
                PathBuf::from("src/types.ts"),
                "export type Message = string;\n",
                true,
            )
            .unwrap();
            let mut dir = parse_javascript_to_graph(
                PathBuf::from("src/types/index.ts"),
                "export type Message = string;\n",
                true,
            )
            .unwrap();
            app.files.append(&mut flat.files);
            app.files.append(&mut dir.files);
            app.symbols.append(&mut flat.symbols);
            app.symbols.append(&mut dir.symbols);
            app.references.append(&mut flat.references);
            app.references.append(&mut dir.references);
            resolve_graph(&mut app);
            app.resolved_edges
                .iter()
                .filter(|edge| edge.kind == ReferenceKind::Import)
                .map(|edge| edge.target_file_path.clone())
                .collect::<Vec<_>>()
        };

        let first = build();
        assert!(!first.is_empty(), "import should resolve");
        for _ in 0..5 {
            assert_eq!(build(), first, "import resolution must be deterministic");
        }
    }

    #[test]
    fn import_never_resolves_to_the_importing_file() {
        // A vendor use-statement whose tail basename collides with the declaring
        // config file must not fabricate a self-import edge.
        let mut service = parse_php_to_graph(
            PathBuf::from("config/octane.php"),
            r#"<?php
use Laravel\Octane\Octane;
return [];
"#,
        )
        .unwrap();
        resolve_graph(&mut service);

        assert!(
            !service.resolved_edges.iter().any(|edge| {
                edge.kind == ReferenceKind::Import
                    && edge.source_file_path == Path::new("config/octane.php")
                    && edge.target_file_path == Path::new("config/octane.php")
            }),
            "a file must never import itself: {:?}",
            service.resolved_edges
        );
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
            ..ResolveConfig::default()
        };
        resolve_graph_with_config(&mut service, &config);

        assert!(service.resolved_edges.iter().any(|edge| {
            edge.kind == ReferenceKind::Import
                && edge.target_file_path == Path::new("lib/support/user.rb")
                && edge.target_symbol_id == "module:lib/support/user.rb"
        }));
    }

    #[test]
    fn namespaced_php_import_never_binds_to_bare_leaf_stem_file() {
        let known_files: HashSet<PathBuf> = [
            PathBuf::from("routes/auth.php"),
            PathBuf::from("app/Support/Facades/Auth.php"),
        ]
        .into_iter()
        .collect();
        let config = ResolveConfig::default();
        // Vendor facade: leaf `Auth` must not bind to `routes/auth.php`.
        let vendor =
            resolve_php_import_paths("Illuminate\\Support\\Facades\\Auth", &known_files, &config);
        assert!(
            !vendor.contains(Path::new("routes/auth.php")),
            "vendor namespace leaf must not stem-match a module file, got {vendor:?}"
        );
        // The trailing directory+file pair still fuzzy-matches a real repo class.
        assert!(vendor.contains(Path::new("app/Support/Facades/Auth.php")));
        // Single-segment import keeps leaf matching.
        let bare = resolve_php_import_paths("Auth", &known_files, &config);
        assert!(bare.contains(Path::new("app/Support/Facades/Auth.php")));
    }

    #[test]
    fn global_tier_never_binds_explicit_receiver_calls_without_receiver_proof() {
        let mut graph = SemanticGraph::default();
        let mut push_method = |file: &str, owner: &str, name: &str| {
            graph.symbols.push(SymbolNode {
                id: format!("method:{file}:{owner}:{name}"),
                file_path: PathBuf::from(file),
                kind: SymbolKind::Method,
                name: String::from(name),
                qualified_name: format!("{owner}::{name}"),
                parent_symbol_id: Some(format!("class:{file}:{owner}")),
                owner_type_name: Some(String::from(owner)),
                visibility: Visibility::Public,
                parameter_count: 2,
                required_parameter_count: 0,
                return_type_name: None,
                start_line: 1,
                end_line: 2,
            });
        };
        // An unrelated repo class that happens to define `warning` and `update`.
        push_method("app/Unrelated.php", "Unrelated", "warning");
        push_method("app/Unrelated.php", "Unrelated", "update");
        // A repo class the Associated call CAN prove its receiver against.
        push_method("app/Known.php", "Known", "update");

        let make_call = |line: usize,
                         target: &str,
                         receiver: &str,
                         form: CallForm,
                         receiver_type: Option<&str>| SemanticReference {
            file_path: PathBuf::from("app/Service.php"),
            enclosing_symbol_id: None,
            kind: ReferenceKind::Call,
            target_name: String::from(target),
            binding_name: None,
            line,
            arity: Some(2),
            receiver_name: Some(String::from(receiver)),
            receiver_type_name: receiver_type.map(str::to_owned),
            call_form: Some(form),
            class_literal_argument: None,
        };
        // Vendor facade static call: `Log::warning(...)` — `Log` is not a repo type.
        graph
            .references
            .push(make_call(3, "warning", "Log", CallForm::Associated, None));
        // Chained builder member call: `...->where(...)->update([...])` — unknown receiver.
        graph.references.push(make_call(
            4,
            "update",
            "DB::connection('tenant')->table('t')->where('id', $id)",
            CallForm::Member,
            None,
        ));
        // Positively-owned static call: `Known::update(...)` must still resolve.
        graph
            .references
            .push(make_call(5, "update", "Known", CallForm::Associated, None));
        // Bare free call `warning(...)` in PHP can never invoke an instance
        // method — must not bind to Unrelated::warning either.
        graph.files.push(FileNode {
            path: PathBuf::from("app/Service.php"),
            language: Language::Php,
        });
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("app/Service.php"),
            enclosing_symbol_id: None,
            kind: ReferenceKind::Call,
            target_name: String::from("warning"),
            binding_name: None,
            line: 6,
            arity: Some(2),
            receiver_name: None,
            receiver_type_name: None,
            call_form: Some(CallForm::Free),
            class_literal_argument: None,
        });
        // Self-receiver call to a method defined nowhere in this class's
        // resolvable universe must stay unresolved, not bind globally.
        graph.references.push(SemanticReference {
            file_path: PathBuf::from("app/Service.php"),
            enclosing_symbol_id: None,
            kind: ReferenceKind::Call,
            target_name: String::from("update"),
            binding_name: None,
            line: 7,
            arity: Some(2),
            receiver_name: Some(String::from("$this")),
            receiver_type_name: None,
            call_form: Some(CallForm::Member),
            class_literal_argument: None,
        });

        resolve_graph(&mut graph);

        assert!(
            !graph
                .resolved_edges
                .iter()
                .any(|edge| edge.target_file_path == Path::new("app/Unrelated.php")),
            "explicit-receiver calls with no receiver proof must stay unresolved, got: {:?}",
            graph
                .resolved_edges
                .iter()
                .map(|edge| (&edge.target_symbol_id, edge.line))
                .collect::<Vec<_>>()
        );
        assert!(graph
            .resolved_edges
            .iter()
            .any(|edge| { edge.line == 5 && edge.target_file_path == Path::new("app/Known.php") }));
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

        let config = load_resolve_config(&fixture, &[]).unwrap();

        assert_eq!(config.ts_projects.len(), 1);
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
            class_literal_argument: None,
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
            class_literal_argument: None,
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
