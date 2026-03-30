pub mod tune;

use crate::detectors::dead_code::{DeadCodeCategory, DeadCodeFinding};
use crate::detectors::hardwiring::{HardwiringCategory, HardwiringFinding};
use crate::external::ExternalFinding;
use crate::security::{SecurityCategory, SecurityFinding};
use globset::{Glob, GlobMatcher};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const POLICY_FILE: &str = ".aigiscode/policy.json";
const RULES_FILE: &str = ".aigiscode/rules.json";

#[derive(Debug, Error)]
pub enum PolicyLoadError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid glob pattern `{pattern}` in {path}: {source}")]
    Glob {
        path: PathBuf,
        pattern: String,
        #[source]
        source: globset::Error,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressionReason {
    Policy,
    Rule,
}

#[derive(Debug, Clone, Default)]
pub struct PolicyBundle {
    graph_orphan_entry_patterns: Vec<GlobMatcher>,
    dead_code_abandoned_entry_patterns: Vec<GlobMatcher>,
    hardwiring_skip_path_patterns: Vec<GlobMatcher>,
    hardwiring_allowed_literals: Vec<String>,
    hardwiring_repeated_literal_min_occurrences: usize,
    security_skip_path_patterns: Vec<GlobMatcher>,
    security_allowed_categories: Vec<String>,
    external_skip_tools: Vec<String>,
    external_skip_categories: Vec<String>,
    external_allowed_rule_ids: Vec<String>,
    exclusion_rules: Vec<CompiledExclusionRule>,
}

impl PolicyBundle {
    pub fn load(root: &Path) -> Result<Self, PolicyLoadError> {
        let policy_path = root.join(POLICY_FILE);
        let rules_path = root.join(RULES_FILE);
        let policy = load_optional_json::<PolicyFile>(&policy_path)?;
        let rules = load_optional_rules(&rules_path)?;

        Ok(Self {
            graph_orphan_entry_patterns: compile_patterns(
                &policy_path,
                &policy.graph.orphan_entry_patterns,
            )?,
            dead_code_abandoned_entry_patterns: compile_patterns(
                &policy_path,
                &policy.dead_code.abandoned_entry_patterns,
            )?,
            hardwiring_skip_path_patterns: compile_patterns(
                &policy_path,
                &policy.hardwiring.skip_path_patterns,
            )?,
            hardwiring_allowed_literals: policy.hardwiring.allowed_literals,
            hardwiring_repeated_literal_min_occurrences: policy
                .hardwiring
                .repeated_literal_min_occurrences
                .unwrap_or(2)
                .max(2),
            security_skip_path_patterns: compile_patterns(
                &policy_path,
                &policy.security.skip_path_patterns,
            )?,
            security_allowed_categories: policy
                .security
                .allowed_categories
                .into_iter()
                .map(|value| normalize_token(&value))
                .collect(),
            external_skip_tools: policy
                .external
                .skip_tools
                .into_iter()
                .map(|value| normalize_token(&value))
                .collect(),
            external_skip_categories: policy
                .external
                .skip_categories
                .into_iter()
                .map(|value| normalize_token(&value))
                .collect(),
            external_allowed_rule_ids: policy.external.allowed_rule_ids,
            exclusion_rules: rules
                .into_iter()
                .map(|rule| CompiledExclusionRule::compile(&rules_path, rule))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub fn suppress_orphan_file(&self, file_path: &Path) -> Option<SuppressionReason> {
        let normalized = normalize_path(file_path);
        if matches_any(&self.graph_orphan_entry_patterns, &normalized) {
            return Some(SuppressionReason::Policy);
        }
        self.exclusion_rules.iter().find_map(|rule| {
            rule.matches_graph_finding(GraphFindingKind::OrphanFile, &normalized)
                .then_some(SuppressionReason::Rule)
        })
    }

    pub fn suppress_dead_code(&self, finding: &DeadCodeFinding) -> Option<SuppressionReason> {
        let normalized = normalize_path(&finding.file_path);
        if matches_any(&self.dead_code_abandoned_entry_patterns, &normalized) {
            return Some(SuppressionReason::Policy);
        }
        self.exclusion_rules.iter().find_map(|rule| {
            rule.matches_dead_code(finding, &normalized)
                .then_some(SuppressionReason::Rule)
        })
    }

    pub fn suppress_hardwiring(
        &self,
        finding: &HardwiringFinding,
        repeated_literal_occurrences: usize,
    ) -> Option<SuppressionReason> {
        let normalized = normalize_path(&finding.file_path);
        if matches_any(&self.hardwiring_skip_path_patterns, &normalized)
            || self
                .hardwiring_allowed_literals
                .iter()
                .any(|literal| literal == &finding.value)
            || (finding.category == HardwiringCategory::RepeatedLiteral
                && repeated_literal_occurrences < self.hardwiring_repeated_literal_min_occurrences)
        {
            return Some(SuppressionReason::Policy);
        }
        self.exclusion_rules.iter().find_map(|rule| {
            rule.matches_hardwiring(finding, &normalized)
                .then_some(SuppressionReason::Rule)
        })
    }

    pub fn suppress_security(&self, finding: &SecurityFinding) -> Option<SuppressionReason> {
        let normalized = normalize_path(&finding.file_path);
        let normalized_category = normalize_token(security_category_name(finding.category));
        if matches_any(&self.security_skip_path_patterns, &normalized)
            || self
                .security_allowed_categories
                .iter()
                .any(|category| category == &normalized_category)
        {
            return Some(SuppressionReason::Policy);
        }
        self.exclusion_rules.iter().find_map(|rule| {
            rule.matches_security(finding, &normalized)
                .then_some(SuppressionReason::Rule)
        })
    }

    pub fn suppress_external(&self, finding: &ExternalFinding) -> Option<SuppressionReason> {
        let normalized_tool = normalize_token(&finding.tool);
        let normalized_category = normalize_token(&finding.category);
        if self
            .external_skip_tools
            .iter()
            .any(|tool| tool == &normalized_tool)
            || self
                .external_skip_categories
                .iter()
                .any(|category| category == &normalized_category)
            || self
                .external_allowed_rule_ids
                .iter()
                .any(|rule_id| rule_id == &finding.rule_id)
        {
            return Some(SuppressionReason::Policy);
        }

        let match_path = external_match_path(finding);
        self.exclusion_rules.iter().find_map(|rule| {
            rule.matches_external(finding, &match_path)
                .then_some(SuppressionReason::Rule)
        })
    }
}

#[derive(Debug, Default, Deserialize)]
struct PolicyFile {
    #[serde(default)]
    graph: GraphPolicy,
    #[serde(default)]
    dead_code: DeadCodePolicy,
    #[serde(default)]
    hardwiring: HardwiringPolicy,
    #[serde(default)]
    security: SecurityPolicy,
    #[serde(default)]
    external: ExternalPolicy,
}

#[derive(Debug, Default, Deserialize)]
struct GraphPolicy {
    #[serde(default)]
    orphan_entry_patterns: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct DeadCodePolicy {
    #[serde(default)]
    abandoned_entry_patterns: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct HardwiringPolicy {
    #[serde(default)]
    repeated_literal_min_occurrences: Option<usize>,
    #[serde(default)]
    skip_path_patterns: Vec<String>,
    #[serde(default)]
    allowed_literals: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct SecurityPolicy {
    #[serde(default)]
    skip_path_patterns: Vec<String>,
    #[serde(default)]
    allowed_categories: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ExternalPolicy {
    #[serde(default)]
    skip_tools: Vec<String>,
    #[serde(default)]
    skip_categories: Vec<String>,
    #[serde(default)]
    allowed_rule_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RulesFile {
    Wrapped { rules: Vec<ExclusionRule> },
    Bare(Vec<ExclusionRule>),
    LegacyWrapped(LegacyRulesFile),
}

#[derive(Debug, Default, Deserialize)]
struct LegacyRulesFile {
    #[serde(default)]
    rules: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExclusionRule {
    #[serde(alias = "type", alias = "kind", alias = "findingType")]
    finding_type: String,
    #[serde(alias = "path", alias = "filePathPattern")]
    file_pattern: String,
    #[serde(default, alias = "name", alias = "value")]
    symbol_name: Option<String>,
    #[serde(default, alias = "source")]
    tool: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphFindingKind {
    OrphanFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuleFindingType {
    OrphanFile,
    UnusedPrivateFunction,
    UnusedImport,
    MagicString,
    RepeatedLiteral,
    HardcodedNetwork,
    EnvOutsideConfig,
    SecurityDangerousApi,
    SecurityCommandExecution,
    SecurityCodeInjection,
    SecurityUnsafeDeserialization,
    SecurityUnsafeHtmlOutput,
    ExternalAny,
    ExternalSast,
    ExternalSecrets,
    ExternalSca,
    ExternalLint,
    ExternalLicense,
    ExternalSourcePolicy,
    ExternalAbandonedDependency,
    ExternalSupplyChainPolicy,
}

impl RuleFindingType {
    fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "orphan" | "orphan_file" => Some(Self::OrphanFile),
            "unused_private_function" => Some(Self::UnusedPrivateFunction),
            "unused_import" => Some(Self::UnusedImport),
            "magic_string" => Some(Self::MagicString),
            "repeated_literal" => Some(Self::RepeatedLiteral),
            "hardcoded_network" | "hardcoded_ip_url" => Some(Self::HardcodedNetwork),
            "env_outside_config" => Some(Self::EnvOutsideConfig),
            "security" | "security_dangerous_api" | "dangerous_api" => {
                Some(Self::SecurityDangerousApi)
            }
            "security_command_execution" | "command_execution" => {
                Some(Self::SecurityCommandExecution)
            }
            "security_code_injection" | "code_injection" => Some(Self::SecurityCodeInjection),
            "security_unsafe_deserialization" | "unsafe_deserialization" => {
                Some(Self::SecurityUnsafeDeserialization)
            }
            "security_unsafe_html_output" | "unsafe_html_output" => {
                Some(Self::SecurityUnsafeHtmlOutput)
            }
            "external" => Some(Self::ExternalAny),
            "sast" | "external_sast" => Some(Self::ExternalSast),
            "secrets" | "external_secrets" => Some(Self::ExternalSecrets),
            "sca" | "external_sca" => Some(Self::ExternalSca),
            "lint" | "external_lint" => Some(Self::ExternalLint),
            "license" | "external_license" => Some(Self::ExternalLicense),
            "source_policy" | "external_source_policy" => Some(Self::ExternalSourcePolicy),
            "abandoned_dependency" | "external_abandoned_dependency" => {
                Some(Self::ExternalAbandonedDependency)
            }
            "supply_chain_policy" | "external_supply_chain_policy" => {
                Some(Self::ExternalSupplyChainPolicy)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct CompiledExclusionRule {
    finding_type: Option<RuleFindingType>,
    file_matcher: GlobMatcher,
    symbol_name: Option<String>,
    tool: Option<String>,
}

impl CompiledExclusionRule {
    fn compile(path: &Path, rule: ExclusionRule) -> Result<Self, PolicyLoadError> {
        Ok(Self {
            finding_type: RuleFindingType::parse(&rule.finding_type),
            file_matcher: compile_pattern(path, &rule.file_pattern)?,
            symbol_name: rule.symbol_name,
            tool: rule.tool.map(|tool| normalize_token(&tool)),
        })
    }

    fn matches_graph_finding(&self, kind: GraphFindingKind, path: &str) -> bool {
        let Some(finding_type) = self.finding_type else {
            return false;
        };
        let expected = match kind {
            GraphFindingKind::OrphanFile => RuleFindingType::OrphanFile,
        };
        finding_type == expected && self.file_matcher.is_match(path)
    }

    fn matches_dead_code(&self, finding: &DeadCodeFinding, path: &str) -> bool {
        let Some(finding_type) = self.finding_type else {
            return false;
        };
        let expected = match finding.category {
            DeadCodeCategory::UnusedPrivateFunction => RuleFindingType::UnusedPrivateFunction,
            DeadCodeCategory::UnusedImport => RuleFindingType::UnusedImport,
        };
        finding_type == expected
            && self.file_matcher.is_match(path)
            && self
                .symbol_name
                .as_ref()
                .is_none_or(|symbol_name| symbol_name == &finding.name)
    }

    fn matches_hardwiring(&self, finding: &HardwiringFinding, path: &str) -> bool {
        let Some(finding_type) = self.finding_type else {
            return false;
        };
        let expected = match finding.category {
            HardwiringCategory::MagicString => RuleFindingType::MagicString,
            HardwiringCategory::RepeatedLiteral => RuleFindingType::RepeatedLiteral,
            HardwiringCategory::HardcodedNetwork => RuleFindingType::HardcodedNetwork,
            HardwiringCategory::EnvOutsideConfig => RuleFindingType::EnvOutsideConfig,
        };
        finding_type == expected
            && self.file_matcher.is_match(path)
            && self
                .symbol_name
                .as_ref()
                .is_none_or(|symbol_name| symbol_name == &finding.value)
    }

    fn matches_security(&self, finding: &SecurityFinding, path: &str) -> bool {
        let Some(finding_type) = self.finding_type else {
            return false;
        };
        let expected = match finding.category {
            SecurityCategory::CommandExecution => RuleFindingType::SecurityCommandExecution,
            SecurityCategory::CodeInjection => RuleFindingType::SecurityCodeInjection,
            SecurityCategory::UnsafeDeserialization => {
                RuleFindingType::SecurityUnsafeDeserialization
            }
            SecurityCategory::UnsafeHtmlOutput => RuleFindingType::SecurityUnsafeHtmlOutput,
        };
        (finding_type == RuleFindingType::SecurityDangerousApi || finding_type == expected)
            && self.file_matcher.is_match(path)
            && self.symbol_name.as_ref().is_none_or(|symbol_name| {
                symbol_name == &finding.fingerprint
                    || symbol_name == security_category_name(finding.category)
            })
    }

    fn matches_external(&self, finding: &ExternalFinding, path: &str) -> bool {
        let Some(finding_type) = self.finding_type else {
            return false;
        };
        let expected = match normalize_token(&finding.category).as_str() {
            "sast" => RuleFindingType::ExternalSast,
            "secrets" => RuleFindingType::ExternalSecrets,
            "sca" => RuleFindingType::ExternalSca,
            "lint" => RuleFindingType::ExternalLint,
            "license" => RuleFindingType::ExternalLicense,
            "source_policy" => RuleFindingType::ExternalSourcePolicy,
            "abandoned_dependency" => RuleFindingType::ExternalAbandonedDependency,
            "supply_chain_policy" => RuleFindingType::ExternalSupplyChainPolicy,
            _ => RuleFindingType::ExternalAny,
        };
        (finding_type == RuleFindingType::ExternalAny || finding_type == expected)
            && self.file_matcher.is_match(path)
            && self
                .tool
                .as_ref()
                .is_none_or(|tool| tool == &normalize_token(&finding.tool))
            && self.symbol_name.as_ref().is_none_or(|symbol_name| {
                symbol_name == &finding.rule_id
                    || finding
                        .extras
                        .get("package_name")
                        .and_then(|value| value.as_str())
                        .is_some_and(|package_name| package_name == symbol_name)
            })
    }
}

fn load_optional_json<T>(path: &Path) -> Result<T, PolicyLoadError>
where
    T: Default + for<'de> Deserialize<'de>,
{
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).map_err(|source| PolicyLoadError::Parse {
            path: path.to_path_buf(),
            source,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(source) => Err(PolicyLoadError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn load_optional_rules(path: &Path) -> Result<Vec<ExclusionRule>, PolicyLoadError> {
    let parsed = load_optional_json::<Option<RulesFile>>(path)?;
    Ok(match parsed {
        None => Vec::new(),
        Some(RulesFile::Bare(rules)) => rules,
        Some(RulesFile::Wrapped { rules }) => rules,
        Some(RulesFile::LegacyWrapped(LegacyRulesFile { rules })) => {
            let _legacy_rule_count = rules.len();
            Vec::new()
        }
    })
}

fn compile_patterns(path: &Path, patterns: &[String]) -> Result<Vec<GlobMatcher>, PolicyLoadError> {
    patterns
        .iter()
        .map(|pattern| compile_pattern(path, pattern))
        .collect()
}

fn compile_pattern(path: &Path, pattern: &str) -> Result<GlobMatcher, PolicyLoadError> {
    Glob::new(pattern)
        .map_err(|source| PolicyLoadError::Glob {
            path: path.to_path_buf(),
            pattern: pattern.to_owned(),
            source,
        })
        .map(|glob| glob.compile_matcher())
}

fn matches_any(patterns: &[GlobMatcher], path: &str) -> bool {
    patterns.iter().any(|pattern| pattern.is_match(path))
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_token(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

fn security_category_name(category: SecurityCategory) -> &'static str {
    match category {
        SecurityCategory::CommandExecution => "command_execution",
        SecurityCategory::CodeInjection => "code_injection",
        SecurityCategory::UnsafeDeserialization => "unsafe_deserialization",
        SecurityCategory::UnsafeHtmlOutput => "unsafe_html_output",
    }
}

fn external_match_path(finding: &ExternalFinding) -> String {
    if let Some(path) = &finding.file_path {
        return normalize_path(path);
    }
    if let Some(package_name) = finding
        .extras
        .get("package_name")
        .and_then(|value| value.as_str())
    {
        return format!("dependency/{package_name}");
    }
    format!(
        "external/{}/{}",
        normalize_token(&finding.tool),
        normalize_token(&finding.category)
    )
}

#[cfg(test)]
mod tests {
    use super::{PolicyBundle, SuppressionReason};
    use crate::detectors::dead_code::{DeadCodeCategory, DeadCodeFinding, DeadCodeProofTier};
    use crate::detectors::hardwiring::{HardwiringCategory, HardwiringFinding};
    use crate::external::{ExternalConfidence, ExternalFinding, ExternalSeverity};
    use crate::security::{
        SecurityCategory, SecurityFinding, SecurityFindingKind, SecuritySeverity,
    };
    use serde_json::{Map, Value};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_policy_and_rules_and_classifies_findings() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::write(
            fixture.join(".aigiscode/policy.json"),
            br#"{
  "graph": { "orphan_entry_patterns": ["src/bootstrap/*.rs"] },
  "dead_code": { "abandoned_entry_patterns": ["src/contracts/**"] },
  "hardwiring": {
    "repeated_literal_min_occurrences": 4,
    "skip_path_patterns": ["src/console/**"],
    "allowed_literals": ["localhost"]
  },
  "security": {
    "skip_path_patterns": ["src/vendor/**"],
    "allowed_categories": ["unsafe_html_output"]
  },
  "external": {
    "skip_tools": ["osv-scanner"],
    "skip_categories": ["license"],
    "allowed_rule_ids": ["GHSA-policy-accepted"]
  }
}"#,
        )
        .unwrap();
        fs::write(
            fixture.join(".aigiscode/rules.json"),
            br#"[
  { "finding_type": "unused_import", "file_pattern": "src/lib.rs", "symbol_name": "RepoAlias" },
  { "finding_type": "magic_string", "file_pattern": "src/app.rs", "symbol_name": "draft" },
  { "finding_type": "sca", "file_pattern": "dependency/serde", "symbol_name": "GHSA-keep-out", "tool": "pip-audit" }
]"#,
        )
        .unwrap();

        let bundle = PolicyBundle::load(&fixture).unwrap();

        assert_eq!(
            bundle.suppress_orphan_file(PathBuf::from("src/bootstrap/main.rs").as_path()),
            Some(SuppressionReason::Policy)
        );
        assert_eq!(
            bundle.suppress_dead_code(&DeadCodeFinding {
                category: DeadCodeCategory::UnusedPrivateFunction,
                symbol_id: String::from("fn1"),
                file_path: PathBuf::from("src/contracts/user.rs"),
                name: String::from("load"),
                line: 4,
                proof_tier: DeadCodeProofTier::Strong,
                fingerprint: String::from("dead-1"),
            }),
            Some(SuppressionReason::Policy)
        );
        assert_eq!(
            bundle.suppress_dead_code(&DeadCodeFinding {
                category: DeadCodeCategory::UnusedImport,
                symbol_id: String::from("imp1"),
                file_path: PathBuf::from("src/lib.rs"),
                name: String::from("RepoAlias"),
                line: 2,
                proof_tier: DeadCodeProofTier::Certain,
                fingerprint: String::from("dead-2"),
            }),
            Some(SuppressionReason::Rule)
        );
        assert_eq!(
            bundle.suppress_hardwiring(
                &HardwiringFinding {
                    category: HardwiringCategory::MagicString,
                    file_path: PathBuf::from("src/app.rs"),
                    line: 8,
                    value: String::from("draft"),
                    context: String::from("if status == \"draft\""),
                    fingerprint: String::from("hard-1"),
                },
                1
            ),
            Some(SuppressionReason::Rule)
        );
        assert_eq!(
            bundle.suppress_hardwiring(
                &HardwiringFinding {
                    category: HardwiringCategory::RepeatedLiteral,
                    file_path: PathBuf::from("src/app.rs"),
                    line: 9,
                    value: String::from("shared-value"),
                    context: String::from("let _ = \"shared-value\""),
                    fingerprint: String::from("hard-2"),
                },
                3
            ),
            Some(SuppressionReason::Policy)
        );
        assert_eq!(
            bundle.suppress_hardwiring(
                &HardwiringFinding {
                    category: HardwiringCategory::MagicString,
                    file_path: PathBuf::from("src/console/cmd.rs"),
                    line: 3,
                    value: String::from("draft"),
                    context: String::from("if status == \"draft\""),
                    fingerprint: String::from("hard-3"),
                },
                1
            ),
            Some(SuppressionReason::Policy)
        );
        assert_eq!(
            bundle.suppress_external(&ExternalFinding {
                tool: String::from("osv-scanner"),
                domain: String::from("security"),
                category: String::from("sca"),
                rule_id: String::from("RUSTSEC-2026-0001"),
                severity: ExternalSeverity::High,
                confidence: ExternalConfidence::High,
                file_path: None,
                line: None,
                locations: Vec::new(),
                message: String::from("tool suppressed"),
                fingerprint: String::from("fp-1"),
                extras: external_extras("serde"),
            }),
            Some(SuppressionReason::Policy)
        );
        assert_eq!(
            bundle.suppress_external(&ExternalFinding {
                tool: String::from("pip-audit"),
                domain: String::from("security"),
                category: String::from("sca"),
                rule_id: String::from("GHSA-keep-out"),
                severity: ExternalSeverity::High,
                confidence: ExternalConfidence::High,
                file_path: None,
                line: None,
                locations: Vec::new(),
                message: String::from("rule suppressed"),
                fingerprint: String::from("fp-2"),
                extras: external_extras("serde"),
            }),
            Some(SuppressionReason::Rule)
        );
        assert_eq!(
            bundle.suppress_external(&ExternalFinding {
                tool: String::from("cargo-deny"),
                domain: String::from("security"),
                category: String::from("sca"),
                rule_id: String::from("GHSA-policy-accepted"),
                severity: ExternalSeverity::High,
                confidence: ExternalConfidence::High,
                file_path: None,
                line: None,
                locations: Vec::new(),
                message: String::from("rule id suppressed"),
                fingerprint: String::from("fp-3"),
                extras: Map::new(),
            }),
            Some(SuppressionReason::Policy)
        );
        assert_eq!(
            bundle.suppress_security(&SecurityFinding {
                kind: SecurityFindingKind::DangerousApi,
                category: SecurityCategory::UnsafeHtmlOutput,
                severity: SecuritySeverity::Low,
                file_path: PathBuf::from("src/ui/view.js"),
                line: 4,
                message: String::from("unsafe html"),
                evidence: String::from("target.innerHTML = html"),
                fingerprint: String::from("sec-1"),
                supporting_scanners: Vec::new(),
                contexts: Vec::new(),
                reachability_path: Vec::new(),
                boundary_input_sources: Vec::new(),
                boundary_input_path: Vec::new(),
                boundary_to_sink_flow: Vec::new(),
            }),
            Some(SuppressionReason::Policy)
        );
        assert_eq!(
            bundle.suppress_security(&SecurityFinding {
                kind: SecurityFindingKind::DangerousApi,
                category: SecurityCategory::CommandExecution,
                severity: SecuritySeverity::High,
                file_path: PathBuf::from("src/vendor/legacy.php"),
                line: 12,
                message: String::from("command exec"),
                evidence: String::from("system($cmd)"),
                fingerprint: String::from("sec-2"),
                supporting_scanners: Vec::new(),
                contexts: Vec::new(),
                reachability_path: Vec::new(),
                boundary_input_sources: Vec::new(),
                boundary_input_path: Vec::new(),
                boundary_to_sink_flow: Vec::new(),
            }),
            Some(SuppressionReason::Policy)
        );
    }

    #[test]
    fn rules_can_suppress_native_security_findings() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::write(
            fixture.join(".aigiscode/rules.json"),
            br#"[
  { "finding_type": "command_execution", "file_pattern": "src/ops/**", "symbol_name": "command_execution" },
  { "finding_type": "dangerous_api", "file_pattern": "src/admin.php", "symbol_name": "sec-fp-2" }
]"#,
        )
        .unwrap();

        let bundle = PolicyBundle::load(&fixture).unwrap();

        assert_eq!(
            bundle.suppress_security(&SecurityFinding {
                kind: SecurityFindingKind::DangerousApi,
                category: SecurityCategory::CommandExecution,
                severity: SecuritySeverity::High,
                file_path: PathBuf::from("src/ops/run.php"),
                line: 7,
                message: String::from("command exec"),
                evidence: String::from("system($cmd)"),
                fingerprint: String::from("sec-fp-1"),
                supporting_scanners: Vec::new(),
                contexts: Vec::new(),
                reachability_path: Vec::new(),
                boundary_input_sources: Vec::new(),
                boundary_input_path: Vec::new(),
                boundary_to_sink_flow: Vec::new(),
            }),
            Some(SuppressionReason::Rule)
        );
        assert_eq!(
            bundle.suppress_security(&SecurityFinding {
                kind: SecurityFindingKind::DangerousApi,
                category: SecurityCategory::CodeInjection,
                severity: SecuritySeverity::Medium,
                file_path: PathBuf::from("src/admin.php"),
                line: 5,
                message: String::from("eval"),
                evidence: String::from("eval($code)"),
                fingerprint: String::from("sec-fp-2"),
                supporting_scanners: Vec::new(),
                contexts: Vec::new(),
                reachability_path: Vec::new(),
                boundary_input_sources: Vec::new(),
                boundary_input_path: Vec::new(),
                boundary_to_sink_flow: Vec::new(),
            }),
            Some(SuppressionReason::Rule)
        );
    }

    #[test]
    fn tolerates_legacy_wrapped_rules_file_shape() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join(".aigiscode")).unwrap();
        fs::write(
            fixture.join(".aigiscode/rules.json"),
            br##"{
  "version": 2,
  "rules": [
    {
      "id": "seed-attr-import",
      "category": "unused_import",
      "checks": [
        {
          "type": "source_regex",
          "params": {
            "pattern": "#\\[{name}[\\s(]"
          }
        }
      ]
    }
  ]
}"##,
        )
        .unwrap();

        let bundle = PolicyBundle::load(&fixture).unwrap();

        assert_eq!(
            bundle.suppress_dead_code(&DeadCodeFinding {
                category: DeadCodeCategory::UnusedImport,
                symbol_id: String::from("imp1"),
                file_path: PathBuf::from("src/lib.rs"),
                name: String::from("RepoAlias"),
                line: 2,
                proof_tier: DeadCodeProofTier::Certain,
                fingerprint: String::from("dead-3"),
            }),
            None
        );
    }

    fn external_extras(package_name: &str) -> Map<String, Value> {
        let mut extras = Map::new();
        extras.insert(
            String::from("package_name"),
            Value::String(package_name.to_string()),
        );
        extras
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-policy-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
