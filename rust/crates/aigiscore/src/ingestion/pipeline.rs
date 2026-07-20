use crate::assessment::{build_architectural_assessment_full, ArchitecturalAssessment};
use crate::contracts::{build_contract_inventory, ContractInventory};
use crate::detectors::dead_code::{analyze_dead_code, DeadCodeResult};
use crate::detectors::hardwiring::{analyze_hardwiring_with_contracts, HardwiringResult};
use crate::external::ExternalAnalysisResult;
use crate::graph::analysis::{analyze_semantic_graph, GraphAnalysis};
use crate::graph::SemanticGraph;
use crate::ingestion::scan::{scan_repository, ScanConfig, ScanError, ScanResult};
use crate::ingestion::structure::{build_structure_graph, StructureGraph};
use crate::parsing::{is_supported_source_file, parse_source_file, ParseFileError};
use crate::plugins::{apply_runtime_plugins, RepoContext};
use crate::resolve::{load_resolve_config, resolve_graph_with_config};
use crate::scanners::ast_grep::{run_ast_grep_scan, AstGrepScanResult};
use crate::security::{analyze_security_findings_with_ast_grep_and_graph, SecurityAnalysisResult};
use crate::surface::{build_architecture_surface, ArchitectureSurface};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngestionPhase {
    Scan,
    Structure,
    Parse,
    Resolve,
    Analyze,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseTiming {
    pub phase: IngestionPhase,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestionPipelineResult {
    pub root: PathBuf,
    pub scan: ScanResult,
    pub structure: StructureGraph,
    pub timings: Vec<PhaseTiming>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticGraphProject {
    pub root: PathBuf,
    pub scan: ScanResult,
    pub structure: StructureGraph,
    pub semantic_graph: SemanticGraph,
    pub timings: Vec<PhaseTiming>,
    #[serde(skip)]
    pub parsed_sources: Vec<(PathBuf, String)>,
}

#[derive(Debug, Serialize)]
pub struct ProjectAnalysis {
    pub root: PathBuf,
    pub scan: ScanResult,
    pub structure: StructureGraph,
    pub semantic_graph: SemanticGraph,
    pub graph_analysis: GraphAnalysis,
    pub architectural_assessment: ArchitecturalAssessment,
    pub contract_inventory: ContractInventory,
    pub dead_code: DeadCodeResult,
    pub hardwiring: HardwiringResult,
    #[serde(default, skip_serializing_if = "SecurityAnalysisResult::is_empty")]
    pub security_analysis: SecurityAnalysisResult,
    #[serde(default, skip_serializing_if = "ExternalAnalysisResult::is_empty")]
    pub external_analysis: ExternalAnalysisResult,
    #[serde(default, skip_serializing_if = "AstGrepScanResult::is_empty")]
    pub ast_grep_scan: AstGrepScanResult,
    pub timings: Vec<PhaseTiming>,
    #[serde(skip)]
    pub parsed_sources: Vec<(PathBuf, String)>,
}

impl ProjectAnalysis {
    pub fn architecture_surface(&self) -> ArchitectureSurface {
        build_architecture_surface(self)
    }
}

#[derive(Debug, Error)]
pub enum ProjectAnalysisError {
    #[error(transparent)]
    Scan(#[from] ScanError),
    #[error("failed to read source file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse source file {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: ParseFileError,
    },
}

pub type RustProjectAnalysis = ProjectAnalysis;
pub type RustProjectAnalysisError = ProjectAnalysisError;

pub fn run_ingestion_pipeline(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
) -> Result<IngestionPipelineResult, ScanError> {
    let root = root.into();

    let scan_started = Instant::now();
    let scan = scan_repository(&root, scan_config)?;
    let scan_elapsed = scan_started.elapsed().as_millis();

    let structure_started = Instant::now();
    let structure = build_structure_graph(&scan.files);
    let structure_elapsed = structure_started.elapsed().as_millis();

    Ok(IngestionPipelineResult {
        root,
        scan,
        structure,
        timings: vec![
            PhaseTiming {
                phase: IngestionPhase::Scan,
                elapsed_ms: scan_elapsed,
            },
            PhaseTiming {
                phase: IngestionPhase::Structure,
                elapsed_ms: structure_elapsed,
            },
        ],
    })
}

pub fn analyze_project(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
) -> Result<ProjectAnalysis, ProjectAnalysisError> {
    let root = root.into();
    let graph_project = build_semantic_graph_project(&root, scan_config)?;
    let SemanticGraphProject {
        root,
        scan,
        structure,
        semantic_graph,
        mut timings,
        parsed_sources,
    } = graph_project;

    let analyze_started = Instant::now();
    trace("analyze start");
    let graph_started = Instant::now();
    let graph_analysis = analyze_semantic_graph(&semantic_graph, &scan.scope);
    trace(&format!(
        "analyze.graph_analysis elapsed_ms={}",
        graph_started.elapsed().as_millis()
    ));

    let contract_started = Instant::now();
    let contract_inventory = build_contract_inventory(&parsed_sources);
    trace(&format!(
        "analyze.contract_inventory elapsed_ms={}",
        contract_started.elapsed().as_millis()
    ));
    let contract_lookup = contract_inventory.lookup();

    let dead_code_started = Instant::now();
    let dead_code = analyze_dead_code(&semantic_graph, &parsed_sources, &contract_inventory, &root);
    trace(&format!(
        "analyze.dead_code elapsed_ms={}",
        dead_code_started.elapsed().as_millis()
    ));

    let hardwiring_started = Instant::now();
    let hardwiring = analyze_hardwiring_with_contracts(&parsed_sources, &contract_lookup);
    trace(&format!(
        "analyze.hardwiring elapsed_ms={}",
        hardwiring_started.elapsed().as_millis()
    ));

    let ast_grep_started = Instant::now();
    let ast_grep_scan = run_ast_grep_scan(&parsed_sources);
    trace(&format!(
        "analyze.ast_grep elapsed_ms={}",
        ast_grep_started.elapsed().as_millis()
    ));

    let security_started = Instant::now();
    let security_analysis = analyze_security_findings_with_ast_grep_and_graph(
        &parsed_sources,
        &contract_inventory,
        &graph_analysis.runtime_entry_candidates,
        &ast_grep_scan,
        Some(&semantic_graph),
    );
    trace(&format!(
        "analyze.security elapsed_ms={}",
        security_started.elapsed().as_millis()
    ));

    let assessment_started = Instant::now();
    // Layer contracts are doctrine-declared; absent or unreadable doctrine
    // simply means no layer enforcement, never a pipeline failure.
    let doctrine_layers = crate::doctrine::load_doctrine_registry(&root)
        .map(|registry| registry.layers)
        .unwrap_or_default();
    let architectural_assessment = build_architectural_assessment_full(
        &graph_analysis,
        &dead_code,
        &hardwiring,
        &ExternalAnalysisResult::default(),
        &parsed_sources,
        &ast_grep_scan,
        Some(&semantic_graph),
        &doctrine_layers,
    );
    trace(&format!(
        "analyze.architectural_assessment elapsed_ms={}",
        assessment_started.elapsed().as_millis()
    ));
    let analyze_elapsed = analyze_started.elapsed().as_millis();
    trace(&format!(
        "analyze complete cycles={} contracts={} dead_code={} hardwiring={} elapsed_ms={analyze_elapsed}",
        graph_analysis.strong_circular_dependencies.len(),
        contract_inventory.summary.routes.unique_values
            + contract_inventory.summary.hooks.unique_values
            + contract_inventory.summary.registered_keys.unique_values
            + contract_inventory.summary.symbolic_literals.unique_values
            + contract_inventory.summary.env_keys.unique_values
            + contract_inventory.summary.config_keys.unique_values,
        dead_code.findings.len(),
        hardwiring.findings.len()
    ));
    timings.push(PhaseTiming {
        phase: IngestionPhase::Analyze,
        elapsed_ms: analyze_elapsed,
    });

    Ok(ProjectAnalysis {
        root,
        scan,
        structure,
        semantic_graph,
        graph_analysis,
        architectural_assessment,
        contract_inventory,
        dead_code,
        hardwiring,
        security_analysis,
        external_analysis: ExternalAnalysisResult::default(),
        ast_grep_scan,
        timings,
        parsed_sources,
    })
}

pub fn analyze_rust_project(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
) -> Result<ProjectAnalysis, ProjectAnalysisError> {
    analyze_project(root, scan_config)
}

pub fn build_semantic_graph_project(
    root: impl Into<PathBuf>,
    scan_config: &ScanConfig,
) -> Result<SemanticGraphProject, ProjectAnalysisError> {
    let root = root.into();
    trace(&format!("analyze_project start {}", root.display()));

    let scan_started = Instant::now();
    let scan = scan_repository(&root, scan_config)?;
    let scan_elapsed = scan_started.elapsed().as_millis();
    trace(&format!(
        "scan complete files={} elapsed_ms={scan_elapsed}",
        scan.files.len()
    ));

    let structure_started = Instant::now();
    let structure = build_structure_graph(&scan.files);
    let structure_elapsed = structure_started.elapsed().as_millis();
    trace(&format!(
        "structure complete nodes={} edges={} elapsed_ms={structure_elapsed}",
        structure.nodes.len(),
        structure.contains_edges.len()
    ));

    let parse_started = Instant::now();
    // Per-file parsing is embarrassingly parallel; results are collected in
    // scan order and merged sequentially, so the graph stays byte-identical
    // to a single-threaded run (Hard Rule: same repo, same bytes).
    let supported_files = scan
        .files
        .iter()
        .filter(|file| is_supported_source_file(&file.relative_path))
        .collect::<Vec<_>>();
    let parsed_results = supported_files
        .par_iter()
        .map(|file| {
            let absolute_path = root.join(&file.relative_path);
            let source = fs::read_to_string(&absolute_path).map_err(|source| {
                ProjectAnalysisError::ReadFile {
                    path: absolute_path.clone(),
                    source,
                }
            })?;
            let parsed =
                parse_source_file(file.relative_path.clone(), &source).map_err(|source| {
                    ProjectAnalysisError::Parse {
                        path: absolute_path.clone(),
                        source,
                    }
                })?;
            Ok((file.relative_path.clone(), source, parsed))
        })
        .collect::<Result<Vec<_>, ProjectAnalysisError>>()?;
    let mut semantic_graph = SemanticGraph::default();
    let mut parsed_sources = Vec::new();
    for (relative_path, source, parsed) in parsed_results {
        merge_semantic_graph(&mut semantic_graph, parsed);
        parsed_sources.push((relative_path, source));
    }
    let parse_elapsed = parse_started.elapsed().as_millis();
    trace(&format!(
        "parse complete semantic_files={} symbols={} references={} elapsed_ms={parse_elapsed}",
        semantic_graph.files.len(),
        semantic_graph.symbols.len(),
        semantic_graph.references.len()
    ));

    let resolve_started = Instant::now();
    let resolve_config = load_resolve_config(&root);
    trace("resolve start");
    resolve_graph_with_config(&mut semantic_graph, &resolve_config);
    let resolve_elapsed = resolve_started.elapsed().as_millis();
    trace(&format!(
        "resolve complete resolved_edges={} elapsed_ms={resolve_elapsed}",
        semantic_graph.resolved_edges.len()
    ));
    let plugins_started = Instant::now();
    apply_runtime_plugins(&RepoContext::new(root.clone()), &mut semantic_graph);
    trace(&format!(
        "runtime plugins complete resolved_edges={} elapsed_ms={}",
        semantic_graph.resolved_edges.len(),
        plugins_started.elapsed().as_millis()
    ));

    Ok(SemanticGraphProject {
        root,
        scan,
        structure,
        semantic_graph,
        parsed_sources,
        timings: vec![
            PhaseTiming {
                phase: IngestionPhase::Scan,
                elapsed_ms: scan_elapsed,
            },
            PhaseTiming {
                phase: IngestionPhase::Structure,
                elapsed_ms: structure_elapsed,
            },
            PhaseTiming {
                phase: IngestionPhase::Parse,
                elapsed_ms: parse_elapsed,
            },
            PhaseTiming {
                phase: IngestionPhase::Resolve,
                elapsed_ms: resolve_elapsed,
            },
        ],
    })
}

fn merge_semantic_graph(target: &mut SemanticGraph, mut parsed: SemanticGraph) {
    target.files.append(&mut parsed.files);
    target.symbols.append(&mut parsed.symbols);
    target.references.append(&mut parsed.references);
    target.resolved_edges.append(&mut parsed.resolved_edges);
}

fn trace(message: &str) {
    if env::var_os("AIGISCORE_TRACE").is_some() {
        eprintln!("[aigiscore] {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze_project, analyze_rust_project, run_ingestion_pipeline, IngestionPhase};
    use crate::graph::{GraphLayer, RelationKind};
    use crate::ingestion::scan::ScanConfig;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn runs_scan_and_structure_as_explicit_phases() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src/core")).unwrap();
        fs::write(fixture.join("src/core/lib.rs"), b"pub fn demo() {}").unwrap();

        let result = run_ingestion_pipeline(&fixture, &ScanConfig::default()).unwrap();

        assert_eq!(result.scan.files.len(), 1);
        assert!(result
            .structure
            .nodes
            .iter()
            .any(|node| node.path == Path::new("src/core/lib.rs")));
        assert_eq!(result.timings.len(), 2);
        assert_eq!(result.timings[0].phase, IngestionPhase::Scan);
        assert_eq!(result.timings[1].phase, IngestionPhase::Structure);
    }

    #[test]
    fn analyzes_a_small_rust_project_end_to_end() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/models.rs"),
            b"pub struct User {}\nimpl User { pub fn save(&self) {} }\n",
        )
        .unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"use crate::models::User;

fn helper() {}
fn unused() {}

fn main() {
    let user = User {};
    let api = "https://api.example.com";
    helper();
    user.save();
    let _ = api;
}
"#,
        )
        .unwrap();

        let result = analyze_rust_project(&fixture, &ScanConfig::default()).unwrap();

        assert_eq!(result.scan.files.len(), 2);
        assert!(count_edges_to(&result, Path::new("src/models.rs")) >= 1);
        assert!(result
            .dead_code
            .findings
            .iter()
            .any(|finding| finding.name == "unused"));
        assert!(result
            .hardwiring
            .findings
            .iter()
            .any(|finding| finding.value == "https://api.example.com"));
        assert_eq!(result.timings.len(), 5);
        assert_eq!(result.timings[2].phase, IngestionPhase::Parse);
        assert_eq!(result.timings[3].phase, IngestionPhase::Resolve);
        assert_eq!(result.timings[4].phase, IngestionPhase::Analyze);
    }

    #[test]
    fn analyzes_a_small_multi_language_project_end_to_end() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::create_dir_all(fixture.join("app/Models")).unwrap();
        fs::create_dir_all(fixture.join("app/models")).unwrap();

        fs::write(
            fixture.join("src/app.ts"),
            br#"import DefaultThing, { User } from "./models";
DefaultThing.run();
const user = new User();
const _unused = user;
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/models.ts"),
            br#"export class User {}
export class Service {
  static run() {}
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/service.py"),
            br#"from .models import User

def run(user: User):
    return user
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/models.py"),
            br#"class User:
    pass
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Service.php"),
            br#"<?php
use App\Models\User;
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Models/User.php"),
            br#"<?php class User {}"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/service.rb"),
            br#"require_relative "./models/user"
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/models/user.rb"),
            br#"class User
end
"#,
        )
        .unwrap();

        let result = analyze_project(&fixture, &ScanConfig::default()).unwrap();

        assert_eq!(result.scan.files.len(), 8);
        assert!(result
            .semantic_graph
            .files
            .iter()
            .any(|file| file.path == Path::new("src/app.ts")));
        assert!(count_edges_to(&result, Path::new("src/models.ts")) >= 1);
        assert!(count_edges_to(&result, Path::new("app/models.py")) >= 1);
        assert!(count_edges_to(&result, Path::new("app/Models/User.php")) >= 1);
        assert!(count_edges_to(&result, Path::new("app/models/user.rb")) >= 1);
        assert_eq!(result.timings.len(), 5);
    }

    #[test]
    fn analyzes_cross_file_factory_receivers_end_to_end() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::create_dir_all(fixture.join("app/Factories")).unwrap();
        fs::create_dir_all(fixture.join("app/Models")).unwrap();
        fs::create_dir_all(fixture.join("app")).unwrap();

        fs::write(
            fixture.join("src/service.ts"),
            br#"import { UserFactory as UF } from "./factory";

function run() {
  UF.buildUser().save();
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/factory.ts"),
            br#"import { User } from "./models";

export class UserFactory {
  static buildUser(): User {
    return new User();
  }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/models.ts"),
            br#"export class User {
  save() {}
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/service.py"),
            br#"from .factory import UserFactory as UF

def run():
    UF.build_user().save()
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/factory.py"),
            br#"from .models import User

class UserFactory:
    @staticmethod
    def build_user() -> User:
        return User()
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/models.py"),
            br#"class User:
    def save(self):
        pass
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Service.php"),
            br#"<?php
use App\Factories\UserFactory as UF;

function run() {
    UF::makeUser()->save();
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Factories/UserFactory.php"),
            br#"<?php
namespace App\Factories;

use App\Models\User;

class UserFactory {
    public static function makeUser(): User {
        return new User();
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Models/User.php"),
            br#"<?php
namespace App\Models;

class User {
    public function save() {}
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/service.rb"),
            br#"require_relative "./user"

def run
  User.new.save
end
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/user.rb"),
            br#"class User
  def save
  end
end
"#,
        )
        .unwrap();

        let result = analyze_project(&fixture, &ScanConfig::default()).unwrap();

        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.target_file_path == Path::new("src/models.ts")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.target_file_path == Path::new("app/models.py")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.target_file_path == Path::new("app/Models/User.php")
                && edge.target_symbol_id.contains("save")
        }));
        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.target_file_path == Path::new("app/user.rb")
                && edge.target_symbol_id.contains("save")
        }));
    }

    #[test]
    fn applies_runtime_queue_plugins_during_project_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("app/Services")).unwrap();
        fs::create_dir_all(fixture.join("app/Jobs")).unwrap();

        fs::write(
            fixture.join("app/Services/EmailSyncManager.php"),
            br#"<?php
namespace App\Services;

use App\Jobs\SyncAccountJob;

final class EmailSyncManager
{
    public static function notifyJob(): void {}

    public function run(): void
    {
        SyncAccountJob::dispatch('tenant', 1);
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("app/Jobs/SyncAccountJob.php"),
            br#"<?php
namespace App\Jobs;

use App\Services\EmailSyncManager;

final class SyncAccountJob
{
    public function handle(): void
    {
        EmailSyncManager::notifyJob();
    }
}
"#,
        )
        .unwrap();

        let result = analyze_project(&fixture, &ScanConfig::default()).unwrap();

        assert!(result.semantic_graph.resolved_edges.iter().any(|edge| {
            edge.relation_kind == RelationKind::Dispatch
                && edge.layer == GraphLayer::Runtime
                && edge.target_file_path == Path::new("app/Jobs/SyncAccountJob.php")
        }));
        assert!(result
            .graph_analysis
            .strong_cycle_findings
            .iter()
            .any(|finding| finding.cycle_class == crate::graph::analysis::CycleClass::Mixed));
    }

    #[test]
    fn builds_contract_inventory_during_project_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("routes")).unwrap();
        fs::create_dir_all(fixture.join("src")).unwrap();

        fs::write(
            fixture.join("routes/web.php"),
            br#"<?php
Route::get('/users', 'UserController@index');
add_action('init', 'boot_users');
config('mail.driver');
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/runtime.ts"),
            br#"type Status = 'draft' | 'published';
const mode = process.env.APP_MODE;
"#,
        )
        .unwrap();

        let result = analyze_project(&fixture, &ScanConfig::default()).unwrap();

        assert_eq!(result.contract_inventory.summary.routes.unique_values, 1);
        assert_eq!(result.contract_inventory.summary.hooks.unique_values, 1);
        assert_eq!(
            result.contract_inventory.summary.config_keys.unique_values,
            1
        );
        assert_eq!(result.contract_inventory.summary.env_keys.unique_values, 1);
        assert!(result
            .contract_inventory
            .symbolic_literals
            .iter()
            .any(|item| item.value == "draft"));
    }

    fn count_edges_to(result: &super::ProjectAnalysis, target: &Path) -> usize {
        result
            .semantic_graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.target_file_path == target)
            .count()
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aigiscore-pipeline-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
