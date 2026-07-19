pub mod javascript;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod vue;

use crate::graph::{Language, SemanticGraph, SymbolKind, SymbolNode, Visibility};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseFileError {
    #[error("unsupported source file for parsing: {0}")]
    Unsupported(PathBuf),
    #[error(transparent)]
    Rust(#[from] rust::RustParseError),
    #[error(transparent)]
    JavaScript(#[from] javascript::JavaScriptParseError),
    #[error(transparent)]
    Php(#[from] php::PhpParseError),
    #[error(transparent)]
    Python(#[from] python::PythonParseError),
    #[error(transparent)]
    Ruby(#[from] ruby::RubyParseError),
}

pub fn parse_source_file(
    file_path: impl Into<PathBuf>,
    source: &str,
) -> Result<SemanticGraph, ParseFileError> {
    let file_path = file_path.into();
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => Ok(rust::parse_rust_to_graph(file_path, source)?),
        Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => Ok(
            javascript::parse_javascript_to_graph(file_path, source, false)?,
        ),
        Some("ts") | Some("tsx") | Some("mts") | Some("cts") => Ok(
            javascript::parse_javascript_to_graph(file_path, source, true)?,
        ),
        Some("vue") => Ok(vue::parse_vue_to_graph(file_path, source)?),
        Some("php") | Some("phtml") | Some("php3") | Some("php4") | Some("php5") | Some("php8") => {
            Ok(php::parse_php_to_graph(file_path, source)?)
        }
        Some("py") => Ok(python::parse_python_to_graph(file_path, source)?),
        Some("rb") | Some("rake") => Ok(ruby::parse_ruby_to_graph(file_path, source)?),
        _ => Err(ParseFileError::Unsupported(file_path)),
    }
}

pub fn is_supported_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(
            "rs" | "js"
                | "jsx"
                | "mjs"
                | "cjs"
                | "ts"
                | "tsx"
                | "mts"
                | "cts"
                | "vue"
                | "php"
                | "phtml"
                | "php3"
                | "php4"
                | "php5"
                | "php8"
                | "py"
                | "rb"
                | "rake"
        )
    )
}

#[cfg(test)]
mod tests {
    use super::{is_supported_source_file, parse_source_file};
    use std::path::Path;

    #[test]
    fn parses_module_script_and_type_variants_as_javascript_family() {
        for path in ["server.mjs", "server.cjs", "types.mts", "types.cts"] {
            assert!(
                is_supported_source_file(Path::new(path)),
                "{path} must be supported"
            );
            let graph = parse_source_file(path, "export function main() {}\nmain();\n").unwrap();
            assert!(
                graph.symbols.iter().any(|symbol| symbol.name == "main"),
                "{path} must yield symbols"
            );
        }
    }
}

pub(crate) fn add_file_module_symbol(
    graph: &mut SemanticGraph,
    file_path: &Path,
    language: Language,
    source: &str,
) {
    let module_name = module_symbol_name(file_path);
    graph.add_symbol(SymbolNode {
        id: format!("module:{}", file_path.display()),
        file_path: file_path.to_path_buf(),
        kind: SymbolKind::Module,
        name: module_name.clone(),
        qualified_name: module_name,
        parent_symbol_id: None,
        owner_type_name: None,
        return_type_name: None,
        visibility: Visibility::Public,
        parameter_count: 0,
        required_parameter_count: 0,
        start_line: 1,
        end_line: source.lines().count().max(1),
    });
    if let Some(file) = graph.files.iter_mut().find(|file| file.path == file_path) {
        file.language = language;
    }
}

pub(crate) fn module_symbol_name(file_path: &Path) -> String {
    let stem = file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("module");
    if matches!(stem, "index" | "__init__" | "mod" | "lib" | "main") {
        return file_path
            .parent()
            .and_then(Path::file_name)
            .and_then(|segment| segment.to_str())
            .unwrap_or(stem)
            .to_owned();
    }
    stem.to_owned()
}
