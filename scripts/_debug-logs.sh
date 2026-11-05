#!/bin/bash

# Internal module; source from the LeeDevKit test runner.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "This script is an internal module. Use ./leedevkit test for project checks."
    exit 1
fi

# Default log directory if not set
DEBUG_LOG_DIR="${LOG_DIR:-.test_logs}"

# Colors (with defaults)
BLUE="${BLUE:-\033[0;34m}"
YELLOW="${YELLOW:-\033[1;33m}"
RED="${RED:-\033[0;31m}"
NC="${NC:-\033[0m}"

leeattend_debug_list() {
    local dir="${1:-$DEBUG_LOG_DIR}"
    echo -e "${BLUE}📁 Listing all logs in $dir:${NC}"
    ls -lh "$dir"
}

leeattend_debug_last() {
    local dir="${1:-$DEBUG_LOG_DIR}"
    local last_log
    last_log=$(printf '%s\n' "$dir"/*.log | head -n 1)  # pty-safe: local file glob
    [[ "$last_log" == "$dir/*.log" ]] && last_log=""
    if [ -n "$last_log" ]; then
        echo -e "${BLUE}📄 Content of last log: $last_log${NC}"
        cat "$last_log"
    else
        echo -e "${YELLOW}No log files found in $dir.${NC}"
    fi
}

leeattend_debug_cat() {
    local pattern="$1"
    local dir="${2:-$DEBUG_LOG_DIR}"
    local file
    file=$(printf '%s\n' "$dir"/*"$pattern"* | head -n 1)  # pty-safe: local file glob
    [[ "$file" == *'*'* ]] && file=""
    if [ -n "$file" ] && [ -f "$file" ]; then
        echo -e "${BLUE}📄 Content of log: $file${NC}"
        cat "$file"
    else
        echo -e "${RED}File matching *$pattern* not found in $dir${NC}"
    fi
}

leeattend_debug_file() {
    local file="$1"
    if [ -f "$file" ]; then
        echo -e "${BLUE}📄 Analyzing file: $file${NC}"
        # Search for errors in this specific log
        grep -Ei "error|panic|failed|exception|backtrace" "$file" || tail -n 20 "$file"  # not-a-pipe: || is fallback, not |
    else
        echo -e "${RED}File not found: $file${NC}"
    fi
}

leeattend_debug_errors() {
    local dir="${1:-$DEBUG_LOG_DIR}"
    echo -e "${YELLOW}🔍 Searching for errors/panics in $dir...${NC}"
    # Using GREP_COLORS to highlight errors if supported
    grep -Ei "error|panic|failed|exception|backtrace" "$dir"/*.log || echo "No common error patterns found."
}

leeattend_debug_tail() {
    local n="${1:-20}"
    local dir="${2:-$DEBUG_LOG_DIR}"
    local last_log
    last_log=$(printf '%s\n' "$dir"/*.log | head -n 1)  # pty-safe: local file glob
    [[ "$last_log" == "$dir/*.log" ]] && last_log=""
    if [ -n "$last_log" ]; then
        echo -e "${BLUE}📄 Last $n lines of $last_log:${NC}"
        tail -n "$n" "$last_log"
    else
        echo -e "${YELLOW}No log files found in $dir.${NC}"
    fi
}

leeattend_summarize_failures() {
    local dir="${1:-$DEBUG_LOG_DIR}"
    local failed_found=false

    echo -e "\n${RED}${BOLD}🔍 FAILED TEST SUMMARY:${NC}"

    for log in "$dir"/*.log; do
        [ -e "$log" ] || continue

        if grep -Eiq "FAIL|panic|error" "$log"; then
            failed_found=true
            local name
            name=$(basename "$log" | sed 's/\.log$//')
            echo -e "${YELLOW}● $name:${NC}"

            if grep -q "pytest" "$log"; then
                grep "FAILED " "$log" | cut -d' ' -f2 | awk '{print "  ✖ " $1}'
            elif grep -qi "playwright" "$log"; then
                grep -E "[0-9]+\) \[" "$log" | awk -F " › " '{print "  ✖ " $NF}'
            else
                tail -n 5 "$log" | sed 's/^/  /'
            fi
        fi
    done

    if [ "$failed_found" = false ]; then
        echo -e "${GREEN}  No failures found in logs.${NC}"
    fi
}

