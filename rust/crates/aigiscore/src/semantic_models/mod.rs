use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SemanticModelPackLayer {
    Ecosystem,
    Framework,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticModelPackDescriptor {
    pub id: &'static str,
    pub description: &'static str,
    pub layer: SemanticModelPackLayer,
    pub contract_categories: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticModelContractCategory {
    Route,
    Hook,
    ConfigKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticModelContractMatch {
    pub category: SemanticModelContractCategory,
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticModelPackScan {
    pub pack_id: &'static str,
    pub matches: Vec<SemanticModelContractMatch>,
}

pub fn built_in_semantic_model_packs() -> &'static [SemanticModelPackDescriptor] {
    &[
        SemanticModelPackDescriptor {
            id: "django_routes",
            description: "Recognize Django path()/re_path() route contracts as externally reachable semantic surfaces.",
            layer: SemanticModelPackLayer::Framework,
            contract_categories: &["routes"],
        },
        SemanticModelPackDescriptor {
            id: "django_signals",
            description: "Recognize Django Signal(), @receiver(...), and signal.connect(...) contracts as semantic hook surfaces.",
            layer: SemanticModelPackLayer::Framework,
            contract_categories: &["hooks"],
        },
        SemanticModelPackDescriptor {
            id: "django_settings",
            description: "Recognize Django settings.FOO and getattr(settings, ...) usage as sanctioned configuration contracts.",
            layer: SemanticModelPackLayer::Framework,
            contract_categories: &["config_keys"],
        },
        SemanticModelPackDescriptor {
            id: "php_hook_maps",
            description: "Recognize declarative PHP *.hook.php / *.hooks.php maps as semantic lifecycle hook contracts.",
            layer: SemanticModelPackLayer::Ecosystem,
            contract_categories: &["hooks"],
        },
        SemanticModelPackDescriptor {
            id: "wordpress_rest_routes",
            description: "Recognize WordPress register_rest_route(...) declarations as externally reachable REST contracts.",
            layer: SemanticModelPackLayer::Framework,
            contract_categories: &["routes"],
        },
    ]
}

pub fn semantic_model_pack_descriptor(id: &str) -> Option<&'static SemanticModelPackDescriptor> {
    built_in_semantic_model_packs()
        .iter()
        .find(|descriptor| descriptor.id == id)
}

pub fn scan_contract_semantic_model_packs(
    path: &Path,
    content: &str,
) -> Vec<SemanticModelPackScan> {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    let mut packs = Vec::new();

    if normalized.ends_with(".py") {
        let route_matches = scan_regex_contracts(
            django_route_patterns(),
            SemanticModelContractCategory::Route,
            content,
        );
        if !route_matches.is_empty() {
            packs.push(SemanticModelPackScan {
                pack_id: "django_routes",
                matches: route_matches,
            });
        }

        let signal_matches = scan_regex_contracts(
            django_signal_patterns(),
            SemanticModelContractCategory::Hook,
            content,
        );
        if !signal_matches.is_empty() {
            packs.push(SemanticModelPackScan {
                pack_id: "django_signals",
                matches: signal_matches,
            });
        }

        let settings_matches = scan_regex_contracts(
            django_settings_patterns(),
            SemanticModelContractCategory::ConfigKey,
            content,
        );
        if !settings_matches.is_empty() {
            packs.push(SemanticModelPackScan {
                pack_id: "django_settings",
                matches: settings_matches,
            });
        }
    }

    if normalized.ends_with(".php") {
        let hook_map_matches = scan_php_hook_map_contracts(path, content);
        if !hook_map_matches.is_empty() {
            packs.push(SemanticModelPackScan {
                pack_id: "php_hook_maps",
                matches: hook_map_matches,
            });
        }

        let wordpress_rest_matches = scan_wordpress_rest_route_contracts(content);
        if !wordpress_rest_matches.is_empty() {
            packs.push(SemanticModelPackScan {
                pack_id: "wordpress_rest_routes",
                matches: wordpress_rest_matches,
            });
        }
    }

    packs
}

fn scan_regex_contracts(
    patterns: Vec<&Regex>,
    category: SemanticModelContractCategory,
    content: &str,
) -> Vec<SemanticModelContractMatch> {
    let mut matches = Vec::new();
    for pattern in patterns {
        for captures in pattern.captures_iter(content) {
            let Some(value_match) = captures.name("value") else {
                continue;
            };
            let value = value_match.as_str().trim();
            if value.is_empty() {
                continue;
            }
            matches.push(SemanticModelContractMatch {
                category,
                value: String::from(value),
                line: line_for_offset(content, value_match.start()),
            });
        }
    }
    matches
}

fn scan_php_hook_map_contracts(path: &Path, content: &str) -> Vec<SemanticModelContractMatch> {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    if !(normalized.ends_with(".hook.php") || normalized.ends_with(".hooks.php")) {
        return Vec::new();
    }

    php_hook_map_key_pattern()
        .captures_iter(content)
        .filter_map(|captures| captures.name("value"))
        .filter_map(|value_match| {
            let value = value_match.as_str().trim();
            (!value.is_empty()).then(|| SemanticModelContractMatch {
                category: SemanticModelContractCategory::Hook,
                value: String::from(value),
                line: line_for_offset(content, value_match.start()),
            })
        })
        .collect()
}

fn scan_wordpress_rest_route_contracts(content: &str) -> Vec<SemanticModelContractMatch> {
    let mut lines = content.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return Vec::new();
    }
    if content.ends_with('\n') {
        lines.push("");
    }

    let mut matches = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("*/")
            || !trimmed.contains("register_rest_route")
        {
            continue;
        }

        let end = (index + 4).min(lines.len());
        let snippet = lines[index..end].join(" ");
        let value = wordpress_rest_route_value_regex()
            .captures(&snippet)
            .map(|captures| {
                format!(
                    "/{}/{}",
                    captures
                        .name("namespace")
                        .map(|value| value.as_str().trim_matches('/'))
                        .unwrap_or("wp-json"),
                    captures
                        .name("route")
                        .map(|value| value.as_str().trim_matches('/'))
                        .unwrap_or("wordpress.rest_route")
                )
            })
            .unwrap_or_else(|| String::from("wordpress.rest_route"));
        matches.push(SemanticModelContractMatch {
            category: SemanticModelContractCategory::Route,
            value,
            line: index + 1,
        });
    }
    matches
}

fn line_for_offset(content: &str, offset: usize) -> usize {
    content[..offset.min(content.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn django_route_patterns() -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            vec![
                Regex::new(r#"\b(?:path|re_path)\s*\(\s*(?:r|fr|rf)?['"](?P<value>[^'"]+)['"]"#)
                    .unwrap(),
            ]
        })
        .iter()
        .collect()
}

fn django_signal_patterns() -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            vec![
                Regex::new(r#"(?m)^\s*(?P<value>[A-Za-z_][A-Za-z0-9_]*)\s*=\s*Signal\s*\("#)
                    .unwrap(),
                Regex::new(r#"@receiver\s*\(\s*(?P<value>[A-Za-z_][A-Za-z0-9_.]*)"#).unwrap(),
                Regex::new(r#"\b(?P<value>[A-Za-z_][A-Za-z0-9_.]*)\.connect\s*\("#).unwrap(),
            ]
        })
        .iter()
        .collect()
}

fn django_settings_patterns() -> Vec<&'static Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            vec![
                Regex::new(r#"\bsettings\.(?P<value>[A-Z][A-Z0-9_]+)\b"#).unwrap(),
                Regex::new(r#"\bgetattr\s*\(\s*settings\s*,\s*['"](?P<value>[A-Z][A-Z0-9_]+)['"]"#)
                    .unwrap(),
            ]
        })
        .iter()
        .collect()
}

fn php_hook_map_key_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r#"['"](?P<value>(?:before|after|on)[A-Z][A-Za-z0-9_]*)['"]\s*=>\s*\["#).unwrap()
    })
}

fn wordpress_rest_route_value_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(
            r#"register_rest_route\s*\(\s*['"](?P<namespace>[^'"]+)['"]\s*,\s*['"](?P<route>[^'"]+)['"]"#,
        )
        .unwrap()
    })
}
