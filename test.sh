#!/bin/bash
# LeeAttend Enterprise Test Proxy
# Ensures venv is ready and delegates to the Python Orchestrator.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Ensure venv and get python path
PYTHON_BIN=$("$SCRIPT_DIR/scripts/_ensure-venv.sh" | tail -n 1)

if [ $# -eq 0 ]; then
    "$PYTHON_BIN" "$SCRIPT_DIR/scripts/_orchestrator.py" test --help
    exit 0
fi

export SKIP_SAFE_RUN=1 # Orchestrator handles safe-run internally
"$PYTHON_BIN" "$SCRIPT_DIR/scripts/_orchestrator.py" test "$@"
