#!/bin/bash
# Toolbox Wrapper for NPM
# This script forwards all arguments to the npm command inside the web_test container environment.
# Note: npm_init may run automatically if web_test depends on it, but running `npm` directly 
# against web_test just overrides the command and uses the environment.

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
cd "$SCRIPT_DIR/.." || exit 1
export PROJECT_ROOT=$(pwd)

# Intercept typecheck typo common for AI agents
ARGS=("$@")
if [[ "${ARGS[0]}" == "run" && "${ARGS[1]}" == "typecheck" ]]; then
    ARGS[1]="type-check"
fi

echo "📦 Running npm inside container environment..."
exec podman-compose -p leeattend-test -f docker-compose.yml -f .compose/docker-compose.test.yml run --rm --entrypoint npm web_test "${ARGS[@]}"
