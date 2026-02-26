#!/usr/bin/env bash
# Benchmark: Python flowmark vs Rust flowmark-rs on a large corpus.
#
# Usage: ./benches/run-bench.sh [corpus_dir] [iterations]
#
# Measures end-to-end wall-clock time for formatting all files in the corpus
# using --check mode (parse + format, no file writes).
# Reports throughput in files/sec and MB/sec.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORPUS_DIR="${1:-$REPO_ROOT/benches/corpus}"
ITERATIONS="${2:-5}"

RUST_BIN="$REPO_ROOT/target/release/flowmark"
PYTHON_BIN="$(command -v flowmark 2>/dev/null || echo "")"

# --- Validation ---

if [ ! -d "$CORPUS_DIR" ]; then
    echo "ERROR: Corpus directory not found: $CORPUS_DIR" >&2
    echo "Run ./benches/generate-bench-corpus.sh first." >&2
    exit 1
fi

if [ ! -x "$RUST_BIN" ]; then
    echo "ERROR: Rust binary not found: $RUST_BIN" >&2
    echo "Run 'cargo build --release' first." >&2
    exit 1
fi

if [ -z "$PYTHON_BIN" ]; then
    echo "WARNING: Python flowmark not found in PATH. Skipping Python benchmarks." >&2
    SKIP_PYTHON=1
else
    SKIP_PYTHON=0
fi

# --- Corpus stats ---

FILE_COUNT=$(find "$CORPUS_DIR" -name '*.md' -type f | wc -l)
TOTAL_BYTES=$(find "$CORPUS_DIR" -name '*.md' -type f -exec cat {} + | wc -c)
TOTAL_MB=$(awk "BEGIN {printf \"%.2f\", $TOTAL_BYTES / 1048576}")

echo "=== Benchmark: flowmark Python vs Rust ==="
echo ""
echo "Corpus:     $CORPUS_DIR"
echo "Files:      $FILE_COUNT"
echo "Total size: ${TOTAL_MB} MB"
echo "Iterations: $ITERATIONS"
echo ""

# --- Helper: get time in milliseconds ---

now_ms() {
    # Use date +%s%N if available (Linux), else fall back to python.
    if date +%s%N > /dev/null 2>&1; then
        echo $(($(date +%s%N) / 1000000))
    else
        python3 -c 'import time; print(int(time.time()*1000))'
    fi
}

# --- Helper: run a command N times, collect wall-clock seconds ---

# Global arrays for results (bash 4+ required).
declare -a RESULT_TIMES

run_timed() {
    local label="$1"
    shift
    local cmd=("$@")
    RESULT_TIMES=()

    echo "--- $label ---"

    # Warmup run (not counted).
    "${cmd[@]}" > /dev/null 2>&1 || true

    for i in $(seq 1 "$ITERATIONS"); do
        local start_ms end_ms elapsed_ms elapsed_s
        start_ms=$(now_ms)
        "${cmd[@]}" > /dev/null 2>&1 || true
        end_ms=$(now_ms)
        elapsed_ms=$((end_ms - start_ms))
        elapsed_s=$(awk "BEGIN {printf \"%.3f\", $elapsed_ms / 1000}")
        RESULT_TIMES+=("$elapsed_s")
        printf "  Run %d: %ss\n" "$i" "$elapsed_s"
    done

    # Compute stats.
    local sum min max
    sum=0
    min="${RESULT_TIMES[0]}"
    max="${RESULT_TIMES[0]}"
    for t in "${RESULT_TIMES[@]}"; do
        sum=$(awk "BEGIN {print $sum + $t}")
        if (( $(awk "BEGIN {print ($t < $min)}") )); then min="$t"; fi
        if (( $(awk "BEGIN {print ($t > $max)}") )); then max="$t"; fi
    done
    local mean
    mean=$(awk "BEGIN {printf \"%.3f\", $sum / $ITERATIONS}")

    # Sort for median.
    local sorted_str median
    sorted_str=$(printf '%s\n' "${RESULT_TIMES[@]}" | sort -g)
    local mid=$(( (ITERATIONS + 1) / 2 ))
    median=$(echo "$sorted_str" | sed -n "${mid}p")

    local files_per_sec mb_per_sec
    files_per_sec=$(awk "BEGIN {printf \"%.1f\", $FILE_COUNT / $median}")
    mb_per_sec=$(awk "BEGIN {printf \"%.2f\", $TOTAL_MB / $median}")

    echo ""
    echo "  Min:     ${min}s"
    echo "  Max:     ${max}s"
    echo "  Mean:    ${mean}s"
    echo "  Median:  ${median}s"
    echo "  Throughput (median): ${files_per_sec} files/s, ${mb_per_sec} MB/s"
    echo ""

    # Export for comparison.
    eval "${label}_MEDIAN=$median"
    eval "${label}_MEAN=$mean"
    eval "${label}_FPS=$files_per_sec"
    eval "${label}_MBPS=$mb_per_sec"
}

# --- Run benchmarks ---

echo "========================================"
echo "Mode: default (width=88 wrapping)"
echo "========================================"
echo ""

# Rust benchmark.
run_timed "RUST_DEFAULT" "$RUST_BIN" --check "$CORPUS_DIR"

# Python benchmark.
if [ "$SKIP_PYTHON" -eq 0 ]; then
    run_timed "PYTHON_DEFAULT" "$PYTHON_BIN" --check "$CORPUS_DIR"
fi

echo "========================================"
echo "Mode: --auto (semantic + cleanups + smartquotes + ellipses)"
echo "========================================"
echo ""

# Rust --auto benchmark.
run_timed "RUST_AUTO" "$RUST_BIN" --check --auto "$CORPUS_DIR"

# Python --auto benchmark.
if [ "$SKIP_PYTHON" -eq 0 ]; then
    run_timed "PYTHON_AUTO" "$PYTHON_BIN" --check --auto "$CORPUS_DIR"
fi

# --- Summary ---

echo "========================================"
echo "SUMMARY"
echo "========================================"
echo ""

if [ "$SKIP_PYTHON" -eq 0 ]; then
    SPEEDUP_DEFAULT=$(awk "BEGIN {printf \"%.1f\", $PYTHON_DEFAULT_MEDIAN / $RUST_DEFAULT_MEDIAN}")
    SPEEDUP_AUTO=$(awk "BEGIN {printf \"%.1f\", $PYTHON_AUTO_MEDIAN / $RUST_AUTO_MEDIAN}")

    printf "%-20s %12s %12s %12s\n" "Mode" "Rust (med)" "Python (med)" "Speedup"
    printf "%-20s %12s %12s %12s\n" "----" "----------" "------------" "-------"
    printf "%-20s %11ss %11ss %11sx\n" "default" "$RUST_DEFAULT_MEDIAN" "$PYTHON_DEFAULT_MEDIAN" "$SPEEDUP_DEFAULT"
    printf "%-20s %11ss %11ss %11sx\n" "--auto" "$RUST_AUTO_MEDIAN" "$PYTHON_AUTO_MEDIAN" "$SPEEDUP_AUTO"
    echo ""
    echo "Rust throughput (default): $RUST_DEFAULT_FPS files/s, $RUST_DEFAULT_MBPS MB/s"
    echo "Rust throughput (--auto):  $RUST_AUTO_FPS files/s, $RUST_AUTO_MBPS MB/s"
else
    printf "%-20s %12s %12s\n" "Mode" "Rust (med)" "files/s"
    printf "%-20s %12s %12s\n" "----" "----------" "-------"
    printf "%-20s %11ss %11s\n" "default" "$RUST_DEFAULT_MEDIAN" "$RUST_DEFAULT_FPS"
    printf "%-20s %11ss %11s\n" "--auto" "$RUST_AUTO_MEDIAN" "$RUST_AUTO_FPS"
fi
