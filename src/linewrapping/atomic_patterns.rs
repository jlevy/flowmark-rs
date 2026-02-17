//! Atomic pattern definitions for constructs that should not be broken during wrapping.
//!
//! Ported from Python: flowmark/linewrapping/atomic_patterns.py
//!
//! Each pattern defines a regex for a specific type of construct (code span, link,
//! template tag, etc.) that should be kept together as a single token during line wrapping.
//!
//! Uses `fancy_regex` for the combined ATOMIC_CONSTRUCT_PATTERN because it requires
//! lookahead assertions (`(?!...)`) which the standard `regex` crate does not support.
//! The TEMPLATE_TAG_PATTERN uses the standard `regex` crate since it only has simple patterns.

use fancy_regex::Regex as FancyRegex;
use once_cell::sync::Lazy;
use regex::Regex;

/// Defines a regex pattern for an atomic construct that should not be broken.
#[derive(Debug)]
pub struct AtomicPattern {
    pub name: &'static str,
    pub pattern: &'static str,
    pub open_delim: &'static str,
    pub close_delim: &'static str,
    pub open_re: &'static str,
    pub close_re: &'static str,
}

// --- Pattern definitions ---

/// Inline code spans with backticks (handles multi-backtick like ``code``)
/// Enumerates common backtick lengths (longest first) since the Rust regex ecosystem
/// handles this more reliably than backreferences for our use case.
pub static INLINE_CODE_SPAN: AtomicPattern = AtomicPattern {
    name: "inline_code_span",
    pattern: r"```(?:(?!```).)+```|``(?:(?!``).)+``|`(?!`)[^`]+`",
    open_delim: "",
    close_delim: "",
    open_re: "",
    close_re: "",
};

/// Markdown links: [text](url) or [text][ref] or [text]
pub static MARKDOWN_LINK: AtomicPattern = AtomicPattern {
    name: "markdown_link",
    pattern: r"\[[^\]]*\](?:\([^)]*\)|\[[^\]]*\])?",
    open_delim: "",
    close_delim: "",
    open_re: "",
    close_re: "",
};

/// Jinja/Markdoc template tags: {% tag %}, {% /tag %}
pub static SINGLE_JINJA_TAG: AtomicPattern = AtomicPattern {
    name: "single_jinja_tag",
    pattern: r"\{%.*?%\}",
    open_delim: "{%",
    close_delim: "%}",
    open_re: r"\{%",
    close_re: r"%\}",
};

/// Jinja comments: {# comment #}
pub static SINGLE_JINJA_COMMENT: AtomicPattern = AtomicPattern {
    name: "single_jinja_comment",
    pattern: r"\{#.*?#\}",
    open_delim: "{#",
    close_delim: "#}",
    open_re: r"\{#",
    close_re: r"#\}",
};

/// Jinja variables: {{ variable }}
pub static SINGLE_JINJA_VAR: AtomicPattern = AtomicPattern {
    name: "single_jinja_var",
    pattern: r"\{\{.*?\}\}",
    open_delim: "{{",
    close_delim: "}}",
    open_re: r"\{\{",
    close_re: r"\}\}",
};

/// HTML comments: <!-- comment -->
pub static SINGLE_HTML_COMMENT: AtomicPattern = AtomicPattern {
    name: "single_html_comment",
    pattern: r"<!--.*?-->",
    open_delim: "<!--",
    close_delim: "-->",
    open_re: r"<!--",
    close_re: r"-->",
};

/// HTML/XML open tags: <tag>
pub static HTML_OPEN_TAG: AtomicPattern = AtomicPattern {
    name: "html_open_tag",
    pattern: r"<[a-zA-Z][^>]*>",
    open_delim: "",
    close_delim: "",
    open_re: "",
    close_re: "",
};

/// HTML/XML close tags: </tag>
pub static HTML_CLOSE_TAG: AtomicPattern = AtomicPattern {
    name: "html_close_tag",
    pattern: r"</[a-zA-Z][^>]*>",
    open_delim: "",
    close_delim: "",
    open_re: "",
    close_re: "",
};

// --- Paired tag patterns ---

fn make_paired_pattern(open_re: &str, close_re: &str, middle_char: &str) -> String {
    format!(
        r"{}(?!\s*/)[^{}]*{}\s*{}\s*/[^{}]*{}",
        open_re, middle_char, close_re, open_re, middle_char, close_re
    )
}

/// Compiled regex combining all patterns with alternation.
/// Paired tags come before single tags for correct matching.
/// Uses `fancy_regex` because paired tag patterns use negative lookahead.
pub static ATOMIC_CONSTRUCT_PATTERN: Lazy<FancyRegex> = Lazy::new(|| {
    let paired_jinja_tag = make_paired_pattern(r"\{%", r"%\}", "%");
    let paired_jinja_comment = make_paired_pattern(r"\{#", r"#\}", "#");
    let paired_jinja_var = make_paired_pattern(r"\{\{", r"\}\}", "}");
    let paired_html_comment =
        r"<!--(?!\s*/)[^-]*(?:-[^-]+)*-->\s*<!--\s*/[^-]*(?:-[^-]+)*-->".to_string();

    let patterns = [
        INLINE_CODE_SPAN.pattern.to_string(),
        MARKDOWN_LINK.pattern.to_string(),
        // Paired tags must come before single tags
        paired_jinja_tag,
        paired_jinja_comment,
        paired_jinja_var,
        paired_html_comment,
        // Single tags
        SINGLE_JINJA_TAG.pattern.to_string(),
        SINGLE_JINJA_COMMENT.pattern.to_string(),
        SINGLE_JINJA_VAR.pattern.to_string(),
        SINGLE_HTML_COMMENT.pattern.to_string(),
        // HTML tags
        HTML_OPEN_TAG.pattern.to_string(),
        HTML_CLOSE_TAG.pattern.to_string(),
    ];

    // (?s) enables DOTALL mode (. matches \n) to match Python's re.DOTALL
    FancyRegex::new(&format!("(?s){}", patterns.join("|"))).unwrap()
});

/// Template tag pattern (single tags only, for smartquotes protection).
/// Uses standard `regex` crate since these patterns don't need lookahead.
pub static TEMPLATE_TAG_PATTERN: Lazy<Regex> = Lazy::new(|| {
    let patterns = [
        SINGLE_JINJA_TAG.pattern,
        SINGLE_JINJA_COMMENT.pattern,
        SINGLE_JINJA_VAR.pattern,
        SINGLE_HTML_COMMENT.pattern,
    ];
    Regex::new(&patterns.join("|")).unwrap()
});
