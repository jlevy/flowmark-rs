//! Collision-safe parser substitution and fail-closed restoration.

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

#[derive(Debug, Clone)]
pub(crate) struct ProtectedSource {
    pub(crate) text: String,
    pub(crate) regions: Vec<ProtectedRegion>,
    pub(crate) tokens: Vec<String>,
    synthetic_block_suffixes: Vec<bool>,
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
            let token_end = text[start..]
                .char_indices()
                .nth(TOKEN_LENGTH)
                .map_or(text.len(), |(offset, _)| start + offset);
            let token = &text[start..token_end];
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
            if scalar != TOKEN_START {
                column += 1;
                position += scalar.len_utf8();
                continue;
            }
            let mut token_end = position;
            for _ in 0..TOKEN_LENGTH {
                let next = text[token_end..]
                    .chars()
                    .next()
                    .ok_or(PreservationError("malformed preservation token during wrapping"))?;
                token_end += next.len_utf8();
            }
            let index = parse_token(&text[position..token_end])?;
            let region = self
                .regions
                .get(index)
                .ok_or(PreservationError("unknown preservation token during wrapping"))?;
            if self.tokens[index] != text[position..token_end] {
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

fn escape_authored_markers(text: &str) -> String {
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
    output
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
    let scalars: Vec<char> = token.chars().collect();
    if scalars.len() != TOKEN_LENGTH
        || scalars.first() != Some(&TOKEN_START)
        || scalars.last() != Some(&TOKEN_END)
    {
        return Err(PreservationError("malformed preservation token"));
    }
    let mut bytes = [0_u8; INDEX_WIDTH];
    for (slot, scalar) in bytes.iter_mut().zip(&scalars[1..=INDEX_WIDTH]) {
        let value = u32::from(*scalar).checked_sub(INDEX_SCALAR_START);
        let value = value.and_then(|value| u8::try_from(value).ok());
        *slot = value.ok_or(PreservationError("malformed preservation token index"))?;
    }
    usize::try_from(u64::from_be_bytes(bytes))
        .map_err(|_| PreservationError("preservation token index exceeds platform size"))
}

pub(crate) fn protect_source(
    source: &NormalizedSource,
    regions: Vec<ProtectedRegion>,
) -> Result<ProtectedSource, PreservationError> {
    validate_regions(source, &regions)?;
    let tokens: Vec<String> = regions.iter().map(|region| encode_token(region.index)).collect();
    let synthetic_block_suffixes: Vec<bool> = regions
        .iter()
        .map(|region| {
            region.form == RegionForm::Block
                && region.end < source.text.len()
                && !source.text[region.end..].starts_with('\n')
        })
        .collect();
    let mut output = String::with_capacity(source.text.len());
    let mut previous_end = 0;
    for ((region, token), synthetic_suffix) in
        regions.iter().zip(&tokens).zip(&synthetic_block_suffixes)
    {
        output.push_str(&escape_authored_markers(&source.text[previous_end..region.start]));
        if region.form == RegionForm::Block {
            if region.scaffold_prefix.is_empty() && !output.is_empty() && !output.ends_with("\n\n")
            {
                // The Python protected-block adapter yields before an otherwise
                // adjacent token. Give comrak the equivalent block boundary.
                output.push('\n');
            }
            output.push_str(&region.scaffold_prefix);
            output.push_str(token);
            output.push('\n');
            if *synthetic_suffix {
                // Keep following Markdown out of comrak's token paragraph. Restoration
                // removes this parser-only blank line using the aligned side-table bit.
                output.push('\n');
            }
        } else {
            output.push_str(token);
        }
        previous_end = region.end;
    }
    output.push_str(&escape_authored_markers(&source.text[previous_end..]));
    Ok(ProtectedSource { text: output, regions, tokens, synthetic_block_suffixes })
}

fn parse_rendered_stream(rendered: &str) -> Result<(Vec<String>, Vec<usize>), PreservationError> {
    let scalars: Vec<char> = rendered.chars().collect();
    let mut gaps = Vec::new();
    let mut indexes = Vec::new();
    let mut gap = String::new();
    let mut position = 0;
    while position < scalars.len() {
        match scalars[position] {
            ESCAPE_MARKER => {
                let code = scalars
                    .get(position + 1)
                    .ok_or(PreservationError("malformed preservation marker escape"))?;
                gap.push(match *code {
                    ESCAPED_ESCAPE => ESCAPE_MARKER,
                    ESCAPED_START => TOKEN_START,
                    ESCAPED_END => TOKEN_END,
                    _ => return Err(PreservationError("malformed preservation marker escape")),
                });
                position += 2;
            }
            TOKEN_START => {
                let end = position + TOKEN_LENGTH;
                if end > scalars.len() {
                    return Err(PreservationError("malformed preservation token"));
                }
                let token: String = scalars[position..end].iter().collect();
                indexes.push(parse_token(&token)?);
                gaps.push(std::mem::take(&mut gap));
                position = end;
            }
            TOKEN_END => return Err(PreservationError("malformed preservation token")),
            scalar => {
                gap.push(scalar);
                position += 1;
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
        || protected.regions.len() != protected.synthetic_block_suffixes.len()
    {
        return Err(PreservationError("protected side-table lengths do not match"));
    }
    for (index, (region, token)) in protected.regions.iter().zip(&protected.tokens).enumerate() {
        if region.index != index || *token != encode_token(index) {
            return Err(PreservationError("protected side table is not canonical"));
        }
    }
    let (mut gaps, indexes) = parse_rendered_stream(rendered)?;
    if indexes != (0..protected.regions.len()).collect::<Vec<_>>() {
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
            if !gaps[index + 1].starts_with('\n') {
                return Err(PreservationError("protected block token lost its structural LF"));
            }
            gaps[index + 1].remove(0);
            if protected.synthetic_block_suffixes[index] {
                if !gaps[index + 1].starts_with('\n') {
                    return Err(PreservationError(
                        "protected block token lost its synthetic paragraph boundary",
                    ));
                }
                gaps[index + 1].remove(0);
            }
        }
    }
    let mut output = gaps[0].clone();
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
            escape_authored_markers(&format!("a{ESCAPE_MARKER}{TOKEN_START}{TOKEN_END}b"));
        let protected = ProtectedSource {
            text: escaped.clone(),
            regions: vec![],
            tokens: vec![],
            synthetic_block_suffixes: vec![],
        };
        let metrics = protected.measure_inline_text(&escaped).expect("valid authored escapes");
        assert_eq!(metrics.first_width, 5);
        assert_eq!(metrics.final_width, 5);
        assert!(!metrics.has_authored_break);
    }
}
