#!/usr/bin/env bash
# Profile the Rust flowmark binary using valgrind/callgrind.
#
# Usage: ./benches/profile-rust.sh [corpus_dir] [mode]
#
# mode: "default" or "auto" (default: "default")
#
# Requires:
#   - valgrind (with callgrind)
#   - A release build with debug symbols:
#     CARGO_PROFILE_RELEASE_STRIP=false CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release
#
# Outputs:
#   /tmp/callgrind-<mode>.out  - Raw callgrind data
#   Annotated profile to stdout

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORPUS_DIR="${1:-$REPO_ROOT/benches/corpus}"
MODE="${2:-default}"

RUST_BIN="$REPO_ROOT/target/release/flowmark"

if [ ! -d "$CORPUS_DIR" ]; then
    echo "ERROR: Corpus directory not found: $CORPUS_DIR" >&2
    echo "Run ./benches/generate-bench-corpus.sh first." >&2
    exit 1
fi

if ! command -v valgrind &> /dev/null; then
    echo "ERROR: valgrind not found. Install with: apt install valgrind" >&2
    exit 1
fi

OUTFILE="/tmp/callgrind-${MODE}.out"

# Build command based on mode.
CMD=("$RUST_BIN")
if [ "$MODE" = "auto" ]; then
    CMD+=("--auto")
fi
CMD+=("$CORPUS_DIR")

echo "=== Profiling flowmark (mode: $MODE) ==="
echo "Binary:  $RUST_BIN"
echo "Corpus:  $CORPUS_DIR"
echo "Output:  $OUTFILE"
echo ""

# Run callgrind.
echo "Running callgrind (this takes a while under instrumentation)..."
valgrind --tool=callgrind --callgrind-out-file="$OUTFILE" \
    "${CMD[@]}" > /dev/null 2>&1

echo "Done. Analyzing..."
echo ""

# Show top functions.
echo "=== Top 50 functions by instruction count ==="
echo ""
callgrind_annotate --auto=yes "$OUTFILE" 2>/dev/null | head -80

echo ""
echo "=== Flowmark-specific functions ==="
echo ""
callgrind_annotate "$OUTFILE" 2>/dev/null | grep -E 'flowmark::' | sort -t'(' -k2 -rn | head -20

echo ""
echo "Raw callgrind data saved to: $OUTFILE"
echo "For detailed analysis: callgrind_annotate --auto=yes $OUTFILE"
