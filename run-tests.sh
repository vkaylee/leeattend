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

# Support NO_COLOR standard
if [[ -n "${NO_COLOR:-}" ]]; then
    GREEN='' RED='' YELLOW='' BLUE='' BOLD='' NC=''
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

show_help() {
    echo -e "${BLUE}${BOLD}🚀 LeeAttend Test Runner${NC}"
    echo -e "Usage: ./run-tests.sh [options] [pattern]"
    echo ""
    echo -e "${BOLD}Options:${NC}"
    echo -e "  ${GREEN}--all${NC}          Run all tests (Lint, Unit, Integration, E2E)"
    echo -e "  ${GREEN}--api${NC}          Run tests for API components (Rust: apiserver, agent-main)"
    echo -e "  ${GREEN}--web${NC}          Run tests for Web component (webdashboard)"
    echo -e "  ${GREEN}--integration${NC}  Run ONLY Integration & E2E tests"
    echo -e "  ${GREEN}--test=NAME${NC}    Run specific test(s) by name/pattern"
    echo -e "  ${GREEN}--test-only${NC}   Skip lint/unit, run ONLY test pattern (fast debug)"
    echo -e "  ${GREEN}--component=NAME${NC}  Limit to specific component: apiserver, agent-main, webdashboard"
    echo -e "  ${GREEN}--clean${NC}        Clean up containers and volumes"
    echo -e "  ${GREEN}--no-color${NC}     Disable color output"
    echo -e "  ${GREEN}--help${NC}         Show this help message"
    echo -e "  ${GREEN}--timeout=N${NC}    Set global timeout in seconds (default: 1800)"
    echo -e "  ${GREEN}--shard=N/M${NC}    Run shard N of M (for CI parallelization)"
    echo ""
    echo -e "${BOLD}Environment Variables:${NC}"
    echo -e "  TIMEOUT_LINT=900           Per lint task (default: 15 min)"
    echo -e "  TIMEOUT_UNIT=900           Per unit test task (default: 15 min)"
    echo -e "  TIMEOUT_INTEGRATION=600    Per integration task (default: 10 min)"
    echo -e "  TIMEOUT_GLOBAL=1800        Global timeout (default: 30 min)"
    echo ""
    echo -e "${BOLD}Quick Examples (for AI agents):${NC}"
    echo -e "  ${YELLOW}# Run new unit test named 'test_new_feature'${NC}"
    echo -e "  ./run-tests.sh --api --test=test_new_feature"
    echo -e ""
    echo -e "  ${YELLOW}# Run only the test, skip lint (faster debug)${NC}"
    echo -e "  ./run-tests.sh --api --test-only --test=test_new_feature"
    echo -e ""
    echo -e "  ${YELLOW}# Run specific component only${NC}"
    echo -e "  ./run-tests.sh --api --component=apiserver --test=my_test"
    echo -e ""
    echo -e "  ${YELLOW}# Run webdashboard test${NC}"
    echo -e "  ./run-tests.sh --web --test=login_page"
    echo ""
    echo -e "${BOLD}Pattern Format:${NC}"
    echo -e "  API: cargo nextest filter (e.g., 'mod::test_name', 'test_name')"
    echo -e "  Web: playwright grep pattern (e.g., 'login', '@tag')"
}

# --- Environment Setup ---
export USE_DOCKER=true
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export DOCKER_COMPOSE_CMD="docker compose -p leeattend-test -f $PROJECT_ROOT/docker-compose.yml -f $PROJECT_ROOT/docker-compose.test.yml"

# Enable cargo incremental builds
export CARGO_INCREMENTAL=1
export CARGO_TARGET_DIR="$PROJECT_ROOT/target"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTC_WRAPPER=sccache
export SCCACHE_DIR="$PROJECT_ROOT/.sccache"

# Auto-include CI override if it exists (e.g., on GitHub Actions)
if [[ -f "$PROJECT_ROOT/docker-compose.ci.yml" ]]; then
    export DOCKER_COMPOSE_CMD="$DOCKER_COMPOSE_CMD -f $PROJECT_ROOT/docker-compose.ci.yml"
fi

# Source all script modules
for script in "test-utils.sh" "test-lint.sh" "test-unit.sh" "test-integration.sh"; do
    if [[ -f "$PROJECT_ROOT/scripts/$script" ]]; then
        source "$PROJECT_ROOT/scripts/$script"
    else
        echo -e "${RED}❌ Missing required script: $script${NC}"
        exit 1
    fi
done

wait_for_web_test() {
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Check if node_modules exists, if not run npm install
        if ! $DOCKER_COMPOSE_CMD exec -T web_test ls /workspace/webdashboard/node_modules/.bin/eslint > /dev/null 2>&1; then
            echo -e "${BLUE}📦 Installing web dependencies (npm install)...${NC}"
            $DOCKER_COMPOSE_CMD exec -T web_test npm install --no-audit --no-fund
        fi

        echo -e "${BLUE}⏳ Waiting for web dependencies to be ready...${NC}"
        local timeout="${TIMEOUT_WEB_READY:-300}"
        local start_time=$SECONDS
        local wait_time=1
        while ! $DOCKER_COMPOSE_CMD exec -T web_test ls /workspace/webdashboard/node_modules/.bin/eslint > /dev/null 2>&1; do
            if (( SECONDS - start_time > timeout )); then
                echo -e "${RED}❌ Timeout waiting for web dependencies!${NC}"
                return 1
            fi
            echo -ne "  ${BLUE}waiting ${wait_time}s...${NC}\r"
            sleep $wait_time
            wait_time=$((wait_time * 2))
            [ $wait_time -gt 10 ] && wait_time=10
        done
        echo -e "                                                        \r"
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
        
        $DOCKER_COMPOSE_CMD up -d --build=false $services
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
        
        $DOCKER_COMPOSE_CMD up -d --build=false $services
        if [[ "$need_web" == "true" ]]; then wait_for_web_test; fi
    fi
    
    leeattend_run_unit "$filter"
}

run_integration() {
    local filter=$1

    # Force sequential for integration to save resources and ensure stability
    export NUM_WORKERS=1
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

        $DOCKER_COMPOSE_CMD up -d --build=false $services
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

if [[ $# -eq 0 ]]; then
    show_help
    exit 0
fi

FILTER=""
MODE=""
PATTERN=""
COMPONENT=""
PHASE_LINT=false
PHASE_UNIT=false
PHASE_INTEGRATION=false
TEST_ONLY=false
SHARD_N=""
SHARD_M=""

# Parse flags and find mode/pattern
for arg in "$@"; do
    if [[ "$arg" == --timeout=* ]]; then
        GLOBAL_TIMEOUT="${arg#*=}"
    elif [[ "$arg" == --test=* ]]; then
        PATTERN="${arg#*=}"
    elif [[ "$arg" == --component=* ]]; then
        COMPONENT="${arg#*=}"
    elif [[ "$arg" == --shard=* ]]; then
        local shard_str="${arg#*=}"
        SHARD_N="${shard_str%%/*}"
        SHARD_M="${shard_str##*/}"
    elif [[ "$arg" == "--no-color" ]]; then
        export NO_COLOR=1
        GREEN='' RED='' YELLOW='' BLUE='' BOLD='' NC=''
    elif [[ "$arg" == --* ]]; then
        # Handle modes and phases
        case "$arg" in
            --all) MODE="all"; FILTER="";;
            --api) MODE="api"; FILTER="apiserver|agent";;
            --web) MODE="web"; FILTER="webdashboard";;
            --integration) MODE="integration"; FILTER="apiserver|agent";;
            --lint) PHASE_LINT=true;;
            --unit) PHASE_UNIT=true;;
            --integration-only) PHASE_INTEGRATION=true;;
            --test-only) TEST_ONLY=true;;
            --clean)
                echo -e "${BLUE}${BOLD}🧹 Cleaning up test containers and volumes...${NC}"
                $DOCKER_COMPOSE_CMD down --volumes --remove-orphans
                echo -e "${GREEN}✅ Cleanup complete!${NC}"
                exit 0
                ;;
            --help) show_help; exit 0;;
        esac
    else
        # If it's not a flag and we haven't set a pattern via --test, 
        # then it's the legacy positional pattern
        if [[ -z "$PATTERN" ]]; then
            PATTERN="$arg"
        fi
    fi
done

if [[ -z "$MODE" ]]; then
    echo -e "${RED}Error: No mode specified (e.g., --all, --api, --web)${NC}"
    show_help
    exit 1
fi

# Validate shard parameters
if [[ -n "$SHARD_N" || -n "$SHARD_M" ]]; then
    if [[ -z "$SHARD_N" || -z "$SHARD_M" || "$SHARD_N" -lt 1 || "$SHARD_M" -lt 1 || "$SHARD_N" -gt "$SHARD_M" ]]; then
        echo -e "${RED}Error: Invalid shard format. Use --shard=N/M (e.g., 1/3)${NC}"
        exit 1
    fi
    echo -e "${BLUE}${BOLD}🧩 Running shard $SHARD_N of $SHARD_M${NC}"
fi

# Apply component filter
if [[ -n "$COMPONENT" ]]; then
    case "$COMPONENT" in
        apiserver)
            FILTER="apiserver"
            echo -e "${BLUE}🎯 Component filter: ${YELLOW}apiserver only${NC}"
            ;;
        agent|agent-main)
            FILTER="agent"
            echo -e "${BLUE}🎯 Component filter: ${YELLOW}agent-main only${NC}"
            ;;
        web|webdashboard)
            FILTER="webdashboard"
            echo -e "${BLUE}🎯 Component filter: ${YELLOW}webdashboard only${NC}"
            ;;
        *)
            echo -e "${RED}Error: Invalid component. Use: apiserver, agent-main, webdashboard${NC}"
            exit 1
            ;;
    esac
fi

# Apply test-only mode (skip lint/unit, run only pattern)
if [[ "$TEST_ONLY" == "true" ]]; then
    PHASE_LINT=false
    PHASE_UNIT=false
    PHASE_INTEGRATION=true
    echo -e "${BLUE}⚡ Test-only mode: skipping lint and unit phases${NC}"
fi

echo -e "${BLUE}${BOLD}🚀 Starting LeeAttend Test Suite in [$MODE] mode${NC}"
echo -e "${BLUE}⏱️ Global timeout: ${GLOBAL_TIMEOUT}s ($(( GLOBAL_TIMEOUT / 60 )) min)${NC}"
if [[ -n "$PATTERN" ]]; then
    echo -e "${BLUE}🎯 Specific test pattern: ${YELLOW}$PATTERN${NC}"
fi
echo -e "${BLUE}================================================================${NC}"

# Export timeout vars for sub-scripts
export TIMEOUT_GLOBAL="${TIMEOUT_GLOBAL:-1800}"
export TIMEOUT_LINT="${TIMEOUT_LINT:-900}"
export TIMEOUT_UNIT="${TIMEOUT_UNIT:-900}"
export TIMEOUT_INTEGRATION="${TIMEOUT_INTEGRATION:-600}"
export TIMEOUT_BUILD="${TIMEOUT_BUILD:-600}"
export TIMEOUT_DB_WAIT="${TIMEOUT_DB_WAIT:-120}"
export TIMEOUT_WEB_READY="${TIMEOUT_WEB_READY:-300}"
export SHARD_N="$SHARD_N"
export SHARD_M="$SHARD_M"

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
    # Kill the main process, let cleanup trap handle the rest
    kill -TERM "$MAIN_PID" 2>/dev/null || true
}

# Start watchdog in background
global_timeout_watchdog &
WATCHDOG_PID=$!

# Cleanup handler: kill watchdog and stop containers on exit
cleanup() {
    local exit_code=$?
    echo -e "\n${BLUE}🧹 Cleaning up test environment...${NC}"

    # Kill watchdog safely
    if [[ -n "${WATCHDOG_PID:-}" ]]; then
        kill "$WATCHDOG_PID" 2>/dev/null && wait "$WATCHDOG_PID" 2>/dev/null || true
        WATCHDOG_PID=""
    fi

    # Stop and remove test containers, volumes, and orphans
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Remove silent redirection to allow visibility of cleanup issues
        $DOCKER_COMPOSE_CMD down --volumes --remove-orphans --timeout 5 || {
            echo -e "${YELLOW}⚠️  Warning: Cleanup encountered some issues.${NC}"
        }
        echo -e "${GREEN}✅ Environment cleaned up.${NC}"
    fi
    
    exit "$exit_code"
}
trap cleanup EXIT INT TERM

# Determine if we should run all phases (legacy compatibility)
if [[ "$PHASE_LINT" == "false" && "$PHASE_UNIT" == "false" && "$PHASE_INTEGRATION" == "false" ]]; then
    if [[ "$MODE" != "integration" ]]; then
        PHASE_LINT=true
        PHASE_UNIT=true
    fi
    PHASE_INTEGRATION=true
fi

# Run specific test phases
if [[ "$PHASE_LINT" == "true" ]]; then
    run_lint "$FILTER"
fi

if [[ "$PHASE_UNIT" == "true" ]]; then
    run_unit "$FILTER:$PATTERN"
fi

if [[ "$PHASE_INTEGRATION" == "true" ]]; then
    run_integration "$FILTER:$PATTERN"
fi

# Kill watchdog since we succeeded
kill "$WATCHDOG_PID" 2>/dev/null || true

# Print final results summary
print_summary_table

echo -e "\n${BLUE}================================================================${NC}"
echo -e "${GREEN}${BOLD}🎉 All tests for [$MODE] passed successfully!${NC}"

