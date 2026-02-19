//! Markdown filling and normalization pipeline.
//!
//! This is the core formatting pipeline that normalizes and wraps Markdown text.
//! It handles the complex interaction between comrak's AST rendering and the
//! text-level normalization needed to match Python/Marko behavior.
//!
//! Ported from Python: `flowmark/linewrapping/markdown_filling.py` and
//! parts of `flowmark/formats/flowmark_markdown.py`
//!
//! # Comrak workarounds
//!
//! Comrak is a CommonMark/GFM parser written in Rust. It differs from
//! Python's marko parser in several ways that required workarounds to
//! achieve output parity. This section documents every workaround,
//! organized by pipeline stage.
//!
//! If a comrak fork or upstream fix addresses any of these, the
//! corresponding workaround can be simplified or removed. Each
//! workaround is tagged with a `COMRAK-WORKAROUNDn` label that appears
//! in the code near the relevant implementation.
//!
//! ## PUA character encoding scheme
//!
//! Several workarounds use Unicode Private Use Area (PUA) characters as
//! sentinel markers that survive comrak's AST construction and rendering
//! without colliding with user content.
//!
//! | Char       | Const               | Purpose                                  |
//! |------------|---------------------|------------------------------------------|
//! | `U+F000`   | `REF_LABEL_START`   | Start of ref-link label in encoded URL   |
//! | `U+F001`   | `REF_LABEL_SEP`     | End of ref-link label in encoded URL     |
//! | `U+F002`   | (in markers)        | FNDEF/REFDEF HTML comment sentinel       |
//! | `U+F003`   | `AUTOLINK_OPEN`     | Autolink `<` replacement                 |
//! | `U+F004`   | `AUTOLINK_CLOSE`    | Autolink `>` replacement                 |
//! | `U+E0xx`   | (computed)          | Escape placeholder for `\x` (xx = ASCII) |
//!
//! ## Pre-parse workarounds (input → comrak)
//!
//! ### COMRAK-WORKAROUND1: Reference link preservation
//!
//! **Problem:** Comrak resolves `[text][label]` reference links during
//! AST construction, replacing them with inline `[text](url)` links.
//! The original label is lost, making round-tripping impossible.
//!
//! **Fix:** Before parsing, extract `[label]: url` definitions and
//! replace `[text][label]` with `[text](\u{F000}label\u{F001})`.
//! During rendering, detect the PUA prefix and emit `[text][label]`.
//! Definitions are stashed as REFDEF HTML comment markers (see W3).
//!
//! **Functions:** `extract_link_ref_defs`, `encode_ref_links`
//!
//! ### COMRAK-WORKAROUND2: Footnote definition preservation
//!
//! **Problem:** Comrak moves all `FootnoteDefinition` nodes to the end
//! of the AST regardless of their source position, and silently drops
//! any definitions that are not referenced in the document body.
//!
//! **Fix:** Extract footnote definitions before parsing and wrap them
//! in HTML comment markers (`<!-- \u{F002}FNDEF\n...\n-->`). Comrak
//! preserves these as `HtmlBlock` nodes at their original positions.
//! During rendering, detect FNDEF markers and re-emit the footnote
//! definitions with proper formatting and line wrapping.
//!
//! **Functions:** `extract_footnote_defs`
//!
//! ### COMRAK-WORKAROUND3: Autolink angle bracket preservation
//!
//! **Problem:** Comrak's autolink extension converts both `<url>` and
//! bare `url` to identical `Link` nodes, losing the angle brackets.
//! After rendering, there is no way to distinguish `<url>` from `url`.
//!
//! **Fix:** Replace `<url>` with `\u{F003}url\u{F004}` before parsing.
//! During rendering, autolinks are detected (text == url) and rendered
//! as bare text. After rendering, PUA markers are restored to `<url>`.
//!
//! **Functions:** `protect_autolinks`, `restore_autolinks`
//!
//! ### COMRAK-WORKAROUND4: Backslash escape preservation
//!
//! **Problem:** Comrak strips backslash escapes in the AST (e.g.,
//! `\~` becomes `~`, `\*` becomes `*`). This loses intentional escapes
//! the author placed in the source.
//!
//! **Fix:** Replace each `\x` with a PUA placeholder (`U+E000` +
//! ASCII code of `x`) before parsing. After rendering, restore the
//! original `\x` sequences. Replacements skip code fences and inline
//! code spans where backslashes are literal.
//!
//! **Functions:** `protect_escapes_outside_code`, `replace_outside_code_spans`
//!
//! ### COMRAK-WORKAROUND5: Typography in footnote bodies
//!
//! **Problem:** FNDEF markers (from W2) become `HtmlBlock` nodes in
//! the AST. The AST-level typography transforms (smart quotes,
//! ellipses) only process `Paragraph`/`Text` nodes, so footnote
//! body text is skipped.
//!
//! **Fix:** Apply typography transforms to the raw text inside FNDEF
//! markers before comrak parsing.
//!
//! **Functions:** `apply_typography_to_fndef_bodies`
//!
//! ### COMRAK-WORKAROUND6: Tag block spacing
//!
//! **Problem:** Jinja/Markdoc/HTML tag-only lines adjacent to block
//! content may not be recognized as block-level elements by comrak
//! without intervening blank lines.
//!
//! **Fix:** Insert blank lines between tag-only lines and adjacent
//! block content before parsing.
//!
//! **Functions:** `preprocess_tag_block_spacing` (in `wrapping::tag_handling`)
//!
//! ## Post-parse workarounds (comrak AST → output)
//!
//! ### COMRAK-WORKAROUND7: Block spacing and sourcepos inaccuracies
//!
//! **Problem:** Comrak's sourcepos for `List`/`Item` nodes includes
//! trailing blank lines, and `HtmlBlock` type 2 can report
//! `end.line < start.line`. This makes it impossible to reliably
//! detect whether blocks were originally separated by blank lines.
//!
//! **Fix:** `last_content_line()` recursively descends into `List` and
//! `Item` nodes to find the true content end line.
//!
//! **Functions:** `last_content_line`
//!
//! ### COMRAK-WORKAROUND8: HTML comment spacing rules
//!
//! **Problem:** Comrak's default block separation inserts blank lines
//! around all block elements, but Python/marko preserves tight spacing
//! around HTML comments and between paragraph→list transitions.
//!
//! **Fix:** In `render_block_children`, three spacing rules suppress
//! blank lines for specific tight transitions:
//! - Rule 1: HTML comment → any block (tight): suppress
//! - Rule 2: Any block → HTML comment (tight): suppress, unless
//!   previous was list/table
//! - Rule 3: Paragraph → list (tight): suppress
//!
//! **Functions:** `render_block_children` (spacing logic)
//!
//! ### COMRAK-WORKAROUND9: Footnote list item rendering
//!
//! **Problem:** Comrak treats `- item` at footnote continuation indent
//! as paragraph continuation text (per `CommonMark`'s rule that bullet
//! lists cannot interrupt paragraphs). Python/marko treats it as a
//! list item within the footnote, rendering continuation lines with
//! 6-space indent (4 footnote + 2 list) instead of 4.
//!
//! **Fix:** In FNDEF rendering, detect body lines starting with list
//! markers (`- `, `* `, `+ `) and render them with proper list item
//! indentation (6-space subsequent indent).
//!
//! **Functions:** FNDEF rendering in `render_block` (`HtmlBlock` handler)
//!
//! ### COMRAK-WORKAROUND10: List looseness over-application
//!
//! **Problem:** Comrak marks an entire list as "loose" when *any*
//! sibling pair has a blank line between them. Python/marko only
//! inserts blank lines where the author explicitly wrote them.
//!
//! **Fix:** In list item rendering, use source positions to check
//! whether blank lines were actually present in the original between
//! specific children, rather than relying on the list's `loose` flag.
//!
//! **Functions:** list rendering in `render_block` (`Item` handler)
//!
//! ## Post-render normalizations
//!
//! ### COMRAK-WORKAROUND11: Period escape cleanup
//!
//! **Problem:** After restoring escape placeholders, `\.` escapes
//! appear throughout the text. Most are unnecessary — they are only
//! needed at line starts where `DIGITS\.` would trigger ordered list
//! interpretation.
//!
//! **Fix:** Remove `\.` escapes except at list-triggering positions.
//!
//! **Functions:** `postprocess_period_escapes`
//!
//! ### COMRAK-WORKAROUND12: Output normalization
//!
//! **Problem:** Comrak's rendering produces minor formatting
//! differences from Python/marko: trailing whitespace on blank lines,
//! space between code fence and language identifier, two spaces after
//! numbered list periods, multiple consecutive blank lines.
//!
//! **Fix:** Four normalization passes clean up these differences.
//!
//! **Functions:** `normalize_comrak_output` (`normalize_blank_lines`,
//! `normalize_code_fences`, `normalize_numbered_lists`,
//! `collapse_blank_lines_outside_code`)

use regex::Regex;
use std::collections::HashSet;
use std::fmt::Write as _;
use std::sync::LazyLock;

use comrak::nodes::{AstNode, ListType, NodeValue, TableAlignment};
use comrak::{Arena, Options};

use crate::config::{DEFAULT_MIN_LINE_LEN, ListSpacing};
use crate::formatter::markdown::flowmark_comrak_options;
use crate::parser::frontmatter::split_frontmatter;
use crate::transform::cleanups::doc_cleanups;
use crate::typography::ellipses::ellipses as apply_ellipses;
use crate::typography::quotes::smart_quotes;
use crate::wrapping::LineWrapper;
use crate::wrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};
use crate::wrapping::tag_handling::preprocess_tag_block_spacing;

// ===== PUA (Private Use Area) markers =====
//
// See module-level docs for the full PUA encoding scheme and the
// COMRAK-WORKAROUND entries that use each marker.

/// COMRAK-WORKAROUND1: start of reference label in PUA-encoded URL.
const REF_LABEL_START: char = '\u{F000}';
/// COMRAK-WORKAROUND1: end/separator of reference label in PUA-encoded URL.
const REF_LABEL_SEP: char = '\u{F001}';

/// Regex for link reference definitions: `[label]: url` or `[label]: url "title"`
/// Handles optional angle-bracket URLs and single/double-quoted or paren-quoted titles.
static LINK_REF_DEF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?m)^[ \t]{0,3}\[([^\]]+)\]:[ \t]+<?([^\s>]+)>?(?:[ \t]+(?:"([^"]*)"|'([^']*)'|\(([^)]*)\)))?[ \t]*$"#,
    )
    .expect("valid LINK_REF_DEF regex")
});

/// Regex for full reference links: `[text][label]`
static FULL_REF_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\[([^\]]+)\]").expect("valid FULL_REF_LINK regex"));

/// Regex for collapsed reference links: `[text][]`
static COLLAPSED_REF_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\[\]").expect("valid COLLAPSED_REF_LINK regex"));

// ===== COMRAK-WORKAROUND12: Output normalization =====

/// Pattern for blank lines with trailing whitespace.
static BLANK_LINE_WS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]+$").expect("valid BLANK_LINE_WS regex"));

/// Pattern for code fence with space before language (horizontal whitespace only).
static CODE_FENCE_SPACE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([ \t]*```)[^\S\n]+(\w)").expect("valid CODE_FENCE_SPACE regex")
});

/// Pattern for numbered list items with two spaces after period.
static NUMBERED_ITEM_TWO_SPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\.  ").expect("valid NUMBERED_ITEM_TWO_SPACES regex"));

/// Normalize blank lines by removing trailing whitespace.
fn normalize_blank_lines(text: &str) -> String {
    BLANK_LINE_WS.replace_all(text, "").into_owned()
}

/// Remove space between code fence and language identifier.
fn normalize_code_fences(text: &str) -> String {
    CODE_FENCE_SPACE.replace_all(text, "$1$2").into_owned()
}

/// Fix numbered list items: convert two spaces to one space after period.
fn normalize_numbered_lists(text: &str) -> String {
    let mut result = String::new();
    for line in text.lines() {
        if let Some(caps) = NUMBERED_ITEM_TWO_SPACES.captures(line) {
            let num = &caps[1];
            let fixed = line.replacen(&format!("{num}.  "), &format!("{num}. "), 1);
            result.push_str(&fixed);
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    // Remove trailing newline if original didn't have one
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

/// COMRAK-WORKAROUND12: Apply all text-level normalizations to comrak output.
fn normalize_comrak_output(text: &str) -> String {
    let text = normalize_blank_lines(text);
    let text = normalize_code_fences(&text);
    let text = normalize_numbered_lists(&text);
    collapse_blank_lines_outside_code(&text)
}

/// Check if a trimmed line is a closing fence matching the given fence string.
fn is_closing_fence(trimmed: &str, fence_str: &str) -> bool {
    if fence_str.is_empty() || !trimmed.starts_with(fence_str) {
        return false;
    }
    let fence_char = fence_str.chars().next().unwrap_or('`');
    trimmed[fence_str.len()..].chars().all(|c| c == fence_char || c.is_whitespace())
}

/// Detect an opening code fence and return the fence string if found.
fn detect_opening_fence(trimmed: &str) -> Option<String> {
    let is_backtick_fence = trimmed.starts_with("```");
    let is_tilde_fence = trimmed.starts_with("~~~");
    if is_backtick_fence || is_tilde_fence {
        let fence_char = if is_backtick_fence { '`' } else { '~' };
        let fence_len = trimmed.chars().take_while(|&c| c == fence_char).count();
        Some(std::iter::repeat_n(fence_char, fence_len).collect())
    } else {
        None
    }
}

/// Process text line-by-line, applying a transformation only outside fenced code blocks.
///
/// `process_outside` receives each non-code, non-fence line and returns zero or more
/// output lines. Code block lines and fence lines are included in the output unchanged.
fn transform_outside_code_fences<F>(text: &str, mut process_outside: F) -> String
where
    F: FnMut(&str) -> Vec<String>,
{
    let lines: Vec<&str> = text.lines().collect();
    let had_trailing_newline = text.ends_with('\n');
    let mut result: Vec<String> = Vec::new();
    let mut in_code = false;
    let mut fence_str = String::new();

    for line in &lines {
        if in_code {
            result.push((*line).to_string());
            if is_closing_fence(line.trim(), &fence_str) {
                in_code = false;
            }
        } else if let Some(fs) = detect_opening_fence(line.trim()) {
            fence_str = fs;
            in_code = true;
            result.push((*line).to_string());
        } else {
            result.extend(process_outside(line));
        }
    }

    let mut output = result.join("\n");
    if had_trailing_newline {
        output.push('\n');
    }
    output
}

/// Collapse multiple blank lines to single blank lines, but preserve
/// content inside code blocks (fenced with backticks or tildes).
///
/// Uses the fence helpers directly (rather than `transform_outside_code_fences`)
/// because it needs to reset the blank-line counter at fence boundaries.
fn collapse_blank_lines_outside_code(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let had_trailing_newline = text.ends_with('\n');
    let mut result: Vec<&str> = Vec::new();
    let mut in_code = false;
    let mut fence_str = String::new();
    let mut consecutive_empty: usize = 0;

    for line in &lines {
        if in_code {
            result.push(line);
            if is_closing_fence(line.trim(), &fence_str) {
                in_code = false;
                consecutive_empty = 0;
            }
        } else if let Some(fs) = detect_opening_fence(line.trim()) {
            fence_str = fs;
            in_code = true;
            consecutive_empty = 0;
            result.push(line);
        } else if line.trim().is_empty() {
            consecutive_empty += 1;
            if consecutive_empty <= 1 {
                result.push(line);
            }
        } else {
            consecutive_empty = 0;
            result.push(line);
        }
    }

    let mut output = result.join("\n");
    if had_trailing_newline {
        output.push('\n');
    }
    output
}

/// COMRAK-WORKAROUND3: PUA replacement for `<` in autolinks.
const AUTOLINK_OPEN: char = '\u{F003}';
/// COMRAK-WORKAROUND3: PUA replacement for `>` in autolinks.
const AUTOLINK_CLOSE: char = '\u{F004}';

/// COMRAK-WORKAROUND3: Regex for angle-bracket autolinks: `<scheme://...>` or `<email@host>`.
static ANGLE_AUTOLINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"<((?:https?|ftp|mailto):[^\s>]+|[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,})>",
    )
    .expect("valid ANGLE_AUTOLINK_RE regex")
});

/// COMRAK-WORKAROUND3: Replace `<url>` with PUA-wrapped text so comrak cannot
/// merge them with bare-URL autolinks. Skips code fences and FNDEF/REFDEF markers.
fn protect_autolinks(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let had_trailing_newline = text.ends_with('\n');
    let mut result_lines: Vec<String> = Vec::new();
    let mut in_code = false;
    let mut fence_str = String::new();
    let mut in_html_comment = false;

    for line in &lines {
        if in_code {
            result_lines.push(line.to_string());
            if is_closing_fence(line.trim(), &fence_str) {
                in_code = false;
            }
            continue;
        }
        if in_html_comment {
            result_lines.push(line.to_string());
            if line.contains("-->") {
                in_html_comment = false;
            }
            continue;
        }
        if let Some(fs) = detect_opening_fence(line.trim()) {
            fence_str = fs;
            in_code = true;
            result_lines.push(line.to_string());
            continue;
        }
        // Skip FNDEF/REFDEF markers — their content contains raw autolinks
        // that should be preserved as-is (they're rendered from the marker, not by comrak).
        if line.trim().starts_with(FNDEF_MARKER_START)
            || line.trim().starts_with(REFDEF_MARKER_PREFIX)
        {
            result_lines.push(line.to_string());
            if !line.contains("-->") {
                in_html_comment = true;
            }
            continue;
        }
        let replaced = ANGLE_AUTOLINK_RE.replace_all(line, |caps: &regex::Captures| {
            format!("{AUTOLINK_OPEN}{}{AUTOLINK_CLOSE}", &caps[1])
        });
        result_lines.push(replaced.into_owned());
    }

    let mut output = result_lines.join("\n");
    if had_trailing_newline && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// COMRAK-WORKAROUND3: Restore PUA-wrapped autolinks back to angle-bracket form.
fn restore_autolinks(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == AUTOLINK_OPEN {
            // Collect until AUTOLINK_CLOSE
            let mut url = String::new();
            for inner_ch in chars.by_ref() {
                if inner_ch == AUTOLINK_CLOSE {
                    break;
                }
                url.push(inner_ch);
            }
            result.push('<');
            result.push_str(&url);
            result.push('>');
        } else {
            result.push(ch);
        }
    }
    result
}

/// COMRAK-WORKAROUND1: HTML comment marker for reference definition placeholders.
/// The full definition text is encoded after the prefix so the render step
/// can emit it without needing external context.
/// Uses PUA character `\u{F002}` to prevent collision with user-authored HTML comments.
const REFDEF_MARKER_PREFIX: &str = "<!-- \u{F002}REFDEF:";

/// Regex for footnote definition start: `[^label]: content`
static FOOTNOTE_DEF_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[ \t]{0,3}\[\^([^\]]+)\]:[ \t]+").expect("valid FOOTNOTE_DEF_START regex")
});

/// COMRAK-WORKAROUND1: Extract link reference definitions from source text (outside
/// code fences). Returns the set of lowercase labels and the text with definitions
/// replaced by HTML comment markers. These markers survive comrak parsing as
/// `HtmlBlock` nodes, preserving the original position of each definition in the AST.
fn extract_link_ref_defs(text: &str) -> (HashSet<String>, String) {
    let mut labels: HashSet<String> = HashSet::new();
    let result = transform_outside_code_fences(text, |line| {
        if let Some(caps) = LINK_REF_DEF.captures(line) {
            let label = &caps[1];
            // Skip footnote definitions (labels starting with ^)
            if label.starts_with('^') {
                return vec![line.to_string()];
            }
            labels.insert(label.to_lowercase());
            vec![format!("{REFDEF_MARKER_PREFIX}{line} -->")]
        } else {
            vec![line.to_string()]
        }
    });
    (labels, result)
}

/// COMRAK-WORKAROUND2: HTML comment marker for footnote definition placeholders.
/// Multi-line: `<!-- \u{F002}FNDEF\n[^label]: content\ncontinuation\n-->`
/// Comrak preserves these as `HtmlBlock` nodes at their original positions.
/// Uses PUA character `\u{F002}` to prevent collision with user-authored HTML comments.
const FNDEF_MARKER_START: &str = "<!-- \u{F002}FNDEF";

/// COMRAK-WORKAROUND2: Extract footnote definitions from source text (outside code
/// fences). Replaces each definition with an HTML comment marker that comrak will
/// preserve as an `HtmlBlock` at the original position. Without this, comrak moves
/// referenced footnotes to the end of the AST and drops unreferenced ones.
fn extract_footnote_defs(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let had_trailing_newline = text.ends_with('\n');
    let mut result_lines: Vec<String> = Vec::new();
    let mut in_code = false;
    let mut fence_str = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if in_code {
            if is_closing_fence(line.trim(), &fence_str) {
                in_code = false;
            }
            result_lines.push(line.to_string());
            i += 1;
            continue;
        }
        if let Some(fs) = detect_opening_fence(line.trim()) {
            fence_str = fs;
            in_code = true;
            result_lines.push(line.to_string());
            i += 1;
            continue;
        }
        if FOOTNOTE_DEF_START.is_match(line) {
            // Collect definition lines: first line + indented continuation lines
            let mut def_lines = vec![line.to_string()];
            let mut j = i + 1;
            while j < lines.len() {
                let cont = lines[j];
                if cont.starts_with("  ") || cont.starts_with('\t') || cont.trim().is_empty() {
                    def_lines.push(cont.to_string());
                    j += 1;
                } else {
                    break;
                }
            }
            // Count and trim trailing blank lines from the definition block
            let mut trailing_blanks = 0;
            while def_lines.last().is_some_and(|l| l.trim().is_empty()) {
                def_lines.pop();
                trailing_blanks += 1;
            }
            // Replace with FNDEF HTML comment marker (multi-line, type-2 HTML block)
            result_lines.push(FNDEF_MARKER_START.to_string());
            for dl in &def_lines {
                result_lines.push(dl.clone());
            }
            result_lines.push("-->".to_string());
            // Re-emit trailing blank lines so spacing between definitions is preserved
            for _ in 0..trailing_blanks {
                result_lines.push(String::new());
            }
            i = j;
        } else {
            result_lines.push(line.to_string());
            i += 1;
        }
    }

    let mut output = result_lines.join("\n");
    if had_trailing_newline && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Repeatedly apply a single-match `Regex::replace` until the text stabilises.
fn replace_until_stable<F>(text: &mut String, re: &Regex, replacer: F)
where
    F: Fn(&regex::Captures) -> String,
{
    loop {
        let new = re.replace(text.as_str(), &replacer);
        if new == *text {
            break;
        }
        *text = new.into_owned();
    }
}

/// COMRAK-WORKAROUND1: Encode reference link labels in PUA markers.
/// `[text][label]` → `[text](\u{F000}label\u{F001})`. Only the label is encoded
/// (not the real URL), avoiding breakage in table cells where titles contain `|`.
/// During rendering, the PUA prefix is detected and `[text][label]` is re-emitted.
fn encode_ref_links(text: &str, labels: &HashSet<String>) -> String {
    if labels.is_empty() {
        return text.to_string();
    }

    transform_outside_code_fences(text, |line| {
        let mut result = line.to_string();
        // Replace full reference links: [text][label]
        replace_until_stable(&mut result, &FULL_REF_LINK, |caps: &regex::Captures| {
            let text_part = &caps[1];
            let label = &caps[2];
            if labels.contains(&label.to_lowercase()) {
                format!("[{text_part}]({REF_LABEL_START}{label}{REF_LABEL_SEP})")
            } else {
                caps[0].to_string()
            }
        });
        // Replace collapsed reference links: [text][]
        replace_until_stable(&mut result, &COLLAPSED_REF_LINK, |caps: &regex::Captures| {
            let text_part = &caps[1];
            let label = text_part;
            if labels.contains(&label.to_lowercase()) {
                format!("[{text_part}]({REF_LABEL_START}{label}{REF_LABEL_SEP})")
            } else {
                caps[0].to_string()
            }
        });
        vec![result]
    })
}

/// COMRAK-WORKAROUND5: Apply typography transforms (smart quotes, ellipsis) to
/// footnote definition bodies inside FNDEF HTML comment markers. These markers become
/// `HtmlBlock` nodes in the comrak AST, which the AST-level typography transforms skip.
fn apply_typography_to_fndef_bodies(text: &str, do_smartquotes: bool, do_ellipses: bool) -> String {
    let mut result = String::new();
    let mut remaining = text.as_bytes();
    let marker = FNDEF_MARKER_START.as_bytes();
    let end_marker = b"-->";

    while !remaining.is_empty() {
        if let Some(pos) = remaining.windows(marker.len()).position(|w| w == marker) {
            // Copy text before the marker
            result.push_str(&String::from_utf8_lossy(&remaining[..pos]));
            let after_marker = &remaining[pos..];
            // Find closing -->
            if let Some(end_pos) =
                after_marker.windows(end_marker.len()).position(|w| w == end_marker)
            {
                let block_end = end_pos + end_marker.len();
                let block = &String::from_utf8_lossy(&after_marker[..block_end]);
                // The block is: <!-- FNDEF\n[^label]: body text\n-->
                // Apply typography to the body (everything after the first line)
                if let Some(first_nl) = block.find('\n') {
                    let header = &block[..=first_nl];
                    let body_and_close = &block[first_nl + 1..];
                    if let Some(close_pos) = body_and_close.rfind("-->") {
                        let body = &body_and_close[..close_pos];
                        let close = &body_and_close[close_pos..];
                        let mut transformed = body.to_string();
                        if do_smartquotes {
                            transformed = smart_quotes(&transformed);
                        }
                        if do_ellipses {
                            transformed = apply_ellipses(&transformed);
                        }
                        result.push_str(header);
                        result.push_str(&transformed);
                        result.push_str(close);
                    } else {
                        result.push_str(block);
                    }
                } else {
                    result.push_str(block);
                }
                remaining = &after_marker[block_end..];
            } else {
                // No closing marker found, copy rest as-is
                result.push_str(&String::from_utf8_lossy(after_marker));
                break;
            }
        } else {
            result.push_str(&String::from_utf8_lossy(remaining));
            break;
        }
    }
    result
}

/// Pattern matching inline code spans (single or double backtick).
/// Regex for inline code spans (backtick-delimited).
static INLINE_CODE_SPAN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"``[^`]+``|`[^`]+`").expect("valid INLINE_CODE_SPAN_RE regex"));

/// COMRAK-WORKAROUND4: Replace `\x` escapes with PUA placeholders, but only outside
/// fenced code blocks AND inline code spans. Prevents comrak from stripping backslash
/// escapes during parsing, while preserving literal backslashes inside code (where they
/// are not `CommonMark` escape sequences).
fn protect_escapes_outside_code(text: &str, placeholders: &[(String, String)]) -> String {
    transform_outside_code_fences(text, |line| {
        let processed = replace_outside_code_spans(line, placeholders);
        vec![processed]
    })
}

/// COMRAK-WORKAROUND4: Apply escape replacements only to text OUTSIDE inline code spans.
fn replace_outside_code_spans(line: &str, placeholders: &[(String, String)]) -> String {
    // Temporarily hide escaped backticks so the code span regex doesn't treat
    // them as code span delimiters (e.g., `\`text\`` is not a code span).
    // We detect code spans on the modified string but apply replacements on the
    // original string using the same byte offsets (since \` and the placeholder
    // are both 2 bytes, offsets are preserved when using a 2-byte placeholder).
    let escaped_backtick_positions: Vec<usize> =
        line.match_indices("\\`").map(|(i, _)| i).collect();

    if escaped_backtick_positions.is_empty() {
        // No escaped backticks — use the fast regex path
        let mut result = String::new();
        let mut last_end = 0;
        for m in INLINE_CODE_SPAN_RE.find_iter(line) {
            let before = &line[last_end..m.start()];
            let mut processed = before.to_string();
            for (escaped, placeholder) in placeholders {
                processed = processed.replace(escaped.as_str(), placeholder.as_str());
            }
            result.push_str(&processed);
            result.push_str(m.as_str());
            last_end = m.end();
        }
        let rest = &line[last_end..];
        let mut processed = rest.to_string();
        for (escaped, placeholder) in placeholders {
            processed = processed.replace(escaped.as_str(), placeholder.as_str());
        }
        result.push_str(&processed);
        return result;
    }

    // Has escaped backticks — replace them with a same-length placeholder,
    // find code spans on the modified string, then apply replacements to
    // the original string using the same byte offsets.
    // Use two ASCII chars that won't appear in markdown for the 2-byte \`
    let modified = line.replace("\\`", "\x01\x01");

    let mut result = String::new();
    let mut last_end = 0;
    for m in INLINE_CODE_SPAN_RE.find_iter(&modified) {
        // Apply replacements to original text outside code spans
        let before = &line[last_end..m.start()];
        let mut processed = before.to_string();
        for (escaped, placeholder) in placeholders {
            processed = processed.replace(escaped.as_str(), placeholder.as_str());
        }
        result.push_str(&processed);
        // Keep code span from original unchanged
        result.push_str(&line[m.start()..m.end()]);
        last_end = m.end();
    }
    let rest = &line[last_end..];
    let mut processed = rest.to_string();
    for (escaped, placeholder) in placeholders {
        processed = processed.replace(escaped.as_str(), placeholder.as_str());
    }
    result.push_str(&processed);
    result
}

/// COMRAK-WORKAROUND11: Remove unnecessary period escapes from the formatted output.
/// Period escapes (`\.`) are only needed at the start of a line where `DIGITS\.`
/// would be interpreted as an ordered list marker. In headings and mid-paragraph,
/// period escapes are unnecessary.
/// Preserves content inside code spans (backtick-delimited) and fenced code blocks.
fn postprocess_period_escapes(text: &str) -> String {
    transform_outside_code_fences(text, |line| {
        let trimmed_start = line.trim_start();

        if trimmed_start.starts_with('#') {
            // Heading line: remove period escapes but preserve code spans
            return vec![remove_period_escapes_preserving_code(line)];
        }

        // Strip blockquote markers for content analysis
        let after_quotes =
            trimmed_start.trim_start_matches(|c: char| c == '>' || c.is_whitespace());

        // Strip unordered list markers (- , * , + ) and optional task list markers
        let after_list_marker = after_quotes
            .strip_prefix("- ")
            .or_else(|| after_quotes.strip_prefix("* "))
            .or_else(|| after_quotes.strip_prefix("+ "))
            .map_or(after_quotes, |rest| {
                // Also strip task list markers: [ ] , [x] , [X]
                rest.strip_prefix("[ ] ")
                    .or_else(|| rest.strip_prefix("[x] "))
                    .or_else(|| rest.strip_prefix("[X] "))
                    .unwrap_or(rest)
            });

        // Check if content starts with DIGITS\.
        let digit_end = after_list_marker
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after_list_marker.len());

        if digit_end > 0 && after_list_marker[digit_end..].starts_with("\\.") {
            // DIGITS\. at effective line start: keep the escape to prevent list interpretation
            vec![line.to_string()]
        } else {
            // No list-like pattern at start: remove period escapes, preserving code spans
            vec![remove_period_escapes_preserving_code(line)]
        }
    })
}

/// Remove `\.` → `.` on a single line, but preserve content inside backtick code spans.
///
/// Uses byte indexing (all relevant delimiters are ASCII) to avoid `Vec<char>` allocation.
fn remove_period_escapes_preserving_code(line: &str) -> String {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if bytes[i] == b'`' {
            // Found backtick(s) - measure opening sequence length
            let bt_count = bytes[i..].iter().take_while(|&&b| b == b'`').count();
            result.push_str(&line[i..i + bt_count]);
            i += bt_count;

            // Find matching closing backtick sequence (same length)
            while i < len {
                if bytes[i] == b'`' {
                    let close_count = bytes[i..].iter().take_while(|&&b| b == b'`').count();
                    result.push_str(&line[i..i + close_count]);
                    i += close_count;
                    if close_count == bt_count {
                        break;
                    }
                } else {
                    // Inside code span: copy literally (no escape processing).
                    // Advance one UTF-8 character at a time.
                    let ch = line[i..].chars().next().expect("valid UTF-8");
                    result.push(ch);
                    i += ch.len_utf8();
                }
            }
        } else if bytes[i] == b'\\' && i + 1 < len && bytes[i + 1] == b'.' {
            // \. outside code span → just .
            result.push('.');
            i += 2;
        } else {
            let ch = line[i..].chars().next().expect("valid UTF-8");
            result.push(ch);
            i += ch.len_utf8();
        }
    }

    result
}

/// COMRAK-WORKAROUND7: Get the last actual content line for a node, compensating
/// for comrak's tendency to include trailing blank lines in List/Item sourcepos.
/// For List nodes, recurses to the last Item's last child to find the true
/// content end line. For other nodes, uses sourcepos directly.
fn last_content_line<'a>(node: &'a AstNode<'a>) -> usize {
    let data = node.data.borrow();
    match &data.value {
        NodeValue::List(_) | NodeValue::Item(_) => {
            drop(data);
            if let Some(last_child) = node.children().last() {
                last_content_line(last_child)
            } else {
                let sp = node.data.borrow().sourcepos;
                if sp.end.line >= sp.start.line { sp.end.line } else { sp.start.line }
            }
        }
        _ => {
            let sp = data.sourcepos;
            if sp.end.line >= sp.start.line { sp.end.line } else { sp.start.line }
        }
    }
}

/// Check if a node is a block-level element that needs blank line separation.
fn is_block_element(node: &AstNode) -> bool {
    matches!(
        node.data.borrow().value,
        NodeValue::Paragraph
            | NodeValue::Heading(_)
            | NodeValue::List(_)
            | NodeValue::BlockQuote
            | NodeValue::CodeBlock(_)
            | NodeValue::ThematicBreak
            | NodeValue::HtmlBlock(_)
            | NodeValue::Table(_)
            | NodeValue::FootnoteDefinition(_)
            | NodeValue::Alert(_)
    )
}

/// Check if inline content ends with a hard break (backslash before newline).
fn inline_ends_with_hard_break<'a>(node: &'a AstNode<'a>) -> bool {
    let children: Vec<_> = node.children().collect();
    if let Some(last_child) = children.last() {
        // Check for explicit LineBreak node
        if matches!(last_child.data.borrow().value, NodeValue::LineBreak) {
            return true;
        }
        // Check if last text node ends with backslash (hard break in headings)
        if let NodeValue::Text(ref text) = last_child.data.borrow().value {
            if text.ends_with('\\') {
                return true;
            }
        }
    }
    false
}

/// COMRAK-WORKAROUND8: Check if a node is a standalone HTML comment
/// (`<!-- ... -->`). These should not force blank line separators when adjacent
/// to other blocks, matching Python's tight spacing around HTML comments.
fn is_html_comment_only(node: &AstNode) -> bool {
    if let NodeValue::HtmlBlock(html) = &node.data.borrow().value {
        let trimmed = html.literal.trim();
        trimmed.starts_with("<!--")
            && trimmed.ends_with("-->")
            && !trimmed.contains('\n')
            && !trimmed.contains(FNDEF_MARKER_START)
            && !trimmed.contains(REFDEF_MARKER_PREFIX)
    } else {
        false
    }
}

/// COMRAK-WORKAROUND1: Check if a node is a REFDEF marker (link reference
/// definition). Consecutive refdefs are grouped tightly (no blank line between
/// them). Footnote definition markers (FNDEF) are NOT included here because
/// Python separates consecutive footnote defs with blank lines.
fn is_refdef_marker(node: &AstNode) -> bool {
    if let NodeValue::HtmlBlock(html) = &node.data.borrow().value {
        html.literal.trim().starts_with(REFDEF_MARKER_PREFIX)
    } else {
        false
    }
}

/// Render block children with proper blank line separation between them.
fn render_block_children<'a>(
    node: &'a AstNode<'a>,
    line_wrapper: &LineWrapper,
    list_spacing: ListSpacing,
    prefix: &str,
    subsequent_prefix: &str,
    in_heading: &mut bool,
    options: &Options,
) -> String {
    let mut output = String::new();
    let mut prev_was_block = false;
    let mut prev_ended_with_double_newline = false;
    let mut prev_was_hard_break_heading = false;
    let mut prev_was_refdef_only = false;
    let mut prev_source_end_line: usize = 0;
    let mut prev_was_html_comment = false;
    let mut prev_was_list_or_table = false;
    let mut prev_was_paragraph = false;

    for child in node.children() {
        let child_is_block = is_block_element(child);
        let child_is_refdef_only = is_refdef_marker(child);
        let child_is_html_comment = is_html_comment_only(child);
        let child_is_list = matches!(child.data.borrow().value, NodeValue::List(_));

        // Check if current child is a hard-break heading
        let child_is_hard_break_heading =
            matches!(child.data.borrow().value, NodeValue::Heading(_))
                && inline_ends_with_hard_break(child);

        // COMRAK-WORKAROUND7: Use source positions to detect whether blocks were
        // originally separated by a blank line. Uses last_content_line() to get the
        // true end of content (compensating for comrak's List/Item nodes including
        // trailing blank lines and HtmlBlock type 2 reporting end.line < start.line).
        let child_source_start = child.data.borrow().sourcepos.start.line;
        let child_source_end = last_content_line(child);
        let originally_tight =
            prev_source_end_line > 0 && child_source_start <= prev_source_end_line + 1;

        // COMRAK-WORKAROUND8: Suppress blank line separator between blocks for
        // specific tight transitions matching Python/marko behavior:
        //
        // Rule 1: HTML comment → any block (tight): suppress separator
        // Rule 2: Any block → HTML comment (tight): suppress, UNLESS prev is
        //         list/table (lists/tables always get a blank line before a
        //         following HTML comment)
        // Rule 3: Paragraph → list (tight): suppress separator
        //
        // All other block pairs get the standard blank line separator.
        let suppress_for_tight = if originally_tight {
            if prev_was_html_comment {
                // Rule 1: HTML comment → any block (tight): suppress
                true
            } else if child_is_html_comment {
                // Rule 2: Any block → HTML comment (tight): suppress,
                // UNLESS prev is list or table (GAP13)
                !prev_was_list_or_table
            } else if child_is_list && prev_was_paragraph {
                // Rule 3: Paragraph → list (tight): suppress (GAP11)
                // This handles cases like "**Header**:\n- item1\n- item2"
                true
            } else {
                false
            }
        } else {
            false
        };

        // Add blank line between consecutive block elements,
        // unless adjacent to a heading ending with a hard break,
        // or between consecutive REFDEF markers (link reference defs are grouped tightly),
        // or tight transition matching Python behavior (HTML comments, paragraph→list).
        // Note: footnote defs DO get blank lines between them (matching Python).
        let both_refdefs = prev_was_refdef_only && child_is_refdef_only;
        let need_separator = child_is_block
            && prev_was_block
            && !prev_ended_with_double_newline
            && !prev_was_hard_break_heading
            && !child_is_hard_break_heading
            && !both_refdefs
            && !suppress_for_tight;
        if need_separator {
            output.push('\n');
        }

        let block_output = render_block(
            child,
            line_wrapper,
            list_spacing,
            prefix,
            subsequent_prefix,
            in_heading,
            options,
        );
        prev_ended_with_double_newline = block_output.ends_with("\n\n");
        prev_was_hard_break_heading = matches!(child.data.borrow().value, NodeValue::Heading(_))
            && inline_ends_with_hard_break(child);
        output.push_str(&block_output);
        prev_was_block = child_is_block;
        prev_was_refdef_only = child_is_refdef_only;
        prev_was_html_comment = child_is_html_comment;
        prev_was_list_or_table =
            matches!(child.data.borrow().value, NodeValue::List(_) | NodeValue::Table(_));
        prev_was_paragraph = matches!(child.data.borrow().value, NodeValue::Paragraph);
        prev_source_end_line = child_source_end;
    }

    output
}

/// Render block children within a quoted context (blockquote or alert).
/// Uses `blank_prefix` (e.g., ">") for blank separator lines between blocks.
#[allow(clippy::too_many_arguments)]
fn render_block_children_quoted<'a>(
    node: &'a AstNode<'a>,
    line_wrapper: &LineWrapper,
    list_spacing: ListSpacing,
    prefix: &str,
    subsequent_prefix: &str,
    blank_prefix: &str,
    in_heading: &mut bool,
    options: &Options,
) -> String {
    let mut output = String::new();
    let mut prev_was_block = false;
    let mut prev_ended_with_double_newline = false;

    for child in node.children() {
        let child_is_block = is_block_element(child);

        // Add blank line between consecutive block elements
        // Use the blank_prefix (e.g., "> ") to maintain the quote context
        if child_is_block && prev_was_block && !prev_ended_with_double_newline {
            output.push_str(blank_prefix);
            output.push_str(" \n");
        }

        let block_output = render_block(
            child,
            line_wrapper,
            list_spacing,
            prefix,
            subsequent_prefix,
            in_heading,
            options,
        );
        prev_ended_with_double_newline = block_output.ends_with("\n\n");
        output.push_str(&block_output);
        prev_was_block = child_is_block;
    }

    output
}

/// Render a single block-level node.
fn render_block<'a>(
    node: &'a AstNode<'a>,
    line_wrapper: &LineWrapper,
    list_spacing: ListSpacing,
    prefix: &str,
    subsequent_prefix: &str,
    in_heading: &mut bool,
    options: &Options,
) -> String {
    let mut output = String::new();

    match &node.data.borrow().value {
        NodeValue::Document => {
            output = render_block_children(
                node,
                line_wrapper,
                list_spacing,
                prefix,
                subsequent_prefix,
                in_heading,
                options,
            );
        }

        NodeValue::Paragraph => {
            // Collect all inline content
            let inline_text = render_inline_children(node, options, *in_heading);

            // Handle GFM tasklist checkbox
            let inline_text = if let Some(tasklist) = get_tasklist_marker(node) {
                format!("{tasklist}{inline_text}")
            } else {
                inline_text
            };

            // Wrap the text
            let wrapped = line_wrapper(&inline_text, prefix, subsequent_prefix);
            output.push_str(&wrapped);
            output.push('\n');
        }

        NodeValue::Heading(heading) => {
            *in_heading = true;
            let level = heading.level;
            let hashes = "#".repeat(level as usize);

            let inline_text = render_inline_children(node, options, true);
            *in_heading = false;

            // Check if heading ends with a hard break (either LineBreak node or trailing backslash)
            let ends_with_hard_break =
                inline_ends_with_hard_break(node) || inline_text.ends_with('\\');

            let _ = writeln!(output, "{prefix}{hashes} {inline_text}");
            if !ends_with_hard_break {
                output.push('\n');
            }
        }

        NodeValue::List(list) => {
            // Determine effective tightness
            let is_tight = match list_spacing {
                ListSpacing::Preserve => list.tight,
                ListSpacing::Tight => can_be_tight(node),
                ListSpacing::Loose => false,
            };

            let is_ordered = matches!(list.list_type, ListType::Ordered);
            let start = list.start;
            let bullet = list.bullet_char;

            for (i, child) in node.children().enumerate() {
                let (item_prefix, item_subsequent) = if is_ordered {
                    let num = start + i;
                    let p = format!("{num}. ");
                    let s = " ".repeat(num.to_string().len() + 2);
                    (format!("{prefix}{p}"), format!("{subsequent_prefix}{s}"))
                } else {
                    let marker = bullet as char;
                    (format!("{prefix}{marker} "), format!("{subsequent_prefix}  "))
                };

                // For loose lists, add blank line between items (except before first)
                // Use the outer prefix (without list indentation) for the blank line
                // to maintain blockquote context
                if !is_tight && i > 0 {
                    let blank_prefix = subsequent_prefix.trim_end();
                    if blank_prefix.is_empty() {
                        output.push('\n');
                    } else {
                        output.push_str(blank_prefix);
                        output.push('\n');
                    }
                }

                render_list_item(
                    child,
                    &mut output,
                    line_wrapper,
                    list_spacing,
                    &item_prefix,
                    &item_subsequent,
                    in_heading,
                    options,
                );
            }
        }

        NodeValue::BlockQuote => {
            let q_prefix = format!("{prefix}> ");
            let q_subsequent = format!("{subsequent_prefix}> ");

            let inner = render_block_children_quoted(
                node,
                line_wrapper,
                list_spacing,
                &q_prefix,
                &q_subsequent,
                &format!("{subsequent_prefix}>"),
                in_heading,
                options,
            );

            // Trim trailing newlines and re-add single newline
            output.push_str(inner.trim_end_matches('\n'));
            output.push('\n');
        }

        NodeValue::CodeBlock(code_block) => {
            let info = &code_block.info;
            let literal = &code_block.literal;
            let code_content = literal.trim_end_matches('\n');

            let fence_char = if code_block.fenced {
                if code_block.fence_char == b'~' { '~' } else { '`' }
            } else {
                '`'
            };

            // Calculate minimum fence length needed
            let fence_len = min_fence_length(code_content, fence_char).max(if code_block.fenced {
                code_block.fence_length
            } else {
                3
            });
            let fence: String = std::iter::repeat_n(fence_char, fence_len).collect();

            let _ = writeln!(output, "{prefix}{fence}{info}");
            let empty_prefix = subsequent_prefix.trim_end();
            for line in code_content.split('\n') {
                if line.is_empty() {
                    output.push_str(empty_prefix);
                    output.push('\n');
                } else {
                    let _ = writeln!(output, "{subsequent_prefix}{line}");
                }
            }
            let _ = writeln!(output, "{subsequent_prefix}{fence}");
        }

        NodeValue::ThematicBreak => {
            let _ = writeln!(output, "{prefix}* * *");
        }

        NodeValue::HtmlBlock(html) => {
            let literal = &html.literal;
            let trimmed = literal.trim();

            // COMRAK-WORKAROUND1: Re-emit reference definition from REFDEF marker.
            if let Some(rest) = trimmed.strip_prefix(REFDEF_MARKER_PREFIX) {
                if let Some(def_text) = rest.strip_suffix("-->") {
                    let def_text = def_text.trim();
                    let _ = writeln!(output, "{prefix}{def_text}");
                    return output;
                }
            }

            // COMRAK-WORKAROUND2 + COMRAK-WORKAROUND9: Re-emit footnote definition
            // from FNDEF marker, with list item detection for proper indentation.
            if trimmed.starts_with(FNDEF_MARKER_START) {
                // Extract content between first line and closing -->
                if let Some(first_nl) = literal.find('\n') {
                    let rest = &literal[first_nl + 1..];
                    if let Some(end_pos) = rest.rfind("-->") {
                        let fn_text = rest[..end_pos].trim_end();
                        // Format the footnote definition with line wrapping.
                        // Parse [^label]: from the first line to get prefix widths.
                        if let Some(caps) = FOOTNOTE_DEF_START.captures(fn_text) {
                            let label = caps[1].to_string();
                            let match_end = caps.get(0).map_or(0, |m| m.end());
                            let label_prefix = format!("[^{label}]: ");
                            let fn_prefix = format!("{prefix}{label_prefix}");
                            let fn_subsequent = format!("{prefix}    ");

                            // Extract body: first line after `[^label]: `, plus
                            // continuation lines (stripped of 4-space indent).
                            // Preserve paragraph structure for multi-paragraph footnotes.
                            let mut body_lines: Vec<&str> = Vec::new();
                            for (li, line) in fn_text.lines().enumerate() {
                                if li == 0 {
                                    body_lines.push(&line[match_end..]);
                                } else {
                                    let stripped = line
                                        .strip_prefix("    ")
                                        .or_else(|| line.strip_prefix('\t'))
                                        .unwrap_or(line);
                                    body_lines.push(stripped);
                                }
                            }

                            // Check if this is a multi-paragraph footnote (contains blank lines)
                            let has_blank_lines =
                                body_lines.iter().skip(1).any(|l| l.trim().is_empty());
                            if has_blank_lines {
                                // Multi-paragraph footnote: split into paragraphs and wrap each.
                                let mut paragraphs: Vec<Vec<&str>> = vec![Vec::new()];
                                for line in &body_lines {
                                    if line.trim().is_empty() {
                                        if !paragraphs
                                            .last()
                                            .expect("paragraphs is non-empty")
                                            .is_empty()
                                        {
                                            paragraphs.push(Vec::new());
                                        }
                                    } else {
                                        paragraphs
                                            .last_mut()
                                            .expect("paragraphs is non-empty")
                                            .push(line);
                                    }
                                }
                                if paragraphs.last().is_some_and(Vec::is_empty) {
                                    paragraphs.pop();
                                }
                                for (pi, para) in paragraphs.iter().enumerate() {
                                    // Detect blockquote paragraphs: lines starting with >
                                    let is_blockquote = para.iter().all(|l| l.starts_with('>'));
                                    if is_blockquote {
                                        // Strip > prefix, join, wrap with blockquote prefix
                                        let bq_body: Vec<&str> = para
                                            .iter()
                                            .map(|l| {
                                                l.strip_prefix("> ")
                                                    .unwrap_or(l.strip_prefix('>').unwrap_or(l))
                                            })
                                            .collect();
                                        let joined = bq_body.join(" ");
                                        let bq_prefix = if pi == 0 {
                                            format!("{fn_prefix}> ")
                                        } else {
                                            format!("{fn_subsequent}> ")
                                        };
                                        let bq_subsequent = format!("{fn_subsequent}> ");
                                        let wrapped =
                                            line_wrapper(joined.trim(), &bq_prefix, &bq_subsequent);
                                        output.push_str(&wrapped);
                                    } else {
                                        let joined = para.join(" ");
                                        let (p, sp) = if pi == 0 {
                                            (fn_prefix.clone(), fn_subsequent.clone())
                                        } else {
                                            (fn_subsequent.clone(), fn_subsequent.clone())
                                        };
                                        let wrapped = line_wrapper(joined.trim(), &p, &sp);
                                        output.push_str(&wrapped);
                                    }
                                    output.push_str("\n\n");
                                }
                            } else {
                                // Single-paragraph footnote.
                                // COMRAK-WORKAROUND9: Detect embedded list items
                                // (lines starting with `- `, `* `, or `+ `).
                                // Python/marko treats these as list blocks, rendering
                                // continuation lines with 2 extra spaces of indent.
                                let list_start_idx = body_lines.iter().skip(1).position(|l| {
                                    l.starts_with("- ")
                                        || l.starts_with("* ")
                                        || l.starts_with("+ ")
                                });
                                if let Some(idx) = list_start_idx {
                                    let idx = idx + 1; // adjust for skip(1)
                                    // Preamble paragraph before the list
                                    let preamble = body_lines[..idx].join(" ");
                                    let wrapped =
                                        line_wrapper(preamble.trim(), &fn_prefix, &fn_subsequent);
                                    output.push_str(&wrapped);
                                    output.push('\n');
                                    // List items: join from the `- ` line through the
                                    // rest, treating as one list item with 6-space
                                    // continuation indent (4 footnote + 2 list item).
                                    let marker = &body_lines[idx][..2]; // "- " etc.
                                    let item_text = &body_lines[idx][2..]; // after marker
                                    let rest: Vec<&str> = body_lines[idx + 1..].to_vec();
                                    let mut full_text = item_text.to_string();
                                    for line in &rest {
                                        full_text.push(' ');
                                        full_text.push_str(line);
                                    }
                                    let list_prefix = format!("{fn_subsequent}{marker}");
                                    let list_subsequent = format!("{fn_subsequent}  ");
                                    let wrapped = line_wrapper(
                                        full_text.trim(),
                                        &list_prefix,
                                        &list_subsequent,
                                    );
                                    output.push_str(&wrapped);
                                    output.push_str("\n\n");
                                } else {
                                    let body = body_lines.join(" ");
                                    let wrapped =
                                        line_wrapper(body.trim(), &fn_prefix, &fn_subsequent);
                                    output.push_str(&wrapped);
                                    // Footnote definitions end with a blank line (matching Python behavior)
                                    output.push_str("\n\n");
                                }
                            }
                        } else {
                            // Fallback: output content lines as-is
                            for line in fn_text.lines() {
                                let _ = writeln!(output, "{prefix}{line}");
                            }
                        }
                        return output;
                    }
                }
            }

            // Check if this HTML block has wrappable text content
            // (e.g., HTML comments/tags mixed with regular text)
            let has_text_content = !trimmed.is_empty()
                && trimmed.contains(|c: char| c.is_alphabetic())
                && trimmed.chars().filter(|&c| c == '<').count() > 0;

            if has_text_content && trimmed.len() > 40 {
                // Collapse internal whitespace and wrap as text
                // Join all lines into a single line first
                let single_line: String =
                    literal.lines().map(str::trim).collect::<Vec<_>>().join(" ").trim().to_string();
                let wrapped = line_wrapper(&single_line, prefix, subsequent_prefix);
                output.push_str(&wrapped);
                output.push('\n');
            } else {
                // Short or non-wrappable HTML: pass through as-is
                output.push_str(prefix);
                output.push_str(literal);
                if !literal.ends_with('\n') {
                    output.push('\n');
                }
            }
        }

        NodeValue::Table(_) => {
            let children: Vec<_> = node.children().collect();
            if children.is_empty() {
                return output;
            }

            // Render header row
            let header = &children[0];
            output.push_str(&render_table_row(header, options));

            // Render delimiter row
            let alignments = get_table_alignments(node);
            let delimiters: Vec<String> = alignments
                .iter()
                .map(|a| match a {
                    TableAlignment::None => "---".to_string(),
                    TableAlignment::Left => ":---".to_string(),
                    TableAlignment::Center => ":---:".to_string(),
                    TableAlignment::Right => "---:".to_string(),
                })
                .collect();
            let _ = writeln!(output, "| {} |", delimiters.join(" | "));

            // Render body rows
            for child in children.iter().skip(1) {
                output.push_str(&render_table_row(child, options));
            }
        }

        NodeValue::TableRow(_) | NodeValue::TableCell => {
            // Handled by render_table_row
        }

        NodeValue::FootnoteDefinition(footnote) => {
            let label = &footnote.name;
            let label_prefix = format!("[^{label}]: ");
            let fn_prefix = format!("{prefix}{label_prefix}");
            let fn_subsequent = format!("{subsequent_prefix}    ");

            let mut first_child = true;
            for child in node.children() {
                let (p, sp) = if first_child {
                    (fn_prefix.clone(), fn_subsequent.clone())
                } else {
                    (fn_subsequent.clone(), fn_subsequent.clone())
                };
                let child_output =
                    render_block(child, line_wrapper, list_spacing, &p, &sp, in_heading, options);
                output.push_str(&child_output);
                first_child = false;
            }

            // Ensure proper ending
            if !output.ends_with("\n\n") {
                if output.ends_with('\n') {
                    output.push('\n');
                } else {
                    output.push_str("\n\n");
                }
            }
        }

        NodeValue::Alert(alert) => {
            let alert_type = format!("{:?}", alert.alert_type).to_uppercase();
            let _ = writeln!(output, "> [!{alert_type}]");

            let q_prefix = format!("{prefix}> ");
            let q_subsequent = format!("{subsequent_prefix}> ");

            let inner = render_block_children_quoted(
                node,
                line_wrapper,
                list_spacing,
                &q_prefix,
                &q_subsequent,
                &format!("{subsequent_prefix}>"),
                in_heading,
                options,
            );

            output.push_str(inner.trim_end_matches('\n'));
            output.push('\n');
        }

        // Inline elements and other node types
        _ => {
            for child in node.children() {
                output.push_str(&render_block(
                    child,
                    line_wrapper,
                    list_spacing,
                    prefix,
                    subsequent_prefix,
                    in_heading,
                    options,
                ));
            }
        }
    }

    output
}

/// Check if a list item needs blank lines between its children.
/// This is true when:
/// - The item's parent list is loose (not tight)
/// - OR the item has multiple block children that aren't just para+nested-list
fn item_needs_child_spacing<'a>(node: &'a AstNode<'a>, parent_is_tight: bool) -> bool {
    if !parent_is_tight {
        // For loose lists, always add blank lines between children
        return true;
    }
    let children: Vec<_> = node.children().collect();
    if children.len() <= 1 {
        return false;
    }
    // For tight lists, only add spacing if there are multiple paragraphs
    // (not counting para + nested list as needing spacing)
    let para_count =
        children.iter().filter(|c| matches!(c.data.borrow().value, NodeValue::Paragraph)).count();
    para_count > 1
}

/// Render a list item's children.
#[allow(clippy::too_many_arguments)]
fn render_list_item<'a>(
    node: &'a AstNode<'a>,
    output: &mut String,
    line_wrapper: &LineWrapper,
    list_spacing: ListSpacing,
    item_prefix: &str,
    item_subsequent: &str,
    in_heading: &mut bool,
    options: &Options,
) {
    let mut first_child = true;
    let children: Vec<_> = node.children().collect();

    // Check if parent list is tight by looking at the list spacing context
    // We determine this by checking if the parent list node is tight
    let parent_is_tight = node.parent().is_some_and(|parent| {
        if let NodeValue::List(list) = &parent.data.borrow().value {
            match list_spacing {
                ListSpacing::Preserve => list.tight,
                ListSpacing::Tight => can_be_tight(parent),
                ListSpacing::Loose => false,
            }
        } else {
            false
        }
    });

    let needs_spacing = item_needs_child_spacing(node, parent_is_tight);

    for (i, child) in children.iter().enumerate() {
        let (p, sp) = if first_child {
            (item_prefix.to_string(), item_subsequent.to_string())
        } else {
            (item_subsequent.to_string(), item_subsequent.to_string())
        };

        // Add blank line between children in a list item
        if !first_child && needs_spacing {
            // Check if previous child was heading that ends with double newline
            let prev_ended_double = if i > 0 {
                matches!(children[i - 1].data.borrow().value, NodeValue::Heading(_))
            } else {
                false
            };

            // Don't add blank line before a heading that ends with hard break
            // (it connects tightly to the following content)
            let current_is_hard_break_heading =
                matches!(&child.data.borrow().value, NodeValue::Heading(_))
                    && inline_ends_with_hard_break(child);

            // Don't add blank line before a short tag-only HTML block
            // (e.g., <!-- comment --> on a continuation line in a list item)
            let current_is_tag_block =
                if let NodeValue::HtmlBlock(html) = &child.data.borrow().value {
                    let trimmed = html.literal.trim();
                    !trimmed.contains('\n')
                        && ((trimmed.starts_with("<!--") && trimmed.ends_with("-->"))
                            || (trimmed.starts_with("{%") && trimmed.ends_with("%}"))
                            || (trimmed.starts_with("{#") && trimmed.ends_with("#}"))
                            || (trimmed.starts_with("{{") && trimmed.ends_with("}}")))
                } else {
                    false
                };

            // COMRAK-WORKAROUND10: In Preserve mode, don't add a blank line before
            // a nested list unless the original source had one. Comrak marks
            // the whole parent list as loose when *any* sibling pair has a blank
            // line, which would insert blanks inside every item. Python/marko
            // only inserts the blank when the author actually wrote one.
            let suppress_nested_blank = if matches!(child.data.borrow().value, NodeValue::List(_))
                && !parent_is_tight
                && list_spacing == ListSpacing::Preserve
                && i > 0
            {
                let prev_end = children[i - 1].data.borrow().sourcepos.end.line;
                let curr_start = child.data.borrow().sourcepos.start.line;
                // No blank line in original source → suppress
                curr_start <= prev_end + 1
            } else {
                false
            };

            if !prev_ended_double
                && !current_is_hard_break_heading
                && !current_is_tag_block
                && !suppress_nested_blank
            {
                output.push('\n');
            }
        }

        let child_output =
            render_block(child, line_wrapper, list_spacing, &p, &sp, in_heading, options);
        output.push_str(&child_output);
        first_child = false;
    }
}

/// Render inline children of a node to a flat string.
fn render_inline_children<'a>(
    node: &'a AstNode<'a>,
    options: &Options,
    in_heading: bool,
) -> String {
    let mut output = String::new();
    for child in node.children() {
        output.push_str(&render_inline(child, options, in_heading));
    }
    output
}

/// Check if a Link node is an autolink (inner text matches URL).
/// Autolinks are created by comrak for `<url>`, `<email>`, and bare URLs.
fn is_autolink(node: &AstNode, link: &comrak::nodes::NodeLink) -> bool {
    // Must have exactly one child that is a Text node
    let Some(first_child) = node.first_child() else {
        return false;
    };
    if first_child.next_sibling().is_some() {
        return false;
    }
    let text = match &first_child.data.borrow().value {
        NodeValue::Text(t) => t.clone(),
        _ => return false,
    };
    // Inner text matches URL (autolink) or URL minus "mailto:" (email autolink)
    let url = &link.url;
    text == *url || url.strip_prefix("mailto:").is_some_and(|stripped| text == stripped)
}

/// Render a single inline node to string.
fn render_inline<'a>(node: &'a AstNode<'a>, options: &Options, in_heading: bool) -> String {
    match &node.data.borrow().value {
        NodeValue::Text(text) => text.clone(),

        NodeValue::Code(code) => {
            let text = &code.literal;
            if text.starts_with('`') || text.ends_with('`') {
                format!("`` {text} ``")
            } else {
                format!("`{text}`")
            }
        }

        NodeValue::Emph => {
            let inner = render_inline_children(node, options, in_heading);
            format!("*{inner}*")
        }

        NodeValue::Strong => {
            let inner = render_inline_children(node, options, in_heading);
            format!("**{inner}**")
        }

        NodeValue::Strikethrough => {
            let inner = render_inline_children(node, options, in_heading);
            format!("~~{inner}~~")
        }

        NodeValue::Link(link) => {
            let inner = render_inline_children(node, options, in_heading);
            // COMRAK-WORKAROUND1: Detect PUA-encoded reference link.
            if link.url.starts_with(REF_LABEL_START) {
                if let Some(sep_pos) = link.url.find(REF_LABEL_SEP) {
                    let label = &link.url[REF_LABEL_START.len_utf8()..sep_pos];
                    format!("[{inner}][{label}]")
                } else {
                    // Malformed PUA marker — strip it and render as inline
                    let url = &link.url[REF_LABEL_START.len_utf8()..];
                    let title = if link.title.is_empty() {
                        String::new()
                    } else {
                        format!(" \"{}\"", link.title.replace('"', "\\\""))
                    };
                    format!("[{inner}]({url}{title})")
                }
            } else if link.title.is_empty() && is_autolink(node, link) {
                // COMRAK-WORKAROUND3: Autolink rendering — inner text matches URL,
                // render as bare text. Angle-bracket autolinks were protected by
                // PUA markers and are restored during postprocessing.
                inner.clone()
            } else {
                let title = if link.title.is_empty() {
                    String::new()
                } else {
                    format!(" \"{}\"", link.title.replace('"', "\\\""))
                };
                format!("[{inner}]({}{})", link.url, title)
            }
        }

        NodeValue::Image(image) => {
            let inner = render_inline_children(node, options, in_heading);
            let title = if image.title.is_empty() {
                String::new()
            } else {
                format!(" \"{}\"", image.title.replace('"', "\\\""))
            };
            format!("![{inner}]({}{})", image.url, title)
        }

        NodeValue::HtmlInline(html) => html.clone(),

        NodeValue::SoftBreak => "\n".to_string(),

        NodeValue::LineBreak => "\\\n".to_string(),

        NodeValue::Escaped => {
            // Escaped character - the children will contain the character.
            // Most escapes are handled via placeholders (pre-processing), but
            // comrak may still create Escaped nodes for some characters.
            let inner = render_inline_children(node, options, in_heading);
            format!("\\{inner}")
        }

        NodeValue::FootnoteReference(fr) => {
            format!("[^{}]", fr.name)
        }

        NodeValue::Math(math) => {
            if math.display_math {
                format!("$${}$$", math.literal)
            } else {
                format!("${}$", math.literal)
            }
        }

        NodeValue::WikiLink(wl) => {
            format!("[[{}]]", wl.url)
        }

        _ => {
            // Fallback: render children
            render_inline_children(node, options, in_heading)
        }
    }
}

/// Check if a list can be rendered tight.
fn can_be_tight<'a>(list_node: &'a AstNode<'a>) -> bool {
    for item in list_node.children() {
        if !matches!(item.data.borrow().value, NodeValue::Item(_)) {
            continue;
        }
        // If the item has more than one child, it must be loose
        if item.children().count() > 1 {
            return false;
        }
    }
    true
}

/// Get tasklist marker for a paragraph if its parent is a tasklist item.
fn get_tasklist_marker<'a>(para_node: &'a AstNode<'a>) -> Option<String> {
    if let Some(parent) = para_node.parent() {
        if let NodeValue::TaskItem(checked) = &parent.data.borrow().value {
            let marker = if checked.is_some() { "[x] " } else { "[ ] " };
            // Only add marker to first paragraph in the item
            if parent.children().next().is_some_and(|c| std::ptr::eq(c, para_node)) {
                return Some(marker.to_string());
            }
        }
    }
    None
}

/// Get table column alignments.
fn get_table_alignments<'a>(table_node: &'a AstNode<'a>) -> Vec<TableAlignment> {
    if let NodeValue::Table(table) = &table_node.data.borrow().value {
        table.alignments.clone()
    } else {
        vec![]
    }
}

/// Render a table row.
fn render_table_row<'a>(row_node: &'a AstNode<'a>, options: &Options) -> String {
    let cells: Vec<String> = row_node
        .children()
        .map(|cell| {
            let content = render_inline_children(cell, options, false);
            content.replace('|', "\\|")
        })
        .collect();
    format!("| {} |\n", cells.join(" | "))
}

/// Pattern matching backtick fence runs in code content.
static BACKTICK_FENCE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ ]{0,3}(`{3,})").expect("valid backtick fence regex"));

/// Pattern matching tilde fence runs in code content.
static TILDE_FENCE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ ]{0,3}(~{3,})").expect("valid tilde fence regex"));

/// Calculate minimum fence length needed for code content.
fn min_fence_length(code_content: &str, fence_char: char) -> usize {
    let re = match fence_char {
        '`' => &*BACKTICK_FENCE_RE,
        '~' => &*TILDE_FENCE_RE,
        _ => return 3,
    };
    let max_len = re
        .captures_iter(code_content)
        .map(|caps| caps.get(1).expect("capture group 1 always exists").as_str().len())
        .max()
        .unwrap_or(0);
    std::cmp::max(3, max_len + 1)
}

/// Normalize and wrap Markdown text filling paragraphs to the full width.
///
/// This is the main entry point for Markdown formatting.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub fn fill_markdown(
    markdown_text: &str,
    dedent_input: bool,
    width: usize,
    semantic: bool,
    cleanups: bool,
    smartquotes: bool,
    ellipses: bool,
    line_wrapper: Option<LineWrapper>,
    list_spacing: ListSpacing,
) -> String {
    // COMRAK-WORKAROUND4: Escaped characters to protect from comrak stripping.
    // Comrak strips backslash escapes (e.g., \~ → ~, \* → *) in the AST.
    // IMPORTANT: \\ must be first so \\X doesn't get partially matched as \X.
    // All 32 CommonMark-escapable ASCII punctuation characters.
    // See https://spec.commonmark.org/0.31.2/#backslash-escapes
    const ESCAPE_CHARS: &[char] = &[
        '\\', '~', '*', '#', '-', '+', '>', '.', '!', '[', ']', '(', ')', '{', '}', '$', '_', '|',
        '`', '"', '%', '&', '\'', ',', '/', ':', ';', '<', '=', '?', '@', '^',
    ];

    let line_wrapper = line_wrapper.unwrap_or_else(|| {
        if semantic {
            line_wrap_by_sentence(width, DEFAULT_MIN_LINE_LEN, true)
        } else {
            line_wrap_to_width(width, true)
        }
    });

    // Extract frontmatter before any processing
    let (frontmatter, content) = split_frontmatter(markdown_text);

    let mut text = if frontmatter.is_empty() { markdown_text.to_string() } else { content };

    if dedent_input {
        text = dedent(&text);
    }

    text = text.trim().to_string();
    text.push('\n');

    // === Pre-parse workarounds (see module-level COMRAK-WORKAROUND docs) ===

    // COMRAK-WORKAROUND6: Ensure proper blank lines around block content within tags.
    text = preprocess_tag_block_spacing(&text);

    // COMRAK-WORKAROUND1: Extract link reference definitions and encode reference
    // links with PUA markers. Must happen before escape placeholder substitution,
    // which would mangle `\[` etc.
    let (ref_labels, text_without_defs) = extract_link_ref_defs(&text);
    text = encode_ref_links(&text_without_defs, &ref_labels);

    // COMRAK-WORKAROUND2: Extract footnote definitions and replace with FNDEF
    // HTML comment markers (preserved as HtmlBlock nodes at original positions).
    text = extract_footnote_defs(&text);

    // COMRAK-WORKAROUND5: Apply typography transforms to footnote definition bodies
    // inside FNDEF markers (which become HtmlBlock nodes that AST transforms skip).
    if smartquotes || ellipses {
        text = apply_typography_to_fndef_bodies(&text, smartquotes, ellipses);
    }

    // COMRAK-WORKAROUND3: Protect angle-bracket autolinks from comrak parsing.
    text = protect_autolinks(&text);

    // COMRAK-WORKAROUND4: Replace `\x` escape sequences with PUA placeholders.
    let mut escape_placeholders: Vec<(String, String)> = Vec::new();
    for &ch in ESCAPE_CHARS {
        let escaped = format!("\\{ch}");
        let placeholder =
            char::from_u32(0xE000 + ch as u32).expect("valid PUA code point").to_string();
        escape_placeholders.push((escaped, placeholder));
    }
    text = protect_escapes_outside_code(&text, &escape_placeholders);

    // === Parse with comrak ===
    let arena = Arena::new();
    let options = flowmark_comrak_options();
    let root = comrak::parse_document(&arena, &text, &options);

    // === AST transforms (not comrak workarounds) ===
    if cleanups {
        doc_cleanups(root);
    }
    if smartquotes {
        apply_smart_quotes_to_ast(root);
    }
    if ellipses {
        apply_ellipses_to_ast(root);
    }

    // === Render AST to markdown ===
    // COMRAK-WORKAROUND1/2/3/7/8/9/10 all apply during rendering (see render_block).
    let mut in_heading = false;
    let result = render_block(root, &line_wrapper, list_spacing, "", "", &mut in_heading, &options);

    // === Post-render workarounds ===

    // COMRAK-WORKAROUND4: Restore escaped characters from PUA placeholders.
    let mut result = result;
    for (escaped, placeholder) in &escape_placeholders {
        result = result.replace(placeholder.as_str(), escaped.as_str());
    }

    // COMRAK-WORKAROUND11: Remove unnecessary period escapes.
    let result = postprocess_period_escapes(&result);

    // COMRAK-WORKAROUND3: Restore autolink angle brackets from PUA placeholders.
    let result = restore_autolinks(&result);

    // COMRAK-WORKAROUND12: Normalize comrak output formatting differences.
    let result = normalize_comrak_output(&result);

    // Reattach frontmatter if present
    if frontmatter.is_empty() { result } else { format!("{frontmatter}{result}") }
}

/// Apply smart quotes to all text nodes in the AST.
/// Works at the paragraph level so quotes spanning inline elements are handled.
fn apply_smart_quotes_to_ast<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let is_para = matches!(
            node.data.borrow().value,
            NodeValue::Paragraph | NodeValue::Heading(_) | NodeValue::TableCell
        );
        if is_para {
            apply_smart_quotes_to_inline_tree(node);
        }
    }
}

/// Collect text nodes from inline tree, apply smart quotes to concatenated text,
/// then redistribute back.
#[allow(clippy::items_after_statements)]
fn apply_smart_quotes_to_inline_tree<'a>(node: &'a AstNode<'a>) {
    // Collect all text nodes with their content
    let mut text_nodes: Vec<&'a AstNode<'a>> = Vec::new();
    let mut concatenated = String::new();
    let mut char_boundaries: Vec<(usize, usize)> = Vec::new(); // (start, len) in chars

    fn collect_text_nodes<'a>(
        node: &'a AstNode<'a>,
        text_nodes: &mut Vec<&'a AstNode<'a>>,
        concatenated: &mut String,
        char_boundaries: &mut Vec<(usize, usize)>,
    ) {
        for child in node.children() {
            let data = child.data.borrow();
            match &data.value {
                NodeValue::Text(text) => {
                    let start = concatenated.chars().count();
                    let len = text.chars().count();
                    concatenated.push_str(text);
                    char_boundaries.push((start, len));
                    text_nodes.push(child);
                }
                NodeValue::Code(_) | NodeValue::HtmlInline(_) => {
                    // Skip code spans and raw HTML - don't apply smart quotes
                    // But add placeholder chars to maintain context
                    concatenated.push('X'); // placeholder for quote context
                }
                NodeValue::SoftBreak => {
                    concatenated.push(' ');
                }
                _ => {
                    // Recurse into emphasis, strong, link, etc.
                    drop(data);
                    collect_text_nodes(child, text_nodes, concatenated, char_boundaries);
                }
            }
        }
    }

    collect_text_nodes(node, &mut text_nodes, &mut concatenated, &mut char_boundaries);

    if text_nodes.is_empty() {
        return;
    }

    // Apply smart quotes to the full concatenated text
    let converted = smart_quotes(&concatenated);

    // Redistribute characters back to text nodes
    let converted_chars: Vec<char> = converted.chars().collect();
    for (i, text_node) in text_nodes.iter().enumerate() {
        let (start, len) = char_boundaries[i];
        if start + len <= converted_chars.len() {
            let new_text: String = converted_chars[start..start + len].iter().collect();
            let mut data = text_node.data.borrow_mut();
            if let NodeValue::Text(ref mut text) = data.value {
                *text = new_text;
            }
        }
    }
}

/// Apply ellipsis conversion to all text nodes in the AST.
fn apply_ellipses_to_ast<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Text(ref mut text) = data.value {
            *text = apply_ellipses(text);
        }
    }
}

/// Simple dedent: remove common leading whitespace from all lines.
fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return text.to_string();
    }

    // Find minimum indentation (ignoring empty lines)
    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return text.to_string();
    }

    lines
        .iter()
        .map(|l| if l.len() >= min_indent { &l[min_indent..] } else { l })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ---- extract_link_ref_defs ----

    #[test]
    fn extract_ref_defs_basic() {
        let input = "Hello\n\n[foo]: https://example.com\n\nWorld\n";
        let (labels, output) = extract_link_ref_defs(input);
        assert!(labels.contains("foo"));
        // Original bare definition line is replaced by a marker wrapping the text
        assert!(output.contains(REFDEF_MARKER_PREFIX));
        assert!(output.contains("https://example.com"));
        assert!(output.contains("World"));
    }

    #[test]
    fn extract_ref_defs_case_insensitive() {
        let input = "[Foo]: https://example.com\n";
        let (labels, _) = extract_link_ref_defs(input);
        assert!(labels.contains("foo"));
        assert!(!labels.contains("Foo"));
    }

    #[test]
    fn extract_ref_defs_inside_code_fence_ignored() {
        let input = "```\n[foo]: https://example.com\n```\n";
        let (labels, output) = extract_link_ref_defs(input);
        assert!(labels.is_empty());
        assert!(output.contains("[foo]:"));
        assert!(!output.contains(REFDEF_MARKER_PREFIX));
    }

    #[test]
    fn extract_ref_defs_with_title() {
        let input = "[bar]: https://example.com \"A title\"\n";
        let (labels, output) = extract_link_ref_defs(input);
        assert!(labels.contains("bar"));
        assert!(output.contains(REFDEF_MARKER_PREFIX));
    }

    #[test]
    fn extract_ref_defs_multiple() {
        let input = "[a]: https://a.com\n[b]: https://b.com\n";
        let (labels, _) = extract_link_ref_defs(input);
        assert_eq!(labels.len(), 2);
        assert!(labels.contains("a"));
        assert!(labels.contains("b"));
    }

    #[test]
    fn extract_ref_defs_skips_footnote_definitions() {
        // Footnote defs like [^label]: url look like ref defs to the regex.
        // They must NOT be treated as ref defs (REFDEF markers), since they are
        // handled separately by extract_footnote_defs.
        let input = "[normal]: https://example.com\n[^note]: https://another.com\n";
        let (labels, output) = extract_link_ref_defs(input);
        assert!(labels.contains("normal"), "Normal ref def should be extracted");
        assert!(
            !labels.contains("^note") && !labels.contains("note"),
            "Footnote label should NOT be in ref def labels"
        );
        // Normal ref def is wrapped in REFDEF marker
        assert!(output.contains(REFDEF_MARKER_PREFIX));
        // Footnote def is left unchanged (not wrapped)
        assert!(
            output.contains("[^note]: https://another.com"),
            "Footnote def should pass through unchanged, got:\n{output}"
        );
    }

    // ---- extract_footnote_defs ----

    #[test]
    fn extract_footnote_basic() {
        let input = "Text.\n\n[^note]: Footnote content.\n\nMore text.\n";
        let output = extract_footnote_defs(input);
        assert!(output.contains(FNDEF_MARKER_START));
        assert!(output.contains("Footnote content."));
        assert!(output.contains("More text."));
        // The definition text is wrapped inside the FNDEF marker, not left bare.
        // Verify the marker structure: starts with FNDEF_MARKER_START, ends with -->
        assert!(output.contains("-->"));
    }

    #[test]
    fn extract_footnote_multiline() {
        let input = "[^long]: First line.\n    Continuation line.\n\nAfter.\n";
        let output = extract_footnote_defs(input);
        assert!(output.contains(FNDEF_MARKER_START));
        assert!(output.contains("First line."));
        assert!(output.contains("Continuation line."));
    }

    #[test]
    fn extract_footnote_consecutive_blank_line_preserved() {
        let input = "[^1]: First.\n\n[^2]: Second.\n";
        let output = extract_footnote_defs(input);
        // Both definitions should be extracted
        let marker_count = output.matches(FNDEF_MARKER_START).count();
        assert_eq!(marker_count, 2, "Should have two FNDEF markers, got:\n{output}");
        // The blank line between them should be preserved
        assert!(
            output.contains("-->\n\n"),
            "Blank line between defs should be preserved, got:\n{output}"
        );
    }

    #[test]
    fn extract_footnote_with_autolink_blank_line_preserved() {
        use crate::config::ListSpacing;

        let input = "[^2]: <https://example.com/path>\n\n[^3]: <https://example.com/other>\n";
        let extracted = extract_footnote_defs(input);
        let marker_count = extracted.matches(FNDEF_MARKER_START).count();
        assert_eq!(marker_count, 2, "Should have two FNDEF markers, got:\n{extracted}");
        assert!(
            extracted.contains("-->\n\n"),
            "Blank line between defs should be preserved after extraction, got:\n{extracted}"
        );
        // Also check that protect_autolinks doesn't destroy the blank line
        let protected = protect_autolinks(&extracted);
        assert!(
            protected.contains("-->\n\n"),
            "Blank line between defs should be preserved after autolink protection, got:\n{protected}"
        );

        // Check the full pipeline output
        let result =
            fill_markdown(input, true, 88, false, false, false, false, None, ListSpacing::Preserve);
        assert!(
            result.contains("\n\n[^3]:"),
            "Full pipeline should preserve blank line between footnote defs with autolinks, got:\n{result}"
        );
    }

    #[test]
    fn extract_footnote_inside_code_fence_ignored() {
        let input = "```\n[^note]: Not a footnote.\n```\n";
        let output = extract_footnote_defs(input);
        assert!(!output.contains(FNDEF_MARKER_START));
        assert!(output.contains("[^note]:"));
    }

    // ---- encode_ref_links ----

    #[test]
    fn encode_full_ref_link() {
        let mut labels = HashSet::new();
        labels.insert("foo".to_string());
        let input = "See [click here][foo] for details.\n";
        let output = encode_ref_links(input, &labels);
        assert!(output.contains(REF_LABEL_START));
        assert!(output.contains(REF_LABEL_SEP));
        assert!(!output.contains("[foo]"));
    }

    #[test]
    fn encode_collapsed_ref_link() {
        let mut labels = HashSet::new();
        labels.insert("example".to_string());
        let input = "See [Example][] for details.\n";
        let output = encode_ref_links(input, &labels);
        assert!(output.contains(REF_LABEL_START));
    }

    #[test]
    fn encode_unknown_label_unchanged() {
        let mut labels = HashSet::new();
        labels.insert("known".to_string());
        let input = "See [text][unknown] for details.\n";
        let output = encode_ref_links(input, &labels);
        assert_eq!(input, output);
    }

    #[test]
    fn encode_empty_labels_passthrough() {
        let labels = HashSet::new();
        let input = "See [text][foo] for details.\n";
        let output = encode_ref_links(input, &labels);
        assert_eq!(input, output);
    }

    #[test]
    fn encode_inside_code_fence_unchanged() {
        let mut labels = HashSet::new();
        labels.insert("foo".to_string());
        let input = "```\n[text][foo]\n```\n";
        let output = encode_ref_links(input, &labels);
        assert!(output.contains("[text][foo]"));
    }

    // ---- replace_until_stable ----

    #[test]
    fn replace_until_stable_multiple_matches() {
        let re = Regex::new(r"ab").unwrap();
        let mut text = "ababab".to_string();
        replace_until_stable(&mut text, &re, |_| "X".to_string());
        assert_eq!(text, "XXX");
    }

    // ---- collision-resistant markers ----

    #[test]
    fn markers_contain_pua_char() {
        assert!(REFDEF_MARKER_PREFIX.contains('\u{F002}'));
        assert!(FNDEF_MARKER_START.contains('\u{F002}'));
    }

    #[test]
    fn user_html_comment_not_treated_as_marker() {
        // A normal HTML comment starting with "<!-- REFDEF:" should NOT be treated
        // as our internal marker since it lacks the PUA character.
        let user_comment = "<!-- REFDEF:see below -->";
        assert!(!user_comment.starts_with(REFDEF_MARKER_PREFIX));
    }
}
