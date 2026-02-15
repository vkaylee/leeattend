#!/bin/bash

# ==============================================================================
# Shared Test Utilities
# ==============================================================================

# Debug: trace ERR for set -e failures
trap 'echo "DEBUG_ERR: line=$LINENO cmd=$BASH_COMMAND exit=$?" >> /tmp/test-err-trace.log' ERR

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
    # Default to 1/2 of available CPU cores now that we use RAM volumes
    export NUM_WORKERS=$(( CPU_CORES / 2 ))
    [ $NUM_WORKERS -lt 1 ] && export NUM_WORKERS=1
    
    REASON="1/2 of System Cores"
    echo -e "${YELLOW}⚙️ Auto-tuned NUM_WORKERS to $NUM_WORKERS ($REASON)${NC}"
else
    # Respect user-provided NUM_WORKERS without capping
    echo -e "${YELLOW}⚙️ Using provided NUM_WORKERS: $NUM_WORKERS${NC}"
fi

# Set Cargo Build Jobs to match workers to prevent thrashing
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-$NUM_WORKERS}
export RUST_TEST_THREADS=$NUM_WORKERS

# Results Tracking
export RESULTS_FILE="$LOG_DIR/results.txt"

add_task_result() {
    # Append to file: Name|Status|Duration
    echo "$1|$2|$3" >> "$RESULTS_FILE"
}

print_summary_table() {
    echo -e "\n${BOLD}================================================================${NC}"
    echo -e "${BOLD}                     TEST EXECUTION SUMMARY                     ${NC}"
    echo -e "${BOLD}================================================================${NC}"
    printf "${BOLD}%-30s | %-10s | %-10s${NC}\n" "Task Name" "Status" "Duration"
    echo "----------------------------------------------------------------"

    if [ -f "$RESULTS_FILE" ]; then
        while IFS='|' read -r name status duration; do
            local status_fmt=""
            if [ "$status" == "PASSED" ]; then
                status_fmt="${GREEN}PASSED${NC}"
            elif [ "$status" == "TIMED OUT" ]; then
                status_fmt="${RED}TIMEOUT${NC}"
            else
                status_fmt="${RED}FAILED${NC}"
            fi

            printf "%-30s | %-10b | %-10s\n" "$name" "$status_fmt" "${duration}s"
        done < "$RESULTS_FILE"
    else
        echo -e "${YELLOW}No results found in $RESULTS_FILE${NC}"
    fi
    echo -e "${BOLD}================================================================${NC}\n"
}

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
        echo -e "${BLUE}⚡ Running tasks in PARALLEL (Max $NUM_WORKERS workers)${NC}"
    fi

    local tasks=("$@")
    local total_tasks=${#tasks[@]}
    local current_task=0
    local running_pids=()
    local running_logs=()
    local running_names=()
    local running_starts=()
    local completed_count=0

    while [ $completed_count -lt $total_tasks ]; do
        # Start new tasks if we have room in the pool
        while [ ${#running_pids[@]} -lt "$NUM_WORKERS" ] && [ $current_task -lt $total_tasks ]; do
            local task="${tasks[$current_task]}"
            current_task=$((current_task + 1))

            local name=$(echo "$task" | cut -d"|" -f2)
            local dir=$(echo "$task" | cut -d"|" -f3)
            local cmd=$(echo "$task" | cut -d"|" -f4-)

            # Filtering logic
            if [ -n "$filter" ]; then
                 if ! (echo "$name" | grep -qE "$filter" || echo "$dir" | grep -qE "$filter"); then
                     completed_count=$((completed_count + 1))
                     continue
                 fi
            fi

            local log_file="$LOG_DIR/${phase_name// /_}_$name.log"
            local start_time=$SECONDS
            echo -e "  [${BLUE}START${NC}] $name (timeout: ${task_timeout}s)"

            if [ "$RUN_SERIAL" = "true" ]; then
                # SERIAL EXECUTION with timeout and low I/O priority
                (cd "$PROJECT_ROOT/$dir" && run_with_timeout "$task_timeout" bash -c "$cmd") > "$log_file" 2>&1
                local status=$?
                local duration=$(( SECONDS - start_time ))

                # Exit code 137 = killed by timeout (SIGKILL after timeout)
                if [ $status -eq 137 ] || [ $status -eq 124 ]; then
                    echo -e "\n${RED}⏰ Task \"$name\" TIMED OUT after ${task_timeout}s${NC}"
                    add_task_result "$name" "TIMED OUT" "$duration"
                    echo "=== TIMEOUT: Task exceeded ${task_timeout}s ===" >> "$log_file"
                    cat "$log_file"
                    echo -e "${RED}Aborting pipeline due to timeout.${NC}"
                    print_summary_table
                    exit 124
                fi

                if [ $status -ne 0 ]; then
                    echo -e "\n${RED}❌ Task \"$name\" failed with status $status.${NC}"
                    add_task_result "$name" "FAILED" "$duration"
                    cat "$log_file"
                    echo -e "${RED}Aborting pipeline due to phase failure.${NC}"
                    print_summary_table
                    exit $status
                else
                    show_log_sequential "$name" "$log_file" $status
                    add_task_result "$name" "PASSED" "$duration"
                    completed_count=$((completed_count + 1))
                fi
            else
                # PARALLEL EXECUTION with timeout and low I/O priority
                (cd "$PROJECT_ROOT/$dir" && run_with_timeout "$task_timeout" bash -c "$cmd") > "$log_file" 2>&1 &
                running_pids+=($!)
                running_logs+=("$log_file")
                running_names+=("$name")
                running_starts+=("$start_time")
            fi
        done

        if [ "$RUN_SERIAL" = "true" ]; then
            continue
        fi

        # Wait for at least one task to complete
        if [ ${#running_pids[@]} -gt 0 ]; then
            local still_running_pids=()
            local still_running_logs=()
            local still_running_names=()
            local still_running_starts=()
            local finished_any=false

            for i in "${!running_pids[@]}"; do
                local pid="${running_pids[$i]}"
                local log="${running_logs[$i]}"
                local name="${running_names[$i]}"
                local start="${running_starts[$i]}"

                if ! kill -0 "$pid" 2>/dev/null; then
                    # Task finished
                    wait "$pid"
                    local status=$?
                    local duration=$(( SECONDS - start ))

                    # Exit code 137 = killed by timeout (SIGKILL)
                    if [ $status -eq 137 ] || [ $status -eq 124 ]; then
                        echo -e "\n${RED}⏰ Task \"$name\" TIMED OUT after ${task_timeout}s${NC}"
                        add_task_result "$name" "TIMED OUT" "$duration"
                        cat "$log"
                        print_summary_table
                        exit 124
                    fi

                    show_log_sequential "$name" "$log" $status

                    if [ $status -ne 0 ]; then
                        echo -e "\n${RED}❌ Task \"$name\" failed with status $status.${NC}"
                        add_task_result "$name" "FAILED" "$duration"
                        cat "$log"
                        echo -e "${RED}Aborting pipeline due to phase failure.${NC}"
                        print_summary_table
                        exit $status
                    fi

                    add_task_result "$name" "PASSED" "$duration"
                    completed_count=$((completed_count + 1))
                    finished_any=true
                else
                    still_running_pids+=("$pid")
                    still_running_logs+=("$log")
                    still_running_names+=("$name")
                    still_running_starts+=("$start")
                fi
            done

            running_pids=(${still_running_pids[@]+"${still_running_pids[@]}"})
            running_logs=(${still_running_logs[@]+"${still_running_logs[@]}"})
            running_names=(${still_running_names[@]+"${still_running_names[@]}"})
            running_starts=(${still_running_starts[@]+"${still_running_starts[@]}"})


            if [ "$finished_any" = "false" ]; then
                local running_str=""
                for n in "${running_names[@]}"; do
                    running_str="${running_str}${n} "
                done
                echo -ne "  [${BLUE}WAIT${NC}] Running: $running_str \r"
                sleep 1
            fi
        else
            # No tasks running and no more tasks to start
            if [ $current_task -ge $total_tasks ]; then
                break
            fi
        fi
    done

    echo -e "\n${GREEN}✅ $phase_name Completed Successfully!${NC}"
    return 0
}
