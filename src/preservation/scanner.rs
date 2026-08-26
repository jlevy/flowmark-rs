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

    fn scaffold<'a>(&self, source: &'a str) -> &'a str {
        let (payload_start, _) = line_payload_bounds(source, self);
        &source[self.start..payload_start]
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

fn scan_blocks(source: &NormalizedSource, lines: &[Line], opaque: &[bool]) -> Vec<Candidate> {
    let text = source.text.as_str();
    let mut candidates = Vec::new();
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
    for (start, end) in inline_scopes(&source.text, &lines, &excluded) {
        inline_candidates.extend(scan_inline_scope(&source.text, start, end));
    }

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
}
