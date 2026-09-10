//! Explicit adoption of source-reviewed architectural conclusions. This is
//! repository policy, separate from an agent merely proposing a conclusion.

use crate::artifacts::ConvergenceStatus;
use crate::ingestion::pipeline::ProjectAnalysis;
use crate::review::decision::{ArchitecturalAction, ArchitecturalConclusion, ArchitecturalConcern};
use crate::review::scope::ReviewScopeIndex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArchitecturalPolicyDisposition { AcceptedPattern, SourceConfirmedConcern }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewedArchitecturalDecision {
    pub id: String,
    pub review_id: String,
    pub claim_index: usize,
    pub disposition: ArchitecturalPolicyDisposition,
    pub concern: ArchitecturalConcern,
    pub conclusion: ArchitecturalConclusion,
    pub action: ArchitecturalAction,
    pub reason: String,
    pub finding_fingerprints: Vec<String>,
    pub anchor_files: Vec<PathBuf>,
    pub source_scope_id: String,
    pub observed_change: ConvergenceStatus,
    pub comparison_baseline_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewedPolicyStatus { Current, Stale }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReviewedPolicyDecision {
    pub decision: ReviewedArchitecturalDecision,
    pub status: ReviewedPolicyStatus,
    pub matching_finding_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReviewedPolicySummary {
    pub accepted_patterns: usize,
    pub source_confirmed_concerns: usize,
    pub stale_decisions: usize,
    /// Unresolved source-confirmed drift against the adoption's eligible baseline,
    /// not a claim that the concern was introduced in the latest scan.
    pub confirmed_drift: Vec<String>,
}

pub(crate) fn summarize(policies: &[ReviewedPolicyDecision]) -> ReviewedPolicySummary {
    let mut summary = ReviewedPolicySummary::default();
    for policy in policies {
        if policy.status == ReviewedPolicyStatus::Stale { summary.stale_decisions += 1; continue; }
        match policy.decision.disposition {
            ArchitecturalPolicyDisposition::AcceptedPattern => summary.accepted_patterns += 1,
            ArchitecturalPolicyDisposition::SourceConfirmedConcern => {
                summary.source_confirmed_concerns += 1;
                if policy.decision.comparison_baseline_id.is_some() && matches!(policy.decision.observed_change, ConvergenceStatus::New | ConvergenceStatus::Worsened) {
                    summary.confirmed_drift.push(policy.decision.id.clone());
                }
            }
        }
    }
    summary
}

impl ReviewedArchitecturalDecision {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.reason.trim().is_empty() || self.reason.len() > 4096 || self.id.trim().is_empty() || self.id.len() > 256 || self.review_id.len() != 32
            || self.source_scope_id.len() != 32 || self.anchor_files.is_empty() || self.anchor_files.len() > 128
            || self.finding_fingerprints.len() > 128 {
            return Err("reviewed decisions require a bounded reason, review/scope identities and one to 128 anchor files".into());
        }
        if !self.review_id.bytes().chain(self.source_scope_id.bytes()).all(|byte| byte.is_ascii_hexdigit()) {
            return Err("review and source-scope identities must be hexadecimal".into());
        }
        if self.comparison_baseline_id.as_ref().is_some_and(|id| id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()))
            || self.finding_fingerprints.iter().any(|fingerprint| fingerprint.is_empty() || fingerprint.len() > 65_536) {
            return Err("reviewed decision baseline and finding identities are invalid".into());
        }
        if self.anchor_files.iter().any(|path| path.as_os_str().is_empty() || path.to_string_lossy().len() > 4096
            || path.to_string_lossy().contains(['\0', '\\']) || path.to_string_lossy().as_bytes().get(1) == Some(&b':')
            || !path.components().all(|component| matches!(component, Component::Normal(_)))) {
            return Err("review anchor files must be repository-relative paths without traversal".into());
        }
        match self.disposition {
            ArchitecturalPolicyDisposition::AcceptedPattern if self.action != ArchitecturalAction::Keep
                || !matches!(self.conclusion, ArchitecturalConclusion::IntentionalVariation | ArchitecturalConclusion::RuntimeEntry | ArchitecturalConclusion::TestSupport) => {
                    return Err("accepted patterns require an explicitly retained variation, runtime entry or support contract".into());
                }
            ArchitecturalPolicyDisposition::SourceConfirmedConcern if !matches!(self.conclusion,
                ArchitecturalConclusion::Violation | ArchitecturalConclusion::IncompleteMigration | ArchitecturalConclusion::UnreachableWithinScope) => {
                    return Err("source-confirmed concerns require a concrete source-supported conclusion".into());
                }
            _ => {}
        }
        if matches!(self.observed_change, ConvergenceStatus::New | ConvergenceStatus::Worsened) && self.comparison_baseline_id.is_none() {
            return Err("new or worsened reviewed drift requires its eligible comparison baseline".into());
        }
        Ok(())
    }
}

pub(crate) fn evaluate(analysis: &ProjectAnalysis, findings: &[crate::review::ReviewFinding], decisions: &[ReviewedArchitecturalDecision]) -> Vec<ReviewedPolicyDecision> {
    if decisions.is_empty() { return Vec::new(); }
    let scopes = ReviewScopeIndex::new(analysis);
    decisions.iter().map(|decision| ReviewedPolicyDecision {
        status: if scopes.id_for_files(&decision.anchor_files).as_deref() == Some(decision.source_scope_id.as_str()) { ReviewedPolicyStatus::Current } else { ReviewedPolicyStatus::Stale },
        matching_finding_ids: findings.iter().filter(|finding| decision.finding_fingerprints.contains(&finding.fingerprint)).map(|finding| finding.id.clone()).collect(),
        decision: decision.clone(),
    }).collect()
}

/// Preserve the rest of the policy document and refuse stale captured inputs.
pub(crate) fn adopt(analysis: &ProjectAnalysis, decision: &ReviewedArchitecturalDecision) -> io::Result<PathBuf> {
    decision.validate().map_err(io::Error::other)?;
    let directory = analysis.root.join(".aigiscode");
    match fs::symlink_metadata(&directory) {
        Ok(metadata) if !metadata.is_dir() => return Err(io::Error::other("policy directory must be a real directory")),
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    fs::create_dir_all(&directory)?;
    let lock_path = directory.join(".reviewed-policy.lock");
    reject_nonregular(&lock_path)?;
    let lock = fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&lock_path)?;
    fs4::FileExt::lock(&lock)?;
    analysis.verify_inputs().map_err(io::Error::other)?;
    if ReviewScopeIndex::new(analysis).id_for_files(&decision.anchor_files).as_deref() != Some(decision.source_scope_id.as_str()) {
        return Err(io::Error::other("reviewed source scope changed before policy adoption"));
    }
    let path = analysis.root.join(super::POLICY_FILE);
    reject_nonregular(&path)?;
    let mut document = match fs::File::open(&path) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
            if bytes.len() > 4 * 1024 * 1024 { return Err(io::Error::other("policy document exceeds 4 MiB")); }
            serde_json::from_slice::<serde_json::Value>(&bytes).map_err(io::Error::other)?
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => serde_json::json!({}),
        Err(error) => return Err(error),
    };
    let policy = document.as_object_mut().ok_or_else(|| io::Error::other("policy must be a JSON object"))?;
    let mut rows: Vec<ReviewedArchitecturalDecision> = serde_json::from_value(policy.get("reviewed_decisions").cloned().unwrap_or_else(|| serde_json::json!([]))).map_err(io::Error::other)?;
    // A new adoption supersedes only the explicitly selected findings, leaving
    // other findings covered by an older decision intact.
    rows.retain_mut(|existing| {
        if existing.id == decision.id { return false; }
        let had_findings = !existing.finding_fingerprints.is_empty();
        existing.finding_fingerprints.retain(|fingerprint| !decision.finding_fingerprints.contains(fingerprint));
        !had_findings || !existing.finding_fingerprints.is_empty()
    });
    rows.push(decision.clone());
    policy.insert("reviewed_decisions".into(), serde_json::to_value(rows).map_err(io::Error::other)?);
    let parsed: super::PolicyFile = serde_json::from_value(document.clone()).map_err(io::Error::other)?;
    validate_decisions(&parsed.reviewed_decisions).map_err(io::Error::other)?;
    let bytes = serde_json::to_vec_pretty(&document).map_err(io::Error::other)?;
    if bytes.len() > 4 * 1024 * 1024 { return Err(io::Error::other("updated policy exceeds 4 MiB")); }
    crate::artifacts::atomic::write(&path, |writer| {
        writer.write_all(&bytes)?;
        writer.write_all(b"\n")
    })?;
    Ok(path)
}

fn reject_nonregular(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_file() => Err(io::Error::other("policy and lock paths must be regular files")),
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

pub(crate) fn validate_decisions(decisions: &[ReviewedArchitecturalDecision]) -> Result<(), String> {
    if decisions.len() > 512 { return Err("reviewed_decisions exceeds 512 entries".into()); }
    let mut ids = HashSet::new();
    let mut fingerprints = HashSet::new();
    for decision in decisions {
        decision.validate()?;
        if !ids.insert(&decision.id) { return Err(format!("duplicate reviewed decision ID: {}", decision.id)); }
        for fingerprint in &decision.finding_fingerprints {
            if !fingerprints.insert(fingerprint) { return Err(format!("finding has multiple reviewed dispositions: {fingerprint}")); }
        }
    }
    Ok(())
}

pub(crate) fn draft(analysis: &ProjectAnalysis, surface: &crate::review::ReviewSurface, record: &crate::review::decision::ArchitecturalReviewRecord, claim_index: usize, finding_ids: &[String], disposition: ArchitecturalPolicyDisposition, reason: String) -> Result<ReviewedArchitecturalDecision, String> {
    let claim = record.proposal.claims.get(claim_index).ok_or_else(|| "claim index is outside the review".to_owned())?;
    let allowed = record.packet_findings.get(&claim.task_packet_id).ok_or_else(|| "claim has no native packet binding".to_owned())?;
    let mut fingerprints = BTreeSet::new();
    let mut changes = Vec::new();
    for id in finding_ids {
        if !allowed.contains(id) { return Err(format!("finding is outside the reviewed packet: {id}")); }
        let finding = surface.findings.iter().find(|finding| finding.id == *id)
            .ok_or_else(|| format!("finding is no longer present: {id}"))?;
        if !claim.evidence_locations.iter().any(|location| finding.file_paths.contains(&location.file_path)) {
            return Err(format!("claim has no source evidence for finding: {id}"));
        }
        fingerprints.insert(finding.fingerprint.clone());
        changes.push(record.finding_changes.get(id).copied().unwrap_or(ConvergenceStatus::NotCompared));
    }
    let observed_change = if changes.contains(&ConvergenceStatus::Worsened) { ConvergenceStatus::Worsened }
        else if changes.contains(&ConvergenceStatus::New) { ConvergenceStatus::New }
        else if changes.iter().any(|status| matches!(status, ConvergenceStatus::NotCompared | ConvergenceStatus::FirstObserved)) { ConvergenceStatus::NotCompared }
        else { changes.first().copied().unwrap_or(ConvergenceStatus::NotCompared) };
    let anchors = crate::review::scope::claim_files(claim);
    let scope_id = crate::review::scope::ReviewScopeIndex::new(analysis).id_for_files(&anchors)
        .ok_or_else(|| "adoption requires captured source anchors".to_owned())?;
    let decision = crate::policy::reviewed::ReviewedArchitecturalDecision {
        id: format!("{}:{}", record.review_id, claim_index), review_id: record.review_id.clone(), claim_index: claim_index,
        disposition: disposition, concern: claim.decision.concern, conclusion: claim.decision.conclusion, action: claim.decision.action,
        reason: reason, finding_fingerprints: fingerprints.into_iter().collect(), anchor_files: anchors, source_scope_id: scope_id,
        observed_change, comparison_baseline_id: record.comparison_baseline_id.clone(),
    };
    decision.validate()?;
    Ok(decision)
}
