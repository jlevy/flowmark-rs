#!/bin/bash
# Generate golden expected files by running Python flowmark v0.6.4 on corner-case inputs.
# These files are the ground truth for parity testing.
#
# Usage: ./scripts/generate-parity-golden.sh
# Requires: uvx (uv tool runner) with access to flowmark==0.6.4

set -euo pipefail

PARITY_DIR="tests/parity"
PYTHON_VERSION="0.6.4"

# Verify Python flowmark version
ACTUAL_VERSION=$(uvx "flowmark@${PYTHON_VERSION}" --version 2>/dev/null || true)
if [ "$ACTUAL_VERSION" != "v${PYTHON_VERSION}" ]; then
    echo "ERROR: Expected Python flowmark v${PYTHON_VERSION}, got: $ACTUAL_VERSION"
    echo "Install with: uv tool install flowmark==${PYTHON_VERSION}"
    exit 1
fi

echo "Using Python flowmark $ACTUAL_VERSION"

generate() {
    local input="$1"
    local mode_name="$2"
    shift 2
    local flags=("$@")

    local base
    base=$(basename "$input" .md)
    local output="${PARITY_DIR}/${base}.expected.${mode_name}.md"

    echo "  ${mode_name} -> ${output}"
    uvx "flowmark@${PYTHON_VERSION}" "${flags[@]}" - < "$input" > "$output"
}

for input in "${PARITY_DIR}"/corner-cases*.md; do
    [ -f "$input" ] || continue
    # Skip expected/actual output files — only process input files
    [[ "$input" == *.expected.* ]] && continue
    [[ "$input" == *.actual.* ]] && continue
    echo "Processing: $input"

    # Default mode (width=88, no semantic)
    generate "$input" "default" "-w" "88"

    # Auto mode (semantic + cleanups + smartquotes + ellipses)
    # Note: --auto implies --inplace which doesn't work with stdin,
    # so we spell out the individual flags.
    generate "$input" "auto" "-w" "88" "--semantic" "--cleanups" "--smartquotes" "--ellipses"

    # Tight mode (semantic + list-spacing tight)
    generate "$input" "tight" "-w" "88" "--semantic" "--list-spacing" "tight"

    # Loose mode (semantic + list-spacing loose)
    generate "$input" "loose" "-w" "88" "--semantic" "--list-spacing" "loose"

    # Plaintext mode
    generate "$input" "plaintext" "-w" "88" "--plaintext"
done

echo ""
echo "Done. Review the .expected.*.md files and commit them."
echo "To verify idempotency, run this script again and check for diffs."
