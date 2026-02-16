#!/bin/bash

# ==============================================================================
# Unified Test Orchestrator for LeeAttend
# Optimized for selective testing and auto-environment setup.
# ==============================================================================

set -euo pipefail

# ANSI Color Codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

show_help() {
    echo -e "${BLUE}${BOLD}🚀 LeeAttend Test Runner${NC}"
    echo -e "Usage: ./run-tests.sh [options] [pattern]"
    echo ""
    echo -e "${BOLD}Options:${NC}"
    echo -e "  ${GREEN}--all${NC}          Run all tests (Lint, Unit, Integration, E2E)"
    echo -e "  ${GREEN}--api${NC}          Run tests for API components (Rust, Python)"
    echo -e "  ${GREEN}--web${NC}          Run tests for Web component (Vite, Playwright)"
    echo -e "  ${GREEN}--integration${NC}  Run ONLY Integration & E2E tests"
    echo -e "  ${GREEN}--clean${NC}        Clean up containers and volumes (run this standalone)"
    echo -e "  ${GREEN}--help${NC}         Show this help message"
    echo -e "  ${GREEN}--timeout=N${NC}    Set global timeout in seconds (default: 1800 = 30min)"
    echo ""
    echo -e "${BOLD}Timeout Configuration (env vars):${NC}"
    echo -e "  TIMEOUT_LINT=180         Per lint task (default: 3 min)"
    echo -e "  TIMEOUT_UNIT=180         Per unit test task (default: 3 min)"
    echo -e "  TIMEOUT_INTEGRATION=300  Per integration task (default: 5 min)"
    echo -e "  TIMEOUT_BUILD=600        Pre-compilation (default: 10 min)"
    echo -e "  TIMEOUT_DB_WAIT=120      Database ready wait (default: 2 min)"
    echo -e "  TIMEOUT_GLOBAL=1800      Global timeout (default: 30 min)"
    echo ""
    echo -e "${BOLD}Pattern:${NC}"
    echo -e "  Optional. Filter test cases by name (e.g., 'auth', 'attendance')."
    echo -e "  Works for both API (cargo/pytest) and Web (playwright grep)."
    echo ""
    echo -e "${BOLD}Examples:${NC}"
    echo -e "  ./run-tests.sh --api robustness   # Lint all API, then run 'robustness' tests"
    echo -e "  ./run-tests.sh --web login        # Lint all Web, then run 'login' tests"
    echo -e "  ./run-tests.sh --all"
    echo -e "  TIMEOUT_GLOBAL=600 ./run-tests.sh --api  # 10 min global timeout"
}

# --- Environment Setup ---
export USE_DOCKER=true
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export DOCKER_COMPOSE_CMD="docker compose -p leeattend-test -f $PROJECT_ROOT/docker-compose.test.yml"

# Source all script modules
source "$PROJECT_ROOT/scripts/test-utils.sh"
source "$PROJECT_ROOT/scripts/test-lint.sh"
source "$PROJECT_ROOT/scripts/test-unit.sh"
source "$PROJECT_ROOT/scripts/test-integration.sh"

wait_for_web_test() {
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Check if node_modules exists, if not run npm install
        if ! $DOCKER_COMPOSE_CMD exec -T web_test ls /workspace/webdashboard/node_modules/.bin/eslint > /dev/null 2>&1; then
            echo -e "${BLUE}📦 Installing web dependencies (npm install)...${NC}"
            $DOCKER_COMPOSE_CMD exec -T web_test npm install --no-audit --no-fund
        fi

        echo -e "${BLUE}⏳ Waiting for web dependencies to be ready...${NC}"
        local timeout=300
        local start_time=$SECONDS
        while ! $DOCKER_COMPOSE_CMD exec -T web_test ls /workspace/webdashboard/node_modules/.bin/eslint > /dev/null 2>&1; do
            if (( SECONDS - start_time > timeout )); then
                echo -e "${RED}❌ Timeout waiting for web dependencies!${NC}"
                return 1
            fi
            sleep 2
        done
        echo -e "${GREEN}✅ Web dependencies ready.${NC}"
    fi
}

run_lint() {
    local filter=$1
    echo -e "\n${BLUE}🔹 Running Linting [$filter]...${NC}"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        echo -e "${BLUE}🏗️ Ensuring Docker containers are up for Linting...${NC}"
        local services=""
        local need_web=false
        
        if [[ "$filter" == *"web"* ]]; then
            # We need apiserver_test for api-sync even if testing web
            services="web_test apiserver_test"
            need_web=true
        elif [[ "$filter" == *"apiserver"* || "$filter" == *"agent"* || "$filter" == *"api"* ]]; then
            services="apiserver_test agent_test"
        else
            services="apiserver_test agent_test web_test"
            need_web=true
        fi
        
        $DOCKER_COMPOSE_CMD up -d $services
        if [[ "$need_web" == "true" ]]; then wait_for_web_test; fi
    fi

    leeattend_run_lint "$filter"
}

run_unit() {
    local filter=$1
    echo -e "\n${BLUE}🔹 Running Unit Tests [$filter]...${NC}"
    
    if [[ "$USE_DOCKER" == "true" ]]; then
        echo -e "${BLUE}🏗️ Ensuring Docker containers are up...${NC}"
        local services=""
        local need_web=false
        
        if [[ "$filter" == *"web"* ]]; then
            services="web_test"
            need_web=true
        elif [[ "$filter" == *"apiserver"* || "$filter" == *"agent"* || "$filter" == *"api"* ]]; then
            # Unit tests cho API/Agent thường không cần DB thật (dùng memory), 
            # nhưng ta bật apiserver_test/agent_test container để chạy code trong đó
            services="apiserver_test agent_test"
        else
            services="apiserver_test agent_test web_test"
            need_web=true
        fi
        
        $DOCKER_COMPOSE_CMD up -d $services
        if [[ "$need_web" == "true" ]]; then wait_for_web_test; fi
    fi
    
    leeattend_run_unit "$filter"
}

run_integration() {
    local filter=$1
    echo -e "\n${BLUE}🔹 Running Integration & E2E Tests [$filter]...${NC}"
    
    if [[ "$USE_DOCKER" == "true" ]]; then
        echo -e "${BLUE}🏗️ Ensuring Docker environment for Integration tests...${NC}"
        local services=""
        local need_web=false
        
        if [[ "$filter" == *"web"* ]]; then
            # Web E2E dùng Mock API, KHÔNG cần DB và Backend thật
            services="web_test"
            need_web=true
        else
            # Integration tests cho API bắt buộc cần DB thật
            services="test_db apiserver_test agent_test"
            if [[ "$filter" == "" || "$filter" == "all" || "$filter" == "api" ]]; then 
                services="$services web_test"
                need_web=true
            fi
        fi

        $DOCKER_COMPOSE_CMD up -d $services
        if [[ "$need_web" == "true" ]]; then wait_for_web_test; fi
    fi

    if [[ "$filter" == *"web"* ]]; then
        # Chạy thẳng E2E không cần setup-test-env (vì không cần DB isolated cho từng worker)
        leeattend_run_integration "$filter"
    else
        # Integration tests cho API vẫn cần môi trường thực
        ./setup-test-env.sh "leeattend_run_integration \"$filter\""
    fi
}

# --- Main Logic ---

GLOBAL_TIMEOUT=${TIMEOUT_GLOBAL:-1800}

if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

FILTER=""
MODE=""
PATTERN="${2:-}"

# Parse flags
for arg in "$@"; do
    if [[ "$arg" == --timeout=* ]]; then
        GLOBAL_TIMEOUT="${arg#*=}"
    fi
done

case "$1" in
    --all)
        MODE="all"
        FILTER=""
        ;;
    --api)
        MODE="api"
        FILTER="apiserver|agent"
        ;;
    --web)
        MODE="web"
        FILTER="webdashboard"
        ;;
    --integration)
        MODE="integration"
        FILTER="apiserver|agent" # Only relevant components for integration
        ;;
    --clean)
        echo -e "${BLUE}${BOLD}🧹 Cleaning up test containers and volumes...${NC}"
        $DOCKER_COMPOSE_CMD down --volumes --remove-orphans
        echo -e "${GREEN}✅ Cleanup complete!${NC}"
        exit 0
        ;;
    --help)
        show_help
        exit 0
        ;;
    *)
        echo -e "${RED}Unknown option: $1${NC}"
        show_help
        exit 1
        ;;
esac

echo -e "${BLUE}${BOLD}🚀 Starting LeeAttend Test Suite in [$MODE] mode${NC}"
echo -e "${BLUE}⏱️ Global timeout: ${GLOBAL_TIMEOUT}s ($(( GLOBAL_TIMEOUT / 60 )) min)${NC}"
if [ -n "$PATTERN" ]; then
    echo -e "${BLUE}🎯 Specific test pattern: ${YELLOW}$PATTERN${NC}"
fi
echo -e "${BLUE}================================================================${NC}"

# Export timeout vars for sub-scripts
export TIMEOUT_GLOBAL="$GLOBAL_TIMEOUT"
export TIMEOUT_LINT="${TIMEOUT_LINT:-600}"
export TIMEOUT_UNIT="${TIMEOUT_UNIT:-1800}"
export TIMEOUT_INTEGRATION="${TIMEOUT_INTEGRATION:-1200}"
export TIMEOUT_BUILD="${TIMEOUT_BUILD:-600}"
export TIMEOUT_DB_WAIT="${TIMEOUT_DB_WAIT:-120}"

# Clear previous results
if [[ -L "$PROJECT_ROOT/.test_logs" ]]; then
    mkdir -p "$(readlink "$PROJECT_ROOT/.test_logs")"
else
    mkdir -p "$PROJECT_ROOT/.test_logs"
fi
rm -f "$PROJECT_ROOT/.test_logs/results.txt"

# Global timeout watchdog (background process)
MAIN_PID=$$
WATCHDOG_PID=""

global_timeout_watchdog() {
    sleep "$GLOBAL_TIMEOUT"
    echo -e "\n${RED}================================================================${NC}"
    echo -e "${RED}${BOLD}⏰ GLOBAL TIMEOUT: Test suite exceeded ${GLOBAL_TIMEOUT}s limit${NC}"
    echo -e "${RED}================================================================${NC}"
    # Kill only the main process, let cleanup trap handle the rest
    kill -TERM "$MAIN_PID" 2>/dev/null || true
}

# Start watchdog in background
global_timeout_watchdog &
WATCHDOG_PID=$!

# Cleanup handler: kill watchdog and stop containers on exit
cleanup() {
    echo -e "\n${BLUE}🧹 Cleaning up test containers and volumes...${NC}"

    # Kill watchdog
    if [[ -n "${WATCHDOG_PID:-}" ]]; then
        kill "$WATCHDOG_PID" 2>/dev/null || true
    fi

    # Stop and remove test containers, volumes, and orphans
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Use a short timeout to force shutdown if containers are stuck
        $DOCKER_COMPOSE_CMD down --volumes --remove-orphans --timeout 5 2>/dev/null || true
        echo -e "${GREEN}✅ Environment cleaned up.${NC}"
    fi
}
trap cleanup EXIT INT TERM

# Run all test phases
if [[ "$MODE" != "integration" ]]; then
    run_lint "$FILTER"
    run_unit "$FILTER:$PATTERN"
fi
run_integration "$FILTER:$PATTERN"

# Kill watchdog since we succeeded
kill "$WATCHDOG_PID" 2>/dev/null || true

# Print final results summary
print_summary_table

echo -e "\n${BLUE}================================================================${NC}"
echo -e "${GREEN}${BOLD}🎉 All tests for [$MODE] passed successfully!${NC}"

