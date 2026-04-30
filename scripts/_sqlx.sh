#!/bin/bash
# Toolbox Wrapper for SQLx

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
source "$SCRIPT_DIR/_compose-env.sh"

echo "🗄️ Running sqlx inside container environment..."
TIMEOUT_CMD=""
if [ -n "$TIMEOUT" ]; then
    TIMEOUT_CMD="timeout $TIMEOUT"
fi

$TIMEOUT_CMD $TOOLBOX_COMPOSE_CMD run -T --rm --entrypoint sqlx apiserver_test "$@" </dev/null 2>&1
