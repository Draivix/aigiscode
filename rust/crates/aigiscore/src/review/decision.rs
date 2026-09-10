//! Architectural proposals describe ownership and behavior, not just a smell label.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArchitecturalConcern {
    DeadCode,
    DryViolation,
    DualPath,
    Overengineering,
    BoundaryViolation,
    StateOwnership,
    ErrorHandling,
    SideEffectOrdering,
    Performance,
    Security,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArchitecturalConclusion {
    Violation,
    IntentionalVariation,
    RuntimeEntry,
    TestSupport,
    IncompleteMigration,
    UnreachableWithinScope,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArchitecturalAction {
    Delete,
    Consolidate,
    Migrate,
    Inline,
    Split,
    Keep,
    Investigate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewedImplementation {
    pub file_path: String,
    pub symbol_id: Option<String>,
    pub responsibility: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BehaviorOutcome {
    /// Index into the decision's compared implementations.
    pub implementation: usize,
    pub behavior: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BehaviorComparison {
    pub input_case: String,
    pub outcomes: Vec<BehaviorOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConsumerChange {
    pub file_path: String,
    pub symbol_id: Option<String>,
    pub change: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArchitecturalDecision {
    pub concern: ArchitecturalConcern,
    /// The reviewer's conclusion remains a proposal; validating source anchors
    /// does not establish semantic equivalence or execute runtime acceptance.
    pub conclusion: ArchitecturalConclusion,
    pub action: ArchitecturalAction,
    pub implementations: Vec<ReviewedImplementation>,
    /// Index into `implementations`; absent when the decision has no survivor.
    pub canonical_owner: Option<usize>,
    pub comparisons: Vec<BehaviorComparison>,
    pub consumer_changes: Vec<ConsumerChange>,
    pub preserved_behavior: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub verification_steps: Vec<String>,
}

/// Native envelope; the model supplies only the proposal, never validation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitecturalReviewRecord {
    pub schema_version: String,
    pub review_id: String,
    pub proposal: crate::agentic::AgenticStructuredReviewResponse,
    pub packet_findings: std::collections::BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_scope_id: String,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub finding_changes: std::collections::BTreeMap<String, crate::artifacts::ConvergenceStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_baseline_id: Option<String>,
}

impl ArchitecturalReviewRecord {
    pub(crate) fn new(proposal: crate::agentic::AgenticStructuredReviewResponse, review: &crate::agentic::AgenticReviewArtifact, analysis: &crate::ingestion::pipeline::ProjectAnalysis) -> Self {
        let packet_findings = review.task_packets.iter()
            .filter(|packet| proposal.claims.iter().any(|claim| claim.task_packet_id == packet.id))
            .map(|packet| (packet.id.clone(), packet.finding_ids.clone())).collect::<std::collections::BTreeMap<_, _>>();
        let mut record = Self {
            schema_version: "2026-09-10".into(),
            review_id: String::new(),
            source_scope_id: super::scope::response_scope_id(&proposal, analysis),
            finding_changes: review.finding_changes.iter().filter(|(id, _)| packet_findings.values().any(|ids| ids.contains(id)))
                .map(|(id, status)| (id.clone(), *status)).collect(),
            comparison_baseline_id: review.comparison_baseline_id.clone(),
            proposal,
            packet_findings,
        };
        record.review_id = record.content_id();
        record
    }

    pub(crate) fn content_id(&self) -> String {
        let bytes = if self.source_scope_id.is_empty() && self.finding_changes.is_empty() && self.comparison_baseline_id.is_none() {
            serde_json::to_vec(&(&self.schema_version, &self.proposal, &self.packet_findings))
        } else {
            serde_json::to_vec(&(&self.schema_version, &self.proposal, &self.packet_findings, &self.source_scope_id, &self.finding_changes, &self.comparison_baseline_id))
        }.expect("review record serializes");
        format!("{:032x}", xxhash_rust::xxh3::xxh3_128(&bytes))
    }
}

pub(crate) fn render_review(record: &ArchitecturalReviewRecord) -> String {
    use std::fmt::Write;
    let response = &record.proposal;
    let mut out = format!(
        "# Architectural review\n\nReview: `{}`\n\nSnapshot: `{}`\n\nSource references validated. Conclusions are reviewer proposals; runtime behavior has not been verified.\n\n{}\n\n{}\n",
        record.review_id, response.source_snapshot_id, response.verdict, response.summary,
    );
    for claim in &response.claims {
        let decision = &claim.decision;
        let _ = writeln!(out, "\n## {}\n\nPacket: `{}` · Severity: {}\n\n{}\n\n{:?}: {:?} → {:?}\n", claim.title, claim.task_packet_id, claim.severity, claim.why_now, decision.concern, decision.conclusion, decision.action);
        for (index, implementation) in decision.implementations.iter().enumerate() {
            let owner = if decision.canonical_owner == Some(index) { " (surviving owner)" } else { "" };
            let _ = writeln!(out, "- `{}`{}: {}", implementation.file_path, owner, implementation.responsibility);
        }
        for comparison in &decision.comparisons {
            let _ = writeln!(out, "\nInput: {}\n", comparison.input_case);
            for outcome in &comparison.outcomes {
                let _ = writeln!(out, "- `{}`: {}", decision.implementations[outcome.implementation].file_path, outcome.behavior);
            }
        }
        for consumer in &decision.consumer_changes {
            let _ = writeln!(out, "\nConsumer `{}`: {}", consumer.file_path, consumer.change);
        }
        for (label, values) in [("Preserve", &decision.preserved_behavior), ("Missing evidence", &decision.missing_evidence), ("Verification required", &decision.verification_steps)] {
            if !values.is_empty() {
                let _ = writeln!(out, "\n{label}:\n");
                for value in values { let _ = writeln!(out, "- {value}"); }
            }
        }
        for location in &claim.evidence_locations {
            let _ = writeln!(out, "\nSource `{}`:{}–{}:\n", location.file_path, location.line.unwrap_or(0), location.end_line.or(location.line).unwrap_or(0));
            for line in location.quote.lines() { let _ = writeln!(out, "    {line}"); }
        }
    }
    out
}
