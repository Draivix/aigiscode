use crate::semantic_models::{
    scan_contract_semantic_model_packs, semantic_model_pack_descriptor,
    SemanticModelContractCategory, SemanticModelPackLayer,
};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const MAX_SAMPLE_LOCATIONS: usize = 5;
const MAX_ITEMS_PER_CATEGORY: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub struct ContractInventory {
    pub summary: ContractInventorySummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_model_packs: Vec<ContractSemanticModelPackUsage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<ContractInventoryItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<ContractInventoryItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registered_keys: Vec<ContractInventoryItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbolic_literals: Vec<ContractInventoryItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_keys: Vec<ContractInventoryItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config_keys: Vec<ContractInventoryItem>,
    /// Pub-sub channel names (emit/listen/publish/subscribe) — the cross-file
    /// wiring that only exists at runtime.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<ContractInventoryItem>,
}

impl ContractInventory {
    pub fn lookup(&self) -> ContractLookup {
        ContractLookup {
            routes: self.routes.iter().map(|item| item.value.clone()).collect(),
            hooks: self.hooks.iter().map(|item| item.value.clone()).collect(),
            registered_keys: self
                .registered_keys
                .iter()
                .map(|item| item.value.clone())
                .collect(),
            symbolic_literals: self
                .symbolic_literals
                .iter()
                .map(|item| item.value.clone())
                .collect(),
            env_keys: self
                .env_keys
                .iter()
                .map(|item| item.value.clone())
                .collect(),
            config_keys: self
                .config_keys
                .iter()
                .map(|item| item.value.clone())
                .collect(),
            channels: self
                .channels
                .iter()
                .map(|item| item.value.clone())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub struct ContractInventorySummary {
    pub routes: ContractCategorySummary,
    pub hooks: ContractCategorySummary,
    pub registered_keys: ContractCategorySummary,
    pub symbolic_literals: ContractCategorySummary,
    pub env_keys: ContractCategorySummary,
    pub config_keys: ContractCategorySummary,
    #[serde(default)]
    pub channels: ContractCategorySummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub struct ContractCategorySummary {
    pub unique_values: usize,
    pub occurrences: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContractInventoryItem {
    pub value: String,
    pub count: usize,
    pub locations: Vec<ContractLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContractLocation {
    pub file_path: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContractLookup {
    pub routes: Vec<String>,
    pub hooks: Vec<String>,
    pub registered_keys: Vec<String>,
    pub symbolic_literals: Vec<String>,
    pub env_keys: Vec<String>,
    pub config_keys: Vec<String>,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContractSemanticModelPackUsage {
    pub id: String,
    pub description: String,
    pub layer: SemanticModelPackLayer,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contract_categories: Vec<String>,
}

impl ContractLookup {
    pub fn contains_literal(&self, value: &str) -> bool {
        [
            &self.routes,
            &self.hooks,
            &self.registered_keys,
            &self.symbolic_literals,
            &self.env_keys,
            &self.config_keys,
            &self.channels,
        ]
        .into_iter()
        .flatten()
        .any(|candidate| candidate == value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractLanguage {
    Php,
    Python,
    Ruby,
    JavaScript,
    TypeScript,
    Vue,
    Rust,
}

#[derive(Debug, Default)]
struct ContractBuckets {
    semantic_model_pack_ids: BTreeSet<&'static str>,
    routes: HashMap<String, ContractEntryAccumulator>,
    hooks: HashMap<String, ContractEntryAccumulator>,
    registered_keys: HashMap<String, ContractEntryAccumulator>,
    symbolic_literals: HashMap<String, ContractEntryAccumulator>,
    env_keys: HashMap<String, ContractEntryAccumulator>,
    config_keys: HashMap<String, ContractEntryAccumulator>,
    channels: HashMap<String, ContractEntryAccumulator>,
}

#[derive(Debug, Default)]
struct ContractEntryAccumulator {
    count: usize,
    locations: Vec<ContractLocation>,
}

pub fn build_contract_inventory(files: &[(PathBuf, String)]) -> ContractInventory {
    let mut buckets = ContractBuckets::default();

    for (path, content) in files {
        let Some(language) = detect_contract_language(path) else {
            continue;
        };
        if is_test_like_path(path) || is_non_runtime_contract_source(path, content) {
            continue;
        }

        // Contract patterns carry their payload inside string arguments
        // (`config('key')`, `Route::get('/path')`), so strings must stay intact
        // — but a call merely *mentioned* in a comment ("falls back to
        // config('mailbox_provisioning')") is documentation, not a declared
        // contract. Blank comment interiors only; Vue SFCs are left raw (their
        // masking happens at the parser layer).
        let comment_masked = crate::lexmask::MaskLanguage::from_path(path).map(|mask_language| {
            crate::lexmask::mask_comments_only(mask_language, content).join("\n")
        });
        let content: &str = comment_masked.as_deref().unwrap_or(content);

        scan_pattern_bucket(&mut buckets.routes, route_patterns(language), path, content);
        scan_pattern_bucket(&mut buckets.hooks, hook_patterns(), path, content);
        // Registered keys are PHP service-container / Artisan / WordPress idioms
        // (`$app->bind`, `->singleton`, `Artisan::command`, `register_setting`).
        // The bare `bind`/`command` verbs otherwise match unrelated frontend calls
        // like a Pusher/Echo `connection.bind('state_change', handler)`.
        if language == ContractLanguage::Php {
            scan_pattern_bucket(
                &mut buckets.registered_keys,
                registered_key_patterns(),
                path,
                content,
            );
        }
        scan_pattern_bucket(&mut buckets.env_keys, env_patterns(), path, content);
        scan_import_meta_env(&mut buckets.env_keys, path, content);
        scan_pattern_bucket(&mut buckets.config_keys, config_patterns(), path, content);
        // Pub-sub channels are runtime wiring: an emit in one file and a
        // listener in another share nothing the call graph can see.
        scan_pattern_bucket(&mut buckets.channels, channel_patterns(), path, content);
        scan_symbolic_literals(&mut buckets.symbolic_literals, path, content, language);
        scan_semantic_model_contracts(&mut buckets, path, content);
    }

    ContractInventory {
        summary: ContractInventorySummary {
            routes: summarize_bucket(&buckets.routes),
            hooks: summarize_bucket(&buckets.hooks),
            registered_keys: summarize_bucket(&buckets.registered_keys),
            symbolic_literals: summarize_bucket(&buckets.symbolic_literals),
            env_keys: summarize_bucket(&buckets.env_keys),
            config_keys: summarize_bucket(&buckets.config_keys),
            channels: summarize_bucket(&buckets.channels),
        },
        semantic_model_packs: serialize_semantic_model_packs(&buckets.semantic_model_pack_ids),
        routes: serialize_bucket(&buckets.routes),
        hooks: serialize_bucket(&buckets.hooks),
        registered_keys: serialize_bucket(&buckets.registered_keys),
        symbolic_literals: serialize_bucket(&buckets.symbolic_literals),
        env_keys: serialize_bucket(&buckets.env_keys),
        config_keys: serialize_bucket(&buckets.config_keys),
        channels: serialize_bucket(&buckets.channels),
    }
}

fn scan_semantic_model_contracts(buckets: &mut ContractBuckets, path: &Path, content: &str) {
    for pack in scan_contract_semantic_model_packs(path, content) {
        register_semantic_model_pack(buckets, pack.pack_id);
        for contract_match in pack.matches {
            match contract_match.category {
                SemanticModelContractCategory::Route => add_bucket_entry(
                    &mut buckets.routes,
                    &contract_match.value,
                    path,
                    contract_match.line,
                ),
                SemanticModelContractCategory::Hook => add_bucket_entry(
                    &mut buckets.hooks,
                    &contract_match.value,
                    path,
                    contract_match.line,
                ),
                SemanticModelContractCategory::ConfigKey => add_bucket_entry(
                    &mut buckets.config_keys,
                    &contract_match.value,
                    path,
                    contract_match.line,
                ),
            }
        }
    }
}

fn summarize_bucket(bucket: &HashMap<String, ContractEntryAccumulator>) -> ContractCategorySummary {
    ContractCategorySummary {
        unique_values: bucket.len(),
        occurrences: bucket.values().map(|entry| entry.count).sum(),
    }
}

fn serialize_bucket(
    bucket: &HashMap<String, ContractEntryAccumulator>,
) -> Vec<ContractInventoryItem> {
    let mut items = bucket
        .iter()
        .map(|(value, entry)| ContractInventoryItem {
            value: value.clone(),
            count: entry.count,
            locations: entry.locations.clone(),
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.value.cmp(&right.value))
    });
    items.truncate(MAX_ITEMS_PER_CATEGORY);
    items
}

fn serialize_semantic_model_packs(
    pack_ids: &BTreeSet<&'static str>,
) -> Vec<ContractSemanticModelPackUsage> {
    pack_ids
        .iter()
        .filter_map(|id| semantic_model_pack_descriptor(id))
        .map(|descriptor| ContractSemanticModelPackUsage {
            id: String::from(descriptor.id),
            description: String::from(descriptor.description),
            layer: descriptor.layer,
            contract_categories: descriptor
                .contract_categories
                .iter()
                .map(|value| String::from(*value))
                .collect(),
        })
        .collect()
}

/// Every file that declares at least one route pattern — full fidelity, no
/// per-category caps (the artifact inventory samples for size; wiring
/// judgments must not inherit that truncation).
pub(crate) fn route_declaring_files(files: &[(PathBuf, String)]) -> HashSet<PathBuf> {
    let mut declaring = HashSet::new();
    for (path, content) in files {
        let Some(language) = detect_contract_language(path) else {
            continue;
        };
        if is_test_like_path(path) || is_non_runtime_contract_source(path, content) {
            continue;
        }
        if route_patterns(language)
            .iter()
            .any(|pattern| pattern.is_match(content))
            || route_declaration_shape().is_match(content)
        {
            declaring.insert(path.clone());
        }
    }
    declaring
}

/// Loose route-declaration shape for wiring judgments only: the inventory
/// patterns demand a leading-slash path literal to extract a value, but a
/// file declaring `Route::get('config', ...)` still wires its controllers.
fn route_declaration_shape() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(
            r"\b(?:Route|Router)(?:::|->)(?:get|post|put|patch|delete|options|any|match|resource|apiResource|group)\s*\(",
        )
        .unwrap()
    })
}

fn scan_pattern_bucket(
    bucket: &mut HashMap<String, ContractEntryAccumulator>,
    patterns: Vec<&Regex>,
    path: &Path,
    content: &str,
) -> usize {
    let mut hits = 0usize;
    for pattern in patterns {
        for captures in pattern.captures_iter(content) {
            let Some(value_match) = captures.name("value") else {
                continue;
            };
            let value = value_match.as_str().trim();
            if value.is_empty() {
                continue;
            }
            let line = line_for_offset(content, value_match.start());
            add_bucket_entry(bucket, value, path, line);
            hits += 1;
        }
    }
    hits
}

fn scan_symbolic_literals(
    bucket: &mut HashMap<String, ContractEntryAccumulator>,
    path: &Path,
    content: &str,
    language: ContractLanguage,
) {
    match language {
        ContractLanguage::JavaScript | ContractLanguage::TypeScript | ContractLanguage::Vue => {
            scan_multiline_type_unions(bucket, path, content);
            for pattern in ts_literal_union_patterns() {
                for captures in pattern.captures_iter(content) {
                    let Some(body) = captures.name("body") else {
                        continue;
                    };
                    add_symbolic_values(bucket, path, content, body.start(), body.as_str());
                }
            }
            for captures in ts_const_array_pattern().captures_iter(content) {
                let Some(body) = captures.name("body") else {
                    continue;
                };
                add_symbolic_values(bucket, path, content, body.start(), body.as_str());
            }
            scan_stimulus_contracts(bucket, path, content);
            scan_custom_events(bucket, path, content);
        }
        ContractLanguage::Php => {
            for captures in php_register_array_pattern().captures_iter(content) {
                let Some(body) = captures.name("body") else {
                    continue;
                };
                for key_match in array_key_pattern().captures_iter(body.as_str()) {
                    let Some(value_match) = key_match.name("value") else {
                        continue;
                    };
                    let value = value_match.as_str().trim();
                    if value.is_empty() {
                        continue;
                    }
                    let offset = body.start() + value_match.start();
                    add_bucket_entry(bucket, value, path, line_for_offset(content, offset));
                }
            }
        }
        ContractLanguage::Python | ContractLanguage::Ruby | ContractLanguage::Rust => {}
    }
}

fn scan_stimulus_contracts(
    bucket: &mut HashMap<String, ContractEntryAccumulator>,
    path: &Path,
    content: &str,
) {
    if let Some(identifier) = stimulus_controller_identifier_for_path(path) {
        add_bucket_entry(bucket, &identifier, path, 1);
    }

    for captures in data_controller_pattern().captures_iter(content) {
        let Some(value_match) = captures.name("value") else {
            continue;
        };
        let line = line_for_offset(content, value_match.start());
        for value in value_match.as_str().split_whitespace() {
            let normalized = value.trim();
            if normalized.is_empty() {
                continue;
            }
            add_bucket_entry(bucket, normalized, path, line);
        }
    }

    for captures in data_action_pattern().captures_iter(content) {
        let Some(value_match) = captures.name("value") else {
            continue;
        };
        let line = line_for_offset(content, value_match.start());
        for action_match in action_controller_pattern().captures_iter(value_match.as_str()) {
            let Some(controller_match) = action_match.name("value") else {
                continue;
            };
            let normalized = controller_match.as_str().trim();
            if normalized.is_empty() {
                continue;
            }
            add_bucket_entry(bucket, normalized, path, line);
        }
    }
}

fn scan_custom_events(
    bucket: &mut HashMap<String, ContractEntryAccumulator>,
    path: &Path,
    content: &str,
) {
    for captures in custom_event_pattern().captures_iter(content) {
        let Some(value_match) = captures.name("value") else {
            continue;
        };
        let value = value_match.as_str().trim();
        if value.is_empty() {
            continue;
        }
        add_bucket_entry(
            bucket,
            value,
            path,
            line_for_offset(content, value_match.start()),
        );
    }
}

fn add_symbolic_values(
    bucket: &mut HashMap<String, ContractEntryAccumulator>,
    path: &Path,
    content: &str,
    start_index: usize,
    body: &str,
) {
    for captures in quoted_value_pattern().captures_iter(body) {
        let Some(value_match) = captures.name("value") else {
            continue;
        };
        let value = value_match.as_str().trim();
        if value.is_empty() {
            continue;
        }
        add_bucket_entry(
            bucket,
            value,
            path,
            line_for_offset(content, start_index + value_match.start()),
        );
    }
}

fn add_bucket_entry(
    bucket: &mut HashMap<String, ContractEntryAccumulator>,
    value: &str,
    path: &Path,
    line: usize,
) {
    let entry = bucket.entry(String::from(value)).or_default();
    entry.count += 1;
    if entry.locations.len() < MAX_SAMPLE_LOCATIONS {
        entry.locations.push(ContractLocation {
            file_path: path.to_path_buf(),
            line,
        });
    }
}

fn scan_multiline_type_unions(
    bucket: &mut HashMap<String, ContractEntryAccumulator>,
    path: &Path,
    content: &str,
) {
    let lines = content.lines().collect::<Vec<_>>();
    let mut offsets = Vec::with_capacity(lines.len());
    let mut cursor = 0usize;
    for line in &lines {
        offsets.push(cursor);
        cursor += line.len() + 1;
    }

    let type_start = multiline_union_start_pattern();
    let mut index = 0usize;
    while index < lines.len() {
        if !type_start.is_match(lines[index]) {
            index += 1;
            continue;
        }
        let mut body_lines = Vec::new();
        let mut next = index + 1;
        while next < lines.len() && lines[next].trim_start().starts_with('|') {
            body_lines.push(lines[next]);
            next += 1;
        }
        if !body_lines.is_empty() {
            add_symbolic_values(
                bucket,
                path,
                content,
                offsets[index],
                &body_lines.join("\n"),
            );
        }
        index = next.max(index + 1);
    }
}

fn register_semantic_model_pack(buckets: &mut ContractBuckets, pack_id: &'static str) {
    buckets.semantic_model_pack_ids.insert(pack_id);
}

fn line_for_offset(content: &str, offset: usize) -> usize {
    content[..offset.min(content.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn detect_contract_language(path: &Path) -> Option<ContractLanguage> {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    if normalized.ends_with(".php") {
        Some(ContractLanguage::Php)
    } else if normalized.ends_with(".py") {
        Some(ContractLanguage::Python)
    } else if normalized.ends_with(".rb") {
        Some(ContractLanguage::Ruby)
    } else if normalized.ends_with(".vue") {
        Some(ContractLanguage::Vue)
    } else if normalized.ends_with(".rs") {
        Some(ContractLanguage::Rust)
    } else if normalized.ends_with(".tsx") || normalized.ends_with(".ts") {
        Some(ContractLanguage::TypeScript)
    } else if normalized.ends_with(".jsx")
        || normalized.ends_with(".js")
        || normalized.ends_with(".mjs")
        || normalized.ends_with(".cjs")
    {
        Some(ContractLanguage::JavaScript)
    } else {
        None
    }
}

fn is_test_like_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();
    let parts = normalized.split('/').filter(|part| !part.is_empty());
    stem.contains("test")
        || parts
            .into_iter()
            .any(|part| matches!(part, "test" | "tests" | "__tests__" | "fixtures" | "spec"))
        || normalized.ends_with("test.php")
        || normalized.ends_with("_test.py")
        || normalized.ends_with("_test.rb")
        || normalized.ends_with("_spec.rb")
}

/// Files that declare or bundle rather than run. The contract inventory captures
/// *runtime* contracts, so TypeScript declaration files (`.d.ts` — compile-time
/// type info whose string-literal unions are type declarations, not usages),
/// minified/generated bundles, and files with an explicit generated-header marker
/// are skipped. On draivix a single generated `entities.d.ts` was the source of
/// 56% of the top symbolic-literal locations (currency unions re-declared per
/// field), crowding out hand-written literals with duplicative type noise.
fn is_non_runtime_contract_source(path: &Path, content: &str) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    if normalized.ends_with(".d.ts")
        || normalized.ends_with(".min.js")
        || normalized.ends_with(".min.css")
        || normalized.contains(".generated.")
    {
        return true;
    }
    let head = content
        .chars()
        .take(600)
        .collect::<String>()
        .to_ascii_lowercase();
    head.contains("@generated")
        || head.contains("auto-generated")
        || head.contains("autogenerated")
        || head.contains("do not edit")
        || head.contains("code generated by")
}

fn stimulus_controller_identifier_for_path(path: &Path) -> Option<String> {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let lower = normalized.to_lowercase();
    let marker = "/controllers/";
    let marker_index = lower.find(marker)?;
    let relative = &normalized[marker_index + marker.len()..];
    let relative_lower = relative.to_lowercase();
    let suffix = [".js", ".ts", ".jsx", ".tsx", ".vue"]
        .into_iter()
        .find(|suffix| relative_lower.ends_with(suffix))?;
    let relative_no_ext = &relative[..relative.len() - suffix.len()];
    let controller_path = relative_no_ext.strip_suffix("_controller")?;
    let parts = controller_path
        .split('/')
        .filter(|part| !part.is_empty())
        .map(|part| part.replace('_', "-"))
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("--"))
}

fn route_patterns(language: ContractLanguage) -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    let patterns = PATTERNS
        .get_or_init(|| {
            vec![
                Regex::new(r#"#\[\s*Route\s*\([^]]*?\bpath\s*:\s*['"](?P<value>/[^'"]*)['"]"#)
                    .unwrap(),
                Regex::new(r#"#\[\s*Route\s*\(\s*['"](?P<value>/[^'"]*)['"]"#).unwrap(),
                Regex::new(r#"\b(?:Route|Router)(?:::|->)(?:get|post|put|patch|delete|options|any|match|resource|apiResource|view|redirect|prefix|group)\s*\(\s*['"](?P<value>/[^'"]*)['"]"#).unwrap(),
                Regex::new(r#"\brouter\.(?:get|post|put|patch|delete|use|all)\s*\(\s*['"`](?P<value>/[^'"`]*)['"`]"#).unwrap(),
                Regex::new(r#"\b(?:get|post|put|patch|delete|match)\s+['"](?P<value>/[^'"]*)['"]"#).unwrap(),
                Regex::new(r#"\bmount\s+[A-Z][A-Za-z0-9_:]+\s*=>\s*['"](?P<value>/[^'"]*)['"]"#).unwrap(),
            ]
        })
        .iter();
    if matches!(language, ContractLanguage::Python) {
        return Vec::new();
    }
    patterns.collect()
}

fn hook_patterns() -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            vec![Regex::new(r#"\b(?:add_action|add_filter|do_action|apply_filters|register_activation_hook|register_deactivation_hook|register_uninstall_hook)\s*\(\s*['"](?P<value>[A-Za-z0-9_.:-]+)['"]"#).unwrap()]
        })
        .iter()
        .collect()
}

fn registered_key_patterns() -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            vec![Regex::new(r#"(?i)\b(?:register_[A-Za-z0-9_]+|bind|singleton|alias|command|schedule)\s*\(\s*['"](?P<value>[A-Za-z0-9_.:-]+)['"]"#).unwrap()]
        })
        .iter()
        .collect()
}

fn env_patterns() -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            vec![
                Regex::new(r#"\benv\s*\(\s*['"](?P<value>[A-Z][A-Z0-9_]*)['"]"#).unwrap(),
                Regex::new(r#"\bgetenv\s*\(\s*['"](?P<value>[A-Z][A-Z0-9_]*)['"]"#).unwrap(),
                Regex::new(
                    r#"\b(?:(?:std::)?env::(?:var|var_os)\s*\(\s*|(?:std::)?env!\s*\(\s*|option_env!\s*\(\s*)["'](?P<value>[A-Z][A-Z0-9_]*)["']"#,
                )
                .unwrap(),
                Regex::new(r#"\$_(?:ENV|SERVER)\s*\[\s*['"](?P<value>[A-Z][A-Z0-9_]*)['"]\s*\]"#)
                    .unwrap(),
                Regex::new(r#"\bos\.getenv\s*\(\s*['"](?P<value>[A-Z][A-Z0-9_]*)['"]"#).unwrap(),
                Regex::new(r#"\bos\.environ(?:\.get)?\s*\(\s*['"](?P<value>[A-Z][A-Z0-9_]*)['"]"#)
                    .unwrap(),
                Regex::new(r#"\bos\.environ\s*\[\s*['"](?P<value>[A-Z][A-Z0-9_]*)['"]\s*\]"#)
                    .unwrap(),
                Regex::new(r#"\bENV\.fetch\s*\(\s*['"](?P<value>[A-Z][A-Z0-9_]*)['"]"#).unwrap(),
                Regex::new(r#"\bENV\s*\[\s*['"](?P<value>[A-Z][A-Z0-9_]*)['"]\s*\]"#).unwrap(),
                Regex::new(r#"\bprocess\.env\.(?P<value>[A-Z][A-Z0-9_]*)"#).unwrap(),
                Regex::new(r#"\bprocess\.env\s*\[\s*['"](?P<value>[A-Z][A-Z0-9_]*)['"]\s*\]"#)
                    .unwrap(),
            ]
        })
        .iter()
        .collect()
}

fn import_meta_env_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| Regex::new(r#"\bimport\.meta\.env\.(?P<value>[A-Z][A-Z0-9_]*)"#).unwrap())
}

/// Vite exposes a fixed set of built-in compile-time constants on
/// `import.meta.env` (`MODE`/`DEV`/`PROD`/`SSR`/`BASE_URL`). These are build-tool
/// internals, not application environment contracts — real app env vars are
/// prefixed (default `VITE_`). Excluding them keeps the env-key inventory from
/// being dominated by framework noise (`DEV` alone appeared 75x on draivix).
fn is_vite_builtin_env(value: &str) -> bool {
    matches!(value, "MODE" | "DEV" | "PROD" | "SSR" | "BASE_URL")
}

fn scan_import_meta_env(
    bucket: &mut HashMap<String, ContractEntryAccumulator>,
    path: &Path,
    content: &str,
) {
    for captures in import_meta_env_pattern().captures_iter(content) {
        let Some(value_match) = captures.name("value") else {
            continue;
        };
        let value = value_match.as_str().trim();
        if value.is_empty() || is_vite_builtin_env(value) {
            continue;
        }
        let line = line_for_offset(content, value_match.start());
        add_bucket_entry(bucket, value, path, line);
    }
}

fn config_patterns() -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            vec![
                Regex::new(r#"\bconfig\s*\(\s*['"](?P<value>[A-Za-z0-9_.:-]+)['"]"#).unwrap(),
                Regex::new(r#"\b(?:get_option|update_option|add_option|delete_option|register_setting)\s*\(\s*['"](?P<value>[A-Za-z0-9_.:-]+)['"]"#).unwrap(),
            ]
        })
        .iter()
        .collect()
}

/// Pub-sub channel usage, both sides of the wire. Patterns are receiver- or
/// verb-scoped so ordinary event APIs (DOM `.on('click')`) stay out; template-
/// literal channels (`entity.${id}`) are captured with their hole.
fn channel_patterns() -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            let value = r#"['\"`](?P<value>[A-Za-z0-9_.:/${}-]+)['\"`]"#;
            vec![
                // Socket.IO / ws send side.
                Regex::new(&format!(r#"\bsocket\s*\.\s*(?:emit|send)\s*\(\s*{value}"#)).unwrap(),
                // Socket.IO / EventEmitter listen side — receiver-scoped so DOM
                // and jQuery `.on(...)` handlers do not leak in.
                Regex::new(&format!(r#"\b(?:socket|io|ws)\s*\.\s*on\s*\(\s*{value}"#)).unwrap(),
                // Laravel Echo (channel/private/join subscribe + listen).
                Regex::new(&format!(
                    r#"\bEcho\s*\.\s*(?:channel|private|join|listen)\s*\(\s*{value}"#
                ))
                .unwrap(),
                // Laravel channel authorization + broadcastOn channel shapes.
                Regex::new(&format!(r#"\bBroadcast::channel\s*\(\s*{value}"#)).unwrap(),
                Regex::new(&format!(
                    r#"\bnew\s+(?:PrivateChannel|PresenceChannel|Channel)\s*\(\s*{value}"#
                ))
                .unwrap(),
                // Redis / generic pub-sub.
                Regex::new(&format!(r#"(?:->|\.)publish\s*\(\s*{value}"#)).unwrap(),
                Regex::new(&format!(r#"(?:->|\.)p?subscribe\s*\(\s*{value}"#)).unwrap(),
                // Pusher-style trigger.
                Regex::new(&format!(r#"(?:->|\.)trigger\s*\(\s*{value}"#)).unwrap(),
            ]
        })
        .iter()
        .collect()
}

fn ts_literal_union_patterns() -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            vec![
                Regex::new(r#"(?m)\b(?:export\s+)?type\s+[A-Za-z0-9_<>,\s]+\s*=\s*(?P<body>(?:['"][A-Za-z0-9_.:/-]+['"]\s*\|\s*)+['"][A-Za-z0-9_.:/-]+['"])"#).unwrap(),
                Regex::new(r#"(?m)[(:]\s*(?P<body>(?:['"][A-Za-z0-9_.:/-]+['"]\s*\|\s*)+['"][A-Za-z0-9_.:/-]+['"])\s*(?:[;,\)])"#).unwrap(),
            ]
        })
        .iter()
        .collect()
}

fn ts_const_array_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r#"(?m)\[(?P<body>(?:\s*['"][A-Za-z0-9_.:/-]+['"]\s*,?)+)\]\s+as\s+const"#)
            .unwrap()
    })
}

fn php_register_array_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r#"(?is)\bregister_[A-Za-z0-9_]+\s*\(\s*array\s*\((?P<body>.{0,2000}?)\)\s*\)"#)
            .unwrap()
    })
}

fn array_key_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r#"['"](?P<value>[A-Za-z0-9_.:-]+)['"]\s*=>"#).unwrap())
}

fn quoted_value_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r#"['"](?P<value>[A-Za-z0-9_.:/-]+)['"]"#).unwrap())
}

fn custom_event_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r#"\bnew\s+CustomEvent\s*\(\s*['"](?P<value>[A-Za-z0-9_.:-]+)['"]"#).unwrap()
    })
}

fn data_controller_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r#"(?i)data-controller\s*=\s*['"](?P<value>[^'"]+)['"]"#).unwrap()
    })
}

fn data_action_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| Regex::new(r#"(?i)data-action\s*=\s*['"](?P<value>[^'"]+)['"]"#).unwrap())
}

fn action_controller_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r#"(?:->)?(?P<value>[A-Za-z0-9_-]+)#"#).unwrap())
}

fn multiline_union_start_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| Regex::new(r#"\b(?:export\s+)?type\s+[A-Za-z0-9_<>,\s]+\s*=\s*$"#).unwrap())
}

#[cfg(test)]
mod tests {
    use super::{build_contract_inventory, ContractCategorySummary};
    use std::path::{Path, PathBuf};

    #[test]
    fn channel_patterns_capture_both_sides_of_pub_sub_wiring() {
        let files = vec![
            (
                PathBuf::from("resources/js/realtime.js"),
                String::from(
                    "Echo.private('orders.5').listen('OrderUpdated');\n\
                     socket.emit('chat.message', payload);\n\
                     socket.on('chat.typing', handler);\n\
                     button.on('click', handler);\n",
                ),
            ),
            (
                PathBuf::from("app/Services/PushService.php"),
                String::from(
                    "<?php\n\
                     $redis->publish('orders.updated', $payload);\n\
                     $redis->subscribe('orders.cancelled');\n\
                     Broadcast::channel('orders.{id}', $callback);\n\
                     return [new PrivateChannel('users.' . $id)];\n",
                ),
            ),
        ];
        let inventory = build_contract_inventory(&files);
        let values = inventory
            .channels
            .iter()
            .map(|item| {
                (
                    item.value.as_str(),
                    item.locations[0].file_path.to_str().unwrap(),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        for expected in [
            "orders.5",
            "chat.message",
            "chat.typing",
            "orders.updated",
            "orders.cancelled",
            "orders.{id}",
        ] {
            assert!(
                values.contains_key(expected),
                "missing channel `{expected}`, got: {values:?}"
            );
        }
        assert_eq!(
            values.get("orders.updated"),
            Some(&"app/Services/PushService.php")
        );
        // DOM event handlers are not pub-sub channels.
        assert!(!values.contains_key("click"));
        assert!(inventory.summary.channels.occurrences >= 6);
    }

    #[test]
    fn route_declaring_files_accepts_slashless_declarations() {
        let files = vec![(
            PathBuf::from("app/Modules/M/routes/api.php"),
            String::from("<?php\nRoute::get('config', [ProxyController::class, 'getConfig']);\n"),
        )];
        let declaring = super::route_declaring_files(&files);
        assert!(declaring.contains(&PathBuf::from("app/Modules/M/routes/api.php")));
    }

    #[test]
    fn builds_contract_inventory_for_routes_hooks_and_runtime_keys() {
        let inventory = build_contract_inventory(&[
            (
                PathBuf::from("routes/web.php"),
                String::from(
                    r#"
Route::get('/users', 'UserController@index');
Route::post('/users', 'UserController@store');
do_action('init');
config('mail.driver');
env('APP_ENV');
"#,
                ),
            ),
            (
                PathBuf::from("src/runtime.ts"),
                String::from(
                    r#"
type Status = 'draft' | 'published';
const keys = ['alpha', 'beta'] as const;
const mode = process.env.APP_MODE;
const event = new CustomEvent('panel.opened');
"#,
                ),
            ),
            (
                PathBuf::from("resources/views/demo.vue"),
                String::from(
                    r#"<div data-controller="search-panel" data-action="click->search-panel#open"></div>"#,
                ),
            ),
        ]);

        assert_eq!(
            inventory.summary.routes,
            ContractCategorySummary {
                unique_values: 1,
                occurrences: 2,
            }
        );
        assert!(inventory.routes.iter().any(|item| item.value == "/users"));
        assert!(inventory.hooks.iter().any(|item| item.value == "init"));
        assert!(inventory
            .config_keys
            .iter()
            .any(|item| item.value == "mail.driver"));
        assert!(inventory
            .env_keys
            .iter()
            .any(|item| item.value == "APP_ENV"));
        assert!(inventory
            .env_keys
            .iter()
            .any(|item| item.value == "APP_MODE"));
        assert!(inventory
            .symbolic_literals
            .iter()
            .any(|item| item.value == "draft"));
        assert!(inventory
            .symbolic_literals
            .iter()
            .any(|item| item.value == "panel.opened"));
        assert!(inventory
            .symbolic_literals
            .iter()
            .any(|item| item.value == "search-panel"));
    }

    #[test]
    fn contract_calls_inside_comments_are_not_declared_contracts() {
        let inventory = build_contract_inventory(&[(
            PathBuf::from("app/Policies/MailPolicy.php"),
            String::from(
                r#"<?php
/**
 * Falls back to config('mailbox_provisioning') when no row exists.
 */
class MailPolicy {
    public function limit(): int {
        // reads config('commented_key') — prose, not a call
        return (int) config('mail.provision_limit');
    }
}
"#,
            ),
        )]);

        let keys: Vec<&str> = inventory
            .config_keys
            .iter()
            .map(|item| item.value.as_str())
            .collect();
        assert!(
            keys.contains(&"mail.provision_limit"),
            "real config call captured: {keys:?}"
        );
        assert!(
            !keys.contains(&"mailbox_provisioning") && !keys.contains(&"commented_key"),
            "config keys mentioned only in comments must not be contracts: {keys:?}"
        );
    }

    #[test]
    fn excludes_vite_builtin_env_but_keeps_real_frontend_env_keys() {
        let inventory = build_contract_inventory(&[(
            PathBuf::from("resources/js/config.ts"),
            String::from(
                r#"
export const cfg = {
    apiBase: import.meta.env.VITE_API_URL,
    debug: import.meta.env.DEV,
    mode: import.meta.env.MODE,
    prod: import.meta.env.PROD,
};
"#,
            ),
        )]);

        assert!(
            inventory
                .env_keys
                .iter()
                .any(|item| item.value == "VITE_API_URL"),
            "real prefixed frontend env key must be captured"
        );
        for builtin in ["DEV", "MODE", "PROD", "SSR", "BASE_URL"] {
            assert!(
                inventory.env_keys.iter().all(|item| item.value != builtin),
                "Vite built-in `{builtin}` must not be an env-key contract"
            );
        }
    }

    #[test]
    fn registered_keys_ignore_frontend_bind_calls() {
        let inventory = build_contract_inventory(&[
            (
                PathBuf::from("app/Providers/AppServiceProvider.php"),
                String::from(r#"$this->app->bind('audit.hydrator', ActionHydrator::class);"#),
            ),
            (
                PathBuf::from("resources/js/realtime/RealtimeTransportAdapter.ts"),
                String::from(r#"connectionBinding.bind('state_change', handler);"#),
            ),
        ]);

        assert!(
            inventory
                .registered_keys
                .iter()
                .any(|item| item.value == "audit.hydrator"),
            "PHP container binding must be captured"
        );
        assert!(
            inventory
                .registered_keys
                .iter()
                .all(|item| item.value != "state_change"),
            "frontend .bind() event listener must not be a registered key"
        );
    }

    #[test]
    fn skips_typescript_declaration_and_generated_files() {
        let inventory = build_contract_inventory(&[
            (
                PathBuf::from("resources/js/types/entities.d.ts"),
                String::from(r#"export type Currency = "CZK" | "EUR" | "USD";"#),
            ),
            (
                PathBuf::from("resources/js/status.ts"),
                String::from(r#"export type Status = "draft" | "published";"#),
            ),
        ]);

        assert!(
            inventory
                .symbolic_literals
                .iter()
                .all(|item| item.value != "CZK" && item.value != "EUR"),
            "type-union literals from a .d.ts declaration file must be skipped"
        );
        assert!(
            inventory
                .symbolic_literals
                .iter()
                .any(|item| item.value == "draft"),
            "literals from a real .ts source must still be captured"
        );
    }

    #[test]
    fn skips_test_like_paths() {
        let inventory = build_contract_inventory(&[(
            PathBuf::from("tests/routes/test_demo.php"),
            String::from(r#"Route::get('/skip', 'DemoController@index');"#),
        )]);

        assert_eq!(inventory.summary.routes.unique_values, 0);
        assert!(inventory.routes.is_empty());
    }

    #[test]
    fn builds_contract_inventory_for_django_and_declarative_php_hooks() {
        let inventory = build_contract_inventory(&[
            (
                PathBuf::from("django/contrib/auth/urls.py"),
                String::from(
                    r#"
from django.urls import path, re_path

urlpatterns = [
    path("login/", views.LoginView.as_view(), name="login"),
    re_path(r"reset/<uidb64>/<token>/", views.PasswordResetConfirmView.as_view()),
]
"#,
                ),
            ),
            (
                PathBuf::from("django/contrib/auth/signals.py"),
                String::from(
                    r#"
from django.dispatch import Signal, receiver
from django.core.signals import setting_changed

user_logged_in = Signal()

@receiver(setting_changed)
def reset_hashers(**kwargs):
    pass
"#,
                ),
            ),
            (
                PathBuf::from("django/contrib/auth/apps.py"),
                String::from(
                    r#"
from django.db.models.signals import post_migrate

def ready():
    post_migrate.connect(create_permissions, dispatch_uid="django.contrib.auth.management.create_permissions")
"#,
                ),
            ),
            (
                PathBuf::from("django/contrib/auth/hashers.py"),
                String::from(
                    r#"
from django.conf import settings

def get_hashers():
    return settings.PASSWORD_HASHERS, getattr(settings, "DEFAULT_HASHING_ALGORITHM", None)
"#,
                ),
            ),
            (
                PathBuf::from("app/Hooks/example.hook.php"),
                String::from(
                    r#"
<?php
return [
    'beforeUpdate' => [
        function (): void {},
    ],
    'afterSave' => [
        function (): void {},
    ],
];
"#,
                ),
            ),
            (
                PathBuf::from("app/Entities/User/User.hooks.php"),
                String::from(
                    r#"
<?php
return [
    'beforeSave' => [
        'sync' => ['handler' => function (): void {}],
    ],
    'afterDelete' => [
        function (): void {},
    ],
];
"#,
                ),
            ),
        ]);

        assert!(inventory.routes.iter().any(|item| item.value == "login/"));
        assert!(inventory
            .routes
            .iter()
            .any(|item| item.value == "reset/<uidb64>/<token>/"));
        assert!(inventory
            .hooks
            .iter()
            .any(|item| item.value == "user_logged_in"));
        assert!(inventory
            .hooks
            .iter()
            .any(|item| item.value == "setting_changed"));
        assert!(inventory
            .hooks
            .iter()
            .any(|item| item.value == "post_migrate"));
        assert!(inventory
            .hooks
            .iter()
            .any(|item| item.value == "beforeUpdate"));
        assert!(inventory.hooks.iter().any(|item| item.value == "afterSave"));
        assert!(inventory
            .hooks
            .iter()
            .any(|item| item.value == "beforeSave"));
        assert!(inventory
            .hooks
            .iter()
            .any(|item| item.value == "afterDelete"));
        assert!(inventory
            .config_keys
            .iter()
            .any(|item| item.value == "PASSWORD_HASHERS"));
        assert!(inventory
            .config_keys
            .iter()
            .any(|item| item.value == "DEFAULT_HASHING_ALGORITHM"));
        assert!(inventory
            .semantic_model_packs
            .iter()
            .any(|pack| pack.id == "django_routes"));
        assert!(inventory
            .semantic_model_packs
            .iter()
            .any(|pack| pack.id == "django_signals"));
        assert!(inventory
            .semantic_model_packs
            .iter()
            .any(|pack| pack.id == "django_settings"));
        assert!(inventory
            .semantic_model_packs
            .iter()
            .any(|pack| pack.id == "php_hook_maps"));
    }

    #[test]
    fn captures_php_attribute_routes_for_symfony_style_controllers() {
        let inventory = build_contract_inventory(&[(
            PathBuf::from("src/Controller/TriggerFlowController.php"),
            String::from(
                r#"
<?php

use Symfony\Component\Routing\Attribute\Route;

#[Route(defaults: ['_routeScope' => ['api']])]
class TriggerFlowController
{
    #[Route(path: '/api/_action/trigger-event/{eventName}', name: 'api.action.trigger_event', methods: ['POST'])]
    public function trigger(): void
    {
    }

    #[Route('/api/_action/trigger-test', name: 'api.action.trigger_test', methods: ['GET'])]
    public function triggerTest(): void
    {
    }
}
"#,
            ),
        )]);

        assert!(inventory
            .routes
            .iter()
            .any(|item| item.value == "/api/_action/trigger-event/{eventName}"));
        assert!(inventory
            .routes
            .iter()
            .any(|item| item.value == "/api/_action/trigger-test"));
    }

    #[test]
    fn tracks_wordpress_rest_route_semantic_model_pack_usage() {
        let inventory = build_contract_inventory(&[(
            PathBuf::from("wordpress/src/wp-includes/rest-api/endpoints/demo.php"),
            String::from(
                r#"
add_action('rest_api_init', function () {
    register_rest_route('wp/v2', '/widgets', [
        'methods' => 'GET',
    ]);
});
"#,
            ),
        )]);

        assert!(inventory
            .routes
            .iter()
            .any(|item| item.value == "/wp/v2/widgets"));
        assert!(inventory
            .semantic_model_packs
            .iter()
            .any(|pack| pack.id == "wordpress_rest_routes"));
    }

    #[test]
    fn does_not_treat_generic_php_path_calls_as_routes() {
        let inventory = build_contract_inventory(&[(
            PathBuf::from("wp-includes/getid3.php"),
            String::from(
                r#"
$dir = rtrim(str_replace(array('/', '\\'), DIRECTORY_SEPARATOR, $this->getid3->option_save_attachments), DIRECTORY_SEPARATOR);
throw new Exception('supplied path ('.$dir.') does not exist, or is not writable');
"#,
            ),
        )]);

        assert!(inventory.routes.is_empty());
        assert_eq!(inventory.summary.routes.unique_values, 0);
    }

    #[test]
    fn records_wordpress_rest_route_locations_even_when_route_is_dynamic() {
        let inventory = build_contract_inventory(&[(
            PathBuf::from(
                "wp-includes/rest-api/endpoints/class-wp-rest-widget-types-controller.php",
            ),
            String::from(
                r#"
public function register_routes() {
    register_rest_route(
        $this->namespace,
        '/' . $this->rest_base,
        array()
    );
}
"#,
            ),
        )]);

        assert!(inventory
            .routes
            .iter()
            .any(|item| item.value == "wordpress.rest_route"));
        assert!(inventory.routes.iter().any(|item| {
            item.value == "wordpress.rest_route"
                && item.locations.iter().any(|location| {
                    location.file_path
                    == Path::new(
                        "wp-includes/rest-api/endpoints/class-wp-rest-widget-types-controller.php",
                    )
                    && location.line == 3
                })
        }));
    }
}
