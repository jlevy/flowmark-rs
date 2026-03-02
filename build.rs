use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "src/version_meta.rs"]
mod version_meta;

fn main() {
    // Read parity version from Cargo.toml [package.metadata.parity]
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let table: toml::Table = cargo_toml.parse().expect("Failed to parse Cargo.toml");

    let parity_version = table
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("parity"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    println!("cargo::rustc-env=PARITY_VERSION={parity_version}");

    let package_version = table
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let base_tag_from_git = run_git(["describe", "--tags", "--abbrev=0", "--match", "v[0-9]*"]);
    let commits_ahead_from_git = base_tag_from_git.as_deref().and_then(git_commits_ahead);
    let git_hash = run_git(["rev-parse", "--short=7", "HEAD"]);
    let release_tag_override = std::env::var("FLOWMARK_RELEASE_TAG").ok();

    let metadata = version_meta::compute_version_metadata(
        package_version,
        parity_version,
        release_tag_override.as_deref(),
        base_tag_from_git.as_deref(),
        commits_ahead_from_git.as_deref(),
        git_hash.as_deref(),
    );

    println!("cargo::rustc-env=FLOWMARK_GIT_BASE_TAG={}", metadata.base_tag);
    println!("cargo::rustc-env=FLOWMARK_GIT_COMMITS_AHEAD={}", metadata.commits_ahead);
    println!("cargo::rustc-env=FLOWMARK_GIT_HASH={}", metadata.git_hash);
    println!("cargo::rustc-env=FLOWMARK_LONG_VERSION={}", metadata.long_version);

    // Canonical docs source for runtime `--docs`: Rust README.
    // The Rust README is generated from a shared docs body + Rust wrapper preface.
    // Fallback to shared docs source if README is unavailable.
    let rust_readme = Path::new("README.md");
    let shared_docs = Path::new("repos/flowmark/docs/shared/flowmark-readme-shared.md");
    let docs_source = if rust_readme.exists() { rust_readme } else { shared_docs };
    let docs_content = std::fs::read_to_string(docs_source)
        .unwrap_or_else(|e| panic!("Failed to read docs source {}: {e}", docs_source.display()));
    let out_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let generated_docs = Path::new(&out_dir).join("flowmark_docs.md");
    std::fs::write(&generated_docs, docs_content).unwrap_or_else(|e| {
        panic!("Failed to write generated docs {}: {e}", generated_docs.display())
    });

    emit_git_rerun_hints();
    println!("cargo::rerun-if-changed=Cargo.toml");
    println!("cargo::rerun-if-changed=README.md");
    println!("cargo::rerun-if-changed=repos/flowmark/docs/shared/flowmark-readme-shared.md");
    println!("cargo::rerun-if-env-changed=FLOWMARK_RELEASE_TAG");
}

fn run_git<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    if value.is_empty() { None } else { Some(value.to_string()) }
}

fn git_commits_ahead(base_tag: &str) -> Option<String> {
    let range = format!("{base_tag}..HEAD");
    let output = Command::new("git").args(["rev-list", "--count", &range]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    if value.is_empty() { None } else { Some(value.to_string()) }
}

fn emit_git_rerun_hints() {
    let dot_git = Path::new(".git");
    println!("cargo::rerun-if-changed=.git");
    println!("cargo::rerun-if-changed=.git/HEAD");
    println!("cargo::rerun-if-changed=.git/packed-refs");
    println!("cargo::rerun-if-changed=.git/refs");

    if let Ok(contents) = std::fs::read_to_string(dot_git) {
        let Some(gitdir_rel_or_abs) =
            contents.lines().next().and_then(|line| line.trim().strip_prefix("gitdir: "))
        else {
            return;
        };
        let mut gitdir = PathBuf::from(gitdir_rel_or_abs);
        if gitdir.is_relative() {
            gitdir = Path::new(".").join(gitdir);
        }
        println!("cargo::rerun-if-changed={}", gitdir.join("HEAD").display());
        println!("cargo::rerun-if-changed={}", gitdir.join("packed-refs").display());
        println!("cargo::rerun-if-changed={}", gitdir.join("refs").display());
    }
}
