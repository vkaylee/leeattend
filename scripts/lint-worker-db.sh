#!/usr/bin/env bash
set -e

echo "🔍 Running JobWorker DB Connection Starvation Linter..."

# Define directories to check
DIRS_TO_CHECK=("apiserver/src/worker/handlers/" "apiserver/src/cache/")

# Flag for errors
ERROR_FOUND=0

# Grep for tokio::join!
for dir in "${DIRS_TO_CHECK[@]}"; do
    if grep -r -n "tokio::join!" "$dir" --include="*.rs"; then
        echo ""
        echo "❌ ERROR: Found 'tokio::join!' inside worker/cache directory."
        echo "   Executing queries in parallel inside a worker handler can lead to"
        echo "   Database Connection Pool Exhaustion (Deadlocks) when worker concurrency"
        echo "   exceeds the pool size."
        echo "   Please refactor these calls to execute sequentially (await one by one)."
        ERROR_FOUND=1
    fi
done

if [ "$ERROR_FOUND" -eq 1 ]; then
    exit 1
else
    echo "✅ No parallel database execution risks found in workers."
    exit 0
fi
