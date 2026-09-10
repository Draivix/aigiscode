//! Revalidate reviewed reasoning against its code neighborhood. Adoption of a
//! review changes policy metadata, not the code evidence being adopted.

use crate::agentic::AgenticStructuredReviewResponse;
use crate::graph::{ResolvedEdge, RuntimeRegistration};
use crate::ingestion::pipeline::ProjectAnalysis;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

pub(crate) struct ReviewScopeIndex<'a> {
    context: String,
    sources: HashMap<&'a Path, u64>,
    edges: HashMap<&'a Path, Vec<&'a ResolvedEdge>>,
    registrations: HashMap<&'a Path, Vec<&'a RuntimeRegistration>>,
    registration_neighbors: HashMap<&'a Path, BTreeSet<&'a Path>>,
}

impl<'a> ReviewScopeIndex<'a> {
    pub(crate) fn new(analysis: &'a ProjectAnalysis) -> Self {
        let mut context = std::collections::hash_map::DefaultHasher::new();
        env!("AIGISCODE_ENGINE_FINGERPRINT").hash(&mut context);
        analysis.root.hash(&mut context);
        analysis.scan.scope_fingerprint.hash(&mut context);
        analysis.scan.semantic_env.fingerprint.0.hash(&mut context);
        analysis.resolve_config_xxh3.hash(&mut context);
        analysis.policy_bundle().review_context_fingerprint().hash(&mut context);
        analysis.doctrine_registry().hash(&mut context);
        analysis.dead_code.backend_orphan_coverage.input_fingerprint.hash(&mut context);
        analysis.semantic_graph.input_coverage().is_complete().hash(&mut context);
        let sources = analysis.parsed_sources.iter().map(|(path, source)|
            (path.as_path(), xxhash_rust::xxh3::xxh3_64(source.as_bytes()))).collect();
        let mut edges = HashMap::<&Path, Vec<&ResolvedEdge>>::new();
        for edge in &analysis.semantic_graph.resolved_edges {
            edges.entry(&edge.source_file_path).or_default().push(edge);
            if edge.target_file_path != edge.source_file_path { edges.entry(&edge.target_file_path).or_default().push(edge); }
        }
        let mut type_files = HashMap::<String, BTreeSet<&Path>>::new();
        for symbol in &analysis.semantic_graph.symbols {
            type_files.entry(symbol.qualified_name.to_ascii_lowercase()).or_default().insert(&symbol.file_path);
        }
        let mut registrations = HashMap::<&Path, Vec<&RuntimeRegistration>>::new();
        let mut registration_neighbors = HashMap::<&Path, BTreeSet<&Path>>::new();
        for registration in &analysis.semantic_graph.runtime_registrations {
            let mut files = BTreeSet::from([registration.file_path.as_path()]);
            for name in std::iter::once(&registration.contract_type).chain(registration.implementation_type.as_ref()) {
                files.extend(type_files.get(&name.to_ascii_lowercase()).into_iter().flatten().copied());
            }
            for file in &files {
                registrations.entry(file).or_default().push(registration);
                registration_neighbors.entry(file).or_default().extend(files.iter().copied());
            }
        }
        Self { context: format!("{:016x}", context.finish()), sources, edges, registrations, registration_neighbors }
    }

    pub(crate) fn id_for_files(&self, anchors: &[PathBuf]) -> Option<String> {
        if anchors.is_empty() || anchors.iter().any(|path| !self.sources.contains_key(path.as_path())) { return None; }
        let mut files = anchors.iter().map(PathBuf::as_path).collect::<BTreeSet<_>>();
        let mut edges = BTreeSet::new();
        let mut registrations = BTreeSet::new();
        for anchor in anchors {
            for edge in self.edges.get(anchor.as_path()).into_iter().flatten() {
                files.insert(&edge.source_file_path);
                files.insert(&edge.target_file_path);
                edges.insert(serde_json::to_string(edge).expect("resolved edge serializes"));
            }
            files.extend(self.registration_neighbors.get(anchor.as_path()).into_iter().flatten().copied());
            for registration in self.registrations.get(anchor.as_path()).into_iter().flatten() {
                registrations.insert(serde_json::to_string(registration).expect("registration serializes"));
            }
        }
        let files = files.into_iter().map(|path| (path, self.sources.get(path).copied())).collect::<BTreeMap<_, _>>();
        let encoded = serde_json::to_vec(&("architectural-review-scope-v1", &self.context, files, edges, registrations))
            .expect("review scope serializes");
        Some(format!("{:032x}", xxhash_rust::xxh3::xxh3_128(&encoded)))
    }
}

pub(crate) fn claim_files(claim: &crate::agentic::AgenticStructuredClaim) -> Vec<PathBuf> {
    claim.decision.implementations.iter().map(|implementation| implementation.file_path.as_str())
        .chain(claim.decision.consumer_changes.iter().map(|consumer| consumer.file_path.as_str()))
        .chain(claim.evidence_locations.iter().map(|location| location.file_path.as_str()))
        .map(PathBuf::from).collect::<BTreeSet<_>>().into_iter().collect()
}

pub(crate) fn response_scope_id(response: &AgenticStructuredReviewResponse, analysis: &ProjectAnalysis) -> String {
    let files = response.claims.iter().flat_map(claim_files).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
    ReviewScopeIndex::new(analysis).id_for_files(&files).unwrap_or_default()
}

pub(crate) fn record_is_stale(record: &super::decision::ArchitecturalReviewRecord, analysis: &ProjectAnalysis) -> bool {
    if record.source_scope_id.is_empty() { record.proposal.source_snapshot_id != crate::agentic::review_snapshot_id(analysis) }
    else { record.source_scope_id != response_scope_id(&record.proposal, analysis) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingestion::{pipeline::analyze_project, scan::ScanConfig};
    use std::fs;

    #[test]
    fn reviewed_scope_tracks_sources_callers_and_configuration_without_adoption_self_invalidation() {
        let root = std::env::temp_dir().join(format!("aigiscode-review-scope-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "mod helper;\nfn main() { helper::normalize(); }\n").unwrap();
        fs::write(root.join("src/helper.rs"), "pub fn normalize() {}\n").unwrap();
        fs::write(root.join("src/unrelated.rs"), "pub fn unrelated() {}\n").unwrap();
        let analyze = || analyze_project(&root, &ScanConfig::default()).unwrap();
        let analysis = analyze();
        assert!(analysis.semantic_graph.resolved_edges.iter().any(|edge| edge.source_file_path == Path::new("src/main.rs") && edge.target_file_path == Path::new("src/helper.rs")));
        let anchors = vec![PathBuf::from("src/helper.rs")];
        let initial = ReviewScopeIndex::new(&analysis).id_for_files(&anchors).unwrap();
        let decision = crate::policy::reviewed::ReviewedArchitecturalDecision {
            id: "retained-helper".into(), review_id: "0".repeat(32), claim_index: 0,
            disposition: crate::policy::reviewed::ArchitecturalPolicyDisposition::AcceptedPattern,
            concern: super::super::decision::ArchitecturalConcern::BoundaryViolation,
            conclusion: super::super::decision::ArchitecturalConclusion::IntentionalVariation,
            action: super::super::decision::ArchitecturalAction::Keep, reason: "The public helper owns the normalization boundary".into(),
            finding_fingerprints: Vec::new(), anchor_files: anchors.clone(), source_scope_id: initial.clone(),
            observed_change: crate::artifacts::ConvergenceStatus::NotCompared, comparison_baseline_id: None,
        };
        crate::policy::reviewed::adopt(&analysis, &decision).unwrap();
        assert_eq!(ReviewScopeIndex::new(&analyze()).id_for_files(&anchors).as_deref(), Some(initial.as_str()));
        fs::write(root.join("src/unrelated.rs"), "pub fn unrelated() { let changed = 1; }\n").unwrap();
        assert_eq!(ReviewScopeIndex::new(&analyze()).id_for_files(&anchors).as_deref(), Some(initial.as_str()));
        fs::write(root.join("src/main.rs"), "mod helper;\nfn main() { helper::normalize(); helper::normalize(); }\n").unwrap();
        assert_ne!(ReviewScopeIndex::new(&analyze()).id_for_files(&anchors).as_deref(), Some(initial.as_str()));
        fs::write(root.join("src/main.rs"), "mod helper;\nfn main() { helper::normalize(); }\n").unwrap();
        fs::write(root.join("src/helper.rs"), "pub fn normalize() { panic!(\"changed contract\"); }\n").unwrap();
        assert_ne!(ReviewScopeIndex::new(&analyze()).id_for_files(&anchors).as_deref(), Some(initial.as_str()));
        fs::write(root.join("src/helper.rs"), "pub fn normalize() {}\n").unwrap();
        fs::write(root.join(".aigiscode/policy.json"), r#"{"graph":{"orphan_entry_patterns":["src/**"]}}"#).unwrap();
        assert_ne!(ReviewScopeIndex::new(&analyze()).id_for_files(&anchors).as_deref(), Some(initial.as_str()));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn provider_registration_changes_invalidate_the_reviewed_implementation() {
        let root = std::env::temp_dir().join(format!("aigiscode-review-registration-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(root.join("app")).unwrap();
        fs::write(root.join("app/Contract.php"), "<?php namespace App; interface Contract { public function send(): void; }\n").unwrap();
        fs::write(root.join("app/Delivery.php"), "<?php namespace App; final class Delivery implements Contract { public function send(): void {} }\n").unwrap();
        fs::write(root.join("app/Other.php"), "<?php namespace App; final class Other implements Contract { public function send(): void {} }\n").unwrap();
        let provider = "<?php namespace App; class Provider extends \\Illuminate\\Support\\ServiceProvider { public function register(): void { $this->app->bind(Contract::class, Delivery::class); } }\n";
        fs::write(root.join("app/Provider.php"), provider).unwrap();
        let analyze = || analyze_project(&root, &ScanConfig::default()).unwrap();
        let analysis = analyze();
        assert!(!analysis.semantic_graph.runtime_registrations.is_empty());
        let anchors = vec![PathBuf::from("app/Delivery.php")];
        let original = ReviewScopeIndex::new(&analysis).id_for_files(&anchors);
        fs::write(root.join("app/Provider.php"), provider.replace("Delivery::class", "Other::class")).unwrap();
        assert_ne!(ReviewScopeIndex::new(&analyze()).id_for_files(&anchors), original);
        fs::remove_dir_all(root).unwrap();
    }

}
