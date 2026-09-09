use super::javascript::{parse_javascript_with_dialect, JavaScriptParseError};
use crate::graph::SemanticGraph;
use crate::coverage::ExtractionGap;
use ast_grep_language::{LanguageExt, SupportLang};
use std::borrow::Cow;
use std::path::PathBuf;

/// Result of isolating the runtime script of a Vue single-file component.
pub struct VueScript {
    /// Source buffer the same byte length and line layout as the original `.vue`
    /// file, with every region outside `<script>` blocks replaced by spaces
    /// (newlines preserved). Parsing this keeps line/column numbers aligned with
    /// the original file so findings point at real `.vue` lines.
    pub masked_source: String,
    /// True when any `<script>` tag declares `lang="ts"` / `lang="tsx"`.
    pub is_typescript: bool,
    pub is_tsx: bool,
    /// Whether at least one `<script>` block was found.
    pub has_script: bool,
    /// Extraction cannot account for all declared script content in this file.
    pub gap: Option<ExtractionGap>,
}

/// Isolate the `<script>` / `<script setup>` content of a Vue SFC.
///
/// Everything outside script blocks (the `<template>`, `<style>`, and the tags
/// themselves) is blanked to spaces while preserving newlines, so the returned
/// buffer parses as plain JS/TS with byte offsets and line numbers unchanged.
pub fn extract_script(source: &str) -> VueScript {
    let bytes = source.as_bytes();
    let mut masked: Vec<u8> = source
        .bytes()
        .map(|b| if b == b'\n' || b == b'\r' { b } else { b' ' })
        .collect();

    let mut is_typescript = false;
    let mut is_tsx = false;
    let mut has_script = false;
    // The HTML external scanner treats embedded NUL as end-of-input. Mask it
    // only for locating block boundaries; JS/TS receives the original bytes.
    let first_nul = source.find('\0');
    let structural_source = if first_nul.is_some() {
        Cow::Owned(source.replace('\0', " "))
    } else {
        Cow::Borrowed(source)
    };
    let ast = SupportLang::Html.ast_grep(&structural_source);
    let root = ast.root();
    let mut gap = first_nul.map(|offset| ExtractionGap {
        reason: String::from("vue_embedded_nul"),
        line: source[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1,
    }).or_else(|| root
        .dfs()
        .find(|node| node.is_error() || node.is_missing())
        .map(|node| ExtractionGap {
            reason: String::from("vue_structure_recovery"),
            line: node.start_pos().line() + 1,
        }));

    // Only top-level SFC script blocks are executable component code. A tag
    // inside a comment, template, style or custom block must never become JS.
    for script in root.children() {
        let Some(tag) = script.children().find(|node| {
            matches!(node.kind().as_ref(), "start_tag" | "self_closing_tag")
        }) else {
            continue;
        };
        if !tag.children().any(|node| {
            node.kind() == "tag_name" && node.text().eq_ignore_ascii_case("script")
        }) {
            continue;
        }
        if (script.kind() != "script_element" && tag.kind() != "self_closing_tag")
            || script.dfs().any(|node| node.is_error() || node.is_missing())
        {
            gap.get_or_insert_with(|| ExtractionGap {
                reason: String::from("vue_structure_recovery"), line: tag.start_pos().line() + 1,
            });
            continue;
        }
        let mut language = String::from("js");
        let mut external = false;
        let mut unsupported_type = false;
        for attribute in tag.children().filter(|node| node.kind() == "attribute") {
            let name = attribute.children().find(|node| node.kind() == "attribute_name")
                .map(|node| node.text().to_ascii_lowercase());
            let value = attribute.dfs().find(|node| node.kind() == "attribute_value");
            let value = value.map(|node| node.text().to_ascii_lowercase());
            match name.as_deref() {
                Some("lang") => language = value.unwrap_or_default(),
                Some("src") => external = true,
                Some("type") => {
                    unsupported_type = !matches!(value.as_deref(),
                        None | Some("" | "module" | "text/javascript" | "application/javascript"));
                }
                _ => {}
            }
        }
        if external {
            gap.get_or_insert_with(|| ExtractionGap {
                reason: String::from("vue_external_script"), line: tag.start_pos().line() + 1,
            });
            continue;
        }
        if unsupported_type || !matches!(language.as_str(), "js" | "jsx" | "ts" | "tsx") {
            gap.get_or_insert_with(|| ExtractionGap {
                reason: String::from("vue_unsupported_script_language"), line: tag.start_pos().line() + 1,
            });
            continue;
        }
        if tag.kind() != "self_closing_tag"
            && !script.children().any(|node| node.kind() == "end_tag" && !node.is_missing())
        {
            gap.get_or_insert_with(|| ExtractionGap {
                reason: String::from("vue_structure_recovery"), line: tag.start_pos().line() + 1,
            });
            continue;
        }
        is_typescript |= matches!(language.as_str(), "ts" | "tsx");
        is_tsx |= language == "tsx";
        has_script = true;
        if let Some(content) = script.children().find(|node| node.kind() == "raw_text") {
            let range = content.range();
            masked[range.clone()].copy_from_slice(&bytes[range]);
        }
    }

    VueScript {
        masked_source: String::from_utf8_lossy(&masked).into_owned(),
        is_typescript,
        is_tsx,
        has_script,
        gap,
    }
}

/// Parse a Vue SFC by extracting its `<script>` block and delegating to the
/// JavaScript/TypeScript parser over a line-preserving masked buffer.
pub fn parse_vue_to_graph(
    file_path: impl Into<PathBuf>,
    source: &str,
) -> Result<SemanticGraph, JavaScriptParseError> {
    let file_path = file_path.into();
    let script = extract_script(source);
    let mut graph = parse_javascript_with_dialect(
        file_path, &script.masked_source, script.is_typescript, script.is_tsx,
    )?;
    for outcome in &mut graph.parse_outcomes {
        // Template bindings and non-script regions are outside this adapter's
        // extraction scope, even when the extracted script has valid syntax.
        outcome.scope = crate::coverage::ParseScope::VueScriptOnly;
        outcome.extraction_gap = script.gap.clone();
    }
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_script_setup_ts_and_preserves_line_numbers() {
        let src = "<script setup lang=\"ts\">\nimport Foo from './Foo.vue'\nconst x = 1\n</script>\n<template>\n  <Foo />\n</template>\n";
        let out = extract_script(src);
        assert!(out.has_script);
        assert!(out.is_typescript);
        // Same length, same newline layout.
        assert_eq!(out.masked_source.len(), src.len());
        assert_eq!(
            out.masked_source.lines().count(),
            src.lines().count(),
            "line count preserved"
        );
        // The import must survive on its original line (line 2).
        let line2 = out.masked_source.lines().nth(1).unwrap();
        assert!(line2.contains("import Foo from './Foo.vue'"));
        // Template content must be blanked (no `<Foo` component tag left as code).
        assert!(!out.masked_source.contains("<template"));
        assert!(!out.masked_source.contains("<Foo"));
    }

    #[test]
    fn parses_imports_into_graph() {
        let src =
            "<template><div/></template>\n<script setup lang=\"ts\">\nimport { useThing } from '@/composables/useThing'\n</script>\n";
        let graph = parse_vue_to_graph("resources/js/Widget.vue", src).unwrap();
        let has_import = graph.references.iter().any(|r| {
            r.target_name.contains("useThing")
                || r.binding_name
                    .as_deref()
                    .is_some_and(|b| b.contains("useThing"))
        });
        assert!(has_import, "expected an import reference for useThing");
    }

    #[test]
    fn plain_js_script_is_not_typescript() {
        let src = "<script>\nexport default { name: 'X' }\n</script>\n";
        let out = extract_script(src);
        assert!(out.has_script);
        assert!(!out.is_typescript);
    }

    #[test]
    fn no_script_block_yields_empty_mask() {
        let src = "<template>\n  <div>hi</div>\n</template>\n";
        let out = extract_script(src);
        assert!(!out.has_script);
        assert!(out.masked_source.trim().is_empty());
        assert_eq!(out.masked_source.len(), src.len());
    }

    #[test]
    fn extracts_only_top_level_scripts_with_real_attribute_values() {
        let source = concat!(
            "<!-- <script>const commentOnly = 1;</script> -->\n",
            "<template><script>const nestedOnly = 2;</script></template>\n",
            "<docs><script>const exampleOnly = 3;</script></docs>\n",
            "<script lang = 'ts'>const visible: number = 4;</script>\n",
            "<script setup>const setupValue = 5;</script>\n",
        );
        let script = extract_script(source);
        assert_eq!(script.gap, None);
        assert!(script.is_typescript);
        assert_eq!(script.masked_source.len(), source.len());
        assert!(!script.masked_source.contains("Only"));
        assert!(script.masked_source.lines().nth(3).unwrap().contains("visible: number"));
        assert!(script.masked_source.lines().nth(4).unwrap().contains("setupValue"));
    }

    #[test]
    fn exposes_unavailable_script_content_in_native_parse_evidence() {
        for (source, reason) in [
            ("<script src='./logic.ts'></script>", "vue_external_script"),
            ("<script lang='coffee'>x = 1</script>", "vue_unsupported_script_language"),
            ("<script>const unfinished = 1;", "vue_structure_recovery"),
        ] {
            let graph = parse_vue_to_graph("src/Widget.vue", source).unwrap();
            assert_eq!(graph.parse_outcomes[0].extraction_gap.as_ref().map(|gap| gap.reason.as_str()), Some(reason));
            assert_eq!(graph.parse_outcomes[0].scope, crate::coverage::ParseScope::VueScriptOnly);
        }
    }

    #[test]
    fn selects_tsx_grammar_for_the_original_vue_path() {
        let graph = parse_vue_to_graph(
            "src/Widget.vue",
            "<script lang='tsx'>export const View = () => <div />;</script>",
        ).unwrap();
        assert_eq!(graph.parse_outcomes[0].file_path, PathBuf::from("src/Widget.vue"));
        assert_eq!(graph.parse_outcomes[0].parser, "tree-sitter-tsx");
        assert!(!graph.parse_outcomes[0].required_recovery);
    }

    #[test]
    fn embedded_nul_does_not_discard_the_rest_of_a_script() {
        let source = "<script setup lang='ts'>\nconst sentinel = '\0';\nconst later = eval(input);\n</script>\n";
        let script = extract_script(source);
        assert!(script.has_script);
        assert!(script.is_typescript);
        assert_eq!(script.masked_source.len(), source.len());
        assert!(script.masked_source.contains("const sentinel = '\0';"));
        assert!(script.masked_source.lines().nth(2).unwrap().contains("eval(input)"));
        let gap = script.gap.unwrap();
        assert_eq!(gap.reason, "vue_embedded_nul");
        assert_eq!(gap.line, 2);
    }

    #[test]
    fn quoted_generic_attribute_does_not_enter_the_script() {
        let source = "<script setup lang=\"ts\" generic=\"T extends Record<string, unknown>\">\nconst value: T | null = null;\n</script>";
        let script = extract_script(source);
        assert!(script.masked_source.lines().next().unwrap().trim().is_empty());
        assert!(script.masked_source.contains("const value: T | null = null;"));
        let graph = parse_vue_to_graph("src/Generic.vue", source).unwrap();
        assert!(!graph.parse_outcomes[0].required_recovery);
    }
}
