use flowmark::config::ListSpacing;
use flowmark::fill_markdown;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use toml::Value;

#[allow(clippy::struct_excessive_bools)]
struct TestCase {
    id: &'static str,
    filename: &'static str,
    semantic: bool,
    cleanups: bool,
    smartquotes: bool,
    ellipses: bool,
}

fn upstream_testdoc_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("repos/flowmark/tests/testdocs")
}

fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

fn known_divergence_ids() -> BTreeSet<String> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/parity_corpus_known_divergences.toml");
    let document = toml::from_str::<Value>(&read_text(&path))
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()));
    document
        .get("divergence")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{} has no divergence array", path.display()))
        .iter()
        .map(|entry| {
            entry
                .get("case_id")
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("{} has a divergence without a case_id", path.display()))
                .to_owned()
        })
        .collect()
}

#[test]
fn reference_documents_are_read_directly_from_the_pinned_upstream() {
    let testdoc_dir = upstream_testdoc_dir();
    let orig_content = read_text(&testdoc_dir.join("testdoc.orig.md"));
    let known = known_divergence_ids();
    let test_cases = [
        TestCase {
            id: "reference.testdoc.plain",
            filename: "testdoc.expected.plain.md",
            semantic: false,
            cleanups: false,
            smartquotes: false,
            ellipses: false,
        },
        TestCase {
            id: "reference.testdoc.semantic",
            filename: "testdoc.expected.semantic.md",
            semantic: true,
            cleanups: false,
            smartquotes: false,
            ellipses: false,
        },
        TestCase {
            id: "reference.testdoc.cleaned",
            filename: "testdoc.expected.cleaned.md",
            semantic: true,
            cleanups: true,
            smartquotes: false,
            ellipses: false,
        },
        TestCase {
            id: "reference.testdoc.auto",
            filename: "testdoc.expected.auto.md",
            semantic: true,
            cleanups: true,
            smartquotes: true,
            ellipses: true,
        },
    ];

    for case in test_cases {
        let expected = read_text(&testdoc_dir.join(case.filename));
        let actual = fill_markdown(
            &orig_content,
            true,
            88,
            case.semantic,
            case.cleanups,
            case.smartquotes,
            case.ellipses,
            None,
            ListSpacing::Preserve,
        );
        assert!(
            actual == expected || known.contains(case.id),
            "{} differs from its pinned upstream output without a ledger entry",
            case.id
        );
    }
}
