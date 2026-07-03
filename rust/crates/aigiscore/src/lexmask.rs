//! Shared lexical masking.
//!
//! Line/regex-based detectors (dangerous-API scanning, complexity heuristics)
//! must match real code, not text that merely appears inside a string literal or
//! comment. `mask_non_code_spans` blanks the interior of string literals and
//! comments (multi-line aware) while preserving delimiters, code, and character
//! positions, so a later `Regex::is_match`/`brace_delta` over the masked lines
//! sees only executable code.
//!
//! Safety bias: the masker fails *open* (leaves text as code, i.e. reports) when
//! a construct is ambiguous, so it can never hide a real finding behind an
//! over-eager mask. Only constructs that can legitimately span physical lines
//! carry state between lines; single/double quotes are line-local except where a
//! language genuinely allows multi-line strings and has no regex-literal
//! ambiguity (PHP, Rust).

use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MaskLanguage {
    Php,
    Python,
    Ruby,
    JavaScript,
    TypeScript,
    Rust,
}

impl MaskLanguage {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|value| value.to_str()) {
            Some("php") => Some(Self::Php),
            Some("py") => Some(Self::Python),
            Some("rb") => Some(Self::Ruby),
            Some("js" | "jsx" | "mjs" | "cjs") => Some(Self::JavaScript),
            Some("ts" | "tsx") => Some(Self::TypeScript),
            Some("rs") => Some(Self::Rust),
            _ => None,
        }
    }
}

struct MaskRules {
    line_slash: bool,       // `//` line comment
    line_hash: bool,        // `#` line comment
    block: bool,            // `/* ... */`
    backtick: bool,         // JS/TS template literal
    triple: bool,           // Python `'''`/`"""`
    heredoc: bool,          // PHP heredoc/nowdoc
    rust_raw: bool,         // Rust `r#"..."#`
    single_quote: bool,     // treat `'` as a string delimiter
    multiline_quotes: bool, // `'`/`"` strings may span physical lines
}

impl MaskRules {
    fn for_language(language: MaskLanguage) -> Self {
        match language {
            MaskLanguage::Php => Self {
                line_slash: true,
                line_hash: true,
                block: true,
                backtick: false,
                triple: false,
                heredoc: true,
                rust_raw: false,
                single_quote: true,
                multiline_quotes: true,
            },
            MaskLanguage::Python => Self {
                line_slash: false,
                line_hash: true,
                block: false,
                backtick: false,
                triple: true,
                heredoc: false,
                rust_raw: false,
                single_quote: true,
                multiline_quotes: false,
            },
            MaskLanguage::Ruby => Self {
                line_slash: false,
                line_hash: true,
                block: false,
                backtick: false,
                triple: false,
                heredoc: false,
                rust_raw: false,
                single_quote: true,
                multiline_quotes: false,
            },
            MaskLanguage::JavaScript | MaskLanguage::TypeScript => Self {
                line_slash: true,
                line_hash: false,
                block: true,
                backtick: true,
                triple: false,
                heredoc: false,
                rust_raw: false,
                single_quote: true,
                multiline_quotes: false,
            },
            // Rust `'` is a char literal / lifetime marker (`'a`), never a string
            // delimiter, so it must not start a masked span. Strings are `"..."`
            // (multi-line, escaped) and raw strings `r#"..."#`.
            MaskLanguage::Rust => Self {
                line_slash: true,
                line_hash: false,
                block: true,
                backtick: false,
                triple: false,
                heredoc: false,
                rust_raw: true,
                single_quote: false,
                multiline_quotes: true,
            },
        }
    }
}

#[derive(Clone, PartialEq)]
enum Carry {
    None,
    Template,         // JS/TS `...`
    BlockComment,     // `/* ... */`
    TripleSingle,     // Python `'''...'''`
    TripleDouble,     // Python `"""..."""`
    Heredoc(String),  // PHP heredoc/nowdoc, terminated by its label
    StrSingle,        // multi-line `'...'`
    StrDouble,        // multi-line `"..."`
    RawString(usize), // Rust raw string, closed by `"` + N `#`
}

/// Blank string-literal and comment interiors of `source` line by line.
pub fn mask_non_code_spans(language: MaskLanguage, source: &str) -> Vec<String> {
    mask(language, source, true)
}

/// Blank only comment interiors, leaving string literals intact. For detectors
/// that match a code construct whose payload is itself a string argument
/// (`config('key')`, `Route::get('/path')`) and so must keep string contents,
/// but must not fire on the construct being *mentioned* inside a comment.
pub fn mask_comments_only(language: MaskLanguage, source: &str) -> Vec<String> {
    mask(language, source, false)
}

fn mask(language: MaskLanguage, source: &str, blank_strings: bool) -> Vec<String> {
    let rules = MaskRules::for_language(language);
    let mut carry = Carry::None;
    source
        .lines()
        .map(|line| mask_line(line, &rules, &mut carry, blank_strings))
        .collect()
}

fn mask_line(line: &str, rules: &MaskRules, carry: &mut Carry, blank_strings: bool) -> String {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();

    // A heredoc/nowdoc body (and its terminator line) is entirely non-code.
    if let Carry::Heredoc(label) = carry.clone() {
        let trimmed = line.trim();
        let terminates = trimmed == label
            || trimmed.strip_prefix(&label).is_some_and(|rest| {
                rest.chars()
                    .next()
                    .is_none_or(|c| matches!(c, ';' | ',' | ')'))
            });
        if terminates {
            *carry = Carry::None;
        }
        // Heredoc body is string content: blanked only in full-mask mode.
        return if blank_strings {
            " ".repeat(n)
        } else {
            line.to_owned()
        };
    }

    let mut out: Vec<char> = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        match carry {
            Carry::BlockComment => {
                if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    out.extend([' ', ' ']);
                    i += 2;
                    *carry = Carry::None;
                } else {
                    out.push(' ');
                    i += 1;
                }
                continue;
            }
            Carry::TripleSingle | Carry::TripleDouble => {
                let q = if *carry == Carry::TripleSingle {
                    '\''
                } else {
                    '"'
                };
                if chars[i] == q && chars.get(i + 1) == Some(&q) && chars.get(i + 2) == Some(&q) {
                    out.extend([q, q, q]);
                    i += 3;
                    *carry = Carry::None;
                } else {
                    out.push(if blank_strings { ' ' } else { chars[i] });
                    i += 1;
                }
                continue;
            }
            Carry::RawString(hashes) => {
                let hashes = *hashes;
                if chars[i] == '"'
                    && (1..=hashes).all(|k| chars.get(i + k) == Some(&'#'))
                    && (hashes == 0 || chars.get(i + hashes) == Some(&'#'))
                {
                    out.push('"');
                    for _ in 0..hashes {
                        out.push('#');
                    }
                    i += 1 + hashes;
                    *carry = Carry::None;
                } else {
                    out.push(if blank_strings { ' ' } else { chars[i] });
                    i += 1;
                }
                continue;
            }
            Carry::Template => {
                let c = chars[i];
                if c == '\\' {
                    out.push(if blank_strings { ' ' } else { c });
                    if i + 1 < n {
                        out.push(if blank_strings { ' ' } else { chars[i + 1] });
                    }
                    i += 2;
                } else if c == '`' {
                    out.push('`');
                    i += 1;
                    *carry = Carry::None;
                } else if c == '$' && chars.get(i + 1) == Some(&'{') {
                    out.extend(['$', '{']);
                    i += 2;
                    let mut depth = 1;
                    while i < n && depth > 0 {
                        match chars[i] {
                            '{' => depth += 1,
                            '}' => depth -= 1,
                            _ => {}
                        }
                        out.push(chars[i]);
                        i += 1;
                    }
                } else {
                    out.push(if blank_strings { ' ' } else { c });
                    i += 1;
                }
                continue;
            }
            Carry::StrSingle | Carry::StrDouble => {
                let q = if *carry == Carry::StrSingle {
                    '\''
                } else {
                    '"'
                };
                let c = chars[i];
                if c == '\\' {
                    out.push(if blank_strings { ' ' } else { c });
                    if i + 1 < n {
                        out.push(if blank_strings { ' ' } else { chars[i + 1] });
                    }
                    i += 2;
                } else if c == q {
                    out.push(q);
                    i += 1;
                    *carry = Carry::None;
                } else {
                    out.push(if blank_strings { ' ' } else { c });
                    i += 1;
                }
                continue;
            }
            Carry::Heredoc(_) | Carry::None => {}
        }

        let c = chars[i];
        if rules.line_slash && c == '/' && chars.get(i + 1) == Some(&'/') {
            out.extend(std::iter::repeat_n(' ', n - i));
            break;
        }
        // PHP 8 attributes (`#[Route('/path')]`) are code, not comments — only a
        // `#` not followed by `[` opens a line comment.
        if rules.line_hash && c == '#' && chars.get(i + 1) != Some(&'[') {
            out.extend(std::iter::repeat_n(' ', n - i));
            break;
        }
        if rules.block && c == '/' && chars.get(i + 1) == Some(&'*') {
            out.extend([' ', ' ']);
            i += 2;
            *carry = Carry::BlockComment;
            continue;
        }
        if rules.rust_raw && (c == 'r' || c == 'b') && !prev_is_ident(&chars, i) {
            if let Some((consumed, hashes)) = rust_raw_string_opener(&chars, i) {
                out.extend(&chars[i..i + consumed]);
                i += consumed;
                *carry = Carry::RawString(hashes);
                continue;
            }
        }
        if rules.heredoc
            && c == '<'
            && chars.get(i + 1) == Some(&'<')
            && chars.get(i + 2) == Some(&'<')
        {
            let mut j = i + 3;
            while chars.get(j) == Some(&' ') {
                j += 1;
            }
            let quote = chars.get(j).copied().filter(|q| matches!(q, '\'' | '"'));
            if quote.is_some() {
                j += 1;
            }
            let start = j;
            while j < n && (chars[j].is_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            let label: String = chars[start..j].iter().collect();
            if let Some(q) = quote {
                if chars.get(j) == Some(&q) {
                    j += 1;
                }
            }
            if !label.is_empty() {
                out.extend(&chars[i..j]);
                i = j;
                *carry = Carry::Heredoc(label);
                continue;
            }
            out.push(c);
            i += 1;
            continue;
        }
        if rules.triple
            && (c == '\'' || c == '"')
            && chars.get(i + 1) == Some(&c)
            && chars.get(i + 2) == Some(&c)
        {
            out.extend([c, c, c]);
            i += 3;
            *carry = if c == '\'' {
                Carry::TripleSingle
            } else {
                Carry::TripleDouble
            };
            continue;
        }
        if rules.backtick && c == '`' {
            out.push('`');
            i += 1;
            *carry = Carry::Template;
            continue;
        }
        if c == '"' || (c == '\'' && rules.single_quote) {
            out.push(c);
            i += 1;
            let mut closed = false;
            while i < n {
                let cc = chars[i];
                if cc == '\\' {
                    out.push(if blank_strings { ' ' } else { cc });
                    if i + 1 < n {
                        out.push(if blank_strings { ' ' } else { chars[i + 1] });
                    }
                    i += 2;
                } else if cc == c {
                    out.push(c);
                    i += 1;
                    closed = true;
                    break;
                } else {
                    out.push(if blank_strings { ' ' } else { cc });
                    i += 1;
                }
            }
            if !closed && rules.multiline_quotes {
                *carry = if c == '\'' {
                    Carry::StrSingle
                } else {
                    Carry::StrDouble
                };
            }
            continue;
        }
        out.push(c);
        i += 1;
    }

    out.into_iter().collect()
}

fn prev_is_ident(chars: &[char], i: usize) -> bool {
    i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_')
}

/// If a Rust raw-string opener starts at `chars[i]` (`r"`, `r#"`, `br#"`, ...),
/// return `(chars_consumed_for_the_opener, hash_count)`.
fn rust_raw_string_opener(chars: &[char], i: usize) -> Option<(usize, usize)> {
    let mut j = i;
    if chars.get(j) == Some(&'b') {
        j += 1;
    }
    if chars.get(j) != Some(&'r') {
        return None;
    }
    j += 1;
    let hash_start = j;
    while chars.get(j) == Some(&'#') {
        j += 1;
    }
    if chars.get(j) != Some(&'"') {
        return None;
    }
    let hashes = j - hash_start;
    Some((j - i + 1, hashes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn masked(language: MaskLanguage, source: &str) -> String {
        mask_non_code_spans(language, source).join("\n")
    }

    #[test]
    fn masks_multiline_template_literal_but_keeps_interpolation() {
        let src = "const t = `\nfor loop in prose ${forEach(x)}\n`;\nfor (const y of z) {}\n";
        let out = mask_non_code_spans(MaskLanguage::TypeScript, src);
        assert!(
            !out[1].contains("for loop in prose"),
            "prose masked: {:?}",
            out[1]
        );
        assert!(
            out[1].contains("forEach(x)"),
            "interpolation kept: {:?}",
            out[1]
        );
        assert!(out[3].contains("for ("), "real loop kept: {:?}", out[3]);
    }

    #[test]
    fn masks_php_multiline_string_and_heredoc() {
        let src = "$s = 'run for each\ntenant';\n$h = <<<SQL\nselect for while\nSQL;\nforeach ($a as $b) {}\n";
        let out = mask_non_code_spans(MaskLanguage::Php, src);
        assert!(
            !out[1].contains("for each"),
            "line-2 string interior masked: {:?}",
            out[1]
        );
        assert!(
            !out[3].contains("for while"),
            "heredoc body masked: {:?}",
            out[3]
        );
        assert!(
            out[5].contains("foreach"),
            "real foreach kept: {:?}",
            out[5]
        );
    }

    #[test]
    fn masks_rust_raw_string_containing_loop_words() {
        let src = "let p = r#\"for x in y { while true {} }\"#;\nfor z in 0..3 {}\n";
        let out = mask_non_code_spans(MaskLanguage::Rust, src);
        assert!(
            !out[0].contains("for x in y"),
            "raw-string interior masked: {:?}",
            out[0]
        );
        assert!(out[1].contains("for z"), "real loop kept: {:?}", out[1]);
    }

    #[test]
    fn rust_char_literal_and_lifetime_do_not_desync() {
        // A lone `'` (lifetime / char) must not start a masked string.
        let src = "fn f<'a>(c: char) -> bool { c == 'x' && \"for\".len() > 0 }\n";
        let out = masked(MaskLanguage::Rust, src);
        assert!(
            out.contains("c == "),
            "code after char literal preserved: {out:?}"
        );
        assert!(
            !out.contains("\"for\""),
            "double-quoted string masked: {out:?}"
        );
    }

    #[test]
    fn masks_python_triple_quote_block() {
        let src = "x = '''\nfor each thing\n'''\nfor i in range(3):\n    pass\n";
        let out = mask_non_code_spans(MaskLanguage::Python, src);
        assert!(
            !out[1].contains("for each"),
            "triple-quote body masked: {:?}",
            out[1]
        );
        assert!(out[3].contains("for i"), "real loop kept: {:?}", out[3]);
    }

    #[test]
    fn comments_only_mode_keeps_strings_but_blanks_comments() {
        let src = "// falls back to config('mailbox_provisioning')\n/* config('block') */\n$v = config('real_key');\n";
        let out = mask_non_code_spans_comments(src);
        assert!(
            !out[0].contains("config("),
            "line comment blanked: {:?}",
            out[0]
        );
        assert!(
            !out[1].contains("config("),
            "block comment blanked: {:?}",
            out[1]
        );
        assert!(
            out[2].contains("config('real_key')"),
            "real call + key kept: {:?}",
            out[2]
        );
    }

    fn mask_non_code_spans_comments(src: &str) -> Vec<String> {
        mask_comments_only(MaskLanguage::Php, src)
    }

    #[test]
    fn keeps_code_positions_and_delimiters() {
        let out = mask_non_code_spans(MaskLanguage::Php, "$x = \"for\";");
        // Same length, quotes and code kept, interior blanked.
        assert_eq!(out[0], "$x = \"   \";");
    }
}
