//! Gitignore and tool-specific ignore file handling.

use std::path::Path;

use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// Read an ignore file and return parsed patterns as strings.
/// Returns `None` if the file doesn't exist, is unreadable, or has no active rules.
pub fn read_ignore_patterns(path: &Path) -> Option<Vec<String>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return None;
    };

    let lines: Vec<String> = text
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .map(|s| s.trim().to_string())
        .collect();

    if lines.is_empty() { None } else { Some(lines) }
}

/// Read an ignore file (gitignore syntax) and return a compiled `Gitignore` matcher.
/// Returns `None` if the file has no active rules or cannot be read.
pub fn read_ignore_file(path: &Path) -> Option<Gitignore> {
    let patterns = read_ignore_patterns(path)?;

    let root = path.parent().unwrap_or(Path::new("."));
    let mut builder = GitignoreBuilder::new(root);
    for line in &patterns {
        builder.add_line(None, line).ok()?;
    }
    builder.build().ok()
}

/// Read `.gitignore` in the given directory and return a compiled matcher,
/// or `None` if the file doesn't exist or is empty.
pub(crate) fn load_gitignore(directory: &Path) -> Option<Gitignore> {
    let gitignore = directory.join(".gitignore");
    if !gitignore.is_file() {
        return None;
    }
    read_ignore_file(&gitignore)
}

/// Walk up from `start_dir` looking for `.{tool_name}ignore` (e.g., `.flowmarkignore`).
/// Returns patterns from the first found, or `None`.
pub(crate) fn load_tool_ignore_patterns(tool_name: &str, start_dir: &Path) -> Option<Vec<String>> {
    let ignore_name = format!(".{tool_name}ignore");
    let Ok(mut current) = start_dir.canonicalize() else {
        return None;
    };
    loop {
        let candidate = current.join(&ignore_name);
        if candidate.is_file() {
            return read_ignore_patterns(&candidate);
        }
        let Some(parent) = current.parent() else {
            break;
        };
        if parent == current {
            break;
        }
        current = parent.to_path_buf();
    }
    None
}
