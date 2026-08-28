//! Shared knowledge of the PUA escape placeholders installed by COMRAK-WORKAROUND4.
//!
//! Comrak strips backslash escapes while building its AST, so `fill_markdown` rewrites
//! every `\<c>` in the source to a two-scalar placeholder pair before parsing:
//! `U+E000 + c` followed by the [`ESCAPE_PLACEHOLDER_FILLER`]. The pair is restored to
//! `\<c>` after rendering.
//!
//! The pair is two scalars wide so it occupies the same number of columns as the `\<c>`
//! it stands in for. That holds for every escape except the period: COMRAK-WORKAROUND11
//! (`postprocess_period_escapes`) drops `\.` back to `.` after rendering, keeping the
//! backslash only where a line begins `DIGITS\.` and would otherwise parse as an ordered
//! list marker.
//!
//! A period pair is therefore the one placeholder whose rendered width depends on where it
//! lands: one column in general, two at the start of a line it keeps its backslash on. Line
//! wrapping runs *before* the cleanup, so it has to ask for the right one. Measuring the raw
//! scalar count everywhere breaks a line one word early, and the word pushed to the new line
//! start is then re-escaped, so the premature break survives every subsequent reformat.
//! Measuring one column everywhere is the mirror-image error: the retained backslash is
//! unaccounted for and the line overruns the wrap width by a column.

/// First scalar of an escape placeholder pair: `U+E000 + <escaped ASCII char>`.
pub(crate) const ESCAPE_PLACEHOLDER_BASE: u32 = 0xE000;

/// Second scalar of every escape placeholder pair.
pub(crate) const ESCAPE_PLACEHOLDER_FILLER: char = '\u{E100}';

/// Placeholder for `\.` (`U+E000 + '.'`).
pub(crate) const PERIOD_ESCAPE_PLACEHOLDER: char = '\u{E02E}';

/// Columns a placeholder pair beginning with `placeholder` occupies once rendered.
///
/// Every escape survives as `\<c>` (two columns) except the period, which
/// `postprocess_period_escapes` reduces to a bare `.` outside `DIGITS\.` line starts.
pub(crate) const fn placeholder_pair_width(placeholder: char) -> usize {
    if matches!(placeholder, PERIOD_ESCAPE_PLACEHOLDER) { 1 } else { 2 }
}

/// Whether `scalar` opens an escape placeholder pair.
pub(crate) fn is_escape_placeholder(scalar: char) -> bool {
    ('\u{E000}'..='\u{E0FF}').contains(&scalar)
}

/// Column width of `text` as it will appear after post-render escape cleanup.
///
/// Identical to `text.chars().count()` except that escape placeholder pairs are measured
/// at their rendered width.
pub(crate) fn rendered_width(text: &str) -> usize {
    // Fast path: the placeholders only ever appear in text that carries the filler.
    if !text.contains(ESCAPE_PLACEHOLDER_FILLER) {
        return text.chars().count();
    }

    let mut width = 0;
    let mut chars = text.chars().peekable();
    while let Some(scalar) = chars.next() {
        if is_escape_placeholder(scalar) && chars.peek() == Some(&ESCAPE_PLACEHOLDER_FILLER) {
            chars.next();
            width += placeholder_pair_width(scalar);
        } else {
            width += 1;
        }
    }
    width
}

/// Whether `text` placed at the start of a line keeps a leading period escape.
///
/// Mirrors the one case `postprocess_period_escapes` preserves: a line that begins
/// `DIGITS\.`, which would otherwise parse as an ordered list marker. Everywhere else the
/// backslash is dropped, so only this shape is a column wider than [`rendered_width`] says.
pub(crate) fn retains_leading_period_escape(text: &str) -> bool {
    let mut chars = text.chars();
    let mut digits = 0usize;
    loop {
        match chars.next() {
            Some(scalar) if scalar.is_ascii_digit() => digits += 1,
            Some(PERIOD_ESCAPE_PLACEHOLDER) => {
                return digits > 0 && chars.next() == Some(ESCAPE_PLACEHOLDER_FILLER);
            }
            _ => return false,
        }
    }
}

/// Column width of `text` when it starts a line, where a `DIGITS\.` escape is retained.
pub(crate) fn rendered_width_at_line_start(text: &str) -> usize {
    rendered_width(text) + usize::from(retains_leading_period_escape(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_measures_by_scalar_count() {
        assert_eq!(rendered_width("hello"), 5);
        assert_eq!(rendered_width(""), 0);
    }

    #[test]
    fn period_pair_measures_as_the_bare_period() {
        // `5\.` protected as placeholders renders as `5.` outside a DIGITS\. line start.
        let protected = format!("5{PERIOD_ESCAPE_PLACEHOLDER}{ESCAPE_PLACEHOLDER_FILLER}");
        assert_eq!(protected.chars().count(), 3);
        assert_eq!(rendered_width(&protected), 2);
    }

    #[test]
    fn other_escape_pairs_keep_their_backslash_width() {
        // `\*` survives post-processing, so the pair stays two columns wide.
        let star = char::from_u32(ESCAPE_PLACEHOLDER_BASE + u32::from(b'*')).expect("valid PUA");
        let protected = format!("a{star}{ESCAPE_PLACEHOLDER_FILLER}");
        assert_eq!(rendered_width(&protected), 3);
    }

    #[test]
    fn placeholder_without_filler_is_left_alone() {
        let protected = format!("a{PERIOD_ESCAPE_PLACEHOLDER}b");
        assert_eq!(rendered_width(&protected), 3);
    }

    #[test]
    fn period_placeholder_matches_the_documented_encoding() {
        assert_eq!(
            char::from_u32(ESCAPE_PLACEHOLDER_BASE + u32::from(b'.')),
            Some(PERIOD_ESCAPE_PLACEHOLDER)
        );
    }

    #[test]
    fn digits_then_period_pair_keeps_its_backslash_at_a_line_start() {
        let protected = format!("5{PERIOD_ESCAPE_PLACEHOLDER}{ESCAPE_PLACEHOLDER_FILLER}");
        assert!(retains_leading_period_escape(&protected));
        // Mid-line the backslash is dropped; at a line start it is not.
        assert_eq!(rendered_width(&protected), 2);
        assert_eq!(rendered_width_at_line_start(&protected), 3);
    }

    #[test]
    fn a_period_pair_not_preceded_by_digits_is_dropped_everywhere() {
        let protected = format!("a{PERIOD_ESCAPE_PLACEHOLDER}{ESCAPE_PLACEHOLDER_FILLER}");
        assert!(!retains_leading_period_escape(&protected));
        assert_eq!(rendered_width_at_line_start(&protected), rendered_width(&protected));
    }

    #[test]
    fn a_leading_period_pair_with_no_digits_is_dropped() {
        let protected = format!("{PERIOD_ESCAPE_PLACEHOLDER}{ESCAPE_PLACEHOLDER_FILLER}x");
        assert!(!retains_leading_period_escape(&protected));
    }

    #[test]
    fn other_leading_escapes_do_not_trigger_the_line_start_adjustment() {
        let star = char::from_u32(ESCAPE_PLACEHOLDER_BASE + u32::from(b'*')).expect("valid PUA");
        let protected = format!("5{star}{ESCAPE_PLACEHOLDER_FILLER}");
        assert!(!retains_leading_period_escape(&protected));
    }
}
