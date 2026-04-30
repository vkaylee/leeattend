#!/bin/bash
# Toolbox Wrapper for Cargo

# Capture where the user is currently calling the script from
CALLER_DIR="$PWD"

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
source "$SCRIPT_DIR/_compose-env.sh"

# Compute relative path from PROJECT_ROOT
REL_DIR=${CALLER_DIR#$PROJECT_ROOT}

TARGET_CONTAINER="apiserver_test"

# If we are in agent-main or the arguments explicitly mention agent, use agent_test
if [[ "$REL_DIR" == *"/agent-main"* ]] || [[ "$*" == *"agent-main"* ]]; then
    TARGET_CONTAINER="agent_test"
fi

cd "$PROJECT_ROOT" || exit 1

echo "🦀 Running cargo inside container environment ($TARGET_CONTAINER) at /workspace$REL_DIR..."
TIMEOUT_CMD=""
if [ -n "$TIMEOUT" ]; then
    TIMEOUT_CMD="timeout $TIMEOUT"
fi

$TIMEOUT_CMD $TOOLBOX_COMPOSE_CMD run -T --rm -e SQLX_OFFLINE=true --workdir "/workspace$REL_DIR" --entrypoint cargo "$TARGET_CONTAINER" "$@" </dev/null 2>&1

