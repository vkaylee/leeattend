#!/bin/bash
# LeeAttend Internal Bootstrap
# Sourced and managed by the Python Orchestrator.
# ==============================================================================

set -euo pipefail

# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\033[0;31m❌ SECURITY VIOLATION: Direct execution of bootstrap is forbidden.\033[0m"
    echo -e "👉 This script is an internal module. Use: \033[1m./test.sh\033[0m"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ANSI Color Codes (fallback if not exported)
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

leeattend_bootstrap_env() {
    # Set essential variables
    export CONTAINER_ENGINE="${CONTAINER_ENGINE:-podman}"
    local _COMPOSE_ENGINE="${DOCKER_COMPOSE_CMD:-podman-compose}"
    export COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-leeattend-test}"
    export PODMAN_COMPOSE_PROJECT_NAME="$COMPOSE_PROJECT_NAME"
    export DOCKER_COMPOSE_CMD="$_COMPOSE_ENGINE -p $COMPOSE_PROJECT_NAME -f $PROJECT_ROOT/.compose/docker-compose.test.yml"
    export USE_DOCKER=true

    # Profile Injection: Ensure podman-compose sees services in profiles
    if [[ -z "${PROFILES:-}" ]]; then
        case "${MODE:-all}" in
            api) export DOCKER_COMPOSE_CMD="$DOCKER_COMPOSE_CMD --profile lint-api --profile unit-api --profile infra-db-init --profile infra-db" ;;
            integration) export DOCKER_COMPOSE_CMD="$DOCKER_COMPOSE_CMD --profile lint-api --profile unit-api --profile int-api --profile infra-db --profile infra-db-init --profile infra-redis" ;;
            web) export DOCKER_COMPOSE_CMD="$DOCKER_COMPOSE_CMD --profile lint-web --profile unit-web --profile e2e-web" ;;
            all) export DOCKER_COMPOSE_CMD="$DOCKER_COMPOSE_CMD --profile lint-api --profile unit-api --profile int-api --profile lint-web --profile unit-web --profile e2e-web --profile infra-db --profile infra-db-init --profile infra-redis" ;;
        esac
    else
        export DOCKER_COMPOSE_CMD="$DOCKER_COMPOSE_CMD $PROFILES"
    fi

    # Source all modules (using underscore prefix)
    for module in test-utils test-lint test-unit test-integration test-coverage db-ops docker-ops lifecycle test-runner cli-parser; do
        if [[ -f "$SCRIPT_DIR/_${module}.sh" ]]; then
            source "$SCRIPT_DIR/_${module}.sh"
        fi
    done
}

# Execute the requested function
# Usage: ./_bootstrap.sh [FUNCTION_NAME] [ARGS...]
if [[ $# -gt 0 ]]; then
    CMD="$1"
    shift
    # Ensure env is bootstrapped if we are calling a function
    leeattend_bootstrap_env
    "$CMD" "$@"
fi
