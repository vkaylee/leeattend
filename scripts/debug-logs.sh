#!/bin/bash
LOG_DIR=".test_logs"

show_help() {
    echo "Usage: ./scripts/debug-logs.sh [option]"
    echo "Options:"
    echo "  --list          List all available log files"
    echo "  --last          Show the content of the most recent log file"
    echo "  --cat [name]    Show the content of a specific log file"
    echo "  --errors        Search for errors/panics in all logs"
    echo "  --tail [N]      Tail the most recent log file (default 20 lines)"
}

if [ ! -d "$LOG_DIR" ]; then
    echo "Error: Log directory $LOG_DIR not found."
    exit 1
fi

case "$1" in
    --list)
        ls -lh "$LOG_DIR"
        ;;
    --last)
        LAST_LOG=$(ls -t "$LOG_DIR"/*.log 2>/dev/null | head -n 1)
        if [ -n "$LAST_LOG" ]; then
            echo "--- Content of $LAST_LOG ---"
            cat "$LAST_LOG"
        else
            echo "No log files found."
        fi
        ;;
    --cat)
        FILE=$(ls "$LOG_DIR"/*"$2"* 2>/dev/null | head -n 1)
        if [ -n "$FILE" ] && [ -f "$FILE" ]; then
            cat "$FILE"
        else
            echo "File matching *$2* not found in $LOG_DIR"
        fi
        ;;
    --errors)
        echo "🔍 Searching for errors in $LOG_DIR..."
        grep -Ei "error|panic|failed|exception" "$LOG_DIR"/*.log
        ;;
    --tail)
        N=${2:-20}
        LAST_LOG=$(ls -t "$LOG_DIR"/*.log 2>/dev/null | head -n 1)
        if [ -n "$LAST_LOG" ]; then
            tail -n "$N" "$LAST_LOG"
        fi
        ;;
    *)
        show_help
        ;;
esac
