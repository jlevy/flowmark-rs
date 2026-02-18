//! Skill system for Claude Code integration.
//!
//! Ported from Python: `flowmark/skill.py`

use std::path::PathBuf;

/// The embedded SKILL.md content.
pub const SKILL_CONTENT: &str = include_str!("SKILL.md");

/// Get the skill (SKILL.md) content.
pub fn get_skill_content() -> &'static str {
    SKILL_CONTENT
}

/// Get documentation content. Tries to find README.md relative to the
/// executable, falling back to basic help text.
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

    // Fallback: basic help text
    "Flowmark: Markdown auto-formatter for clean diffs and semantic line breaks.\n\
     \n\
     For full documentation, visit: https://github.com/jlevy/flowmark-rs\n"
        .to_string()
}

/// Install the flowmark skill to the agent configuration directory.
///
/// Default location: `~/.claude/skills/flowmark/SKILL.md`
/// Custom: `{agent_base}/skills/flowmark/SKILL.md`
///
/// # Errors
///
/// Returns an error if the directory cannot be created or the file cannot be written.
pub fn install_skill(agent_base: Option<&str>) -> Result<(), String> {
    let base: PathBuf = if let Some(custom) = agent_base {
        PathBuf::from(custom)
    } else {
        let Some(home) = dirs::home_dir() else {
            return Err("Could not determine home directory".to_string());
        };
        home.join(".claude")
    };

    let skill_dir = base.join("skills").join("flowmark");
    let skill_path = skill_dir.join("SKILL.md");

    std::fs::create_dir_all(&skill_dir).map_err(|e| format!("Permission denied: {e}"))?;

    std::fs::write(&skill_path, SKILL_CONTENT).map_err(|e| format!("Installation failed: {e}"))?;

    eprintln!("Installed flowmark skill to {}", skill_path.display());

    if agent_base.is_some() {
        eprintln!("Tip: Commit .claude/skills/ to share with team");
    }

    Ok(())
}
