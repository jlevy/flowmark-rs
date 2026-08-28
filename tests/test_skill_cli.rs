//! CLI-level tests for `flowmark --install-skill --surfaces` parsing and the
//! project-root git recommendation.
//!
//! Ported from Python: `test_skill.py::TestInstallSkillCli` and the git-recommendation
//! cases in `TestInstallSkill`. These drive the built binary with a temp working
//! directory, the Rust analog of Python's `monkeypatch.chdir` + `capsys`.

#![cfg(feature = "cli")]
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;

/// Cargo points `CARGO_BIN_EXE_flowmark` at the CLI it built for this test, so the path
/// follows the active profile and carries the platform executable suffix.
fn rust_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_flowmark"))
}

struct RunOutput {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_in(cwd: &std::path::Path, args: &[&str]) -> RunOutput {
    let out = Command::new(rust_binary())
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to run binary: {e}"));
    RunOutput {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

#[test]
fn test_default_writes_all_three_surfaces() {
    let dir = tempfile::tempdir().unwrap();
    let r = run_in(dir.path(), &["--install-skill"]);
    assert_eq!(r.code, 0);
    assert!(dir.path().join(".agents/skills/flowmark/SKILL.md").exists());
    assert!(dir.path().join(".claude/skills/flowmark/SKILL.md").exists());
    assert!(dir.path().join(".agents/skills/flowmark/references/project-setup.md").exists());
    assert!(dir.path().join(".claude/skills/flowmark/references/project-setup.md").exists());
    assert!(dir.path().join("AGENTS.md").exists());
}

#[test]
fn test_surfaces_subset() {
    let dir = tempfile::tempdir().unwrap();
    let r = run_in(dir.path(), &["--install-skill", "--surfaces=portable,agents-md"]);
    assert_eq!(r.code, 0);
    assert!(dir.path().join(".agents/skills/flowmark/SKILL.md").exists());
    assert!(dir.path().join("AGENTS.md").exists());
    assert!(!dir.path().join(".claude").exists());
}

#[test]
fn test_surfaces_all_alias() {
    let dir = tempfile::tempdir().unwrap();
    let r = run_in(dir.path(), &["--install-skill", "--surfaces=all"]);
    assert_eq!(r.code, 0);
    assert!(dir.path().join(".agents/skills/flowmark/SKILL.md").exists());
    assert!(dir.path().join(".claude/skills/flowmark/SKILL.md").exists());
    assert!(dir.path().join("AGENTS.md").exists());
}

#[test]
fn test_unknown_surface_errors_out() {
    let dir = tempfile::tempdir().unwrap();
    let r = run_in(dir.path(), &["--install-skill", "--surfaces=cursor"]);
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("unknown surface 'cursor'"));
    assert!(!dir.path().join(".claude").exists());
    assert!(!dir.path().join(".agents").exists());
}

#[test]
fn test_empty_surfaces_value_errors_out() {
    let dir = tempfile::tempdir().unwrap();
    let r = run_in(dir.path(), &["--install-skill", "--surfaces="]);
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("empty value"));
}

#[test]
fn test_surfaces_with_agent_base_errors_out() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base");
    let r = run_in(
        dir.path(),
        &["--install-skill", "--agent-base", base.to_str().unwrap(), "--surfaces=portable"],
    );
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("incompatible with --agent-base"));
}

#[test]
fn test_install_outside_git_repo_recommends_project_root() {
    let dir = tempfile::tempdir().unwrap();
    let r = run_in(dir.path(), &["--install-skill"]);
    assert!(r.stdout.contains("not inside a git repository"));
}

#[test]
fn test_install_inside_git_repo_has_no_recommendation() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    let r = run_in(dir.path(), &["--install-skill"]);
    assert!(!r.stdout.contains("not inside a git repository"));
}

#[test]
fn test_install_agent_base_skips_git_recommendation() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base");
    let r = run_in(dir.path(), &["--install-skill", "--agent-base", base.to_str().unwrap()]);
    assert!(!r.stdout.contains("not inside a git repository"));
}
