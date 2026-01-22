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

show_help() {
    echo -e "${BLUE}${BOLD}🚀 LeeAttend Test Runner${NC}"
    echo -e "Usage: ./run-tests.sh [options] [pattern]"
    echo ""
    echo -e "${BOLD}Options:${NC}"
    echo -e "  ${GREEN}--all${NC}          Run all tests (Lint, Unit, Integration, E2E)"
    echo -e "  ${GREEN}--api${NC}          Run tests for API components (Rust, Python)"
    echo -e "  ${GREEN}--web${NC}          Run tests for Web component (Vite, Playwright)"
    echo -e "  ${GREEN}--help${NC}         Show this help message"
    echo ""
    echo -e "${BOLD}Pattern:${NC}"
    echo -e "  Optional. Filter test cases by name (e.g., 'auth', 'attendance')."
    echo -e "  Works for both API (cargo/pytest) and Web (playwright grep)."
    echo ""
    echo -e "${BOLD}Examples:${NC}"
    echo -e "  ./run-tests.sh --api robustness   # Lint all API, then run 'robustness' tests"
    echo -e "  ./run-tests.sh --web login        # Lint all Web, then run 'login' tests"
    echo -e "  ./run-tests.sh --all"
}

run_lint() {
    local filter=$1
    echo -e "\n${BLUE}🔹 Running Linting [$filter]...${NC}"
    ./scripts/test-lint.sh "$filter"
}

run_unit() {
    local filter=$1
    echo -e "\n${BLUE}🔹 Running Unit Tests [$filter]...${NC}"
    ./scripts/test-unit.sh "$filter"
}

run_integration() {
    local filter=$1
    echo -e "\n${BLUE}🔹 Running Integration & E2E Tests [$filter]...${NC}"
    # Integration tests always require the DB environment
    # We pass the arguments as separate words, but setup-test-env.sh uses eval "$@"
    # which is generally dangerous but we can make it work by being careful.
    ./setup-test-env.sh "./scripts/test-integration.sh \"$filter\""
}

# --- Main Logic ---

if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

FILTER=""
MODE=""
PATTERN="${2:-}"

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
    *)
        echo -e "${RED}Unknown option: $1${NC}"
        show_help
        exit 1
        ;;
esac

echo -e "${BLUE}${BOLD}🚀 Starting LeeAttend Test Suite in [$MODE] mode${NC}"
if [ -n "$PATTERN" ]; then
    echo -e "${BLUE}🎯 Specific test pattern: ${YELLOW}$PATTERN${NC}"
fi
echo -e "${BLUE}================================================================${NC}"

# Lint luôn chạy trên toàn bộ component (không lọc theo pattern)
run_lint "$FILTER"

# Unit & Integration sẽ nhận pattern để lọc test case
run_unit "$FILTER $PATTERN"
run_integration "$FILTER $PATTERN"

echo -e "\n${BLUE}================================================================${NC}"
echo -e "${GREEN}${BOLD}🎉 All tests for [$MODE] passed successfully!${NC}"

