#!/bin/bash

# ==============================================================================
# Unified Test Orchestrator for LeeAttend
# Optimized for selective testing and auto-environment setup.
# Includes WSL2 RAM Disk optimization to avoid Disk I/O overload.
# ==============================================================================

set -euo pipefail

# ANSI Color Codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# ==============================================================================
# WSL2 RAM Disk Optimization
# Moves migrations to /dev/shm to avoid 1GB/s Disk I/O causing system freeze
# ==============================================================================

PROJECT_NAME="leeattend"
RAM_DISK_BASE="/dev/shm/${PROJECT_NAME}_migrations"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Directories containing migrations and metadata
MIGRATION_DIRS=(
    "${SCRIPT_DIR}/apiserver/migrations"
    "${SCRIPT_DIR}/agent-main/migrations"
    "${SCRIPT_DIR}/apiserver/.sqlx"
    "${SCRIPT_DIR}/.test_logs"
)

# Track what we've symlinked for cleanup
declare -a SYMLINKED_DIRS=()
WSL_MODE_ACTIVE=false

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

# Cleanup function - restore original directories from backup
cleanup_ram_disk() {
    if [[ "$WSL_MODE_ACTIVE" != "true" ]]; then
        return 0
    fi

    echo -e "\n${BLUE}🔄 Restoring migration directories from backup...${NC}"

    for dir in "${SYMLINKED_DIRS[@]}"; do
        local backup_dir="${dir}_backup"

        # Remove symlink if it exists
        if [[ -L "$dir" ]]; then
            rm -f "$dir"
        fi

        # Restore from backup
        if [[ -d "$backup_dir" ]]; then
            mv "$backup_dir" "$dir"
            echo -e "  ${GREEN}✓${NC} Restored: $dir"
        fi
    done

    # Cleanup RAM disk
    if [[ -d "$RAM_DISK_BASE" ]]; then
        rm -rf "$RAM_DISK_BASE"
        echo -e "  ${GREEN}✓${NC} Cleaned up RAM disk: $RAM_DISK_BASE"
    fi

    WSL_MODE_ACTIVE=false
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Setup RAM disk for migrations
setup_ram_disk() {
    if ! is_wsl; then
        echo -e "${BLUE}ℹ Not running on WSL, skipping RAM disk optimization${NC}"
        return 0
    fi

    echo -e "${YELLOW}${BOLD}⚡ WSL detected - Enabling RAM Disk optimization${NC}"
    echo -e "${BLUE}  Moving migrations to /dev/shm to avoid Disk I/O overload...${NC}"

    # Note: Trap is set by combined_cleanup in main script, not here
    # This avoids trap stacking issues

    WSL_MODE_ACTIVE=true

    # Create RAM disk base directory
    mkdir -p "$RAM_DISK_BASE"

    for dir in "${MIGRATION_DIRS[@]}"; do
        if [[ ! -d "$dir" ]]; then
            echo -e "  ${YELLOW}⚠${NC} Skipping (not found): $dir"
            continue
        fi

        local dir_name=$(basename "$dir")
        local parent_name=$(basename "$(dirname "$dir")")
        local ram_target="${RAM_DISK_BASE}/${parent_name}_${dir_name}"
        local backup_dir="${dir}_backup"

        # Safety check: don't proceed if backup already exists (previous failed run)
        if [[ -d "$backup_dir" ]]; then
            echo -e "  ${RED}⚠${NC} Backup already exists: $backup_dir"
            echo -e "  ${YELLOW}  Restoring previous backup first...${NC}"
            if [[ -L "$dir" ]]; then
                rm -f "$dir"
            fi
            mv "$backup_dir" "$dir"
        fi

        # Copy to RAM disk
        cp -r "$dir" "$ram_target"

        # Backup original directory
        mv "$dir" "$backup_dir"

        # Create symlink to RAM disk
        ln -s "$ram_target" "$dir"

        SYMLINKED_DIRS+=("$dir")
        echo -e "  ${GREEN}✓${NC} RAM-linked: $dir -> $ram_target"
    done

    echo -e "${GREEN}✓ RAM disk setup complete${NC}\n"
}

show_help() {
    echo -e "${BLUE}${BOLD}🚀 LeeAttend Test Runner${NC}"
    echo -e "Usage: ./run-tests.sh [options] [pattern]"
    echo ""
    echo -e "${BOLD}Options:${NC}"
    echo -e "  ${GREEN}--all${NC}          Run all tests (Lint, Unit, Integration, E2E)"
    echo -e "  ${GREEN}--api${NC}          Run tests for API components (Rust, Python)"
    echo -e "  ${GREEN}--web${NC}          Run tests for Web component (Vite, Playwright)"
    echo -e "  ${GREEN}--help${NC}         Show this help message"
    echo -e "  ${GREEN}--no-ram${NC}       Disable RAM disk optimization (even on WSL)"
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
    echo -e "${BOLD}WSL Optimization:${NC}"
    echo -e "  On WSL, migrations are auto-moved to RAM (/dev/shm) to avoid I/O overload."
    echo -e "  Tests run with low I/O priority (ionice -c 3) to prevent system freeze."
    echo -e "  Backups are created and restored automatically on exit/interrupt."
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
export DOCKER_COMPOSE_CMD="docker compose -f $PROJECT_ROOT/docker-compose.test.yml"

wait_for_web_test() {
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        echo -e "${BLUE}⏳ Waiting for web dependencies (npm ci) to complete...${NC}"
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
            services="web_test"
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

    ./scripts/test-lint.sh "$filter"
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
    
    ./scripts/test-unit.sh "$filter"
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
        ./scripts/test-integration.sh "$filter"
    else
        # Integration tests cho API vẫn cần môi trường thực
        ./setup-test-env.sh "./scripts/test-integration.sh \"$filter\""
    fi
}

# --- Main Logic ---

DISABLE_RAM_DISK=false
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
    if [[ "$arg" == "--no-ram" ]]; then
        DISABLE_RAM_DISK=true
    elif [[ "$arg" == --timeout=* ]]; then
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
    --help)
        show_help
        exit 0
        ;;
    --no-ram)
        # Handle --no-ram as first arg with mode as second
        if [[ -n "${2:-}" && "$2" != "--no-ram" ]]; then
            case "$2" in
                --all) MODE="all"; FILTER="" ;;
                --api) MODE="api"; FILTER="apiserver|agent" ;;
                --web) MODE="web"; FILTER="webdashboard" ;;
                *) echo -e "${RED}Unknown option: $2${NC}"; show_help; exit 1 ;;
            esac
            PATTERN="${3:-}"
        else
            echo -e "${RED}--no-ram requires a mode (--api, --web, --all)${NC}"
            show_help
            exit 1
        fi
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

# Set initial trap for RAM disk cleanup (before setup_ram_disk runs)
trap cleanup_ram_disk EXIT INT TERM

# Setup RAM disk for WSL optimization (unless disabled)
if [[ "$DISABLE_RAM_DISK" != "true" ]]; then
    setup_ram_disk
fi

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

# Combined cleanup handler (now includes watchdog)
combined_cleanup() {
    # Kill watchdog first (if it exists)
    if [[ -n "$WATCHDOG_PID" ]]; then
        kill "$WATCHDOG_PID" 2>/dev/null || true
    fi
    # Then cleanup RAM disk
    cleanup_ram_disk
}
trap combined_cleanup EXIT INT TERM

# Run all test phases
run_lint "$FILTER"
run_unit "$FILTER:$PATTERN"
run_integration "$FILTER:$PATTERN"

# Kill watchdog since we succeeded
kill "$WATCHDOG_PID" 2>/dev/null || true

echo -e "\n${BLUE}================================================================${NC}"
echo -e "${GREEN}${BOLD}🎉 All tests for [$MODE] passed successfully!${NC}"

