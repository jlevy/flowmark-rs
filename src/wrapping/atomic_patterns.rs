//! Atomic pattern definitions for constructs that should not be broken during wrapping.
//!
//! Ported from Python: `flowmark/linewrapping/atomic_patterns.py`
//!
//! Note: Rust's `regex` crate does not support backreferences or lookahead.
//! Patterns are simplified to work without these features while maintaining
//! equivalent behavior for common cases.

use regex::Regex;
use std::sync::LazyLock;

/// Defines a regex pattern for an atomic construct that should not be broken.
#[derive(Debug)]
pub(crate) struct AtomicPattern {
    pub(crate) pattern: &'static str,
    pub(crate) open_delim: &'static str,
    pub(crate) close_delim: &'static str,
    pub(crate) open_re: &'static str,
    pub(crate) close_re: &'static str,
}

/// Jinja/Markdoc template tags: {% tag %}, {% /tag %}
pub(crate) static SINGLE_JINJA_TAG: AtomicPattern = AtomicPattern {
    pattern: r"\{%.*?%\}",
    open_delim: "{%",
    close_delim: "%}",
    open_re: r"\{%",
    close_re: r"%\}",
};

/// Jinja comments: {# comment #}
pub(crate) static SINGLE_JINJA_COMMENT: AtomicPattern = AtomicPattern {
    pattern: r"\{#.*?#\}",
    open_delim: "{#",
    close_delim: "#}",
    open_re: r"\{#",
    close_re: r"#\}",
};

/// Jinja variables: {{ variable }}
pub(crate) static SINGLE_JINJA_VAR: AtomicPattern = AtomicPattern {
    pattern: r"\{\{.*?\}\}",
    open_delim: "{{",
    close_delim: "}}",
    open_re: r"\{\{",
    close_re: r"\}\}",
};

/// HTML comments: <!-- comment -->
pub(crate) static SINGLE_HTML_COMMENT: AtomicPattern = AtomicPattern {
    pattern: r"<!--.*?-->",
    open_delim: "<!--",
    close_delim: "-->",
    open_re: r"<!--",
    close_re: r"-->",
};

/// HTML/XML open tags: <tag>
pub(crate) static HTML_OPEN_TAG: AtomicPattern = AtomicPattern {
    pattern: r"<[a-zA-Z][^>]*>",
    open_delim: "",
    close_delim: "",
    open_re: "",
    close_re: "",
};

/// HTML/XML close tags: </tag>
pub(crate) static HTML_CLOSE_TAG: AtomicPattern = AtomicPattern {
    pattern: r"</[a-zA-Z][^>]*>",
    open_delim: "",
    close_delim: "",
    open_re: "",
    close_re: "",
};

/// Compiled regex combining all atomic patterns with alternation.
///
/// Patterns are in priority order: code spans (multi-backtick before single),
/// links, paired tags, single tags, HTML tags.
///
/// Note: The Python version uses backreferences for code spans (`` (`+)...\1 ``).
/// Since Rust's regex crate doesn't support backreferences, we handle common
/// code span cases: double-backtick, then single-backtick.
///
/// Similarly, paired tag patterns use simplified matching without lookahead.
pub(crate) static ATOMIC_CONSTRUCT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    let patterns = [
        // Double-backtick code spans: ``code``
        r"``[^`]+``",
        // Single-backtick code spans: `code`
        r"`[^`]+`",
        // Markdown links: [text](url) or [text][ref] or [text]
        r"\[[^\]]*\](?:\([^)]*\)|\[[^\]]*\])?",
        // Paired Jinja tags: {% tag %}...{% /tag %}
        r"\{%[^%]*%\}\s*\{%\s*/[^%]*%\}",
        // Paired Jinja comments: {# tag #}...{# /tag #}
        r"\{#[^#]*#\}\s*\{#\s*/[^#]*#\}",
        // Paired Jinja vars: {{ tag }}...{{ /tag }}
        r"\{\{[^}]*\}\}\s*\{\{\s*/[^}]*\}\}",
        // Paired HTML comments: <!-- tag -->...<!-- /tag -->
        r"<!--[^-]*(?:-[^-]+)*-->\s*<!--\s*/[^-]*(?:-[^-]+)*-->",
        // Single Jinja tags
        SINGLE_JINJA_TAG.pattern,
        SINGLE_JINJA_COMMENT.pattern,
        SINGLE_JINJA_VAR.pattern,
        SINGLE_HTML_COMMENT.pattern,
        // HTML tags
        HTML_OPEN_TAG.pattern,
        HTML_CLOSE_TAG.pattern,
    ];
    // Use (?s) for DOTALL mode
    Regex::new(&format!("(?s){}", patterns.join("|")))
        .expect("valid ATOMIC_CONSTRUCT_PATTERN regex")
});
