#!/bin/bash
# LeeAttend Venv Guard
# Ensures a consistent Python environment for the Enterprise Orchestrator.
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENV_PATH="$PROJECT_ROOT/.venv"

# ANSI Colors
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Detect host python interpreter
if command -v python3 >/dev/null 2>&1; then
    HOST_PYTHON="python3"
elif command -v python >/dev/null 2>&1; then
    HOST_PYTHON="python"
else
    echo -e "${RED}❌ Error: Python not found. Please install Python 3.${NC}" >&2
    exit 1
fi

if [[ ! -d "$VENV_PATH" ]]; then
    echo -e "${BLUE}${BOLD}🐍 Creating Python Virtual Environment using $HOST_PYTHON...${NC}" >&2
    $HOST_PYTHON -m venv "$VENV_PATH"
fi

# Path to the venv python interpreter
VENV_PYTHON="$VENV_PATH/bin/python3"

# Check for essential dependencies
# psutil: safe-run | pytest: tests | pyyaml: compose | requests: health | ruff/mypy: premium lint
if ! "$VENV_PYTHON" -c "import psutil, pytest, yaml, requests, ruff, mypy" 2>/dev/null; then
    echo -e "${YELLOW}📦 Installing enterprise linter suite (ruff, mypy)...${NC}" >&2
    "$VENV_PYTHON" -m pip install --quiet psutil pytest pyyaml requests pytest-cov ruff mypy types-psutil types-pyyaml types-requests
fi

echo "$VENV_PYTHON"
