#!/bin/bash
# Unlock SQLite database files by clearing stale WAL/SHM locks
# Usage: ./unlock-db.sh [database.db]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DB_FILE="${1:-$SCRIPT_DIR/test-agent.db}"

if [[ ! -f "$DB_FILE" ]]; then
    echo "Error: Database file not found: $DB_FILE"
    exit 1
fi

echo "Unlocking: $DB_FILE"

# Remove stale WAL files
rm -f "${DB_FILE}-shm" "${DB_FILE}-wal" 2>/dev/null && echo "  Removed WAL/SHM files" || true

# Checkpoint and vacuum
if sqlite3 "$DB_FILE" "PRAGMA wal_checkpoint(TRUNCATE); VACUUM;" >/dev/null 2>&1; then
    echo "  Checkpoint completed"
else
    echo "  Warning: Could not checkpoint (database may still be in use)"
fi

# Verify access
if sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM sqlite_master;" >/dev/null 2>&1; then
    echo "✓ Database unlocked and accessible"
else
    echo "✗ Database still locked. Check for running processes:"
    echo "  lsof $DB_FILE"
    echo "  fuser $DB_FILE"
    exit 1
fi
