#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# Shared Compose Environment — sourced by toolbox wrappers
# ═══════════════════════════════════════════════════════════════════════════════
# Provides:
#   TOOLBOX_COMPOSE_CMD  — compose command with correct project, engine, files
#   PROJECT_ROOT         — absolute path to project root
#
# Usage:
#   source "$(dirname "$(readlink -f "$0")")/_compose-env.sh"

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "❌ This script is meant to be sourced, not run directly."
    exit 1
fi

_TOOLBOX_DIR="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"
PROJECT_ROOT="$(cd "$_TOOLBOX_DIR/.." && pwd)"
export PROJECT_ROOT

if [[ -z "${TOOLBOX_COMPOSE_CMD:-}" ]]; then
    if command -v podman-compose &>/dev/null; then
        _ENGINE="podman-compose"
    elif command -v docker-compose &>/dev/null; then
        _ENGINE="docker-compose"
    elif command -v docker &>/dev/null; then
        _ENGINE="docker compose"
    else
        echo "No supported Compose engine found." >&2
        return 1
    fi
    # The test Compose file is self-contained; enable both infrastructure and application profiles.
    export TOOLBOX_COMPOSE_CMD="$_ENGINE -p leeattend-test --profile infra-db --profile infra-redis --profile api --profile frontend -f $PROJECT_ROOT/.compose/docker-compose.test.yml"
fi

unset _TOOLBOX_DIR _ENGINE
