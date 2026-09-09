//! Complexity observations, actionability and operation-flow projections.

use super::{
    push_unique_string, ArchitecturalAssessmentFinding, ArchitecturalAssessmentKind,
    ArchitecturalComplexityFlowStep, ArchitecturalComplexityFlowStepKind,
    ArchitecturalComplexitySite, ArchitecturalPressureHop, ComplexityEvidenceSource,
};
use crate::scanners::ast_grep::{
    AstGrepComplexitySubtype, AstGrepFindingKind, AstGrepScanResult, BoundedMembership,
};
use crate::graph::analysis::GraphAnalysis;
use crate::graph::{RelationKind, SemanticGraph};
use regex::Regex;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// Small fixed membership tables are low-value complexity pressure. Larger
// literal tables remain reviewable even though their cardinality is bounded.
const MAX_SMALL_LITERAL_SET_ITEMS: usize = 16;

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

pub(super) fn detect_algorithmic_complexity_hotspots(
    parsed_sources: &[(PathBuf, String)],
    ast_grep_scan: &AstGrepScanResult,
) -> Vec<ArchitecturalAssessmentFinding> {
    let mut findings = Vec::new();
    let ast_grep_by_path = build_ast_grep_complexity_lookup(ast_grep_scan);
    let mut bounded_by_path = HashMap::<&Path, Vec<&BoundedMembership>>::new();
    for finding in &ast_grep_scan.findings {
        if let AstGrepFindingKind::AlgorithmicComplexity { bounded_membership: Some(bound), .. } = &finding.kind {
            if bound.max_items > MAX_SMALL_LITERAL_SET_ITEMS {
                continue;
            }
            bounded_by_path.entry(&finding.file_path).or_default().push(bound);
        }
    }
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
        let masked_lines = crate::lexmask::mask_file_non_code_spans(path, source)
            .expect("complexity language must have a lexical masker");
        let bounded = bounded_by_path.get(path.as_path()).map(Vec::as_slice).unwrap_or(&[]);
        let mut observations = match language {
            ComplexityLanguage::Brace => detect_brace_language_complexity(&masked_lines, bounded),
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
                evidence_anchors: Vec::new(),
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
    if path.extension().and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("vue"))
    {
        return Some(ComplexityLanguage::Brace);
    }
    match crate::lexmask::MaskLanguage::from_path(path) {
        Some(crate::lexmask::MaskLanguage::Rust | crate::lexmask::MaskLanguage::JavaScript
            | crate::lexmask::MaskLanguage::TypeScript | crate::lexmask::MaskLanguage::Php) => Some(ComplexityLanguage::Brace),
        Some(crate::lexmask::MaskLanguage::Python) => Some(ComplexityLanguage::Python),
        _ => None,
    }
}

fn detect_brace_language_complexity(
    masked_lines: &[String],
    bounded: &[&BoundedMembership],
) -> Vec<ComplexityObservation> {
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
                let character_column = raw_line[..raw_line.len() - raw_line.trim_start().len() + matched.start()]
                    .chars().count() + 2;
                let bounded_literal = token.starts_with(".includes") && bounded.iter().any(|bound| {
                    bound.method_line == line_number && bound.method_character_column == character_column
                });
                if !bounded_literal && !is_fixed_set_membership_scan(&token, line, matched.end()) {
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
        let AstGrepFindingKind::AlgorithmicComplexity { subtype, bounded_membership, .. } = &finding.kind else {
            continue;
        };
        if bounded_membership.as_ref().is_some_and(|bound| bound.max_items <= MAX_SMALL_LITERAL_SET_ITEMS) {
            continue;
        }
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
        ComplexitySubtype::HttpCallInLoop => 760u16,
        ComplexitySubtype::DatabaseQueryInLoop => 740u16,
        ComplexitySubtype::FilesystemReadInLoop => 660u16,
        ComplexitySubtype::CacheLookupInLoop => 620u16,
        ComplexitySubtype::JsonDecodeInLoop => 600u16,
        ComplexitySubtype::SortInLoop => 580u16,
        ComplexitySubtype::RegexCompileInLoop => 560u16,
        ComplexitySubtype::NestedIteration => 540u16,
        ComplexitySubtype::CollectionScanInLoop => 520u16,
    };
    let occurrence_boost = (occurrences.saturating_sub(1) as u16 * 30).min(150);
    base + occurrence_boost
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

pub(super) fn attach_complexity_graph_pressure(
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
        .filter(|path| !is_non_runtime_source_path(path))
        .cloned()
        .collect::<HashSet<_>>();
    let entry_paths = if entry_roots.is_empty() {
        HashMap::new()
    } else {
        graph_reachability_paths_for_hotspots(semantic_graph, &entry_roots)
    };
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
        if !supports_complexity_entry_pressure(edge.relation_kind)
            || edge.strength == crate::graph::EdgeStrength::Inferred
            || matches!(edge.kind, crate::graph::ReferenceKind::Type | crate::graph::ReferenceKind::TypeImport)
            || is_non_runtime_source_path(&edge.source_file_path)
            || is_non_runtime_source_path(&edge.target_file_path)
        {
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

    // BFS discovery order decides which entry path becomes the exemplar, so
    // roots must enter in sorted order — HashSet iteration is process-random
    // and would make the exemplar (and its fingerprint) run-dependent.
    let mut sorted_roots = roots.iter().cloned().collect::<Vec<_>>();
    sorted_roots.sort();
    let mut visited = sorted_roots.iter().cloned().collect::<HashSet<_>>();
    let mut queue = sorted_roots
        .into_iter()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanners::ast_grep::run_ast_grep_scan;

    #[test]
    fn reachability_exemplar_is_deterministic_regardless_of_root_hash_order() {
        use crate::graph::{
            EdgeOrigin, EdgeStrength, GraphLayer, ReferenceKind, ResolutionTier, ResolvedEdge,
        };
        let mut graph = SemanticGraph::default();
        let edge = |source: &str, target: &str, line: usize| ResolvedEdge {
            source_file_path: PathBuf::from(source),
            source_symbol_id: None,
            target_file_path: PathBuf::from(target),
            target_symbol_id: String::from("fn:target"),
            reference_target_name: None,
            kind: ReferenceKind::Call,
            relation_kind: RelationKind::Call,
            layer: GraphLayer::Structural,
            strength: EdgeStrength::Hard,
            origin: EdgeOrigin::Resolver,
            resolution_tier: ResolutionTier::ImportScoped,
            confidence_millis: 900,
            reason: String::from("test"),
            line,
            occurrence_index: 0,
        };
        // Two roots reach the same hotspot directly: the lexicographically
        // first root must win on every run, not whichever the HashSet
        // iteration happens to visit first.
        graph
            .resolved_edges
            .push(edge("src/zeta.ts", "src/hot.ts", 3));
        graph
            .resolved_edges
            .push(edge("src/alpha.ts", "src/hot.ts", 7));
        let roots = std::collections::HashSet::from([
            PathBuf::from("src/zeta.ts"),
            PathBuf::from("src/alpha.ts"),
        ]);

        let paths = super::graph_reachability_paths_for_hotspots(&graph, &roots);
        let chain = paths
            .get(&PathBuf::from("src/hot.ts"))
            .expect("hotspot must be reachable");
        assert_eq!(chain[0].file_path, PathBuf::from("src/alpha.ts"));
        assert_eq!(chain[0].line, Some(7));
    }



    fn assess(path: &str, source: &str) -> Vec<ArchitecturalAssessmentFinding> {
        let sources = [(PathBuf::from(path), source.to_owned())];
        let scan = run_ast_grep_scan(&sources);
        detect_algorithmic_complexity_hotspots(&sources, &scan)
    }

    #[test]
    fn vue_script_operations_reach_assessment_without_template_loops() {
        let source = concat!(
            "<template><div>for (x) { for (y) { JSON.parse(value) } }</div></template>\n",
            "<!-- <script>for (x) { for (y) {} }</script> -->\n",
            "<script setup lang='ts'>\n",
            "for (const word of words) {\n",
            "  const pattern = new RegExp(word);\n",
            "}\n</script>\n",
        );
        let findings = assess("src/Search.vue", source);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].warning_families.contains(&"complexity:regex_compile_in_loop".to_owned()));
        assert_eq!(findings[0].expensive_operation_sites[0].line, 5);
        assert_eq!(findings[0].expensive_operation_sites[0].source, ComplexityEvidenceSource::AstGrep);
    }

    #[test]
    fn module_extensions_share_native_complexity_dispatch() {
        for extension in ["mjs", "cjs", "mts", "cts", "JS", "TSX"] {
            let findings = assess(&format!("src/load.{extension}"),
                "for (const row of rows) {\n  JSON.parse(row);\n}\n");
            assert!(findings.iter().any(|finding| finding.warning_families
                .contains(&"complexity:json_decode_in_loop".to_owned())), "{extension}");
        }
    }

    #[test]
    fn small_literal_membership_is_retained_as_evidence_but_not_pressure() {
        let source = concat!(
            "<script setup lang='ts'>\nfor (const item of items) {\n",
            "  const label = '🙂'; ['a', 'b'].includes(item);\n",
            "  [\n    'c', 'd',\n  ].includes(item);\n}\n</script>\n",
        );
        let sources = [(PathBuf::from("src/Fields.VUE"), source.to_owned())];
        let scan = run_ast_grep_scan(&sources);
        assert_eq!(scan.findings.len(), 2);
        assert!(scan.findings.iter().all(|finding| matches!(&finding.kind,
            AstGrepFindingKind::AlgorithmicComplexity { bounded_membership: Some(_), .. })));
        assert!(detect_algorithmic_complexity_hotspots(&sources, &scan).is_empty());
    }

    #[test]
    fn literal_filter_does_not_hide_other_calls_on_the_same_line_or_in_arguments() {
        let findings = assess("src/Fields.ts", concat!(
            "for (const item of items) {\n",
            "  ['a'].includes(item); values.includes(item);\n",
            "  ['b'].includes(\n    nested.includes(item)\n  );\n}\n",
        ));
        let tokens = findings.iter().flat_map(|finding| &finding.expensive_operation_sites)
            .map(|site| site.token.as_str()).collect::<Vec<_>>();
        assert!(tokens.contains(&"values.includes(item)"));
        assert!(tokens.contains(&"nested.includes(item)"));
        assert!(!tokens.iter().any(|token| token.starts_with("['")));
    }

    #[test]
    fn spread_callbacks_and_large_literal_tables_remain_reviewable() {
        let values = (0..17).map(|value| value.to_string()).collect::<Vec<_>>().join(",");
        for expression in [
            "[...lookup].includes(item)".to_owned(),
            "[getValue()].includes(item)".to_owned(),
            "['a'].some(callback)".to_owned(),
            format!("[{values}].includes(item)"),
        ] {
            let source = format!("for (const item of items) {{\n  {expression};\n}}\n");
            assert!(assess("src/Fields.ts", &source).iter().any(|finding|
                finding.warning_families.contains(&"complexity:collection_scan_in_loop".to_owned())),
                "{expression}");
        }
    }

    #[test]
    fn runtime_pressure_does_not_use_guesses_type_imports_or_test_bridges() {
        use crate::graph::{EdgeOrigin, EdgeStrength, GraphLayer, ReferenceKind, ResolutionTier, ResolvedEdge};
        let edge = ResolvedEdge {
            source_file_path: PathBuf::from("src/entry.ts"),
            source_symbol_id: None,
            target_file_path: PathBuf::from("src/hot.ts"),
            target_symbol_id: "function:hot".into(),
            reference_target_name: Some("hot".into()),
            kind: ReferenceKind::Call,
            relation_kind: RelationKind::Call,
            layer: GraphLayer::Structural,
            strength: EdgeStrength::Hard,
            origin: EdgeOrigin::Resolver,
            resolution_tier: ResolutionTier::ImportScoped,
            confidence_millis: 900,
            reason: "call:import-scoped".into(),
            line: 2,
            occurrence_index: 0,
        };
        let roots = HashSet::from([PathBuf::from("src/entry.ts")]);
        let graph = |edges| SemanticGraph { resolved_edges: edges, ..Default::default() };
        assert!(graph_reachability_paths_for_hotspots(&graph(vec![edge.clone()]), &roots)
            .contains_key(Path::new("src/hot.ts")));

        let mut inferred = edge.clone();
        inferred.strength = EdgeStrength::Inferred;
        assert!(!graph_reachability_paths_for_hotspots(&graph(vec![inferred]), &roots)
            .contains_key(Path::new("src/hot.ts")));
        let mut type_import = edge.clone();
        type_import.kind = ReferenceKind::TypeImport;
        type_import.relation_kind = RelationKind::Import;
        assert!(!graph_reachability_paths_for_hotspots(&graph(vec![type_import]), &roots)
            .contains_key(Path::new("src/hot.ts")));

        let mut to_test = edge.clone();
        to_test.target_file_path = PathBuf::from("tests/helper.ts");
        let mut from_test = edge.clone();
        from_test.source_file_path = PathBuf::from("tests/helper.ts");
        assert!(!graph_reachability_paths_for_hotspots(&graph(vec![to_test, from_test]), &roots)
            .contains_key(Path::new("src/hot.ts")));

        let mut modeled = edge;
        modeled.strength = EdgeStrength::Dynamic;
        modeled.origin = EdgeOrigin::Plugin;
        modeled.relation_kind = RelationKind::Dispatch;
        assert!(graph_reachability_paths_for_hotspots(&graph(vec![modeled]), &roots)
            .contains_key(Path::new("src/hot.ts")));
    }

    #[test]
    fn missing_runtime_roots_preserve_local_operation_evidence() {
        let mut findings = assess("src/worker.ts",
            "for (const word of words) {\n  new RegExp(word);\n}\n");
        attach_complexity_graph_pressure(&mut findings, &GraphAnalysis::default(), Some(&SemanticGraph::default()));
        assert_eq!(findings.len(), 1);
        assert!(findings[0].pressure_path.is_empty());
        assert_eq!(findings[0].expensive_operation_flow.len(), 1);
        assert_eq!(findings[0].expensive_operation_flow[0].kind, ArchitecturalComplexityFlowStepKind::OperationSite);
    }

    #[test]
    fn io_in_loop_outranks_structural_nesting_and_severity_discriminates() {
        use super::{complexity_severity_millis, ComplexitySubtype};
        // IO repeated in a loop dominates purely-structural nesting.
        assert!(
            complexity_severity_millis(ComplexitySubtype::HttpCallInLoop, 1)
                > complexity_severity_millis(ComplexitySubtype::NestedIteration, 1)
        );
        assert!(
            complexity_severity_millis(ComplexitySubtype::DatabaseQueryInLoop, 1)
                > complexity_severity_millis(ComplexitySubtype::NestedIteration, 1)
        );
        // Occurrence count raises severity but is capped, and even the worst
        // subtype leaves headroom below the ceiling for reachability boosts.
        let one = complexity_severity_millis(ComplexitySubtype::NestedIteration, 1);
        let many = complexity_severity_millis(ComplexitySubtype::NestedIteration, 50);
        assert!(many > one);
        assert!(
            complexity_severity_millis(ComplexitySubtype::HttpCallInLoop, 50) < 1000,
            "base + occurrence boost must not saturate the ceiling on its own"
        );
    }

}
