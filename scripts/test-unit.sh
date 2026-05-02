#!/bin/bash

# ==============================================================================
# ⚠️ INTERNAL MODULE — DO NOT RUN DIRECTLY
# Sourced by: ./run-tests.sh
# ==============================================================================
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "❌ This script is an internal module. Do not run directly."
    echo "👉 Use: ./run-tests.sh --api  (or --all)"
    exit 1
fi
# ==============================================================================

# LeeAttend Unit Testing Methods
# ==============================================================================

leeattend_run_unit() {
    local filter_input="${1:-}"
    
    # Extract filter and optional pattern
    local COMPONENT_FILTER=$(echo "$filter_input" | cut -d':' -f1)
    local TEST_PATTERN=$(echo "$filter_input" | cut -d':' -f2-)

    # If pattern is the same as filter (no space), then pattern is empty
    if [ "$COMPONENT_FILTER" == "$TEST_PATTERN" ]; then
        TEST_PATTERN=""
    fi

    # Apply shard filtering to test pattern
    if [[ -n "${SHARD_N:-}" && -n "${SHARD_M:-}" ]]; then
        local shard_flag="--shard=$SHARD_N/$SHARD_M"
        echo -e "${BLUE}🧩 Applying shard filter: $shard_flag${NC}"
    else
        local shard_flag=""
    fi

    local BACKEND_CMD="cargo nextest run --workspace --lib $TEST_PATTERN --no-tests=pass"
    local WEB_CMD="bun run test -- $TEST_PATTERN --passWithNoTests"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        BACKEND_CMD="$DOCKER_COMPOSE_CMD exec -T -w /workspace apiserver bash -c \"cargo nextest run --workspace --lib $TEST_PATTERN $shard_flag --no-tests=pass || [ \\$? -eq 4 ]\" </dev/null"
        WEB_CMD="$DOCKER_COMPOSE_CMD exec -T webdashboard bun run test -- $TEST_PATTERN $shard_flag --passWithNoTests </dev/null"
    fi

    run_parallel_ordered "Phase 1 (Unit Tests)" "$COMPONENT_FILTER" \
        "1|rust-backend|apiserver|$BACKEND_CMD" \
        "4|webdashboard|webdashboard|$WEB_CMD"
}
