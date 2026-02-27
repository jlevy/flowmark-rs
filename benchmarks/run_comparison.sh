#!/usr/bin/env bash
# Run performance comparison across 6 markdown formatters.
# Each formatter is run 3 times on the full corpus (~924 files, 10 MB).
# The corpus is restored to pristine state before each formatter's runs.
#
# Usage: ./benchmarks/run_comparison.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORPUS_DIR="$SCRIPT_DIR/corpus"
PRISTINE_DIR="/tmp/corpus_pristine"
RESULTS_FILE="$SCRIPT_DIR/results/comparison_results.txt"

RUST_BIN="$REPO_ROOT/target/release/flowmark"
PYTHON_BIN="$(command -v flowmark)"
PRETTIER_BIN="$(command -v prettier)"
DPRINT_BIN="$HOME/.dprint/bin/dprint"
MDFORMAT_BIN="$(command -v mdformat)"
MARKDOWNFMT_BIN="$HOME/go/bin/markdownfmt"

export PATH="$HOME/.dprint/bin:$HOME/go/bin:$PATH"

NUM_RUNS=3
NUM_FILES=$(find "$CORPUS_DIR" -name '*.md' | wc -l)

mkdir -p "$SCRIPT_DIR/results"

restore_corpus() {
    rm -rf "$CORPUS_DIR"
    cp -r "$PRISTINE_DIR" "$CORPUS_DIR"
}

# Run a command and output wall-clock seconds (with millisecond precision)
time_cmd() {
    local start end elapsed
    start=$(date +%s%N)
    eval "$@" > /dev/null 2>&1
    local exit_code=$?
    end=$(date +%s%N)
    elapsed=$(( (end - start) ))
    # Convert nanoseconds to seconds with 3 decimal places
    echo "scale=3; $elapsed / 1000000000" | bc
    return $exit_code
}

echo "=== Markdown Formatter Comparison Benchmark ==="
echo ""
echo "Date: $(date -u '+%Y-%m-%d %H:%M UTC')"
echo "Platform: $(uname -srm)"
echo "Corpus: $NUM_FILES Markdown files ($(du -sh "$CORPUS_DIR" | cut -f1))"
echo "Runs per formatter: $NUM_RUNS"
echo ""
echo "Formatters:"
echo "  flowmark (Python): $($PYTHON_BIN --version 2>&1)"
echo "  flowmark (Rust):   $($RUST_BIN --version 2>&1)"
echo "  prettier:          $(prettier --version 2>&1)"
echo "  dprint:            $(dprint --version 2>&1)"
echo "  mdformat:          $(mdformat --version 2>&1)"
echo "  markdownfmt:       (Go, latest)"
echo ""

# ---- Define formatter commands ----
# Each formats all .md files in the corpus in-place.

fmt_flowmark_py() {
    "$PYTHON_BIN" --auto "$CORPUS_DIR"
}

fmt_flowmark_rs() {
    "$RUST_BIN" --auto "$CORPUS_DIR"
}

fmt_prettier() {
    # --ignore-path /dev/null: disable .gitignore so gitignored corpus files are included
    prettier --write "$CORPUS_DIR/**/*.md" --ignore-path /dev/null --log-level silent
}

fmt_dprint() {
    # --incremental=false: disable caching so every file is re-read and re-formatted
    cd "$CORPUS_DIR" && dprint fmt --config dprint.json --incremental=false --log-level silent 2>/dev/null; cd "$REPO_ROOT"
}

fmt_mdformat() {
    mdformat "$CORPUS_DIR"
}

fmt_markdownfmt() {
    find "$CORPUS_DIR" -name '*.md' -exec "$MARKDOWNFMT_BIN" -w {} +
}

# ---- Benchmark runner ----
run_benchmark() {
    local name="$1"
    local cmd="$2"
    local times=()

    echo "--- $name ---"

    # Restore corpus and do warmup
    restore_corpus
    echo "  Warmup..."
    eval "$cmd" > /dev/null 2>&1 || true

    # Timed runs (on already-formatted files = steady state)
    for i in $(seq 1 $NUM_RUNS); do
        local t
        t=$(time_cmd "$cmd")
        times+=("$t")
        echo "  Run $i: ${t}s"
    done

    # Compute mean and stddev
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

    # Store results for summary table
    echo "$name|$mean|$stddev|$cv|$min|$max|${times[0]}|${times[1]}|${times[2]}" >> "$RESULTS_FILE"
}

# Clear previous results
> "$RESULTS_FILE"

echo "=========================================="
echo "Running benchmarks..."
echo "=========================================="
echo ""

run_benchmark "flowmark (Rust) v0.2.4"      "fmt_flowmark_rs"
run_benchmark "dprint v0.52.0"               "fmt_dprint"
run_benchmark "prettier v3.8.1"              "fmt_prettier"
run_benchmark "markdownfmt (Go)"             "fmt_markdownfmt"
run_benchmark "mdformat v1.0.0"              "fmt_mdformat"
run_benchmark "flowmark (Python) v0.6.4"     "fmt_flowmark_py"

echo "=========================================="
echo "All benchmarks complete!"
echo "=========================================="
echo ""
echo "Results saved to: $RESULTS_FILE"
echo ""

# Print summary table
echo "=== Summary Table ==="
echo ""
printf "%-30s %10s %10s %6s %10s %10s\n" "Formatter" "Mean (s)" "StdDev" "CV%" "Min (s)" "Max (s)"
printf "%-30s %10s %10s %6s %10s %10s\n" "------------------------------" "----------" "----------" "------" "----------" "----------"
while IFS='|' read -r name mean stddev cv min max r1 r2 r3; do
    printf "%-30s %10s %10s %6s %10s %10s\n" "$name" "$mean" "$stddev" "$cv" "$min" "$max"
done < "$RESULTS_FILE"
