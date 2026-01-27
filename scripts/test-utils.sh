#!/bin/bash

# ==============================================================================
# Shared Test Utilities
# ==============================================================================

# ANSI Color Codes
export GREEN="\033[0;32m"
export RED="\033[0;31m"
export YELLOW="\033[1;33m"
export BLUE="\033[0;34m"
export BOLD="\033[1m"
export NC="\033[0m" # No Color

export PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export LOG_DIR="$PROJECT_ROOT/.test_logs"

mkdir -p "$LOG_DIR"

# --- Resource Auto-Detection ---
CPU_CORES=$(nproc)
# Get Total Memory in GB (integer)
# Use robust awk to handle spaces
TOTAL_MEM_KB=$(awk "/MemTotal/ {print \$2}" /proc/meminfo)
TOTAL_MEM_GB=$((TOTAL_MEM_KB / 1024 / 1024))

if [ -z "${NUM_WORKERS:-}" ]; then
    # Rule 1: Memory Bottleneck (conservative: 1 worker per 4GB RAM)
    # Rust linking is very RAM heavy. 8GB RAM -> 2 Workers max.
    WORKERS_BY_RAM=$((TOTAL_MEM_GB / 4))
    [ $WORKERS_BY_RAM -lt 1 ] && WORKERS_BY_RAM=1

    # Rule 2: CPU Bottleneck (conservative: 1 worker per 2 cores)
    WORKERS_BY_CPU=$((CPU_CORES / 2))
    [ $WORKERS_BY_CPU -lt 1 ] && WORKERS_BY_CPU=1

    # Take the minimum of the two constraints
    if [ $WORKERS_BY_RAM -lt $WORKERS_BY_CPU ]; then
        export NUM_WORKERS=$WORKERS_BY_RAM
        REASON="RAM Limit (${TOTAL_MEM_GB}GB)"
    else
        export NUM_WORKERS=$WORKERS_BY_CPU
        REASON="CPU Limit (${CPU_CORES} Cores)"
    fi
    
    # Cap at 4 for stability unless overridden
    if [ $NUM_WORKERS -gt 4 ]; then NUM_WORKERS=4; fi
    
    echo -e "${YELLOW}⚙️ Auto-tuned NUM_WORKERS to $NUM_WORKERS ($REASON)${NC}"
else
    echo -e "${YELLOW}⚙️ Using provided NUM_WORKERS: $NUM_WORKERS${NC}"
fi

# Set Cargo Build Jobs to match workers to prevent thrashing
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-$NUM_WORKERS}

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

# Core runner: Executes tasks. Run sequentially if NUM_WORKERS=1, else parallel.
# Usage: run_parallel_ordered "Phase Name" "FilterTag" "Job1|Dir|Cmd" "Job2|Dir|Cmd" ...
run_parallel_ordered() {
    local phase_name="$1"
    local filter="${2:-}"
    shift 2
    
    local pids=()
    local logs=()
    local names=()
    
    echo -e "\n${YELLOW}${BOLD}🚀 $phase_name${NC}"
    if [ -n "$filter" ]; then
        echo -e "${BLUE}🔍 Filtering jobs matching: $filter${NC}"
    fi

    # Check for Serial Mode
    local RUN_SERIAL=false
    if [ "$NUM_WORKERS" -le 1 ]; then
        RUN_SERIAL=true
        echo -e "${YELLOW}⚠️ Low resources (Workers=1). Running tasks SEQUENTIALLY to prevent freeze.${NC}"
    else
        echo -e "${BLUE}⚡ Running tasks in PARALLEL (Max $NUM_WORKERS workers implicit logic)${NC}"
    fi
    
    for task in "$@"; do
        local name=$(echo "$task" | cut -d"|" -f2)
        local dir=$(echo "$task" | cut -d"|" -f3)
        local cmd=$(echo "$task" | cut -d"|" -f4-)

        # Filtering logic
        if [ -n "$filter" ]; then
             if echo "$name" | grep -qE "$filter" || echo "$dir" | grep -qE "$filter"; then
                 : # Match found
             else
                 continue
             fi
        fi
        
        local log_file="$LOG_DIR/${phase_name// /_}_$name.log"
        echo -e "  [${BLUE}START${NC}] $name"
        
        if [ "$RUN_SERIAL" = "true" ]; then
            # SERIAL EXECUTION
            # We cd to PROJECT_ROOT first to ensure relative paths work
            (cd "$PROJECT_ROOT/$dir" && eval "$cmd") > "$log_file" 2>&1
            local status=$?
            
            # Special case for pytest
            if [[ "$name" == *"agent-zk-worker"* && $status -eq 5 ]]; then status=0; fi

            if [ $status -ne 0 ]; then
                echo -e "\n${RED}❌ Task \"$name\" failed with status $status.${NC}"
                cat "$log_file"
                echo -e "${RED}Aborting pipeline due to phase failure.${NC}"
                exit $status
            else
                show_log_sequential "$name" "$log_file" $status
            fi
        else
            # PARALLEL EXECUTION
            # We cd to PROJECT_ROOT first to ensure relative paths work
            (cd "$PROJECT_ROOT/$dir" && eval "$cmd") > "$log_file" 2>&1 & 
            pids+=($!)
            logs+=("$log_file")
            names+=("$name")
        fi
    done

    if [ "$RUN_SERIAL" = "true" ]; then
        echo -e "\n${GREEN}✅ $phase_name Completed Successfully!${NC}"
        return 0
    fi

    if [ ${#pids[@]} -eq 0 ]; then
        echo -e "${YELLOW}⚠️ No jobs matched the filter in this phase.${NC}"
        return 0
    fi
    
    local phase_status=0
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
        
        # Special case for pytest: exit code 5 means "no tests collected"
        if [[ "$name" == *"agent-zk-worker"* && $status -eq 5 ]]; then
            status=0
        fi

        show_log_sequential "$name" "$log" $status
        
        if [ $status -ne 0 ]; then
            echo -e "\n${RED}❌ Task \"$name\" failed with status $status.${NC}"
            phase_status=$status
        fi
    done
    
    if [ $phase_status -ne 0 ]; then
        echo -e "${RED}Aborting pipeline due to phase failure.${NC}"
        exit $phase_status
    fi

    echo -e "\n${GREEN}✅ $phase_name Completed Successfully!${NC}"
    return 0
}
