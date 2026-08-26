//! Stable recognizer precedence shared with the Python implementation.

use super::model::RegionKind;

pub(crate) const fn priority(kind: RegionKind) -> u8 {
    match kind {
        RegionKind::MathGitlabInline | RegionKind::MathMystInline => 10,
        RegionKind::CodeSpan => 20,
        RegionKind::MathParenInline | RegionKind::MathEnvironmentInline => 30,
        RegionKind::MathDollarInline | RegionKind::MathDoubleDollarInline => 40,
        RegionKind::PandocMultilineTable
        | RegionKind::ObsidianCallout
        | RegionKind::ColonContainer
        | RegionKind::TomlFrontmatter => 45,
        RegionKind::MathDollarBlock
        | RegionKind::MathBracketBlock
        | RegionKind::MathEnvironmentBlock => 50,
    }
}

pub(crate) const fn stable_name(kind: RegionKind) -> &'static str {
    match kind {
        RegionKind::MathGitlabInline => "math_gitlab_inline",
        RegionKind::MathMystInline => "math_myst_inline",
        RegionKind::CodeSpan => "code_span",
        RegionKind::MathParenInline => "math_paren_inline",
        RegionKind::MathEnvironmentInline => "math_environment_inline",
        RegionKind::MathDollarInline => "math_dollar_inline",
        RegionKind::MathDoubleDollarInline => "math_double_dollar_inline",
        RegionKind::PandocMultilineTable => "pandoc_multiline_table",
        RegionKind::ObsidianCallout => "obsidian_callout",
        RegionKind::ColonContainer => "colon_container",
        RegionKind::TomlFrontmatter => "toml_frontmatter",
        RegionKind::MathDollarBlock => "math_dollar_block",
        RegionKind::MathBracketBlock => "math_bracket_block",
        RegionKind::MathEnvironmentBlock => "math_environment_block",
    }
}
