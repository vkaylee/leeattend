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

# ==============================================================================
# Timeout Configuration (in seconds)
# ==============================================================================
export TIMEOUT_LINT=${TIMEOUT_LINT:-600}           # 10 min per lint task (for first compile)
export TIMEOUT_UNIT=${TIMEOUT_UNIT:-300}           # 5 min per unit test task
export TIMEOUT_INTEGRATION=${TIMEOUT_INTEGRATION:-600}  # 10 min per integration task
export TIMEOUT_BUILD=${TIMEOUT_BUILD:-600}         # 10 min for pre-compilation
export TIMEOUT_DB_WAIT=${TIMEOUT_DB_WAIT:-120}     # 2 min for DB ready check
export TIMEOUT_MIGRATION=${TIMEOUT_MIGRATION:-120} # 2 min for migrations
export TIMEOUT_GLOBAL=${TIMEOUT_GLOBAL:-1800}      # 30 min global timeout

# ==============================================================================
# WSL I/O Optimization
# ==============================================================================

# Detect WSL environment
is_wsl() {
    if grep -qiE "(microsoft|wsl)" /proc/version 2>/dev/null; then
        return 0
    fi
    if [[ -n "${WSL_DISTRO_NAME:-}" ]]; then
        return 0
    fi
    return 1
}

# Run command with low I/O priority (WSL optimization)
run_low_io() {
    if is_wsl && command -v ionice &>/dev/null; then
        ionice -c 3 "$@"
    else
        "$@"
    fi
}

# Run command with timeout and optional low I/O priority
run_with_timeout() {
    local timeout_sec=$1
    shift
    if is_wsl && command -v ionice &>/dev/null; then
        timeout --signal=KILL "${timeout_sec}s" ionice -c 3 "$@"
    else
        timeout --signal=KILL "${timeout_sec}s" "$@"
    fi
}

# ==============================================================================
# Resource Auto-Detection
# ==============================================================================
CPU_CORES=$(nproc)
TOTAL_MEM_KB=$(awk "/MemTotal/ {print \$2}" /proc/meminfo)
TOTAL_MEM_GB=$((TOTAL_MEM_KB / 1024 / 1024))

if [ -z "${NUM_WORKERS:-}" ]; then
    # Default to 1/4 of available CPU cores to prevent system freeze
    export NUM_WORKERS=$(( CPU_CORES / 4 ))
    [ $NUM_WORKERS -lt 1 ] && export NUM_WORKERS=1
    REASON="1/4 of System Cores"

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

# Determine timeout based on phase name
get_phase_timeout() {
    local phase_name="$1"
    case "$phase_name" in
        *Lint*) echo "$TIMEOUT_LINT" ;;
        *Unit*) echo "$TIMEOUT_UNIT" ;;
        *Integration*|*E2E*) echo "$TIMEOUT_INTEGRATION" ;;
        *) echo "$TIMEOUT_UNIT" ;;
    esac
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
    local task_timeout=$(get_phase_timeout "$phase_name")

    echo -e "\n${YELLOW}${BOLD}🚀 $phase_name${NC}"
    echo -e "${BLUE}⏱️ Task timeout: ${task_timeout}s${NC}"
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
        echo -e "  [${BLUE}START${NC}] $name (timeout: ${task_timeout}s)"

        if [ "$RUN_SERIAL" = "true" ]; then
            # SERIAL EXECUTION with timeout and low I/O priority
            (cd "$PROJECT_ROOT/$dir" && run_with_timeout "$task_timeout" bash -c "$cmd") > "$log_file" 2>&1
            local status=$?

            # Exit code 137 = killed by timeout (SIGKILL after timeout)
            if [ $status -eq 137 ] || [ $status -eq 124 ]; then
                echo -e "\n${RED}⏰ Task \"$name\" TIMED OUT after ${task_timeout}s${NC}"
                echo "=== TIMEOUT: Task exceeded ${task_timeout}s ===" >> "$log_file"
                cat "$log_file"
                echo -e "${RED}Aborting pipeline due to timeout.${NC}"
                exit 124
            fi

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
            # PARALLEL EXECUTION with timeout and low I/O priority
            (cd "$PROJECT_ROOT/$dir" && run_with_timeout "$task_timeout" bash -c "$cmd") > "$log_file" 2>&1 &
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

        # Exit code 137 = killed by timeout (SIGKILL)
        if [ $status -eq 137 ] || [ $status -eq 124 ]; then
            echo -e "\n${RED}⏰ Task \"$name\" TIMED OUT after ${task_timeout}s${NC}"
            phase_status=124
            continue
        fi

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
