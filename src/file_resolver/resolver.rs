//! `FileResolver` — main entry point for file discovery.
//!
//! Resolves a mix of files, directories, and glob patterns into a deduplicated,
//! sorted list of concrete file paths, applying all configured filters.
//!
//! Uses `ignore::WalkBuilder` (the same walker used by ripgrep) for efficient
//! directory traversal with native gitignore support.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use super::config::FileResolverConfig;

/// Characters that indicate a path is a glob pattern rather than a literal path.
const GLOB_CHARS: &[char] = &['*', '?', '['];

/// Discovers files matching configured include patterns while respecting
/// gitignore, tool-specific ignore files, and default/custom exclusions.
pub struct FileResolver {
    config: FileResolverConfig,
    exclude_patterns: Vec<String>,
    compiled_includes: Vec<glob::Pattern>,
    compiled_dir_excludes: Vec<glob::Pattern>,
}

impl FileResolver {
    /// Create a new `FileResolver` with the given configuration.
    pub fn new(config: FileResolverConfig) -> Self {
        let exclude_patterns = config.effective_exclude();
        let include_patterns = config.effective_include();

        let compiled_includes =
            include_patterns.iter().filter_map(|p| glob::Pattern::new(p).ok()).collect();

        // Pre-compile directory exclude patterns (patterns ending with '/')
        let compiled_dir_excludes = exclude_patterns
            .iter()
            .filter(|p| p.ends_with('/'))
            .filter_map(|p| glob::Pattern::new(&p[..p.len() - 1]).ok())
            .collect();

        Self { config, exclude_patterns, compiled_includes, compiled_dir_excludes }
    }

    /// Resolve input paths into a sorted, deduplicated list of files.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` with `NotFound` kind if a path doesn't exist
    /// and is not a glob pattern.
    pub fn resolve(&mut self, paths: &[&str]) -> std::io::Result<Vec<PathBuf>> {
        let mut seen: HashSet<PathBuf> = HashSet::new();
        let mut result: Vec<PathBuf> = Vec::new();

        for raw_path in paths {
            let p = Path::new(raw_path);

            if p.is_file() {
                let resolved = canonicalize_or_absolute(p);
                if !seen.contains(&resolved) && self.should_include_explicit(p) {
                    seen.insert(resolved.clone());
                    result.push(resolved);
                }
            } else if p.is_dir() {
                for found in self.walk_directory(p) {
                    if !seen.contains(&found) {
                        seen.insert(found.clone());
                        result.push(found);
                    }
                }
            } else if raw_path.contains(GLOB_CHARS) {
                for found in self.expand_glob(raw_path) {
                    let resolved = canonicalize_or_absolute(&found);
                    if !seen.contains(&resolved) {
                        seen.insert(resolved.clone());
                        result.push(resolved);
                    }
                }
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Path not found: {raw_path}"),
                ));
            }
        }

        result.sort();
        Ok(result)
    }

    /// Check if an explicitly-named file should be included.
    ///
    /// Files named explicitly on the command line override exclusions by default
    /// (matching Black/Ruff). Under `force_exclude` — the flag pre-commit hooks set —
    /// both configured/default patterns and a tool ignore file (`.flowmarkignore`)
    /// apply to explicit files too.
    fn should_include_explicit(&self, path: &Path) -> bool {
        if self.config.force_exclude {
            if spec_matches_path(&self.exclude_patterns, path) {
                return false;
            }
            if let Some(tool_ignore) = self.load_tool_ignore(path) {
                if spec_matches_path(&tool_ignore, path) {
                    return false;
                }
            }
        }
        !self.exceeds_max_size(path)
    }

    /// Walk up from the file's directory looking for `.{tool_name}ignore` and return its
    /// gitignore-style pattern lines (comments and blanks stripped). Mirrors Python
    /// `load_tool_ignore`.
    fn load_tool_ignore(&self, path: &Path) -> Option<Vec<String>> {
        let ignore_name = format!(".{}ignore", self.config.tool_name);
        let mut current = canonicalize_or_absolute(path.parent()?);
        loop {
            let candidate = current.join(&ignore_name);
            if candidate.is_file() {
                let text = std::fs::read_to_string(&candidate).ok()?;
                let patterns: Vec<String> = text
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .map(String::from)
                    .collect();
                return if patterns.is_empty() { None } else { Some(patterns) };
            }
            current = current.parent()?.to_path_buf();
        }
    }

    /// Walk a directory tree using `ignore::WalkBuilder` for efficient traversal.
    ///
    /// `WalkBuilder` handles gitignore natively, uses `DirEntry::file_type()` (no
    /// extra stat syscalls), and prunes excluded directories without descending.
    fn walk_directory(&self, root: &Path) -> Vec<PathBuf> {
        // Canonicalize the root once for consistent output paths.
        let canonical_root = root.canonicalize().unwrap_or_else(|_| {
            if root.is_absolute() {
                root.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(root)
            }
        });

        let mut builder = WalkBuilder::new(&canonical_root);

        // Don't skip hidden files — our exclude patterns handle specific hidden dirs.
        builder.hidden(false);
        // Don't read .ignore files — we only use .gitignore and .flowmarkignore.
        builder.ignore(false);
        // Gitignore support. Only read .gitignore files within the walked tree
        // (matches Python behavior). Don't read global git excludes or parent
        // directory gitignore files.
        builder.git_ignore(self.config.respect_gitignore);
        builder.git_global(false);
        builder.git_exclude(false);
        builder.parents(false);
        builder.require_git(false);
        // Tool-specific ignore file (e.g., .flowmarkignore).
        builder.add_custom_ignore_filename(format!(".{}ignore", self.config.tool_name));
        // Max file size (0 = no limit, handled by not setting it).
        if self.config.files_max_size > 0 {
            builder.max_filesize(Some(self.config.files_max_size));
        }

        // Clone pre-compiled exclude patterns into the filter closure.
        let dir_excludes = self.compiled_dir_excludes.clone();
        builder.filter_entry(move |entry| {
            // Always allow the root directory (depth 0) through.
            if entry.depth() == 0 {
                return true;
            }
            let Some(ft) = entry.file_type() else { return true };
            if !ft.is_dir() {
                return true;
            }
            let name = entry.file_name().to_string_lossy();
            !dir_excludes.iter().any(|p| p.matches(&name))
        });

        let compiled_includes = &self.compiled_includes;
        let mut results = Vec::new();
        for entry in builder.build().flatten() {
            let Some(ft) = entry.file_type() else { continue };
            if !ft.is_file() {
                continue;
            }
            let path = entry.into_path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if compiled_includes.iter().any(|p| p.matches(name)) {
                    results.push(path);
                }
            }
        }
        results
    }

    /// Expand a glob pattern, then apply all filters (include, exclude,
    /// gitignore, tool-ignore, and max-size).
    fn expand_glob(&self, pattern: &str) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let Ok(entries) = glob::glob(pattern) else {
            return results;
        };

        for entry in entries.flatten() {
            if entry.is_file() {
                if !self.glob_entry_passes_filters(&entry) {
                    continue;
                }
                results.push(entry);
            }
        }
        results
    }

    /// Check whether a glob-expanded file entry passes all configured filters.
    fn glob_entry_passes_filters(&self, entry: &Path) -> bool {
        let Some(name) = entry.file_name().and_then(|n| n.to_str()) else {
            return false;
        };
        if !self.compiled_includes.iter().any(|p| p.matches(name)) {
            return false;
        }
        if self.exceeds_max_size(entry) {
            return false;
        }
        // Check directory components against exclude patterns
        for component in entry.components() {
            if Some(component.as_os_str()) == entry.file_name() {
                continue;
            }
            let part = component.as_os_str().to_string_lossy();
            if self.compiled_dir_excludes.iter().any(|p| p.matches(&part)) {
                return false;
            }
        }
        true
    }

    /// Check if a file exceeds the configured max size. 0 = no limit.
    fn exceeds_max_size(&self, path: &Path) -> bool {
        if self.config.files_max_size == 0 {
            return false;
        }
        match path.metadata() {
            Ok(meta) => meta.len() > self.config.files_max_size,
            Err(_) => false,
        }
    }
}

/// Match an explicitly-named file against a gitignore-style pattern list, checking the
/// basename, the path relative to cwd (so multi-component patterns like `docs/api/`
/// work), and each ancestor directory component. Mirrors Python `_spec_matches_path`.
fn spec_matches_path(patterns: &[String], path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if matches_any_pattern(patterns, name) {
            return true;
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let abs = canonicalize_or_absolute(path);
        let cwd_abs = canonicalize_or_absolute(&cwd);
        if let Ok(rel) = abs.strip_prefix(&cwd_abs) {
            let rel_posix = rel.to_string_lossy().replace('\\', "/");
            if matches_relative_path(patterns, &rel_posix) {
                return true;
            }
        }
    }
    if let Some(parent) = path.parent() {
        for component in parent.components() {
            let part = component.as_os_str().to_string_lossy();
            if matches_any_pattern(patterns, &format!("{part}/")) {
                return true;
            }
        }
    }
    false
}

/// Match a cwd-relative POSIX path against gitignore-style patterns. Directory patterns
/// (e.g. `docs/api/`) match the directory and everything beneath it; other patterns are
/// glob-matched against the whole relative path. A leading `/` anchors a pattern to the
/// ignore-file/cwd root — for our already-cwd-relative path that is equivalent to the
/// slash-stripped form, matching gitignore/pathspec semantics (`/docs/api/`).
fn matches_relative_path(patterns: &[String], rel_posix: &str) -> bool {
    for pattern in patterns {
        let pattern = pattern.strip_prefix('/').unwrap_or(pattern);
        if let Some(dir) = pattern.strip_suffix('/') {
            if rel_posix == dir || rel_posix.starts_with(&format!("{dir}/")) {
                return true;
            }
        } else if let Ok(gp) = glob::Pattern::new(pattern) {
            if gp.matches(rel_posix) {
                return true;
            }
        }
    }
    false
}

/// Check if a name matches any of the given gitignore-style patterns.
///
/// - Patterns ending with `/` match directory names (trailing slash stripped for glob matching)
/// - `*` wildcards match anything except `/`
/// - Simple glob matching via `glob::Pattern`
fn matches_any_pattern(patterns: &[String], name: &str) -> bool {
    let is_dir_query = name.ends_with('/');
    let bare_name = if is_dir_query { &name[..name.len() - 1] } else { name };

    for pattern in patterns {
        let pattern_is_dir = pattern.ends_with('/');
        let bare_pattern =
            if pattern_is_dir { &pattern[..pattern.len() - 1] } else { pattern.as_str() };

        // Directory patterns only match directory queries
        if pattern_is_dir && !is_dir_query {
            continue;
        }

        if let Ok(glob_pattern) = glob::Pattern::new(bare_pattern) {
            if glob_pattern.matches(bare_name) {
                return true;
            }
        }
    }
    false
}

/// Resolve a path to its canonical form, falling back to the absolute path.
fn canonicalize_or_absolute(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(path)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::matches_relative_path;

    fn pats(p: &[&str]) -> Vec<String> {
        p.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn directory_pattern_matches_nested_path() {
        assert!(matches_relative_path(&pats(&["docs/api/"]), "docs/api/notes.md"));
        assert!(matches_relative_path(&pats(&["docs/api/"]), "docs/api"));
    }

    #[test]
    fn root_anchored_directory_pattern_matches_nested_path() {
        // Regression: a leading `/` anchors to the ignore-file/cwd root; for an already
        // cwd-relative path this is equivalent to the slash-stripped form.
        assert!(matches_relative_path(&pats(&["/docs/api/"]), "docs/api/notes.md"));
        assert!(matches_relative_path(&pats(&["/docs/api/"]), "docs/api"));
        assert!(matches_relative_path(&pats(&["/t.md"]), "t.md"));
    }

    #[test]
    fn non_matching_pattern_does_not_match() {
        assert!(!matches_relative_path(&pats(&["/docs/api/"]), "docs/other/notes.md"));
        assert!(!matches_relative_path(&pats(&["docs/api/"]), "docs/apidocs/notes.md"));
    }
}
