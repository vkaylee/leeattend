#!/bin/bash

# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\033[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.\033[0m"
    echo -e "👉 This script is an internal module. Use: \033[1m./leedevkit test\033[0m"
    exit 1
fi


# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e " [0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden. [0m"
    echo -e "👉 This script is an internal module. Use:  [1m./leedevkit test [0m"
    exit 1
fi

# scripts/install-hooks.sh

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
PROJECT_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)

echo "🛠️ Installing LeeAttend Git Hooks..."

if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo "❌ .git directory not found. Are you in a git repository?"
    exit 1
fi

# Ensure hooks directory exists
mkdir -p "$PROJECT_ROOT/.git/hooks"

# Install pre-commit hook
if [ -f "$PROJECT_ROOT/.git-hooks/pre-commit" ]; then
    chmod +x "$PROJECT_ROOT/.git-hooks/pre-commit"
    ln -sf "../../.git-hooks/pre-commit" "$PROJECT_ROOT/.git/hooks/pre-commit"
    echo "✅ pre-commit hook installed."
else
    echo "❌ .git-hooks/pre-commit not found."
    exit 1
fi

echo "🚀 Git hooks setup complete!"
