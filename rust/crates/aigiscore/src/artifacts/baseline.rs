//! A baseline is a sealed capture, never three unrelated JSON files or an assumed empty state.

use super::{
    read_json_artifact_if_exists, ScanManifest, ScanManifestEntry, ARCHITECTURE_SURFACE_FILE,
    CONTRACT_INVENTORY_FILE, REVIEW_SURFACE_FILE, SCAN_MANIFEST_FILE, SEMANTIC_REVISION,
};
use crate::contracts::ContractInventory;
use crate::ingestion::hash::HashingIo;
use crate::ingestion::pipeline::ProjectAnalysis;
use crate::review::ReviewSurface;
use crate::surface::ArchitectureSurface;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SnapshotIdentity {
    pub root: String,
    pub engine_version: String,
    #[serde(default)]
    pub engine_fingerprint: String,
    pub semantic_revision: u32,
    pub source_fingerprint: String,
    #[serde(default)]
    pub input_inventory_fingerprint: String,
    #[serde(default)]
    pub semantic_env_fingerprint: String,
    pub scope_fingerprint: String,
    pub resolve_config_fingerprint: String,
    #[serde(default)]
    pub assessment_config_fingerprint: String,
    pub external_tools: Vec<String>,
    pub external_checks_complete: bool,
}

impl SnapshotIdentity {
    pub fn capture(analysis: &ProjectAnalysis) -> Self {
        let mut assessment_config = std::collections::hash_map::DefaultHasher::new();
        analysis.policy_bundle().fingerprint().hash(&mut assessment_config);
        analysis.doctrine_registry().hash(&mut assessment_config);
        let mut external_tools = analysis
            .external_analysis
            .tool_runs
            .iter()
            .map(|run| run.tool.clone())
            .collect::<Vec<_>>();
        external_tools.sort();
        external_tools.dedup();
        Self {
            root: analysis.scan.root.display().to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
            engine_fingerprint: env!("AIGISCODE_ENGINE_FINGERPRINT").to_owned(),
            semantic_revision: SEMANTIC_REVISION,
            input_inventory_fingerprint: analysis.scan.input_fingerprint(),
            semantic_env_fingerprint: format!("{:032x}", analysis.scan.semantic_env.fingerprint.0),
            source_fingerprint: source_fingerprint(
                &analysis
                    .parsed_sources
                    .iter()
                    .map(|(path, source)| ScanManifestEntry {
                        path: path.display().to_string(),
                        xxh3: format!("{:016x}", xxhash_rust::xxh3::xxh3_64(source.as_bytes())),
                    })
                    .collect::<Vec<_>>(),
            ),
            scope_fingerprint: analysis.scan.scope_fingerprint.clone(),
            resolve_config_fingerprint: analysis.resolve_config_xxh3.clone(),
            assessment_config_fingerprint: format!("{:016x}", assessment_config.finish()),
            external_tools,
            external_checks_complete: analysis.external_analysis.is_complete(),
        }
    }
}

fn source_fingerprint(files: &[ScanManifestEntry]) -> String {
    let mut files = files.iter().collect::<Vec<_>>();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let mut hash = xxhash_rust::xxh3::Xxh3::new();
    for file in files {
        for value in [&file.path, &file.xxh3] {
            hash.update(&(value.len() as u64).to_le_bytes());
            hash.update(value.as_bytes());
        }
    }
    format!("{:032x}", hash.digest128())
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineHashes {
    pub architecture_surface: String,
    pub review_surface: String,
    pub contract_inventory: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaselineAvailability {
    Missing,
    Partial,
    Unverified,
    Inconsistent,
    Verified,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaselineComparison {
    InitialSnapshot,
    Comparable,
    #[default]
    NotCompared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaselineReason {
    NoBaseline,
    MissingFamilyMember,
    MissingSeal,
    HashMismatch,
    ManifestChanged,
    DifferentRoot,
    DifferentEngine,
    DifferentScope,
    DifferentExternalTools,
    CurrentInputsIncomplete,
    PreviousInputsIncomplete,
    CurrentSecondaryChecksIncomplete,
    PreviousSecondaryChecksIncomplete,
    ExternalChecksIncomplete,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BaselineAssessment {
    pub availability: BaselineAvailability,
    pub comparison: BaselineComparison,
    pub current: SnapshotIdentity,
    pub previous: Option<SnapshotIdentity>,
    pub reasons: Vec<BaselineReason>,
}

impl BaselineAssessment {
    pub fn is_comparable(&self) -> bool {
        self.comparison == BaselineComparison::Comparable
    }
}

#[derive(Debug)]
pub struct BaselineSnapshot {
    pub(super) architecture: Option<ArchitectureSurface>,
    pub(super) review: Option<ReviewSurface>,
    pub(super) contracts: Option<ContractInventory>,
    identity: Option<SnapshotIdentity>,
    availability: BaselineAvailability,
    reasons: Vec<BaselineReason>,
}

impl BaselineSnapshot {
    pub fn empty() -> Self {
        Self {
            architecture: None,
            review: None,
            contracts: None,
            identity: None,
            availability: BaselineAvailability::Missing,
            reasons: vec![BaselineReason::NoBaseline],
        }
    }

    pub fn load(output_dir: &Path) -> io::Result<Self> {
        let Some(snapshot) = super::ArtifactSnapshot::pin(output_dir)? else {
            return Ok(Self::empty());
        };
        Self::load_pinned(&snapshot.directory)
    }

    pub(crate) fn load_pinned(output_dir: &Path) -> io::Result<Self> {
        let manifest_path = output_dir.join(SCAN_MANIFEST_FILE);
        let manifest = read_json_artifact_if_exists::<ScanManifest>(&manifest_path)?;
        let architecture =
            read_hashed::<ArchitectureSurface>(&output_dir.join(ARCHITECTURE_SURFACE_FILE))?;
        let review = read_hashed::<ReviewSurface>(&output_dir.join(REVIEW_SURFACE_FILE))?;
        let contracts =
            read_hashed::<ContractInventory>(&output_dir.join(CONTRACT_INVENTORY_FILE))?;
        if manifest.is_none() && architecture.is_none() && review.is_none() && contracts.is_none() {
            return Ok(Self::empty());
        }
        let mut availability = BaselineAvailability::Verified;
        let mut reasons = Vec::new();
        if architecture.is_none() || review.is_none() || contracts.is_none() {
            availability = BaselineAvailability::Partial;
            reasons.push(BaselineReason::MissingFamilyMember);
        }
        let identity = manifest
            .as_ref()
            .and_then(|manifest| manifest.snapshot_identity.clone());
        match manifest
            .as_ref()
            .and_then(|manifest| manifest.baseline_hashes.as_ref())
        {
            Some(hashes) if identity.is_some() => {
                if architecture
                    .as_ref()
                    .is_some_and(|(_, hash)| hash != &hashes.architecture_surface)
                    || review
                        .as_ref()
                        .is_some_and(|(_, hash)| hash != &hashes.review_surface)
                    || contracts
                        .as_ref()
                        .is_some_and(|(_, hash)| hash != &hashes.contract_inventory)
                    || manifest.as_ref().zip(identity.as_ref()).is_some_and(
                        |(manifest, identity)| {
                            source_fingerprint(&manifest.files) != identity.source_fingerprint
                                || manifest.semantic_revision != identity.semantic_revision
                                || manifest.aigiscode_version != identity.engine_version
                                || manifest.resolve_config_xxh3
                                    != identity.resolve_config_fingerprint
                        },
                    )
                {
                    availability = BaselineAvailability::Inconsistent;
                    reasons.push(BaselineReason::HashMismatch);
                }
            }
            _ => {
                if availability == BaselineAvailability::Verified {
                    availability = BaselineAvailability::Unverified;
                }
                reasons.push(BaselineReason::MissingSeal);
            }
        }
        // Publication during these reads cannot silently turn the three inputs into
        // a baseline. The loaded objects remain an immutable observation once verified.
        if read_json_artifact_if_exists::<ScanManifest>(&manifest_path)? != manifest {
            availability = BaselineAvailability::Inconsistent;
            reasons.push(BaselineReason::ManifestChanged);
        }
        Ok(Self {
            architecture: architecture.map(|(value, _)| value),
            review: review.map(|(value, _)| value),
            contracts: contracts.map(|(value, _)| value),
            identity,
            availability,
            reasons,
        })
    }

    pub fn assess(&self, analysis: &ProjectAnalysis) -> BaselineAssessment {
        let current = SnapshotIdentity::capture(analysis);
        let mut reasons = self.reasons.clone();
        if !analysis.semantic_graph.input_coverage().is_complete() {
            reasons.push(BaselineReason::CurrentInputsIncomplete);
        }
        if !analysis.ast_grep_scan.coverage.is_complete() {
            reasons.push(BaselineReason::CurrentSecondaryChecksIncomplete);
        }
        if !current.external_checks_complete {
            reasons.push(BaselineReason::ExternalChecksIncomplete);
        }
        if self.availability == BaselineAvailability::Verified {
            if self.architecture.as_ref().is_none_or(|surface| !surface.overview.ast_grep_coverage.is_complete()) {
                reasons.push(BaselineReason::PreviousSecondaryChecksIncomplete);
            }
            if self
                .architecture
                .as_ref()
                .is_none_or(|surface| !surface.overview.input_coverage.is_complete())
            {
                reasons.push(BaselineReason::PreviousInputsIncomplete);
            }
            if let Some(previous) = &self.identity {
                if previous.root != current.root {
                    reasons.push(BaselineReason::DifferentRoot);
                }
                if previous.engine_version != current.engine_version
                    || previous.semantic_revision != current.semantic_revision
                    || previous.engine_fingerprint != current.engine_fingerprint
                {
                    reasons.push(BaselineReason::DifferentEngine);
                }
                if previous.scope_fingerprint != current.scope_fingerprint
                    || current.scope_fingerprint.is_empty()
                {
                    reasons.push(BaselineReason::DifferentScope);
                }
                if previous.external_tools != current.external_tools {
                    reasons.push(BaselineReason::DifferentExternalTools);
                }
                if !previous.external_checks_complete {
                    reasons.push(BaselineReason::ExternalChecksIncomplete);
                }
            }
        }
        let comparison = if self.availability == BaselineAvailability::Missing {
            BaselineComparison::InitialSnapshot
        } else if self.availability == BaselineAvailability::Verified && reasons.is_empty() {
            BaselineComparison::Comparable
        } else {
            BaselineComparison::NotCompared
        };
        BaselineAssessment {
            availability: self.availability,
            comparison,
            current,
            previous: self.identity.clone(),
            reasons,
        }
    }

    pub(super) fn is_verified(&self) -> bool {
        self.availability == BaselineAvailability::Verified
    }
}

fn read_hashed<T: DeserializeOwned>(path: &Path) -> io::Result<Option<(T, String)>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut reader = BufReader::new(HashingIo::new(file));
    let value = serde_json::from_reader(&mut reader).map_err(|error| {
        io::Error::other(format!("invalid baseline {}: {error}", path.display()))
    })?;
    Ok(Some((value, reader.get_ref().content_hash().to_string())))
}
