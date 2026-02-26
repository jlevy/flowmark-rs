#!/usr/bin/env bash
# Generate a benchmark corpus of ~1,000 Markdown files in a deep directory tree.
# Uses all .md files from the repo (tests, docs, README) as source material.
#
# Usage: ./benchmarks/generate_corpus.sh [target_count]
#   target_count: approximate number of files to generate (default: 1000)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORPUS_DIR="$SCRIPT_DIR/corpus"
TARGET_COUNT="${1:-1000}"

echo "=== Generating benchmark corpus ==="
echo "Target: ~$TARGET_COUNT files"
echo "Output: $CORPUS_DIR"

# Clean previous corpus
rm -rf "$CORPUS_DIR"
mkdir -p "$CORPUS_DIR"

# Collect all source .md files from the repo (excluding generated/build dirs)
SOURCE_FILES=()
while IFS= read -r -d '' file; do
    SOURCE_FILES+=("$file")
done < <(find "$REPO_ROOT" -name "*.md" \
    -not -path "*/.git/*" \
    -not -path "*/target/*" \
    -not -path "*/.tbd/*" \
    -not -path "*/benchmarks/corpus/*" \
    -not -path "*/node_modules/*" \
    -not -path "*/.venv/*" \
    -print0 | sort -z)

SOURCE_COUNT="${#SOURCE_FILES[@]}"
echo "Found $SOURCE_COUNT source .md files"

if [ "$SOURCE_COUNT" -eq 0 ]; then
    echo "ERROR: No source .md files found" >&2
    exit 1
fi

# Calculate how many full sets we need
# Each "set" is one copy of all source files
SETS_NEEDED=$(( (TARGET_COUNT + SOURCE_COUNT - 1) / SOURCE_COUNT ))
FILES_PER_BATCH="$SOURCE_COUNT"
# Organize into batches of 5 sets each, nested 3 levels deep
SETS_PER_BATCH=5
BATCHES_NEEDED=$(( (SETS_NEEDED + SETS_PER_BATCH - 1) / SETS_PER_BATCH ))

echo "Plan: $SETS_NEEDED sets of $SOURCE_COUNT files across $BATCHES_NEEDED batches"

total_files=0
set_idx=0

for batch in $(seq 0 $((BATCHES_NEEDED - 1))); do
    batch_dir=$(printf "$CORPUS_DIR/batch_%03d" "$batch")
    for set_in_batch in $(seq 0 $((SETS_PER_BATCH - 1))); do
        if [ "$set_idx" -ge "$SETS_NEEDED" ]; then
            break 2
        fi
        set_dir=$(printf "$batch_dir/set_%02d" "$set_in_batch")
        # Add a subdirectory level for depth
        sub_idx=$((set_idx % 4))
        case $sub_idx in
            0) sub_path="docs" ;;
            1) sub_path="content/deep" ;;
            2) sub_path="notes/archive" ;;
            3) sub_path="pages" ;;
        esac
        dest_dir="$set_dir/$sub_path"
        mkdir -p "$dest_dir"

        for src_file in "${SOURCE_FILES[@]}"; do
            # Preserve original filename but prefix with set index to avoid collisions
            basename_file="$(basename "$src_file")"
            cp "$src_file" "$dest_dir/$basename_file"
            total_files=$((total_files + 1))
        done

        set_idx=$((set_idx + 1))
    done
done

echo ""
echo "=== Corpus generated ==="
echo "Total files: $total_files"
echo "Directory tree depth: 4-5 levels"
echo "Location: $CORPUS_DIR"

# Show tree structure summary
echo ""
echo "Structure:"
find "$CORPUS_DIR" -type d | head -20
echo "..."
echo ""
echo "Total size: $(du -sh "$CORPUS_DIR" | cut -f1)"
