#!/bin/bash
# Toolbox Wrapper for Cargo
# This script forwards all arguments to the cargo command inside the appropriate container environment.

# Capture where the user is currently calling the script from
CALLER_DIR="$PWD"

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
export PROJECT_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)

# Compute relative path from PROJECT_ROOT
# If calling from root, REL_DIR is empty
# If calling from agent-main, REL_DIR is "/agent-main"
REL_DIR=${CALLER_DIR#$PROJECT_ROOT}

TARGET_CONTAINER="apiserver_test"

# If we are in agent-main or the arguments explicitly mention agent, use agent_test
if [[ "$REL_DIR" == *"/agent-main"* ]] || [[ "$*" == *"agent-main"* ]]; then
    TARGET_CONTAINER="agent_test"
fi

cd "$PROJECT_ROOT" || exit 1

echo "🦀 Running cargo inside container environment ($TARGET_CONTAINER) at /workspace$REL_DIR..."
exec podman-compose -p leeattend-test -f docker-compose.yml -f .compose/docker-compose.test.yml run -T --rm -e SQLX_OFFLINE=true --workdir "/workspace$REL_DIR" --entrypoint cargo "$TARGET_CONTAINER" "$@" 2>&1

