use crate::scanners::ast_grep::AstGrepFrameworkMisuseSubtype;
use std::path::Path;

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
    use super::framework_misuse_catalogs_for_file;
    use std::path::Path;

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
}
