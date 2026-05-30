//! Cross-agent skill installation for flowmark.
//!
//! Ported from Python: `flowmark/skill.py`.
//!
//! Installs the flowmark `SKILL.md` so AI coding agents can discover and use it. By
//! default this installs project-locally into both the portable `.agents/skills/flowmark/`
//! location (read by Codex, Gemini CLI, pi) and the `.claude/skills/flowmark/` mirror
//! (Claude Code reads only that path), plus a marker-bounded block in `AGENTS.md`. A
//! legacy single-base install (`agent_base`, e.g. `~/.claude`) is kept for
//! explicit/global installs.
//!
//! # Cross-implementation porting contract
//!
//! The authored `SKILL.md` pins BOTH packages via two placeholders and is byte-identical
//! across the Python and Rust implementations — only the substitution sources differ:
//!
//! - Python `flowmark`: `__FLOWMARK_VERSION__` <- its own installed version (dynamic),
//!   `__FLOWMARK_RS_VERSION__` <- a sibling flowmark-rs discovery constant.
//! - Rust `flowmark-rs` (here): `__FLOWMARK_RS_VERSION__` <- its own version (dynamic),
//!   `__FLOWMARK_VERSION__` <- a sibling flowmark (Python) discovery constant.
//!
//! The roles swap, but a default install from either side produces identical artifacts.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

/// The embedded SKILL.md template (still contains the version placeholders).
pub const SKILL_CONTENT: &str = include_str!("SKILL.md");
/// Embedded documentation content generated at build time.
pub const DOCS_CONTENT: &str = include_str!(concat!(env!("OUT_DIR"), "/flowmark_docs.md"));

/// Format version stamped on every flowmark-generated artifact (skill SKILL.md mirrors
/// and the AGENTS.md block). One monotonically-increasing `fNN` across the project; the
/// `surface=` field distinguishes which artifact. Bump when any artifact's shape changes;
/// a future flowmark uses the stamp to upgrade older shapes and refuses to clobber a
/// newer format it doesn't understand.
pub const FLOWMARK_FORMAT: &str = "f02";

// Placeholders in the authored SKILL.md, replaced by `compose_skill` with concrete pins.
const VERSION_PLACEHOLDER: &str = "__FLOWMARK_VERSION__"; // sibling package (flowmark, Python)
const RS_VERSION_PLACEHOLDER: &str = "__FLOWMARK_RS_VERSION__"; // this package (flowmark-rs)

/// This port's own discovery pin: the fallback flowmark-rs version baked into the
/// committed discovery copy and used when the running build's version is not a real,
/// PyPI-installable release. Must be a real release (guarded by tests).
pub const FLOWMARK_RS_DISCOVERY_VERSION: &str = "0.3.0";

/// Sibling Python `flowmark` discovery pin substituted for `__FLOWMARK_VERSION__`. The
/// two packages are numbered independently; this tracks the Python parity baseline.
pub const FLOWMARK_PY_DISCOVERY_VERSION: &str = "0.7.2";

/// Project-local skill directory name.
pub const SKILL_DIRNAME: &str = "flowmark";

/// Surface identifiers, matching the `surface=` artifact stamp and the `--surfaces` flag.
pub const SURFACE_PORTABLE: &str = "portable"; // .agents/skills/flowmark/SKILL.md
pub const SURFACE_CLAUDE: &str = "claude"; // .claude/skills/flowmark/SKILL.md
pub const SURFACE_AGENTS_MD: &str = "agents-md"; // marker-bounded block in AGENTS.md
/// All three project-local surfaces (the default when `--surfaces` is omitted).
pub const ALL_SURFACES: [&str; 3] = [SURFACE_PORTABLE, SURFACE_CLAUDE, SURFACE_AGENTS_MD];

/// `AGENTS.md` block markers.
pub const AGENTS_BEGIN_PREFIX: &str = "<!-- BEGIN FLOWMARK INTEGRATION";
pub const AGENTS_END_MARKER: &str = "<!-- END FLOWMARK INTEGRATION -->";

static FORMAT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"format=f(\d+)").expect("valid FORMAT_RE"));
static AGENTS_BEGIN_STAMP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"{}\s+format=f(\d+)", regex::escape(AGENTS_BEGIN_PREFIX)))
        .expect("valid AGENTS_BEGIN_STAMP_RE")
});
static AGENTS_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "(?s){}.*?{}",
        regex::escape(AGENTS_BEGIN_PREFIX),
        regex::escape(AGENTS_END_MARKER)
    ))
    .expect("valid AGENTS_BLOCK_RE")
});
/// An exact-pinnable published `PyPI` release: a plain release segment, optionally `.postN`.
static PYPI_RELEASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+(?:\.\d+)*(?:\.post\d+)?$").expect("valid PYPI_RELEASE_RE"));

/// Whether `version_str` is a real, exact-pinnable published `PyPI` release. Dev
/// (`.devN`), pre-release (`aN`/`bN`/`rcN`), and local (`+<hash>`) versions are rejected.
pub fn is_pypi_release(version_str: &str) -> bool {
    PYPI_RELEASE_RE.is_match(version_str)
}

/// Pick a PyPI-installable pin: the installed version if it is a real release, otherwise
/// the discovery fallback. A dev/local build reports a version that was never published.
fn pin_or_discovery(installed: &str, discovery: &str) -> String {
    if is_pypi_release(installed) { installed.to_string() } else { discovery.to_string() }
}

/// The flowmark-rs version to pin in the recommended `uvx --from flowmark-rs==<pin>`
/// bootstrap line. Falls back to `FLOWMARK_RS_DISCOVERY_VERSION` for dev/local builds.
pub fn flowmark_rs_version() -> String {
    pin_or_discovery(env!("CARGO_PKG_VERSION"), FLOWMARK_RS_DISCOVERY_VERSION)
}

/// The authored SKILL.md template (still contains the version placeholders).
pub fn get_skill_content() -> &'static str {
    SKILL_CONTENT
}

/// Render the SKILL.md template into a final skill document.
///
/// `version` is this port's own (`flowmark-rs`) pin substituted into the recommended
/// runner line. Pass `None` to pin the running build's version. Deterministic.
pub fn compose_skill(version: Option<&str>) -> String {
    let pin = version.map_or_else(flowmark_rs_version, String::from);
    SKILL_CONTENT
        .replace(RS_VERSION_PLACEHOLDER, &pin)
        .replace(VERSION_PLACEHOLDER, FLOWMARK_PY_DISCOVERY_VERSION)
}

fn format_num() -> i64 {
    FLOWMARK_FORMAT.trim_start_matches('f').parse().unwrap_or(0)
}

fn generated_marker() -> String {
    // No internal `.` so flowmark's sentence-wrap leaves the line intact.
    format!(
        "<!-- DO NOT EDIT: `flowmark --install-skill` (format={FLOWMARK_FORMAT} surface=skill-md) -->"
    )
}

/// The skill document to write to disk: `compose_skill` plus a `DO NOT EDIT` +
/// format-version marker inserted after the YAML frontmatter (so frontmatter stays first
/// and the marker survives a `flowmark --auto` pass).
pub fn render_skill_file(version: Option<&str>) -> String {
    let composed = compose_skill(version);
    let marker = generated_marker();
    let delimiter = "\n---\n";
    let front = "---\n";
    if let Some(rest) = composed.strip_prefix(front) {
        if let Some(rel) = rest.find(delimiter) {
            let split = front.len() + rel + delimiter.len();
            let head = &composed[..split];
            let body = &composed[split..];
            return format!("{head}{marker}\n\n{body}");
        }
    }
    format!("{marker}\n\n{composed}")
}

/// The committed repo-root discovery copy (`skills/flowmark/SKILL.md`), pinned to the
/// discovery versions so its `uvx --from` bootstrap lines run without flowmark
/// pre-installed.
pub fn discovery_skill_text() -> String {
    render_skill_file(Some(FLOWMARK_RS_DISCOVERY_VERSION))
}

/// Read README.md relative to the executable; falls back to embedded docs content.
pub fn get_docs_content() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let readme = dir.join("README.md");
            if let Ok(content) = std::fs::read_to_string(&readme) {
                return content;
            }
            if let Some(parent) = dir.parent() {
                let readme = parent.join("README.md");
                if let Ok(content) = std::fs::read_to_string(&readme) {
                    return content;
                }
            }
        }
    }
    DOCS_CONTENT.to_string()
}

/// The compact, marker-bounded flowmark block for a project's `AGENTS.md`.
///
/// `version` is this port's own pin (default `None` -> the running build's version). Short
/// lines and no mid-document frontmatter, so a `flowmark --auto` pass leaves it unchanged.
pub fn agents_md_block(version: Option<&str>) -> String {
    let pin = version.map_or_else(flowmark_rs_version, String::from);
    format!(
        "{AGENTS_BEGIN_PREFIX} format={FLOWMARK_FORMAT} surface=agents-md -->\n\
         ## flowmark\n\
         \n\
         Auto-format Markdown with `flowmark` for clean, semantic git diffs.\n\
         \n\
         - Run `flowmark --auto <files>` on Markdown you create or edit.\n\
         - Run `flowmark --docs` for full usage and `flowmark --skill` for the skill.\n\
         - If `flowmark` is not on `PATH`, use a pinned `uvx` runner (never `@latest`).\n\
         - Fast Rust port (recommended): `uvx --from flowmark-rs=={pin} flowmark`.\n\
         - Python build (library / newest patch): `uvx --from flowmark=={FLOWMARK_PY_DISCOVERY_VERSION} flowmark`.\n\
         \n\
         {AGENTS_END_MARKER}"
    )
}

/// Per-surface install outcome. `action` is one of `installed`, `updated`, `unchanged`,
/// or `blocked-newer`.
#[derive(Debug, Clone)]
pub struct InstallResult {
    pub surface: String,
    pub path: PathBuf,
    pub action: String,
}

impl InstallResult {
    fn new(surface: &str, path: PathBuf, action: &str) -> Self {
        Self { surface: surface.to_string(), path, action: action.to_string() }
    }
}

/// Format number stamped on an existing generated file; `Some(0)` if unmarked; `None` if
/// the file is absent.
fn existing_format(path: &Path) -> Option<i64> {
    if !path.is_file() {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    Some(FORMAT_RE.captures(&text).and_then(|c| c[1].parse().ok()).unwrap_or(0))
}

/// Replace every flowmark BEGIN/END region in `existing` with a single fresh `block`,
/// preserving user content outside the markers and collapsing duplicate/stale blocks.
fn replace_all_flowmark_blocks(existing: &str, block: &str) -> String {
    let matches: Vec<_> = AGENTS_BLOCK_RE.find_iter(existing).collect();
    if matches.is_empty() {
        return existing.to_string();
    }
    let mut out = String::new();
    out.push_str(&existing[..matches[0].start()]);
    out.push_str(block);
    for i in 1..matches.len() {
        out.push_str(&existing[matches[i - 1].end()..matches[i].start()]);
    }
    out.push_str(&existing[matches[matches.len() - 1].end()..]);
    out
}

fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

/// Insert or refresh the flowmark block in `AGENTS.md`, preserving all content outside the
/// markers. Idempotent; honors the forward-compatibility guard; collapses duplicate or
/// stale blocks to one current block.
pub fn update_agents_md(path: &Path, version: Option<&str>) -> std::io::Result<InstallResult> {
    let surface = "AGENTS.md (flowmark block)";
    let existing = std::fs::read_to_string(path).ok();

    if let Some(ref e) = existing {
        if let Some(caps) = AGENTS_BEGIN_STAMP_RE.captures(e) {
            if caps[1].parse::<i64>().unwrap_or(0) > format_num() {
                return Ok(InstallResult::new(surface, path.to_path_buf(), "blocked-newer"));
            }
        }
    }

    let block = agents_md_block(version);
    let new_content = match &existing {
        Some(e) if e.contains(AGENTS_BEGIN_PREFIX) => replace_all_flowmark_blocks(e, &block),
        Some(e) if !e.is_empty() => {
            let sep = if e.ends_with('\n') { "\n" } else { "\n\n" };
            format!("{e}{sep}{block}\n")
        }
        _ => format!("{block}\n"),
    };

    if existing.as_deref() == Some(new_content.as_str()) {
        return Ok(InstallResult::new(surface, path.to_path_buf(), "unchanged"));
    }
    let action = if existing.is_some() { "updated" } else { "installed" };
    write_file(path, &new_content)?;
    Ok(InstallResult::new(surface, path.to_path_buf(), action))
}

/// Write a SKILL.md surface, honoring the forward-compatibility guard.
fn write_surface(skill_dir: &Path, surface: &str, content: &str) -> std::io::Result<InstallResult> {
    let target = skill_dir.join("SKILL.md");
    if let Some(fmt) = existing_format(&target) {
        if fmt > format_num() {
            return Ok(InstallResult::new(surface, target, "blocked-newer"));
        }
    }
    if target.is_file() && std::fs::read_to_string(&target).ok().as_deref() == Some(content) {
        return Ok(InstallResult::new(surface, target, "unchanged"));
    }
    let action = if target.exists() { "updated" } else { "installed" };
    write_file(&target, content)?;
    Ok(InstallResult::new(surface, target, action))
}

/// Whether `path` or any ancestor contains a `.git` entry (a repo dir or the `.git` file
/// used by worktrees/submodules).
fn is_within_git_repo(path: &Path) -> bool {
    let mut current = Some(path);
    while let Some(p) = current {
        if p.join(".git").exists() {
            return true;
        }
        current = p.parent();
    }
    false
}

fn print_install_summary(results: &[InstallResult]) {
    println!("\nFlowmark skill installation:");
    for r in results {
        if r.action == "blocked-newer" {
            println!(
                "  ‼️  {}: {} was generated by a NEWER flowmark.",
                r.surface,
                r.path.display()
            );
            println!(
                "      Upgrade flowmark (e.g. `uv tool install --upgrade flowmark`) and retry."
            );
        } else {
            println!("  ✅ {:<9} {}: {}", r.action, r.surface, r.path.display());
        }
    }
    println!();
}

/// Install the flowmark skill, version-pinned to the running build.
///
/// With `agent_base` set, does a single-base install to `{agent_base}/skills/flowmark/`
/// (explicit/global, e.g. `~/.claude`) and `surfaces` is ignored. Otherwise installs
/// project-locally under `project_root` (default: cwd); `surfaces` selects which of the
/// three project-local surfaces to write (`None` = all three). Idempotent; prints a
/// per-surface summary and a git-repo recommendation when relevant.
///
/// # Errors
///
/// Returns an error if a skill file cannot be created or written, or if `agent_base`
/// contains a `..` traversal component.
#[allow(clippy::implicit_hasher)]
pub fn install_skill(
    agent_base: Option<&str>,
    project_root: Option<&Path>,
    surfaces: Option<&HashSet<String>>,
) -> Result<Vec<InstallResult>, String> {
    let content = render_skill_file(None);
    let selected: HashSet<String> = surfaces
        .cloned()
        .unwrap_or_else(|| ALL_SURFACES.iter().map(|s| (*s).to_string()).collect());

    let mut results: Vec<InstallResult> = Vec::new();
    let mut project_local_root: Option<PathBuf> = None;

    let io_err = |e: std::io::Error| format!("Installation failed: {e}");

    if let Some(base) = agent_base {
        let base_path = PathBuf::from(base);
        if base_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(format!("invalid --agent-base path (contains '..'): {base}"));
        }
        let base_abs = base_path
            .canonicalize()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(&base_path));
        let label = base_abs.display().to_string();
        results.push(
            write_surface(&base_abs.join("skills").join(SKILL_DIRNAME), &label, &content)
                .map_err(io_err)?,
        );
    } else {
        let root = match project_root {
            Some(p) => p.to_path_buf(),
            None => std::env::current_dir().map_err(|e| e.to_string())?,
        };
        project_local_root = Some(root.clone());
        if selected.contains(SURFACE_PORTABLE) {
            let dir = root.join(".agents").join("skills").join(SKILL_DIRNAME);
            results
                .push(write_surface(&dir, ".agents/skills (portable)", &content).map_err(io_err)?);
        }
        if selected.contains(SURFACE_AGENTS_MD) {
            results.push(update_agents_md(&root.join("AGENTS.md"), None).map_err(io_err)?);
        }
        if selected.contains(SURFACE_CLAUDE) {
            let dir = root.join(".claude").join("skills").join(SKILL_DIRNAME);
            results.push(
                write_surface(&dir, ".claude/skills (Claude Code)", &content).map_err(io_err)?,
            );
        }
    }

    print_install_summary(&results);
    if let Some(root) = project_local_root {
        if !is_within_git_repo(&root) {
            println!(
                "Note: {} is not inside a git repository. Agent skills are best installed at \
                 a project root so agents working in that project discover them. Re-run \
                 `flowmark --install-skill` from your project directory, or use \
                 `--agent-base <DIR>` (e.g. ~/.claude) for an explicit global install.\n",
                root.display()
            );
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_or_discovery_passes_real_release_through() {
        assert_eq!(pin_or_discovery("0.9.4", "0.3.0"), "0.9.4");
    }

    #[test]
    fn pin_or_discovery_falls_back_on_dev_version() {
        assert_eq!(pin_or_discovery("0.3.0.dev29+c40ee1b", "0.3.0"), "0.3.0");
        assert_eq!(pin_or_discovery("garbage", "0.3.0"), "0.3.0");
    }

    #[test]
    fn is_pypi_release_accepts_real_releases() {
        for v in ["0.7.0", "1.2.3", "0.7", "10.20.30", "0.7.0.post1"] {
            assert!(is_pypi_release(v), "{v} should be a release");
        }
    }

    #[test]
    fn is_pypi_release_rejects_non_releases() {
        for v in [
            "0.7.1.dev29+c40ee1b",
            "0.7.0.dev1",
            "1.0.0+local",
            "1.0.0a1",
            "1.0.0rc1",
            "",
            "garbage",
        ] {
            assert!(!is_pypi_release(v), "{v} should be rejected");
        }
    }
}
