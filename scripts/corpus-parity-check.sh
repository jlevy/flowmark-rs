#!/bin/bash
# Corpus parity check: run both the pinned Python flowmark and Rust flowmark on a directory
# of markdown files and report any differences.
#
# Usage: ./scripts/corpus-parity-check.sh [corpus_dir] [rust_binary]
# Default corpus_dir: attic/test-docs
# Default rust_binary: target/release/flowmark
#
# Exit code: 0 = full parity, 1 = differences found

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CORPUS_DIR="${1:-attic/test-docs}"
RUST_BIN="${2:-target/release/flowmark}"
# Read Python parity version from the single source of truth in Cargo.toml
PYTHON_VERSION=$(grep -A1 '\[package.metadata.parity\]' "$REPO_ROOT/Cargo.toml" | grep version | sed 's/.*"\(.*\)"/\1/')

if [ ! -d "$CORPUS_DIR" ]; then
    echo "ERROR: Corpus directory not found: $CORPUS_DIR"
    exit 2
fi

if [ ! -x "$RUST_BIN" ]; then
    echo "ERROR: Rust binary not found: $RUST_BIN"
    echo "Build with: cargo build --release"
    exit 2
fi

# Verify Python version
ACTUAL_VERSION=$(uvx "flowmark@${PYTHON_VERSION}" --version 2>/dev/null || true)
if [ "$ACTUAL_VERSION" != "v${PYTHON_VERSION}" ]; then
    echo "ERROR: Expected Python flowmark v${PYTHON_VERSION}, got: $ACTUAL_VERSION"
    exit 2
fi

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Corpus parity check"
echo "  Corpus:  $CORPUS_DIR"
echo "  Rust:    $RUST_BIN"
echo "  Python:  flowmark v${PYTHON_VERSION}"
echo ""

cp -a "$CORPUS_DIR" "$TMPDIR/td-py"
cp -a "$CORPUS_DIR" "$TMPDIR/td-rs"

echo "Running Python flowmark..."
uvx "flowmark@${PYTHON_VERSION}" --auto --inplace "$TMPDIR/td-py/" 2>/dev/null

echo "Running Rust flowmark..."
"$RUST_BIN" --auto --inplace "$TMPDIR/td-rs/" 2>/dev/null

echo ""
echo "Comparing outputs..."

DIFF_FILES=$(diff -rq "$TMPDIR/td-py/" "$TMPDIR/td-rs/" 2>/dev/null | grep "^Files" || true)

if [ -z "$DIFF_FILES" ]; then
    FILE_COUNT=$(find "$CORPUS_DIR" -name "*.md" | wc -l | tr -d ' ')
    echo "PASS: 0 differences across $FILE_COUNT files"
    exit 0
else
    echo "FAIL: Differences found:"
    echo "$DIFF_FILES" | while read -r line; do
        F1=$(echo "$line" | sed 's/Files \([^ ]*\) and.*/\1/')
        F2=$(echo "$line" | sed 's/Files .* and \([^ ]*\) differ/\1/')
        REL=$(echo "$F1" | sed "s|$TMPDIR/td-py/||")
        echo ""
        echo "=== $REL ==="
        diff -u "$F1" "$F2" | head -30
    done
    DIFF_COUNT=$(echo "$DIFF_FILES" | wc -l | tr -d ' ')
    echo ""
    echo "Total: $DIFF_COUNT files with differences"
    exit 1
fi
