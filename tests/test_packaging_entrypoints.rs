//! Packaging entrypoint test: assert both `flowmark` and `flowmark-rs` binaries
//! point to the same source path with the same required features.
//!
//! Ported from Python: `tests/test_packaging_entrypoints.py` (v0.6.5, commit `d866c08`).
//! Python's test asserts `[project.scripts].flowmark` and `flowmark-py` both resolve
//! to `flowmark.cli:main`. The Rust analog asserts the two `[[bin]]` entries in
//! Cargo.toml share `path` and `required-features`.

use std::path::PathBuf;

#[test]
fn test_flowmark_rs_alias_entrypoint() {
    let cargo_toml = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&cargo_toml).expect("read Cargo.toml");

    let parsed: toml::Value = toml::from_str(&text).expect("parse Cargo.toml as TOML");
    let bins =
        parsed.get("bin").expect("missing [[bin]] entries").as_array().expect("[[bin]] is array");

    let mut flowmark_entry: Option<&toml::Value> = None;
    let mut flowmark_rs_entry: Option<&toml::Value> = None;
    for bin in bins {
        let name = bin.get("name").and_then(|v| v.as_str()).expect("[[bin]].name");
        match name {
            "flowmark" => flowmark_entry = Some(bin),
            "flowmark-rs" => flowmark_rs_entry = Some(bin),
            _ => {}
        }
    }

    let flowmark = flowmark_entry.expect("missing [[bin]] flowmark");
    let flowmark_rs = flowmark_rs_entry.expect("missing [[bin]] flowmark-rs");

    let path_for = |bin: &toml::Value| {
        bin.get("path").and_then(|v| v.as_str()).expect("[[bin]].path").to_string()
    };
    let features_for = |bin: &toml::Value| {
        bin.get("required-features")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|x| x.as_str().map(str::to_string)).collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    assert_eq!(
        path_for(flowmark),
        path_for(flowmark_rs),
        "flowmark and flowmark-rs binaries must point to the same source path"
    );
    assert_eq!(
        features_for(flowmark),
        features_for(flowmark_rs),
        "flowmark and flowmark-rs binaries must require the same features"
    );
}
