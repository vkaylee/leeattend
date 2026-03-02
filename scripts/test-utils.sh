#!/bin/bash

# ==============================================================================
# ⚠️ INTERNAL MODULE — DO NOT RUN DIRECTLY
# Sourced by: ./run-tests.sh
# ==============================================================================
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "❌ This script is an internal module. Do not run directly."
    echo "👉 Use: ./run-tests.sh --help"
    exit 1
fi
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

if [[ -n "${NO_COLOR:-}" ]]; then
    export GREEN='' RED='' YELLOW='' BLUE='' BOLD='' NC=''
fi

export PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export LOG_DIR="$PROJECT_ROOT/.test_logs"

# Source debug utilities
source "$PROJECT_ROOT/scripts/debug-logs.sh"

mkdir -p "$LOG_DIR"

# ==============================================================================
# Timeout Configuration (in seconds)
# ==============================================================================
export TIMEOUT_LINT=${TIMEOUT_LINT:-900}           # 15 min per lint task (for first compile)
export TIMEOUT_UNIT=${TIMEOUT_UNIT:-900}           # 15 min per unit test task
export TIMEOUT_INTEGRATION=${TIMEOUT_INTEGRATION:-1200}  # 20 min per integration task
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
CPU_CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
TOTAL_MEM_KB=$(awk "/MemTotal/ {print \$2}" /proc/meminfo 2>/dev/null || echo 8388608)
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

# Helper to handle task failure
on_task_failure() {
    local name=$1
    local log_file=$2
    local status=$3

    echo -e "\n${RED}${BOLD}❌ Task \"$name\" failed with status $status.${NC}"
    
    # Create symlink for easier access
    ln -sf "$log_file" "$LOG_DIR/latest-failure.log"

    # Show concise summary of failures in this specific file
    leeattend_summarize_failures "$(dirname "$log_file")" | grep -A 10 "$name"
    
    echo -e "${YELLOW}--- FULL LOG CONTENT FOR $name ---${NC}"
    if [ -f "$log_file" ]; then
        # If log is huge, show last 200 lines, otherwise show all
        local line_count=$(wc -l < "$log_file")
        if [ "$line_count" -gt 200 ]; then
            echo -e "${BLUE}(Log is large, showing last 200 lines)${NC}"
            tail -n 200 "$log_file"
        else
            cat "$log_file"
        fi
    else
        echo -e "${RED}Log file not found: $log_file${NC}"
    fi
    echo -e "${YELLOW}----------------------------------${NC}"
    
    echo -e "${BLUE}💡 Tip: Full log at $log_file${NC}"
    echo -e "${BLUE}💡 Tip: Run 'cat .test_logs/latest-failure.log' to see full error.${NC}"
}

# Helper to handle task timeout
on_task_timeout() {
    local name=$1
    local log_file=$2
    local timeout=$3

    echo -e "\n${RED}${BOLD}⏰ Task \"$name\" TIMED OUT after ${timeout}s${NC}"
    
    # Create symlink for easier access
    ln -sf "$log_file" "$LOG_DIR/latest-failure.log"

    echo -e "${YELLOW}--- LOG CONTENT (LAST SEEN) FOR $name ---${NC}"
    if [ -f "$log_file" ]; then
        tail -n 100 "$log_file"
    else
        echo -e "${RED}Log file not found: $log_file${NC}"
    fi
    echo -e "${YELLOW}-----------------------------------------${NC}"
    
    echo -e "${BLUE}💡 Tip: Full log at $log_file${NC}"
    echo -e "${BLUE}💡 Tip: Run 'cat .test_logs/latest-failure.log' to see where it hung.${NC}"
}

# Core runner: Executes tasks. Run sequentially if NUM_WORKERS=1, else parallel.
# Usage: run_parallel_ordered "Phase Name" "FilterTag" "Job1|Dir|Cmd" "Job2|Dir|Cmd" ...
run_parallel_ordered() {
    local phase_name="$1"
    local filter="${2:-}"
    shift 2

    local tasks=("$@")
    local filtered_tasks=()
    
    # 1. Pre-filter tasks
    for task in "${tasks[@]}"; do
        local name=$(echo "$task" | cut -d"|" -f2)
        local dir=$(echo "$task" | cut -d"|" -f3)
        if [ -n "$filter" ]; then
            if (echo "$name" | grep -qE "$filter" || echo "$dir" | grep -qE "$filter"); then
                filtered_tasks+=("$task")
            fi
        else
            filtered_tasks+=("$task")
        fi
    done

    local total_tasks=${#filtered_tasks[@]}
    if [ "$total_tasks" -eq 0 ]; then
        return 0
    fi

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

    local current_task=0
    local running_pids=()
    local running_logs=()
    local running_names=()
    local running_starts=()
    local completed_count=0
    local tail_pids=()  # Must be declared HERE (outside loop) to persist across iterations

    while [ $completed_count -lt $total_tasks ]; do
        # Start new tasks if we have room in the pool
        while [ ${#running_pids[@]} -lt "$NUM_WORKERS" ] && [ $current_task -lt $total_tasks ]; do
            local task="${filtered_tasks[$current_task]}"
            current_task=$((current_task + 1))

            local name=$(echo "$task" | cut -d"|" -f2)
            local dir=$(echo "$task" | cut -d"|" -f3)
            local cmd=$(echo "$task" | cut -d"|" -f4-)

            local log_file="$LOG_DIR/${phase_name// /_}_$name.log"
            local start_time=$SECONDS
            echo -e "  [${BLUE}START${NC}] $name (timeout: ${task_timeout}s)"

            if [ "$RUN_SERIAL" = "true" ]; then
                # SERIAL EXECUTION — Stream live output safely
                > "$log_file" # truncate
                
                # Chạy test trong background nhưng lấy PID ngay lập tức
                set +e
                (cd "$PROJECT_ROOT/$dir" && run_with_timeout "$task_timeout" bash -c "$cmd" </dev/null > "$log_file" 2>&1) &
                local test_pid=$!
                
                # tail -f sẽ tự động thoát khi test_pid kết thúc (PTY-safe)
                tail -f "$log_file" --pid=$test_pid 2>/dev/null || true
                
                # Chờ lấy exit code thực sự của test
                wait $test_pid
                local status=$?
                set -e
                set -e
                local duration=$(( SECONDS - start_time ))

                if [ $status -eq 137 ] || [ $status -eq 124 ]; then
                    add_task_result "$name" "TIMED OUT" "$duration"
                    echo "=== TIMEOUT: Task exceeded ${task_timeout}s ===" >> "$log_file"
                    on_task_timeout "$name" "$log_file" "$task_timeout"
                    exit 124
                fi

                if [ $status -ne 0 ]; then
                    on_task_failure "$name" "$log_file" "$status"
                    add_task_result "$name" "FAILED" "$duration"
                    exit $status
                else
                    show_log_sequential "$name" "$log_file" $status
                    add_task_result "$name" "PASSED" "$duration"
                    completed_count=$((completed_count + 1))
                fi
            else
                # PARALLEL EXECUTION
                (cd "$PROJECT_ROOT/$dir" && run_with_timeout "$task_timeout" bash -c "$cmd") </dev/null > "$log_file" 2>&1 &
                running_pids+=($!)
                running_logs+=("$log_file")
                running_names+=("$name")
                running_starts+=("$start_time")
            fi
        done

        if [ "$RUN_SERIAL" = "true" ]; then continue; fi

        # Wait for at least one task to complete
        if [ ${#running_pids[@]} -gt 0 ]; then
            echo -e "  [WAIT] Running: ${running_names[*]}  \n"
            
            # Start a background tail to show progress (PASS/FAIL/TIMEOUT lines only)
            # This gives the user feedback that tests are actually running, not frozen
            for log in "${running_logs[@]}"; do
                touch "$log" # ensure file exists
                # Extract only important test execution lines to prevent flooding
                tail -f "$log" 2>/dev/null | grep --line-buffered -E "^(    PASS|    FAIL|    TIMEOUT|panicked|test result:)" </dev/null &
                tail_pids+=($!)
            done

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
                    set +e
                    wait "$pid"
                    local status=$?
                    set -e
                    local duration=$(( SECONDS - start ))

                    if [ $status -eq 137 ] || [ $status -eq 124 ]; then
                        add_task_result "$name" "TIMED OUT" "$duration"
                        on_task_timeout "$name" "$log" "$task_timeout"
                        exit 124
                    fi

                    show_log_sequential "$name" "$log" $status

                    if [ $status -ne 0 ]; then
                        on_task_failure "$name" "$log" "$status"
                        add_task_result "$name" "FAILED" "$duration"
                        sleep 2
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

            set +e
            running_pids=(${still_running_pids[@]+"${still_running_pids[@]}"})
            running_logs=(${still_running_logs[@]+"${still_running_logs[@]}"})
            running_names=(${still_running_names[@]+"${still_running_names[@]}"})
            running_starts=(${still_running_starts[@]+"${still_running_starts[@]}"})
            set -e

            if [ "$finished_any" = "false" ]; then
                local running_str=""
                for n in "${running_names[@]}"; do running_str="${running_str}${n} "; done
                echo -ne "  [${BLUE}WAIT${NC}] Running: $running_str \r"
                sleep 1
            fi
        fi
    done

    # Cleanup ALL tail streaming processes (must be AFTER the loop)
    for tp in "${tail_pids[@]+"${tail_pids[@]}"}"; do
        kill "$tp" 2>/dev/null || true
    done
    # Wait to suppress bash "Terminated" / "Killed" messages
    for tp in "${tail_pids[@]+"${tail_pids[@]}"}"; do
        wait "$tp" 2>/dev/null || true
    done

    echo -ne "                                                                \r"
    echo -e "${GREEN}✅ $phase_name Completed Successfully!${NC}"
    return 0
}
