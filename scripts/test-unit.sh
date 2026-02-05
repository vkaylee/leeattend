#!/bin/bash
set -uo pipefail
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

# Note: For cargo, pattern is just an argument. For npm, we might need -- <pattern>
# We'll use a simple approach for now.

run_parallel_ordered "Phase 1 (Unit Tests)" "$COMPONENT_FILTER" \
    "1|apiserver|apiserver|SQLX_OFFLINE=true cargo test --lib $TEST_PATTERN" \
    "2|agent-main|agent-main|SQLX_OFFLINE=true cargo test --lib $TEST_PATTERN" \
    "3|agent-zk-worker|agent-zk-worker|if [ -x ./venv/bin/python3 ]; then ./venv/bin/python3 -m pytest -k \"$TEST_PATTERN\"; else echo 'SKIP: venv not setup - run: python3 -m venv venv && ./venv/bin/pip install pytest' && exit 0; fi" \
    "4|webdashboard|webdashboard|npm run test -- $TEST_PATTERN --passWithNoTests"
