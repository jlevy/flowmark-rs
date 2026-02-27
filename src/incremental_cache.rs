//! Persistent incremental cache for unchanged-file fast paths.

use crate::config::FormatOptions;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const INCREMENTAL_DIR_NAME: &str = "incremental";
const MANIFEST_FORMAT_VERSION: i64 = 1;
const TOML_KEY_VERSION: &str = "version";
const TOML_KEY_FINGERPRINT: &str = "fingerprint";
const TOML_KEY_HASHES: &str = "hashes";
const HEX_RADIX: u32 = 16;
const HASH_FILE_EXTENSION: &str = "toml";

#[derive(Debug, Clone)]
struct CacheManifest {
    formatter_fingerprint: u64,
    hashes: HashSet<u64>,
}

/// Incremental cache for CLI formatting runs.
///
/// Hash entries represent formatted file content for a single project + formatter
/// fingerprint. A cache hit means the current file text is already known-formatted.
#[derive(Debug)]
pub struct IncrementalCache {
    manifest_path: PathBuf,
    formatter_fingerprint: u64,
    read_hashes: HashSet<u64>,
    write_hashes: Mutex<HashSet<u64>>,
}

impl IncrementalCache {
    /// Open (or initialize) an incremental cache for a project root.
    ///
    /// The cache file path is derived from the project root hash under:
    /// `<cache_dir>/incremental/<project-hash>.toml`.
    pub fn open(
        cache_dir: &Path,
        project_root: &Path,
        formatter_fingerprint: u64,
    ) -> std::io::Result<Self> {
        let incremental_dir = cache_dir.join(INCREMENTAL_DIR_NAME);
        fs::create_dir_all(&incremental_dir)?;

        let canonical_project_root =
            project_root.canonicalize().unwrap_or_else(|_| project_root.to_path_buf());
        let project_hash = hash_string(&canonical_project_root.to_string_lossy());
        let manifest_name = format!("{project_hash:016x}.{HASH_FILE_EXTENSION}");
        let manifest_path = incremental_dir.join(manifest_name);

        let read_hashes = match load_manifest(&manifest_path) {
            Some(manifest) if manifest.formatter_fingerprint == formatter_fingerprint => {
                manifest.hashes
            }
            _ => HashSet::new(),
        };

        Ok(Self {
            manifest_path,
            formatter_fingerprint,
            read_hashes,
            write_hashes: Mutex::new(HashSet::new()),
        })
    }

    /// Returns true when the input bytes are already known-formatted.
    ///
    /// On hit, this also records the hash in the current write set so a later
    /// flush preserves warm entries.
    pub fn is_known_formatted(&self, _path: &Path, input_bytes: &[u8]) -> bool {
        let hash = hash_bytes(input_bytes);
        if self.read_hashes.contains(&hash) {
            self.add_write_hash(hash);
            true
        } else {
            false
        }
    }

    /// Record formatted output bytes in the current write set.
    pub fn record_formatted(&self, _path: &Path, formatted_bytes: &[u8]) {
        let hash = hash_bytes(formatted_bytes);
        self.add_write_hash(hash);
    }

    /// Persist cache state atomically.
    pub fn flush(&self) -> std::io::Result<()> {
        let mut hashes = self.read_hashes.clone();
        if let Ok(write_hashes) = self.write_hashes.lock() {
            hashes.extend(write_hashes.iter().copied());
        }

        let manifest = CacheManifest { formatter_fingerprint: self.formatter_fingerprint, hashes };
        save_manifest_atomic(&self.manifest_path, &manifest)
    }

    #[cfg(test)]
    fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    fn add_write_hash(&self, hash: u64) {
        if let Ok(mut write_hashes) = self.write_hashes.lock() {
            write_hashes.insert(hash);
        }
    }
}

/// Compute a formatter fingerprint used for cache invalidation.
///
/// Fingerprint input includes:
/// - binary version
/// - formatting options
/// - config file path + bytes when available
pub fn compute_formatter_fingerprint(
    opts: &FormatOptions,
    binary_version: &str,
    config_path: Option<&Path>,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    binary_version.hash(&mut hasher);
    opts.width.hash(&mut hasher);
    opts.plaintext.hash(&mut hasher);
    opts.semantic.hash(&mut hasher);
    opts.cleanups.hash(&mut hasher);
    opts.smartquotes.hash(&mut hasher);
    opts.ellipses.hash(&mut hasher);
    opts.list_spacing.to_string().hash(&mut hasher);

    if let Some(path) = config_path {
        path.to_string_lossy().hash(&mut hasher);
        if let Ok(bytes) = fs::read(path) {
            bytes.hash(&mut hasher);
        }
    }

    hasher.finish()
}

fn load_manifest(path: &Path) -> Option<CacheManifest> {
    let text = fs::read_to_string(path).ok()?;
    let value = toml::from_str::<toml::Value>(&text).ok()?;
    let table = value.as_table()?;

    let version = table.get(TOML_KEY_VERSION)?.as_integer()?;
    if version != MANIFEST_FORMAT_VERSION {
        return None;
    }

    let fingerprint = table.get(TOML_KEY_FINGERPRINT)?.as_str()?;
    let formatter_fingerprint = parse_hash_hex(fingerprint)?;

    let hashes = table
        .get(TOML_KEY_HASHES)?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str())
        .filter_map(parse_hash_hex)
        .collect::<HashSet<_>>();

    Some(CacheManifest { formatter_fingerprint, hashes })
}

fn save_manifest_atomic(path: &Path, manifest: &CacheManifest) -> std::io::Result<()> {
    use std::io::Write;

    let mut hashes = manifest.hashes.iter().map(|hash| format!("{hash:016x}")).collect::<Vec<_>>();
    hashes.sort_unstable();

    let hashes_values = hashes.into_iter().map(toml::Value::String).collect::<Vec<_>>();

    let mut table = toml::map::Map::new();
    table.insert(TOML_KEY_VERSION.to_string(), toml::Value::Integer(MANIFEST_FORMAT_VERSION));
    table.insert(
        TOML_KEY_FINGERPRINT.to_string(),
        toml::Value::String(format!("{:016x}", manifest.formatter_fingerprint)),
    );
    table.insert(TOML_KEY_HASHES.to_string(), toml::Value::Array(hashes_values));

    let content = toml::to_string(&toml::Value::Table(table))
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut temp_file = tempfile::NamedTempFile::new_in(parent)?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.persist(path).map_err(|error| error.error)?;
    Ok(())
}

fn parse_hash_hex(value: &str) -> Option<u64> {
    u64::from_str_radix(value, HEX_RADIX).ok()
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn hash_string(value: &str) -> u64 {
    hash_bytes(value.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::{IncrementalCache, compute_formatter_fingerprint};
    use crate::config::{FormatOptions, ListSpacing};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn cache_round_trip_restores_known_hashes() {
        let cache_root = tempdir().expect("create cache root");
        let project_root = tempdir().expect("create project root");
        let fingerprint = 42_u64;

        let cache = IncrementalCache::open(cache_root.path(), project_root.path(), fingerprint)
            .expect("open cache");
        cache.record_formatted(Path::new("a.md"), b"# Hello\n");
        cache.flush().expect("flush cache");

        let reopened = IncrementalCache::open(cache_root.path(), project_root.path(), fingerprint)
            .expect("re-open cache");
        assert!(reopened.is_known_formatted(Path::new("a.md"), b"# Hello\n"));
    }

    #[test]
    fn cache_invalidates_when_fingerprint_changes() {
        let cache_root = tempdir().expect("create cache root");
        let project_root = tempdir().expect("create project root");

        let cache = IncrementalCache::open(cache_root.path(), project_root.path(), 100)
            .expect("open cache");
        cache.record_formatted(Path::new("a.md"), b"# Hello\n");
        cache.flush().expect("flush cache");

        let reopened = IncrementalCache::open(cache_root.path(), project_root.path(), 200)
            .expect("re-open cache");
        assert!(!reopened.is_known_formatted(Path::new("a.md"), b"# Hello\n"));
    }

    #[test]
    fn cache_ignores_corrupt_manifest_and_recovers() {
        let cache_root = tempdir().expect("create cache root");
        let project_root = tempdir().expect("create project root");
        let fingerprint = 300_u64;

        let cache = IncrementalCache::open(cache_root.path(), project_root.path(), fingerprint)
            .expect("open cache");
        let manifest_path = cache.manifest_path().to_path_buf();
        if let Some(parent) = manifest_path.parent() {
            fs::create_dir_all(parent).expect("create manifest dir");
        }
        fs::write(&manifest_path, "{ definitely = not_toml").expect("write corrupt manifest");

        let reopened = IncrementalCache::open(cache_root.path(), project_root.path(), fingerprint)
            .expect("re-open cache");
        assert!(!reopened.is_known_formatted(Path::new("a.md"), b"# Hello\n"));

        reopened.record_formatted(Path::new("a.md"), b"# Hello\n");
        reopened.flush().expect("flush repaired cache");

        let reloaded = IncrementalCache::open(cache_root.path(), project_root.path(), fingerprint)
            .expect("re-load repaired cache");
        assert!(reloaded.is_known_formatted(Path::new("a.md"), b"# Hello\n"));
    }

    #[test]
    fn cache_is_project_scoped() {
        let cache_root = tempdir().expect("create cache root");
        let project_one = tempdir().expect("create project 1");
        let project_two = tempdir().expect("create project 2");
        let fingerprint = 7_u64;

        let cache_one = IncrementalCache::open(cache_root.path(), project_one.path(), fingerprint)
            .expect("open cache 1");
        cache_one.record_formatted(Path::new("a.md"), b"# Hello\n");
        cache_one.flush().expect("flush cache 1");

        let cache_two = IncrementalCache::open(cache_root.path(), project_two.path(), fingerprint)
            .expect("open cache 2");
        assert!(!cache_two.is_known_formatted(Path::new("a.md"), b"# Hello\n"));
    }

    #[test]
    fn formatter_fingerprint_changes_with_config_content() {
        let config_dir = tempdir().expect("create config dir");
        let config_path = config_dir.path().join("flowmark.toml");
        fs::write(&config_path, "width = 88\n").expect("write config");

        let opts = FormatOptions {
            width: 88,
            plaintext: false,
            semantic: true,
            cleanups: true,
            smartquotes: true,
            ellipses: true,
            list_spacing: ListSpacing::Preserve,
        };

        let first = compute_formatter_fingerprint(&opts, "0.0.0", Some(&config_path));
        fs::write(&config_path, "width = 90\n").expect("write changed config");
        let second = compute_formatter_fingerprint(&opts, "0.0.0", Some(&config_path));

        assert_ne!(first, second);
    }
}
