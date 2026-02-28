//! Runtime settings shared by CLI and cache components.

use std::path::PathBuf;

/// Fallback directory used when the OS cache root cannot be resolved.
pub const FALLBACK_CACHE_DIR: &str = ".flowmark-cache";

/// Application-specific cache directory name under the cache root.
pub const APP_CACHE_DIR: &str = "flowmark";

/// Subdirectory containing incremental cache manifests.
pub const INCREMENTAL_CACHE_SUBDIR: &str = "incremental";

/// Resolve the default cache root directory.
///
/// Uses the OS cache location when available and falls back to a local
/// `.flowmark-cache/flowmark` path when unavailable.
pub fn default_cache_root() -> PathBuf {
    dirs::cache_dir().map_or_else(
        || PathBuf::from(FALLBACK_CACHE_DIR).join(APP_CACHE_DIR),
        |cache_dir| cache_dir.join(APP_CACHE_DIR),
    )
}
