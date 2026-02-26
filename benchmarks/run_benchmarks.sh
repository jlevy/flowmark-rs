#!/usr/bin/env bash
# Run end-to-end performance comparison of Python flowmark vs Rust flowmark.
# Requires: hyperfine, flowmark (Python), target/release/flowmark (Rust)
#
# Usage: ./benchmarks/run_benchmarks.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORPUS_DIR="$SCRIPT_DIR/corpus"
RESULTS_DIR="$SCRIPT_DIR/results"
RUST_BIN="$REPO_ROOT/target/release/flowmark"
PYTHON_BIN="$(which flowmark)"

mkdir -p "$RESULTS_DIR"

echo "=== Performance Comparison: Python vs Rust flowmark ==="
echo ""
echo "Python: $PYTHON_BIN ($(flowmark --version 2>&1))"
echo "Rust:   $RUST_BIN ($($RUST_BIN --version 2>&1))"
echo "Corpus: $CORPUS_DIR"
echo "Files:  $(find "$CORPUS_DIR" -name '*.md' | wc -l) .md files"
echo "Size:   $(du -sh "$CORPUS_DIR" | cut -f1)"
echo ""

# Verify corpus exists
if [ ! -d "$CORPUS_DIR" ]; then
    echo "ERROR: Corpus not found. Run generate_corpus.sh first." >&2
    exit 1
fi

# Verify both binaries work
echo "--- Smoke test ---"
SAMPLE_FILE=$(find "$CORPUS_DIR" -name '*.md' | head -1)
echo "Testing Python on: $SAMPLE_FILE"
flowmark "$SAMPLE_FILE" > /dev/null 2>&1
echo "  OK"
echo "Testing Rust on: $SAMPLE_FILE"
"$RUST_BIN" "$SAMPLE_FILE" > /dev/null 2>&1
echo "  OK"
echo ""

# ============================================================
# Benchmark 1: Full batch formatting (--auto mode, in-place)
# ============================================================
echo "=========================================="
echo "Benchmark 1: Full batch --auto formatting"
echo "=========================================="
echo ""
echo "This formats all 1000+ files in-place with --auto flag."
echo "(Each run re-formats already-formatted files, measuring steady-state.)"
echo ""

# First, do one warmup pass with each to ensure files are formatted
echo "--- Pre-formatting corpus with Rust (warmup) ---"
"$RUST_BIN" --auto "$CORPUS_DIR" 2>&1 || true
echo "Done."
echo ""

hyperfine \
    --warmup 1 \
    --min-runs 5 \
    --export-json "$RESULTS_DIR/bench_auto_format.json" \
    --export-markdown "$RESULTS_DIR/bench_auto_format.md" \
    --command-name "Python flowmark --auto" \
    "flowmark --auto $CORPUS_DIR" \
    --command-name "Rust flowmark --auto" \
    "$RUST_BIN --auto $CORPUS_DIR"

echo ""
echo "Results saved to: $RESULTS_DIR/bench_auto_format.json"
echo ""

# ============================================================
# Benchmark 2: File discovery only (--list-files)
# ============================================================
echo "=========================================="
echo "Benchmark 2: File discovery (--list-files)"
echo "=========================================="
echo ""

hyperfine \
    --warmup 2 \
    --min-runs 10 \
    --export-json "$RESULTS_DIR/bench_list_files.json" \
    --export-markdown "$RESULTS_DIR/bench_list_files.md" \
    --command-name "Python flowmark --list-files" \
    "flowmark --list-files $CORPUS_DIR" \
    --command-name "Rust flowmark --list-files" \
    "$RUST_BIN --list-files $CORPUS_DIR"

echo ""
echo "Results saved to: $RESULTS_DIR/bench_list_files.json"
echo ""

# ============================================================
# Benchmark 3: Single large file (testdoc.orig.md)
# ============================================================
echo "=========================================="
echo "Benchmark 3: Single large file formatting"
echo "=========================================="
echo ""
echo "Formats testdoc.orig.md (1,734 lines) to stdout."
echo ""

TESTDOC="$REPO_ROOT/tests/testdocs/testdoc.orig.md"

hyperfine \
    --warmup 3 \
    --min-runs 20 \
    --export-json "$RESULTS_DIR/bench_single_file.json" \
    --export-markdown "$RESULTS_DIR/bench_single_file.md" \
    --command-name "Python flowmark (single file)" \
    "flowmark $TESTDOC > /dev/null" \
    --command-name "Rust flowmark (single file)" \
    "$RUST_BIN $TESTDOC > /dev/null"

echo ""
echo "Results saved to: $RESULTS_DIR/bench_single_file.json"
echo ""

# ============================================================
# Benchmark 4: Semantic mode on corpus
# ============================================================
echo "=========================================="
echo "Benchmark 4: Semantic mode formatting"
echo "=========================================="
echo ""

hyperfine \
    --warmup 1 \
    --min-runs 5 \
    --export-json "$RESULTS_DIR/bench_semantic.json" \
    --export-markdown "$RESULTS_DIR/bench_semantic.md" \
    --command-name "Python flowmark --semantic" \
    "flowmark --semantic $CORPUS_DIR" \
    --command-name "Rust flowmark --semantic" \
    "$RUST_BIN --semantic $CORPUS_DIR"

echo ""
echo "Results saved to: $RESULTS_DIR/bench_semantic.json"
echo ""

echo "=========================================="
echo "All benchmarks complete!"
echo "=========================================="
echo ""
echo "Results in: $RESULTS_DIR/"
ls -la "$RESULTS_DIR/"
