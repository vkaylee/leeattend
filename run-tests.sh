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

# --- PTY Hang Protection (Always Active) ---
# Automatically wraps this script in safe-run.py to prevent Podman/Docker
# daemons from leaking PTY file descriptors and causing infinite hangs.
# Opt out with: SKIP_SAFE_RUN=1 ./run-tests.sh --api
if [[ "${SKIP_SAFE_RUN:-0}" == "0" && "${_SAFE_RUN_ACTIVE:-0}" == "0" ]]; then
    export _SAFE_RUN_ACTIVE=1
    exec "$SCRIPT_DIR/scripts/safe-run.py" 1200 "$0" "$@"
fi
# --- Environment Setup ---
export USE_DOCKER=true
export COMPOSE_PROJECT_NAME="leeattend-test"
export PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export DOCKER_COMPOSE_CMD="podman-compose -p $COMPOSE_PROJECT_NAME -f $PROJECT_ROOT/.compose/docker-compose.test.yml"

# Cargo cache settings
export CARGO_TARGET_DIR="$PROJECT_ROOT/target"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

# Auto-include CI override only in CI environments (GitHub Actions, etc.)
if [[ "${CI:-}" == "true" && -f "$PROJECT_ROOT/.compose/docker-compose.ci.yml" ]]; then
    export DOCKER_COMPOSE_CMD="$DOCKER_COMPOSE_CMD -f $PROJECT_ROOT/.compose/docker-compose.ci.yml"
fi

# ── Compose Configuration Validation ─────────────────────────────────
# Fail early if there are syntax errors or broken variables in the compose files.
if [[ "${USE_DOCKER:-}" == "true" ]]; then
    if ! $DOCKER_COMPOSE_CMD config -q > /dev/null 2>&1; then
        echo -e "${RED}❌ Docker Compose configuration is invalid or contains broken variables.${NC}"
        echo -e "${YELLOW}Run '$DOCKER_COMPOSE_CMD config' manually to see the exact error.${NC}"
        exit 1
    fi
fi

# Source all modules
for module in test-utils test-lint test-unit test-integration test-coverage db-ops docker-ops lifecycle test-runner cli-parser; do
    if [[ -f "$PROJECT_ROOT/scripts/${module}.sh" ]]; then
        source "$PROJECT_ROOT/scripts/${module}.sh"
    else
        echo -e "${RED}❌ Missing required module: scripts/${module}.sh${NC}"
        exit 1
    fi
done

# Initialize CLI parser
parse_cli_args "$@"

# --- Main Orchestration Pipeline ---
trap cleanup EXIT INT TERM
# Run specific test phases
if [[ "${GLOBAL_TIMEOUT:-0}" -gt 0 ]]; then
    start_watchdog "$GLOBAL_TIMEOUT"
fi

if [[ "$PHASE_LINT" == "true" ]]; then
    run_lint "$FILTER"
fi

if [[ "$PHASE_UNIT" == "true" ]]; then
    run_unit "$FILTER:$PATTERN"
fi

if [[ "$PHASE_INTEGRATION" == "true" ]]; then
    run_integration "$FILTER:$PATTERN"

fi

if [[ "$COVERAGE" == "true" ]]; then
    # Coverage requires containers to be running
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        _cov_services=""
        if [[ "$FILTER" == *"web"* ]]; then
            _cov_services="webdashboard"
        elif [[ "$FILTER" == *"apiserver"* || "$FILTER" == *"agent"* ]]; then
            _cov_services="db apiserver agent-main"
        else
            _cov_services="db apiserver agent-main webdashboard"
        fi
        dc build $_cov_services
        dc up -d $_cov_services
        if [[ "$FILTER" == *"web"* || -z "$FILTER" ]]; then
            wait_for_webdashboard
        fi
        if [[ "$FILTER" != *"web"* ]] || [[ -z "$FILTER" ]]; then
            # Force re-migration: db uses tmpfs (RAM disk), so data may
            # have been lost if the container was rebuilt/restarted between
            # earlier phases and coverage.  Reset the cache flag so
            # ensure_db_migrated() actually runs migrations again.
            _DB_MIGRATED=false
            ensure_db_migrated
        fi
    fi
    leeattend_run_coverage "$FILTER"
fi


# Print final results summary
print_summary_table

echo -e "\n${BLUE}================================================================${NC}"
echo -e "${GREEN}${BOLD}🎉 All tests for [$MODE] passed successfully!${NC}"
