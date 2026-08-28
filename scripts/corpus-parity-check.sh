#!/bin/bash
# Corpus parity check: run both the pinned Python flowmark and Rust flowmark on a directory
# of markdown files and report any differences.
#
# Usage: ./scripts/corpus-parity-check.sh <corpus_dir> [rust_binary]
# Default rust_binary: target/release/flowmark
#
# Environment:
#   FLOWMARK_PARITY_PYTHON_BIN   Exact Python flowmark executable to audit. When unset,
#                                use the released version pinned in Cargo.toml via uvx.
#   FLOWMARK_PARITY_PYTHON_LABEL Immutable commit/version label for an executable override.
#   FLOWMARK_PARITY_REPORT_DIR   Persistent report directory. Defaults below target/.
#   FLOWMARK_PARITY_EXPECTED_CORPUS_SHA256
#                                Optional pinned corpus digest; mismatch aborts the audit.
#   FLOWMARK_PARITY_KEEP_TMP      Set to 1 to retain the formatted Python and Rust trees
#                                and record their temporary workspace in the report.
#
# Exit code: 0 = full parity, 1 = differences found, 2 = invalid audit setup.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <corpus_dir> [rust_binary]"
    echo "The corpus must be explicit so its origin and digest can be recorded."
    exit 2
fi

CORPUS_DIR="$1"
RUST_BIN="${2:-target/release/flowmark}"
# Read Python parity version from the single source of truth in Cargo.toml
PYTHON_VERSION=$(grep -A1 '\[package.metadata.parity\]' "$REPO_ROOT/Cargo.toml" | grep version | sed 's/.*"\(.*\)"/\1/')
PYTHON_BIN="${FLOWMARK_PARITY_PYTHON_BIN:-}"
PYTHON_LABEL="${FLOWMARK_PARITY_PYTHON_LABEL:-}"
REPORT_DIR="${FLOWMARK_PARITY_REPORT_DIR:-$REPO_ROOT/target/corpus-parity/$(date -u +%Y%m%dT%H%M%SZ)}"
EXPECTED_CORPUS_DIGEST="${FLOWMARK_PARITY_EXPECTED_CORPUS_SHA256:-}"
KEEP_TMP="${FLOWMARK_PARITY_KEEP_TMP:-0}"

if [ "$KEEP_TMP" != "0" ] && [ "$KEEP_TMP" != "1" ]; then
    echo "ERROR: FLOWMARK_PARITY_KEEP_TMP must be 0 or 1"
    exit 2
fi

if [ ! -d "$CORPUS_DIR" ]; then
    echo "ERROR: Corpus directory not found: $CORPUS_DIR"
    exit 2
fi

if [ ! -x "$RUST_BIN" ]; then
    echo "ERROR: Rust binary not found: $RUST_BIN"
    echo "Build with: cargo build --release"
    exit 2
fi

if [ -n "$PYTHON_BIN" ]; then
    if [ ! -x "$PYTHON_BIN" ]; then
        echo "ERROR: Python flowmark executable not found: $PYTHON_BIN"
        exit 2
    fi
    if [ -z "$PYTHON_LABEL" ]; then
        echo "ERROR: FLOWMARK_PARITY_PYTHON_LABEL is required with FLOWMARK_PARITY_PYTHON_BIN"
        echo "Use an immutable commit or release label, not a branch name."
        exit 2
    fi
    PYTHON_COMMAND=("$PYTHON_BIN")
else
    PYTHON_COMMAND=(uvx "flowmark@${PYTHON_VERSION}")
    PYTHON_LABEL="flowmark v${PYTHON_VERSION} (Cargo.toml parity baseline)"
fi

ACTUAL_VERSION=$("${PYTHON_COMMAND[@]}" --version 2>/dev/null || true)
if [ -z "$ACTUAL_VERSION" ]; then
    echo "ERROR: Python flowmark executable did not report a version"
    exit 2
fi
if [ -z "$PYTHON_BIN" ] && [ "$ACTUAL_VERSION" != "v${PYTHON_VERSION}" ]; then
    echo "ERROR: Expected Python flowmark v${PYTHON_VERSION}, got: $ACTUAL_VERSION"
    exit 2
fi

AUDIT_TMP_DIR=$(mktemp -d)
if [ "$KEEP_TMP" = "1" ]; then
    echo "Retaining temporary audit workspace: $AUDIT_TMP_DIR"
else
    trap 'rm -rf "$AUDIT_TMP_DIR"' EXIT
fi
mkdir -p "$REPORT_DIR"

CORPUS_DIGEST=$(
    cd "$CORPUS_DIR"
    find . -type f -name "*.md" -print \
        | LC_ALL=C sort \
        | while IFS= read -r file; do shasum -a 256 "$file"; done \
        | shasum -a 256 \
        | awk '{print $1}'
)
FILE_COUNT=$(find "$CORPUS_DIR" -type f -name "*.md" | wc -l | tr -d ' ')
if [ -n "$EXPECTED_CORPUS_DIGEST" ] && [ "$CORPUS_DIGEST" != "$EXPECTED_CORPUS_DIGEST" ]; then
    echo "ERROR: Corpus digest mismatch"
    echo "  Expected: $EXPECTED_CORPUS_DIGEST"
    echo "  Actual:   $CORPUS_DIGEST"
    exit 2
fi

echo "Corpus parity check"
echo "  Corpus:  $CORPUS_DIR"
echo "  Files:   $FILE_COUNT Markdown files"
echo "  SHA-256: $CORPUS_DIGEST"
echo "  Rust:    $RUST_BIN"
echo "  Python:  $PYTHON_LABEL"
echo "  Report:  $REPORT_DIR"
echo ""

cat > "$REPORT_DIR/metadata.txt" <<EOF
corpus=$CORPUS_DIR
markdown_files=$FILE_COUNT
corpus_sha256=$CORPUS_DIGEST
rust_binary=$RUST_BIN
python=$PYTHON_LABEL
python_reported_version=$ACTUAL_VERSION
EOF

cp -a "$CORPUS_DIR" "$AUDIT_TMP_DIR/td-py"
cp -a "$CORPUS_DIR" "$AUDIT_TMP_DIR/td-rs"

(
    cd "$CORPUS_DIR"
    find . -type f -name "*.md" -print | sed 's#^\./##' | LC_ALL=C sort
) > "$REPORT_DIR/corpus-files.txt"

    "${PYTHON_COMMAND[@]}" --list-files --no-respect-gitignore --files-max-size 0 \
    "$AUDIT_TMP_DIR/td-py/" \
    | sed 's#^.*/td-py/##' \
    | LC_ALL=C sort > "$REPORT_DIR/python-selected-files.txt"
"$RUST_BIN" --list-files --no-respect-gitignore --files-max-size 0 \
    "$AUDIT_TMP_DIR/td-rs/" \
    | sed 's#^.*/td-rs/##' \
    | LC_ALL=C sort > "$REPORT_DIR/rust-selected-files.txt"

if ! diff -u "$REPORT_DIR/python-selected-files.txt" "$REPORT_DIR/rust-selected-files.txt" \
    > "$REPORT_DIR/selection-diff.patch"; then
    echo "ERROR: Python and Rust selected different corpus files"
    echo "Complete diff: $REPORT_DIR/selection-diff.patch"
    exit 2
fi

if ! diff -u "$REPORT_DIR/corpus-files.txt" "$REPORT_DIR/python-selected-files.txt" \
    > "$REPORT_DIR/excluded-files.patch"; then
    echo "ERROR: The formatter selection did not include every Markdown corpus file"
    echo "Complete diff: $REPORT_DIR/excluded-files.patch"
    exit 2
fi

echo "selected_markdown_files=$FILE_COUNT" >> "$REPORT_DIR/metadata.txt"
if [ "$KEEP_TMP" = "1" ]; then
    echo "temporary_workspace=$AUDIT_TMP_DIR" >> "$REPORT_DIR/metadata.txt"
fi

echo "Running Python flowmark..."
if ! "${PYTHON_COMMAND[@]}" --auto --no-respect-gitignore --files-max-size 0 "$AUDIT_TMP_DIR/td-py/" \
    > "$REPORT_DIR/python.stdout" 2> "$REPORT_DIR/python.stderr"; then
    echo "ERROR: Python formatter failed; see $REPORT_DIR/python.stderr"
    cat "$REPORT_DIR/python.stderr"
    exit 2
fi

echo "Running Rust flowmark..."
if ! "$RUST_BIN" --auto --no-respect-gitignore --files-max-size 0 "$AUDIT_TMP_DIR/td-rs/" \
    > "$REPORT_DIR/rust.stdout" 2> "$REPORT_DIR/rust.stderr"; then
    echo "ERROR: Rust formatter failed; see $REPORT_DIR/rust.stderr"
    cat "$REPORT_DIR/rust.stderr"
    exit 2
fi

echo ""
echo "Comparing outputs..."

diff -rq "$AUDIT_TMP_DIR/td-py/" "$AUDIT_TMP_DIR/td-rs/" > "$REPORT_DIR/differences.txt" || true
diff -ruN "$AUDIT_TMP_DIR/td-py/" "$AUDIT_TMP_DIR/td-rs/" > "$REPORT_DIR/diff.patch" || true

if [ ! -s "$REPORT_DIR/differences.txt" ]; then
    echo "PASS: 0 differences across $FILE_COUNT files"
    echo "result=PASS" >> "$REPORT_DIR/metadata.txt"
    exit 0
else
    DIFF_COUNT=$(wc -l < "$REPORT_DIR/differences.txt" | tr -d ' ')
    echo "FAIL: $DIFF_COUNT files differ"
    echo "Complete file list: $REPORT_DIR/differences.txt"
    echo "Complete diff:      $REPORT_DIR/diff.patch"
    echo "result=FAIL" >> "$REPORT_DIR/metadata.txt"
    echo "different_files=$DIFF_COUNT" >> "$REPORT_DIR/metadata.txt"
    exit 1
fi
