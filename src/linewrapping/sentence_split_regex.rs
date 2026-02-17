//! Sentence splitting using regex heuristics.
//!
//! Ported from Python: flowmark/linewrapping/sentence_split_regex.py

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for detecting end of sentence. Matches words ending in lowercase letter
/// followed by sentence-ending punctuation (with optional quotes/parens).
///
/// Python: `(\b\p{L}+[\p{Ll}])([.?!]['\"'")]?|['\"'")][.?!]) *$`
/// Note: The Python version uses the `regex` crate which supports Unicode properties.
/// We approximate this with a simpler pattern that works for Latin text.
static SENTENCE_END_RE: Lazy<Regex> = Lazy::new(|| {
    // Match: word boundary, 2+ letters ending in lowercase, then sentence-end punct
    // with optional quote/paren wrapping
    Regex::new(r"(?:\b\p{L}+\p{Ll})([.?!]['\x22\u{2018}\u{2019}\u{201c}\u{201d}]?|['\x22\u{2018}\u{2019}\u{201c}\u{201d}][.?!])\s*$").unwrap()
});

/// Default minimum sentence length in characters.
pub const SENTENCE_MIN_LENGTH: usize = 15;

/// Check if a word appears to end a sentence.
pub fn heuristic_end_of_sentence(word: &str) -> bool {
    SENTENCE_END_RE.is_match(word)
}

/// Split text into sentences using an approximate, fast regex heuristic.
///
/// Goal is to be conservative, not perfect, avoiding excessive breaks.
///
/// The default heuristic: End of sentence must be two letters or more,
/// with the last letter lowercase, followed by a period, exclamation point,
/// or question mark. A final or preceding parenthesis or quote is allowed.
pub fn split_sentences_regex(text: &str, min_length: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut sentences: Vec<String> = Vec::new();
    let mut sentence: Vec<&str> = Vec::new();
    let mut words_len: usize = 0;

    for word in words {
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
    split_sentences_regex(text, min_length)
        .into_iter()
        .take(n)
        .collect()
}

/// Return the first sentence from the text. Returns input text unchanged if no
/// sentences are found.
pub fn first_sentence(text: &str, min_length: usize) -> String {
    let sentences = split_sentences_regex(text, min_length);
    sentences
        .into_iter()
        .next()
        .unwrap_or_else(|| text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LONG_TEXT: &str = "End of sentence must be two letters or more, with the last letter lowercase, followed by a period, exclamation point, question mark. A final or preceding parenthesis or quote is allowed. Does not break on colon or semicolon as that seems to have false positives too often with code or other syntax.";

    const FIRST_SENTENCE: &str = "End of sentence must be two letters or more, with the last letter lowercase, followed by a period, exclamation point, question mark.";

    #[test]
    fn test_split_sentences() {
        assert_eq!(
            split_sentences_regex("test!", SENTENCE_MIN_LENGTH),
            vec!["test!"]
        );
        assert_eq!(
            split_sentences_regex("test! random words", SENTENCE_MIN_LENGTH),
            vec!["test! random words"]
        );

        let split = split_sentences_regex(LONG_TEXT, SENTENCE_MIN_LENGTH);
        assert_eq!(split.len(), 3);
        assert_eq!(split[0], FIRST_SENTENCE);
    }

    #[test]
    fn test_first_sentence() {
        assert_eq!(first_sentence(LONG_TEXT, SENTENCE_MIN_LENGTH), FIRST_SENTENCE);
        assert_eq!(first_sentence("", SENTENCE_MIN_LENGTH), "");
        assert_eq!(first_sentence(" ", SENTENCE_MIN_LENGTH), "");
        assert_eq!(first_sentence("hello", SENTENCE_MIN_LENGTH), "hello");
        assert_eq!(first_sentence(" hello\n", SENTENCE_MIN_LENGTH), "hello");
    }
}
