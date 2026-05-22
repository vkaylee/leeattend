#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# Legacy Data Migration Runner
# ═══════════════════════════════════════════════════════════════
# Usage: ./scripts/run_migration.sh
#
# Prerequisites:
#   - Docker/Podman compose available
#   - dbbackup/bk.sql exists
#   - Python 3 with psycopg2 installed
# ═══════════════════════════════════════════════════════════════

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log()  { echo -e "${BLUE}[migration]${NC} $1"; }
ok()   { echo -e "${GREEN}[migration]${NC} ✅ $1"; }
warn() { echo -e "${YELLOW}[migration]${NC} ⚠️  $1"; }
err()  { echo -e "${RED}[migration]${NC} ❌ $1"; }

# ─── Detect compose command ───
if command -v podman-compose &>/dev/null; then
    COMPOSE="podman-compose"
elif command -v docker-compose &>/dev/null; then
    COMPOSE="docker-compose"
elif command -v docker &>/dev/null && docker compose version &>/dev/null 2>&1; then
    COMPOSE="docker compose"
else
    err "No compose command found (docker compose / podman-compose)"
    exit 1
fi
log "Using: $COMPOSE"
COMPOSE="$COMPOSE --profile migration"

# ─── Check backup file ───
if [ ! -f "dbbackup/bk.sql" ]; then
    err "Backup file not found: dbbackup/bk.sql"
    exit 1
fi
log "Backup file: dbbackup/bk.sql ($(du -h dbbackup/bk.sql | cut -f1))"

# ─── Step 1: Start db + db_legacy ───
log "Starting database services..."
$COMPOSE up -d db
$COMPOSE up -d db_legacy

# ─── Step 2: Wait for healthy ───
log "Waiting for databases to be healthy..."
for svc in db db_legacy; do
    for i in $(seq 1 30); do
        if $COMPOSE exec "$svc" pg_isready -q 2>/dev/null; then
            ok "$svc is ready"
            break
        fi
        if [ "$i" -eq 30 ]; then
            err "$svc did not become ready in time"
            exit 1
        fi
        sleep 2
    done
done

# ─── Step 3: Restore backup into db_legacy ───
log "Checking if legacy data needs to be restored..."
RAILWAY_EXISTS=$($COMPOSE exec db_legacy psql -U postgres -tAc "SELECT 1 FROM pg_database WHERE datname='railway'" 2>/dev/null || echo "")
if [ "$RAILWAY_EXISTS" = "1" ]; then
    warn "Database 'railway' already exists in db_legacy, skipping restore"
else
    log "Restoring backup into db_legacy (this may take a few minutes)..."
    $COMPOSE exec db_legacy psql -U postgres -f /dbbackup/bk.sql > /dev/null 2>&1 || {
        # pg_dumpall may have errors (role exists, etc.) — check if railway DB was created
        RAILWAY_CHECK=$($COMPOSE exec db_legacy psql -U postgres -tAc "SELECT 1 FROM pg_database WHERE datname='railway'" 2>/dev/null || echo "")
        if [ "$RAILWAY_CHECK" != "1" ]; then
            err "Failed to restore backup — 'railway' database not created"
            exit 1
        fi
        warn "Restore had some warnings (expected for pg_dumpall), but 'railway' DB exists"
    }
    ok "Backup restored successfully"
fi

# ─── Step 4: Run system migrations on db ───
log "Ensuring system DB has migrations applied..."
# The apiserver handles this on startup. For manual migration, we use diesel directly.
# For now, we assume the db service has had migrations run by apiserver.
# If not, user should start apiserver once to apply migrations.

# ─── Step 5: Determine ports ───
# Find the host port mappings for the containers
DB_PORT=$($COMPOSE port db 5432 2>/dev/null | cut -d: -f2 || echo "5432")
DB_LEGACY_PORT=$($COMPOSE port db_legacy 5432 2>/dev/null | cut -d: -f2 || echo "")

if [ -z "$DB_LEGACY_PORT" ]; then
    warn "db_legacy port not exposed, will try internal docker network"
    # For internal access, we need to run the script inside a container
    # or expose the port. Let's expose it.
    err "Please add 'ports: [\"5433:5432\"]' to db_legacy in docker-compose.yml, or run the migration script from within the docker network."
    err "Alternative: run the Python script directly with proper URLs"
    log ""
    log "Manual migration command:"
    log "  LEGACY_DB_URL=postgres://postgres:postgres@<db_legacy_host>:5432/railway \\"
    log "  SYSTEM_DB_URL=postgres://admin:change_this_password_immediately@<db_host>:5432/leeattend \\"
    log "  python3 scripts/migrate_legacy_data.py"
    exit 1
fi

# Source .env for DB credentials
set -a
source .env 2>/dev/null || true
set +a

LEGACY_DB_URL="postgres://postgres:postgres@localhost:${DB_LEGACY_PORT}/railway"
SYSTEM_DB_URL="postgres://${DB_USER:-admin}:${DB_PASSWORD:-change_this_password_immediately}@localhost:${DB_PORT}/leeattend"

log "Legacy DB URL: localhost:${DB_LEGACY_PORT}/railway"
log "System DB URL: localhost:${DB_PORT}/leeattend"

# ─── Step 6: Run Python migration ───
log "Running migration script..."
LEGACY_DB_URL="$LEGACY_DB_URL" \
SYSTEM_DB_URL="$SYSTEM_DB_URL" \
"$SCRIPT_DIR/_python.sh" scripts/migrate_legacy_data.py

EXIT_CODE=$?
if [ $EXIT_CODE -eq 0 ]; then
    ok "Migration completed successfully! 🎉"
    log ""
    log "Next steps:"
    log "  1. Stop db_legacy: $COMPOSE --profile migration stop db_legacy"
    log "  2. Start full stack: $COMPOSE up -d"
    log "  3. Login and verify data"
else
    err "Migration failed with exit code $EXIT_CODE"
    exit $EXIT_CODE
fi
