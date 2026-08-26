//! Validation tests for the Rust CLI's upstream-owned skill runtime mirror.
//!
//! The main Flowmark repository publishes the public discovery bundle. flowmark-rs
//! embeds only the authored sources needed for `--skill` and `--install-skill` parity;
//! `scripts/generate_rust_readme.py` checks those sources byte-for-byte against the
//! pinned `repos/flowmark` submodule.

use std::path::{Path, PathBuf};

use flowmark::config::ListSpacing;
use flowmark::reformat_text;
use flowmark::skills::{
    FLOWMARK_RS_DISCOVERY_VERSION, compose_skill, discovery_project_setup_text,
    discovery_skill_text, get_project_setup_content, get_skill_content, is_pypi_release,
};

fn repo_path(relative: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn readme() -> String {
    std::fs::read_to_string(repo_path("README.md")).expect("read README")
}

fn release_pinned_bundle() -> String {
    format!("{}\n{}", discovery_skill_text(), discovery_project_setup_text())
}

#[test]
fn test_packaged_runtime_sources_match_embedded_content() {
    let skill = std::fs::read_to_string(repo_path("src/skills/SKILL.md"))
        .expect("read runtime skill source")
        .replace("\r\n", "\n");
    let reference = std::fs::read_to_string(repo_path("src/skills/references/project-setup.md"))
        .expect("read runtime skill reference")
        .replace("\r\n", "\n");
    assert_eq!(skill, get_skill_content().replace("\r\n", "\n"));
    assert_eq!(reference, get_project_setup_content().replace("\r\n", "\n"));
}

#[test]
fn test_rust_repo_does_not_publish_a_second_discovery_bundle() {
    assert!(!repo_path("skills/flowmark/SKILL.md").exists());
    assert!(!repo_path("skills/flowmark/references/project-setup.md").exists());
}

#[test]
fn test_release_pinned_bundle_is_flowmark_stable() {
    for (name, text) in
        [("SKILL.md", discovery_skill_text()), ("project-setup.md", discovery_project_setup_text())]
    {
        let formatted =
            reformat_text(&text, 88, false, true, true, false, false, ListSpacing::Preserve);
        assert_eq!(formatted, text, "`flowmark` over {name} must be a no-op");
    }
}

#[test]
fn test_runtime_skill_bundles_its_project_setup_reference() {
    assert!(get_skill_content().contains("references/project-setup.md"));
    assert!(repo_path("src/skills/references/project-setup.md").is_file());
}

#[test]
fn test_project_setup_hooks_cover_common_markdown_extensions() {
    let reference = get_project_setup_content();
    assert!(reference.contains(r#"glob: "*.{md,mdc,markdown}""#));
    assert!(reference.contains(r"files: '\.(md|mdc|markdown)$'"));
}

#[test]
fn test_readme_uses_main_repository_for_skill_distribution() {
    let text = readme();
    assert!(text.contains("npx skills add jlevy/flowmark@flowmark"));
    assert!(!text.contains("jlevy/flowmark-rs@flowmark"));
    assert!(!text.contains("](skills/flowmark/"));
    assert!(text.contains("github.com/jlevy/flowmark/blob/main/skills/flowmark/"));
}

#[test]
fn test_readme_uses_the_current_rust_runner_pin() {
    let text = readme();
    assert!(text.contains(&format!("flowmark-rs=={FLOWMARK_RS_DISCOVERY_VERSION}")));
    assert!(!text.contains("__FLOWMARK_RS_VERSION__"));
    assert!(!text.contains("flowmark-rs@latest"));
}

#[test]
fn test_rs_discovery_version_is_resolvable() {
    assert!(is_pypi_release(FLOWMARK_RS_DISCOVERY_VERSION));
}

#[test]
fn test_release_pinned_bundle_has_resolvable_version_pins() {
    let text = release_pinned_bundle();
    assert!(!text.contains("__FLOWMARK_VERSION__"));
    assert!(!text.contains("__FLOWMARK_RS_VERSION__"));
    assert!(!text.contains("flowmark==<version>"));
    for cap in regex_lite_pins(&text, "flowmark==") {
        assert!(
            !cap.contains(".dev") && !cap.contains('+'),
            "dev/local pin in release-pinned skill bundle: {cap}"
        );
    }
    let rs_pins = regex_lite_pins(&text, "flowmark-rs==");
    assert!(!rs_pins.is_empty(), "skill bundle missing a flowmark-rs== pin");
    for cap in rs_pins {
        assert!(!cap.contains(".dev") && !cap.contains('+'), "dev/local Rust pin: {cap}");
    }
}

#[test]
fn test_release_pinned_bundle_references_both_packages() {
    let text = release_pinned_bundle();
    assert!(text.contains("github.com/jlevy/flowmark-rs"));
    assert!(text.contains("github.com/jlevy/flowmark)"));
    assert!(text.contains("uvx --from flowmark-rs=="));
    assert!(text.contains("uvx --from flowmark=="));
}

#[test]
fn test_release_pinned_bundle_never_uses_at_latest() {
    let text = release_pinned_bundle();
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
    assert!(frontmatter.lines().any(|line| line == "name: flowmark"));
    let description = frontmatter
        .lines()
        .find_map(|line| line.strip_prefix("description: "))
        .expect("description field");
    assert!(description.len() <= 1024); // Agent Skills cap
    assert!(frontmatter.lines().any(|line| line.starts_with("allowed-tools: ")));
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
