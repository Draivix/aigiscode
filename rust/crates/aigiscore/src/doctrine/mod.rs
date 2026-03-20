use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::ops::Not;
use std::path::{Path, PathBuf};
use thiserror::Error;

const DOCTRINE_FILE: &str = ".aigiscode/doctrine.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DoctrineRegistry {
    pub version: String,
    pub clauses: Vec<DoctrineClause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DoctrineClause {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: DoctrineCategory,
    pub default_disposition: DoctrineDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_mechanism: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guidance: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DoctrineCategory {
    Architecture,
    ChangeGovernance,
    Configuration,
    Maintainability,
    MechanismChoice,
    Security,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DoctrineDisposition {
    Inform,
    Warn,
    Block,
}

#[derive(Debug, Error)]
pub enum DoctrineLoadError {
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
}

#[derive(Debug, Default, Deserialize)]
struct DoctrineOverrideFile {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    clauses: Vec<DoctrineClause>,
}

pub fn built_in_doctrine_registry() -> DoctrineRegistry {
    DoctrineRegistry {
        version: String::from("2026-03"),
        clauses: vec![
            clause(
                "configuration.coherence",
                "Configuration coherence",
                "Configuration should be centralized through sanctioned config boundaries instead of scattered literals and ambient access.",
                DoctrineCategory::Configuration,
                DoctrineDisposition::Warn,
                &[
                    "Prefer config-boundary access over direct env or ad hoc string constants.",
                ],
            ),
            clause(
                "guardian.architectonic-quality",
                "Architectonic quality",
                "Changes should strengthen global coherence and avoid introducing local fixes that degrade the system shape.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Judge changes by their effect on system shape, not only local correctness.",
                ],
            ),
            clause_with_preferred(
                "guardian.avoid-homegrown-parser",
                "Avoid homegrown parsers",
                "Do not build custom parser or mini-protocol stacks when a native contract or battle-tested parser is already available.",
                DoctrineCategory::MechanismChoice,
                DoctrineDisposition::Warn,
                "battle_tested_parser_or_native_contract",
                &[
                    "Prefer native framework contracts and established parser libraries over regex-heavy mini-parsers.",
                ],
            ),
            clause(
                "guardian.boundary-stability",
                "Boundary stability",
                "Boundary files should not become unstable dependency hubs or broad coordination chokepoints.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Keep transport, domain, persistence, and runtime boundaries explicit and narrow.",
                ],
            ),
            clause(
                "guardian.centralized-damage",
                "Centralized damage",
                "One file should not accumulate too many unrelated responsibilities or translation concerns.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Reduce god files, giant registries, and normalization chokepoints.",
                ],
            ),
            clause(
                "guardian.change-governance",
                "Change governance",
                "Guard decisions should judge new or worsened debt, not relitigate untouched legacy state.",
                DoctrineCategory::ChangeGovernance,
                DoctrineDisposition::Warn,
                &[
                    "Prefer diff-local enforcement with clear obligations and radius.",
                ],
            ),
            clause(
                "guardian.change-radius",
                "Change radius",
                "Changes that affect central files or contracts require reviewing neighboring files in the graph-connected radius.",
                DoctrineCategory::ChangeGovernance,
                DoctrineDisposition::Warn,
                &[
                    "Review one-hop dependent files when central contracts or hotspots change.",
                ],
            ),
            clause(
                "guardian.contract-coherence",
                "Contract coherence",
                "Externally visible routes, hooks, config keys, and symbolic runtime contracts should stay explicit and synchronized with implementation.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Public and runtime-facing surfaces should be declared and versionable.",
                ],
            ),
            clause(
                "guardian.diff-local-judgment",
                "Diff-local judgment",
                "Guard mode should reason about what changed and what got worse, not only static repo debt totals.",
                DoctrineCategory::ChangeGovernance,
                DoctrineDisposition::Warn,
                &[
                    "Favor new and worsened regressions over inherited baseline debt.",
                ],
            ),
            clause(
                "guardian.domain-coherence",
                "Domain coherence",
                "One domain concept should have one canonical representation and one obvious solution path.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Eliminate split identity models and concept forks.",
                ],
            ),
            clause(
                "guardian.mechanism-coherence",
                "Mechanism coherence",
                "A concern should not be implemented through multiple competing orchestration mechanisms without a clear sanctioned reason.",
                DoctrineCategory::MechanismChoice,
                DoctrineDisposition::Warn,
                &[
                    "Prefer one sanctioned solution path for each concern.",
                ],
            ),
            clause_with_preferred(
                "guardian.minimal-mechanism",
                "Minimal mechanism",
                "Prefer the smallest sanctioned mechanism that solves the problem without decorative abstraction or redundant indirection.",
                DoctrineCategory::MechanismChoice,
                DoctrineDisposition::Warn,
                "single_authoritative_domain_boundary",
                &[
                    "Avoid wrapper-only layers and extra coordinators that add no architectural clarity.",
                ],
            ),
            clause_with_preferred(
                "guardian.native-vs-library",
                "Native versus library",
                "Do not reinvent or bypass battle-tested native/framework/library capabilities without a strong reason.",
                DoctrineCategory::MechanismChoice,
                DoctrineDisposition::Warn,
                "battle_tested_parser_or_native_contract",
                &[
                    "Prefer battle-tested dependencies or sanctioned framework primitives over custom implementations.",
                ],
            ),
            clause(
                "guardian.overengineering",
                "Overengineering",
                "Avoid decorative abstraction sprawl, wrapper-only layers, and needless mechanism multiplication.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Keep abstractions justified by clear domain or lifecycle pressure.",
                ],
            ),
            clause_with_preferred(
                "guardian.sanctioned-paths",
                "Sanctioned paths",
                "Sensitive runtime concerns should flow through sanctioned config, auth, queue, notification, and integration boundaries.",
                DoctrineCategory::MechanismChoice,
                DoctrineDisposition::Warn,
                "single_sanctioned_configuration_path",
                &[
                    "Bypasses should be explicit exceptions, not ambient practice.",
                ],
            ),
            clause_with_preferred(
                "guardian.single-canonical-representation",
                "Single canonical representation",
                "A concept should not drift between object, scalar, snake_case, and camelCase identities without explicit migration boundaries.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                "single_canonical_domain_contract",
                &[
                    "Choose one canonical representation and retire aliases.",
                ],
            ),
            clause_with_preferred(
                "guardian.single-solution-path",
                "Single solution path",
                "Multiple implementation paths for the same concern should collapse onto one preferred mechanism.",
                DoctrineCategory::MechanismChoice,
                DoctrineDisposition::Warn,
                "single_sanctioned_orchestration_path",
                &[
                    "Parallel mechanisms should be temporary, owned, and time-bounded.",
                ],
            ),
            clause(
                "guardian.structural-coherence",
                "Structural coherence",
                "Structural dependency shape matters: avoid new cycles, unstable hubs, and uncontrolled fan-out.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Prefer clear layering and low graph pressure.",
                ],
            ),
            clause(
                "guardian.translation-hotspot",
                "Translation hotspot",
                "Files that repeatedly translate between competing representations are signs of unresolved architectural drift.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Eliminate repeated normalization and compatibility glue where possible.",
                ],
            ),
            clause_with_preferred(
                "guardian.trust-boundaries",
                "Trust boundaries",
                "Dangerous primitives on externally reachable or weakly bounded paths deserve stronger scrutiny and tighter guard behavior.",
                DoctrineCategory::Security,
                DoctrineDisposition::Block,
                "sanctioned_security_boundary",
                &[
                    "Escalate dangerous APIs when they touch routes, hooks, or interactive surfaces.",
                ],
            ),
            clause(
                "maintainability.remove-dead-code",
                "Remove dead code",
                "Unused code should be removed or clearly quarantined so it stops distorting the architecture surface.",
                DoctrineCategory::Maintainability,
                DoctrineDisposition::Warn,
                &[
                    "Delete dead code instead of letting it accumulate as ambient debt.",
                ],
            ),
            clause(
                "pattern.coherence",
                "Pattern coherence",
                "New code should preserve existing coherent patterns instead of inventing local variants.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Converge on the dominant pattern unless doctrine explicitly allows an exception.",
                ],
            ),
            clause(
                "security.coherence",
                "Security coherence",
                "Security-sensitive behavior should be explicit, layered, and consistent across the codebase.",
                DoctrineCategory::Security,
                DoctrineDisposition::Warn,
                &[
                    "Avoid ad hoc security controls and ambient dangerous primitives.",
                ],
            ),
            clause(
                "security.external-evidence",
                "External security evidence",
                "External scanner findings should feed the same typed review and guard loop as native findings.",
                DoctrineCategory::Security,
                DoctrineDisposition::Warn,
                &[
                    "Normalize external security signals instead of treating them as detached noise.",
                ],
            ),
            clause(
                "structural.coherence",
                "Structural coherence",
                "Dependency structure should remain legible, layered, and justifiable.",
                DoctrineCategory::Architecture,
                DoctrineDisposition::Warn,
                &[
                    "Avoid unexpected coupling, hidden runtime expansion, and unowned centralization.",
                ],
            ),
        ],
    }
}

impl DoctrineRegistry {
    pub fn clause(&self, id: &str) -> Option<&DoctrineClause> {
        self.clauses.iter().find(|clause| clause.id == id)
    }
}

pub fn doctrine_clause(id: &str) -> Option<DoctrineClause> {
    built_in_doctrine_registry()
        .clauses
        .into_iter()
        .find(|clause| clause.id == id)
}

pub fn load_doctrine_registry(root: &Path) -> Result<DoctrineRegistry, DoctrineLoadError> {
    let path = root.join(DOCTRINE_FILE);
    let mut registry = built_in_doctrine_registry();
    if path.exists().not() {
        return Ok(registry);
    }

    let payload = fs::read(&path).map_err(|source| DoctrineLoadError::Read {
        path: path.clone(),
        source,
    })?;
    let overrides: DoctrineOverrideFile =
        serde_json::from_slice(&payload).map_err(|source| DoctrineLoadError::Parse {
            path: path.clone(),
            source,
        })?;

    if let Some(version) = overrides.version {
        registry.version = version;
    }

    let mut clauses = registry
        .clauses
        .into_iter()
        .map(|clause| (clause.id.clone(), clause))
        .collect::<BTreeMap<_, _>>();
    for clause in overrides.clauses {
        clauses.insert(clause.id.clone(), clause);
    }
    registry.clauses = clauses.into_values().collect();
    Ok(registry)
}

fn clause(
    id: &str,
    title: &str,
    description: &str,
    category: DoctrineCategory,
    default_disposition: DoctrineDisposition,
    guidance: &[&str],
) -> DoctrineClause {
    clause_with_optional_preferred(
        id,
        title,
        description,
        category,
        default_disposition,
        None,
        guidance,
    )
}

fn clause_with_preferred(
    id: &str,
    title: &str,
    description: &str,
    category: DoctrineCategory,
    default_disposition: DoctrineDisposition,
    preferred_mechanism: &str,
    guidance: &[&str],
) -> DoctrineClause {
    clause_with_optional_preferred(
        id,
        title,
        description,
        category,
        default_disposition,
        Some(preferred_mechanism),
        guidance,
    )
}

fn clause_with_optional_preferred(
    id: &str,
    title: &str,
    description: &str,
    category: DoctrineCategory,
    default_disposition: DoctrineDisposition,
    preferred_mechanism: Option<&str>,
    guidance: &[&str],
) -> DoctrineClause {
    DoctrineClause {
        id: String::from(id),
        title: String::from(title),
        description: String::from(description),
        category,
        default_disposition,
        preferred_mechanism: preferred_mechanism.map(String::from),
        guidance: guidance.iter().map(|item| String::from(*item)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{built_in_doctrine_registry, doctrine_clause, load_doctrine_registry};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn built_in_registry_covers_current_doctrine_refs() {
        for id in [
            "configuration.coherence",
            "guardian.architectonic-quality",
            "guardian.avoid-homegrown-parser",
            "guardian.boundary-stability",
            "guardian.centralized-damage",
            "guardian.change-governance",
            "guardian.change-radius",
            "guardian.contract-coherence",
            "guardian.diff-local-judgment",
            "guardian.domain-coherence",
            "guardian.mechanism-coherence",
            "guardian.minimal-mechanism",
            "guardian.native-vs-library",
            "guardian.overengineering",
            "guardian.sanctioned-paths",
            "guardian.single-canonical-representation",
            "guardian.single-solution-path",
            "guardian.structural-coherence",
            "guardian.translation-hotspot",
            "guardian.trust-boundaries",
            "maintainability.remove-dead-code",
            "pattern.coherence",
            "security.coherence",
            "security.external-evidence",
            "structural.coherence",
        ] {
            assert!(
                doctrine_clause(id).is_some(),
                "missing doctrine clause: {id}"
            );
        }
        let registry = built_in_doctrine_registry();
        assert!(registry.clauses.len() >= 20);
    }

    #[test]
    fn load_doctrine_registry_merges_repo_overrides() {
        let root = unique_fixture_dir("doctrine-override");
        fs::create_dir_all(root.join(".aigiscode")).unwrap();
        fs::write(
            root.join(".aigiscode/doctrine.json"),
            r#"{
  "version": "repo-1",
  "clauses": [
    {
      "id": "guardian.native-vs-library",
      "title": "Repo-native vs library",
      "description": "Repo override",
      "category": "MechanismChoice",
      "default_disposition": "Block",
      "preferred_mechanism": "repo_sanctioned_parser",
      "guidance": ["Prefer the repo-sanctioned parser."]
    },
    {
      "id": "repo.notifications",
      "title": "Repo notifications",
      "description": "Use queue-based notifications only.",
      "category": "MechanismChoice",
      "default_disposition": "Warn",
      "preferred_mechanism": "queue_notification_boundary",
      "guidance": ["Route notifications through the sanctioned queue path."]
    }
  ]
}"#,
        )
        .unwrap();

        let registry = load_doctrine_registry(&root).unwrap();
        assert_eq!(registry.version, "repo-1");
        assert!(registry
            .clauses
            .iter()
            .any(|clause| clause.id == "repo.notifications"));
        let overridden = registry
            .clauses
            .iter()
            .find(|clause| clause.id == "guardian.native-vs-library")
            .unwrap();
        assert_eq!(overridden.title, "Repo-native vs library");
        assert_eq!(
            overridden.default_disposition,
            super::DoctrineDisposition::Block
        );
        assert_eq!(
            overridden.preferred_mechanism.as_deref(),
            Some("repo_sanctioned_parser")
        );
        assert_eq!(
            registry
                .clauses
                .iter()
                .find(|clause| clause.id == "repo.notifications")
                .and_then(|clause| clause.preferred_mechanism.as_deref()),
            Some("queue_notification_boundary")
        );
    }

    fn unique_fixture_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("aigiscode-{label}-{unique}"))
    }
}
