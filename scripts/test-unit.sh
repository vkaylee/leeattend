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

APISERVER_CMD="SQLX_OFFLINE=true cargo nextest run --lib $TEST_PATTERN --no-tests=pass"
AGENT_MAIN_CMD="SQLX_OFFLINE=true cargo nextest run --lib $TEST_PATTERN --no-tests=pass"
AGENT_ZK_CMD="if [ -x ./venv/bin/python3 ]; then ./venv/bin/python3 -m pytest -k \"$TEST_PATTERN\"; else echo 'SKIP: venv not setup' && exit 0; fi"
WEB_CMD="npm run test -- $TEST_PATTERN --passWithNoTests"

if [[ "${USE_DOCKER:-}" == "true" ]]; then
    # For Unit Tests, we set CENTRAL_SERVER_URL to invalid to prevent retry loops if tests mistakenly try to connect
    APISERVER_CMD="$DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c \"SQLX_OFFLINE=true cargo nextest run --lib $TEST_PATTERN --no-tests=pass\""
    AGENT_MAIN_CMD="$DOCKER_COMPOSE_CMD exec -T -e CENTRAL_SERVER_URL=ws://unit-test-invalid:3000 agent_test bash -c \"SQLX_OFFLINE=true cargo nextest run --lib $TEST_PATTERN --no-tests=pass\""
    WEB_CMD="$DOCKER_COMPOSE_CMD exec -T web_test npm run test -- $TEST_PATTERN --passWithNoTests"
fi

run_parallel_ordered "Phase 1 (Unit Tests)" "$COMPONENT_FILTER" \
    "1|apiserver|apiserver|$APISERVER_CMD" \
    "2|agent-main|agent-main|$AGENT_MAIN_CMD" \
    "3|agent-zk-worker|agent-zk-worker|$AGENT_ZK_CMD" \
    "4|webdashboard|webdashboard|$WEB_CMD"
