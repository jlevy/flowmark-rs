//! Build-time version metadata helpers shared by build scripts and tests.

/// Derived version metadata used for CLI `--version` output and related env vars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionMetadata {
    /// Base tag shown in the long version metadata section.
    pub base_tag: String,
    /// Commits-ahead marker persisted for diagnostics.
    pub commits_ahead: String,
    /// Short git hash persisted for diagnostics.
    pub git_hash: String,
    /// Human-readable long version string printed by `flowmark --version`.
    pub long_version: String,
}

/// Compute stable vs dev version metadata.
///
/// Rules:
/// - If `release_tag_override` matches `v{package_version}`, force a stable version string.
/// - Otherwise, include `-dev.N+gHASH` only when `commits_ahead_from_git` is numeric and > 0.
/// - If git metadata is unavailable/ambiguous, default to stable (no `dev.unknown` suffix).
pub fn compute_version_metadata(
    package_version: &str,
    parity_version: &str,
    release_tag_override: Option<&str>,
    base_tag_from_git: Option<&str>,
    commits_ahead_from_git: Option<&str>,
    git_hash_from_git: Option<&str>,
) -> VersionMetadata {
    let expected_release_tag = format!("v{package_version}");
    let release_tag_override = release_tag_override.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    });

    let git_hash = git_hash_from_git.unwrap_or("unknown").to_string();
    let commits_ahead_raw = commits_ahead_from_git.unwrap_or("unknown");

    if release_tag_override == Some(expected_release_tag.as_str()) {
        let long_version = format!(
            "{package_version} (Rust port of flowmark-py {parity_version}; base {expected_release_tag})"
        );
        return VersionMetadata {
            base_tag: expected_release_tag,
            commits_ahead: "0".to_string(),
            git_hash,
            long_version,
        };
    }

    let base_tag = base_tag_from_git.unwrap_or(&expected_release_tag).to_string();
    let dev_suffix = match commits_ahead_raw.parse::<u64>() {
        Ok(commits) if commits > 0 => format!("-dev.{commits}+g{git_hash}"),
        _ => String::new(),
    };

    let long_version = format!(
        "{package_version}{dev_suffix} (Rust port of flowmark-py {parity_version}; base {base_tag})"
    );

    VersionMetadata {
        base_tag,
        commits_ahead: commits_ahead_raw.to_string(),
        git_hash,
        long_version,
    }
}
