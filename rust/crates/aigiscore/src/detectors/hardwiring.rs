use crate::contracts::ContractLookup;
use crate::identity::{normalized_path, stable_fingerprint};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwiringCategory {
    MagicString,
    RepeatedLiteral,
    HardcodedNetwork,
    EnvOutsideConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwiringFinding {
    pub category: HardwiringCategory,
    pub file_path: PathBuf,
    pub line: usize,
    pub value: String,
    pub context: String,
    #[serde(default)]
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HardwiringResult {
    pub findings: Vec<HardwiringFinding>,
}

pub fn analyze_hardwiring(files: &[(PathBuf, String)]) -> HardwiringResult {
    analyze_hardwiring_with_contracts(files, &ContractLookup::default())
}

pub fn analyze_hardwiring_with_contracts(
    files: &[(PathBuf, String)],
    contract_lookup: &ContractLookup,
) -> HardwiringResult {
    let magic_re = Regex::new(r#"(?:==|!=)\s*"([^"\n]{3,})""#).expect("magic regex");
    let url_re = Regex::new(r#""(https?://[^"\n]+)""#).expect("url regex");
    let env_re = Regex::new(
        r#"\b(?:(?:std::)?env::(?:var|var_os)\s*\(|(?:std::)?env!\s*\(|option_env!\s*\()"#,
    )
    .expect("env regex");
    let string_re = Regex::new(r#""([^"\n]{3,})""#).expect("string regex");

    let mut findings = Vec::new();
    let mut repeated: HashMap<String, Vec<(PathBuf, usize, String)>> = HashMap::new();

    for (path, content) in files {
        for (idx, line) in content.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();
            if is_comment_line(trimmed) {
                continue;
            }

            for caps in magic_re.captures_iter(line) {
                let value = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                findings.push(HardwiringFinding {
                    category: HardwiringCategory::MagicString,
                    file_path: path.clone(),
                    line: line_no,
                    value: value.to_owned(),
                    context: trimmed.to_owned(),
                    fingerprint: hardwiring_fingerprint(
                        HardwiringCategory::MagicString,
                        path,
                        value,
                    ),
                });
            }

            for caps in url_re.captures_iter(line) {
                let value = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                findings.push(HardwiringFinding {
                    category: HardwiringCategory::HardcodedNetwork,
                    file_path: path.clone(),
                    line: line_no,
                    value: value.to_owned(),
                    context: trimmed.to_owned(),
                    fingerprint: hardwiring_fingerprint(
                        HardwiringCategory::HardcodedNetwork,
                        path,
                        value,
                    ),
                });
            }

            if env_re.is_match(line) && !is_config_like_path(path) {
                findings.push(HardwiringFinding {
                    category: HardwiringCategory::EnvOutsideConfig,
                    file_path: path.clone(),
                    line: line_no,
                    value: String::from("env"),
                    context: trimmed.to_owned(),
                    fingerprint: hardwiring_fingerprint(
                        HardwiringCategory::EnvOutsideConfig,
                        path,
                        "env",
                    ),
                });
            }

            for caps in string_re.captures_iter(line) {
                let value = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                if value.starts_with("http://") || value.starts_with("https://") || value.len() < 4
                {
                    continue;
                }
                if !is_constant_shaped_literal(value) {
                    continue;
                }
                if should_ignore_repeated_literal(path, trimmed, value, contract_lookup) {
                    continue;
                }
                repeated.entry(value.to_owned()).or_default().push((
                    path.clone(),
                    line_no,
                    trimmed.to_owned(),
                ));
            }
        }
    }

    for (value, occurrences) in repeated {
        // Repetition inside one file is a local style choice; hardwiring drift
        // is the same literal re-entered across files.
        let distinct_files = occurrences
            .iter()
            .map(|(path, _, _)| path)
            .collect::<HashSet<_>>();
        if distinct_files.len() < 2 {
            continue;
        }
        for (file_path, line, context) in occurrences {
            let fingerprint =
                hardwiring_fingerprint(HardwiringCategory::RepeatedLiteral, &file_path, &value);
            findings.push(HardwiringFinding {
                category: HardwiringCategory::RepeatedLiteral,
                file_path,
                line,
                value: value.clone(),
                context,
                fingerprint,
            });
        }
    }

    findings.sort_by(|left, right| {
        left.file_path
            .cmp(&right.file_path)
            .then(left.line.cmp(&right.line))
            .then(left.value.cmp(&right.value))
    });

    HardwiringResult { findings }
}

pub fn analyze_rust_hardwiring(files: &[(PathBuf, String)]) -> HardwiringResult {
    analyze_hardwiring(files)
}

fn hardwiring_fingerprint(category: HardwiringCategory, file_path: &Path, value: &str) -> String {
    stable_fingerprint(&[
        "hardwiring",
        hardwiring_category_label(category),
        &normalized_path(file_path),
        value,
    ])
}

fn hardwiring_category_label(category: HardwiringCategory) -> &'static str {
    match category {
        HardwiringCategory::MagicString => "magic-string",
        HardwiringCategory::RepeatedLiteral => "repeated-literal",
        HardwiringCategory::HardcodedNetwork => "hardcoded-network",
        HardwiringCategory::EnvOutsideConfig => "env-outside-config",
    }
}

fn is_config_like_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().to_lowercase();
    normalized.contains("config") || normalized.ends_with("build.rs")
}

fn is_comment_line(trimmed: &str) -> bool {
    trimmed.starts_with("//")
        || trimmed.starts_with('*')
        || trimmed.starts_with("/*")
        || trimmed.starts_with('#')
}

// A repeated string only signals hardwired configuration when it has constant
// DNA: mixed case, digits, or namespace/path/host separators. Bare words and
// dash-joined tokens are dominated by array keys, metadata field names, and
// CSS classes, and quote/interpolation characters mean the regex captured a
// concatenation fragment rather than a literal.
fn is_constant_shaped_literal(value: &str) -> bool {
    if value.trim().len() != value.len() {
        return false;
    }
    if value
        .chars()
        .any(|c| matches!(c, '\'' | '"' | '$' | '(' | ')' | '%' | '<' | '>' | '=' | '{' | '}'))
    {
        return false;
    }
    value
        .chars()
        .any(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || matches!(c, '.' | '/' | ':' | '@'))
}

fn should_ignore_repeated_literal(
    path: &Path,
    context: &str,
    value: &str,
    contract_lookup: &ContractLookup,
) -> bool {
    if contract_lookup.contains_literal(value) {
        return true;
    }
    let normalized_path = path.to_string_lossy().replace('\\', "/");
    let is_console_command = normalized_path.contains("/Console/Commands/")
        || normalized_path.ends_with("/Console/Command.php")
        || normalized_path.ends_with("Command.php");
    if is_console_command
        && (context.contains("$this->info(")
            || context.contains("$this->error(")
            || context.contains("$this->warn(")
            || context.contains("$this->line(")
            || context.contains("$this->comment(")
            || context.starts_with("protected $signature =")
            || context.starts_with("protected $description =")
            || context.starts_with("{--")
            || context.starts_with("{action="))
    {
        return true;
    }

    if is_printf_placeholder_literal(value)
        || is_control_escape_literal(value)
        || is_markup_utility_literal(context, value)
    {
        return true;
    }

    value.contains("{$")
        || value.contains("{$")
        || value.contains("\\u{")
        || value.contains("\\x")
        || context.contains("e.g.")
}

fn is_printf_placeholder_literal(value: &str) -> bool {
    static PRINTF_PLACEHOLDER_RE: OnceLock<Regex> = OnceLock::new();
    PRINTF_PLACEHOLDER_RE
        .get_or_init(|| Regex::new(r"^(?:%\d+\$[bcdeEufFgGosxX]|%[bcdeEufFgGosxX])+$").unwrap())
        .is_match(value)
}

fn is_control_escape_literal(value: &str) -> bool {
    static CONTROL_ESCAPE_RE: OnceLock<Regex> = OnceLock::new();
    CONTROL_ESCAPE_RE
        .get_or_init(|| Regex::new(r"^(?:\\[rnt]|\\u\{[0-9A-Fa-f]+\}|&#1[03];)+$").unwrap())
        .is_match(value)
}

fn is_markup_utility_literal(context: &str, value: &str) -> bool {
    let lowered_context = context.to_ascii_lowercase();
    let lowered_value = value.to_ascii_lowercase();
    let is_markupish_context = contains_markup_tag(context)
        || lowered_context.contains("class=")
        || lowered_context.contains("classname=")
        || lowered_context.contains("type=")
        || lowered_context.contains("aria-")
        || lowered_context.contains("data-")
        || lowered_context.contains("\"text\":")
        || lowered_context.contains("'text':")
        || lowered_context.contains("\"type\":")
        || lowered_context.contains("'type':");
    if !is_markupish_context {
        return false;
    }

    matches!(
        lowered_value.as_str(),
        "button"
            | "text"
            | "submit"
            | "hidden"
            | "checkbox"
            | "radio"
            | "password"
            | "email"
            | "search"
            | "url"
            | "tel"
            | "number"
            | "screen-reader-text"
    )
}

fn contains_markup_tag(context: &str) -> bool {
    static MARKUP_TAG_RE: OnceLock<Regex> = OnceLock::new();
    MARKUP_TAG_RE
        .get_or_init(|| Regex::new(r"</?[A-Za-z][^>]*>").unwrap())
        .is_match(context)
}

#[cfg(test)]
mod tests {
    use super::{analyze_hardwiring_with_contracts, analyze_rust_hardwiring, HardwiringCategory};
    use crate::contracts::ContractLookup;
    use std::path::PathBuf;

    #[test]
    fn detects_rust_hardwiring_signals() {
        let result = analyze_rust_hardwiring(&[
            (
                PathBuf::from("src/main.rs"),
                String::from(
                    r#"
fn main() {
    if status == "draft" {}
    let url = "https://api.example.com/v1";
    let mode = std::env::var("APP_MODE").unwrap();
    let first = "shared.value";
    let _ = mode;
}
"#,
                ),
            ),
            (
                PathBuf::from("src/config.rs"),
                String::from(
                    r#"fn config() { let _ = std::env::var("APP_MODE"); let second = "shared.value"; }"#,
                ),
            ),
        ]);

        assert!(result.findings.iter().any(|finding| finding.category
            == HardwiringCategory::MagicString
            && finding.value == "draft"));
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.category == HardwiringCategory::HardcodedNetwork));
        assert_eq!(
            result
                .findings
                .iter()
                .filter(|finding| finding.category == HardwiringCategory::EnvOutsideConfig)
                .count(),
            1
        );
        assert_eq!(
            result
                .findings
                .iter()
                .filter(|finding| {
                    finding.category == HardwiringCategory::RepeatedLiteral
                        && finding.value == "shared.value"
                })
                .count(),
            2
        );
    }

    #[test]
    fn ignores_console_output_and_signature_literals_for_repeated_literal_noise() {
        let result = analyze_rust_hardwiring(&[(
            PathBuf::from("app/Console/Commands/DemoCommand.php"),
            String::from(
                r#"
protected $signature = 'demo:run
    {--filter= : Filters as JSON (e.g., {"status":["active"]})}';

$this->error("User {$userId} not found");
$this->warn("User {$userId} not found");
$this->info("Connected to tenant: {$tenant}");
"#,
            ),
        )]);

        assert!(!result.findings.iter().any(|finding| {
            finding.category == HardwiringCategory::RepeatedLiteral
                && (finding.value == "status"
                    || finding.value == "active"
                    || finding.value == "User {$userId} not found"
                    || finding.value == "Connected to tenant: {$tenant}")
        }));
    }

    #[test]
    fn ignores_declared_contract_literals_for_repeated_literal_noise() {
        let result = analyze_hardwiring_with_contracts(
            &[(
                PathBuf::from("app/runtime.ts"),
                String::from(
                    r#"
const first = "user.created";
const second = "user.created";
const route = "/users";
const route2 = "/users";
"#,
                ),
            )],
            &ContractLookup {
                hooks: vec![String::from("user.created")],
                routes: vec![String::from("/users")],
                ..ContractLookup::default()
            },
        );

        assert!(!result.findings.iter().any(|finding| {
            finding.category == HardwiringCategory::RepeatedLiteral
                && (finding.value == "user.created" || finding.value == "/users")
        }));
    }

    #[test]
    fn ignores_printf_placeholders_control_escapes_and_markup_tokens() {
        let result = analyze_rust_hardwiring(&[
            (
                PathBuf::from("wp/admin/a.php"),
                String::from(
                    r#"
$label = "%1$s";
$other = "%1$s";
$newline = "\r\n";
$other_newline = "\r\n";
$button = "<button type=\"button\" class=\"button\">";
$type = "<input type=\"text\" />";
$screen = "<span class=\"screen-reader-text\">";
"#,
                ),
            ),
            (
                PathBuf::from("wp/admin/b.php"),
                String::from(
                    r#"
$label2 = "%1$s";
$newline2 = "\r\n";
$button2 = "<div data-component=\"button\"></div>";
$type2 = "<div data-type=\"text\"></div>";
$screen2 = "<label class=\"screen-reader-text\"></label>";
"#,
                ),
            ),
        ]);

        assert!(!result.findings.iter().any(|finding| {
            finding.category == HardwiringCategory::RepeatedLiteral
                && matches!(
                    finding.value.as_str(),
                    "%1$s" | "\\r\\n" | "button" | "text" | "screen-reader-text"
                )
        }));
    }

    #[test]
    fn bare_word_literals_stay_magic_strings_but_not_repeated_literals() {
        let result = analyze_rust_hardwiring(&[
            (
                PathBuf::from("src/a.ts"),
                String::from(
                    r#"
if (kind === "text" && count < limit) {
    render("text");
}
"#,
                ),
            ),
            (
                PathBuf::from("src/b.ts"),
                String::from(
                    r#"
if (kind === "text" && count < limit) {
    render("text");
}
"#,
                ),
            ),
        ]);

        // The behavior-gating comparison stays visible as a magic string; the
        // bare word itself has no constant DNA, so it is not repeated-literal
        // hardwiring.
        assert!(result.findings.iter().any(|finding| {
            finding.category == HardwiringCategory::MagicString && finding.value == "text"
        }));
        assert!(!result.findings.iter().any(|finding| {
            finding.category == HardwiringCategory::RepeatedLiteral && finding.value == "text"
        }));
    }

    #[test]
    fn skips_comment_lines_and_same_file_repetition_for_repeated_literals() {
        let result = analyze_rust_hardwiring(&[
            (
                PathBuf::from("src/a.php"),
                String::from(
                    r#"
// The "UTF-8" default lives here and in b.php.
$encoding = "UTF-8";
$fallback = "UTF-8";
$local = "only.here";
$local2 = "only.here";
"#,
                ),
            ),
            (
                PathBuf::from("src/b.php"),
                String::from(r#"$encoding = "UTF-8";"#),
            ),
        ]);

        let utf8 = result
            .findings
            .iter()
            .filter(|finding| {
                finding.category == HardwiringCategory::RepeatedLiteral
                    && finding.value == "UTF-8"
            })
            .collect::<Vec<_>>();
        // Three code occurrences across two files; the comment mention in
        // a.php must not add a fourth.
        assert_eq!(utf8.len(), 3);
        // Repetition confined to one file is not cross-file drift.
        assert!(!result.findings.iter().any(|finding| {
            finding.category == HardwiringCategory::RepeatedLiteral
                && finding.value == "only.here"
        }));
    }
}
