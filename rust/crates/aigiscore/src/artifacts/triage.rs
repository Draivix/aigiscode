//! Evidence-based ordering for repository triage; artifact emission stays in the parent.

use super::{
    build_semantic_state_proof_summary, build_status_summary_from_finding_ids_or_file,
    build_topology_state_flow_refs, packet_priority_rank, preview_severity_rank,
    topology_freshness_label, topology_semantic_state_kind_label,
    topology_semantic_state_proof_label, topology_semantic_state_proof_summary_suffix,
    topology_zone_path, ConvergenceStatus, RepositoryTopologyFindingPreview,
    RepositoryTopologyFocusCluster, RepositoryTopologyPacketPreview,
    RepositoryTopologyRecommendedSlice, RepositoryTopologyStateFlowPreview,
    RepositoryTopologyTriageStep, RepositoryTopologyZone,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

/// The single recorded signal supporting a target's review priority. Zone totals
/// and unrelated findings cannot strengthen this signal's precision or urgency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryTopologyPriorityBasis {
    pub evidence_id: String,
    pub priority: String,
    pub precision: String,
}

impl RepositoryTopologyPriorityBasis {
    fn rank(&self) -> (u8, u8) {
        (
            packet_priority_rank(&self.priority),
            precision_rank(&self.precision),
        )
    }
}

pub(super) fn precision_rank(precision: &str) -> u8 {
    match precision {
        "certain" | "exact" => 3,
        "strong" | "modeled" => 2,
        "heuristic" => 1,
        _ => 0,
    }
}

pub(super) fn build_topology_recommended_start(
    zones: &[RepositoryTopologyZone],
) -> Option<RepositoryTopologyRecommendedSlice> {
    zones.iter().flat_map(|zone| {
        zone.focus_clusters.iter().filter_map(move |cluster| {
            let basis = cluster.priority_basis.as_ref()?;
            Some((basis.rank(), RepositoryTopologyRecommendedSlice {
                priority_basis: Some(basis.clone()),
                zone_path: zone.path.clone(),
                target_file: cluster.primary_target_file.clone(),
                label: cluster.label.clone(),
                priority: basis.priority.clone(),
                reason: format!(
                    "Review `{}`: signal `{}` carries {} review priority and {} detector precision. Validate the source and owning contract before changing code; zone-wide counts do not determine this priority.",
                    cluster.primary_target_file, basis.evidence_id, basis.priority, basis.precision,
                ),
                supporting_zone_count: zone.linked_zones.len(),
            }))
        })
    }).max_by(|left, right| {
        left.0.cmp(&right.0)
            .then_with(|| right.1.target_file.cmp(&left.1.target_file))
            .then_with(|| right.1.zone_path.cmp(&left.1.zone_path))
    }).map(|(_, slice)| slice)
}

pub(super) fn build_topology_focus_clusters(
    zone_path: &str,
    guardian_packet_previews: &[RepositoryTopologyPacketPreview],
    visible_finding_previews: &[RepositoryTopologyFindingPreview],
    finding_status_lookup: &HashMap<String, ConvergenceStatus>,
    file_status_lookup: &HashMap<String, Vec<ConvergenceStatus>>,
    semantic_state_flow_lookup: &HashMap<String, Vec<RepositoryTopologyStateFlowPreview>>,
) -> Vec<RepositoryTopologyFocusCluster> {
    #[derive(Default)]
    struct FocusAccumulator {
        priority_basis: Option<RepositoryTopologyPriorityBasis>,
        label: String,
        visible_finding_ids: BTreeSet<String>,
        guardian_packet_ids: BTreeSet<String>,
        example_files: BTreeSet<String>,
        highest_visible_finding_severity: Option<String>,
        highest_guardian_packet_priority: Option<String>,
        triage_summary: Option<String>,
    }

    impl FocusAccumulator {
        fn consider(
            &mut self,
            basis: RepositoryTopologyPriorityBasis,
            label: String,
            summary: String,
        ) {
            let replace = self.priority_basis.as_ref().is_none_or(|current| {
                basis.rank() > current.rank()
                    || (basis.rank() == current.rank() && basis.evidence_id < current.evidence_id)
            });
            if replace {
                self.priority_basis = Some(basis);
                self.label = label;
                self.triage_summary = Some(summary);
            }
        }
    }

    let mut clusters = BTreeMap::<String, FocusAccumulator>::new();
    for packet in guardian_packet_previews {
        let key = packet.primary_target_file.clone();
        if topology_zone_path(Path::new(&key)) != zone_path {
            continue;
        }
        let cluster = clusters.entry(key.clone()).or_default();
        cluster.consider(
            RepositoryTopologyPriorityBasis {
                evidence_id: packet.id.clone(),
                priority: packet.priority.clone(),
                precision: packet.precision.clone(),
            },
            packet.focus.replace('_', " "),
            packet.summary.clone(),
        );
        cluster.guardian_packet_ids.insert(packet.id.clone());
        cluster
            .example_files
            .insert(packet.primary_target_file.clone());
        cluster.highest_guardian_packet_priority = Some(
            cluster
                .highest_guardian_packet_priority
                .as_deref()
                .map(|existing| {
                    if packet_priority_rank(&packet.priority) > packet_priority_rank(existing) {
                        packet.priority.as_str()
                    } else {
                        existing
                    }
                })
                .unwrap_or(packet.priority.as_str())
                .to_string(),
        );
        for finding_id in &packet.finding_ids {
            cluster.visible_finding_ids.insert(finding_id.clone());
        }
    }
    for finding in visible_finding_previews {
        let key = finding
            .file_paths
            .first()
            .cloned()
            .unwrap_or_else(|| String::from("unknown"));
        if topology_zone_path(Path::new(&key)) != zone_path {
            continue;
        }
        let cluster = clusters.entry(key.clone()).or_default();
        cluster.consider(
            RepositoryTopologyPriorityBasis {
                evidence_id: finding.id.clone(),
                priority: finding.severity.clone(),
                precision: finding.precision.clone(),
            },
            finding.title.clone(),
            finding.summary.clone(),
        );
        cluster.visible_finding_ids.insert(finding.id.clone());
        for file in &finding.file_paths {
            cluster.example_files.insert(file.clone());
        }
        cluster.highest_visible_finding_severity = Some(
            cluster
                .highest_visible_finding_severity
                .as_deref()
                .map(|existing| {
                    if preview_severity_rank(&finding.severity) > preview_severity_rank(existing) {
                        finding.severity.as_str()
                    } else {
                        existing
                    }
                })
                .unwrap_or(finding.severity.as_str())
                .to_string(),
        );
    }

    let mut result = clusters
        .into_iter()
        .map(|(primary_target_file, cluster)| {
            let semantic_state_flows_for_target = semantic_state_flow_lookup
                .get(&primary_target_file)
                .cloned()
                .unwrap_or_default();
            let semantic_state_flow_proof_summary =
                build_semantic_state_proof_summary(&semantic_state_flows_for_target);
            let visible_finding_ids = cluster
                .visible_finding_ids
                .into_iter()
                .take(8)
                .collect::<Vec<_>>();
            let finding_status_summary = build_status_summary_from_finding_ids_or_file(
                &visible_finding_ids,
                Some(primary_target_file.as_str()),
                finding_status_lookup,
                file_status_lookup,
            );
            RepositoryTopologyFocusCluster {
                priority_basis: cluster.priority_basis,
                id: format!("cluster:{primary_target_file}"),
                label: cluster.label,
                primary_target_file: primary_target_file.clone(),
                freshness: topology_freshness_label(&finding_status_summary),
                finding_status_summary,
                highest_visible_finding_severity: cluster.highest_visible_finding_severity,
                highest_guardian_packet_priority: cluster.highest_guardian_packet_priority,
                visible_finding_ids,
                guardian_packet_ids: cluster.guardian_packet_ids.into_iter().take(8).collect(),
                example_files: cluster.example_files.into_iter().take(5).collect(),
                triage_summary: format!(
                    "{}{}",
                    cluster
                        .triage_summary
                        .unwrap_or_else(|| String::from("No triage summary available.")),
                    topology_semantic_state_proof_summary_suffix(
                        &semantic_state_flow_proof_summary
                    )
                ),
                semantic_state_flow_labels: semantic_state_flows_for_target
                    .iter()
                    .take(3)
                    .map(|flow| flow.label.clone())
                    .collect(),
                semantic_state_flow_refs: build_topology_state_flow_refs(
                    &semantic_state_flows_for_target,
                    3,
                ),
                semantic_state_flow_proof_summary,
                causal_bridges: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| {
        right
            .priority_basis
            .as_ref()
            .map(RepositoryTopologyPriorityBasis::rank)
            .cmp(
                &left
                    .priority_basis
                    .as_ref()
                    .map(RepositoryTopologyPriorityBasis::rank),
            )
            .then(left.primary_target_file.cmp(&right.primary_target_file))
    });
    result.truncate(5);
    result
}

pub(super) fn build_topology_zone_triage_step_objects(
    clusters: &[RepositoryTopologyFocusCluster],
    packets: &[RepositoryTopologyPacketPreview],
    findings: &[RepositoryTopologyFindingPreview],
) -> Vec<RepositoryTopologyTriageStep> {
    clusters.iter().take(3).filter_map(|cluster| {
        let basis = cluster.priority_basis.as_ref()?;
        let packet = packets.iter().find(|packet| packet.id == basis.evidence_id);
        let finding = findings.iter().find(|finding| finding.id == basis.evidence_id);
        let artifact_refs = packet.map(|packet| &packet.artifact_refs)
            .or_else(|| finding.map(|finding| &finding.artifact_refs))?;
        Some(RepositoryTopologyTriageStep {
            priority_basis: Some(basis.clone()),
            action: format!(
                "Review `{}` in `{}` using `{}` ({} priority, {} detector precision). Validate the source and owning contract before choosing a change.{}",
                cluster.label, cluster.primary_target_file, basis.evidence_id, basis.priority, basis.precision,
                topology_semantic_state_proof_summary_suffix(&cluster.semantic_state_flow_proof_summary),
            ),
            priority: basis.priority.clone(),
            target_file: cluster.primary_target_file.clone(),
            step_kind: String::from(if packet.is_some() { "guardian_packet" } else { "visible_finding" }),
            freshness: cluster.freshness.clone(),
            finding_status_summary: cluster.finding_status_summary.clone(),
            packet_id: packet.map(|packet| packet.id.clone()),
            finding_id: finding.map(|finding| finding.id.clone()),
            artifact_refs: artifact_refs.clone(),
            doctrine_refs: packet.map(|packet| packet.doctrine_refs.clone()).unwrap_or_default(),
            semantic_state_flow_labels: cluster.semantic_state_flow_labels.clone(),
            semantic_state_flow_refs: cluster.semantic_state_flow_refs.clone(),
            semantic_state_flow_proof_summary: cluster.semantic_state_flow_proof_summary.clone(),
            causal_bridges: Vec::new(),
        })
    }).collect()
}

pub(super) fn build_topology_zone_triage_steps(
    structured: &[RepositoryTopologyTriageStep],
    flows: &[RepositoryTopologyStateFlowPreview],
) -> Vec<String> {
    let mut steps = structured
        .iter()
        .map(|step| step.action.clone())
        .collect::<Vec<_>>();
    for flow in flows.iter().take(3_usize.saturating_sub(steps.len())) {
        steps.push(format!(
            "Trace semantic state `{}` from `{}` to `{}` (`{}` / `{}` proof).",
            flow.label,
            flow.writer_file,
            flow.reader_file,
            topology_semantic_state_kind_label(flow.kind),
            topology_semantic_state_proof_label(flow.proof_tier),
        ));
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(
        id: &str,
        file: &str,
        severity: &str,
        precision: &str,
    ) -> RepositoryTopologyFindingPreview {
        RepositoryTopologyFindingPreview {
            id: id.to_owned(),
            severity: severity.to_owned(),
            precision: precision.to_owned(),
            family: String::from("graph"),
            title: id.to_owned(),
            summary: format!("Review signal {id}"),
            file_paths: vec![file.to_owned()],
            line: Some(1),
            artifact_refs: vec![String::from("review-surface.json")],
        }
    }

    #[test]
    fn priority_and_precision_must_come_from_the_same_signal() {
        let findings = vec![
            finding("candidate", "app/A.php", "high", "heuristic"),
            finding("minor-fact", "app/A.php", "low", "certain"),
            finding("modeled-risk", "app/B.php", "high", "modeled"),
        ];
        let clusters = build_topology_focus_clusters(
            "app",
            &[],
            &findings,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(clusters[0].primary_target_file, "app/B.php");
        let candidate = &clusters[1];
        let basis = candidate.priority_basis.as_ref().unwrap();
        assert_eq!(basis.evidence_id, "candidate");
        assert_eq!(basis.precision, "heuristic");
        assert_eq!(candidate.label, "candidate");
        let steps = build_topology_zone_triage_step_objects(&clusters, &[], &findings);
        assert_eq!(steps[0].finding_id.as_deref(), Some("modeled-risk"));
        assert_eq!(steps[0].priority_basis, clusters[0].priority_basis);
        assert_eq!(
            build_topology_zone_triage_steps(&steps, &[])[0],
            steps[0].action
        );
    }

    #[test]
    fn foreign_primary_targets_do_not_displace_local_clusters() {
        let mut foreign = finding("external", "clients/A.php", "high", "certain");
        foreign.file_paths.push(String::from("app/A.php"));
        let mut findings = vec![
            foreign,
            finding("second", "app/B.php", "high", "modeled"),
            finding("first", "app/A.php", "high", "modeled"),
        ];
        let forward = build_topology_focus_clusters(
            "app",
            &[],
            &findings,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        findings.reverse();
        let reversed = build_topology_focus_clusters(
            "app",
            &[],
            &findings,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(forward, reversed);
        assert_eq!(forward.len(), 2);
        assert_eq!(forward[0].primary_target_file, "app/A.php");
    }
}
