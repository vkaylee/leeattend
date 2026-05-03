#!/bin/bash
# Proxy for Orchestrator Toolbox (Python)
# Ensures venv and runs python3 with all project dependencies available.

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
VENV_PYTHON=$("$SCRIPT_DIR/_ensure-venv.sh")

exec "$VENV_PYTHON" "$@"
