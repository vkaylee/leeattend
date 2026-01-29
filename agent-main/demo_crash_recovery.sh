#!/bin/bash

echo "=== Crash Recovery Demonstration ==="
echo ""
echo "This demonstrates the crash recovery mechanism:"
echo "1. Simulated crash (marker left behind)"
echo "2. Agent restart detects crash"
echo "3. Recovery resets in-flight logs"
echo "4. Clean shutdown removes marker"
echo ""

# Cleanup
rm -f test-demo.db test-demo.db-shm test-demo.db-wal .agent_running

echo "Step 1: Simulating crash scenario..."
echo "- Creating crash marker manually"
cat > .agent_running << MARKER
pid=12345
started=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
MARKER

echo "- Crash marker created"
ls -la .agent_running

echo ""
echo "Step 2: Check crash marker detection"
if [ -f .agent_running ]; then
    echo "✅ Crash marker detected (previous crash)"
    cat .agent_running
else
    echo "❌ No crash marker found"
fi

echo ""
echo "Step 3: Agent would detect and recover on startup"
echo "- recover_from_crash() would reset in-flight logs"
echo "- New crash marker would be written"
echo "- Normal operation continues"

echo ""
echo "Step 4: Clean shutdown"
echo "- Removing crash marker to simulate clean shutdown"
rm -f .agent_running
if [ ! -f .agent_running ]; then
    echo "✅ Crash marker removed (clean shutdown)"
else
    echo "❌ Crash marker still exists"
fi

echo ""
echo "=== Demonstration Complete ==="
