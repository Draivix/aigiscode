use crate::graph::{
    CallForm, EdgeOrigin, EdgeStrength, GraphLayer, Language, ReferenceKind, RelationKind,
    ResolutionTier, ResolvedEdge, SemanticGraph, SymbolKind,
};
use crate::plugins::{RepoContext, RuntimePlugin};
use std::collections::{HashMap, HashSet};

pub struct ContainerResolutionPlugin;

impl RuntimePlugin for ContainerResolutionPlugin {
    fn id(&self) -> &'static str {
        "laravel_container"
    }

    fn emit_edges(&self, _repo: &RepoContext, graph: &SemanticGraph) -> Vec<ResolvedEdge> {
        let php_files = graph
            .files
            .iter()
            .filter(|file| file.language == Language::Php)
            .map(|file| file.path.as_path())
            .collect::<HashSet<_>>();
        let mut declarations = HashMap::<String, Vec<_>>::new();
        for symbol in &graph.symbols {
            if php_files.contains(symbol.file_path.as_path())
                && matches!(
                    symbol.kind,
                    SymbolKind::Class
                        | SymbolKind::Interface
                        | SymbolKind::Trait
                        | SymbolKind::Enum
                )
            {
                declarations
                    .entry(symbol.qualified_name.to_ascii_lowercase())
                    .or_default()
                    .push(symbol);
            }
        }
        let mut edges = Vec::new();
        let mut occurrences = HashMap::new();
        for reference in &graph.references {
            if reference.kind != ReferenceKind::Call
                || !php_files.contains(reference.file_path.as_path())
            {
                continue;
            }
            let occurrence = occurrences
                .entry((&reference.file_path, reference.line, &reference.target_name))
                .or_insert(0usize);
            let occurrence_index = *occurrence;
            *occurrence += 1;
            let is_container = match reference.call_form {
                Some(CallForm::Free) => matches!(
                    reference.target_name.trim_start_matches('\\'),
                    "app" | "resolve"
                ),
                Some(CallForm::Member) => {
                    reference.target_name == "make"
                        && matches!(
                            reference.receiver_name.as_deref(),
                            Some("app()" | "$app" | "$this->app")
                        )
                }
                _ => false,
            };
            if !is_container {
                continue;
            }
            let Some(binding) = &reference.class_literal_argument else {
                continue;
            };
            let Some(candidates) = declarations.get(&binding.to_ascii_lowercase()) else {
                continue;
            };
            let [target] = candidates.as_slice() else {
                continue;
            };
            edges.push(ResolvedEdge::new(
                reference.file_path.clone(), reference.enclosing_symbol_id.clone(),
                target.file_path.clone(), target.id.clone(), ReferenceKind::Call,
                ResolutionTier::ImportScoped, 900,
                format!("container binding names declared PHP type {binding}; exact first-argument class literal"),
                reference.line,
            ).with_reference_identity(reference.target_name.clone(), occurrence_index)
                .with_metadata(RelationKind::ContainerResolution, GraphLayer::Framework, EdgeStrength::Dynamic, EdgeOrigin::Plugin));
        }
        edges
    }
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
    fn exact_call_arguments_and_declared_namespaces_own_container_edges() {
        let fixture = create_fixture();
        for directory in ["caller", "unrelated-layout", "elsewhere"] {
            fs::create_dir_all(fixture.join(directory)).unwrap();
        }
        fs::write(
            fixture.join("caller/run.php"),
            r#"<?php
namespace Caller;
use Domain\Service as Alias;
use Domain\{OtherService as Grouped};
function run($unknown, $factory): void {
    app();
    app(); app(Alias::class); app(Grouped::class);
    app($unknown);
    app(Alias::OTHER);
    app($unknown, Alias::class);
    $factory->make(Alias::class);
    app()->make(
        Alias::class
    );
    app(\Domain\Service::class);
    app(Missing::class);
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("unrelated-layout/one.php"),
            "<?php namespace Domain; class Service { const OTHER = 'not-a-class'; }",
        )
        .unwrap();
        fs::write(
            fixture.join("unrelated-layout/two.php"),
            "<?php namespace Domain; interface OtherService {}",
        )
        .unwrap();
        fs::write(
            fixture.join("elsewhere/decoy.php"),
            "<?php namespace Elsewhere; class Service {} class Missing {}",
        )
        .unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let edges = analysis
            .semantic_graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.relation_kind == RelationKind::ContainerResolution)
            .collect::<Vec<_>>();
        assert_eq!(
            edges.iter().map(|edge| edge.line).collect::<Vec<_>>(),
            vec![7, 7, 12, 15]
        );
        assert_eq!(edges[0].occurrence_index, 1);
        assert_eq!(edges[1].occurrence_index, 2);
        assert!(edges
            .iter()
            .all(|edge| edge.target_file_path.starts_with("unrelated-layout")));
        assert_eq!(
            edges[1].target_file_path,
            PathBuf::from("unrelated-layout/two.php")
        );
    }

    #[test]
    fn repeated_namespace_blocks_keep_separate_import_scopes() {
        let fixture = create_fixture();
        fs::write(
            fixture.join("definitions.php"),
            "<?php namespace Domain; class One {} class Two {}",
        )
        .unwrap();
        fs::write(fixture.join("calls.php"), r#"<?php
namespace Left { use Domain\One as Selected; class Local {} function run() { app(Selected::class); app(Local::class); } }
namespace Right { use Domain\Two as Selected; class Local {} function run() { app(Selected::class); app(Local::class); } }
namespace { use function Domain\One; function run() { app(One::class); } }
"#).unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let graph = &analysis.semantic_graph;
        let names = graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.relation_kind == RelationKind::ContainerResolution)
            .map(|edge| {
                graph
                    .symbols
                    .iter()
                    .find(|symbol| symbol.id == edge.target_symbol_id)
                    .unwrap()
                    .qualified_name
                    .as_str()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec!["Domain\\One", "Left\\Local", "Domain\\Two", "Right\\Local"]
        );
    }

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
        let edges = plugin.emit_edges(&RepoContext::new(&fixture, &[]), &graph);

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
