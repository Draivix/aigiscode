use crate::detectors::dead_code::DeadCodeResult;
use crate::detectors::hardwiring::HardwiringResult;
use crate::external::ExternalAnalysisResult;
use crate::graph::analysis::GraphAnalysis;
use crate::ingestion::pipeline::{PhaseTiming, ProjectAnalysis, SemanticGraphProject};
use crate::kuzu_index::{
    build_dependency_graph_artifact, build_evidence_graph_artifact, DependencyGraphArtifact,
    EvidenceGraphArtifact,
};
use crate::policy::{PolicyBundle, PolicyLoadError};
use crate::review::{build_review_surface, ReviewSurface};
use crate::surface::ArchitectureSurface;
use serde::Serialize;
use std::fs;
use std::io;
use std::ops::Not;
use std::path::{Path, PathBuf};

pub const DEFAULT_OUTPUT_DIR_NAME: &str = ".aigiscode";
pub const DETERMINISTIC_ANALYSIS_FILE: &str = "deterministic-analysis.json";
pub const SEMANTIC_GRAPH_FILE: &str = "semantic-graph.json";
pub const DEPENDENCY_GRAPH_FILE: &str = "dependency-graph.json";
pub const EVIDENCE_GRAPH_FILE: &str = "evidence-graph.json";
pub const DETERMINISTIC_FINDINGS_FILE: &str = "deterministic-findings.json";
pub const EXTERNAL_ANALYSIS_FILE: &str = "external-analysis.json";
pub const ARCHITECTURE_SURFACE_FILE: &str = "architecture-surface.json";
pub const REVIEW_SURFACE_FILE: &str = "review-surface.json";
pub const AGENT_HANDOFF_FILE: &str = "aigiscode-handoff.json";
pub const AIGISCODE_REPORT_FILE: &str = "aigiscode-report.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPaths {
    pub output_dir: PathBuf,
    pub deterministic_analysis: PathBuf,
    pub semantic_graph: PathBuf,
    pub dependency_graph: PathBuf,
    pub evidence_graph: PathBuf,
    pub deterministic_findings: PathBuf,
    pub external_analysis: PathBuf,
    pub architecture_surface: PathBuf,
    pub review_surface: PathBuf,
    pub agent_handoff: PathBuf,
    pub aigiscode_report: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct DeterministicFindingsArtifact<'a> {
    pub root: &'a Path,
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub symbols: usize,
    pub references: usize,
    pub resolved_edges: usize,
    pub graph_analysis: &'a GraphAnalysis,
    pub dead_code: &'a DeadCodeResult,
    pub hardwiring: &'a HardwiringResult,
    pub timings: &'a [PhaseTiming],
}

#[derive(Debug, Serialize)]
pub struct DependencyGraphJsonArtifact<'a> {
    pub root: &'a Path,
    pub dependency_graph: DependencyGraphArtifact,
}

#[derive(Debug, Serialize)]
pub struct EvidenceGraphJsonArtifact<'a> {
    pub root: &'a Path,
    pub evidence_graph: EvidenceGraphArtifact,
}

#[derive(Debug, Serialize)]
pub struct AigiscodeReportArtifact<'a> {
    pub root: &'a Path,
    pub summary: ReportSummary,
    pub feedback_loop: FeedbackLoopSummary,
    pub graph_analysis: &'a GraphAnalysis,
    pub dead_code: &'a DeadCodeResult,
    pub hardwiring: &'a HardwiringResult,
    pub external_analysis: &'a ExternalAnalysisResult,
    pub architecture_surface: &'a ArchitectureSurface,
    pub review_surface: &'a ReviewSurface,
    pub agent_handoff: &'a AgentHandoffArtifact,
    pub timings: &'a [PhaseTiming],
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub symbols: usize,
    pub references: usize,
    pub resolved_edges: usize,
    pub strong_cycle_count: usize,
    pub total_cycle_count: usize,
    pub dead_code_count: usize,
    pub hardwiring_count: usize,
    pub external_tool_count: usize,
    pub external_finding_count: usize,
    pub visible_findings: usize,
    pub accepted_by_policy: usize,
    pub suppressed_by_rule: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FeedbackLoopSummary {
    pub detected_total: usize,
    pub actionable_visible: usize,
    pub accepted_by_policy: usize,
    pub suppressed_by_rule: usize,
    pub ai_reviewed: usize,
    pub rules_generated: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentHandoffArtifact {
    pub root: String,
    pub summary: AgentHandoffSummary,
    pub feedback_loop: FeedbackLoopSummary,
    pub next_steps: Vec<String>,
    pub top_findings: Vec<AgentHandoffFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentHandoffSummary {
    pub scanned_files: usize,
    pub analyzed_files: usize,
    pub strong_cycle_count: usize,
    pub bottleneck_count: usize,
    pub visible_findings: usize,
    pub dead_code_count: usize,
    pub hardwiring_count: usize,
    pub external_finding_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentHandoffFinding {
    pub id: String,
    pub family: String,
    pub severity: String,
    pub title: String,
    pub summary: String,
    pub file_paths: Vec<String>,
    pub line: Option<usize>,
}

pub fn default_output_dir(root: &Path) -> PathBuf {
    root.join(DEFAULT_OUTPUT_DIR_NAME)
}

pub fn write_project_analysis_artifacts(
    analysis: &ProjectAnalysis,
    output_dir: Option<&Path>,
) -> io::Result<ArtifactPaths> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&analysis.root));
    fs::create_dir_all(&output_dir)?;

    let paths = ArtifactPaths {
        deterministic_analysis: output_dir.join(DETERMINISTIC_ANALYSIS_FILE),
        semantic_graph: output_dir.join(SEMANTIC_GRAPH_FILE),
        dependency_graph: output_dir.join(DEPENDENCY_GRAPH_FILE),
        evidence_graph: output_dir.join(EVIDENCE_GRAPH_FILE),
        deterministic_findings: output_dir.join(DETERMINISTIC_FINDINGS_FILE),
        external_analysis: output_dir.join(EXTERNAL_ANALYSIS_FILE),
        architecture_surface: output_dir.join(ARCHITECTURE_SURFACE_FILE),
        review_surface: output_dir.join(REVIEW_SURFACE_FILE),
        agent_handoff: output_dir.join(AGENT_HANDOFF_FILE),
        aigiscode_report: output_dir.join(AIGISCODE_REPORT_FILE),
        output_dir,
    };

    let findings = DeterministicFindingsArtifact {
        root: &analysis.root,
        scanned_files: analysis.scan.files.len(),
        analyzed_files: analysis.semantic_graph.files.len(),
        symbols: analysis.semantic_graph.symbols.len(),
        references: analysis.semantic_graph.references.len(),
        resolved_edges: analysis.semantic_graph.resolved_edges.len(),
        graph_analysis: &analysis.graph_analysis,
        dead_code: &analysis.dead_code,
        hardwiring: &analysis.hardwiring,
        timings: &analysis.timings,
    };
    let surface = analysis.architecture_surface();
    let dependency_graph = DependencyGraphJsonArtifact {
        root: &analysis.root,
        dependency_graph: build_dependency_graph_artifact(&analysis.semantic_graph),
    };
    let evidence_graph = EvidenceGraphJsonArtifact {
        root: &analysis.root,
        evidence_graph: build_evidence_graph_artifact(&analysis.semantic_graph),
    };
    let policy_bundle = PolicyBundle::load(&analysis.root).map_err(policy_error_to_io)?;
    let review_surface = build_review_surface(analysis, &surface, &policy_bundle);
    let feedback_loop = build_feedback_loop_summary(&review_surface);
    let agent_handoff = build_agent_handoff_artifact(analysis, &review_surface);
    let report = AigiscodeReportArtifact {
        root: &analysis.root,
        summary: ReportSummary {
            scanned_files: analysis.scan.files.len(),
            analyzed_files: analysis.semantic_graph.files.len(),
            symbols: analysis.semantic_graph.symbols.len(),
            references: analysis.semantic_graph.references.len(),
            resolved_edges: analysis.semantic_graph.resolved_edges.len(),
            strong_cycle_count: analysis.graph_analysis.strong_circular_dependencies.len(),
            total_cycle_count: analysis.graph_analysis.circular_dependencies.len(),
            dead_code_count: analysis.dead_code.findings.len(),
            hardwiring_count: analysis.hardwiring.findings.len(),
            external_tool_count: analysis.external_analysis.tool_runs.len(),
            external_finding_count: analysis.external_analysis.findings.len(),
            visible_findings: review_surface.summary.visible_findings,
            accepted_by_policy: review_surface.summary.accepted_by_policy,
            suppressed_by_rule: review_surface.summary.suppressed_by_rule,
        },
        feedback_loop: feedback_loop.clone(),
        graph_analysis: &analysis.graph_analysis,
        dead_code: &analysis.dead_code,
        hardwiring: &analysis.hardwiring,
        external_analysis: &analysis.external_analysis,
        architecture_surface: &surface,
        review_surface: &review_surface,
        agent_handoff: &agent_handoff,
        timings: &analysis.timings,
    };

    write_json(&paths.deterministic_analysis, &report)?;
    write_json(&paths.semantic_graph, &analysis.semantic_graph)?;
    write_json(&paths.dependency_graph, &dependency_graph)?;
    write_json(&paths.evidence_graph, &evidence_graph)?;
    write_json(&paths.deterministic_findings, &findings)?;
    write_json(&paths.external_analysis, &analysis.external_analysis)?;
    write_json(&paths.architecture_surface, &surface)?;
    write_json(&paths.review_surface, &review_surface)?;
    write_json(&paths.agent_handoff, &agent_handoff)?;
    write_json(&paths.aigiscode_report, &report)?;

    Ok(paths)
}

pub fn build_feedback_loop_summary(review_surface: &ReviewSurface) -> FeedbackLoopSummary {
    FeedbackLoopSummary {
        detected_total: review_surface.summary.total_findings,
        actionable_visible: review_surface.summary.visible_findings,
        accepted_by_policy: review_surface.summary.accepted_by_policy,
        suppressed_by_rule: review_surface.summary.suppressed_by_rule,
        ai_reviewed: review_surface.summary.ai_reviewed,
        rules_generated: review_surface.summary.rules_generated,
    }
}

pub fn build_agent_handoff_artifact(
    analysis: &ProjectAnalysis,
    review_surface: &ReviewSurface,
) -> AgentHandoffArtifact {
    let feedback_loop = build_feedback_loop_summary(review_surface);
    let visible_findings = review_surface
        .findings
        .iter()
        .filter(|finding| finding.is_visible)
        .collect::<Vec<_>>();
    let high_visible = visible_findings
        .iter()
        .filter(|finding| finding.severity == crate::review::ReviewFindingSeverity::High)
        .count();
    let mut next_steps = Vec::new();

    if analysis
        .graph_analysis
        .strong_circular_dependencies
        .is_empty()
        .not()
    {
        next_steps.push(format!(
            "Break {} strong cycle groups before adding more features.",
            analysis.graph_analysis.strong_circular_dependencies.len()
        ));
    }
    if analysis.graph_analysis.bottleneck_files.is_empty().not() {
        next_steps.push(format!(
            "Refactor the top {} bottleneck files to reduce architectural pressure.",
            analysis.graph_analysis.bottleneck_files.len().min(5)
        ));
    }
    if analysis.dead_code.findings.is_empty().not() {
        next_steps.push(format!(
            "Remove or suppress {} dead-code findings after sampling truth.",
            analysis.dead_code.findings.len()
        ));
    }
    if analysis.hardwiring.findings.is_empty().not() {
        next_steps.push(format!(
            "Triage {} hardwiring findings and convert repeated accepted patterns into policy.",
            analysis.hardwiring.findings.len()
        ));
    }
    if analysis.external_analysis.findings.is_empty().not() {
        next_steps.push(format!(
            "Review {} external security findings and feed accepted patterns back into rules.",
            analysis.external_analysis.findings.len()
        ));
    }
    if next_steps.is_empty() {
        next_steps.push(String::from(
            "No major actionable findings remain; keep the current architecture baseline stable.",
        ));
    }

    AgentHandoffArtifact {
        root: analysis.root.display().to_string(),
        summary: AgentHandoffSummary {
            scanned_files: analysis.scan.files.len(),
            analyzed_files: analysis.semantic_graph.files.len(),
            strong_cycle_count: analysis.graph_analysis.strong_circular_dependencies.len(),
            bottleneck_count: analysis.graph_analysis.bottleneck_files.len(),
            visible_findings: review_surface.summary.visible_findings,
            dead_code_count: analysis.dead_code.findings.len(),
            hardwiring_count: analysis.hardwiring.findings.len(),
            external_finding_count: analysis.external_analysis.findings.len(),
        },
        feedback_loop,
        next_steps,
        top_findings: visible_findings
            .into_iter()
            .take(if high_visible > 0 { 8 } else { 5 })
            .map(|finding| AgentHandoffFinding {
                id: finding.id.clone(),
                family: format!("{:?}", finding.family),
                severity: format!("{:?}", finding.severity),
                title: finding.title.clone(),
                summary: finding.summary.clone(),
                file_paths: finding.file_paths.clone(),
                line: finding.line,
            })
            .collect(),
    }
}

pub fn write_architecture_surface_artifact(
    surface: &ArchitectureSurface,
    root: &Path,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(root));
    fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(ARCHITECTURE_SURFACE_FILE);
    write_json(&path, surface)?;
    Ok(path)
}

pub fn write_semantic_graph_artifact(
    project: &SemanticGraphProject,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&project.root));
    fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(SEMANTIC_GRAPH_FILE);
    write_json(&path, &project.semantic_graph)?;
    Ok(path)
}

pub fn write_dependency_graph_artifact(
    project: &SemanticGraphProject,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&project.root));
    fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(DEPENDENCY_GRAPH_FILE);
    let dependency_graph = DependencyGraphJsonArtifact {
        root: &project.root,
        dependency_graph: build_dependency_graph_artifact(&project.semantic_graph),
    };
    write_json(&path, &dependency_graph)?;
    Ok(path)
}

pub fn write_evidence_graph_artifact(
    project: &SemanticGraphProject,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(&project.root));
    fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(EVIDENCE_GRAPH_FILE);
    let evidence_graph = EvidenceGraphJsonArtifact {
        root: &project.root,
        evidence_graph: build_evidence_graph_artifact(&project.semantic_graph),
    };
    write_json(&path, &evidence_graph)?;
    Ok(path)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let payload = serde_json::to_vec_pretty(value).map_err(|error| {
        io::Error::other(format!("failed to serialize {}: {error}", path.display()))
    })?;
    let mut data = payload;
    data.push(b'\n');
    fs::write(path, data)
}

fn policy_error_to_io(error: PolicyLoadError) -> io::Error {
    io::Error::other(error)
}

#[cfg(test)]
mod tests {
    use super::{
        default_output_dir, write_architecture_surface_artifact, write_dependency_graph_artifact,
        write_evidence_graph_artifact, write_project_analysis_artifacts,
        write_semantic_graph_artifact, AGENT_HANDOFF_FILE, AIGISCODE_REPORT_FILE,
        ARCHITECTURE_SURFACE_FILE, DEPENDENCY_GRAPH_FILE, DETERMINISTIC_ANALYSIS_FILE,
        DETERMINISTIC_FINDINGS_FILE, EVIDENCE_GRAPH_FILE, EXTERNAL_ANALYSIS_FILE,
        REVIEW_SURFACE_FILE, SEMANTIC_GRAPH_FILE,
    };
    use crate::ingestion::pipeline::{analyze_project, build_semantic_graph_project};
    use crate::ingestion::scan::ScanConfig;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_project_analysis_artifact_family() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod models;
use crate::models::User;
fn main() {
    let status = "draft";
    let user = User;
    let _ = user;
    let _ = status;
}
"#,
        )
        .unwrap();
        fs::write(fixture.join("src/models.rs"), b"pub struct User;\n").unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let output_dir = fixture.join("artifacts");
        let paths = write_project_analysis_artifacts(&analysis, Some(&output_dir)).unwrap();

        assert_eq!(paths.output_dir, output_dir);
        assert!(paths
            .deterministic_analysis
            .ends_with(DETERMINISTIC_ANALYSIS_FILE));
        assert!(paths.semantic_graph.ends_with(SEMANTIC_GRAPH_FILE));
        assert!(paths.dependency_graph.ends_with(DEPENDENCY_GRAPH_FILE));
        assert!(paths.evidence_graph.ends_with(EVIDENCE_GRAPH_FILE));
        assert!(paths
            .deterministic_findings
            .ends_with(DETERMINISTIC_FINDINGS_FILE));
        assert!(paths.external_analysis.ends_with(EXTERNAL_ANALYSIS_FILE));
        assert!(paths
            .architecture_surface
            .ends_with(ARCHITECTURE_SURFACE_FILE));
        assert!(paths.review_surface.ends_with(REVIEW_SURFACE_FILE));
        assert!(paths.agent_handoff.ends_with(AGENT_HANDOFF_FILE));
        assert!(paths.aigiscode_report.ends_with(AIGISCODE_REPORT_FILE));

        let findings_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.deterministic_findings).unwrap())
                .unwrap();
        assert_eq!(findings_payload["scanned_files"], 2);
        assert!(findings_payload["resolved_edges"].as_u64().unwrap() >= 1);
        assert!(findings_payload["hardwiring"]["findings"]
            .as_array()
            .is_some());

        let report_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.aigiscode_report).unwrap()).unwrap();
        assert_eq!(report_payload["summary"]["scanned_files"], 2);
        assert!(
            report_payload["architecture_surface"]["overview"]["resolved_edges"]
                .as_u64()
                .unwrap()
                >= 1
        );
        assert_eq!(
            report_payload["review_surface"]["summary"]["accepted_by_policy"],
            0
        );
        assert_eq!(
            report_payload["feedback_loop"]["detected_total"]
                .as_u64()
                .unwrap(),
            report_payload["review_surface"]["summary"]["total_findings"]
                .as_u64()
                .unwrap()
        );
        assert!(report_payload["agent_handoff"]["top_findings"]
            .as_array()
            .is_some());

        let dependency_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.dependency_graph).unwrap()).unwrap();
        assert_eq!(
            dependency_payload["dependency_graph"]["view"],
            Value::String(String::from("dependency_view"))
        );
        assert!(dependency_payload["dependency_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|node| node["kind"] != Value::String(String::from("MODULE"))));

        let evidence_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.evidence_graph).unwrap()).unwrap();
        assert_eq!(
            evidence_payload["evidence_graph"]["view"],
            Value::String(String::from("evidence_view"))
        );
        assert!(evidence_payload["evidence_graph"]["edges"]
            .as_array()
            .is_some());

        let handoff_payload: Value =
            serde_json::from_str(&fs::read_to_string(paths.agent_handoff).unwrap()).unwrap();
        assert_eq!(
            handoff_payload["summary"]["scanned_files"]
                .as_u64()
                .unwrap(),
            2
        );
        assert!(handoff_payload["next_steps"].as_array().is_some());
    }

    #[test]
    fn defaults_to_repo_aigiscode_directory_for_surface_artifact() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn main() {
    let url = "https://api.example.com";
    let _ = url;
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let surface = analysis.architecture_surface();
        let output_path = write_architecture_surface_artifact(&surface, &fixture, None).unwrap();

        assert_eq!(
            output_path,
            default_output_dir(&fixture).join(ARCHITECTURE_SURFACE_FILE)
        );
        assert!(output_path.exists());
    }

    #[test]
    fn writes_semantic_graph_artifact_without_full_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod models;
use crate::models::User;
fn main() {
    let _ = User {};
}
"#,
        )
        .unwrap();
        fs::write(fixture.join("src/models.rs"), b"pub struct User;\n").unwrap();

        let graph_project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let output_path = write_semantic_graph_artifact(&graph_project, None).unwrap();

        assert_eq!(
            output_path,
            default_output_dir(&fixture).join(SEMANTIC_GRAPH_FILE)
        );
        let payload: Value =
            serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();
        assert!(payload["resolved_edges"].as_array().is_some());
    }

    #[test]
    fn writes_dependency_graph_artifact_without_full_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod models;
use crate::models::User;
fn main() {
    let _ = User {};
}
"#,
        )
        .unwrap();
        fs::write(fixture.join("src/models.rs"), b"pub struct User;\n").unwrap();

        let graph_project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let output_path = write_dependency_graph_artifact(&graph_project, None).unwrap();

        assert_eq!(
            output_path,
            default_output_dir(&fixture).join(DEPENDENCY_GRAPH_FILE)
        );
        let payload: Value =
            serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();
        assert_eq!(
            payload["dependency_graph"]["view"],
            Value::String(String::from("dependency_view"))
        );
        assert!(payload["dependency_graph"]["edges"].as_array().is_some());
    }

    #[test]
    fn writes_evidence_graph_artifact_without_full_analysis() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"mod models;
use crate::models::User;
fn main() {
    let _ = User {};
}
"#,
        )
        .unwrap();
        fs::write(fixture.join("src/models.rs"), b"pub struct User;\n").unwrap();

        let graph_project = build_semantic_graph_project(&fixture, &ScanConfig::default()).unwrap();
        let output_path = write_evidence_graph_artifact(&graph_project, None).unwrap();

        assert_eq!(
            output_path,
            default_output_dir(&fixture).join(EVIDENCE_GRAPH_FILE)
        );
        let payload: Value =
            serde_json::from_str(&fs::read_to_string(output_path).unwrap()).unwrap();
        assert_eq!(
            payload["evidence_graph"]["view"],
            Value::String(String::from("evidence_view"))
        );
        assert!(payload["evidence_graph"]["edges"].as_array().is_some());
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-artifacts-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
