#!/bin/bash
# Performance benchmark comparing du, wdu.sh, and dutop

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo "======================================"
echo "  DuTop Performance Benchmark"
echo "======================================"
echo ""

# Change to project root
cd "$(dirname "$0")/.."

# Ensure dutop is built
if [ ! -f "./target/release/dutop" ]; then
    echo "Building dutop in release mode..."
    cargo build --release
    echo ""
fi

# Function to run single benchmark
run_single() {
    local cmd=$1
    local dir=$2

    # Use time command, redirect stderr to stdout to capture timing
    { time $cmd "$dir" > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}'
}

# Function to benchmark a tool multiple times
benchmark_tool() {
    local name=$1
    local cmd=$2
    local dir=$3
    local iterations=$4

    echo -n "  $name: "

    local times=()
    for i in $(seq 1 $iterations); do
        local result=$(run_single "$cmd" "$dir")
        times+=("$result")
        echo -n "."
    done

    # Calculate average (parse time format)
    local total_ms=0
    for t in "${times[@]}"; do
        # Convert Xs or Xm format to milliseconds
        local ms=$(echo "$t" | awk '{
            val = $1
            if (index(val, "m") > 0) {
                # Minutes format: 0m1.234s
                gsub(/[ms]/, " ", val)
                split(val, parts, " ")
                print (parts[1] * 60 + parts[2]) * 1000
            } else {
                # Seconds format: 1.234s
                gsub(/s/, "", val)
                print val * 1000
            }
        }')
        total_ms=$(echo "$total_ms + $ms" | bc)
    done

    local avg_ms=$(echo "scale=2; $total_ms / $iterations" | bc)
    echo " avg: ${avg_ms}ms"

    echo "$avg_ms"
}

# Function to benchmark directory
benchmark_directory() {
    local dir=$1
    local label=$2
    local iterations=${3:-5}

    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}$label${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    if [ ! -d "$dir" ]; then
        echo -e "${RED}Directory not found: $dir${NC}"
        return
    fi

    # Get stats
    local file_count=$(find "$dir" -type f 2>/dev/null | wc -l | tr -d ' ')
    local dir_count=$(find "$dir" -type d 2>/dev/null | wc -l | tr -d ' ')
    local size=$(du -sh "$dir" 2>/dev/null | awk '{print $1}')

    echo "Path: $dir"
    echo "Size: $size | Files: $file_count | Dirs: $dir_count"
    echo ""

    # Run benchmarks
    local du_time=$(benchmark_tool "du        " "du -sk" "$dir" "$iterations")
    local wdu_time=$(benchmark_tool "wdu.sh    " "./archive/wdu.sh -n 10" "$dir" "$iterations")
    local dutop_time=$(benchmark_tool "dutop     " "./target/release/dutop -n 10" "$dir" "$iterations")

    # Results
    echo ""
    echo "┌─────────────┬───────────────┐"
    echo "│ Tool        │ Avg Time (ms) │"
    echo "├─────────────┼───────────────┤"
    printf "│ %-11s │ %13s │\n" "du" "$du_time"
    printf "│ %-11s │ %13s │\n" "wdu.sh" "$wdu_time"
    printf "│ %-11s │ %13s │\n" "dutop" "$dutop_time"
    echo "└─────────────┴───────────────┘"

    # Calculate speedups
    local vs_du=$(echo "scale=2; $du_time / $dutop_time" | bc)
    local vs_wdu=$(echo "scale=2; $wdu_time / $dutop_time" | bc)

    echo ""
    echo "Performance:"
    if (( $(echo "$vs_du >= 1" | bc -l) )); then
        printf "  vs du:     ${GREEN}%.2fx faster${NC}\n" $vs_du
    else
        printf "  vs du:     ${RED}%.2fx slower${NC}\n" $(echo "scale=2; 1/$vs_du" | bc)
    fi

    if (( $(echo "$vs_wdu >= 1" | bc -l) )); then
        printf "  vs wdu.sh: ${GREEN}%.2fx faster${NC}\n" $vs_wdu
    else
        printf "  vs wdu.sh: ${RED}%.2fx slower${NC}\n" $(echo "scale=2; 1/$vs_wdu" | bc)
    fi
}

# Run benchmarks
echo "Running $1 iterations per test..."
ITERATIONS=${1:-5}

benchmark_directory "." "Current Project" "$ITERATIONS"
benchmark_directory "/tmp" "System /tmp" "$ITERATIONS"

# Optional: Downloads if exists and readable
if [ -r "$HOME/Downloads" ]; then
    benchmark_directory "$HOME/Downloads" "Downloads Folder" 3
fi

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Benchmark Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
