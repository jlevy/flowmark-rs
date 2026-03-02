#!/usr/bin/env bash
# Profile the Rust flowmark binary using valgrind's callgrind.
# Requires: valgrind, callgrind_annotate
#
# Usage: ./benchmarks/profile_rust.sh [mode]
#   mode: "single" (default) - profile on testdoc.orig.md
#         "batch"  - profile on one batch (~450 files)
#
# Output:
#   benchmarks/results/callgrind_<mode>.out     - raw callgrind data
#   benchmarks/results/profile_<mode>.txt       - annotated top functions
#
# NOTE: Must build with debug symbols first:
#   CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"
RUST_BIN="$REPO_ROOT/target/release/flowmark"
MODE="${1:-single}"

mkdir -p "$RESULTS_DIR"

echo "=== Profiling Rust flowmark (mode: $MODE) ==="

# Verify debug symbols
if file "$RUST_BIN" | grep -q 'stripped'; then
    echo "WARNING: Binary is stripped. Rebuild with:"
    echo "  CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release"
    echo ""
fi

case "$MODE" in
    single)
        TARGET="$REPO_ROOT/tests/testdocs/testdoc.orig.md"
        CALLGRIND_OUT="$RESULTS_DIR/callgrind_single.out"
        echo "Target: $TARGET ($(wc -l < "$TARGET") lines)"
        echo ""
        echo "Running callgrind..."
        valgrind --tool=callgrind --callgrind-out-file="$CALLGRIND_OUT" \
            "$RUST_BIN" "$TARGET" > /dev/null 2>&1
        ;;
    batch)
        TARGET="$SCRIPT_DIR/corpus/batch_000"
        CALLGRIND_OUT="$RESULTS_DIR/callgrind_batch.out"
        echo "Target: $TARGET ($(find "$TARGET" -name '*.md' | wc -l) files)"
        echo ""
        echo "Running callgrind..."
        valgrind --tool=callgrind --callgrind-out-file="$CALLGRIND_OUT" \
            "$RUST_BIN" --auto "$TARGET" > /dev/null 2>&1
        ;;
    *)
        echo "Unknown mode: $MODE" >&2
        exit 1
        ;;
esac

echo "Done. Analyzing..."
echo ""

PROFILE_OUT="$RESULTS_DIR/profile_${MODE}.txt"
callgrind_annotate --auto=yes "$CALLGRIND_OUT" > "$PROFILE_OUT" 2>/dev/null

echo "Top 30 functions by instruction count:"
echo ""
head -60 "$PROFILE_OUT" | tail -45
echo ""
echo "Full annotated output: $PROFILE_OUT"
echo "Raw callgrind data:    $CALLGRIND_OUT"
