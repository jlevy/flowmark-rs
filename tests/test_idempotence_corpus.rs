//! Corpus-wide idempotence gate.
//!
//! Formatting is meant to reach a fixed point in one pass: `format(format(x)) ==
//! format(x)`. Where it does not, `flowmark --check` reports files flowmark itself just
//! wrote, and repeated runs rewrite authored content.
//!
//! The shared conformance corpus already verifies this per case, but each case pins one
//! CLI invocation, so the option space is sampled one point per document. This walks
//! every Markdown document the project ships across a mode matrix instead, which is what
//! surfaces width-dependent instability.
//!
//! Known failures live in `tests/idempotence_known_divergences.toml` and are asserted
//! exactly: an unlisted failure fails the build, and a listed entry that now passes also
//! fails it, so the ledger shrinks and cannot rot.
//!
//! See `docs/project/specs/active/plan-2026-08-27-idempotence-verification.md`.

use flowmark::{FormatOptions, ListSpacing};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Modes covering the option space's independent axes rather than its combinations.
/// Width earns two entries: instability that only appears when wrapping pushes a
/// hazardous token to the start of a line is invisible at the default width.
fn modes() -> Vec<(&'static str, FormatOptions)> {
    let base = |width: usize| FormatOptions {
        width,
        plaintext: false,
        semantic: false,
        cleanups: false,
        smartquotes: false,
        ellipses: false,
        list_spacing: ListSpacing::Preserve,
    };
    vec![
        ("default", base(88)),
        ("semantic", FormatOptions { semantic: true, ..base(88) }),
        ("cleanups", FormatOptions { cleanups: true, ..base(88) }),
        (
            "typography",
            FormatOptions {
                semantic: true,
                cleanups: true,
                smartquotes: true,
                ellipses: true,
                ..base(88)
            },
        ),
        ("nowrap", base(0)),
        ("narrow", base(40)),
    ]
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_markdown(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
}

/// Every corpus the project ships, plus its own docs. The repository's docs are the only
/// genuinely human-authored prose in the set, and a doc flowmark cannot format stably is
/// itself a defect.
fn corpus_documents() -> Vec<PathBuf> {
    let root = project_root();
    let upstream = root.join("repos/flowmark/tests");
    assert!(
        upstream.is_dir(),
        "shared corpus missing at {}; run `git submodule update --init --recursive`",
        upstream.display()
    );

    let mut documents = Vec::new();
    collect_markdown(&upstream, &mut documents);
    collect_markdown(&root.join("docs"), &mut documents);
    let readme = root.join("README.md");
    if readme.is_file() {
        documents.push(readme);
    }
    documents.sort();
    assert!(
        documents.len() > 1_000,
        "expected the full shared corpus, found only {} documents",
        documents.len()
    );
    documents
}

/// A ledger entry names one document and one mode, keyed as `relative/path::mode`.
fn load_ledger() -> BTreeSet<String> {
    let path = project_root().join("tests/idempotence_known_divergences.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let value = text
        .parse::<toml::Table>()
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()));
    value
        .get("divergence")
        .and_then(toml::Value::as_array)
        .expect("ledger must define a divergence array")
        .iter()
        .map(|entry| {
            let document = entry.get("document").and_then(toml::Value::as_str);
            let mode = entry.get("mode").and_then(toml::Value::as_str);
            let (document, mode) =
                document.zip(mode).expect("each divergence needs a document and a mode");
            format!("{document}::{mode}")
        })
        .collect()
}

/// Formatting must reach a fixed point in one pass for every shipped document in every
/// mode, except the exact set the ledger names.
#[test]
fn every_corpus_document_reaches_a_fixed_point() {
    let root = project_root();
    let ledger = load_ledger();
    let modes = modes();
    let mut observed = BTreeSet::new();
    let mut checks = 0_usize;

    for document in corpus_documents() {
        let Ok(source) = std::fs::read_to_string(&document) else {
            // Deliberately invalid UTF-8 fixtures are not Markdown documents.
            continue;
        };
        let relative = document.strip_prefix(&root).unwrap_or(&document).display().to_string();
        for (name, options) in &modes {
            checks += 1;
            let once = options.reformat_text(&source);
            if options.reformat_text(&once) != once {
                observed.insert(format!("{relative}::{name}"));
            }
        }
    }

    let unexpected: Vec<_> = observed.difference(&ledger).cloned().collect();
    let stale: Vec<_> = ledger.difference(&observed).cloned().collect();

    assert!(
        unexpected.is_empty(),
        "{} of {checks} checks are newly not a fixed point.\n\
         Add a ledger entry only with a tracking bead, or fix the defect:\n  {}",
        unexpected.len(),
        unexpected.join("\n  ")
    );
    assert!(
        stale.is_empty(),
        "{} ledger entries now pass and must be removed:\n  {}",
        stale.len(),
        stale.join("\n  ")
    );
}
