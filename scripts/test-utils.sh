#!/bin/bash

# ==============================================================================
# Shared Test Utilities
# ==============================================================================

# ANSI Color Codes
export GREEN='\033[0;32m'
export RED='\033[0;31m'
export YELLOW='\033[1;33m'
export BLUE='\033[0;34m'
export BOLD='\033[1m'
export NC='\033[0m' # No Color

export PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export LOG_DIR="$PROJECT_ROOT/.test_logs"

mkdir -p "$LOG_DIR"

# Calculate workers: half of CPU cores (min 1)
CPU_CORES=$(nproc)
export NUM_WORKERS=$(( CPU_CORES / 2 ))
if [ $NUM_WORKERS -lt 1 ]; then NUM_WORKERS=1; fi

# Helper to print log sequentially
show_log_sequential() {
    local name=$1
    local log_file=$2
    local status=$3

    echo -e "\n${BOLD}--- Log for: $name ---${NC}"
    if [ $status -eq 0 ]; then
        echo -e "${GREEN}${BOLD}Result: $name PASSED${NC}"
    elif [ $status -eq 124 ]; then
        echo -e "${RED}${BOLD}Result: $name TIMED OUT${NC}"
    else
        echo -e "${RED}${BOLD}Result: $name FAILED (Exit Code: $status)${NC}"
    fi
    echo -e "${BOLD}-----------------------${NC}"
}

# Core runner: Executes tasks in parallel but displays logs in real-time when it's their turn.
run_parallel_ordered() {
    local phase_name="$1"
    shift
    
    local pids=()
    local logs=()
    local names=()
    
    echo -e "\n${YELLOW}${BOLD}🚀 $phase_name (Parallel Execution)${NC}"
    
    for task in "$@"; do
        local name=$(echo "$task" | cut -d'|' -f2)
        local dir=$(echo "$task" | cut -d'|' -f3)
        local cmd=$(echo "$task" | cut -d'|' -f4-)
        
        # Resolve dir relative to PROJECT_ROOT if needed, or just assume it is relative to where script runs?
        # The original script ran from root. These scripts might be in scripts/
        # So we should ensure 'dir' is treated correctly.
        # The original passed 'apiserver', 'agent-main' etc. which are in PROJECT_ROOT.
        
        local log_file="$LOG_DIR/${phase_name// /_}_$name.log"
        echo -e "  [${BLUE}START${NC}] $name"
        
        # We cd to PROJECT_ROOT first to ensure relative paths work
        (cd "$PROJECT_ROOT/$dir" && eval "$cmd") > "$log_file" 2>&1 & 
        pids+=($!)
        logs+=("$log_file")
        names+=("$name")
    done
    
    for i in "${!pids[@]}"; do
        local pid="${pids[$i]}"
        local log="${logs[$i]}"
        local name="${names[$i]}"

        echo -e "\n${BOLD}--- Real-time Log for: $name ---${NC}"
        # Stream log if process is still running
        if kill -0 "$pid" 2>/dev/null; then
             tail -n +1 -f "$log" --pid="$pid"
        else
             cat "$log"
        fi
        
        wait "$pid"
        local status=$?
        
        show_log_sequential "$name" "$log" $status
        
        if [ $status -ne 0 ]; then
            echo -e "\n${RED}❌ Task '$name' failed with status $status.${NC}"
            echo -e "${YELLOW}Killing other tasks in this phase...${NC}"
            # Kill all other processes in the same process group or just the captured pids
            for p in "${pids[@]}"; do
                kill "$p" 2>/dev/null || true
            done
            echo -e "${RED}Aborting pipeline.${NC}"
            exit 1
        fi
    done
    
    echo -e "\n${GREEN}✅ $phase_name Completed Successfully!${NC}"
}
