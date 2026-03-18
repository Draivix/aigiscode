use crate::detectors::hardwiring::HardwiringFinding;
use crate::ingestion::pipeline::ProjectAnalysis;
use crate::policy::{PolicyBundle, PolicyLoadError, SuppressionReason};
use crate::surface::{
    ArchitectureSurface, SurfaceFinding, SurfaceFindingFamily, SurfaceFindingSeverity,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    Unreviewed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyStatus {
    None,
    AcceptedByPolicy,
    ExcludedByRule,
}

impl PolicyStatus {
    fn from_suppression(reason: Option<SuppressionReason>) -> Self {
        match reason {
            None => Self::None,
            Some(SuppressionReason::Policy) => Self::AcceptedByPolicy,
            Some(SuppressionReason::Rule) => Self::ExcludedByRule,
        }
    }

    fn is_visible(self) -> bool {
        self == Self::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_findings: usize,
    pub visible_findings: usize,
    pub unreviewed_findings: usize,
    pub accepted_by_policy: usize,
    pub suppressed_by_rule: usize,
    pub ai_reviewed: usize,
    pub rules_generated: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub id: String,
    pub family: ReviewFindingFamily,
    pub severity: ReviewFindingSeverity,
    pub title: String,
    pub summary: String,
    pub file_paths: Vec<String>,
    pub line: Option<usize>,
    pub review_status: ReviewStatus,
    pub policy_status: PolicyStatus,
    pub is_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewSurface {
    pub root: String,
    pub summary: ReviewSummary,
    pub findings: Vec<ReviewFinding>,
}

pub fn load_review_surface(analysis: &ProjectAnalysis) -> Result<ReviewSurface, PolicyLoadError> {
    let policy_bundle = PolicyBundle::load(&analysis.root)?;
    let architecture_surface = analysis.architecture_surface();
    Ok(build_review_surface(
        analysis,
        &architecture_surface,
        &policy_bundle,
    ))
}

pub fn build_review_surface(
    analysis: &ProjectAnalysis,
    architecture_surface: &ArchitectureSurface,
    policy_bundle: &PolicyBundle,
) -> ReviewSurface {
    let repeated_literal_occurrences = repeated_literal_occurrences(&analysis.hardwiring.findings);
    let suppression_by_id =
        suppression_by_id(analysis, policy_bundle, &repeated_literal_occurrences);

    let findings = architecture_surface
        .highlights
        .iter()
        .map(|finding| {
            ReviewFinding::from_surface_finding(
                finding,
                suppression_by_id
                    .get(&finding.id)
                    .copied()
                    .unwrap_or(PolicyStatus::None),
            )
        })
        .collect::<Vec<_>>();

    let accepted_by_policy = findings
        .iter()
        .filter(|finding| finding.policy_status == PolicyStatus::AcceptedByPolicy)
        .count();
    let suppressed_by_rule = findings
        .iter()
        .filter(|finding| finding.policy_status == PolicyStatus::ExcludedByRule)
        .count();
    let visible_findings = findings.iter().filter(|finding| finding.is_visible).count();

    ReviewSurface {
        root: analysis.root.display().to_string(),
        summary: ReviewSummary {
            total_findings: findings.len(),
            visible_findings,
            unreviewed_findings: visible_findings,
            accepted_by_policy,
            suppressed_by_rule,
            ai_reviewed: 0,
            rules_generated: 0,
        },
        findings,
    }
}

impl ReviewFinding {
    fn from_surface_finding(finding: &SurfaceFinding, policy_status: PolicyStatus) -> Self {
        Self {
            id: finding.id.clone(),
            family: ReviewFindingFamily::from_surface_family(finding.family),
            severity: ReviewFindingSeverity::from_surface_severity(finding.severity),
            title: finding.title.clone(),
            summary: finding.summary.clone(),
            file_paths: finding
                .file_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            line: finding.line,
            review_status: ReviewStatus::Unreviewed,
            policy_status,
            is_visible: policy_status.is_visible(),
        }
    }
}

fn suppression_by_id(
    analysis: &ProjectAnalysis,
    policy_bundle: &PolicyBundle,
    repeated_literal_occurrences: &HashMap<String, usize>,
) -> HashMap<String, PolicyStatus> {
    let mut suppression = HashMap::new();

    for path in &analysis.graph_analysis.orphan_files {
        let id = format!("graph:orphan:{}", path.display());
        suppression.insert(
            id,
            PolicyStatus::from_suppression(policy_bundle.suppress_orphan_file(path)),
        );
    }

    for finding in &analysis.dead_code.findings {
        let id = format!(
            "dead-code:{}:{}:{}",
            finding.file_path.display(),
            finding.line,
            finding.name
        );
        suppression.insert(
            id,
            PolicyStatus::from_suppression(policy_bundle.suppress_dead_code(finding)),
        );
    }

    for finding in &analysis.hardwiring.findings {
        let id = format!(
            "hardwiring:{}:{}:{}",
            finding.file_path.display(),
            finding.line,
            finding.value
        );
        let occurrences = repeated_literal_occurrences
            .get(&finding.value)
            .copied()
            .unwrap_or(1);
        suppression.insert(
            id,
            PolicyStatus::from_suppression(policy_bundle.suppress_hardwiring(finding, occurrences)),
        );
    }

    for finding in &analysis.external_analysis.findings {
        let id = format!("external:{}:{}", finding.tool, finding.fingerprint);
        suppression.insert(
            id,
            PolicyStatus::from_suppression(policy_bundle.suppress_external(finding)),
        );
    }

    suppression
}

fn repeated_literal_occurrences(findings: &[HardwiringFinding]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for finding in findings {
        if finding.category == crate::detectors::hardwiring::HardwiringCategory::RepeatedLiteral {
            *counts.entry(finding.value.clone()).or_insert(0) += 1;
        }
    }
    counts
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewFindingFamily {
    Graph,
    DeadCode,
    Hardwiring,
    External,
}

impl ReviewFindingFamily {
    fn from_surface_family(family: SurfaceFindingFamily) -> Self {
        match family {
            SurfaceFindingFamily::Graph => Self::Graph,
            SurfaceFindingFamily::DeadCode => Self::DeadCode,
            SurfaceFindingFamily::Hardwiring => Self::Hardwiring,
            SurfaceFindingFamily::External => Self::External,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewFindingSeverity {
    High,
    Medium,
    Low,
}

impl ReviewFindingSeverity {
    fn from_surface_severity(severity: SurfaceFindingSeverity) -> Self {
        match severity {
            SurfaceFindingSeverity::High => Self::High,
            SurfaceFindingSeverity::Medium => Self::Medium,
            SurfaceFindingSeverity::Low => Self::Low,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{load_review_surface, PolicyStatus, ReviewStatus};
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builds_review_surface_with_policy_and_rules_overlays() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src/contracts")).unwrap();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::write(
            fixture.join(".aigiscode/policy.json"),
            br#"{
  "dead_code": { "abandoned_entry_patterns": ["src/contracts/**"] },
  "external": { "allowed_rule_ids": ["demo-external-policy"] }
}"#,
        )
        .unwrap();
        fs::write(
            fixture.join(".aigiscode/rules.json"),
            br#"[
  { "finding_type": "magic_string", "file_pattern": "src/main.rs", "symbol_name": "draft" },
  { "finding_type": "secrets", "file_pattern": "src/main.rs", "symbol_name": "demo-external-rule", "tool": "gitleaks" }
]"#,
        )
        .unwrap();
        fs::write(
            fixture.join(".aigiscode/external-analysis.json"),
            br#"{
  "tool_runs": [
    {
      "tool": "gitleaks",
      "command": ["gitleaks", "detect"],
      "status": "findings",
      "exit_code": 0,
      "artifact_path": ".aigiscode/reports/demo/raw/gitleaks.json",
      "summary": { "finding_count": 2 }
    }
  ],
  "findings": [
    {
      "tool": "gitleaks",
      "domain": "security",
      "category": "secrets",
      "rule_id": "demo-external-rule",
      "severity": "high",
      "confidence": "medium",
      "file_path": "src/main.rs",
      "line": 2,
      "message": "hardcoded token",
      "fingerprint": "demo-external-fp-1",
      "extras": {}
    },
    {
      "tool": "gitleaks",
      "domain": "security",
      "category": "secrets",
      "rule_id": "demo-external-policy",
      "severity": "high",
      "confidence": "medium",
      "file_path": "src/main.rs",
      "line": 3,
      "message": "allowed token",
      "fingerprint": "demo-external-fp-2",
      "extras": {}
    }
  ]
}"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn main() {
    if status == "draft" {
        let _ = status;
    }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/contracts/repo.rs"),
            br#"pub struct Repo;
fn load() {}
"#,
        )
        .unwrap();

        let mut analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        analysis.external_analysis = serde_json::from_str(
            &fs::read_to_string(fixture.join(".aigiscode/external-analysis.json")).unwrap(),
        )
        .unwrap();
        let review_surface = load_review_surface(&analysis).unwrap();

        assert!(review_surface.summary.total_findings >= 4);
        assert!(review_surface.summary.accepted_by_policy >= 1);
        assert!(review_surface.summary.suppressed_by_rule >= 2);
        assert_eq!(
            review_surface.summary.visible_findings,
            review_surface.summary.unreviewed_findings
        );
        assert!(review_surface.findings.iter().any(|finding| {
            finding.id == "external:gitleaks:demo-external-fp-1"
                && finding.policy_status == PolicyStatus::ExcludedByRule
                && !finding.is_visible
        }));
        assert!(review_surface.findings.iter().any(|finding| {
            finding.id == "external:gitleaks:demo-external-fp-2"
                && finding.policy_status == PolicyStatus::AcceptedByPolicy
                && !finding.is_visible
        }));
        assert!(review_surface.findings.iter().all(|finding| {
            finding.review_status == ReviewStatus::Unreviewed
                && (finding.policy_status == PolicyStatus::None) == finding.is_visible
        }));
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-review-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
