#!/bin/bash
# Toolbox Wrapper for SQLx

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
source "$SCRIPT_DIR/_compose-env.sh"

echo "🗄️ Running sqlx inside container environment..."
TIMEOUT_SEC="${TIMEOUT:-600}"
"$SCRIPT_DIR/safe-run.py" "$TIMEOUT_SEC" $TOOLBOX_COMPOSE_CMD run -T --rm --entrypoint sqlx apiserver_test "$@"
