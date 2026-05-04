#!/bin/bash
# Wrapper for git command using _safe_run.py to prevent PTY hangs
# Argument processing is delegated to _git_wrapper.py to ensure 100% robust string handling

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
VENV_PYTHON=$("$SCRIPT_DIR/_ensure-venv.sh")

exec "$VENV_PYTHON" "$SCRIPT_DIR/_git_wrapper.py" "$@"
