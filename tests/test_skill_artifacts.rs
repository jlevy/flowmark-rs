//! Drift and validation tests for the generated skill bundle under `skills/flowmark/`
//! and the skill frontmatter.
//!
//! Ported from Python: `test_skill_artifacts.py`. The Python suite also checks dogfooded
//! repo surfaces (README + installed `.agents`/`.claude`/`AGENTS.md`); the Rust port keeps
//! a single committed discovery bundle, so the drift checks target that bundle and the
//! generator functions.

use std::path::PathBuf;

use flowmark::config::ListSpacing;
use flowmark::reformat_text;
use flowmark::skills::{
    FLOWMARK_RS_DISCOVERY_VERSION, compose_skill, discovery_project_setup_text,
    discovery_skill_text, is_pypi_release,
};

fn discovery_copy_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skills/flowmark/SKILL.md")
}

fn discovery_reference_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skills/flowmark/references/project-setup.md")
}

fn readme_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md")
}

/// Read the committed discovery copy, normalizing CRLF → LF so the line-ending-sensitive
/// comparisons below hold on Windows (the repo carries no `.gitattributes`, so git may
/// check the file out with CRLF; the generator output is always LF).
fn discovery_copy() -> String {
    std::fs::read_to_string(discovery_copy_path())
        .expect("read committed discovery copy")
        .replace("\r\n", "\n")
}

fn discovery_reference() -> String {
    std::fs::read_to_string(discovery_reference_path())
        .expect("read committed project-setup reference")
        .replace("\r\n", "\n")
}

#[test]
fn test_discovery_copy_matches_generator() {
    assert_eq!(
        discovery_copy(),
        discovery_skill_text().replace("\r\n", "\n"),
        "committed skills/flowmark/SKILL.md drifted from discovery_skill_text(); regenerate it"
    );
    assert_eq!(
        discovery_reference(),
        discovery_project_setup_text().replace("\r\n", "\n"),
        "committed project-setup reference drifted from its generator"
    );
}

#[test]
fn test_discovery_copy_is_flowmark_stable() {
    let text = discovery_copy();
    // Python's library `reformat_text` defaults: semantic + cleanups on.
    let formatted =
        reformat_text(&text, 88, false, true, true, false, false, ListSpacing::Preserve);
    assert_eq!(formatted, text, "`flowmark` over the discovery copy must be a no-op");
    let reference = discovery_reference();
    let formatted_reference =
        reformat_text(&reference, 88, false, true, true, false, false, ListSpacing::Preserve);
    assert_eq!(
        formatted_reference, reference,
        "`flowmark` over the discovery reference must be a no-op"
    );
}

#[test]
fn test_discovery_skill_bundles_its_project_setup_reference() {
    assert!(discovery_copy().contains("references/project-setup.md"));
    assert!(discovery_reference_path().is_file());
}

#[test]
fn test_project_setup_hooks_cover_common_markdown_extensions() {
    let reference = discovery_reference();
    assert!(reference.contains(r#"glob: "*.{md,mdc,markdown}""#));
    assert!(reference.contains(r"files: '\.(md|mdc|markdown)$'"));
}

#[test]
fn test_readme_uses_the_current_rust_runner_pin() {
    let readme = std::fs::read_to_string(readme_path()).expect("read README");
    assert!(readme.contains(&format!("flowmark-rs=={FLOWMARK_RS_DISCOVERY_VERSION}")));
    assert!(!readme.contains("__FLOWMARK_RS_VERSION__"));
    assert!(!readme.contains("flowmark-rs@latest"));
}

#[test]
fn test_rs_discovery_version_is_resolvable() {
    assert!(is_pypi_release(FLOWMARK_RS_DISCOVERY_VERSION));
}

#[test]
fn test_discovery_copy_has_resolvable_version_pin() {
    let text = format!("{}\n{}", discovery_copy(), discovery_reference());
    assert!(!text.contains("__FLOWMARK_VERSION__"));
    assert!(!text.contains("__FLOWMARK_RS_VERSION__"));
    assert!(!text.contains("flowmark==<version>"));
    for cap in regex_lite_pins(&text, "flowmark==") {
        assert!(
            !cap.contains(".dev") && !cap.contains('+'),
            "dev/local pin in discovery copy: {cap}"
        );
    }
    let rs_pins = regex_lite_pins(&text, "flowmark-rs==");
    assert!(!rs_pins.is_empty(), "discovery copy missing a flowmark-rs== pin");
    for cap in rs_pins {
        assert!(!cap.contains(".dev") && !cap.contains('+'), "dev/local Rust pin: {cap}");
    }
}

#[test]
fn test_discovery_copy_references_both_packages() {
    let text = discovery_copy();
    assert!(text.contains("github.com/jlevy/flowmark-rs"));
    assert!(text.contains("github.com/jlevy/flowmark)"));
    assert!(text.contains("uvx --from flowmark-rs=="));
    assert!(text.contains("uvx --from flowmark=="));
}

#[test]
fn test_discovery_copy_never_uses_at_latest() {
    let text = discovery_copy();
    assert!(!text.contains("uvx flowmark@latest"));
    assert!(!text.contains("uvx --from flowmark@latest"));
    assert!(!text.contains("uvx flowmark-rs@latest"));
    assert!(!text.contains("uvx --from flowmark-rs@latest"));
}

#[test]
fn test_skill_frontmatter_is_valid() {
    let content = compose_skill(Some("1.2.3"));
    assert!(content.starts_with("---\n"));
    let end = content.find("\n---\n").expect("frontmatter terminator");
    let frontmatter = &content[4..end];
    assert!(frontmatter.lines().any(|l| l == "name: flowmark"));
    let description = frontmatter
        .lines()
        .find_map(|l| l.strip_prefix("description: "))
        .expect("description field");
    assert!(description.len() <= 1024); // Agent Skills cap
    assert!(frontmatter.lines().any(|l| l.starts_with("allowed-tools: ")));
}

/// Extract the version token following each occurrence of `prefix` (e.g. `flowmark-rs==`),
/// stopping at whitespace or closing punctuation. A small stand-in for the Python regex.
fn regex_lite_pins(text: &str, prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut search = text;
    while let Some(idx) = search.find(prefix) {
        let after = &search[idx + prefix.len()..];
        // `flowmark==` must not match the `flowmark-rs==` prefix.
        if prefix == "flowmark==" && after.starts_with("rs==") {
            search = &search[idx + prefix.len()..];
            continue;
        }
        let token: String = after
            .chars()
            .take_while(|c| !c.is_whitespace() && !matches!(c, '`' | ')' | '"' | '\''))
            .collect();
        if token.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            out.push(token);
        }
        search = &search[idx + prefix.len()..];
    }
    out
}
