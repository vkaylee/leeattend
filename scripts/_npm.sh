#!/bin/bash
# Toolbox Wrapper for NPM

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
source "$SCRIPT_DIR/_compose-env.sh"

# Intercept typecheck typo common for AI agents
ARGS=("$@")
if [[ "${ARGS[0]}" == "run" && "${ARGS[1]}" == "typecheck" ]]; then
    ARGS[1]="type-check"
fi

echo "📦 Running npm inside container environment..."
TIMEOUT_CMD=""
if [ -n "$TIMEOUT" ]; then
    TIMEOUT_CMD="timeout $TIMEOUT"
fi

$TIMEOUT_CMD $TOOLBOX_COMPOSE_CMD run -T --rm --entrypoint npm web_test "${ARGS[@]}" </dev/null 2>&1
