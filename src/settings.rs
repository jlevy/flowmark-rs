//! Runtime settings shared by CLI and cache components.

use std::path::{Path, PathBuf};

/// Home-directory fallback directory name when OS cache root cannot be resolved.
pub const FALLBACK_CACHE_DIR: &str = ".flowmark-cache";

/// Temp-directory fallback directory name when OS and home cache roots
/// cannot be resolved.
pub const TEMP_FALLBACK_CACHE_DIR: &str = "flowmark-cache";

/// Application-specific cache directory name under the cache root.
pub const APP_CACHE_DIR: &str = "flowmark";

/// Subdirectory containing incremental cache manifests.
pub const INCREMENTAL_CACHE_SUBDIR: &str = "incremental";

/// Source used to resolve the default cache root directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheRootSource {
    /// OS-native user cache directory from `dirs::cache_dir()`.
    OsCacheDir,
    /// Home directory fallback: `dirs::home_dir()/.flowmark-cache/flowmark`.
    HomeFallback,
    /// Temp directory fallback: `std::env::temp_dir()/flowmark-cache/flowmark`.
    TempFallback,
}

/// Resolved cache root path and the source used to derive it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRootResolution {
    /// Absolute or relative cache root path.
    pub path: PathBuf,
    /// Resolution source for path diagnostics.
    pub source: CacheRootSource,
}

/// Resolve the default cache root directory with source metadata.
///
/// Resolution order:
/// 1. `dirs::cache_dir()/flowmark`
/// 2. `dirs::home_dir()/.flowmark-cache/flowmark`
/// 3. `std::env::temp_dir()/flowmark-cache/flowmark`
pub fn resolve_default_cache_root() -> CacheRootResolution {
    let temp_dir = std::env::temp_dir();
    resolve_default_cache_root_with(dirs::cache_dir(), dirs::home_dir(), &temp_dir)
}

/// Resolve only the cache root path.
pub fn default_cache_root() -> PathBuf {
    resolve_default_cache_root().path
}

fn resolve_default_cache_root_with(
    os_cache: Option<PathBuf>,
    home: Option<PathBuf>,
    temp_dir: &Path,
) -> CacheRootResolution {
    if let Some(cache_dir) = os_cache {
        return CacheRootResolution {
            path: cache_dir.join(APP_CACHE_DIR),
            source: CacheRootSource::OsCacheDir,
        };
    }

    if let Some(home_dir) = home {
        return CacheRootResolution {
            path: home_dir.join(FALLBACK_CACHE_DIR).join(APP_CACHE_DIR),
            source: CacheRootSource::HomeFallback,
        };
    }

    CacheRootResolution {
        path: temp_dir.join(TEMP_FALLBACK_CACHE_DIR).join(APP_CACHE_DIR),
        source: CacheRootSource::TempFallback,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        APP_CACHE_DIR, CacheRootSource, FALLBACK_CACHE_DIR, TEMP_FALLBACK_CACHE_DIR,
        resolve_default_cache_root_with,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn default_cache_root_prefers_os_cache_dir() {
        let resolution = resolve_default_cache_root_with(
            Some(PathBuf::from("/os-cache")),
            Some(PathBuf::from("/home")),
            Path::new("/tmp"),
        );

        assert_eq!(resolution.source, CacheRootSource::OsCacheDir);
        assert_eq!(resolution.path, PathBuf::from("/os-cache").join(APP_CACHE_DIR));
    }

    #[test]
    fn default_cache_root_uses_home_fallback_when_os_cache_missing() {
        let resolution = resolve_default_cache_root_with(
            None,
            Some(PathBuf::from("/home/user")),
            Path::new("/tmp"),
        );

        assert_eq!(resolution.source, CacheRootSource::HomeFallback);
        assert_eq!(
            resolution.path,
            PathBuf::from("/home/user").join(FALLBACK_CACHE_DIR).join(APP_CACHE_DIR)
        );
    }

    #[test]
    fn default_cache_root_uses_temp_fallback_when_os_and_home_missing() {
        let resolution = resolve_default_cache_root_with(None, None, Path::new("/tmp"));

        assert_eq!(resolution.source, CacheRootSource::TempFallback);
        assert_eq!(
            resolution.path,
            PathBuf::from("/tmp").join(TEMP_FALLBACK_CACHE_DIR).join(APP_CACHE_DIR)
        );
    }
}
