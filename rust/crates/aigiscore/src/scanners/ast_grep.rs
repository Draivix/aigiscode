use crate::scanners::framework_catalogs::{
    framework_complexity_catalogs_for_file, framework_misuse_catalogs_for_file,
};
use ast_grep_core::Pattern;
use ast_grep_language::{LanguageExt, SupportLang};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AstGrepScanResult {
    pub scanner: String,
    pub scanned_files: usize,
    pub matched_files: usize,
    pub rule_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_files: Vec<AstGrepSkippedFile>,
    pub findings: Vec<AstGrepFinding>,
}

impl AstGrepScanResult {
    pub fn is_empty(&self) -> bool {
        self.findings.is_empty() && self.skipped_files.is_empty()
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AstGrepSkippedFile {
    pub file_path: PathBuf,
    pub bytes: usize,
    pub reason: String,
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
    DatabaseQueryInLoop,
    HttpCallInLoop,
    CacheLookupInLoop,
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

#[derive(Debug, Clone, Copy, Default)]
struct AstGrepFamilyPrefilter {
    complexity: bool,
    security: bool,
    framework_misuse: bool,
}

impl AstGrepFamilyPrefilter {
    fn any(self) -> bool {
        self.complexity || self.security || self.framework_misuse
    }
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

const AST_GREP_MAX_FILE_BYTES: usize = 150_000;

// `find_all(&str)` recompiles the pattern for every node visited, which makes
// scan cost proportional to pattern-count x node-count. Compile each static
// rule pattern once per language and reuse it across all files.
fn compiled_pattern(language: SupportLang, source: &'static str) -> &'static Pattern {
    static CACHE: OnceLock<Mutex<HashMap<(SupportLang, &'static str), &'static Pattern>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .entry((language, source))
        .or_insert_with(|| Box::leak(Box::new(Pattern::new(source, language))))
}

// ast-grep reads `$` + uppercase/underscore as a metavariable, so a PHP
// superglobal pattern like `$_ENV[...]` compiles to "any subscript expression"
// and over-matches every array access. Anchor those patterns back to their
// literal superglobal prefix at match time.
fn pattern_literal_anchor(pattern: &str) -> Option<&str> {
    if !pattern.starts_with("$_") {
        return None;
    }
    pattern.find('[').map(|end| &pattern[..end])
}

fn trace_slow_pattern(path: &Path, rule_id: &str, pattern: &str, matches: usize, started: Instant) {
    let elapsed = started.elapsed().as_millis();
    if elapsed >= 100 {
        trace(&format!(
            "ast_grep.pattern_slow path={} rule={} pattern={:?} matches={matches} elapsed_ms={elapsed}",
            path.display(),
            rule_id,
            pattern,
        ));
    }
}

pub fn run_ast_grep_scan(parsed_sources: &[(PathBuf, String)]) -> AstGrepScanResult {
    let scan_started = Instant::now();
    trace(&format!(
        "ast_grep.scan start files={}",
        parsed_sources.len()
    ));
    let mut findings = Vec::new();
    let mut rule_ids = BTreeSet::new();
    let mut matched_files = BTreeSet::new();
    let mut seen = BTreeSet::<String>::new();
    let mut scanned_files = 0usize;
    let mut skipped_files = Vec::new();

    for (path, source) in parsed_sources {
        let file_started = Instant::now();
        let findings_before = findings.len();
        trace(&format!(
            "ast_grep.file start path={} bytes={}",
            path.display(),
            source.len()
        ));
        let Some(language) = support_lang_for_path(path) else {
            continue;
        };
        if source.len() > AST_GREP_MAX_FILE_BYTES {
            trace(&format!(
                "ast_grep.file skip path={} reason=file_too_large bytes={}",
                path.display(),
                source.len()
            ));
            skipped_files.push(AstGrepSkippedFile {
                file_path: path.clone(),
                bytes: source.len(),
                reason: String::from("file_too_large_for_secondary_scan"),
            });
            continue;
        }
        let prefilter = lexical_family_prefilter(path, source);
        if !prefilter.any() {
            trace(&format!(
                "ast_grep.file skip path={} reason=no_family_prefilter_hit bytes={}",
                path.display(),
                source.len()
            ));
            skipped_files.push(AstGrepSkippedFile {
                file_path: path.clone(),
                bytes: source.len(),
                reason: String::from("no_family_prefilter_hit"),
            });
            continue;
        }
        scanned_files += 1;
        trace(&format!(
            "ast_grep.file parse start path={}",
            path.display()
        ));
        let ast = language.ast_grep(source);
        trace(&format!("ast_grep.file parse done path={}", path.display()));
        let root = ast.root();

        if prefilter.complexity {
            if let Some(rule_set) = complexity_rule_set_for_path(path) {
                let loop_patterns = rule_set
                    .loop_patterns
                    .iter()
                    .map(|loop_pattern| compiled_pattern(language, loop_pattern))
                    .collect::<Vec<_>>();
                for rule in rule_set.rules {
                    for pattern in rule.patterns {
                        let pattern_started = Instant::now();
                        let mut pattern_matches = 0usize;
                        let literal_anchor = pattern_literal_anchor(pattern);
                        for matched in root.find_all(compiled_pattern(language, pattern)) {
                            pattern_matches += 1;
                            if literal_anchor
                                .is_some_and(|anchor| !matched.text().starts_with(anchor))
                            {
                                continue;
                            }
                            if !loop_patterns
                                .iter()
                                .any(|loop_pattern| matched.inside(*loop_pattern))
                            {
                                continue;
                            }
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
                        trace_slow_pattern(
                            path,
                            rule.rule_id,
                            pattern,
                            pattern_matches,
                            pattern_started,
                        );
                    }
                }
            }
            for catalog in framework_complexity_catalogs_for_file(path, source) {
                let loop_patterns = rule_set_loop_patterns_for_catalog(catalog)
                    .iter()
                    .map(|loop_pattern| compiled_pattern(language, loop_pattern))
                    .collect::<Vec<_>>();
                for rule in catalog.rules {
                    for pattern in rule.patterns {
                        let pattern_started = Instant::now();
                        let mut pattern_matches = 0usize;
                        let literal_anchor = pattern_literal_anchor(pattern);
                        for matched in root.find_all(compiled_pattern(language, pattern)) {
                            pattern_matches += 1;
                            if literal_anchor
                                .is_some_and(|anchor| !matched.text().starts_with(anchor))
                            {
                                continue;
                            }
                            if !loop_patterns
                                .iter()
                                .any(|loop_pattern| matched.inside(*loop_pattern))
                            {
                                continue;
                            }
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
                                language: String::from(catalog.language_label),
                                provenance: format!("ast_grep.pattern.{}", catalog.framework_id),
                                file_path: path.clone(),
                                line,
                                token: token.clone(),
                                message: String::from(rule.message),
                                matched_text: token,
                                kind: AstGrepFindingKind::AlgorithmicComplexity {
                                    subtype: rule.subtype,
                                    loop_family: String::from(loop_family_for_catalog(catalog)),
                                },
                            });
                        }
                        trace_slow_pattern(
                            path,
                            rule.rule_id,
                            pattern,
                            pattern_matches,
                            pattern_started,
                        );
                    }
                }
            }
        }

        if prefilter.security {
            if let Some(security_rule_set) = security_rule_set_for_path(path) {
                for rule in security_rule_set.rules {
                    for pattern in rule.patterns {
                        let pattern_started = Instant::now();
                        let mut pattern_matches = 0usize;
                        let literal_anchor = pattern_literal_anchor(pattern);
                        for matched in root.find_all(compiled_pattern(language, pattern)) {
                            pattern_matches += 1;
                            if literal_anchor
                                .is_some_and(|anchor| !matched.text().starts_with(anchor))
                            {
                                continue;
                            }
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
                        trace_slow_pattern(
                            path,
                            rule.rule_id,
                            pattern,
                            pattern_matches,
                            pattern_started,
                        );
                    }
                }
            }
        }

        if prefilter.framework_misuse {
            let framework_catalogs = framework_misuse_catalogs_for_file(path, source);
            if framework_catalogs.is_empty() {
                if let Some(framework_rule_set) = framework_misuse_rule_set_for_path(path) {
                    for rule in framework_rule_set.rules {
                        if is_within_sanctioned_boundary(rule.subtype, path) {
                            continue;
                        }
                        for pattern in rule.patterns {
                            let pattern_started = Instant::now();
                            let mut pattern_matches = 0usize;
                            let literal_anchor = pattern_literal_anchor(pattern);
                            for matched in root.find_all(compiled_pattern(language, pattern)) {
                                pattern_matches += 1;
                                if literal_anchor
                                    .is_some_and(|anchor| !matched.text().starts_with(anchor))
                                {
                                    continue;
                                }
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
                            trace_slow_pattern(
                                path,
                                rule.rule_id,
                                pattern,
                                pattern_matches,
                                pattern_started,
                            );
                        }
                    }
                }
            }

            for catalog in framework_catalogs {
                for rule in catalog.rules {
                    if is_within_sanctioned_boundary(rule.subtype, path) {
                        continue;
                    }
                    for pattern in rule.patterns {
                        let pattern_started = Instant::now();
                        let mut pattern_matches = 0usize;
                        let literal_anchor = pattern_literal_anchor(pattern);
                        for matched in root.find_all(compiled_pattern(language, pattern)) {
                            pattern_matches += 1;
                            if literal_anchor
                                .is_some_and(|anchor| !matched.text().starts_with(anchor))
                            {
                                continue;
                            }
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
                        trace_slow_pattern(
                            path,
                            rule.rule_id,
                            pattern,
                            pattern_matches,
                            pattern_started,
                        );
                    }
                }
            }
        }
        trace(&format!(
            "ast_grep.file path={} added_findings={} total_findings={} elapsed_ms={}",
            path.display(),
            findings.len().saturating_sub(findings_before),
            findings.len(),
            file_started.elapsed().as_millis()
        ));
    }

    findings.sort_by(|left, right| {
        left.file_path
            .cmp(&right.file_path)
            .then(left.line.cmp(&right.line))
            .then(left.rule_id.cmp(&right.rule_id))
            .then(left.token.cmp(&right.token))
    });

    let result = AstGrepScanResult {
        scanner: String::from("ast_grep"),
        scanned_files,
        matched_files: matched_files.len(),
        rule_ids: rule_ids.into_iter().collect(),
        skipped_files,
        findings,
    };
    trace(&format!(
        "ast_grep.scan complete scanned_files={} findings={} elapsed_ms={}",
        result.scanned_files,
        result.findings.len(),
        scan_started.elapsed().as_millis()
    ));
    result
}

fn trace(message: &str) {
    if env::var_os("AIGISCORE_TRACE").is_some() {
        eprintln!("[aigiscore] {message}");
    }
}

fn support_lang_for_path(path: &Path) -> Option<SupportLang> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("php" | "phtml" | "php3" | "php4" | "php5" | "php8") => Some(SupportLang::Php),
        Some("py") => Some(SupportLang::Python),
        Some("js" | "jsx") => Some(SupportLang::JavaScript),
        Some("ts") => Some(SupportLang::TypeScript),
        Some("tsx") => Some(SupportLang::Tsx),
        Some("rb" | "rake") => Some(SupportLang::Ruby),
        Some("rs") => Some(SupportLang::Rust),
        _ => None,
    }
}

fn lexical_family_prefilter(path: &Path, source: &str) -> AstGrepFamilyPrefilter {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("php" | "phtml" | "php3" | "php4" | "php5" | "php8") => AstGrepFamilyPrefilter {
            complexity: has_any(source, &["for", "foreach", "while"])
                && has_any(
                    source,
                    &[
                        "in_array",
                        "array_search",
                        "sort(",
                        "usort",
                        "ksort",
                        "asort",
                        "json_decode",
                        "file_get_contents",
                        "file_exists",
                        "DB::",
                        "Http::",
                        "Cache::",
                    ],
                ),
            security: has_any(
                source,
                &[
                    "exec",
                    "system",
                    "passthru",
                    "shell_exec",
                    "proc_open",
                    "popen",
                    "eval",
                    "assert",
                    "unserialize",
                ],
            ),
            framework_misuse: has_any(
                source,
                &["env(", "getenv(", "$_ENV", "app(", "resolve(", "make("],
            ),
        },
        Some("py") => AstGrepFamilyPrefilter {
            complexity: has_any(source, &["for ", "while "])
                && has_any(
                    source,
                    &[
                        "sorted(",
                        ".sort(",
                        "re.compile",
                        "json.loads",
                        "json.load",
                        "os.path.exists",
                        ".exists(",
                        ".read_text(",
                        ".read_bytes(",
                        ".objects.",
                        "connection.cursor().execute",
                        "requests.",
                        "cache.get",
                        "cache.get_many",
                        "cache.get_or_set",
                    ],
                ),
            security: has_any(
                source,
                &[
                    "os.system",
                    "subprocess.",
                    "eval(",
                    "exec(",
                    "pickle.load",
                    "pickle.loads",
                    "yaml.load",
                ],
            ),
            framework_misuse: has_any(source, &["os.environ", "os.getenv"]),
        },
        Some("js" | "jsx" | "ts" | "tsx") => AstGrepFamilyPrefilter {
            complexity: has_any(source, &["for ", "for(", "while ", "while("])
                && has_any(
                    source,
                    &[
                        ".includes(",
                        ".find(",
                        ".some(",
                        ".sort(",
                        "new RegExp",
                        "JSON.parse",
                        "fs.readFile",
                        "fs.existsSync",
                    ],
                ),
            security: has_any(
                source,
                &[
                    "child_process.exec",
                    "eval(",
                    "new Function",
                    "document.write",
                    "innerHTML",
                    "outerHTML",
                ],
            ),
            framework_misuse: has_any(source, &["process.env"]),
        },
        Some("rb" | "rake") => AstGrepFamilyPrefilter {
            complexity: has_any(source, &["for ", "while "])
                && has_any(
                    source,
                    &[
                        ".find(",
                        ".find_by(",
                        ".where(",
                        ".exists?(",
                        ".count(",
                        "Net::HTTP.",
                        "Faraday.",
                        "Rails.cache.",
                    ],
                ),
            security: has_any(
                source,
                &[
                    "system(",
                    "exec(",
                    "IO.popen",
                    "eval(",
                    "instance_eval",
                    "class_eval",
                    "Marshal.load",
                    "YAML.load",
                ],
            ),
            framework_misuse: has_any(source, &["ENV[", "ENV.fetch"]),
        },
        Some("rs") => AstGrepFamilyPrefilter {
            complexity: has_any(source, &["for ", "for(", "while ", "while("])
                && has_any(
                    source,
                    &[
                        ".sort(",
                        ".sort_by(",
                        ".sort_unstable(",
                        ".sort_unstable_by(",
                        "Regex::new",
                        "serde_json::from_",
                        "std::fs::read",
                        "std::fs::read_to_string",
                        "std::fs::metadata",
                        "fs::read",
                        "fs::read_to_string",
                        "fs::metadata",
                    ],
                ),
            security: false,
            framework_misuse: has_any(
                source,
                &[
                    "std::env::var",
                    "std::env::var_os",
                    "env::var",
                    "env::var_os",
                    "env!(",
                    "option_env!(",
                ],
            ),
        },
        _ => AstGrepFamilyPrefilter::default(),
    }
}

fn has_any(source: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| source.contains(needle))
}

fn loop_patterns_for_extension(extension: &str) -> &'static [&'static str] {
    match extension {
        "php" | "phtml" | "php3" | "php4" | "php5" | "php8" => PHP_LOOP_PATTERNS,
        "py" => PYTHON_LOOP_PATTERNS,
        "js" | "jsx" | "ts" | "tsx" => JS_LOOP_PATTERNS,
        "rb" | "rake" => &[
            "for $ITEM in $ITER do\n  $$$BODY\nend",
            "while $COND do\n  $$$BODY\nend",
        ],
        "rs" => RUST_LOOP_PATTERNS,
        _ => &[],
    }
}

fn loop_family_for_catalog(
    catalog: &crate::scanners::framework_catalogs::FrameworkComplexityCatalog,
) -> &'static str {
    match catalog.language_label {
        "python" => "indent_loop",
        _ => "brace_loop",
    }
}

fn rule_set_loop_patterns_for_catalog(
    catalog: &crate::scanners::framework_catalogs::FrameworkComplexityCatalog,
) -> &'static [&'static str] {
    match catalog.language_label {
        "php" => PHP_LOOP_PATTERNS,
        "python" => PYTHON_LOOP_PATTERNS,
        "javascript" | "typescript" => JS_LOOP_PATTERNS,
        "ruby" => loop_patterns_for_extension("rb"),
        "rust" => RUST_LOOP_PATTERNS,
        _ => &[],
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

/// Whether `path` already sits inside the sanctioned boundary a misuse subtype
/// polices, so a clue there would contradict the rule's own semantics — a
/// `raw_env_outside_config` hit inside `config/` is not a clue, it is the
/// sanctioned place for env access. Shared with the assessment layer so the
/// raw scanner artifact and the downstream findings agree on the boundary.
pub fn is_within_sanctioned_boundary(subtype: AstGrepFrameworkMisuseSubtype, path: &Path) -> bool {
    match subtype {
        AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig => is_configuration_boundary_path(path),
        AstGrepFrameworkMisuseSubtype::RawContainerLookupOutsideBoundary => {
            is_dependency_boundary_path(path)
        }
    }
}

pub fn is_configuration_boundary_path(path: &Path) -> bool {
    // Leading slash so a repo-root-relative path (`config/app.php`) matches the
    // same `/config/` segment test as a nested one.
    let normalized = format!(
        "/{}",
        path.to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase()
    );
    normalized.contains("/config/")
        || normalized.contains("/settings/")
        || normalized.ends_with("/settings.py")
        || normalized.ends_with("/wp-config.php")
        || normalized.ends_with("/config.php")
}

pub fn is_dependency_boundary_path(path: &Path) -> bool {
    let normalized = format!(
        "/{}",
        path.to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase()
    );
    normalized.contains("/providers/")
        || normalized.contains("/bootstrap/")
        || normalized.contains("/config/")
        || normalized.contains("/settings/")
        || normalized.contains("/initializers/")
        || normalized.contains("/dependencyinjection/")
        || normalized.contains("/container/")
        || normalized.ends_with("serviceprovider.php")
        || normalized.ends_with("/container.php")
        || normalized.ends_with("/container.rs")
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
    fn env_clues_are_not_emitted_inside_config_boundaries() {
        let result = run_ast_grep_scan(&[
            (
                PathBuf::from("app/Modules/Accounting/config/accounting.php"),
                String::from("<?php return ['key' => env('ACCOUNTING_KEY')];\n"),
            ),
            (
                PathBuf::from("app/Services/PricingService.php"),
                String::from(
                    "<?php class PricingService { public function rate() { return env('RATE'); } }\n",
                ),
            ),
        ]);

        let env_paths: Vec<String> = result
            .findings
            .iter()
            .filter(|finding| finding.rule_id.contains("raw_env_outside_config"))
            .map(|finding| finding.file_path.display().to_string())
            .collect();
        // The rule polices env access *outside* config: a hit inside `config/`
        // contradicts the rule's own semantics and must not appear in the raw
        // artifact.
        assert_eq!(
            env_paths,
            vec![String::from("app/Services/PricingService.php")],
            "only the out-of-boundary env access is a clue: {env_paths:?}"
        );
    }

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
    fn detects_javascript_sort_inside_compact_loop() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("src/order.js"),
            String::from("function reorder(groups){for(const group of groups){group.sort();}}\n"),
        )]);

        assert_eq!(result.findings.len(), 1);
        assert!(result.findings[0].token.contains(".sort("));
    }

    #[test]
    fn detects_laravel_database_query_inside_loop() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("app/Services/InvoiceSyncService.php"),
            String::from(
                r#"
<?php
use Illuminate\Support\Facades\DB;

final class InvoiceSyncService {
    public function sync(array $ids): void {
        foreach ($ids as $id) {
            DB::select('select * from invoices where id = ?', [$id]);
        }
    }
}
"#,
            ),
        )]);

        assert!(result
            .rule_ids
            .iter()
            .any(|rule_id| { rule_id == "complexity/php/framework/laravel_db_query_in_loop" }));
        assert!(result.findings.iter().any(|finding| match &finding.kind {
            AstGrepFindingKind::AlgorithmicComplexity { subtype, .. } => {
                *subtype == AstGrepComplexitySubtype::DatabaseQueryInLoop
                    && finding.provenance == "ast_grep.pattern.laravel"
            }
            _ => false,
        }));
    }

    #[test]
    fn detects_django_cache_lookup_inside_loop() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("app/services/report.py"),
            String::from(
                r#"
from django.core.cache import cache

def collect(keys):
    out = []
    for key in keys:
        out.append(cache.get(key))
    return out
"#,
            ),
        )]);

        assert!(result.rule_ids.iter().any(|rule_id| {
            rule_id == "complexity/python/framework/django_cache_lookup_in_loop"
        }));
        assert!(result.findings.iter().any(|finding| match &finding.kind {
            AstGrepFindingKind::AlgorithmicComplexity { subtype, .. } => {
                *subtype == AstGrepComplexitySubtype::CacheLookupInLoop
                    && finding.provenance == "ast_grep.pattern.django"
            }
            _ => false,
        }));
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
    fn php_superglobal_pattern_matches_only_literal_superglobal_access() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("app/Services/ReportService.phtml"),
            String::from(
                r#"
<?php

final class ReportService
{
    public function build(array $row): array
    {
        $mode = $_ENV['APP_MODE'];
        return [$row['id'], $row['name'], $mode];
    }
}
"#,
            ),
        )]);

        let env_findings = result
            .findings
            .iter()
            .filter(|finding| {
                matches!(
                    finding.kind,
                    AstGrepFindingKind::FrameworkMisuse {
                        subtype: AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
                    }
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(env_findings.len(), 1);
        assert!(env_findings[0].token.starts_with("$_ENV["));
    }

    #[test]
    fn detects_php_raw_container_lookup_as_framework_misuse() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("app/Services/ReportService.phtml"),
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

    #[test]
    fn scans_ruby_security_rules_without_complexity_support() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("config/initializers/danger.rake"),
            String::from(
                r#"
payload = params[:code]
eval(payload)
"#,
            ),
        )]);

        assert_eq!(result.findings.len(), 1);
        match &result.findings[0].kind {
            AstGrepFindingKind::SecurityDangerousApi { category, .. } => {
                assert_eq!(*category, AstGrepSecurityCategory::CodeInjection);
            }
            _ => panic!("expected security finding"),
        }
    }

    #[test]
    fn skips_oversized_files_with_explicit_reason() {
        let mut source = String::from("<?php\n");
        source.push_str(&"// filler to exceed scanner budget\n".repeat(6000));

        let result = run_ast_grep_scan(&[(PathBuf::from("app/HugeJob.php"), source)]);

        assert!(result.findings.is_empty());
        assert_eq!(result.skipped_files.len(), 1);
        assert_eq!(
            result.skipped_files[0].reason,
            "file_too_large_for_secondary_scan"
        );
        assert!(!result.is_empty());
    }

    #[test]
    fn records_prefilter_skips_with_explicit_reason() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("src/app.ts"),
            String::from("export const answer = 42;\n"),
        )]);

        assert!(result.findings.is_empty());
        assert_eq!(result.scanned_files, 0);
        assert_eq!(result.skipped_files.len(), 1);
        assert_eq!(result.skipped_files[0].reason, "no_family_prefilter_hit");
        assert_eq!(
            result.skipped_files[0].file_path,
            PathBuf::from("src/app.ts")
        );
        assert!(!result.is_empty());
    }

    #[test]
    fn detects_rust_read_to_string_inside_loop() {
        let result = run_ast_grep_scan(&[(
            PathBuf::from("src/loader.rs"),
            String::from(
                r#"
fn load(paths: &[String]) {
    for path in paths.iter() {
        let _ = std::fs::read_to_string(path);
    }
}
"#,
            ),
        )]);

        assert_eq!(result.findings.len(), 1);
        assert!(result.findings[0].token.contains("read_to_string"));
    }
}
