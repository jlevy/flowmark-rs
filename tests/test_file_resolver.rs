//! Tests for the file resolver module.
//!
//! Ported from Python: `test_file_resolver.py` (31 tests)

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use flowmark::file_resolver::{FileResolver, FileResolverConfig};

/// Helper: create a temp directory with a standard file structure for testing.
fn create_test_tree(dir: &std::path::Path) {
    // Create directory structure
    fs::create_dir_all(dir.join("docs")).expect("create docs dir");
    fs::create_dir_all(dir.join("node_modules/pkg")).expect("create node_modules dir");
    fs::create_dir_all(dir.join(".venv/lib")).expect("create .venv dir");
    fs::create_dir_all(dir.join("drafts")).expect("create drafts dir");
    fs::create_dir_all(dir.join("src")).expect("create src dir");

    // Create files
    fs::write(dir.join("README.md"), "# README\n").expect("write README.md");
    fs::write(dir.join("docs/api.md"), "# API\n").expect("write api.md");
    fs::write(dir.join("docs/guide.md"), "# Guide\n").expect("write guide.md");
    fs::write(dir.join("node_modules/pkg/README.md"), "# Pkg\n")
        .expect("write node_modules README.md");
    fs::write(dir.join(".venv/lib/README.md"), "# Venv\n").expect("write .venv README.md");
    fs::write(dir.join("drafts/wip.md"), "# WIP\n").expect("write wip.md");
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").expect("write main.rs");
}

fn file_names(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from)).collect()
}

// --- Config tests (4) ---

#[test]
fn test_config_effective_include() {
    let config = FileResolverConfig::default();
    assert_eq!(config.effective_include(), vec!["*.md".to_string()]);
}

#[test]
fn test_config_effective_include_custom_base() {
    let config = FileResolverConfig {
        include: vec!["*.txt".to_string()],
        extend_include: vec!["*.mdx".to_string()],
        ..FileResolverConfig::default()
    };
    assert_eq!(config.effective_include(), vec!["*.txt".to_string(), "*.mdx".to_string()]);
}

#[test]
fn test_config_effective_exclude_replaced() {
    let config = FileResolverConfig {
        exclude: Some(vec!["custom_dir/".to_string()]),
        ..FileResolverConfig::default()
    };
    let effective = config.effective_exclude();
    assert_eq!(effective, vec!["custom_dir/".to_string()]);
}

#[test]
fn test_config_effective_exclude_extended() {
    let config = FileResolverConfig {
        extend_exclude: vec!["extra/".to_string()],
        ..FileResolverConfig::default()
    };
    let effective = config.effective_exclude();
    // Should have all defaults plus "extra/"
    assert!(effective.contains(&".git/".to_string()));
    assert!(effective.contains(&"node_modules/".to_string()));
    assert!(effective.last() == Some(&"extra/".to_string()));
}

// --- Resolver core (7) ---

#[test]
fn test_resolver_single_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("test.md"), "# Test\n").expect("write test.md");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let test_file = dir.path().join("test.md").to_string_lossy().to_string();
    let result = resolver.resolve(&[&test_file]).expect("resolve");

    assert_eq!(result.len(), 1);
    assert!(result[0].ends_with("test.md"));
}

#[test]
fn test_resolver_directory_recursion() {
    let dir = tempfile::tempdir().expect("create temp dir");
    create_test_tree(dir.path());

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    assert!(names.contains(&"api.md".to_string()));
    assert!(names.contains(&"guide.md".to_string()));
    // Non-.md files should not be included
    assert!(!names.contains(&"main.rs".to_string()));
}

#[test]
fn test_resolver_excludes_default_dirs() {
    let dir = tempfile::tempdir().expect("create temp dir");
    create_test_tree(dir.path());

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    // node_modules and .venv should be excluded by default
    for path in &result {
        let path_str = path.to_string_lossy();
        assert!(!path_str.contains("node_modules"), "node_modules should be excluded");
        assert!(!path_str.contains(".venv"), ".venv should be excluded");
    }
}

#[test]
fn test_resolver_respects_gitignore() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("build")).expect("create build dir");
    fs::write(dir.path().join("README.md"), "# README\n").expect("write README.md");
    fs::write(dir.path().join("build/output.md"), "# Output\n").expect("write output.md");
    fs::write(dir.path().join(".gitignore"), "build/\n").expect("write .gitignore");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    assert!(!names.contains(&"output.md".to_string()));
}

#[test]
fn test_resolver_no_respect_gitignore() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("ignored")).expect("create ignored dir");
    fs::write(dir.path().join("README.md"), "# README\n").expect("write README.md");
    fs::write(dir.path().join("ignored/found.md"), "# Found\n").expect("write found.md");
    fs::write(dir.path().join(".gitignore"), "ignored/\n").expect("write .gitignore");

    let config = FileResolverConfig { respect_gitignore: false, ..FileResolverConfig::default() };
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    assert!(names.contains(&"found.md".to_string()));
}

#[test]
fn test_resolver_force_exclude_filters_explicit_files() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("node_modules")).expect("create node_modules");
    fs::write(dir.path().join("node_modules/README.md"), "# Pkg\n").expect("write file");

    let config = FileResolverConfig { force_exclude: true, ..FileResolverConfig::default() };
    let mut resolver = FileResolver::new(config);
    let file_str = dir.path().join("node_modules/README.md").to_string_lossy().to_string();
    let result = resolver.resolve(&[&file_str]).expect("resolve");

    assert!(result.is_empty(), "force_exclude should filter node_modules/README.md");
}

#[test]
fn test_resolver_explicit_files_bypass_exclusions_by_default() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("node_modules")).expect("create node_modules");
    fs::write(dir.path().join("node_modules/README.md"), "# Pkg\n").expect("write file");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let file_str = dir.path().join("node_modules/README.md").to_string_lossy().to_string();
    let result = resolver.resolve(&[&file_str]).expect("resolve");

    assert_eq!(result.len(), 1, "explicit files should bypass exclusions by default");
}

// --- Filter options (5) ---

#[test]
fn test_resolver_extend_include() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("readme.md"), "# MD\n").expect("write md");
    fs::write(dir.path().join("page.mdx"), "# MDX\n").expect("write mdx");

    let config = FileResolverConfig {
        extend_include: vec!["*.mdx".to_string()],
        ..FileResolverConfig::default()
    };
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"readme.md".to_string()));
    assert!(names.contains(&"page.mdx".to_string()));
}

#[test]
fn test_resolver_exclude_replaces_defaults() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("custom_dir")).expect("create custom_dir");
    fs::create_dir_all(dir.path().join("node_modules")).expect("create node_modules");
    fs::write(dir.path().join("README.md"), "# Root\n").expect("write root");
    fs::write(dir.path().join("custom_dir/doc.md"), "# Custom\n").expect("write custom");
    fs::write(dir.path().join("node_modules/README.md"), "# NM\n").expect("write nm");

    let config = FileResolverConfig {
        exclude: Some(vec!["custom_dir/".to_string()]),
        ..FileResolverConfig::default()
    };
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    // custom_dir should be excluded (replaced defaults)
    assert!(!result.iter().any(|p| p.to_string_lossy().contains("custom_dir")));
    // node_modules is NOT excluded (defaults replaced)
    assert!(result.iter().any(|p| p.to_string_lossy().contains("node_modules")));
}

#[test]
fn test_resolver_extend_exclude() {
    let dir = tempfile::tempdir().expect("create temp dir");
    create_test_tree(dir.path());

    let config = FileResolverConfig {
        extend_exclude: vec!["drafts/".to_string()],
        ..FileResolverConfig::default()
    };
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    // drafts should be excluded
    assert!(!result.iter().any(|p| p.to_string_lossy().contains("drafts")));
    // node_modules should still be excluded (defaults preserved)
    assert!(!result.iter().any(|p| p.to_string_lossy().contains("node_modules")));
}

#[test]
fn test_resolver_files_max_size() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("small.md"), "# Small\n").expect("write small");
    // Create a file larger than 1MB
    let large_content = "x".repeat(2 * 1024 * 1024);
    fs::write(dir.path().join("large.md"), &large_content).expect("write large");

    let config = FileResolverConfig::default(); // 1MB limit
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"small.md".to_string()));
    assert!(!names.contains(&"large.md".to_string()));
}

#[test]
fn test_resolver_files_max_size_zero_disables() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let large_content = "x".repeat(2 * 1024 * 1024);
    fs::write(dir.path().join("large.md"), &large_content).expect("write large");

    let config = FileResolverConfig { files_max_size: 0, ..FileResolverConfig::default() };
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"large.md".to_string()));
}

// --- Glob and deduplication (4) ---

#[test]
fn test_resolver_glob_pattern() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("docs")).expect("create docs");
    fs::write(dir.path().join("docs/api.md"), "# API\n").expect("write api.md");
    fs::write(dir.path().join("docs/guide.md"), "# Guide\n").expect("write guide.md");
    fs::write(dir.path().join("docs/notes.txt"), "notes").expect("write notes.txt");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let pattern = format!("{}/*.md", dir.path().join("docs").display());
    let result = resolver.resolve(&[&pattern]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"api.md".to_string()));
    assert!(names.contains(&"guide.md".to_string()));
    assert!(!names.contains(&"notes.txt".to_string()));
}

#[test]
fn test_resolver_mixed_inputs() {
    let dir = tempfile::tempdir().expect("create temp dir");
    create_test_tree(dir.path());

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let file_str = dir.path().join("README.md").to_string_lossy().to_string();
    let docs_str = dir.path().join("docs").to_string_lossy().to_string();
    let result = resolver.resolve(&[&file_str, &docs_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    assert!(names.contains(&"api.md".to_string()));
    assert!(names.contains(&"guide.md".to_string()));
}

#[test]
fn test_resolver_deduplication() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("test.md"), "# Test\n").expect("write test.md");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let file_str = dir.path().join("test.md").to_string_lossy().to_string();
    let result = resolver.resolve(&[&file_str, &file_str]).expect("resolve");

    assert_eq!(result.len(), 1, "same file listed twice should produce one result");
}

#[test]
fn test_resolver_sorted_output() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("c.md"), "# C\n").expect("write c.md");
    fs::write(dir.path().join("a.md"), "# A\n").expect("write a.md");
    fs::write(dir.path().join("b.md"), "# B\n").expect("write b.md");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert_eq!(names, {
        let mut sorted = names.clone();
        sorted.sort();
        sorted
    });
}

/// H2 regression test: glob expansion must apply exclusion filters.
/// Previously, `expand_glob` only checked include patterns and `max_size`,
/// allowing `node_modules/` etc. through when using `**/*.md`.
#[test]
fn test_resolver_glob_excludes_default_dirs() {
    let dir = tempfile::tempdir().expect("create temp dir");
    create_test_tree(dir.path());

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let pattern = format!("{}/**/*.md", dir.path().display());
    let result = resolver.resolve(&[&pattern]).expect("resolve");

    // node_modules and .venv should be excluded even via glob expansion
    for path in &result {
        let path_str = path.to_string_lossy();
        assert!(!path_str.contains("node_modules"), "glob should exclude node_modules: {path_str}");
        assert!(!path_str.contains(".venv"), "glob should exclude .venv: {path_str}");
    }
    // But normal files should still be found
    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    assert!(names.contains(&"api.md".to_string()));
}

// --- Error handling (1) ---

#[test]
fn test_resolver_file_not_found() {
    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let result = resolver.resolve(&["/nonexistent/path/file.md"]);

    assert!(result.is_err());
    let err = result.expect_err("should error");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    assert!(err.to_string().contains("Path not found"));
}

// --- Tool ignore (2) ---

#[test]
fn test_resolver_flowmarkignore() {
    let dir = tempfile::tempdir().expect("create temp dir");
    create_test_tree(dir.path());
    fs::write(dir.path().join(".flowmarkignore"), "drafts/\n").expect("write .flowmarkignore");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    assert!(!result.iter().any(|p| p.to_string_lossy().contains("drafts")));
}

#[test]
fn test_resolver_tool_ignore_per_walk_root() {
    let root = tempfile::tempdir().expect("create temp dir");

    // Create two separate walk roots
    let root_a = root.path().join("a");
    let root_b = root.path().join("b");
    fs::create_dir_all(&root_a).expect("create a");
    fs::create_dir_all(&root_b).expect("create b");
    fs::create_dir_all(root_a.join("skip")).expect("create a/skip");
    fs::create_dir_all(root_b.join("skip")).expect("create b/skip");

    fs::write(root_a.join("readme.md"), "# A\n").expect("write a/readme.md");
    fs::write(root_a.join("skip/hidden.md"), "# Hidden A\n").expect("write a/skip/hidden.md");
    fs::write(root_b.join("readme.md"), "# B\n").expect("write b/readme.md");
    fs::write(root_b.join("skip/visible.md"), "# Visible B\n").expect("write b/skip/visible.md");

    // Only root A has a .flowmarkignore
    fs::write(root_a.join(".flowmarkignore"), "skip/\n").expect("write .flowmarkignore");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);

    let a_str = root_a.to_string_lossy().to_string();
    let result_a = resolver.resolve(&[&a_str]).expect("resolve a");
    assert!(!result_a.iter().any(|p| p.to_string_lossy().contains("hidden")));

    let mut resolver_b = FileResolver::new(FileResolverConfig::default());
    let b_str = root_b.to_string_lossy().to_string();
    let result_b = resolver_b.resolve(&[&b_str]).expect("resolve b");
    assert!(result_b.iter().any(|p| p.to_string_lossy().contains("visible")));
}

// --- Gitignore specifics (5) ---

#[test]
fn test_resolver_nested_gitignore() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("sub")).expect("create sub");
    fs::write(dir.path().join("README.md"), "# Root\n").expect("write root");
    fs::write(dir.path().join("sub/doc.md"), "# Doc\n").expect("write doc");
    fs::write(dir.path().join("sub/draft.md"), "# Draft\n").expect("write draft");
    fs::write(dir.path().join("sub/.gitignore"), "draft.md\n").expect("write nested .gitignore");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    assert!(names.contains(&"doc.md".to_string()));
    assert!(!names.contains(&"draft.md".to_string()));
}

#[test]
fn test_resolver_nested_gitignore_combines_parent_rules() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("sub")).expect("create sub");
    fs::write(dir.path().join("README.md"), "# Root\n").expect("write root");
    fs::write(dir.path().join("sub/doc.md"), "# Doc\n").expect("write doc");
    fs::write(dir.path().join("sub/output.log"), "log").expect("write log");
    // Parent .gitignore ignores *.log
    fs::write(dir.path().join(".gitignore"), "*.log\n").expect("write root .gitignore");

    let config = FileResolverConfig {
        extend_include: vec!["*.log".to_string()],
        ..FileResolverConfig::default()
    };
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    // *.log should still be excluded by parent gitignore even in subdirectory
    let names = file_names(&result);
    assert!(!names.contains(&"output.log".to_string()));
}

/// Regression test: nested .gitignore with a directory pattern must exclude
/// that subdirectory. Previously the manual walker failed to apply nested
/// gitignore directory patterns (e.g., `generated/` in `sub/.gitignore`).
#[test]
fn test_resolver_nested_gitignore_dir_pattern() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("sub/generated")).expect("create sub/generated");
    fs::create_dir_all(dir.path().join("sub/kept")).expect("create sub/kept");
    fs::write(dir.path().join("README.md"), "# Root\n").expect("write root");
    fs::write(dir.path().join("sub/kept/visible.md"), "# Visible\n").expect("write visible");
    fs::write(dir.path().join("sub/generated/output.md"), "# Output\n").expect("write output");
    // Nested .gitignore excludes the generated/ directory
    fs::write(dir.path().join("sub/.gitignore"), "generated/\n")
        .expect("write nested .gitignore");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    assert!(names.contains(&"visible.md".to_string()), "sub/kept/visible.md should be included");
    assert!(
        !names.contains(&"output.md".to_string()),
        "sub/generated/output.md should be excluded by nested gitignore dir pattern"
    );
}

#[test]
fn test_resolver_gitignore_file_patterns() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("README.md"), "# Root\n").expect("write root");
    fs::write(dir.path().join("draft.md"), "# Draft\n").expect("write draft");
    fs::write(dir.path().join(".gitignore"), "draft.md\n").expect("write .gitignore");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    assert!(!names.contains(&"draft.md".to_string()));
}

#[test]
fn test_resolver_gitignore_wildcard_file_pattern() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::write(dir.path().join("README.md"), "# Root\n").expect("write root");
    fs::write(dir.path().join("temp.md"), "# Temp\n").expect("write temp.md");
    fs::write(dir.path().join("temp.txt"), "temp").expect("write temp.txt");
    fs::write(dir.path().join(".gitignore"), "temp.*\n").expect("write .gitignore");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"README.md".to_string()));
    assert!(!names.contains(&"temp.md".to_string()));
}

/// H3 regression test: gitignore path-based patterns must use relative path, not filename.
/// Previously, `matched(filename, false)` passed bare filename so patterns like
/// `sub/ignore-me.md` would never match.
#[test]
fn test_resolver_gitignore_path_based_pattern() {
    let dir = tempfile::tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("sub")).expect("create sub");
    fs::write(dir.path().join("sub/keep.md"), "# Keep\n").expect("write keep.md");
    fs::write(dir.path().join("sub/ignore-me.md"), "# Ignore\n").expect("write ignore-me.md");
    // Path-based pattern: only sub/ignore-me.md, not all ignore-me.md
    fs::write(dir.path().join(".gitignore"), "sub/ignore-me.md\n").expect("write .gitignore");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    assert!(names.contains(&"keep.md".to_string()), "keep.md should be included");
    assert!(
        !names.contains(&"ignore-me.md".to_string()),
        "sub/ignore-me.md should be excluded by path-based gitignore pattern"
    );
}

// --- Ignore file internals (3) ---

#[test]
fn test_read_ignore_file_missing() {
    use flowmark::file_resolver::gitignore::read_ignore_file;
    let result = read_ignore_file(std::path::Path::new("/nonexistent/.gitignore"));
    assert!(result.is_none());
}

#[test]
fn test_read_ignore_file_unreadable() {
    use flowmark::file_resolver::gitignore::read_ignore_file;

    // Create a file and make it unreadable
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join(".gitignore");
    fs::write(&path, "build/\n").expect("write file");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o000);
        std::fs::set_permissions(&path, perms).expect("set permissions");
    }

    let result = read_ignore_file(&path);

    // When running as root, permission-based read failures don't apply.
    // Check if we can actually read the file despite 0o000 permissions (i.e., root).
    #[cfg(unix)]
    {
        let still_readable = std::fs::read_to_string(&path).is_ok();
        if !still_readable {
            assert!(result.is_none(), "unreadable file should return None");
        }
        // If still readable (running as root), the test is a no-op
    }

    #[cfg(not(unix))]
    let _ = result;
}

#[test]
fn test_read_ignore_file_non_utf8() {
    use flowmark::file_resolver::gitignore::read_ignore_file;

    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join(".gitignore");

    // Write non-UTF-8 bytes
    let mut file = fs::File::create(&path).expect("create file");
    file.write_all(&[0xFF, 0xFE, 0x00, 0x01]).expect("write bytes");
    drop(file);

    let result = read_ignore_file(&path);
    // Non-UTF-8 should return None (std::fs::read_to_string fails on invalid UTF-8)
    assert!(result.is_none(), "non-UTF-8 file should return None");
}

// --- Flowmarkignore positive (1) ---

#[test]
fn test_resolver_flowmarkignore_positive_assertion() {
    let dir = tempfile::tempdir().expect("create temp dir");
    create_test_tree(dir.path());
    fs::write(dir.path().join(".flowmarkignore"), "drafts/\n").expect("write .flowmarkignore");

    let config = FileResolverConfig::default();
    let mut resolver = FileResolver::new(config);
    let dir_str = dir.path().to_string_lossy().to_string();
    let result = resolver.resolve(&[&dir_str]).expect("resolve");

    let names = file_names(&result);
    // Verify exactly which files are kept
    assert!(names.contains(&"README.md".to_string()));
    assert!(names.contains(&"api.md".to_string()));
    assert!(names.contains(&"guide.md".to_string()));
    // drafts should be excluded
    assert!(!names.contains(&"wip.md".to_string()));
    // node_modules and .venv should be excluded by defaults
    assert!(!result.iter().any(|p| p.to_string_lossy().contains("node_modules")));
    assert!(!result.iter().any(|p| p.to_string_lossy().contains(".venv")));
}

/// M7 regression test: `should_include_explicit` with same-named directory and file.
/// Previously, the code compared component names to filename, skipping directory
/// components with the same name as the file (e.g., `excluded/excluded`).
#[test]
fn test_resolver_force_exclude_same_name_dir_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let excluded_dir = dir.path().join("excluded");
    fs::create_dir_all(&excluded_dir).expect("create excluded dir");
    // File named "excluded.md" inside directory "excluded" — different name, control case
    fs::write(excluded_dir.join("excluded.md"), "# Test\n").expect("write file");

    let config = FileResolverConfig {
        force_exclude: true,
        exclude: Some(vec!["excluded/".to_string()]),
        ..FileResolverConfig::default()
    };
    let mut resolver = FileResolver::new(config);
    let file_str = excluded_dir.join("excluded.md").to_string_lossy().to_string();
    let result = resolver.resolve(&[&file_str]).expect("resolve");

    assert!(result.is_empty(), "file inside 'excluded/' dir should be excluded with force_exclude");
}
