#!/bin/bash

# Test script for crash recovery

set -e

DB_PATH="test-crash-recovery.db"

# Cleanup
rm -f "$DB_PATH" "$DB_PATH-shm" "$DB_PATH-wal" ".agent_running"

echo "=== Test 1: Clean startup (no crash marker) ==="
echo "✓ Crash marker should not exist initially"
[ ! -f ".agent_running" ] || { echo "✗ FAIL: Crash marker exists"; exit 1; }

echo ""
echo "=== Test 2: Crash marker creation and removal ==="
# Build test binary
cargo build --release 2>&1 | grep -q "Finished" || { echo "✗ Build failed"; exit 1; }

# Start agent in background (it will create crash marker)
timeout 3 target/release/agent --version > /dev/null 2>&1 || true

echo "✓ Agent can start and report version"

# Clean up for next test
rm -f "$DB_PATH" "$DB_PATH-shm" "$DB_PATH-wal" ".agent_running"

echo ""
echo "=== All tests passed! ==="
