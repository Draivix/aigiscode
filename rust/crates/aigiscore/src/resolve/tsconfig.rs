//! Per-source TypeScript configuration, read once with its complete input fingerprint.

use super::{normalize_relative_path, relativize_to_root, TsPathAlias};
use globset::{GlobBuilder, GlobMatcher};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid resolver configuration {path}: {message}")]
pub struct ResolveConfigError {
    pub path: PathBuf,
    pub message: String,
}

fn error(path: &Path, message: impl ToString) -> ResolveConfigError {
    ResolveConfigError {
        path: path.to_path_buf(),
        message: message.to_string(),
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompilerOptions {
    base_url: Option<String>,
    paths: Option<BTreeMap<String, Vec<String>>>,
    out_dir: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Extends {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
struct Reference {
    path: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    extends: Option<Extends>,
    #[serde(default)]
    compiler_options: CompilerOptions,
    files: Option<Vec<String>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    #[serde(default)]
    references: Vec<Reference>,
}

#[derive(Debug, Clone, Default)]
struct EffectiveConfig {
    base_url: Option<PathBuf>,
    paths: Option<(PathBuf, BTreeMap<String, Vec<String>>)>,
    out_dir: Option<PathBuf>,
    files: Option<Vec<PathBuf>>,
    include: Option<Vec<PathBuf>>,
    exclude: Option<Vec<PathBuf>>,
}

impl EffectiveConfig {
    fn inherit(&mut self, base: Self) {
        if base.base_url.is_some() {
            self.base_url = base.base_url;
        }
        if base.paths.is_some() {
            self.paths = base.paths;
        }
        if base.out_dir.is_some() {
            self.out_dir = base.out_dir;
        }
        if base.files.is_some() {
            self.files = base.files;
        }
        if base.include.is_some() {
            self.include = base.include;
        }
        if base.exclude.is_some() {
            self.exclude = base.exclude;
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct TsProject {
    pub directory: PathBuf,
    pub aliases: Vec<TsPathAlias>,
    pub base_url: Option<PathBuf>,
    files: Vec<PathBuf>,
    includes: Vec<GlobMatcher>,
    excludes: Vec<GlobMatcher>,
}

impl TsProject {
    pub fn contains(&self, file: &Path) -> bool {
        self.files.iter().any(|candidate| candidate == file)
            || (self.includes.iter().any(|pattern| pattern.is_match(file))
                && !self.excludes.iter().any(|pattern| pattern.is_match(file)))
    }
}

#[derive(Default)]
pub(super) struct ConfigReader {
    // Absence is an input too: adding a closer tsconfig must invalidate fast load.
    inputs: BTreeMap<PathBuf, Option<Vec<u8>>>,
    effective: BTreeMap<PathBuf, EffectiveConfig>,
    active: BTreeSet<PathBuf>,
    active_projects: BTreeSet<PathBuf>,
    projects: BTreeMap<PathBuf, TsProject>,
}

impl ConfigReader {
    pub fn input_paths(&self) -> Vec<PathBuf> {
        self.inputs.keys().cloned().collect()
    }

    pub fn read(&mut self, path: &Path) -> Result<Option<Vec<u8>>, ResolveConfigError> {
        let path = normalize_relative_path(path);
        if let Some(bytes) = self.inputs.get(&path) {
            return Ok(bytes.clone());
        }
        let bytes = match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => None,
            Err(failure) => return Err(error(&path, failure)),
        };
        self.inputs.insert(path, bytes.clone());
        Ok(bytes)
    }

    pub fn fingerprint(&self) -> String {
        let mut hash = xxhash_rust::xxh3::Xxh3::new();
        for (path, bytes) in &self.inputs {
            let name = path.to_string_lossy();
            hash.update(&(name.len() as u64).to_le_bytes());
            hash.update(name.as_bytes());
            hash.update(&[u8::from(bytes.is_some())]);
            if let Some(bytes) = bytes {
                hash.update(&(bytes.len() as u64).to_le_bytes());
                hash.update(bytes);
            }
        }
        format!("{:016x}", hash.digest())
    }

    pub fn load_projects(
        &mut self,
        root: &Path,
        files: &[PathBuf],
    ) -> Result<Vec<TsProject>, ResolveConfigError> {
        let mut directories = BTreeSet::from([root.to_path_buf()]);
        for file in files {
            for directory in file.parent().into_iter().flat_map(Path::ancestors) {
                directories.insert(normalize_relative_path(&root.join(directory)));
            }
        }
        for directory in directories {
            let tsconfig = directory.join("tsconfig.json");
            let jsconfig = directory.join("jsconfig.json");
            if self.read(&tsconfig)?.is_some() {
                self.project(root, &tsconfig)?;
            } else if self.read(&jsconfig)?.is_some() {
                self.project(root, &jsconfig)?;
            }
        }
        Ok(self.projects.values().cloned().collect())
    }

    fn config(&mut self, path: &Path) -> Result<Config, ResolveConfigError> {
        let bytes = self
            .read(path)?
            .ok_or_else(|| error(path, "configuration not found"))?;
        let json = jsonc(&bytes).map_err(|message| error(path, message))?;
        serde_json::from_slice(&json).map_err(|failure| error(path, failure))
    }

    fn effective(&mut self, path: &Path) -> Result<EffectiveConfig, ResolveConfigError> {
        if let Some(config) = self.effective.get(path) {
            return Ok(config.clone());
        }
        if self.active.len() >= 64 || !self.active.insert(path.to_path_buf()) {
            return Err(error(path, "cyclic or excessively deep extends chain"));
        }
        let raw = self.config(path)?;
        let directory = path.parent().unwrap_or(Path::new(""));
        let mut config = EffectiveConfig::default();
        let parents = match raw.extends {
            Some(Extends::One(parent)) => vec![parent],
            Some(Extends::Many(parents)) => parents,
            None => Vec::new(),
        };
        for parent in parents {
            let parent = self.resolve_extends(directory, &parent)?;
            config.inherit(self.effective(&parent)?);
        }
        let absolute = |path: String| normalize_relative_path(&directory.join(path));
        if let Some(base) = raw.compiler_options.base_url {
            config.base_url = Some(absolute(base));
        }
        if let Some(paths) = raw.compiler_options.paths {
            for (pattern, targets) in &paths {
                if pattern.matches('*').count() > 1
                    || targets.iter().any(|target| target.matches('*').count() > 1)
                {
                    return Err(error(
                        path,
                        "paths patterns and targets may contain at most one wildcard",
                    ));
                }
            }
            config.paths = Some((directory.to_path_buf(), paths));
        }
        if let Some(out) = raw.compiler_options.out_dir {
            config.out_dir = Some(absolute(out));
        }
        if let Some(files) = raw.files {
            config.files = Some(files.into_iter().map(absolute).collect());
        }
        if let Some(include) = raw.include {
            config.include = Some(include.into_iter().map(absolute).collect());
        }
        if let Some(exclude) = raw.exclude {
            config.exclude = Some(exclude.into_iter().map(absolute).collect());
        }
        self.active.remove(path);
        self.effective.insert(path.to_path_buf(), config.clone());
        Ok(config)
    }

    fn resolve_extends(
        &mut self,
        directory: &Path,
        name: &str,
    ) -> Result<PathBuf, ResolveConfigError> {
        if name.starts_with('.') || Path::new(name).is_absolute() {
            return self.config_path(&directory.join(name));
        }
        // Package-based tsconfig inheritance uses the nearest node_modules entry.
        for ancestor in directory.ancestors() {
            let candidate = ancestor.join("node_modules").join(name);
            if let Some(path) = self.try_config_path(&candidate)? {
                return Ok(path);
            }
        }
        Err(error(
            &directory.join(name),
            "extended package configuration not found",
        ))
    }

    fn try_config_path(&mut self, candidate: &Path) -> Result<Option<PathBuf>, ResolveConfigError> {
        let candidate = normalize_relative_path(candidate);
        // Check directories separately so a directory read is not confused with a
        // missing file; package.json's tsconfig entry can name a non-default file.
        match fs::metadata(&candidate) {
            Ok(metadata) if metadata.is_dir() => {
                let package = candidate.join("package.json");
                if let Some(bytes) = self.read(&package)? {
                    let json: serde_json::Value = serde_json::from_slice(&bytes)
                        .map_err(|failure| error(&package, failure))?;
                    if let Some(target) = json.get("tsconfig") {
                        let target = target
                            .as_str()
                            .ok_or_else(|| error(&package, "tsconfig must be a string"))?;
                        let target = normalize_relative_path(&candidate.join(target));
                        if self.read(&target)?.is_some() {
                            return Ok(Some(target));
                        }
                        return Err(error(&target, "package tsconfig entry not found"));
                    }
                }
                let path = candidate.join("tsconfig.json");
                return Ok(self.read(&path)?.map(|_| path));
            }
            Err(failure) if failure.kind() != std::io::ErrorKind::NotFound => {
                return Err(error(&candidate, failure))
            }
            _ => {}
        }
        if self.read(&candidate)?.is_some() {
            return Ok(Some(candidate));
        }
        if candidate
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("json")
        {
            let mut name = candidate.into_os_string();
            name.push(".json");
            let path = PathBuf::from(name);
            if self.read(&path)?.is_some() {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn config_path(&mut self, candidate: &Path) -> Result<PathBuf, ResolveConfigError> {
        self.try_config_path(candidate)?
            .ok_or_else(|| error(candidate, "referenced configuration not found"))
    }

    fn project(&mut self, root: &Path, path: &Path) -> Result<(), ResolveConfigError> {
        if self.active_projects.contains(path) || self.active_projects.len() >= 64 {
            return Err(error(
                path,
                "cyclic or excessively deep project reference chain",
            ));
        }
        if self.projects.contains_key(path) {
            return Ok(());
        }
        self.active_projects.insert(path.to_path_buf());
        let config = self.effective(path)?;
        let directory = path.parent().unwrap_or(root);
        let includes = config.include.unwrap_or_else(|| {
            if config.files.is_some() {
                Vec::new()
            } else {
                vec![directory.join("**/*")]
            }
        });
        let mut excludes = config.exclude.unwrap_or_else(|| {
            ["node_modules", "bower_components", "jspm_packages"]
                .into_iter()
                .map(|name| directory.join(name))
                .collect()
        });
        if let Some(out) = config.out_dir {
            excludes.push(out);
        }
        let aliases = config
            .paths
            .map(|(origin, paths)| {
                let base = config.base_url.as_ref().unwrap_or(&origin);
                paths
                    .into_iter()
                    .map(|(pattern, targets)| TsPathAlias {
                        pattern,
                        targets,
                        base_dir: relativize_to_root(base, root),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let patterns = |paths: Vec<PathBuf>| -> Result<Vec<GlobMatcher>, ResolveConfigError> {
            paths
                .into_iter()
                .map(|pattern| {
                    let mut pattern = relativize_to_root(&pattern, root)
                        .to_string_lossy()
                        .replace('\\', "/");
                    if !pattern.contains('*')
                        && !pattern.contains('?')
                        && Path::new(&pattern).extension().is_none()
                    {
                        pattern.push_str("/**/*");
                    }
                    GlobBuilder::new(&pattern)
                        .literal_separator(true)
                        .build()
                        .map(|glob| glob.compile_matcher())
                        .map_err(|failure| error(path, failure))
                })
                .collect()
        };
        self.projects.insert(
            path.to_path_buf(),
            TsProject {
                directory: relativize_to_root(directory, root),
                aliases,
                base_url: config
                    .base_url
                    .as_ref()
                    .map(|base| relativize_to_root(base, root)),
                files: config
                    .files
                    .unwrap_or_default()
                    .iter()
                    .map(|file| relativize_to_root(file, root))
                    .collect(),
                includes: patterns(includes)?,
                excludes: patterns(excludes)?,
            },
        );
        // References are independent projects, never inherited compiler options.
        for reference in self.config(path)?.references {
            let referenced = self.config_path(&directory.join(reference.path))?;
            self.project(root, &referenced)?;
        }
        self.active_projects.remove(path);
        Ok(())
    }
}

/// Remove JSONC comments/trailing commas without changing string contents or line offsets.
fn jsonc(source: &[u8]) -> Result<Vec<u8>, &'static str> {
    let source = source.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(source);
    let mut json = source.to_vec();
    let mut index = 0;
    let mut quoted = false;
    while index < source.len() {
        match source[index] {
            b'\\' if quoted => {
                index += 2;
                continue;
            }
            b'"' => quoted = !quoted,
            b'/' if !quoted && source.get(index + 1) == Some(&b'/') => {
                while index < source.len() && !matches!(source[index], b'\n' | b'\r') {
                    json[index] = b' ';
                    index += 1;
                }
                continue;
            }
            b'/' if !quoted && source.get(index + 1) == Some(&b'*') => {
                json[index] = b' ';
                json[index + 1] = b' ';
                index += 2;
                while index + 1 < source.len() && &source[index..index + 2] != b"*/" {
                    if !matches!(source[index], b'\n' | b'\r') {
                        json[index] = b' ';
                    }
                    index += 1;
                }
                if index + 1 >= source.len() {
                    return Err("unterminated JSONC comment");
                }
                json[index] = b' ';
                json[index + 1] = b' ';
                index += 2;
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    index = 0;
    quoted = false;
    while index < json.len() {
        match json[index] {
            b'\\' if quoted => {
                index += 2;
                continue;
            }
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                let has_value = json[..index]
                    .iter()
                    .rev()
                    .find(|byte| !byte.is_ascii_whitespace())
                    .is_some_and(|byte| !matches!(byte, b'[' | b'{' | b',' | b':'));
                if has_value
                    && json[index + 1..]
                        .iter()
                        .find(|byte| !byte.is_ascii_whitespace())
                        .is_some_and(|byte| matches!(byte, b'}' | b']'))
                {
                    json[index] = b' ';
                }
            }
            _ => {}
        }
        index += 1;
    }
    Ok(json)
}
