#!/usr/bin/env bash
# Generate a benchmark corpus of ~1000 Markdown files from this repo's test data.
# Usage: ./benches/generate-bench-corpus.sh [output_dir] [num_batches]
#
# Default: benches/corpus/ with 40 batches (each batch copies all source files).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUTPUT_DIR="${1:-$REPO_ROOT/benches/corpus}"
NUM_BATCHES="${2:-40}"

# Source files to include in the corpus.
SOURCE_FILES=()

# Content fixtures (21 files, ~6.5 KB total) - exercise all markdown features.
for f in "$REPO_ROOT"/tests/tryscript/fixtures/content/*.md; do
    [ -f "$f" ] && SOURCE_FILES+=("$f")
done

# Large test document (77 KB) - comprehensive real-world document.
[ -f "$REPO_ROOT/tests/testdocs/testdoc.orig.md" ] && \
    SOURCE_FILES+=("$REPO_ROOT/tests/testdocs/testdoc.orig.md")

# Parity test (5 KB) - corner cases.
[ -f "$REPO_ROOT/tests/parity/corner-cases.md" ] && \
    SOURCE_FILES+=("$REPO_ROOT/tests/parity/corner-cases.md")

# Documentation files.
for f in "$REPO_ROOT"/README.md "$REPO_ROOT"/docs/port-status.md "$REPO_ROOT"/docs/porting-log-review.md; do
    [ -f "$f" ] && SOURCE_FILES+=("$f")
done

NUM_SOURCE=${#SOURCE_FILES[@]}
if [ "$NUM_SOURCE" -eq 0 ]; then
    echo "ERROR: No source markdown files found." >&2
    exit 1
fi

# Clean and create output directory.
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

TOTAL_FILES=0

for batch in $(seq -w 0 $((NUM_BATCHES - 1))); do
    BATCH_DIR="$OUTPUT_DIR/batch-$batch"
    mkdir -p "$BATCH_DIR/content" "$BATCH_DIR/docs" "$BATCH_DIR/tests"

    for src in "${SOURCE_FILES[@]}"; do
        base="$(basename "$src")"
        # Distribute files into subdirectories based on their source location.
        case "$src" in
            */fixtures/content/*)
                cp "$src" "$BATCH_DIR/content/$base"
                ;;
            */testdocs/* | */parity/*)
                cp "$src" "$BATCH_DIR/tests/$base"
                ;;
            *)
                cp "$src" "$BATCH_DIR/docs/$base"
                ;;
        esac
        TOTAL_FILES=$((TOTAL_FILES + 1))
    done
done

# Calculate total size.
TOTAL_SIZE=$(du -sh "$OUTPUT_DIR" | cut -f1)

echo "Corpus generated:"
echo "  Source files:  $NUM_SOURCE unique files"
echo "  Batches:       $NUM_BATCHES"
echo "  Total files:   $TOTAL_FILES"
echo "  Total size:    $TOTAL_SIZE"
echo "  Output:        $OUTPUT_DIR"
