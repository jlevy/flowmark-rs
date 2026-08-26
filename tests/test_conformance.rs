//! Run the built Rust CLI against the upstream language-neutral conformance corpus.
#![cfg(feature = "cli")]

#[path = "support/conformance.rs"]
mod conformance;

use conformance::{load_known_divergences, load_manifest, run_case, select_cases, upstream_root};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use toml::Value;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_flowmark"))
}

#[test]
fn shared_traceability_matches_the_pinned_upstream() {
    let root = project_root();
    let upstream = upstream_root(&root);
    let traceability_path = root.join("admin/port-coverage-mapping/shared-conformance.toml");
    let traceability_text = std::fs::read_to_string(&traceability_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", traceability_path.display()));
    let traceability = toml::from_str::<Value>(&traceability_text)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", traceability_path.display()));
    assert_eq!(traceability.get("schema_version").and_then(Value::as_integer), Some(1));

    let recorded_commit = fixture_value_string(&traceability, "upstream_commit");
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&upstream)
        .output()
        .expect("git is required to validate the pinned upstream submodule");
    assert!(output.status.success(), "cannot resolve the pinned upstream commit");
    let actual_commit =
        String::from_utf8(output.stdout).expect("git commit must be UTF-8").trim().to_owned();
    assert_eq!(recorded_commit, actual_commit, "shared traceability has a stale commit");

    let manifest_path = upstream.join(fixture_value_string(&traceability, "manifest"));
    let manifest = load_manifest(&manifest_path, &upstream)
        .unwrap_or_else(|error| panic!("cannot load shared conformance manifest: {error}"));
    let manifest_change_ids: BTreeSet<&str> =
        manifest.cases.iter().map(|case| case.change_id.as_str()).collect();
    let recorded_change_ids: BTreeSet<&str> = traceability
        .get("change")
        .and_then(Value::as_array)
        .expect("shared traceability must define change entries")
        .iter()
        .map(|entry| fixture_value_string(entry, "id"))
        .collect();
    assert_eq!(recorded_change_ids, manifest_change_ids, "shared change-ID mapping is stale");
}

#[test]
fn shared_conformance_corpus_matches_or_has_a_current_divergence() {
    let root = project_root();
    let upstream = upstream_root(&root);
    let manifest = load_manifest(&upstream.join("tests/parity_corpus/manifest.toml"), &upstream)
        .unwrap_or_else(|error| panic!("cannot load shared conformance manifest: {error}"));
    assert_eq!(manifest.corpus, "flowmark-language-neutral-conformance");
    let cases = select_cases(&manifest, &[], &[], &[])
        .unwrap_or_else(|error| panic!("cannot select shared conformance cases: {error}"));
    let divergences =
        load_known_divergences(&root.join("tests/parity_corpus_known_divergences.toml"))
            .unwrap_or_else(|error| panic!("cannot load known-divergence ledger: {error}"));
    let known: BTreeMap<&str, _> =
        divergences.iter().map(|entry| (entry.case_id.as_str(), entry)).collect();
    let active_ids: BTreeSet<&str> = cases.iter().map(|case| case.id.as_str()).collect();
    let mut failures = Vec::new();
    for divergence in &divergences {
        if !active_ids.contains(divergence.case_id.as_str()) {
            failures.push(format!(
                "ledger case {:?} is missing or deferred (tracker {}): {}",
                divergence.case_id, divergence.tracker, divergence.reason
            ));
        }
    }

    let mut passed = 0_usize;
    let mut known_failures = 0_usize;
    let mut change_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for case in cases {
        *change_counts.entry(case.change_id.as_str()).or_default() += 1;
        match (
            run_case(case, &manifest, &rust_binary(), &upstream, Duration::from_secs(30)),
            known.get(case.id.as_str()),
        ) {
            (Ok(pass_count), None) => {
                passed += 1;
                println!("PASS {} ({pass_count} passes)", case.id);
            }
            (Ok(_), Some(divergence)) => failures.push(format!(
                "stale divergence {:?} now passes (tracker {}): {}",
                case.id, divergence.tracker, divergence.reason
            )),
            (Err(error), Some(divergence)) => {
                known_failures += 1;
                println!(
                    "KNOWN-DIVERGENCE {} [{}; {}] {}: {}",
                    case.id, divergence.tracker, divergence.reason, case.description, error
                );
            }
            (Err(error), None) => failures.push(format!("{}: {error}", case.description)),
        }
    }
    println!(
        "shared conformance: {passed} pass, {known_failures} known divergence; change IDs {change_counts:?}"
    );
    assert!(failures.is_empty(), "shared conformance failures:\n{}", failures.join("\n\n"));
}

#[test]
fn shared_runner_fixtures_have_the_same_stable_error_codes() {
    let root = project_root();
    let upstream = upstream_root(&root);
    let index_path = upstream.join("tests/parity_corpus/runner-fixtures/manifest.toml");
    let index_text = std::fs::read_to_string(&index_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", index_path.display()));
    let index = toml::from_str::<Value>(&index_text)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", index_path.display()));
    let fixtures = index
        .get("fixture")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{} has no fixture array", index_path.display()));
    assert!(!fixtures.is_empty(), "shared runner fixture list is empty");

    for fixture in fixtures {
        let table = fixture
            .as_table()
            .unwrap_or_else(|| panic!("runner fixture must be a table: {fixture:?}"));
        let id = fixture_string(table, "id");
        let manifest_path = upstream.join(fixture_string(table, "manifest"));
        let outcome = fixture_string(table, "outcome");
        let expected_code = fixture_string(table, "code");
        let actual_code = if outcome == "manifest-error" {
            load_manifest(&manifest_path, &upstream)
                .expect_err("malformed runner fixture unexpectedly validated")
                .code
        } else if outcome == "case-failure" {
            let manifest = load_manifest(&manifest_path, &upstream)
                .unwrap_or_else(|error| panic!("fixture {id} did not validate: {error}"));
            run_case(
                &manifest.cases[0],
                &manifest,
                &rust_binary(),
                &upstream,
                Duration::from_secs(30),
            )
            .expect_err("intentional failure fixture unexpectedly passed")
            .code
        } else {
            panic!("fixture {id} has unknown outcome {outcome:?}");
        };
        assert_eq!(actual_code, expected_code, "runner fixture {id}");
    }
}

fn fixture_string<'a>(table: &'a toml::map::Map<String, Value>, field: &str) -> &'a str {
    table
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("runner fixture field {field:?} must be a string"))
}

fn fixture_value_string<'a>(value: &'a Value, field: &str) -> &'a str {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("field {field:?} must be a string"))
}

#[test]
fn exact_selection_can_run_a_deferred_case() {
    let upstream = upstream_root(&project_root());
    let manifest = load_manifest(&upstream.join("tests/parity_corpus/manifest.toml"), &upstream)
        .unwrap_or_else(|error| panic!("cannot load shared conformance manifest: {error}"));
    let selected = select_cases(&manifest, &["commonmark.default.0017"], &[], &[])
        .unwrap_or_else(|error| panic!("cannot select a deferred case explicitly: {error}"));
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id, "commonmark.default.0017");
}

#[cfg(unix)]
#[test]
fn runner_timeout_is_bounded_and_has_a_stable_code() {
    use std::os::unix::fs::PermissionsExt;

    let upstream = upstream_root(&project_root());
    let manifest = load_manifest(&upstream.join("tests/parity_corpus/manifest.toml"), &upstream)
        .unwrap_or_else(|error| panic!("cannot load shared conformance manifest: {error}"));
    let case = select_cases(&manifest, &["cli.stdin.wrap"], &[], &[])
        .unwrap_or_else(|error| panic!("cannot select timeout seed case: {error}"))[0];
    let temporary = tempfile::tempdir().expect("create timeout test directory");
    let executable = temporary.path().join("slow-flowmark");
    std::fs::write(&executable, b"#!/bin/sh\nsleep 5\n").expect("write timeout test executable");
    let mut permissions =
        std::fs::metadata(&executable).expect("read timeout executable metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&executable, permissions)
        .expect("make timeout test executable runnable");

    let error =
        run_case(case, &manifest, Path::new(&executable), &upstream, Duration::from_millis(50))
            .expect_err("slow executable unexpectedly completed");
    assert_eq!(error.code, "timeout");
    assert!(error.to_string().contains("cli.stdin.wrap"));
    assert!(error.to_string().len() <= 8_300);
}
