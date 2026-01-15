#!/bin/bash

# ==============================================================================
# Unified Test Orchestrator for LeeAttend (Sequential Log Output)
# 1. Parallel Unit Tests
# 2. Database Environment Setup
# 3. Parallel Integration Tests
# ==============================================================================

set -uo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.test.yml"
LOG_DIR="$PROJECT_ROOT/.test_logs"

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
    docker-compose -f "$COMPOSE_FILE" down -v > /dev/null 2>&1
    echo -e "${GREEN}Done.${NC}"
}

trap cleanup EXIT

echo -e "${BLUE}${BOLD}🚀 LeeAttend Test Orchestrator${NC}"

# Helper to print log sequentially
show_log_sequential() {
    local name=$1
    local log_file=$2
    local status=$3

    echo -e "\n${BOLD}--- Log for: $name ---${NC}"
    if [ -f "$log_file" ]; then
        cat "$log_file"
    else
        echo -e "${RED}Log file not found.${NC}"
    fi

    if [ $status -eq 0 ]; then
        echo -e "${GREEN}${BOLD}Result: $name PASSED${NC}"
    else
        echo -e "${RED}${BOLD}Result: $name FAILED (Exit Code: $status)${NC}"
    fi
    echo -e "${BOLD}-----------------------${NC}"
}

# --- PHASE 1: PARALLEL UNIT TESTS ---
echo -e "\n${YELLOW}📦 Phase 1: Running Unit Tests (Parallel Execution)${NC}"

# Define tasks
tasks=(
    "apiserver:cargo test --lib:apiserver"
    "agent-main:cargo test --lib:agent-main"
    "agent-zk-worker:venv/bin/pytest:agent-zk-worker"
    "webdashboard:npm run test:webdashboard"
)

pids=()
logs=()
names=()

for task in "${tasks[@]}"; do
    IFS=":" read -r name cmd dir <<< "$task"
    log_file="$LOG_DIR/unit_$name.log"
    echo -e "  [${BLUE}START${NC}] $name Unit Tests..."
    
    (cd "$dir" && eval "$cmd") > "$log_file" 2>&1 &
    pids+=($!)
    logs+=("$log_file")
    names+=("$name")
done

echo -e "  ${BLUE}Waiting for all unit tests to finish...${NC}"

unit_failed=0
for i in "${!pids[@]}"; do
    wait "${pids[$i]}"
    status=$?
    show_log_sequential "${names[$i]}" "${logs[$i]}" $status
    if [ $status -ne 0 ]; then
        unit_failed=1
    fi
done

if [ $unit_failed -ne 0 ]; then
    echo -e "\n${RED}❌ Some unit tests failed. Aborting integration tests.${NC}"
    exit 1
fi

echo -e "\n${GREEN}✅ All Unit Tests Passed!${NC}"

# --- PHASE 2: DATABASE SETUP ---
echo -e "\n${YELLOW}🗄️ Phase 2: Setting up Postgres Environment${NC}"

docker-compose -f "$COMPOSE_FILE" up -d test_db > /dev/null 2>&1

echo -n "  Waiting for database to be ready..."
MAX_RETRIES=30
COUNT=0
until docker-compose -f "$COMPOSE_FILE" exec test_db pg_isready -U test_user -d test_database > /dev/null 2>&1; do
    printf "."
    sleep 1
    COUNT=$((COUNT+1))
    if [ $COUNT -ge $MAX_RETRIES ]; then
        echo -e "\n${RED}❌ Database failed to start in time.${NC}"
        exit 1
    fi
done
echo -e " ${GREEN}Ready!${NC}"

# --- PHASE 3: PARALLEL INTEGRATION TESTS ---
echo -e "\n${YELLOW}🔗 Phase 3: Running Integration & E2E Tests (Parallel Execution)${NC}"

# Set DATABASE_URL for Rust integration tests
export DATABASE_URL="postgres://test_user:test_password@localhost:5433/test_database"

int_tasks=(
    "apiserver-int:cargo test --test '*':apiserver"
    "playwright:npx playwright test:webdashboard"
)

int_pids=()
int_logs=()
int_names=()

for task in "${int_tasks[@]}"; do
    IFS=":" read -r name cmd dir <<< "$task"
    log_file="$LOG_DIR/int_$name.log"
    echo -e "  [${BLUE}START${NC}] $name Integration Tests..."
    
    (cd "$dir" && eval "$cmd") > "$log_file" 2>&1 &
    int_pids+=($!)
    int_logs+=("$log_file")
    int_names+=("$name")
done

echo -e "  ${BLUE}Waiting for integration tests to finish...${NC}"

int_failed=0
for i in "${!int_pids[@]}"; do
    wait "${int_pids[$i]}"
    status=$?
    show_log_sequential "${int_names[$i]}" "${int_logs[$i]}" $status
    if [ $status -ne 0 ]; then
        int_failed=1
    fi
done

if [ $int_failed -ne 0 ]; then
    echo -e "\n${RED}❌ Some integration tests failed.${NC}"
    exit 1
fi

echo -e "\n${GREEN}${BOLD}🎉 All tests passed successfully!${NC}"