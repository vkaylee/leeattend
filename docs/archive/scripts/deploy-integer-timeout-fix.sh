#!/bin/bash
# Quick deployment of BLPOP integer timeout fix
set -e

echo "=== Deploying BLPOP Integer Timeout Fix ==="
echo ""

# Stop existing worker
echo "Step 1: Stopping existing worker..."
pkill -f "apiserver.*worker" || echo "  No worker running"
sleep 1

# Start fixed worker
echo ""
echo "Step 2: Starting fixed worker..."
./target/release/apiserver worker > worker.log 2>&1 &
NEW_PID=$!
echo "  ✅ Worker started: PID ${NEW_PID}"

# Monitor logs
echo ""
echo "Step 3: Monitoring logs for 5 seconds..."
sleep 5

echo ""
echo "=== Recent Logs ==="
tail -15 worker.log

echo ""
echo "=== Verification ==="
echo "Check for BLPOP errors:"
if grep -q "timeout is not an integer" worker.log; then
    echo "  ❌ ERROR STILL PRESENT - Investigation needed"
    exit 1
else
    echo "  ✅ No BLPOP timeout errors found"
fi

echo ""
echo "Monitor continuously:"
echo "  tail -f worker.log | grep -E '(Failed to dequeue|ERROR)'"
echo ""
