//! Reuse native per-file resolution only while the complete lookup context agrees.
//! Parsing, override discovery, runtime plugins and assessments remain independent.

use super::{append_override_edges, resolve_references, ResolutionContext, ResolveConfig};
use crate::graph::{ResolvedEdge, SemanticGraph, SemanticReference};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use xxhash_rust::xxh3::Xxh3;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ResolutionWork {
    pub context_changed: bool,
    /// Counts reference-bearing files, not all parsed sources.
    pub files_processed: usize,
    pub files_reused: usize,
    /// Resolution attempts, including sites that remain unresolved.
    pub references_processed: usize,
    pub references_reused: usize,
}

struct FileResolution {
    references: u128,
    // Sparse offsets preserve unresolved sites and original reference ordering.
    edges: Vec<(usize, ResolvedEdge)>,
}

/// Process-local cache owned by the single index writer. It stores native edges
/// before plugins/recovery downgrades, never the transformed published edges.
#[derive(Default)]
pub struct ResolutionCache {
    context: Option<u128>,
    files: HashMap<PathBuf, FileResolution>,
}

impl ResolutionCache {
    pub fn resolve(&mut self, graph: &mut SemanticGraph, config: &ResolveConfig) -> ResolutionWork {
        let context = ResolutionContext::from_graph(graph, config);
        let fingerprint = context_fingerprint(&context);
        let mut work = ResolutionWork {
            context_changed: self.context != Some(fingerprint),
            ..ResolutionWork::default()
        };
        if work.context_changed {
            self.files.clear();
        }
        let mut grouped = HashMap::<&Path, Vec<(usize, &SemanticReference)>>::new();
        for (index, reference) in graph.references.iter().enumerate() {
            grouped
                .entry(&reference.file_path)
                .or_default()
                .push((index, reference));
        }
        self.files
            .retain(|path, _| grouped.contains_key(path.as_path()));
        let mut resolved = Vec::new();
        for (file, references) in grouped {
            let mut hash = Xxh3::new();
            for (_, reference) in &references {
                reference.hash(&mut hash);
            }
            let fingerprint = hash.digest128();
            if self
                .files
                .get(file)
                .is_some_and(|cached| cached.references == fingerprint)
            {
                work.files_reused += 1;
                work.references_reused += references.len();
            } else {
                work.files_processed += 1;
                work.references_processed += references.len();
                self.files.insert(
                    file.to_path_buf(),
                    FileResolution {
                        references: fingerprint,
                        edges: resolve_references(
                            references.iter().map(|(_, reference)| *reference),
                            &context,
                        ),
                    },
                );
            }
            // Native output will be transformed downstream; never mutate the cache.
            for (offset, edge) in &self.files[file].edges {
                resolved.push((references[*offset].0, edge.clone()));
            }
        }
        resolved.sort_unstable_by_key(|(index, _)| *index);
        graph.resolved_edges.clear();
        for (_, edge) in resolved {
            graph.add_resolved_edge(edge);
        }
        // Method positions and the inheritance graph can change independently of
        // symbol lookup identity. Recompute this graph-wide pass every time.
        append_override_edges(graph);
        self.context = Some(fingerprint);
        work
    }
}

fn hash_map<K: Hash + Ord, V: Hash>(map: &HashMap<K, V>, hash: &mut Xxh3) {
    let mut entries = map.iter().collect::<Vec<_>>();
    entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
    entries.hash(hash);
}

fn context_fingerprint(context: &ResolutionContext) -> u128 {
    // Exhaustive destructuring makes a new context input a compiler error here
    // until it participates in invalidation. Vector order is semantically relevant.
    let ResolutionContext {
        file_index,
        global_index,
        qualified_index,
        import_map,
        named_import_map,
        declared_imports,
        module_index,
        declared_module_bindings,
        reference_import_map,
        language_map,
    } = context;
    let mut hash = Xxh3::new();
    hash_map(file_index, &mut hash);
    hash_map(global_index, &mut hash);
    hash_map(qualified_index, &mut hash);
    let imported = import_map
        .iter()
        .map(|(path, files)| {
            let mut files = files.iter().collect::<Vec<_>>();
            files.sort_unstable();
            (path, files)
        })
        .collect::<HashMap<_, _>>();
    hash_map(&imported, &mut hash);
    hash_map(named_import_map, &mut hash);
    let mut declared = declared_imports.iter().collect::<Vec<_>>();
    declared.sort_unstable();
    declared.hash(&mut hash);
    hash_map(module_index, &mut hash);
    hash_map(declared_module_bindings, &mut hash);
    hash_map(reference_import_map, &mut hash);
    hash_map(language_map, &mut hash);
    hash.digest128()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::parse_source_file;
    use crate::resolve::{load_resolve_config, resolve_graph_with_config};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn parse(files: &[(&str, &str)]) -> SemanticGraph {
        let mut graph = SemanticGraph::default();
        for (path, source) in files {
            let mut file = parse_source_file(PathBuf::from(path), source).unwrap();
            graph.files.append(&mut file.files);
            graph.symbols.append(&mut file.symbols);
            graph.references.append(&mut file.references);
            graph.parse_outcomes.append(&mut file.parse_outcomes);
        }
        graph
    }

    fn compare(
        cache: &mut ResolutionCache,
        mut graph: SemanticGraph,
        config: &ResolveConfig,
    ) -> ResolutionWork {
        let mut cold = graph.clone();
        resolve_graph_with_config(&mut cold, config);
        let work = cache.resolve(&mut graph, config);
        assert_eq!(graph, cold, "incremental resolution must equal a fresh resolution including edge order and provenance");
        assert_eq!(
            work.references_processed + work.references_reused,
            graph.references.len()
        );
        work
    }

    #[test]
    fn body_edits_reuse_other_files_but_symbol_additions_and_deletions_invalidate() {
        let provider = "<?php namespace Domain; class Writer { public function send(): void { $this->accept(); } private function accept(): void {} }";
        let caller = "<?php namespace App; use Domain\\Writer; function execute(Writer $writer): void { $writer->send(); missing(); }";
        let edited = "<?php namespace App; use Domain\\Writer; function execute(Writer $writer): void { $writer->send(); $writer->send(); missing(); }";
        let mut cache = ResolutionCache::default();
        let config = ResolveConfig::default();
        let before = parse(&[("provider.php", provider), ("caller.php", caller)]);
        assert!(compare(&mut cache, before.clone(), &config).context_changed);
        assert_eq!(compare(&mut cache, before, &config).references_processed, 0);
        let body = parse(&[("provider.php", provider), ("caller.php", edited)]);
        let work = compare(&mut cache, body, &config);
        assert!(!work.context_changed);
        assert_eq!(work.files_processed, 1);
        assert!(work.files_reused > 0);

        // Previously unresolved lookup must acquire the newly declared target.
        let added = parse(&[
            ("provider.php", provider),
            ("caller.php", edited),
            (
                "added.php",
                "<?php function missing(): void { strlen('value'); }",
            ),
        ]);
        let work = compare(&mut cache, added.clone(), &config);
        assert!(work.context_changed);
        assert_eq!(work.files_reused, 0);
        let mut resolved = added;
        cache.resolve(&mut resolved, &config);
        assert!(resolved
            .resolved_edges
            .iter()
            .any(|edge| edge.target_file_path == Path::new("added.php")));
        assert!(cache.files.contains_key(Path::new("added.php")));
        let deleted = parse(&[("provider.php", provider), ("caller.php", edited)]);
        assert!(compare(&mut cache, deleted, &config).context_changed);
        assert!(!cache.files.contains_key(Path::new("added.php")));
    }

    #[test]
    fn signature_changes_and_duplicate_targets_invalidate_the_complete_context() {
        let caller = "<?php function execute(): void { service()->send(); }";
        let mut cache = ResolutionCache::default();
        let config = ResolveConfig::default();
        for provider in [
            "<?php class First { function send(): void {} } class Second { function send(): void {} } function service(): First { return new First; }",
            "<?php class First { function send(): void {} } class Second { function send(): void {} } function service(): Second { return new Second; }",
            "<?php class First { function send(): void {} } class Second { function send($value): void {} } function service(): Second { return new Second; }",
        ] {
            assert!(compare(&mut cache, parse(&[("caller.php", caller), ("provider.php", provider)]), &config).context_changed);
        }
        let duplicate = parse(&[
            ("caller.php", caller),
            ("one.php", "<?php function service(): void {}"),
            ("two.php", "<?php function service(): void {}"),
        ]);
        assert!(compare(&mut cache, duplicate, &config).context_changed);
    }

    #[test]
    fn interleaved_files_and_same_line_occurrences_preserve_fresh_order() {
        let mut graph = parse(&[
            ("a.rs", "fn first() { known(); known(); } fn known() {}"),
            ("b.rs", "fn second() { second(); second(); }"),
        ]);
        let mut cache = ResolutionCache::default();
        let config = ResolveConfig::default();
        compare(&mut cache, graph.clone(), &config);
        graph.references.reverse();
        let work = compare(&mut cache, graph.clone(), &config);
        assert!(!work.context_changed);
        assert_eq!(
            compare(&mut cache, graph.clone(), &config).references_processed,
            0
        );
        cache.resolve(&mut graph, &config);
        let occurrences = graph
            .resolved_edges
            .iter()
            .filter(|edge| edge.reference_target_name.as_deref() == Some("known"))
            .map(|edge| edge.occurrence_index)
            .collect::<Vec<_>>();
        assert_eq!(occurrences, [0, 1]);
    }

    #[test]
    fn downstream_mutations_do_not_contaminate_cached_native_provenance() {
        let input = parse(&[
            ("src/main.rs", "fn main() { support(); }"),
            ("tests/support.rs", "pub fn support() {}"),
        ]);
        let config = ResolveConfig::default();
        let mut cache = ResolutionCache::default();
        let mut transformed = input.clone();
        cache.resolve(&mut transformed, &config);
        for edge in &mut transformed.resolved_edges {
            edge.reason.push_str("; downstream transformation");
            edge.confidence_millis = 1;
        }
        let mut cold = input;
        resolve_graph_with_config(&mut cold, &config);
        let work = cache.resolve(&mut transformed, &config);
        assert_eq!(work.references_processed, 0);
        assert_eq!(transformed, cold);
    }

    #[test]
    fn changed_tsconfig_aliases_refresh_import_targets() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("aigiscode-resolver-cache-{nonce}"));
        fs::create_dir_all(&root).unwrap();
        let files = [
            ("a.ts", "export function execute() {}"),
            ("b.ts", "export function execute() {}"),
            ("caller.ts", "import { execute } from '@api'; execute();"),
        ];
        for (path, source) in files {
            fs::write(root.join(path), source).unwrap();
        }
        let graph = parse(&files);
        let paths = graph
            .files
            .iter()
            .map(|file| file.path.clone())
            .collect::<Vec<_>>();
        let mut cache = ResolutionCache::default();
        for target in ["a.ts", "b.ts"] {
            fs::write(
                root.join("tsconfig.json"),
                serde_json::json!({
                    "compilerOptions": {"baseUrl": ".", "paths": {"@api": [target]}}
                })
                .to_string(),
            )
            .unwrap();
            let config = load_resolve_config(&root, &paths).unwrap();
            assert!(compare(&mut cache, graph.clone(), &config).context_changed);
            let mut resolved = graph.clone();
            cache.resolve(&mut resolved, &config);
            assert!(resolved
                .resolved_edges
                .iter()
                .any(|edge| edge.source_file_path == Path::new("caller.ts")
                    && edge.target_file_path == Path::new(target)));
        }
        fs::remove_dir_all(root).unwrap();
    }
}
