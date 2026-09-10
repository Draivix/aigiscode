pub mod decision;
pub(crate) mod validation;
pub(crate) mod scope;

use crate::detectors::hardwiring::HardwiringFinding;
use crate::evidence::EvidenceAnchor;
use crate::ingestion::pipeline::ProjectAnalysis;
use crate::policy::{PolicyBundle, PolicyLoadError, SuppressionReason};
use crate::surface::{
    effective_orphan_files, ArchitectureSurface, SurfaceFinding, SurfaceFindingFamily,
    SurfaceFindingPhase, SurfaceFindingSeverity,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub enum ReviewStatus {
    #[default]
    Unreviewed,
    SourceReviewedProposal,
    StaleReview,
    AcceptedSourceReview,
    SourceConfirmedConcern,
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
    #[serde(default)]
    pub accepted_architectural_decisions: usize,
    #[serde(default)]
    pub source_confirmed_concerns: usize,
    #[serde(default)]
    pub stale_architectural_decisions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub id: String,
    pub fingerprint: String,
    pub family: ReviewFindingFamily,
    #[serde(default)]
    pub phase: SurfaceFindingPhase,
    pub severity: ReviewFindingSeverity,
    pub precision: String,
    pub confidence_millis: u16,
    pub title: String,
    pub summary: String,
    pub file_paths: Vec<String>,
    pub line: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_anchor: Option<EvidenceAnchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_anchors: Vec<EvidenceAnchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<EvidenceAnchor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_context: Vec<String>,
    pub provenance: Vec<String>,
    pub doctrine_refs: Vec<String>,
    pub review_status: ReviewStatus,
    pub policy_status: PolicyStatus,
    pub is_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewSurface {
    pub root: String,
    pub summary: ReviewSummary,
    pub findings: Vec<ReviewFinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architectural_review: Option<ArchitecturalReviewContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reviewed_policies: Vec<crate::policy::reviewed::ReviewedPolicyDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchitecturalReviewStatus {
    SourceAnchoredProposal,
    Stale,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitecturalReviewContext {
    pub status: ArchitecturalReviewStatus,
    pub review_id: Option<String>,
    pub source_snapshot_id: Option<String>,
    pub reason: Option<String>,
    pub claims: Vec<crate::agentic::AgenticStructuredClaim>,
}

/// Review proposals annotate deterministic findings; they never suppress them.
pub(crate) fn apply_architectural_review(
    surface: &mut ReviewSurface,
    analysis: &ProjectAnalysis,
    record: Result<Option<decision::ArchitecturalReviewRecord>, String>,
) {
    for finding in &mut surface.findings { finding.review_status = ReviewStatus::Unreviewed; }
    surface.architectural_review = match record {
        Ok(Some(record)) => {
            let stale = scope::record_is_stale(&record, analysis);
            let error = if record.schema_version != "2026-09-10" || record.review_id != record.content_id() {
                Some("unsupported or modified review record".into())
            } else if stale {
                Some("analyzed inputs changed; conclusions require another review".into())
            } else {
                validation::validate_record(&record, analysis).err()
            };
            let valid_shape = record.schema_version == "2026-09-10" && record.review_id == record.content_id();
            let status = if stale && valid_shape { ArchitecturalReviewStatus::Stale }
                else if error.is_some() { ArchitecturalReviewStatus::Invalid }
                else { ArchitecturalReviewStatus::SourceAnchoredProposal };
            if status != ArchitecturalReviewStatus::Invalid {
                for claim in &record.proposal.claims {
                    if claim.decision.conclusion == decision::ArchitecturalConclusion::Unknown { continue; }
                    if let Some(ids) = record.packet_findings.get(&claim.task_packet_id) {
                        for finding in surface.findings.iter_mut().filter(|finding| ids.contains(&finding.id)
                            && claim.evidence_locations.iter().any(|location| finding.file_paths.contains(&location.file_path))) {
                            finding.review_status = if stale { ReviewStatus::StaleReview } else { ReviewStatus::SourceReviewedProposal };
                        }
                    }
                }
            }
            Some(ArchitecturalReviewContext {
                claims: if status == ArchitecturalReviewStatus::SourceAnchoredProposal { record.proposal.claims } else { Vec::new() },
                status, review_id: Some(record.review_id), source_snapshot_id: Some(record.proposal.source_snapshot_id), reason: error,
            })
        }
        Ok(None) => None,
        Err(reason) => Some(ArchitecturalReviewContext {
            status: ArchitecturalReviewStatus::Invalid, review_id: None, source_snapshot_id: None, reason: Some(reason), claims: Vec::new(),
        }),
    };
    apply_reviewed_policy(surface);
}

fn apply_reviewed_policy(surface: &mut ReviewSurface) {
    use crate::policy::reviewed::{ArchitecturalPolicyDisposition, ReviewedPolicyStatus};
    for policy in &surface.reviewed_policies {
        for finding in surface.findings.iter_mut().filter(|finding| policy.matching_finding_ids.contains(&finding.id)) {
            if policy.status == ReviewedPolicyStatus::Stale {
                if finding.review_status == ReviewStatus::Unreviewed { finding.review_status = ReviewStatus::StaleReview; }
                continue;
            }
            match policy.decision.disposition {
                ArchitecturalPolicyDisposition::AcceptedPattern => {
                    finding.policy_status = PolicyStatus::AcceptedByPolicy;
                    finding.is_visible = false;
                    finding.review_status = ReviewStatus::AcceptedSourceReview;
                }
                ArchitecturalPolicyDisposition::SourceConfirmedConcern => {
                    finding.review_status = ReviewStatus::SourceConfirmedConcern;
                    finding.policy_status = PolicyStatus::None;
                    finding.is_visible = true;
                }
            }
            let reason = format!("Reviewed policy {}: {}", policy.decision.id, policy.decision.reason);
            if !finding.supporting_context.contains(&reason) { finding.supporting_context.push(reason); }
            if !finding.provenance.iter().any(|source| source == "reviewed_architectural_policy") { finding.provenance.push("reviewed_architectural_policy".into()); }
        }
    }
    surface.summary.visible_findings = surface.findings.iter().filter(|finding| finding.is_visible).count();
    surface.summary.accepted_by_policy = surface.findings.iter().filter(|finding| finding.policy_status == PolicyStatus::AcceptedByPolicy).count();
    surface.summary.suppressed_by_rule = surface.findings.iter().filter(|finding| finding.policy_status == PolicyStatus::ExcludedByRule).count();
    surface.summary.ai_reviewed = surface.findings.iter().filter(|finding| matches!(finding.review_status, ReviewStatus::SourceReviewedProposal | ReviewStatus::AcceptedSourceReview | ReviewStatus::SourceConfirmedConcern)).count();
    surface.summary.unreviewed_findings = surface.findings.iter().filter(|finding| finding.is_visible && matches!(finding.review_status, ReviewStatus::Unreviewed | ReviewStatus::StaleReview)).count();
    surface.summary.accepted_architectural_decisions = surface.reviewed_policies.iter().filter(|policy| policy.status == ReviewedPolicyStatus::Current && policy.decision.disposition == ArchitecturalPolicyDisposition::AcceptedPattern).count();
    surface.summary.source_confirmed_concerns = surface.reviewed_policies.iter().filter(|policy| policy.status == ReviewedPolicyStatus::Current && policy.decision.disposition == ArchitecturalPolicyDisposition::SourceConfirmedConcern).count();
    surface.summary.stale_architectural_decisions = surface.reviewed_policies.iter().filter(|policy| policy.status == ReviewedPolicyStatus::Stale).count();
}

pub fn load_review_surface(analysis: &ProjectAnalysis) -> Result<ReviewSurface, PolicyLoadError> {
    let architecture_surface = analysis.architecture_surface();
    let mut surface = build_review_surface(
        analysis,
        &architecture_surface,
        analysis.policy_bundle(),
    );
    crate::artifacts::attach_architectural_review(analysis, &mut surface, None);
    Ok(surface)
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

    let reviewed_policies = crate::policy::reviewed::evaluate(analysis, &findings, policy_bundle.reviewed_decisions());
    let mut surface = ReviewSurface {
        root: analysis.root.display().to_string(),
        summary: ReviewSummary {
            total_findings: findings.len(),
            visible_findings,
            unreviewed_findings: visible_findings,
            accepted_by_policy,
            suppressed_by_rule,
            ai_reviewed: 0,
            rules_generated: 0,
            accepted_architectural_decisions: 0,
            source_confirmed_concerns: 0,
            stale_architectural_decisions: 0,
        },
        findings,
        architectural_review: None,
        reviewed_policies,
    };
    apply_reviewed_policy(&mut surface);
    surface
}

impl ReviewFinding {
    fn from_surface_finding(finding: &SurfaceFinding, policy_status: PolicyStatus) -> Self {
        Self {
            id: finding.id.clone(),
            fingerprint: finding.fingerprint.clone(),
            family: ReviewFindingFamily::from_surface_family(finding.family),
            phase: finding.phase,
            severity: ReviewFindingSeverity::from_surface_severity(finding.severity),
            precision: finding.precision.clone(),
            confidence_millis: finding.confidence_millis,
            title: finding.title.clone(),
            summary: finding.summary.clone(),
            file_paths: finding
                .file_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            line: finding.line,
            primary_anchor: finding.primary_anchor.clone(),
            evidence_anchors: finding.evidence_anchors.clone(),
            locations: finding.locations.clone(),
            supporting_context: finding.supporting_context.clone(),
            provenance: finding.provenance.clone(),
            doctrine_refs: finding.doctrine_refs.clone(),
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

    for path in effective_orphan_files(analysis) {
        let id = format!("graph:orphan:{}", path.display());
        suppression.insert(
            id,
            PolicyStatus::from_suppression(policy_bundle.suppress_orphan_file(&path)),
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

    for finding in &analysis.security_analysis.findings {
        let id = format!("security:native:{}", finding.fingerprint);
        suppression.insert(
            id,
            PolicyStatus::from_suppression(policy_bundle.suppress_security(finding)),
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
    Security,
    External,
}

impl ReviewFindingFamily {
    fn from_surface_family(family: SurfaceFindingFamily) -> Self {
        match family {
            SurfaceFindingFamily::Graph => Self::Graph,
            SurfaceFindingFamily::DeadCode => Self::DeadCode,
            SurfaceFindingFamily::Hardwiring => Self::Hardwiring,
            SurfaceFindingFamily::Security => Self::Security,
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
        assert!(review_surface
            .findings
            .iter()
            .all(|finding| !finding.fingerprint.is_empty()));
    }

    #[test]
    fn native_security_findings_follow_policy_suppression() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::write(
            fixture.join(".aigiscode/policy.json"),
            br#"{
  "security": { "allowed_categories": ["unsafe_html_output"] }
}"#,
        )
        .unwrap();
        fs::write(fixture.join("app.js"), "target.innerHTML = html;\n").unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let review_surface = load_review_surface(&analysis).unwrap();

        let finding = review_surface
            .findings
            .iter()
            .find(|finding| finding.family == super::ReviewFindingFamily::Security)
            .expect("expected native security finding");
        assert_eq!(finding.policy_status, PolicyStatus::AcceptedByPolicy);
        assert!(!finding.is_visible);
        assert_eq!(finding.review_status, ReviewStatus::Unreviewed);
    }

    #[test]
    fn route_declared_php_attribute_controller_is_not_suppressed_as_orphan() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src/Controller")).unwrap();
        fs::write(
            fixture.join("src/Controller/TriggerFlowController.php"),
            br#"<?php
#[Route(path: '/api/_action/trigger-event/{eventName}')]
final class TriggerFlowController {
    public function __invoke(): void {}
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let review_surface = load_review_surface(&analysis).unwrap();

        assert!(review_surface
            .findings
            .iter()
            .all(|finding| finding.id != "graph:orphan:src/Controller/TriggerFlowController.php"));
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
