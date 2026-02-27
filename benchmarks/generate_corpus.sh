#!/usr/bin/env bash
# Generate a benchmark corpus of markdown files in a deep directory tree.
# Uses repository .md files as source material and produces exactly target_count files.
#
# Usage: ./benchmarks/generate_corpus.sh [target_count]
#   target_count: number of files to generate (default: 1000)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORPUS_DIR="$SCRIPT_DIR/corpus"
TARGET_COUNT="${1:-1000}"

if ! [[ "$TARGET_COUNT" =~ ^[0-9]+$ ]] || [ "$TARGET_COUNT" -le 0 ]; then
    echo "ERROR: target_count must be a positive integer (got: $TARGET_COUNT)" >&2
    exit 1
fi

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

SET_SIZE=100
TOTAL_SETS=$(( (TARGET_COUNT + SET_SIZE - 1) / SET_SIZE ))
echo "Plan: $TARGET_COUNT files across $TOTAL_SETS sets (set size: $SET_SIZE)"

for i in $(seq 0 $((TARGET_COUNT - 1))); do
    src_index=$((i % SOURCE_COUNT))
    src_file="${SOURCE_FILES[$src_index]}"

    set_idx=$((i / SET_SIZE))
    batch_idx=$((set_idx / 5))
    set_in_batch=$((set_idx % 5))

    batch_dir=$(printf "$CORPUS_DIR/batch_%03d" "$batch_idx")
    set_dir=$(printf "$batch_dir/set_%02d" "$set_in_batch")

    # Add a subdirectory level for depth
    sub_idx=$((i % 4))
    case $sub_idx in
        0) sub_path="docs" ;;
        1) sub_path="content/deep" ;;
        2) sub_path="notes/archive" ;;
        3) sub_path="pages" ;;
    esac
    dest_dir="$set_dir/$sub_path"
    mkdir -p "$dest_dir"

    # Prefix with generated index to guarantee uniqueness even with duplicate basenames.
    base_name="$(basename "$src_file")"
    dest_name="$(printf "%05d_%s" "$i" "$base_name")"
    cp "$src_file" "$dest_dir/$dest_name"
done

# Create a dprint config in the corpus root so dprint benchmarks are reproducible.
cat > "$CORPUS_DIR/dprint.json" <<'JSON'
{
  "includes": ["**/*.md"],
  "plugins": ["https://plugins.dprint.dev/markdown-0.21.1.wasm"]
}
JSON

echo ""
echo "=== Corpus generated ==="
echo "Total files: $(find "$CORPUS_DIR" -name '*.md' | wc -l)"
echo "Directory tree depth: 4-5 levels"
echo "Location: $CORPUS_DIR"

# Show tree structure summary
echo ""
echo "Structure:"
find "$CORPUS_DIR" -type d | head -20
echo "..."
echo ""
echo "Total size: $(du -sh "$CORPUS_DIR" | cut -f1)"
