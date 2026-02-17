use std::path::Path;
use flowmark::fill_markdown;
use flowmark::config::ListSpacing;

#[test]
fn test_reference_doc_formats() {
    let testdoc_dir = Path::new("tests/testdocs");
    let orig_path = testdoc_dir.join("testdoc.orig.md");

    assert!(orig_path.exists(), "Original test document not found at {:?}", orig_path);

    let orig_content = std::fs::read_to_string(&orig_path).expect("Failed to read original document");

    struct TestCase {
        name: &'static str,
        filename: &'static str,
        semantic: bool,
        cleanups: bool,
        smartquotes: bool,
        ellipses: bool,
    }

    let test_cases = [
        TestCase {
            name: "plain",
            filename: "testdoc.expected.plain.md",
            semantic: false,
            cleanups: false,
            smartquotes: false,
            ellipses: false,
        },
        TestCase {
            name: "semantic",
            filename: "testdoc.expected.semantic.md",
            semantic: true,
            cleanups: false,
            smartquotes: false,
            ellipses: false,
        },
        TestCase {
            name: "cleaned",
            filename: "testdoc.expected.cleaned.md",
            semantic: true,
            cleanups: true,
            smartquotes: false,
            ellipses: false,
        },
        TestCase {
            name: "auto",
            filename: "testdoc.expected.auto.md",
            semantic: true,
            cleanups: true,
            smartquotes: true,
            ellipses: true,
        },
    ];

    let mut all_pass = true;
    for case in &test_cases {
        let test_doc = testdoc_dir.join(case.filename);
        let expected = std::fs::read_to_string(&test_doc)
            .unwrap_or_else(|_| panic!("Failed to read expected document: {}", case.filename));

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

        if actual != expected {
            let actual_path = testdoc_dir.join(format!("testdoc.actual.{}.md", case.name));
            std::fs::write(&actual_path, &actual)
                .unwrap_or_else(|_| panic!("Failed to write actual output for {}", case.name));
            eprintln!("actual was different from expected for {}!", case.name);
            eprintln!("Saving actual to: {:?}", actual_path);

            // Show first difference
            let expected_lines: Vec<&str> = expected.lines().collect();
            let actual_lines: Vec<&str> = actual.lines().collect();
            for (i, (exp, act)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
                if exp != act {
                    eprintln!("First difference at line {} for {}:", i + 1, case.name);
                    eprintln!("  Expected: {:?}", exp);
                    eprintln!("  Actual:   {:?}", act);
                    break;
                }
            }
            if expected_lines.len() != actual_lines.len() {
                eprintln!(
                    "Line count differs for {}: expected {}, got {}",
                    case.name,
                    expected_lines.len(),
                    actual_lines.len()
                );
            }

            all_pass = false;
        }
    }

    assert!(all_pass, "One or more reference document format tests failed");
}
