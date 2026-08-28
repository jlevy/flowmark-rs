//! The wrap-width invariant, asserted directly rather than by comparison.
//!
//! Every other wrapping gate in this repo is a *comparison*: golden files, the shared
//! conformance corpus, and the Rust/Python corpus audit all check that two outputs agree.
//! A comparison stays green when both sides are wrong in the same way, and it only covers
//! the inputs someone thought to collect. Neither caught a line-start escape being measured
//! one column short, because no corpus document happened to put an escaped numeral at a
//! wrapped line start with words filling to the boundary.
//!
//! This gate asserts the property instead: for input built only from short breakable words,
//! no output line may exceed the configured width. The generated space sweeps the wrap
//! boundary, every CommonMark-escapable character, several widths, and the block contexts
//! that carry a continuation indent — so a width miscount anywhere in either wrap loop
//! surfaces here without anyone having to guess the shape of the input first.

use flowmark::config::ListSpacing;
use flowmark::fill_markdown;

/// Widths worth sweeping: the default, plus a narrow one where an off-by-one is easier to hit.
const WIDTHS: [usize; 2] = [40, 88];

/// All 32 CommonMark-escapable ASCII punctuation characters.
const ESCAPABLE: [char; 32] = [
    '\\', '~', '*', '#', '-', '+', '>', '.', '!', '[', ']', '(', ')', '{', '}', '$', '_', '|', '`',
    '"', '%', '&', '\'', ',', '/', ':', ';', '<', '=', '?', '@', '^',
];

/// Block contexts whose continuation lines carry an indent or a marker prefix.
const CONTEXTS: [(&str, &str); 3] = [("paragraph", ""), ("list item", "- "), ("blockquote", "> ")];

fn fmt(input: &str, width: usize) -> String {
    fill_markdown(input, true, width, true, false, false, false, None, ListSpacing::Preserve)
}

/// A run of short words whose joined length is exactly `target`.
fn filler(target: usize) -> Option<String> {
    if target == 0 {
        return None;
    }
    let mut words: Vec<String> = Vec::new();
    let mut len = 0usize;
    loop {
        let next = if len == 0 { 5 } else { len + 6 };
        if next > target {
            break;
        }
        words.push("alpha".to_owned());
        len = next;
    }
    let remainder = target - len;
    if remainder > 0 {
        // One padding word plus the space that precedes it, unless it would be empty.
        if remainder == 1 && !words.is_empty() {
            return None;
        }
        let pad = if words.is_empty() { remainder } else { remainder - 1 };
        words.push("b".repeat(pad));
    }
    Some(words.join(" "))
}

/// Every output line must fit the width. Inputs use only short breakable words, so there is
/// no token that legitimately overruns and the bound is unconditional.
///
/// Both the head and the tail are swept. The head decides *where* the escaped token lands
/// relative to the wrap boundary; the tail decides how exactly the resulting line fills to
/// that boundary. Sweeping the head alone — with a fixed tail — reaches the interesting
/// position but almost never lands on the column where an off-by-one becomes visible, which
/// is precisely how a one-column miscount survived every gate in this repo.
#[test]
fn no_output_line_exceeds_the_wrap_width() {
    let mut checked = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for width in WIDTHS {
        for escapable in ESCAPABLE {
            let lowest = width.saturating_sub(6).max(1);
            for head_len in lowest..=width {
                let Some(head) = filler(head_len) else { continue };
                for tail_words in 0..=10usize {
                    for pad in 1..=6usize {
                        let tail = format!("{}{}", "alpha ".repeat(tail_words), "b".repeat(pad));
                        let input = format!(
                            "{head} 5\\{escapable} {tail} end end end end end end end end.\n"
                        );

                        for line in fmt(&input, width).lines() {
                            checked += 1;
                            let columns = line.chars().count();
                            if columns > width {
                                failures.push(format!(
                                    "width={width} escape={escapable:?} head_len={head_len} \
                                     tail_words={tail_words} pad={pad}: {columns} cols: {line:?}"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    assert!(checked > 10_000, "generated space collapsed: only {checked} lines checked");
    assert!(
        failures.is_empty(),
        "{} of {checked} output lines exceed their wrap width:\n  {}",
        failures.len(),
        failures.iter().take(10).cloned().collect::<Vec<_>>().join("\n  ")
    );
}

/// The same invariant inside the block contexts that carry a continuation indent, where the
/// available width is reduced by a marker or indent the wrapper has to account for.
#[test]
fn no_output_line_exceeds_the_wrap_width_in_indented_contexts() {
    let mut checked = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for (context_name, prefix) in CONTEXTS {
        for width in WIDTHS {
            let lowest = width.saturating_sub(6).max(1);
            for head_len in lowest..=width {
                let Some(head) = filler(head_len.saturating_sub(prefix.len())) else {
                    continue;
                };
                for tail_words in 0..=8usize {
                    for pad in 1..=6usize {
                        let tail = format!("{}{}", "alpha ".repeat(tail_words), "b".repeat(pad));
                        let input = format!(
                            "{prefix}{head} 5\\. {tail} end end end end end end end end.\n"
                        );

                        for line in fmt(&input, width).lines() {
                            checked += 1;
                            let columns = line.chars().count();
                            if columns > width {
                                failures.push(format!(
                                    "context={context_name} width={width} head_len={head_len} \
                                     tail_words={tail_words} pad={pad}: {columns} cols: {line:?}"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    assert!(checked > 1000, "generated space collapsed: only {checked} lines checked");
    assert!(
        failures.is_empty(),
        "{} of {checked} output lines exceed their wrap width:\n  {}",
        failures.len(),
        failures.iter().take(10).cloned().collect::<Vec<_>>().join("\n  ")
    );
}

/// Reformatting must not keep changing the output. A width miscount often shows up here
/// first: the line breaks somewhere the next pass disagrees with.
#[test]
fn wrapping_around_escapes_reaches_a_fixed_point() {
    let mut checked = 0usize;

    for width in WIDTHS {
        for escapable in ESCAPABLE {
            let lowest = width.saturating_sub(4).max(1);
            for head_len in lowest..=width {
                let Some(head) = filler(head_len) else { continue };
                for tail_words in 0..=6usize {
                    let tail = format!("{}bbb", "alpha ".repeat(tail_words));
                    let input = format!("{head} 5\\{escapable} {tail} tail tail end.\n");

                    let once = fmt(&input, width);
                    let twice = fmt(&once, width);
                    assert_eq!(
                        once, twice,
                        "not a fixed point at width={width} escape={escapable:?} \
                         head_len={head_len} tail_words={tail_words}"
                    );
                    checked += 1;
                }
            }
        }
    }

    assert!(checked > 1000, "generated space collapsed: only {checked} cases checked");
}

/// The sweep must actually place an escaped numeral first on a continuation line — the
/// position where the backslash is retained. Without this, the gates above can pass by
/// never constructing the case they exist to cover.
#[test]
fn the_sweep_reaches_an_escaped_numeral_at_a_line_start() {
    let mut reached = 0usize;

    for width in WIDTHS {
        let lowest = width.saturating_sub(6).max(1);
        for head_len in lowest..=width {
            let Some(head) = filler(head_len) else { continue };
            let input = format!("{head} 5\\. tail tail tail tail tail tail end.\n");
            for line in fmt(&input, width).lines() {
                if line.starts_with("5\\.") {
                    reached += 1;
                }
            }
        }
    }

    assert!(reached > 0, "no generated case put an escaped numeral at a line start");
}
