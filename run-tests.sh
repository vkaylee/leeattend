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
    echo "Single entry point for ALL test and Docker operations in LeeAttend."
    echo "Never run scripts in scripts/ directly — they are internal modules."
    echo ""
    echo -e "${BOLD}📋 USAGE${NC}"
    echo "    ./run-tests.sh [MODE] [OPTIONS]"
    echo ""
    echo -e "${BOLD}🎯 TEST MODES (choose one):${NC}"
    echo -e "    ${GREEN}--all${NC}              Run ALL tests: Lint → Unit → Integration → E2E"
    echo -e "    ${GREEN}--api${NC}              Run API tests: apiserver (Rust) + agent-main (Rust)"
    echo -e "    ${GREEN}--web${NC}              Run Web tests: webdashboard (React + Playwright)"
    echo -e "    ${GREEN}--integration${NC}      Run ONLY integration + E2E tests (skip lint/unit)"
    echo ""
    echo -e "${BOLD}📊 COVERAGE MODE (combine with test modes):${NC}"
    echo -e "    ${GREEN}--coverage${NC}         Measure test coverage (Rust: llvm-cov, Web: vitest)"
    echo -e "                       Outputs: .test_logs/coverage-*.lcov + HTML reports"
    echo ""
    echo -e "${BOLD}🐳 DOCKER ENVIRONMENT MODE:${NC}"
    echo -e "    ${GREEN}--docker=ENV CMD${NC}   Run any docker compose command for ENV (dev|test|prod)"
    echo -e "                       ENV profiles: dev → docker-compose.dev.yml"
    echo -e "                                      test → docker-compose.test.yml"
    echo -e "                                      prod → docker-compose.yml only"
    echo -e "                       CMD = any docker compose subcommand (up, down, build, exec, logs, ps...)"
    echo ""
    echo -e "${BOLD}⚡ TEST OPTIONS (combine with test modes):${NC}"
    echo -e "    ${GREEN}--test=PATTERN${NC}     Run ONLY tests matching PATTERN (regex for Rust, string for Playwright)"
    echo -e "    ${GREEN}--test-only${NC}        Skip lint + unit phases, run ONLY integration/E2E matching PATTERN"
    echo -e "    ${GREEN}--component=NAME${NC}   Limit to one component: apiserver | agent-main | webdashboard"
    echo -e "    ${GREEN}--smoke${NC}            Quick check: run lint + unit only (skip slow integration/E2E)"
    echo -e "    ${GREEN}--list${NC}             List available tests without running them"
    echo -e "    ${GREEN}--dry-run${NC}          Show what commands would execute, without running anything"
    echo -e "    ${GREEN}--shard=N/M${NC}        CI parallelization: run shard N of M (e.g., --shard=1/3)"
    echo -e "    ${GREEN}--timeout=SECONDS${NC}  Override global timeout (default: 1800 = 30 min)"
    echo ""
    echo -e "${BOLD}🔧 CACHE MANAGEMENT:${NC}"
    echo -e "    ${GREEN}--warm-cache${NC}       Pre-compile all dependencies (run once on fresh machine / CI)"
    echo -e "    ${GREEN}--cache-stats${NC}      Show size and status of Cargo/NPM caches"
    echo -e "    ${GREEN}--no-cache${NC}         Force fresh build, ignoring all caches"
    echo -e "    ${GREEN}--clean-cache${NC}      Delete cache only (keep containers + volumes running)"
    echo -e "    ${GREEN}--clean${NC}            Full reset: stop containers, delete volumes + cache"
    echo ""
    echo -e "${BOLD}📜 LOG ACCESS:${NC}"
    echo -e "    ${GREEN}--logs${NC}             Dump all test logs from the persistent Docker volume"
    echo -e "    ${GREEN}--tail-logs${NC}        Live-tail test logs from the Docker volume"
    echo ""
    echo -e "${BOLD}════════════════════════════════════════ EXAMPLES ════════════════════════════════════════${NC}"
    echo ""
    echo -e "${YELLOW}# 1. Running a specific test by name:${NC}"
    echo "    ./run-tests.sh --api --test=my_test_name"
    echo "    ./run-tests.sh --api --test=auth::test_login_success"
    echo "    # Skip lint if it has compile errors (fast iteration):"
    echo "    ./run-tests.sh --api --test-only --test=my_test_name"
    echo ""
    echo -e "${YELLOW}# 2. Running tests for a specific component only:${NC}"
    echo "    ./run-tests.sh --api --component=apiserver"
    echo "    ./run-tests.sh --api --component=agent-main"
    echo "    ./run-tests.sh --web --component=webdashboard"
    echo "    ./run-tests.sh --api --component=apiserver --test=my_test"
    echo ""
    echo -e "${YELLOW}# 3. Debugging a failing test (step by step):${NC}"
    echo "    ./run-tests.sh --api --dry-run                          # Step 1: see what would run"
    echo "    ./run-tests.sh --api --test-only --test=failing_test    # Step 2: run only that test"
    echo "    ./run-tests.sh --logs                                    # Step 3: read logs if it still fails"
    echo ""
    echo -e "${YELLOW}# 4. Quick smoke check before committing:${NC}"
    echo "    ./run-tests.sh --api --smoke    # lint + unit, skip integration"
    echo "    ./run-tests.sh --all --smoke    # all components, no integration"
    echo ""
    echo -e "${YELLOW}# 5. Full test pipeline:${NC}"
    echo "    ./run-tests.sh --all            # Lint → Unit → Integration → E2E"
    echo "    ./run-tests.sh --api            # API only: Lint → Unit → Integration"
    echo "    ./run-tests.sh --web            # Web only: Lint → Unit → Playwright E2E"
    echo ""
    echo -e "${YELLOW}# 6. Docker environment management (replaces docker-helper.sh):${NC}"
    echo "    ./run-tests.sh --docker=dev up -d                # Start dev environment"
    echo "    ./run-tests.sh --docker=dev down                 # Stop dev environment"
    echo "    ./run-tests.sh --docker=dev logs -f              # Tail all dev logs"
    echo "    ./run-tests.sh --docker=dev logs -f apiserver    # Tail one service log"
    echo "    ./run-tests.sh --docker=dev exec apiserver bash  # Shell into container"
    echo "    ./run-tests.sh --docker=dev build apiserver      # Rebuild one service"
    echo "    ./run-tests.sh --docker=test build               # Rebuild test environment"
    echo "    ./run-tests.sh --docker=prod ps                  # Check prod service status"
    echo ""
    echo -e "${YELLOW}# 7. First time setup / fresh machine:${NC}"
    echo "    ./run-tests.sh --all --warm-cache    # Pre-compile all deps first"
    echo "    ./run-tests.sh --all                 # Then run tests (much faster)"
    echo ""
    echo -e "${YELLOW}# 8. CI/CD parallel sharding:${NC}"
    echo "    ./run-tests.sh --api --shard=1/3     # Job 1 of 3"
    echo "    ./run-tests.sh --api --shard=2/3     # Job 2 of 3"
    echo "    ./run-tests.sh --api --shard=3/3     # Job 3 of 3"
    echo ""
    echo -e "${YELLOW}# 9. Troubleshooting & maintenance:${NC}"
    echo "    ./run-tests.sh --cache-stats         # Check cache size"
    echo "    ./run-tests.sh --clean-cache         # Clear cache, keep containers"
    echo "    ./run-tests.sh --clean               # Full reset"
    echo "    ./run-tests.sh --logs                # Read test logs from Docker volume"
    echo "    ./run-tests.sh --tail-logs           # Live-tail logs"
    echo ""
    echo -e "${BOLD}🔄 TEST PATTERN FORMATS:${NC}"
    echo -e "${YELLOW}  API - Rust (cargo nextest):${NC}"
    echo "    test_name          → any test whose name contains 'test_name' (regex)"
    echo "    module::test_name  → exact test in a module"
    echo "    module             → all tests in a module"
    echo ""
    echo -e "${YELLOW}  Web - Playwright:${NC}"
    echo "    login_page         → tests whose title contains 'login_page'"
    echo "    @smoke             → tests tagged with @smoke"
    echo ""
    echo -e "${BOLD}📦 INTERNAL MODULES (DO NOT RUN DIRECTLY):${NC}"
    echo "    scripts/test-utils.sh        → shared utilities, sourced by run-tests.sh"
    echo "    scripts/test-lint.sh         → lint phase definitions"
    echo "    scripts/test-unit.sh         → unit test phase definitions"
    echo "    scripts/test-integration.sh  → integration/E2E phase definitions"
    echo "    scripts/debug-logs.sh        → log analysis functions"
    echo "    setup-test-env.sh            → DB isolation setup (called automatically)"
    echo ""
    echo -e "${BOLD}⚠️ RULES FOR AI AGENTS:${NC}"
    echo "    1. ALWAYS use ./run-tests.sh — NEVER run scripts/ files directly"
    echo "    2. Use --dry-run to verify what will run before executing"
    echo "    3. Use --test-only + --test=NAME for fast iteration on a single test"
    echo "    4. Use --docker=dev exec SERVICE bash to run ad-hoc commands in containers"
    echo "    5. Test logs are stored in Docker volume. Use --logs or --tail-logs to read them"
    echo "    6. Cache persists across runs. Use --clean only when something is truly broken"
}

# --- Docker env command handler (must run before sourcing modules) ---
# Handles: ./run-tests.sh --docker=ENV [docker compose subcommand...]
for _arg in "$@"; do
    if [[ "$_arg" == --docker=* ]]; then
        _env="${_arg#--docker=}"
        _PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        case "$_env" in
            dev)
                _files="-p leeattend-dev -f $_PROJECT_ROOT/docker-compose.yml -f $_PROJECT_ROOT/docker-compose.dev.yml"
                ;;
            test)
                _files="-p leeattend-test -f $_PROJECT_ROOT/docker-compose.test.yml"
                ;;
            prod|base)
                _files="-p leeattend -f $_PROJECT_ROOT/docker-compose.yml"
                ;;
            *)
                echo -e "${RED}Error: Unknown environment '$_env'. Use: dev, test, prod${NC}"
                echo "Usage: ./run-tests.sh --docker=ENV [docker compose subcommand...]"
                exit 1
                ;;
        esac
        # Collect remaining args (everything except --docker=ENV)
        _docker_args=()
        for _a in "$@"; do
            [[ "$_a" == --docker=* ]] && continue
            _docker_args+=("$_a")
        done
        echo "Running: docker compose $_files ${_docker_args[*]}"
        docker compose $_files "${_docker_args[@]}"
        exit $?
    fi
done

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

# ── PTY-Safe Docker Compose wrappers ────────────────────────────────
#
# dc()          — Normal use. Redirects stdin only (</dev/null).
#                 Output is preserved so callers can see Docker progress.
#                 Safe for: dc up, dc exec, dc build, etc.
#
# dc_detached() — Cleanup use. Runs in a NEW session (setsid) with ALL
#                 FDs detached. Guarantees no Docker child can hold PTY.
#                 Safe for: dc_detached down, dc_detached volume prune.
#
dc() {
    $DOCKER_COMPOSE_CMD "$@" </dev/null
}
dc_detached() {
    setsid $DOCKER_COMPOSE_CMD "$@" </dev/null >/dev/null 2>&1
}
export -f dc dc_detached

# Source all script modules
for script in "test-utils.sh" "test-lint.sh" "test-unit.sh" "test-integration.sh" "test-coverage.sh"; do
    if [[ -f "$PROJECT_ROOT/scripts/$script" ]]; then
        source "$PROJECT_ROOT/scripts/$script"
    else
        echo -e "${RED}❌ Missing required script: $script${NC}"
        exit 1
    fi
done

wait_for_web_test() {
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Always run npm install to ensure node_modules are in sync with package.json.
        # This is idempotent (fast no-op when deps are up-to-date) and prevents
        # stale named volumes from missing newly added dependencies.
        echo -e "${BLUE}📦 Syncing web dependencies (npm install)...${NC}"
        dc exec -T web_test npm install --no-audit --no-fund

        echo -e "${BLUE}⏳ Waiting for web dependencies to be ready...${NC}"
        local timeout="${TIMEOUT_WEB_READY:-300}"
        local start_time=$SECONDS
        local wait_time=1
        while ! dc exec -T web_test ls /workspace/webdashboard/node_modules/.bin/eslint > /dev/null 2>&1; do
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

# Ensure DB is migrated before any cargo compile (sqlx verifies queries at compile time)
_DB_MIGRATED=false
ensure_db_migrated() {
    if [[ "$_DB_MIGRATED" == "true" ]]; then return 0; fi
    if [[ "${USE_DOCKER:-}" != "true" ]]; then return 0; fi

    echo -e "${BLUE}🗄️ Ensuring database is migrated (required for sqlx compile checks)...${NC}"

    # Start test_db + apiserver_test (which has sqlx-cli)
    dc up -d --build=false test_db apiserver_test

    # Wait for DB healthy
    local timeout="${TIMEOUT_DB_WAIT:-120}"
    local start_time=$SECONDS
    echo -n "  Waiting for database..."
    until dc exec -T test_db pg_isready -U test_user -d test_database > /dev/null 2>&1; do
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
    if dc exec -T apiserver_test bash -c "sqlx migrate run --database-url '$DB_URL'" > /dev/null 2>&1; then
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
        
        dc up -d --build=false $services
        if [[ "$need_web" == "true" ]]; then wait_for_web_test; fi

        # Migrations must run before any Rust compilation (sqlx verifies queries at compile time).
        # Even --web mode needs this because api-sync runs `cargo run -- gen-openapi`.
        ensure_db_migrated
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
        
        dc up -d --build=false $services
        if [[ "$need_web" == "true" ]]; then wait_for_web_test; fi

        # Migrations must run before cargo compile (sqlx verifies queries at compile time)
        if [[ "$filter" != *"web"* ]] || [[ "$filter" == "" ]]; then
            ensure_db_migrated
        fi
    fi
    
    leeattend_run_unit "$filter"
}

run_integration() {
    local filter=$1

    # Integration tests use isolated DBs — parallel is safe, keep auto-detected NUM_WORKERS
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

        dc up -d --build=false $services
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
COVERAGE=false
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
        # FIX: Cannot use `local` outside a function in bash.
        # `local shard_str=...` here would cause: "bash: local: can only be used in a function"
        # With `set -e` enabled, this error silently terminates the entire script.
        # Using a prefixed temp variable (_shard_str) instead.
        _shard_str="${arg#*=}"
        SHARD_N="${_shard_str%%/*}"
        SHARD_M="${_shard_str##*/}"
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
            --coverage) COVERAGE=true;;
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
                dc down --volumes --remove-orphans
                rm -rf "$PROJECT_ROOT/target" "$PROJECT_ROOT/.sccache"
                echo -e "${GREEN}✅ Cleanup complete!${NC}"
                exit 0
                ;;
            --clean-cache)
                echo -e "${BLUE}${BOLD}🧹 Cleaning cache only (keeping containers)...${NC}"
                rm -rf "$PROJECT_ROOT/target" "$PROJECT_ROOT/.sccache"
                if [[ "${USE_DOCKER:-}" == "true" ]]; then
                    dc exec -T apiserver_test bash -c "rm -rf /workspace/target /sccache 2>/dev/null || true"
                    dc exec -T agent_test bash -c "rm -rf /workspace/target /sccache 2>/dev/null || true"
                    dc exec -T web_test bash -c "rm -rf /workspace/.npm-cache /workspace/webdashboard/node_modules 2>/dev/null || true"
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
        dc exec -T apiserver_test bash -c "du -sh /workspace/target 2>/dev/null || echo 'No cache'" || true
        dc exec -T apiserver_test bash -c "sccache --show-stats 2>/dev/null | head -10 || echo 'sccache not available'" || true
        
        echo ""
        echo -e "${YELLOW}=== Cargo (agent-main) ===${NC}"
        dc exec -T agent_test bash -c "du -sh /workspace/target 2>/dev/null || echo 'No cache'" || true
        dc exec -T agent_test bash -c "sccache --show-stats 2>/dev/null | head -10 || echo 'sccache not available'" || true
        
        echo ""
        echo -e "${YELLOW}=== NPM ===${NC}"
        dc exec -T web_test bash -c "du -sh /workspace/.npm-cache 2>/dev/null || echo 'No cache'" || true
        dc exec -T web_test bash -c "du -sh /workspace/webdashboard/node_modules 2>/dev/null || echo 'No node_modules'" || true
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
    [[ "$COVERAGE" == "true" ]] && echo -e "  ✓ Coverage (cargo llvm-cov / vitest)"
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
        dc up -d --build=false apiserver_test agent_test web_test
        
        echo -e "${BLUE}📦 Pre-compiling apiserver...${NC}"
        dc exec -T apiserver_test bash -c "cargo check --lib 2>/dev/null || true" &
        
        echo -e "${BLUE}📦 Pre-compiling agent-main...${NC}"
        dc exec -T agent_test bash -c "cargo check --lib 2>/dev/null || true" &
        
        echo -e "${BLUE}📦 Installing web dependencies...${NC}"
        dc exec -T web_test npm install --no-audit --no-fund &
        
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

# NOTE: Global watchdog removed. Per-phase timeouts (TIMEOUT_LINT,
# TIMEOUT_UNIT, TIMEOUT_INTEGRATION, TIMEOUT_BUILD) already protect
# each stage individually. The background watchdog process caused PTY
# file descriptor inheritance issues when run-tests.sh was invoked from
# git hooks or AI agent terminals, making commands appear to hang
# indefinitely even after successful completion.

# Cleanup handler: stop containers on exit
cleanup() {
    local exit_code=$?
    echo -e "\n${BLUE}🧹 Cleaning up test environment...${NC}"

    # ── PTY Safety: Kill all child processes ──────────────────────────
    # When run from AI agents or git hooks, orphan docker-exec processes
    # can hold PTY file descriptors open, making the parent appear hung.
    # Kill direct children of this script (not ourselves) to clean up.
    # Ignore TERM in this handler so we survive our own cleanup kills.
    trap '' TERM
    pkill -TERM -P $$ 2>/dev/null || true
    sleep 1
    pkill -KILL -P $$ 2>/dev/null || true

    # Also kill any cargo/nextest processes left inside Docker containers
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        for ctr in apiserver_test agent_test; do
            timeout 5 setsid docker exec "${COMPOSE_PROJECT_NAME}-${ctr}-1" \
                pkill -f 'cargo|nextest' </dev/null >/dev/null 2>&1 || true
        done
    fi

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # CRITICAL: All docker commands use full FD redirect (</dev/null >/dev/null 2>&1)
        # to completely detach from PTY. Even stdout/stderr can hold FDs open
        # if Docker CLI spawns child processes that inherit them.

        # Sync local logs into the persistent volume
        echo -e "${BLUE}📦 Backing up test logs to docker volume 'test-logs'...${NC}"
        timeout 10 setsid docker run --rm \
            -v leeattend-test_test-logs:/logs \
            -v "$PROJECT_ROOT/.test_logs:/local" \
            alpine cp -r /local/. /logs/ </dev/null >/dev/null 2>&1 || true

        # Stop containers — use dc_detached for complete PTY detachment
        dc_detached down --remove-orphans --timeout 5 || {
            echo -e "${YELLOW}⚠️  Warning: Cleanup encountered some issues.${NC}"
        }
        # Only remove ephemeral volumes, leave target-cache, sccache-cache, test-logs intact
        setsid docker volume prune -f \
            --filter "label=com.docker.compose.project=$COMPOSE_PROJECT_NAME" \
            --filter "label=com.docker.compose.volume=shared-data" \
            </dev/null >/dev/null 2>&1 || true
        echo -e "${GREEN}✅ Environment cleaned up (logs and cache preserved).${NC}"
    fi

    # ── Final PTY sweep ──────────────────────────────────────────────
    # Docker commands above may have spawned NEW child processes.
    # One last aggressive kill to catch any stragglers before exit.
    pkill -KILL -P $$ 2>/dev/null || true

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

if [[ "$COVERAGE" == "true" ]]; then
    # Coverage requires containers to be running
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        _cov_services=""
        if [[ "$FILTER" == *"web"* ]]; then
            _cov_services="web_test"
        elif [[ "$FILTER" == *"apiserver"* || "$FILTER" == *"agent"* ]]; then
            _cov_services="test_db apiserver_test agent_test"
        else
            _cov_services="test_db apiserver_test agent_test web_test"
        fi
        dc up -d --build=false $_cov_services
        if [[ "$FILTER" == *"web"* || -z "$FILTER" ]]; then
            wait_for_web_test
        fi
        if [[ "$FILTER" != *"web"* ]] || [[ -z "$FILTER" ]]; then
            ensure_db_migrated
        fi
    fi
    leeattend_run_coverage "$FILTER"
fi


# Print final results summary
print_summary_table

echo -e "\n${BLUE}================================================================${NC}"
echo -e "${GREEN}${BOLD}🎉 All tests for [$MODE] passed successfully!${NC}"

