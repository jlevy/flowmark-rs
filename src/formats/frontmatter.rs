//! YAML frontmatter handling.
//!
//! Ported from Python: flowmark/formats/frontmatter.py

/// Split a text document into frontmatter and content parts.
///
/// Checks if the string starts with YAML frontmatter, delimited by `---`
/// lines. If so, returns the frontmatter (including the `---` lines) and the
/// rest of the document. If no frontmatter is found, returns an empty string
/// and the original text.
pub fn split_frontmatter(text: &str) -> (String, String) {
    let lines: Vec<&str> = text.split('\n').collect();

    // Skip empty lines at the beginning
    let mut start_idx = 0;
    while start_idx < lines.len() && lines[start_idx].trim().is_empty() {
        start_idx += 1;
    }

    // If no content or doesn't start with '---', return empty frontmatter
    if start_idx >= lines.len() || lines[start_idx].trim() != "---" {
        return (String::new(), text.to_string());
    }

    // Look for the closing '---'
    let mut end_idx = start_idx + 1;
    while end_idx < lines.len() {
        if lines[end_idx].trim() == "---" {
            // Found the closing delimiter - extract frontmatter and content
            let frontmatter = lines[start_idx..=end_idx].join("\n") + "\n";
            let content = lines[end_idx + 1..].join("\n");
            return (frontmatter, content);
        }
        end_idx += 1;
    }

    // If no closing delimiter found, everything is considered frontmatter
    (text.to_string(), String::new())
}

/// Check if the text starts with YAML frontmatter.
pub fn has_frontmatter(text: &str) -> bool {
    let (frontmatter, _) = split_frontmatter(text);
    !frontmatter.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter_with_frontmatter() {
        let text = "---\ntitle: Test\n---\nContent here";
        let (fm, content) = split_frontmatter(text);
        assert_eq!(fm, "---\ntitle: Test\n---\n");
        assert_eq!(content, "Content here");
    }

    #[test]
    fn test_split_frontmatter_without_frontmatter() {
        let text = "No frontmatter here";
        let (fm, content) = split_frontmatter(text);
        assert_eq!(fm, "");
        assert_eq!(content, "No frontmatter here");
    }

    #[test]
    fn test_split_frontmatter_empty() {
        let text = "";
        let (fm, content) = split_frontmatter(text);
        assert_eq!(fm, "");
        assert_eq!(content, "");
    }

    #[test]
    fn test_has_frontmatter() {
        assert!(has_frontmatter("---\ntitle: Test\n---\nContent"));
        assert!(!has_frontmatter("No frontmatter"));
    }

    #[test]
    fn test_split_frontmatter_no_closing() {
        let text = "---\ntitle: Test\nno closing";
        let (fm, content) = split_frontmatter(text);
        assert_eq!(fm, text);
        assert_eq!(content, "");
    }
}
