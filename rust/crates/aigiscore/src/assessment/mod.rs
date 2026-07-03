use crate::detectors::dead_code::DeadCodeResult;
use crate::detectors::hardwiring::{HardwiringCategory, HardwiringResult};
use crate::external::{ExternalAnalysisResult, ExternalSeverity};
use crate::graph::analysis::{BottleneckFile, GraphAnalysis};
use crate::graph::{RelationKind, SemanticGraph};
use crate::identity::{normalized_path, stable_fingerprint};
use crate::scanners::ast_grep::{
    run_ast_grep_scan, AstGrepComplexitySubtype, AstGrepFindingKind, AstGrepFrameworkMisuseSubtype,
    AstGrepScanResult,
};
use crate::scanners::framework_catalogs::{
    dispatch_mechanism_catalogs_for_file, MechanismMarkerKind,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArchitecturalAssessmentKind {
    WarningHeavyHotspot,
    SplitIdentityModel,
    CompatibilityScar,
    DuplicateMechanism,
    SanctionedPathBypass,
    HandRolledParsing,
    AbstractionSprawl,
    AlgorithmicComplexityHotspot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitecturalAssessmentFinding {
    pub kind: ArchitecturalAssessmentKind,
    pub file_path: PathBuf,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_file_paths: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_identifiers: Vec<String>,
    pub warning_count: usize,
    pub warning_weight: usize,
    pub bottleneck_centrality_millis: u32,
    pub warning_families: Vec<String>,
    pub severity_millis: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pressure_path: Vec<ArchitecturalPressureHop>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expensive_operation_sites: Vec<ArchitecturalComplexitySite>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expensive_operation_flow: Vec<ArchitecturalComplexityFlowStep>,
    #[serde(default)]
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitecturalPressureHop {
    pub file_path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_to_next: Option<RelationKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_symbol: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityEvidenceSource {
    Native,
    AstGrep,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitecturalComplexitySite {
    pub file_path: PathBuf,
    pub line: usize,
    pub subtype: String,
    pub token: String,
    pub source: ComplexityEvidenceSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchitecturalComplexityFlowStepKind {
    PressureHop,
    OperationSite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitecturalComplexityFlowStep {
    pub kind: ArchitecturalComplexityFlowStepKind,
    pub file_path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_to_next: Option<RelationKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_source: Option<ComplexityEvidenceSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ArchitecturalAssessment {
    pub findings: Vec<ArchitecturalAssessmentFinding>,
}

impl ArchitecturalAssessment {
    pub fn count_by_kind(&self, kind: ArchitecturalAssessmentKind) -> usize {
        self.findings
            .iter()
            .filter(|finding| finding.kind == kind)
            .count()
    }
}

pub fn build_architectural_assessment(
    graph_analysis: &GraphAnalysis,
    dead_code: &DeadCodeResult,
    hardwiring: &HardwiringResult,
    external_analysis: &ExternalAnalysisResult,
    parsed_sources: &[(PathBuf, String)],
) -> ArchitecturalAssessment {
    let ast_grep_scan = run_ast_grep_scan(parsed_sources);
    build_architectural_assessment_with_ast_grep_and_graph(
        graph_analysis,
        dead_code,
        hardwiring,
        external_analysis,
        parsed_sources,
        &ast_grep_scan,
        None,
    )
}

pub fn build_architectural_assessment_with_ast_grep(
    graph_analysis: &GraphAnalysis,
    dead_code: &DeadCodeResult,
    hardwiring: &HardwiringResult,
    external_analysis: &ExternalAnalysisResult,
    parsed_sources: &[(PathBuf, String)],
    ast_grep_scan: &AstGrepScanResult,
) -> ArchitecturalAssessment {
    build_architectural_assessment_with_ast_grep_and_graph(
        graph_analysis,
        dead_code,
        hardwiring,
        external_analysis,
        parsed_sources,
        ast_grep_scan,
        None,
    )
}

pub fn build_architectural_assessment_with_ast_grep_and_graph(
    graph_analysis: &GraphAnalysis,
    dead_code: &DeadCodeResult,
    hardwiring: &HardwiringResult,
    external_analysis: &ExternalAnalysisResult,
    parsed_sources: &[(PathBuf, String)],
    ast_grep_scan: &AstGrepScanResult,
    semantic_graph: Option<&SemanticGraph>,
) -> ArchitecturalAssessment {
    let mut findings = detect_warning_heavy_hotspots(
        &graph_analysis.bottleneck_files,
        dead_code,
        hardwiring,
        external_analysis,
    );
    let split_identity_findings = detect_split_identity_models(parsed_sources);
    let compatibility_scars = detect_compatibility_scars(
        &split_identity_findings,
        &graph_analysis.bottleneck_files,
        parsed_sources,
    );
    let duplicate_mechanisms =
        detect_duplicate_mechanisms(&graph_analysis.bottleneck_files, parsed_sources);
    let sanctioned_path_bypasses = detect_sanctioned_path_bypasses(
        &graph_analysis.bottleneck_files,
        hardwiring,
        parsed_sources,
        ast_grep_scan,
    );
    let hand_rolled_parsing =
        detect_hand_rolled_parsing(&graph_analysis.bottleneck_files, parsed_sources);
    let abstraction_sprawl =
        detect_abstraction_sprawl(&graph_analysis.bottleneck_files, parsed_sources);
    findings.extend(split_identity_findings);
    findings.extend(compatibility_scars);
    findings.extend(duplicate_mechanisms);
    findings.extend(sanctioned_path_bypasses);
    findings.extend(hand_rolled_parsing);
    findings.extend(detect_filesystem_page_resolution_stacks(
        &graph_analysis
            .bottleneck_files
            .iter()
            .map(|bottleneck| (bottleneck.file_path.clone(), bottleneck.centrality_millis))
            .collect::<HashMap<_, _>>(),
        parsed_sources,
    ));
    findings.extend(detect_manifest_backed_policy_engine_stacks(
        &graph_analysis
            .bottleneck_files
            .iter()
            .map(|bottleneck| (bottleneck.file_path.clone(), bottleneck.centrality_millis))
            .collect::<HashMap<_, _>>(),
        parsed_sources,
    ));
    findings.extend(abstraction_sprawl);
    findings.extend(detect_algorithmic_complexity_hotspots(
        parsed_sources,
        ast_grep_scan,
    ));
    attach_complexity_graph_pressure(&mut findings, graph_analysis, semantic_graph);
    for finding in &mut findings {
        finding.fingerprint = architectural_assessment_fingerprint(finding);
    }
    findings.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(left.file_path.cmp(&right.file_path))
            .then(left.kind.cmp(&right.kind))
    });
    ArchitecturalAssessment { findings }
}

fn attach_complexity_graph_pressure(
    findings: &mut [ArchitecturalAssessmentFinding],
    graph_analysis: &GraphAnalysis,
    semantic_graph: Option<&SemanticGraph>,
) {
    let Some(semantic_graph) = semantic_graph else {
        return;
    };
    let entry_roots = graph_analysis
        .runtime_entry_candidates
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    if entry_roots.is_empty() {
        return;
    }
    let entry_paths = graph_reachability_paths_for_hotspots(semantic_graph, &entry_roots);
    for finding in findings
        .iter_mut()
        .filter(|finding| finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot)
    {
        if entry_roots.contains(&finding.file_path) {
            push_unique_string(
                &mut finding.warning_families,
                String::from("pressure:direct_runtime_entry"),
            );
            push_unique_string(
                &mut finding.related_identifiers,
                format!("entry_path: {}", finding.file_path.display()),
            );
            finding.pressure_path = vec![ArchitecturalPressureHop {
                file_path: finding.file_path.clone(),
                line: None,
                relation_to_next: None,
                source_symbol: None,
                target_symbol: None,
            }];
            finding.expensive_operation_flow = build_complexity_operation_flow(
                &finding.pressure_path,
                &finding.expensive_operation_sites,
            );
            finding.severity_millis = finding.severity_millis.saturating_add(60).min(1000);
            continue;
        }
        let Some(path) = entry_paths.get(&finding.file_path) else {
            continue;
        };
        push_unique_string(
            &mut finding.warning_families,
            String::from("pressure:entry_reachable_via_graph"),
        );
        push_unique_string(
            &mut finding.related_identifiers,
            format!(
                "entry_path: {}",
                path.iter()
                    .map(|part| part.file_path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
        );
        if let Some(symbols) = architectural_pressure_symbol_summary(path) {
            push_unique_string(
                &mut finding.related_identifiers,
                format!("pressure_path_symbols: {symbols}"),
            );
        }
        if path
            .iter()
            .any(|hop| matches!(hop.relation_to_next, Some(RelationKind::Call)))
        {
            push_unique_string(
                &mut finding.warning_families,
                String::from("pressure:caller_callee_path"),
            );
            finding.severity_millis = finding.severity_millis.saturating_add(40).min(1000);
        }
        finding.pressure_path = path.clone();
        finding.expensive_operation_flow =
            build_complexity_operation_flow(path, &finding.expensive_operation_sites);
        for related in path
            .iter()
            .map(|hop| &hop.file_path)
            .filter(|path| **path != finding.file_path)
        {
            if !finding.related_file_paths.contains(related) {
                finding.related_file_paths.push(related.clone());
            }
        }
        finding.severity_millis = finding.severity_millis.saturating_add(80).min(1000);
    }
    for finding in findings
        .iter_mut()
        .filter(|finding| finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot)
    {
        if finding.expensive_operation_flow.is_empty()
            && !finding.expensive_operation_sites.is_empty()
        {
            finding.expensive_operation_flow = build_complexity_operation_flow(
                &finding.pressure_path,
                &finding.expensive_operation_sites,
            );
        }
    }
}

fn graph_reachability_paths_for_hotspots(
    semantic_graph: &SemanticGraph,
    roots: &HashSet<PathBuf>,
) -> HashMap<PathBuf, Vec<ArchitecturalPressureHop>> {
    const MAX_HOPS: usize = 8;
    #[derive(Clone)]
    struct OutboundHop {
        target_file_path: PathBuf,
        line: usize,
        relation_kind: RelationKind,
        source_symbol: Option<String>,
        target_symbol: Option<String>,
    }

    #[derive(Clone)]
    struct PredecessorHop {
        previous_file_path: PathBuf,
        line: usize,
        relation_kind: RelationKind,
        source_symbol: Option<String>,
        target_symbol: Option<String>,
    }

    let mut outbound = HashMap::<&Path, Vec<OutboundHop>>::new();
    let mut predecessor = HashMap::<PathBuf, PredecessorHop>::new();
    let symbol_names = semantic_graph
        .symbols
        .iter()
        .map(|symbol| (symbol.id.clone(), symbol.name.clone()))
        .collect::<HashMap<_, _>>();

    for edge in &semantic_graph.resolved_edges {
        if !supports_complexity_entry_pressure(edge.relation_kind) {
            continue;
        }
        outbound
            .entry(edge.source_file_path.as_path())
            .or_default()
            .push(OutboundHop {
                target_file_path: edge.target_file_path.clone(),
                line: edge.line,
                relation_kind: edge.relation_kind,
                source_symbol: edge
                    .source_symbol_id
                    .as_ref()
                    .and_then(|id| symbol_names.get(id))
                    .cloned(),
                target_symbol: symbol_names.get(&edge.target_symbol_id).cloned(),
            });
    }
    for targets in outbound.values_mut() {
        targets.sort_by(|left, right| {
            pressure_relation_rank(right.relation_kind)
                .cmp(&pressure_relation_rank(left.relation_kind))
                .then(left.line.cmp(&right.line))
                .then(left.target_file_path.cmp(&right.target_file_path))
        });
    }

    let mut visited = roots.iter().cloned().collect::<HashSet<_>>();
    let mut queue = roots
        .iter()
        .cloned()
        .map(|path| (path, 0usize))
        .collect::<VecDeque<_>>();

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= MAX_HOPS {
            continue;
        }
        let Some(targets) = outbound.get(current.as_path()) else {
            continue;
        };
        for target in targets {
            if !visited.insert(target.target_file_path.clone()) {
                continue;
            }
            predecessor.insert(
                target.target_file_path.clone(),
                PredecessorHop {
                    previous_file_path: current.clone(),
                    line: target.line,
                    relation_kind: target.relation_kind,
                    source_symbol: target.source_symbol.clone(),
                    target_symbol: target.target_symbol.clone(),
                },
            );
            queue.push_back((target.target_file_path.clone(), depth + 1));
        }
    }

    predecessor
        .keys()
        .cloned()
        .map(|target| {
            let mut chain = vec![ArchitecturalPressureHop {
                file_path: target.clone(),
                line: None,
                relation_to_next: None,
                source_symbol: None,
                target_symbol: None,
            }];
            let mut cursor = target.clone();
            while let Some(previous) = predecessor.get(&cursor) {
                chain.push(ArchitecturalPressureHop {
                    file_path: previous.previous_file_path.clone(),
                    line: Some(previous.line),
                    relation_to_next: Some(previous.relation_kind),
                    source_symbol: previous.source_symbol.clone(),
                    target_symbol: previous.target_symbol.clone(),
                });
                cursor = previous.previous_file_path.clone();
            }
            chain.reverse();
            (target, chain)
        })
        .collect()
}

fn pressure_relation_rank(kind: RelationKind) -> usize {
    match kind {
        RelationKind::Call => 5,
        RelationKind::Dispatch => 4,
        RelationKind::ContainerResolution => 3,
        RelationKind::EventPublish => 2,
        RelationKind::Import => 1,
        _ => 0,
    }
}

fn architectural_pressure_symbol_summary(path: &[ArchitecturalPressureHop]) -> Option<String> {
    let labels = path
        .iter()
        .filter_map(|hop| {
            hop.source_symbol
                .as_ref()
                .zip(hop.target_symbol.as_ref())
                .map(|(source, target)| format!("{source} -> {target}"))
        })
        .collect::<Vec<_>>();
    (!labels.is_empty()).then(|| labels.join(" | "))
}

fn supports_complexity_entry_pressure(kind: RelationKind) -> bool {
    matches!(
        kind,
        RelationKind::Import
            | RelationKind::Call
            | RelationKind::Dispatch
            | RelationKind::ContainerResolution
            | RelationKind::EventPublish
    )
}

fn push_unique_string(target: &mut Vec<String>, value: String) {
    if !target.contains(&value) {
        target.push(value);
    }
}

fn detect_warning_heavy_hotspots(
    bottlenecks: &[BottleneckFile],
    dead_code: &DeadCodeResult,
    hardwiring: &HardwiringResult,
    external_analysis: &ExternalAnalysisResult,
) -> Vec<ArchitecturalAssessmentFinding> {
    let mut warning_count = HashMap::<PathBuf, usize>::new();
    let mut warning_weight = HashMap::<PathBuf, usize>::new();
    let mut families = HashMap::<PathBuf, HashSet<String>>::new();

    for finding in &dead_code.findings {
        *warning_count.entry(finding.file_path.clone()).or_default() += 1;
        *warning_weight.entry(finding.file_path.clone()).or_default() += 1;
        families
            .entry(finding.file_path.clone())
            .or_default()
            .insert(String::from("dead_code"));
    }

    for finding in &hardwiring.findings {
        *warning_count.entry(finding.file_path.clone()).or_default() += 1;
        *warning_weight.entry(finding.file_path.clone()).or_default() +=
            hardwiring_weight(finding.category);
        families
            .entry(finding.file_path.clone())
            .or_default()
            .insert(String::from("hardwiring"));
    }

    for finding in &external_analysis.findings {
        let Some(file_path) = finding.file_path.clone() else {
            continue;
        };
        *warning_count.entry(file_path.clone()).or_default() += 1;
        *warning_weight.entry(file_path.clone()).or_default() += external_weight(finding.severity);
        families
            .entry(file_path)
            .or_default()
            .insert(String::from("external"));
    }

    let max_centrality = bottlenecks
        .iter()
        .map(|bottleneck| bottleneck.centrality_millis)
        .max()
        .unwrap_or(1)
        .max(1);
    let mut max_weight = 1usize;
    let mut candidates = Vec::new();
    for bottleneck in bottlenecks {
        let count = warning_count
            .get(&bottleneck.file_path)
            .copied()
            .unwrap_or_default();
        let weight = warning_weight
            .get(&bottleneck.file_path)
            .copied()
            .unwrap_or_default();
        let family_count = families
            .get(&bottleneck.file_path)
            .map(HashSet::len)
            .unwrap_or_default();
        if bottleneck.centrality_millis < 250
            || count < 3
            || weight < 4
            || (family_count < 2 && weight < 6)
        {
            continue;
        }
        max_weight = max_weight.max(weight);
        candidates.push((bottleneck, count, weight, family_count));
    }

    let mut findings = candidates
        .into_iter()
        .map(
            |(bottleneck, count, weight, _family_count)| ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::WarningHeavyHotspot,
                file_path: bottleneck.file_path.clone(),
                related_file_paths: Vec::new(),
                related_identifiers: Vec::new(),
                warning_count: count,
                warning_weight: weight,
                bottleneck_centrality_millis: bottleneck.centrality_millis,
                warning_families: families
                    .remove(&bottleneck.file_path)
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                severity_millis: (((bottleneck.centrality_millis as f64 / max_centrality as f64)
                    + (weight as f64 / max_weight as f64))
                    / 2.0
                    * 1000.0)
                    .round() as u16,
                pressure_path: Vec::new(),
                expensive_operation_sites: Vec::new(),
                expensive_operation_flow: Vec::new(),
                fingerprint: String::new(),
            },
        )
        .collect::<Vec<_>>();
    findings.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(right.warning_weight.cmp(&left.warning_weight))
            .then(left.file_path.cmp(&right.file_path))
    });
    findings
}

fn hardwiring_weight(category: HardwiringCategory) -> usize {
    match category {
        HardwiringCategory::MagicString | HardwiringCategory::RepeatedLiteral => 1,
        HardwiringCategory::HardcodedNetwork | HardwiringCategory::EnvOutsideConfig => 2,
    }
}

fn external_weight(severity: ExternalSeverity) -> usize {
    match severity {
        ExternalSeverity::High => 3,
        ExternalSeverity::Medium => 2,
        ExternalSeverity::Low => 1,
    }
}

fn architectural_assessment_fingerprint(finding: &ArchitecturalAssessmentFinding) -> String {
    let kind = architectural_assessment_kind_label(finding.kind);
    let primary_path = normalized_path(&finding.file_path);
    let related_paths = sorted_display_parts(&finding.related_file_paths);
    let related_identifiers = sorted_string_parts(&finding.related_identifiers);
    let warning_families = sorted_string_parts(&finding.warning_families);
    let mut parts = vec!["architecture", kind, primary_path.as_str()];
    parts.extend(related_paths.iter().map(String::as_str));
    parts.extend(related_identifiers.iter().map(String::as_str));
    parts.extend(warning_families.iter().map(String::as_str));
    stable_fingerprint(&parts)
}

fn architectural_assessment_kind_label(kind: ArchitecturalAssessmentKind) -> &'static str {
    match kind {
        ArchitecturalAssessmentKind::WarningHeavyHotspot => "warning-heavy-hotspot",
        ArchitecturalAssessmentKind::SplitIdentityModel => "split-identity-model",
        ArchitecturalAssessmentKind::CompatibilityScar => "compatibility-scar",
        ArchitecturalAssessmentKind::DuplicateMechanism => "duplicate-mechanism",
        ArchitecturalAssessmentKind::SanctionedPathBypass => "sanctioned-path-bypass",
        ArchitecturalAssessmentKind::HandRolledParsing => "hand-rolled-parsing",
        ArchitecturalAssessmentKind::AbstractionSprawl => "abstraction-sprawl",
        ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot => {
            "algorithmic-complexity-hotspot"
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComplexityLanguage {
    Brace,
    Python,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ComplexitySubtype {
    NestedIteration,
    CollectionScanInLoop,
    SortInLoop,
    RegexCompileInLoop,
    JsonDecodeInLoop,
    FilesystemReadInLoop,
    DatabaseQueryInLoop,
    HttpCallInLoop,
    CacheLookupInLoop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ComplexityObservationSource {
    Native,
    AstGrep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComplexityObservation {
    subtype: ComplexitySubtype,
    line: usize,
    token: String,
    source: ComplexityObservationSource,
}

fn detect_algorithmic_complexity_hotspots(
    parsed_sources: &[(PathBuf, String)],
    ast_grep_scan: &AstGrepScanResult,
) -> Vec<ArchitecturalAssessmentFinding> {
    let mut findings = Vec::new();
    let ast_grep_by_path = build_ast_grep_complexity_lookup(ast_grep_scan);
    for (path, source) in parsed_sources {
        // Migrations, seeders, and test code run once (offline, during deploy
        // or CI), not on the hot request path, so their loops are not runtime
        // algorithmic hotspots.
        if is_non_runtime_source_path(path) {
            continue;
        }
        let Some(language) = complexity_language(path) else {
            continue;
        };
        // Blank string-literal and comment interiors before the line/brace scan
        // so loop keywords in prose (`"Cleared cache for tenant"`) and braces
        // inside strings never register as nested iteration or corrupt depth.
        let masked_lines = match crate::lexmask::MaskLanguage::from_path(path) {
            Some(mask_language) => crate::lexmask::mask_non_code_spans(mask_language, source),
            None => source.lines().map(str::to_owned).collect(),
        };
        let mut observations = match language {
            ComplexityLanguage::Brace => detect_brace_language_complexity(&masked_lines),
            ComplexityLanguage::Python => detect_python_complexity(&masked_lines),
        };
        if let Some(scanner_observations) = ast_grep_by_path.get(path) {
            observations.extend(scanner_observations.iter().cloned());
        }
        // The secondary scanner matches `in_array($$$ARGS)` structurally and
        // cannot see that the haystack is an inline literal set. Apply the same
        // fixed-set discrimination here so both planes agree: a membership test
        // against a small literal array is O(1), not a collection scan.
        let source_lines: Vec<&str> = source.lines().collect();
        observations.retain(|observation| {
            if observation.subtype != ComplexitySubtype::CollectionScanInLoop {
                return true;
            }
            match source_lines.get(observation.line.saturating_sub(1)) {
                Some(line) => !line_is_fixed_set_membership_scan(line.trim()),
                None => true,
            }
        });
        if observations.is_empty() {
            continue;
        }
        observations.sort_by(|left, right| {
            left.line
                .cmp(&right.line)
                .then(left.token.cmp(&right.token))
                .then(left.source.cmp(&right.source))
        });
        observations.dedup_by(|left, right| {
            left.subtype == right.subtype && left.line == right.line && left.token == right.token
        });
        let mut grouped = HashMap::<ComplexitySubtype, Vec<ComplexityObservation>>::new();
        for observation in observations {
            grouped
                .entry(observation.subtype)
                .or_default()
                .push(observation);
        }
        for (subtype, mut subtype_observations) in grouped {
            subtype_observations.sort_by(|left, right| left.line.cmp(&right.line));
            subtype_observations
                .first()
                .expect("complexity subgroup must be non-empty");
            let mut related_identifiers = subtype_observations
                .iter()
                .map(|observation| observation.token.clone())
                .collect::<Vec<_>>();
            related_identifiers.sort();
            related_identifiers.dedup();
            related_identifiers.truncate(4);
            if related_identifiers.is_empty() {
                related_identifiers.push(String::from("loop"));
            }
            let mut warning_families =
                vec![format!("complexity:{}", complexity_subtype_label(subtype))];
            if subtype_observations
                .iter()
                .any(|observation| observation.source == ComplexityObservationSource::AstGrep)
            {
                warning_families.push(String::from("scanner:ast_grep"));
            }
            findings.push(ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot,
                file_path: path.clone(),
                related_file_paths: Vec::new(),
                related_identifiers,
                warning_count: subtype_observations.len(),
                warning_weight: complexity_weight(subtype) * subtype_observations.len(),
                bottleneck_centrality_millis: 0,
                warning_families,
                severity_millis: complexity_severity_millis(subtype, subtype_observations.len()),
                pressure_path: Vec::new(),
                expensive_operation_sites: complexity_operation_sites_for_observations(
                    path,
                    &subtype_observations,
                ),
                expensive_operation_flow: Vec::new(),
                fingerprint: String::new(),
            });
        }
    }
    findings.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(left.file_path.cmp(&right.file_path))
            .then(left.warning_families.cmp(&right.warning_families))
    });
    findings
}

// Files that never execute on the hot runtime path: schema migrations,
// database seeders, and test/spec code. Their loops carry no request-time
// algorithmic pressure.
fn is_non_runtime_source_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    if normalized.split('/').any(|segment| {
        matches!(
            segment,
            "migrations" | "seeders" | "tests" | "test" | "spec" | "__tests__"
        )
    }) {
        return true;
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_lowercase();
    file_name.ends_with("seeder.php")
        || file_name.ends_with("_test.rs")
        || file_name.ends_with("_test.go")
        || file_name.ends_with("_test.py")
        || file_name.starts_with("test_")
        || file_name.contains(".test.")
        || file_name.contains(".spec.")
}

fn complexity_language(path: &Path) -> Option<ComplexityLanguage> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("rs" | "js" | "jsx" | "ts" | "tsx" | "php") => Some(ComplexityLanguage::Brace),
        Some("py") => Some(ComplexityLanguage::Python),
        _ => None,
    }
}

fn detect_brace_language_complexity(masked_lines: &[String]) -> Vec<ComplexityObservation> {
    let mut findings = Vec::new();
    let mut brace_depth = 0usize;
    let mut loop_thresholds = Vec::<usize>::new();
    for (index, raw_line) in masked_lines.iter().map(String::as_str).enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            let (open_braces, close_braces) = brace_delta(raw_line);
            brace_depth = brace_depth
                .saturating_add(open_braces)
                .saturating_sub(close_braces);
            while loop_thresholds
                .last()
                .is_some_and(|threshold| brace_depth < *threshold)
            {
                loop_thresholds.pop();
            }
            continue;
        }
        while loop_thresholds
            .last()
            .is_some_and(|threshold| brace_depth < *threshold)
        {
            loop_thresholds.pop();
        }
        let in_loop = !loop_thresholds.is_empty();
        if loop_line_pattern().is_match(line) && in_loop {
            findings.push(ComplexityObservation {
                subtype: ComplexitySubtype::NestedIteration,
                line: line_number,
                token: String::from("loop"),
                source: ComplexityObservationSource::Native,
            });
        }
        if in_loop {
            if let Some(matched) = collection_scan_pattern().find(line) {
                let token = matched.as_str().trim().to_owned();
                // `in_array($x, ['a', 'b'], true)` / `array_search($x, [..])`
                // against an inline literal array is a fixed-set membership test
                // (small constant size), not an O(n) scan of an unbounded
                // collection. Only a variable haystack is worth flagging.
                if !is_fixed_set_membership_scan(&token, line, matched.end()) {
                    findings.push(ComplexityObservation {
                        subtype: ComplexitySubtype::CollectionScanInLoop,
                        line: line_number,
                        token,
                        source: ComplexityObservationSource::Native,
                    });
                }
            }
            if let Some(token) = first_regex_token(sort_in_loop_pattern(), line) {
                findings.push(ComplexityObservation {
                    subtype: ComplexitySubtype::SortInLoop,
                    line: line_number,
                    token,
                    source: ComplexityObservationSource::Native,
                });
            }
            if let Some(token) = first_regex_token(regex_compile_pattern(), line) {
                findings.push(ComplexityObservation {
                    subtype: ComplexitySubtype::RegexCompileInLoop,
                    line: line_number,
                    token,
                    source: ComplexityObservationSource::Native,
                });
            }
            if let Some(token) = first_regex_token(json_decode_pattern(), line) {
                findings.push(ComplexityObservation {
                    subtype: ComplexitySubtype::JsonDecodeInLoop,
                    line: line_number,
                    token,
                    source: ComplexityObservationSource::Native,
                });
            }
            if let Some(token) = first_regex_token(filesystem_read_pattern(), line) {
                findings.push(ComplexityObservation {
                    subtype: ComplexitySubtype::FilesystemReadInLoop,
                    line: line_number,
                    token,
                    source: ComplexityObservationSource::Native,
                });
            }
        }
        let (open_braces, close_braces) = brace_delta(raw_line);
        if loop_line_pattern().is_match(line) && open_braces > 0 {
            loop_thresholds.push(brace_depth + open_braces);
        }
        brace_depth = brace_depth
            .saturating_add(open_braces)
            .saturating_sub(close_braces);
        while loop_thresholds
            .last()
            .is_some_and(|threshold| brace_depth < *threshold)
        {
            loop_thresholds.pop();
        }
    }
    findings
}

fn detect_python_complexity(masked_lines: &[String]) -> Vec<ComplexityObservation> {
    let mut findings = Vec::new();
    let mut loop_indents = Vec::<usize>::new();
    for (index, raw_line) in masked_lines.iter().map(String::as_str).enumerate() {
        let line_number = index + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = raw_line.len().saturating_sub(raw_line.trim_start().len());
        while loop_indents
            .last()
            .is_some_and(|active_indent| indent <= *active_indent)
        {
            loop_indents.pop();
        }
        let in_loop = !loop_indents.is_empty();
        if python_loop_pattern().is_match(trimmed) && in_loop {
            findings.push(ComplexityObservation {
                subtype: ComplexitySubtype::NestedIteration,
                line: line_number,
                token: String::from("loop"),
                source: ComplexityObservationSource::Native,
            });
        }
        if in_loop {
            if let Some(token) = first_regex_token(collection_scan_pattern(), trimmed) {
                findings.push(ComplexityObservation {
                    subtype: ComplexitySubtype::CollectionScanInLoop,
                    line: line_number,
                    token,
                    source: ComplexityObservationSource::Native,
                });
            }
            if let Some(token) = first_regex_token(sort_in_loop_pattern(), trimmed) {
                findings.push(ComplexityObservation {
                    subtype: ComplexitySubtype::SortInLoop,
                    line: line_number,
                    token,
                    source: ComplexityObservationSource::Native,
                });
            }
            if let Some(token) = first_regex_token(regex_compile_pattern(), trimmed) {
                findings.push(ComplexityObservation {
                    subtype: ComplexitySubtype::RegexCompileInLoop,
                    line: line_number,
                    token,
                    source: ComplexityObservationSource::Native,
                });
            }
            if let Some(token) = first_regex_token(json_decode_pattern(), trimmed) {
                findings.push(ComplexityObservation {
                    subtype: ComplexitySubtype::JsonDecodeInLoop,
                    line: line_number,
                    token,
                    source: ComplexityObservationSource::Native,
                });
            }
            if let Some(token) = first_regex_token(filesystem_read_pattern(), trimmed) {
                findings.push(ComplexityObservation {
                    subtype: ComplexitySubtype::FilesystemReadInLoop,
                    line: line_number,
                    token,
                    source: ComplexityObservationSource::Native,
                });
            }
        }
        if python_loop_pattern().is_match(trimmed) {
            loop_indents.push(indent);
        }
    }
    findings
}

fn build_ast_grep_complexity_lookup(
    ast_grep_scan: &AstGrepScanResult,
) -> HashMap<PathBuf, Vec<ComplexityObservation>> {
    let mut lookup = HashMap::<PathBuf, Vec<ComplexityObservation>>::new();
    for finding in &ast_grep_scan.findings {
        let AstGrepFindingKind::AlgorithmicComplexity { subtype, .. } = &finding.kind else {
            continue;
        };
        lookup
            .entry(finding.file_path.clone())
            .or_default()
            .push(ComplexityObservation {
                subtype: ast_grep_complexity_subtype(*subtype),
                line: finding.line,
                token: finding.token.clone(),
                source: ComplexityObservationSource::AstGrep,
            });
    }
    lookup
}

fn ast_grep_complexity_subtype(subtype: AstGrepComplexitySubtype) -> ComplexitySubtype {
    match subtype {
        AstGrepComplexitySubtype::CollectionScanInLoop => ComplexitySubtype::CollectionScanInLoop,
        AstGrepComplexitySubtype::SortInLoop => ComplexitySubtype::SortInLoop,
        AstGrepComplexitySubtype::RegexCompileInLoop => ComplexitySubtype::RegexCompileInLoop,
        AstGrepComplexitySubtype::JsonDecodeInLoop => ComplexitySubtype::JsonDecodeInLoop,
        AstGrepComplexitySubtype::FilesystemReadInLoop => ComplexitySubtype::FilesystemReadInLoop,
        AstGrepComplexitySubtype::DatabaseQueryInLoop => ComplexitySubtype::DatabaseQueryInLoop,
        AstGrepComplexitySubtype::HttpCallInLoop => ComplexitySubtype::HttpCallInLoop,
        AstGrepComplexitySubtype::CacheLookupInLoop => ComplexitySubtype::CacheLookupInLoop,
    }
}

fn complexity_evidence_source(source: ComplexityObservationSource) -> ComplexityEvidenceSource {
    match source {
        ComplexityObservationSource::Native => ComplexityEvidenceSource::Native,
        ComplexityObservationSource::AstGrep => ComplexityEvidenceSource::AstGrep,
    }
}

fn complexity_operation_sites_for_observations(
    path: &Path,
    observations: &[ComplexityObservation],
) -> Vec<ArchitecturalComplexitySite> {
    let mut observations = observations.to_vec();
    observations.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(right.source.cmp(&left.source))
            .then(left.token.len().cmp(&right.token.len()))
            .then(left.token.cmp(&right.token))
    });
    observations.dedup_by(|left, right| left.subtype == right.subtype && left.line == right.line);
    observations
        .into_iter()
        .take(3)
        .map(|observation| ArchitecturalComplexitySite {
            file_path: path.to_path_buf(),
            line: observation.line,
            subtype: String::from(complexity_subtype_label(observation.subtype)),
            token: observation.token,
            source: complexity_evidence_source(observation.source),
        })
        .collect()
}

fn build_complexity_operation_flow(
    pressure_path: &[ArchitecturalPressureHop],
    sites: &[ArchitecturalComplexitySite],
) -> Vec<ArchitecturalComplexityFlowStep> {
    let mut flow = pressure_path
        .iter()
        .map(|hop| ArchitecturalComplexityFlowStep {
            kind: ArchitecturalComplexityFlowStepKind::PressureHop,
            file_path: hop.file_path.clone(),
            line: hop.line,
            relation_to_next: hop.relation_to_next,
            source_symbol: hop.source_symbol.clone(),
            target_symbol: hop.target_symbol.clone(),
            subtype: None,
            token: None,
            evidence_source: None,
        })
        .collect::<Vec<_>>();
    flow.extend(
        sites
            .iter()
            .take(1)
            .map(|site| ArchitecturalComplexityFlowStep {
                kind: ArchitecturalComplexityFlowStepKind::OperationSite,
                file_path: site.file_path.clone(),
                line: Some(site.line),
                relation_to_next: None,
                source_symbol: None,
                target_symbol: None,
                subtype: Some(site.subtype.clone()),
                token: Some(site.token.clone()),
                evidence_source: Some(site.source),
            }),
    );
    flow
}

fn brace_delta(line: &str) -> (usize, usize) {
    let mut open_braces = 0usize;
    let mut close_braces = 0usize;
    for character in line.chars() {
        match character {
            '{' => open_braces += 1,
            '}' => close_braces += 1,
            _ => {}
        }
    }
    (open_braces, close_braces)
}

fn complexity_weight(subtype: ComplexitySubtype) -> usize {
    match subtype {
        ComplexitySubtype::NestedIteration => 3,
        ComplexitySubtype::CollectionScanInLoop => 2,
        ComplexitySubtype::SortInLoop => 3,
        ComplexitySubtype::RegexCompileInLoop => 2,
        ComplexitySubtype::JsonDecodeInLoop => 2,
        ComplexitySubtype::FilesystemReadInLoop => 2,
        ComplexitySubtype::DatabaseQueryInLoop => 4,
        ComplexitySubtype::HttpCallInLoop => 4,
        ComplexitySubtype::CacheLookupInLoop => 3,
    }
}

fn complexity_severity_millis(subtype: ComplexitySubtype, occurrences: usize) -> u16 {
    let base = match subtype {
        ComplexitySubtype::NestedIteration => 860u16,
        ComplexitySubtype::CollectionScanInLoop => 740u16,
        ComplexitySubtype::SortInLoop => 820u16,
        ComplexitySubtype::RegexCompileInLoop => 760u16,
        ComplexitySubtype::JsonDecodeInLoop => 780u16,
        ComplexitySubtype::FilesystemReadInLoop => 800u16,
        ComplexitySubtype::DatabaseQueryInLoop => 900u16,
        ComplexitySubtype::HttpCallInLoop => 920u16,
        ComplexitySubtype::CacheLookupInLoop => 840u16,
    };
    (base + (occurrences.saturating_sub(1) as u16 * 40)).min(980)
}

fn complexity_subtype_label(subtype: ComplexitySubtype) -> &'static str {
    match subtype {
        ComplexitySubtype::NestedIteration => "nested_iteration",
        ComplexitySubtype::CollectionScanInLoop => "collection_scan_in_loop",
        ComplexitySubtype::SortInLoop => "sort_in_loop",
        ComplexitySubtype::RegexCompileInLoop => "regex_compile_in_loop",
        ComplexitySubtype::JsonDecodeInLoop => "json_decode_in_loop",
        ComplexitySubtype::FilesystemReadInLoop => "filesystem_read_in_loop",
        ComplexitySubtype::DatabaseQueryInLoop => "database_query_in_loop",
        ComplexitySubtype::HttpCallInLoop => "http_call_in_loop",
        ComplexitySubtype::CacheLookupInLoop => "cache_lookup_in_loop",
    }
}

// Whether a source line's collection-scan call is a fixed-set membership test
// (`in_array($x, [literal], ...)`). Shared by the native line scanner and the
// secondary-scanner merge so both planes discriminate identically.
fn line_is_fixed_set_membership_scan(line: &str) -> bool {
    match collection_scan_pattern().find(line) {
        Some(matched) => is_fixed_set_membership_scan(matched.as_str().trim(), line, matched.end()),
        None => false,
    }
}

// True when a PHP `in_array`/`array_search` call scans an inline literal array
// (`[...]` or `array(...)`) rather than a variable collection. The haystack is
// then a fixed, small set — a constant-time membership check, not an algorithmic
// hotspot. `args_start` is the byte offset just past the opening paren.
fn is_fixed_set_membership_scan(token: &str, line: &str, args_start: usize) -> bool {
    if !(token.starts_with("in_array") || token.starts_with("array_search")) {
        return false;
    }
    let bytes = line.as_bytes();
    let mut depth: i32 = 1;
    let mut in_single = false;
    let mut in_double = false;
    let mut i = args_start;
    let mut haystack_start = None;
    while i < bytes.len() {
        let c = bytes[i];
        if in_single {
            if c == b'\'' {
                in_single = false;
            }
            i += 1;
            continue;
        }
        if in_double {
            if c == b'"' {
                in_double = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    return false;
                }
            }
            b',' if depth == 1 => {
                haystack_start = Some(i + 1);
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let Some(mut j) = haystack_start else {
        return false;
    };
    while j < bytes.len() && (bytes[j] as char).is_whitespace() {
        j += 1;
    }
    let rest = &line[j..];
    rest.starts_with('[') || rest.starts_with("array(")
}

fn first_regex_token(pattern: &Regex, line: &str) -> Option<String> {
    pattern
        .find(line)
        .map(|matched| matched.as_str().trim().to_owned())
}

fn loop_line_pattern() -> &'static Regex {
    static LOOP_LINE_PATTERN: OnceLock<Regex> = OnceLock::new();
    LOOP_LINE_PATTERN
        .get_or_init(|| Regex::new(r"\b(for(each)?|while)\b").expect("valid loop pattern"))
}

fn python_loop_pattern() -> &'static Regex {
    static PYTHON_LOOP_PATTERN: OnceLock<Regex> = OnceLock::new();
    PYTHON_LOOP_PATTERN
        .get_or_init(|| Regex::new(r"^(for|while)\b").expect("valid python loop pattern"))
}

fn collection_scan_pattern() -> &'static Regex {
    static COLLECTION_SCAN_PATTERN: OnceLock<Regex> = OnceLock::new();
    COLLECTION_SCAN_PATTERN.get_or_init(|| {
        Regex::new(r"(\.contains\s*\(|\.includes\s*\(|\.find\s*\(|\.any\s*\(|\.position\s*\(|in_array\s*\(|array_search\s*\()")
            .expect("valid collection scan pattern")
    })
}

fn sort_in_loop_pattern() -> &'static Regex {
    static SORT_IN_LOOP_PATTERN: OnceLock<Regex> = OnceLock::new();
    SORT_IN_LOOP_PATTERN.get_or_init(|| {
        Regex::new(r"(\.sort(_by)?\s*\(|sort_by\s*\(|sort_unstable(_by)?\s*\(|usort\s*\(|ksort\s*\(|asort\s*\()")
            .expect("valid sort in loop pattern")
    })
}

fn regex_compile_pattern() -> &'static Regex {
    static REGEX_COMPILE_PATTERN: OnceLock<Regex> = OnceLock::new();
    REGEX_COMPILE_PATTERN.get_or_init(|| {
        Regex::new(r"(Regex::new\s*\(|new\s+RegExp\s*\()").expect("valid regex compile pattern")
    })
}

fn json_decode_pattern() -> &'static Regex {
    static JSON_DECODE_PATTERN: OnceLock<Regex> = OnceLock::new();
    JSON_DECODE_PATTERN.get_or_init(|| {
        Regex::new(
            r"(\bjson_decode\s*\(|\bjson\.loads\s*\(|\bjson\.load\s*\(|\bJSON\.parse\s*\(|\bserde_json::from_(str|slice|reader)\s*\()",
        )
        .expect("valid json decode pattern")
    })
}

fn filesystem_read_pattern() -> &'static Regex {
    static FILESYSTEM_READ_PATTERN: OnceLock<Regex> = OnceLock::new();
    FILESYSTEM_READ_PATTERN.get_or_init(|| {
        Regex::new(
            r"(\bstd::fs::(read|read_to_string|metadata)\s*\(|\bfs::(read|read_to_string|metadata)\s*\(|\bfs\.(readFileSync|readFile|existsSync)\s*\(|\bfile_get_contents\s*\(|\bfile_exists\s*\(|\bos\.path\.exists\s*\(|\bPath\([^)]*\)\.(read_text|read_bytes|exists)\s*\(|\bPathBuf::from\([^)]*\)\.exists\s*\()",
        )
        .expect("valid filesystem read pattern")
    })
}

fn sorted_display_parts(paths: &[PathBuf]) -> Vec<String> {
    let mut parts = paths
        .iter()
        .map(|path| normalized_path(path))
        .collect::<Vec<_>>();
    parts.sort();
    parts.dedup();
    parts
}

fn sorted_string_parts(values: &[String]) -> Vec<String> {
    let mut parts = values.to_vec();
    parts.sort();
    parts.dedup();
    parts
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentityVariantKind {
    Base,
    Identity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IdentifierStyle {
    Snake,
    Camel,
    Pascal,
    Other,
}

#[derive(Debug, Default)]
struct IdentityVariantAccumulator {
    base_occurrences: usize,
    identity_occurrences: usize,
    variant_counts: HashMap<String, usize>,
    file_counts: HashMap<PathBuf, usize>,
    code_styles_by_file: HashMap<PathBuf, HashSet<IdentifierStyle>>,
}

// Identifiers inside string literals are data contracts (array keys, column
// names, payload fields), not code naming choices; scanning them makes every
// ORM property / column-name pair look like naming drift.
// Whether the identifier at `start` (a byte offset into `masked`) is read
// through a member-access or scope-resolution operator: `$obj->field` /
// `Class::field` (PHP), `obj.field` / `obj?.field` (JS/TS/Python/Ruby). Such a
// position is an object field or external column, not a bare same-layer
// identifier, so it does not evidence naming drift on its own.
fn is_member_access_position(masked: &str, start: usize) -> bool {
    let prefix = masked[..start].trim_end();
    prefix.ends_with("->")
        || prefix.ends_with("::")
        // A single `.` preceding the identifier is member access; guard against
        // `..`/`...` (ranges, spreads) which are not field reads.
        || (prefix.ends_with('.') && !prefix.ends_with(".."))
}

// Blank every non-code region — string literals (including multi-line and
// heredoc/nowdoc bodies) and comments — to spaces while preserving newlines so
// line numbers stay aligned. Identifiers are only extracted from the code that
// survives. This matters for naming-drift detection: without it, snake_case
// database column names embedded in SQL heredocs and multi-line query strings
// leak out as if they were code identifiers sitting next to their camelCase
// entity properties, which is the normal ORM boundary, not drift. Comments are
// masked too so an apostrophe in a comment cannot open a phantom string.
fn mask_string_literals(content: &str) -> String {
    enum State {
        Normal,
        LineComment,
        BlockComment,
        Quote(char),
        Heredoc,
    }
    let chars: Vec<char> = content.chars().collect();
    let mut masked = String::with_capacity(content.len());
    let mut state = State::Normal;
    let mut escaped = false;
    let mut heredoc_label = String::new();
    let mut i = 0usize;
    let blank = |ch: char| if ch == '\n' { '\n' } else { ' ' };
    while i < chars.len() {
        let ch = chars[i];
        match state {
            State::Normal => {
                if ch == '/' && chars.get(i + 1) == Some(&'/') {
                    state = State::LineComment;
                    masked.push_str("  ");
                    i += 2;
                } else if ch == '/' && chars.get(i + 1) == Some(&'*') {
                    state = State::BlockComment;
                    masked.push_str("  ");
                    i += 2;
                } else if ch == '#' && !matches!(chars.get(i + 1), Some('[') | Some('!')) {
                    // PHP/Python/Ruby/shell line comment (but not Rust `#[`/`#!`).
                    state = State::LineComment;
                    masked.push(' ');
                    i += 1;
                } else if ch == '<'
                    && chars.get(i + 1) == Some(&'<')
                    && chars.get(i + 2) == Some(&'<')
                {
                    // Heredoc / nowdoc: `<<<LABEL`, `<<<'LABEL'`, `<<<"LABEL"`.
                    let mut j = i + 3;
                    while matches!(chars.get(j), Some(' ') | Some('\t')) {
                        j += 1;
                    }
                    let opened_quote = matches!(chars.get(j), Some('\'') | Some('"'));
                    if opened_quote {
                        j += 1;
                    }
                    let label_start = j;
                    while chars
                        .get(j)
                        .is_some_and(|c| c.is_alphanumeric() || *c == '_')
                    {
                        j += 1;
                    }
                    if j > label_start {
                        heredoc_label = chars[label_start..j].iter().collect();
                        if opened_quote && matches!(chars.get(j), Some('\'') | Some('"')) {
                            j += 1;
                        }
                        for &c in &chars[i..j] {
                            masked.push(blank(c));
                        }
                        i = j;
                        state = State::Heredoc;
                    } else {
                        masked.push(ch);
                        i += 1;
                    }
                } else if matches!(ch, '"' | '\'' | '`') {
                    state = State::Quote(ch);
                    escaped = false;
                    masked.push(' ');
                    i += 1;
                } else {
                    masked.push(ch);
                    i += 1;
                }
            }
            State::LineComment => {
                if ch == '\n' {
                    state = State::Normal;
                    masked.push('\n');
                } else {
                    masked.push(' ');
                }
                i += 1;
            }
            State::BlockComment => {
                if ch == '*' && chars.get(i + 1) == Some(&'/') {
                    state = State::Normal;
                    masked.push_str("  ");
                    i += 2;
                } else {
                    masked.push(blank(ch));
                    i += 1;
                }
            }
            State::Quote(quote) => {
                // Strings span newlines (masked across them); comment masking
                // above prevents a stray apostrophe from opening a phantom one.
                if !escaped && ch == quote {
                    state = State::Normal;
                }
                escaped = !escaped && ch == '\\';
                masked.push(blank(ch));
                i += 1;
            }
            State::Heredoc => {
                let at_line_start = i == 0 || chars[i - 1] == '\n';
                if at_line_start {
                    let mut j = i;
                    while matches!(chars.get(j), Some(' ') | Some('\t')) {
                        j += 1;
                    }
                    let token_start = j;
                    while chars
                        .get(j)
                        .is_some_and(|c| c.is_alphanumeric() || *c == '_')
                    {
                        j += 1;
                    }
                    let candidate: String = chars[token_start..j].iter().collect();
                    if candidate == heredoc_label {
                        for &c in &chars[i..j] {
                            masked.push(blank(c));
                        }
                        i = j;
                        state = State::Normal;
                        continue;
                    }
                }
                masked.push(blank(ch));
                i += 1;
            }
        }
    }
    masked
}

// Generated artifacts legitimately mirror both storage and code spellings;
// naming drift inside them is a generator concern, not per-file debt.
fn is_generated_source(content: &str) -> bool {
    let head = content
        .chars()
        .take(600)
        .collect::<String>()
        .to_ascii_lowercase();
    head.contains("auto-generated")
        || head.contains("autogenerated")
        || head.contains("do not edit")
        || head.contains("@generated")
        || head.contains("code generated by")
}

fn detect_split_identity_models(
    parsed_sources: &[(PathBuf, String)],
) -> Vec<ArchitecturalAssessmentFinding> {
    let mut groups = HashMap::<String, IdentityVariantAccumulator>::new();

    for (path, content) in parsed_sources {
        if is_low_signal_identity_path(path) {
            continue;
        }
        if is_generated_source(content) {
            continue;
        }
        let masked = mask_string_literals(content);
        for occurrence in identifier_pattern().find_iter(&masked) {
            let identifier = occurrence.as_str();
            let Some((stem, variant_kind)) = classify_identity_variant(identifier) else {
                continue;
            };
            let group = groups.entry(stem).or_default();
            match variant_kind {
                IdentityVariantKind::Base => group.base_occurrences += 1,
                IdentityVariantKind::Identity => group.identity_occurrences += 1,
            }
            *group
                .variant_counts
                .entry(String::from(identifier))
                .or_default() += 1;
            *group.file_counts.entry(path.clone()).or_default() += 1;
            // Pascal is excluded: type names are conventionally Pascal while
            // members are camel/snake, so that mix is not drift.
            let style = identifier_style(identifier);
            // The drift signal is a file mixing snake and camel spellings of one
            // concept as bare, same-layer identifiers. A token in a member-access
            // or scope position (`$row->source_entity_id`, `Table::status_code`)
            // is a database column / external field read across a boundary, not a
            // code identity worth unifying, so it must not count toward the mix.
            if matches!(style, IdentifierStyle::Snake | IdentifierStyle::Camel)
                && !is_member_access_position(&masked, occurrence.start())
            {
                group
                    .code_styles_by_file
                    .entry(path.clone())
                    .or_default()
                    .insert(style);
            }
        }
    }

    let mut findings = groups
        .into_iter()
        .filter_map(|(stem, group)| {
            if group.base_occurrences == 0 || group.identity_occurrences == 0 {
                return None;
            }
            if group.variant_counts.len() < 2 {
                return None;
            }
            let total_occurrences = group.base_occurrences + group.identity_occurrences;
            if total_occurrences < 5 {
                return None;
            }
            if group
                .variant_counts
                .keys()
                .all(|variant| has_accessor_prefix(variant))
            {
                return None;
            }
            // Object-form and id-form of one concept coexisting across layers
            // is how relational code normally looks. The drift signal is a
            // single file mixing snake and camel spellings in code positions.
            if !group
                .code_styles_by_file
                .values()
                .any(|styles| styles.len() >= 2)
            {
                return None;
            }
            let naming_styles = group
                .variant_counts
                .keys()
                .map(|variant| identifier_style(variant))
                .filter(|style| *style != IdentifierStyle::Other)
                .collect::<HashSet<_>>();

            let mut variant_counts = group.variant_counts.into_iter().collect::<Vec<_>>();
            variant_counts.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
            let related_identifiers = variant_counts
                .iter()
                .take(6)
                .map(|(variant, _count)| variant.clone())
                .collect::<Vec<_>>();

            let mut file_counts = group.file_counts.into_iter().collect::<Vec<_>>();
            file_counts.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
            let (primary_file, _) = file_counts.first()?;
            if file_counts.len() < 2 {
                return None;
            }
            if related_identifiers.len() < 3 && naming_styles.len() < 2 {
                return None;
            }
            let related_file_paths = file_counts
                .iter()
                .skip(1)
                .take(5)
                .map(|(path, _count)| path.clone())
                .collect::<Vec<_>>();
            let distinct_files = file_counts.len();
            let severity = (300
                + (distinct_files.min(4) * 125)
                + (related_identifiers.len().min(4) * 100)
                + (total_occurrences.min(12) * 25))
                .min(1000) as u16;

            Some(ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::SplitIdentityModel,
                file_path: primary_file.clone(),
                related_file_paths,
                related_identifiers,
                warning_count: total_occurrences,
                warning_weight: group.identity_occurrences,
                bottleneck_centrality_millis: 0,
                warning_families: vec![format!("concept:{stem}")],
                severity_millis: severity,
                pressure_path: Vec::new(),
                expensive_operation_sites: Vec::new(),
                expensive_operation_flow: Vec::new(),
                fingerprint: String::new(),
            })
        })
        .collect::<Vec<_>>();
    findings.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(left.file_path.cmp(&right.file_path))
    });
    findings
}

fn detect_compatibility_scars(
    split_identity_findings: &[ArchitecturalAssessmentFinding],
    bottlenecks: &[BottleneckFile],
    parsed_sources: &[(PathBuf, String)],
) -> Vec<ArchitecturalAssessmentFinding> {
    let content_by_path = parsed_sources
        .iter()
        .map(|(path, content)| (path.clone(), content))
        .collect::<HashMap<_, _>>();
    let bottleneck_by_path = bottlenecks
        .iter()
        .map(|bottleneck| (bottleneck.file_path.clone(), bottleneck.centrality_millis))
        .collect::<HashMap<_, _>>();
    let mut split_by_file = HashMap::<PathBuf, Vec<&ArchitecturalAssessmentFinding>>::new();
    for finding in split_identity_findings {
        split_by_file
            .entry(finding.file_path.clone())
            .or_default()
            .push(finding);
    }

    let mut findings = split_by_file
        .into_iter()
        .filter_map(|(file_path, split_findings)| {
            let keyword_hits = content_by_path
                .get(&file_path)
                .map(|content| compatibility_keyword_pattern().find_iter(content).count())
                .unwrap_or_default();
            if split_findings.len() < 2 && keyword_hits < 2 {
                return None;
            }

            let mut related_identifiers = split_findings
                .iter()
                .flat_map(|finding| finding.related_identifiers.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            related_identifiers.truncate(6);

            let mut related_file_paths = split_findings
                .iter()
                .flat_map(|finding| finding.related_file_paths.iter().cloned())
                .filter(|path| *path != file_path)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            related_file_paths.truncate(6);

            let mut warning_families = vec![String::from("split_identity")];
            if keyword_hits > 0 {
                warning_families.push(format!("compatibility_keywords:{keyword_hits}"));
            }
            warning_families.extend(
                split_findings
                    .iter()
                    .filter_map(|finding| finding.warning_families.first().cloned())
                    .take(3),
            );

            let centrality = bottleneck_by_path
                .get(&file_path)
                .copied()
                .unwrap_or_default();
            let severity = (350
                + (split_findings.len().min(4) * 140)
                + (keyword_hits.min(5) * 50)
                + (related_file_paths.len().min(4) * 45)
                + ((centrality / 250).min(180) as usize))
                .min(1000) as u16;

            Some(ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::CompatibilityScar,
                file_path,
                related_file_paths,
                related_identifiers,
                warning_count: split_findings.len(),
                warning_weight: keyword_hits + split_findings.len() * 2,
                bottleneck_centrality_millis: centrality,
                warning_families,
                severity_millis: severity,
                pressure_path: Vec::new(),
                expensive_operation_sites: Vec::new(),
                expensive_operation_flow: Vec::new(),
                fingerprint: String::new(),
            })
        })
        .collect::<Vec<_>>();
    findings.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(left.file_path.cmp(&right.file_path))
    });
    findings
}

#[derive(Debug, Default)]
struct DuplicateMechanismAccumulator {
    file_mechanisms: HashMap<PathBuf, BTreeSet<String>>,
    file_token_counts: HashMap<PathBuf, usize>,
}

#[derive(Debug, Default)]
struct AbstractionSprawlAccumulator {
    file_roles: HashMap<PathBuf, BTreeSet<String>>,
    file_token_counts: HashMap<PathBuf, usize>,
}

#[derive(Debug, Default)]
struct HandRolledParsingAccumulator {
    file_roles: HashMap<PathBuf, BTreeSet<String>>,
    file_scores: HashMap<PathBuf, usize>,
    file_concepts: HashMap<PathBuf, BTreeSet<String>>,
}

fn detect_duplicate_mechanisms(
    bottlenecks: &[BottleneckFile],
    parsed_sources: &[(PathBuf, String)],
) -> Vec<ArchitecturalAssessmentFinding> {
    let bottleneck_by_path = bottlenecks
        .iter()
        .map(|bottleneck| (bottleneck.file_path.clone(), bottleneck.centrality_millis))
        .collect::<HashMap<_, _>>();
    let mut groups = HashMap::<String, DuplicateMechanismAccumulator>::new();

    for (path, content) in parsed_sources {
        if is_low_signal_identity_path(path) {
            continue;
        }
        let mechanism_families = detect_mechanism_families(path, content);
        if mechanism_families.is_empty() {
            continue;
        }

        let concept_tokens = duplicate_mechanism_tokens(path, content);
        if concept_tokens.is_empty() {
            continue;
        }

        for token in concept_tokens {
            let group = groups.entry(token).or_default();
            group
                .file_mechanisms
                .entry(path.clone())
                .or_default()
                .extend(mechanism_families.iter().cloned());
            *group.file_token_counts.entry(path.clone()).or_default() += 1;
        }
    }

    let findings = groups
        .into_iter()
        .filter_map(|(concept, group)| {
            if group.file_mechanisms.len() < 2 {
                return None;
            }

            let family_set = group
                .file_mechanisms
                .values()
                .flat_map(|families| families.iter().cloned())
                .collect::<BTreeSet<_>>();
            if family_set.len() < 2 {
                return None;
            }
            if family_set.len() == 2
                && family_set.contains("direct_notifications")
                && family_set.contains("queue_jobs")
            {
                return None;
            }
            if group
                .file_mechanisms
                .values()
                .all(|families| families.len() < 2)
            {
                return None;
            }

            let mut ranked_files = group
                .file_mechanisms
                .into_iter()
                .map(|(path, families)| {
                    let token_count = group
                        .file_token_counts
                        .get(&path)
                        .copied()
                        .unwrap_or_default();
                    let centrality = bottleneck_by_path.get(&path).copied().unwrap_or_default();
                    let anchor_rank = duplicate_mechanism_anchor_rank(&path);
                    (path, families, token_count, centrality, anchor_rank)
                })
                .collect::<Vec<_>>();
            ranked_files.sort_by(|left, right| {
                right
                    .3
                    .cmp(&left.3)
                    .then(right.4.cmp(&left.4))
                    .then(right.1.len().cmp(&left.1.len()))
                    .then(right.2.cmp(&left.2))
                    .then(left.0.cmp(&right.0))
            });

            let (primary_file, _, _, primary_centrality, _) = ranked_files.first()?;

            let related_file_paths = ranked_files
                .iter()
                .skip(1)
                .take(5)
                .map(|(path, _, _, _, _)| path.clone())
                .collect::<Vec<_>>();
            let warning_families = family_set
                .iter()
                .map(|family| format!("mechanism:{family}"))
                .collect::<Vec<_>>();
            let severity = (320
                + (family_set.len().min(4) * 140)
                + (ranked_files.len().min(4) * 90)
                + ((*primary_centrality / 250).min(180) as usize))
                .min(1000) as u16;

            Some(ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::DuplicateMechanism,
                file_path: primary_file.clone(),
                related_file_paths,
                related_identifiers: vec![format!("concept:{concept}")],
                warning_count: ranked_files.len(),
                warning_weight: family_set.len(),
                bottleneck_centrality_millis: *primary_centrality,
                warning_families,
                severity_millis: severity,
                pressure_path: Vec::new(),
                expensive_operation_sites: Vec::new(),
                expensive_operation_flow: Vec::new(),
                fingerprint: String::new(),
            })
        })
        .collect::<Vec<_>>();

    let mut deduped = HashMap::<PathBuf, ArchitecturalAssessmentFinding>::new();
    for finding in findings {
        let entry = deduped
            .entry(finding.file_path.clone())
            .or_insert_with(|| finding.clone());
        if entry.file_path != finding.file_path {
            continue;
        }
        if entry.severity_millis < finding.severity_millis {
            entry.severity_millis = finding.severity_millis;
        }
        entry.warning_count = entry.warning_count.max(finding.warning_count);
        entry.warning_weight = entry.warning_weight.max(finding.warning_weight);
        entry.bottleneck_centrality_millis = entry
            .bottleneck_centrality_millis
            .max(finding.bottleneck_centrality_millis);
        entry.related_file_paths = entry
            .related_file_paths
            .iter()
            .cloned()
            .chain(finding.related_file_paths.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        entry.related_identifiers = entry
            .related_identifiers
            .iter()
            .cloned()
            .chain(finding.related_identifiers.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        entry.warning_families = entry
            .warning_families
            .iter()
            .cloned()
            .chain(finding.warning_families.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
    }

    let mut findings = deduped.into_values().collect::<Vec<_>>();
    findings.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(left.file_path.cmp(&right.file_path))
    });
    findings
}

fn detect_sanctioned_path_bypasses(
    bottlenecks: &[BottleneckFile],
    hardwiring: &HardwiringResult,
    parsed_sources: &[(PathBuf, String)],
    ast_grep_scan: &AstGrepScanResult,
) -> Vec<ArchitecturalAssessmentFinding> {
    let bottleneck_by_path = bottlenecks
        .iter()
        .map(|bottleneck| (bottleneck.file_path.clone(), bottleneck.centrality_millis))
        .collect::<HashMap<_, _>>();
    let env_findings_by_path = hardwiring
        .findings
        .iter()
        .filter(|finding| finding.category == HardwiringCategory::EnvOutsideConfig)
        .fold(
            HashMap::<PathBuf, Vec<&crate::detectors::hardwiring::HardwiringFinding>>::new(),
            |mut acc, finding| {
                acc.entry(finding.file_path.clone())
                    .or_default()
                    .push(finding);
                acc
            },
        );
    let ast_grep_env_lines_by_path = build_ast_grep_framework_lookup(
        ast_grep_scan,
        AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
    );
    let ast_grep_container_lines_by_path = build_ast_grep_framework_lookup(
        ast_grep_scan,
        AstGrepFrameworkMisuseSubtype::RawContainerLookupOutsideBoundary,
    );

    let mut findings = parsed_sources
        .iter()
        .filter_map(|(path, content)| {
            if is_low_signal_identity_path(path) {
                return None;
            }
            let native_lines = env_findings_by_path
                .get(path)
                .into_iter()
                .flat_map(|findings| findings.iter().map(|finding| finding.line))
                .collect::<BTreeSet<_>>();
            let ast_grep_lines = ast_grep_env_lines_by_path
                .get(path)
                .cloned()
                .unwrap_or_default();
            let container_lines = ast_grep_container_lines_by_path
                .get(path)
                .cloned()
                .unwrap_or_default();
            if native_lines.is_empty() && ast_grep_lines.is_empty() && container_lines.is_empty() {
                return None;
            }
            let centrality = bottleneck_by_path.get(path).copied().unwrap_or_default();
            let mut related_identifiers = Vec::new();
            let mut warning_families = Vec::new();
            let mut warning_lines = BTreeSet::new();

            if !is_configuration_boundary_path(path) {
                let config_markers = sanctioned_config_markers(content);
                if !config_markers.is_empty()
                    && (!native_lines.is_empty() || !ast_grep_lines.is_empty())
                {
                    related_identifiers.extend(config_markers);
                    related_identifiers.push(String::from("raw_env"));
                    warning_families.extend([
                        String::from("concern:configuration"),
                        String::from("bypass:raw_env"),
                        String::from("sanctioned:config_access"),
                    ]);
                    warning_lines.extend(native_lines.iter().copied());
                    warning_lines.extend(ast_grep_lines.iter().copied());
                    if !ast_grep_lines.is_empty() {
                        warning_families.push(String::from("scanner:ast_grep"));
                    }
                }
            }

            if !is_dependency_boundary_path(path) {
                let dependency_markers = sanctioned_dependency_markers(content);
                if !dependency_markers.is_empty() && !container_lines.is_empty() {
                    related_identifiers.extend(dependency_markers);
                    related_identifiers.push(String::from("raw_container_lookup"));
                    warning_families.extend([
                        String::from("concern:dependency_resolution"),
                        String::from("bypass:container_lookup"),
                        String::from("sanctioned:dependency_injection"),
                        String::from("scanner:ast_grep"),
                    ]);
                    warning_lines.extend(container_lines.iter().copied());
                }
            }

            if warning_lines.is_empty() {
                return None;
            }
            related_identifiers.sort();
            related_identifiers.dedup();
            warning_families.sort();
            warning_families.dedup();
            let warning_count = warning_lines.len();

            Some(ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::SanctionedPathBypass,
                file_path: path.clone(),
                related_file_paths: Vec::new(),
                related_identifiers,
                warning_count,
                warning_weight: warning_count + 1,
                bottleneck_centrality_millis: centrality,
                warning_families,
                severity_millis: (520
                    + warning_count.min(4) as u16 * 70
                    + ((centrality / 200).min(180) as u16))
                    .min(1000),
                pressure_path: Vec::new(),
                expensive_operation_sites: Vec::new(),
                expensive_operation_flow: Vec::new(),
                fingerprint: String::new(),
            })
        })
        .collect::<Vec<_>>();

    findings.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(left.file_path.cmp(&right.file_path))
    });
    findings
}

fn build_ast_grep_framework_lookup(
    ast_grep_scan: &AstGrepScanResult,
    expected_subtype: AstGrepFrameworkMisuseSubtype,
) -> HashMap<PathBuf, BTreeSet<usize>> {
    let mut lookup = HashMap::<PathBuf, BTreeSet<usize>>::new();
    for finding in &ast_grep_scan.findings {
        let AstGrepFindingKind::FrameworkMisuse { subtype } = &finding.kind else {
            continue;
        };
        if *subtype != expected_subtype {
            continue;
        }
        lookup
            .entry(finding.file_path.clone())
            .or_default()
            .insert(finding.line);
    }
    lookup
}

fn detect_abstraction_sprawl(
    bottlenecks: &[BottleneckFile],
    parsed_sources: &[(PathBuf, String)],
) -> Vec<ArchitecturalAssessmentFinding> {
    let bottleneck_by_path = bottlenecks
        .iter()
        .map(|bottleneck| (bottleneck.file_path.clone(), bottleneck.centrality_millis))
        .collect::<HashMap<_, _>>();
    let mut groups = HashMap::<String, AbstractionSprawlAccumulator>::new();

    for (path, content) in parsed_sources {
        if is_low_signal_identity_path(path) {
            continue;
        }
        let roles = abstraction_roles(path, content);
        if roles.is_empty() {
            continue;
        }
        let concepts = abstraction_sprawl_concepts(path, content);
        if concepts.is_empty() {
            continue;
        }

        for concept in concepts {
            let group = groups.entry(concept).or_default();
            group
                .file_roles
                .entry(path.clone())
                .or_default()
                .extend(roles.iter().cloned());
            *group.file_token_counts.entry(path.clone()).or_default() += 1;
        }
    }

    let findings = groups
        .into_iter()
        .filter_map(|(concept, group)| {
            if group.file_roles.len() < 3 {
                return None;
            }

            let role_set = group
                .file_roles
                .values()
                .flat_map(|roles| roles.iter().cloned())
                .collect::<BTreeSet<_>>();
            if role_set.len() < 4 {
                return None;
            }
            let nontrivial_roles = role_set
                .iter()
                .filter(|role| {
                    !matches!(
                        role.as_str(),
                        "service" | "helper" | "handler" | "client" | "validator"
                    )
                })
                .count();
            if nontrivial_roles < 2 {
                return None;
            }

            let mut ranked_files = group
                .file_roles
                .into_iter()
                .map(|(path, roles)| {
                    let token_count = group
                        .file_token_counts
                        .get(&path)
                        .copied()
                        .unwrap_or_default();
                    let centrality = bottleneck_by_path.get(&path).copied().unwrap_or_default();
                    (path, roles, token_count, centrality)
                })
                .collect::<Vec<_>>();
            ranked_files.sort_by(|left, right| {
                right
                    .3
                    .cmp(&left.3)
                    .then(right.1.len().cmp(&left.1.len()))
                    .then(right.2.cmp(&left.2))
                    .then(left.0.cmp(&right.0))
            });

            let (primary_file, primary_roles, _, primary_centrality) = ranked_files.first()?;
            if primary_roles.len() < 2 && *primary_centrality < 250 {
                return None;
            }

            let related_file_paths = ranked_files
                .iter()
                .skip(1)
                .take(6)
                .map(|(path, _, _, _)| path.clone())
                .collect::<Vec<_>>();
            let mut related_identifiers = vec![format!("concept:{concept}")];
            related_identifiers.extend(role_set.iter().take(6).map(|role| format!("role:{role}")));
            let warning_families = role_set
                .iter()
                .map(|role| format!("abstraction_role:{role}"))
                .collect::<Vec<_>>();
            let severity = (360
                + (role_set.len().min(6) * 90)
                + (ranked_files.len().min(5) * 80)
                + ((*primary_centrality / 250).min(180) as usize))
                .min(1000) as u16;

            Some(ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::AbstractionSprawl,
                file_path: primary_file.clone(),
                related_file_paths,
                related_identifiers,
                warning_count: ranked_files.len(),
                warning_weight: role_set.len(),
                bottleneck_centrality_millis: *primary_centrality,
                warning_families,
                severity_millis: severity,
                pressure_path: Vec::new(),
                expensive_operation_sites: Vec::new(),
                expensive_operation_flow: Vec::new(),
                fingerprint: String::new(),
            })
        })
        .collect::<Vec<_>>();

    let mut deduped = HashMap::<PathBuf, ArchitecturalAssessmentFinding>::new();
    for finding in findings {
        let entry = deduped
            .entry(finding.file_path.clone())
            .or_insert_with(|| finding.clone());
        if entry.severity_millis < finding.severity_millis {
            entry.severity_millis = finding.severity_millis;
        }
        entry.warning_count = entry.warning_count.max(finding.warning_count);
        entry.warning_weight = entry.warning_weight.max(finding.warning_weight);
        entry.bottleneck_centrality_millis = entry
            .bottleneck_centrality_millis
            .max(finding.bottleneck_centrality_millis);
        entry.related_file_paths = entry
            .related_file_paths
            .iter()
            .cloned()
            .chain(finding.related_file_paths.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        entry.related_identifiers = entry
            .related_identifiers
            .iter()
            .cloned()
            .chain(finding.related_identifiers.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        entry.warning_families = entry
            .warning_families
            .iter()
            .cloned()
            .chain(finding.warning_families.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
    }

    let mut findings = deduped.into_values().collect::<Vec<_>>();
    findings.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(left.file_path.cmp(&right.file_path))
    });
    findings
}

fn detect_hand_rolled_parsing(
    bottlenecks: &[BottleneckFile],
    parsed_sources: &[(PathBuf, String)],
) -> Vec<ArchitecturalAssessmentFinding> {
    let bottleneck_by_path = bottlenecks
        .iter()
        .map(|bottleneck| (bottleneck.file_path.clone(), bottleneck.centrality_millis))
        .collect::<HashMap<_, _>>();
    let mut groups = HashMap::<PathBuf, HandRolledParsingAccumulator>::new();

    for (path, content) in parsed_sources {
        if is_low_signal_identity_path(path) {
            continue;
        }
        let roles = parsing_roles(path);
        let score = parsing_primitive_score(content);
        if roles.is_empty() || score < parsing_role_minimum_score(&roles) {
            continue;
        }
        let concepts = parsing_concepts(path);
        let directory = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let group = groups.entry(directory).or_default();
        group
            .file_roles
            .entry(path.clone())
            .or_default()
            .extend(roles.iter().cloned());
        group
            .file_concepts
            .entry(path.clone())
            .or_default()
            .extend(concepts);
        *group.file_scores.entry(path.clone()).or_default() += score;
    }

    let mut findings = groups
        .into_iter()
        .filter_map(|(directory, group)| {
            if group.file_roles.len() < 2 {
                return None;
            }

            let role_set = group
                .file_roles
                .values()
                .flat_map(|roles| roles.iter().cloned())
                .collect::<BTreeSet<_>>();
            let concept_set = group
                .file_concepts
                .values()
                .flat_map(|concepts| concepts.iter().cloned())
                .collect::<BTreeSet<_>>();
            let is_primary_parser_cluster =
                role_set.iter().any(|role| is_primary_parsing_role(role)) && role_set.len() >= 2;
            let is_contract_stack_cluster = role_set.len() >= 3
                && role_set.iter().any(|role| is_contract_stack_role(role))
                && role_set
                    .iter()
                    .any(|role| matches!(role.as_str(), "resolver" | "definition" | "normalizer"))
                && concept_set.is_empty().not();
            let is_schema_validation_cluster = is_contract_stack_cluster
                && role_set.iter().any(|role| role == "validator")
                && role_set
                    .iter()
                    .any(|role| matches!(role.as_str(), "definition" | "normalizer"))
                && concept_set
                    .iter()
                    .any(|concept| matches!(concept.as_str(), "preference" | "schema"));
            let is_definition_engine_cluster = is_primary_parser_cluster.not()
                && role_set.len() >= 3
                && role_set.iter().any(|role| role == "definition")
                && role_set.iter().any(|role| role == "validator").not()
                && role_set
                    .iter()
                    .any(|role| matches!(role.as_str(), "resolver" | "normalizer"))
                && group.file_roles.len() >= 3;
            if !is_primary_parser_cluster
                && !is_contract_stack_cluster
                && !is_definition_engine_cluster
            {
                return None;
            }

            let total_score = group.file_scores.values().sum::<usize>();
            let minimum_total_score = if is_definition_engine_cluster || is_contract_stack_cluster {
                5
            } else {
                8
            };
            if total_score < minimum_total_score {
                return None;
            }

            let mut ranked_files = group
                .file_roles
                .into_iter()
                .map(|(path, roles)| {
                    let score = group.file_scores.get(&path).copied().unwrap_or_default();
                    let centrality = bottleneck_by_path.get(&path).copied().unwrap_or_default();
                    let primary_role_bonus = roles
                        .iter()
                        .filter(|role| is_primary_parsing_role(role))
                        .count();
                    (path, roles, score, centrality, primary_role_bonus)
                })
                .collect::<Vec<_>>();
            ranked_files.sort_by(|left, right| {
                right
                    .3
                    .cmp(&left.3)
                    .then(right.4.cmp(&left.4))
                    .then(right.2.cmp(&left.2))
                    .then(right.1.len().cmp(&left.1.len()))
                    .then(left.0.cmp(&right.0))
            });

            let (primary_file, _, primary_score, primary_centrality, _) = ranked_files.first()?;

            let related_file_paths = ranked_files
                .iter()
                .skip(1)
                .take(6)
                .map(|(path, _, _, _, _)| path.clone())
                .collect::<Vec<_>>();
            let mut related_identifiers =
                vec![format!("directory:{}", normalized_path(&directory))];
            related_identifiers.extend(role_set.iter().take(6).map(|role| format!("role:{role}")));
            related_identifiers.extend(
                concept_set
                    .iter()
                    .take(4)
                    .map(|concept| format!("concept:{concept}")),
            );
            let mut warning_families = role_set
                .iter()
                .map(|role| format!("parsing_role:{role}"))
                .collect::<Vec<_>>();
            warning_families.push(String::from(if is_schema_validation_cluster {
                "concern:custom_schema_validation"
            } else if is_definition_engine_cluster {
                "concern:custom_definition_engine"
            } else if is_contract_stack_cluster {
                "concern:custom_contract_stack"
            } else {
                "concern:custom_parsing"
            }));

            let severity = (360
                + (role_set.len().min(5) * 80)
                + (ranked_files.len().min(4) * 90)
                + (total_score.min(12) * 20)
                + ((*primary_centrality / 250).min(140) as usize))
                .min(1000) as u16;

            Some(ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::HandRolledParsing,
                file_path: primary_file.clone(),
                related_file_paths,
                related_identifiers,
                warning_count: ranked_files.len(),
                warning_weight: total_score.max(*primary_score),
                bottleneck_centrality_millis: *primary_centrality,
                warning_families,
                severity_millis: severity,
                pressure_path: Vec::new(),
                expensive_operation_sites: Vec::new(),
                expensive_operation_flow: Vec::new(),
                fingerprint: String::new(),
            })
        })
        .collect::<Vec<_>>();
    findings.extend(detect_scheduler_dsl_stacks(
        &bottleneck_by_path,
        parsed_sources,
    ));

    findings.sort_by(|left, right| {
        right
            .severity_millis
            .cmp(&left.severity_millis)
            .then(left.file_path.cmp(&right.file_path))
    });
    findings
}

fn detect_scheduler_dsl_stacks(
    bottleneck_by_path: &HashMap<PathBuf, u32>,
    parsed_sources: &[(PathBuf, String)],
) -> Vec<ArchitecturalAssessmentFinding> {
    let mut ranked_files = parsed_sources
        .iter()
        .filter_map(|(path, content)| {
            if is_low_signal_identity_path(path) {
                return None;
            }
            let roles = scheduler_stack_roles(path);
            if roles.is_empty() {
                return None;
            }
            let concepts = scheduler_stack_concepts(path, content);
            if concepts.is_empty() {
                return None;
            }
            let score = scheduler_stack_score(content);
            if score < scheduler_stack_minimum_score(&roles) {
                return None;
            }
            let centrality = bottleneck_by_path.get(path).copied().unwrap_or_default();
            let primary_bonus = scheduler_stack_primary_bonus(&roles);
            Some((
                path.clone(),
                roles,
                concepts,
                score,
                centrality,
                primary_bonus,
            ))
        })
        .collect::<Vec<_>>();

    if ranked_files.len() < 2 {
        return Vec::new();
    }

    let role_set = ranked_files
        .iter()
        .flat_map(|(_, roles, _, _, _, _)| roles.iter().cloned())
        .collect::<BTreeSet<_>>();
    if !role_set.contains("registry")
        || role_set.iter().all(|role| {
            !matches!(
                role.as_str(),
                "executor" | "scheduler" | "command" | "runner"
            )
        })
    {
        return Vec::new();
    }

    let concept_set = ranked_files
        .iter()
        .flat_map(|(_, _, concepts, _, _, _)| concepts.iter().cloned())
        .collect::<BTreeSet<_>>();
    let total_score = ranked_files
        .iter()
        .map(|(_, _, _, score, _, _)| *score)
        .sum::<usize>();
    if total_score < 6 {
        return Vec::new();
    }

    ranked_files.sort_by(|left, right| {
        right
            .4
            .cmp(&left.4)
            .then(right.5.cmp(&left.5))
            .then(right.3.cmp(&left.3))
            .then(right.1.len().cmp(&left.1.len()))
            .then(left.0.cmp(&right.0))
    });

    let Some((primary_file, _, _, primary_score, primary_centrality, _)) = ranked_files.first()
    else {
        return Vec::new();
    };

    let related_file_paths = ranked_files
        .iter()
        .skip(1)
        .take(6)
        .map(|(path, _, _, _, _, _)| path.clone())
        .collect::<Vec<_>>();
    let mut related_identifiers = vec![String::from("concern:scheduler")];
    related_identifiers.extend(role_set.iter().take(6).map(|role| format!("role:{role}")));
    related_identifiers.extend(
        concept_set
            .iter()
            .take(4)
            .map(|concept| format!("concept:{concept}")),
    );
    let mut warning_families = role_set
        .iter()
        .map(|role| format!("parsing_role:{role}"))
        .collect::<Vec<_>>();
    warning_families.push(String::from("concern:custom_scheduler_dsl"));

    let severity = (420
        + (role_set.len().min(5) * 80)
        + (ranked_files.len().min(4) * 100)
        + (total_score.min(12) * 18)
        + ((*primary_centrality / 250).min(140) as usize))
        .min(1000) as u16;

    vec![ArchitecturalAssessmentFinding {
        kind: ArchitecturalAssessmentKind::HandRolledParsing,
        file_path: primary_file.clone(),
        related_file_paths,
        related_identifiers,
        warning_count: ranked_files.len(),
        warning_weight: total_score.max(*primary_score),
        bottleneck_centrality_millis: *primary_centrality,
        warning_families,
        severity_millis: severity,
        pressure_path: Vec::new(),
        expensive_operation_sites: Vec::new(),
        expensive_operation_flow: Vec::new(),
        fingerprint: String::new(),
    }]
}

fn detect_filesystem_page_resolution_stacks(
    bottleneck_by_path: &HashMap<PathBuf, u32>,
    parsed_sources: &[(PathBuf, String)],
) -> Vec<ArchitecturalAssessmentFinding> {
    let mut ranked_files = parsed_sources
        .iter()
        .filter_map(|(path, content)| {
            if is_low_signal_identity_path(path) {
                return None;
            }
            let roles = filesystem_page_resolution_roles(path);
            if roles.is_empty() {
                return None;
            }
            let has_transport_role = roles
                .iter()
                .any(|role| matches!(role.as_str(), "route" | "router" | "page" | "action"));
            if !has_transport_role {
                return None;
            }
            let concepts = filesystem_page_resolution_concepts(path, content);
            if concepts.is_empty() {
                return None;
            }
            let has_transport_concept = concepts
                .iter()
                .any(|concept| matches!(concept.as_str(), "page" | "route" | "action"));
            if !has_transport_concept {
                return None;
            }
            let score = filesystem_page_resolution_score(content);
            if score < filesystem_page_resolution_minimum_score(&roles) {
                return None;
            }
            let centrality = bottleneck_by_path.get(path).copied().unwrap_or_default();
            let primary_bonus = filesystem_page_resolution_primary_bonus(&roles, content);
            Some((
                path.clone(),
                roles,
                concepts,
                score,
                centrality,
                primary_bonus,
            ))
        })
        .collect::<Vec<_>>();

    if ranked_files.len() < 2 {
        return Vec::new();
    }

    let role_set = ranked_files
        .iter()
        .flat_map(|(_, roles, _, _, _, _)| roles.iter().cloned())
        .collect::<BTreeSet<_>>();
    let concept_set = ranked_files
        .iter()
        .flat_map(|(_, _, concepts, _, _, _)| concepts.iter().cloned())
        .collect::<BTreeSet<_>>();
    let total_score = ranked_files
        .iter()
        .map(|(_, _, _, score, _, _)| *score)
        .sum::<usize>();

    let has_registry_or_resolver = role_set
        .iter()
        .any(|role| matches!(role.as_str(), "registry" | "resolver"));
    let has_route_surface = role_set
        .iter()
        .any(|role| matches!(role.as_str(), "route" | "router" | "page" | "action"));
    let has_page_and_route_concepts = concept_set.contains("page")
        && (concept_set.contains("route") || concept_set.contains("action"));
    let has_module_or_fallback = concept_set.contains("module")
        || concept_set.contains("fallback")
        || concept_set.contains("inertia");

    if !has_registry_or_resolver
        || !has_route_surface
        || !has_page_and_route_concepts
        || !has_module_or_fallback
        || total_score < 8
    {
        return Vec::new();
    }

    ranked_files.sort_by(|left, right| {
        right
            .4
            .cmp(&left.4)
            .then(right.5.cmp(&left.5))
            .then(right.3.cmp(&left.3))
            .then(right.1.len().cmp(&left.1.len()))
            .then(left.0.cmp(&right.0))
    });

    let Some((primary_file, _, _, primary_score, primary_centrality, _)) = ranked_files.first()
    else {
        return Vec::new();
    };

    let related_file_paths = ranked_files
        .iter()
        .skip(1)
        .take(6)
        .map(|(path, _, _, _, _, _)| path.clone())
        .collect::<Vec<_>>();
    let mut related_identifiers = vec![String::from("concern:page_resolution")];
    related_identifiers.extend(role_set.iter().take(6).map(|role| format!("role:{role}")));
    related_identifiers.extend(
        concept_set
            .iter()
            .take(8)
            .map(|concept| format!("concept:{concept}")),
    );
    let mut warning_families = role_set
        .iter()
        .map(|role| format!("parsing_role:{role}"))
        .collect::<Vec<_>>();
    warning_families.push(String::from("concern:filesystem_page_resolution"));

    let severity = (440
        + (role_set.len().min(6) * 70)
        + (ranked_files.len().min(4) * 90)
        + (total_score.min(14) * 18)
        + ((*primary_centrality / 250).min(140) as usize))
        .min(1000) as u16;

    vec![ArchitecturalAssessmentFinding {
        kind: ArchitecturalAssessmentKind::HandRolledParsing,
        file_path: primary_file.clone(),
        related_file_paths,
        related_identifiers,
        warning_count: ranked_files.len(),
        warning_weight: total_score.max(*primary_score),
        bottleneck_centrality_millis: *primary_centrality,
        warning_families,
        severity_millis: severity,
        pressure_path: Vec::new(),
        expensive_operation_sites: Vec::new(),
        expensive_operation_flow: Vec::new(),
        fingerprint: String::new(),
    }]
}

fn detect_manifest_backed_policy_engine_stacks(
    bottleneck_by_path: &HashMap<PathBuf, u32>,
    parsed_sources: &[(PathBuf, String)],
) -> Vec<ArchitecturalAssessmentFinding> {
    let mut ranked_files = parsed_sources
        .iter()
        .filter_map(|(path, content)| {
            if is_low_signal_identity_path(path) {
                return None;
            }
            let roles = manifest_policy_engine_roles(path);
            if roles.is_empty() {
                return None;
            }
            let concepts = manifest_policy_engine_concepts(path, content);
            if !concepts.contains("template") {
                return None;
            }
            let score = manifest_policy_engine_score(content);
            if score < manifest_policy_engine_minimum_score(&roles) {
                return None;
            }
            let centrality = bottleneck_by_path.get(path).copied().unwrap_or_default();
            let primary_bonus = manifest_policy_engine_primary_bonus(&roles, content);
            Some((
                path.clone(),
                roles,
                concepts,
                score,
                centrality,
                primary_bonus,
            ))
        })
        .collect::<Vec<_>>();

    if ranked_files.len() < 3 {
        return Vec::new();
    }

    let role_set = ranked_files
        .iter()
        .flat_map(|(_, roles, _, _, _, _)| roles.iter().cloned())
        .collect::<BTreeSet<_>>();
    let concept_set = ranked_files
        .iter()
        .flat_map(|(_, _, concepts, _, _, _)| concepts.iter().cloned())
        .collect::<BTreeSet<_>>();
    let total_score = ranked_files
        .iter()
        .map(|(_, _, _, score, _, _)| *score)
        .sum::<usize>();

    let has_registry = role_set.contains("registry");
    let has_compiler = role_set.contains("compiler");
    let has_runtime = role_set
        .iter()
        .any(|role| matches!(role.as_str(), "instantiation" | "runtime" | "service"));
    let has_policy_and_manifest =
        concept_set.contains("policy") && concept_set.contains("manifest");
    let has_graph_runtime_shape = concept_set.contains("node")
        && (concept_set.contains("edge")
            || concept_set.contains("workflow")
            || concept_set.contains("instantiation"));

    if !has_registry
        || !has_compiler
        || !has_runtime
        || !has_policy_and_manifest
        || !has_graph_runtime_shape
        || total_score < 10
    {
        return Vec::new();
    }

    ranked_files.sort_by(|left, right| {
        right
            .5
            .cmp(&left.5)
            .then(right.4.cmp(&left.4))
            .then(right.3.cmp(&left.3))
            .then(right.1.len().cmp(&left.1.len()))
            .then(left.0.cmp(&right.0))
    });

    let Some((primary_file, _, _, primary_score, primary_centrality, _)) = ranked_files.first()
    else {
        return Vec::new();
    };

    let related_file_paths = ranked_files
        .iter()
        .skip(1)
        .take(6)
        .map(|(path, _, _, _, _, _)| path.clone())
        .collect::<Vec<_>>();
    let mut related_identifiers = vec![String::from("concern:policy_engine")];
    related_identifiers.extend(role_set.iter().take(6).map(|role| format!("role:{role}")));
    related_identifiers.extend(
        concept_set
            .iter()
            .take(6)
            .map(|concept| format!("concept:{concept}")),
    );
    let mut warning_families = role_set
        .iter()
        .map(|role| format!("parsing_role:{role}"))
        .collect::<Vec<_>>();
    warning_families.push(String::from("concern:manifest_backed_policy_engine"));

    let severity = (460
        + (role_set.len().min(6) * 70)
        + (ranked_files.len().min(4) * 90)
        + (total_score.min(16) * 16)
        + ((*primary_centrality / 250).min(140) as usize))
        .min(1000) as u16;

    vec![ArchitecturalAssessmentFinding {
        kind: ArchitecturalAssessmentKind::HandRolledParsing,
        file_path: primary_file.clone(),
        related_file_paths,
        related_identifiers,
        warning_count: ranked_files.len(),
        warning_weight: total_score.max(*primary_score),
        bottleneck_centrality_millis: *primary_centrality,
        warning_families,
        severity_millis: severity,
        pressure_path: Vec::new(),
        expensive_operation_sites: Vec::new(),
        expensive_operation_flow: Vec::new(),
        fingerprint: String::new(),
    }]
}

fn sanctioned_config_markers(content: &str) -> Vec<String> {
    let normalized = content.to_ascii_lowercase();
    let mut markers = Vec::new();
    if normalized.contains("config(") || normalized.contains("config::") {
        markers.push(String::from("config_access"));
    }
    if normalized.contains("settings.") || normalized.contains("django.conf import settings") {
        markers.push(String::from("settings_access"));
    }
    if normalized.contains("get_config")
        || normalized.contains("load_config")
        || normalized.contains("configuration")
    {
        markers.push(String::from("configuration_service"));
    }
    markers
}

fn sanctioned_dependency_markers(content: &str) -> Vec<String> {
    let normalized = content.to_ascii_lowercase();
    let mut markers = Vec::new();
    if normalized.contains("__construct(")
        || normalized.contains("constructor(")
        || normalized.contains("fn new(")
    {
        markers.push(String::from("constructor_injection"));
    }
    if normalized.contains("bind(")
        || normalized.contains("singleton(")
        || normalized.contains("scoped(")
        || normalized.contains("register(")
        || normalized.contains("serviceprovider")
        || normalized.contains("provide(")
    {
        markers.push(String::from("provider_binding"));
    }
    markers
}

fn is_configuration_boundary_path(path: &Path) -> bool {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    normalized.contains("/config/")
        || normalized.contains("/settings/")
        || normalized.ends_with("/settings.py")
        || normalized.ends_with("/wp-config.php")
        || normalized.ends_with("/config.php")
}

fn is_dependency_boundary_path(path: &Path) -> bool {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    normalized.contains("/providers/")
        || normalized.contains("/bootstrap/")
        || normalized.contains("/config/")
        || normalized.contains("/settings/")
        || normalized.contains("/initializers/")
        || normalized.contains("/dependencyinjection/")
        || normalized.contains("/container/")
        || normalized.ends_with("serviceprovider.php")
        || normalized.ends_with("/container.php")
        || normalized.ends_with("/container.rs")
}

/// Generic dispatch-mechanism classifier. The vocabulary of what counts as a
/// lifecycle hook / event bus / queue job / notification is framework-specific
/// data owned by `scanners::framework_catalogs`, gated by language so a framework's
/// idioms never match a foreign-language file. This keeps the `DuplicateMechanism`
/// engine (grouping by concept, family-set thresholds, scoring) framework-agnostic.
fn detect_mechanism_families(path: &Path, content: &str) -> BTreeSet<String> {
    let mut families = BTreeSet::new();
    let catalogs = dispatch_mechanism_catalogs_for_file(path, content);
    if catalogs.is_empty() {
        return families;
    }
    let normalized_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let normalized_content = content.to_ascii_lowercase();

    for catalog in catalogs {
        for marker in catalog.markers {
            let haystack = match marker.kind {
                MechanismMarkerKind::PathContains => &normalized_path,
                MechanismMarkerKind::ContentContains => &normalized_content,
            };
            if haystack.contains(marker.needle) {
                families.insert(marker.family.to_string());
            }
        }
        if let Some(pattern) = catalog.notification_pattern {
            if pattern().is_match(&normalized_content) {
                families.insert(String::from("direct_notifications"));
            }
        }
    }

    families
}

fn duplicate_mechanism_tokens(path: &Path, content: &str) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    for segment in path.iter() {
        let segment = segment.to_string_lossy();
        tokens.extend(duplicate_mechanism_concepts_from_words(
            split_identifier_words(segment.as_ref()),
        ));
    }
    for identifier in identifier_pattern().find_iter(content).map(|m| m.as_str()) {
        if identifier.contains('_').not()
            && identifier.chars().any(|ch| ch.is_ascii_uppercase()).not()
        {
            continue;
        }
        tokens.extend(duplicate_mechanism_concepts_from_words(
            split_identifier_words(identifier),
        ));
    }
    tokens
}

fn abstraction_roles(path: &Path, _content: &str) -> BTreeSet<String> {
    let mut roles = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        let words = split_identifier_words(stem);
        roles.extend(words.into_iter().filter(|word| is_abstraction_role(word)));
    }
    roles
}

fn abstraction_sprawl_concepts(path: &Path, _content: &str) -> BTreeSet<String> {
    let mut concepts = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        concepts.extend(abstraction_concepts_from_words(split_identifier_words(
            stem,
        )));
    }
    if let Some(parent) = path.parent().and_then(|parent| parent.file_name()) {
        if let Some(parent) = parent.to_str() {
            concepts.extend(abstraction_concepts_from_words(split_identifier_words(
                parent,
            )));
        }
    }
    concepts
}

fn abstraction_concepts_from_words(words: Vec<String>) -> BTreeSet<String> {
    let filtered = words
        .into_iter()
        .filter(|word| !is_abstraction_role(word))
        .filter(|word| is_abstraction_concept_word(word))
        .collect::<Vec<_>>();
    let mut concepts = BTreeSet::new();
    if let Some(word) = filtered.first() {
        if word.len() >= 7 {
            concepts.insert(word.clone());
        }
    }
    concepts
}

fn parsing_roles(path: &Path) -> BTreeSet<String> {
    let mut roles = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        roles.extend(
            split_identifier_words(stem)
                .into_iter()
                .filter(|word| is_parsing_role(word)),
        );
    }
    roles
}

fn parsing_primitive_score(content: &str) -> usize {
    parsing_primitive_pattern().find_iter(content).count()
}

fn parsing_role_minimum_score(roles: &BTreeSet<String>) -> usize {
    if roles.iter().any(|role| is_primary_parsing_role(role)) {
        3
    } else if roles.iter().all(|role| role != "validator")
        && roles
            .iter()
            .any(|role| matches!(role.as_str(), "definition" | "resolver" | "normalizer"))
    {
        1
    } else {
        2
    }
}

fn is_parsing_role(word: &str) -> bool {
    matches!(
        word,
        "parser"
            | "tokenizer"
            | "lexer"
            | "scanner"
            | "matcher"
            | "validator"
            | "normalizer"
            | "decoder"
            | "encoder"
            | "resolver"
            | "definition"
    )
}

fn is_primary_parsing_role(word: &str) -> bool {
    matches!(word, "parser" | "tokenizer" | "lexer" | "scanner")
}

fn is_contract_stack_role(word: &str) -> bool {
    matches!(
        word,
        "validator" | "resolver" | "definition" | "normalizer" | "matcher"
    )
}

fn scheduler_stack_roles(path: &Path) -> BTreeSet<String> {
    let mut roles = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        roles.extend(split_identifier_words(stem).into_iter().filter(|word| {
            matches!(
                word.as_str(),
                "registry" | "executor" | "scheduler" | "command" | "controller" | "runner"
            )
        }));
    }
    roles
}

fn scheduler_stack_score(content: &str) -> usize {
    scheduler_stack_pattern().find_iter(content).count()
}

fn scheduler_stack_minimum_score(roles: &BTreeSet<String>) -> usize {
    if roles.iter().any(|role| {
        matches!(
            role.as_str(),
            "executor" | "scheduler" | "command" | "runner"
        )
    }) {
        2
    } else {
        1
    }
}

fn scheduler_stack_primary_bonus(roles: &BTreeSet<String>) -> usize {
    roles
        .iter()
        .filter(|role| {
            matches!(
                role.as_str(),
                "executor" | "scheduler" | "command" | "runner"
            )
        })
        .count()
}

fn scheduler_stack_concepts(path: &Path, content: &str) -> BTreeSet<String> {
    let mut concepts = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        concepts.extend(
            split_identifier_words(stem)
                .into_iter()
                .filter(|word| matches!(word.as_str(), "job" | "schedule" | "scheduled" | "cron")),
        );
    }
    let normalized = content.to_ascii_lowercase();
    if normalized.contains("scheduled job") || normalized.contains("scheduled-job") {
        concepts.insert(String::from("scheduled"));
        concepts.insert(String::from("job"));
    }
    if normalized.contains("cronexpression") || normalized.contains("cron") {
        concepts.insert(String::from("cron"));
    }
    if normalized.contains("schedule:run") || normalized.contains("scheduling") {
        concepts.insert(String::from("schedule"));
    }
    concepts
}

fn scheduler_stack_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(
            r#"(?i)cronexpression|schedule:run|scheduled_jobs|scheduled[- ]job|config\('jobs|config\("jobs|manifest\['jobs'\]|artisan::call|dispatch\s*\(\s*new\s+[A-Za-z_\\]|cache::lock|->isdue\("#,
        )
        .expect("scheduler stack pattern should compile")
    })
}

fn filesystem_page_resolution_roles(path: &Path) -> BTreeSet<String> {
    let mut roles = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        roles.extend(split_identifier_words(stem).into_iter().filter(|word| {
            matches!(
                word.as_str(),
                "resolver" | "registry" | "route" | "router" | "page" | "action"
            )
        }));
    }
    roles
}

fn filesystem_page_resolution_score(content: &str) -> usize {
    filesystem_page_resolution_pattern()
        .find_iter(content)
        .count()
}

fn filesystem_page_resolution_minimum_score(roles: &BTreeSet<String>) -> usize {
    if roles
        .iter()
        .any(|role| matches!(role.as_str(), "resolver" | "registry"))
    {
        3
    } else {
        2
    }
}

fn filesystem_page_resolution_primary_bonus(roles: &BTreeSet<String>, content: &str) -> usize {
    let mut bonus = roles
        .iter()
        .filter(|role| matches!(role.as_str(), "resolver" | "registry"))
        .count();
    let normalized = content.to_ascii_lowercase();
    if normalized.contains("file::allfiles")
        || normalized.contains("file::exists")
        || normalized.contains("resources/js/pages")
    {
        bonus += 1;
    }
    bonus
}

fn filesystem_page_resolution_concepts(path: &Path, content: &str) -> BTreeSet<String> {
    let mut concepts = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        concepts.extend(split_identifier_words(stem).into_iter().filter(|word| {
            matches!(
                word.as_str(),
                "page" | "route" | "module" | "action" | "fallback"
            )
        }));
    }
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    if normalized.contains("/pages/") || normalized.ends_with(".page.php") {
        concepts.insert(String::from("page"));
    }
    if normalized.contains("/actions/") {
        concepts.insert(String::from("action"));
    }
    let normalized_content = content.to_ascii_lowercase();
    if normalized_content.contains("@route") || normalized_content.contains("route::match") {
        concepts.insert(String::from("route"));
    }
    if normalized_content.contains("module page")
        || normalized_content.contains("app/modules/")
        || normalized_content.contains("getenabledmodules")
    {
        concepts.insert(String::from("module"));
    }
    if normalized_content.contains("fallback") {
        concepts.insert(String::from("fallback"));
    }
    if normalized_content.contains("inertia") || normalized_content.contains("resources/js/pages") {
        concepts.insert(String::from("inertia"));
        concepts.insert(String::from("page"));
    }
    concepts
}

fn filesystem_page_resolution_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(
            r#"(?i)(file::allfiles|file::exists|base_path\s*\(|storage_path\s*\(|resources/js/pages|\.page\.php|@route\b|route::match|parseroute|parsepagefile|parseactionfile|extractclassinfo|require_once|fallback|module page|action route|page route)"#,
        )
        .expect("filesystem page resolution pattern should compile")
    })
}

fn manifest_policy_engine_roles(path: &Path) -> BTreeSet<String> {
    let mut roles = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        roles.extend(split_identifier_words(stem).into_iter().filter(|word| {
            matches!(
                word.as_str(),
                "template"
                    | "policy"
                    | "registry"
                    | "compiler"
                    | "instantiation"
                    | "runtime"
                    | "resolver"
                    | "service"
            )
        }));
    }
    roles
}

fn manifest_policy_engine_score(content: &str) -> usize {
    manifest_policy_engine_pattern().find_iter(content).count()
}

fn manifest_policy_engine_minimum_score(roles: &BTreeSet<String>) -> usize {
    if roles.iter().any(|role| {
        matches!(
            role.as_str(),
            "registry" | "compiler" | "instantiation" | "runtime"
        )
    }) {
        2
    } else {
        1
    }
}

fn manifest_policy_engine_primary_bonus(roles: &BTreeSet<String>, content: &str) -> usize {
    let mut bonus = roles
        .iter()
        .filter(|role| {
            matches!(
                role.as_str(),
                "registry" | "compiler" | "instantiation" | "runtime"
            )
        })
        .count();
    let normalized = content.to_ascii_lowercase();
    if normalized.contains("templatepolicies")
        || normalized.contains("assignmentresolvers")
        || normalized.contains("deadline resolvers")
        || normalized.contains("resolveassignmentresolver")
        || normalized.contains("resolvedeadlineresolver")
    {
        bonus += 2;
    }
    if normalized.contains("topologicalsort")
        || normalized.contains("normalizenodes")
        || normalized.contains("normalizeedges")
        || normalized.contains("applyassignmentpolicy")
        || normalized.contains("applydeadlinepolicy")
    {
        bonus += 1;
    }
    bonus
}

fn manifest_policy_engine_concepts(path: &Path, content: &str) -> BTreeSet<String> {
    let mut concepts = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        concepts.extend(split_identifier_words(stem).into_iter().filter(|word| {
            matches!(
                word.as_str(),
                "template"
                    | "policy"
                    | "manifest"
                    | "node"
                    | "edge"
                    | "workflow"
                    | "instantiation"
                    | "resolver"
            )
        }));
    }
    let normalized_content = content.to_ascii_lowercase();
    if normalized_content.contains("templatepolicies")
        || normalized_content.contains("manifest['templatepolicies']")
        || normalized_content.contains("manifest[\"templatepolicies\"]")
        || normalized_content.contains("moduleregistry")
    {
        concepts.insert(String::from("manifest"));
        concepts.insert(String::from("policy"));
        concepts.insert(String::from("template"));
    }
    if normalized_content.contains("nodesbykey")
        || normalized_content.contains("normalizenodes")
        || normalized_content.contains("nodekey")
    {
        concepts.insert(String::from("node"));
    }
    if normalized_content.contains("normalizeedges")
        || normalized_content.contains("deferrededges")
        || normalized_content.contains("edge references")
    {
        concepts.insert(String::from("edge"));
    }
    if normalized_content.contains("topologicalsort")
        || normalized_content.contains("dependson")
        || normalized_content.contains("instantiate(")
    {
        concepts.insert(String::from("workflow"));
        concepts.insert(String::from("instantiation"));
    }
    concepts
}

fn manifest_policy_engine_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(
            r#"(?i)(templatePolicies|assignmentResolvers|deadlineResolvers|resolveAssignmentResolver|resolveDeadlineResolver|normalizeNodes|normalizeEdges|topologicalSort|deferredEdges|dependsOn|applyAssignmentPolicy|applyDeadlinePolicy|extractPolicy|createdByNode|nodeKey|moduleRegistry->getEnabledModules|manifest\['templatePolicies'\]|manifest\["templatePolicies"\])"#,
        )
        .expect("manifest policy engine pattern should compile")
    })
}

fn parsing_concepts(path: &Path) -> BTreeSet<String> {
    let mut concepts = BTreeSet::new();
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        concepts.extend(parsing_concepts_from_words(split_identifier_words(stem)));
    }
    if let Some(parent) = path.parent().and_then(|parent| parent.file_name()) {
        if let Some(parent) = parent.to_str() {
            concepts.extend(parsing_concepts_from_words(split_identifier_words(parent)));
        }
    }
    concepts
}

fn parsing_concepts_from_words(words: Vec<String>) -> BTreeSet<String> {
    words
        .into_iter()
        .filter(|word| {
            matches!(
                word.as_str(),
                "filter"
                    | "query"
                    | "contract"
                    | "definition"
                    | "schema"
                    | "criteria"
                    | "sort"
                    | "order"
                    | "rule"
                    | "preference"
            )
        })
        .collect()
}

fn is_abstraction_role(word: &str) -> bool {
    matches!(
        word,
        "service"
            | "manager"
            | "helper"
            | "provider"
            | "factory"
            | "adapter"
            | "resolver"
            | "registry"
            | "builder"
            | "gateway"
            | "normalizer"
            | "mapper"
            | "wrapper"
            | "orchestrator"
            | "dispatcher"
            | "compiler"
            | "validator"
            | "loader"
            | "handler"
            | "client"
            | "planner"
            | "router"
            | "broadcaster"
            | "executor"
            | "store"
            | "repository"
            | "policy"
    )
}

fn is_abstraction_concept_word(word: &str) -> bool {
    word.len() >= 4 && !is_generic_concept_stopword(word) && !is_structural_layer_word(word)
}

/// Non-domain filler words that are never a meaningful sprawl concept.
fn is_generic_concept_stopword(word: &str) -> bool {
    matches!(
        word,
        "abstract"
            | "default"
            | "global"
            | "common"
            | "system"
            | "value"
            | "field"
            | "fields"
            | "types"
            | "type"
            | "data"
            | "core"
            | "base"
            | "module"
            | "modules"
            | "view"
            | "views"
    )
}

/// Structural/layer directory words (universal OOP/web-app layering vocabulary,
/// not framework-specific) and role plurals. A concept is extracted from file
/// stems *and parent directory names*, so a layer directory like `Services/`,
/// `Support/`, or `components/` would otherwise become a fake "concept" that
/// groups architecturally-unrelated files sharing only that layer, guaranteeing
/// a 4+ role spread in any large codebase. Role singulars are already filtered
/// upstream via `is_abstraction_role`; this covers the plural/directory forms.
fn is_structural_layer_word(word: &str) -> bool {
    matches!(
        word,
        "support"
            | "supports"
            | "http"
            | "console"
            | "middleware"
            | "resources"
            | "resource"
            | "foundation"
            | "shared"
            | "components"
            | "component"
            | "integrations"
            | "integration"
            | "actions"
            | "action"
            | "requests"
            | "responses"
            | "contracts"
            | "concerns"
            | "interfaces"
            | "interface"
            // role plurals (singulars handled by is_abstraction_role)
            | "services"
            | "managers"
            | "helpers"
            | "providers"
            | "factories"
            | "adapters"
            | "resolvers"
            | "registries"
            | "builders"
            | "gateways"
            | "normalizers"
            | "mappers"
            | "wrappers"
            | "orchestrators"
            | "dispatchers"
            | "compilers"
            | "validators"
            | "loaders"
            | "handlers"
            | "clients"
            | "planners"
            | "routers"
            | "broadcasters"
            | "executors"
            | "stores"
            | "repositories"
            | "policies"
    )
}

fn duplicate_mechanism_concepts_from_words(words: Vec<String>) -> BTreeSet<String> {
    let filtered = words
        .into_iter()
        .filter(|word| is_duplicate_mechanism_concept_word(word))
        .collect::<Vec<_>>();
    filtered
        .windows(2)
        .filter_map(|window| match window {
            [left, right] => Some(format!("{left}_{right}")),
            _ => None,
        })
        .collect()
}

fn is_duplicate_mechanism_concept_word(token: &str) -> bool {
    token.len() >= 5
        && !matches!(
            token,
            "class"
                | "trait"
                | "module"
                | "service"
                | "manager"
                | "helper"
                | "util"
                | "utils"
                | "create"
                | "after"
                | "before"
                | "construct"
                | "method"
                | "function"
                | "namespace"
                | "readonly"
                | "object"
                | "scope"
                | "services"
                | "channel"
                | "channels"
                | "handler"
                | "listener"
                | "event"
                | "signal"
                | "hook"
                | "hooks"
                | "queue"
                | "queues"
                | "job"
                | "jobs"
                | "dispatch"
                | "mailer"
                | "mail"
                | "provider"
                | "providers"
                | "controller"
                | "model"
                | "entity"
                | "entities"
                | "resource"
                | "resources"
                | "default"
                | "public"
                | "private"
                | "static"
                | "return"
                | "string"
                | "array"
                | "value"
                | "false"
                | "true"
                | "final"
                | "admin"
                | "index"
                | "other"
                | "else"
                | "their"
                | "there"
                | "where"
                | "which"
                | "would"
                | "could"
                | "should"
                | "about"
                | "afterwards"
                | "beforehand"
        )
}

fn duplicate_mechanism_anchor_rank(path: &Path) -> u8 {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    if normalized.contains("/hooks/")
        || normalized.contains("/listeners/")
        || normalized.contains("/jobs/")
    {
        return 0;
    }
    2
}

fn identifier_pattern() -> &'static Regex {
    static IDENTIFIER_PATTERN: OnceLock<Regex> = OnceLock::new();
    IDENTIFIER_PATTERN.get_or_init(|| Regex::new(r"\b[A-Za-z_][A-Za-z0-9_]{4,}\b").unwrap())
}

fn compatibility_keyword_pattern() -> &'static Regex {
    static COMPATIBILITY_KEYWORD_PATTERN: OnceLock<Regex> = OnceLock::new();
    COMPATIBILITY_KEYWORD_PATTERN.get_or_init(|| {
        Regex::new(
            r"(?i)\b(alias|aliases|fallback|legacy|compat|compatibility|canonical|normalize|normalized|normalizer|resolver|mapping|mapper|translate|adapter)\b",
        )
        .unwrap()
    })
}

fn parsing_primitive_pattern() -> &'static Regex {
    static PARSING_PRIMITIVE_PATTERN: OnceLock<Regex> = OnceLock::new();
    PARSING_PRIMITIVE_PATTERN.get_or_init(|| {
        Regex::new(
            r#"(?i)(json_decode|json_encode|preg_match|preg_replace|preg_split|explode\s*\(|split\s*\(|trim\s*\(|substr\s*\(|str_starts_with\s*\(|str_ends_with\s*\(|parse[A-Z][A-Za-z0-9_]*\s*\(|tokeni[sz]e\s*\(|regex|pattern\b|matcher\b|validate[A-Z][A-Za-z0-9_]*\s*\(|\$schema\s*\[\s*['"](type|enum|required|properties|items|nullable|minlength|maxlength|min|max)['"]\s*\]|['"](type|enum|required|properties|items|nullable|minlength|maxlength|min|max)['"]\s*=>|assert[A-Z][A-Za-z0-9_]*Schema\s*\(|array_key_exists\s*\()"#,
        )
        .unwrap()
    })
}

fn classify_identity_variant(identifier: &str) -> Option<(String, IdentityVariantKind)> {
    let mut words = split_identifier_words(identifier);
    strip_accessor_prefixes(&mut words);
    if words.len() < 2 {
        return None;
    }

    let variant_kind = match words.last().map(String::as_str) {
        Some("id" | "ids" | "uuid" | "uuids" | "guid" | "guids") => {
            words.pop();
            IdentityVariantKind::Identity
        }
        _ => IdentityVariantKind::Base,
    };
    if words.len() < 2 {
        return None;
    }

    let stem = words.join("_");
    (stem.len() >= 8).then_some((stem, variant_kind))
}

fn split_identifier_words(identifier: &str) -> Vec<String> {
    if identifier.contains('_') {
        return identifier
            .split('_')
            .filter(|part| part.is_empty().not())
            .map(|part| part.to_ascii_lowercase())
            .collect();
    }

    let chars = identifier.chars().collect::<Vec<_>>();
    let mut words = Vec::new();
    let mut current = String::new();
    for (index, ch) in chars.iter().enumerate() {
        let boundary = index > 0
            && ch.is_ascii_uppercase()
            && (chars[index - 1].is_ascii_lowercase()
                || chars
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_lowercase()));
        if boundary && current.is_empty().not() {
            words.push(current.to_ascii_lowercase());
            current.clear();
        }
        if ch.is_ascii_alphanumeric() {
            current.push(*ch);
        }
    }
    if current.is_empty().not() {
        words.push(current.to_ascii_lowercase());
    }
    words
}

fn strip_accessor_prefixes(words: &mut Vec<String>) {
    while words.len() > 2
        && words
            .first()
            .is_some_and(|prefix| matches!(prefix.as_str(), "get" | "set" | "has" | "with"))
    {
        words.remove(0);
    }
}

fn has_accessor_prefix(identifier: &str) -> bool {
    let words = split_identifier_words(identifier);
    words
        .first()
        .is_some_and(|prefix| matches!(prefix.as_str(), "get" | "set" | "has" | "with"))
}

fn identifier_style(identifier: &str) -> IdentifierStyle {
    if identifier.contains('_') {
        return IdentifierStyle::Snake;
    }
    let Some(first) = identifier.chars().next() else {
        return IdentifierStyle::Other;
    };
    if first.is_ascii_uppercase() {
        return IdentifierStyle::Pascal;
    }
    if identifier.chars().any(|ch| ch.is_ascii_uppercase()) {
        return IdentifierStyle::Camel;
    }
    IdentifierStyle::Other
}

fn is_low_signal_identity_path(path: &Path) -> bool {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    [
        "/test/",
        "/tests/",
        "/__tests__/",
        "/spec/",
        "/specs/",
        "/migrations/",
        "/migration/",
        "/fixtures/",
        "/seeders/",
        "/factories/",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
        || normalized.starts_with("test/")
        || normalized.starts_with("tests/")
        || normalized.contains(".test.")
        || normalized.contains(".spec.")
}

#[cfg(test)]
mod tests {
    use super::{
        build_architectural_assessment, build_architectural_assessment_with_ast_grep_and_graph,
        detect_mechanism_families, ArchitecturalAssessmentKind,
    };
    use crate::detectors::dead_code::{
        DeadCodeCategory, DeadCodeFinding, DeadCodeProofTier, DeadCodeResult,
    };
    use crate::detectors::hardwiring::{HardwiringCategory, HardwiringFinding, HardwiringResult};
    use crate::external::{
        ExternalAnalysisResult, ExternalConfidence, ExternalFinding, ExternalSeverity,
    };
    use crate::graph::analysis::{BottleneckFile, GraphAnalysis};
    use crate::graph::{RelationKind, SemanticGraph, SymbolKind, Visibility};
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use crate::scanners::ast_grep::run_ast_grep_scan;
    use serde_json::Map;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_warning_heavy_hotspots_from_centrality_and_findings() {
        let graph_analysis = GraphAnalysis {
            bottleneck_files: vec![BottleneckFile {
                file_path: PathBuf::from("src/service.ts"),
                centrality_millis: 800,
                fingerprint: String::new(),
            }],
            ..GraphAnalysis::default()
        };
        let dead_code = DeadCodeResult {
            findings: vec![DeadCodeFinding {
                category: DeadCodeCategory::UnusedPrivateFunction,
                symbol_id: String::from("a"),
                file_path: PathBuf::from("src/service.ts"),
                name: String::from("unused"),
                line: 10,
                proof_tier: DeadCodeProofTier::Strong,
                fingerprint: String::new(),
            }],
        };
        let hardwiring = HardwiringResult {
            findings: vec![
                HardwiringFinding {
                    category: HardwiringCategory::EnvOutsideConfig,
                    file_path: PathBuf::from("src/service.ts"),
                    line: 11,
                    value: String::from("env"),
                    context: String::from("process.env.APP_ENV"),
                    fingerprint: String::new(),
                },
                HardwiringFinding {
                    category: HardwiringCategory::RepeatedLiteral,
                    file_path: PathBuf::from("src/service.ts"),
                    line: 12,
                    value: String::from("shared"),
                    context: String::from("shared"),
                    fingerprint: String::new(),
                },
            ],
        };
        let external_analysis = ExternalAnalysisResult {
            findings: vec![ExternalFinding {
                tool: String::from("ruff"),
                domain: String::from("security"),
                category: String::from("sast"),
                rule_id: String::from("S123"),
                severity: ExternalSeverity::Medium,
                confidence: ExternalConfidence::High,
                file_path: Some(PathBuf::from("src/service.ts")),
                line: Some(13),
                locations: Vec::new(),
                message: String::from("issue"),
                fingerprint: String::from("fp"),
                extras: Map::new(),
            }],
            ..ExternalAnalysisResult::default()
        };

        let assessment = build_architectural_assessment(
            &graph_analysis,
            &dead_code,
            &hardwiring,
            &external_analysis,
            &[],
        );

        assert_eq!(assessment.findings.len(), 1);
        assert_eq!(
            assessment.findings[0].kind,
            ArchitecturalAssessmentKind::WarningHeavyHotspot
        );
        assert_eq!(assessment.findings[0].warning_count, 4);
        assert!(assessment.findings[0]
            .warning_families
            .contains(&String::from("dead_code")));
        assert!(assessment.findings[0]
            .warning_families
            .contains(&String::from("hardwiring")));
        assert!(assessment.findings[0]
            .warning_families
            .contains(&String::from("external")));
    }

    #[test]
    fn skips_non_central_files_even_when_noisy() {
        let graph_analysis = GraphAnalysis {
            bottleneck_files: vec![BottleneckFile {
                file_path: PathBuf::from("src/noisy.ts"),
                centrality_millis: 120,
                fingerprint: String::new(),
            }],
            ..GraphAnalysis::default()
        };
        let dead_code = DeadCodeResult {
            findings: vec![
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("a"),
                    file_path: PathBuf::from("src/noisy.ts"),
                    name: String::from("unused"),
                    line: 10,
                    proof_tier: DeadCodeProofTier::Strong,
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("b"),
                    file_path: PathBuf::from("src/noisy.ts"),
                    name: String::from("unused2"),
                    line: 11,
                    proof_tier: DeadCodeProofTier::Strong,
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("c"),
                    file_path: PathBuf::from("src/noisy.ts"),
                    name: String::from("unused3"),
                    line: 12,
                    proof_tier: DeadCodeProofTier::Strong,
                    fingerprint: String::new(),
                },
            ],
        };

        let assessment = build_architectural_assessment(
            &graph_analysis,
            &dead_code,
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[],
        );

        assert!(assessment.findings.is_empty());
    }

    #[test]
    fn skips_single_family_noise_on_central_files() {
        let graph_analysis = GraphAnalysis {
            bottleneck_files: vec![BottleneckFile {
                file_path: PathBuf::from("src/busy.ts"),
                centrality_millis: 800,
                fingerprint: String::new(),
            }],
            ..GraphAnalysis::default()
        };
        let dead_code = DeadCodeResult {
            findings: vec![
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("a"),
                    file_path: PathBuf::from("src/busy.ts"),
                    name: String::from("unused"),
                    line: 10,
                    proof_tier: DeadCodeProofTier::Strong,
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("b"),
                    file_path: PathBuf::from("src/busy.ts"),
                    name: String::from("unused2"),
                    line: 11,
                    proof_tier: DeadCodeProofTier::Strong,
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("c"),
                    file_path: PathBuf::from("src/busy.ts"),
                    name: String::from("unused3"),
                    line: 12,
                    proof_tier: DeadCodeProofTier::Strong,
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("d"),
                    file_path: PathBuf::from("src/busy.ts"),
                    name: String::from("unused4"),
                    line: 13,
                    proof_tier: DeadCodeProofTier::Strong,
                    fingerprint: String::new(),
                },
            ],
        };

        let assessment = build_architectural_assessment(
            &graph_analysis,
            &dead_code,
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[],
        );

        assert!(assessment.findings.is_empty());
    }

    #[test]
    fn detects_split_identity_models_from_source_identifiers() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Services/Assignments.php"),
                    String::from(
                        r#"
                        $assignedUser = $entity['assignedUser'] ?? null;
                        $assignedUserId = $entity['assignedUserId'] ?? null;
                        if ($assignedUserId !== null) {
                            return $this->getAssignedUserId();
                        }
                        return $assignedUser;
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Support/Assignments.php"),
                    String::from(
                        r#"
                        $assigned_user_id = $assignedUserId;
                        $payload['assigned_user_id'] = $assigned_user_id;
                    "#,
                    ),
                ),
            ],
        );

        let split = assessment
            .findings
            .iter()
            .find(|finding| finding.kind == ArchitecturalAssessmentKind::SplitIdentityModel)
            .expect("expected split identity model finding");
        assert_eq!(
            split.file_path,
            PathBuf::from("app/Services/Assignments.php")
        );
        assert!(split
            .related_identifiers
            .contains(&String::from("assignedUser")));
        assert!(split
            .related_identifiers
            .contains(&String::from("assignedUserId")));
    }

    #[test]
    fn snake_case_only_in_sql_strings_is_not_split_identity() {
        // The snake_case spelling appears only as a database column inside a SQL
        // heredoc and a quoted query-builder argument, while the camelCase is the
        // PHP entity property. This is the normal ORM boundary and must not be
        // reported as a split-identity model.
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Services/BusinessUnitBudgetManager.php"),
                    String::from(
                        r#"<?php
class BusinessUnitBudgetManager {
    public function load(array $options): array {
        $businessUnitId = (int) $options['businessUnitId'];
        $rows = $this->db
            ->where('business_unit_id', $businessUnitId)
            ->columns(['business_unit_id', 'amount'])
            ->all();
        $sql = <<<SQL
            SELECT amount FROM budgets
            WHERE business_unit_id = ?
        SQL;
        return $this->run($sql, [$businessUnitId]);
    }
}
"#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/BusinessUnitReportService.php"),
                    String::from(
                        r#"<?php
class BusinessUnitReportService {
    public function report(int $businessUnitId): array {
        return $this->finder->forBusinessUnitId($businessUnitId);
    }
}
"#,
                    ),
                ),
            ],
        );

        assert!(
            !assessment.findings.iter().any(|finding| {
                finding.kind == ArchitecturalAssessmentKind::SplitIdentityModel
                    && finding
                        .warning_families
                        .iter()
                        .any(|family| family.contains("business_unit"))
            }),
            "snake DB column in SQL strings must not trigger split identity, got: {:?}",
            assessment.findings
        );
    }

    #[test]
    fn detects_compatibility_scars_when_multiple_split_concepts_accumulate_in_one_file() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Services/EntityNormalizer.php"),
                    centrality_millis: 700,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Services/EntityNormalizer.php"),
                    String::from(
                        r#"
                        $assignedUser = $payload['assignedUser'] ?? null;
                        $assigned_user_id = $payload['assigned_user_id'] ?? null;
                        $getAssignedUserId = $this->getAssignedUserId();
                        $createdBy = $payload['createdBy'] ?? null;
                        $created_by_id = $payload['created_by_id'] ?? null;
                        $getCreatedById = $this->getCreatedById();
                        $fallback = $assigned_user_id ?: $created_by_id;
                        $normalized = $this->normalizePayload($fallback);
                        $resolver = $this->mappingResolver($normalized);
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/Assignments.php"),
                    String::from("$assignedUser = $entity['assignedUser']; $assigned_user_id = 1;"),
                ),
                (
                    PathBuf::from("app/Services/Audit.php"),
                    String::from("$createdBy = $entity['createdBy']; $created_by_id = 1;"),
                ),
            ],
        );

        assert!(assessment.findings.iter().any(|finding| {
            finding.kind == ArchitecturalAssessmentKind::CompatibilityScar
                && finding.file_path == Path::new("app/Services/EntityNormalizer.php")
        }));
    }

    #[test]
    fn detects_duplicate_mechanisms_for_the_same_concern() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Notifications/AssignmentNotificationService.php"),
                    centrality_millis: 600,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Hooks/assignment_notifications.hook.php"),
                    String::from(
                        "return ['afterSave' => 'sendAssignmentNotification', 'beforeUpdate' => 'emitAssignmentChanged'];",
                    ),
                ),
                (
                    PathBuf::from("app/Listeners/AssignmentNotificationListener.php"),
                    String::from(
                        "final class AssignmentNotificationListener { public function handle(AssignmentChangedEvent $event): void { $this->publishAssignmentUpdate($event); } }",
                    ),
                ),
                (
                    PathBuf::from("app/Jobs/AssignmentNotificationJob.php"),
                    String::from(
                        "final class AssignmentNotificationJob implements ShouldQueue { public function handle(): void { self::dispatch($this->assignmentId)->onQueue('notifications'); Mail::send('assignment', []); } }",
                    ),
                ),
                (
                    PathBuf::from("app/Notifications/AssignmentNotificationService.php"),
                    String::from(
                        "final class AssignmentNotificationService { public function notifyAssignment(): void { Mail::send('assignment', []); $this->sendAssignmentEmail(); $this->queueAssignmentDigest(); } }",
                    ),
                ),
            ],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| finding.kind == ArchitecturalAssessmentKind::DuplicateMechanism)
            .expect("expected duplicate mechanism finding");
        assert!(
            finding.file_path == Path::new("app/Notifications/AssignmentNotificationService.php")
                || finding.file_path == Path::new("app/Hooks/assignment_notifications.hook.php")
                || finding.file_path == Path::new("app/Jobs/AssignmentNotificationJob.php")
        );
        assert!(finding
            .warning_families
            .contains(&String::from("mechanism:lifecycle_hooks")));
        assert!(finding
            .warning_families
            .contains(&String::from("mechanism:event_bus")));
        assert!(finding
            .warning_families
            .contains(&String::from("mechanism:queue_jobs")));
        assert!(finding
            .warning_families
            .contains(&String::from("mechanism:direct_notifications")));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier.starts_with("concept:assignment")));
        assert!(finding.related_file_paths.iter().any(|path| {
            path == &PathBuf::from("app/Notifications/AssignmentNotificationService.php")
                || path == &PathBuf::from("app/Hooks/assignment_notifications.hook.php")
                || path == &PathBuf::from("app/Jobs/AssignmentNotificationJob.php")
        }));
    }

    #[test]
    fn frontend_component_files_do_not_contribute_backend_mechanism_families() {
        // A Vue component with `defineEmits`/`emit`, a variable named `queued_files`,
        // and `addEventListener` would previously be tagged event_bus + queue_jobs.
        let families = detect_mechanism_families(
            Path::new("resources/js/components/ItemsGrid.vue"),
            r#"
                const emit = defineEmits(['update']);
                const queued_files = ref([]);
                window.addEventListener('resize', onResize);
                function onChange() { emit('update'); store.dispatch('items/refresh'); }
            "#,
        );
        assert!(
            families.is_empty(),
            "frontend component must contribute no backend mechanism families, got: {families:?}"
        );
    }

    #[test]
    fn bare_queue_and_event_vocabulary_is_not_a_backend_mechanism() {
        // Plain server-side file that merely mentions the words, with no real
        // dispatch syntax, must not be classified into any mechanism family.
        let families = detect_mechanism_families(
            Path::new("app/Services/OutboxService.php"),
            r#"
                final class OutboxService {
                    public function flush(): void {
                        $this->queuedFiles = [];   // drain the queue for this job
                        $this->emitProgress();
                    }
                }
            "#,
        );
        assert!(
            families.is_empty(),
            "bare queue/job/emit vocabulary must not classify a mechanism, got: {families:?}"
        );

        // Real dispatch syntax still classifies.
        let real = detect_mechanism_families(
            Path::new("app/Services/DispatchService.php"),
            "SendReport::dispatch($id); event(new ReportReady($id));",
        );
        assert!(real.contains("queue_jobs"), "got: {real:?}");
        assert!(real.contains("event_bus"), "got: {real:?}");
    }

    #[test]
    fn ignores_wordy_hook_files_without_real_notification_mechanism_calls() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("admin-filters.php"),
                String::from(
                    r#"
                    add_action('admin_init', 'wp_admin_headers');
                    add_filter('screen_options_show_submit', '__return_true');
                    // notification email preferences live elsewhere
                "#,
                ),
            )],
        );

        assert!(assessment
            .findings
            .iter()
            .all(|finding| finding.kind != ArchitecturalAssessmentKind::DuplicateMechanism));
    }

    #[test]
    fn detects_sanctioned_path_bypass_for_raw_env_outside_config_boundary() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Services/ReportService.py"),
                    centrality_millis: 520,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult {
                findings: vec![HardwiringFinding {
                    category: HardwiringCategory::EnvOutsideConfig,
                    file_path: PathBuf::from("app/Services/ReportService.py"),
                    line: 10,
                    value: String::from("APP_MODE"),
                    context: String::from("os.environ.get('APP_MODE')"),
                    fingerprint: String::new(),
                }],
            },
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/Services/ReportService.py"),
                String::from(
                    r#"
from django.conf import settings

def build_report():
    mode = os.environ.get("APP_MODE")
    timeout = settings.REPORT_TIMEOUT
    return mode, timeout
"#,
                ),
            )],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| finding.kind == ArchitecturalAssessmentKind::SanctionedPathBypass)
            .expect("expected sanctioned path bypass finding");
        assert_eq!(
            finding.file_path,
            PathBuf::from("app/Services/ReportService.py")
        );
        assert!(finding
            .related_identifiers
            .contains(&String::from("raw_env")));
        assert!(finding
            .warning_families
            .contains(&String::from("concern:configuration")));
    }

    #[test]
    fn ast_grep_detects_sanctioned_path_bypass_for_python_raw_env_when_native_hardwiring_is_silent()
    {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Services/ReportService.py"),
                    centrality_millis: 520,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/Services/ReportService.py"),
                String::from(
                    r#"
import os
from django.conf import settings

def build_report():
    mode = os.environ.get("APP_MODE")
    timeout = settings.REPORT_TIMEOUT
    return mode, timeout
"#,
                ),
            )],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| finding.kind == ArchitecturalAssessmentKind::SanctionedPathBypass)
            .expect("expected sanctioned path bypass finding");
        assert_eq!(finding.warning_count, 1);
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "scanner:ast_grep"));
        assert!(finding
            .related_identifiers
            .contains(&String::from("raw_env")));
    }

    #[test]
    fn ast_grep_detects_sanctioned_path_bypass_for_php_container_lookup_outside_boundary() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Services/ReportService.php"),
                    centrality_millis: 610,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/Services/ReportService.php"),
                String::from(
                    r#"
<?php

final class ReportService
{
    public function __construct(private TenantManager $tenantManager) {}

    public function build(): array
    {
        return app(TenantManager::class)->current();
    }
}
"#,
                ),
            )],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| finding.kind == ArchitecturalAssessmentKind::SanctionedPathBypass)
            .expect("expected sanctioned path bypass finding");
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "concern:dependency_resolution"));
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "scanner:ast_grep"));
        assert!(finding
            .related_identifiers
            .contains(&String::from("raw_container_lookup")));
        assert!(finding
            .related_identifiers
            .contains(&String::from("constructor_injection")));
    }

    #[test]
    fn ignores_php_container_lookup_inside_provider_boundary() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Providers/AppServiceProvider.php"),
                    centrality_millis: 610,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/Providers/AppServiceProvider.php"),
                String::from(
                    r#"
<?php

final class AppServiceProvider extends ServiceProvider
{
    public function register(): void
    {
        app(TenantManager::class);
        $this->app->singleton(TenantManager::class);
    }
}
"#,
                ),
            )],
        );

        assert!(assessment
            .findings
            .iter()
            .all(|finding| finding.kind != ArchitecturalAssessmentKind::SanctionedPathBypass));
    }

    #[test]
    fn detects_abstraction_sprawl_for_concept_with_many_roles() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Notifications/NotificationService.php"),
                    centrality_millis: 640,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Notifications/NotificationService.php"),
                    String::from("final class NotificationService {}"),
                ),
                (
                    PathBuf::from("app/Notifications/NotificationPayloadBuilder.php"),
                    String::from("final class NotificationPayloadBuilder {}"),
                ),
                (
                    PathBuf::from("app/Notifications/NotificationTypeRegistry.php"),
                    String::from("final class NotificationTypeRegistry {}"),
                ),
                (
                    PathBuf::from("app/Notifications/NotificationPreferenceResolver.php"),
                    String::from("final class NotificationPreferenceResolver {}"),
                ),
                (
                    PathBuf::from("app/Notifications/NotificationChannelDispatcher.php"),
                    String::from("final class NotificationChannelDispatcher {}"),
                ),
            ],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| finding.kind == ArchitecturalAssessmentKind::AbstractionSprawl)
            .expect("expected abstraction sprawl finding");
        assert_eq!(
            finding.file_path,
            PathBuf::from("app/Notifications/NotificationService.php")
        );
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier.starts_with("concept:notification")));
        assert!(finding
            .warning_families
            .contains(&String::from("abstraction_role:service")));
        assert!(finding
            .warning_families
            .contains(&String::from("abstraction_role:builder")));
        assert!(finding
            .warning_families
            .contains(&String::from("abstraction_role:registry")));
        assert!(finding
            .warning_families
            .contains(&String::from("abstraction_role:resolver")));
    }

    #[test]
    fn layer_directory_words_do_not_form_abstraction_sprawl_concepts() {
        // Files across unrelated modules that share only the `Services/` layer
        // directory must not be grouped into a "services" sprawl concept.
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Modules/Billing/Services/InvoiceRegistry.php"),
                    String::from("final class InvoiceRegistry {}"),
                ),
                (
                    PathBuf::from("app/Modules/Chat/Services/MessageDispatcher.php"),
                    String::from("final class MessageDispatcher {}"),
                ),
                (
                    PathBuf::from("app/Modules/Crm/Services/LeadResolver.php"),
                    String::from("final class LeadResolver {}"),
                ),
                (
                    PathBuf::from("app/Modules/Hr/Services/PayrollFactory.php"),
                    String::from("final class PayrollFactory {}"),
                ),
            ],
        );

        assert!(
            assessment.findings.iter().all(|finding| {
                finding.kind != ArchitecturalAssessmentKind::AbstractionSprawl
                    || !finding
                        .related_identifiers
                        .contains(&String::from("concept:services"))
            }),
            "layer directory `services` must not become a sprawl concept"
        );
    }

    #[test]
    fn detects_hand_rolled_parsing_stack_for_custom_query_contract_flow() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Services/Filter/QueryContractParser.php"),
                    centrality_millis: 540,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Services/Filter/QueryContractParser.php"),
                    String::from(
                        r#"
                        final class QueryContractParser {
                            public function parse(Request $request): array {
                                $filters = json_decode($request->input('filters', '[]'), true);
                                $parts = array_map(trim(...), explode(',', (string) $request->query('sort')));
                                return $this->parseSort($parts);
                            }
                            private function parseSort(array $parts): array { return $parts; }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/Filter/FilterValidator.php"),
                    String::from(
                        r#"
                        final class FilterValidator {
                            public function validateOperator(string $operator): bool {
                                return preg_match('/^[a-z_]+$/', $operator) === 1;
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/Filter/FilterDefinitionResolver.php"),
                    String::from(
                        r#"
                        final class FilterDefinitionResolver {
                            public function resolve(string $name): array {
                                $normalized = trim($name);
                                return ['key' => substr($normalized, 0, 10)];
                            }
                        }
                    "#,
                    ),
                ),
            ],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| finding.kind == ArchitecturalAssessmentKind::HandRolledParsing)
            .expect("expected hand-rolled parsing finding");
        assert_eq!(
            finding.file_path,
            PathBuf::from("app/Services/Filter/QueryContractParser.php")
        );
        assert!(finding
            .warning_families
            .contains(&String::from("parsing_role:parser")));
        assert!(finding
            .warning_families
            .contains(&String::from("parsing_role:validator")));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier.starts_with("directory:")));
    }

    #[test]
    fn detects_homegrown_scheduler_dsl_stack() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Services/Jobs/ScheduledJobExecutor.php"),
                    centrality_millis: 480,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Services/Settings/JobRegistry.php"),
                    String::from(
                        r#"
                        final class JobRegistry {
                            public function getJobs(): array {
                                $jobs = config('jobs.jobs', []);
                                foreach ($this->moduleRegistry->getEnabledModules() as $module) {
                                    $manifestJobs = $module['manifest']['jobs'] ?? [];
                                    $jobs = array_merge($jobs, $manifestJobs);
                                }
                                return $jobs;
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/Jobs/ScheduledJobExecutor.php"),
                    String::from(
                        r#"
                        final class ScheduledJobExecutor {
                            public function execute(array $config): string {
                                $command = $config['command'] ?? '';
                                $exitCode = Artisan::call($command);
                                dispatch(new SyncTenantJob());
                                return (string) $exitCode;
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Console/Commands/ErpScheduledJobsRunCommand.php"),
                    String::from(
                        r#"
                        final class ErpScheduledJobsRunCommand {
                            public function handle(): int {
                                $expression = new CronExpression('* * * * *');
                                if ($expression->isDue(now())) {
                                    Cache::lock('scheduled_job:tenant:sync', 900);
                                }
                                return 0;
                            }
                        }
                    "#,
                    ),
                ),
            ],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::HandRolledParsing
                    && finding
                        .warning_families
                        .contains(&String::from("concern:custom_scheduler_dsl"))
            })
            .expect("expected homegrown scheduler dsl finding");
        assert!(finding
            .warning_families
            .contains(&String::from("parsing_role:registry")));
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "parsing_role:executor" || family == "parsing_role:command"));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier == "concept:schedule" || identifier == "concept:job"));
    }

    #[test]
    fn detects_filesystem_page_resolution_layer() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Services/_Core/ModulePageResolver.php"),
                    centrality_millis: 560,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Services/_Core/ModulePageResolver.php"),
                    String::from(
                        r#"
                        final class ModulePageResolver {
                            public static function resolve(string $entityType, string $action): string {
                                $pagePath = base_path("app/Modules/{$moduleName}/resources/js/Pages/{$entityType}/{$action}.vue");
                                if (File::exists($pagePath)) {
                                    return "{$moduleName}/{$entityType}/{$action}";
                                }
                                return "Entity/{$action}";
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/Page/PageRouteRegistry.php"),
                    String::from(
                        r#"
                        final class PageRouteRegistry {
                            public function build(): array {
                                foreach (File::allFiles(app_path('Pages')) as $file) {
                                    if (str_ends_with($file->getPathname(), '.page.php')) {
                                        $definition = $this->parsePageFile($file->getPathname());
                                    }
                                }
                                return [];
                            }
                            private function parseRoute(string $docblock): ?array {
                                preg_match('/@route\s+([A-Z\|,\s]+)\s+([^\s]+)/', $docblock, $matches);
                                return ['path' => $matches[2] ?? null];
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/Action/ActionRouteRegistry.php"),
                    String::from(
                        r#"
                        final class ActionRouteRegistry {
                            public function registerRoutes(): void {
                                Route::match(['POST'], 'actions/run', ExampleAction::class);
                            }
                            private function parseActionFile(string $path): ?array {
                                require_once $path;
                                return ['path' => 'actions/run'];
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/I18n/LocaleResolver.php"),
                    String::from(
                        r#"
                        final class LocaleResolver {
                            public function getFallbackLanguage(): string {
                                return 'en_US';
                            }
                        }
                    "#,
                    ),
                ),
            ],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::HandRolledParsing
                    && finding
                        .warning_families
                        .contains(&String::from("concern:filesystem_page_resolution"))
            })
            .expect("expected filesystem page resolution finding");
        assert_eq!(
            finding.file_path,
            PathBuf::from("app/Services/_Core/ModulePageResolver.php")
        );
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier == "concept:page"));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier == "concept:route"));
    }

    #[test]
    fn ignores_plain_route_files_without_filesystem_page_resolution_layer() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("routes/web.php"),
                    String::from(
                        r#"
                        Route::middleware(['auth'])->group(function (): void {
                            Route::get('/dashboard', [DashboardController::class, 'index']);
                            Route::post('/profile', [ProfileController::class, 'update']);
                        });
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Http/Controllers/DashboardController.php"),
                    String::from("final class DashboardController {}"),
                ),
            ],
        );

        assert!(assessment.findings.iter().all(|finding| {
            finding.kind != ArchitecturalAssessmentKind::HandRolledParsing
                || !finding
                    .warning_families
                    .contains(&String::from("concern:filesystem_page_resolution"))
        }));
    }

    #[test]
    fn detects_manifest_backed_policy_engine_layer() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![
                    BottleneckFile {
                        file_path: PathBuf::from(
                            "app/Services/_Core/Templates/TemplatePolicyRegistry.php",
                        ),
                        centrality_millis: 610,
                        fingerprint: String::new(),
                    },
                    BottleneckFile {
                        file_path: PathBuf::from(
                            "app/Services/_Core/Templates/TemplateInstantiationService.php",
                        ),
                        centrality_millis: 580,
                        fingerprint: String::new(),
                    },
                ],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Services/_Core/Templates/TemplatePolicyRegistry.php"),
                    String::from(
                        r#"
                        final class TemplatePolicyRegistry {
                            private array $assignmentResolvers = [];
                            private array $deadlineResolvers = [];

                            private function loadIfNeeded(): void
                            {
                                foreach ($this->moduleRegistry->getEnabledModules() as $module) {
                                    $manifest = $module['manifest'] ?? null;
                                    $policies = $manifest['templatePolicies'] ?? null;
                                }
                            }

                            public function resolveAssignmentResolver(?string $key): ?TemplateAssignmentResolverInterface
                            {
                                return null;
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/_Core/Templates/TemplateCompiler.php"),
                    String::from(
                        r#"
                        final class TemplateCompiler {
                            public function compile(array $definition): array
                            {
                                $nodesByKey = $this->normalizeNodes($definition['nodes'] ?? null);
                                $edges = $this->normalizeEdges($definition['edges'] ?? null);
                                $orderedKeys = $this->topologicalSort(array_keys($nodesByKey), []);
                                return ['nodesByKey' => $nodesByKey, 'orderedKeys' => $orderedKeys, 'deferredEdges' => $edges];
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/_Core/Templates/TemplateInstantiationService.php"),
                    String::from(
                        r#"
                        final class TemplateInstantiationService {
                            public function instantiate(array $definition): array
                            {
                                $compiled = $this->compiler->compile($definition);
                                $fieldValues = $this->applyAssignmentPolicy([], [], [], []);
                                $fieldValues = $this->applyDeadlinePolicy([], $fieldValues, [], []);
                                return ['createdByNode' => ['root' => 1], 'nodeKey' => 'root'];
                            }
                        }
                    "#,
                    ),
                ),
            ],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::HandRolledParsing
                    && finding
                        .warning_families
                        .contains(&String::from("concern:manifest_backed_policy_engine"))
            })
            .expect("expected manifest-backed policy engine finding");
        assert_eq!(
            finding.file_path,
            PathBuf::from("app/Services/_Core/Templates/TemplatePolicyRegistry.php")
        );
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier == "concept:manifest"));
        assert!(finding.related_identifiers.iter().any(|identifier| {
            identifier == "concept:workflow" || identifier == "concept:instantiation"
        }));
    }

    #[test]
    fn ignores_single_parser_file_without_cluster() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/Support/SlugParser.php"),
                String::from(
                    r#"
                    final class SlugParser {
                        public function parse(string $value): array {
                            return explode('-', trim($value));
                        }
                    }
                "#,
                ),
            )],
        );

        assert!(assessment
            .findings
            .iter()
            .all(|finding| finding.kind != ArchitecturalAssessmentKind::HandRolledParsing));
    }

    #[test]
    fn detects_hand_rolled_contract_stack_without_primary_parser_role() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from("app/Services/Filter/FilterDefinitionResolver.php"),
                    centrality_millis: 520,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Services/Filter/FilterValidator.php"),
                    String::from(
                        r#"
                        final class FilterValidator {
                            public function validateOperator(string $operator): bool {
                                return preg_match('/^[a-z_]+$/', $operator) === 1;
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/Filter/FilterDefinitionResolver.php"),
                    String::from(
                        r#"
                        final class FilterDefinitionResolver {
                            public function resolve(string $name): array {
                                $normalized = trim($name);
                                return ['key' => substr($normalized, 0, 10)];
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Services/Filter/FilterNormalizer.php"),
                    String::from(
                        r#"
                        final class FilterNormalizer {
                            public function normalize(string $value): string {
                                return substr(trim(strtolower($value)), 0, 32);
                            }
                        }
                    "#,
                    ),
                ),
            ],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| finding.kind == ArchitecturalAssessmentKind::HandRolledParsing)
            .expect("expected hand-rolled parsing finding");
        assert_eq!(
            finding.file_path,
            PathBuf::from("app/Services/Filter/FilterDefinitionResolver.php")
        );
        assert!(finding
            .warning_families
            .contains(&String::from("concern:custom_contract_stack")));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier == "concept:filter"));
    }

    #[test]
    fn detects_homegrown_schema_validation_stack() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis {
                bottleneck_files: vec![BottleneckFile {
                    file_path: PathBuf::from(
                        "app/Modules/Preferences/Services/PreferenceValueValidator.php",
                    ),
                    centrality_millis: 480,
                    fingerprint: String::new(),
                }],
                ..GraphAnalysis::default()
            },
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Modules/Preferences/Services/PreferenceDefinition.php"),
                    String::from(
                        r#"
                        final class PreferenceDefinition {
                            public array $schema = [
                                'type' => 'object',
                                'required' => ['mode'],
                                'properties' => ['mode' => ['type' => 'string']],
                            ];
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Modules/Preferences/Services/PreferenceValueValidator.php"),
                    String::from(
                        r#"
                        final class PreferenceValueValidator {
                            public function validate(PreferenceDefinition $definition, mixed $value): void {
                                $schema = $definition->schema;
                                $this->assertMatchesSchema($value, $schema);
                            }

                            private function assertMatchesSchema(mixed $value, array $schema): void {
                                if (array_key_exists('enum', $schema)) {}
                                if (($schema['nullable'] ?? false) && $value === null) { return; }
                                if (($schema['type'] ?? null) === 'object') {
                                    $this->assertObjectSchema($value, $schema);
                                }
                            }

                            private function assertObjectSchema(mixed $value, array $schema): void {
                                $properties = $schema['properties'] ?? [];
                                $required = $schema['required'] ?? [];
                            }
                        }
                    "#,
                    ),
                ),
                (
                    PathBuf::from("app/Modules/Preferences/Services/PreferenceValueNormalizer.php"),
                    String::from(
                        r#"
                        final class PreferenceValueNormalizer {
                            public function normalize(array $schema, mixed $value): mixed {
                                if (($schema['type'] ?? null) === 'string') {
                                    return trim((string) $value);
                                }
                                return $value;
                            }
                        }
                    "#,
                    ),
                ),
            ],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| finding.kind == ArchitecturalAssessmentKind::HandRolledParsing)
            .expect("expected hand-rolled parsing finding");
        assert_eq!(
            finding.file_path,
            PathBuf::from("app/Modules/Preferences/Services/PreferenceValueValidator.php")
        );
        assert!(finding
            .warning_families
            .contains(&String::from("concern:custom_schema_validation")));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier == "concept:preference"));
    }

    #[test]
    fn detects_nested_iteration_as_algorithmic_complexity_hotspot() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("src/solver.rs"),
                String::from(
                    r#"
                    fn solve(items: &[u32], needles: &[u32]) -> usize {
                        let mut matches = 0;
                        for item in items {
                            for needle in needles {
                                if item == needle {
                                    matches += 1;
                                }
                            }
                        }
                        matches
                    }
                "#,
                ),
            )],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
            })
            .expect("expected algorithmic complexity hotspot");
        assert_eq!(finding.file_path, PathBuf::from("src/solver.rs"));
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "complexity:nested_iteration"));
    }

    #[test]
    fn loop_keyword_inside_a_string_is_not_nested_iteration() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/Console/Commands/WarmCommand.php"),
                String::from(
                    r#"<?php
class WarmCommand {
    public function handle($tenants) {
        foreach ($tenants as $tenant) {
            $this->info("Warming cache for tenant: {$tenant->slug}");
            $this->line("Cleared cache for while running");
        }
    }
}
"#,
                ),
            )],
        );

        // The single foreach has no nested loop; the words "for"/"while" only
        // appear inside prose strings, so nothing should be reported.
        assert!(
            !assessment.findings.iter().any(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
            }),
            "loop keywords inside string literals must not produce a hotspot: {:?}",
            assessment
                .findings
                .iter()
                .filter(|f| f.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot)
                .map(|f| (f.file_path.clone(), f.warning_families.clone()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn detects_collection_scan_in_loop_as_algorithmic_complexity_hotspot() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("src/filter.ts"),
                String::from(
                    r#"
                    export function keepKnown(items: string[], allowList: string[]): string[] {
                        const kept: string[] = [];
                        for (const item of items) {
                            if (allowList.includes(item)) {
                                kept.push(item);
                            }
                        }
                        return kept;
                    }
                "#,
                ),
            )],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
            })
            .expect("expected algorithmic complexity hotspot");
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "complexity:collection_scan_in_loop"));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier.contains(".includes(")));
    }

    #[test]
    fn in_array_against_literal_set_is_not_a_collection_scan_hotspot() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/Services/Router.php"),
                String::from(
                    r#"<?php
function route($rows) {
    foreach ($rows as $row) {
        if (in_array($row->status, ['delivered', 'returned'], true)) {
            handle($row);
        }
    }
}
"#,
                ),
            )],
        );

        // The only in-loop operation is a fixed-set membership check; no
        // collection-scan hotspot should be raised.
        assert!(
            !assessment.findings.iter().any(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
                    && finding
                        .warning_families
                        .iter()
                        .any(|family| family == "complexity:collection_scan_in_loop")
            }),
            "literal-array in_array must not be a collection scan, got: {:?}",
            assessment.findings
        );
    }

    #[test]
    fn in_array_against_variable_collection_is_still_a_hotspot() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/Services/Router.php"),
                String::from(
                    r#"<?php
function route($rows, $allowed) {
    foreach ($rows as $row) {
        if (in_array($row->status, $allowed, true)) {
            handle($row);
        }
    }
}
"#,
                ),
            )],
        );

        assert!(
            assessment.findings.iter().any(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
                    && finding
                        .warning_families
                        .iter()
                        .any(|family| family == "complexity:collection_scan_in_loop")
            }),
            "variable-haystack in_array must remain a hotspot, got: {:?}",
            assessment.findings
        );
    }

    #[test]
    fn migrations_and_seeders_are_not_complexity_hotspots() {
        let nested_loop = r#"<?php
foreach ($tables as $table) {
    foreach ($table->columns as $column) {
        migrate($column);
    }
}
"#;
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from(
                        "app/Modules/HR/migrations/2026_02_21_120000_add_parallel_work.php",
                    ),
                    String::from(nested_loop),
                ),
                (
                    PathBuf::from("database/seeders/AccountingSeeder.php"),
                    String::from(nested_loop),
                ),
                (
                    PathBuf::from("app/Services/RuntimeLoader.php"),
                    String::from(nested_loop),
                ),
            ],
        );

        let hotspot_files: Vec<_> = assessment
            .findings
            .iter()
            .filter(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
            })
            .map(|finding| finding.file_path.to_string_lossy().to_string())
            .collect();
        assert!(
            hotspot_files.iter().any(|p| p.contains("RuntimeLoader")),
            "runtime file must still be a hotspot, got: {hotspot_files:?}"
        );
        assert!(
            !hotspot_files
                .iter()
                .any(|p| p.contains("migrations") || p.contains("seeders")),
            "migration/seeder files must not be hotspots, got: {hotspot_files:?}"
        );
    }

    #[test]
    fn detects_json_decode_in_loop_as_algorithmic_complexity_hotspot() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("src/importer.py"),
                String::from(
                    r#"
                    import json

                    def load_rows(lines):
                        rows = []
                        for line in lines:
                            rows.append(json.loads(line))
                        return rows
                "#,
                ),
            )],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
            })
            .expect("expected algorithmic complexity hotspot");
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "complexity:json_decode_in_loop"));
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "scanner:ast_grep"));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier.contains("json.loads(")));
    }

    #[test]
    fn detects_filesystem_read_in_loop_as_algorithmic_complexity_hotspot() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("src/loader.ts"),
                String::from(
                    r#"
                    import fs from "fs";

                    export function loadAll(paths: string[]): string[] {
                        const out: string[] = [];
                        for (const path of paths) {
                            out.push(fs.readFileSync(path, "utf8"));
                        }
                        return out;
                    }
                "#,
                ),
            )],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
            })
            .expect("expected algorithmic complexity hotspot");
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "complexity:filesystem_read_in_loop"));
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "scanner:ast_grep"));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier.contains("fs.readFileSync(")));
    }

    #[test]
    fn detects_laravel_database_query_in_loop_as_algorithmic_complexity_hotspot() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/Services/InvoiceSyncService.php"),
                String::from(
                    r#"
                    <?php
                    use Illuminate\Support\Facades\DB;

                    final class InvoiceSyncService {
                        public function sync(array $ids): array {
                            $rows = [];
                            foreach ($ids as $id) {
                                $rows[] = DB::select('select * from invoices where id = ?', [$id]);
                            }
                            return $rows;
                        }
                    }
                "#,
                ),
            )],
        );

        let finding = assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
            })
            .expect("expected algorithmic complexity hotspot");
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "complexity:database_query_in_loop"));
        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "scanner:ast_grep"));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| identifier.contains("DB::select(")));
        assert_eq!(finding.expensive_operation_sites.len(), 1);
        assert_eq!(finding.expensive_operation_sites[0].line, 9);
        assert_eq!(
            finding.expensive_operation_sites[0].subtype,
            "database_query_in_loop"
        );
        assert!(finding.expensive_operation_sites[0]
            .token
            .contains("DB::select("));
    }

    #[test]
    fn graph_reachable_complexity_hotspot_records_entry_pressure() {
        let fixture = unique_fixture_dir("assessment-complexity-entry-pressure");
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.ts"),
            "import { run } from './runner';\n\nrun();\n",
        )
        .unwrap();
        fs::write(
            fixture.join("src/runner.ts"),
            r#"
export function run(items: string[][]) {
  for (const row of items) {
    row.sort();
  }
}
"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let finding = analysis
            .architectural_assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
                    && finding.file_path == PathBuf::from("src/runner.ts")
            })
            .expect("expected graph-reachable complexity hotspot");

        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "pressure:entry_reachable_via_graph"));
        assert!(finding
            .related_file_paths
            .contains(&PathBuf::from("src/main.ts")));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| { identifier == "entry_path: src/main.ts -> src/runner.ts" }));
        assert_eq!(finding.pressure_path.len(), 2);
        assert_eq!(
            finding.pressure_path[0].file_path,
            PathBuf::from("src/main.ts")
        );
        assert_eq!(finding.pressure_path[0].line, Some(3));
        assert_eq!(
            finding.pressure_path[0].relation_to_next,
            Some(RelationKind::Call)
        );
        assert_eq!(finding.pressure_path[0].source_symbol, None);
        assert_eq!(
            finding.pressure_path[0].target_symbol.as_deref(),
            Some("run")
        );
        assert_eq!(
            finding.pressure_path[1].file_path,
            PathBuf::from("src/runner.ts")
        );
        assert_eq!(finding.pressure_path[1].line, None);
        assert_eq!(finding.pressure_path[1].relation_to_next, None);
        assert_eq!(finding.pressure_path[1].source_symbol, None);
        assert_eq!(finding.pressure_path[1].target_symbol, None);
        assert_eq!(finding.expensive_operation_sites.len(), 1);
        assert_eq!(finding.expensive_operation_sites[0].line, 4);
        assert_eq!(finding.expensive_operation_sites[0].subtype, "sort_in_loop");
        assert!(finding.expensive_operation_sites[0]
            .token
            .contains("row.sort("));
        assert_eq!(finding.expensive_operation_flow.len(), 3);
        assert_eq!(
            finding.expensive_operation_flow[0].file_path,
            PathBuf::from("src/main.ts")
        );
        assert_eq!(
            finding.expensive_operation_flow[1].file_path,
            PathBuf::from("src/runner.ts")
        );
        assert_eq!(
            finding.expensive_operation_flow[2].subtype.as_deref(),
            Some("sort_in_loop")
        );
        assert_eq!(finding.expensive_operation_flow[2].line, Some(4));
    }

    #[test]
    fn complexity_pressure_paths_prefer_call_edges_over_plain_imports_when_both_exist() {
        let parsed_sources = vec![
            (
                PathBuf::from("src/main.ts"),
                String::from("import { handler } from './handler';\nhandler();\n"),
            ),
            (
                PathBuf::from("src/handler.ts"),
                String::from(
                    "import { run } from './runner';\nexport function handler(items) { return run(items); }\n",
                ),
            ),
            (
                PathBuf::from("src/runner.ts"),
                String::from(
                    "export function run(items) { for (const row of items) { row.sort(); } }\n",
                ),
            ),
        ];
        let graph_analysis = GraphAnalysis {
            runtime_entry_candidates: vec![PathBuf::from("src/main.ts")],
            ..GraphAnalysis::default()
        };
        let semantic_graph = SemanticGraph {
            files: Vec::new(),
            symbols: vec![
                crate::graph::SymbolNode {
                    id: String::from("symbol:src/handler.ts:handler"),
                    file_path: PathBuf::from("src/handler.ts"),
                    kind: SymbolKind::Function,
                    name: String::from("handler"),
                    qualified_name: String::from("handler"),
                    parent_symbol_id: None,
                    owner_type_name: None,
                    return_type_name: None,
                    visibility: Visibility::Public,
                    parameter_count: 1,
                    required_parameter_count: 1,
                    start_line: 2,
                    end_line: 2,
                },
                crate::graph::SymbolNode {
                    id: String::from("symbol:src/runner.ts:run"),
                    file_path: PathBuf::from("src/runner.ts"),
                    kind: SymbolKind::Function,
                    name: String::from("run"),
                    qualified_name: String::from("run"),
                    parent_symbol_id: None,
                    owner_type_name: None,
                    return_type_name: None,
                    visibility: Visibility::Public,
                    parameter_count: 1,
                    required_parameter_count: 1,
                    start_line: 1,
                    end_line: 1,
                },
            ],
            references: Vec::new(),
            resolved_edges: vec![
                crate::graph::ResolvedEdge {
                    source_file_path: PathBuf::from("src/main.ts"),
                    source_symbol_id: None,
                    target_file_path: PathBuf::from("src/handler.ts"),
                    target_symbol_id: String::from("symbol:src/handler.ts:handler"),
                    reference_target_name: None,
                    kind: crate::graph::ReferenceKind::Call,
                    relation_kind: RelationKind::Import,
                    layer: crate::graph::GraphLayer::Structural,
                    strength: crate::graph::EdgeStrength::Hard,
                    origin: crate::graph::EdgeOrigin::Resolver,
                    resolution_tier: crate::graph::ResolutionTier::ImportScoped,
                    confidence_millis: 800,
                    reason: String::from("import"),
                    line: 1,
                    occurrence_index: 0,
                },
                crate::graph::ResolvedEdge {
                    source_file_path: PathBuf::from("src/handler.ts"),
                    source_symbol_id: None,
                    target_file_path: PathBuf::from("src/runner.ts"),
                    target_symbol_id: String::from("symbol:src/runner.ts:run"),
                    reference_target_name: None,
                    kind: crate::graph::ReferenceKind::Call,
                    relation_kind: RelationKind::Import,
                    layer: crate::graph::GraphLayer::Structural,
                    strength: crate::graph::EdgeStrength::Hard,
                    origin: crate::graph::EdgeOrigin::Resolver,
                    resolution_tier: crate::graph::ResolutionTier::ImportScoped,
                    confidence_millis: 800,
                    reason: String::from("import"),
                    line: 1,
                    occurrence_index: 0,
                },
                crate::graph::ResolvedEdge {
                    source_file_path: PathBuf::from("src/handler.ts"),
                    source_symbol_id: Some(String::from("symbol:src/handler.ts:handler")),
                    target_file_path: PathBuf::from("src/runner.ts"),
                    target_symbol_id: String::from("symbol:src/runner.ts:run"),
                    reference_target_name: Some(String::from("run")),
                    kind: crate::graph::ReferenceKind::Call,
                    relation_kind: RelationKind::Call,
                    layer: crate::graph::GraphLayer::Structural,
                    strength: crate::graph::EdgeStrength::Hard,
                    origin: crate::graph::EdgeOrigin::Resolver,
                    resolution_tier: crate::graph::ResolutionTier::ImportScoped,
                    confidence_millis: 950,
                    reason: String::from("call"),
                    line: 2,
                    occurrence_index: 0,
                },
            ],
        };

        let assessment = build_architectural_assessment_with_ast_grep_and_graph(
            &graph_analysis,
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &parsed_sources,
            &run_ast_grep_scan(&parsed_sources),
            Some(&semantic_graph),
        );
        let finding = assessment
            .findings
            .iter()
            .find(|finding| {
                finding.kind == ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
                    && finding.file_path == PathBuf::from("src/runner.ts")
            })
            .expect("expected graph-reachable complexity hotspot");

        assert!(finding
            .warning_families
            .iter()
            .any(|family| family == "pressure:caller_callee_path"));
        assert!(finding
            .related_identifiers
            .iter()
            .any(|identifier| { identifier == "pressure_path_symbols: handler -> run" }));
        assert_eq!(finding.pressure_path.len(), 3);
        assert_eq!(
            finding.pressure_path[0].file_path,
            PathBuf::from("src/main.ts")
        );
        assert_eq!(
            finding.pressure_path[1].relation_to_next,
            Some(RelationKind::Call)
        );
        assert_eq!(finding.pressure_path[1].line, Some(2));
        assert_eq!(
            finding.pressure_path[1].source_symbol.as_deref(),
            Some("handler")
        );
        assert_eq!(
            finding.pressure_path[1].target_symbol.as_deref(),
            Some("run")
        );
    }

    #[test]
    fn ignores_simple_single_loop_without_expensive_inner_work() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("src/linear.py"),
                String::from(
                    r#"
                    def accumulate(items):
                        total = 0
                        for item in items:
                            total += item
                        return total
                "#,
                ),
            )],
        );

        assert!(assessment.findings.iter().all(|finding| {
            finding.kind != ArchitecturalAssessmentKind::AlgorithmicComplexityHotspot
        }));
    }

    #[test]
    fn ignores_plain_service_clusters_without_role_variety() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[
                (
                    PathBuf::from("app/Orders/OrderService.php"),
                    String::from("final class OrderService {}"),
                ),
                (
                    PathBuf::from("app/Orders/OrderExportService.php"),
                    String::from("final class OrderExportService {}"),
                ),
                (
                    PathBuf::from("app/Orders/OrderListService.php"),
                    String::from("final class OrderListService {}"),
                ),
            ],
        );

        assert!(assessment
            .findings
            .iter()
            .all(|finding| finding.kind != ArchitecturalAssessmentKind::AbstractionSprawl));
    }

    #[test]
    fn ignores_migration_only_identity_variants() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("app/migrations/2026_create_tasks.php"),
                String::from(
                    r#"
                    Schema::create('tasks', function ($table) {
                        $table->unsignedBigInteger('assigned_user_id')->nullable();
                    });
                "#,
                ),
            )],
        );

        assert!(assessment
            .findings
            .iter()
            .all(|finding| finding.kind != ArchitecturalAssessmentKind::SplitIdentityModel));
    }

    #[test]
    fn ignores_test_prefix_identity_variants() {
        let assessment = build_architectural_assessment(
            &GraphAnalysis::default(),
            &DeadCodeResult::default(),
            &HardwiringResult::default(),
            &ExternalAnalysisResult::default(),
            &[(
                PathBuf::from("test/client.py"),
                String::from(
                    "content_type = 1\ncontent_type_id = 2\nget_content_type = 3\nContentType = 4\ncontent_type = 5\n",
                ),
            )],
        );

        assert!(assessment
            .findings
            .iter()
            .all(|finding| finding.kind != ArchitecturalAssessmentKind::SplitIdentityModel));
    }

    fn unique_fixture_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("aigiscode-{label}-{unique}"))
    }
}
