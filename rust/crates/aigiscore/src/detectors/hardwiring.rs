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
    // Only *runtime* env access is configuration that should be centralized.
    // Compile-time `env!`/`option_env!` macros embed build metadata
    // (`CARGO_PKG_VERSION`, `CARGO_MANIFEST_DIR`) into the binary and are not
    // runtime configuration, so they are deliberately excluded.
    let env_re =
        Regex::new(r#"\b(?:std::)?env::(?:var|var_os)\s*\("#).expect("env regex");
    let string_re = Regex::new(r#""([^"\n]{3,})""#).expect("string regex");

    let mut findings = Vec::new();
    let mut repeated: HashMap<String, Vec<(PathBuf, usize, String)>> = HashMap::new();

    for (path, content) in files {
        // Literals in test code are fixtures, not architectural hardwiring.
        // Directory-scoped scans exclude test folders, but inline Rust
        // `#[cfg(test)]` modules and test files that slip into the scan must be
        // skipped here so the detector does not report fixture strings.
        if is_test_source_path(path) {
            continue;
        }
        // A single-file component's `<template>` and `<style>` are markup and
        // CSS, not logic. Their quoted attribute values are overwhelmingly Vue
        // binding expressions (`@submit="handleSubmit"`, `:component="Document"`)
        // and utility classes, not hardwired configuration. Scan only the
        // `<script>` block; the mask preserves line numbers so findings still
        // point at real `.vue` lines.
        let masked_sfc;
        let content: &str = if is_vue_sfc(path) {
            masked_sfc = crate::parsing::vue::extract_script(content).masked_source;
            &masked_sfc
        } else {
            content
        };
        let test_line_ranges = rust_cfg_test_line_ranges(path, content);
        for (idx, line) in content.lines().enumerate() {
            let line_no = idx + 1;
            if line_in_ranges(line_no, &test_line_ranges) {
                continue;
            }
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
                let match_start = caps.get(0).map(|m| m.start()).unwrap_or(0);
                if is_non_endpoint_url(value, line, match_start) {
                    continue;
                }
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

            // A raw env-access token inside a string literal is data, not a live
            // call — e.g. this scanner's own ast-grep rule definitions
            // (`"std::env::var($$$ARGS)"`) must not flag themselves.
            let env_call = env_re
                .find(line)
                .filter(|m| !is_inside_string_literal(line, m.start()));
            if env_call.is_some() && !is_config_like_path(path) {
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

fn is_vue_sfc(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("vue")
}

// A file whose path marks it as test/spec code across the common ecosystems.
// Its string literals are fixtures, not architectural configuration.
fn is_test_source_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    if normalized
        .split('/')
        .any(|segment| matches!(segment, "tests" | "test" | "spec" | "__tests__"))
    {
        return true;
    }
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name.to_lowercase(),
        None => return false,
    };
    file_name.ends_with("_test.rs")
        || file_name.ends_with("_test.go")
        || file_name.ends_with("_test.py")
        || file_name.starts_with("test_")
        || file_name.ends_with("test.php")
        || file_name.ends_with("spec.php")
        || [".test.", ".spec."]
            .iter()
            .any(|marker| file_name.contains(marker))
}

// Line ranges (1-based, inclusive) covered by inline Rust `#[cfg(test)]`
// modules. Uses brace balancing over the raw text, which matches the detector's
// text-based nature and is sufficient for the canonical `#[cfg(test)] mod tests`
// shape. Non-Rust files return no ranges.
fn rust_cfg_test_line_ranges(path: &Path, content: &str) -> Vec<(usize, usize)> {
    if path.extension().and_then(|e| e.to_str()) != Some("rs") {
        return Vec::new();
    }
    let lines: Vec<&str> = content.lines().collect();
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim_start().starts_with("#[cfg(test)]") {
            let start = i + 1;
            let mut depth: i32 = 0;
            let mut opened = false;
            let mut j = i;
            while j < lines.len() {
                for ch in lines[j].chars() {
                    match ch {
                        '{' => {
                            depth += 1;
                            opened = true;
                        }
                        '}' => depth -= 1,
                        _ => {}
                    }
                }
                if opened && depth <= 0 {
                    break;
                }
                j += 1;
            }
            ranges.push((start, j.min(lines.len().saturating_sub(1)) + 1));
            i = j + 1;
        } else {
            i += 1;
        }
    }
    ranges
}

fn line_in_ranges(line_no: usize, ranges: &[(usize, usize)]) -> bool {
    ranges
        .iter()
        .any(|(start, end)| line_no >= *start && line_no <= *end)
}

fn is_comment_line(trimmed: &str) -> bool {
    trimmed.starts_with("//")
        || trimmed.starts_with('*')
        || trimmed.starts_with("/*")
        || trimmed.starts_with('#')
}

// Whether the byte offset `target` on `line` falls inside a single-line string
// literal. Used to keep code-shaped patterns (an `env::var(` call) from matching
// when they are merely the text of a string constant. Single-line only: quote
// state resets each line, which is correct for the code tokens this guards.
fn is_inside_string_literal(line: &str, target: usize) -> bool {
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (idx, c) in line.char_indices() {
        if idx >= target {
            return quote.is_some();
        }
        match quote {
            Some(q) => {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == q {
                    quote = None;
                }
            }
            None => {
                if matches!(c, '\'' | '"' | '`') {
                    quote = Some(c);
                }
            }
        }
    }
    quote.is_some()
}

// A URL literal is only a hardcoded *network endpoint* when the program actually
// connects to it. Two large classes of URL literals are constant identifiers or
// display targets, never externalizable configuration, so flagging them is noise:
//
//   * XML namespace URIs, schema (`.xsd`) and DTD (`.dtd`) locations, and W3C /
//     `schemas.*` standards URLs. `xmlns="http://www.w3.org/2000/svg"` is a
//     spec-mandated constant; you cannot move it to config.
//   * Hyperlink / attribution targets in markup — a URL that is the value of an
//     HTML `href`/`src` attribute is something a user clicks (a footer social
//     link, a map attribution), not an endpoint the code calls.
//
// `match_start` is the byte offset of the opening quote of the URL literal on
// `line`, so the immediately-preceding attribute name can be inspected without
// re-scanning the whole line.
fn is_non_endpoint_url(value: &str, line: &str, match_start: usize) -> bool {
    let value_l = value.to_ascii_lowercase();
    let host_and_path = value_l
        .trim_start_matches("http://")
        .trim_start_matches("https://");

    // XML namespace / schema / DTD identifiers.
    if value_l.ends_with(".xsd")
        || value_l.ends_with(".dtd")
        || value_l.contains("/xmlschema")
        || host_and_path.starts_with("www.w3.org/")
        || host_and_path.starts_with("w3.org/")
        || host_and_path.starts_with("schemas.")
    {
        return true;
    }
    let line_l = line.to_ascii_lowercase();
    if line_l.contains("xmlns") || line_l.contains("<!doctype") || line_l.contains("schemalocation")
    {
        return true;
    }

    // Markup URL-valued attribute: the attribute name directly preceding the
    // quoted URL references a resource for display or metadata (`href`/`src`
    // links, a `canonical` SEO URL), not an endpoint the code connects to.
    let prefix = line[..match_start.min(line.len())]
        .trim_end()
        .to_ascii_lowercase();
    let attribute = prefix.trim_end_matches(['"', '\'', '=']).trim_end();
    let last_token = attribute
        .rsplit(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .unwrap_or_default();
    matches!(last_token, "href" | "src" | "canonical")
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
    if value.chars().any(|c| {
        matches!(
            c,
            '\'' | '"' | '$' | '(' | ')' | '%' | '<' | '>' | '=' | '{' | '}'
        )
    }) {
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
        || is_css_class_literal(context, value)
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

// A repeated literal that is the value of a `class`/`className` attribute and is
// shaped like a CSS/utility-class list (Tailwind and friends) is presentation,
// not hardwired configuration. Requires the class-attribute context so a plain
// word appearing elsewhere is never suppressed on class-shape alone.
fn is_css_class_literal(context: &str, value: &str) -> bool {
    let lowered_context = context.to_ascii_lowercase();
    let in_class_attribute = lowered_context.contains("class=")
        || lowered_context.contains("classname=")
        || lowered_context.contains(":class=")
        || lowered_context.contains("class:")
        || lowered_context.contains("classname:");
    if !in_class_attribute {
        return false;
    }
    is_css_class_value(value)
}

fn is_css_class_value(value: &str) -> bool {
    let tokens: Vec<&str> = value.split_whitespace().collect();
    if tokens.is_empty() {
        return false;
    }
    // Every token must be lowercase and use only CSS-class characters. Tailwind
    // uses `-`, `:`, `/`, `.`, `!`, and arbitrary-value brackets `[...]` whose
    // contents may include `#` (hex colors, `text-[#94a3b8]`), `,` (rgb/hsl),
    // `%`, and `@` (container-query variants). Uppercase letters mean it is not
    // a utility-class string. At least one token must carry a `-` or `:` so a
    // bare word is not misread as a class list.
    let mut has_utility_marker = false;
    for token in &tokens {
        if token.contains('-') || token.contains(':') {
            has_utility_marker = true;
        }
        if !token.chars().all(|c| {
            c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || matches!(c, '-' | ':' | '/' | '.' | '[' | ']' | '%' | '!' | '_' | '#' | ',' | '@')
        }) {
            return false;
        }
    }
    has_utility_marker || tokens.len() >= 2
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
    fn tailwind_arbitrary_value_classname_is_not_a_repeated_literal() {
        let contracts = ContractLookup::default();
        let sources: Vec<(PathBuf, String)> = (0..2)
            .map(|i| {
                (
                    PathBuf::from(format!("website/src/pages/Panel{i}.tsx")),
                    String::from(
                        "export const P = () => (\n  <p className=\"text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]\">x</p>\n);\n",
                    ),
                )
            })
            .collect();

        let result = analyze_hardwiring_with_contracts(&sources, &contracts);

        assert!(
            !result.findings.iter().any(|f| {
                f.category == HardwiringCategory::RepeatedLiteral
                    && f.value.contains("text-[")
            }),
            "a Tailwind className with an arbitrary hex value must not be a repeated literal: {:?}",
            result
                .findings
                .iter()
                .filter(|f| f.category == HardwiringCategory::RepeatedLiteral)
                .map(|f| f.value.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn env_outside_config_ignores_compile_time_macros_and_rule_strings() {
        let result = analyze_rust_hardwiring(&[(
            PathBuf::from("src/scanners/rules.rs").to_path_buf(),
            String::from(
                r#"
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
fn patterns() -> Vec<&'static str> {
    vec!["std::env::var($$$ARGS)", "env::var_os($$$ARGS)"]
}
fn read() -> Option<String> {
    std::env::var("OPENAI_API_KEY").ok()
}
"#,
            ),
        )]);

        let env_findings: Vec<usize> = result
            .findings
            .iter()
            .filter(|f| f.category == HardwiringCategory::EnvOutsideConfig)
            .map(|f| f.line)
            .collect();
        // Only the real runtime `std::env::var(...)` call on line 9 is flagged;
        // the compile-time `env!` macro and the two rule-definition strings are
        // not runtime env access.
        assert_eq!(
            env_findings, vec![9],
            "expected only the runtime env read, got {env_findings:?}"
        );
    }

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
    fn xml_namespace_and_markup_link_urls_are_not_hardcoded_network() {
        let contracts = ContractLookup::default();
        let result = analyze_hardwiring_with_contracts(
            &[(
                PathBuf::from("app/Export/DocxWriter.php"),
                String::from(
                    r#"<?php
class DocxWriter {
    public function body(): string {
        $svg = '<svg xmlns="http://www.w3.org/2000/svg" width="10"></svg>';
        $doc = '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">';
        $schema = "http://www.fio.cz/schema/importIB.xsd";
        $footer = '<a href="https://github.com/Draivix/aigiscode">GitHub</a>';
        $endpoint = "https://api.telegram.org/bot123/getUpdates";
        return $svg . $doc . $schema . $footer . $endpoint;
    }
}
"#,
                ),
            )],
            &contracts,
        );

        let network = result
            .findings
            .iter()
            .filter(|f| f.category == HardwiringCategory::HardcodedNetwork)
            .map(|f| f.value.as_str())
            .collect::<Vec<_>>();
        // Only the real callable endpoint survives; namespace/schema/hyperlink
        // URLs are suppressed as non-endpoints.
        assert_eq!(
            network,
            vec!["https://api.telegram.org/bot123/getUpdates"],
            "expected only the real endpoint, got {network:?}"
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
                finding.category == HardwiringCategory::RepeatedLiteral && finding.value == "UTF-8"
            })
            .collect::<Vec<_>>();
        // Three code occurrences across two files; the comment mention in
        // a.php must not add a fourth.
        assert_eq!(utf8.len(), 3);
        // Repetition confined to one file is not cross-file drift.
        assert!(!result.findings.iter().any(|finding| {
            finding.category == HardwiringCategory::RepeatedLiteral && finding.value == "only.here"
        }));
    }

    #[test]
    fn skips_test_regions_and_test_files_for_hardwiring() {
        let result = analyze_rust_hardwiring(&[
            (
                PathBuf::from("src/config.rs"),
                String::from(
                    r#"
fn boot() {
    connect("db.hostname.internal");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_connects() {
        let fixture = build("db.hostname.internal");
        assert_eq!(fixture.host, "db.hostname.internal");
    }
}
"#,
                ),
            ),
            (
                PathBuf::from("src/runtime.rs"),
                String::from(
                    r#"
fn start() {
    connect("db.hostname.internal");
}
"#,
                ),
            ),
            (
                PathBuf::from("tests/integration.rs"),
                String::from(r#"fn setup() { connect("db.hostname.internal"); }"#),
            ),
        ]);

        let hits = result
            .findings
            .iter()
            .filter(|finding| {
                finding.category == HardwiringCategory::RepeatedLiteral
                    && finding.value == "db.hostname.internal"
            })
            .collect::<Vec<_>>();
        // Only the two production occurrences count; the `#[cfg(test)]` module
        // and the `tests/` file must not contribute.
        assert_eq!(hits.len(), 2, "got: {hits:?}");
        assert!(hits.iter().all(|finding| {
            let p = finding.file_path.to_string_lossy();
            !p.contains("tests/") && finding.line < 6
        }));
    }

    #[test]
    fn skips_css_class_attribute_values_for_repeated_literals() {
        let result = analyze_rust_hardwiring(&[
            (
                PathBuf::from("website/src/pages/A.tsx"),
                String::from(
                    r#"
export function A() {
    return <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">{key}</div>;
}
const region = "eu-central-1";
"#,
                ),
            ),
            (
                PathBuf::from("website/src/pages/B.tsx"),
                String::from(
                    r#"
export function B() {
    return <section className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8" />;
}
const region = "eu-central-1";
"#,
                ),
            ),
        ]);

        // The Tailwind class list is presentation, not hardwired config.
        assert!(
            !result.findings.iter().any(|finding| {
                finding.category == HardwiringCategory::RepeatedLiteral
                    && finding.value == "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8"
            }),
            "css class list must not be flagged, got: {:?}",
            result.findings
        );
        // A genuine cross-file config literal on a non-class line still flags.
        assert!(result.findings.iter().any(|finding| {
            finding.category == HardwiringCategory::RepeatedLiteral
                && finding.value == "eu-central-1"
        }));
    }

    #[test]
    fn scans_only_the_script_block_of_vue_single_file_components() {
        let vue_a = r#"<template>
  <NButton @click="handleSubmit" :component="OverflowMenuVertical">
    <span class="max-w-7xl mx-auto px-4">go</span>
  </NButton>
</template>
<script setup lang="ts">
const bucket = "acme-prod-bucket-01";
</script>
"#;
        let vue_b = r#"<template>
  <NButton @click="handleSubmit" :component="OverflowMenuVertical" />
</template>
<script setup lang="ts">
const bucket = "acme-prod-bucket-01";
</script>
"#;
        let result = analyze_rust_hardwiring(&[
            (PathBuf::from("resources/js/A.vue"), String::from(vue_a)),
            (PathBuf::from("resources/js/B.vue"), String::from(vue_b)),
        ]);

        // Template binding expressions and component references are code, not
        // hardwired literals.
        for phantom in ["handleSubmit", "OverflowMenuVertical"] {
            assert!(
                !result
                    .findings
                    .iter()
                    .any(|finding| finding.value == phantom),
                "template binding {phantom:?} must not be flagged, got: {:?}",
                result.findings
            );
        }
        // A real repeated literal inside the `<script>` block still flags.
        assert!(
            result.findings.iter().any(|finding| {
                finding.category == HardwiringCategory::RepeatedLiteral
                    && finding.value == "acme-prod-bucket-01"
            }),
            "script-block literal must still flag, got: {:?}",
            result.findings
        );
    }
}
