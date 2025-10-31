#!/bin/zsh

# Quick disk usage display - optimized version
# Shows disk usage of current directory or specified directory
# Original thanks to bolk http://bolknote.ru/2011/09/14/~3407#07

set -eo pipefail

# Configuration
DEFAULT_LIST_LENGTH=10
MIN_BAR_WIDTH=20
MAX_BAR_WIDTH=40
SIZE_COL_WIDTH=7
MIN_NAME_WIDTH=20

# Initialize colors based on terminal capabilities
declare GREEN YELLOW RED RESET

# Only use colors if outputting to a terminal
if [[ -t 1 ]] || [[ "${FORCE_COLOR:-}" == "1" ]]; then
    # Try 256-color mode if supported, otherwise fall back to 8-color
    if [[ ${TERM:-} =~ 256color ]] || [[ ${COLORTERM:-} =~ (truecolor|24bit) ]]; then
        # 256-color mode
        GREEN=$'\033[38;5;34m'
        YELLOW=$'\033[38;5;220m'
        RED=$'\033[38;5;160m'
        RESET=$'\033[0m'
    else
        # Basic 8-color mode (more compatible)
        GREEN=$'\033[32m'
        YELLOW=$'\033[33m'
        RED=$'\033[31m'
        RESET=$'\033[0m'
    fi
else
    # No colors for non-terminal output
    GREEN=""
    YELLOW=""
    RED=""
    RESET=""
fi

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS] [DIRECTORY]

Display disk usage in a graphical bar chart format.

Options:
  -n NUMBER     Number of items to display (default: $DEFAULT_LIST_LENGTH)
  -d DEPTH      Maximum depth for du (default: 1)
  -h, --help    Show this help message

Arguments:
  DIRECTORY     Directory to analyze (default: current directory)

Examples:
  $(basename "$0")              # Current directory, top 10
  $(basename "$0") -n 20        # Current directory, top 20
  $(basename "$0") ~/Downloads  # Downloads folder
  $(basename "$0") -n 15 ~/work # Work folder, top 15
EOF
    exit 0
}

main() {
    local list_length=$DEFAULT_LIST_LENGTH
    local target_dir="."
    local max_depth=1

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                ;;
            -n)
                list_length="$2"
                shift 2
                ;;
            -d)
                max_depth="$2"
                shift 2
                ;;
            -*)
                echo "Unknown option: $1" >&2
                usage
                ;;
            *)
                target_dir="$1"
                shift
                ;;
        esac
    done

    # Validate directory
    if [[ ! -d "$target_dir" ]]; then
        echo "Error: Directory '$target_dir' not found" >&2
        exit 1
    fi

    if [[ ! -r "$target_dir" ]]; then
        echo "Error: Cannot read directory '$target_dir'" >&2
        exit 1
    fi

    # Change to target directory
    cd "$target_dir" || exit 1

    local full_path=$(pwd)
    echo "Analyzing: $full_path"

    # Refuse to scan home directory (too slow due to Library/ caches)
    if [[ "$full_path" == "$HOME" ]]; then
        echo ""
        echo "⚠️  Cannot scan home directory (too slow due to Library/ caches)"
        echo ""
        echo "Showing quick summary of common directories instead:"
        echo ""

        # Show quick du -sh for common directories
        local dirs=("Documents" "Downloads" "Desktop" "Pictures" "Movies" "Music" "work" "projects")
        for dir in "${dirs[@]}"; do
            if [[ -d "$HOME/$dir" ]]; then
                local size=$(command du -sh "$HOME/$dir" 2>/dev/null | awk '{print $1}')
                printf "  %-12s %s\n" "$size" "~/$dir"
            fi
        done

        echo ""
        echo "To analyze a specific directory, run: wdu ~/Documents (or any other path)"
        return 0
    fi

    echo "Scanning directories..."

    # Get disk usage (only immediate children, not recursive)
    local du_output
    local du_numeric_output
    local du_all_output
    local max_size_cmd
    local total_size

    # Use du to get all items (no filtering - shows everything including venv/, node_modules/, .git/, etc.)
    # Use 'command du' to bypass any shell alias (like 'du -h')
    # Note: -s (summary) with glob patterns ./* and ./.[!.]* already gives immediate children only
    # Disable pipefail temporarily for glob expansion and enable NULL_GLOB to handle empty directories
    setopt NULL_GLOB 2>/dev/null || true
    set +o pipefail

    # Get all items for total calculation
    du_all_output=$(command du -sk ./* ./.[!.]* 2>/dev/null)

    # Calculate total size of all items
    total_size=$(echo "$du_all_output" | awk '{sum += $1} END {print sum}')

    # Get top N items for display
    du_numeric_output=$(echo "$du_all_output" | sort -rn | head -n "$list_length")
    unsetopt NULL_GLOB 2>/dev/null || true
    # Convert to human-readable format (using awk)
    # Note: du -sk outputs in KB
    du_output=$(echo "$du_numeric_output" | awk 'BEGIN {FS="\t"; OFS="\t"} {
        size = $1
        item = $2

        # Convert KB to human-readable
        if (size >= 1048576) printf "%.1fG\t%s\n", size/1048576, item
        else if (size >= 1024) printf "%.0fM\t%s\n", size/1024, item
        else printf "%.0fK\t%s\n", size, item
    }' || true)
    max_size_cmd=$(echo "$du_numeric_output" | head -1 | awk '{print $1}')
    set -o pipefail

    if [[ -z "$du_output" ]]; then
        echo "┌────────────────────┐"
        echo "│ No files found     │"
        echo "└────────────────────┘"
        return 0
    fi

    # Get max size for scaling (numeric)
    local max_size="${max_size_cmd:-1}"
    [[ -z "$max_size" || "$max_size" -eq 0 ]] && max_size=1

    # Create associative array to cache numeric sizes (avoid redundant du calls)
    typeset -A size_cache
    while IFS=$'\t' read -r size item; do
        [[ -z "${item:-}" ]] && continue
        size_cache[$item]=$size
    done <<< "$du_numeric_output"

    # Calculate optimal column widths
    local term_width=${COLUMNS:-$(tput cols 2>/dev/null || echo 80)}

    # Find longest filename
    local max_name_len=0
    while IFS=$'\t' read -r _ item; do
        [[ -z "${item:-}" ]] && continue
        local item_len=${#item}
        (( item_len > max_name_len )) && max_name_len=$item_len
    done <<< "$du_output"

    # Calculate widths: │ BAR │ SIZE │ NAME │
    # Fixed: borders (6 chars: "│ ", " │ ", " │ ", " │")
    local fixed_width=$((6 + SIZE_COL_WIDTH))
    local available=$((term_width - fixed_width))

    # Allocate space: prefer bar width, then name
    local bar_width=$MIN_BAR_WIDTH
    local name_width=$max_name_len

    # Adjust if content is too wide
    if (( bar_width + name_width > available )); then
        # Prioritize name, adjust bar if needed
        name_width=$((available - bar_width))
        if (( name_width < MIN_NAME_WIDTH )); then
            name_width=$MIN_NAME_WIDTH
            bar_width=$((available - name_width))
            (( bar_width < MIN_BAR_WIDTH )) && bar_width=$MIN_BAR_WIDTH
        fi
    fi

    # Cap bar width at maximum
    (( bar_width > MAX_BAR_WIDTH )) && bar_width=$MAX_BAR_WIDTH

    # Ensure minimum widths
    (( name_width < MIN_NAME_WIDTH )) && name_width=$MIN_NAME_WIDTH

    # Build box borders
    local bar_border=$(printf '─%.0s' $(seq 1 $((bar_width + 2))))
    local size_border=$(printf '─%.0s' $(seq 1 $((SIZE_COL_WIDTH + 2))))
    local name_border=$(printf '─%.0s' $(seq 1 $((name_width + 2))))

    echo "┌${bar_border}┬${size_border}┬${name_border}┐"

    # Display each item
    while IFS=$'\t' read -r human_size item; do
        [[ -z "${item:-}" ]] && continue

        # Get numeric size from cache (avoiding redundant du call)
        local size=${size_cache[$item]:-0}
        [[ -z "$size" ]] && size=0

        # Calculate bar length
        local percent=$(( (size * bar_width) / max_size ))
        (( percent > bar_width )) && percent=$bar_width
        (( percent < 0 )) && percent=0

        # Select color based on percentage of bar width
        local color=$GREEN
        local threshold_yellow=$((bar_width * 33 / 100))
        local threshold_red=$((bar_width * 50 / 100))
        (( percent >= threshold_yellow )) && color=$YELLOW
        (( percent >= threshold_red )) && color=$RED

        # Create bar
        local bar=""
        if (( percent > 0 )); then
            bar=$(printf "%${percent}s" | tr ' ' '█')
        fi
        local empty=$(printf "%$((bar_width - percent))s" | tr ' ' '░')

        # Truncate filename if needed
        local display_name="$item"
        if (( ${#display_name} > name_width )); then
            display_name="${display_name:0:$((name_width - 3))}..."
        fi

        # Output line with proper alignment
        printf "│ ${color}%-${bar_width}s${RESET} │ %${SIZE_COL_WIDTH}s │ %-${name_width}s │\n" \
            "${bar}${empty}" "$human_size" "$display_name"

    done <<< "$du_output"

    echo "└${bar_border}┴${size_border}┴${name_border}┘"

    # Display total storage
    local total_human
    if [[ -n "$total_size" && "$total_size" -gt 0 ]]; then
        # Convert total to human-readable format
        if (( total_size >= 1048576 )); then
            total_human=$(awk -v size="$total_size" 'BEGIN {printf "%.1fG", size/1048576}')
        elif (( total_size >= 1024 )); then
            total_human=$(awk -v size="$total_size" 'BEGIN {printf "%.0fM", size/1024}')
        else
            total_human=$(awk -v size="$total_size" 'BEGIN {printf "%.0fK", size}')
        fi
        echo "Total: $total_human"
    fi
}

main "$@"
