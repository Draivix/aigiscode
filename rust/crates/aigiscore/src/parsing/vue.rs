use super::javascript::{parse_javascript_to_graph, JavaScriptParseError};
use crate::graph::SemanticGraph;
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
    /// Whether at least one `<script>` block was found.
    pub has_script: bool,
}

/// Isolate the `<script>` / `<script setup>` content of a Vue SFC.
///
/// Everything outside script blocks (the `<template>`, `<style>`, and the tags
/// themselves) is blanked to spaces while preserving newlines, so the returned
/// buffer parses as plain JS/TS with byte offsets and line numbers unchanged.
pub fn extract_script(source: &str) -> VueScript {
    let bytes = source.as_bytes();
    let lower = source.to_ascii_lowercase();
    let mut masked: Vec<u8> = source
        .bytes()
        .map(|b| if b == b'\n' || b == b'\r' { b } else { b' ' })
        .collect();

    let mut is_typescript = false;
    let mut has_script = false;
    let mut cursor = 0usize;

    while let Some(rel) = lower[cursor..].find("<script") {
        let tag_start = cursor + rel;
        // Confirm this is a real `<script` tag (next char is whitespace or `>`).
        let after = tag_start + "<script".len();
        let boundary_ok = lower[after..]
            .chars()
            .next()
            .map(|c| c.is_whitespace() || c == '>')
            .unwrap_or(false);
        let Some(tag_end_rel) = lower[tag_start..].find('>') else {
            break;
        };
        let content_start = tag_start + tag_end_rel + 1;
        if !boundary_ok {
            cursor = content_start;
            continue;
        }

        let open_tag = &lower[tag_start..content_start];
        if open_tag.contains("lang=\"ts\"")
            || open_tag.contains("lang='ts'")
            || open_tag.contains("lang=\"tsx\"")
            || open_tag.contains("lang='tsx'")
        {
            is_typescript = true;
        }

        let Some(close_rel) = lower[content_start..].find("</script") else {
            break;
        };
        let content_end = content_start + close_rel;
        // Copy the real script bytes back into the masked buffer verbatim.
        masked[content_start..content_end].copy_from_slice(&bytes[content_start..content_end]);
        has_script = true;

        // Advance past this closing tag.
        cursor = match lower[content_end..].find('>') {
            Some(gt) => content_end + gt + 1,
            None => break,
        };
    }

    VueScript {
        masked_source: String::from_utf8_lossy(&masked).into_owned(),
        is_typescript,
        has_script,
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
    parse_javascript_to_graph(file_path, &script.masked_source, script.is_typescript)
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
}
