//! Generated-input property checks for the preservation pipeline.
//!
//! These cover invariants that whole-document preservation must hold for *any* input,
//! including malformed and adversarial ones. Two are enforced on every run:
//!
//! - the formatter never aborts (preservation failures fall back to the normalized
//!   source rather than panicking), and
//! - output always satisfies the normalization contract.
//!
//! A third — that formatting is a fixed point in one pass — does not yet hold for
//! adversarial input in either port, so it runs on demand as a reproducer harness
//! rather than a gate. See `generated_documents_reach_a_fixed_point`.
//!
//! The generator is a fixed-seed LCG rather than a property-testing crate so the suite
//! stays dependency-free and every failure reproduces exactly from its seed.

use flowmark::ListSpacing;
use flowmark::formatter::filling::fill_markdown;

/// Fragments that stress the boundaries between protected constructs. Adjacent pairs
/// are where the pre-parse scanner and comrak can disagree about block structure.
const FRAGMENTS: &[&str] = &[
    "$",
    "$$",
    "`",
    "``",
    "```",
    "\\(",
    "\\)",
    "\\[",
    "\\]",
    "\\begin{align}",
    "\\end{align}",
    "|",
    ":::",
    "+++",
    "---",
    "> ",
    "- ",
    "1. ",
    "  ",
    "\n",
    "\n\n",
    "x",
    " ",
    "[!NOTE]",
    "[!WARNING]",
    "{#id}",
    "[[w]]",
    "{r}`c`",
    "<div>",
    "</div>",
    "<v>",
    ": def",
    "Term",
    "+---+",
    "+===+",
    "~~~",
    "\t",
    "\\$",
    "\\`",
    "\\\\",
    "é",
    "漢",
    "\u{feff}",
    "\u{f0000}",
    "\u{f0001}",
    "\u{f0002}",
];

/// Deterministic 64-bit LCG. Fixed constants keep every generated corpus reproducible
/// across platforms and runs.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 =
            self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() >> 33).expect("32-bit value fits usize") % bound
    }
}

fn generate(seed: u64, max_fragments: usize) -> String {
    let mut rng = Lcg(seed);
    let count = rng.below(max_fragments) + 1;
    (0..count).map(|_| FRAGMENTS[rng.below(FRAGMENTS.len())]).collect()
}

/// Mirrors `fill_markdown`'s own boolean flags one for one, so the field count follows
/// the public API rather than a design choice this test is free to make.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy)]
struct Mode {
    name: &'static str,
    width: usize,
    semantic: bool,
    cleanups: bool,
    smartquotes: bool,
    ellipses: bool,
}

const MODES: &[Mode] = &[
    Mode {
        name: "default",
        width: 88,
        semantic: false,
        cleanups: false,
        smartquotes: false,
        ellipses: false,
    },
    Mode {
        name: "semantic",
        width: 88,
        semantic: true,
        cleanups: false,
        smartquotes: false,
        ellipses: false,
    },
    Mode {
        name: "cleanups",
        width: 88,
        semantic: false,
        cleanups: true,
        smartquotes: false,
        ellipses: false,
    },
    Mode {
        name: "typography",
        width: 88,
        semantic: true,
        cleanups: true,
        smartquotes: true,
        ellipses: true,
    },
    Mode {
        name: "nowrap",
        width: 0,
        semantic: false,
        cleanups: false,
        smartquotes: false,
        ellipses: false,
    },
];

/// Format at an explicit width with every optional transform off.
///
/// The `MODES` table fixes each mode's width; list spacing only misbehaves when wrapping
/// actually splits a line, so these cases need to choose the width directly.
fn format_at_width(width: usize, input: &str) -> String {
    fill_markdown(input, false, width, false, false, false, false, None, ListSpacing::Preserve)
}

fn format_with(mode: Mode, input: &str) -> String {
    fill_markdown(
        input,
        false,
        mode.width,
        mode.semantic,
        mode.cleanups,
        mode.smartquotes,
        mode.ellipses,
        None,
        ListSpacing::Preserve,
    )
}

/// Formatting must never abort, whatever the input.
///
/// Preservation failures are required to fall back to the normalized source rather than
/// panic, so a panic here is a regression in the fail-soft contract that PR #81 review
/// R1 and R2 established. Reaching the assertion at all means every case survived.
#[test]
fn generated_documents_never_abort() {
    const CASES: u64 = 2_000;

    for seed in 0..CASES {
        let input = generate(seed, 24);
        for mode in MODES {
            let once = format_with(*mode, &input);
            let _ = format_with(*mode, &once);
        }
    }
}

/// Output must always satisfy the normalization contract: no carriage returns and
/// exactly one terminating newline. Valid UTF-8 is guaranteed by the return type.
#[test]
fn generated_documents_satisfy_the_output_normalization_contract() {
    const CASES: u64 = 2_000;

    for seed in 0..CASES {
        let input = generate(seed, 24);
        for mode in MODES {
            let output = format_with(*mode, &input);
            assert!(
                output.ends_with('\n') && !output.ends_with("\n\n"),
                "seed {seed} mode {} must end with exactly one LF: {output:?}",
                mode.name
            );
            assert!(
                !output.contains('\r'),
                "seed {seed} mode {} must not emit CR: {output:?}",
                mode.name
            );
        }
    }
}

/// Reproducer harness for the fixed-point property, run on demand:
/// `cargo test --test test_preservation_properties -- --ignored --nocapture`.
///
/// Formatting formatted output should be a no-op; where it is not, `--check` reports a
/// file the formatter itself just wrote. That guarantee does not yet hold for
/// adversarial input in either port — the surviving shapes are escape sequences in fence
/// info strings (fmr-c6xs / fm-ww33) and interior `U+FEFF` (fmr-uao3 / fm-jtwj), both of
/// which Python reproduces too, so closing them needs an agreed target in the
/// language-neutral manifest first.
///
/// This is ignored rather than deleted because it is how those shapes were found, and
/// how the next ones will be. It prints every failing seed with its exact input. Promote
/// it to a gate once the shared cases land.
#[test]
#[ignore = "reproducer harness: the fixed-point property has known shared gaps"]
fn generated_documents_reach_a_fixed_point() {
    const CASES: u64 = 2_000;
    let mut failures = Vec::new();

    for seed in 0..CASES {
        let input = generate(seed, 24);
        for mode in MODES {
            let once = format_with(*mode, &input);
            let twice = format_with(*mode, &once);
            if once != twice {
                failures.push(format!(
                    "seed {seed} mode {}\n  input: {input:?}\n  once:  {once:?}\n  twice: {twice:?}",
                    mode.name
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} generated cases are not a fixed point:\n{}",
        failures.len(),
        CASES * MODES.len() as u64,
        failures.join("\n")
    );
}

/// The specific shapes that broke in review must stay fixed: a callout marker abutting
/// each protected inline form used to abort (R1), and a false fence opener used to lose
/// its escape and grow on every pass (R10, fmr-sh2b).
#[test]
fn reviewed_regression_shapes_stay_fixed() {
    let shapes = [
        "> [!NOTE]<v>",
        "> [!NOTE]`x`",
        "> [!NOTE]$x$",
        "> [!NOTE][[w]]",
        "> [!WARNING]<b>t</b>",
        "```\\$`$",
        "```\\`",
        "```a\\`b",
        "```\n ",
    ];

    for shape in shapes {
        for mode in MODES {
            let once = format_with(*mode, shape);
            let twice = format_with(*mode, &once);
            assert_eq!(once, twice, "{shape:?} in mode {} must be a fixed point", mode.name);
        }
    }
}

/// The bridge's synthetic block boundary must not be observable in list spacing.
///
/// A wrapped list continuation beginning with `|` is a Pandoc line block, so the bridge
/// protects it and gives comrak a blank line where Python's parser simply breaks the
/// paragraph. That blank line loosened the whole list, and the renderer then spent it on
/// separation between items far from the token, which restoration cannot take back
/// (fmr-0pxh). The shared corpus pins the exact bytes as
/// `preservation.extension.line-block.wrapped-pipe-continuation`; this covers the
/// invariant behind it in both directions, which no single golden can.
#[test]
fn synthetic_block_boundaries_leave_list_spacing_as_authored() {
    let tight = concat!(
        "- Bead: fmr-hr43 | Scope: Phase 8.5/8.6 | Repo: playbook\n",
        "- Depends on: WI-1, WI-4\n",
        "- Findings: F1-F12 (12 lessons + anti-patterns)\n",
    );
    let wrapped = format_at_width(40, tight);
    assert_eq!(
        wrapped,
        concat!(
            "- Bead: fmr-hr43 | Scope: Phase 8.5/8.6\n",
            "  | Repo: playbook\n",
            "- Depends on: WI-1, WI-4\n",
            "- Findings: F1-F12 (12 lessons +\n",
            "  anti-patterns)\n",
        ),
        "wrapping must not loosen a tight list around a protected line block"
    );
    assert_eq!(format_at_width(40, &wrapped), wrapped, "the wrapped form must be a fixed point");

    // The repair must not reach a list the author really did write loose.
    let loose = concat!(
        "- Bead: fmr-hr43 | Scope: Phase 8.5/8.6\n",
        "  | Repo: playbook\n",
        "\n",
        "- Depends on: WI-1, WI-4\n",
        "\n",
        "- Findings: F1-F12\n",
    );
    assert_eq!(format_at_width(40, loose), loose, "an authored loose list must stay loose");
}

/// Pins a shape the fixed-point harness still reports.
///
/// `~~~\\[` loses one escape level per pass in flowmark-rs v0.3.2, on this branch, and
/// in Python flowmark alike, so it is a shared-contract bug rather than a Rust
/// regression. Tracked as fmr-c6xs (Rust) and fm-ww33 (Python).
///
/// Asserting the *current* shared behavior keeps the divergence visible and makes this
/// test fail the moment either port changes it. Replace with a fixed-point assertion
/// once the shared case lands.
#[test]
fn escaped_backslash_in_fence_info_string_is_a_known_shared_divergence() {
    let once = format_with(MODES[0], "~~~\\\\[");
    let twice = format_with(MODES[0], &once);

    assert_eq!(once, "~~~\\[\n~~~\n", "first pass changed; re-check fmr-c6xs");
    assert_eq!(twice, "~~~[\n~~~\n", "second pass changed; re-check fmr-c6xs");
    assert_ne!(once, twice, "fmr-c6xs looks fixed: promote this to a fixed-point assertion");
}

/// Pins the other shape the fixed-point harness still reports.
///
/// An interior `U+FEFF` before leading whitespace leaves a space that the next pass
/// removes, in Python and Rust alike. Tracked as fmr-uao3 (Rust) and fm-jtwj (Python);
/// see the sibling test above for why this is pinned rather than gated.
#[test]
fn interior_bom_with_leading_whitespace_is_a_known_shared_divergence() {
    let once = format_with(MODES[0], " \u{feff}\t\\(");
    let twice = format_with(MODES[0], &once);

    assert_eq!(once, " \\(\n", "first pass changed; re-check fmr-uao3");
    assert_eq!(twice, "\\(\n", "second pass changed; re-check fmr-uao3");
    assert_ne!(once, twice, "fmr-uao3 looks fixed: promote this to a fixed-point assertion");
}
