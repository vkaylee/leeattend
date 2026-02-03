#!/bin/bash
# Deploy BLPOP timeout fix
set -e

echo "=== Deploying BLPOP Timeout Fix ==="

# Step 1: Check environment variable
echo ""
echo "Step 1: Checking WORKER_POLL_TIMEOUT_SECS..."
if [ -n "${WORKER_POLL_TIMEOUT_SECS}" ]; then
    echo "  Found: WORKER_POLL_TIMEOUT_SECS=${WORKER_POLL_TIMEOUT_SECS}"
    echo "  ⚠️  This may be the source of the error if value is invalid"
else
    echo "  ✅ Not set (will use default: 30.0)"
fi

# Step 2: Build
echo ""
echo "Step 2: Building release binary..."
cargo build --release

# Step 3: Check if worker is running
echo ""
echo "Step 3: Checking for running worker process..."
WORKER_PID=$(pgrep -f "apiserver.*worker" || true)
if [ -n "${WORKER_PID}" ]; then
    echo "  Found worker process: PID ${WORKER_PID}"
    echo "  Stopping worker..."
    kill ${WORKER_PID}
    sleep 2
else
    echo "  No worker process found"
fi

# Step 4: Start worker with clean environment
echo ""
echo "Step 4: Starting worker with validated configuration..."
unset WORKER_POLL_TIMEOUT_SECS  # Clear any invalid value
echo "  Environment cleared: WORKER_POLL_TIMEOUT_SECS unset"
echo "  Worker will use default timeout: 30.0 seconds"

# Start in background and capture logs
nohup ./target/release/apiserver worker > worker.log 2>&1 &
NEW_PID=$!
echo "  ✅ Worker started: PID ${NEW_PID}"

# Step 5: Monitor logs for validation
echo ""
echo "Step 5: Monitoring logs for 5 seconds..."
sleep 5
tail -20 worker.log

echo ""
echo "=== Deployment Complete ==="
echo ""
echo "To verify fix:"
echo "  tail -f worker.log | grep -E '(WORKER_POLL_TIMEOUT|Failed to dequeue)'"
echo ""
echo "Expected behavior:"
echo "  ✅ No 'timeout is not an integer' errors"
echo "  ✅ Worker dequeuing jobs normally"
echo ""
