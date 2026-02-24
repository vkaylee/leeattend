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
    echo "================================================================================"
    echo "Unified test orchestrator for LeeAttend project. This script manages all testing"
    echo "including linting, unit tests, integration tests, and E2E tests."
    echo ""
    echo -e "${BOLD}📋 USAGE${NC}"
    echo "    ./run-tests.sh [MODE] [OPTIONS] [TEST_PATTERN]"
    echo ""
    echo -e "${BOLD}🎯 MODES (choose one):${NC}"
    echo -e "    ${GREEN}--all${NC}          Run ALL tests: Lint → Unit → Integration → E2E"
    echo -e "    ${GREEN}--api${NC}          Run API tests: apiserver (Rust) + agent-main (Rust)"
    echo -e "    ${GREEN}--web${NC}          Run Web tests: webdashboard (React + Playwright)"
    echo -e "    ${GREEN}--integration${NC}    Run ONLY integration + E2E tests (skip lint/unit)"
    echo ""
    echo -e "${BOLD}⚡ OPTIONS:${NC}"
    echo -e "    ${GREEN}--test=PATTERN${NC}     Run tests matching PATTERN"
    echo -e "    ${GREEN}--test-only${NC}         Skip lint+unit, run ONLY test pattern (fast debug)"
    echo -e "    ${GREEN}--component=NAME${NC}    Limit to: apiserver, agent-main, webdashboard"
    echo -e "    ${GREEN}--smoke${NC}            Quick smoke test: lint + unit only (no integration)"
    echo -e "    ${GREEN}--list${NC}              List available tests without running"
    echo -e "    ${GREEN}--dry-run${NC}           Show what would run without executing"
    echo ""
    echo -e "${BOLD}🔧 CACHE MANAGEMENT:${NC}"
    echo -e "    ${GREEN}--warm-cache${NC}        Pre-compile dependencies (first run / CI)"
    echo -e "    ${GREEN}--cache-stats${NC}       Show cargo/npm cache statistics"
    echo -e "    ${GREEN}--no-cache${NC}           Disable cache, force fresh build"
    echo -e "    ${GREEN}--clean-cache${NC}        Clean cache only (keep containers)"
    echo -e "    ${GREEN}--clean${NC}              Full cleanup: containers + volumes + cache"
    echo ""
    echo -e "${BOLD}📜 LOG MANAGEMENT:${NC}"
    echo -e "    ${GREEN}--logs${NC}              Extract and display test logs from the internal Docker volume"
    echo -e "    ${GREEN}--tail-logs${NC}         Tail the test logs live from the volume (Docker required)"
    echo ""
    echo -e "${BOLD}⏱️ TIMING OPTIONS:${NC}"
    echo -e "    ${GREEN}--timeout=SECONDS${NC}    Global timeout (default: 1800 = 30 min)"
    echo -e "    ${GREEN}--shard=N/M${NC}          Run shard N of M (CI parallelization)"
    echo ""
    echo -e "${BOLD}📖 DETAILED EXAMPLES FOR AI AGENTS:${NC}"
    echo ""
    echo -e "${YELLOW}# 1. Running new test after creation:${NC}"
    echo "    ./run-tests.sh --api --test=my_new_test"
    echo "    # Use --test-only to skip lint errors:"
    echo "    ./run-tests.sh --api --test-only --test=my_new_test"
    echo ""
    echo -e "${YELLOW}# 2. Debugging a failing test:${NC}"
    echo "    # Step 1: See what would run"
    echo "    ./run-tests.sh --api --dry-run"
    echo "    # Step 2: Run only the test (skip lint)"
    echo "    ./run-tests.sh --api --test-only --test=failing_test"
    echo "    # Step 3: Run specific component"
    echo "    ./run-tests.sh --api --component=apiserver --test=failing_test"
    echo "    # Step 4: If integration tests fail, check real logs from volume"
    echo "    ./run-tests.sh --logs"
    echo ""
    echo -e "${YELLOW}# 3. Quick smoke test before commit:${NC}"
    echo "    ./run-tests.sh --api --smoke"
    echo "    ./run-tests.sh --web --smoke"
    echo "    ./run-tests.sh --all --smoke"
    echo ""
    echo -e "${YELLOW}# 4. Running specific component only:${NC}"
    echo "    ./run-tests.sh --api --component=apiserver"
    echo "    ./run-tests.sh --api --component=agent-main"
    echo "    ./run-tests.sh --web --component=webdashboard"
    echo ""
    echo -e "${YELLOW}# 5. CI/CD parallel execution:${NC}"
    echo "    # Job 1 of 3:"
    echo "    ./run-tests.sh --api --shard=1/3 --smoke"
    echo "    # Job 2 of 3:"
    echo "    ./run-tests.sh --api --shard=2/3 --smoke"
    echo "    # Job 3 of 3:"
    echo "    ./run-tests.sh --api --shard=3/3 --smoke"
    echo ""
    echo -e "${YELLOW}# 6. First time setup / Fresh machine:${NC}"
    echo "    # Step 1: Warm up cache"
    echo "    ./run-tests.sh --all --warm-cache"
    echo "    # Step 2: Run tests (now faster)"
    echo "    ./run-tests.sh --all"
    echo ""
    echo -e "${YELLOW}# 7. Troubleshooting:${NC}"
    echo "    ./run-tests.sh --cache-stats    # View cache"
    echo "    ./run-tests.sh --clean-cache    # Clean cache"
    echo "    ./run-tests.sh --clean          # Full reset"
    echo "    ./run-tests.sh --logs           # View logs in volume"
    echo ""
    echo -e "${YELLOW}# 8. Web/E2E testing:${NC}"
    echo "    ./run-tests.sh --web --test=login_page"
    echo "    ./run-tests.sh --web --smoke"
    echo "    ./run-tests.sh --web --integration"
    echo ""
    echo -e "${BOLD}🔄 TEST PATTERN FORMATS:${NC}"
    echo -e "${YELLOW}API (Rust/cargo nextest):${NC}"
    echo "    mod_name           # All tests in module"
    echo "    mod::test_name    # Specific test in module"
    echo "    test_name         # Any test matching name (regex)"
    echo ""
    echo -e "${YELLOW}Web (Playwright):${NC}"
    echo "    string             # Tests containing string"
    echo "    @tag              # Tests with @tag"
    echo ""
    echo -e "${BOLD}📦 CACHE & PERSISTENCE ARCHITECTURE:${NC}"
    echo "    /workspace/target      → Rust compiled artifacts (shared target-cache volume)"
    echo "    /workspace/cargo       → Downloaded crates (shared)"
    echo "    /sccache             → sccache objects (shared)"
    echo "    /.test_logs          → Test logs (shared test-logs volume)"
    echo "    /workspace/.npm-cache → NPM packages"
    echo ""
    echo -e "${BOLD}⚠️ IMPORTANT NOTES FOR AI:${NC}"
    echo "    1. ALWAYS use ./run-tests.sh - never run test scripts directly"
    echo "    2. Logs are kept in a Docker volume 'test-logs'. Use ./run-tests.sh --logs to read."
    echo "    3. Use --dry-run first to verify what will execute"
    echo "    4. Use --test-only for faster iteration during debugging"
    echo "    5. Cache persists until --clean or --clean-cache is run"
    echo ""
    echo -e "${BOLD}💡 QUICK REFERENCE:${NC}"
    echo "    # Most common for development:"
    echo "    ./run-tests.sh --api --test=TEST_NAME"
    echo "    "
    echo "    # Full pipeline:"
    echo "    ./run-tests.sh --all"
    echo "    "
    echo "    # For CI (GitHub Actions):"
    echo "    ./run-tests.sh --api --shard=1/3"
}

# --- Environment Setup ---
export USE_DOCKER=true
export COMPOSE_PROJECT_NAME="leeattend-test"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export DOCKER_COMPOSE_CMD="docker compose -p $COMPOSE_PROJECT_NAME -f $PROJECT_ROOT/docker-compose.yml -f $PROJECT_ROOT/docker-compose.test.yml"

# Cargo cache settings
export CARGO_TARGET_DIR="$PROJECT_ROOT/target"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

# Auto-include CI override only in CI environments (GitHub Actions, etc.)
if [[ "${CI:-}" == "true" && -f "$PROJECT_ROOT/docker-compose.ci.yml" ]]; then
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

# Ensure DB is migrated before any cargo compile (required for SQLX_OFFLINE=false)
_DB_MIGRATED=false
ensure_db_migrated() {
    if [[ "$_DB_MIGRATED" == "true" ]]; then return 0; fi
    if [[ "${USE_DOCKER:-}" != "true" ]]; then return 0; fi

    echo -e "${BLUE}🗄️ Ensuring database is migrated (required for sqlx compile checks)...${NC}"

    # Start test_db + apiserver_test (which has sqlx-cli)
    $DOCKER_COMPOSE_CMD up -d --build=false test_db apiserver_test

    # Wait for DB healthy
    local timeout="${TIMEOUT_DB_WAIT:-120}"
    local start_time=$SECONDS
    echo -n "  Waiting for database..."
    until $DOCKER_COMPOSE_CMD exec -T test_db pg_isready -U test_user -d test_database > /dev/null 2>&1; do
        printf "."
        sleep 1
        if (( SECONDS - start_time > timeout )); then
            echo -e "\n${RED}❌ Database wait timed out after ${timeout}s${NC}"
            return 1
        fi
    done
    echo -e " ${GREEN}Ready!${NC}"

    # Run migrations
    local DB_URL="postgres://test_user:test_password@test_db:5432/test_database?sslmode=disable"
    if $DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c "sqlx migrate run --database-url '$DB_URL'" > /dev/null 2>&1; then
        echo -e "  ${GREEN}✅ Migrations applied.${NC}"
    else
        echo -e "  ${RED}❌ Migration failed!${NC}"
        return 1
    fi

    _DB_MIGRATED=true
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

        # Migrations must run before clippy/compile (SQLX_OFFLINE=false)
        if [[ "$filter" != *"web"* ]] || [[ "$filter" == "" ]]; then
            ensure_db_migrated
        fi
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

        # Migrations must run before cargo compile (SQLX_OFFLINE=false)
        if [[ "$filter" != *"web"* ]] || [[ "$filter" == "" ]]; then
            ensure_db_migrated
        fi
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
LIST_TESTS=false
DRY_RUN=false
SMOKE=false
WARM_CACHE=false
CACHE_STATS=false
NO_CACHE=false
SHOW_LOGS=false
TAIL_LOGS=false
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
            --list) LIST_TESTS=true;;
            --dry-run) DRY_RUN=true;;
            --smoke) SMOKE=true;;
            --warm-cache) WARM_CACHE=true;;
            --cache-stats) CACHE_STATS=true;;
            --no-cache) NO_CACHE=true;;
            --logs) SHOW_LOGS=true;;
            --tail-logs) TAIL_LOGS=true;;
            --clean)
                echo -e "${BLUE}${BOLD}🧹 Cleaning up test containers and volumes...${NC}"
                $DOCKER_COMPOSE_CMD down --volumes --remove-orphans
                rm -rf "$PROJECT_ROOT/target" "$PROJECT_ROOT/.sccache"
                echo -e "${GREEN}✅ Cleanup complete!${NC}"
                exit 0
                ;;
            --clean-cache)
                echo -e "${BLUE}${BOLD}🧹 Cleaning cache only (keeping containers)...${NC}"
                rm -rf "$PROJECT_ROOT/target" "$PROJECT_ROOT/.sccache"
                if [[ "${USE_DOCKER:-}" == "true" ]]; then
                    $DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c "rm -rf /workspace/target /sccache 2>/dev/null || true"
                    $DOCKER_COMPOSE_CMD exec -T agent_test bash -c "rm -rf /workspace/target /sccache 2>/dev/null || true"
                    $DOCKER_COMPOSE_CMD exec -T web_test bash -c "rm -rf /workspace/.npm-cache /workspace/webdashboard/node_modules 2>/dev/null || true"
                fi
                echo -e "${GREEN}✅ Cache cleaned!${NC}"
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

# Handle log extraction
if [[ "$SHOW_LOGS" == "true" || "$TAIL_LOGS" == "true" ]]; then
    if [[ "${USE_DOCKER:-}" != "true" ]]; then
        echo -e "${YELLOW}Logs from volume are only accessible when using Docker.${NC}"
        echo -e "Local logs: cat $PROJECT_ROOT/.test_logs/*.log"
        exit 0
    fi
    
    echo -e "${BLUE}${BOLD}📜 Extracting test logs from persistent volume...${NC}"
    # Use a temporary busybox container to read from the log volume
    if [[ "$TAIL_LOGS" == "true" ]]; then
        docker run --rm -v leeattend-test_test-logs:/logs alpine tail -f /logs/results.txt /logs/*.log 2>/dev/null || echo "No logs found or container exited."
    else
        docker run --rm -v leeattend-test_test-logs:/logs alpine cat /logs/results.txt /logs/*.log 2>/dev/null | grep -v 'No such file or directory' || echo "No logs found."
    fi
    exit 0
fi

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

# Handle --cache-stats
if [[ "$CACHE_STATS" == "true" ]]; then
    echo -e "${BLUE}${BOLD}📊 Cache Statistics${NC}"
    echo ""
    
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        echo -e "${YELLOW}=== Cargo (apiserver) ===${NC}"
        $DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c "du -sh /workspace/target 2>/dev/null || echo 'No cache'" || true
        $DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c "sccache --show-stats 2>/dev/null | head -10 || echo 'sccache not available'" || true
        
        echo ""
        echo -e "${YELLOW}=== Cargo (agent-main) ===${NC}"
        $DOCKER_COMPOSE_CMD exec -T agent_test bash -c "du -sh /workspace/target 2>/dev/null || echo 'No cache'" || true
        $DOCKER_COMPOSE_CMD exec -T agent_test bash -c "sccache --show-stats 2>/dev/null | head -10 || echo 'sccache not available'" || true
        
        echo ""
        echo -e "${YELLOW}=== NPM ===${NC}"
        $DOCKER_COMPOSE_CMD exec -T web_test bash -c "du -sh /workspace/.npm-cache 2>/dev/null || echo 'No cache'" || true
        $DOCKER_COMPOSE_CMD exec -T web_test bash -c "du -sh /workspace/webdashboard/node_modules 2>/dev/null || echo 'No node_modules'" || true
    else
        echo -e "${YELLOW}=== Local Cargo ===${NC}"
        du -sh "$PROJECT_ROOT/target" 2>/dev/null || echo "No cache"
        
        echo ""
        echo -e "${YELLOW}=== Local NPM ===${NC}"
        du -sh "$HOME/.npm" 2>/dev/null || echo "No npm cache"
    fi
    
    exit 0
fi

# Apply test-only mode (skip lint/unit, run only pattern)
if [[ "$TEST_ONLY" == "true" ]]; then
    PHASE_LINT=false
    PHASE_UNIT=false
    PHASE_INTEGRATION=true
    echo -e "${BLUE}⚡ Test-only mode: skipping lint and unit phases${NC}"
fi

# Apply smoke mode (lint + unit only, skip integration for speed)
if [[ "$SMOKE" == "true" ]]; then
    PHASE_LINT=true
    PHASE_UNIT=true
    PHASE_INTEGRATION=false
    echo -e "${BLUE}🔥 Smoke mode: running lint + unit only${NC}"
fi

# Set default phases if not specified
if [[ "$PHASE_LINT" == "false" && "$PHASE_UNIT" == "false" && "$PHASE_INTEGRATION" == "false" ]]; then
    if [[ "$MODE" != "integration" ]]; then
        PHASE_LINT=true
        PHASE_UNIT=true
    fi
    PHASE_INTEGRATION=true
fi

# Handle --dry-run (show what would execute)
if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${BLUE}${BOLD}🔍 Dry-run: showing what would execute...${NC}"
    echo -e "${BLUE}Mode: $MODE${NC}"
    echo -e "${BLUE}Filter: $FILTER${NC}"
    echo -e "${BLUE}Pattern: ${PATTERN:-none}${NC}"
    echo -e "${BLUE}Component: ${COMPONENT:-all}${NC}"
    echo -e "${BLUE}Phases:${NC}"
    [[ "$PHASE_LINT" == "true" ]] && echo -e "  ✓ Lint"
    [[ "$PHASE_UNIT" == "true" ]] && echo -e "  ✓ Unit Tests"
    [[ "$PHASE_INTEGRATION" == "true" ]] && echo -e "  ✓ Integration Tests"
    echo -e "${GREEN}💡 This is a dry-run. No tests were executed.${NC}"
    exit 0
fi

# Handle --list (show available tests)
if [[ "$LIST_TESTS" == "true" ]]; then
    echo -e "${BLUE}${BOLD}📋 Listing available tests...${NC}"
    if [[ "$MODE" == "api" || "$MODE" == "all" ]]; then
        echo -e "\n${YELLOW}=== API Tests (apiserver) ===${NC}"
        echo "  cargo nextest list --message-format=plain 2>/dev/null || echo 'Run in container for full list'"
    fi
    if [[ "$MODE" == "web" || "$MODE" == "all" ]]; then
        echo -e "\n${YELLOW}=== Web Tests (webdashboard) ===${NC}"
        echo "  npm test -- --list 2>/dev/null || echo 'Run in container for full list'"
    fi
    echo -e "${GREEN}💡 Tip: Run with USE_DOCKER=true to get actual list${NC}"
    exit 0
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
export TIMEOUT_INTEGRATION="${TIMEOUT_INTEGRATION:-1200}"
export TIMEOUT_BUILD="${TIMEOUT_BUILD:-600}"
export TIMEOUT_DB_WAIT="${TIMEOUT_DB_WAIT:-120}"
export TIMEOUT_WEB_READY="${TIMEOUT_WEB_READY:-300}"
export SHARD_N="$SHARD_N"
export SHARD_M="$SHARD_M"

# Handle --no-cache
if [[ "$NO_CACHE" == "true" ]]; then
    echo -e "${YELLOW}⚠️ Cache disabled - forcing fresh build${NC}"
    unset RUSTC_WRAPPER
    export CARGO_NET_OFFLINE=true
    export NPM_CONFIG_CACHE=/dev/null
fi

# Handle --warm-cache (pre-compile dependencies)
if [[ "$WARM_CACHE" == "true" ]]; then
    echo -e "${BLUE}${BOLD}🔥 Warming up cache - pre-compiling dependencies...${NC}"
    
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Start containers
        $DOCKER_COMPOSE_CMD up -d --build=false apiserver_test agent_test web_test
        
        echo -e "${BLUE}📦 Pre-compiling apiserver...${NC}"
        $DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c "cargo check --lib 2>/dev/null || true" &
        
        echo -e "${BLUE}📦 Pre-compiling agent-main...${NC}"
        $DOCKER_COMPOSE_CMD exec -T agent_test bash -c "cargo check --lib 2>/dev/null || true" &
        
        echo -e "${BLUE}📦 Installing web dependencies...${NC}"
        $DOCKER_COMPOSE_CMD exec -T web_test npm install --no-audit --no-fund &
        
        # Wait for all to complete
        wait
        
        echo -e "${GREEN}✅ Cache warmed up!${NC}"
        echo -e "${BLUE}💡 Run tests now: ./run-tests.sh --api${NC}"
    else
        echo -e "${YELLOW}⚠️ Warm-cache only works with Docker. Use --help for options.${NC}"
    fi
    exit 0
fi

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

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Sync local logs into the persistent volume 
        echo -e "${BLUE}📦 Backing up test logs to docker volume 'test-logs'...${NC}"
        docker run --rm -v leeattend-test_test-logs:/logs -v "$PROJECT_ROOT/.test_logs:/local" alpine cp -r /local/. /logs/ 2>/dev/null || true

        # Remove silent redirection to allow visibility of cleanup issues
        $DOCKER_COMPOSE_CMD down --remove-orphans --timeout 5 || {
            echo -e "${YELLOW}⚠️  Warning: Cleanup encountered some issues.${NC}"
        }
        # Only remove ephemeral volumes, leave target-cache, sccache-cache, test-logs intact
        docker volume prune -f --filter "label=com.docker.compose.project=$COMPOSE_PROJECT_NAME" --filter "label=com.docker.compose.volume=shared-data" > /dev/null 2>&1 || true
        echo -e "${GREEN}✅ Environment cleaned up (logs and cache preserved).${NC}"
    fi
    
    exit "$exit_code"
}
trap cleanup EXIT INT TERM

# Run specific test phases
if [[ "$PHASE_LINT" == "true" ]]; then
    run_lint "$FILTER"
fi

if [[ "$PHASE_UNIT" == "true" ]]; then
    run_unit "$FILTER:$PATTERN"
fi

if [[ "$PHASE_INTEGRATION" == "true" ]]; then
    INT_PATTERN="$PATTERN"
    if [[ "$SMOKE" == "true" && -z "$INT_PATTERN" ]]; then
        INT_PATTERN="smoke"
    fi
    run_integration "$FILTER:$INT_PATTERN"
fi

# Kill watchdog since we succeeded
kill "$WATCHDOG_PID" 2>/dev/null || true

# Print final results summary
print_summary_table

echo -e "\n${BLUE}================================================================${NC}"
echo -e "${GREEN}${BOLD}🎉 All tests for [$MODE] passed successfully!${NC}"

