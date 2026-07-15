//! Tests for the cross-agent skill installation module.
//!
//! Ported from Python: `test_skill.py`. The version-pin roles are swapped for the Rust
//! port (it pins its own `flowmark-rs` version dynamically and keeps a sibling Python
//! `flowmark` discovery constant), so `compose_skill("1.2.3")` yields `flowmark-rs==1.2.3`
//! where the Python tests expect `flowmark==1.2.3`.
#![allow(clippy::unwrap_used)]

use std::collections::HashSet;
use std::fs;

use flowmark::config::ListSpacing;
use flowmark::reformat_text;
use flowmark::skills::{
    AGENTS_BEGIN_PREFIX, AGENTS_END_MARKER, FLOWMARK_PY_DISCOVERY_VERSION,
    FLOWMARK_RS_DISCOVERY_VERSION, SURFACE_CLAUDE, SURFACE_PORTABLE, agents_md_block,
    compose_project_setup, compose_skill, flowmark_rs_version, get_docs_content,
    get_project_setup_content, get_skill_content, install_skill, is_pypi_release, update_agents_md,
};

fn surfaces(names: &[&str]) -> HashSet<String> {
    names.iter().map(|s| (*s).to_string()).collect()
}

/// Mirrors Python's library `reformat_text` defaults (semantic + cleanups on).
fn reformat(text: &str) -> String {
    reformat_text(text, 88, false, true, true, false, false, ListSpacing::Preserve)
}

// --- TestGetSkillContent ---

#[test]
fn test_skill_content_loads() {
    assert!(!get_skill_content().is_empty());
}

#[test]
fn test_skill_content_has_metadata() {
    let content = get_skill_content();
    assert!(content.contains("name: flowmark"));
    assert!(content.contains("description:"));
    assert!(content.contains("allowed-tools:"));
}

#[test]
fn test_skill_content_has_usage() {
    let content = get_skill_content();
    assert!(content.contains("# Flowmark"));
    assert!(content.contains("flowmark --auto"));
}

#[test]
fn test_skill_routes_repository_adoption_to_bundled_reference() {
    let skill = get_skill_content();
    assert!(skill.contains("references/project-setup.md"));
    assert!(skill.contains("same runner with `--install-skill`"));
    let reference = get_project_setup_content();
    assert!(reference.contains("## Auto-Fix on Commit"));
    assert!(reference.contains("## Disable Competing Markdown Formatters"));
}

// --- TestComposeSkill (version roles swapped vs Python) ---

#[test]
fn test_compose_substitutes_explicit_version() {
    let rendered = compose_skill(Some("1.2.3"));
    assert!(rendered.contains("flowmark-rs==1.2.3"));
    assert!(!rendered.contains("__FLOWMARK_RS_VERSION__"));
}

#[test]
fn test_compose_substitutes_both_package_pins() {
    let rendered = compose_skill(Some("1.2.3"));
    assert!(!rendered.contains("__FLOWMARK_VERSION__"));
    // The own (Rust) pin tracks the passed version; the sibling Python pin is its constant.
    assert!(rendered.contains("flowmark-rs==1.2.3"));
    assert!(rendered.contains(&format!("flowmark=={FLOWMARK_PY_DISCOVERY_VERSION}")));
}

#[test]
fn test_compose_recommends_rust_offers_python() {
    let rendered = compose_skill(Some("1.2.3"));
    assert!(rendered.contains("github.com/jlevy/flowmark-rs"));
    assert!(rendered.contains("github.com/jlevy/flowmark)"));
    assert!(rendered.contains("uvx --from flowmark-rs=="));
}

#[test]
fn test_compose_default_pins_installed_version() {
    let rendered = compose_skill(None);
    assert!(!rendered.contains("__FLOWMARK_RS_VERSION__"));
    assert!(rendered.contains(&format!("flowmark-rs=={}", flowmark_rs_version())));
}

#[test]
fn test_compose_doc_pin_is_stable() {
    let rendered = compose_skill(Some(FLOWMARK_RS_DISCOVERY_VERSION));
    assert!(rendered.contains(&format!("flowmark-rs=={FLOWMARK_RS_DISCOVERY_VERSION}")));
    assert_eq!(rendered, compose_skill(Some(FLOWMARK_RS_DISCOVERY_VERSION))); // deterministic
}

#[test]
fn test_compose_preserves_frontmatter() {
    assert!(compose_skill(Some("1.2.3")).starts_with("---\nname: flowmark\n"));
}

#[test]
fn test_compose_project_setup_substitutes_rust_pin() {
    let rendered = compose_project_setup(Some("1.2.3"));
    assert!(rendered.contains("flowmark-rs==1.2.3"));
    assert!(!rendered.contains("__FLOWMARK_RS_VERSION__"));
}

#[test]
fn test_skill_routes_details_to_cli() {
    let content = get_skill_content();
    assert!(content.contains("flowmark --help"));
    assert!(content.contains("flowmark --docs"));
    assert!(!content.contains("emeraldwalk.runonsave"));
}

// --- TestVersionPin ---

#[test]
fn test_real_releases_are_accepted() {
    for v in ["0.7.0", "1.2.3", "0.7", "10.20.30", "0.7.0.post1"] {
        assert!(is_pypi_release(v), "{v}");
    }
}

#[test]
fn test_non_releases_are_rejected() {
    for v in [
        "0.7.1.dev29+c40ee1b",
        "0.6.6.dev7+6de6e10",
        "0.7.0.dev1",
        "1.0.0+local",
        "1.0.0a1",
        "1.0.0b2",
        "1.0.0rc1",
        "",
        "garbage",
    ] {
        assert!(!is_pypi_release(v), "{v}");
    }
}

#[test]
fn test_own_version_is_a_resolvable_pin() {
    // Whatever the running build reports, the pin baked into the skill is PyPI-installable.
    assert!(is_pypi_release(&flowmark_rs_version()));
    assert!(compose_skill(None).contains(&format!("flowmark-rs=={}", flowmark_rs_version())));
}

// --- TestGetDocsContent ---

#[test]
fn test_docs_content_loads() {
    assert!(!get_docs_content().is_empty());
}

#[test]
fn test_docs_content_is_readme() {
    // The Rust port's docs are its own README (distinct from the Python README's
    // sections), so the assertions check sections present in the flowmark-rs README.
    let content = get_docs_content();
    assert!(content.to_lowercase().contains("# flowmark"));
    assert!(content.contains("## Semantic Line Breaks"));
    assert!(content.contains("### Quick Start"));
}

#[test]
fn test_docs_content_has_vscode_cursor_setup() {
    let content = get_docs_content();
    assert!(content.contains("Use in VSCode/Cursor") || content.contains("Use in VS Code/Cursor"));
    assert!(content.contains("emeraldwalk.runonsave"));
}

// --- TestInstallSkill (project-local surfaces) ---

#[test]
fn test_install_default_writes_both_project_local_surfaces() {
    let dir = tempfile::tempdir().expect("temp dir");
    install_skill(None, Some(dir.path()), None).expect("install");
    let portable = dir.path().join(".agents/skills/flowmark/SKILL.md");
    let claude = dir.path().join(".claude/skills/flowmark/SKILL.md");
    assert!(portable.exists());
    assert!(claude.exists());
    assert!(portable.parent().unwrap().join("references/project-setup.md").exists());
    assert!(claude.parent().unwrap().join("references/project-setup.md").exists());
    assert!(fs::read_to_string(&claude).unwrap().contains("name: flowmark"));
}

#[test]
fn test_install_target_selection() {
    let dir = tempfile::tempdir().expect("temp dir");
    install_skill(None, Some(dir.path()), Some(&surfaces(&[SURFACE_PORTABLE]))).expect("install");
    assert!(dir.path().join(".agents/skills/flowmark/SKILL.md").exists());
    assert!(dir.path().join(".agents/skills/flowmark/references/project-setup.md").exists());
    assert!(!dir.path().join(".claude").exists());
    assert!(!dir.path().join("AGENTS.md").exists());
}

#[test]
fn test_installed_file_has_do_not_edit_and_format_stamp() {
    let dir = tempfile::tempdir().expect("temp dir");
    install_skill(None, Some(dir.path()), None).expect("install");
    let content = fs::read_to_string(dir.path().join(".claude/skills/flowmark/SKILL.md")).unwrap();
    assert!(content.contains("DO NOT EDIT"));
    assert!(content.contains("format=f03 surface=skill-md"));
    assert!(content.starts_with("---\nname: flowmark\n"));
    let reference =
        fs::read_to_string(dir.path().join(".claude/skills/flowmark/references/project-setup.md"))
            .unwrap();
    assert!(reference.contains("format=f03 surface=skill-reference"));
}

#[test]
fn test_install_is_idempotent() {
    let dir = tempfile::tempdir().expect("temp dir");
    let first = install_skill(None, Some(dir.path()), None).expect("install");
    assert!(first.iter().all(|r| r.action == "installed"));
    let second = install_skill(None, Some(dir.path()), None).expect("install");
    assert!(second.iter().all(|r| r.action == "unchanged"));
}

#[test]
fn test_forward_compat_guard_blocks_newer_format() {
    let dir = tempfile::tempdir().expect("temp dir");
    let target = dir.path().join(".claude/skills/flowmark/SKILL.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "<!-- format=f99 surface=skill-md -->\nnewer").unwrap();

    let results =
        install_skill(None, Some(dir.path()), Some(&surfaces(&[SURFACE_CLAUDE]))).expect("install");

    assert_eq!(results.iter().map(|r| r.action.as_str()).collect::<Vec<_>>(), ["blocked-newer"]);
    assert_eq!(fs::read_to_string(&target).unwrap(), "<!-- format=f99 surface=skill-md -->\nnewer");
}

#[test]
fn test_forward_compat_guard_checks_bundled_reference() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skill_dir = dir.path().join(".claude/skills/flowmark");
    let target = skill_dir.join("references/project-setup.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "<!-- format=f99 surface=skill-reference -->\nnewer").unwrap();

    let results =
        install_skill(None, Some(dir.path()), Some(&surfaces(&[SURFACE_CLAUDE]))).expect("install");

    assert_eq!(results.iter().map(|r| r.action.as_str()).collect::<Vec<_>>(), ["blocked-newer"]);
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "<!-- format=f99 surface=skill-reference -->\nnewer"
    );
    assert!(!skill_dir.join("SKILL.md").exists());
}

// --- TestAgentsMdBlock ---

#[test]
fn test_block_is_marker_bounded_with_format() {
    let block = agents_md_block(Some("1.2.3"));
    assert!(block.starts_with(AGENTS_BEGIN_PREFIX));
    assert!(block.contains("format=f03"));
    assert!(block.trim_end().ends_with(AGENTS_END_MARKER));
    assert!(block.contains("flowmark-rs==1.2.3"));
}

#[test]
fn test_block_is_flowmark_auto_stable() {
    let block = agents_md_block(Some("0.7.0"));
    let doc = format!("# Project\n\nUser-authored notes.\n\n{block}\n");
    assert!(reformat(&doc).contains(&block));
}

#[test]
fn test_update_creates_and_preserves_user_content() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("AGENTS.md");
    fs::write(&path, "# My Project\n\nHand-written guidance.\n").unwrap();

    update_agents_md(&path, Some("0.7.0")).expect("update");

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("Hand-written guidance."));
    assert!(content.contains(AGENTS_BEGIN_PREFIX));
}

#[test]
fn test_update_replaces_only_the_marked_region() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("AGENTS.md");
    update_agents_md(&path, Some("0.7.0")).expect("update");
    let appended = fs::read_to_string(&path).unwrap() + "\n## User Section\n\nKeep me.\n";
    fs::write(&path, appended).unwrap();

    update_agents_md(&path, Some("9.9.9")).expect("update");

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("Keep me."));
    assert!(content.contains("flowmark-rs==9.9.9"));
    assert!(!content.contains("flowmark-rs==0.7.0"));
    assert_eq!(content.matches(AGENTS_BEGIN_PREFIX).count(), 1);
}

#[test]
fn test_update_is_idempotent() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("AGENTS.md");
    assert_eq!(update_agents_md(&path, Some("0.7.0")).expect("update").action, "installed");
    assert_eq!(update_agents_md(&path, Some("0.7.0")).expect("update").action, "unchanged");
}

#[test]
fn test_update_collapses_duplicate_stale_blocks() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("AGENTS.md");
    let stale = agents_md_block(Some("1.0.0"));
    fs::write(
        &path,
        format!(
            "# Project\n\nUser-authored notes.\n\n{stale}\n\n## User Section\n\nKeep me.\n\n{stale}\n"
        ),
    )
    .unwrap();

    update_agents_md(&path, Some("2.0.0")).expect("update");

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content.matches(AGENTS_BEGIN_PREFIX).count(), 1);
    assert!(!content.contains("flowmark-rs==1.0.0"));
    assert!(content.contains("flowmark-rs==2.0.0"));
    assert!(content.contains("Keep me."));
}

#[test]
fn test_update_guard_blocks_newer_format() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("AGENTS.md");
    fs::write(
        &path,
        format!(
            "{AGENTS_BEGIN_PREFIX} format=f99 surface=agents-md -->\nnewer\n{AGENTS_END_MARKER}\n"
        ),
    )
    .unwrap();
    let result = update_agents_md(&path, Some("0.7.0")).expect("update");
    assert_eq!(result.action, "blocked-newer");
    assert!(fs::read_to_string(&path).unwrap().contains("format=f99"));
}

// --- Single-base (agent_base) install ---

#[test]
fn test_install_skill_custom_base() {
    let dir = tempfile::tempdir().expect("temp dir");
    let base = dir.path().join(".claude");
    install_skill(Some(base.to_str().unwrap()), None, None).expect("install");
    let skill_file = base.join("skills/flowmark/SKILL.md");
    assert!(skill_file.exists());
    assert!(base.join("skills/flowmark/references/project-setup.md").exists());
    assert!(fs::read_to_string(&skill_file).unwrap().contains("name: flowmark"));
}

#[test]
fn test_install_skill_creates_directories() {
    let dir = tempfile::tempdir().expect("temp dir");
    let base = dir.path().join("deep/nested/path");
    install_skill(Some(base.to_str().unwrap()), None, None).expect("install");
    assert!(base.join("skills/flowmark/SKILL.md").exists());
    assert!(base.join("skills/flowmark/references/project-setup.md").exists());
}

#[test]
fn test_install_skill_overwrites_existing() {
    let dir = tempfile::tempdir().expect("temp dir");
    let base = dir.path().join(".claude");
    let skill_dir = base.join("skills/flowmark");
    fs::create_dir_all(&skill_dir).unwrap();
    let skill_file = skill_dir.join("SKILL.md");
    fs::write(&skill_file, "old content").unwrap();

    install_skill(Some(base.to_str().unwrap()), None, None).expect("install");

    let content = fs::read_to_string(&skill_file).unwrap();
    assert!(!content.contains("old content"));
    assert!(content.contains("name: flowmark"));
}

/// M5 regression: `install_skill` with path traversal is rejected.
#[test]
fn test_install_skill_rejects_path_traversal() {
    let result = install_skill(Some("../../tmp/evil"), None, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains(".."));
}

/// M5: `install_skill` with a safe relative base works.
#[test]
fn test_install_skill_safe_relative_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let safe = dir.path().join("project/.claude");
    install_skill(Some(safe.to_str().unwrap()), None, None).expect("install");
    assert!(safe.join("skills/flowmark/SKILL.md").exists());
    assert!(safe.join("skills/flowmark/references/project-setup.md").exists());
}
