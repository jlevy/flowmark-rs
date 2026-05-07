//! Tests for build-time version metadata derivation.

#[path = "../src/version_meta.rs"]
mod version_meta;

use version_meta::compute_version_metadata;

#[test]
fn release_tag_override_forces_stable_version_output() {
    let metadata = compute_version_metadata(
        "0.2.6",
        "0.6.5",
        Some("v0.2.6"),
        Some("v0.2.5"),
        Some("3"),
        Some("abcdef0"),
    );

    assert_eq!(metadata.base_tag, "v0.2.6");
    assert_eq!(metadata.commits_ahead, "0");
    assert_eq!(metadata.git_hash, "abcdef0");
    assert_eq!(metadata.long_version, "0.2.6 (Rust port of flowmark-py 0.6.5; base v0.2.6)");
}

#[test]
fn no_git_metadata_defaults_to_stable_version_output() {
    let metadata = compute_version_metadata("0.2.6", "0.6.5", None, None, None, None);

    assert_eq!(metadata.base_tag, "v0.2.6");
    assert_eq!(metadata.commits_ahead, "unknown");
    assert_eq!(metadata.git_hash, "unknown");
    assert_eq!(metadata.long_version, "0.2.6 (Rust port of flowmark-py 0.6.5; base v0.2.6)");
}

#[test]
fn commits_ahead_produces_dev_suffix() {
    let metadata = compute_version_metadata(
        "0.2.6",
        "0.6.5",
        None,
        Some("v0.2.5"),
        Some("4"),
        Some("abcdef0"),
    );

    assert_eq!(metadata.base_tag, "v0.2.5");
    assert_eq!(metadata.commits_ahead, "4");
    assert_eq!(metadata.git_hash, "abcdef0");
    assert_eq!(
        metadata.long_version,
        "0.2.6-dev.4+gabcdef0 (Rust port of flowmark-py 0.6.5; base v0.2.5)"
    );
}
