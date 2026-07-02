use crate::detectors::hardwiring::HardwiringCategory;
use crate::ingestion::pipeline::ProjectAnalysis;
use crate::review::{load_review_surface, ReviewSurface};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::POLICY_FILE;

pub const POLICY_SUGGESTED_FILE: &str = "policy.suggested.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TuneSuggestion {
    pub field: String,
    pub value: JsonValue,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TuneSummary {
    pub visible_findings: usize,
    pub accepted_by_policy: usize,
    pub suppressed_by_rule: usize,
    pub runtime_entry_candidates: usize,
    pub repeated_literal_values: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TuneOutput {
    pub root: PathBuf,
    pub current_policy_path: PathBuf,
    pub suggested_policy_path: PathBuf,
    pub suggested_policy: JsonValue,
    pub suggestions: Vec<TuneSuggestion>,
    pub summary: TuneSummary,
}

pub fn suggest_policy_patch(
    analysis: &ProjectAnalysis,
    review_surface: &ReviewSurface,
) -> TuneOutput {
    let current_policy_path = analysis.root.join(POLICY_FILE);
    let suggested_policy_path = analysis.root.join(".aigiscode").join(POLICY_SUGGESTED_FILE);
    let existing_policy = read_json_map(&current_policy_path).unwrap_or_default();

    let runtime_entry_paths = analysis
        .graph_analysis
        .runtime_entry_candidates
        .iter()
        .map(|path| normalize_path(path.as_path()))
        .collect::<Vec<_>>();
    let repeated_literal_stats = repeated_literal_stats(analysis);

    let mut suggestions = Vec::new();
    let mut patch = JsonMap::new();

    let existing_orphan_patterns =
        nested_string_array(&existing_policy, &["graph", "orphan_entry_patterns"]);
    let orphan_candidates = runtime_entry_paths
        .iter()
        .filter(|path| !existing_orphan_patterns.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    if !orphan_candidates.is_empty() {
        insert_nested(
            &mut patch,
            &["graph", "orphan_entry_patterns"],
            JsonValue::Array(
                orphan_candidates
                    .iter()
                    .cloned()
                    .map(JsonValue::String)
                    .collect(),
            ),
        );
        suggestions.push(TuneSuggestion {
            field: String::from("graph.orphan_entry_patterns"),
            value: JsonValue::Array(
                orphan_candidates
                    .iter()
                    .cloned()
                    .map(JsonValue::String)
                    .collect(),
            ),
            reason: format!(
                "These {} files were classified as runtime entry candidates rather than structural orphans and can be suppressed explicitly instead of re-triaged every run.",
                orphan_candidates.len()
            ),
        });
    }

    let existing_min_occurrences = nested_usize(
        &existing_policy,
        &["hardwiring", "repeated_literal_min_occurrences"],
    )
    .unwrap_or(2);
    if should_raise_repeated_literal_threshold(&repeated_literal_stats, existing_min_occurrences) {
        let proposed = existing_min_occurrences.max(3);
        insert_nested(
            &mut patch,
            &["hardwiring", "repeated_literal_min_occurrences"],
            JsonValue::from(proposed as u64),
        );
        suggestions.push(TuneSuggestion {
            field: String::from("hardwiring.repeated_literal_min_occurrences"),
            value: JsonValue::from(proposed as u64),
            reason: format!(
                "{} repeated-literal values occur exactly twice while only {} occur four or more times; raising the threshold to {} should cut low-signal duplication noise without hiding stronger repetition clusters.",
                repeated_literal_stats.exactly_two_values,
                repeated_literal_stats.four_or_more_values,
                proposed
            ),
        });
    }

    TuneOutput {
        root: analysis.root.clone(),
        current_policy_path,
        suggested_policy_path,
        suggested_policy: JsonValue::Object(patch),
        suggestions,
        summary: TuneSummary {
            visible_findings: review_surface.summary.visible_findings,
            accepted_by_policy: review_surface.summary.accepted_by_policy,
            suppressed_by_rule: review_surface.summary.suppressed_by_rule,
            runtime_entry_candidates: runtime_entry_paths.len(),
            repeated_literal_values: repeated_literal_stats.total_values,
        },
    }
}

pub fn write_policy_suggestion(
    output: &TuneOutput,
    output_dir: Option<&Path>,
) -> io::Result<PathBuf> {
    let output_path = output_dir
        .map(|dir| dir.join(POLICY_SUGGESTED_FILE))
        .unwrap_or_else(|| output.suggested_policy_path.clone());
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut payload = serde_json::to_vec_pretty(&output.suggested_policy).map_err(|error| {
        io::Error::other(format!("failed to serialize suggested policy: {error}"))
    })?;
    payload.push(b'\n');
    fs::write(&output_path, payload)?;
    Ok(output_path)
}

pub fn load_or_build_review_surface(
    analysis: &ProjectAnalysis,
) -> Result<ReviewSurface, super::PolicyLoadError> {
    load_review_surface(analysis)
}

#[derive(Debug, Default)]
struct RepeatedLiteralStats {
    total_values: usize,
    exactly_two_values: usize,
    three_or_more_values: usize,
    four_or_more_values: usize,
}

fn repeated_literal_stats(analysis: &ProjectAnalysis) -> RepeatedLiteralStats {
    let mut counts = BTreeMap::<String, usize>::new();
    for finding in &analysis.hardwiring.findings {
        if finding.category != HardwiringCategory::RepeatedLiteral {
            continue;
        }
        *counts.entry(finding.value.clone()).or_default() += 1;
    }

    RepeatedLiteralStats {
        total_values: counts.len(),
        exactly_two_values: counts.values().filter(|count| **count == 2).count(),
        three_or_more_values: counts.values().filter(|count| **count >= 3).count(),
        four_or_more_values: counts.values().filter(|count| **count >= 4).count(),
    }
}

fn should_raise_repeated_literal_threshold(
    stats: &RepeatedLiteralStats,
    current_threshold: usize,
) -> bool {
    current_threshold < 3
        && stats.exactly_two_values >= 10
        && stats.exactly_two_values > stats.three_or_more_values
}

fn read_json_map(path: &Path) -> Option<JsonMap<String, JsonValue>> {
    let payload = fs::read(path).ok()?;
    let value = serde_json::from_slice::<JsonValue>(&payload).ok()?;
    value.as_object().cloned()
}

fn nested_string_array(root: &JsonMap<String, JsonValue>, path: &[&str]) -> HashSet<String> {
    nested_value(root, path)
        .and_then(JsonValue::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(JsonValue::as_str)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

fn nested_usize(root: &JsonMap<String, JsonValue>, path: &[&str]) -> Option<usize> {
    nested_value(root, path)
        .and_then(JsonValue::as_u64)
        .map(|value| value as usize)
}

fn nested_value<'a>(root: &'a JsonMap<String, JsonValue>, path: &[&str]) -> Option<&'a JsonValue> {
    let (first, rest) = path.split_first()?;
    let value = root.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    value.as_object().and_then(|next| nested_value(next, rest))
}

fn insert_nested(target: &mut JsonMap<String, JsonValue>, path: &[&str], value: JsonValue) {
    if let Some((first, rest)) = path.split_first() {
        if rest.is_empty() {
            target.insert(String::from(*first), value);
            return;
        }

        let entry = target
            .entry(String::from(*first))
            .or_insert_with(|| JsonValue::Object(JsonMap::new()));
        let object = entry
            .as_object_mut()
            .expect("nested policy patch path should remain object");
        insert_nested(object, rest, value);
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::{load_or_build_review_surface, suggest_policy_patch, write_policy_suggestion};
    use crate::ingestion::pipeline::analyze_project;
    use crate::ingestion::scan::ScanConfig;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn suggests_runtime_entry_patterns_and_literal_threshold() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(fixture.join("src/app.ts"), br#"export const ready = true;"#).unwrap();
        fs::write(
            fixture.join("src/index.ts"),
            br#"import { ready } from "./app";
if (!ready) { throw new Error("boot failed"); }"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/main.ts"),
            br#"const a = "alpha.v1";
const c = "beta.v1";
const e = "gamma.v1";
const g = "delta.v1";
const i = "epsilon.v1";
const k = "zeta.v1";
const m = "lambda.v1";
const o = "theta.v1";
const q = "iota.v1";
const s = "kappa.v1";"#,
        )
        .unwrap();
        fs::write(
            fixture.join("src/other.ts"),
            br#"const b = "alpha.v1";
const d = "beta.v1";
const f = "gamma.v1";
const h = "delta.v1";
const j = "epsilon.v1";
const l = "zeta.v1";
const n = "lambda.v1";
const p = "theta.v1";
const r = "iota.v1";
const t = "kappa.v1";"#,
        )
        .unwrap();
        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let review_surface = load_or_build_review_surface(&analysis).unwrap();
        let output = suggest_policy_patch(&analysis, &review_surface);

        assert!(output
            .suggestions
            .iter()
            .any(|suggestion| suggestion.field == "graph.orphan_entry_patterns"));
        assert!(output
            .suggestions
            .iter()
            .any(|suggestion| suggestion.field == "hardwiring.repeated_literal_min_occurrences"));
        let policy = output.suggested_policy.as_object().unwrap();
        assert!(policy["graph"]["orphan_entry_patterns"].is_array());
        assert_eq!(
            policy["hardwiring"]["repeated_literal_min_occurrences"],
            Value::from(3)
        );
    }

    #[test]
    fn writes_policy_suggested_json() {
        let fixture = create_fixture();
        fs::create_dir_all(fixture.join("src")).unwrap();
        fs::write(
            fixture.join("src/main.rs"),
            br#"fn main() { let _ = std::env::var("APP_MODE"); }"#,
        )
        .unwrap();

        let analysis = analyze_project(&fixture, &ScanConfig::default()).unwrap();
        let review_surface = load_or_build_review_surface(&analysis).unwrap();
        let output = suggest_policy_patch(&analysis, &review_surface);
        let path = write_policy_suggestion(&output, None).unwrap();

        assert!(path.ends_with(".aigiscode/policy.suggested.json"));
        let payload: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert!(payload.is_object());
    }

    fn create_fixture() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aigiscore-policy-tune-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
