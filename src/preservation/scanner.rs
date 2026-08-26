//! Portable, parser-independent scanner for source-exact math regions.

use std::collections::HashMap;

use super::model::{
    Candidate, ContainerContext, NormalizedSource, PreservationError, ProtectedRegion, RegionKind,
};
use super::registry::{priority, stable_name};

type ByteRange = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ContainerKind {
    Quote,
    ListItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ContainerFrame {
    kind: ContainerKind,
    identity: usize,
    content_column: usize,
}

#[derive(Debug, Clone)]
struct Line {
    index: usize,
    start: usize,
    end: usize,
    content_end: usize,
    content_start: usize,
    logical_column: usize,
    context: ContainerContext,
    key: Vec<ContainerFrame>,
    frames: Vec<ContainerFrame>,
    lazy: bool,
    starts_list: bool,
}

impl Line {
    fn payload<'a>(&self, source: &'a str) -> &'a str {
        let (start, end) = line_payload_bounds(source, self);
        source[start..end].trim_end_matches([' ', '\t'])
    }

    fn exact_payload<'a>(&self, source: &'a str) -> &'a str {
        let (start, end) = line_payload_bounds(source, self);
        &source[start..end]
    }

    fn scaffold<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.content_start]
    }

    fn is_blank(&self, source: &str) -> bool {
        self.payload(source).is_empty()
    }
}

fn advance_column(column: usize, byte: u8) -> usize {
    if byte == b'\t' { column + (4 - column % 4) } else { column + 1 }
}

fn consume_whitespace(
    bytes: &[u8],
    mut index: usize,
    end: usize,
    mut column: usize,
) -> (usize, usize) {
    while index < end && matches!(bytes[index], b' ' | b'\t') {
        column = advance_column(column, bytes[index]);
        index += 1;
    }
    (index, column)
}

fn consume_indent(
    bytes: &[u8],
    mut index: usize,
    end: usize,
    mut column: usize,
    maximum: usize,
) -> (usize, usize) {
    let origin = column;
    while index < end && matches!(bytes[index], b' ' | b'\t') {
        let next_column = advance_column(column, bytes[index]);
        if next_column - origin > maximum {
            break;
        }
        index += 1;
        column = next_column;
    }
    (index, column)
}

fn consume_to_column(
    bytes: &[u8],
    mut index: usize,
    end: usize,
    mut column: usize,
    target: usize,
) -> Option<(usize, usize)> {
    while index < end && matches!(bytes[index], b' ' | b'\t') && column < target {
        column = advance_column(column, bytes[index]);
        index += 1;
    }
    (column >= target).then_some((index, column))
}

fn consume_quote_marker(
    bytes: &[u8],
    index: usize,
    end: usize,
    column: usize,
) -> Option<(usize, usize, usize)> {
    let (mut marker_index, mut marker_column) = consume_indent(bytes, index, end, column, 3);
    let identity = marker_index;
    if marker_index >= end || bytes[marker_index] != b'>' {
        return None;
    }
    marker_index += 1;
    marker_column += 1;
    if marker_index < end && matches!(bytes[marker_index], b' ' | b'\t') {
        marker_column = advance_column(marker_column, bytes[marker_index]);
        marker_index += 1;
    }
    Some((marker_index, marker_column, identity))
}

fn list_marker_end(bytes: &[u8], index: usize, end: usize) -> Option<usize> {
    if index >= end {
        return None;
    }
    let marker_end = if matches!(bytes[index], b'-' | b'*' | b'+') {
        index + 1
    } else if bytes[index].is_ascii_digit() {
        let mut cursor = index;
        while cursor < end && cursor - index < 9 && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor >= end || !matches!(bytes[cursor], b'.' | b')') {
            return None;
        }
        cursor + 1
    } else {
        return None;
    };
    (marker_end == end || matches!(bytes[marker_end], b' ' | b'\t')).then_some(marker_end)
}

fn consume_list_marker(
    bytes: &[u8],
    index: usize,
    end: usize,
    column: usize,
) -> Option<(usize, usize, usize, usize)> {
    let (marker_index, marker_column) = consume_indent(bytes, index, end, column, 3);
    let marker_end = list_marker_end(bytes, marker_index, end)?;
    let after_marker_column = marker_column + marker_end - marker_index;
    if marker_end == end {
        return Some((marker_end, after_marker_column, after_marker_column + 1, marker_index));
    }

    let (padding_end, padding_column) =
        consume_whitespace(bytes, marker_end, end, after_marker_column);
    let padding_width = padding_column - after_marker_column;
    if padding_end < end && padding_width <= 4 {
        Some((padding_end, padding_column, padding_column, marker_index))
    } else {
        let content_column = advance_column(after_marker_column, bytes[marker_end]);
        Some((marker_end + 1, content_column, content_column, marker_index))
    }
}

fn match_frame(
    bytes: &[u8],
    index: usize,
    end: usize,
    column: usize,
    frame: ContainerFrame,
) -> Option<(usize, usize)> {
    match frame.kind {
        ContainerKind::Quote => consume_quote_marker(bytes, index, end, column)
            .map(|(index, column, _)| (index, column)),
        ContainerKind::ListItem => {
            consume_to_column(bytes, index, end, column, frame.content_column)
        }
    }
}

fn parse_new_frames(
    bytes: &[u8],
    mut index: usize,
    end: usize,
    mut column: usize,
    frames: &mut Vec<ContainerFrame>,
) -> (usize, usize, bool) {
    let original_len = frames.len();
    while index < end {
        if let Some((next, next_column, identity)) = consume_quote_marker(bytes, index, end, column)
        {
            frames.push(ContainerFrame { kind: ContainerKind::Quote, identity, content_column: 0 });
            index = next;
            column = next_column;
            continue;
        }
        let Some((next, next_column, content_column, identity)) =
            consume_list_marker(bytes, index, end, column)
        else {
            break;
        };
        frames.push(ContainerFrame { kind: ContainerKind::ListItem, identity, content_column });
        index = next;
        column = next_column;
    }
    let starts_list =
        frames[original_len..].iter().any(|frame| frame.kind == ContainerKind::ListItem);
    (index, column, starts_list)
}

fn container_context(frames: &[ContainerFrame]) -> ContainerContext {
    let blockquote_depth = frames.iter().filter(|frame| frame.kind == ContainerKind::Quote).count();
    let lists: Vec<_> =
        frames.iter().filter(|frame| frame.kind == ContainerKind::ListItem).collect();
    ContainerContext {
        blockquote_depth,
        list_depth: lists.len(),
        content_column: lists.last().map_or(0, |frame| frame.content_column),
    }
}

fn starts_block_structure(bytes: &[u8], start: usize, end: usize, column: usize) -> bool {
    let (content_start, _) = consume_indent(bytes, start, end, column, 3);
    if content_start >= end {
        return false;
    }
    let content = &bytes[content_start..end];
    let hashes = content.iter().take_while(|byte| **byte == b'#').count();
    let atx_heading = (1..=6).contains(&hashes)
        && (hashes == content.len()
            || content.get(hashes).is_some_and(|byte| matches!(byte, b' ' | b'\t')));
    let fence = content.starts_with(b"```") || content.starts_with(b"~~~");
    let structural_math = matches!(content, b"$$" | b"\\[" | b"\\]")
        || content.starts_with(b"\\begin{")
        || content.starts_with(b"\\end{");
    atx_heading
        || consume_quote_marker(bytes, start, end, column).is_some()
        || consume_list_marker(bytes, start, end, column).is_some()
        || fence
        || structural_math
}

fn build_lines(source: &str) -> Vec<Line> {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut active_frames = Vec::<ContainerFrame>::new();
    let mut previous_allows_lazy = false;
    for (index, part) in source.split_inclusive('\n').enumerate() {
        let end = start + part.len();
        let content_end = if part.ends_with('\n') { end - 1 } else { end };
        let mut content_start = start;
        let mut logical_column = 0;
        let mut matched_count = 0;
        for frame in &active_frames {
            let Some((next, next_column)) =
                match_frame(bytes, content_start, content_end, logical_column, *frame)
            else {
                break;
            };
            content_start = next;
            logical_column = next_column;
            matched_count += 1;
        }

        let remaining_blank =
            source[content_start..content_end].trim_matches([' ', '\t']).is_empty();
        let lazy = matched_count < active_frames.len()
            && previous_allows_lazy
            && !remaining_blank
            && !starts_block_structure(bytes, content_start, content_end, logical_column);
        let mut frames;
        let starts_list;
        if lazy {
            frames = active_frames.clone();
            starts_list = false;
        } else {
            frames = active_frames[..matched_count].to_vec();
            (content_start, logical_column, starts_list) =
                parse_new_frames(bytes, content_start, content_end, logical_column, &mut frames);
        }
        let context = container_context(&frames);
        lines.push(Line {
            index,
            start,
            end,
            content_end,
            content_start,
            logical_column,
            context,
            key: frames.clone(),
            frames: frames.clone(),
            lazy,
            starts_list,
        });

        if remaining_blank {
            let mut retained = active_frames[..matched_count].to_vec();
            for frame in &active_frames[matched_count..] {
                if frame.kind == ContainerKind::Quote {
                    break;
                }
                retained.push(*frame);
            }
            active_frames = if frames.len() > matched_count { frames } else { retained };
            previous_allows_lazy = false;
        } else {
            active_frames = frames;
            previous_allows_lazy =
                !starts_block_structure(bytes, content_start, content_end, logical_column);
        }
        start = end;
    }
    lines
}

fn line_payload_bounds(source: &str, line: &Line) -> (usize, usize) {
    let (start, _) = consume_indent(
        source.as_bytes(),
        line.content_start,
        line.content_end,
        line.logical_column,
        3,
    );
    (start, line.content_end)
}

fn content_under_frames(
    source: &str,
    line: &Line,
    frames: &[ContainerFrame],
) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut index = line.start;
    let mut column = 0;
    for (frame_index, frame) in frames.iter().enumerate() {
        let Some((next, next_column)) = match_frame(bytes, index, line.content_end, column, *frame)
        else {
            let remaining_blank =
                source[index..line.content_end].trim_matches([' ', '\t']).is_empty();
            let remaining_lists = frames[frame_index..]
                .iter()
                .all(|remaining| remaining.kind == ContainerKind::ListItem);
            return (remaining_blank && remaining_lists).then_some((line.content_end, column));
        };
        index = next;
        column = next_column;
    }
    Some((index, column))
}

fn payload_under_frames<'a>(
    source: &'a str,
    line: &Line,
    frames: &[ContainerFrame],
) -> Option<&'a str> {
    let (index, column) = content_under_frames(source, line, frames)?;
    let (start, _) = consume_indent(source.as_bytes(), index, line.content_end, column, 3);
    Some(source[start..line.content_end].trim_end_matches([' ', '\t']))
}

fn exact_payload_under_frames<'a>(
    source: &'a str,
    line: &Line,
    frames: &[ContainerFrame],
) -> Option<&'a str> {
    let (index, column) = content_under_frames(source, line, frames)?;
    let (start, _) = consume_indent(source.as_bytes(), index, line.content_end, column, 3);
    Some(&source[start..line.content_end])
}

fn line_indent_width(source: &str, line: &Line) -> usize {
    let bytes = source.as_bytes();
    let mut index = line.content_start;
    let mut column = line.logical_column;
    while index < line.content_end && matches!(bytes[index], b' ' | b'\t') {
        column = advance_column(column, bytes[index]);
        index += 1;
    }
    column - line.logical_column
}

fn escape_is_even(text: &str, position: usize) -> bool {
    let bytes = text.as_bytes();
    let mut cursor = position;
    let mut count = 0;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        cursor -= 1;
        count += 1;
    }
    count % 2 == 0
}

fn backtick_runs(text: &str, start: usize, end: usize) -> Vec<ByteRange> {
    let bytes = text.as_bytes();
    let mut runs = Vec::new();
    let mut index = start;
    while index < end {
        if bytes[index] != b'`' {
            index += text[index..].chars().next().map_or(1, char::len_utf8);
            continue;
        }
        let run_start = index;
        while index < end && bytes[index] == b'`' {
            index += 1;
        }
        runs.push((run_start, index));
    }
    runs
}

fn scan_backticks(text: &str, start: usize, end: usize) -> Vec<Candidate> {
    let mut pending = HashMap::<usize, usize>::new();
    let mut candidates = Vec::new();
    for (run_start, run_end) in backtick_runs(text, start, end) {
        let length = run_end - run_start;
        if let Some(opener) = pending.remove(&length) {
            candidates.push(Candidate::inline(RegionKind::CodeSpan, opener, run_end));
        } else {
            pending.insert(length, run_start);
        }
    }
    candidates
}

fn scan_composite_math(text: &str, start: usize, end: usize) -> Vec<Candidate> {
    let mut gitlab = HashMap::<usize, usize>::new();
    let mut myst = HashMap::<usize, usize>::new();
    let mut candidates = Vec::new();
    for (run_start, run_end) in backtick_runs(text, start, end) {
        let length = run_end - run_start;
        let closes_gitlab =
            run_end < end && text.as_bytes()[run_end] == b'$' && escape_is_even(text, run_end);
        if let Some(opener) = gitlab.get(&length).copied().filter(|_| closes_gitlab) {
            candidates.push(Candidate::inline(RegionKind::MathGitlabInline, opener, run_end + 1));
            gitlab.remove(&length);
        } else if run_start > start
            && text.as_bytes()[run_start - 1] == b'$'
            && escape_is_even(text, run_start - 1)
        {
            gitlab.entry(length).or_insert(run_start - 1);
        }

        if let Some(opener) = myst.remove(&length) {
            candidates.push(Candidate::inline(RegionKind::MathMystInline, opener, run_end));
        } else if run_start >= start + 6 && &text.as_bytes()[run_start - 6..run_start] == b"{math}"
        {
            myst.insert(length, run_start - 6);
        }
    }
    candidates
}

fn scan_angle_spans(text: &str, start: usize, end: usize) -> Vec<Candidate> {
    let bytes = text.as_bytes();
    let mut candidates = Vec::new();
    let mut index = start;
    while index < end {
        if bytes[index] != b'<' || index + 1 >= end {
            index += 1;
            continue;
        }
        let next = text[index + 1..end].chars().next().expect("nonempty angle-span tail");
        if next.is_whitespace() || matches!(next, '<' | '>') {
            index += 1;
            continue;
        }

        let terminator = if text[index..end].starts_with("<!--") {
            Some("-->")
        } else if text[index..end].starts_with("<![CDATA[") {
            Some("]]>")
        } else if text[index..end].starts_with("<?") {
            Some("?>")
        } else {
            None
        };
        if let Some(terminator) = terminator {
            if let Some(relative) = text[index + 2..end].find(terminator) {
                let span_end = index + 2 + relative + terminator.len();
                candidates.push(Candidate::inline(RegionKind::RawHtmlInline, index, span_end));
                index = span_end;
                continue;
            }
            index += 1;
            continue;
        }

        let fallback_close = bytes[index + 1..end].iter().position(|byte| *byte == b'>');
        let mut quote = None;
        let mut cursor = index + 1;
        let mut close = None;
        while cursor < end {
            match (quote, bytes[cursor]) {
                (Some(active), byte) if active == byte => quote = None,
                (None, byte @ (b'\'' | b'"')) => quote = Some(byte),
                (None, b'>') => {
                    close = Some(cursor);
                    break;
                }
                _ => {}
            }
            cursor += 1;
        }
        let close = close.or_else(|| fallback_close.map(|relative| index + 1 + relative));
        let Some(close) = close else {
            index += 1;
            continue;
        };
        let span_end = close + 1;
        candidates.push(Candidate::inline(RegionKind::RawHtmlInline, index, span_end));
        index = span_end;
    }
    candidates
}

fn attribute_atom_end(text: &str, mut index: usize, end: usize, value: bool) -> Option<usize> {
    let start = index;
    while index < end {
        let scalar = text[index..end].chars().next().expect("attribute atom boundary");
        if scalar.is_whitespace() {
            break;
        }
        if scalar == '\\' {
            index += 1;
            let escaped = text[index..end].chars().next()?;
            index += escaped.len_utf8();
            continue;
        }
        if matches!(scalar, '{' | '}' | '"' | '\'') || !value && scalar == '=' {
            break;
        }
        index += scalar.len_utf8();
    }
    (index > start).then_some(index)
}

fn valid_attribute_body(text: &str, start: usize, end: usize) -> bool {
    let mut index = start;
    let mut attributes = 0;
    loop {
        while index < end {
            let scalar = text[index..end].chars().next().expect("attribute whitespace boundary");
            if !scalar.is_whitespace() {
                break;
            }
            index += scalar.len_utf8();
        }
        if index == end {
            return attributes > 0;
        }
        if matches!(text.as_bytes()[index], b'.' | b'#') {
            let Some(token_end) = attribute_atom_end(text, index + 1, end, false) else {
                return false;
            };
            index = token_end;
            attributes += 1;
            continue;
        }

        let Some(key_end) = attribute_atom_end(text, index, end, false) else {
            return false;
        };
        if key_end >= end || text.as_bytes()[key_end] != b'=' {
            return false;
        }
        index = key_end + 1;
        if index < end && matches!(text.as_bytes()[index], b'"' | b'\'') {
            let quote = text.as_bytes()[index];
            index += 1;
            while index < end && text.as_bytes()[index] != quote {
                if text.as_bytes()[index] == b'\\' {
                    index += 1;
                    if index >= end {
                        return false;
                    }
                }
                index +=
                    text[index..end].chars().next().expect("quoted attribute boundary").len_utf8();
            }
            if index >= end {
                return false;
            }
            index += 1;
        } else {
            let Some(value_end) = attribute_atom_end(text, index, end, true) else {
                return false;
            };
            index = value_end;
        }
        attributes += 1;
    }
}

fn attribute_group_end(text: &str, start: usize, end: usize) -> Option<usize> {
    if start >= end || text.as_bytes()[start] != b'{' || !escape_is_even(text, start) {
        return None;
    }
    let mut index = start + 1;
    let mut quote = None;
    while index < end {
        let scalar = text[index..end].chars().next()?;
        if scalar == '\n' {
            return None;
        }
        if scalar == '\\' {
            index += 1;
            let escaped = text[index..end].chars().next()?;
            index += escaped.len_utf8();
            continue;
        }
        match quote {
            Some(active) if scalar == active => quote = None,
            None if matches!(scalar, '"' | '\'') => quote = Some(scalar),
            None if scalar == '{' => return None,
            None if scalar == '}' => {
                return valid_attribute_body(text, start + 1, index).then_some(index + 1);
            }
            Some(_) | None => {}
        }
        index += scalar.len_utf8();
    }
    None
}

fn is_setext_heading(payload: &str) -> bool {
    let payload = payload.trim_end_matches([' ', '\t']);
    let indent = payload.bytes().take_while(|byte| *byte == b' ').count();
    if indent > 3 {
        return false;
    }
    let rule = &payload[indent..];
    !rule.is_empty()
        && (rule.bytes().all(|byte| byte == b'=') || rule.bytes().all(|byte| byte == b'-'))
}

fn compatible_attribute_predecessor(
    text: &str,
    start: usize,
    group_end: usize,
    scope_end: usize,
) -> bool {
    let Some(previous) = text[..start].chars().next_back() else {
        return false;
    };
    if !previous.is_whitespace() {
        return matches!(previous, ')' | ']' | '}' | '`' | '$' | '>' | '*' | '_');
    }
    let line_start = text[..start].rfind('\n').map_or(0, |index| index + 1);
    if is_heading(text[line_start..start].trim_start_matches([' ', '\t'])) {
        return true;
    }
    let Some(relative_line_end) = text[group_end..scope_end].find('\n') else {
        return false;
    };
    let line_end = group_end + relative_line_end;
    if !text[group_end..line_end].trim_matches([' ', '\t']).is_empty() {
        return false;
    }
    let next_end = text[line_end + 1..scope_end]
        .find('\n')
        .map_or(scope_end, |relative| line_end + 1 + relative);
    is_setext_heading(&text[line_end + 1..next_end])
}

fn scan_attribute_groups(text: &str, start: usize, end: usize) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut index = start;
    while index < end {
        if text.as_bytes()[index] != b'{' {
            index += text[index..end].chars().next().map_or(1, char::len_utf8);
            continue;
        }
        let Some(group_end) = attribute_group_end(text, index, end) else {
            index += 1;
            continue;
        };
        if !compatible_attribute_predecessor(text, index, group_end, end) {
            index += 1;
            continue;
        }
        candidates.push(Candidate::inline(RegionKind::AttributeGroupInline, index, group_end));
        index = group_end;
    }
    candidates
}

fn scan_paren_math(text: &str, start: usize, end: usize) -> Vec<Candidate> {
    let bytes = text.as_bytes();
    let mut opener = None;
    let mut candidates = Vec::new();
    let mut index = start;
    while index + 1 < end {
        if bytes[index] == b'\\' && bytes[index + 1] == b'(' && escape_is_even(text, index) {
            opener.get_or_insert(index);
            index += 2;
        } else if bytes[index] == b'\\' && bytes[index + 1] == b')' && escape_is_even(text, index) {
            if let Some(open) = opener.take() {
                candidates.push(Candidate::inline(RegionKind::MathParenInline, open, index + 2));
            }
            index += 2;
        } else {
            index += text[index..].chars().next().map_or(1, char::len_utf8);
        }
    }
    candidates
}

fn environment_event(text: &str, index: usize, end: usize) -> Option<(&str, &str, usize)> {
    if !escape_is_even(text, index) {
        return None;
    }
    let (command, name_start) = if text[index..end].starts_with("\\begin{") {
        ("begin", index + 7)
    } else if text[index..end].starts_with("\\end{") {
        ("end", index + 5)
    } else {
        return None;
    };
    let tail = &text[name_start..end];
    let close = tail.find('}')?;
    if close == 0 || tail[..close].contains(['{', '\n']) {
        return None;
    }
    Some((command, &tail[..close], name_start + close + 1))
}

fn scan_inline_environments(text: &str, start: usize, end: usize) -> Vec<Candidate> {
    let mut stack: Vec<(String, usize)> = Vec::new();
    let mut candidates = Vec::new();
    let mut index = start;
    while index < end {
        if text.as_bytes()[index] != b'\\' {
            index += text[index..].chars().next().map_or(1, char::len_utf8);
            continue;
        }
        let Some((command, name, event_end)) = environment_event(text, index, end) else {
            index += 1;
            continue;
        };
        if command == "begin" {
            stack.push((name.to_owned(), index));
        } else if stack.last().is_some_and(|(open_name, _)| open_name == name) {
            let (_, opener) = stack.pop().expect("checked stack");
            candidates.push(Candidate::inline(
                RegionKind::MathEnvironmentInline,
                opener,
                event_end,
            ));
        }
        index = event_end;
    }
    candidates
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DollarState {
    None,
    Single(usize),
    Double { opener: usize, fallback: Option<usize> },
}

fn scan_dollars(text: &str, start: usize, end: usize) -> Vec<Candidate> {
    let bytes = text.as_bytes();
    let mut candidates = Vec::new();
    let mut state = DollarState::None;
    let mut index = start;
    while index < end {
        if bytes[index] != b'$' || !escape_is_even(text, index) {
            index += text[index..].chars().next().map_or(1, char::len_utf8);
            continue;
        }
        let run_start = index;
        while index < end && bytes[index] == b'$' {
            index += 1;
        }
        let run_end = index;
        let mut position = run_start;
        while position < run_end {
            let remaining = run_end - position;
            match state {
                DollarState::None if remaining >= 2 => {
                    state = DollarState::Double { opener: position, fallback: None };
                    position += 2;
                }
                DollarState::None => {
                    state = DollarState::Single(position);
                    position += 1;
                }
                DollarState::Single(opener) => {
                    candidates.push(Candidate::inline(
                        RegionKind::MathDollarInline,
                        opener,
                        position + 1,
                    ));
                    state = DollarState::None;
                    position += 1;
                }
                DollarState::Double { opener, .. } if remaining >= 2 => {
                    candidates.push(Candidate::inline(
                        RegionKind::MathDoubleDollarInline,
                        opener,
                        position + 2,
                    ));
                    state = DollarState::None;
                    position += 2;
                }
                DollarState::Double { opener, fallback: None } => {
                    state = DollarState::Double { opener, fallback: Some(position) };
                    position += 1;
                }
                DollarState::Double { opener, fallback: Some(fallback) } => {
                    candidates.push(Candidate::inline(
                        RegionKind::MathDollarInline,
                        fallback,
                        position + 1,
                    ));
                    state = DollarState::Double { opener, fallback: None };
                    position += 1;
                }
            }
        }
    }
    candidates
}

fn arbitrate(mut candidates: Vec<Candidate>, start: usize, end: usize) -> Vec<Candidate> {
    candidates.retain(|candidate| candidate.start >= start && candidate.end <= end);
    candidates.sort_by_key(|candidate| candidate.start);
    let mut selected = Vec::new();
    let mut previous_end = start;
    let mut index = 0;
    while index < candidates.len() {
        let group_start = candidates[index].start;
        let mut best = candidates[index].clone();
        index += 1;
        while index < candidates.len() && candidates[index].start == group_start {
            let alternate = &candidates[index];
            let best_key =
                (priority(best.kind), usize::MAX - (best.end - best.start), stable_name(best.kind));
            let alternate_key = (
                priority(alternate.kind),
                usize::MAX - (alternate.end - alternate.start),
                stable_name(alternate.kind),
            );
            if alternate_key < best_key {
                best = alternate.clone();
            }
            index += 1;
        }
        if best.start >= previous_end {
            previous_end = best.end;
            selected.push(best);
        }
    }
    selected
}

fn scan_inline_scope(text: &str, start: usize, end: usize) -> Vec<Candidate> {
    let mut candidates = scan_composite_math(text, start, end);
    candidates.extend(scan_backticks(text, start, end));
    candidates.extend(scan_paren_math(text, start, end));
    candidates.extend(scan_inline_environments(text, start, end));
    candidates.extend(scan_attribute_groups(text, start, end));
    candidates.extend(scan_dollars(text, start, end));
    arbitrate(candidates, start, end)
}

fn fence_opener(payload: &str) -> Option<(u8, usize)> {
    let bytes = payload.as_bytes();
    let character = *bytes.first()?;
    if !matches!(character, b'`' | b'~') {
        return None;
    }
    let length = bytes.iter().take_while(|byte| **byte == character).count();
    if length < 3 || (character == b'`' && bytes[length..].contains(&b'`')) {
        return None;
    }
    Some((character, length))
}

fn fence_closes(payload: &str, character: u8, length: usize) -> bool {
    let bytes = payload.as_bytes();
    bytes.len() >= length && bytes.iter().all(|byte| *byte == character)
}

fn opaque_line_flags(source: &str, lines: &[Line]) -> Vec<bool> {
    let mut covered = vec![false; lines.len()];

    if lines.first().is_some_and(|line| {
        line.start == 0 && line.key.is_empty() && &source[line.start..line.content_end] == "---"
    }) {
        if let Some(closer) = lines
            .iter()
            .skip(1)
            .position(|line| matches!(&source[line.start..line.content_end], "---" | "..."))
        {
            covered[..=closer + 1].fill(true);
        }
    }

    let mut index = 0;
    while index < lines.len() {
        if covered[index] {
            index += 1;
            continue;
        }
        let Some((character, length)) = fence_opener(lines[index].payload(source)) else {
            index += 1;
            continue;
        };
        let mut final_index = index;
        for candidate in index + 1..lines.len() {
            let Some(payload) =
                payload_under_frames(source, &lines[candidate], &lines[index].frames)
            else {
                break;
            };
            final_index = candidate;
            if fence_closes(payload, character, length) {
                break;
            }
        }
        covered[index..=final_index].fill(true);
        index = final_index + 1;
    }

    let mut index = 0;
    while index < lines.len() {
        if covered[index] || line_indent_width(source, &lines[index]) < 4 {
            index += 1;
            continue;
        }
        if index > 0 && !covered[index - 1] && !lines[index - 1].is_blank(source) {
            index += 1;
            continue;
        }
        let mut final_index = index;
        let mut cursor = index + 1;
        while cursor < lines.len() && !covered[cursor] {
            if lines[cursor].is_blank(source) {
                cursor += 1;
                continue;
            }
            if lines[cursor].key == lines[index].key
                && line_indent_width(source, &lines[cursor]) >= 4
            {
                final_index = cursor;
                cursor += 1;
            } else {
                break;
            }
        }
        covered[index..=final_index].fill(true);
        index = final_index + 1;
    }
    covered
}

fn dollar_closer(payload: &str) -> bool {
    if payload == "$$" {
        return true;
    }
    let Some(label) = payload.strip_prefix("$$").and_then(|rest| {
        (!rest.is_empty() && rest.starts_with([' ', '\t'])).then_some(rest.trim_start())
    }) else {
        return false;
    };
    (label.starts_with('(')
        && label.ends_with(')')
        && !label[1..label.len() - 1].contains(['(', ')', '\n']))
        || (label.starts_with('{')
            && label.ends_with('}')
            && !label[1..label.len() - 1].contains(['{', '}', '\n']))
}

fn exact_environment_line(payload: &str) -> Option<(&str, &str)> {
    let (command, tail) = if let Some(tail) = payload.strip_prefix("\\begin{") {
        ("begin", tail)
    } else {
        ("end", payload.strip_prefix("\\end{")?)
    };
    let name = tail.strip_suffix('}')?;
    (!name.is_empty() && !name.contains(['{', '}', '\n'])).then_some((command, name))
}

fn multiline_table_rule(payload: &str) -> Option<usize> {
    let bytes = payload.as_bytes();
    let mut index = 0;
    let mut segments = 0;
    while index < bytes.len() {
        let run_start = index;
        while index < bytes.len() && bytes[index] == b'-' {
            index += 1;
        }
        if index - run_start < 3 {
            return None;
        }
        segments += 1;
        if index == bytes.len() {
            return Some(segments);
        }
        let whitespace_start = index;
        while index < bytes.len() && matches!(bytes[index], b' ' | b'\t') {
            index += 1;
        }
        if index == whitespace_start || index == bytes.len() {
            return None;
        }
    }
    None
}

fn is_table_caption(payload: &str) -> bool {
    ["Table:", "table:", ":"].iter().any(|prefix| {
        payload
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t'))
    })
}

fn caption_extent_after(source: &str, lines: &[Line], closer_index: usize, opener: &Line) -> usize {
    let mut index = closer_index + 1;
    let Some(mut payload) =
        lines.get(index).and_then(|line| payload_under_frames(source, line, &opener.frames))
    else {
        return closer_index;
    };
    if payload.is_empty() {
        index += 1;
        let Some(next_payload) =
            lines.get(index).and_then(|line| payload_under_frames(source, line, &opener.frames))
        else {
            return closer_index;
        };
        payload = next_payload;
    }
    if !is_table_caption(payload) {
        return closer_index;
    }
    let mut final_index = index;
    index += 1;
    while let Some(payload) =
        lines.get(index).and_then(|line| payload_under_frames(source, line, &opener.frames))
    {
        if payload.is_empty() {
            break;
        }
        final_index = index;
        index += 1;
    }
    final_index
}

fn scan_pandoc_multiline_tables(source: &str, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut opener_index = 0;
    while opener_index < lines.len() {
        let opener = &lines[opener_index];
        let opener_payload = opener.payload(source);
        let opener_rule = (!opaque[opener_index] && !opener.lazy)
            .then(|| multiline_table_rule(opener_payload))
            .flatten();
        let Some(opener_segments) = opener_rule else {
            opener_index += 1;
            continue;
        };

        let headered = opener_segments == 1;
        let mut header_separator_seen = false;
        let mut body_content_seen = false;
        let mut body_blank_seen = false;
        let mut closer_index = None;
        let mut scan_index = opener_index + 1;
        while scan_index < lines.len() {
            if opaque[scan_index] {
                break;
            }
            let Some(payload) = payload_under_frames(source, &lines[scan_index], &opener.frames)
            else {
                break;
            };
            let rule = multiline_table_rule(payload);
            if payload == opener_payload && body_content_seen {
                if (headered && header_separator_seen) || (!headered && body_blank_seen) {
                    closer_index = Some(scan_index);
                }
                break;
            }
            if headered && !header_separator_seen && rule.is_some_and(|segments| segments >= 2) {
                header_separator_seen = true;
            } else if payload.is_empty() {
                if body_content_seen {
                    body_blank_seen = true;
                }
            } else if !headered || header_separator_seen {
                body_content_seen = true;
            }
            scan_index += 1;
        }

        let Some(closer_index) = closer_index else {
            opener_index += 1;
            continue;
        };
        let final_index = caption_extent_after(source, lines, closer_index, opener);
        candidates.push(Candidate::block(
            RegionKind::PandocMultilineTable,
            opener.start,
            lines[final_index].end,
            opener.context,
            opener.scaffold(source).to_owned(),
        ));
        opener_index = final_index + 1;
    }
    candidates
}

fn is_obsidian_callout(payload: &str) -> bool {
    let Some(rest) = payload.strip_prefix("[!") else {
        return false;
    };
    let Some(marker_end) = rest.find(']') else {
        return false;
    };
    let kind = &rest[..marker_end];
    if kind.is_empty()
        || !kind.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index > 0 && matches!(byte, b'_' | b'-'))
        })
    {
        return false;
    }
    let mut suffix = &rest[marker_end + 1..];
    if suffix.starts_with(['+', '-']) {
        suffix = &suffix[1..];
    }
    suffix.is_empty() || suffix.starts_with(' ') || suffix.starts_with('\t')
}

fn scan_obsidian_callouts(source: &str, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for (opener_index, opener) in lines.iter().enumerate() {
        if opaque[opener_index]
            || opener.lazy
            || opener.frames.last().map(|frame| frame.kind) != Some(ContainerKind::Quote)
            || !is_obsidian_callout(opener.payload(source))
            || opener_index > 0 && lines[opener_index - 1].frames == opener.frames
        {
            continue;
        }

        let mut final_index = opener_index;
        for (candidate_index, candidate) in lines.iter().enumerate().skip(opener_index + 1) {
            if candidate.frames.len() < opener.frames.len()
                || candidate.frames[..opener.frames.len()] != opener.frames
            {
                break;
            }
            final_index = candidate_index;
        }
        candidates.push(Candidate::block(
            RegionKind::ObsidianCallout,
            opener.start,
            lines[final_index].end,
            opener.context,
            opener.scaffold(source).to_owned(),
        ));
    }
    candidates
}

fn colon_fence_is_opener(payload: &str) -> Option<bool> {
    let colon_count = payload.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    Some(!payload[colon_count..].trim_matches([' ', '\t']).is_empty())
}

fn scan_colon_containers(source: &str, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let mut stacks = HashMap::<Vec<ContainerFrame>, Vec<usize>>::new();
    let mut candidates = Vec::new();
    for line in lines {
        if opaque[line.index] || line.lazy {
            continue;
        }
        let Some(is_opener) = colon_fence_is_opener(line.payload(source)) else {
            continue;
        };
        let stack = stacks.entry(line.key.clone()).or_default();
        if is_opener {
            stack.push(line.index);
            continue;
        }
        let Some(opener_index) = stack.pop() else {
            continue;
        };
        let opener = &lines[opener_index];
        candidates.push(Candidate::block(
            RegionKind::ColonContainer,
            opener.start,
            line.end,
            opener.context,
            opener.scaffold(source).to_owned(),
        ));
    }
    candidates
}

fn is_root_exact_delimiter(source: &str, line: &Line, delimiter: &str) -> bool {
    line.key.is_empty() && !line.lazy && source.get(line.start..line.content_end) == Some(delimiter)
}

fn scan_toml_frontmatter(source: &str, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let Some(opener) = lines.first() else {
        return Vec::new();
    };
    if opaque[opener.index] || !is_root_exact_delimiter(source, opener, "+++") {
        return Vec::new();
    }
    for closer in lines.iter().skip(1) {
        if is_root_exact_delimiter(source, closer, "+++") {
            return vec![Candidate::block(
                RegionKind::TomlFrontmatter,
                opener.start,
                closer.end,
                opener.context,
                opener.scaffold(source).to_owned(),
            )];
        }
    }
    Vec::new()
}

fn definition_marker(source: &str, line: &Line, frames: &[ContainerFrame]) -> Option<usize> {
    if line.lazy {
        return None;
    }
    let (index, column) = content_under_frames(source, line, frames)?;
    let (mut index, mut column) =
        consume_indent(source.as_bytes(), index, line.content_end, column, 2);
    if index >= line.content_end || !matches!(source.as_bytes()[index], b':' | b'~') {
        return None;
    }
    index += 1;
    column += 1;
    if index < line.content_end && !matches!(source.as_bytes()[index], b' ' | b'\t') {
        return None;
    }
    let (_, column) = consume_whitespace(source.as_bytes(), index, line.content_end, column);
    Some(column)
}

fn definition_continuation_column(
    source: &str,
    line: &Line,
    frames: &[ContainerFrame],
) -> Option<usize> {
    let (mut index, mut column) = content_under_frames(source, line, frames)?;
    while index < line.content_end && matches!(source.as_bytes()[index], b' ' | b'\t') {
        column = advance_column(column, source.as_bytes()[index]);
        index += 1;
    }
    Some(column)
}

fn definition_payload<'a>(
    source: &'a str,
    line: &Line,
    frames: &[ContainerFrame],
) -> Option<&'a str> {
    if line.frames == frames {
        Some(line.payload(source))
    } else {
        payload_under_frames(source, line, frames)
    }
}

fn definition_term_line(
    source: &str,
    line: &Line,
    frames: &[ContainerFrame],
    opaque: bool,
) -> bool {
    if opaque || line.lazy || line.frames != frames {
        return false;
    }
    let Some(payload) = definition_payload(source, line, frames) else {
        return false;
    };
    if payload.is_empty()
        || definition_marker(source, line, frames).is_some()
        || colon_fence_is_opener(payload).is_some()
    {
        return false;
    }
    !starts_block_structure(
        source.as_bytes(),
        line.content_start,
        line.content_end,
        line.logical_column,
    )
}

fn definition_term_start(
    source: &str,
    lines: &[Line],
    marker_index: usize,
    frames: &[ContainerFrame],
    opaque: &[bool],
) -> Option<usize> {
    let mut term_end = marker_index.checked_sub(1)?;
    content_under_frames(source, &lines[term_end], frames)?;
    if definition_payload(source, &lines[term_end], frames)?.is_empty() {
        term_end = term_end.checked_sub(1)?;
        content_under_frames(source, &lines[term_end], frames)?;
        if definition_payload(source, &lines[term_end], frames)?.is_empty() {
            return None;
        }
    }
    if !definition_term_line(source, &lines[term_end], frames, opaque[term_end]) {
        return None;
    }
    let mut term_start = term_end;
    while term_start > 0
        && definition_term_line(source, &lines[term_start - 1], frames, opaque[term_start - 1])
    {
        term_start -= 1;
    }
    Some(term_start)
}

fn definition_term_sequence(
    source: &str,
    lines: &[Line],
    start_index: usize,
    frames: &[ContainerFrame],
    opaque: &[bool],
) -> Option<(usize, usize)> {
    let mut index = start_index;
    let mut terms = 0;
    while index < lines.len() {
        let line = &lines[index];
        if opaque[index] || line.lazy || content_under_frames(source, line, frames).is_none() {
            return None;
        }
        let payload = definition_payload(source, line, frames)?;
        if payload.is_empty() {
            break;
        }
        if line.frames != frames {
            return None;
        }
        if let Some(marker_column) = definition_marker(source, line, frames) {
            return (terms > 0).then_some((index, marker_column));
        }
        if !definition_term_line(source, line, frames, false) {
            return None;
        }
        terms += 1;
        index += 1;
    }
    if terms == 0 || index >= lines.len() {
        return None;
    }
    index += 1;
    let line = lines.get(index)?;
    if opaque[index] || line.lazy || line.frames != frames {
        return None;
    }
    definition_marker(source, line, frames).map(|column| (index, column))
}

fn scan_definition_lists(source: &str, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut marker_index = 0;
    while marker_index < lines.len() {
        let marker = &lines[marker_index];
        let frames = marker.frames.as_slice();
        let marker_column =
            (!opaque[marker_index]).then(|| definition_marker(source, marker, frames)).flatten();
        let term_start = marker_column
            .and_then(|_| definition_term_start(source, lines, marker_index, frames, opaque));
        let (Some(mut marker_column), Some(term_start)) = (marker_column, term_start) else {
            marker_index += 1;
            continue;
        };

        let mut final_index = marker_index;
        let mut scan_index = marker_index + 1;
        let mut blank_start = None;
        while scan_index < lines.len() {
            let line = &lines[scan_index];
            if content_under_frames(source, line, frames).is_none() {
                break;
            }
            let payload = definition_payload(source, line, frames)
                .expect("compatible definition-list line has a payload");
            if payload.is_empty() {
                blank_start.get_or_insert(scan_index);
                scan_index += 1;
                continue;
            }

            let next_marker_column =
                (!opaque[scan_index]).then(|| definition_marker(source, line, frames)).flatten();
            if blank_start.is_none() {
                final_index = scan_index;
                if let Some(column) = next_marker_column {
                    marker_column = column;
                }
                scan_index += 1;
                continue;
            }

            let continuation_column = definition_continuation_column(source, line, frames);
            let deeper_container = line.frames.len() > frames.len();
            if let Some(column) = next_marker_column {
                final_index = scan_index;
                marker_column = column;
                blank_start = None;
                scan_index += 1;
                continue;
            }
            if deeper_container || continuation_column.is_some_and(|column| column >= marker_column)
            {
                final_index = scan_index;
                blank_start = None;
                scan_index += 1;
                continue;
            }
            let Some((next_marker_index, column)) =
                definition_term_sequence(source, lines, scan_index, frames, opaque)
            else {
                break;
            };
            final_index = next_marker_index;
            marker_column = column;
            blank_start = None;
            scan_index = next_marker_index + 1;
        }

        let opener = &lines[term_start];
        candidates.push(Candidate::block(
            RegionKind::DefinitionList,
            opener.start,
            lines[final_index].end,
            opener.context,
            opener.scaffold(source).to_owned(),
        ));
        marker_index = final_index + 1;
    }
    candidates
}

fn grid_border(payload: &str) -> Option<(u8, Vec<usize>)> {
    let bytes = payload.as_bytes();
    if bytes.len() < 3 || bytes.first() != Some(&b'+') || bytes.last() != Some(&b'+') {
        return None;
    }
    let mut segments = Vec::new();
    let mut index = 1;
    let character = *bytes.get(index)?;
    if !matches!(character, b'-' | b'=') {
        return None;
    }
    while index < bytes.len() - 1 {
        let start = index;
        while index < bytes.len() - 1 && bytes[index] == character {
            index += 1;
        }
        if start == index || bytes[index] != b'+' {
            return None;
        }
        segments.push(index - start);
        index += 1;
    }
    (index == bytes.len()).then_some((character, segments))
}

fn scan_pandoc_grid_tables(source: &str, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut opener_index = 0;
    while opener_index < lines.len() {
        let opener = &lines[opener_index];
        let opener_payload = opener.payload(source);
        let opener_border =
            (!opaque[opener_index] && !opener.lazy).then(|| grid_border(opener_payload)).flatten();
        let Some((b'-', opener_signature)) = opener_border else {
            opener_index += 1;
            continue;
        };

        let opener_width = opener_payload.len();
        let mut content_since_border = false;
        let mut closer_index = None;
        let mut scan_index = opener_index + 1;
        while scan_index < lines.len() {
            let line = &lines[scan_index];
            if opaque[scan_index] || content_under_frames(source, line, &opener.frames).is_none() {
                break;
            }
            let payload = payload_under_frames(source, line, &opener.frames)
                .expect("compatible grid-table line has a payload");
            if let Some((_, signature)) = grid_border(payload) {
                if !content_since_border || payload.len() != opener_width {
                    break;
                }
                let next_is_content = lines.get(scan_index + 1).is_some_and(|next| {
                    payload_under_frames(source, next, &opener.frames).is_some_and(|next_payload| {
                        next_payload.starts_with('|') && next_payload.ends_with('|')
                    })
                });
                if signature == opener_signature && !next_is_content {
                    closer_index = Some(scan_index);
                    break;
                }
                content_since_border = false;
                scan_index += 1;
                continue;
            }
            if payload.starts_with('|') && payload.ends_with('|') {
                content_since_border = true;
                scan_index += 1;
                continue;
            }
            break;
        }

        let Some(closer_index) = closer_index else {
            opener_index += 1;
            continue;
        };
        let final_index = caption_extent_after(source, lines, closer_index, opener);
        candidates.push(Candidate::block(
            RegionKind::PandocGridTable,
            opener.start,
            lines[final_index].end,
            opener.context,
            opener.scaffold(source).to_owned(),
        ));
        opener_index = final_index + 1;
    }
    candidates
}

enum HtmlBlockEnd {
    Terminator(String),
    BlankLine,
}

fn html_name_prefix<'a>(payload: &'a str, name: &str) -> Option<&'a str> {
    let tail = payload.get(1..)?;
    let tail = tail.get(name.len()..).filter(|_| tail[..name.len()].eq_ignore_ascii_case(name))?;
    (tail.is_empty() || tail.starts_with([' ', '\t', '>']) || tail.starts_with("/>"))
        .then_some(tail)
}

fn raw_html_tag(payload: &str) -> Option<String> {
    ["script", "pre", "style", "textarea"]
        .iter()
        .find_map(|name| html_name_prefix(payload, name).map(|_| format!("</{name}>")))
}

fn starts_standard_html_tag(payload: &str) -> bool {
    const TAGS: &[&str] = &[
        "address",
        "article",
        "aside",
        "base",
        "basefont",
        "blockquote",
        "body",
        "caption",
        "center",
        "col",
        "colgroup",
        "dd",
        "details",
        "dialog",
        "dir",
        "div",
        "dl",
        "dt",
        "fieldset",
        "figcaption",
        "figure",
        "footer",
        "form",
        "frame",
        "frameset",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "head",
        "header",
        "hr",
        "html",
        "iframe",
        "legend",
        "li",
        "link",
        "main",
        "menu",
        "menuitem",
        "nav",
        "noframes",
        "ol",
        "optgroup",
        "option",
        "p",
        "param",
        "search",
        "section",
        "summary",
        "table",
        "tbody",
        "td",
        "tfoot",
        "th",
        "thead",
        "title",
        "tr",
        "track",
        "ul",
    ];
    let opening = payload.strip_prefix('<').unwrap_or("");
    let opening = opening.strip_prefix('/').unwrap_or(opening);
    TAGS.iter().any(|name| {
        opening.get(..name.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(name))
            && opening.get(name.len()..).is_some_and(|tail| {
                tail.is_empty() || tail.starts_with([' ', '\t', '>']) || tail.starts_with("/>")
            })
    })
}

fn is_complete_html_tag(payload: &str) -> bool {
    let payload = payload.trim_end_matches([' ', '\t']);
    let Some(mut inner) = payload.strip_prefix('<').and_then(|tail| tail.strip_suffix('>')) else {
        return false;
    };
    inner = inner.strip_prefix('/').unwrap_or(inner);
    let name_len =
        inner.bytes().take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'-').count();
    if name_len == 0 || !inner.as_bytes()[0].is_ascii_alphabetic() {
        return false;
    }
    let rest = &inner[name_len..];
    rest.is_empty()
        || rest == "/"
        || matches!(rest.as_bytes().first(), Some(b' ' | b'\t')) && !rest.contains(['<', '>'])
}

fn html_block_start(payload: &str, allow_type_seven: bool) -> Option<HtmlBlockEnd> {
    if let Some(terminator) = raw_html_tag(payload) {
        return Some(HtmlBlockEnd::Terminator(terminator));
    }
    if payload.starts_with("<!--") {
        return Some(HtmlBlockEnd::Terminator("-->".to_owned()));
    }
    if payload.starts_with("<?") {
        return Some(HtmlBlockEnd::Terminator("?>".to_owned()));
    }
    if payload.starts_with("<![CDATA[") {
        return Some(HtmlBlockEnd::Terminator("]]>".to_owned()));
    }
    if payload
        .strip_prefix("<!")
        .and_then(|tail| tail.as_bytes().first())
        .is_some_and(u8::is_ascii_alphabetic)
    {
        return Some(HtmlBlockEnd::Terminator(">".to_owned()));
    }
    if starts_standard_html_tag(payload) || allow_type_seven && is_complete_html_tag(payload) {
        return Some(HtmlBlockEnd::BlankLine);
    }
    None
}

fn scan_raw_html_blocks(source: &str, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut opener_index = 0;
    while opener_index < lines.len() {
        let opener = &lines[opener_index];
        let previous_allows_type_seven = opener_index == 0
            || lines[opener_index - 1].frames != opener.frames
            || payload_under_frames(source, &lines[opener_index - 1], &opener.frames)
                .is_some_and(str::is_empty);
        let start = (!opaque[opener_index] && !opener.lazy)
            .then(|| html_block_start(opener.payload(source), previous_allows_type_seven))
            .flatten();
        let Some(end_condition) = start else {
            opener_index += 1;
            continue;
        };

        let mut final_index = opener_index;
        let mut scan_index = opener_index + 1;
        let opener_closed = match &end_condition {
            HtmlBlockEnd::Terminator(terminator) => opener
                .payload(source)
                .to_ascii_lowercase()
                .contains(&terminator.to_ascii_lowercase()),
            HtmlBlockEnd::BlankLine => false,
        };
        while !opener_closed && scan_index < lines.len() {
            let line = &lines[scan_index];
            let Some(payload) = payload_under_frames(source, line, &opener.frames) else {
                break;
            };
            if matches!(end_condition, HtmlBlockEnd::BlankLine) && payload.is_empty() {
                break;
            }
            final_index = scan_index;
            if let HtmlBlockEnd::Terminator(terminator) = &end_condition
                && payload.to_ascii_lowercase().contains(&terminator.to_ascii_lowercase())
            {
                break;
            }
            scan_index += 1;
        }

        candidates.push(Candidate::block(
            RegionKind::RawHtmlBlock,
            opener.start,
            lines[final_index].end,
            opener.context,
            opener.scaffold(source).to_owned(),
        ));
        opener_index = final_index + 1;
    }
    candidates
}

fn scan_attribute_group_blocks(source: &str, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for line in lines {
        let payload = line.payload(source);
        if opaque[line.index] || line.lazy || !payload.starts_with('{') {
            continue;
        }
        if attribute_group_end(payload, 0, payload.len()) != Some(payload.len()) {
            continue;
        }
        candidates.push(Candidate::block(
            RegionKind::AttributeGroupBlock,
            line.start,
            line.end,
            line.context,
            line.scaffold(source).to_owned(),
        ));
    }
    candidates
}

fn is_pandoc_line(payload: &str) -> bool {
    if payload != "|" && !payload.starts_with("| ") {
        return false;
    }
    !(payload.ends_with('|') && payload.bytes().filter(|byte| *byte == b'|').count() >= 3)
}

fn scan_pandoc_line_blocks(source: &str, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut opener_index = 0;
    while opener_index < lines.len() {
        let opener = &lines[opener_index];
        if opaque[opener_index] || opener.lazy || !is_pandoc_line(opener.exact_payload(source)) {
            opener_index += 1;
            continue;
        }
        let mut final_index = opener_index;
        let mut scan_index = opener_index + 1;
        while scan_index < lines.len() {
            if opaque[scan_index] {
                break;
            }
            let line = &lines[scan_index];
            let compatible = exact_payload_under_frames(source, line, &opener.frames);
            if !compatible.is_some_and(is_pandoc_line) {
                break;
            }
            final_index = scan_index;
            scan_index += 1;
        }
        candidates.push(Candidate::block(
            RegionKind::PandocLineBlock,
            opener.start,
            lines[final_index].end,
            opener.context,
            opener.scaffold(source).to_owned(),
        ));
        opener_index = final_index + 1;
    }
    candidates
}

fn scan_blocks(source: &NormalizedSource, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let text = source.text.as_str();
    let mut candidates = scan_toml_frontmatter(text, lines, opaque);
    candidates.extend(scan_pandoc_multiline_tables(text, lines, opaque));
    candidates.extend(scan_obsidian_callouts(text, lines, opaque));
    candidates.extend(scan_colon_containers(text, lines, opaque));
    candidates.extend(scan_definition_lists(text, lines, opaque));
    candidates.extend(scan_pandoc_grid_tables(text, lines, opaque));
    candidates.extend(scan_raw_html_blocks(text, lines, opaque));
    candidates.extend(scan_attribute_group_blocks(text, lines, opaque));
    candidates.extend(scan_pandoc_line_blocks(text, lines, opaque));
    let mut dollars = HashMap::<Vec<ContainerFrame>, usize>::new();
    let mut brackets = HashMap::<Vec<ContainerFrame>, usize>::new();
    let mut environments = HashMap::<Vec<ContainerFrame>, Vec<(String, usize)>>::new();

    for line in lines {
        if opaque[line.index] || line.lazy {
            continue;
        }
        let payload = line.payload(text);
        if payload == "$$" {
            if let Some(opener_index) = dollars.remove(line.key.as_slice()) {
                let opener = &lines[opener_index];
                candidates.push(Candidate::block(
                    RegionKind::MathDollarBlock,
                    opener.start,
                    line.end,
                    opener.context,
                    opener.scaffold(text).to_owned(),
                ));
            } else {
                dollars.insert(line.key.clone(), line.index);
            }
            continue;
        }
        if dollar_closer(payload) {
            if let Some(opener_index) = dollars.remove(line.key.as_slice()) {
                let opener = &lines[opener_index];
                candidates.push(Candidate::block(
                    RegionKind::MathDollarBlock,
                    opener.start,
                    line.end,
                    opener.context,
                    opener.scaffold(text).to_owned(),
                ));
            }
            continue;
        }
        if payload == "\\[" {
            brackets.entry(line.key.clone()).or_insert(line.index);
            continue;
        }
        if payload == "\\]" {
            if let Some(opener_index) = brackets.remove(line.key.as_slice()) {
                let opener = &lines[opener_index];
                candidates.push(Candidate::block(
                    RegionKind::MathBracketBlock,
                    opener.start,
                    line.end,
                    opener.context,
                    opener.scaffold(text).to_owned(),
                ));
            }
            continue;
        }
        if let Some((command, name)) = exact_environment_line(payload) {
            let stack = environments.entry(line.key.clone()).or_default();
            if command == "begin" {
                stack.push((name.to_owned(), line.index));
            } else if stack.last().is_some_and(|(open_name, _)| open_name == name) {
                let (_, opener_index) = stack.pop().expect("checked environment stack");
                let opener = &lines[opener_index];
                candidates.push(Candidate::block(
                    RegionKind::MathEnvironmentBlock,
                    opener.start,
                    line.end,
                    opener.context,
                    opener.scaffold(text).to_owned(),
                ));
            }
        }
    }
    arbitrate(candidates, 0, source.byte_length())
}

fn overlaps(range: ByteRange, ranges: &[ByteRange]) -> bool {
    ranges.iter().any(|other| range.0 < other.1 && other.0 < range.1)
}

fn structural_pipe_ranges(text: &str, start: usize, end: usize) -> Vec<ByteRange> {
    let code: Vec<ByteRange> = arbitrate(scan_backticks(text, start, end), start, end)
        .into_iter()
        .map(|candidate| (candidate.start, candidate.end))
        .collect();
    let mut boundaries = vec![start];
    let mut index = start;
    while index < end {
        if text.as_bytes()[index] == b'|'
            && escape_is_even(text, index)
            && !overlaps((index, index + 1), &code)
        {
            boundaries.push(index);
            boundaries.push(index + 1);
        }
        index += text[index..].chars().next().map_or(1, char::len_utf8);
    }
    boundaries.push(end);
    boundaries
        .chunks_exact(2)
        .filter_map(|pair| (pair[0] < pair[1]).then_some((pair[0], pair[1])))
        .collect()
}

fn is_heading(payload: &str) -> bool {
    let hashes = payload.as_bytes().iter().take_while(|byte| **byte == b'#').count();
    (1..=6).contains(&hashes)
        && (hashes == payload.len()
            || payload.as_bytes().get(hashes).is_some_and(|byte| matches!(byte, b' ' | b'\t')))
}

fn inline_scopes(source: &str, lines: &[Line], excluded: &[bool]) -> Vec<ByteRange> {
    let mut scopes = Vec::new();
    let mut paragraph_start = None;
    let mut paragraph_end = 0;
    let mut paragraph_key: Option<&[ContainerFrame]> = None;

    let flush = |scopes: &mut Vec<ByteRange>, start: &mut Option<usize>, end: usize| {
        if let Some(value) = start.take() {
            if value < end {
                scopes.push((value, end));
            }
        }
    };

    for line in lines {
        let payload = line.payload(source);
        if excluded[line.index]
            || payload.is_empty()
            || paragraph_key.is_some_and(|key| key != line.key.as_slice())
        {
            flush(&mut scopes, &mut paragraph_start, paragraph_end);
            paragraph_key = None;
        }
        if excluded[line.index] || payload.is_empty() {
            continue;
        }
        if payload.contains('|') {
            flush(&mut scopes, &mut paragraph_start, paragraph_end);
            paragraph_key = None;
            let (payload_start, _) = line_payload_bounds(source, line);
            scopes.extend(structural_pipe_ranges(source, payload_start, line.content_end));
            continue;
        }
        if is_heading(payload) || (line.starts_list && paragraph_start.is_some()) {
            flush(&mut scopes, &mut paragraph_start, paragraph_end);
            paragraph_key = None;
        }
        if is_heading(payload) {
            let (payload_start, _) = line_payload_bounds(source, line);
            scopes.push((payload_start, line.end));
            continue;
        }
        let (payload_start, _) = line_payload_bounds(source, line);
        paragraph_start.get_or_insert(payload_start);
        paragraph_key.get_or_insert(line.key.as_slice());
        paragraph_end = line.end;
    }
    flush(&mut scopes, &mut paragraph_start, paragraph_end);
    scopes
}

fn angle_inline_scopes(source: &str, lines: &[Line], excluded: &[bool]) -> Vec<ByteRange> {
    let mut scopes = Vec::new();
    let mut start = None;
    let mut end = 0;
    let mut frames: Option<&[ContainerFrame]> = None;
    for line in lines {
        let blank = line.payload(source).is_empty();
        if excluded[line.index]
            || blank
            || frames.is_some_and(|active| active != line.frames.as_slice())
        {
            if let Some(scope_start) = start.take() {
                scopes.push((scope_start, end));
            }
            frames = None;
        }
        if excluded[line.index] || blank {
            continue;
        }
        if start.is_none() {
            start = Some(line.content_start);
            frames = Some(line.frames.as_slice());
        }
        end = line.end;
    }
    if let Some(scope_start) = start {
        scopes.push((scope_start, end));
    }
    scopes
}

pub(crate) fn scan_protected_regions(
    source: &NormalizedSource,
) -> Result<Vec<ProtectedRegion>, PreservationError> {
    let lines = build_lines(&source.text);
    let opaque = opaque_line_flags(&source.text, &lines);
    let block_candidates = scan_blocks(source, &lines, &opaque);
    let mut excluded = opaque;
    for (line_index, line) in lines.iter().enumerate() {
        if block_candidates
            .iter()
            .any(|candidate| line.start < candidate.end && candidate.start < line.end)
        {
            excluded[line_index] = true;
        }
    }

    let mut inline_candidates = Vec::new();
    for (start, end) in angle_inline_scopes(&source.text, &lines, &excluded) {
        inline_candidates.extend(scan_angle_spans(&source.text, start, end));
    }
    for (start, end) in inline_scopes(&source.text, &lines, &excluded) {
        inline_candidates.extend(scan_inline_scope(&source.text, start, end));
    }
    let inline_candidates = arbitrate(inline_candidates, 0, source.byte_length());

    let mut candidates = block_candidates;
    candidates.extend(inline_candidates);
    candidates.sort_by_key(|candidate| candidate.start);
    let mut regions = Vec::with_capacity(candidates.len());
    let mut previous_end = 0;
    for candidate in candidates {
        if candidate.start < previous_end {
            return Err(PreservationError("selected block and inline regions overlap"));
        }
        previous_end = candidate.end;
        regions.push(ProtectedRegion::from_candidate(source, candidate, regions.len())?);
    }
    Ok(regions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preservation::normalization::normalize_source;

    #[test]
    fn code_wins_and_complete_math_survives() {
        let source = normalize_source("Code `$not$`, math $a * b$, then $c$.\n");
        let regions = scan_protected_regions(&source).expect("valid scan");
        let slices: Vec<&str> = regions.iter().map(|region| region.source.as_str()).collect();
        assert_eq!(slices, ["`$not$`", "$a * b$", "$c$"]);
    }

    #[test]
    fn block_math_owns_nested_markdown() {
        let source = normalize_source("> $$\n> a * b\n> $$ (eq)\n");
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].form, super::super::model::RegionForm::Block);
        assert_eq!(regions[0].source, source.text);
    }

    #[test]
    fn multiline_table_requires_a_complete_structural_pair() {
        let source = normalize_source(
            "-----\nHeader A   Header B\n--- ---\nvalue      value\n\nnext       row\n-----\n\n---\n",
        );
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].kind, RegionKind::PandocMultilineTable);
        assert!(regions[0].source.ends_with("next       row\n-----\n"));
    }

    #[test]
    fn callout_marker_must_be_the_first_line_of_its_quote() {
        let source = normalize_source(
            "> ordinary first line\n> [!note] too late\n\n> [!tip]- valid\ncontinued lazily\n",
        );
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].kind, RegionKind::ObsidianCallout);
        assert_eq!(regions[0].source, "> [!tip]- valid\ncontinued lazily\n");
    }

    #[test]
    fn colon_container_closers_ignore_run_length_and_fenced_code() {
        let source =
            normalize_source(":::: outer\n```text\n:::\n```\n::: inner\nbody\n:::::\n:::\n");
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].kind, RegionKind::ColonContainer);
        assert_eq!(regions[0].source, source.text);
    }

    #[test]
    fn toml_frontmatter_requires_exact_root_delimiters() {
        let source =
            normalize_source("+++\ntitle = \"Exact\"\n+++\n\n  +++\nnot frontmatter\n  +++\n");
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].kind, RegionKind::TomlFrontmatter);
        assert_eq!(regions[0].source, "+++\ntitle = \"Exact\"\n+++\n");
    }

    #[test]
    fn definition_lists_require_marker_columns_and_stop_before_plain_suffix() {
        let source = normalize_source(
            "Term\n: Definition\n\n  Continued block\n\nSuffix\n\nNot a term\n   : over-indented\n",
        );
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].kind, RegionKind::DefinitionList);
        assert_eq!(regions[0].source, "Term\n: Definition\n\n  Continued block\n");
    }

    #[test]
    fn grid_tables_require_compatible_outer_borders() {
        let source = normalize_source(
            "+-----+-----+\n| A   | B   |\n+=====+=====+\n| 1   | 2   |\n+-----+-----+\n\n+---+---+\nnot a table\n+---+---+\n",
        );
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].kind, RegionKind::PandocGridTable);
        assert!(regions[0].source.ends_with("| 1   | 2   |\n+-----+-----+\n"));
    }

    #[test]
    fn raw_html_uses_commonmark_boundaries_and_inline_angle_scopes() {
        let source = normalize_source(
            "  <script>\n*not emphasis*\n\n</script>\n\nParagraph <x-card\n data-a=\"raw\"> tail\n",
        );
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].kind, RegionKind::RawHtmlBlock);
        assert_eq!(regions[0].source, "  <script>\n*not emphasis*\n\n</script>\n");
        assert_eq!(regions[0].scaffold_prefix, "");
        assert_eq!(regions[1].kind, RegionKind::RawHtmlInline);
        assert_eq!(regions[1].source, "<x-card\n data-a=\"raw\">");
    }

    #[test]
    fn attribute_groups_require_valid_attributes_and_compatible_placement() {
        let source = normalize_source(
            "# Heading {#id .wide key=\"raw ...\"}\n\n[link](url){.button}{#second} ordinary {.detached}\n\n{.standalone data-ü=\"välue\"}\n\n[bad](url){#}\n",
        );
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 4);
        assert_eq!(regions[0].kind, RegionKind::AttributeGroupInline);
        assert_eq!(regions[1].source, "{.button}");
        assert_eq!(regions[2].source, "{#second}");
        assert_eq!(regions[3].kind, RegionKind::AttributeGroupBlock);
        assert_eq!(regions[3].source, "{.standalone data-ü=\"välue\"}\n");
    }

    #[test]
    fn line_blocks_require_spaced_bars_and_do_not_claim_tables() {
        let source = normalize_source(
            "| first\n|\n| third $x$\n\n> | quoted\n> | continued\n\n| Header | Value |\n| --- | --- |\n| one | two |\n\n\\| escaped\n|unspaced\n    | code\n",
        );
        let regions = scan_protected_regions(&source).expect("valid scan");
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].kind, RegionKind::PandocLineBlock);
        assert_eq!(regions[0].source, "| first\n|\n| third $x$\n");
        assert_eq!(regions[1].kind, RegionKind::PandocLineBlock);
        assert_eq!(regions[1].source, "> | quoted\n> | continued\n");
    }
}
