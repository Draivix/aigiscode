use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceAnchor {
    pub file_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    pub label: String,
}
