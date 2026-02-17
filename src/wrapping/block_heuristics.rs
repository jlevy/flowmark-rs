//! Block content detection using simple line-based heuristics.
//!
//! Ported from Python: `flowmark/linewrapping/block_heuristics.py`

/// Check if a line looks like a GFM table row.
pub(crate) fn line_is_table_row(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

/// Check if a line looks like a `CommonMark` list item.
///
/// Per `CommonMark` spec:
/// - Bullet list markers: `-`, `+`, or `*` followed by at least one space/tab
/// - Ordered list markers: 1-9 digits followed by `.` or `)` then space/tab
pub(crate) fn line_is_list_item(line: &str) -> bool {
    let stripped = line.trim_start();
    if stripped.is_empty() {
        return false;
    }

    let bytes = stripped.as_bytes();

    // Unordered list: -, *, + followed by space or tab
    if matches!(bytes[0], b'-' | b'*' | b'+') {
        return bytes.len() > 1 && matches!(bytes[1], b' ' | b'\t');
    }

    // Ordered list: digits followed by . or ) then space or tab
    if bytes[0].is_ascii_digit() {
        let mut i = 1;
        while i < bytes.len() && i < 9 && bytes[i].is_ascii_digit() {
            i += 1;
        }
        // Must have . or ) followed by space/tab
        if i < bytes.len() && matches!(bytes[i], b'.' | b')') && i + 1 < bytes.len() && matches!(bytes[i + 1], b' ' | b'\t') {
            return true;
        }
    }

    false
}

/// Check if a line is block content (table row or list item).
pub(crate) fn line_is_block_content(line: &str) -> bool {
    line_is_table_row(line) || line_is_list_item(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_is_table_row() {
        assert!(line_is_table_row("| A | B |"));
        assert!(line_is_table_row("|---|---|"));
        assert!(line_is_table_row("| Cell |"));
        assert!(line_is_table_row("  | Indented |"));
        assert!(line_is_table_row("\t| Tab indented |"));

        assert!(!line_is_table_row("Not a table"));
        assert!(!line_is_table_row("A | B"));
        assert!(!line_is_table_row(""));
        assert!(!line_is_table_row("   "));
    }

    #[test]
    fn test_line_is_list_item_unordered() {
        assert!(line_is_list_item("- Item"));
        assert!(line_is_list_item("* Item"));
        assert!(line_is_list_item("+ Item"));
        assert!(line_is_list_item("-\tTab after marker"));
        assert!(line_is_list_item("  - Indented item"));
        assert!(line_is_list_item("- "));

        assert!(!line_is_list_item("-"));
        assert!(!line_is_list_item("-Item"));
        assert!(!line_is_list_item("---"));
        assert!(!line_is_list_item("***"));

        assert!(line_is_list_item("- -"));
    }

    #[test]
    fn test_line_is_list_item_ordered() {
        assert!(line_is_list_item("1. Item"));
        assert!(line_is_list_item("1) Item"));
        assert!(line_is_list_item("10. Item"));
        assert!(line_is_list_item("999. Item"));
        assert!(line_is_list_item("1.\tTab after marker"));
        assert!(line_is_list_item("  1. Indented"));
        assert!(line_is_list_item("1. "));

        assert!(!line_is_list_item("1.0 version"));
        assert!(!line_is_list_item("1.0.0"));
        assert!(!line_is_list_item("1.Item"));
        assert!(!line_is_list_item("1."));
        assert!(!line_is_list_item("1"));
        assert!(!line_is_list_item("12345678901. Item"));
    }

    #[test]
    fn test_line_is_block_content() {
        assert!(line_is_block_content("| A | B |"));
        assert!(line_is_block_content("|---|---|"));
        assert!(line_is_block_content("- Item"));
        assert!(line_is_block_content("1. Item"));
        assert!(!line_is_block_content("Regular text"));
        assert!(!line_is_block_content("1.0.0 version"));
        assert!(!line_is_block_content(""));
    }
}
