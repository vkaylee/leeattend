#!/bin/bash
# Toolbox Wrapper for bun

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
source "$SCRIPT_DIR/_compose-env.sh"

# Intercept typecheck typo common for AI agents
ARGS=("$@")
if [[ "${ARGS[0]}" == "run" && "${ARGS[1]}" == "typecheck" ]]; then
    ARGS[1]="type-check"
fi

echo "📦 Running bun inside container environment..."
TIMEOUT_SEC="${TIMEOUT:-600}"
"$SCRIPT_DIR/safe-run.py" "$TIMEOUT_SEC" $TOOLBOX_COMPOSE_CMD run -T --rm --entrypoint bash webdashboard -c "npm install -g bun && bun ${ARGS[*]}"
