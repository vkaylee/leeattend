#!/bin/bash
set -euo pipefail

# Get absolute path of the script directory
currentDir=$(cd "$(dirname "$0")" && pwd)
PID_FILE="${currentDir}/.nport.5173.pid"
LOG_FILE="${currentDir}/.nport.5173.log"

# Check if the PID file exists and if the process is actually running
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if ps -p "$OLD_PID" > /dev/null 2>&1; then
        echo "Error: NPORT is already running with PID $OLD_PID."
        exit 1
    else
        echo "Warning: Stale PID file found. Removing it..."
        rm -f "$PID_FILE"
    fi
fi

echo "Starting NPORT..."

# Run in background using nohup and a subshell to manage the PID file lifecycle
# We use bash -c to ensure the trap is correctly registered in the background process
nohup bash -c "
    trap 'rm -f \"$PID_FILE\"' EXIT
    echo \"[$(date)] Starting nport...\" >> \"$LOG_FILE\" 
    npx nport 5173 -s leeattend >> \"$LOG_FILE\" 2>&1
" > /dev/null 2>&1 &

NOHUP_PID=$!

if [ -z "$NOHUP_PID" ]; then
    echo "Error: Failed to start background process."
    exit 1
fi

echo "$NOHUP_PID" > "$PID_FILE"

echo "NPORT started with PID: $NOHUP_PID"
echo "PID stored in: $PID_FILE"
echo "Logs: $LOG_FILE"
echo "To stop: kill $NOHUP_PID"
