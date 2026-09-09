use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

use sarif_rust::types::{
    ArtifactLocation as SarifArtifactLocation, Level as SarifLevel, ReportingDescriptor,
    ReportingDescriptorReference, Result as SarifResult, Run as SarifRun,
};

use crate::evidence::EvidenceAnchor;

mod process;
mod report;
use process::{CapturePaths, ProcessError, ReportStream};

const REPORTS_DIR: &str = "reports";
const RAW_DIR: &str = "raw";
const GITLEAKS_TOOL: &str = "gitleaks";
const OPENGREP_TOOL: &str = "opengrep";
const PIP_AUDIT_TOOL: &str = "pip-audit";
const OSV_SCANNER_TOOL: &str = "osv-scanner";
const COMPOSER_AUDIT_TOOL: &str = "composer-audit";
const NPM_AUDIT_TOOL: &str = "npm-audit";
const CARGO_DENY_TOOL: &str = "cargo-deny";
const CARGO_CLIPPY_TOOL: &str = "cargo-clippy";
const RUFF_TOOL: &str = "ruff";
const TRIVY_TOOL: &str = "trivy";
const GRYPE_TOOL: &str = "grype";

const SUPPORTED_EXTERNAL_TOOLS: &[&str] = &[
    OPENGREP_TOOL,
    TRIVY_TOOL,
    GRYPE_TOOL,
    RUFF_TOOL,
    GITLEAKS_TOOL,
    PIP_AUDIT_TOOL,
    OSV_SCANNER_TOOL,
    COMPOSER_AUDIT_TOOL,
    NPM_AUDIT_TOOL,
    CARGO_DENY_TOOL,
    CARGO_CLIPPY_TOOL,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalToolStatus {
    Passed,
    Findings,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalFailureKind {
    InvalidReport,
    ProcessExit,
    ProcessIo,
    Timeout,
    UnsupportedTool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExternalCheckStatus {
    NotRequested,
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExternalCheckSummary {
    pub status: ExternalCheckStatus,
    pub requested: usize,
    pub completed: usize,
    pub failed: usize,
    pub unavailable: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSeverity {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalToolRun {
    pub tool: String,
    pub command: Vec<String>,
    pub status: ExternalToolStatus,
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_kind: Option<ExternalFailureKind>,
    pub artifact_path: String,
    pub summary: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalFinding {
    pub tool: String,
    pub domain: String,
    pub category: String,
    pub rule_id: String,
    pub severity: ExternalSeverity,
    pub confidence: ExternalConfidence,
    pub file_path: Option<PathBuf>,
    pub line: Option<usize>,
    #[serde(default)]
    pub locations: Vec<EvidenceAnchor>,
    pub message: String,
    pub fingerprint: String,
    pub extras: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExternalAnalysisResult {
    pub tool_runs: Vec<ExternalToolRun>,
    pub findings: Vec<ExternalFinding>,
    pub raw_artifact_dir: Option<String>,
}

impl ExternalAnalysisResult {
    pub fn is_empty(&self) -> bool {
        self.tool_runs.is_empty() && self.findings.is_empty()
    }

    /// Whether every requested tool produced a usable result, including findings.
    pub fn is_complete(&self) -> bool {
        self.tool_runs.iter().all(|run| {
            matches!(
                run.status,
                ExternalToolStatus::Passed | ExternalToolStatus::Findings
            )
        })
    }

    pub fn check_summary(&self) -> ExternalCheckSummary {
        ExternalCheckSummary {
            status: if self.tool_runs.is_empty() {
                ExternalCheckStatus::NotRequested
            } else if self.is_complete() {
                ExternalCheckStatus::Complete
            } else {
                ExternalCheckStatus::Incomplete
            },
            requested: self.tool_runs.len(),
            completed: self
                .tool_runs
                .iter()
                .filter(|run| {
                    matches!(
                        run.status,
                        ExternalToolStatus::Passed | ExternalToolStatus::Findings
                    )
                })
                .count(),
            failed: self
                .tool_runs
                .iter()
                .filter(|run| run.status == ExternalToolStatus::Failed)
                .count(),
            unavailable: self
                .tool_runs
                .iter()
                .filter(|run| run.status == ExternalToolStatus::Unavailable)
                .count(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ExternalAnalysisError {
    #[error("failed to resolve external-analysis project root {path}: {source}")]
    ProjectRoot {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to create external-analysis raw artifact directory {path}: {source}")]
    CreateRawDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

pub fn collect_external_analysis(
    project_path: &Path,
    output_dir: &Path,
    selected_tools: &[String],
) -> Result<ExternalAnalysisResult, ExternalAnalysisError> {
    let normalized_tools = normalize_selected_tools(selected_tools);
    if normalized_tools.is_empty() {
        return Ok(ExternalAnalysisResult::default());
    }

    let project_path =
        fs::canonicalize(project_path).map_err(|source| ExternalAnalysisError::ProjectRoot {
            path: project_path.to_path_buf(),
            source,
        })?;

    let run_id = current_run_id();
    let raw_dir = output_dir.join(REPORTS_DIR).join(&run_id).join(RAW_DIR);
    fs::create_dir_all(&raw_dir).map_err(|source| ExternalAnalysisError::CreateRawDir {
        path: raw_dir.clone(),
        source,
    })?;
    let raw_dir =
        fs::canonicalize(&raw_dir).map_err(|source| ExternalAnalysisError::CreateRawDir {
            path: raw_dir,
            source,
        })?;

    let mut result = ExternalAnalysisResult {
        tool_runs: Vec::new(),
        findings: Vec::new(),
        raw_artifact_dir: Some(raw_dir.display().to_string()),
    };

    for tool in normalized_tools {
        let (tool_run, findings) = match tool.as_str() {
            OPENGREP_TOOL => run_opengrep(&project_path, &raw_dir),
            TRIVY_TOOL => run_trivy(&project_path, &raw_dir),
            GRYPE_TOOL => run_grype(&project_path, &raw_dir),
            RUFF_TOOL => run_ruff(&project_path, &raw_dir),
            GITLEAKS_TOOL => run_gitleaks(&project_path, &raw_dir),
            PIP_AUDIT_TOOL => run_pip_audit(&project_path, &raw_dir),
            OSV_SCANNER_TOOL => run_osv_scanner(&project_path, &raw_dir),
            COMPOSER_AUDIT_TOOL => run_composer_audit(&project_path, &raw_dir),
            NPM_AUDIT_TOOL => run_npm_audit(&project_path, &raw_dir),
            CARGO_DENY_TOOL => run_cargo_deny(&project_path, &raw_dir),
            CARGO_CLIPPY_TOOL => run_cargo_clippy(&project_path, &raw_dir),
            unsupported => (
                ExternalToolRun {
                    tool: unsupported.to_string(),
                    command: Vec::new(),
                    status: ExternalToolStatus::Failed,
                    exit_code: None,
                    failure_kind: Some(ExternalFailureKind::UnsupportedTool),
                    artifact_path: raw_dir
                        .join(format!("{unsupported}.json"))
                        .display()
                        .to_string(),
                    summary: summary_map(&[(
                        "error",
                        Value::String(format!("unsupported external tool: {unsupported}")),
                    )]),
                },
                Vec::new(),
            ),
        };
        result.tool_runs.push(tool_run);
        result.findings.extend(findings);
    }

    Ok(result)
}

fn normalize_selected_tools(selected_tools: &[String]) -> Vec<String> {
    let mut tools = Vec::new();
    let mut seen = HashSet::<String>::new();

    for raw in selected_tools {
        for token in raw.split(',') {
            let tool = token.trim().to_lowercase();
            if tool.is_empty() {
                continue;
            }
            if tool == "all" {
                for candidate in SUPPORTED_EXTERNAL_TOOLS {
                    let candidate = (*candidate).to_string();
                    if seen.insert(candidate.clone()) {
                        tools.push(candidate);
                    }
                }
                continue;
            }
            if seen.insert(tool.clone()) {
                tools.push(tool);
            }
        }
    }

    tools
}

fn current_run_id() -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{timestamp}-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

#[derive(Debug, Clone, Copy)]
struct SarifFallback {
    domain: &'static str,
    category: &'static str,
}

#[derive(Debug, Clone, Default)]
struct SarifRuleInfo {
    name: Option<String>,
    short_description: Option<String>,
    full_description: Option<String>,
    help_uri: Option<String>,
    tags: Vec<String>,
}

fn run_sarif_tool(
    tool: &str,
    command: Vec<String>,
    artifact_name: &str,
    project_path: &Path,
    raw_dir: &Path,
) -> (ExternalToolRun, Vec<ExternalFinding>) {
    run_tool(
        tool,
        command,
        project_path,
        &raw_dir.join(artifact_name),
        Duration::from_secs(300),
        ReportStream::Stdout,
    )
}

fn run_opengrep(project_path: &Path, raw_dir: &Path) -> (ExternalToolRun, Vec<ExternalFinding>) {
    run_sarif_tool(
        OPENGREP_TOOL,
        vec![
            String::from(OPENGREP_TOOL),
            String::from("scan"),
            String::from("--config"),
            String::from("auto"),
            String::from("--sarif"),
            project_path.display().to_string(),
        ],
        "opengrep.sarif",
        project_path,
        raw_dir,
    )
}

fn run_trivy(project_path: &Path, raw_dir: &Path) -> (ExternalToolRun, Vec<ExternalFinding>) {
    run_sarif_tool(
        TRIVY_TOOL,
        vec![
            String::from(TRIVY_TOOL),
            String::from("fs"),
            String::from("--format"),
            String::from("sarif"),
            String::from("--scanners"),
            String::from("vuln,secret,misconfig,license"),
            project_path.display().to_string(),
        ],
        "trivy.sarif",
        project_path,
        raw_dir,
    )
}

fn run_grype(project_path: &Path, raw_dir: &Path) -> (ExternalToolRun, Vec<ExternalFinding>) {
    run_sarif_tool(
        GRYPE_TOOL,
        vec![
            String::from(GRYPE_TOOL),
            format!("dir:{}", project_path.display()),
            String::from("-o"),
            String::from("sarif"),
        ],
        "grype.sarif",
        project_path,
        raw_dir,
    )
}

fn run_ruff(project_path: &Path, raw_dir: &Path) -> (ExternalToolRun, Vec<ExternalFinding>) {
    let artifact_path = raw_dir.join("ruff-security.json");
    let command = vec![
        String::from(RUFF_TOOL),
        String::from("check"),
        String::from("--select"),
        String::from("S"),
        String::from("--output-format"),
        String::from("json"),
        String::from("--exit-zero"),
        project_path.display().to_string(),
    ];

    run_tool(
        RUFF_TOOL,
        command,
        project_path,
        &artifact_path,
        Duration::from_secs(60),
        ReportStream::Stdout,
    )
}

fn run_gitleaks(project_path: &Path, raw_dir: &Path) -> (ExternalToolRun, Vec<ExternalFinding>) {
    let artifact_path = raw_dir.join("gitleaks.json");
    let command = vec![
        String::from(GITLEAKS_TOOL),
        String::from("detect"),
        String::from("--source"),
        project_path.display().to_string(),
        String::from("--report-format"),
        String::from("json"),
        String::from("--report-path"),
        artifact_path.display().to_string(),
        String::from("--no-banner"),
        String::from("--exit-code"),
        String::from("0"),
        String::from("--redact"),
    ];

    run_tool(
        GITLEAKS_TOOL,
        command,
        project_path,
        &artifact_path,
        Duration::from_secs(120),
        ReportStream::File,
    )
}

fn run_pip_audit(project_path: &Path, raw_dir: &Path) -> (ExternalToolRun, Vec<ExternalFinding>) {
    let artifact_path = raw_dir.join("pip-audit.json");
    let command = vec![
        String::from(PIP_AUDIT_TOOL),
        String::from("--strict"),
        String::from("--format"),
        String::from("json"),
        String::from("--output"),
        artifact_path.display().to_string(),
        project_path.display().to_string(),
    ];

    run_tool(
        PIP_AUDIT_TOOL,
        command,
        project_path,
        &artifact_path,
        Duration::from_secs(120),
        ReportStream::File,
    )
}

fn run_osv_scanner(project_path: &Path, raw_dir: &Path) -> (ExternalToolRun, Vec<ExternalFinding>) {
    let artifact_path = raw_dir.join("osv-scanner.json");
    let command = vec![
        String::from(OSV_SCANNER_TOOL),
        String::from("scan"),
        String::from("source"),
        String::from("--recursive"),
        project_path.display().to_string(),
        String::from("--format"),
        String::from("json"),
        String::from("--output-file"),
        artifact_path.display().to_string(),
    ];

    run_tool(
        OSV_SCANNER_TOOL,
        command,
        project_path,
        &artifact_path,
        Duration::from_secs(120),
        ReportStream::File,
    )
}

fn run_composer_audit(
    project_path: &Path,
    raw_dir: &Path,
) -> (ExternalToolRun, Vec<ExternalFinding>) {
    let artifact_path = raw_dir.join("composer-audit.json");
    let command = vec![
        String::from("composer"),
        String::from("--no-plugins"),
        String::from("--no-scripts"),
        String::from("audit"),
        String::from("--locked"),
        String::from("--format=json"),
        String::from("--no-interaction"),
        String::from("--no-ansi"),
    ];

    if !project_path.join("composer.json").is_file()
        || !project_path.join("composer.lock").is_file()
    {
        return unavailable_run(
            COMPOSER_AUDIT_TOOL,
            command,
            &artifact_path,
            "composer.json and composer.lock are required for a source-locked audit",
        );
    }
    run_tool(
        COMPOSER_AUDIT_TOOL,
        command,
        project_path,
        &artifact_path,
        Duration::from_secs(120),
        ReportStream::Stdout,
    )
}

fn run_npm_audit(project_path: &Path, raw_dir: &Path) -> (ExternalToolRun, Vec<ExternalFinding>) {
    let artifact_path = raw_dir.join("npm-audit.json");
    let command = vec![
        String::from("npm"),
        String::from("audit"),
        String::from("--json"),
        String::from("--omit=dev"),
    ];

    if !project_path.join("package.json").exists() {
        return unavailable_run(
            NPM_AUDIT_TOOL,
            command,
            &artifact_path,
            "package.json not found",
        );
    }
    run_tool(
        NPM_AUDIT_TOOL,
        command,
        project_path,
        &artifact_path,
        Duration::from_secs(120),
        ReportStream::Stdout,
    )
}

fn run_cargo_deny(project_path: &Path, raw_dir: &Path) -> (ExternalToolRun, Vec<ExternalFinding>) {
    let artifact_path = raw_dir.join("cargo-deny.jsonl");
    let command = vec![
        String::from(CARGO_DENY_TOOL),
        String::from("--format"),
        String::from("json"),
        String::from("check"),
        String::from("advisories"),
        String::from("bans"),
        String::from("licenses"),
        String::from("sources"),
        String::from("--hide-inclusion-graph"),
    ];

    if !project_path.join("Cargo.toml").exists() {
        return unavailable_run(
            CARGO_DENY_TOOL,
            command,
            &artifact_path,
            "Cargo.toml not found",
        );
    }
    run_tool(
        CARGO_DENY_TOOL,
        command,
        project_path,
        &artifact_path,
        Duration::from_secs(300),
        ReportStream::Stderr,
    )
}

fn run_cargo_clippy(
    project_path: &Path,
    raw_dir: &Path,
) -> (ExternalToolRun, Vec<ExternalFinding>) {
    let artifact_path = raw_dir.join("cargo-clippy.jsonl");
    let command = vec![
        String::from("cargo"),
        String::from("clippy"),
        String::from("--message-format=json"),
        String::from("--all-targets"),
        String::from("--workspace"),
        String::from("--"),
        String::from("-W"),
        String::from("clippy::all"),
    ];

    if !project_path.join("Cargo.toml").exists() {
        return unavailable_run(
            CARGO_CLIPPY_TOOL,
            command,
            &artifact_path,
            "Cargo.toml not found",
        );
    }
    run_tool(
        CARGO_CLIPPY_TOOL,
        command,
        project_path,
        &artifact_path,
        Duration::from_secs(300),
        ReportStream::Stdout,
    )
}

fn run_tool(
    tool: &str,
    command: Vec<String>,
    project_path: &Path,
    artifact_path: &Path,
    timeout: Duration,
    stream: ReportStream,
) -> (ExternalToolRun, Vec<ExternalFinding>) {
    if command.first().and_then(|binary| which(binary)).is_none() {
        return unavailable_run(tool, command, artifact_path, "executable not found on PATH");
    }
    let captures = CapturePaths::new(artifact_path, stream);
    let execution = process::run(&command, Some(project_path), &captures, timeout);
    let (mut run, findings) = match execution {
        Err(error) => {
            let (kind, message) = match error {
                ProcessError::Timeout(duration) => (
                    ExternalFailureKind::Timeout,
                    format!("timed out after {}ms", duration.as_millis()),
                ),
                ProcessError::Io(error) => (ExternalFailureKind::ProcessIo, error.to_string()),
            };
            failed_run(tool, command, artifact_path, None, kind, message)
        }
        Ok(status) => match report::read(tool, project_path, artifact_path) {
            Ok(report) => completed_run(
                tool,
                command,
                artifact_path,
                status.code(),
                report.findings,
                Map::new(),
                report.error,
            ),
            Err(error) => failed_run(
                tool,
                command,
                artifact_path,
                status.code(),
                ExternalFailureKind::InvalidReport,
                error,
            ),
        },
    };
    run.summary.insert(
        "stdout_path".into(),
        Value::String(captures.stdout.display().to_string()),
    );
    run.summary.insert(
        "stderr_path".into(),
        Value::String(captures.stderr.display().to_string()),
    );
    run.summary.insert(
        "process_scope".into(),
        Value::String(
            if cfg!(unix) {
                "process_group"
            } else {
                "direct_child"
            }
            .into(),
        ),
    );
    match captures.stderr_preview() {
        Ok(preview) => run.summary.extend(summary_map(&stderr_summary(preview))),
        Err(error) => {
            run.summary
                .insert("stderr_read_error".into(), Value::String(error.to_string()));
            if matches!(
                run.status,
                ExternalToolStatus::Passed | ExternalToolStatus::Findings
            ) {
                run.status = ExternalToolStatus::Failed;
                run.failure_kind = Some(ExternalFailureKind::ProcessIo);
                run.summary.insert(
                    "error".into(),
                    Value::String(format!("raw stderr is unavailable: {error}")),
                );
            }
        }
    }
    (run, findings)
}

fn completed_run(
    tool: &str,
    command: Vec<String>,
    artifact_path: &Path,
    exit_code: Option<i32>,
    findings: Vec<ExternalFinding>,
    mut summary: Map<String, Value>,
    report_error: Option<String>,
) -> (ExternalToolRun, Vec<ExternalFinding>) {
    summary.insert(
        String::from("finding_count"),
        Value::from(findings.len() as u64),
    );
    let failure = report_error
        .map(|error| (ExternalFailureKind::InvalidReport, error))
        .or_else(|| {
            report::exit_error(tool, exit_code, !findings.is_empty())
                .map(|error| (ExternalFailureKind::ProcessExit, error))
        });
    let status = if failure.is_some() {
        ExternalToolStatus::Failed
    } else if findings.is_empty() {
        ExternalToolStatus::Passed
    } else {
        ExternalToolStatus::Findings
    };

    if let Some((_, error)) = &failure {
        summary.insert("error".into(), Value::String(error.clone()));
    }
    (
        ExternalToolRun {
            tool: tool.to_string(),
            command,
            status,
            exit_code,
            failure_kind: failure.map(|(kind, _)| kind),
            artifact_path: artifact_path.display().to_string(),
            summary,
        },
        findings,
    )
}

fn failed_run(
    tool: &str,
    command: Vec<String>,
    artifact_path: &Path,
    exit_code: Option<i32>,
    failure_kind: ExternalFailureKind,
    error: String,
) -> (ExternalToolRun, Vec<ExternalFinding>) {
    (
        ExternalToolRun {
            tool: tool.to_string(),
            command,
            status: ExternalToolStatus::Failed,
            exit_code,
            failure_kind: Some(failure_kind),
            artifact_path: artifact_path.display().to_string(),
            summary: summary_map(&[("error", Value::String(error))]),
        },
        Vec::new(),
    )
}

fn unavailable_run(
    tool: &str,
    command: Vec<String>,
    artifact_path: &Path,
    message: &str,
) -> (ExternalToolRun, Vec<ExternalFinding>) {
    (
        ExternalToolRun {
            tool: tool.to_string(),
            command,
            status: ExternalToolStatus::Unavailable,
            exit_code: None,
            failure_kind: None,
            artifact_path: artifact_path.display().to_string(),
            summary: summary_map(&[("message", Value::String(message.to_string()))]),
        },
        Vec::new(),
    )
}

fn summary_map(entries: &[(&str, Value)]) -> Map<String, Value> {
    entries
        .iter()
        .map(|(key, value)| (String::from(*key), value.clone()))
        .collect()
}

fn stderr_summary(stderr: String) -> Vec<(&'static str, Value)> {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        Vec::new()
    } else {
        vec![("stderr", Value::String(trimmed.to_string()))]
    }
}

fn which(binary: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
fn parse_sarif_output(
    tool: &str,
    project_path: &Path,
    payload: &str,
    fallback: SarifFallback,
) -> Vec<ExternalFinding> {
    let sarif = sarif_rust::from_str(payload).expect("valid SARIF fixture");

    let mut findings = Vec::new();
    for run in sarif.runs {
        let rule_lookup = build_sarif_rule_lookup(&run);
        findings.extend(parse_sarif_run(
            tool,
            project_path,
            &run,
            &rule_lookup,
            fallback,
        ));
    }
    findings
}

fn parse_sarif_run(
    tool: &str,
    project_path: &Path,
    run: &SarifRun,
    rule_lookup: &HashMap<String, SarifRuleInfo>,
    fallback: SarifFallback,
) -> Vec<ExternalFinding> {
    let mut findings = Vec::new();
    let tool_name = if run.tool.driver.name.trim().is_empty() {
        tool.to_string()
    } else {
        run.tool.driver.name.clone()
    };

    for result in run.results.as_ref().into_iter().flatten() {
        let rule_id = sarif_rule_id(result, rule_lookup).unwrap_or_else(|| String::from("sarif"));
        let rule_info = rule_lookup.get(&rule_id);
        let (file_path, line) = sarif_primary_location(project_path, result);
        let locations = sarif_locations(project_path, result, file_path.clone(), line);
        let message = sarif_message_text(result).unwrap_or_else(|| rule_id.clone());
        let (domain, category) =
            infer_sarif_domain_category(tool, &rule_id, &message, rule_info, fallback);
        let severity = sarif_severity(result, rule_info);
        let confidence = sarif_confidence(tool, category.as_str());
        let fingerprint = sarif_fingerprint(tool, &rule_id, &file_path, line, &message, result);
        findings.push(ExternalFinding {
            tool: tool_name.clone(),
            domain,
            category,
            rule_id,
            severity,
            confidence,
            file_path,
            line,
            locations,
            message,
            fingerprint,
            extras: sarif_extras(result, rule_info),
        });
    }

    findings
}

fn build_sarif_rule_lookup(run: &SarifRun) -> HashMap<String, SarifRuleInfo> {
    let mut lookup = HashMap::new();
    if let Some(rules) = run.tool.driver.rules.as_ref() {
        for rule in rules {
            lookup.insert(rule.id.clone(), sarif_rule_info(rule));
        }
    }
    lookup
}

fn sarif_rule_info(rule: &ReportingDescriptor) -> SarifRuleInfo {
    let tags = rule
        .properties
        .as_ref()
        .and_then(|properties| properties.get("tags"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(|value| value.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    SarifRuleInfo {
        name: rule.name.clone(),
        short_description: rule
            .short_description
            .as_ref()
            .map(|value| value.text.clone()),
        full_description: rule
            .full_description
            .as_ref()
            .map(|value| value.text.clone()),
        help_uri: rule.help_uri.clone(),
        tags,
    }
}

fn sarif_rule_id(
    result: &SarifResult,
    _rule_lookup: &HashMap<String, SarifRuleInfo>,
) -> Option<String> {
    result
        .rule_id
        .clone()
        .or_else(|| result.rule.as_ref().and_then(sarif_rule_reference_id))
}

fn sarif_rule_reference_id(reference: &ReportingDescriptorReference) -> Option<String> {
    reference.id.clone()
}

fn infer_sarif_domain_category(
    tool: &str,
    rule_id: &str,
    message: &str,
    rule_info: Option<&SarifRuleInfo>,
    fallback: SarifFallback,
) -> (String, String) {
    let mut signals = vec![
        tool.to_ascii_lowercase(),
        rule_id.to_ascii_lowercase(),
        message.to_ascii_lowercase(),
    ];
    if let Some(rule_info) = rule_info {
        if let Some(name) = &rule_info.name {
            signals.push(name.to_ascii_lowercase());
        }
        if let Some(description) = &rule_info.short_description {
            signals.push(description.to_ascii_lowercase());
        }
        if let Some(description) = &rule_info.full_description {
            signals.push(description.to_ascii_lowercase());
        }
        signals.extend(rule_info.tags.iter().cloned());
    }

    if signals.iter().any(|signal| {
        signal.contains("secret")
            || signal.contains("access key")
            || signal.contains("api key")
            || signal.contains("token")
            || signal.contains("credential")
            || signal.contains("password")
    }) {
        return (String::from("security"), String::from("secrets"));
    }
    if signals
        .iter()
        .any(|signal| signal.contains("license") || signal.contains("licence"))
    {
        return (String::from("security"), String::from("license"));
    }
    if signals.iter().any(|signal| {
        signal.contains("misconfig")
            || signal.contains("misconfiguration")
            || signal.contains("configuration")
            || signal.contains("policy")
    }) {
        return (String::from("security"), String::from("source_policy"));
    }
    if signals.iter().any(|signal| {
        signal.contains("vuln")
            || signal.contains("advisory")
            || signal.contains("cve")
            || signal.contains("ghsa")
            || signal.contains("rustsec")
            || signal.contains("osv")
            || signal.contains("package")
    }) {
        return (String::from("security"), String::from("sca"));
    }

    (
        String::from(fallback.domain),
        String::from(fallback.category),
    )
}

fn sarif_primary_location(
    project_path: &Path,
    result: &SarifResult,
) -> (Option<PathBuf>, Option<usize>) {
    let location = result
        .locations
        .as_ref()
        .and_then(|locations| locations.first());
    let file_path = location
        .and_then(|location| location.physical_location.as_ref())
        .and_then(|physical| physical.artifact_location.as_ref())
        .and_then(sarif_artifact_path)
        .map(|path| project_relative_path(project_path, &path))
        .or_else(|| {
            result
                .analysis_target
                .as_ref()
                .and_then(sarif_artifact_path)
                .map(|path| project_relative_path(project_path, &path))
        });
    let line = location
        .and_then(|location| location.physical_location.as_ref())
        .and_then(|physical| physical.region.as_ref())
        .and_then(|region| region.start_line)
        .map(|value| value as usize);
    (file_path, line)
}

fn sarif_locations(
    project_path: &Path,
    result: &SarifResult,
    primary_file_path: Option<PathBuf>,
    primary_line: Option<usize>,
) -> Vec<EvidenceAnchor> {
    let mut locations = Vec::new();

    if let Some(path) = primary_file_path {
        push_sarif_location(&mut locations, path, primary_line, "primary");
    }

    for location in result.locations.as_ref().into_iter().flatten().skip(1) {
        if let Some((file_path, line)) = sarif_location_anchor(project_path, location) {
            push_sarif_location(&mut locations, file_path, line, "supporting");
        }
    }

    for location in result.related_locations.as_ref().into_iter().flatten() {
        if let Some((file_path, line)) = sarif_location_anchor(project_path, location) {
            push_sarif_location(&mut locations, file_path, line, "supporting");
        }
    }

    locations
}

fn sarif_location_anchor(
    project_path: &Path,
    location: &sarif_rust::types::Location,
) -> Option<(PathBuf, Option<usize>)> {
    let file_path = location
        .physical_location
        .as_ref()
        .and_then(|physical| physical.artifact_location.as_ref())
        .and_then(sarif_artifact_path)
        .map(|path| project_relative_path(project_path, &path))?;
    let line = location
        .physical_location
        .as_ref()
        .and_then(|physical| physical.region.as_ref())
        .and_then(|region| region.start_line)
        .map(|value| value as usize);
    Some((file_path, line))
}

fn push_sarif_location(
    locations: &mut Vec<EvidenceAnchor>,
    file_path: PathBuf,
    line: Option<usize>,
    label: &str,
) {
    if locations
        .iter()
        .any(|location| location.file_path == file_path && location.line == line)
    {
        return;
    }
    locations.push(EvidenceAnchor {
        file_path,
        line,
        label: String::from(label),
    });
}

fn sarif_artifact_path(location: &SarifArtifactLocation) -> Option<String> {
    let raw = location.uri.as_ref()?;
    Some(
        raw.strip_prefix("file://")
            .unwrap_or(raw)
            .trim()
            .to_string(),
    )
}

fn sarif_message_text(result: &SarifResult) -> Option<String> {
    result
        .message
        .text
        .clone()
        .or_else(|| result.message.markdown.clone())
        .or_else(|| result.message.id.clone())
}

fn sarif_severity(result: &SarifResult, rule_info: Option<&SarifRuleInfo>) -> ExternalSeverity {
    let _ = rule_info;
    match result.level.clone().unwrap_or(SarifLevel::Warning) {
        SarifLevel::Error => ExternalSeverity::High,
        SarifLevel::Warning => ExternalSeverity::Medium,
        SarifLevel::Note | SarifLevel::None => ExternalSeverity::Low,
    }
}

fn sarif_confidence(tool: &str, category: &str) -> ExternalConfidence {
    match (tool, category) {
        (GRYPE_TOOL, _) | (TRIVY_TOOL, "sca") | (TRIVY_TOOL, "license") => ExternalConfidence::High,
        (_, "secrets") => ExternalConfidence::High,
        _ => ExternalConfidence::Medium,
    }
}

fn sarif_fingerprint(
    tool: &str,
    rule_id: &str,
    file_path: &Option<PathBuf>,
    line: Option<usize>,
    message: &str,
    result: &SarifResult,
) -> String {
    if let Some(value) = result
        .fingerprints
        .as_ref()
        .and_then(|fingerprints| fingerprints.values().next())
    {
        return value.clone();
    }
    if let Some(value) = result
        .partial_fingerprints
        .as_ref()
        .and_then(|fingerprints| fingerprints.values().next())
    {
        return value.clone();
    }
    let file_component = file_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();
    let line_component = line.unwrap_or(1).to_string();
    stable_fingerprint(&[tool, rule_id, &file_component, &line_component, message])
}

fn sarif_extras(result: &SarifResult, rule_info: Option<&SarifRuleInfo>) -> Map<String, Value> {
    let mut extras = Map::new();
    if let Some(level) = &result.level {
        extras.insert(
            String::from("sarif_level"),
            Value::String(level.to_string()),
        );
    }
    if let Some(rule_info) = rule_info {
        if let Some(name) = &rule_info.name {
            extras.insert(String::from("rule_name"), Value::String(name.clone()));
        }
        if let Some(help_uri) = &rule_info.help_uri {
            extras.insert(String::from("help_uri"), Value::String(help_uri.clone()));
        }
        if !rule_info.tags.is_empty() {
            extras.insert(
                String::from("tags"),
                Value::Array(rule_info.tags.iter().cloned().map(Value::String).collect()),
            );
        }
    }
    if let Some(value) = &result.properties {
        extras.insert(
            String::from("sarif_properties"),
            Value::Object(value.clone().into_iter().collect()),
        );
    }
    if let Some(related_locations) = &result.related_locations {
        extras.insert(
            String::from("related_location_count"),
            Value::from(related_locations.len() as u64),
        );
    }
    if let Some(code_flows) = &result.code_flows {
        extras.insert(
            String::from("code_flow_count"),
            Value::from(code_flows.len() as u64),
        );
        let thread_flow_count: usize = code_flows
            .iter()
            .map(|code_flow| code_flow.thread_flows.len())
            .sum();
        let thread_flow_location_count: usize = code_flows
            .iter()
            .flat_map(|code_flow| &code_flow.thread_flows)
            .map(|thread_flow| thread_flow.locations.len())
            .sum();
        extras.insert(
            String::from("thread_flow_count"),
            Value::from(thread_flow_count as u64),
        );
        extras.insert(
            String::from("thread_flow_location_count"),
            Value::from(thread_flow_location_count as u64),
        );
    }
    if let Some(stacks) = &result.stacks {
        extras.insert(
            String::from("stack_count"),
            Value::from(stacks.len() as u64),
        );
        let stack_frame_count: usize = stacks.iter().map(|stack| stack.frames.len()).sum();
        extras.insert(
            String::from("stack_frame_count"),
            Value::from(stack_frame_count as u64),
        );
    }
    extras
}

fn parse_ruff_payload(project_path: &Path, payload: &Value) -> Vec<ExternalFinding> {
    let Some(items) = payload.as_array() else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| item.as_object())
        .map(|item| ExternalFinding {
            tool: String::from(RUFF_TOOL),
            domain: String::from("security"),
            category: String::from("sast"),
            rule_id: item
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            severity: ruff_severity(item.get("code").and_then(Value::as_str).unwrap_or_default()),
            confidence: ExternalConfidence::Medium,
            file_path: item
                .get("filename")
                .and_then(Value::as_str)
                .map(|path| project_relative_path(project_path, path)),
            line: item
                .get("location")
                .and_then(Value::as_object)
                .and_then(|location| location.get("row"))
                .and_then(Value::as_u64)
                .map(|row| row as usize),
            locations: Vec::new(),
            message: item
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string(),
            fingerprint: stable_fingerprint(&[
                RUFF_TOOL,
                item.get("code").and_then(Value::as_str).unwrap_or_default(),
                item.get("filename")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                &item
                    .get("location")
                    .and_then(Value::as_object)
                    .and_then(|location| location.get("row"))
                    .and_then(Value::as_u64)
                    .unwrap_or_default()
                    .to_string(),
                item.get("message")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            ]),
            extras: map_from_pairs([(
                "documentation_url",
                item.get("url").cloned().unwrap_or(Value::Null),
            )]),
        })
        .collect()
}

fn parse_gitleaks_payload(project_path: &Path, payload: &Value) -> Vec<ExternalFinding> {
    let Some(items) = payload.as_array() else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| item.as_object())
        .map(|item| {
            let file = item.get("File").and_then(Value::as_str).unwrap_or_default();
            let line = item
                .get("StartLine")
                .and_then(Value::as_u64)
                .or_else(|| item.get("Line").and_then(Value::as_u64))
                .map(|line| line as usize);
            ExternalFinding {
                tool: String::from(GITLEAKS_TOOL),
                domain: String::from("security"),
                category: String::from("secrets"),
                rule_id: item
                    .get("RuleID")
                    .and_then(Value::as_str)
                    .unwrap_or("gitleaks")
                    .to_string(),
                severity: ExternalSeverity::High,
                confidence: ExternalConfidence::Medium,
                file_path: Some(project_relative_path(project_path, file)),
                line,
                locations: Vec::new(),
                message: item
                    .get("Description")
                    .and_then(Value::as_str)
                    .unwrap_or("Potential secret detected by Gitleaks")
                    .trim()
                    .to_string(),
                fingerprint: item
                    .get("Fingerprint")
                    .and_then(Value::as_str)
                    .map(String::from)
                    .unwrap_or_else(|| {
                        stable_fingerprint(&[
                            GITLEAKS_TOOL,
                            file,
                            &line.unwrap_or(1).to_string(),
                            item.get("RuleID")
                                .and_then(Value::as_str)
                                .unwrap_or_default(),
                        ])
                    }),
                extras: map_from_pairs([
                    ("commit", item.get("Commit").cloned().unwrap_or(Value::Null)),
                    ("author", item.get("Author").cloned().unwrap_or(Value::Null)),
                    (
                        "entropy",
                        item.get("Entropy").cloned().unwrap_or(Value::Null),
                    ),
                    ("match", item.get("Match").cloned().unwrap_or(Value::Null)),
                ]),
            }
        })
        .collect()
}

fn parse_pip_audit_payload(payload: &Value) -> Vec<ExternalFinding> {
    let dependencies = payload
        .as_array()
        .cloned()
        .or_else(|| {
            payload
                .get("dependencies")
                .and_then(Value::as_array)
                .cloned()
        })
        .unwrap_or_default();

    let mut findings = Vec::new();
    for dependency in dependencies {
        let Some(dependency) = dependency.as_object() else {
            continue;
        };
        let package_name = dependency
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let package_version = dependency
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(vulns) = dependency.get("vulns").and_then(Value::as_array) else {
            continue;
        };
        for vuln in vulns {
            let Some(vuln) = vuln.as_object() else {
                continue;
            };
            let vuln_id = vuln
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("vulnerability");
            findings.push(ExternalFinding {
                tool: String::from(PIP_AUDIT_TOOL),
                domain: String::from("security"),
                category: String::from("sca"),
                rule_id: vuln_id.to_string(),
                severity: ExternalSeverity::High,
                confidence: ExternalConfidence::High,
                file_path: None,
                line: None,
                locations: Vec::new(),
                message: format!("{package_name} {package_version} is affected by {vuln_id}"),
                fingerprint: stable_fingerprint(&[
                    PIP_AUDIT_TOOL,
                    package_name,
                    package_version,
                    vuln_id,
                ]),
                extras: map_from_pairs([
                    (
                        "aliases",
                        vuln.get("aliases")
                            .cloned()
                            .unwrap_or(Value::Array(Vec::new())),
                    ),
                    (
                        "fix_versions",
                        vuln.get("fix_versions")
                            .cloned()
                            .unwrap_or(Value::Array(Vec::new())),
                    ),
                    (
                        "description",
                        vuln.get("description").cloned().unwrap_or(Value::Null),
                    ),
                    ("package_name", Value::String(package_name.to_string())),
                    (
                        "package_version",
                        Value::String(package_version.to_string()),
                    ),
                ]),
            });
        }
    }
    findings
}

fn parse_osv_scanner_payload(payload: &Value) -> Vec<ExternalFinding> {
    let Some(results) = payload.get("results").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for result in results {
        let Some(result) = result.as_object() else {
            continue;
        };
        let Some(packages) = result.get("packages").and_then(Value::as_array) else {
            continue;
        };
        for package_entry in packages {
            let Some(package_entry) = package_entry.as_object() else {
                continue;
            };
            let Some(package) = package_entry.get("package").and_then(Value::as_object) else {
                continue;
            };
            let package_name = package
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let package_version = package
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let ecosystem = package
                .get("ecosystem")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let Some(vulnerabilities) = package_entry
                .get("vulnerabilities")
                .and_then(Value::as_array)
            else {
                continue;
            };
            for vuln in vulnerabilities {
                let Some(vuln) = vuln.as_object() else {
                    continue;
                };
                let vuln_id = vuln.get("id").and_then(Value::as_str).unwrap_or("osv");
                findings.push(ExternalFinding {
                    tool: String::from(OSV_SCANNER_TOOL),
                    domain: String::from("security"),
                    category: String::from("sca"),
                    rule_id: vuln_id.to_string(),
                    severity: ExternalSeverity::High,
                    confidence: ExternalConfidence::High,
                    file_path: None,
                    line: None,
                    locations: Vec::new(),
                    message: format!("{package_name} {package_version} is affected by {vuln_id}"),
                    fingerprint: stable_fingerprint(&[
                        OSV_SCANNER_TOOL,
                        package_name,
                        package_version,
                        vuln_id,
                    ]),
                    extras: map_from_pairs([
                        ("ecosystem", Value::String(ecosystem.to_string())),
                        (
                            "summary",
                            vuln.get("summary").cloned().unwrap_or(Value::Null),
                        ),
                        (
                            "details",
                            vuln.get("details").cloned().unwrap_or(Value::Null),
                        ),
                        (
                            "aliases",
                            vuln.get("aliases")
                                .cloned()
                                .unwrap_or(Value::Array(Vec::new())),
                        ),
                        ("package_name", Value::String(package_name.to_string())),
                        (
                            "package_version",
                            Value::String(package_version.to_string()),
                        ),
                    ]),
                });
            }
        }
    }
    findings
}

fn json_collection(value: &Value) -> impl Iterator<Item = &Value> {
    value.as_array().into_iter().flatten().chain(
        value
            .as_object()
            .into_iter()
            .flat_map(|object| object.values()),
    )
}

fn parse_composer_audit_payload(payload: &Value) -> Vec<ExternalFinding> {
    let mut findings = Vec::new();
    if let Some(advisories) = payload.get("advisories").and_then(Value::as_object) {
        for (package_name, advisories) in advisories {
            for advisory in json_collection(advisories) {
                let Some(advisory) = advisory.as_object() else {
                    continue;
                };
                let advisory_id = advisory
                    .get("advisoryId")
                    .and_then(Value::as_str)
                    .or_else(|| advisory.get("cve").and_then(Value::as_str))
                    .unwrap_or(package_name);
                let title = advisory
                    .get("title")
                    .and_then(Value::as_str)
                    .or_else(|| advisory.get("link").and_then(Value::as_str))
                    .unwrap_or(package_name);
                findings.push(ExternalFinding {
                    tool: String::from(COMPOSER_AUDIT_TOOL),
                    domain: String::from("security"),
                    category: String::from("sca"),
                    rule_id: advisory_id.to_string(),
                    severity: severity_from_label(
                        advisory
                            .get("severity")
                            .and_then(Value::as_str)
                            .unwrap_or("high"),
                    ),
                    confidence: ExternalConfidence::High,
                    file_path: Some(PathBuf::from("composer.lock")),
                    line: Some(1),
                    locations: Vec::new(),
                    message: format!("{package_name}: {title}"),
                    fingerprint: stable_fingerprint(&[
                        COMPOSER_AUDIT_TOOL,
                        package_name,
                        advisory_id,
                    ]),
                    extras: map_from_pairs([
                        ("cve", advisory.get("cve").cloned().unwrap_or(Value::Null)),
                        ("link", advisory.get("link").cloned().unwrap_or(Value::Null)),
                        (
                            "affected_versions",
                            advisory
                                .get("affectedVersions")
                                .cloned()
                                .unwrap_or(Value::Null),
                        ),
                    ]),
                });
            }
        }
    }

    if let Some(abandoned) = payload.get("abandoned").and_then(Value::as_object) {
        for (package_name, replacement) in abandoned {
            findings.push(ExternalFinding {
                tool: String::from(COMPOSER_AUDIT_TOOL),
                domain: String::from("security"),
                category: String::from("abandoned_dependency"),
                rule_id: String::from("abandoned-package"),
                severity: ExternalSeverity::Medium,
                confidence: ExternalConfidence::High,
                file_path: Some(PathBuf::from("composer.lock")),
                line: Some(1),
                locations: Vec::new(),
                message: format!("{package_name}: package is abandoned"),
                fingerprint: stable_fingerprint(&[
                    COMPOSER_AUDIT_TOOL,
                    package_name,
                    replacement.as_str().unwrap_or_default(),
                ]),
                extras: map_from_pairs([("replacement", replacement.clone())]),
            });
        }
    }

    if let Some(packages) = payload.get("filter").and_then(Value::as_object) {
        for (package_name, entries) in packages {
            for entry in json_collection(entries) {
                let Some(list) = entry.get("listName").and_then(Value::as_str) else {
                    continue;
                };
                let id = entry.get("id").and_then(Value::as_str).unwrap_or(list);
                let reason = entry
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("matched dependency policy");
                let constraint = entry
                    .get("constraint")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                findings.push(ExternalFinding {
                    tool: COMPOSER_AUDIT_TOOL.into(),
                    domain: "security".into(),
                    category: "dependency_policy".into(),
                    rule_id: id.into(),
                    severity: if list == "malware" {
                        ExternalSeverity::High
                    } else {
                        ExternalSeverity::Medium
                    },
                    confidence: ExternalConfidence::High,
                    file_path: Some(PathBuf::from("composer.lock")),
                    line: Some(1),
                    locations: Vec::new(),
                    message: format!("{package_name}: {list}: {reason}"),
                    fingerprint: stable_fingerprint(&[
                        COMPOSER_AUDIT_TOOL,
                        package_name,
                        list,
                        id,
                        constraint,
                    ]),
                    extras: entry.as_object().cloned().unwrap_or_default(),
                });
            }
        }
    }

    findings
}

fn parse_npm_audit_payload(payload: &Value) -> Vec<ExternalFinding> {
    let Some(vulnerabilities) = payload.get("vulnerabilities").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for (package_name, vulnerability) in vulnerabilities {
        let Some(vulnerability) = vulnerability.as_object() else {
            continue;
        };
        let severity = vulnerability
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or("medium");
        let mut rule_id = package_name.clone();
        let mut title = package_name.clone();
        if let Some(via) = vulnerability.get("via").and_then(Value::as_array) {
            for entry in via {
                let Some(entry) = entry.as_object() else {
                    continue;
                };
                rule_id = entry
                    .get("source")
                    .and_then(Value::as_i64)
                    .map(|value| value.to_string())
                    .or_else(|| entry.get("name").and_then(Value::as_str).map(String::from))
                    .unwrap_or_else(|| package_name.clone());
                title = entry
                    .get("title")
                    .and_then(Value::as_str)
                    .or_else(|| entry.get("url").and_then(Value::as_str))
                    .map(String::from)
                    .unwrap_or_else(|| package_name.clone());
                break;
            }
        }
        findings.push(ExternalFinding {
            tool: String::from(NPM_AUDIT_TOOL),
            domain: String::from("security"),
            category: String::from("sca"),
            rule_id,
            severity: severity_from_label(severity),
            confidence: ExternalConfidence::High,
            file_path: Some(PathBuf::from("package-lock.json")),
            line: Some(1),
            locations: Vec::new(),
            message: format!("{package_name}: {title}"),
            fingerprint: stable_fingerprint(&[NPM_AUDIT_TOOL, package_name, severity]),
            extras: map_from_pairs([
                (
                    "is_direct",
                    vulnerability
                        .get("isDirect")
                        .cloned()
                        .unwrap_or(Value::Null),
                ),
                (
                    "fix_available",
                    vulnerability
                        .get("fixAvailable")
                        .cloned()
                        .unwrap_or(Value::Null),
                ),
                (
                    "nodes",
                    vulnerability.get("nodes").cloned().unwrap_or(Value::Null),
                ),
            ]),
        });
    }
    findings
}

fn parse_cargo_deny_output(payload: &str) -> Vec<ExternalFinding> {
    if payload.trim().is_empty() {
        return Vec::new();
    }

    let mut findings = Vec::new();
    for line in payload.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(entry) = entry.as_object() else {
            continue;
        };
        if entry.get("type").and_then(Value::as_str) != Some("diagnostic") {
            continue;
        }
        let Some(fields) = entry.get("fields").and_then(Value::as_object) else {
            continue;
        };
        let advisory = fields.get("advisory").and_then(Value::as_object);
        let labels = fields.get("labels").and_then(Value::as_array);
        let primary_label = labels
            .and_then(|labels| labels.first())
            .and_then(Value::as_object);
        let line_no = primary_label
            .and_then(|label| label.get("line"))
            .and_then(Value::as_u64)
            .map(|line| line as usize)
            .or(Some(1));
        let file_path = primary_label
            .and_then(|label| label.get("file"))
            .and_then(Value::as_str)
            .map(PathBuf::from);
        let code = fields
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let message = fields
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("cargo-deny emitted a diagnostic")
            .trim()
            .to_string();
        let advisory_id = advisory
            .and_then(|advisory| advisory.get("id"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let (category, domain) = cargo_deny_category(code, advisory.is_some());
        findings.push(ExternalFinding {
            tool: String::from(CARGO_DENY_TOOL),
            domain: domain.to_string(),
            category: category.to_string(),
            rule_id: if advisory_id.is_empty() {
                if code.is_empty() {
                    category.to_string()
                } else {
                    code.to_string()
                }
            } else {
                advisory_id.to_string()
            },
            severity: severity_from_label(
                fields
                    .get("severity")
                    .and_then(Value::as_str)
                    .unwrap_or("warning"),
            ),
            confidence: ExternalConfidence::High,
            file_path,
            line: line_no,
            locations: Vec::new(),
            message,
            fingerprint: stable_fingerprint(&[
                CARGO_DENY_TOOL,
                code,
                advisory_id,
                &line_no.unwrap_or(1).to_string(),
            ]),
            extras: map_from_pairs([
                (
                    "labels",
                    fields
                        .get("labels")
                        .cloned()
                        .unwrap_or(Value::Array(Vec::new())),
                ),
                (
                    "notes",
                    fields
                        .get("notes")
                        .cloned()
                        .unwrap_or(Value::Array(Vec::new())),
                ),
                (
                    "advisory",
                    advisory
                        .map(|advisory| Value::Object(advisory.clone()))
                        .unwrap_or(Value::Object(Map::new())),
                ),
            ]),
        });
    }
    findings
}

fn parse_cargo_clippy_output(project_path: &Path, payload: &str) -> Vec<ExternalFinding> {
    if payload.trim().is_empty() {
        return Vec::new();
    }

    let mut findings = Vec::new();
    for line in payload.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(entry) = entry.as_object() else {
            continue;
        };
        if entry.get("reason").and_then(Value::as_str) != Some("compiler-message") {
            continue;
        }
        let Some(message) = entry.get("message").and_then(Value::as_object) else {
            continue;
        };
        let Some(code) = message.get("code").and_then(Value::as_object) else {
            continue;
        };
        let rule_id = code.get("code").and_then(Value::as_str).unwrap_or_default();
        if !rule_id.starts_with("clippy::") {
            continue;
        }
        let primary_span = message
            .get("spans")
            .and_then(Value::as_array)
            .and_then(|spans| {
                spans.iter().find_map(|span| {
                    let span = span.as_object()?;
                    if span
                        .get("is_primary")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        Some(span)
                    } else {
                        None
                    }
                })
            });
        let file_path = primary_span
            .and_then(|span| span.get("file_name"))
            .and_then(Value::as_str)
            .map(|path| project_relative_path(project_path, path));
        let line_no = primary_span
            .and_then(|span| span.get("line_start"))
            .and_then(Value::as_u64)
            .map(|value| value as usize);
        let rendered = message.get("rendered").cloned().unwrap_or(Value::Null);
        let message_text = message
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        findings.push(ExternalFinding {
            tool: String::from(CARGO_CLIPPY_TOOL),
            domain: String::from("quality"),
            category: String::from("lint"),
            rule_id: rule_id.to_string(),
            severity: severity_from_label(
                message
                    .get("level")
                    .and_then(Value::as_str)
                    .unwrap_or("warning"),
            ),
            confidence: ExternalConfidence::High,
            file_path,
            line: line_no,
            locations: Vec::new(),
            message: message_text.clone(),
            fingerprint: stable_fingerprint(&[
                CARGO_CLIPPY_TOOL,
                rule_id,
                &line_no.unwrap_or(1).to_string(),
                &message_text,
            ]),
            extras: map_from_pairs([("rendered", rendered)]),
        });
    }
    findings
}

fn project_relative_path(project_path: &Path, raw_path: &str) -> PathBuf {
    let candidate = PathBuf::from(raw_path);
    if candidate.is_absolute() {
        candidate
            .strip_prefix(project_path)
            .map(PathBuf::from)
            .unwrap_or(candidate)
    } else {
        candidate
    }
}

fn stable_fingerprint(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|part| part.replace('|', "%7C"))
        .collect::<Vec<_>>()
        .join("|")
}

fn map_from_pairs<const N: usize>(pairs: [(&str, Value); N]) -> Map<String, Value> {
    pairs
        .into_iter()
        .map(|(key, value)| (String::from(key), value))
        .collect()
}

fn ruff_severity(code: &str) -> ExternalSeverity {
    match code {
        "S106" | "S107" => ExternalSeverity::High,
        _ => ExternalSeverity::Medium,
    }
}

fn severity_from_label(label: &str) -> ExternalSeverity {
    match label.to_ascii_lowercase().as_str() {
        "error" | "critical" | "high" => ExternalSeverity::High,
        "warning" | "medium" | "moderate" => ExternalSeverity::Medium,
        _ => ExternalSeverity::Low,
    }
}

fn cargo_deny_category(code: &str, has_advisory: bool) -> (&'static str, &'static str) {
    if has_advisory {
        return ("sca", "security");
    }
    match code {
        "accepted"
        | "rejected"
        | "unlicensed"
        | "skipped-private-workspace-crate"
        | "license-not-encountered"
        | "license-exception-not-encountered"
        | "missing-clarification-file"
        | "parse-error"
        | "empty-license-field"
        | "no-license-field"
        | "gather-failure" => ("license", "security"),
        "git-source-underspecified"
        | "allowed-source"
        | "allowed-by-organization"
        | "source-not-allowed"
        | "unmatched-source"
        | "unmatched-organization" => ("source_policy", "security"),
        _ => ("supply_chain_policy", "quality"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cargo_deny_category, normalize_selected_tools, parse_cargo_deny_output,
        parse_gitleaks_payload, parse_npm_audit_payload, parse_osv_scanner_payload,
        parse_sarif_output, ExternalConfidence, ExternalSeverity, SarifFallback,
    };
    use serde_json::json;
    use std::path::Path;

    #[test]
    fn normalizes_selected_tools_with_all_and_dedupes() {
        let selected = normalize_selected_tools(&[
            String::from("gitleaks"),
            String::from("all"),
            String::from("gitleaks,npm-audit"),
        ]);

        assert!(selected.iter().any(|tool| tool == "gitleaks"));
        assert!(selected.iter().any(|tool| tool == "npm-audit"));
        assert_eq!(
            selected
                .iter()
                .filter(|tool| tool.as_str() == "gitleaks")
                .count(),
            1
        );
    }

    #[test]
    fn parses_gitleaks_payload_into_security_findings() {
        let payload = json!([
            {
                "RuleID": "generic-api-key",
                "Description": "Hardcoded API key",
                "File": "/repo/src/main.rs",
                "StartLine": 7
            }
        ]);

        let findings = parse_gitleaks_payload(Path::new("/repo"), &payload);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].tool, "gitleaks");
        assert_eq!(findings[0].category, "secrets");
        assert_eq!(
            findings[0].file_path.as_deref(),
            Some(Path::new("src/main.rs"))
        );
    }

    #[test]
    fn parses_osv_scanner_payload() {
        let payload = json!({
            "results": [
                {
                    "packages": [
                        {
                            "package": { "name": "serde", "version": "1.0.0", "ecosystem": "crates.io" },
                            "vulnerabilities": [
                                { "id": "RUSTSEC-2025-0001", "summary": "bad", "details": "worse", "aliases": ["CVE-2025-1"] }
                            ]
                        }
                    ]
                }
            ]
        });

        let findings = parse_osv_scanner_payload(&payload);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "RUSTSEC-2025-0001");
        assert_eq!(findings[0].severity, ExternalSeverity::High);
    }

    #[test]
    fn parses_sarif_output_for_opengrep_sast_findings() {
        let payload = r#"{
          "version": "2.1.0",
          "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
          "runs": [
            {
              "tool": {
                "driver": {
                  "name": "OpenGrep",
                  "rules": [
                    {
                      "id": "python.lang.security.audit.dangerous-subprocess-use",
                      "name": "dangerous-subprocess-use",
                      "shortDescription": { "text": "Dangerous subprocess usage" },
                      "properties": { "tags": ["security", "command-injection"] }
                    }
                  ]
                }
              },
              "results": [
                {
                  "ruleId": "python.lang.security.audit.dangerous-subprocess-use",
                  "level": "error",
                  "message": { "text": "User input reaches subprocess" },
                  "locations": [
                    {
                      "physicalLocation": {
                        "artifactLocation": { "uri": "app/views.py" },
                        "region": { "startLine": 18 }
                      }
                    },
                    {
                      "physicalLocation": {
                        "artifactLocation": { "uri": "app/helpers.py" },
                        "region": { "startLine": 9 }
                      }
                    }
                  ],
                  "relatedLocations": [
                    {
                      "physicalLocation": {
                        "artifactLocation": { "uri": "app/urls.py" },
                        "region": { "startLine": 3 }
                      }
                    }
                  ],
                  "codeFlows": [
                    {
                      "threadFlows": [
                        {
                          "locations": [
                            {
                              "location": {
                                "physicalLocation": {
                                  "artifactLocation": { "uri": "app/urls.py" },
                                  "region": { "startLine": 3 }
                                }
                              }
                            },
                            {
                              "location": {
                                "physicalLocation": {
                                  "artifactLocation": { "uri": "app/views.py" },
                                  "region": { "startLine": 18 }
                                }
                              }
                            }
                          ]
                        }
                      ]
                    }
                  ],
                  "stacks": [
                    {
                      "frames": [
                        {
                          "location": {
                            "physicalLocation": {
                              "artifactLocation": { "uri": "app/urls.py" },
                              "region": { "startLine": 3 }
                            }
                          }
                        }
                      ]
                    }
                  ],
                  "fingerprints": { "primary": "sarif-fp-1" }
                }
              ]
            }
          ]
        }"#;

        let findings = parse_sarif_output(
            "opengrep",
            Path::new("/repo"),
            payload,
            SarifFallback {
                domain: "security",
                category: "sast",
            },
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].tool, "OpenGrep");
        assert_eq!(findings[0].category, "sast");
        assert_eq!(findings[0].severity, ExternalSeverity::High);
        assert_eq!(findings[0].confidence, ExternalConfidence::Medium);
        assert_eq!(
            findings[0].file_path.as_deref(),
            Some(Path::new("app/views.py"))
        );
        assert_eq!(findings[0].line, Some(18));
        assert_eq!(findings[0].fingerprint, "sarif-fp-1");
        assert_eq!(findings[0].locations.len(), 3);
        assert_eq!(findings[0].locations[0].label, "primary");
        assert_eq!(
            findings[0].locations[0].file_path,
            Path::new("app/views.py")
        );
        assert_eq!(findings[0].locations[1].label, "supporting");
        assert_eq!(
            findings[0].locations[1].file_path,
            Path::new("app/helpers.py")
        );
        assert_eq!(findings[0].locations[2].label, "supporting");
        assert_eq!(findings[0].locations[2].file_path, Path::new("app/urls.py"));
        assert_eq!(
            findings[0].extras.get("related_location_count"),
            Some(&json!(1))
        );
        assert_eq!(findings[0].extras.get("code_flow_count"), Some(&json!(1)));
        assert_eq!(findings[0].extras.get("thread_flow_count"), Some(&json!(1)));
        assert_eq!(
            findings[0].extras.get("thread_flow_location_count"),
            Some(&json!(2))
        );
        assert_eq!(findings[0].extras.get("stack_count"), Some(&json!(1)));
        assert_eq!(findings[0].extras.get("stack_frame_count"), Some(&json!(1)));
    }

    #[test]
    fn parses_sarif_output_and_infers_trivy_categories() {
        let payload = r#"{
          "version": "2.1.0",
          "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
          "runs": [
            {
              "tool": {
                "driver": {
                  "name": "Trivy",
                  "rules": [
                    {
                      "id": "aws-access-key",
                      "name": "aws-access-key",
                      "shortDescription": { "text": "Hardcoded AWS access key" },
                      "properties": { "tags": ["secret", "credential"] }
                    }
                  ]
                }
              },
              "results": [
                {
                  "ruleId": "aws-access-key",
                  "level": "warning",
                  "message": { "text": "AWS access key detected" },
                  "locations": [
                    {
                      "physicalLocation": {
                        "artifactLocation": { "uri": "config/.env" },
                        "region": { "startLine": 4 }
                      }
                    }
                  ]
                }
              ]
            }
          ]
        }"#;

        let findings = parse_sarif_output(
            "trivy",
            Path::new("/repo"),
            payload,
            SarifFallback {
                domain: "security",
                category: "sca",
            },
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].tool, "Trivy");
        assert_eq!(findings[0].category, "secrets");
        assert_eq!(findings[0].confidence, ExternalConfidence::High);
        assert_eq!(
            findings[0].file_path.as_deref(),
            Some(Path::new("config/.env"))
        );
    }

    #[test]
    fn parses_npm_audit_payload() {
        let payload = json!({
            "vulnerabilities": {
                "lodash": {
                    "severity": "high",
                    "isDirect": true,
                    "fixAvailable": true,
                    "nodes": ["node_modules/lodash"],
                    "via": [
                        { "source": 1096475, "title": "Prototype pollution" }
                    ]
                }
            }
        });

        let findings = parse_npm_audit_payload(&payload);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].tool, "npm-audit");
        assert_eq!(
            findings[0].file_path.as_deref(),
            Some(Path::new("package-lock.json"))
        );
    }

    #[test]
    fn parses_cargo_deny_jsonl_output() {
        let payload = r#"{"type":"diagnostic","fields":{"code":"source-not-allowed","severity":"warning","message":"crate source blocked","labels":[{"file":"Cargo.lock","line":1}]}}"#;

        let findings = parse_cargo_deny_output(payload);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "source_policy");
        assert_eq!(findings[0].domain, "security");
    }

    #[test]
    fn cargo_deny_category_distinguishes_advisories() {
        assert_eq!(
            cargo_deny_category("source-not-allowed", false),
            ("source_policy", "security")
        );
        assert_eq!(cargo_deny_category("anything", true), ("sca", "security"));
    }
}
