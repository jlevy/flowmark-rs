//! Sentence splitting using regex heuristics.
//!
//! Ported from Python: `flowmark/linewrapping/sentence_split_regex.py`

use regex::Regex;
use std::sync::LazyLock;

/// Regex for detecting end of sentence.
///
/// Matches a word ending in a lowercase letter followed by sentence-ending
/// punctuation (`.?!`), optionally with quotes.
///
/// Note: Python uses `\p{L}` and `\p{Ll}` for Unicode letter classes.
/// Rust's `regex` crate supports these.
static SENTENCE_END_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Match sentence-ending punctuation (.?!) optionally followed by a closing
    // quote or paren, OR a closing quote/paren followed by punctuation.
    // Matches Python's regex: `)` is in the same character class as quotes,
    // appearing either before or after the punctuation mark.
    // This avoids false positives from URLs like [Google](url)." where the
    // `)` from the link syntax would otherwise bridge to the sentence-final `."`
    Regex::new(
        r"(\b\p{L}+[\p{Ll}])([.?!]['\x22\u{2019}\u{201d})]?|['\x22\u{2019}\u{201d})][.?!]) *$",
    )
    .expect("valid SENTENCE_END_RE regex")
});

/// Check if a word looks like the end of a sentence.
pub(crate) fn heuristic_end_of_sentence(word: &str) -> bool {
    SENTENCE_END_RE.is_match(word)
}

/// Split text into sentences using an approximate, fast regex heuristic.
///
/// Goal is to be conservative, not perfect, avoiding excessive breaks.
///
/// The default heuristic: End of sentence must be two letters or more,
/// with the last letter lowercase, followed by a period, exclamation point,
/// question mark. A final or preceding parenthesis or quote is allowed.
pub fn split_sentences_regex(text: &str, min_length: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut sentences: Vec<String> = Vec::new();
    let mut sentence: Vec<&str> = Vec::new();
    let mut words_len: usize = 0;

    for word in &words {
        sentence.push(word);
        words_len += word.len();
        let sentence_len = words_len + sentence.len() - 1;
        if heuristic_end_of_sentence(word) && sentence_len >= min_length {
            sentences.push(sentence.join(" "));
            sentence.clear();
            words_len = 0;
        }
    }

    if !sentence.is_empty() {
        sentences.push(sentence.join(" "));
    }

    sentences
}

/// Atomic-aware sentence splitter: never breaks a sentence inside a Markdown
/// link, code span, autolink, or bare URL.
///
/// Ported from Python `split_sentences_atomic` in v0.7.0 (commit `c9bc36f`).
/// Like `split_sentences_regex` but iterates over "atomic words" — whitespace-
/// delimited tokens with `MARKDOWN_INLINE_PATTERN` matches treated as opaque
/// indivisible units that glue to adjacent non-space characters. Sentences are
/// returned as verbatim slices of the input (no whitespace normalization),
/// matching Python `SentenceSpan.text == source[start:end]`.
pub fn split_sentences_atomic(text: &str, min_length: usize) -> Vec<String> {
    let atomic_matches: Vec<(usize, usize)> =
        crate::wrapping::atomic_patterns::MARKDOWN_INLINE_PATTERN
            .find_iter(text)
            .map(|m| (m.start(), m.end()))
            .collect();
    let mut atomic_iter = atomic_matches.into_iter().peekable();

    let mut current_word = String::new();
    let mut cw_start: usize = 0;
    let mut cw_end: usize = 0;
    let mut have_sent = false;
    let mut sent_start: usize = 0;
    let mut sent_end: usize = 0;
    let mut char_total: usize = 0;
    let mut word_total: usize = 0;
    let mut sentences: Vec<String> = Vec::new();

    let flush_word = |current_word: &mut String,
                      cw_start: usize,
                      cw_end: usize,
                      have_sent: &mut bool,
                      sent_start: &mut usize,
                      sent_end: &mut usize,
                      char_total: &mut usize,
                      word_total: &mut usize,
                      sentences: &mut Vec<String>| {
        if current_word.is_empty() {
            return;
        }
        if !*have_sent {
            *sent_start = cw_start;
            *have_sent = true;
        }
        *sent_end = cw_end;
        *char_total += current_word.chars().count();
        *word_total += 1;
        let sentence_len = *char_total + *word_total - 1;
        if heuristic_end_of_sentence(current_word) && sentence_len >= min_length {
            sentences.push(text[*sent_start..*sent_end].to_string());
            *have_sent = false;
            *sent_end = 0;
            *char_total = 0;
            *word_total = 0;
        }
        current_word.clear();
    };

    let mut pos: usize = 0;
    while pos < text.len() {
        // If an atomic match starts at this position, consume it as a unit.
        if let Some(&(s, e)) = atomic_iter.peek() {
            if s == pos {
                if current_word.is_empty() {
                    cw_start = s;
                }
                current_word.push_str(&text[s..e]);
                cw_end = e;
                pos = e;
                atomic_iter.next();
                continue;
            }
            // Skip any atomic matches that overlap a position we've already passed
            // (regex returns non-overlapping matches in order, so this only happens
            // if our cursor advanced past `s` mid-character — shouldn't occur).
            if s < pos {
                atomic_iter.next();
                continue;
            }
        }
        // Plain-text character.
        let c = text[pos..].chars().next().expect("non-empty remaining text");
        let clen = c.len_utf8();
        if c.is_whitespace() {
            flush_word(
                &mut current_word,
                cw_start,
                cw_end,
                &mut have_sent,
                &mut sent_start,
                &mut sent_end,
                &mut char_total,
                &mut word_total,
                &mut sentences,
            );
        } else {
            if current_word.is_empty() {
                cw_start = pos;
            }
            current_word.push(c);
            cw_end = pos + clen;
        }
        pos += clen;
    }
    // Final word + trailing sentence.
    flush_word(
        &mut current_word,
        cw_start,
        cw_end,
        &mut have_sent,
        &mut sent_start,
        &mut sent_end,
        &mut char_total,
        &mut word_total,
        &mut sentences,
    );
    if have_sent {
        sentences.push(text[sent_start..sent_end].to_string());
    }
    sentences
}

/// Return the first n sentences from the text.
/// Not used in the production formatting pipeline.
pub fn first_sentences(text: &str, n: usize, min_length: usize) -> Vec<String> {
    let sentences = split_sentences_regex(text, min_length);
    sentences.into_iter().take(n).collect()
}

/// Return the first sentence from the text. Returns input text unchanged if no
/// sentences are found.
/// Not used in the production formatting pipeline.
pub fn first_sentence(text: &str, min_length: usize) -> String {
    let sentences = split_sentences_regex(text, min_length);
    sentences.into_iter().next().unwrap_or_else(|| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heuristic_end_of_sentence() {
        assert!(heuristic_end_of_sentence("word."));
        assert!(heuristic_end_of_sentence("word?"));
        assert!(heuristic_end_of_sentence("word!"));
        assert!(!heuristic_end_of_sentence("word"));
        assert!(!heuristic_end_of_sentence("A."));
        assert!(!heuristic_end_of_sentence("1."));
    }

    #[test]
    fn test_split_sentences() {
        let text = "Hello world. This is a test. And another sentence.";
        let sentences = split_sentences_regex(text, 15);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "Hello world. This is a test.");
        assert_eq!(sentences[1], "And another sentence.");
    }

    #[test]
    fn test_split_sentences_no_min() {
        let text = "Hello world. This is a test. And another sentence.";
        let sentences = split_sentences_regex(text, 0);
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn test_first_sentence() {
        let text = "Hello world. This is a test.";
        let s = first_sentence(text, 15);
        assert_eq!(s, "Hello world. This is a test.");
    }
}
