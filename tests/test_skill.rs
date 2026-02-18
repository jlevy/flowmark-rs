#![cfg(feature = "cli")]
//! Tests for the skill installation module.
//!
//! Ported from Python: `test_skill.py` (9 tests)

use std::fs;

use flowmark::skills::{get_docs_content, get_skill_content, install_skill};

// --- Skill content loading (3 tests) ---

#[test]
fn test_skill_content_loads() {
    let content = get_skill_content();
    assert!(!content.is_empty(), "SKILL.md should be loadable and non-empty");
}

#[test]
fn test_skill_content_has_metadata() {
    let content = get_skill_content();
    assert!(content.contains("name: flowmark"), "should contain name: flowmark");
    assert!(content.contains("description:"), "should contain description:");
    assert!(content.contains("allowed-tools:"), "should contain allowed-tools:");
}

#[test]
fn test_skill_content_has_usage() {
    let content = get_skill_content();
    assert!(content.contains("# Flowmark"), "should contain # Flowmark heading");
    // Rust binary uses `flowmark` command directly (Python uses `uvx flowmark`)
    assert!(content.contains("flowmark --auto"), "should contain usage instructions");
}

// --- Docs content loading (2 tests) ---

#[test]
fn test_docs_content_loads() {
    let content = get_docs_content();
    assert!(!content.is_empty(), "docs content should be non-empty");
}

#[test]
fn test_docs_content_has_flowmark_reference() {
    let content = get_docs_content();
    // In test environment, README.md is not next to the test binary,
    // so we get fallback content. Both real README and fallback reference flowmark.
    let lower = content.to_lowercase();
    assert!(lower.contains("flowmark"), "docs content should reference flowmark");
}

// --- Skill installation (4 tests) ---

#[test]
fn test_install_skill_default() {
    // We can't easily mock home_dir() in Rust, so we test with custom base
    // which exercises the same code path minus home directory lookup.
    // The default path test is covered indirectly by the CLI --install-skill test.
    let dir = tempfile::tempdir().expect("create temp dir");
    let base = dir.path().join(".claude");

    install_skill(Some(base.to_str().expect("path to str"))).expect("install skill");

    let skill_file = base.join("skills").join("flowmark").join("SKILL.md");
    assert!(skill_file.exists(), "SKILL.md should be created");

    let content = fs::read_to_string(&skill_file).expect("read SKILL.md");
    assert!(content.contains("name: flowmark"), "should contain name: flowmark");
}

#[test]
fn test_install_skill_custom_base() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let custom_base = dir.path().join(".claude");

    install_skill(Some(custom_base.to_str().expect("path to str"))).expect("install skill");

    let skill_file = custom_base.join("skills").join("flowmark").join("SKILL.md");
    assert!(skill_file.exists(), "SKILL.md should be created at custom base");

    let content = fs::read_to_string(&skill_file).expect("read SKILL.md");
    assert!(content.contains("name: flowmark"), "should contain name: flowmark");
}

#[test]
fn test_install_skill_creates_directories() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let custom_base = dir.path().join("deep").join("nested").join("path");

    install_skill(Some(custom_base.to_str().expect("path to str"))).expect("install skill");

    let skill_file = custom_base.join("skills").join("flowmark").join("SKILL.md");
    assert!(skill_file.exists(), "SKILL.md should be created in deeply nested path");
}

#[test]
fn test_install_skill_overwrites_existing() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let custom_base = dir.path().join(".claude");
    let skill_dir = custom_base.join("skills").join("flowmark");
    fs::create_dir_all(&skill_dir).expect("create skill dir");

    // Write dummy content
    let skill_file = skill_dir.join("SKILL.md");
    fs::write(&skill_file, "old content").expect("write old content");

    install_skill(Some(custom_base.to_str().expect("path to str"))).expect("install skill");

    let content = fs::read_to_string(&skill_file).expect("read SKILL.md");
    assert!(!content.contains("old content"), "old content should be overwritten");
    assert!(content.contains("name: flowmark"), "should contain name: flowmark");
}
