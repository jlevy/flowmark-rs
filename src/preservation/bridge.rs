//! Collision-safe parser substitution and fail-closed restoration.

use std::borrow::Cow;

use crate::escape_placeholders::{
    ESCAPE_PLACEHOLDER_FILLER, PERIOD_ESCAPE_PLACEHOLDER, placeholder_pair_width,
};

use super::model::{
    NormalizedSource, PreservationError, ProtectedRegion, RegionForm, RegionKind, validate_regions,
};

const ESCAPE_MARKER: char = '\u{f0000}';
const TOKEN_START: char = '\u{f0001}';
const TOKEN_END: char = '\u{f0002}';
const ESCAPED_ESCAPE: char = '\u{f0003}';
const ESCAPED_START: char = '\u{f0004}';
const ESCAPED_END: char = '\u{f0005}';
const INDEX_SCALAR_START: u32 = 0xF0100;
const INDEX_WIDTH: usize = 8;
const TOKEN_LENGTH: usize = INDEX_WIDTH + 2;
const TOKEN_BYTE_LENGTH: usize = TOKEN_LENGTH * 4;

#[derive(Debug, Clone)]
pub(crate) struct ProtectedSource {
    pub(crate) text: String,
    pub(crate) regions: Vec<ProtectedRegion>,
    pub(crate) tokens: Vec<String>,
    synthetic_block_prefixes: Vec<bool>,
    synthetic_block_suffixes: Vec<bool>,
    /// 1-based line numbers in `text` holding a blank line this bridge inserted.
    ///
    /// The reference implementation teaches its parser about block tokens directly, so
    /// it needs no such line. comrak cannot be extended that way, so these lines stand
    /// in for the parser break — and, like the break they replace, they must leave no
    /// trace in the output. See `repair_synthetic_list_looseness`.
    synthetic_blank_lines: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProtectedMetrics {
    pub(crate) first_width: usize,
    pub(crate) final_width: usize,
    pub(crate) has_authored_break: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InlineRewriteSegment<'a> {
    Mutable(&'a str),
    Immutable { source: &'a str, context: String },
}

impl ProtectedSource {
    /// 1-based protected-text line numbers holding a bridge-inserted blank line.
    pub(crate) fn synthetic_blank_lines(&self) -> &[usize] {
        &self.synthetic_blank_lines
    }

    /// Split parser-facing text into mutable gaps and immutable preservation tokens.
    ///
    /// Code-span tokens contribute their CommonMark-normalized body only as rewrite
    /// context. The authored token itself remains immutable and is restored from the
    /// side table after rendering.
    pub(crate) fn inline_rewrite_segments<'a>(
        &self,
        text: &'a str,
    ) -> Result<Vec<InlineRewriteSegment<'a>>, PreservationError> {
        let mut segments = Vec::new();
        let mut previous_end = 0;
        for (start, _) in text.match_indices(TOKEN_START) {
            if start < previous_end {
                continue;
            }
            if start > previous_end {
                segments.push(InlineRewriteSegment::Mutable(&text[previous_end..start]));
            }
            let token_end = start
                .checked_add(TOKEN_BYTE_LENGTH)
                .ok_or(PreservationError("malformed preservation token during rewrite"))?;
            let token = text
                .get(start..token_end)
                .ok_or(PreservationError("malformed preservation token during rewrite"))?;
            let index = parse_token(token)?;
            let region = self
                .regions
                .get(index)
                .ok_or(PreservationError("unknown preservation token during rewrite"))?;
            if self.tokens.get(index).map(String::as_str) != Some(token) {
                return Err(PreservationError("unknown token reached inline rewrite"));
            }
            let context = if region.kind == RegionKind::CodeSpan {
                code_span_rewrite_context(&region.source)?
            } else {
                token.to_owned()
            };
            segments.push(InlineRewriteSegment::Immutable { source: token, context });
            previous_end = token_end;
        }
        if previous_end < text.len() {
            segments.push(InlineRewriteSegment::Mutable(&text[previous_end..]));
        }
        Ok(segments)
    }

    pub(crate) fn measure_inline_text(
        &self,
        text: &str,
    ) -> Result<ProtectedMetrics, PreservationError> {
        let mut column = 0;
        let mut first_width = None;
        let mut has_authored_break = false;
        let mut position = 0;
        while position < text.len() {
            let scalar = text[position..]
                .chars()
                .next()
                .ok_or(PreservationError("invalid wrapping position"))?;
            if scalar == ESCAPE_MARKER {
                let marker_end = position + scalar.len_utf8();
                let code = text[marker_end..]
                    .chars()
                    .next()
                    .ok_or(PreservationError("malformed preservation marker escape"))?;
                if !matches!(code, ESCAPED_ESCAPE | ESCAPED_START | ESCAPED_END) {
                    return Err(PreservationError("malformed preservation marker escape"));
                }
                column += 1;
                position = marker_end + code.len_utf8();
                continue;
            }
            if scalar == PERIOD_ESCAPE_PLACEHOLDER {
                let pair_end = position + scalar.len_utf8();
                if text[pair_end..].starts_with(ESCAPE_PLACEHOLDER_FILLER) {
                    column += placeholder_pair_width(scalar);
                    position = pair_end + ESCAPE_PLACEHOLDER_FILLER.len_utf8();
                    continue;
                }
            }
            if scalar != TOKEN_START {
                column += 1;
                position += scalar.len_utf8();
                continue;
            }
            let token_end = position
                .checked_add(TOKEN_BYTE_LENGTH)
                .ok_or(PreservationError("malformed preservation token during wrapping"))?;
            let token = text
                .get(position..token_end)
                .ok_or(PreservationError("malformed preservation token during wrapping"))?;
            let index = parse_token(token)?;
            let region = self
                .regions
                .get(index)
                .ok_or(PreservationError("unknown preservation token during wrapping"))?;
            if self.tokens[index] != token {
                return Err(PreservationError("unknown token reached inline wrapping"));
            }
            if region.form == RegionForm::Block {
                // comrak represents a standalone inert block line as a paragraph.
                // Its body is still opaque and line-boundary validation happens on
                // restoration, so expose only a minimal indivisible width here.
                column += 1;
                position = token_end;
                continue;
            }
            let widths = &region.logical_widths;
            let first = *widths
                .first()
                .ok_or(PreservationError("protected inline token has no width metadata"))?;
            column += first;
            if widths.len() > 1 {
                first_width.get_or_insert(column);
                has_authored_break = true;
                column = *widths.last().expect("non-empty widths");
            }
            position = token_end;
        }
        Ok(ProtectedMetrics {
            first_width: first_width.unwrap_or(column),
            final_width: column,
            has_authored_break,
        })
    }
}

fn code_span_rewrite_context(source: &str) -> Result<String, PreservationError> {
    let delimiter_width = source.bytes().take_while(|byte| *byte == b'`').count();
    if delimiter_width == 0
        || source.len() < delimiter_width * 2
        || !source.ends_with(&"`".repeat(delimiter_width))
    {
        return Err(PreservationError("protected code span has malformed delimiters"));
    }
    let mut body = source[delimiter_width..source.len() - delimiter_width].replace('\n', " ");
    if !body.trim().is_empty() && body.starts_with(' ') && body.ends_with(' ') {
        body.remove(0);
        body.pop();
    }
    Ok(body)
}

fn escape_authored_markers(text: &str) -> Cow<'_, str> {
    if !text.contains(ESCAPE_MARKER) && !text.contains(TOKEN_START) && !text.contains(TOKEN_END) {
        return Cow::Borrowed(text);
    }
    let mut output = String::with_capacity(text.len());
    for scalar in text.chars() {
        let escaped = match scalar {
            ESCAPE_MARKER => Some(ESCAPED_ESCAPE),
            TOKEN_START => Some(ESCAPED_START),
            TOKEN_END => Some(ESCAPED_END),
            _ => None,
        };
        if let Some(code) = escaped {
            output.push(ESCAPE_MARKER);
            output.push(code);
        } else {
            output.push(scalar);
        }
    }
    Cow::Owned(output)
}

pub(crate) fn encode_token(index: usize) -> String {
    let index = u64::try_from(index).expect("preservation index must fit in u64");
    let mut token = String::with_capacity(TOKEN_LENGTH * 4);
    token.push(TOKEN_START);
    for byte in index.to_be_bytes() {
        token.push(char::from_u32(INDEX_SCALAR_START + u32::from(byte)).expect("valid PUA scalar"));
    }
    token.push(TOKEN_END);
    token
}

fn parse_token(token: &str) -> Result<usize, PreservationError> {
    let mut scalars = token.chars();
    if scalars.next() != Some(TOKEN_START) {
        return Err(PreservationError("malformed preservation token"));
    }
    let mut bytes = [0_u8; INDEX_WIDTH];
    for slot in &mut bytes {
        let scalar = scalars.next().ok_or(PreservationError("malformed preservation token"))?;
        let value = u32::from(scalar).checked_sub(INDEX_SCALAR_START);
        let value = value.and_then(|value| u8::try_from(value).ok());
        *slot = value.ok_or(PreservationError("malformed preservation token index"))?;
    }
    if scalars.next() != Some(TOKEN_END) || scalars.next().is_some() {
        return Err(PreservationError("malformed preservation token"));
    }
    usize::try_from(u64::from_be_bytes(bytes))
        .map_err(|_| PreservationError("preservation token index exceeds platform size"))
}

/// 1-based line number of the empty line just closed by the trailing LF of `output`.
///
/// Called immediately after pushing the LF that terminates a synthetic blank line, so
/// the blank line is the one ending at `output.len()`: one less than the line the next
/// character would open.
fn line_number_of_last_line(output: &str) -> usize {
    output.bytes().filter(|byte| *byte == b'\n').count()
}

pub(crate) fn protect_source(
    source: &NormalizedSource,
    regions: Vec<ProtectedRegion>,
) -> Result<ProtectedSource, PreservationError> {
    validate_regions(source, &regions)?;
    let tokens: Vec<String> = regions.iter().map(|region| encode_token(region.index)).collect();
    let synthetic_block_prefixes: Vec<bool> = regions
        .iter()
        .map(|region| {
            region.form == RegionForm::Block
                && region.start > 0
                && source.text.as_bytes()[region.start - 1] == b'\n'
                && (region.start < 2 || source.text.as_bytes()[region.start - 2] != b'\n')
        })
        .collect();
    let synthetic_block_suffixes: Vec<bool> = regions
        .iter()
        .map(|region| {
            region.form == RegionForm::Block
                && region.end < source.text.len()
                && !source.text[region.end..].starts_with('\n')
        })
        .collect();
    let mut output = String::with_capacity(source.text.len());
    let mut synthetic_blank_lines = Vec::new();
    let mut previous_end = 0;
    for ((region, token), synthetic_suffix) in
        regions.iter().zip(&tokens).zip(&synthetic_block_suffixes)
    {
        output.push_str(&escape_authored_markers(&source.text[previous_end..region.start]));
        if region.form == RegionForm::Block {
            if !output.is_empty() && !output.ends_with("\n\n") {
                // The Python protected-block adapter yields before an otherwise
                // adjacent token. Give comrak the equivalent block boundary even
                // inside a quote or list scaffold; otherwise it can merge the token
                // into the preceding paragraph and restoration would discard prose.
                output.push('\n');
                synthetic_blank_lines.push(line_number_of_last_line(&output));
            }
            output.push_str(&region.scaffold_prefix);
            output.push_str(token);
            output.push('\n');
            if *synthetic_suffix {
                // Keep following Markdown out of comrak's token paragraph. Restoration
                // removes this parser-only blank line using the aligned side-table bit.
                output.push('\n');
                synthetic_blank_lines.push(line_number_of_last_line(&output));
            }
        } else {
            output.push_str(token);
        }
        previous_end = region.end;
    }
    output.push_str(&escape_authored_markers(&source.text[previous_end..]));
    Ok(ProtectedSource {
        text: output,
        regions,
        tokens,
        synthetic_block_prefixes,
        synthetic_block_suffixes,
        synthetic_blank_lines,
    })
}

fn parse_rendered_stream(rendered: &str) -> Result<(Vec<String>, Vec<usize>), PreservationError> {
    if !rendered.contains(ESCAPE_MARKER)
        && !rendered.contains(TOKEN_START)
        && !rendered.contains(TOKEN_END)
    {
        return Ok((vec![rendered.to_owned()], Vec::new()));
    }

    let mut gaps = Vec::new();
    let mut indexes = Vec::new();
    let mut gap = String::new();
    let mut position = 0;
    while position < rendered.len() {
        let scalar = rendered[position..]
            .chars()
            .next()
            .ok_or(PreservationError("invalid rendered UTF-8 position"))?;
        match scalar {
            ESCAPE_MARKER => {
                let code_start = position + scalar.len_utf8();
                let code = rendered[code_start..]
                    .chars()
                    .next()
                    .ok_or(PreservationError("malformed preservation marker escape"))?;
                gap.push(match code {
                    ESCAPED_ESCAPE => ESCAPE_MARKER,
                    ESCAPED_START => TOKEN_START,
                    ESCAPED_END => TOKEN_END,
                    _ => return Err(PreservationError("malformed preservation marker escape")),
                });
                position = code_start + code.len_utf8();
            }
            TOKEN_START => {
                let token_end = position
                    .checked_add(TOKEN_BYTE_LENGTH)
                    .ok_or(PreservationError("malformed preservation token"))?;
                let token = rendered
                    .get(position..token_end)
                    .ok_or(PreservationError("malformed preservation token"))?;
                indexes.push(parse_token(token)?);
                gaps.push(std::mem::take(&mut gap));
                position = token_end;
            }
            TOKEN_END => return Err(PreservationError("malformed preservation token")),
            _ => {
                let marker_start = rendered[position..]
                    .find([ESCAPE_MARKER, TOKEN_START, TOKEN_END])
                    .map_or(rendered.len(), |relative| position + relative);
                gap.push_str(&rendered[position..marker_start]);
                position = marker_start;
            }
        }
    }
    gaps.push(gap);
    Ok((gaps, indexes))
}

pub(crate) fn restore_source(
    rendered: &str,
    protected: &ProtectedSource,
) -> Result<String, PreservationError> {
    if protected.regions.len() != protected.tokens.len()
        || protected.regions.len() != protected.synthetic_block_prefixes.len()
        || protected.regions.len() != protected.synthetic_block_suffixes.len()
    {
        return Err(PreservationError("protected side-table lengths do not match"));
    }
    for (index, (region, token)) in protected.regions.iter().zip(&protected.tokens).enumerate() {
        if region.index != index || parse_token(token)? != index {
            return Err(PreservationError("protected side table is not canonical"));
        }
    }
    let (mut gaps, indexes) = parse_rendered_stream(rendered)?;
    if indexes.len() != protected.regions.len()
        || indexes.iter().copied().ne(0..protected.regions.len())
    {
        return Err(PreservationError(
            "preservation tokens are missing, duplicated, reordered, or malformed",
        ));
    }
    for (index, region) in protected.regions.iter().enumerate() {
        if region.form == RegionForm::Block {
            // comrak renders a structural quote/list prefix around the inert token.
            // The authored prefix belongs to `region.source`; discard only the
            // renderer-owned suffix on the token's physical line.
            if let Some(line_break) = gaps[index].rfind('\n') {
                gaps[index].truncate(line_break + 1);
            } else {
                gaps[index].clear();
            }
            if protected.synthetic_block_prefixes[index] && gaps[index].ends_with("\n\n") {
                gaps[index].pop();
            }
            if !gaps[index + 1].starts_with('\n') {
                return Err(PreservationError("protected block token lost its structural LF"));
            }
            gaps[index + 1].remove(0);
            // Remove the synthetic paragraph boundary when the renderer kept it. A
            // tight list drops it on its own, and there is then nothing to take back.
            if protected.synthetic_block_suffixes[index] && gaps[index + 1].starts_with('\n') {
                gaps[index + 1].remove(0);
            }
        }
    }
    let capacity = gaps.iter().map(String::len).sum::<usize>()
        + protected.regions.iter().map(|region| region.source.len()).sum::<usize>();
    let mut output = String::with_capacity(capacity);
    output.push_str(&gaps[0]);
    for (index, region) in protected.regions.iter().enumerate() {
        output.push_str(&region.source);
        output.push_str(&gaps[index + 1]);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_width_token_round_trips_u64_indexes() {
        for index in [0, 1, 255, 256, usize::MAX] {
            let token = encode_token(index);
            assert_eq!(token.chars().count(), TOKEN_LENGTH);
            assert_eq!(parse_token(&token), Ok(index));
        }
    }

    #[test]
    fn authored_marker_escapes_retain_one_column_each() {
        let escaped =
            escape_authored_markers(&format!("a{ESCAPE_MARKER}{TOKEN_START}{TOKEN_END}b"))
                .into_owned();
        let protected = ProtectedSource {
            text: escaped.clone(),
            regions: vec![],
            tokens: vec![],
            synthetic_block_prefixes: vec![],
            synthetic_block_suffixes: vec![],
            synthetic_blank_lines: vec![],
        };
        let metrics = protected.measure_inline_text(&escaped).expect("valid authored escapes");
        assert_eq!(metrics.first_width, 5);
        assert_eq!(metrics.final_width, 5);
        assert!(!metrics.has_authored_break);
    }

    #[test]
    fn marker_free_text_uses_bulk_fast_paths() {
        let escaped = escape_authored_markers("ordinary text");
        assert!(matches!(escaped, Cow::Borrowed("ordinary text")));

        let (gaps, indexes) = parse_rendered_stream("ordinary rendered text")
            .expect("marker-free rendered text is valid");
        assert_eq!(gaps, ["ordinary rendered text"]);
        assert!(indexes.is_empty());
    }

    #[test]
    fn restoration_removes_only_a_parser_synthesized_block_prefix_blank() {
        use crate::preservation::{normalize_source, scan_protected_regions};

        let source = normalize_source("Before\n$$\nbody\n$$\nAfter");
        let regions = scan_protected_regions(&source).expect("valid scan");
        let protected = protect_source(&source, regions).expect("valid protection");

        assert_eq!(
            restore_source(&protected.text, &protected).expect("valid restoration"),
            "Before\n$$\nbody\n$$\nAfter\n"
        );
    }
}
