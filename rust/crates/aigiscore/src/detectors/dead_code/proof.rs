//! Required evidence scopes for absence claims. A complete file is not a
//! complete application, and an unrelated parse gap need not invalidate a file.

use crate::graph::{ReferenceKind, SemanticGraph, SymbolKind, SymbolNode, Visibility};
use crate::ingestion::scan::{AnalysisBoundaryTruth, AnalysisScope};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeadCodeProofScope {
    LocalBinding,
    ClassPrivateDispatch,
    ModuleReachability,
    RuntimeRegistration,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeadCodeProof {
    pub scope: DeadCodeProofScope,
    /// Local source anchors. Module-wide inputs are bound by the scan manifest.
    pub source_files: Vec<PathBuf>,
    pub missing_evidence: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeadCodeScopeCoverage {
    pub complete_local_binding_files: usize,
    pub deferred_local_binding_files: usize,
    pub checked_private_dispatch_symbols: usize,
    pub deferred_private_dispatch_symbols: usize,
    pub module_reachability_checked: bool,
    pub runtime_registration_checked: bool,
}

pub(super) struct ProofScopes<'a> {
    complete_files: HashSet<&'a Path>,
    class_files: HashMap<&'a str, Option<Vec<PathBuf>>>,
    pub whole_modules: bool,
}

impl<'a> ProofScopes<'a> {
    pub fn new(graph: &'a SemanticGraph, scope: &AnalysisScope) -> Self {
        let dynamic_binding_files = graph.references.iter().filter(|reference| reference.kind == ReferenceKind::Call
            && ["eval", "exec", "locals", "globals"].iter()
                .any(|target| reference.target_name.trim_start_matches('\\').eq_ignore_ascii_case(target)))
            .map(|reference| reference.file_path.as_path()).collect::<HashSet<_>>();
        let complete_files = graph.parse_outcomes.iter()
            .filter(|outcome| outcome.is_complete_source())
            .filter(|outcome| !dynamic_binding_files.contains(outcome.file_path.as_path()))
            .map(|outcome| outcome.file_path.as_path()).collect::<HashSet<_>>();
        let symbols = graph.symbols.iter().map(|symbol| (symbol.id.as_str(), symbol)).collect::<HashMap<_, _>>();
        let containers = graph.symbols.iter().filter(|symbol| matches!(symbol.kind, SymbolKind::Class | SymbolKind::Trait | SymbolKind::Enum))
            .map(|symbol| symbol.id.as_str()).collect::<HashSet<_>>();
        let mut edges = HashMap::<(&Path, usize, Option<&str>, &str), Vec<&SymbolNode>>::new();
        for edge in &graph.resolved_edges {
            if edge.kind == ReferenceKind::Type {
                if let (Some(target), Some(name)) = (symbols.get(edge.target_symbol_id.as_str()), edge.reference_target_name.as_deref()) {
                    edges.entry((&edge.source_file_path, edge.line, edge.source_symbol_id.as_deref(), name)).or_default().push(target);
                }
            }
        }
        let mut traits = HashMap::<&str, Vec<&str>>::new();
        let mut unknown = HashSet::new();
        for reference in &graph.references {
            let Some(owner) = reference.enclosing_symbol_id.as_deref() else { continue; };
            let container = if containers.contains(owner) { Some(owner) }
                else { symbols.get(owner).and_then(|symbol| symbol.parent_symbol_id.as_deref()) };
            if reference.kind == ReferenceKind::Call && dynamic_dispatch_target(&reference.target_name) {
                if let Some(container) = container { unknown.insert(container); }
            }
            // PHP trait-use references are emitted at container scope. Unresolved
            // container type references conservatively leave that scope open.
            if reference.kind == ReferenceKind::Type && containers.contains(owner) {
                match edges.get(&(reference.file_path.as_path(), reference.line, Some(owner), reference.target_name.as_str())) {
                    Some(targets) if targets.len() == 1 => {
                        for target in targets.iter().filter(|target| target.kind == SymbolKind::Trait) {
                            traits.entry(owner).or_default().push(target.id.as_str());
                        }
                    }
                    _ => { unknown.insert(owner); }
                }
            }
        }
        let mut class_files = HashMap::new();
        for class in graph.symbols.iter().filter(|symbol| matches!(symbol.kind, SymbolKind::Class | SymbolKind::Enum)) {
            if !super::is_php_file(&class.file_path) { continue; }
            let mut pending = vec![class.id.as_str()];
            let mut visited = HashSet::new();
            let mut files = Vec::new();
            let mut complete = true;
            while let Some(id) = pending.pop() {
                if !visited.insert(id) { continue; }
                let Some(symbol) = symbols.get(id) else { complete = false; break; };
                if unknown.contains(id) || !complete_files.contains(symbol.file_path.as_path()) {
                    complete = false;
                    break;
                }
                files.push(symbol.file_path.clone());
                pending.extend(traits.get(id).into_iter().flatten().copied());
            }
            files.sort();
            files.dedup();
            class_files.insert(class.id.as_str(), complete.then_some(files));
        }
        Self {
            complete_files,
            class_files,
            whole_modules: graph.input_coverage().is_complete()
                && scope.boundary_truth == AnalysisBoundaryTruth::CompleteRepository,
        }
    }

    pub fn local_binding(&self, path: &Path) -> bool { self.complete_files.contains(path) }

    pub fn private_dispatch(&self, symbol: &SymbolNode) -> Option<DeadCodeProof> {
        if !self.local_binding(&symbol.file_path) { return None; }
        if symbol.visibility == Visibility::Private && symbol.kind == SymbolKind::Method
            && super::is_php_file(&symbol.file_path)
        {
            let files = self.class_files.get(symbol.parent_symbol_id.as_deref()?)?.as_ref()?;
            return Some(DeadCodeProof {
                scope: DeadCodeProofScope::ClassPrivateDispatch,
                source_files: files.clone(),
                missing_evidence: vec!["Reflection, generated code and externally bound closures require runtime review before removal".into()],
            });
        }
        self.whole_modules.then(|| DeadCodeProof {
            scope: DeadCodeProofScope::ModuleReachability,
            source_files: vec![symbol.file_path.clone()],
            missing_evidence: vec!["External consumers, registration and dynamic dispatch require review before removal".into()],
        })
    }
}

fn dynamic_dispatch_target(target: &str) -> bool {
    let target = target.trim_start_matches('\\').to_ascii_lowercase();
    target.starts_with('$') || target.starts_with('{')
        || matches!(target.as_str(), "eval" | "call_user_func" | "call_user_func_array" | "forward_static_call" | "forward_static_call_array" | "reflectionmethod" | "reflectionclass" | "getattr" | "setattr")
}
