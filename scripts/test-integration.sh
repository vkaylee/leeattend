#!/bin/bash
set -euo pipefail
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SOURCE_DIR/test-utils.sh"

# Extract filter and optional pattern
INPUT_FILTER="${1:-}"
COMPONENT_FILTER=$(echo "$INPUT_FILTER" | cut -d':' -f1)
TEST_PATTERN=$(echo "$INPUT_FILTER" | cut -d':' -f2-)

# If pattern is the same as filter (no space), then pattern is empty
if [ "$COMPONENT_FILTER" == "$TEST_PATTERN" ]; then
    TEST_PATTERN=""
fi

export DATABASE_URL="postgres://test_user:test_password@localhost:5433/leeattend_test_template"

echo -e "  Pre-compiling backend for integration tests (timeout: ${TIMEOUT_BUILD}s)..."
run_with_timeout "$TIMEOUT_BUILD" bash -c "cd '$PROJECT_ROOT/apiserver' && cargo test --no-run -j ${CARGO_BUILD_JOBS:-2}" > "$LOG_DIR/prebuild.log" 2>&1
prebuild_exit_code=$?
if [ $prebuild_exit_code -ne 0 ]; then
    if [ $prebuild_exit_code -eq 137 ] || [ $prebuild_exit_code -eq 124 ]; then
        echo -e "${RED}❌ Backend pre-compile TIMED OUT after ${TIMEOUT_BUILD}s. Check $LOG_DIR/prebuild.log${NC}"
    else
        echo -e "${RED}❌ Backend build failed (exit: $prebuild_exit_code). Check $LOG_DIR/prebuild.log${NC}"
    fi
    tail -50 "$LOG_DIR/prebuild.log"
    exit 1
fi

export RUST_TEST_THREADS=$NUM_WORKERS

# For Playwright, we use -g for grep if pattern is provided
PW_ARGS=""
if [ -n "$TEST_PATTERN" ]; then
    PW_ARGS="-g \"$TEST_PATTERN\""
fi

# Note: Individual task timeouts are handled by run_parallel_ordered via TIMEOUT_INTEGRATION
run_parallel_ordered "Phase 3 (Integration & E2E)" "$COMPONENT_FILTER" \
    "1|apiserver-int|apiserver|cargo test --test '*' $TEST_PATTERN -- --test-threads $NUM_WORKERS" \
    "2|playwright-e2e|webdashboard|npx playwright test $PW_ARGS --workers $NUM_WORKERS"
