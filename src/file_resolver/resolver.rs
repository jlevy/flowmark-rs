//! `FileResolver` — main entry point for file discovery.
//!
//! Resolves a mix of files, directories, and glob patterns into a deduplicated,
//! sorted list of concrete file paths, applying all configured filters.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use ignore::gitignore::Gitignore;

use super::config::FileResolverConfig;
use super::gitignore::{load_gitignore, load_tool_ignore_patterns};

/// Characters that indicate a path is a glob pattern rather than a literal path.
const GLOB_CHARS: &[char] = &['*', '?', '['];

/// Discovers files matching configured include patterns while respecting
/// gitignore, tool-specific ignore files, and default/custom exclusions.
pub struct FileResolver {
    config: FileResolverConfig,
    exclude_patterns: Vec<String>,
    include_patterns: Vec<String>,
    tool_ignore_cache: HashMap<PathBuf, Option<Vec<String>>>,
    gitignore_cache: HashMap<PathBuf, Option<Gitignore>>,
}

impl FileResolver {
    /// Create a new `FileResolver` with the given configuration.
    pub fn new(config: FileResolverConfig) -> Self {
        let exclude_patterns = config.effective_exclude();
        let include_patterns = config.effective_include();
        Self {
            config,
            exclude_patterns,
            include_patterns,
            tool_ignore_cache: HashMap::new(),
            gitignore_cache: HashMap::new(),
        }
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
                    let resolved = canonicalize_or_absolute(&found);
                    if !seen.contains(&resolved) {
                        seen.insert(resolved.clone());
                        result.push(resolved);
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
    fn should_include_explicit(&self, path: &Path) -> bool {
        if self.config.force_exclude {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if matches_any_pattern(&self.exclude_patterns, name) {
                    return false;
                }
            }
            for component in path.components() {
                if Some(component.as_os_str()) == path.file_name() {
                    continue;
                }
                let part = component.as_os_str().to_string_lossy();
                let dir_with_slash = format!("{part}/");
                if matches_any_pattern(&self.exclude_patterns, &dir_with_slash) {
                    return false;
                }
            }
        }
        !self.exceeds_max_size(path)
    }

    /// Walk a directory tree, pruning excluded directories in-place.
    fn walk_directory(&mut self, root: &Path) -> Vec<PathBuf> {
        let tool_ignore_patterns = self.get_tool_ignore_patterns(root);
        let mut results = Vec::new();
        self.walk_recursive(root, root, tool_ignore_patterns.as_deref(), &mut results);
        results
    }

    fn walk_recursive(
        &mut self,
        current: &Path,
        root: &Path,
        tool_ignore_patterns: Option<&[String]>,
        results: &mut Vec<PathBuf>,
    ) {
        let Ok(entries) = std::fs::read_dir(current) else {
            return;
        };

        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut files: Vec<PathBuf> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }

        dirs.sort();
        files.sort();

        // Collect gitignore specs for this directory
        let gitignore_specs: Vec<Gitignore> = if self.config.respect_gitignore {
            self.get_gitignore_chain(current, root)
        } else {
            Vec::new()
        };

        // Process files
        for filepath in &files {
            let Some(filename) = filepath.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            if !matches_any_pattern(&self.include_patterns, filename) {
                continue;
            }
            if self.exceeds_max_size(filepath) {
                continue;
            }
            if gitignore_specs.iter().any(|spec| spec.matched(filename, false).is_ignore()) {
                continue;
            }
            if let Some(ti_patterns) = tool_ignore_patterns {
                if matches_any_pattern(ti_patterns, filename) {
                    continue;
                }
            }
            results.push(filepath.clone());
        }

        // Recurse into non-excluded directories
        let rel_to_root = current.strip_prefix(root).unwrap_or(Path::new(""));
        for dir_path in &dirs {
            let Some(dirname) = dir_path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let rel_path = rel_to_root.join(dirname);
            if !self.is_dir_excluded(dirname, &rel_path, current, tool_ignore_patterns, Some(root))
            {
                self.walk_recursive(dir_path, root, tool_ignore_patterns, results);
            }
        }
    }

    /// Check if a directory should be pruned during traversal.
    fn is_dir_excluded(
        &mut self,
        dirname: &str,
        rel_path: &Path,
        current_dir: &Path,
        tool_ignore_patterns: Option<&[String]>,
        walk_root: Option<&Path>,
    ) -> bool {
        let dir_with_slash = format!("{dirname}/");
        let rel_with_slash = format!("{}/", rel_path.display());

        if matches_any_pattern(&self.exclude_patterns, &dir_with_slash) {
            return true;
        }
        if matches_any_pattern(&self.exclude_patterns, &rel_with_slash) {
            return true;
        }

        if self.config.respect_gitignore {
            let root = walk_root.unwrap_or(current_dir);
            let chain = self.get_gitignore_chain(current_dir, root);
            for spec in &chain {
                if spec.matched(&dir_with_slash, true).is_ignore() {
                    return true;
                }
            }
        }

        if let Some(ti_patterns) = tool_ignore_patterns {
            if matches_any_pattern(ti_patterns, &dir_with_slash) {
                return true;
            }
            if matches_any_pattern(ti_patterns, &rel_with_slash) {
                return true;
            }
        }

        false
    }

    /// Expand a glob pattern, then apply all filters.
    fn expand_glob(&self, pattern: &str) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let Ok(entries) = glob::glob(pattern) else {
            return results;
        };

        for entry in entries.flatten() {
            if entry.is_file() {
                if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
                    if matches_any_pattern(&self.include_patterns, name)
                        && !self.exceeds_max_size(&entry)
                    {
                        results.push(entry);
                    }
                }
            }
        }
        results
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

    /// Load and cache gitignore for a directory.
    fn get_gitignore(&mut self, directory: &Path) -> Option<Gitignore> {
        let key = directory.to_path_buf();
        if let Some(cached) = self.gitignore_cache.get(&key) {
            return cached.clone();
        }
        let result = load_gitignore(directory);
        self.gitignore_cache.insert(key, result.clone());
        result
    }

    /// Collect all gitignore specs from `walk_root` down to `directory` (inclusive).
    fn get_gitignore_chain(&mut self, directory: &Path, walk_root: &Path) -> Vec<Gitignore> {
        let mut specs: Vec<Gitignore> = Vec::new();

        let resolved_root = walk_root.canonicalize().unwrap_or_else(|_| walk_root.to_path_buf());
        let resolved_dir = directory.canonicalize().unwrap_or_else(|_| directory.to_path_buf());

        let mut current = resolved_root;
        loop {
            if let Some(spec) = self.get_gitignore(&current) {
                specs.push(spec);
            }
            if current == resolved_dir {
                break;
            }
            let Ok(remaining) = resolved_dir.strip_prefix(&current) else {
                break;
            };
            match remaining.components().next() {
                Some(c) => current = current.join(c),
                None => break,
            }
        }
        specs
    }

    /// Lazily load tool-specific ignore patterns, cached per resolved start directory.
    fn get_tool_ignore_patterns(&mut self, start_dir: &Path) -> Option<Vec<String>> {
        let resolved = start_dir.canonicalize().unwrap_or_else(|_| start_dir.to_path_buf());
        if let Some(cached) = self.tool_ignore_cache.get(&resolved) {
            return cached.clone();
        }
        let result = load_tool_ignore_patterns(&self.config.tool_name, start_dir);
        self.tool_ignore_cache.insert(resolved, result.clone());
        result
    }
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
