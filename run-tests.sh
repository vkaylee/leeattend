#!/bin/bash

# ==============================================================================
# Unified Test Orchestrator for LeeAttend (Real-time Sequential Output)
# Parallel Workers: 4
# ==============================================================================

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.test.yml"
LOG_DIR="$PROJECT_ROOT/.test_logs"

# Calculate workers: half of CPU cores (min 1)
CPU_CORES=$(nproc)
NUM_WORKERS=$(( CPU_CORES / 2 ))
if [ $NUM_WORKERS -lt 1 ]; then NUM_WORKERS=1; fi

echo -e "💻 System: $CPU_CORES cores detected. Using $NUM_WORKERS workers."

# ANSI Color Codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Ensure log directory exists
mkdir -p "$LOG_DIR"

cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up test environment...${NC}"
    # Stop all containers and remove volumes (-v)
    docker-compose -f "$COMPOSE_FILE" down -v > /dev/null 2>&1
    
    # Remove any leftover signal files if necessary
    rm -f apiserver/.api_port* webdashboard/.web_port*
    
    echo -e "${GREEN}Done.${NC}"
}

trap cleanup EXIT INT TERM

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
        
        local log_file="$LOG_DIR/${phase_name// /_}_$name.log"
        echo -e "  [${BLUE}START${NC}] $name"
        
        (cd "$dir" && eval "$cmd") > "$log_file" 2>&1 &
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

echo -e "${BLUE}${BOLD}🚀 LeeAttend Unified Test Runner${NC}"

# --- PHASE 0: LINTING ---
run_parallel_ordered "Phase 0 (Linting)" \
    "1|apiserver-clippy|apiserver|cargo clippy -- -D warnings" \
    "2|agent-main-clippy|agent-main|cargo clippy -- -D warnings" \
    "3|webdashboard-lint|webdashboard|npm run lint -- --max-warnings 0"

# --- PHASE 1: UNIT TESTS ---
run_parallel_ordered "Phase 1 (Unit Tests)" \
    "1|apiserver|apiserver|cargo test --lib" \
    "2|agent-main|agent-main|cargo test --lib" \
    "3|agent-zk-worker|agent-zk-worker|venv/bin/pytest" \
    "4|webdashboard|webdashboard|npm run test"

# --- PHASE 2: DATABASE SETUP ---
echo -e "\n${YELLOW}${BOLD}🗄️ Phase 2: Setting up Postgres Environment${NC}"

docker-compose -f "$COMPOSE_FILE" up -d test_db > /dev/null 2>&1

echo -n "  Waiting for database to be ready..."
until docker-compose -f "$COMPOSE_FILE" exec test_db pg_isready -U test_user -d test_database > /dev/null 2>&1; do
    printf "."
    sleep 1
done
echo -e " ${GREEN}Ready!${NC}"

echo -e "  Creating isolated databases for $NUM_WORKERS workers..."
for i in $(seq 0 $((NUM_WORKERS-1))); do
    docker-compose -f "$COMPOSE_FILE" exec test_db psql -U test_user -d postgres -c "CREATE DATABASE test_database_e2e_$i" > /dev/null 2>&1 || true
    echo -e "    DB e2e_$i ready."
done

echo -e "  Running migrations..."
(cd apiserver && sqlx migrate run --database-url "postgres://test_user:test_password@localhost:5433/test_database") > "$LOG_DIR/mig_int.log" 2>&1 || { echo -e "${RED}Migration failed for main test DB${NC}"; exit 1; }

for i in $(seq 0 $((NUM_WORKERS-1))); do
    (cd apiserver && sqlx migrate run --database-url "postgres://test_user:test_password@localhost:5433/test_database_e2e_$i") > "$LOG_DIR/mig_e2e_$i.log" 2>&1 || { echo -e "${RED}Migration failed for worker $i${NC}"; exit 1; }
    echo -e "    Migrations e2e_$i applied."
done

echo -e "  Pre-compiling backend for integration tests..."
(cd apiserver && cargo build) > "$LOG_DIR/prebuild.log" 2>&1 || { echo -e "${RED}Backend build failed${NC}"; exit 1; }
echo -e "  ${GREEN}Infrastructure ready.${NC}"

# --- PHASE 3: INTEGRATION TESTS ---
export DATABASE_URL="postgres://test_user:test_password@localhost:5433/test_database"

run_parallel_ordered "Phase 3 (Integration & E2E)" \
    "1|apiserver-int|apiserver|timeout 300s cargo test --test '*' -- --test-threads $NUM_WORKERS" \
    "2|playwright-e2e|webdashboard|timeout 300s npx playwright test --workers $NUM_WORKERS"

echo -e "\n${GREEN}${BOLD}🎉 All tests passed successfully!${NC}"
