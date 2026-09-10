use super::McpState;
use crate::assessment::behavior::ImplementationComparison;
use crate::assessment::wiring::{ExecutionPathAssessment, WiringSymbol};
use crate::graph::{SemanticGraph, SymbolNode};
use crate::parsing::behavior::FunctionBehavior;
use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ImplementationContextParams {
    /// One to eight exact symbol IDs, qualified names or unambiguous names.
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(super) struct ImplementationContextOutput {
    pub source_snapshot_id: String,
    pub input_coverage: crate::coverage::InputCoverage,
    pub selected: Vec<WiringSymbol>,
    pub execution_paths: ExecutionPathAssessment,
    pub body_count: usize,
    pub body_previews: Vec<FunctionBehavior>,
    pub bodies_truncated: bool,
    pub comparison_count: usize,
    pub comparisons: Vec<ImplementationComparison>,
    pub comparisons_truncated: bool,
    pub reviewed_policies: Vec<crate::policy::reviewed::ReviewedPolicyDecision>,
}

pub(super) fn resolve_targets<'a>(graph: &'a SemanticGraph, targets: &[String]) -> Result<Vec<&'a SymbolNode>, McpError> {
    if targets.is_empty() || targets.len() > 8 || targets.iter().any(|target| target.trim().is_empty() || target.len() > 4096) {
        return Err(McpError::invalid_params("targets must contain one to eight nonempty symbol IDs or names (at most 4096 bytes each)", None));
    }
    let mut selected = Vec::new();
    for target in targets {
        let implementation = |symbol: &&SymbolNode| matches!(symbol.kind, crate::graph::SymbolKind::Function | crate::graph::SymbolKind::Method
            | crate::graph::SymbolKind::Class | crate::graph::SymbolKind::Interface | crate::graph::SymbolKind::Struct | crate::graph::SymbolKind::Trait | crate::graph::SymbolKind::Enum);
        let exact_id = graph.symbols.iter().filter(implementation).find(|symbol| symbol.id == *target);
        let matches = exact_id.into_iter().chain(graph.symbols.iter().filter(implementation).filter(|symbol| exact_id.is_none()
            && (symbol.name == *target || symbol.qualified_name == *target))).collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(McpError::invalid_params(format!("target {target:?} has {} matches; use an exact ID from find_symbol", matches.len()),
                Some(serde_json::json!({"candidate_ids": matches.iter().take(8).map(|symbol| &symbol.id).collect::<Vec<_>>()}))));
        }
        if !selected.iter().any(|symbol: &&SymbolNode| symbol.id == matches[0].id) { selected.push(matches[0]); }
    }
    Ok(selected)
}

pub(super) fn implementation_context(snapshot: &McpState, params: ImplementationContextParams) -> Result<ImplementationContextOutput, McpError> {
    let analysis = &snapshot.analysis;
    let graph = &analysis.semantic_graph;
    let selected = resolve_targets(graph, &params.targets)?;
    let ids = selected.iter().map(|symbol| symbol.id.clone()).collect::<Vec<_>>();
    let members = graph.symbols.iter().filter(|symbol| ids.contains(&symbol.id) || symbol.parent_symbol_id.as_ref().is_some_and(|id| ids.contains(id)))
        .map(|symbol| symbol.id.as_str()).collect::<HashSet<_>>();
    let bodies = graph.function_behaviors.iter().filter(|body| members.contains(body.symbol_id.as_str())).collect::<Vec<_>>();
    let comparisons = analysis.architectural_assessment.behavior.comparisons.iter().filter(|comparison|
        members.contains(comparison.left.symbol_id.as_str()) || members.contains(comparison.right.symbol_id.as_str())
        || comparison.related_implementations.iter().any(|implementation| members.contains(implementation.symbol_id.as_str())))
        .collect::<Vec<_>>();
    Ok(ImplementationContextOutput {
        source_snapshot_id: crate::agentic::review_snapshot_id(analysis),
        input_coverage: graph.input_coverage(), selected: selected.into_iter().map(Into::into).collect(),
        execution_paths: crate::assessment::wiring::assess(graph, &analysis.contract_inventory, &ids),
        body_count: bodies.len(), bodies_truncated: bodies.len() > 32, body_previews: bodies.into_iter().take(32).cloned().collect(),
        comparison_count: comparisons.len(), comparisons_truncated: comparisons.len() > 16, comparisons: comparisons.into_iter().take(16).cloned().collect(),
        reviewed_policies: snapshot.review_surface.reviewed_policies.iter().filter(|policy|
            policy.decision.anchor_files.iter().any(|path| graph.symbols.iter().any(|symbol| ids.contains(&symbol.id) && &symbol.file_path == path)))
            .take(16).cloned().collect(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct PrepareArchitecturalReviewParams {
    /// Exact implementation names/IDs, finding IDs/fingerprints or behavior-comparison IDs.
    pub targets: Vec<String>,
    pub concern: crate::review::decision::ArchitecturalConcern,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(super) struct PreparedArchitecturalReview {
    pub source_snapshot_id: String,
    pub selected_implementations: Vec<WiringSymbol>,
    pub context_truncated: bool,
    pub task_packets: Vec<crate::agentic::AgenticTaskPacket>,
    pub structured_output_schema: serde_json::Value,
    pub instructions: String,
    pub captured_source: String,
}

struct PreparedContext {
    review: crate::agentic::AgenticReviewArtifact,
    selected: Vec<WiringSymbol>,
    truncated: bool,
}

fn prepare(snapshot: &McpState, params: &PrepareArchitecturalReviewParams) -> Result<PreparedContext, McpError> {
    use crate::evidence::EvidenceAnchor;
    if params.targets.is_empty() || params.targets.len() > 8 || params.targets.iter().any(|target| target.is_empty() || target.len() > 4096) {
        return Err(McpError::invalid_params("select one to eight bounded review targets", None));
    }
    let analysis = &snapshot.analysis;
    analysis.verify_inputs().map_err(|error| McpError::internal_error(error.to_string(), None))?;
    let graph = &analysis.semantic_graph;
    let mut selected = BTreeMap::<String, &SymbolNode>::new();
    let mut finding_ids = BTreeSet::new();
    let mut files = BTreeSet::new();
    let mut anchors = Vec::new();
    for target in &params.targets {
        if let Some(finding) = snapshot.review_surface.findings.iter().find(|finding| finding.id == *target || finding.fingerprint == *target) {
            finding_ids.insert(finding.id.clone());
            files.extend(finding.file_paths.iter().cloned());
            anchors.extend(finding.evidence_anchors.iter().cloned());
            if let Some(anchor) = &finding.primary_anchor { anchors.push(anchor.clone()); }
            continue;
        }
        if let Some(comparison) = analysis.architectural_assessment.behavior.comparisons.iter().find(|comparison| comparison.id == *target) {
            for implementation in std::iter::once(&comparison.left).chain(std::iter::once(&comparison.right)).chain(&comparison.related_implementations) {
                if let Some(symbol) = graph.symbols.iter().find(|symbol| symbol.id == implementation.symbol_id) { selected.insert(symbol.id.clone(), symbol); }
            }
            for finding in analysis.architectural_assessment.findings.iter().filter(|finding| finding.behavior_comparison_id.as_deref() == Some(comparison.id.as_str())) {
                finding_ids.insert(crate::surface::duplicate_mechanism_finding_id(finding));
            }
            continue;
        }
        for symbol in resolve_targets(graph, &[target.clone()])? { selected.insert(symbol.id.clone(), symbol); }
    }
    if selected.len() > 8 { return Err(McpError::invalid_params("the selected comparisons contain more than eight implementations; narrow the review", None)); }
    for symbol in selected.values() {
        files.insert(symbol.file_path.display().to_string());
        anchors.push(EvidenceAnchor { file_path: symbol.file_path.clone(), line: Some(symbol.start_line), label: "selected_implementation".into() });
    }
    let mut truncated = files.len() > 32 || anchors.len() > 64;
    let primary = anchors.first().cloned().or_else(|| files.first().map(|file| EvidenceAnchor { file_path: file.into(), line: Some(1), label: "selected_finding".into() }))
        .ok_or_else(|| McpError::invalid_params("review targets have no captured source location", None))?;
    let ids = selected.keys().cloned().collect::<Vec<_>>();
    for edge in graph.resolved_edges.iter().filter(|edge| edge.kind == crate::graph::ReferenceKind::Call && ids.contains(&edge.target_symbol_id)) {
        if anchors.len() >= 64 { truncated = true; break; }
        anchors.push(EvidenceAnchor { file_path: edge.source_file_path.clone(), line: Some(edge.line), label: "captured_consumer".into() });
        files.insert(edge.source_file_path.display().to_string());
    }
    truncated |= files.len() > 32;
    let primary_file = primary.file_path.display().to_string();
    let mut target_files = vec![primary_file.clone()];
    target_files.extend(files.into_iter().filter(|file| *file != primary_file).take(31));
    anchors.retain(|anchor| target_files.iter().any(|file| PathBuf::from(file) == anchor.file_path));
    anchors.truncate(64);
    let identity = serde_json::to_vec(&(&params.concern, &ids, &finding_ids)).expect("review selection serializes");
    let packet_id = format!("architecture-review:{:032x}", xxhash_rust::xxh3::xxh3_128(&identity));
    let mut handoff = snapshot.handoff.clone();
    handoff.guardian_packets = vec![crate::artifacts::GuardianPacket {
        id: packet_id, priority: "high".into(), focus: "architectural_review".into(),
        primary_target_file: primary.file_path.display().to_string(), precision: "modeled".into(), confidence_millis: 0,
        summary: format!("Review {:?} for the explicitly selected capability; justify the smallest action and preserve its contracts", params.concern),
        target_files, primary_anchor: Some(primary), evidence_anchors: anchors, locations: Vec::new(),
        finding_ids: finding_ids.into_iter().collect(), context_labels: if truncated { vec!["source_context_truncated".into()] } else { Vec::new() },
        provenance: vec!["explicit_architectural_review_selection".into()],
        doctrine_refs: vec!["guardian.architectonic-quality".into(), "guardian.minimal-mechanism".into()], preferred_mechanism: None,
        obligations: vec![crate::artifacts::GuardianObligation {
            action: "Compare concrete inputs, errors, effects, ownership and consumers; name the surviving owner and migration or retain a justified boundary".into(),
            acceptance: "Source references and scope are valid; unresolved requirements stay explicit, and runtime acceptance is not implied by the proposal".into(),
        }],
        suppressibility: crate::artifacts::GuardianSuppressibility { allowed: false, requires_reason: true, expiry_required: false },
        investigation_questions: vec!["Which behavior and public/lifecycle contracts must survive, and what proof is still missing?".into()],
    }];
    let mut review = crate::agentic::build_agentic_review_artifact(analysis, analysis.doctrine_registry(), &handoff, &snapshot.native_guard, &snapshot.native_convergence);
    for packet in &mut review.task_packets {
        if !ids.is_empty() { packet.execution_paths = Some(crate::assessment::wiring::assess(graph, &analysis.contract_inventory, &ids)); }
    }
    Ok(PreparedContext { review, selected: selected.into_values().map(Into::into).collect(), truncated })
}

pub(super) fn prepare_review(snapshot: &McpState, params: PrepareArchitecturalReviewParams) -> Result<PreparedArchitecturalReview, McpError> {
    let context = prepare(snapshot, &params)?;
    Ok(PreparedArchitecturalReview {
        source_snapshot_id: context.review.source_snapshot_id.clone(), selected_implementations: context.selected, context_truncated: context.truncated,
        structured_output_schema: context.review.execution.structured_output.json_schema.clone(),
        captured_source: crate::agent_runtime::captured_review_context(&context.review, &snapshot.analysis),
        instructions: format!("{}\n{}", context.review.system_prompt, context.review.user_prompt), task_packets: context.review.task_packets,
    })
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SubmitArchitecturalReviewParams {
    pub selection: PrepareArchitecturalReviewParams,
    pub response: crate::agentic::AgenticStructuredReviewResponse,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(super) struct SubmittedArchitecturalReview {
    pub review_id: String,
    pub source_scope_id: String,
    pub review_json: String,
    pub review_markdown: String,
    pub takes_effect: String,
}

pub(super) fn submit_review(snapshot: &McpState, params: SubmitArchitecturalReviewParams) -> Result<SubmittedArchitecturalReview, McpError> {
    if !snapshot.allow_review_writes { return Err(McpError::invalid_params("--no-write prevents publishing an architectural review", None)); }
    if params.response.claims.len() > 64 { return Err(McpError::invalid_params("review response exceeds 64 claims", None)); }
    let bytes = serde_json::to_vec(&params.response).map_err(|error| McpError::invalid_params(error.to_string(), None))?;
    if bytes.len() > crate::artifacts::AGENT_REVIEW_MAX_BYTES { return Err(McpError::invalid_params("review response exceeds 4 MiB", None)); }
    let context = prepare(snapshot, &params.selection)?;
    if params.response.claims.iter().any(|claim| claim.decision.concern != params.selection.concern) {
        return Err(McpError::invalid_params("every claim must address the selected architectural concern", None));
    }
    crate::review::validation::validate(&params.response, &context.review, &snapshot.analysis)
        .map_err(|error| McpError::invalid_params(error, None))?;
    let record = crate::review::decision::ArchitecturalReviewRecord::new(params.response, &context.review, &snapshot.analysis);
    let paths = crate::artifacts::default_agent_run_paths(&snapshot.analysis.root, Some(&snapshot.review_output_dir));
    snapshot.analysis.verify_inputs().map_err(|error| McpError::internal_error(error.to_string(), None))?;
    crate::artifacts::write_agent_review(&paths, &record).map_err(|error| McpError::internal_error(error.to_string(), None))?;
    Ok(SubmittedArchitecturalReview { review_id: record.review_id, source_scope_id: record.source_scope_id,
        review_json: paths.review_json.display().to_string(), review_markdown: paths.review_markdown.display().to_string(),
        takes_effect: "The source-reviewed proposal is published; report and MCP summary views incorporate it on the next analysis".into() })
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AdoptArchitecturalDecisionParams {
    pub review_id: String,
    pub claim_index: usize,
    /// Explicit finding IDs to affect; an empty list records intent without suppressing diagnostics.
    pub finding_ids: Vec<String>,
    pub disposition: crate::policy::reviewed::ArchitecturalPolicyDisposition,
    pub reason: String,
    /// False prepares a reviewable policy entry; true explicitly adopts it in repository policy.
    #[serde(default)]
    pub apply: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(super) struct AdoptedArchitecturalDecision {
    pub decision: crate::policy::reviewed::ReviewedArchitecturalDecision,
    pub applied: bool,
    pub policy_path: String,
    pub requires_reindex: bool,
}

pub(super) fn adopt_decision(snapshot: &McpState, params: AdoptArchitecturalDecisionParams) -> Result<AdoptedArchitecturalDecision, McpError> {
    if params.apply && !snapshot.allow_review_writes { return Err(McpError::invalid_params("--no-write prevents policy adoption", None)); }
    if params.finding_ids.len() > 128 { return Err(McpError::invalid_params("adoption exceeds 128 finding IDs", None)); }
    let analysis = &snapshot.analysis;
    analysis.verify_inputs().map_err(|error| McpError::internal_error(error.to_string(), None))?;
    let record = crate::artifacts::load_agent_review_by_id(&analysis.root, Some(&snapshot.review_output_dir), &params.review_id)
        .map_err(|error| McpError::internal_error(error.to_string(), None))?
        .ok_or_else(|| McpError::invalid_params("unknown review identity", None))?;
    if record.review_id != params.review_id { return Err(McpError::invalid_params("review archive identity does not match the request", None)); }
    crate::review::validation::validate_record(&record, analysis).map_err(|error| McpError::invalid_params(error, None))?;
    let decision = crate::policy::reviewed::draft(analysis, &snapshot.review_surface, &record, params.claim_index, &params.finding_ids, params.disposition, params.reason)
        .map_err(|error| McpError::invalid_params(error, None))?;
    let policy_path = analysis.root.join(crate::policy::POLICY_FILE);
    if params.apply { crate::policy::reviewed::adopt(analysis, &decision).map_err(|error| McpError::internal_error(error.to_string(), None))?; }
    Ok(AdoptedArchitecturalDecision { decision, applied: params.apply, policy_path: policy_path.display().to_string(), requires_reindex: params.apply })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agentic::{AgenticStructuredClaim, AgenticStructuredEvidenceLocation, AgenticStructuredReviewResponse};
    use crate::review::decision::{ArchitecturalAction, ArchitecturalConclusion, ArchitecturalConcern, ArchitecturalDecision, ReviewedImplementation};
    use crate::policy::reviewed::{ArchitecturalPolicyDisposition, ReviewedPolicyStatus};
    use std::fs;

    const SOURCE: &str = "fn main() { let mode = std::env::var(\"APP_MODE\").unwrap_or_default(); println!(\"{}\", mode); }\n";

    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!("aigiscode-review-feedback-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), SOURCE).unwrap();
        root
    }

    fn response(state: &McpState) -> (PrepareArchitecturalReviewParams, AgenticStructuredReviewResponse, String) {
        let finding = state.review_surface.findings.iter().find(|finding| finding.family == crate::review::ReviewFindingFamily::Hardwiring && finding.file_paths.contains(&"src/main.rs".into())).unwrap();
        let selection = PrepareArchitecturalReviewParams { targets: vec!["main".into(), finding.id.clone()], concern: ArchitecturalConcern::BoundaryViolation };
        let context = prepare(state, &selection).unwrap();
        let response = AgenticStructuredReviewResponse {
            source_snapshot_id: context.review.source_snapshot_id.clone(), verdict: "retain".into(), summary: "Retain executable bootstrap ownership".into(),
            claims: vec![AgenticStructuredClaim {
                task_packet_id: context.review.task_packets[0].id.clone(), title: "Bootstrap owns process configuration".into(), severity: "info".into(), why_now: "The executable owns process configuration".into(),
                decision: ArchitecturalDecision {
                    concern: ArchitecturalConcern::BoundaryViolation, conclusion: ArchitecturalConclusion::IntentionalVariation, action: ArchitecturalAction::Keep,
                    implementations: vec![ReviewedImplementation { file_path: "src/main.rs".into(), symbol_id: None, responsibility: "Process bootstrap".into() }],
                    canonical_owner: Some(0), comparisons: Vec::new(), consumer_changes: Vec::new(), preserved_behavior: vec!["Read executable configuration".into()], missing_evidence: Vec::new(), verification_steps: Vec::new(),
                },
                evidence_locations: vec![AgenticStructuredEvidenceLocation { file_path: "src/main.rs".into(), line: Some(1), end_line: Some(1), quote: SOURCE.trim_end().into() }],
            }],
        };
        (selection, response, finding.id.clone())
    }

    #[test]
    fn native_review_preview_adoption_and_stale_reopening_preserve_raw_findings() {
        let root = fixture();
        let state = super::super::build_mcp_state(&root, None, true, false).unwrap();
        let (selection, response, finding_id) = response(&state);
        let submitted = submit_review(&state, SubmitArchitecturalReviewParams { selection, response }).unwrap();
        let draft = AdoptArchitecturalDecisionParams { review_id: submitted.review_id.clone(), claim_index: 0, finding_ids: vec![finding_id.clone()], disposition: ArchitecturalPolicyDisposition::AcceptedPattern, reason: "Executable bootstrap is the approved process configuration owner".into(), apply: false };
        let preview = adopt_decision(&state, draft.clone()).unwrap();
        assert!(!preview.applied);
        assert!(!root.join(".aigiscode/policy.json").exists());
        let tune = crate::policy::tune::suggest_policy_patch(&state.analysis, &state.review_surface);
        assert!(tune.suggestions.iter().any(|suggestion| suggestion.field == "reviewed_decisions"));
        assert!(tune.suggested_policy.get("reviewed_decisions").is_none());
        adopt_decision(&state, AdoptArchitecturalDecisionParams { apply: true, ..draft.clone() }).unwrap();
        let adopted = super::super::build_mcp_state(&root, None, true, false).unwrap();
        let finding = adopted.review_surface.findings.iter().find(|finding| finding.id == finding_id).unwrap();
        assert!(!finding.is_visible);
        assert_eq!(finding.review_status, crate::review::ReviewStatus::AcceptedSourceReview);
        assert_eq!(adopted.review_surface.reviewed_policies[0].status, ReviewedPolicyStatus::Current);
        assert!(adopted.handoff.guardian_packets.iter().all(|packet| !packet.finding_ids.contains(&finding_id)));
        assert!(adopted.native_convergence.reviewed_policy.confirmed_drift.is_empty());
        assert!(!adopted.native_convergence.findings.iter().any(|delta| delta.status == crate::artifacts::ConvergenceStatus::Improved));
        fs::write(root.join("src/main.rs"), SOURCE.replace("unwrap_or_default()", "expect(\"required\")")).unwrap();
        let changed = super::super::build_mcp_state(&root, None, true, false).unwrap();
        assert_eq!(changed.review_surface.reviewed_policies[0].status, ReviewedPolicyStatus::Stale);
        assert_eq!(changed.native_convergence.reviewed_policy.stale_decisions, 1);
        assert!(changed.review_surface.findings.iter().any(|finding| finding.id == finding_id && finding.is_visible));
        assert!(adopt_decision(&changed, draft).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn confirmed_source_concern_overrides_broad_policy_without_inventing_new_drift() {
        let root = fixture();
        fs::create_dir_all(root.join(".aigiscode")).unwrap();
        fs::write(root.join(".aigiscode/policy.json"), r#"{"hardwiring":{"skip_path_patterns":["src/**"]},"project_note":"preserve"}"#).unwrap();
        let state = super::super::build_mcp_state(&root, None, true, false).unwrap();
        let (selection, mut response, finding_id) = response(&state);
        response.claims[0].decision.conclusion = ArchitecturalConclusion::Violation;
        response.claims[0].decision.action = ArchitecturalAction::Investigate;
        response.claims[0].decision.missing_evidence = vec!["Choose an explicit fallback contract before changing behavior".into()];
        let submitted = submit_review(&state, SubmitArchitecturalReviewParams { selection, response }).unwrap();
        adopt_decision(&state, AdoptArchitecturalDecisionParams {
            review_id: submitted.review_id, claim_index: 0, finding_ids: vec![finding_id.clone()],
            disposition: ArchitecturalPolicyDisposition::SourceConfirmedConcern,
            reason: "This executable must not silently accept absent required configuration".into(), apply: true,
        }).unwrap();
        let changed = super::super::build_mcp_state(&root, None, true, false).unwrap();
        let finding = changed.review_surface.findings.iter().find(|finding| finding.id == finding_id).unwrap();
        assert!(finding.is_visible);
        assert_eq!(finding.review_status, crate::review::ReviewStatus::SourceConfirmedConcern);
        assert_eq!(changed.native_convergence.reviewed_policy.source_confirmed_concerns, 1);
        assert!(changed.native_convergence.reviewed_policy.confirmed_drift.is_empty());
        assert!(changed.handoff.guardian_packets.iter().any(|packet| packet.id.starts_with("guardian:reviewed:")));
        let policy: serde_json::Value = serde_json::from_slice(&fs::read(root.join(".aigiscode/policy.json")).unwrap()).unwrap();
        assert_eq!(policy["project_note"], "preserve");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn no_write_rejects_publication_and_review_archive_preserves_prior_records() {
        let root = fixture();
        let readonly = super::super::build_mcp_state(&root, None, false, false).unwrap();
        let (selection, response, _) = response(&readonly);
        assert!(submit_review(&readonly, SubmitArchitecturalReviewParams { selection: selection.clone(), response: response.clone() }).is_err());
        assert!(!root.join(".aigiscode/agent-review.json").exists());
        let state = super::super::build_mcp_state(&root, None, true, false).unwrap();
        let mut false_absence = response.clone();
        false_absence.claims[0].decision.conclusion = ArchitecturalConclusion::UnreachableWithinScope;
        false_absence.claims[0].decision.action = ArchitecturalAction::Delete;
        false_absence.claims[0].decision.verification_steps = vec!["Execute startup after migration".into()];
        assert!(submit_review(&state, SubmitArchitecturalReviewParams { selection: selection.clone(), response: false_absence }).is_err());
        let first = submit_review(&state, SubmitArchitecturalReviewParams { selection: selection.clone(), response: response.clone() }).unwrap();
        let mut revised = response;
        revised.summary = "The same bootstrap boundary remains necessary".into();
        let second = submit_review(&state, SubmitArchitecturalReviewParams { selection, response: revised }).unwrap();
        assert_ne!(first.review_id, second.review_id);
        assert!(crate::artifacts::load_agent_review_by_id(&root, None, &first.review_id).unwrap().is_some());
        assert!(crate::artifacts::load_agent_review_by_id(&root, None, &second.review_id).unwrap().is_some());
        fs::remove_dir_all(root).unwrap();
    }
}
