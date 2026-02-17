use flowmark::{first_sentence, split_sentences_regex};

const LONG_TEXT: &str = "End of sentence must be two letters or more, with the last letter lowercase, followed by a period, exclamation point, question mark. A final or preceding parenthesis or quote is allowed. Does not break on colon or semicolon as that seems to have false positives too often with code or other syntax.";

const FIRST_SENTENCE: &str = "End of sentence must be two letters or more, with the last letter lowercase, followed by a period, exclamation point, question mark.";

#[test]
fn test_split_sentences() {
    assert_eq!(split_sentences_regex("test!", 0), vec!["test!"]);
    assert_eq!(
        split_sentences_regex("test! random words", 0),
        vec!["test!", "random words"]
    );

    let split_sentences = split_sentences_regex(LONG_TEXT, 15);
    assert_eq!(split_sentences.len(), 3);
    assert_eq!(split_sentences[0], FIRST_SENTENCE);
}

#[test]
fn test_first_sentence() {
    assert_eq!(first_sentence(LONG_TEXT, 15), FIRST_SENTENCE);

    assert_eq!(first_sentence("", 15), "");
    assert_eq!(first_sentence("hello", 15), "hello");
}
