#!/bin/bash

# ==============================================================================
# LeeAttend Debug Log Utilities
# This script defines methods for analyzing test logs.
# Sourced by other scripts, not intended for direct execution.
# ==============================================================================

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
    local last_log=$(ls -t "$dir"/*.log 2>/dev/null | head -n 1)
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
    local file=$(ls "$dir"/*"$pattern"* 2>/dev/null | head -n 1)
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
        grep -Ei "error|panic|failed|exception|backtrace" "$file" || tail -n 20 "$file"
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
    local last_log=$(ls -t "$dir"/*.log 2>/dev/null | head -n 1)
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
        
        # Check if the log actually contains a failure
        if grep -Ei "FAIL|panic|error" "$log" > /dev/null; then
            failed_found=true
            local name=$(basename "$log" | sed 's/\.log$//')
            echo -e "${YELLOW}● $name:${NC}"
            
            # Extract specific failed test names based on runner type
            if grep -q "Nextest" "$log"; then
                # Rust Nextest format: FAIL [ 0.1s] crate::module::test_name
                grep "FAIL \[" "$log" | awk '{print "  ✖ " $NF}'
            elif grep -q "pytest" "$log"; then
                # Python Pytest format: FAILED tests/path.py::test_name - Error
                grep "FAILED " "$log" | cut -d' ' -f2 | awk '{print "  ✖ " $1}'
            elif grep -q "playwright" "$log"; then
                # Playwright format: 1) [chromium] › test.spec.ts:12:3 › test name
                grep -E "[0-9]+\) \[" "$log" | awk -F " › " '{print "  ✖ " $NF}'
            else
                # Fallback: show last few lines if pattern unknown
                tail -n 5 "$log" | sed 's/^/  /'
            fi
        fi
    done

    if [ "$failed_found" = false ]; then
        echo -e "${GREEN}  No failures found in logs.${NC}"
    fi
}

