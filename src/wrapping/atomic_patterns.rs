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
/// Patterns are in priority order: code spans/fences (longest fence first),
/// links, paired tags, single tags, HTML tags.
///
/// Note: The Python version uses a backreference pattern for code spans.
/// Since Rust's regex crate doesn't support backreferences, we handle common
/// fence lengths explicitly: quadruple, triple, double, single backtick.
/// Triple/quadruple patterns also match fenced code blocks.
///
/// Similarly, paired tag patterns use simplified matching without lookahead.
pub(crate) static ATOMIC_CONSTRUCT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    let patterns = [
        // Quadruple-backtick fences/spans: ````content```` (lazy match)
        // These also match fenced code blocks, which Python's html_md_word_splitter
        // treats as atomic via backreferences. Likely a Python bug (fmr-5u8i) but
        // we match for parity.
        r"````[\s\S]*?````",
        // Triple-backtick fences/spans: ```content``` (lazy match)
        r"```[\s\S]*?```",
        // Double-backtick code spans: ``code``
        r"``[^`]+``",
        // Single-backtick code spans: `code`
        r"`[^`]+`",
        // Markdown links: [text](url) or [text][ref] or [text]
        r"\[[^\]]*\](?:\([^)]*\)|\[[^\]]*\])?",
        // Paired Jinja tags: {% tag %}...{% /tag %}
        // The opening tag must start with a letter (not `/`) to avoid
        // matching two closing tags as a pair.
        r"\{%\s+[a-zA-Z][^%]*%\}\s*\{%\s*/[^%]*%\}",
        // Paired Jinja comments: {# tag #}...{# /tag #}
        r"\{#\s*[a-zA-Z][^#]*#\}\s*\{#\s*/[^#]*#\}",
        // Paired Jinja vars: {{ tag }}...{{ /tag }}
        r"\{\{\s*[a-zA-Z][^}]*\}\}\s*\{\{\s*/[^}]*\}\}",
        // Paired HTML comments: <!-- tag -->...<!-- /tag -->
        r"<!--\s*[a-zA-Z:][^-]*(?:-[^-]+)*-->\s*<!--\s*/[^-]*(?:-[^-]+)*-->",
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
