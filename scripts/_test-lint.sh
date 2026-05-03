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

# LeeAttend Linting Methods
# ==============================================================================

leeattend_run_lint() {
    local filter="${1:-}"
    local mode="${MODE:-all}"
    
    local BACKEND_CMD="cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check"
    local WEB_LINT_CMD="bun run lint"
    local WEB_TYPECHECK_CMD="bun run type-check"
    local SYNC_API_CMD="./scripts/_sync-api.sh"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Lint requires DB with migrations (sqlx verifies queries at compile time)
        BACKEND_CMD="$DOCKER_COMPOSE_CMD exec -T -w /workspace apiserver bash -c \"cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check\" </dev/null"
        WEB_LINT_CMD="$DOCKER_COMPOSE_CMD exec -T webdashboard bun run lint </dev/null"
        WEB_TYPECHECK_CMD="$DOCKER_COMPOSE_CMD exec -T webdashboard bun run type-check </dev/null"
        
        # API Sync via shared volume between apiserver and webdashboard:
        SYNC_API_CMD="$DOCKER_COMPOSE_CMD exec -T apiserver bash -c \"cargo run -- gen-openapi > /workspace/shared/openapi.json\" </dev/null && $DOCKER_COMPOSE_CMD exec -T webdashboard bash -c \"npx openapi-typescript /workspace/shared/openapi.json -o ./src/types/api-generated.ts\" </dev/null"
    fi

    local TASKS=()
    # Always run backend lint in 'api', 'integration', or 'all'
    if [[ "$mode" == "all" || "$mode" == "api" || "$mode" == "integration" ]]; then
        TASKS+=("1|rust-backend-clippy|apiserver|$BACKEND_CMD")
    fi

    # Run web lint in 'web' or 'all'
    if [[ "$mode" == "all" || "$mode" == "web" ]]; then
        TASKS+=("3|webdashboard-lint|webdashboard|$WEB_LINT_CMD")
        TASKS+=("4|webdashboard-typecheck|webdashboard|$WEB_TYPECHECK_CMD")
    fi

    # API Sync only if BOTH are involved or explicitly 'all'
    if [[ "$mode" == "all" ]]; then
        TASKS+=("0|api-sync|webdashboard|$SYNC_API_CMD")
    fi

    run_parallel_ordered "Phase 0 Linting" "$filter" "${TASKS[@]}"
}
