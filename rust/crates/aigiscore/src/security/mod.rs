use crate::contracts::ContractInventory;
use crate::graph::{RelationKind, SemanticGraph};
use crate::scanners::ast_grep::{run_ast_grep_scan, AstGrepFindingKind, AstGrepScanResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityFindingKind {
    DangerousApi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityCategory {
    CommandExecution,
    CodeInjection,
    UnsafeDeserialization,
    UnsafeHtmlOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecuritySeverity {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecurityContext {
    ExternallyReachable,
    EntryReachableViaGraph,
    BoundaryInputInSameFile,
    BoundaryInputReachableViaGraph,
    InteractiveExecution,
    CacheStorage,
    DatabaseTooling,
    MigrationSupport,
    DevelopmentRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub kind: SecurityFindingKind,
    pub category: SecurityCategory,
    pub severity: SecuritySeverity,
    pub file_path: PathBuf,
    pub line: usize,
    pub message: String,
    pub evidence: String,
    pub fingerprint: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_scanners: Vec<String>,
    #[serde(default)]
    pub contexts: Vec<SecurityContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reachability_path: Vec<SecurityReachabilityHop>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_input_sources: Vec<SecurityInputSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_input_path: Vec<SecurityReachabilityHop>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_to_sink_flow: Vec<SecurityFlowStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SecurityAnalysisResult {
    pub findings: Vec<SecurityFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityReachabilityHop {
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
    pub reference_target_name: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub occurrence_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityInputKind {
    RequestQuery,
    RequestBody,
    RequestParams,
    RequestInput,
    CliArgument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityInputSource {
    pub kind: SecurityInputKind,
    pub file_path: PathBuf,
    pub line: usize,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityFlowStepKind {
    BoundaryInput,
    GraphHop,
    Sink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityFlowStep {
    pub kind: SecurityFlowStepKind,
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
    pub reference_target_name: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub occurrence_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_kind: Option<SecurityInputKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

impl SecurityAnalysisResult {
    pub fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }
}

pub fn analyze_security_findings(
    parsed_sources: &[(PathBuf, String)],
    contract_inventory: &ContractInventory,
    runtime_entry_candidates: &[PathBuf],
) -> SecurityAnalysisResult {
    let ast_grep_scan = run_ast_grep_scan(parsed_sources);
    analyze_security_findings_with_ast_grep(
        parsed_sources,
        contract_inventory,
        runtime_entry_candidates,
        &ast_grep_scan,
    )
}

pub fn analyze_security_findings_with_graph(
    parsed_sources: &[(PathBuf, String)],
    contract_inventory: &ContractInventory,
    runtime_entry_candidates: &[PathBuf],
    semantic_graph: &SemanticGraph,
) -> SecurityAnalysisResult {
    let ast_grep_scan = run_ast_grep_scan(parsed_sources);
    analyze_security_findings_with_ast_grep_and_graph(
        parsed_sources,
        contract_inventory,
        runtime_entry_candidates,
        &ast_grep_scan,
        Some(semantic_graph),
    )
}

pub fn analyze_security_findings_with_ast_grep(
    parsed_sources: &[(PathBuf, String)],
    contract_inventory: &ContractInventory,
    runtime_entry_candidates: &[PathBuf],
    ast_grep_scan: &AstGrepScanResult,
) -> SecurityAnalysisResult {
    analyze_security_findings_with_ast_grep_and_graph(
        parsed_sources,
        contract_inventory,
        runtime_entry_candidates,
        ast_grep_scan,
        None,
    )
}

pub fn analyze_security_findings_with_ast_grep_and_graph(
    parsed_sources: &[(PathBuf, String)],
    contract_inventory: &ContractInventory,
    runtime_entry_candidates: &[PathBuf],
    ast_grep_scan: &AstGrepScanResult,
    semantic_graph: Option<&SemanticGraph>,
) -> SecurityAnalysisResult {
    let externally_reachable_files =
        externally_reachable_files(contract_inventory, runtime_entry_candidates);
    let boundary_input_sources = parsed_sources
        .iter()
        .map(|(path, content)| (path.clone(), collect_boundary_input_sources(path, content)))
        .filter(|(_path, sources)| !sources.is_empty())
        .collect::<HashMap<_, _>>();
    let boundary_input_paths = semantic_graph
        .map(|graph| {
            boundary_input_paths_via_graph(
                graph,
                &boundary_input_sources
                    .keys()
                    .cloned()
                    .collect::<HashSet<_>>(),
            )
        })
        .unwrap_or_default();
    let graph_reachability_paths = semantic_graph
        .map(|graph| entry_reachability_paths_via_graph(graph, &externally_reachable_files))
        .unwrap_or_default();
    let graph_reachable_files = graph_reachability_paths
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    let mut findings = Vec::new();

    for (path, content) in parsed_sources {
        if is_test_like_path(path) {
            continue;
        }
        findings.extend(find_dangerous_api_findings(
            path,
            content,
            externally_reachable_files.contains(path),
            graph_reachable_files.contains(path),
        ));
    }

    merge_ast_grep_security_support(
        &mut findings,
        ast_grep_scan,
        &externally_reachable_files,
        &graph_reachable_files,
    );
    attach_graph_reachability_paths(&mut findings, &graph_reachability_paths);
    attach_boundary_input_sources(
        &mut findings,
        &boundary_input_sources,
        &boundary_input_paths,
    );

    findings.sort_by(|left, right| {
        severity_rank(right.severity)
            .cmp(&severity_rank(left.severity))
            .then(left.file_path.cmp(&right.file_path))
            .then(left.line.cmp(&right.line))
            .then(left.message.cmp(&right.message))
    });

    SecurityAnalysisResult { findings }
}

fn externally_reachable_files(
    contract_inventory: &ContractInventory,
    runtime_entry_candidates: &[PathBuf],
) -> HashSet<PathBuf> {
    let mut files = runtime_entry_candidates
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    for location in contract_inventory
        .routes
        .iter()
        .flat_map(|item| item.locations.iter())
    {
        files.insert(location.file_path.clone());
    }
    files
}

fn find_dangerous_api_findings(
    path: &Path,
    content: &str,
    externally_reachable: bool,
    graph_reachable: bool,
) -> Vec<SecurityFinding> {
    let Some(language) = detect_language(path) else {
        return Vec::new();
    };

    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line_no = index + 1;
            let trimmed = line.trim_start();
            if trimmed.is_empty() || is_comment_line(language, trimmed) {
                return None;
            }
            classify_line(
                path,
                trimmed,
                externally_reachable,
                graph_reachable,
                line_no,
                language,
            )
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceLanguage {
    Php,
    Python,
    Ruby,
    JavaScript,
    TypeScript,
}

fn detect_language(path: &Path) -> Option<SourceLanguage> {
    match path.extension().and_then(|value| value.to_str()) {
        Some("php") => Some(SourceLanguage::Php),
        Some("py") => Some(SourceLanguage::Python),
        Some("rb") => Some(SourceLanguage::Ruby),
        Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => Some(SourceLanguage::JavaScript),
        Some("ts") | Some("tsx") => Some(SourceLanguage::TypeScript),
        _ => None,
    }
}

fn classify_line(
    path: &Path,
    line: &str,
    externally_reachable: bool,
    graph_reachable: bool,
    line_no: usize,
    language: SourceLanguage,
) -> Option<SecurityFinding> {
    let (category, api_name) =
        match language {
            SourceLanguage::Php => classify_php_dangerous_api(line)?,
            SourceLanguage::Python => classify_python_dangerous_api(line)?,
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
                classify_javascript_dangerous_api(line)?
            }
            _ => dangerous_api_patterns(language).iter().find_map(
                |(category, pattern, api_name)| {
                    pattern.is_match(line).then_some((*category, *api_name))
                },
            )?,
        };
    let severity = match category {
        SecurityCategory::UnsafeHtmlOutput => {
            if externally_reachable || graph_reachable {
                SecuritySeverity::Medium
            } else {
                SecuritySeverity::Low
            }
        }
        SecurityCategory::CommandExecution
        | SecurityCategory::CodeInjection
        | SecurityCategory::UnsafeDeserialization => {
            if externally_reachable || graph_reachable {
                SecuritySeverity::High
            } else {
                SecuritySeverity::Medium
            }
        }
    };
    let contexts =
        classify_security_contexts(path, category, externally_reachable, graph_reachable);

    Some(SecurityFinding {
        kind: SecurityFindingKind::DangerousApi,
        category,
        severity,
        file_path: path.to_path_buf(),
        line: line_no,
        message: dangerous_api_message(category, api_name, &contexts),
        evidence: line.trim().to_string(),
        fingerprint: format!(
            "dangerous-api|{}|{}|{}|{}",
            path.display(),
            line_no,
            security_category_label(category),
            api_name
        ),
        supporting_scanners: Vec::new(),
        contexts,
        reachability_path: Vec::new(),
        boundary_input_sources: Vec::new(),
        boundary_input_path: Vec::new(),
        boundary_to_sink_flow: Vec::new(),
    })
}

fn merge_ast_grep_security_support(
    findings: &mut Vec<SecurityFinding>,
    ast_grep_scan: &AstGrepScanResult,
    externally_reachable_files: &HashSet<PathBuf>,
    graph_reachable_files: &HashSet<PathBuf>,
) {
    let mut fingerprint_to_index = findings
        .iter()
        .enumerate()
        .map(|(index, finding)| (finding.fingerprint.clone(), index))
        .collect::<std::collections::HashMap<_, _>>();

    for finding in &ast_grep_scan.findings {
        let AstGrepFindingKind::SecurityDangerousApi { .. } = &finding.kind else {
            continue;
        };
        if is_test_like_path(&finding.file_path) {
            continue;
        }
        let Some(language) = detect_language(&finding.file_path) else {
            continue;
        };
        let trimmed = finding.matched_text.trim();
        if trimmed.is_empty() || is_comment_line(language, trimmed) {
            continue;
        }
        let externally_reachable = externally_reachable_files.contains(&finding.file_path);
        let graph_reachable = graph_reachable_files.contains(&finding.file_path);
        let Some(native_finding) = classify_line(
            &finding.file_path,
            trimmed,
            externally_reachable,
            graph_reachable,
            finding.line,
            language,
        ) else {
            continue;
        };
        if let Some(index) = fingerprint_to_index
            .get(&native_finding.fingerprint)
            .copied()
        {
            let scanners = &mut findings[index].supporting_scanners;
            if !scanners.iter().any(|scanner| scanner == "ast_grep") {
                scanners.push(String::from("ast_grep"));
            }
            continue;
        }
        let fingerprint = native_finding.fingerprint.clone();
        findings.push(SecurityFinding {
            supporting_scanners: vec![String::from("ast_grep")],
            ..native_finding
        });
        fingerprint_to_index.insert(fingerprint, findings.len() - 1);
    }
}

fn attach_graph_reachability_paths(
    findings: &mut [SecurityFinding],
    reachability_paths: &HashMap<PathBuf, Vec<SecurityReachabilityHop>>,
) {
    for finding in findings {
        if !finding
            .contexts
            .contains(&SecurityContext::EntryReachableViaGraph)
        {
            continue;
        }
        let Some(path) = reachability_paths.get(&finding.file_path) else {
            continue;
        };
        let mut hops = path.clone();
        if let Some(last) = hops.last_mut() {
            last.line = Some(finding.line);
            last.relation_to_next = None;
        }
        finding.reachability_path = hops;
    }
}

fn attach_boundary_input_sources(
    findings: &mut [SecurityFinding],
    boundary_input_sources: &HashMap<PathBuf, Vec<SecurityInputSource>>,
    boundary_input_paths: &HashMap<PathBuf, Vec<SecurityReachabilityHop>>,
) {
    for finding in findings {
        let mut relevant_sources = boundary_input_sources
            .get(&finding.file_path)
            .map(|sources| {
                sources
                    .iter()
                    .filter(|source| source.line <= finding.line)
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut boundary_path = Vec::new();
        let mut boundary_context = None;

        if !relevant_sources.is_empty() {
            boundary_context = Some(SecurityContext::BoundaryInputInSameFile);
        } else if let Some(path) = boundary_input_paths.get(&finding.file_path) {
            let Some(source_file) = path.first().map(|hop| hop.file_path.clone()) else {
                continue;
            };
            let Some(sources) = boundary_input_sources.get(&source_file) else {
                continue;
            };
            relevant_sources = sources.clone();
            boundary_path = path.clone();
            boundary_context = Some(SecurityContext::BoundaryInputReachableViaGraph);
        }
        if relevant_sources.is_empty() {
            continue;
        }
        relevant_sources.sort_by(|left, right| {
            left.line
                .cmp(&right.line)
                .then(input_kind_rank(left.kind).cmp(&input_kind_rank(right.kind)))
                .then(left.evidence.cmp(&right.evidence))
        });
        relevant_sources.dedup_by(|left, right| {
            left.kind == right.kind && left.line == right.line && left.evidence == right.evidence
        });
        relevant_sources.truncate(4);
        let boundary_to_sink_flow = build_boundary_to_sink_flow(
            &relevant_sources,
            &boundary_path,
            &finding.file_path,
            finding.line,
        );
        finding.boundary_input_sources = relevant_sources;
        finding.boundary_input_path = boundary_path;
        finding.boundary_to_sink_flow = boundary_to_sink_flow;
        if let Some(boundary_context) = boundary_context {
            if !finding.contexts.contains(&boundary_context) {
                finding.contexts.push(boundary_context);
            }
        }
        finding.severity = escalate_for_boundary_input(finding.category, finding.severity);
        if !finding.message.contains("boundary-derived input") {
            if finding
                .contexts
                .contains(&SecurityContext::BoundaryInputReachableViaGraph)
            {
                finding
                    .message
                    .push_str(" with graph-reachable boundary-derived input");
            } else {
                finding
                    .message
                    .push_str(" with boundary-derived input in the same file");
            }
        }
    }
}

fn build_boundary_to_sink_flow(
    boundary_input_sources: &[SecurityInputSource],
    boundary_input_path: &[SecurityReachabilityHop],
    sink_file_path: &Path,
    sink_line: usize,
) -> Vec<SecurityFlowStep> {
    let mut flow = Vec::new();
    if let Some(primary_source) = boundary_input_sources.first() {
        flow.push(SecurityFlowStep {
            kind: SecurityFlowStepKind::BoundaryInput,
            file_path: primary_source.file_path.clone(),
            line: Some(primary_source.line),
            relation_to_next: None,
            source_symbol: None,
            target_symbol: None,
            reference_target_name: None,
            occurrence_index: 0,
            input_kind: Some(primary_source.kind),
            evidence: Some(primary_source.evidence.clone()),
        });
    }
    flow.extend(boundary_input_path.iter().map(|hop| SecurityFlowStep {
        kind: SecurityFlowStepKind::GraphHop,
        file_path: hop.file_path.clone(),
        line: hop.line,
        relation_to_next: hop.relation_to_next,
        source_symbol: hop.source_symbol.clone(),
        target_symbol: hop.target_symbol.clone(),
        reference_target_name: hop.reference_target_name.clone(),
        occurrence_index: hop.occurrence_index,
        input_kind: None,
        evidence: None,
    }));
    flow.push(SecurityFlowStep {
        kind: SecurityFlowStepKind::Sink,
        file_path: sink_file_path.to_path_buf(),
        line: Some(sink_line),
        relation_to_next: None,
        source_symbol: None,
        target_symbol: None,
        reference_target_name: None,
        occurrence_index: 0,
        input_kind: None,
        evidence: None,
    });
    flow
}

fn classify_security_contexts(
    path: &Path,
    category: SecurityCategory,
    externally_reachable: bool,
    graph_reachable: bool,
) -> Vec<SecurityContext> {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let mut contexts = Vec::new();

    if externally_reachable {
        contexts.push(SecurityContext::ExternallyReachable);
    } else if graph_reachable {
        contexts.push(SecurityContext::EntryReachableViaGraph);
    }
    if normalized.contains("/management/commands/") || normalized.ends_with("/shell.py") {
        contexts.push(SecurityContext::InteractiveExecution);
    }
    if category == SecurityCategory::UnsafeDeserialization && normalized.contains("/cache/") {
        contexts.push(SecurityContext::CacheStorage);
    }
    if normalized.contains("/db/backends/") {
        contexts.push(SecurityContext::DatabaseTooling);
    }
    if normalized.contains("/migrations/") {
        contexts.push(SecurityContext::MigrationSupport);
    }
    if normalized.contains("/autoreload")
        || normalized.ends_with("/version.py")
        || normalized.contains("/management/utils.py")
    {
        contexts.push(SecurityContext::DevelopmentRuntime);
    }

    contexts
}

fn collect_boundary_input_sources(path: &Path, content: &str) -> Vec<SecurityInputSource> {
    let Some(language) = detect_language(path) else {
        return Vec::new();
    };
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line_no = index + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() || is_comment_line(language, trimmed) {
                return None;
            }
            classify_boundary_input_kind(language, trimmed).map(|kind| SecurityInputSource {
                kind,
                file_path: path.to_path_buf(),
                line: line_no,
                evidence: trimmed.to_string(),
            })
        })
        .collect()
}

fn classify_boundary_input_kind(language: SourceLanguage, line: &str) -> Option<SecurityInputKind> {
    let lower = line.to_ascii_lowercase();
    match language {
        SourceLanguage::Php => {
            if lower.contains("$_get[")
                || lower.contains("->query(")
                || lower.contains("request()->query(")
            {
                return Some(SecurityInputKind::RequestQuery);
            }
            if lower.contains("$_post[")
                || lower.contains("->post(")
                || lower.contains("request()->post(")
                || lower.contains("->json(")
                || lower.contains("request()->json(")
                || lower.contains("->getcontent(")
                || lower.contains("request()->getcontent(")
            {
                return Some(SecurityInputKind::RequestBody);
            }
            if lower.contains("$_request[")
                || lower.contains("->input(")
                || lower.contains("request()->input(")
            {
                return Some(SecurityInputKind::RequestInput);
            }
            if lower.contains("->route(") || lower.contains("request()->route(") {
                return Some(SecurityInputKind::RequestParams);
            }
            if lower.contains("$argv") {
                return Some(SecurityInputKind::CliArgument);
            }
        }
        SourceLanguage::Python => {
            if line.contains("request.GET") || lower.contains("request.args") {
                return Some(SecurityInputKind::RequestQuery);
            }
            if line.contains("request.POST")
                || lower.contains("request.body")
                || lower.contains("request.data")
                || lower.contains("request.form")
                || lower.contains("request.get_json(")
            {
                return Some(SecurityInputKind::RequestBody);
            }
            if lower.contains("request.query_params") {
                return Some(SecurityInputKind::RequestParams);
            }
            if lower.contains("sys.argv") {
                return Some(SecurityInputKind::CliArgument);
            }
        }
        SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
            if lower.contains("req.query")
                || lower.contains("request.query")
                || lower.contains("ctx.request.query")
            {
                return Some(SecurityInputKind::RequestQuery);
            }
            if lower.contains("req.body")
                || lower.contains("request.body")
                || lower.contains("ctx.request.body")
            {
                return Some(SecurityInputKind::RequestBody);
            }
            if lower.contains("req.params")
                || lower.contains("request.params")
                || lower.contains("ctx.params")
            {
                return Some(SecurityInputKind::RequestParams);
            }
            if lower.contains("process.argv") {
                return Some(SecurityInputKind::CliArgument);
            }
        }
        SourceLanguage::Ruby => {
            if lower.contains("request.query_parameters") {
                return Some(SecurityInputKind::RequestQuery);
            }
            if lower.contains("request.request_parameters") || lower.contains("request.body") {
                return Some(SecurityInputKind::RequestBody);
            }
            if lower.contains("params[")
                || lower.contains("params.fetch(")
                || lower.contains("request.params")
            {
                return Some(SecurityInputKind::RequestParams);
            }
            if line.contains("ARGV") {
                return Some(SecurityInputKind::CliArgument);
            }
        }
    }
    None
}

fn input_kind_rank(kind: SecurityInputKind) -> usize {
    match kind {
        SecurityInputKind::RequestQuery => 0,
        SecurityInputKind::RequestBody => 1,
        SecurityInputKind::RequestParams => 2,
        SecurityInputKind::RequestInput => 3,
        SecurityInputKind::CliArgument => 4,
    }
}

fn escalate_for_boundary_input(
    category: SecurityCategory,
    severity: SecuritySeverity,
) -> SecuritySeverity {
    match category {
        SecurityCategory::UnsafeHtmlOutput => match severity {
            SecuritySeverity::Low => SecuritySeverity::Medium,
            other => other,
        },
        SecurityCategory::CommandExecution
        | SecurityCategory::CodeInjection
        | SecurityCategory::UnsafeDeserialization => SecuritySeverity::High,
    }
}

fn classify_php_dangerous_api(line: &str) -> Option<(SecurityCategory, &'static str)> {
    if contains_php_free_function_call(
        line,
        &[
            "exec",
            "system",
            "passthru",
            "shell_exec",
            "proc_open",
            "popen",
        ],
    ) {
        return Some((SecurityCategory::CommandExecution, "php-command-exec"));
    }

    if contains_php_free_function_call(line, &["eval"]) || contains_php_assert_string_eval(line) {
        return Some((SecurityCategory::CodeInjection, "php-eval"));
    }

    if contains_php_free_function_call(line, &["unserialize"]) {
        return Some((SecurityCategory::UnsafeDeserialization, "php-unserialize"));
    }

    None
}

fn classify_python_dangerous_api(line: &str) -> Option<(SecurityCategory, &'static str)> {
    if python_command_exec_pattern().is_match(line) {
        return Some((SecurityCategory::CommandExecution, "python-command-exec"));
    }

    if contains_python_builtin_call(line, "eval") || contains_python_builtin_call(line, "exec") {
        return Some((SecurityCategory::CodeInjection, "python-eval"));
    }

    if python_pickle_pattern().is_match(line) {
        return Some((
            SecurityCategory::UnsafeDeserialization,
            "python-deserialize",
        ));
    }

    if python_yaml_load_pattern().is_match(line) && !is_safe_yaml_loader_usage(line) {
        return Some((
            SecurityCategory::UnsafeDeserialization,
            "python-deserialize",
        ));
    }

    None
}

fn classify_javascript_dangerous_api(line: &str) -> Option<(SecurityCategory, &'static str)> {
    let (category, api_name) = dangerous_api_patterns(SourceLanguage::JavaScript)
        .iter()
        .find_map(|(category, pattern, api_name)| {
            pattern.is_match(line).then_some((*category, *api_name))
        })?;

    if category == SecurityCategory::UnsafeHtmlOutput && is_static_html_assignment(line) {
        return None;
    }

    Some((category, api_name))
}

fn contains_php_free_function_call(line: &str, names: &[&str]) -> bool {
    names.iter().any(|name| {
        line.match_indices(name).any(|(index, _)| {
            is_php_call_boundary(line, index, name.len())
                && is_php_free_function_context(line, index)
        })
    })
}

fn contains_php_assert_string_eval(line: &str) -> bool {
    for (index, _) in line.match_indices("assert") {
        if !is_php_call_boundary(line, index, "assert".len())
            || !is_php_free_function_context(line, index)
        {
            continue;
        }

        let args = line[index + "assert".len()..].trim_start();
        if !args.starts_with('(') {
            continue;
        }
        let mut chars = args[1..].trim_start().chars();
        if matches!(chars.next(), Some('\'') | Some('"')) {
            return true;
        }
    }

    false
}

fn contains_python_builtin_call(line: &str, name: &str) -> bool {
    for (index, _) in line.match_indices(name) {
        if !is_python_call_boundary(line, index, name.len())
            || !is_python_free_function_context(line, index)
        {
            continue;
        }
        return true;
    }

    false
}

fn is_php_call_boundary(line: &str, start: usize, name_len: usize) -> bool {
    let before = line[..start].chars().next_back();
    if before
        .map(|value| value.is_ascii_alphanumeric() || value == '_')
        .unwrap_or(false)
    {
        return false;
    }

    line[start + name_len..].trim_start().starts_with('(')
}

fn is_php_free_function_context(line: &str, start: usize) -> bool {
    let prefix = line[..start].trim_end();
    if prefix.ends_with("->") || prefix.ends_with("::") {
        return false;
    }

    let lower_prefix = prefix.to_ascii_lowercase();
    !lower_prefix.ends_with("function")
}

fn is_python_call_boundary(line: &str, start: usize, name_len: usize) -> bool {
    let before = line[..start].chars().next_back();
    if before
        .map(|value| value.is_ascii_alphanumeric() || value == '_')
        .unwrap_or(false)
    {
        return false;
    }

    line[start + name_len..].trim_start().starts_with('(')
}

fn is_python_free_function_context(line: &str, start: usize) -> bool {
    let prefix = line[..start].trim_end();
    if prefix.ends_with('.') {
        return false;
    }

    !prefix.to_ascii_lowercase().ends_with("def")
}

fn dangerous_api_message(
    category: SecurityCategory,
    api_name: &str,
    contexts: &[SecurityContext],
) -> String {
    let suffix = security_context_suffix(contexts);
    match category {
        SecurityCategory::CommandExecution => {
            format!("Dangerous command execution API `{api_name}` used{suffix}")
        }
        SecurityCategory::CodeInjection => {
            format!("Dangerous code-evaluation API `{api_name}` used{suffix}")
        }
        SecurityCategory::UnsafeDeserialization => {
            format!("Unsafe deserialization API `{api_name}` used{suffix}")
        }
        SecurityCategory::UnsafeHtmlOutput => {
            format!("Unsafe HTML output API `{api_name}` used{suffix}")
        }
    }
}

fn security_context_suffix(contexts: &[SecurityContext]) -> String {
    if contexts.contains(&SecurityContext::ExternallyReachable) {
        return String::from(" in externally reachable code");
    }
    if contexts.contains(&SecurityContext::EntryReachableViaGraph) {
        return String::from(" in graph-reachable entry code");
    }
    if contexts.contains(&SecurityContext::BoundaryInputInSameFile) {
        return String::from(" in code with boundary-derived input");
    }
    if contexts.contains(&SecurityContext::BoundaryInputReachableViaGraph) {
        return String::from(" in code with graph-reachable boundary-derived input");
    }
    if contexts.contains(&SecurityContext::InteractiveExecution) {
        return String::from(" in interactive execution code");
    }
    if contexts.contains(&SecurityContext::CacheStorage) {
        return String::from(" in cache/storage code");
    }
    if contexts.contains(&SecurityContext::DatabaseTooling) {
        return String::from(" in database tooling");
    }
    if contexts.contains(&SecurityContext::MigrationSupport) {
        return String::from(" in migration support code");
    }
    if contexts.contains(&SecurityContext::DevelopmentRuntime) {
        return String::from(" in development/runtime tooling");
    }

    String::new()
}

fn entry_reachability_paths_via_graph(
    semantic_graph: &SemanticGraph,
    roots: &HashSet<PathBuf>,
) -> HashMap<PathBuf, Vec<SecurityReachabilityHop>> {
    reachability_paths_via_graph(semantic_graph, roots)
}

fn boundary_input_paths_via_graph(
    semantic_graph: &SemanticGraph,
    roots: &HashSet<PathBuf>,
) -> HashMap<PathBuf, Vec<SecurityReachabilityHop>> {
    reachability_paths_via_graph(semantic_graph, roots)
}

fn reachability_paths_via_graph(
    semantic_graph: &SemanticGraph,
    roots: &HashSet<PathBuf>,
) -> HashMap<PathBuf, Vec<SecurityReachabilityHop>> {
    const MAX_HOPS: usize = 8;
    #[derive(Clone)]
    struct ReachabilityEdge {
        target: PathBuf,
        line: usize,
        relation_kind: RelationKind,
        source_symbol: Option<String>,
        target_symbol: Option<String>,
        reference_target_name: Option<String>,
        occurrence_index: usize,
    }

    #[derive(Clone)]
    struct Predecessor {
        source: PathBuf,
        line: usize,
        relation_kind: RelationKind,
        source_symbol: Option<String>,
        target_symbol: Option<String>,
        reference_target_name: Option<String>,
        occurrence_index: usize,
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct PathScore {
        causal_points: usize,
        symbol_points: usize,
        occurrence_points: usize,
        depth_penalty: usize,
    }

    impl PathScore {
        fn root() -> Self {
            Self {
                causal_points: 0,
                symbol_points: 0,
                occurrence_points: 0,
                depth_penalty: 0,
            }
        }

        fn advance(self, edge: &ReachabilityEdge) -> Self {
            Self {
                causal_points: self.causal_points + reachability_relation_rank(edge.relation_kind),
                symbol_points: self.symbol_points
                    + usize::from(edge.source_symbol.is_some() && edge.target_symbol.is_some()),
                occurrence_points: self.occurrence_points
                    + usize::from(
                        edge.occurrence_index > 0 || edge.reference_target_name.is_some(),
                    ),
                depth_penalty: self.depth_penalty + 1,
            }
        }

        fn is_better_than(&self, other: &Self) -> bool {
            (
                self.causal_points,
                self.symbol_points,
                self.occurrence_points,
                usize::MAX - self.depth_penalty,
            ) > (
                other.causal_points,
                other.symbol_points,
                other.occurrence_points,
                usize::MAX - other.depth_penalty,
            )
        }
    }

    let symbol_names = semantic_graph
        .symbols
        .iter()
        .map(|symbol| (symbol.id.clone(), symbol.qualified_name.clone()))
        .collect::<HashMap<_, _>>();
    let mut outbound = HashMap::<&Path, Vec<ReachabilityEdge>>::new();
    for edge in &semantic_graph.resolved_edges {
        if !supports_security_entry_reachability(edge.relation_kind) {
            continue;
        }
        outbound
            .entry(edge.source_file_path.as_path())
            .or_default()
            .push(ReachabilityEdge {
                target: edge.target_file_path.clone(),
                line: edge.line,
                relation_kind: edge.relation_kind,
                source_symbol: edge
                    .source_symbol_id
                    .as_ref()
                    .and_then(|id| symbol_names.get(id))
                    .cloned(),
                target_symbol: symbol_names.get(&edge.target_symbol_id).cloned(),
                reference_target_name: edge.reference_target_name.clone(),
                occurrence_index: edge.occurrence_index,
            });
    }
    for targets in outbound.values_mut() {
        targets.sort_by(|left, right| {
            reachability_relation_rank(right.relation_kind)
                .cmp(&reachability_relation_rank(left.relation_kind))
                .then(
                    usize::from(right.source_symbol.is_some() && right.target_symbol.is_some())
                        .cmp(&usize::from(
                            left.source_symbol.is_some() && left.target_symbol.is_some(),
                        )),
                )
                .then(
                    usize::from(
                        right.occurrence_index > 0 || right.reference_target_name.is_some(),
                    )
                    .cmp(&usize::from(
                        left.occurrence_index > 0 || left.reference_target_name.is_some(),
                    )),
                )
                .then(left.line.cmp(&right.line))
                .then(left.target.cmp(&right.target))
        });
    }

    let mut best_scores = roots
        .iter()
        .cloned()
        .map(|path| (path, PathScore::root()))
        .collect::<HashMap<_, _>>();
    let mut predecessors = HashMap::<PathBuf, Predecessor>::new();
    let predecessor_chain_contains =
        |start: &Path, needle: &Path, predecessors: &HashMap<PathBuf, Predecessor>| {
            let mut cursor = start.to_path_buf();
            while let Some(previous) = predecessors.get(&cursor) {
                if previous.source == needle {
                    return true;
                }
                cursor = previous.source.clone();
            }
            false
        };

    let mut queue = roots
        .iter()
        .cloned()
        .map(|path| (path, 0usize))
        .collect::<VecDeque<_>>();

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= MAX_HOPS {
            continue;
        }
        let Some(current_score) = best_scores.get(&current).copied() else {
            continue;
        };
        let Some(targets) = outbound.get(current.as_path()) else {
            continue;
        };
        for target in targets {
            let target_path = target.target.clone();
            if target_path == current
                || predecessor_chain_contains(
                    current.as_path(),
                    target_path.as_path(),
                    &predecessors,
                )
            {
                continue;
            }
            let candidate_score = current_score.advance(target);
            let should_replace = best_scores
                .get(&target_path)
                .map(|existing| candidate_score.is_better_than(existing))
                .unwrap_or(true);
            if !should_replace {
                continue;
            }
            best_scores.insert(target_path.clone(), candidate_score);
            if !roots.contains(&target_path) {
                predecessors.insert(
                    target_path.clone(),
                    Predecessor {
                        source: current.clone(),
                        line: target.line,
                        relation_kind: target.relation_kind,
                        source_symbol: target.source_symbol.clone(),
                        target_symbol: target.target_symbol.clone(),
                        reference_target_name: target.reference_target_name.clone(),
                        occurrence_index: target.occurrence_index,
                    },
                );
            }
            queue.push_back((target_path, depth + 1));
        }
    }

    predecessors
        .keys()
        .cloned()
        .map(|target| {
            let mut chain = Vec::new();
            let mut cursor = target.clone();
            chain.push(cursor.clone());
            while let Some(previous) = predecessors.get(&cursor) {
                cursor = previous.source.clone();
                chain.push(cursor.clone());
            }
            chain.reverse();

            let hops = chain
                .iter()
                .enumerate()
                .map(|(index, file_path)| {
                    if let Some(next) = chain.get(index + 1) {
                        let predecessor = predecessors
                            .get(next)
                            .expect("security reachability predecessor should exist");
                        SecurityReachabilityHop {
                            file_path: file_path.clone(),
                            line: Some(predecessor.line),
                            relation_to_next: Some(predecessor.relation_kind),
                            source_symbol: predecessor.source_symbol.clone(),
                            target_symbol: predecessor.target_symbol.clone(),
                            reference_target_name: predecessor.reference_target_name.clone(),
                            occurrence_index: predecessor.occurrence_index,
                        }
                    } else {
                        SecurityReachabilityHop {
                            file_path: file_path.clone(),
                            line: None,
                            relation_to_next: None,
                            source_symbol: None,
                            target_symbol: None,
                            reference_target_name: None,
                            occurrence_index: 0,
                        }
                    }
                })
                .collect::<Vec<_>>();

            (target, hops)
        })
        .collect()
}

fn reachability_relation_rank(kind: RelationKind) -> usize {
    match kind {
        RelationKind::Call => 5,
        RelationKind::Dispatch => 4,
        RelationKind::ContainerResolution => 3,
        RelationKind::EventPublish => 2,
        RelationKind::Import => 1,
        _ => 0,
    }
}

fn supports_security_entry_reachability(kind: RelationKind) -> bool {
    matches!(
        kind,
        RelationKind::Import
            | RelationKind::Call
            | RelationKind::Dispatch
            | RelationKind::ContainerResolution
            | RelationKind::EventPublish
    )
}

fn dangerous_api_patterns(
    language: SourceLanguage,
) -> &'static [(SecurityCategory, Regex, &'static str)] {
    match language {
        SourceLanguage::Php => php_patterns(),
        SourceLanguage::Python => python_patterns(),
        SourceLanguage::Ruby => ruby_patterns(),
        SourceLanguage::JavaScript | SourceLanguage::TypeScript => javascript_patterns(),
    }
}

fn php_patterns() -> &'static [(SecurityCategory, Regex, &'static str)] {
    static PATTERNS: OnceLock<Vec<(SecurityCategory, Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![
            (
                SecurityCategory::CommandExecution,
                Regex::new(r"\b(exec|system|passthru|shell_exec|proc_open|popen)\s*\(")
                    .expect("valid regex"),
                "php-command-exec",
            ),
            (
                SecurityCategory::CodeInjection,
                Regex::new(r"\b(eval|assert)\s*\(").expect("valid regex"),
                "php-eval",
            ),
            (
                SecurityCategory::UnsafeDeserialization,
                Regex::new(r"\bunserialize\s*\(").expect("valid regex"),
                "php-unserialize",
            ),
        ]
    })
}

fn python_patterns() -> &'static [(SecurityCategory, Regex, &'static str)] {
    static PATTERNS: OnceLock<Vec<(SecurityCategory, Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![(
            SecurityCategory::CommandExecution,
            Regex::new(
                r"\bos\.system\s*\(|\bsubprocess\.(Popen|call|run|check_call|check_output)\s*\(",
            )
            .expect("valid regex"),
            "python-command-exec",
        )]
    })
}

fn ruby_patterns() -> &'static [(SecurityCategory, Regex, &'static str)] {
    static PATTERNS: OnceLock<Vec<(SecurityCategory, Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![
            (
                SecurityCategory::CommandExecution,
                Regex::new(r"\b(system|exec)\s*\(|\bIO\.popen\s*\(").expect("valid regex"),
                "ruby-command-exec",
            ),
            (
                SecurityCategory::CodeInjection,
                Regex::new(r"\b(eval|instance_eval|class_eval)\s*\(").expect("valid regex"),
                "ruby-eval",
            ),
            (
                SecurityCategory::UnsafeDeserialization,
                Regex::new(r"\b(Marshal|YAML)\.load\s*\(").expect("valid regex"),
                "ruby-deserialize",
            ),
        ]
    })
}

fn javascript_patterns() -> &'static [(SecurityCategory, Regex, &'static str)] {
    static PATTERNS: OnceLock<Vec<(SecurityCategory, Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![
            (
                SecurityCategory::CommandExecution,
                Regex::new(r"\bchild_process\.(exec|execSync)\s*\(").expect("valid regex"),
                "javascript-command-exec",
            ),
            (
                SecurityCategory::CodeInjection,
                Regex::new(r"\beval\s*\(|\bnew\s+Function\s*\(").expect("valid regex"),
                "javascript-eval",
            ),
            (
                SecurityCategory::UnsafeHtmlOutput,
                Regex::new(r"\.\s*(innerHTML|outerHTML)\s*=|\bdocument\.write\s*\(")
                    .expect("valid regex"),
                "javascript-html-output",
            ),
        ]
    })
}

fn python_command_exec_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"\bos\.system\s*\(|\bsubprocess\.(Popen|call|run|check_call|check_output)\s*\(")
            .expect("valid regex")
    })
}

fn python_pickle_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\bpickle\.(load|loads)\s*\(").expect("valid regex"))
}

fn python_yaml_load_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\byaml\.load\s*\(").expect("valid regex"))
}

fn is_safe_yaml_loader_usage(line: &str) -> bool {
    line.contains("SafeLoader") || line.contains("CSafeLoader")
}

fn is_static_html_assignment(line: &str) -> bool {
    let normalized = line.replace(' ', "");
    normalized.contains(".innerHTML=''")
        || normalized.contains(".innerHTML=\"\"")
        || normalized.contains(".innerHTML=``")
        || normalized.contains(".outerHTML=''")
        || normalized.contains(".outerHTML=\"\"")
        || normalized.contains(".outerHTML=``")
}

fn is_comment_line(language: SourceLanguage, trimmed: &str) -> bool {
    match language {
        SourceLanguage::Php | SourceLanguage::JavaScript | SourceLanguage::TypeScript => {
            trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
                || trimmed.starts_with("*/")
        }
        SourceLanguage::Python | SourceLanguage::Ruby => trimmed.starts_with('#'),
    }
}

fn security_category_label(category: SecurityCategory) -> &'static str {
    match category {
        SecurityCategory::CommandExecution => "command_execution",
        SecurityCategory::CodeInjection => "code_injection",
        SecurityCategory::UnsafeDeserialization => "unsafe_deserialization",
        SecurityCategory::UnsafeHtmlOutput => "unsafe_html_output",
    }
}

fn severity_rank(severity: SecuritySeverity) -> u8 {
    match severity {
        SecuritySeverity::High => 3,
        SecuritySeverity::Medium => 2,
        SecuritySeverity::Low => 1,
    }
}

fn is_test_like_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().to_ascii_lowercase();
    [
        "test/",
        "tests/",
        "/test/",
        "/tests/",
        "/__tests__/",
        "/spec/",
        "/specs/",
        "/fixtures/",
    ]
    .iter()
    .any(|fragment| normalized.contains(fragment))
        || normalized.ends_with(".test.js")
        || normalized.ends_with(".test.ts")
        || normalized.ends_with(".spec.js")
        || normalized.ends_with(".spec.ts")
        || normalized.ends_with("_test.py")
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_security_findings, analyze_security_findings_with_ast_grep,
        analyze_security_findings_with_graph, SecurityCategory, SecurityContext,
        SecurityFindingKind, SecurityInputKind, SecuritySeverity,
    };
    use crate::contracts::build_contract_inventory;
    use crate::graph::{
        EdgeOrigin, EdgeStrength, GraphLayer, ReferenceKind, RelationKind, ResolutionTier,
        ResolvedEdge, SemanticGraph,
    };
    use crate::scanners::ast_grep::run_ast_grep_scan;
    use std::path::Path;

    #[test]
    fn detects_dangerous_php_apis_and_escalates_in_reachable_files() {
        let parsed_sources = vec![
            (
                Path::new("app/routes.php").to_path_buf(),
                String::from(
                    r#"Route::post('/admin/run', function () {
    system($command);
});"#,
                ),
            ),
            (
                Path::new("app/worker.php").to_path_buf(),
                String::from("system($command);\n"),
            ),
        ];
        let inventory = build_contract_inventory(&parsed_sources);

        let result = analyze_security_findings(&parsed_sources, &inventory, &[]);

        assert_eq!(result.findings.len(), 2);
        assert_eq!(result.findings[0].kind, SecurityFindingKind::DangerousApi);
        assert_eq!(
            result.findings[0].category,
            SecurityCategory::CommandExecution
        );
        assert_eq!(result.findings[0].severity, SecuritySeverity::High);
        assert_eq!(result.findings[1].severity, SecuritySeverity::Medium);
    }

    #[test]
    fn ignores_comment_lines_and_detects_html_output_as_lower_severity() {
        let parsed_sources = vec![(
            Path::new("resources/app.js").to_path_buf(),
            String::from(
                r#"// element.innerHTML = userValue;
target.innerHTML = html;
"#,
            ),
        )];
        let inventory = build_contract_inventory(&parsed_sources);

        let result = analyze_security_findings(&parsed_sources, &inventory, &[]);

        assert_eq!(result.findings.len(), 1);
        assert_eq!(
            result.findings[0].category,
            SecurityCategory::UnsafeHtmlOutput
        );
        assert_eq!(result.findings[0].severity, SecuritySeverity::Low);
        assert_eq!(result.findings[0].line, 2);
    }

    #[test]
    fn ignores_static_html_resets() {
        let parsed_sources = vec![(
            Path::new("contrib/admin/static/admin/js/SelectBox.js").to_path_buf(),
            String::from("box.innerHTML = '';\n"),
        )];
        let inventory = build_contract_inventory(&parsed_sources);

        let result = analyze_security_findings(&parsed_sources, &inventory, &[]);

        assert!(result.findings.is_empty());
    }

    #[test]
    fn ignores_php_member_calls_and_method_declarations() {
        let parsed_sources = vec![(
            Path::new("wp-includes/sample.php").to_path_buf(),
            String::from(
                r#"public function unserialize($data) {}
$query = $this->mysql->exec('CREATE TABLE demo');
assert($this->body !== null);
$output = shell_exec($commandline);
$data = unserialize($payload);
"#,
            ),
        )];
        let inventory = build_contract_inventory(&parsed_sources);

        let result = analyze_security_findings(&parsed_sources, &inventory, &[]);

        assert_eq!(result.findings.len(), 2);
        assert_eq!(
            result.findings[0].category,
            SecurityCategory::CommandExecution
        );
        assert_eq!(
            result.findings[1].category,
            SecurityCategory::UnsafeDeserialization
        );
    }

    #[test]
    fn detects_php_assert_only_for_string_arguments() {
        let parsed_sources = vec![(
            Path::new("wp-includes/assertions.php").to_path_buf(),
            String::from(
                r#"assert($this->body !== null);
assert('phpinfo();');
"#,
            ),
        )];
        let inventory = build_contract_inventory(&parsed_sources);

        let result = analyze_security_findings(&parsed_sources, &inventory, &[]);

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].category, SecurityCategory::CodeInjection);
    }

    #[test]
    fn ignores_python_member_eval_and_safe_yaml_load() {
        let parsed_sources = vec![(
            Path::new("django/template/smartif.py").to_path_buf(),
            String::from(
                r#"def eval(self, context):
    return self.value
x.eval(context)
objects = yaml.load(stream, Loader=SafeLoader)
exec(user_input)
"#,
            ),
        )];
        let inventory = build_contract_inventory(&parsed_sources);

        let result = analyze_security_findings(&parsed_sources, &inventory, &[]);

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].category, SecurityCategory::CodeInjection);
        assert_eq!(result.findings[0].evidence, "exec(user_input)");
    }

    #[test]
    fn ignores_test_prefix_paths() {
        let parsed_sources = vec![(
            Path::new("test/runner.py").to_path_buf(),
            String::from("pickle.loads(pickle.dumps(obj))\n"),
        )];
        let inventory = build_contract_inventory(&parsed_sources);

        let result = analyze_security_findings(&parsed_sources, &inventory, &[]);

        assert!(result.findings.is_empty());
    }

    #[test]
    fn tags_security_contexts_for_django_style_paths() {
        let parsed_sources = vec![
            (
                Path::new("core/cache/backends/filebased.py").to_path_buf(),
                String::from("return pickle.loads(data)\n"),
            ),
            (
                Path::new("core/management/commands/shell.py").to_path_buf(),
                String::from("exec(options['command'])\n"),
            ),
        ];
        let inventory = build_contract_inventory(&parsed_sources);

        let result = analyze_security_findings(&parsed_sources, &inventory, &[]);

        assert_eq!(result.findings.len(), 2);
        assert!(result.findings[0]
            .contexts
            .contains(&SecurityContext::CacheStorage));
        assert!(result.findings[0].message.contains("cache/storage"));
        assert!(result.findings[1]
            .contexts
            .contains(&SecurityContext::InteractiveExecution));
        assert!(result.findings[1].message.contains("interactive execution"));
    }

    #[test]
    fn ast_grep_reinforces_native_dangerous_api_findings() {
        let parsed_sources = vec![(
            Path::new("resources/admin.js").to_path_buf(),
            String::from("export function run(input) { return eval(input); }\n"),
        )];
        let inventory = build_contract_inventory(&parsed_sources);
        let ast_grep_scan = run_ast_grep_scan(&parsed_sources);

        let result = analyze_security_findings_with_ast_grep(
            &parsed_sources,
            &inventory,
            &[],
            &ast_grep_scan,
        );

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].kind, SecurityFindingKind::DangerousApi);
        assert_eq!(result.findings[0].category, SecurityCategory::CodeInjection);
        assert!(result.findings[0]
            .supporting_scanners
            .iter()
            .any(|scanner| scanner == "ast_grep"));
    }

    #[test]
    fn graph_reachable_sink_escalates_even_when_not_in_direct_route_file() {
        let parsed_sources = vec![
            (
                Path::new("app/routes.php").to_path_buf(),
                String::from("Route::post('/admin/run', [Runner::class, 'run']);\n"),
            ),
            (
                Path::new("app/runner.php").to_path_buf(),
                String::from("system($command);\n"),
            ),
        ];
        let inventory = build_contract_inventory(&parsed_sources);
        let semantic_graph = SemanticGraph {
            resolved_edges: vec![ResolvedEdge {
                source_file_path: Path::new("app/routes.php").to_path_buf(),
                source_symbol_id: None,
                target_file_path: Path::new("app/runner.php").to_path_buf(),
                target_symbol_id: String::from("function:app/runner.php:run"),
                reference_target_name: Some(String::from("Runner")),
                kind: ReferenceKind::Import,
                relation_kind: RelationKind::Import,
                layer: GraphLayer::Structural,
                strength: EdgeStrength::Hard,
                origin: EdgeOrigin::Resolver,
                resolution_tier: ResolutionTier::ImportScoped,
                confidence_millis: 900,
                reason: String::from("import:import-scoped"),
                line: 1,
                occurrence_index: 0,
            }],
            ..SemanticGraph::default()
        };

        let result =
            analyze_security_findings_with_graph(&parsed_sources, &inventory, &[], &semantic_graph);

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].file_path, Path::new("app/runner.php"));
        assert_eq!(result.findings[0].severity, SecuritySeverity::High);
        assert!(result.findings[0]
            .contexts
            .contains(&SecurityContext::EntryReachableViaGraph));
        assert!(result.findings[0]
            .message
            .contains("graph-reachable entry code"));
        assert_eq!(
            result.findings[0]
                .reachability_path
                .iter()
                .map(|hop| hop.file_path.clone())
                .collect::<Vec<_>>(),
            vec![
                Path::new("app/routes.php").to_path_buf(),
                Path::new("app/runner.php").to_path_buf()
            ]
        );
        assert_eq!(result.findings[0].reachability_path[0].line, Some(1));
        assert_eq!(
            result.findings[0].reachability_path[0].relation_to_next,
            Some(RelationKind::Import)
        );
        assert_eq!(result.findings[0].reachability_path[1].line, Some(1));
    }

    #[test]
    fn same_file_boundary_input_escalates_dangerous_api_and_records_source() {
        let parsed_sources = vec![(
            Path::new("src/handler.ts").to_path_buf(),
            String::from(
                r#"
export function handler(req: any) {
  const script = req.body.script;
  eval(script);
}
"#,
            ),
        )];
        let inventory = build_contract_inventory(&parsed_sources);

        let result = analyze_security_findings(&parsed_sources, &inventory, &[]);
        let finding = result
            .findings
            .iter()
            .find(|finding| finding.category == SecurityCategory::CodeInjection)
            .expect("expected code injection finding");

        assert_eq!(finding.severity, SecuritySeverity::High);
        assert!(finding
            .contexts
            .contains(&SecurityContext::BoundaryInputInSameFile));
        assert!(finding.message.contains("boundary-derived input"));
        assert_eq!(finding.boundary_input_sources.len(), 1);
        assert_eq!(
            finding.boundary_input_sources[0].kind,
            SecurityInputKind::RequestBody
        );
        assert_eq!(finding.boundary_input_sources[0].line, 3);
    }

    #[test]
    fn graph_reachable_boundary_input_records_multi_file_provenance() {
        let parsed_sources = vec![
            (
                Path::new("src/handler.ts").to_path_buf(),
                String::from(
                    r#"
import { run } from './runner';

export function handler(req: any) {
  const script = req.body.script;
  return run(script);
}
"#,
                ),
            ),
            (
                Path::new("src/runner.ts").to_path_buf(),
                String::from(
                    r#"
export function run(script: string) {
  return eval(script);
}
"#,
                ),
            ),
        ];
        let inventory = build_contract_inventory(&parsed_sources);
        let semantic_graph = SemanticGraph {
            resolved_edges: vec![ResolvedEdge {
                source_file_path: Path::new("src/handler.ts").to_path_buf(),
                source_symbol_id: None,
                target_file_path: Path::new("src/runner.ts").to_path_buf(),
                target_symbol_id: String::from("function:src/runner.ts:run"),
                reference_target_name: Some(String::from("run")),
                kind: ReferenceKind::Import,
                relation_kind: RelationKind::Import,
                layer: GraphLayer::Structural,
                strength: EdgeStrength::Hard,
                origin: EdgeOrigin::Resolver,
                resolution_tier: ResolutionTier::ImportScoped,
                confidence_millis: 900,
                reason: String::from("import:import-scoped"),
                line: 1,
                occurrence_index: 0,
            }],
            ..SemanticGraph::default()
        };

        let result =
            analyze_security_findings_with_graph(&parsed_sources, &inventory, &[], &semantic_graph);
        let finding = result
            .findings
            .iter()
            .find(|finding| finding.file_path == Path::new("src/runner.ts"))
            .expect("expected runner finding");

        assert_eq!(finding.severity, SecuritySeverity::High);
        assert!(finding
            .contexts
            .contains(&SecurityContext::BoundaryInputReachableViaGraph));
        assert!(finding
            .message
            .contains("graph-reachable boundary-derived input"));
        assert_eq!(finding.boundary_input_sources.len(), 1);
        assert_eq!(
            finding.boundary_input_sources[0].kind,
            SecurityInputKind::RequestBody
        );
        assert_eq!(
            finding.boundary_input_sources[0].file_path,
            Path::new("src/handler.ts")
        );
        assert_eq!(finding.boundary_input_path.len(), 2);
        assert_eq!(
            finding.boundary_input_path[0].file_path,
            Path::new("src/handler.ts")
        );
        assert_eq!(finding.boundary_input_path[0].line, Some(1));
        assert_eq!(
            finding.boundary_input_path[0].relation_to_next,
            Some(RelationKind::Import)
        );
        assert_eq!(
            finding.boundary_input_path[1].file_path,
            Path::new("src/runner.ts")
        );
    }

    #[test]
    fn security_paths_prefer_call_edges_over_plain_imports_when_both_exist() {
        let parsed_sources = vec![
            (
                Path::new("src/handler.ts").to_path_buf(),
                String::from(
                    r#"
import { run } from './runner';

export function handler(req: any) {
  const script = req.body.script;
  return run(script);
}
"#,
                ),
            ),
            (
                Path::new("src/runner.ts").to_path_buf(),
                String::from(
                    r#"
export function run(script: string) {
  return eval(script);
}
"#,
                ),
            ),
        ];
        let inventory = build_contract_inventory(&parsed_sources);
        let semantic_graph = SemanticGraph {
            resolved_edges: vec![
                ResolvedEdge {
                    source_file_path: Path::new("src/handler.ts").to_path_buf(),
                    source_symbol_id: None,
                    target_file_path: Path::new("src/runner.ts").to_path_buf(),
                    target_symbol_id: String::from("function:src/runner.ts:run"),
                    reference_target_name: Some(String::from("run")),
                    kind: ReferenceKind::Import,
                    relation_kind: RelationKind::Import,
                    layer: GraphLayer::Structural,
                    strength: EdgeStrength::Hard,
                    origin: EdgeOrigin::Resolver,
                    resolution_tier: ResolutionTier::ImportScoped,
                    confidence_millis: 900,
                    reason: String::from("import:import-scoped"),
                    line: 1,
                    occurrence_index: 0,
                },
                ResolvedEdge {
                    source_file_path: Path::new("src/handler.ts").to_path_buf(),
                    source_symbol_id: Some(String::from("function:src/handler.ts:handler")),
                    target_file_path: Path::new("src/runner.ts").to_path_buf(),
                    target_symbol_id: String::from("function:src/runner.ts:run"),
                    reference_target_name: Some(String::from("run")),
                    kind: ReferenceKind::Call,
                    relation_kind: RelationKind::Call,
                    layer: GraphLayer::Structural,
                    strength: EdgeStrength::Hard,
                    origin: EdgeOrigin::Resolver,
                    resolution_tier: ResolutionTier::ImportScoped,
                    confidence_millis: 920,
                    reason: String::from("call:import-scoped"),
                    line: 5,
                    occurrence_index: 0,
                },
            ],
            symbols: vec![
                crate::graph::SymbolNode {
                    id: String::from("function:src/handler.ts:handler"),
                    file_path: Path::new("src/handler.ts").to_path_buf(),
                    kind: crate::graph::SymbolKind::Function,
                    name: String::from("handler"),
                    qualified_name: String::from("handler"),
                    parent_symbol_id: None,
                    owner_type_name: None,
                    return_type_name: None,
                    visibility: crate::graph::Visibility::Public,
                    parameter_count: 1,
                    required_parameter_count: 1,
                    start_line: 4,
                    end_line: 6,
                },
                crate::graph::SymbolNode {
                    id: String::from("function:src/runner.ts:run"),
                    file_path: Path::new("src/runner.ts").to_path_buf(),
                    kind: crate::graph::SymbolKind::Function,
                    name: String::from("run"),
                    qualified_name: String::from("run"),
                    parent_symbol_id: None,
                    owner_type_name: None,
                    return_type_name: None,
                    visibility: crate::graph::Visibility::Public,
                    parameter_count: 1,
                    required_parameter_count: 1,
                    start_line: 2,
                    end_line: 4,
                },
            ],
            ..SemanticGraph::default()
        };

        let result =
            analyze_security_findings_with_graph(&parsed_sources, &inventory, &[], &semantic_graph);
        let finding = result
            .findings
            .iter()
            .find(|finding| finding.file_path == Path::new("src/runner.ts"))
            .expect("expected runner finding");

        assert_eq!(
            finding.boundary_input_path[0].relation_to_next,
            Some(RelationKind::Call)
        );
        assert_eq!(
            finding.boundary_input_path[0].source_symbol.as_deref(),
            Some("handler")
        );
        assert_eq!(
            finding.boundary_input_path[0].target_symbol.as_deref(),
            Some("run")
        );
        assert_eq!(
            finding.boundary_input_path[0]
                .reference_target_name
                .as_deref(),
            Some("run")
        );
        assert_eq!(finding.boundary_input_path[0].line, Some(5));
    }

    #[test]
    fn security_paths_prefer_more_causal_multi_hop_chain_over_shorter_import_path() {
        let parsed_sources = vec![
            (
                Path::new("src/main.ts").to_path_buf(),
                String::from(
                    r#"
import { handle } from './handler';
import { run } from './runner';

export function main(req: any) {
  const script = req.body.script;
  handle(script);
  return run('static');
}
"#,
                ),
            ),
            (
                Path::new("src/handler.ts").to_path_buf(),
                String::from(
                    r#"
import { run } from './runner';

export function handle(script: string) {
  return run(script);
}
"#,
                ),
            ),
            (
                Path::new("src/runner.ts").to_path_buf(),
                String::from(
                    r#"
export function run(script: string) {
  return eval(script);
}
"#,
                ),
            ),
        ];
        let inventory = build_contract_inventory(&parsed_sources);
        let semantic_graph = SemanticGraph {
            resolved_edges: vec![
                ResolvedEdge {
                    source_file_path: Path::new("src/main.ts").to_path_buf(),
                    source_symbol_id: None,
                    target_file_path: Path::new("src/runner.ts").to_path_buf(),
                    target_symbol_id: String::from("function:src/runner.ts:run"),
                    reference_target_name: Some(String::from("run")),
                    kind: ReferenceKind::Import,
                    relation_kind: RelationKind::Import,
                    layer: GraphLayer::Structural,
                    strength: EdgeStrength::Hard,
                    origin: EdgeOrigin::Resolver,
                    resolution_tier: ResolutionTier::ImportScoped,
                    confidence_millis: 900,
                    reason: String::from("import:direct"),
                    line: 3,
                    occurrence_index: 0,
                },
                ResolvedEdge {
                    source_file_path: Path::new("src/main.ts").to_path_buf(),
                    source_symbol_id: Some(String::from("function:src/main.ts:main")),
                    target_file_path: Path::new("src/handler.ts").to_path_buf(),
                    target_symbol_id: String::from("function:src/handler.ts:handle"),
                    reference_target_name: Some(String::from("handle")),
                    kind: ReferenceKind::Call,
                    relation_kind: RelationKind::Call,
                    layer: GraphLayer::Structural,
                    strength: EdgeStrength::Hard,
                    origin: EdgeOrigin::Resolver,
                    resolution_tier: ResolutionTier::ImportScoped,
                    confidence_millis: 940,
                    reason: String::from("call:handle"),
                    line: 6,
                    occurrence_index: 1,
                },
                ResolvedEdge {
                    source_file_path: Path::new("src/handler.ts").to_path_buf(),
                    source_symbol_id: Some(String::from("function:src/handler.ts:handle")),
                    target_file_path: Path::new("src/runner.ts").to_path_buf(),
                    target_symbol_id: String::from("function:src/runner.ts:run"),
                    reference_target_name: Some(String::from("run")),
                    kind: ReferenceKind::Call,
                    relation_kind: RelationKind::Call,
                    layer: GraphLayer::Structural,
                    strength: EdgeStrength::Hard,
                    origin: EdgeOrigin::Resolver,
                    resolution_tier: ResolutionTier::ImportScoped,
                    confidence_millis: 950,
                    reason: String::from("call:run"),
                    line: 5,
                    occurrence_index: 2,
                },
            ],
            symbols: vec![
                crate::graph::SymbolNode {
                    id: String::from("function:src/main.ts:main"),
                    file_path: Path::new("src/main.ts").to_path_buf(),
                    kind: crate::graph::SymbolKind::Function,
                    name: String::from("main"),
                    qualified_name: String::from("main"),
                    parent_symbol_id: None,
                    owner_type_name: None,
                    return_type_name: None,
                    visibility: crate::graph::Visibility::Public,
                    parameter_count: 1,
                    required_parameter_count: 1,
                    start_line: 5,
                    end_line: 8,
                },
                crate::graph::SymbolNode {
                    id: String::from("function:src/handler.ts:handle"),
                    file_path: Path::new("src/handler.ts").to_path_buf(),
                    kind: crate::graph::SymbolKind::Function,
                    name: String::from("handle"),
                    qualified_name: String::from("handle"),
                    parent_symbol_id: None,
                    owner_type_name: None,
                    return_type_name: None,
                    visibility: crate::graph::Visibility::Public,
                    parameter_count: 1,
                    required_parameter_count: 1,
                    start_line: 4,
                    end_line: 6,
                },
                crate::graph::SymbolNode {
                    id: String::from("function:src/runner.ts:run"),
                    file_path: Path::new("src/runner.ts").to_path_buf(),
                    kind: crate::graph::SymbolKind::Function,
                    name: String::from("run"),
                    qualified_name: String::from("run"),
                    parent_symbol_id: None,
                    owner_type_name: None,
                    return_type_name: None,
                    visibility: crate::graph::Visibility::Public,
                    parameter_count: 1,
                    required_parameter_count: 1,
                    start_line: 2,
                    end_line: 4,
                },
            ],
            ..SemanticGraph::default()
        };

        let result =
            analyze_security_findings_with_graph(&parsed_sources, &inventory, &[], &semantic_graph);
        let finding = result
            .findings
            .iter()
            .find(|finding| finding.file_path == Path::new("src/runner.ts"))
            .expect("expected runner finding");

        assert_eq!(finding.boundary_input_path.len(), 3);
        assert_eq!(
            finding.boundary_input_path[0].file_path,
            Path::new("src/main.ts")
        );
        assert_eq!(
            finding.boundary_input_path[0].relation_to_next,
            Some(RelationKind::Call)
        );
        assert_eq!(
            finding.boundary_input_path[0].target_symbol.as_deref(),
            Some("handle")
        );
        assert_eq!(finding.boundary_input_path[0].occurrence_index, 1);
        assert_eq!(
            finding.boundary_input_path[1].file_path,
            Path::new("src/handler.ts")
        );
        assert_eq!(
            finding.boundary_input_path[1].relation_to_next,
            Some(RelationKind::Call)
        );
        assert_eq!(
            finding.boundary_input_path[1].source_symbol.as_deref(),
            Some("handle")
        );
        assert_eq!(
            finding.boundary_input_path[1].target_symbol.as_deref(),
            Some("run")
        );
        assert_eq!(finding.boundary_input_path[1].occurrence_index, 2);
    }
}
