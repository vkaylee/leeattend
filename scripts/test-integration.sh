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

echo -e "  Pre-compiling backend for integration tests..."
(cd "$PROJECT_ROOT/apiserver" && cargo test --no-run -j ${CARGO_BUILD_JOBS:-2}) > "$LOG_DIR/prebuild.log" 2>&1 || { echo -e "${RED}Backend build failed. Check $LOG_DIR/prebuild.log${NC}"; exit 1; }

export RUST_TEST_THREADS=$NUM_WORKERS

# For Playwright, we use -g for grep if pattern is provided
PW_ARGS=""
if [ -n "$TEST_PATTERN" ]; then
    PW_ARGS="-g \"$TEST_PATTERN\""
fi

run_parallel_ordered "Phase 3 (Integration & E2E)" "$COMPONENT_FILTER" \
    "1|apiserver-int|apiserver|timeout 300s cargo test --test '*' $TEST_PATTERN -- --test-threads $NUM_WORKERS" \
    "2|playwright-e2e|webdashboard|timeout 300s npx playwright test $PW_ARGS --workers $NUM_WORKERS"
