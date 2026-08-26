//! Skill system for Claude Code integration.
//!
//! Ported from Python: `flowmark/skill.py`

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The embedded SKILL.md content.
pub const SKILL_CONTENT: &str = include_str!("SKILL.md");

/// The embedded repository-adoption reference installed beside `SKILL.md`.
pub const PROJECT_SETUP_CONTENT: &str = include_str!("references/project-setup.md");
/// Embedded documentation content generated at build time.
pub const DOCS_CONTENT: &str = include_str!(concat!(env!("OUT_DIR"), "/flowmark_docs.md"));

/// Get the skill (SKILL.md) content.
pub fn get_skill_content() -> &'static str {
    SKILL_CONTENT
}

/// Get documentation content. Tries to find README.md relative to the
/// executable, falling back to embedded README content.
pub fn get_docs_content() -> String {
    // Try to find README.md relative to the executable
    if let Ok(exe) = std::env::current_exe() {
        // Check alongside the binary
        if let Some(dir) = exe.parent() {
            let readme = dir.join("README.md");
            if let Ok(content) = std::fs::read_to_string(&readme) {
                return content;
            }
            // Check one level up (e.g., if binary is in bin/ or target/)
            if let Some(parent) = dir.parent() {
                let readme = parent.join("README.md");
                if let Ok(content) = std::fs::read_to_string(&readme) {
                    return content;
                }
            }
        }
    }

    // Fallback: embedded docs content.
    DOCS_CONTENT.to_string()
}

/// A project-local agent-discovery surface.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SkillSurface {
    /// `.agents/skills/flowmark/SKILL.md` for portable agent discovery.
    Portable,
    /// `.claude/skills/flowmark/SKILL.md` for Claude Code.
    Claude,
    /// A marker-bounded Flowmark section in `AGENTS.md`.
    AgentsMd,
}

/// Every project-local skill surface in deterministic order.
pub const ALL_SURFACES: &[SkillSurface] =
    &[SkillSurface::Portable, SkillSurface::Claude, SkillSurface::AgentsMd];

/// Validate a `--surfaces` value for clap while preserving its original spelling.
pub fn validate_surfaces(value: &str) -> Result<String, String> {
    parse_surface_set(value).map(|_| value.to_owned())
}

/// Parse a previously validated comma-separated surface list.
pub fn parse_surfaces(value: &str) -> Vec<SkillSurface> {
    parse_surface_set(value).expect("clap validated --surfaces").into_iter().collect()
}

fn parse_surface_set(value: &str) -> Result<BTreeSet<SkillSurface>, String> {
    let tokens: Vec<_> =
        value.split(',').map(str::trim).filter(|token| !token.is_empty()).collect();
    if tokens.is_empty() {
        return Err(
            "--surfaces requires a comma-separated list of portable, claude, agents-md, or all"
                .to_owned(),
        );
    }
    let mut surfaces = BTreeSet::new();
    for token in tokens {
        match token {
            "portable" => {
                surfaces.insert(SkillSurface::Portable);
            }
            "claude" => {
                surfaces.insert(SkillSurface::Claude);
            }
            "agents-md" => {
                surfaces.insert(SkillSurface::AgentsMd);
            }
            "all" => surfaces.extend(ALL_SURFACES.iter().copied()),
            unknown => {
                return Err(format!(
                    "unknown surface {unknown:?}; valid values: portable, claude, agents-md, all"
                ));
            }
        }
    }
    Ok(surfaces)
}

fn write_skill(directory: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("failed to create {}: {error}", directory.display()))?;
    let reference_path = directory.join("references/project-setup.md");
    let reference_directory = reference_path
        .parent()
        .ok_or_else(|| format!("reference path has no parent: {}", reference_path.display()))?;
    std::fs::create_dir_all(reference_directory)
        .map_err(|error| format!("failed to create {}: {error}", reference_directory.display()))?;
    std::fs::write(&reference_path, PROJECT_SETUP_CONTENT)
        .map_err(|error| format!("failed to write {}: {error}", reference_path.display()))?;
    let skill_path = directory.join("SKILL.md");
    std::fs::write(&skill_path, SKILL_CONTENT)
        .map_err(|error| format!("failed to write {}: {error}", skill_path.display()))?;
    Ok(skill_path)
}

fn install_agents_md(path: &Path) -> Result<(), String> {
    const BEGIN: &str = "<!-- BEGIN FLOWMARK INTEGRATION";
    const END: &str = "<!-- END FLOWMARK INTEGRATION -->";
    const BLOCK: &str = "<!-- BEGIN FLOWMARK INTEGRATION format=f03 surface=agents-md -->\n\
## flowmark\n\n\
Auto-format Markdown with `flowmark` for clean, semantic git diffs.\n\n\
- Run `flowmark --docs` for full usage and `flowmark --skill` for the skill.\n\
- Run `flowmark --auto <files>` on Markdown you create or edit.\n\n\
<!-- END FLOWMARK INTEGRATION -->";

    let current = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    let updated = if let Some(start) = current.find(BEGIN) {
        let suffix = &current[start..];
        let relative_end = suffix.find(END).ok_or_else(|| {
            format!("{} contains an unterminated Flowmark integration block", path.display())
        })?;
        let end = start + relative_end + END.len();
        format!("{}{}{}", &current[..start], BLOCK, &current[end..])
    } else if current.is_empty() {
        format!("{BLOCK}\n")
    } else {
        format!("{}\n\n{BLOCK}\n", current.trim_end())
    };
    std::fs::write(path, updated)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

/// Install the Flowmark skill to an explicit legacy base.
///
/// # Errors
///
/// Returns an error when a target path cannot be read, created, or written.
pub fn install_skill(agent_base: Option<&str>) -> Result<(), String> {
    let base = agent_base.map_or_else(
        || {
            dirs::home_dir()
                .map(|home| home.join(".claude"))
                .ok_or_else(|| "could not determine home directory".to_owned())
        },
        |custom| Ok(PathBuf::from(custom)),
    )?;
    let base =
        base.to_str().ok_or_else(|| format!("agent base path is not UTF-8: {}", base.display()))?;
    install_skill_surfaces(Some(base), ALL_SURFACES)
}

/// Install the Flowmark skill to an explicit base or selected project-local surfaces.
///
/// # Errors
///
/// Returns an error when a target path cannot be read, created, or written.
pub fn install_skill_surfaces(
    agent_base: Option<&str>,
    surfaces: &[SkillSurface],
) -> Result<(), String> {
    if let Some(custom) = agent_base {
        let base = PathBuf::from(custom);
        if base.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
            return Err(format!("invalid --agent-base path (contains '..'): {custom}"));
        }
        let skill_path = write_skill(&base.join("skills/flowmark"))?;
        println!("Installed flowmark skill to {}", skill_path.display());
        return Ok(());
    }

    let root = std::env::current_dir()
        .map_err(|error| format!("failed to resolve current directory: {error}"))?;
    for surface in surfaces {
        match surface {
            SkillSurface::Portable => {
                let path = write_skill(&root.join(".agents/skills/flowmark"))?;
                println!("Installed portable flowmark skill to {}", path.display());
            }
            SkillSurface::Claude => {
                let path = write_skill(&root.join(".claude/skills/flowmark"))?;
                println!("Installed Claude flowmark skill to {}", path.display());
            }
            SkillSurface::AgentsMd => {
                let path = root.join("AGENTS.md");
                install_agents_md(&path)?;
                println!("Installed Flowmark integration in {}", path.display());
            }
        }
    }
    Ok(())
}
