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
export PROJECT_ROOT="$(cd "$_TOOLBOX_DIR/.." && pwd)"

if [[ -z "${TOOLBOX_COMPOSE_CMD:-}" ]]; then
    _ENGINE="podman-compose"
    command -v podman-compose &>/dev/null || _ENGINE="docker-compose"
    export TOOLBOX_COMPOSE_CMD="$_ENGINE -p leeattend-test -f $PROJECT_ROOT/docker-compose.yml -f $PROJECT_ROOT/.compose/docker-compose.test.yml"
fi

unset _TOOLBOX_DIR _ENGINE
