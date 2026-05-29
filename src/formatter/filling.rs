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
//! | `U+F005`   | `ENTITY_AMP`        | HTML entity `&` replacement              |
//! | `U+E0xx`   | (computed)          | Escape placeholder for `\x` (xx = ASCII) |
//! | `U+E100`   | (filler)            | Width-preserving filler for escape pairs  |
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
//! replace `[text][label]` with `[text](\u{F000}HEX(label)\u{F001})`.
//! The label is hex-encoded so the payload is URL-safe through comrak's
//! parser (labels can contain spaces and punctuation like
//! `"St. John's School"`, which would otherwise break URL parsing).
//! During rendering, detect the PUA prefix, hex-decode the label, and
//! emit `[text][label]`. Definitions are stashed as REFDEF HTML
//! comment markers and re-emitted with the label lowercased to match
//! Python's marko normalization. Reference *images* (`![alt][label]`,
//! `![alt][]`, `![alt]`) are inlined to `![alt](url)` in pre-parse,
//! mirroring Python's `render_image` which always inlines.
//!
//! **Functions:** `extract_link_ref_defs`, `inline_image_refs`,
//! `encode_ref_links`
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
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

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

/// Aggregated stage-level performance counters for `fill_markdown`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FillPerfStats {
    /// Number of `fill_markdown` calls recorded.
    pub files: u64,
    /// Total preprocessing time.
    pub preprocess_ns: u128,
    /// Total comrak parse time.
    pub parse_ns: u128,
    /// Total AST transform time.
    pub transforms_ns: u128,
    /// Total render time.
    pub render_ns: u128,
    /// Total postprocess time.
    pub postprocess_ns: u128,
}

impl FillPerfStats {
    /// Total tracked nanoseconds across stages.
    pub fn total_ns(self) -> u128 {
        self.preprocess_ns
            + self.parse_ns
            + self.transforms_ns
            + self.render_ns
            + self.postprocess_ns
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct FillPerfSample {
    preprocess: Duration,
    parse: Duration,
    transforms: Duration,
    render: Duration,
    postprocess: Duration,
}

impl FillPerfStats {
    fn add_sample(&mut self, sample: FillPerfSample) {
        self.files += 1;
        self.preprocess_ns += sample.preprocess.as_nanos();
        self.parse_ns += sample.parse.as_nanos();
        self.transforms_ns += sample.transforms.as_nanos();
        self.render_ns += sample.render.as_nanos();
        self.postprocess_ns += sample.postprocess.as_nanos();
    }
}

static PERF_STATS_ENABLED: AtomicBool = AtomicBool::new(false);
static PERF_STATS: Mutex<FillPerfStats> = Mutex::new(FillPerfStats {
    files: 0,
    preprocess_ns: 0,
    parse_ns: 0,
    transforms_ns: 0,
    render_ns: 0,
    postprocess_ns: 0,
});

/// Enable or disable stage-level `fill_markdown` performance collection.
pub fn set_fill_perf_stats_enabled(enabled: bool) {
    PERF_STATS_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Reset accumulated `fill_markdown` performance counters.
pub fn reset_fill_perf_stats() {
    if let Ok(mut stats) = PERF_STATS.lock() {
        *stats = FillPerfStats::default();
    }
}

/// Snapshot accumulated `fill_markdown` performance counters.
pub fn get_fill_perf_stats() -> FillPerfStats {
    if let Ok(stats) = PERF_STATS.lock() { *stats } else { FillPerfStats::default() }
}

fn record_fill_perf_sample(sample: FillPerfSample) {
    if let Ok(mut stats) = PERF_STATS.lock() {
        stats.add_sample(sample);
    }
}

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

/// Regex for badge-pattern reference links: `[![alt](url)][label]`. The
/// outer link's text is an inline image, which contains a `]` that the
/// generic `FULL_REF_LINK` regex can't span (its text group rejects `]`).
/// Must run before `FULL_REF_LINK` so the inner `[alt](url)` isn't picked
/// off as a stray match.
static BADGE_FULL_REF_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(!\[[^\]]*\]\([^)]*\))\]\[([^\]]+)\]").expect("valid BADGE_FULL_REF_LINK regex")
});

/// Regex for badge-pattern collapsed reference links: `[![alt](url)][]`.
static BADGE_COLLAPSED_REF_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(!\[[^\]]*\]\([^)]*\))\]\[\]").expect("valid BADGE_COLLAPSED_REF_LINK regex")
});

/// Regex for badge-pattern shortcut reference links: `[![alt](url)]` not
/// followed by `[`, `(`, or `:`. Group 2 captures the trailing char.
static BADGE_SHORTCUT_REF_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(!\[[^\]]*\]\([^)]*\))\]([^\[(:]|$)")
        .expect("valid BADGE_SHORTCUT_REF_LINK regex")
});

/// Regex for collapsed reference links: `[text][]`
static COLLAPSED_REF_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\[\]").expect("valid COLLAPSED_REF_LINK regex"));

/// Regex for shortcut reference links: `[text]` not followed by `[`, `(`, or `:`
/// (which would make it a full/collapsed ref, an inline link, or a definition).
/// Group 2 captures the trailing char (or end-of-line) to emulate negative
/// lookahead, which the `regex` crate does not support; it is re-emitted as-is.
static SHORTCUT_REF_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[([^\]]+)\]([^\[(:]|$)").expect("valid SHORTCUT_REF_LINK regex")
});

/// Regex for full reference images: `![alt][label]`.
static IMAGE_FULL_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\[([^\]]+)\]").expect("valid IMAGE_FULL_REF regex"));

/// Regex for collapsed reference images: `![alt][]`.
static IMAGE_COLLAPSED_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^\]]+)\]\[\]").expect("valid IMAGE_COLLAPSED_REF regex"));

/// Regex for shortcut reference images: `![alt]` not followed by `[`, `(`, or
/// `:`. Group 2 captures the trailing char (or end-of-line); re-emitted as-is.
static IMAGE_SHORTCUT_REF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"!\[([^\]]+)\]([^\[(:]|$)").expect("valid IMAGE_SHORTCUT_REF regex")
});

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
    if !text.contains("```") {
        return text.to_string();
    }
    CODE_FENCE_SPACE.replace_all(text, "$1$2").into_owned()
}

/// Fix numbered list items: convert two spaces to one space after period.
fn normalize_numbered_lists(text: &str) -> String {
    if !text.contains(".  ") {
        return text.to_string();
    }
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

/// COMRAK-WORKAROUND13: PUA replacement for `&` in HTML entities.
/// Prevents comrak from decoding entities like `&amp;` → `&`.
const ENTITY_AMP: char = '\u{F005}';

/// COMRAK-WORKAROUND13: Regex matching HTML named/decimal/hex entities.
static HTML_ENTITY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"&(?:[a-zA-Z][a-zA-Z0-9]*|#[0-9]+|#x[0-9a-fA-F]+);").expect("valid regex")
});

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
            result_lines.push((*line).to_string());
            if is_closing_fence(line.trim(), &fence_str) {
                in_code = false;
            }
            continue;
        }
        if in_html_comment {
            result_lines.push((*line).to_string());
            if line.contains("-->") {
                in_html_comment = false;
            }
            continue;
        }
        if let Some(fs) = detect_opening_fence(line.trim()) {
            fence_str = fs;
            in_code = true;
            result_lines.push((*line).to_string());
            continue;
        }
        // Skip FNDEF/REFDEF markers — their content contains raw autolinks
        // that should be preserved as-is (they're rendered from the marker, not by comrak).
        if line.trim().starts_with(FNDEF_MARKER_START)
            || line.trim().starts_with(REFDEF_MARKER_PREFIX)
        {
            result_lines.push((*line).to_string());
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

/// COMRAK-WORKAROUND13: Replace `&` in HTML entities with a PUA placeholder
/// so comrak cannot decode them. Operates outside fenced code blocks.
fn protect_html_entities(text: &str) -> String {
    transform_outside_code_fences(text, |line| {
        let replaced = HTML_ENTITY_RE.replace_all(line, |caps: &regex::Captures| {
            // Replace leading '&' with PUA char, keep the rest (e.g., "amp;")
            format!("{ENTITY_AMP}{}", &caps[0][1..])
        });
        vec![replaced.into_owned()]
    })
}

/// COMRAK-WORKAROUND13: Restore PUA entity placeholders back to `&`.
fn restore_html_entities(text: &str) -> String {
    text.replace(ENTITY_AMP, "&")
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
/// code fences). Returns a map of lowercase label → formatted destination (`url` or
/// `url "title"`) and the text with definitions replaced by HTML comment markers.
/// These markers survive comrak parsing as `HtmlBlock` nodes, preserving the original
/// position of each definition in the AST. The destination map drives reference-image
/// inlining (see `inline_image_refs`).
fn extract_link_ref_defs(text: &str) -> (HashMap<String, String>, String) {
    let mut defs: HashMap<String, String> = HashMap::new();
    let result = transform_outside_code_fences(text, |line| {
        if let Some(caps) = LINK_REF_DEF.captures(line) {
            let label = &caps[1];
            // Skip footnote definitions (labels starting with ^)
            if label.starts_with('^') {
                return vec![line.to_string()];
            }
            let url = caps.get(2).map_or("", |m| m.as_str());
            let title = caps
                .get(3)
                .or_else(|| caps.get(4))
                .or_else(|| caps.get(5))
                .map_or("", |m| m.as_str());
            let destination =
                if title.is_empty() { url.to_string() } else { format!("{url} \"{title}\"") };
            defs.insert(label.to_lowercase(), destination);
            vec![format!("{REFDEF_MARKER_PREFIX}{line} -->")]
        } else {
            vec![line.to_string()]
        }
    });
    (defs, result)
}

/// COMRAK-WORKAROUND1: Lowercase the `[label]` portion of a link reference
/// definition line for emission. The URL, title, and surrounding whitespace
/// are preserved verbatim. Returns the input unchanged when it does not parse
/// as a ref-def (defensive: REFDEF markers only wrap matched lines).
fn lowercase_refdef_label(def: &str) -> String {
    if let Some(caps) = LINK_REF_DEF.captures(def) {
        if let Some(label) = caps.get(1) {
            let mut result = String::with_capacity(def.len());
            result.push_str(&def[..label.start()]);
            result.push_str(&label.as_str().to_lowercase());
            result.push_str(&def[label.end()..]);
            return result;
        }
    }
    def.to_string()
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

/// COMRAK-WORKAROUND1: Extract the lowercase alt text from a badge text
/// segment like `![alt](url)`. Used as the label for collapsed/shortcut
/// badge reference links. Returns `None` when the segment doesn't parse.
fn badge_alt_lowercase(image_markdown: &str) -> Option<String> {
    let stripped = image_markdown.strip_prefix("![")?;
    let alt_end = stripped.find(']')?;
    Some(stripped[..alt_end].to_lowercase())
}

/// COMRAK-WORKAROUND1: Hex-encode a label so the PUA-bounded payload only
/// contains URL-safe ASCII (`[0-9a-f]`). Natural Markdown labels like
/// `"St. John's School"` contain spaces/apostrophes, which break comrak's
/// `[text](url)` parsing (spaces terminate the URL token unless angle-bracketed).
/// Hex-encoding the label sidesteps the URL syntax entirely.
fn encode_hex_label(label: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(label.len() * 2);
    for b in label.as_bytes() {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// COMRAK-WORKAROUND1: Inverse of [`encode_hex_label`]. Returns `None` if the
/// payload is not valid hex or not valid UTF-8 (e.g. a legacy plain-text label
/// from before the hex-encoding switch — the caller can fall back).
fn decode_hex_label(hex: &str) -> Option<String> {
    if hex.is_empty() || hex.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.as_bytes().chunks(2) {
        let high = (chunk[0] as char).to_digit(16)?;
        let low = (chunk[1] as char).to_digit(16)?;
        // `to_digit(16)` returns 0..=15, so `(high << 4) | low` is always 0..=255.
        let byte = u8::try_from((high << 4) | low).ok()?;
        bytes.push(byte);
    }
    String::from_utf8(bytes).ok()
}

/// COMRAK-WORKAROUND1: Encode reference link labels in PUA markers.
/// `[text][label]` → `[text](\u{F000}HEX(label)\u{F001})`. The label is
/// hex-encoded so the payload is URL-safe through comrak's parser regardless of
/// label content. During rendering, the PUA prefix is detected and the label is
/// hex-decoded to re-emit `[text][label]` (or the collapsed `[text][]` when the
/// rendered text equals the label per issue #45).
fn encode_ref_links(text: &str, defs: &HashMap<String, String>) -> String {
    if defs.is_empty() {
        return text.to_string();
    }

    transform_outside_code_fences(text, |line| {
        let mut result = line.to_string();
        // Badge pattern first: [![alt](url)][label]. The image text contains
        // `]` which the generic FULL_REF_LINK regex cannot span, and without
        // this dedicated handler the trailing `[label]` would be misparsed
        // as a shortcut ref (preceded by `]`).
        result = BADGE_FULL_REF_LINK
            .replace_all(&result, |caps: &regex::Captures| {
                let text_part = &caps[1];
                let label = caps[2].to_lowercase();
                if defs.contains_key(&label) {
                    let hex = encode_hex_label(&label);
                    format!("[{text_part}]({REF_LABEL_START}{hex}{REF_LABEL_SEP})")
                } else {
                    caps[0].to_string()
                }
            })
            .into_owned();
        result = BADGE_COLLAPSED_REF_LINK
            .replace_all(&result, |caps: &regex::Captures| {
                let text_part = &caps[1];
                // Collapsed label is the image alt text — extract from `![alt](...)`.
                match badge_alt_lowercase(text_part) {
                    Some(label) if defs.contains_key(&label) => {
                        let hex = encode_hex_label(&label);
                        format!("[{text_part}]({REF_LABEL_START}{hex}{REF_LABEL_SEP})")
                    }
                    _ => caps[0].to_string(),
                }
            })
            .into_owned();
        result = BADGE_SHORTCUT_REF_LINK
            .replace_all(&result, |caps: &regex::Captures| {
                let text_part = &caps[1];
                let trailing = &caps[2];
                match badge_alt_lowercase(text_part) {
                    Some(label) if defs.contains_key(&label) => {
                        let hex = encode_hex_label(&label);
                        format!("[{text_part}]({REF_LABEL_START}{hex}{REF_LABEL_SEP}){trailing}")
                    }
                    _ => caps[0].to_string(),
                }
            })
            .into_owned();
        // Replace full reference links: [text][label]. Encode the matched
        // definition's normalized (lowercase) label so rendering can emit the
        // canonical reference form (see the Link branch in render_inline).
        replace_until_stable(&mut result, &FULL_REF_LINK, |caps: &regex::Captures| {
            let text_part = &caps[1];
            let label = caps[2].to_lowercase();
            if defs.contains_key(&label) {
                let hex = encode_hex_label(&label);
                format!("[{text_part}]({REF_LABEL_START}{hex}{REF_LABEL_SEP})")
            } else {
                caps[0].to_string()
            }
        });
        // Replace collapsed reference links: [text][]. The label is the
        // normalized link text.
        replace_until_stable(&mut result, &COLLAPSED_REF_LINK, |caps: &regex::Captures| {
            let text_part = &caps[1];
            let label = text_part.to_lowercase();
            if defs.contains_key(&label) {
                let hex = encode_hex_label(&label);
                format!("[{text_part}]({REF_LABEL_START}{hex}{REF_LABEL_SEP})")
            } else {
                caps[0].to_string()
            }
        });
        // Replace shortcut reference links: [text] not followed by `[`, `(`, or
        // `:`. The label is the normalized text; only encoded when it matches a
        // definition. Group 2 captures the trailing char (or end-of-line) and is
        // re-emitted verbatim. Uses replace_all (single pass) rather than
        // replace_until_stable because identity-replaced non-matching brackets
        // would otherwise halt the loop before later matches are reached.
        result = SHORTCUT_REF_LINK
            .replace_all(&result, |caps: &regex::Captures| {
                let text_part = &caps[1];
                let trailing = &caps[2];
                let label = text_part.to_lowercase();
                if defs.contains_key(&label) {
                    let hex = encode_hex_label(&label);
                    format!("[{text_part}]({REF_LABEL_START}{hex}{REF_LABEL_SEP}){trailing}")
                } else {
                    caps[0].to_string()
                }
            })
            .into_owned();
        vec![result]
    })
}

/// COMRAK-WORKAROUND1: Inline reference images (`![alt][label]`, `![alt][]`,
/// `![alt]`) by substituting the matched definition's destination directly,
/// producing `![alt](url)` or `![alt](url "title")`. This mirrors Python
/// flowmark's `render_image`, which always emits the inline image form
/// regardless of the source syntax. Inlining at the pre-parse stage avoids
/// PUA encoding for image URLs (where comrak's parser would otherwise leak
/// the marker into the rendered output).
fn inline_image_refs(text: &str, defs: &HashMap<String, String>) -> String {
    if defs.is_empty() {
        return text.to_string();
    }

    transform_outside_code_fences(text, |line| {
        let mut result = line.to_string();
        // Full: ![alt][label] → ![alt](dest)
        result = IMAGE_FULL_REF
            .replace_all(&result, |caps: &regex::Captures| {
                let alt = &caps[1];
                let label = caps[2].to_lowercase();
                if let Some(dest) = defs.get(&label) {
                    format!("![{alt}]({dest})")
                } else {
                    caps[0].to_string()
                }
            })
            .into_owned();
        // Collapsed: ![alt][] → ![alt](dest) where label is lowercase(alt)
        result = IMAGE_COLLAPSED_REF
            .replace_all(&result, |caps: &regex::Captures| {
                let alt = &caps[1];
                let label = alt.to_lowercase();
                if let Some(dest) = defs.get(&label) {
                    format!("![{alt}]({dest})")
                } else {
                    caps[0].to_string()
                }
            })
            .into_owned();
        // Shortcut: ![alt] (not followed by `[`, `(`, or `:`) → ![alt](dest)
        result = IMAGE_SHORTCUT_REF
            .replace_all(&result, |caps: &regex::Captures| {
                let alt = &caps[1];
                let trailing = &caps[2];
                let label = alt.to_lowercase();
                if let Some(dest) = defs.get(&label) {
                    format!("![{alt}]({dest}){trailing}")
                } else {
                    caps[0].to_string()
                }
            })
            .into_owned();
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

/// COMRAK-WORKAROUND4: Replace `\x` escapes with PUA placeholders outside fenced code
/// blocks. Prevents comrak from stripping backslash escapes during parsing. Within each
/// non-fenced line, replacements are applied uniformly (including inside inline code
/// spans) because Comrak can strip escapes even in code-span-like contexts such as GFM
/// table cells (P8).
fn protect_escapes_outside_code(text: &str, escape_set: &[char]) -> String {
    transform_outside_code_fences(text, |line| {
        let processed = replace_escapes_in_line(line, escape_set);
        vec![processed]
    })
}

/// COMRAK-WORKAROUND4: Replace `\<char>` escape sequences with PUA placeholders in a
/// single pass. For each `\<char>` where `<char>` is in `ESCAPE_CHARS`, emits the
/// corresponding PUA placeholder (U+E000 + `char_value`, U+E100 filler).
/// This replaces 32 sequential `.replace()` calls with one scan.
fn replace_escapes_in_line(line: &str, escape_set: &[char]) -> String {
    const PUA_FILLER: char = '\u{E100}';

    // Fast path: no backslash means no escapes to replace.
    if !line.contains('\\') {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next) = chars.peek() {
                if escape_set.contains(&next) {
                    chars.next(); // consume the escaped char
                    let pua = char::from_u32(0xE000 + next as u32).expect("valid PUA");
                    result.push(pua);
                    result.push(PUA_FILLER);
                    continue;
                }
            }
            result.push(ch);
        } else {
            result.push(ch);
        }
    }

    result
}

/// COMRAK-WORKAROUND4 (post-processing): Restore PUA escape placeholders to original
/// `\<char>` escapes in a single pass. Each placeholder is a 2-char sequence:
/// PUA char (U+E000 + `original_ascii`) followed by filler U+E100.
/// This replaces 32 sequential `.replace()` calls with one scan.
fn restore_pua_escape_placeholders(text: &str) -> String {
    const PUA_FILLER: char = '\u{E100}';

    // Fast path: if no PUA chars present, return as-is.
    if !text.contains(|c: char| ('\u{E000}'..='\u{E0FF}').contains(&c)) {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ('\u{E000}'..='\u{E0FF}').contains(&ch) {
            if chars.peek() == Some(&PUA_FILLER) {
                chars.next(); // consume filler
                // Recover original ASCII char: pua_char = 0xE000 + ascii_value
                let original = char::from_u32(ch as u32 - 0xE000).unwrap_or(ch);
                result.push('\\');
                result.push(original);
            } else {
                // PUA char without filler — pass through unchanged
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// COMRAK-WORKAROUND11: Remove unnecessary period escapes from the formatted output.
/// Period escapes (`\.`) are only needed at the start of a line where `DIGITS\.`
/// would be interpreted as an ordered list marker. In headings and mid-paragraph,
/// period escapes are unnecessary.
/// Preserves content inside code spans (backtick-delimited) and fenced code blocks.
fn postprocess_period_escapes(text: &str) -> String {
    if !text.contains("\\.") {
        return text.to_string();
    }
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
        // fmr-rscu: TaskItem must recurse like Item. comrak reports a task-list
        // item's own sourcepos as extending through the trailing blank line
        // (e.g. `- [ ] b\n\n* * *` ends the item at the blank), so falling through
        // to the `_` arm over-counts the list's end and wrongly marks the
        // following thematic break as originally-tight (dropping its blank).
        NodeValue::List(_) | NodeValue::Item(_) | NodeValue::TaskItem(_) => {
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
    let mut prev_was_thematic_break = false;

    for child in node.children() {
        let child_is_block = is_block_element(child);
        let child_is_refdef_only = is_refdef_marker(child);
        let child_is_html_comment = is_html_comment_only(child);
        let child_is_list = matches!(child.data.borrow().value, NodeValue::List(_));
        let child_is_code_block = matches!(child.data.borrow().value, NodeValue::CodeBlock(_));
        let child_is_thematic_break = matches!(child.data.borrow().value, NodeValue::ThematicBreak);
        let child_is_table = matches!(child.data.borrow().value, NodeValue::Table(_));
        let child_is_blockquote = matches!(child.data.borrow().value, NodeValue::BlockQuote);

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
        // Rule 4: Paragraph → code block (tight): suppress separator
        // Rule 5: Thematic break adjacent to any block (tight): suppress separator
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
                // Note: render_block_children is only called for Document-level
                // children, so list_spacing mode doesn't affect this rule.
                // List-item-level spacing is handled in render_list_item.
                true
            } else if child_is_code_block && prev_was_paragraph {
                // Rule 4: Paragraph → code block (tight): suppress (P6)
                // This handles cases like "**Config**:\n```json\n{}\n```"
                // Note: same as Rule 3, this is Document-level only.
                true
            } else if prev_was_thematic_break || child_is_thematic_break {
                // Rule 5: Thematic break adjacent to any block (tight): suppress (D17)
                // Python/marko preserves source tightness around `* * *`, while
                // comrak forces blank lines on both sides. Symmetric: applies
                // whether the break precedes or follows the neighboring block.
                true
            } else if child_is_table && prev_was_paragraph {
                // Rule 6: Paragraph → table (tight): suppress (v0.7.0 #36).
                // The "Wide Table Adjacent to Paragraph" fixture exercises this —
                // a table written tight against the preceding paragraph stays
                // tight, matching Python flowmark v0.7.0.
                true
            } else if child_is_blockquote && prev_was_paragraph {
                // Rule 7: Paragraph → blockquote (tight): suppress (fmr-iblt).
                // A blockquote written directly under a paragraph (e.g. a bold
                // label line "**Current text:**\n> [quote]") stays tight, matching
                // Python/marko. comrak otherwise forces a blank separator.
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
        prev_was_thematic_break = child_is_thematic_break;
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
    let mut prev_source_end_line: usize = 0;

    for child in node.children() {
        let child_is_block = is_block_element(child);
        let child_is_blockquote = matches!(child.data.borrow().value, NodeValue::BlockQuote);

        // Use source positions to detect whether blocks were originally tight.
        let child_source_start = child.data.borrow().sourcepos.start.line;
        let child_source_end = last_content_line(child);
        let originally_tight =
            prev_source_end_line > 0 && child_source_start <= prev_source_end_line + 1;

        // Add blank line between consecutive block elements.
        // Use the blank_prefix (e.g., "> ") to maintain the quote context.
        // Suppress separator before nested blockquotes only when the original
        // source was tight (no blank line). When the source had a blank `>`
        // line between blocks, preserve it to match Python behavior.
        let suppress = child_is_blockquote && originally_tight;
        if child_is_block && prev_was_block && !prev_ended_with_double_newline && !suppress {
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
        prev_source_end_line = child_source_end;
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
            // Determine effective tightness.
            // Python's --list-spacing tight forces tight only when all items are
            // simple (single paragraph, no sublists/code blocks). When any item
            // is "complex" (has sublists, code blocks, or multiple paragraphs),
            // Python treats the entire list as loose even in tight mode.
            let any_item_is_complex = node.children().any(|item| {
                let children: Vec<_> = item.children().collect();
                let has_sublist =
                    children.iter().any(|c| matches!(c.data.borrow().value, NodeValue::List(_)));
                let has_code = children
                    .iter()
                    .any(|c| matches!(c.data.borrow().value, NodeValue::CodeBlock(_)));
                let para_count = children
                    .iter()
                    .filter(|c| matches!(c.data.borrow().value, NodeValue::Paragraph))
                    .count();
                has_sublist || has_code || para_count > 1
            });
            let is_tight = match list_spacing {
                ListSpacing::Preserve => list.tight,
                ListSpacing::Tight => !any_item_is_complex,
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
            // Only output content lines if the code block has actual content.
            // An empty code block (content is empty or whitespace-only) should
            // produce just the opening and closing fences with no lines between
            // them, matching Python's behavior (D16).
            if !code_content.is_empty() {
                let empty_prefix = subsequent_prefix.trim_end();
                for line in code_content.split('\n') {
                    if line.is_empty() {
                        output.push_str(empty_prefix);
                        output.push('\n');
                    } else {
                        let _ = writeln!(output, "{subsequent_prefix}{line}");
                    }
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

            // COMRAK-WORKAROUND1: Re-emit reference definition from REFDEF
            // marker. The label is lowercased to match Python flowmark, which
            // emits `element.label` (marko normalizes ref-def labels to
            // lowercase). The URL and any title are preserved verbatim.
            if let Some(rest) = trimmed.strip_prefix(REFDEF_MARKER_PREFIX) {
                if let Some(def_text) = rest.strip_suffix("-->") {
                    let def_text = def_text.trim();
                    let lowered = lowercase_refdef_label(def_text);
                    let _ = writeln!(output, "{prefix}{lowered}");
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

                                // COMRAK-WORKAROUND9a: Detect blockquote continuation
                                // (lines starting with `>`). Python/marko preserves these
                                // on separate lines under the footnote definition.
                                let blockquote_start_idx =
                                    body_lines.iter().skip(1).position(|l| l.starts_with('>'));

                                // COMRAK-WORKAROUND9b: Detect embedded list items
                                // (lines starting with `- `, `* `, or `+ `).
                                let list_start_idx = body_lines.iter().skip(1).position(|l| {
                                    l.starts_with("- ")
                                        || l.starts_with("* ")
                                        || l.starts_with("+ ")
                                });

                                if let Some(bq_idx) = blockquote_start_idx {
                                    let bq_idx = bq_idx + 1; // adjust for skip(1)
                                    // Preamble paragraph before the blockquote
                                    let preamble = body_lines[..bq_idx].join(" ");
                                    let wrapped =
                                        line_wrapper(preamble.trim(), &fn_prefix, &fn_subsequent);
                                    output.push_str(&wrapped);
                                    output.push('\n');
                                    // Blockquote lines
                                    for line in &body_lines[bq_idx..] {
                                        let bq_body = line
                                            .strip_prefix("> ")
                                            .unwrap_or(line.strip_prefix('>').unwrap_or(line));
                                        let bq_prefix = format!("{fn_subsequent}> ");
                                        let bq_subsequent = format!("{fn_subsequent}> ");
                                        let wrapped = line_wrapper(
                                            bq_body.trim(),
                                            &bq_prefix,
                                            &bq_subsequent,
                                        );
                                        output.push_str(&wrapped);
                                    }
                                    output.push_str("\n\n");
                                } else if let Some(idx) = list_start_idx {
                                    let idx = idx + 1; // adjust for skip(1)
                                    // Preamble paragraph before the list
                                    let preamble = body_lines[..idx].join(" ");
                                    let wrapped =
                                        line_wrapper(preamble.trim(), &fn_prefix, &fn_subsequent);
                                    output.push_str(&wrapped);
                                    // In loose mode, add blank separator between
                                    // footnote preamble and embedded list.
                                    if list_spacing == ListSpacing::Loose {
                                        output.push_str("\n\n");
                                    } else {
                                        output.push('\n');
                                    }
                                    // Render each list item separately.
                                    // Python/marko treats each `- ` line as a separate item.
                                    let mut current_marker = "";
                                    let mut current_text = String::new();
                                    for line in &body_lines[idx..] {
                                        let is_item_start = line.starts_with("- ")
                                            || line.starts_with("* ")
                                            || line.starts_with("+ ");
                                        if is_item_start {
                                            // Flush previous item if any
                                            if !current_text.is_empty() {
                                                let list_prefix =
                                                    format!("{fn_subsequent}{current_marker}");
                                                let list_subsequent = format!("{fn_subsequent}  ");
                                                let wrapped = line_wrapper(
                                                    current_text.trim(),
                                                    &list_prefix,
                                                    &list_subsequent,
                                                );
                                                output.push_str(&wrapped);
                                                output.push('\n');
                                            }
                                            current_marker = &line[..2];
                                            current_text = line[2..].to_string();
                                        } else {
                                            // Continuation of current item
                                            current_text.push(' ');
                                            current_text.push_str(line);
                                        }
                                    }
                                    // Flush last item
                                    if !current_text.is_empty() {
                                        let list_prefix =
                                            format!("{fn_subsequent}{current_marker}");
                                        let list_subsequent = format!("{fn_subsequent}  ");
                                        let wrapped = line_wrapper(
                                            current_text.trim(),
                                            &list_prefix,
                                            &list_subsequent,
                                        );
                                        output.push_str(&wrapped);
                                    }
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
                // fmr-8vy3: Preserve blank-line-separated paragraphs within the HTML
                // block. Python/marko reflows each paragraph independently and keeps
                // the blank line between them; collapsing the whole block onto one
                // logical line (the previous behavior) lost internal blank lines in
                // multi-paragraph HTML comments (e.g. generated-file banners).
                let mut paragraphs: Vec<String> = Vec::new();
                let mut cur: Vec<&str> = Vec::new();
                for line in literal.lines() {
                    if line.trim().is_empty() {
                        if !cur.is_empty() {
                            paragraphs
                                .push(cur.iter().map(|s| s.trim()).collect::<Vec<_>>().join(" "));
                            cur.clear();
                        }
                    } else {
                        cur.push(line);
                    }
                }
                if !cur.is_empty() {
                    paragraphs.push(cur.iter().map(|s| s.trim()).collect::<Vec<_>>().join(" "));
                }
                for (pi, para) in paragraphs.iter().enumerate() {
                    if pi > 0 {
                        // blank line between paragraphs
                        output.push('\n');
                    }
                    let (p, sp) = if pi == 0 {
                        (prefix, subsequent_prefix)
                    } else {
                        (subsequent_prefix, subsequent_prefix)
                    };
                    output.push_str(&line_wrapper(para.trim(), p, sp));
                    output.push('\n');
                }
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
                // In loose mode, add blank separator between footnote children
                // (matching Python behavior where footnote para + list get spacing).
                if !first_child && list_spacing == ListSpacing::Loose {
                    output.push('\n');
                }
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
///
/// In Loose mode or when the list is natively loose (Preserve + `!tight`):
/// always add blank lines between children.
///
/// In Tight mode: Python makes the list loose between ITEMS (handled by
/// `is_tight`), but within each item only adds spacing when the item has
/// code blocks, multiple paragraphs, or complex sublists (sublists with
/// deeper nesting). Simple para+sublist items stay tight within.
///
/// In Preserve mode with tight list: only add spacing for multiple paragraphs.
fn item_needs_child_spacing<'a>(
    node: &'a AstNode<'a>,
    parent_is_tight: bool,
    list_spacing: ListSpacing,
) -> bool {
    let children: Vec<_> = node.children().collect();
    if children.len() <= 1 {
        return false;
    }

    match list_spacing {
        ListSpacing::Loose => true,
        ListSpacing::Preserve => {
            if !parent_is_tight {
                return true;
            }
            // fmr-fle0: For tight preserved lists, preserve SOURCE spacing — add
            // within-item spacing only where two children were originally separated
            // by a blank line. The previous `para_count > 1` heuristic wrongly added
            // blank lines around a fenced code block written tight between two
            // paragraphs (the paragraphs are separated by the code block, not a
            // blank), where Python/marko keeps the whole item tight.
            let mut prev_end: usize = 0;
            for c in &children {
                let start = c.data.borrow().sourcepos.start.line;
                if prev_end > 0 && start > prev_end + 1 {
                    return true;
                }
                prev_end = last_content_line(c);
            }
            false
        }
        ListSpacing::Tight => {
            // Python's tight mode adds within-item spacing for items with:
            // 1. Code blocks
            // 2. Complex sublists (sublists with deeper nesting)
            // 3. Multiple paragraphs
            // 4. Children that were originally separated by blank lines
            let has_code =
                children.iter().any(|c| matches!(c.data.borrow().value, NodeValue::CodeBlock(_)));
            if has_code {
                return true;
            }
            // Check if any child sublist is effectively loose. A sublist is
            // effectively loose when it has complex items (sublists, code blocks,
            // or multi-paragraph) or when Comrak marked it as not tight.
            let has_effectively_loose_sublist = children.iter().any(|c| {
                if let NodeValue::List(sub_list) = &c.data.borrow().value {
                    if !sub_list.tight {
                        return true;
                    }
                    // Check for complex items: sublists, code blocks, multi-para
                    c.children().any(|item| {
                        let ch: Vec<_> = item.children().collect();
                        let has_sub = ch
                            .iter()
                            .any(|gc| matches!(gc.data.borrow().value, NodeValue::List(_)));
                        let has_code = ch
                            .iter()
                            .any(|gc| matches!(gc.data.borrow().value, NodeValue::CodeBlock(_)));
                        let paras = ch
                            .iter()
                            .filter(|gc| matches!(gc.data.borrow().value, NodeValue::Paragraph))
                            .count();
                        has_sub || has_code || paras > 1
                    })
                } else {
                    false
                }
            });
            if has_effectively_loose_sublist {
                return true;
            }
            // Check if any consecutive children were originally separated
            // by a blank line (i.e. not tight). If so, preserve spacing.
            let mut prev_end: usize = 0;
            for c in &children {
                let start = c.data.borrow().sourcepos.start.line;
                if prev_end > 0 && start > prev_end + 1 {
                    return true;
                }
                prev_end = last_content_line(c);
            }
            false
        }
    }
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

    // Check if parent list is effectively tight, using the same logic as
    // the List rendering arm. For Tight mode, lists with complex items
    // (sublists, code blocks, multi-paragraph) are treated as loose.
    let parent_is_tight = node.parent().is_some_and(|parent| {
        let data = parent.data.borrow();
        if let NodeValue::List(list) = &data.value {
            match list_spacing {
                ListSpacing::Preserve => list.tight,
                ListSpacing::Tight => {
                    // Mirror the any_item_is_complex check from the List arm
                    let any_complex = parent.children().any(|item| {
                        let ch: Vec<_> = item.children().collect();
                        let has_sub =
                            ch.iter().any(|c| matches!(c.data.borrow().value, NodeValue::List(_)));
                        let has_code = ch
                            .iter()
                            .any(|c| matches!(c.data.borrow().value, NodeValue::CodeBlock(_)));
                        let paras = ch
                            .iter()
                            .filter(|c| matches!(c.data.borrow().value, NodeValue::Paragraph))
                            .count();
                        has_sub || has_code || paras > 1
                    });
                    !any_complex
                }
                ListSpacing::Loose => false,
            }
        } else {
            false
        }
    });

    let needs_spacing = item_needs_child_spacing(node, parent_is_tight, list_spacing);

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

            // COMRAK-WORKAROUND10: Don't add a blank line before a child block
            // unless the original source had one. Comrak marks the whole parent
            // list as loose when *any* sibling pair has a blank line, which would
            // insert blanks inside every item. Python/marko only inserts the
            // blank when the author actually wrote one.
            //
            // Applies to:
            // - List children in Preserve mode (original workaround)
            // - CodeBlock children in all modes (D12b/P6: mixed loose/tight lists)
            let suppress_nested_blank = if !parent_is_tight && i > 0 {
                let child_value = &child.data.borrow().value;
                let should_check = matches!(child_value, NodeValue::CodeBlock(_))
                    || (matches!(child_value, NodeValue::List(_))
                        && list_spacing == ListSpacing::Preserve);
                if should_check {
                    let prev_end = children[i - 1].data.borrow().sourcepos.end.line;
                    let curr_start = child.data.borrow().sourcepos.start.line;
                    // No blank line in original source → suppress
                    curr_start <= prev_end + 1
                } else {
                    false
                }
            } else {
                false
            };

            if !prev_ended_double
                && !current_is_hard_break_heading
                && !current_is_tag_block
                && !suppress_nested_blank
            {
                // Use the item's subsequent prefix to maintain blockquote
                // context and list indentation on blank separator lines (P7).
                // Python preserves the full list-content indent (e.g., ">    "
                // for numbered lists in blockquotes), not just the bare ">" marker.
                if item_subsequent.trim().is_empty() {
                    output.push('\n');
                } else {
                    output.push_str(item_subsequent);
                    output.push('\n');
                }
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
/// Only URLs with a scheme (http://, https://, etc.) or email addresses can be
/// autolinks — relative paths like `[foo.md](foo.md)` are explicit links even
/// when text == URL.
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
    let url = &link.url;
    // Comrak's autolink extension only creates autolinks for URLs with a scheme
    // or email addresses. Relative paths are never autolinks.
    let has_scheme = url.contains("://") || url.starts_with("mailto:");
    let is_email = !url.contains("://") && url.contains('@');
    if !has_scheme && !is_email {
        return false;
    }
    // Inner text matches URL (autolink) or URL minus "mailto:" (email autolink)
    text == *url || url.strip_prefix("mailto:").is_some_and(|stripped| text == stripped)
}

/// Render a single inline node to string.
fn render_inline<'a>(node: &'a AstNode<'a>, options: &Options, in_heading: bool) -> String {
    match &node.data.borrow().value {
        NodeValue::Text(text) => text.to_string(),

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
                    // Label is hex-encoded inside the PUA markers so the URL
                    // contains only ASCII hex digits — preventing comrak from
                    // breaking on spaces/punctuation in natural labels like
                    // "St. John's School". Decode back to the original label.
                    let hex = &link.url[REF_LABEL_START.len_utf8()..sep_pos];
                    let decoded = decode_hex_label(hex);
                    let label = decoded.as_deref().unwrap_or(hex);
                    // Issue #45: when the link text equals the normalized label,
                    // emit the unambiguous collapsed form [text][] rather than a
                    // fragile shortcut [text]. Otherwise emit the full form.
                    if inner.as_str() == label {
                        format!("[{inner}][]")
                    } else {
                        format!("[{inner}][{label}]")
                    }
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

/// Get tasklist marker for a paragraph if its parent is a tasklist item.
fn get_tasklist_marker<'a>(para_node: &'a AstNode<'a>) -> Option<String> {
    if let Some(parent) = para_node.parent() {
        if let NodeValue::TaskItem(checked) = &parent.data.borrow().value {
            let marker = if checked.symbol.is_some() { "[x] " } else { "[ ] " };
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
    let perf_enabled = PERF_STATS_ENABLED.load(Ordering::Relaxed);
    let mut perf_sample = FillPerfSample::default();

    // Extract frontmatter before any processing
    let (frontmatter, content) = split_frontmatter(markdown_text);

    let mut text = if frontmatter.is_empty() { markdown_text.to_string() } else { content };

    if dedent_input {
        text = dedent(&text);
    }

    text = text.trim().to_string();
    text.push('\n');
    let preprocess_start = perf_enabled.then(Instant::now);

    // === Pre-parse workarounds (see module-level COMRAK-WORKAROUND docs) ===

    // COMRAK-WORKAROUND6: Ensure proper blank lines around block content within tags.
    text = preprocess_tag_block_spacing(&text);

    // COMRAK-WORKAROUND1: Extract link reference definitions, inline reference
    // images (Python flowmark always emits the inline image form), and encode
    // reference *links* with PUA markers. Must happen before escape placeholder
    // substitution, which would mangle `\[` etc.
    let (ref_defs, text_without_defs) = extract_link_ref_defs(&text);
    text = inline_image_refs(&text_without_defs, &ref_defs);
    text = encode_ref_links(&text, &ref_defs);

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

    // COMRAK-WORKAROUND13: Protect HTML entities from comrak decoding.
    text = protect_html_entities(&text);

    // COMRAK-WORKAROUND4: Replace `\x` escape sequences with PUA placeholders.
    // Each `\x` is 2 chars, so use a 2-char PUA placeholder to preserve width
    // during line wrapping. Uses a single-pass scan per line instead of 32 sequential
    // `.replace()` calls.
    text = protect_escapes_outside_code(&text, ESCAPE_CHARS);
    if let Some(start) = preprocess_start {
        perf_sample.preprocess = start.elapsed();
    }

    // === Parse with comrak ===
    let parse_start = perf_enabled.then(Instant::now);
    let arena = Arena::new();
    let options = flowmark_comrak_options();
    let root = comrak::parse_document(&arena, &text, &options);
    if let Some(start) = parse_start {
        perf_sample.parse = start.elapsed();
    }

    // === AST transforms (not comrak workarounds) ===
    let transforms_start = perf_enabled.then(Instant::now);
    if cleanups {
        doc_cleanups(root);
    }
    if smartquotes {
        apply_smart_quotes_to_ast(root);
    }
    if ellipses {
        apply_ellipses_to_ast(root);
    }
    if let Some(start) = transforms_start {
        perf_sample.transforms = start.elapsed();
    }

    // === Render AST to markdown ===
    // COMRAK-WORKAROUND1/2/3/7/8/9/10 all apply during rendering (see render_block).
    let render_start = perf_enabled.then(Instant::now);
    let mut in_heading = false;
    let result = render_block(root, &line_wrapper, list_spacing, "", "", &mut in_heading, &options);
    if let Some(start) = render_start {
        perf_sample.render = start.elapsed();
    }

    // === Post-render workarounds ===
    let postprocess_start = perf_enabled.then(Instant::now);

    // COMRAK-WORKAROUND4: Restore escaped characters from PUA placeholders.
    // Single-pass scan: any PUA char in range U+E000..U+E100 followed by U+E100 filler
    // is restored to the original `\<char>` escape.
    let result = restore_pua_escape_placeholders(&result);

    // COMRAK-WORKAROUND11: Remove unnecessary period escapes.
    let result = postprocess_period_escapes(&result);

    // COMRAK-WORKAROUND13: Restore HTML entity ampersands from PUA placeholders.
    let result = restore_html_entities(&result);

    // COMRAK-WORKAROUND3: Restore autolink angle brackets from PUA placeholders.
    let result = restore_autolinks(&result);

    // COMRAK-WORKAROUND12: Normalize comrak output formatting differences.
    let result = normalize_comrak_output(&result);

    // Python always outputs at least a trailing newline for empty/whitespace input.
    let result = if result.is_empty() { "\n".to_string() } else { result };

    // Reattach frontmatter if present
    let result = if frontmatter.is_empty() { result } else { format!("{frontmatter}{result}") };
    if let Some(start) = postprocess_start {
        perf_sample.postprocess = start.elapsed();
    }
    if perf_enabled {
        record_fill_perf_sample(perf_sample);
    }
    result
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
                NodeValue::Code(code) => {
                    // Skip code spans - don't apply smart quotes inside them.
                    // Use the last character of the code content as a placeholder
                    // to preserve context for smart quote detection (P9).
                    // Python's behavior is context-sensitive: apostrophe after
                    // code ending with a word char (e.g., `config`'s) IS converted
                    // to a smart quote, but after a non-word char (e.g., `foo()`'s)
                    // it stays ASCII. Using the actual last char matches this.
                    let last_char = code.literal.chars().last().unwrap_or(' ');
                    concatenated.push(if last_char.is_alphanumeric() || last_char == '_' {
                        last_char
                    } else {
                        ' '
                    });
                }
                NodeValue::HtmlInline(_) => {
                    concatenated.push(' ');
                }
                NodeValue::SoftBreak => {
                    // Preserve as '\n' (not ' ') so the smart-quote regex's
                    // multiline `^` anchor can match the start of a following
                    // sentence's opening quote. Matches Python flowmark which
                    // coalesces RawText across soft line breaks while keeping
                    // the newline character intact. Single char either way, so
                    // char_boundaries are unaffected.
                    concatenated.push('\n');
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
                *text = new_text.into();
            }
        }
    }
}

/// Apply ellipsis conversion to all text nodes in the AST.
fn apply_ellipses_to_ast<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Text(ref mut text) = data.value {
            *text = apply_ellipses(text).into();
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
        let (defs, output) = extract_link_ref_defs(input);
        assert_eq!(defs.get("foo").map(String::as_str), Some("https://example.com"));
        // Original bare definition line is replaced by a marker wrapping the text
        assert!(output.contains(REFDEF_MARKER_PREFIX));
        assert!(output.contains("https://example.com"));
        assert!(output.contains("World"));
    }

    #[test]
    fn extract_ref_defs_case_insensitive() {
        let input = "[Foo]: https://example.com\n";
        let (defs, _) = extract_link_ref_defs(input);
        assert!(defs.contains_key("foo"));
        assert!(!defs.contains_key("Foo"));
    }

    #[test]
    fn extract_ref_defs_inside_code_fence_ignored() {
        let input = "```\n[foo]: https://example.com\n```\n";
        let (defs, output) = extract_link_ref_defs(input);
        assert!(defs.is_empty());
        assert!(output.contains("[foo]:"));
        assert!(!output.contains(REFDEF_MARKER_PREFIX));
    }

    #[test]
    fn extract_ref_defs_with_title() {
        let input = "[bar]: https://example.com \"A title\"\n";
        let (defs, output) = extract_link_ref_defs(input);
        // Destination map captures the title so reference-image inlining can
        // round-trip `![alt][bar]` → `![alt](url "A title")`.
        assert_eq!(defs.get("bar").map(String::as_str), Some("https://example.com \"A title\""));
        assert!(output.contains(REFDEF_MARKER_PREFIX));
    }

    #[test]
    fn extract_ref_defs_multiple() {
        let input = "[a]: https://a.com\n[b]: https://b.com\n";
        let (defs, _) = extract_link_ref_defs(input);
        assert_eq!(defs.len(), 2);
        assert!(defs.contains_key("a"));
        assert!(defs.contains_key("b"));
    }

    #[test]
    fn extract_ref_defs_skips_footnote_definitions() {
        // Footnote defs like [^label]: url look like ref defs to the regex.
        // They must NOT be treated as ref defs (REFDEF markers), since they are
        // handled separately by extract_footnote_defs.
        let input = "[normal]: https://example.com\n[^note]: https://another.com\n";
        let (defs, output) = extract_link_ref_defs(input);
        assert!(defs.contains_key("normal"), "Normal ref def should be extracted");
        assert!(
            !defs.contains_key("^note") && !defs.contains_key("note"),
            "Footnote label should NOT be in ref def map"
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

    fn refdef_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect()
    }

    #[test]
    fn encode_full_ref_link() {
        let defs = refdef_map(&[("foo", "https://example.com")]);
        let input = "See [click here][foo] for details.\n";
        let output = encode_ref_links(input, &defs);
        assert!(output.contains(REF_LABEL_START));
        assert!(output.contains(REF_LABEL_SEP));
        assert!(!output.contains("[foo]"));
    }

    #[test]
    fn encode_collapsed_ref_link() {
        let defs = refdef_map(&[("example", "https://example.com")]);
        let input = "See [Example][] for details.\n";
        let output = encode_ref_links(input, &defs);
        assert!(output.contains(REF_LABEL_START));
    }

    #[test]
    fn encode_shortcut_ref_link() {
        let defs = refdef_map(&[("foo", "https://example.com")]);
        let input = "See [foo] for details.\n";
        let output = encode_ref_links(input, &defs);
        assert!(output.contains(REF_LABEL_START), "shortcut ref should be encoded");
        // The trailing space after the shortcut is preserved.
        assert!(output.contains(") for details."));
    }

    #[test]
    fn encode_shortcut_ref_lowercases_label() {
        let defs = refdef_map(&[("foo", "https://example.com")]);
        let input = "See [Foo] here.\n";
        let output = encode_ref_links(input, &defs);
        // The encoded label is normalized to lowercase ("foo") and then
        // hex-encoded (`66 6f 6f` → "666f6f") so the URL payload is URL-safe.
        let expected_payload = encode_hex_label("foo");
        assert_eq!(expected_payload, "666f6f");
        assert!(
            output.contains(&format!("{REF_LABEL_START}{expected_payload}{REF_LABEL_SEP}")),
            "expected hex-encoded label in output: {output:?}"
        );
    }

    #[test]
    fn hex_label_round_trip_handles_spaces_and_punctuation() {
        for label in ["foo", "st. john's school", "an example", "café"] {
            let hex = encode_hex_label(label);
            // Hex payload is URL-safe (ASCII hex only).
            assert!(
                hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')),
                "hex payload must be URL-safe: {hex:?}"
            );
            assert_eq!(decode_hex_label(&hex).as_deref(), Some(label));
        }
    }

    #[test]
    fn decode_hex_label_rejects_invalid() {
        // Empty, odd length, and non-hex chars are not valid encodings.
        assert!(decode_hex_label("").is_none());
        assert!(decode_hex_label("abc").is_none());
        assert!(decode_hex_label("xy").is_none());
    }

    #[test]
    fn encode_unknown_label_unchanged() {
        let defs = refdef_map(&[("known", "https://example.com")]);
        let input = "See [text][unknown] for details.\n";
        let output = encode_ref_links(input, &defs);
        assert_eq!(input, output);
    }

    #[test]
    fn encode_empty_labels_passthrough() {
        let defs: HashMap<String, String> = HashMap::new();
        let input = "See [text][foo] for details.\n";
        let output = encode_ref_links(input, &defs);
        assert_eq!(input, output);
    }

    #[test]
    fn encode_inside_code_fence_unchanged() {
        let defs = refdef_map(&[("foo", "https://example.com")]);
        let input = "```\n[text][foo]\n```\n";
        let output = encode_ref_links(input, &defs);
        assert!(output.contains("[text][foo]"));
    }

    // ---- inline_image_refs ----

    #[test]
    fn inline_image_full_ref() {
        let defs = refdef_map(&[("img", "https://example.com/img.png")]);
        let input = "![alt][img]\n";
        let output = inline_image_refs(input, &defs);
        assert_eq!(output, "![alt](https://example.com/img.png)\n");
    }

    #[test]
    fn inline_image_collapsed_ref() {
        let defs = refdef_map(&[("alt", "https://example.com/img.png")]);
        let input = "![alt][]\n";
        let output = inline_image_refs(input, &defs);
        assert_eq!(output, "![alt](https://example.com/img.png)\n");
    }

    #[test]
    fn inline_image_shortcut_ref() {
        let defs = refdef_map(&[("alt", "https://example.com/img.png")]);
        let input = "![alt]\n";
        let output = inline_image_refs(input, &defs);
        assert_eq!(output, "![alt](https://example.com/img.png)\n");
    }

    #[test]
    fn inline_image_with_title() {
        let defs = refdef_map(&[("img", "https://example.com/img.png \"My title\"")]);
        let input = "![alt][img]\n";
        let output = inline_image_refs(input, &defs);
        assert_eq!(output, "![alt](https://example.com/img.png \"My title\")\n");
    }

    #[test]
    fn inline_image_label_lowercased_lookup() {
        // Label match is case-insensitive (defs map is keyed by lowercase).
        let defs = refdef_map(&[("img", "https://example.com/img.png")]);
        let input = "![Alt][IMG]\n";
        let output = inline_image_refs(input, &defs);
        assert_eq!(output, "![Alt](https://example.com/img.png)\n");
    }

    #[test]
    fn inline_image_no_def_unchanged() {
        let defs: HashMap<String, String> = HashMap::new();
        let input = "![alt][missing]\n";
        let output = inline_image_refs(input, &defs);
        assert_eq!(output, input);
    }

    #[test]
    fn inline_image_inside_code_fence_unchanged() {
        let defs = refdef_map(&[("img", "https://example.com/img.png")]);
        let input = "```\n![alt][img]\n```\n";
        let output = inline_image_refs(input, &defs);
        assert!(output.contains("![alt][img]"));
    }

    // ---- lowercase_refdef_label ----

    #[test]
    fn lowercase_refdef_label_basic() {
        assert_eq!(
            lowercase_refdef_label("[Foo]: https://example.com"),
            "[foo]: https://example.com"
        );
    }

    #[test]
    fn lowercase_refdef_label_preserves_title() {
        assert_eq!(
            lowercase_refdef_label("[Foo Bar]: https://example.com \"A Title\""),
            "[foo bar]: https://example.com \"A Title\""
        );
    }

    #[test]
    fn lowercase_refdef_label_passthrough_when_no_match() {
        // Defensive path: non-ref-def input is returned unchanged.
        assert_eq!(lowercase_refdef_label("not a refdef"), "not a refdef");
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
