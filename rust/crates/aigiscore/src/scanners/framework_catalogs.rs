use crate::scanners::ast_grep::{AstGrepComplexitySubtype, AstGrepFrameworkMisuseSubtype};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
pub(crate) struct FrameworkMisuseRuleSpec {
    pub rule_id: &'static str,
    pub family: &'static str,
    pub message: &'static str,
    pub subtype: AstGrepFrameworkMisuseSubtype,
    pub patterns: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FrameworkMisuseCatalog {
    pub framework_id: &'static str,
    pub language_label: &'static str,
    pub rules: &'static [FrameworkMisuseRuleSpec],
    pub matches_file: fn(&Path, &str) -> bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FrameworkComplexityRuleSpec {
    pub rule_id: &'static str,
    pub family: &'static str,
    pub message: &'static str,
    pub subtype: AstGrepComplexitySubtype,
    pub patterns: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FrameworkComplexityCatalog {
    pub framework_id: &'static str,
    pub language_label: &'static str,
    pub rules: &'static [FrameworkComplexityRuleSpec],
    pub matches_file: fn(&Path, &str) -> bool,
}

/// Where a dispatch-mechanism marker is matched: against the file path or its body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MechanismMarkerKind {
    PathContains,
    ContentContains,
}

/// One framework-specific marker that classifies a file into a dispatch-mechanism
/// family (`lifecycle_hooks`, `event_bus`, `queue_jobs`, `direct_notifications`).
/// The generic `DuplicateMechanism` engine in `assessment` owns the grouping and
/// scoring; the vocabulary of what *counts* as a mechanism lives here as data so
/// core stays framework-agnostic and markers only ever apply to their own language.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MechanismMarkerSpec {
    pub family: &'static str,
    pub kind: MechanismMarkerKind,
    pub needle: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DispatchMechanismCatalog {
    pub markers: &'static [MechanismMarkerSpec],
    /// Optional word-boundary regex for the `direct_notifications` family, whose
    /// vocabulary (`wp_mail(`, `Mail::send(`, `->notify(`, `phpmailer`, …) does not
    /// reduce cleanly to plain substrings without matching `email(`/`gmail(`.
    pub notification_pattern: Option<fn() -> &'static Regex>,
    pub matches_file: fn(&Path, &str) -> bool,
}

const LARAVEL_PHP_FRAMEWORK_MISUSE_RULES: &[FrameworkMisuseRuleSpec] = &[FrameworkMisuseRuleSpec {
    rule_id: "framework_misuse/php/raw_env_outside_config",
    family: "framework_misuse",
    message: "Raw environment access should stay inside a config/bootstrap boundary.",
    subtype: AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
    patterns: &["env($$$ARGS)", "getenv($$$ARGS)", "$_ENV[$$$ARGS]"],
},
FrameworkMisuseRuleSpec {
    rule_id: "framework_misuse/php/raw_container_lookup_outside_boundary",
    family: "framework_misuse",
    message:
        "Raw container lookup should stay inside provider/bootstrap seams or be replaced by injection.",
    subtype: AstGrepFrameworkMisuseSubtype::RawContainerLookupOutsideBoundary,
    patterns: &[
        "app($CLASS)",
        "app()->make($CLASS)",
        "resolve($CLASS)",
        "App::make($CLASS)",
        "Container::getInstance()->make($CLASS)",
        "$this->app->make($CLASS)",
        "$app->make($CLASS)",
    ],
}];

const LARAVEL_PHP_FRAMEWORK_MISUSE_CATALOG: FrameworkMisuseCatalog = FrameworkMisuseCatalog {
    framework_id: "laravel",
    language_label: "php",
    rules: LARAVEL_PHP_FRAMEWORK_MISUSE_RULES,
    matches_file: is_laravel_php_file,
};

const LARAVEL_PHP_FRAMEWORK_COMPLEXITY_RULES: &[FrameworkComplexityRuleSpec] = &[
    FrameworkComplexityRuleSpec {
        rule_id: "complexity/php/framework/laravel_db_query_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated Laravel database queries inside a loop should be hoisted, batched, or prefetched.",
        subtype: AstGrepComplexitySubtype::DatabaseQueryInLoop,
        patterns: &[
            "DB::select($$$ARGS)",
            "DB::insert($$$ARGS)",
            "DB::update($$$ARGS)",
            "DB::delete($$$ARGS)",
            "DB::statement($$$ARGS)",
            "DB::unprepared($$$ARGS)",
            "DB::table($$$ARGS)->get()",
            "DB::table($$$ARGS)->first()",
            "DB::table($$$ARGS)->exists()",
            "DB::table($$$ARGS)->count()",
            "DB::table($$$ARGS)->value($$$ARGS)",
        ],
    },
    FrameworkComplexityRuleSpec {
        rule_id: "complexity/php/framework/laravel_http_call_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated Laravel HTTP client calls inside a loop should be batched, pooled, or moved behind a bulk endpoint.",
        subtype: AstGrepComplexitySubtype::HttpCallInLoop,
        patterns: &[
            "Http::get($$$ARGS)",
            "Http::post($$$ARGS)",
            "Http::put($$$ARGS)",
            "Http::patch($$$ARGS)",
            "Http::delete($$$ARGS)",
            "Http::send($$$ARGS)",
        ],
    },
    FrameworkComplexityRuleSpec {
        rule_id: "complexity/php/framework/laravel_cache_lookup_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated Laravel cache lookups inside a loop should be grouped, memoized, or replaced by bulk retrieval.",
        subtype: AstGrepComplexitySubtype::CacheLookupInLoop,
        patterns: &[
            "Cache::get($$$ARGS)",
            "Cache::has($$$ARGS)",
            "Cache::remember($$$ARGS)",
            "Cache::rememberForever($$$ARGS)",
            "Cache::many($$$ARGS)",
        ],
    },
];

const LARAVEL_PHP_FRAMEWORK_COMPLEXITY_CATALOG: FrameworkComplexityCatalog =
    FrameworkComplexityCatalog {
        framework_id: "laravel",
        language_label: "php",
        rules: LARAVEL_PHP_FRAMEWORK_COMPLEXITY_RULES,
        matches_file: is_laravel_php_file,
    };

const DJANGO_PYTHON_FRAMEWORK_MISUSE_RULES: &[FrameworkMisuseRuleSpec] =
    &[FrameworkMisuseRuleSpec {
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

const DJANGO_PYTHON_FRAMEWORK_MISUSE_CATALOG: FrameworkMisuseCatalog = FrameworkMisuseCatalog {
    framework_id: "django",
    language_label: "python",
    rules: DJANGO_PYTHON_FRAMEWORK_MISUSE_RULES,
    matches_file: is_django_python_file,
};

const DJANGO_PYTHON_FRAMEWORK_COMPLEXITY_RULES: &[FrameworkComplexityRuleSpec] = &[
    FrameworkComplexityRuleSpec {
        rule_id: "complexity/python/framework/django_db_query_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated Django ORM or cursor queries inside a loop should be prefetched, aggregated, or batched.",
        subtype: AstGrepComplexitySubtype::DatabaseQueryInLoop,
        patterns: &[
            "$MODEL.objects.get($$$ARGS)",
            "$MODEL.objects.filter($$$ARGS)",
            "$MODEL.objects.exists()",
            "$MODEL.objects.count()",
            "connection.cursor().execute($$$ARGS)",
        ],
    },
    FrameworkComplexityRuleSpec {
        rule_id: "complexity/python/framework/django_http_call_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated outbound HTTP calls inside a loop should be batched, pooled, or replaced by a bulk fetch path.",
        subtype: AstGrepComplexitySubtype::HttpCallInLoop,
        patterns: &[
            "requests.get($$$ARGS)",
            "requests.post($$$ARGS)",
            "requests.put($$$ARGS)",
            "requests.delete($$$ARGS)",
            "requests.request($$$ARGS)",
        ],
    },
    FrameworkComplexityRuleSpec {
        rule_id: "complexity/python/framework/django_cache_lookup_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated Django cache lookups inside a loop should be grouped, memoized, or replaced by bulk retrieval.",
        subtype: AstGrepComplexitySubtype::CacheLookupInLoop,
        patterns: &[
            "cache.get($$$ARGS)",
            "cache.get_many($$$ARGS)",
            "cache.has_key($$$ARGS)",
            "cache.get_or_set($$$ARGS)",
        ],
    },
];

const DJANGO_PYTHON_FRAMEWORK_COMPLEXITY_CATALOG: FrameworkComplexityCatalog =
    FrameworkComplexityCatalog {
        framework_id: "django",
        language_label: "python",
        rules: DJANGO_PYTHON_FRAMEWORK_COMPLEXITY_RULES,
        matches_file: is_django_python_file,
    };

const RAILS_RUBY_FRAMEWORK_MISUSE_RULES: &[FrameworkMisuseRuleSpec] = &[FrameworkMisuseRuleSpec {
    rule_id: "framework_misuse/ruby/raw_env_outside_config",
    family: "framework_misuse",
    message: "Raw environment access should stay inside a config/bootstrap boundary.",
    subtype: AstGrepFrameworkMisuseSubtype::RawEnvOutsideConfig,
    patterns: &["ENV[$$$ARGS]", "ENV.fetch($$$ARGS)"],
}];

const RAILS_RUBY_FRAMEWORK_MISUSE_CATALOG: FrameworkMisuseCatalog = FrameworkMisuseCatalog {
    framework_id: "rails",
    language_label: "ruby",
    rules: RAILS_RUBY_FRAMEWORK_MISUSE_RULES,
    matches_file: is_rails_ruby_file,
};

const RAILS_RUBY_FRAMEWORK_COMPLEXITY_RULES: &[FrameworkComplexityRuleSpec] = &[
    FrameworkComplexityRuleSpec {
        rule_id: "complexity/ruby/framework/rails_db_query_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated Rails database queries inside a loop should be eager loaded, aggregated, or batched.",
        subtype: AstGrepComplexitySubtype::DatabaseQueryInLoop,
        patterns: &[
            "$MODEL.find($$$ARGS)",
            "$MODEL.find_by($$$ARGS)",
            "$MODEL.where($$$ARGS)",
            "$MODEL.exists?($$$ARGS)",
            "$MODEL.count($$$ARGS)",
        ],
    },
    FrameworkComplexityRuleSpec {
        rule_id: "complexity/ruby/framework/rails_http_call_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated outbound HTTP calls inside a loop should be batched, pooled, or replaced by a bulk fetch path.",
        subtype: AstGrepComplexitySubtype::HttpCallInLoop,
        patterns: &[
            "Net::HTTP.get($$$ARGS)",
            "Net::HTTP.post($$$ARGS)",
            "Faraday.get($$$ARGS)",
            "Faraday.post($$$ARGS)",
        ],
    },
    FrameworkComplexityRuleSpec {
        rule_id: "complexity/ruby/framework/rails_cache_lookup_in_loop",
        family: "algorithmic_complexity",
        message: "Repeated Rails cache lookups inside a loop should be grouped, memoized, or replaced by bulk retrieval.",
        subtype: AstGrepComplexitySubtype::CacheLookupInLoop,
        patterns: &[
            "Rails.cache.read($$$ARGS)",
            "Rails.cache.fetch($$$ARGS)",
            "Rails.cache.exist?($$$ARGS)",
        ],
    },
];

const RAILS_RUBY_FRAMEWORK_COMPLEXITY_CATALOG: FrameworkComplexityCatalog =
    FrameworkComplexityCatalog {
        framework_id: "rails",
        language_label: "ruby",
        rules: RAILS_RUBY_FRAMEWORK_COMPLEXITY_RULES,
        matches_file: is_rails_ruby_file,
    };

/// PHP / Laravel / WordPress dispatch idioms. Markers require real syntax
/// (`ShouldQueue`, `::dispatch(`, `event(new `, `add_action(`, framework dir
/// segments) rather than bare English words, so a variable named `queued_files`
/// or prose mentioning "event" does not classify a mechanism.
const PHP_DISPATCH_MECHANISM_MARKERS: &[MechanismMarkerSpec] = &[
    // lifecycle hooks — WordPress actions/filters + ORM save hooks
    MechanismMarkerSpec {
        family: "lifecycle_hooks",
        kind: MechanismMarkerKind::PathContains,
        needle: "/hooks/",
    },
    MechanismMarkerSpec {
        family: "lifecycle_hooks",
        kind: MechanismMarkerKind::PathContains,
        needle: ".hook.php",
    },
    MechanismMarkerSpec {
        family: "lifecycle_hooks",
        kind: MechanismMarkerKind::PathContains,
        needle: ".hooks.php",
    },
    MechanismMarkerSpec {
        family: "lifecycle_hooks",
        kind: MechanismMarkerKind::ContentContains,
        needle: "add_action(",
    },
    MechanismMarkerSpec {
        family: "lifecycle_hooks",
        kind: MechanismMarkerKind::ContentContains,
        needle: "add_filter(",
    },
    MechanismMarkerSpec {
        family: "lifecycle_hooks",
        kind: MechanismMarkerKind::ContentContains,
        needle: "beforesave",
    },
    MechanismMarkerSpec {
        family: "lifecycle_hooks",
        kind: MechanismMarkerKind::ContentContains,
        needle: "aftersave",
    },
    // event bus — Laravel events/listeners + broadcast + generic dispatcher
    MechanismMarkerSpec {
        family: "event_bus",
        kind: MechanismMarkerKind::PathContains,
        needle: "/listeners/",
    },
    MechanismMarkerSpec {
        family: "event_bus",
        kind: MechanismMarkerKind::ContentContains,
        needle: "dispatchesevents",
    },
    MechanismMarkerSpec {
        family: "event_bus",
        kind: MechanismMarkerKind::ContentContains,
        needle: "shouldbroadcast",
    },
    MechanismMarkerSpec {
        family: "event_bus",
        kind: MechanismMarkerKind::ContentContains,
        needle: "event(new ",
    },
    MechanismMarkerSpec {
        family: "event_bus",
        kind: MechanismMarkerKind::ContentContains,
        needle: "::listen(",
    },
    MechanismMarkerSpec {
        family: "event_bus",
        kind: MechanismMarkerKind::ContentContains,
        needle: "->listen(",
    },
    MechanismMarkerSpec {
        family: "event_bus",
        kind: MechanismMarkerKind::ContentContains,
        needle: "eventdispatcher",
    },
    // queue jobs — Laravel queued job markers + dispatch idioms
    MechanismMarkerSpec {
        family: "queue_jobs",
        kind: MechanismMarkerKind::PathContains,
        needle: "/jobs/",
    },
    MechanismMarkerSpec {
        family: "queue_jobs",
        kind: MechanismMarkerKind::ContentContains,
        needle: "shouldqueue",
    },
    MechanismMarkerSpec {
        family: "queue_jobs",
        kind: MechanismMarkerKind::ContentContains,
        needle: "->onqueue(",
    },
    MechanismMarkerSpec {
        family: "queue_jobs",
        kind: MechanismMarkerKind::ContentContains,
        needle: "::dispatch(",
    },
    MechanismMarkerSpec {
        family: "queue_jobs",
        kind: MechanismMarkerKind::ContentContains,
        needle: "dispatch(new ",
    },
];

fn php_direct_notification_pattern() -> &'static Regex {
    static PHP_DIRECT_NOTIFICATION_PATTERN: OnceLock<Regex> = OnceLock::new();
    PHP_DIRECT_NOTIFICATION_PATTERN.get_or_init(|| {
        Regex::new(
            r"(?i)(\bwp_mail\s*\(|\bmail\s*\(|\bmail::send\s*\(|\bsendmail\b|\bmailer\b|\bnotify\s*\(|->notify\s*\(|::notify\s*\(|\bphpmailer\b)",
        )
        .unwrap()
    })
}

const PHP_DISPATCH_MECHANISM_CATALOG: DispatchMechanismCatalog = DispatchMechanismCatalog {
    markers: PHP_DISPATCH_MECHANISM_MARKERS,
    notification_pattern: Some(php_direct_notification_pattern),
    matches_file: is_php_source_file,
};

/// Dispatch-mechanism catalogs whose language gate matches this file. Language
/// gating is what stops PHP/Laravel markers from being tested against Rust, Vue,
/// or TypeScript files — including the analyzer's own source, which contains these
/// marker strings as literals.

/// How a role-shaped file proves it is wired into its framework channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RoleWiringEvidence {
    /// The file itself declares a route contract, or its container name is
    /// mentioned inside any file that declares route contracts.
    RouteContract,
    /// The container name co-occurs on a line with one of the wiring markers
    /// anywhere outside the file itself (and outside test trees).
    LexicalMarker,
}

/// A framework artifact role shape and the evidence that proves wiring.
/// The generic `UnwiredFrameworkArtifact` engine in `assessment` owns matching
/// and judgment; the role vocabulary lives here as data.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RoleWiringSpec {
    pub role: &'static str,
    /// Directory segments that mark the role (any match).
    pub path_segments: &'static [&'static str],
    /// File-stem suffixes that mark the role (any match).
    pub name_suffixes: &'static [&'static str],
    pub evidence: RoleWiringEvidence,
    /// Line-level markers for LexicalMarker evidence.
    pub markers: &'static [&'static str],
    /// Fix directive shown when unwired.
    pub advice: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RoleWiringCatalog {
    pub framework_id: &'static str,
    pub roles: &'static [RoleWiringSpec],
    pub matches_file: fn(&Path, &str) -> bool,
}

static LARAVEL_PHP_ROLE_WIRING_CATALOG: RoleWiringCatalog = RoleWiringCatalog {
    framework_id: "laravel",
    matches_file: is_php_source_file,
    roles: &[
        RoleWiringSpec {
            role: "controller",
            path_segments: &["Controllers"],
            name_suffixes: &["Controller"],
            evidence: RoleWiringEvidence::RouteContract,
            markers: &[],
            advice: "Register a route pointing at this controller (routes file or #[Route] attribute), or delete it.",
        },
        // NOTE: a "job never dispatched" role was evaluated and dropped —
        // real repos wire jobs through dynamic channels (module manifests,
        // DB-driven schedulers, console commands outside the scan slice)
        // that static markers cannot prove, and the zero-reference case is
        // already the orphan detector's finding.
    ],
};

pub(crate) fn role_wiring_catalogs_for_file(
    path: &Path,
    source: &str,
) -> Vec<&'static RoleWiringCatalog> {
    [&LARAVEL_PHP_ROLE_WIRING_CATALOG]
        .into_iter()
        .filter(|catalog| (catalog.matches_file)(path, source))
        .collect()
}

pub(crate) fn dispatch_mechanism_catalogs_for_file(
    path: &Path,
    source: &str,
) -> Vec<&'static DispatchMechanismCatalog> {
    [&PHP_DISPATCH_MECHANISM_CATALOG]
        .into_iter()
        .filter(|catalog| (catalog.matches_file)(path, source))
        .collect()
}

pub(crate) fn framework_misuse_catalogs_for_file(
    path: &Path,
    source: &str,
) -> Vec<&'static FrameworkMisuseCatalog> {
    [
        &LARAVEL_PHP_FRAMEWORK_MISUSE_CATALOG,
        &DJANGO_PYTHON_FRAMEWORK_MISUSE_CATALOG,
        &RAILS_RUBY_FRAMEWORK_MISUSE_CATALOG,
    ]
    .into_iter()
    .filter(|catalog| (catalog.matches_file)(path, source))
    .collect()
}

pub(crate) fn framework_complexity_catalogs_for_file(
    path: &Path,
    source: &str,
) -> Vec<&'static FrameworkComplexityCatalog> {
    [
        &LARAVEL_PHP_FRAMEWORK_COMPLEXITY_CATALOG,
        &DJANGO_PYTHON_FRAMEWORK_COMPLEXITY_CATALOG,
        &RAILS_RUBY_FRAMEWORK_COMPLEXITY_CATALOG,
    ]
    .into_iter()
    .filter(|catalog| (catalog.matches_file)(path, source))
    .collect()
}

/// Any PHP source file. Dispatch-mechanism markers are Laravel/WordPress idioms,
/// so the language gate is the extension alone — no framework-context requirement,
/// which preserves detection on plain `.hooks.php` / `Jobs/` / `Listeners/` files.
fn is_php_source_file(path: &Path, _source: &str) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("php" | "phtml" | "php3" | "php4" | "php5" | "php8")
    )
}

fn is_laravel_php_file(path: &Path, source: &str) -> bool {
    let normalized_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    if !matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("php" | "phtml" | "php3" | "php4" | "php5" | "php8")
    ) {
        return false;
    }

    let normalized_source = source.to_ascii_lowercase();
    normalized_path.contains("/app/")
        || normalized_path.contains("/routes/")
        || normalized_path.contains("/bootstrap/")
        || normalized_path.contains("/config/")
        || normalized_source.contains("illuminate\\")
        || normalized_source.contains("serviceprovider")
        || normalized_source.contains("app(")
        || normalized_source.contains("$this->app")
        || normalized_source.contains("resolve(")
        || normalized_source.contains("config(")
}

fn is_django_python_file(path: &Path, source: &str) -> bool {
    if !matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("py")
    ) {
        return false;
    }
    let normalized_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let normalized_source = source.to_ascii_lowercase();
    normalized_path.ends_with("/settings.py")
        || normalized_path.contains("/management/commands/")
        || normalized_source.contains("from django.conf import settings")
        || normalized_source.contains("django.conf import settings")
        || normalized_source.contains("from django.core.cache import cache")
        || normalized_source.contains("django.core.cache")
        || normalized_source.contains("settings.")
        || normalized_source.contains("django.apps")
        || normalized_source.contains("django.db")
}

fn is_rails_ruby_file(path: &Path, source: &str) -> bool {
    if !matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rb" | "rake")
    ) {
        return false;
    }
    let normalized_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let normalized_source = source.to_ascii_lowercase();
    normalized_path.contains("/config/environments/")
        || normalized_path.contains("/config/initializers/")
        || normalized_path.contains("/app/")
        || normalized_path.ends_with("/config/application.rb")
        || normalized_source.contains("rails.")
        || normalized_source.contains("activesupport")
        || normalized_source.contains("railtie")
}

#[cfg(test)]
mod tests {
    use super::{
        dispatch_mechanism_catalogs_for_file, framework_complexity_catalogs_for_file,
        framework_misuse_catalogs_for_file,
    };
    use std::path::Path;

    #[test]
    fn enables_php_dispatch_catalog_only_for_php_files() {
        let php = dispatch_mechanism_catalogs_for_file(
            Path::new("app/Jobs/SyncAccountJob.php"),
            "final class SyncAccountJob implements ShouldQueue {}",
        );
        assert_eq!(php.len(), 1);
        assert!(php[0]
            .markers
            .iter()
            .any(|marker| marker.family == "queue_jobs"));

        // The analyzer's own Rust source contains these marker strings as literals;
        // the language gate must keep them from ever matching a non-PHP file.
        assert!(dispatch_mechanism_catalogs_for_file(
            Path::new("rust/crates/aigiscore/src/assessment/mod.rs"),
            r#"needle: "shouldqueue", needle: "add_action(", needle: "event(new ""#,
        )
        .is_empty());
        assert!(dispatch_mechanism_catalogs_for_file(
            Path::new("resources/js/components/ItemsGrid.vue"),
            "const queued = ref([]); emit('update');",
        )
        .is_empty());
    }

    #[test]
    fn enables_laravel_catalog_for_php_app_service_shapes() {
        let catalogs = framework_misuse_catalogs_for_file(
            Path::new("app/Services/ReportService.php"),
            r#"
<?php
use Illuminate\Support\Facades\App;

final class ReportService {
    public function build(): array {
        return app(TenantManager::class)->current();
    }
}
"#,
        );

        assert_eq!(catalogs.len(), 1);
        assert_eq!(catalogs[0].framework_id, "laravel");
    }

    #[test]
    fn skips_laravel_catalog_for_django_python_file() {
        let catalogs = framework_misuse_catalogs_for_file(
            Path::new("src/service.py"),
            "import os\nfrom django.conf import settings\n",
        );

        assert!(catalogs
            .iter()
            .all(|catalog| catalog.framework_id != "laravel"));
    }

    #[test]
    fn enables_django_catalog_for_settings_aware_python_files() {
        let catalogs = framework_misuse_catalogs_for_file(
            Path::new("app/services/report.py"),
            r#"
import os
from django.conf import settings

def build():
    return os.environ.get("APP_MODE"), settings.TIMEOUT
"#,
        );

        assert_eq!(catalogs.len(), 1);
        assert_eq!(catalogs[0].framework_id, "django");
    }

    #[test]
    fn enables_rails_catalog_for_initializer_ruby_files() {
        let catalogs = framework_misuse_catalogs_for_file(
            Path::new("config/initializers/runtime.rb"),
            r#"
module RuntimeConfig
  def self.env
    ENV["APP_MODE"] || Rails.env
  end
end
"#,
        );

        assert_eq!(catalogs.len(), 1);
        assert_eq!(catalogs[0].framework_id, "rails");
    }

    #[test]
    fn enables_laravel_complexity_catalog_for_php_runtime_files() {
        let catalogs = framework_complexity_catalogs_for_file(
            Path::new("app/Services/InvoiceSyncService.php"),
            r#"
<?php
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Cache;

final class InvoiceSyncService {}
"#,
        );

        assert_eq!(catalogs.len(), 1);
        assert_eq!(catalogs[0].framework_id, "laravel");
    }

    #[test]
    fn enables_django_complexity_catalog_for_orm_aware_files() {
        let catalogs = framework_complexity_catalogs_for_file(
            Path::new("app/services/report.py"),
            r#"
from django.db import connection
from django.core.cache import cache
"#,
        );

        assert_eq!(catalogs.len(), 1);
        assert_eq!(catalogs[0].framework_id, "django");
    }

    #[test]
    fn enables_rails_complexity_catalog_for_app_ruby_files() {
        let catalogs = framework_complexity_catalogs_for_file(
            Path::new("app/services/report_runner.rb"),
            r#"
module ReportRunner
  def self.run
    Rails.cache.fetch("x") { 1 }
  end
end
"#,
        );

        assert_eq!(catalogs.len(), 1);
        assert_eq!(catalogs[0].framework_id, "rails");
    }
}
