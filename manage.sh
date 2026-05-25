#!/bin/bash
# LeeAttend Enterprise Management Proxy
# Ensures venv is ready and delegates to the Python Orchestrator.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Ensure venv and get python path
PYTHON_BIN=$("$SCRIPT_DIR/scripts/_ensure-venv.sh" | tail -n 1)

if [ $# -eq 0 ]; then
    "$PYTHON_BIN" "$SCRIPT_DIR/scripts/_orchestrator.py" manage --help
    exit 0
fi

# Run orchestrator in background so we can trap signals
"$PYTHON_BIN" "$SCRIPT_DIR/scripts/_orchestrator.py" manage "$@" &
PID=$!

cleanup_trap() {
    # Forward SIGTERM to Python process
    kill -TERM "$PID" 2>/dev/null
    wait "$PID" 2>/dev/null
    exit 143
}

trap cleanup_trap TERM INT HUP

wait "$PID"
exit $?
