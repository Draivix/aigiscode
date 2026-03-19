use crate::contracts::ContractLookup;
use crate::identity::{normalized_path, stable_fingerprint};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
        if occurrences.len() < 2 {
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

    value.contains("{$")
        || value.contains("{$")
        || value.contains("\\u{")
        || value.contains("\\x")
        || context.contains("e.g.")
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
    let first = "shared-value";
    let second = "shared-value";
    let _ = mode;
}
"#,
                ),
            ),
            (
                PathBuf::from("src/config.rs"),
                String::from(r#"fn config() { let _ = std::env::var("APP_MODE"); }"#),
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
                        && finding.value == "shared-value"
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
}
