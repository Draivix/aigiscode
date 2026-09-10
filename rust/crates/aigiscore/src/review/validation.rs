//! Validate review references against captured facts, without certifying reasoning.

use crate::agentic::{review_snapshot_id, AgenticReviewArtifact, AgenticStructuredReviewResponse};
use crate::ingestion::pipeline::ProjectAnalysis;
use super::decision::{ArchitecturalAction as Action, ArchitecturalConclusion as Conclusion};
use std::collections::HashSet;
use std::path::{Component, Path};

pub(crate) fn validate(
    response: &AgenticStructuredReviewResponse,
    review: &AgenticReviewArtifact,
    analysis: &ProjectAnalysis,
) -> Result<(), String> {
    if response.source_snapshot_id != review.source_snapshot_id
        || review.source_snapshot_id != review_snapshot_id(analysis)
    {
        return Err("review source snapshot does not match the analyzed inputs".into());
    }
    validate_absence_claims(response, analysis, |packet| review.task_packets.iter()
        .find(|candidate| candidate.id == packet).map(|candidate| candidate.finding_ids.as_slice()).unwrap_or(&[]))?;
    validate_proposal(response, analysis,
        &review.task_packets.iter().map(|packet| packet.id.as_str()).collect(),
        &review.execution.structured_output.must_cover_task_packets.iter().map(String::as_str).collect())
}

pub(crate) fn validate_record(record: &super::decision::ArchitecturalReviewRecord, analysis: &ProjectAnalysis) -> Result<(), String> {
    if record.schema_version != "2026-09-10" || record.review_id != record.content_id() {
        return Err("unsupported or modified review record".into());
    }
    if super::scope::record_is_stale(record, analysis) {
        return Err("review source scope does not match the analyzed inputs".into());
    }
    if record.finding_changes.keys().any(|id| !record.packet_findings.values().any(|ids| ids.contains(id))) {
        return Err("review change evidence names an unrelated finding".into());
    }
    if record.comparison_baseline_id.is_none() && record.finding_changes.values().any(|status|
        matches!(status, crate::artifacts::ConvergenceStatus::New | crate::artifacts::ConvergenceStatus::Worsened)) {
        return Err("reviewed drift lacks its eligible baseline identity".into());
    }
    let packets = record.packet_findings.keys().map(String::as_str).collect();
    validate_absence_claims(&record.proposal, analysis, |packet| record.packet_findings.get(packet).map(Vec::as_slice).unwrap_or(&[]))?;
    validate_proposal(&record.proposal, analysis, &packets, &packets)
}

fn validate_absence_claims<'a>(response: &AgenticStructuredReviewResponse, analysis: &ProjectAnalysis, finding_ids: impl Fn(&str) -> &'a [String]) -> Result<(), String> {
    use crate::detectors::dead_code::DeadCodeProofScope;
    for claim in &response.claims {
        if claim.decision.conclusion != Conclusion::UnreachableWithinScope { continue; }
        let ids = finding_ids(&claim.task_packet_id);
        for implementation in &claim.decision.implementations {
            let proven = analysis.dead_code.findings.iter().any(|finding| {
                ids.contains(&crate::surface::dead_code_finding_id(finding))
                    && finding.file_path == Path::new(&implementation.file_path)
                    && implementation.symbol_id.as_deref().is_none_or(|id| id == finding.symbol_id)
                    && matches!(finding.proof.scope, DeadCodeProofScope::LocalBinding | DeadCodeProofScope::ClassPrivateDispatch)
                    && finding.proof.missing_evidence.is_empty()
                    && claim.evidence_locations.iter().any(|location| location.file_path == implementation.file_path
                        && location.line.is_some_and(|start| start <= finding.line)
                        && location.end_line.or(location.line).is_some_and(|end| end >= finding.line))
            });
            if !proven { return Err(format!("unreachability requires a selected native finding with complete local or private-dispatch proof and a citation at its declaration: {}", implementation.file_path)); }
        }
    }
    Ok(())
}

fn validate_proposal(
    response: &AgenticStructuredReviewResponse,
    analysis: &ProjectAnalysis,
    allowed_packets: &HashSet<&str>,
    required_packets: &HashSet<&str>,
) -> Result<(), String> {
    let mut covered = HashSet::new();
    for claim in &response.claims {
        if !allowed_packets.contains(claim.task_packet_id.as_str()) {
            return Err(format!("unknown task packet: {}", claim.task_packet_id));
        }
        covered.insert(claim.task_packet_id.as_str());
        let decision = &claim.decision;
        if decision.conclusion == Conclusion::Unknown {
            if decision.action != Action::Investigate || !has_text(&decision.missing_evidence) {
                return Err("unknown conclusions require investigation and missing evidence".into());
            }
        } else if decision.implementations.is_empty() || claim.evidence_locations.is_empty() {
            return Err("architectural conclusions require implementations and source evidence".into());
        }
        if matches!(decision.conclusion, Conclusion::IntentionalVariation | Conclusion::RuntimeEntry)
            && decision.action != Action::Keep {
            return Err("intentional variation and runtime entries must retain their justified implementation".into());
        }
        if let Some(owner) = decision.canonical_owner {
            if owner >= decision.implementations.len() {
                return Err("canonical owner is outside the compared implementations".into());
            }
        }
        if matches!(decision.action, Action::Consolidate | Action::Migrate)
            && (decision.implementations.len() < 2 || decision.canonical_owner.is_none()
                || decision.comparisons.is_empty() || decision.consumer_changes.is_empty())
        {
            return Err("consolidation and migration require compared paths, an owner and consumer changes".into());
        }
        if !matches!(decision.action, Action::Keep | Action::Investigate)
            && (!has_text(&decision.preserved_behavior) || !has_text(&decision.verification_steps))
        {
            return Err("source changes require preserved behavior and verification steps".into());
        }
        for implementation in &decision.implementations {
            validate_reference(analysis, &implementation.file_path, implementation.symbol_id.as_deref())?;
            if implementation.responsibility.trim().is_empty() {
                return Err("implementation responsibility must be explicit".into());
            }
            if decision.conclusion != Conclusion::Unknown
                && !claim.evidence_locations.iter().any(|location| location.file_path == implementation.file_path)
            {
                return Err(format!("implementation lacks source evidence: {}", implementation.file_path));
            }
        }
        for consumer in &decision.consumer_changes {
            validate_reference(analysis, &consumer.file_path, consumer.symbol_id.as_deref())?;
            if consumer.change.trim().is_empty() {
                return Err("consumer migration must describe its change".into());
            }
        }
        for comparison in &decision.comparisons {
            let mut compared = HashSet::new();
            if comparison.input_case.trim().is_empty() {
                return Err("behavior comparison requires an input case".into());
            }
            for outcome in &comparison.outcomes {
                if outcome.implementation >= decision.implementations.len()
                    || !compared.insert(outcome.implementation) || outcome.behavior.trim().is_empty()
                {
                    return Err("invalid or duplicate behavior comparison implementation".into());
                }
            }
            if compared.len() != decision.implementations.len() || compared.is_empty() {
                return Err("compare every implementation under the same input case".into());
            }
        }
        for location in &claim.evidence_locations {
            let source = captured_source(analysis, &location.file_path)?;
            let start = location.line.ok_or("source evidence requires a start line")?;
            let end = location.end_line.unwrap_or(start);
            let lines: Vec<_> = source.lines().collect();
            if start == 0 || end < start || end > lines.len() || location.quote.trim().is_empty() {
                return Err(format!("invalid source span: {}", location.file_path));
            }
            if !lines[start - 1..end].join("\n").contains(&location.quote) {
                return Err(format!("source quote does not match: {}:{start}", location.file_path));
            }
        }
    }
    for packet in required_packets {
        if !covered.contains(packet) {
            return Err(format!("required task packet was not reviewed: {packet}"));
        }
    }
    analysis.verify_inputs().map_err(|error| error.to_string())
}

fn has_text(values: &[String]) -> bool {
    !values.is_empty() && values.iter().all(|value| !value.trim().is_empty())
}

fn captured_source<'a>(analysis: &'a ProjectAnalysis, file: &str) -> Result<&'a str, String> {
    let path = Path::new(file);
    if file.is_empty() || file.contains('\\')
        || path.components().any(|part| !matches!(part, Component::Normal(_)))
        || file.split('/').any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("source reference is not a canonical relative path: {file}"));
    }
    analysis.parsed_sources.iter().find(|(candidate, _)| candidate == path)
        .map(|(_, source)| source.as_str())
        .ok_or_else(|| format!("source reference was not captured: {file}"))
}

fn validate_reference(analysis: &ProjectAnalysis, file: &str, symbol: Option<&str>) -> Result<(), String> {
    captured_source(analysis, file)?;
    if let Some(id) = symbol {
        let mut matches = analysis.semantic_graph.symbols.iter().filter(|symbol| symbol.id == id);
        if !matches.next().is_some_and(|symbol| symbol.file_path == Path::new(file)) || matches.next().is_some() {
            return Err(format!("symbol does not uniquely belong to source: {file} ({id})"));
        }
    }
    Ok(())
}
