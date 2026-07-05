use crate::contracts::ContractInventory;
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
    /// A frontend module (component, composable, utility) that nothing in the
    /// analyzed corpus references: no inbound resolved edge, no import
    /// specifier whose tail matches its name, and not covered by any
    /// `import.meta.glob` prefix or framework auto-loading convention. If the
    /// repository's tests are excluded from the scan, a test-only module also
    /// lands here — dead from production's point of view either way.
    OrphanModule,
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
    /// Deletion confidence for orphan findings: safe_delete (explicit import
    /// graph, every channel checked) | probably_delete (autoload world —
    /// dynamic construction from strings outside the repo stays possible).
    /// Empty for non-orphan categories.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub delete_verdict: String,
    /// The suppression channels that came back silent — the positive evidence
    /// a deleter reviews before acting.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delete_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DeadCodeResult {
    pub findings: Vec<DeadCodeFinding>,
}

pub fn analyze_dead_code(
    graph: &SemanticGraph,
    parsed_sources: &[(PathBuf, String)],
    contract_inventory: &ContractInventory,
    repo_root: &Path,
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
            delete_verdict: String::new(),
            delete_evidence: Vec::new(),
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
            delete_verdict: String::new(),
            delete_evidence: Vec::new(),
        });
    }

    findings.extend(detect_orphan_modules(graph, parsed_sources));
    findings.extend(detect_backend_orphan_modules(
        graph,
        parsed_sources,
        contract_inventory,
        repo_root,
    ));

    findings.sort_by(|left, right| {
        left.file_path
            .cmp(&right.file_path)
            .then(left.line.cmp(&right.line))
            .then(left.name.cmp(&right.name))
    });

    DeadCodeResult { findings }
}

const FRONTEND_MODULE_EXTENSIONS: &[&str] = &["vue", "ts", "tsx", "js", "jsx", "mjs", "cjs"];

/// Frontend modules that nothing references. Backend files are exempt because
/// server frameworks autoload by convention (PSR-4, Rails constants), so a
/// missing inbound edge proves nothing there; the frontend import graph is
/// explicit, which makes "no importer anywhere" meaningful evidence.
fn detect_orphan_modules(
    graph: &SemanticGraph,
    parsed_sources: &[(PathBuf, String)],
) -> Vec<DeadCodeFinding> {
    // Any cross-file resolved edge into a file proves it is alive.
    let inbound_files = graph
        .resolved_edges
        .iter()
        .filter(|edge| edge.source_file_path != edge.target_file_path)
        .map(|edge| edge.target_file_path.as_path())
        .collect::<HashSet<_>>();

    // Import specifier tails — including imports that never resolved. If any
    // import anywhere ends in the candidate's stem, the module is considered
    // referenced even when resolution failed (aliases, unusual roots).
    let mut import_tails = HashSet::new();
    for reference in graph
        .references
        .iter()
        .filter(|reference| reference.kind == ReferenceKind::Import)
    {
        let module_specifier = reference
            .target_name
            .split("::")
            .next()
            .unwrap_or_default()
            .trim_end_matches('/');
        let tail = module_specifier
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .trim();
        if !tail.is_empty() {
            import_tails.insert(strip_frontend_extension(tail).to_ascii_lowercase());
        }
    }

    // Modules are also loaded through plain string paths that never surface as
    // import references: `new Worker(new URL('./renderWorker.ts', ...))`,
    // `audioWorklet.addModule('../worklets/processor.js')`, re-export
    // specifiers the parser misses. Any quoted relative/alias path literal in
    // frontend source contributes its tail. Suppression-only, so the looseness
    // of a lexical scan cannot fabricate a finding.
    for (path, source) in parsed_sources {
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if !FRONTEND_MODULE_EXTENSIONS.contains(&extension) {
            continue;
        }
        for captures in path_literal_pattern().captures_iter(source) {
            let Some(literal) = captures.get(1).map(|m| m.as_str()) else {
                continue;
            };
            let tail = literal
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .trim();
            if !tail.is_empty() {
                import_tails.insert(strip_frontend_extension(tail).to_ascii_lowercase());
            }
        }
    }

    // Files under a static `import.meta.glob('...')` prefix are lazily loaded
    // by the bundler even though no explicit import names them.
    let glob_prefixes = collect_import_meta_glob_prefixes(parsed_sources);

    let mut findings = Vec::new();
    for (path, _) in parsed_sources {
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if !FRONTEND_MODULE_EXTENSIONS.contains(&extension) {
            continue;
        }
        if is_orphan_exempt_path(path) {
            continue;
        }
        if inbound_files.contains(path.as_path()) {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let stem = strip_frontend_extension(file_name);
        if stem.is_empty() || import_tails.contains(&stem.to_ascii_lowercase()) {
            continue;
        }
        let normalized = normalized_path(path);
        if glob_prefixes
            .iter()
            .any(|prefix| normalized.contains(prefix.as_str()))
        {
            continue;
        }
        findings.push(DeadCodeFinding {
            category: DeadCodeCategory::OrphanModule,
            symbol_id: format!("module:{normalized}"),
            file_path: path.clone(),
            name: stem.to_owned(),
            line: 1,
            proof_tier: dead_code_proof_tier(DeadCodeCategory::OrphanModule),
            fingerprint: dead_code_fingerprint(DeadCodeCategory::OrphanModule, path, stem),
            delete_verdict: String::from("safe_delete"),
            delete_evidence: vec![
                String::from("no inbound resolved edge anywhere in the corpus"),
                String::from("no import specifier tail matches the module stem"),
                String::from("no quoted path literal (worker URL, addModule, re-export) names it"),
                String::from("not covered by any import.meta.glob prefix"),
                String::from("not a framework convention path (pages/routes/config/entry stems)"),
            ],
        });
    }
    findings
}

const BACKEND_ORPHAN_EXTENSIONS: &[&str] = &["php"];

/// Backend (PHP) files that nothing in the analyzed corpus provably reaches.
/// Autoloading makes every class loadable by string, so this stays Heuristic
/// tier and stacks suppression channels: any inbound resolved edge, any
/// contract declaration in the file (routes/hooks/registered keys — the
/// framework wires those), any quoted path literal ending in the file name,
/// any framework convention suffix shared by sibling files (`*.hooks.php`
/// discovered dynamically), and a lexical backstop — if any declared
/// container name is mentioned anywhere else in the corpus (`Foo::class`
/// strings in config, route arrays, reflection targets, docblocks), the file
/// is treated as alive. Files that declare no container (pure function/side
/// -effect files) are never flagged: composer `files` autoload can run them
/// unconditionally.
fn detect_backend_orphan_modules(
    graph: &SemanticGraph,
    parsed_sources: &[(PathBuf, String)],
    contract_inventory: &ContractInventory,
    repo_root: &Path,
) -> Vec<DeadCodeFinding> {
    let inbound_files = graph
        .resolved_edges
        .iter()
        .filter(|edge| edge.source_file_path != edge.target_file_path)
        .map(|edge| edge.target_file_path.as_path())
        .collect::<HashSet<_>>();

    // Files that declare framework contracts are wired in by the framework
    // even with zero code references.
    let mut contract_files = HashSet::new();
    for items in [
        &contract_inventory.routes,
        &contract_inventory.hooks,
        &contract_inventory.registered_keys,
    ] {
        for item in items.iter() {
            for location in &item.locations {
                contract_files.insert(location.file_path.as_path());
            }
        }
    }

    // Convention suffixes: a multi-dot basename shape (`User.hooks.php`)
    // shared across >= 2 directories is a framework discovery channel, not an
    // orphan shape. Derived from the corpus itself — no
    // framework vocabulary.
    let mut suffix_files: HashMap<String, HashSet<&Path>> = HashMap::new();
    for (path, _) in parsed_sources {
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let mut parts = file_name.splitn(2, '.');
        let _stem = parts.next();
        if let Some(suffix) = parts.next() {
            if suffix.contains('.') {
                suffix_files
                    .entry(suffix.to_ascii_lowercase())
                    .or_default()
                    .insert(path.parent().unwrap_or_else(|| Path::new("")));
            }
        }
    }
    let convention_suffixes = suffix_files
        .iter()
        .filter(|(_, dirs)| dirs.len() >= 2)
        .map(|(suffix, _)| suffix.clone())
        .collect::<HashSet<_>>();

    // Quoted path literal tails across the whole corpus: `require 'Legacy.php'`,
    // template/module manifests naming PHP files.
    let mut path_tails = HashSet::new();
    for (_, source) in parsed_sources {
        for captures in path_literal_pattern().captures_iter(source) {
            if let Some(literal) = captures.get(1).map(|m| m.as_str()) {
                let tail = literal
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or_default();
                if !tail.is_empty() {
                    path_tails.insert(tail.to_ascii_lowercase());
                }
            }
        }
    }

    // Top-level containers per file.
    let mut containers_by_file: HashMap<&Path, Vec<&str>> = HashMap::new();
    for symbol in &graph.symbols {
        if symbol.parent_symbol_id.is_none()
            && matches!(
                symbol.kind,
                SymbolKind::Class
                    | SymbolKind::Interface
                    | SymbolKind::Trait
                    | SymbolKind::Struct
                    | SymbolKind::Enum
            )
        {
            containers_by_file
                .entry(symbol.file_path.as_path())
                .or_default()
                .push(symbol.name.as_str());
        }
    }

    // Convention stems: frameworks construct class names from directory names
    // at runtime (`"App\\Modules\\{$name}\\{$name}ServiceProvider"`), which no
    // lexical scan can see. If a file stem equals its enclosing convention
    // directory name plus a suffix, and that suffix recurs under >= 3 distinct
    // directories, the stem shape is a discovery convention — exempt.
    let mut stem_suffix_dirs: HashMap<String, HashSet<&Path>> = HashMap::new();
    let convention_stem_suffix = |path: &Path| -> Option<String> {
        let stem = path.file_stem()?.to_str()?;
        let dir = path.parent()?;
        let dir_name = dir.file_name()?.to_str()?;
        let suffix = stem.strip_prefix(dir_name)?;
        (!suffix.is_empty()).then(|| suffix.to_ascii_lowercase())
    };
    for (path, _) in parsed_sources {
        if let (Some(suffix), Some(dir)) = (convention_stem_suffix(path), path.parent()) {
            stem_suffix_dirs.entry(suffix).or_default().insert(dir);
        }
    }
    let convention_stem_suffixes = stem_suffix_dirs
        .iter()
        .filter(|(_, dirs)| dirs.len() >= 3)
        .map(|(suffix, _)| suffix.clone())
        .collect::<HashSet<_>>();

    let mut candidates: Vec<(PathBuf, String, Vec<&str>)> = Vec::new();
    for (path, _) in parsed_sources {
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if !BACKEND_ORPHAN_EXTENSIONS.contains(&extension) {
            continue;
        }
        if is_backend_orphan_exempt_path(path) {
            continue;
        }
        if convention_stem_suffix(path)
            .is_some_and(|suffix| convention_stem_suffixes.contains(&suffix))
        {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let mut name_parts = file_name.splitn(2, '.');
        let stem = name_parts.next().unwrap_or_default();
        if let Some(suffix) = name_parts.next() {
            if suffix.contains('.') && convention_suffixes.contains(&suffix.to_ascii_lowercase()) {
                continue;
            }
        }
        if inbound_files.contains(path.as_path()) || contract_files.contains(path.as_path()) {
            continue;
        }
        if path_tails.contains(&file_name.to_ascii_lowercase()) {
            continue;
        }
        let Some(containers) = containers_by_file.get(path.as_path()) else {
            // No container declared: side-effect or function file — composer
            // `files` autoload can run it unconditionally. Never flag.
            continue;
        };
        let mentioned = containers.iter().any(|name| {
            // Names too short to be discriminating suppress the finding —
            // suppression-only looseness cannot fabricate an orphan.
            name.len() < 4 || identifier_mentioned_in_other_files(name, path, parsed_sources)
        });
        if mentioned {
            continue;
        }
        candidates.push((path.clone(), stem.to_owned(), containers.clone()));
    }

    // Final suppression sweep over files OUTSIDE the analyzed slice (excluded
    // command dirs, bootstrap wiring, composer manifests): a scoped analysis
    // must not accuse a file that excluded code still points at. Reading
    // excluded files is safe here because it can only remove findings.
    let out_of_slice = collect_out_of_slice_sources(repo_root, parsed_sources);
    let mut findings = Vec::new();
    for (path, stem, containers) in candidates {
        let mentioned_outside = containers.iter().any(|name| {
            out_of_slice
                .iter()
                .any(|source| identifier_mentioned(name, source))
        });
        if mentioned_outside {
            continue;
        }
        let normalized = normalized_path(&path);
        findings.push(DeadCodeFinding {
            category: DeadCodeCategory::OrphanModule,
            symbol_id: format!("module:{normalized}"),
            file_path: path.clone(),
            name: stem.clone(),
            line: 1,
            proof_tier: DeadCodeProofTier::Heuristic,
            fingerprint: dead_code_fingerprint(DeadCodeCategory::OrphanModule, &path, &stem),
            delete_verdict: String::from("probably_delete"),
            delete_evidence: vec![
                String::from("no inbound resolved edge anywhere in the corpus"),
                String::from("declares no framework contract (route/hook/registration)"),
                String::from("no quoted path literal names the file"),
                String::from("not a corpus convention shape (multi-dot suffix or directory-derived stem)"),
                String::from(
                    "container names unmentioned corpus-wide, including out-of-slice non-test files",
                ),
                String::from(
                    "residual risk: runtime can still build the class name from strings that do not appear in the repo",
                ),
            ],
        });
    }
    findings
}

/// Text files under the repo root that are NOT part of the analyzed slice —
/// excluded directories, bootstrap wiring, manifests. Used exclusively for
/// suppression. Bounded: code/config extensions, files <= 1 MiB, vendored and
/// generated trees skipped.
fn collect_out_of_slice_sources(
    repo_root: &Path,
    parsed_sources: &[(PathBuf, String)],
) -> Vec<String> {
    const SWEEP_EXTENSIONS: &[&str] = &[
        "php", "json", "yaml", "yml", "xml", "neon", "ini", "sh", "env", "ts", "js",
    ];
    if repo_root.as_os_str().is_empty() {
        return Vec::new();
    }
    let parsed: HashSet<&Path> = parsed_sources
        .iter()
        .map(|(path, _)| path.as_path())
        .collect();
    let mut sources = Vec::new();
    let mut stack = vec![repo_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                // Test trees cannot prove production aliveness: a module only
                // exercised by its tests is dead production code.
                if name.starts_with('.')
                    || matches!(
                        name.as_ref(),
                        "vendor"
                            | "node_modules"
                            | "storage"
                            | "dist"
                            | "build"
                            | "target"
                            | "tests"
                            | "Tests"
                            | "__tests__"
                            | "test"
                            | "Test"
                    )
                {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if name.contains(".spec.") || name.contains(".test.") || name.ends_with("Test.php") {
                continue;
            }
            let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
                continue;
            };
            if !SWEEP_EXTENSIONS.contains(&extension) {
                continue;
            }
            let relative = path.strip_prefix(repo_root).unwrap_or(&path);
            if parsed.contains(relative) || parsed.contains(path.as_path()) {
                continue;
            }
            if entry
                .metadata()
                .map(|meta| meta.len() > 1_048_576)
                .unwrap_or(true)
            {
                continue;
            }
            if let Ok(source) = std::fs::read_to_string(&path) {
                sources.push(source);
            }
        }
    }
    sources
}

pub(crate) fn identifier_mentioned(name: &str, source: &str) -> bool {
    let is_ident = |byte: u8| byte == b'_' || byte.is_ascii_alphanumeric();
    let bytes = source.as_bytes();
    let mut start = 0;
    while let Some(position) = source[start..].find(name) {
        let begin = start + position;
        let end = begin + name.len();
        let before_ok = begin == 0 || !is_ident(bytes[begin - 1]);
        let after_ok = end >= bytes.len() || !is_ident(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        start = end;
    }
    false
}

/// Paths a backend framework reaches without a code reference: migrations,
/// seeders, config/route/bootstrap files loaded by convention, views,
/// entry scripts, vendored and generated trees.
fn is_backend_orphan_exempt_path(path: &Path) -> bool {
    let normalized = normalized_path(path).to_ascii_lowercase();
    let has_segment = |segment: &str| normalized.split('/').any(|part| part == segment);
    if has_segment("migrations")
        || has_segment("seeders")
        || has_segment("seeds")
        || has_segment("database")
        || has_segment("config")
        || has_segment("routes")
        || has_segment("bootstrap")
        || has_segment("public")
        || has_segment("views")
        || has_segment("resources")
        || has_segment("tests")
        || has_segment("__tests__")
        || has_segment("vendor")
        || has_segment("storage")
        || has_segment("stubs")
        || has_segment("bin")
    {
        return true;
    }
    let file_name = normalized.rsplit('/').next().unwrap_or_default();
    if file_name.ends_with(".blade.php") {
        return true;
    }
    let stem = file_name.split('.').next().unwrap_or_default();
    matches!(
        stem,
        "index" | "artisan" | "server" | "bootstrap" | "autoload"
    )
}

/// Word-boundary lexical mention of `name` in any file other than `origin`.
/// Deliberately covers strings, comments, and code alike: any of them proves
/// a human or framework still points at the identifier.
fn identifier_mentioned_in_other_files(
    name: &str,
    origin: &Path,
    parsed_sources: &[(PathBuf, String)],
) -> bool {
    parsed_sources
        .iter()
        .any(|(path, source)| path != origin && identifier_mentioned(name, source))
}

/// Paths a frontend framework or build system reaches without an import:
/// route-mapped page components (Inertia/Nuxt/Next), route/config/type
/// declaration directories, entry stems, and test/generated artifacts.
fn is_orphan_exempt_path(path: &Path) -> bool {
    let normalized = normalized_path(path).to_ascii_lowercase();
    let has_segment = |segment: &str| normalized.split('/').any(|part| part == segment);
    if has_segment("pages")
        || has_segment("routes")
        || has_segment("config")
        || has_segment("types")
        || has_segment("__tests__")
        || has_segment("tests")
        || has_segment("node_modules")
    {
        return true;
    }
    let file_name = normalized.rsplit('/').next().unwrap_or_default();
    if file_name.ends_with(".d.ts")
        || file_name.contains(".min.")
        || file_name.contains(".generated.")
        || file_name.contains(".spec.")
        || file_name.contains(".test.")
        || file_name.contains(".stories.")
        || file_name.contains(".config.")
    {
        return true;
    }
    let stem = strip_frontend_extension(file_name);
    matches!(
        stem,
        "index" | "main" | "app" | "bootstrap" | "setup" | "entry"
    )
}

fn strip_frontend_extension(name: &str) -> &str {
    for extension in FRONTEND_MODULE_EXTENSIONS {
        if let Some(stripped) = name.strip_suffix(extension) {
            if let Some(stem) = stripped.strip_suffix('.') {
                return stem;
            }
        }
    }
    name
}

/// Static prefixes of `import.meta.glob('...')` specifiers, resolved against
/// the declaring file's directory, normalized with forward slashes. Matching
/// is by substring containment, which is deliberately loose: glob coverage
/// only ever suppresses findings, so looseness cannot create a false positive.
fn collect_import_meta_glob_prefixes(parsed_sources: &[(PathBuf, String)]) -> Vec<String> {
    let mut prefixes = Vec::new();
    for (path, source) in parsed_sources {
        if !source.contains("import.meta.glob") {
            continue;
        }
        for segment in source.split("import.meta.glob").skip(1) {
            // Accept every call shape: `glob('a')`, `glob<T>(['a', 'b'])`,
            // `glob('a', { eager: true })`. Take the argument span up to the
            // closing paren and pull every quoted specifier out of it — extra
            // strings from an options object can only add suppression, never a
            // false positive.
            let Some(open) = segment.find('(') else {
                continue;
            };
            let arguments = &segment[open + 1
                ..segment[open..]
                    .find(')')
                    .map(|close| open + close)
                    .unwrap_or(segment.len())];
            for spec in extract_quoted_strings(arguments) {
                let static_prefix = spec
                    .split(['*', '{'])
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches('/');
                if static_prefix.is_empty() {
                    continue;
                }
                let resolved =
                    if static_prefix.starts_with("./") || static_prefix.starts_with("../") {
                        let base = path.parent().unwrap_or_else(|| Path::new(""));
                        normalized_path(&normalize_relative_segments(&base.join(static_prefix)))
                    } else {
                        // Alias (`@/...`) or root-absolute specifier: keep the
                        // path part after the alias marker for containment
                        // matching.
                        static_prefix
                            .trim_start_matches('@')
                            .trim_start_matches('/')
                            .to_owned()
                    };
                if !resolved.is_empty() {
                    prefixes.push(resolved);
                }
            }
        }
    }
    prefixes
}

/// A quoted relative or alias module path (`'./x'`, `'../y/z.ts'`, `'@/a/b'`).
/// Matched per occurrence with a path-safe charset, so prose apostrophes in
/// surrounding markup cannot desync the scan.
fn path_literal_pattern() -> &'static regex::Regex {
    static PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    PATTERN.get_or_init(|| {
        regex::Regex::new(r#"['"`]((?:\.{1,2}|@)/[A-Za-z0-9_@$./\-]+)['"`]"#)
            .expect("valid path literal pattern")
    })
}

fn extract_quoted_strings(text: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if matches!(c, '\'' | '"' | '`') {
            let mut value = String::new();
            for inner in chars.by_ref() {
                if inner == c {
                    break;
                }
                value.push(inner);
            }
            if !value.is_empty() {
                strings.push(value);
            }
        }
    }
    strings
}

fn normalize_relative_segments(path: &Path) -> PathBuf {
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            other => parts.push(other.as_os_str().to_owned()),
        }
    }
    parts.iter().collect()
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
        DeadCodeCategory::OrphanModule => "orphan-module",
    }
}

pub fn dead_code_proof_tier(category: DeadCodeCategory) -> DeadCodeProofTier {
    match category {
        DeadCodeCategory::UnusedImport => DeadCodeProofTier::Certain,
        DeadCodeCategory::UnusedPrivateFunction => DeadCodeProofTier::Strong,
        // Aliases, string-based component resolution, and out-of-scan consumers
        // (tests, sibling packages) are invisible to the import graph, so an
        // orphan verdict is evidence, not proof.
        DeadCodeCategory::OrphanModule => DeadCodeProofTier::Heuristic,
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
    use crate::contracts::ContractInventory;
    use crate::parsing::javascript::parse_javascript_to_graph;
    use crate::parsing::php::parse_php_to_graph;
    use crate::parsing::python::parse_python_to_graph;
    use crate::parsing::rust::parse_rust_to_graph;
    use crate::parsing::vue::parse_vue_to_graph;
    use crate::resolve::resolve_graph;
    use std::path::{Path, PathBuf};

    #[test]
    fn detects_orphan_frontend_modules_with_all_suppression_channels() {
        let consumer_path = PathBuf::from("resources/js/Pages/Home.vue");
        let consumer_source = String::from(
            r#"<script setup lang="ts">
import Imported from '../components/Imported.vue'
const widgets = import.meta.glob<{ default: unknown }>([
    '../widgets/*.manifest.ts',
])
const worker = new Worker(new URL('../workers/renderWorker.ts', import.meta.url))
export { helper } from '@/utils/reExported'
</script>
<template><Imported /></template>
"#,
        );
        let sources = vec![
            (consumer_path.clone(), consumer_source),
            (
                PathBuf::from("resources/js/components/Imported.vue"),
                String::from("<script setup>const a = 1</script>"),
            ),
            (
                PathBuf::from("resources/js/components/Orphan.vue"),
                String::from("<script setup>const b = 2</script>"),
            ),
            (
                PathBuf::from("resources/js/widgets/core.manifest.ts"),
                String::from("export default { id: 'core' }\n"),
            ),
            (
                PathBuf::from("resources/js/workers/renderWorker.ts"),
                String::from("self.onmessage = () => {}\n"),
            ),
            (
                PathBuf::from("resources/js/utils/reExported.ts"),
                String::from("export const helper = 1\n"),
            ),
            (
                PathBuf::from("resources/js/utils/index.ts"),
                String::from("export const entry = 1\n"),
            ),
        ];
        let mut graph = parse_vue_to_graph(consumer_path, sources[0].1.as_str()).unwrap();
        for (path, source) in &sources[1..] {
            let parsed = if path.extension().is_some_and(|e| e == "vue") {
                parse_vue_to_graph(path.clone(), source).unwrap()
            } else {
                parse_javascript_to_graph(path.clone(), source, true).unwrap()
            };
            graph.files.extend(parsed.files);
            graph.symbols.extend(parsed.symbols);
            graph.references.extend(parsed.references);
        }
        resolve_graph(&mut graph);

        let result = analyze_dead_code(
            &graph,
            &sources,
            &ContractInventory::default(),
            Path::new(""),
        );
        let orphans: Vec<&str> = result
            .findings
            .iter()
            .filter(|f| f.category == DeadCodeCategory::OrphanModule)
            .map(|f| f.name.as_str())
            .collect();

        assert_eq!(
            orphans,
            vec!["Orphan"],
            "only the truly unreferenced module is an orphan; import, glob, \
             worker-URL, re-export, and index-stem channels all suppress: {orphans:?}"
        );
    }

    #[test]
    fn detects_backend_orphan_php_classes_with_all_suppression_channels() {
        let sources = vec![
            (
                PathBuf::from("app/Services/Consumer.php"),
                String::from(
                    "<?php\nuse App\\Services\\UsedService;\nclass Consumer { public function run(): void { (new UsedService())->go(); } }\n",
                ),
            ),
            (
                PathBuf::from("app/Services/UsedService.php"),
                String::from("<?php\nclass UsedService { public function go(): void {} }\n"),
            ),
            (
                PathBuf::from("app/Services/DeadService.php"),
                String::from("<?php\nclass DeadService { public function never(): void {} }\n"),
            ),
            (
                // Mentioned only as a ::class string in another file: alive.
                PathBuf::from("app/Services/StringWired.php"),
                String::from("<?php\nclass StringWired {}\n"),
            ),
            (
                PathBuf::from("app/Providers/Wiring.php"),
                String::from(
                    "<?php\nclass Wiring { public function map(): array { return ['handler' => StringWired::class]; } }\n",
                ),
            ),
            (
                // Convention suffix shared across directories: framework channel.
                PathBuf::from("app/Entities/User/User.hooks.php"),
                String::from("<?php\nclass UserHooksUnusual {}\n"),
            ),
            (
                PathBuf::from("app/Entities/Order/Order.hooks.php"),
                String::from("<?php\nclass OrderHooksUnusual {}\n"),
            ),
            (
                // No container symbol: side-effect file, never flagged.
                PathBuf::from("app/Support/helpers_extra.php"),
                String::from("<?php\nfunction totally_unused_helper(): int { return 1; }\n"),
            ),
        ];
        let mut graph = parse_php_to_graph(sources[0].0.clone(), &sources[0].1).unwrap();
        for (path, source) in &sources[1..] {
            let parsed = parse_php_to_graph(path.clone(), source).unwrap();
            graph.files.extend(parsed.files);
            graph.symbols.extend(parsed.symbols);
            graph.references.extend(parsed.references);
        }
        resolve_graph(&mut graph);

        // Entry classes are wired by framework contracts (routes, registered
        // keys) rather than code references — that channel must suppress.
        let mut inventory = ContractInventory::default();
        inventory
            .routes
            .push(crate::contracts::ContractInventoryItem {
                value: String::from("/api/run"),
                count: 1,
                locations: vec![crate::contracts::ContractLocation {
                    file_path: PathBuf::from("app/Services/Consumer.php"),
                    line: 3,
                }],
            });
        inventory
            .registered_keys
            .push(crate::contracts::ContractInventoryItem {
                value: String::from("handler"),
                count: 1,
                locations: vec![crate::contracts::ContractLocation {
                    file_path: PathBuf::from("app/Providers/Wiring.php"),
                    line: 2,
                }],
            });

        let result = analyze_dead_code(&graph, &sources, &inventory, Path::new(""));
        let orphans: Vec<&str> = result
            .findings
            .iter()
            .filter(|f| f.category == DeadCodeCategory::OrphanModule)
            .map(|f| f.name.as_str())
            .collect();

        assert_eq!(
            orphans,
            vec!["DeadService"],
            "inbound-edge, contract-location, ::class-string, convention-suffix, \
             and no-container channels must all suppress: {orphans:?}"
        );
    }

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
        let result = analyze_dead_code(
            &graph,
            &[(path, source.to_string())],
            &ContractInventory::default(),
            Path::new(""),
        );

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
        let result = analyze_dead_code(
            &graph,
            &parsed_sources,
            &ContractInventory::default(),
            Path::new(""),
        );

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
        let result = analyze_dead_code(
            &graph,
            &parsed_sources,
            &ContractInventory::default(),
            Path::new(""),
        );

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
        let result = analyze_dead_code(
            &graph,
            &parsed_sources,
            &ContractInventory::default(),
            Path::new(""),
        );

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
        let result = analyze_dead_code(&graph, &[], &ContractInventory::default(), Path::new(""));

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
