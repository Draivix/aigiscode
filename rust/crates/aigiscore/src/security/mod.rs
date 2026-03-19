use crate::contracts::ContractInventory;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    #[serde(default)]
    pub contexts: Vec<SecurityContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SecurityAnalysisResult {
    pub findings: Vec<SecurityFinding>,
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
    let externally_reachable_files =
        externally_reachable_files(contract_inventory, runtime_entry_candidates);
    let mut findings = Vec::new();

    for (path, content) in parsed_sources {
        if is_test_like_path(path) {
            continue;
        }
        findings.extend(find_dangerous_api_findings(
            path,
            content,
            externally_reachable_files.contains(path),
        ));
    }

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
            classify_line(path, trimmed, externally_reachable, line_no, language)
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
            if externally_reachable {
                SecuritySeverity::Medium
            } else {
                SecuritySeverity::Low
            }
        }
        SecurityCategory::CommandExecution
        | SecurityCategory::CodeInjection
        | SecurityCategory::UnsafeDeserialization => {
            if externally_reachable {
                SecuritySeverity::High
            } else {
                SecuritySeverity::Medium
            }
        }
    };
    let contexts = classify_security_contexts(path, category, externally_reachable);

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
        contexts,
    })
}

fn classify_security_contexts(
    path: &Path,
    category: SecurityCategory,
    externally_reachable: bool,
) -> Vec<SecurityContext> {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let mut contexts = Vec::new();

    if externally_reachable {
        contexts.push(SecurityContext::ExternallyReachable);
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
        analyze_security_findings, SecurityCategory, SecurityContext, SecurityFindingKind,
        SecuritySeverity,
    };
    use crate::contracts::build_contract_inventory;
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
}
