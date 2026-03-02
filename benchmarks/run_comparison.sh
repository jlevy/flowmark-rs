#!/usr/bin/env bash
# Run performance comparison across available markdown formatters.
#
# Usage: ./benchmarks/run_comparison.sh [mode] [num_runs]
#   mode: fresh|steady|first-run|second-run (default: fresh)
#   num_runs: timed runs per formatter (default: 3)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORPUS_DIR="$SCRIPT_DIR/corpus"
PRISTINE_DIR="/tmp/flowmark_bench_corpus_pristine"
WORK_DIR="/tmp/flowmark_bench_corpus_work"
RESULTS_FILE="$SCRIPT_DIR/results/comparison_results.txt"

MODE="${1:-fresh}"
NUM_RUNS="${2:-3}"
INCLUDE_OPTIONAL_FORMATTERS="${INCLUDE_OPTIONAL_FORMATTERS:-0}"
SKIP_MARKDOWNFMT="${SKIP_MARKDOWNFMT:-0}"
MARKDOWNFMT_BATCH_SIZE="${MARKDOWNFMT_BATCH_SIZE:-200}"

CANON_MODE="$MODE"
if [ "$MODE" = "first-run" ]; then
    CANON_MODE="fresh"
fi
if [ "$MODE" = "second-run" ]; then
    CANON_MODE="steady"
fi
if [[ "$CANON_MODE" != "fresh" && "$CANON_MODE" != "steady" ]]; then
    echo "ERROR: mode must be fresh|steady|first-run|second-run (got: $MODE)" >&2
    exit 1
fi
if ! [[ "$NUM_RUNS" =~ ^[0-9]+$ ]] || [ "$NUM_RUNS" -le 0 ]; then
    echo "ERROR: num_runs must be a positive integer (got: $NUM_RUNS)" >&2
    exit 1
fi

mkdir -p "$SCRIPT_DIR/results"

if [ ! -d "$CORPUS_DIR" ]; then
    echo "ERROR: Benchmark corpus missing at $CORPUS_DIR" >&2
    echo "Run: ./benchmarks/generate_corpus.sh" >&2
    exit 1
fi

# Always refresh pristine corpus from current benchmark corpus so reruns stay current.
rm -rf "$PRISTINE_DIR"
cp -R "$CORPUS_DIR" "$PRISTINE_DIR"

restore_corpus() {
    rm -rf "$WORK_DIR"
    cp -R "$PRISTINE_DIR" "$WORK_DIR"
}

time_cmd() {
    local start end elapsed
    start=$(date +%s%N)
    eval "$@" > /dev/null 2>&1
    local exit_code=$?
    end=$(date +%s%N)
    elapsed=$((end - start))
    echo "scale=3; $elapsed / 1000000000" | bc
    return $exit_code
}

find_dprint_bin() {
    if command -v dprint >/dev/null 2>&1; then
        command -v dprint
        return 0
    fi
    if [ -x "$HOME/.dprint/bin/dprint" ]; then
        echo "$HOME/.dprint/bin/dprint"
        return 0
    fi
    if [ -x "$REPO_ROOT/attic/dprint/target/release/dprint" ]; then
        echo "$REPO_ROOT/attic/dprint/target/release/dprint"
        return 0
    fi
    return 1
}

BENCHMARKS=()

# shellcheck disable=SC2317
add_benchmark() {
    local name="$1"
    local cmd="$2"
    BENCHMARKS+=("$name|$cmd")
}

RUST_BIN="$REPO_ROOT/target/release/flowmark"
if [ ! -x "$RUST_BIN" ]; then
    echo "ERROR: Rust binary not found at $RUST_BIN" >&2
    echo "Run: cargo build --release" >&2
    exit 1
fi

DPRINT_BIN=""
if DPRINT_BIN="$(find_dprint_bin)"; then
    :
fi

PRETTIER_BIN=""
if command -v prettier >/dev/null 2>&1; then
    PRETTIER_BIN="$(command -v prettier)"
fi

MDFORMAT_BIN=""
if command -v mdformat >/dev/null 2>&1; then
    MDFORMAT_BIN="$(command -v mdformat)"
fi

MARKDOWNFMT_BIN=""
if [ -x "$HOME/go/bin/markdownfmt" ]; then
    MARKDOWNFMT_BIN="$HOME/go/bin/markdownfmt"
elif command -v markdownfmt >/dev/null 2>&1; then
    MARKDOWNFMT_BIN="$(command -v markdownfmt)"
fi

PYTHON_FLOWMARK_BIN=""
if command -v flowmark >/dev/null 2>&1; then
    CANDIDATE="$(command -v flowmark)"
    VERSION_OUT="$($CANDIDATE --version 2>&1 || true)"
    # Rust binary includes "parity: flowmark-py" in version output.
    if ! echo "$VERSION_OUT" | grep -q "parity: flowmark-py"; then
        PYTHON_FLOWMARK_BIN="$CANDIDATE"
    fi
fi

# Ensure dprint config exists in corpus root
if [ ! -f "$CORPUS_DIR/dprint.json" ]; then
    cat > "$CORPUS_DIR/dprint.json" <<'JSON'
{
  "includes": ["**/*.md"],
  "plugins": ["https://plugins.dprint.dev/markdown-0.21.1.wasm"]
}
JSON
fi

# Register benchmarks
FLOWMARK_CACHE_ARG="--no-cache"
if [ "$CANON_MODE" = "steady" ]; then
    FLOWMARK_CACHE_ARG=""
fi
add_benchmark "flowmark-rs ($("$RUST_BIN" --version 2>&1))" "\"$RUST_BIN\" --auto $FLOWMARK_CACHE_ARG \"$WORK_DIR\""
if [ -n "$DPRINT_BIN" ]; then
    DPRINT_INCREMENTAL_ARG="--incremental=false"
    if [ "$CANON_MODE" = "steady" ]; then
        DPRINT_INCREMENTAL_ARG=""
    fi
    add_benchmark "dprint ($($DPRINT_BIN --version 2>&1))" "(cd \"$WORK_DIR\" && \"$DPRINT_BIN\" fmt --config dprint.json $DPRINT_INCREMENTAL_ARG --log-level silent .)"
fi

if [ "$INCLUDE_OPTIONAL_FORMATTERS" = "1" ]; then
    if [ -n "$MARKDOWNFMT_BIN" ] && [ "$SKIP_MARKDOWNFMT" != "1" ]; then
        add_benchmark "markdownfmt" "find \"$WORK_DIR\" -name '*.md' -print0 | xargs -0 -n \"$MARKDOWNFMT_BATCH_SIZE\" \"$MARKDOWNFMT_BIN\" -w"
    elif [ -n "$MARKDOWNFMT_BIN" ] && [ "$SKIP_MARKDOWNFMT" = "1" ]; then
        echo "Note: markdownfmt skipped (SKIP_MARKDOWNFMT=1)."
    fi
    if [ -n "$PRETTIER_BIN" ]; then
        add_benchmark "prettier ($($PRETTIER_BIN --version 2>&1))" "\"$PRETTIER_BIN\" --write \"$WORK_DIR/**/*.md\" --ignore-path /dev/null --log-level silent"
    fi
    if [ -n "$MDFORMAT_BIN" ]; then
        add_benchmark "mdformat ($($MDFORMAT_BIN --version 2>&1))" "\"$MDFORMAT_BIN\" \"$WORK_DIR\""
    fi
    if [ -n "$PYTHON_FLOWMARK_BIN" ]; then
        add_benchmark "flowmark-py ($($PYTHON_FLOWMARK_BIN --version 2>&1))" "\"$PYTHON_FLOWMARK_BIN\" --auto \"$WORK_DIR\""
    fi
else
    echo "Note: optional formatters disabled (set INCLUDE_OPTIONAL_FORMATTERS=1 to include prettier/mdformat/markdownfmt/flowmark-py)."
    echo ""
fi

NUM_FILES=$(find "$CORPUS_DIR" -name '*.md' | wc -l)

echo "=== Markdown Formatter Comparison Benchmark ==="
echo ""
echo "Date: $(date -u '+%Y-%m-%d %H:%M UTC')"
echo "Platform: $(uname -srm)"
echo "Mode: $MODE"
echo "Corpus: $NUM_FILES Markdown files ($(du -sh "$CORPUS_DIR" | cut -f1))"
echo "Work dir: $WORK_DIR"
echo "Runs per formatter: $NUM_RUNS"
echo ""

echo "Formatters to run:"
for entry in "${BENCHMARKS[@]}"; do
    IFS='|' read -r name _cmd <<< "$entry"
    echo "  - $name"
done
echo ""

echo "=========================================="
echo "Running benchmarks..."
echo "=========================================="
echo ""

> "$RESULTS_FILE"

run_benchmark() {
    local name="$1"
    local cmd="$2"
    local times=()

    echo "--- $name ---"

    restore_corpus
    echo "  Warmup..."
    eval "$cmd" > /dev/null 2>&1 || {
        echo "  Skipping (warmup failed)"
        echo ""
        return
    }

    for i in $(seq 1 "$NUM_RUNS"); do
        if [ "$CANON_MODE" = "fresh" ]; then
            restore_corpus
        fi
        local t
        t=$(time_cmd "$cmd")
        times+=("$t")
        echo "  Run $i: ${t}s"
    done

    local sum=0
    for t in "${times[@]}"; do
        sum=$(echo "$sum + $t" | bc)
    done
    local mean
    mean=$(echo "scale=3; $sum / $NUM_RUNS" | bc)

    local variance=0
    for t in "${times[@]}"; do
        local diff
        diff=$(echo "$t - $mean" | bc)
        variance=$(echo "$variance + $diff * $diff" | bc)
    done
    local stddev
    stddev=$(echo "scale=3; sqrt($variance / $NUM_RUNS)" | bc)

    local min max
    min="${times[0]}"
    max="${times[0]}"
    for t in "${times[@]}"; do
        if (( $(echo "$t < $min" | bc -l) )); then min="$t"; fi
        if (( $(echo "$t > $max" | bc -l) )); then max="$t"; fi
    done

    local cv
    if (( $(echo "$mean > 0" | bc -l) )); then
        cv=$(echo "scale=1; $stddev / $mean * 100" | bc)
    else
        cv="0.0"
    fi

    echo "  Mean: ${mean}s ± ${stddev}s (CV: ${cv}%, range: ${min}–${max}s)"
    echo ""

    echo "$name|$mean|$stddev|$cv|$min|$max" >> "$RESULTS_FILE"
}

for entry in "${BENCHMARKS[@]}"; do
    IFS='|' read -r name cmd <<< "$entry"
    run_benchmark "$name" "$cmd"
done

echo "=========================================="
echo "All benchmarks complete!"
echo "=========================================="
echo ""
echo "Results saved to: $RESULTS_FILE"
echo ""

echo "=== Summary Table ==="
echo ""
printf "%-44s %10s %10s %6s %10s %10s\n" "Formatter" "Mean (s)" "StdDev" "CV%" "Min (s)" "Max (s)"
printf "%-44s %10s %10s %6s %10s %10s\n" "--------------------------------------------" "----------" "----------" "------" "----------" "----------"
while IFS='|' read -r name mean stddev cv min max; do
    printf "%-44s %10s %10s %6s %10s %10s\n" "$name" "$mean" "$stddev" "$cv" "$min" "$max"
done < "$RESULTS_FILE"
