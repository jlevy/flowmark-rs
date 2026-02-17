//! Sentence splitting using regex heuristics.
//!
//! Ported from Python: flowmark/linewrapping/sentence_split_regex.py

use regex::Regex;
use std::sync::LazyLock;

/// Minimum length for a sentence in characters.
pub const SENTENCE_MIN_LENGTH: usize = 15;

/// Regex for detecting end of sentence.
///
/// Matches a word ending in a lowercase letter followed by sentence-ending
/// punctuation (`.?!`), optionally with quotes.
///
/// Note: Python uses `\p{L}` and `\p{Ll}` for Unicode letter classes.
/// Rust's `regex` crate supports these.
static SENTENCE_END_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Allow optional closing brackets/parens/angle brackets between word and punctuation
    // to handle cases like [link](url). and <tag>word</tag>.
    Regex::new(r"(\b\p{L}+[\p{Ll}])[)\]>]*([.?!]['\x22\u{2019}\u{201d}]?|['\x22\u{2019}\u{201d}][.?!]) *$")
        .unwrap()
});

/// Check if a word looks like the end of a sentence.
pub fn heuristic_end_of_sentence(word: &str) -> bool {
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

/// Return the first n sentences from the text.
pub fn first_sentences(text: &str, n: usize, min_length: usize) -> Vec<String> {
    let sentences = split_sentences_regex(text, min_length);
    sentences.into_iter().take(n).collect()
}

/// Return the first sentence from the text. Returns input text unchanged if no
/// sentences are found.
pub fn first_sentence(text: &str, min_length: usize) -> String {
    let sentences = split_sentences_regex(text, min_length);
    sentences
        .into_iter()
        .next()
        .unwrap_or_else(|| text.to_string())
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
        let sentences = split_sentences_regex(text, SENTENCE_MIN_LENGTH);
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
        let s = first_sentence(text, SENTENCE_MIN_LENGTH);
        assert_eq!(s, "Hello world. This is a test.");
    }
}
