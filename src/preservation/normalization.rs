//! UTF-8 source normalization shared by every formatter boundary.

use super::model::NormalizedSource;

const BOM: char = '\u{feff}';

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn canonical_terminal_lf(mut text: String) -> String {
    while text.ends_with('\n') {
        text.pop();
    }
    text.push('\n');
    text
}

pub(crate) fn normalize_source(source: &str) -> NormalizedSource {
    let had_bom = source.starts_with(BOM);
    let payload = source.strip_prefix(BOM).unwrap_or(source);
    NormalizedSource { text: canonical_terminal_lf(normalize_line_endings(payload)), had_bom }
}

pub(crate) fn finalize_output(source: &NormalizedSource, output: &str) -> String {
    let output = output.strip_prefix(BOM).unwrap_or(output);
    let mut finalized = canonical_terminal_lf(normalize_line_endings(output));
    if source.had_bom {
        finalized.insert(0, BOM);
    }
    finalized
}
