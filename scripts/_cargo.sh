#!/bin/bash
# Toolbox Wrapper for Cargo
# This script forwards all arguments to the cargo command inside the apiserver_test container environment.

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
cd "$SCRIPT_DIR/.." || exit 1

echo "🦀 Running cargo inside container environment..."
exec podman-compose -p leeattend-test -f docker-compose.yml -f .compose/docker-compose.test.yml run --rm --entrypoint cargo apiserver_test "$@"
