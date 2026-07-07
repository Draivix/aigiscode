use crate::graph::{
    CallForm, EdgeOrigin, EdgeStrength, GraphLayer, ReferenceKind, RelationKind, ResolutionTier,
    ResolvedEdge, SemanticGraph, SymbolKind,
};
use crate::plugins::{
    import_targets_by_binding, leaf_symbol_name, same_file_symbol_targets, RepoContext,
    RuntimePlugin,
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct ContainerResolutionPlugin;

impl RuntimePlugin for ContainerResolutionPlugin {
    fn id(&self) -> &'static str {
        "laravel_container"
    }

    fn emit_edges(&self, repo: &RepoContext, graph: &SemanticGraph) -> Vec<ResolvedEdge> {
        let symbols_by_id = graph
            .symbols
            .iter()
            .map(|symbol| (symbol.id.clone(), symbol))
            .collect::<HashMap<_, _>>();
        let import_targets = import_targets_by_binding(graph, &symbols_by_id, |symbol| {
            matches!(symbol.kind, SymbolKind::Class | SymbolKind::Struct)
        });
        let same_file_symbols = same_file_symbol_targets(graph);
        let class_targets = class_targets_by_leaf(graph);
        let mut source_cache = HashMap::<PathBuf, Vec<String>>::new();
        let mut emitted = HashSet::<(PathBuf, String, usize)>::new();
        let mut edges = Vec::new();

        for reference in graph
            .references
            .iter()
            .filter(|reference| is_container_reference(reference))
        {
            let Some(binding_name) = extract_container_binding(reference, repo, &mut source_cache)
            else {
                continue;
            };
            let Some((target_symbol_id, target_file_path, channel)) = resolve_container_target(
                &reference.file_path,
                &binding_name,
                &import_targets,
                &same_file_symbols,
                &class_targets,
            ) else {
                continue;
            };
            if !emitted.insert((
                reference.file_path.clone(),
                target_symbol_id.clone(),
                reference.line,
            )) {
                continue;
            }
            edges.push(
                ResolvedEdge::new(
                    reference.file_path.clone(),
                    reference.enclosing_symbol_id.clone(),
                    target_file_path,
                    target_symbol_id,
                    ReferenceKind::Call,
                    channel.resolution_tier(),
                    channel.confidence_millis(),
                    format!(
                        "framework container resolution via {} ({})",
                        container_via(reference),
                        channel.evidence_label()
                    ),
                    reference.line,
                )
                .with_metadata(
                    RelationKind::ContainerResolution,
                    GraphLayer::Framework,
                    EdgeStrength::Dynamic,
                    EdgeOrigin::Plugin,
                ),
            );
        }

        edges
    }
}

fn is_container_reference(reference: &crate::graph::SemanticReference) -> bool {
    reference.kind == ReferenceKind::Call
        && matches!(
            (reference.call_form, reference.target_name.as_str()),
            (Some(CallForm::Free), "app")
                | (Some(CallForm::Member), "make")
                | (Some(CallForm::Member), "bound")
        )
}

fn extract_container_binding(
    reference: &crate::graph::SemanticReference,
    repo: &RepoContext,
    source_cache: &mut HashMap<PathBuf, Vec<String>>,
) -> Option<String> {
    match (reference.call_form, reference.target_name.as_str()) {
        (Some(CallForm::Free), "app") => {
            let snippet = source_snippet(repo, &reference.file_path, reference.line, source_cache)?;
            app_helper_regex()
                .captures(&snippet)
                .and_then(|captures| captures.name("class"))
                .map(|value| normalize_binding(value.as_str()))
        }
        (Some(CallForm::Member), "make") => {
            let receiver_name = reference.receiver_name.as_deref()?;
            if let Some(binding) = extract_helper_receiver_binding(receiver_name) {
                return Some(binding);
            }
            if !is_make_receiver(receiver_name) {
                return None;
            }
            let snippet = source_snippet(repo, &reference.file_path, reference.line, source_cache)?;
            make_call_regex()
                .captures(&snippet)
                .and_then(|captures| captures.name("class"))
                .map(|value| normalize_binding(value.as_str()))
        }
        _ => None,
    }
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
    let end = (start + 5).min(lines.len());
    Some(lines[start..end].join(" "))
}

fn extract_helper_receiver_binding(receiver_name: &str) -> Option<String> {
    let suffix = receiver_name.strip_prefix("app(")?.strip_suffix(')')?;
    let binding = suffix.strip_suffix("::class")?;
    Some(normalize_binding(binding))
}

fn normalize_binding(binding: &str) -> String {
    binding.trim().trim_start_matches('\\').to_owned()
}

fn is_make_receiver(receiver: &str) -> bool {
    matches!(receiver, "app()" | "$app" | "$this->app")
}

fn container_via(reference: &crate::graph::SemanticReference) -> String {
    match (reference.call_form, reference.target_name.as_str()) {
        (Some(CallForm::Free), "app") => String::from("app(Foo::class)"),
        (Some(CallForm::Member), "make") => {
            let receiver = reference.receiver_name.as_deref().unwrap_or("<receiver>");
            format!("{receiver}->make(Foo::class)")
        }
        _ => String::from("container"),
    }
}

/// How a container binding was bound to a concrete class. The channel decides
/// how much the rest of the pipeline may trust the edge: an imported class, a
/// same-file class, a namespace-qualified literal whose path matches, or a
/// same-directory sibling (PHP same-namespace needs no `use`) all pin the
/// target exactly; a bare name matched against a repo-unique class is only a
/// plausible guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerResolutionChannel {
    ImportedClass,
    SameFileClass,
    QualifiedPathVerified,
    SameDirectoryClass,
    GlobalUniqueName,
}

impl ContainerResolutionChannel {
    fn resolution_tier(self) -> ResolutionTier {
        match self {
            Self::ImportedClass => ResolutionTier::ImportScoped,
            Self::SameFileClass => ResolutionTier::SameFile,
            Self::QualifiedPathVerified | Self::SameDirectoryClass | Self::GlobalUniqueName => {
                ResolutionTier::Global
            }
        }
    }

    fn confidence_millis(self) -> u16 {
        match self {
            Self::ImportedClass
            | Self::SameFileClass
            | Self::QualifiedPathVerified
            | Self::SameDirectoryClass => 900,
            Self::GlobalUniqueName => 650,
        }
    }

    fn evidence_label(self) -> &'static str {
        match self {
            Self::ImportedClass => "imported class",
            Self::SameFileClass => "same-file class",
            Self::QualifiedPathVerified => "qualified name, path-verified",
            Self::SameDirectoryClass => "same-directory class",
            Self::GlobalUniqueName => "globally unique name",
        }
    }
}

fn resolve_container_target(
    file_path: &Path,
    binding_name: &str,
    import_targets: &HashMap<(PathBuf, String), (String, PathBuf)>,
    same_file_symbols: &HashMap<(PathBuf, String), (String, PathBuf)>,
    class_targets: &HashMap<String, Vec<(String, PathBuf)>>,
) -> Option<(String, PathBuf, ContainerResolutionChannel)> {
    let leaf = leaf_symbol_name(binding_name);
    if let Some((symbol_id, target)) = import_targets.get(&(file_path.to_path_buf(), leaf.clone()))
    {
        return Some((
            symbol_id.clone(),
            target.clone(),
            ContainerResolutionChannel::ImportedClass,
        ));
    }
    if let Some((symbol_id, target)) =
        same_file_symbols.get(&(file_path.to_path_buf(), leaf.clone()))
    {
        return Some((
            symbol_id.clone(),
            target.clone(),
            ContainerResolutionChannel::SameFileClass,
        ));
    }
    let candidates = class_targets.get(&leaf)?;
    if binding_name.contains('\\') {
        if let Some((symbol_id, target)) = qualified_path_match(binding_name, candidates) {
            return Some((
                symbol_id,
                target,
                ContainerResolutionChannel::QualifiedPathVerified,
            ));
        }
    }
    if let Some((symbol_id, target)) = same_directory_match(file_path, candidates) {
        return Some((
            symbol_id,
            target,
            ContainerResolutionChannel::SameDirectoryClass,
        ));
    }
    if candidates.len() == 1 {
        let (symbol_id, target) = &candidates[0];
        return Some((
            symbol_id.clone(),
            target.clone(),
            ContainerResolutionChannel::GlobalUniqueName,
        ));
    }
    None
}

/// A namespace-qualified binding (`App\Entities\_Core\EntityManager`) names its
/// own path under PSR-4-style layouts: the candidate whose trailing directory
/// segments equal the namespace segments (case-insensitive) is the target.
/// Requires a unique match so an ambiguous layout falls through to weaker
/// channels instead of guessing.
fn qualified_path_match(
    binding_name: &str,
    candidates: &[(String, PathBuf)],
) -> Option<(String, PathBuf)> {
    let namespace_segments = binding_name.split('\\').collect::<Vec<_>>();
    let dir_segments = &namespace_segments[..namespace_segments.len().saturating_sub(1)];
    if dir_segments.is_empty() {
        return None;
    }
    let mut matches = candidates.iter().filter(|(_, path)| {
        let dirs = path
            .parent()
            .map(|parent| {
                parent
                    .components()
                    .map(|component| component.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        dirs.len() >= dir_segments.len()
            && dirs[dirs.len() - dir_segments.len()..]
                .iter()
                .zip(dir_segments)
                .all(|(dir, segment)| dir.eq_ignore_ascii_case(segment))
    });
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(first.clone())
}

/// PHP resolves a bare class name against the current namespace before
/// anything else, and PSR-4 puts same-namespace classes in the same directory
/// — so a same-directory candidate is an exact-semantics match, not a guess.
fn same_directory_match(
    file_path: &Path,
    candidates: &[(String, PathBuf)],
) -> Option<(String, PathBuf)> {
    let directory = file_path.parent()?;
    let mut matches = candidates
        .iter()
        .filter(|(_, path)| path.parent() == Some(directory));
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(first.clone())
}

fn class_targets_by_leaf(graph: &SemanticGraph) -> HashMap<String, Vec<(String, PathBuf)>> {
    let mut grouped = HashMap::<String, Vec<(String, PathBuf)>>::new();
    for symbol in graph
        .symbols
        .iter()
        .filter(|symbol| matches!(symbol.kind, SymbolKind::Class | SymbolKind::Struct))
    {
        grouped
            .entry(symbol.name.clone())
            .or_default()
            .push((symbol.id.clone(), symbol.file_path.clone()));
    }
    grouped
}

fn app_helper_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"app\s*\(\s*(?P<class>[\\A-Za-z_][\\A-Za-z0-9_]*)\s*::\s*class\b")
            .expect("valid app helper regex")
    })
}

fn make_call_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"(?:app\s*\(\s*\)|\$app|\$this->app)\s*->\s*make\s*\(\s*(?P<class>[\\A-Za-z_][\\A-Za-z0-9_]*)\s*::\s*class\b",
        )
        .expect("valid make call regex")
    })
}

#[cfg(test)]
mod tests {
    use super::ContainerResolutionPlugin;
    use crate::graph::{GraphLayer, RelationKind};
    use crate::ingestion::scan::ScanConfig;
    use crate::plugins::{RepoContext, RuntimePlugin};
    use crate::resolve::resolve_graph;
    use crate::{ingestion::pipeline::analyze_project, parsing::php::parse_php_to_graph};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn emits_framework_edges_for_app_helper_resolution() {
        let fixture = create_fixture();
        let service_path = fixture.join("app/Events/AiCommandEvent.php");
        let dependency_path = fixture.join("app/Services/TenantManager.php");
        fs::create_dir_all(service_path.parent().unwrap()).unwrap();
        fs::create_dir_all(dependency_path.parent().unwrap()).unwrap();
        fs::write(
            &service_path,
            r#"<?php
namespace App\Events;

use App\Services\TenantManager;

final class AiCommandEvent
{
    public function tenant(): ?string
    {
        return app(TenantManager::class)->getCurrentTenant();
    }
}
"#,
        )
        .unwrap();
        fs::write(
            &dependency_path,
            r#"<?php
namespace App\Services;

final class TenantManager
{
    public function getCurrentTenant(): ?string
    {
        return null;
    }
}
"#,
        )
        .unwrap();

        let mut graph = parse_php_to_graph(
            PathBuf::from("app/Events/AiCommandEvent.php"),
            &fs::read_to_string(&service_path).unwrap(),
        )
        .unwrap();
        let mut imported = parse_php_to_graph(
            PathBuf::from("app/Services/TenantManager.php"),
            &fs::read_to_string(&dependency_path).unwrap(),
        )
        .unwrap();
        graph.files.append(&mut imported.files);
        graph.symbols.append(&mut imported.symbols);
        graph.references.append(&mut imported.references);
        resolve_graph(&mut graph);

        let plugin = ContainerResolutionPlugin;
        let edges = plugin.emit_edges(&RepoContext::new(&fixture), &graph);

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].layer, GraphLayer::Framework);
        assert_eq!(edges[0].relation_kind, RelationKind::ContainerResolution);
        assert_eq!(
            edges[0].target_file_path,
            PathBuf::from("app/Services/TenantManager.php")
        );
    }

    #[test]
    fn emits_framework_edges_for_make_member_call_variants() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app/Services")).unwrap();
        fs::create_dir_all(fixture.join("app/Providers")).unwrap();
        fs::create_dir_all(fixture.join("scripts")).unwrap();
        fs::write(
            fixture.join("app/Services/TenantManager.php"),
            r#"<?php
namespace App\Services;

final class TenantManager {}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Providers/AppServiceProvider.php"),
            r#"<?php
namespace App\Providers;

use App\Services\TenantManager;

final class AppServiceProvider
{
    public function register(): void
    {
        app()->make(TenantManager::class);
        $app->make(TenantManager::class);
        $this->app->make(TenantManager::class);
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("scripts/bootstrap.php"),
            r#"<?php
$app->make(\App\Services\TenantManager::class);
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let framework_edges = analysis
            .semantic_graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.relation_kind == RelationKind::ContainerResolution)
            .collect::<Vec<_>>();

        assert_eq!(framework_edges.len(), 4);
        assert!(framework_edges
            .iter()
            .all(|edge| edge.layer == GraphLayer::Framework));
        assert!(framework_edges.iter().any(|edge| edge.line == 10));
        assert!(framework_edges.iter().any(|edge| edge.line == 11));
        assert!(framework_edges.iter().any(|edge| edge.line == 12));
        assert!(framework_edges.iter().any(|edge| edge.line == 2));
    }

    // PHP resolves a bare class name against its own namespace without a
    // `use` statement, and a namespace-qualified literal names its PSR-4
    // path — both channels must produce high-confidence edges even when a
    // same-named decoy class exists elsewhere (which kills the old
    // globally-unique-name fallback).
    #[test]
    fn resolves_same_namespace_and_qualified_bindings_with_high_confidence() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app/Services")).unwrap();
        fs::create_dir_all(fixture.join("app/Entities")).unwrap();
        fs::create_dir_all(fixture.join("app/Legacy")).unwrap();
        fs::write(
            fixture.join("app/Services/TenantDb.php"),
            r#"<?php
namespace App\Services;

final class TenantDb
{
    public function tenant(): ?string
    {
        return app(TenantManager::class)->getCurrentTenant();
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Services/TenantManager.php"),
            r#"<?php
namespace App\Services;

final class TenantManager
{
    public function getCurrentTenant(): ?string
    {
        return null;
    }

    public function reset(): void
    {
        app(\App\Entities\EntityManager::class)->clear();
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Entities/EntityManager.php"),
            r#"<?php
namespace App\Entities;

final class EntityManager
{
    public function clear(): void
    {
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Legacy/EntityManager.php"),
            r#"<?php
namespace App\Legacy;

final class EntityManager
{
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let container_edges = analysis
            .semantic_graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.relation_kind == RelationKind::ContainerResolution)
            .collect::<Vec<_>>();

        let same_directory = container_edges
            .iter()
            .find(|edge| edge.source_file_path == PathBuf::from("app/Services/TenantDb.php"))
            .expect("same-namespace binding resolves");
        assert_eq!(
            same_directory.target_file_path,
            PathBuf::from("app/Services/TenantManager.php")
        );
        assert_eq!(same_directory.confidence_millis, 900);

        let qualified = container_edges
            .iter()
            .find(|edge| edge.source_file_path == PathBuf::from("app/Services/TenantManager.php"))
            .expect("qualified binding resolves despite decoy class");
        assert_eq!(
            qualified.target_file_path,
            PathBuf::from("app/Entities/EntityManager.php")
        );
        assert_eq!(qualified.confidence_millis, 900);
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-container-plugin-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
