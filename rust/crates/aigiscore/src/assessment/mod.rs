use crate::detectors::dead_code::DeadCodeResult;
use crate::detectors::hardwiring::{HardwiringCategory, HardwiringResult};
use crate::external::{ExternalAnalysisResult, ExternalSeverity};
use crate::graph::analysis::{BottleneckFile, GraphAnalysis};
use crate::identity::{normalized_path, stable_fingerprint};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
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
    #[serde(default)]
    pub fingerprint: String,
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
    );
    findings.extend(split_identity_findings);
    findings.extend(compatibility_scars);
    findings.extend(duplicate_mechanisms);
    findings.extend(sanctioned_path_bypasses);
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
    }
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
}

fn detect_split_identity_models(
    parsed_sources: &[(PathBuf, String)],
) -> Vec<ArchitecturalAssessmentFinding> {
    let mut groups = HashMap::<String, IdentityVariantAccumulator>::new();

    for (path, content) in parsed_sources {
        if is_low_signal_identity_path(path) {
            continue;
        }
        for identifier in identifier_pattern().find_iter(content).map(|m| m.as_str()) {
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
            let Some((primary_file, _)) = file_counts.first() else {
                return None;
            };
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

    let mut findings = groups
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

            let Some((primary_file, _, _, primary_centrality, _)) = ranked_files.first() else {
                return None;
            };

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

fn detect_sanctioned_path_bypasses(
    bottlenecks: &[BottleneckFile],
    hardwiring: &HardwiringResult,
    parsed_sources: &[(PathBuf, String)],
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

    let mut findings = parsed_sources
        .iter()
        .filter_map(|(path, content)| {
            if is_low_signal_identity_path(path) || is_configuration_boundary_path(path) {
                return None;
            }
            let env_findings = env_findings_by_path.get(path)?;
            let config_markers = sanctioned_config_markers(content);
            if config_markers.is_empty() {
                return None;
            }
            let centrality = bottleneck_by_path.get(path).copied().unwrap_or_default();
            let mut related_identifiers = config_markers;
            related_identifiers.push(String::from("raw_env"));
            related_identifiers.sort();
            related_identifiers.dedup();

            Some(ArchitecturalAssessmentFinding {
                kind: ArchitecturalAssessmentKind::SanctionedPathBypass,
                file_path: path.clone(),
                related_file_paths: Vec::new(),
                related_identifiers,
                warning_count: env_findings.len(),
                warning_weight: env_findings.len() + 1,
                bottleneck_centrality_millis: centrality,
                warning_families: vec![
                    String::from("concern:configuration"),
                    String::from("bypass:raw_env"),
                    String::from("sanctioned:config_access"),
                ],
                severity_millis: (520
                    + env_findings.len().min(4) as u16 * 70
                    + ((centrality / 200).min(180) as u16))
                    .min(1000),
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

fn detect_mechanism_families(path: &Path, content: &str) -> BTreeSet<String> {
    let mut families = BTreeSet::new();
    let normalized_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let normalized_content = content.to_ascii_lowercase();

    if normalized_path.contains("/hooks/")
        || normalized_path.ends_with(".hook.php")
        || normalized_path.ends_with(".hooks.php")
        || normalized_content.contains("add_action(")
        || normalized_content.contains("add_filter(")
        || normalized_content.contains("beforesave")
        || normalized_content.contains("aftersave")
    {
        families.insert(String::from("lifecycle_hooks"));
    }

    if normalized_path.contains("/listeners/")
        || normalized_content.contains("event")
            && (normalized_content.contains("listener")
                || normalized_content.contains("subscribe")
                || normalized_content.contains("dispatchesevents")
                || normalized_content.contains("signal")
                || normalized_content.contains("emit")
                || normalized_content.contains("publish"))
    {
        families.insert(String::from("event_bus"));
    }

    if normalized_path.contains("/jobs/")
        || normalized_content.contains("shouldqueue")
        || normalized_content.contains("->onqueue(")
        || normalized_content.contains("::dispatch(")
        || normalized_content.contains(" dispatch(")
        || normalized_content.contains(" queue")
        || normalized_content.contains(" job")
    {
        families.insert(String::from("queue_jobs"));
    }

    if direct_notification_pattern().is_match(&normalized_content) {
        families.insert(String::from("direct_notifications"));
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

fn direct_notification_pattern() -> &'static Regex {
    static DIRECT_NOTIFICATION_PATTERN: OnceLock<Regex> = OnceLock::new();
    DIRECT_NOTIFICATION_PATTERN.get_or_init(|| {
        Regex::new(
            r"(?i)(\bwp_mail\s*\(|\bmail\s*\(|\bmail::send\s*\(|\bsendmail\b|\bmailer\b|\bnotify\s*\(|->notify\s*\(|::notify\s*\(|\bphpmailer\b)",
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
    use super::{build_architectural_assessment, ArchitecturalAssessmentKind};
    use crate::detectors::dead_code::{DeadCodeCategory, DeadCodeFinding, DeadCodeResult};
    use crate::detectors::hardwiring::{HardwiringCategory, HardwiringFinding, HardwiringResult};
    use crate::external::{
        ExternalAnalysisResult, ExternalConfidence, ExternalFinding, ExternalSeverity,
    };
    use crate::graph::analysis::{BottleneckFile, GraphAnalysis};
    use serde_json::Map;
    use std::path::PathBuf;

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
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("b"),
                    file_path: PathBuf::from("src/noisy.ts"),
                    name: String::from("unused2"),
                    line: 11,
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("c"),
                    file_path: PathBuf::from("src/noisy.ts"),
                    name: String::from("unused3"),
                    line: 12,
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
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("b"),
                    file_path: PathBuf::from("src/busy.ts"),
                    name: String::from("unused2"),
                    line: 11,
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("c"),
                    file_path: PathBuf::from("src/busy.ts"),
                    name: String::from("unused3"),
                    line: 12,
                    fingerprint: String::new(),
                },
                DeadCodeFinding {
                    category: DeadCodeCategory::UnusedPrivateFunction,
                    symbol_id: String::from("d"),
                    file_path: PathBuf::from("src/busy.ts"),
                    name: String::from("unused4"),
                    line: 13,
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
                        $payload['assigned_user_id'] = $assignedUserId;
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
                && finding.file_path == PathBuf::from("app/Services/EntityNormalizer.php")
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
            finding.file_path
                == PathBuf::from("app/Notifications/AssignmentNotificationService.php")
                || finding.file_path
                    == PathBuf::from("app/Hooks/assignment_notifications.hook.php")
                || finding.file_path == PathBuf::from("app/Jobs/AssignmentNotificationJob.php")
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
}
