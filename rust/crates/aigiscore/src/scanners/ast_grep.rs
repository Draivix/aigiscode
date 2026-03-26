use crate::scanners::framework_catalogs::framework_misuse_catalogs_for_file;
use ast_grep_language::{LanguageExt, SupportLang};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AstGrepScanResult {
    pub scanner: String,
    pub scanned_files: usize,
    pub matched_files: usize,
    pub rule_ids: Vec<String>,
    pub findings: Vec<AstGrepFinding>,
}

impl AstGrepScanResult {
    pub fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn family_counts(&self) -> AstGrepFamilyCounts {
        let mut counts = AstGrepFamilyCounts::default();
        for finding in &self.findings {
            match finding.kind {
                AstGrepFindingKind::AlgorithmicComplexity { .. } => {
                    counts.algorithmic_complexity += 1;
                }
                AstGrepFindingKind::FrameworkMisuse { .. } => {
                    counts.framework_misuse += 1;
                }
                AstGrepFindingKind::SecurityDangerousApi { .. } => {
                    counts.security_dangerous_api += 1;
                }
            }
        }
        counts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AstGrepFamilyCounts {
    pub algorithmic_complexity: usize,
    pub framework_misuse: usize,
    pub security_dangerous_api: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstGrepFinding {
    pub rule_id: String,
    pub family: String,
    pub language: String,
    pub provenance: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub token: String,
    pub message: String,
    pub matched_text: String,
    pub kind: AstGrepFindingKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AstGrepFindingKind {
    AlgorithmicComplexity {
        subtype: AstGrepComplexitySubtype,
        loop_family: String,
    },
    FrameworkMisuse {
        subtype: AstGrepFrameworkMisuseSubtype,
    },
    SecurityDangerousApi {
        category: AstGrepSecurityCategory,
        api_name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AstGrepComplexitySubtype {
    CollectionScanInLoop,
    SortInLoop,
    RegexCompileInLoop,
    JsonDecodeInLoop,
    FilesystemReadInLoop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AstGrepFrameworkMisuseSubtype {
    RawEnvOutsideConfig,
    RawContainerLookupOutsideBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AstGrepSecurityCategory {
    CommandExecution,
    CodeInjection,
    UnsafeDeserialization,
    UnsafeHtmlOutput,
}

#[derive(Debug, Clone, Copy)]
struct AstGrepRule {
    rule_id: &'static str,
    family: &'static str,
    message: &'static str,
    subtype: AstGrepComplexitySubtype,
    patterns: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
struct AstGrepRuleSet {
    language_label: &'static str,
    language_key: &'static str,
    loop_family: &'static str,
    loop_patterns: &'static [&'static str],
    rules: &'static [AstGrepRule],
}

#[derive(Debug, Clone, Copy)]
struct AstGrepSecurityRule {
    rule_id: &'static str,
    family: &'static str,
    message: &'static str,
    category: AstGrepSecurityCategory,
    api_name: &'static str,
    patterns: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
struct AstGrepSecurityRuleSet {
    language_label: &'static str,
    rules: &'static [AstGrepSecurityRule],
}

#[derive(Debug, Clone, Copy)]
struct AstGrepFrameworkMisuseRule {
    rule_id: &'static str,
    family: &'static str,
    message: &'static str,
    subtype: AstGrepFrameworkMisuseSubtype,
    patterns: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
struct AstGrepFrameworkMisuseRuleSet {
    language_label: &'static str,
    rules: &'static [AstGrepFrameworkMisuseRule],
}

const PHP_LOOP_PATTERNS: &[&str] = &[
    "for ($INIT; $COND; $UPDATE) { $$$BODY }",
    "foreach ($ITER as $ITEM) { $$$BODY }",
    "foreach ($ITER as $KEY => $VALUE) { $$$BODY }",
    "while ($COND) { $$$BODY }",
];

const PHP_RULES: &[AstGrepRule] = &[
    AstGrepRule {
        rule_id: "complexity/php/collection_scan_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated collection scans inside a loop should be indexed or hoisted.",
        subtype: AstGrepComplexitySubtype::CollectionScanInLoop,
        patterns: &["in_array($$$ARGS)", "array_search($$$ARGS)"],
    },
    AstGrepRule {
        rule_id: "complexity/php/sort_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated sorting inside a loop should be moved out or precomputed.",
        subtype: AstGrepComplexitySubtype::SortInLoop,
        patterns: &[
            "sort($$$ARGS)",
            "usort($$$ARGS)",
            "ksort($$$ARGS)",
            "asort($$$ARGS)",
        ],
    },
    AstGrepRule {
        rule_id: "complexity/php/json_decode_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated JSON decode inside a loop should be hoisted, batched, or cached.",
        subtype: AstGrepComplexitySubtype::JsonDecodeInLoop,
        patterns: &["json_decode($$$ARGS)"],
    },
    AstGrepRule {
        rule_id: "complexity/php/filesystem_read_in_loop",
        family: "algorithmic_complexity",
        message:
            "Repeated filesystem reads/checks inside a loop should be hoisted, cached, or batched.",
        subtype: AstGrepComplexitySubtype::FilesystemReadInLoop,
        patterns: &["file_get_contents($$$ARGS)", "file_exists($$$ARGS)"],
    },
];

const PHP_SECURITY_RULES: &[AstGrepSecurityRule] = &[
    AstGrepSecurityRule {
        rule_id: "security/php/command_exec",
        family: "security_dangerous_api",
        message: "Dangerous command execution primitive should be isolated or removed.",
        category: AstGrepSecurityCategory::CommandExecution,
        api_name: "php-command-exec",
        patterns: &[
            "exec($$$ARGS)",
            "system($$$ARGS)",
            "passthru($$$ARGS)",
            "shell_exec($$$ARGS)",
            "proc_open($$$ARGS)",
            "popen($$$ARGS)",
        ],
    },
    AstGrepSecurityRule {
        rule_id: "security/php/eval",
        family: "security_dangerous_api",
        message: "Dangerous code-evaluation primitive should be removed from the path.",
        category: AstGrepSecurityCategory::CodeInjection,
        api_name: "php-eval",
        patterns: &["eval($$$ARGS)", "assert($$$ARGS)"],
    },
    AstGrepSecurityRule {
        rule_id: "security/php/unserialize",
        family: "security_dangerous_api",
        message: "Unsafe deserialization primitive should be isolated or replaced.",
        category: AstGrepSecurityCategory::UnsafeDeserialization,
        api_name: "php-unserialize",
        patterns: &["unserialize($$$ARGS)"],
    },
];

const PHP_FRAMEWORK_MISUSE_RULES: &[AstGrepFrameworkMisuseRule] = &[AstGrepFrameworkMisuseRule {
    rule_id: "framework_misuse/php/raw_env_outside_config",
    family: "framework_misuse",
    message: "Raw environment access should stay inside a config/bootstrap boundary.",
    subtype: AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
    patterns: &["env($$$ARGS)", "getenv($$$ARGS)", "$_ENV[$$$ARGS]"],
}];

const PYTHON_LOOP_PATTERNS: &[&str] =
    &["for $ITEM in $ITER:\n  $$$BODY", "while $COND:\n  $$$BODY"];

const PYTHON_RULES: &[AstGrepRule] = &[
    AstGrepRule {
        rule_id: "complexity/python/sort_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated sorting inside a loop should be moved out or precomputed.",
        subtype: AstGrepComplexitySubtype::SortInLoop,
        patterns: &["sorted($$$ARGS)", "$TARGET.sort()"],
    },
    AstGrepRule {
        rule_id: "complexity/python/regex_compile_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated regex compilation inside a loop should be hoisted or cached.",
        subtype: AstGrepComplexitySubtype::RegexCompileInLoop,
        patterns: &["re.compile($$$ARGS)"],
    },
    AstGrepRule {
        rule_id: "complexity/python/json_decode_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated JSON decode inside a loop should be hoisted, batched, or cached.",
        subtype: AstGrepComplexitySubtype::JsonDecodeInLoop,
        patterns: &["json.loads($$$ARGS)", "json.load($$$ARGS)"],
    },
    AstGrepRule {
        rule_id: "complexity/python/filesystem_read_in_loop",
        family: "algorithmic_complexity",
        message:
            "Repeated filesystem reads/checks inside a loop should be hoisted, cached, or batched.",
        subtype: AstGrepComplexitySubtype::FilesystemReadInLoop,
        patterns: &[
            "os.path.exists($$$ARGS)",
            "Path($$$ARGS).exists()",
            "Path($$$ARGS).read_text($$$ARGS)",
            "Path($$$ARGS).read_bytes()",
        ],
    },
];

const PYTHON_SECURITY_RULES: &[AstGrepSecurityRule] = &[
    AstGrepSecurityRule {
        rule_id: "security/python/command_exec",
        family: "security_dangerous_api",
        message: "Dangerous command execution primitive should be isolated or removed.",
        category: AstGrepSecurityCategory::CommandExecution,
        api_name: "python-command-exec",
        patterns: &[
            "os.system($$$ARGS)",
            "subprocess.run($$$ARGS)",
            "subprocess.call($$$ARGS)",
            "subprocess.check_call($$$ARGS)",
            "subprocess.check_output($$$ARGS)",
            "subprocess.Popen($$$ARGS)",
        ],
    },
    AstGrepSecurityRule {
        rule_id: "security/python/eval",
        family: "security_dangerous_api",
        message: "Dangerous code-evaluation primitive should be removed from the path.",
        category: AstGrepSecurityCategory::CodeInjection,
        api_name: "python-eval",
        patterns: &["eval($$$ARGS)", "exec($$$ARGS)"],
    },
    AstGrepSecurityRule {
        rule_id: "security/python/deserialize",
        family: "security_dangerous_api",
        message: "Unsafe deserialization primitive should be isolated or replaced.",
        category: AstGrepSecurityCategory::UnsafeDeserialization,
        api_name: "python-deserialize",
        patterns: &[
            "pickle.load($$$ARGS)",
            "pickle.loads($$$ARGS)",
            "yaml.load($$$ARGS)",
        ],
    },
];

const PYTHON_FRAMEWORK_MISUSE_RULES: &[AstGrepFrameworkMisuseRule] =
    &[AstGrepFrameworkMisuseRule {
        rule_id: "framework_misuse/python/raw_env_outside_config",
        family: "framework_misuse",
        message: "Raw environment access should stay inside a config/bootstrap boundary.",
        subtype: AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
        patterns: &[
            "os.environ[$$$ARGS]",
            "os.environ.get($$$ARGS)",
            "os.getenv($$$ARGS)",
        ],
    }];

const JS_LOOP_PATTERNS: &[&str] = &[
    "for ($INIT; $COND; $UPDATE) { $$$BODY }",
    "for (const $ITEM of $ITER) { $$$BODY }",
    "for (let $ITEM of $ITER) { $$$BODY }",
    "for (const $KEY in $ITER) { $$$BODY }",
    "for (let $KEY in $ITER) { $$$BODY }",
    "while ($COND) { $$$BODY }",
];

const JS_RULES: &[AstGrepRule] = &[
    AstGrepRule {
        rule_id: "complexity/javascript/collection_scan_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated collection scans inside a loop should be indexed or hoisted.",
        subtype: AstGrepComplexitySubtype::CollectionScanInLoop,
        patterns: &[
            "$ARRAY.includes($$$ARGS)",
            "$ARRAY.find($$$ARGS)",
            "$ARRAY.some($$$ARGS)",
        ],
    },
    AstGrepRule {
        rule_id: "complexity/javascript/sort_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated sorting inside a loop should be moved out or precomputed.",
        subtype: AstGrepComplexitySubtype::SortInLoop,
        patterns: &["$ARRAY.sort($$$ARGS)"],
    },
    AstGrepRule {
        rule_id: "complexity/javascript/regex_compile_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated regex compilation inside a loop should be hoisted or cached.",
        subtype: AstGrepComplexitySubtype::RegexCompileInLoop,
        patterns: &["new RegExp($$$ARGS)"],
    },
    AstGrepRule {
        rule_id: "complexity/javascript/json_decode_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated JSON parse inside a loop should be hoisted, batched, or cached.",
        subtype: AstGrepComplexitySubtype::JsonDecodeInLoop,
        patterns: &["JSON.parse($$$ARGS)"],
    },
    AstGrepRule {
        rule_id: "complexity/javascript/filesystem_read_in_loop",
        family: "algorithmic_complexity",
        message:
            "Repeated filesystem reads/checks inside a loop should be hoisted, cached, or batched.",
        subtype: AstGrepComplexitySubtype::FilesystemReadInLoop,
        patterns: &[
            "fs.readFileSync($$$ARGS)",
            "fs.readFile($$$ARGS)",
            "fs.existsSync($$$ARGS)",
        ],
    },
];

const JS_SECURITY_RULES: &[AstGrepSecurityRule] = &[
    AstGrepSecurityRule {
        rule_id: "security/javascript/command_exec",
        family: "security_dangerous_api",
        message: "Dangerous command execution primitive should be isolated or removed.",
        category: AstGrepSecurityCategory::CommandExecution,
        api_name: "javascript-command-exec",
        patterns: &[
            "child_process.exec($$$ARGS)",
            "child_process.execSync($$$ARGS)",
        ],
    },
    AstGrepSecurityRule {
        rule_id: "security/javascript/eval",
        family: "security_dangerous_api",
        message: "Dangerous code-evaluation primitive should be removed from the path.",
        category: AstGrepSecurityCategory::CodeInjection,
        api_name: "javascript-eval",
        patterns: &["eval($$$ARGS)", "new Function($$$ARGS)"],
    },
    AstGrepSecurityRule {
        rule_id: "security/javascript/html_output",
        family: "security_dangerous_api",
        message: "Unsafe HTML output primitive should be isolated or sanitized.",
        category: AstGrepSecurityCategory::UnsafeHtmlOutput,
        api_name: "javascript-html-output",
        patterns: &[
            "document.write($$$ARGS)",
            "$TARGET.innerHTML = $VALUE",
            "$TARGET.outerHTML = $VALUE",
        ],
    },
];

const JS_FRAMEWORK_MISUSE_RULES: &[AstGrepFrameworkMisuseRule] = &[AstGrepFrameworkMisuseRule {
    rule_id: "framework_misuse/javascript/raw_env_outside_config",
    family: "framework_misuse",
    message: "Raw environment access should stay inside a config/bootstrap boundary.",
    subtype: AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
    patterns: &["process.env.$NAME", "process.env[$$$ARGS]"],
}];

const RUST_LOOP_PATTERNS: &[&str] = &["for $ITEM in $ITER { $$$BODY }", "while $COND { $$$BODY }"];

const RUST_RULES: &[AstGrepRule] = &[
    AstGrepRule {
        rule_id: "complexity/rust/sort_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated sorting inside a loop should be moved out or precomputed.",
        subtype: AstGrepComplexitySubtype::SortInLoop,
        patterns: &[
            "$TARGET.sort()",
            "$TARGET.sort_by($$$ARGS)",
            "$TARGET.sort_unstable()",
            "$TARGET.sort_unstable_by($$$ARGS)",
        ],
    },
    AstGrepRule {
        rule_id: "complexity/rust/regex_compile_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated regex compilation inside a loop should be hoisted or cached.",
        subtype: AstGrepComplexitySubtype::RegexCompileInLoop,
        patterns: &["Regex::new($$$ARGS)"],
    },
    AstGrepRule {
        rule_id: "complexity/rust/json_decode_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated JSON parse inside a loop should be hoisted, batched, or cached.",
        subtype: AstGrepComplexitySubtype::JsonDecodeInLoop,
        patterns: &[
            "serde_json::from_str($$$ARGS)",
            "serde_json::from_slice($$$ARGS)",
            "serde_json::from_reader($$$ARGS)",
        ],
    },
    AstGrepRule {
        rule_id: "complexity/rust/filesystem_read_in_loop",
        family: "algorithmic_complexity",
        message:
            "Repeated filesystem reads/checks inside a loop should be hoisted, cached, or batched.",
        subtype: AstGrepComplexitySubtype::FilesystemReadInLoop,
        patterns: &[
            "std::fs::read($$$ARGS)",
            "std::fs::read_to_string($$$ARGS)",
            "std::fs::metadata($$$ARGS)",
            "fs::read($$$ARGS)",
            "fs::read_to_string($$$ARGS)",
            "fs::metadata($$$ARGS)",
        ],
    },
];

const RUBY_SECURITY_RULES: &[AstGrepSecurityRule] = &[
    AstGrepSecurityRule {
        rule_id: "security/ruby/command_exec",
        family: "security_dangerous_api",
        message: "Dangerous command execution primitive should be isolated or removed.",
        category: AstGrepSecurityCategory::CommandExecution,
        api_name: "ruby-command-exec",
        patterns: &["system($$$ARGS)", "exec($$$ARGS)", "IO.popen($$$ARGS)"],
    },
    AstGrepSecurityRule {
        rule_id: "security/ruby/eval",
        family: "security_dangerous_api",
        message: "Dangerous code-evaluation primitive should be removed from the path.",
        category: AstGrepSecurityCategory::CodeInjection,
        api_name: "ruby-eval",
        patterns: &[
            "eval($$$ARGS)",
            "instance_eval($$$ARGS)",
            "class_eval($$$ARGS)",
        ],
    },
    AstGrepSecurityRule {
        rule_id: "security/ruby/deserialize",
        family: "security_dangerous_api",
        message: "Unsafe deserialization primitive should be isolated or replaced.",
        category: AstGrepSecurityCategory::UnsafeDeserialization,
        api_name: "ruby-deserialize",
        patterns: &["Marshal.load($$$ARGS)", "YAML.load($$$ARGS)"],
    },
];

const RUBY_FRAMEWORK_MISUSE_RULES: &[AstGrepFrameworkMisuseRule] = &[AstGrepFrameworkMisuseRule {
    rule_id: "framework_misuse/ruby/raw_env_outside_config",
    family: "framework_misuse",
    message: "Raw environment access should stay inside a config/bootstrap boundary.",
    subtype: AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
    patterns: &["ENV[$$$ARGS]", "ENV.fetch($$$ARGS)"],
}];

const RUST_SECURITY_RULES: &[AstGrepSecurityRule] = &[];

const RUST_FRAMEWORK_MISUSE_RULES: &[AstGrepFrameworkMisuseRule] = &[AstGrepFrameworkMisuseRule {
    rule_id: "framework_misuse/rust/raw_env_outside_config",
    family: "framework_misuse",
    message: "Raw environment access should stay inside a config/bootstrap boundary.",
    subtype: AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
    patterns: &[
        "std::env::var($$$ARGS)",
        "std::env::var_os($$$ARGS)",
        "env::var($$$ARGS)",
        "env::var_os($$$ARGS)",
        "env!($$$ARGS)",
        "option_env!($$$ARGS)",
    ],
}];

const PHP_RULE_SET: AstGrepRuleSet = AstGrepRuleSet {
    language_label: "php",
    language_key: "php",
    loop_family: "brace_loop",
    loop_patterns: PHP_LOOP_PATTERNS,
    rules: PHP_RULES,
};

const PHP_SECURITY_RULE_SET: AstGrepSecurityRuleSet = AstGrepSecurityRuleSet {
    language_label: "php",
    rules: PHP_SECURITY_RULES,
};

const PHP_FRAMEWORK_MISUSE_RULE_SET: AstGrepFrameworkMisuseRuleSet =
    AstGrepFrameworkMisuseRuleSet {
        language_label: "php",
        rules: PHP_FRAMEWORK_MISUSE_RULES,
    };

const PYTHON_RULE_SET: AstGrepRuleSet = AstGrepRuleSet {
    language_label: "python",
    language_key: "py",
    loop_family: "indent_loop",
    loop_patterns: PYTHON_LOOP_PATTERNS,
    rules: PYTHON_RULES,
};

const PYTHON_SECURITY_RULE_SET: AstGrepSecurityRuleSet = AstGrepSecurityRuleSet {
    language_label: "python",
    rules: PYTHON_SECURITY_RULES,
};

const PYTHON_FRAMEWORK_MISUSE_RULE_SET: AstGrepFrameworkMisuseRuleSet =
    AstGrepFrameworkMisuseRuleSet {
        language_label: "python",
        rules: PYTHON_FRAMEWORK_MISUSE_RULES,
    };

const JAVASCRIPT_RULE_SET: AstGrepRuleSet = AstGrepRuleSet {
    language_label: "javascript",
    language_key: "js",
    loop_family: "brace_loop",
    loop_patterns: JS_LOOP_PATTERNS,
    rules: JS_RULES,
};

const JAVASCRIPT_SECURITY_RULE_SET: AstGrepSecurityRuleSet = AstGrepSecurityRuleSet {
    language_label: "javascript",
    rules: JS_SECURITY_RULES,
};

const JAVASCRIPT_FRAMEWORK_MISUSE_RULE_SET: AstGrepFrameworkMisuseRuleSet =
    AstGrepFrameworkMisuseRuleSet {
        language_label: "javascript",
        rules: JS_FRAMEWORK_MISUSE_RULES,
    };

const TYPESCRIPT_RULE_SET: AstGrepRuleSet = AstGrepRuleSet {
    language_label: "typescript",
    language_key: "ts",
    loop_family: "brace_loop",
    loop_patterns: JS_LOOP_PATTERNS,
    rules: JS_RULES,
};

const TYPESCRIPT_SECURITY_RULE_SET: AstGrepSecurityRuleSet = AstGrepSecurityRuleSet {
    language_label: "typescript",
    rules: JS_SECURITY_RULES,
};

const TYPESCRIPT_FRAMEWORK_MISUSE_RULE_SET: AstGrepFrameworkMisuseRuleSet =
    AstGrepFrameworkMisuseRuleSet {
        language_label: "typescript",
        rules: JS_FRAMEWORK_MISUSE_RULES,
    };

const RUBY_SECURITY_RULE_SET: AstGrepSecurityRuleSet = AstGrepSecurityRuleSet {
    language_label: "ruby",
    rules: RUBY_SECURITY_RULES,
};

const RUBY_FRAMEWORK_MISUSE_RULE_SET: AstGrepFrameworkMisuseRuleSet =
    AstGrepFrameworkMisuseRuleSet {
        language_label: "ruby",
        rules: RUBY_FRAMEWORK_MISUSE_RULES,
    };

const RUST_RULE_SET: AstGrepRuleSet = AstGrepRuleSet {
    language_label: "rust",
    language_key: "rs",
    loop_family: "brace_loop",
    loop_patterns: RUST_LOOP_PATTERNS,
    rules: RUST_RULES,
};

const RUST_SECURITY_RULE_SET: AstGrepSecurityRuleSet = AstGrepSecurityRuleSet {
    language_label: "rust",
    rules: RUST_SECURITY_RULES,
};

const RUST_FRAMEWORK_MISUSE_RULE_SET: AstGrepFrameworkMisuseRuleSet =
    AstGrepFrameworkMisuseRuleSet {
        language_label: "rust",
        rules: RUST_FRAMEWORK_MISUSE_RULES,
    };

pub fn run_ast_grep_scan(parsed_sources: &[(PathBuf, String)]) -> AstGrepScanResult {
    let mut findings = Vec::new();
    let mut rule_ids = BTreeSet::new();
    let mut matched_files = BTreeSet::new();
    let mut seen = BTreeSet::<String>::new();
    let mut scanned_files = 0usize;

    for (path, source) in parsed_sources {
        let Some(rule_set) = complexity_rule_set_for_path(path) else {
            continue;
        };
        let Ok(language) = rule_set.language_key.parse::<SupportLang>() else {
            continue;
        };
        scanned_files += 1;
        let ast = language.ast_grep(source);
        let root = ast.root();
        for loop_pattern in rule_set.loop_patterns {
            for loop_node in root.find_all(loop_pattern) {
                for rule in rule_set.rules {
                    for pattern in rule.patterns {
                        for matched in loop_node.find_all(pattern) {
                            let line = matched.start_pos().line() + 1;
                            let token = compact_snippet(&matched.text());
                            let key = format!(
                                "complexity|{}|{}|{}|{}",
                                path.display(),
                                rule.rule_id,
                                line,
                                token
                            );
                            if !seen.insert(key) {
                                continue;
                            }
                            matched_files.insert(path.clone());
                            rule_ids.insert(String::from(rule.rule_id));
                            findings.push(AstGrepFinding {
                                rule_id: String::from(rule.rule_id),
                                family: String::from(rule.family),
                                language: String::from(rule_set.language_label),
                                provenance: String::from("ast_grep.pattern"),
                                file_path: path.clone(),
                                line,
                                token: token.clone(),
                                message: String::from(rule.message),
                                matched_text: token,
                                kind: AstGrepFindingKind::AlgorithmicComplexity {
                                    subtype: rule.subtype,
                                    loop_family: String::from(rule_set.loop_family),
                                },
                            });
                        }
                    }
                }
            }
        }

        if let Some(security_rule_set) = security_rule_set_for_path(path) {
            for rule in security_rule_set.rules {
                for pattern in rule.patterns {
                    for matched in root.find_all(pattern) {
                        let line = matched.start_pos().line() + 1;
                        let token = compact_snippet(&matched.text());
                        let key = format!(
                            "security|{}|{}|{}|{}",
                            path.display(),
                            rule.rule_id,
                            line,
                            token
                        );
                        if !seen.insert(key) {
                            continue;
                        }
                        matched_files.insert(path.clone());
                        rule_ids.insert(String::from(rule.rule_id));
                        findings.push(AstGrepFinding {
                            rule_id: String::from(rule.rule_id),
                            family: String::from(rule.family),
                            language: String::from(security_rule_set.language_label),
                            provenance: String::from("ast_grep.pattern"),
                            file_path: path.clone(),
                            line,
                            token: token.clone(),
                            message: String::from(rule.message),
                            matched_text: token,
                            kind: AstGrepFindingKind::SecurityDangerousApi {
                                category: rule.category,
                                api_name: String::from(rule.api_name),
                            },
                        });
                    }
                }
            }
        }

        let framework_catalogs = framework_misuse_catalogs_for_file(path, source);
        if framework_catalogs.is_empty() {
            if let Some(framework_rule_set) = framework_misuse_rule_set_for_path(path) {
                for rule in framework_rule_set.rules {
                    for pattern in rule.patterns {
                        for matched in root.find_all(pattern) {
                            let line = matched.start_pos().line() + 1;
                            let token = compact_snippet(&matched.text());
                            let key = format!(
                                "framework|{}|{}|{}|{}",
                                path.display(),
                                rule.rule_id,
                                line,
                                token
                            );
                            if !seen.insert(key) {
                                continue;
                            }
                            matched_files.insert(path.clone());
                            rule_ids.insert(String::from(rule.rule_id));
                            findings.push(AstGrepFinding {
                                rule_id: String::from(rule.rule_id),
                                family: String::from(rule.family),
                                language: String::from(framework_rule_set.language_label),
                                provenance: String::from("ast_grep.pattern"),
                                file_path: path.clone(),
                                line,
                                token: token.clone(),
                                message: String::from(rule.message),
                                matched_text: token,
                                kind: AstGrepFindingKind::FrameworkMisuse {
                                    subtype: rule.subtype,
                                },
                            });
                        }
                    }
                }
            }
        }

        for catalog in framework_catalogs {
            for rule in catalog.rules {
                for pattern in rule.patterns {
                    for matched in root.find_all(pattern) {
                        let line = matched.start_pos().line() + 1;
                        let token = compact_snippet(&matched.text());
                        let key = format!(
                            "framework|{}|{}|{}|{}",
                            path.display(),
                            rule.rule_id,
                            line,
                            token
                        );
                        if !seen.insert(key) {
                            continue;
                        }
                        matched_files.insert(path.clone());
                        rule_ids.insert(String::from(rule.rule_id));
                        findings.push(AstGrepFinding {
                            rule_id: String::from(rule.rule_id),
                            family: String::from(rule.family),
                            language: String::from(catalog.language_label),
                            provenance: format!("ast_grep.pattern.{}", catalog.framework_id),
                            file_path: path.clone(),
                            line,
                            token: token.clone(),
                            message: String::from(rule.message),
                            matched_text: token,
                            kind: AstGrepFindingKind::FrameworkMisuse {
                                subtype: rule.subtype,
                            },
                        });
                    }
                }
            }
        }
    }

    findings.sort_by(|left, right| {
        left.file_path
            .cmp(&right.file_path)
            .then(left.line.cmp(&right.line))
            .then(left.rule_id.cmp(&right.rule_id))
            .then(left.token.cmp(&right.token))
    });

    AstGrepScanResult {
        scanner: String::from("ast_grep"),
        scanned_files,
        matched_files: matched_files.len(),
        rule_ids: rule_ids.into_iter().collect(),
        findings,
    }
}

fn complexity_rule_set_for_path(path: &Path) -> Option<&'static AstGrepRuleSet> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("php" | "phtml" | "php3" | "php4" | "php5" | "php8") => Some(&PHP_RULE_SET),
        Some("py") => Some(&PYTHON_RULE_SET),
        Some("js" | "jsx") => Some(&JAVASCRIPT_RULE_SET),
        Some("ts" | "tsx") => Some(&TYPESCRIPT_RULE_SET),
        Some("rs") => Some(&RUST_RULE_SET),
        _ => None,
    }
}

fn security_rule_set_for_path(path: &Path) -> Option<&'static AstGrepSecurityRuleSet> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("php" | "phtml" | "php3" | "php4" | "php5" | "php8") => Some(&PHP_SECURITY_RULE_SET),
        Some("py") => Some(&PYTHON_SECURITY_RULE_SET),
        Some("js" | "jsx") => Some(&JAVASCRIPT_SECURITY_RULE_SET),
        Some("ts" | "tsx") => Some(&TYPESCRIPT_SECURITY_RULE_SET),
        Some("rb" | "rake") => Some(&RUBY_SECURITY_RULE_SET),
        Some("rs") => Some(&RUST_SECURITY_RULE_SET),
        _ => None,
    }
}

fn framework_misuse_rule_set_for_path(
    path: &Path,
) -> Option<&'static AstGrepFrameworkMisuseRuleSet> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("php" | "phtml" | "php3" | "php4" | "php5" | "php8") => {
            Some(&PHP_FRAMEWORK_MISUSE_RULE_SET)
        }
        Some("py") => Some(&PYTHON_FRAMEWORK_MISUSE_RULE_SET),
        Some("js" | "jsx") => Some(&JAVASCRIPT_FRAMEWORK_MISUSE_RULE_SET),
        Some("ts" | "tsx") => Some(&TYPESCRIPT_FRAMEWORK_MISUSE_RULE_SET),
        Some("rb" | "rake") => Some(&RUBY_FRAMEWORK_MISUSE_RULE_SET),
        Some("rs") => Some(&RUST_FRAMEWORK_MISUSE_RULE_SET),
        _ => None,
    }
}

fn compact_snippet(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.len() <= 160 {
        normalized
    } else {
        format!("{}...", &normalized[..157])
    }
}

#[cfg(test)]
mod tests {
    use super::{
        run_ast_grep_scan, AstGrepComplexitySubtype, AstGrepFindingKind,
        AstGrepFrameworkMisuseSubtype, AstGrepSecurityCategory,
    };
    use std::path::PathBuf;

    #[test]
    fn detects_python_json_decode_inside_loop() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("src/importer.py"),
            String::from(
                r#"
import json

def load_rows(lines):
    rows = []
    for line in lines:
        rows.append(json.loads(line))
    return rows
"#,
            ),
        )]);

        assert_eq!(result.scanner, "ast_grep");
        assert_eq!(result.findings.len(), 1);
        assert!(result
            .rule_ids
            .iter()
            .any(|rule_id| rule_id == "complexity/python/json_decode_in_loop"));
        match &result.findings[0].kind {
            AstGrepFindingKind::AlgorithmicComplexity { subtype, .. } => {
                assert_eq!(*subtype, AstGrepComplexitySubtype::JsonDecodeInLoop);
            }
            AstGrepFindingKind::FrameworkMisuse { .. } => panic!("expected complexity finding"),
            AstGrepFindingKind::SecurityDangerousApi { .. } => {
                panic!("expected complexity finding")
            }
        }
    }

    #[test]
    fn detects_javascript_filesystem_read_inside_loop() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("src/loader.ts"),
            String::from(
                r#"
import fs from "fs";

export function loadAll(paths: string[]): string[] {
    const out: string[] = [];
    for (const path of paths) {
        out.push(fs.readFileSync(path, "utf8"));
    }
    return out;
}
"#,
            ),
        )]);

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].line, 7);
        assert!(result.findings[0].token.contains("fs.readFileSync"));
    }

    #[test]
    fn detects_rust_regex_compile_inside_loop() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("src/filter.rs"),
            String::from(
                r#"
use regex::Regex;

fn compile_all(items: &[String]) {
    for item in items {
        let _ = Regex::new(item);
    }
}
"#,
            ),
        )]);

        assert_eq!(result.findings.len(), 1);
        assert!(result
            .rule_ids
            .iter()
            .any(|rule_id| rule_id == "complexity/rust/regex_compile_in_loop"));
    }

    #[test]
    fn detects_javascript_sort_inside_loop() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("src/order.ts"),
            String::from(
                r#"
export function reorder(groups: string[][]) {
    for (const group of groups) {
        group.sort();
    }
}
"#,
            ),
        )]);

        assert_eq!(result.findings.len(), 1);
        assert!(result
            .rule_ids
            .iter()
            .any(|rule_id| rule_id == "complexity/javascript/sort_in_loop"));
        assert!(result.findings[0].token.contains(".sort("));
    }

    #[test]
    fn detects_javascript_eval_as_security_dangerous_api() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("src/admin.js"),
            String::from(
                r#"
export function run(payload) {
    return eval(payload);
}
"#,
            ),
        )]);

        assert_eq!(result.findings.len(), 1);
        assert!(result
            .rule_ids
            .iter()
            .any(|rule_id| rule_id == "security/javascript/eval"));
        match &result.findings[0].kind {
            AstGrepFindingKind::SecurityDangerousApi { category, api_name } => {
                assert_eq!(*category, AstGrepSecurityCategory::CodeInjection);
                assert_eq!(api_name, "javascript-eval");
            }
            AstGrepFindingKind::FrameworkMisuse { .. } => panic!("expected security finding"),
            AstGrepFindingKind::AlgorithmicComplexity { .. } => {
                panic!("expected security finding")
            }
        }
    }

    #[test]
    fn detects_python_raw_env_access_as_framework_misuse() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("src/service.py"),
            String::from(
                r#"
import os

def build():
    return os.environ.get("APP_MODE")
"#,
            ),
        )]);

        assert_eq!(result.findings.len(), 1);
        assert!(result
            .rule_ids
            .iter()
            .any(|rule_id| rule_id == "framework_misuse/python/raw_env_outside_config"));
        match &result.findings[0].kind {
            AstGrepFindingKind::FrameworkMisuse { subtype } => {
                assert_eq!(*subtype, AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig);
            }
            AstGrepFindingKind::AlgorithmicComplexity { .. } => {
                panic!("expected framework misuse finding")
            }
            AstGrepFindingKind::SecurityDangerousApi { .. } => {
                panic!("expected framework misuse finding")
            }
        }
    }

    #[test]
    fn detects_php_raw_container_lookup_as_framework_misuse() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("app/Services/ReportService.php"),
            String::from(
                r#"
<?php

final class ReportService
{
    public function build(): array
    {
        return app(TenantManager::class)->current();
    }
}
"#,
            ),
        )]);

        assert!(result.rule_ids.iter().any(|rule_id| {
            rule_id == "framework_misuse/php/raw_container_lookup_outside_boundary"
        }));
        assert!(result.findings.iter().any(|finding| matches!(
            finding.kind,
            AstGrepFindingKind::FrameworkMisuse {
                subtype: AstGrepFrameworkMisuseSubtype::RawContainerLookupOutsideBoundary,
            }
        )));
    }

    #[test]
    fn django_raw_env_uses_framework_catalog_provenance() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("app/services/report.py"),
            String::from(
                r#"
import os
from django.conf import settings

def build():
    return os.environ.get("APP_MODE"), settings.TIMEOUT
"#,
            ),
        )]);

        let finding = result
            .findings
            .iter()
            .find(|finding| {
                matches!(
                    finding.kind,
                    AstGrepFindingKind::FrameworkMisuse {
                        subtype: AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
                    }
                )
            })
            .expect("expected framework misuse finding");
        assert_eq!(finding.provenance, "ast_grep.pattern.django");
    }

    #[test]
    fn counts_findings_by_family() {
        let result = run_ast_grep_scan(&[
            (
                PathBuf::from("src/importer.py"),
                String::from(
                    r#"
import json
for line in lines:
    json.loads(line)
"#,
                ),
            ),
            (
                PathBuf::from("src/admin.js"),
                String::from("export function run(input) { return eval(input); }\n"),
            ),
            (
                PathBuf::from("src/service.py"),
                String::from(
                    r#"
import os
from django.conf import settings
def build():
    return os.environ.get("APP_MODE"), settings.TIMEOUT
"#,
                ),
            ),
        ]);

        let counts = result.family_counts();
        assert_eq!(counts.algorithmic_complexity, 1);
        assert_eq!(counts.security_dangerous_api, 1);
        assert_eq!(counts.framework_misuse, 1);
    }
}
