#!/bin/bash
# Proxy for Orchestrator Toolbox (NPM/Bun)

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
VENV_PYTHON=$("$SCRIPT_DIR/_ensure-venv.sh")

# Pass all arguments to orchestrator run command
$VENV_PYTHON "$SCRIPT_DIR/_orchestrator.py" run npm "$@"
