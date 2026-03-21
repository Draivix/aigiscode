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
    HandRolledParsing,
    AbstractionSprawl,
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
    let hand_rolled_parsing =
        detect_hand_rolled_parsing(&graph_analysis.bottleneck_files, parsed_sources);
    let abstraction_sprawl =
        detect_abstraction_sprawl(&graph_analysis.bottleneck_files, parsed_sources);
    findings.extend(split_identity_findings);
    findings.extend(compatibility_scars);
    findings.extend(duplicate_mechanisms);
    findings.extend(sanctioned_path_bypasses);
    findings.extend(hand_rolled_parsing);
    findings.extend(abstraction_sprawl);
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
        ArchitecturalAssessmentKind::HandRolledParsing => "hand-rolled-parsing",
        ArchitecturalAssessmentKind::AbstractionSprawl => "abstraction-sprawl",
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

            let Some((primary_file, primary_roles, _, primary_centrality)) = ranked_files.first()
            else {
                return None;
            };
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
            let minimum_total_score = if is_primary_parser_cluster {
                8
            } else if is_definition_engine_cluster {
                5
            } else if is_contract_stack_cluster {
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

            let Some((primary_file, _, primary_score, primary_centrality, _)) =
                ranked_files.first()
            else {
                return None;
            };

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
    word.len() >= 4
        && !matches!(
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
}
