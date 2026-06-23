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
COMPOSE_DEV="$COMPOSE -f docker-compose.yml -f .compose/docker-compose.dev.yml"

# ─── Check backup file ───
if [ ! -f "dbbackup/bk.sql" ]; then
    err "Backup file not found: dbbackup/bk.sql"
    exit 1
fi
log "Backup file: dbbackup/bk.sql ($(du -h dbbackup/bk.sql | cut -f1))"

# ─── Step 1: Start db_system + db_workspace + db_legacy ───
log "Starting database services..."
$COMPOSE_DEV up -d db_system
$COMPOSE_DEV up -d db_workspace
$COMPOSE_DEV up -d db_legacy

# ─── Step 2: Wait for healthy ───
log "Waiting for databases to be healthy..."
for svc in db_system db_workspace db_legacy; do
    for i in $(seq 1 30); do
        if $COMPOSE exec "$svc" pg_isready -q 2>/dev/null || $COMPOSE exec "$svc" pg_isready -U postgres -q 2>/dev/null || $COMPOSE exec "$svc" pg_isready -U tenant_user -q 2>/dev/null; then
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
# Check if table 'employees' exists in 'railway' database
RAILWAY_EXISTS=$($COMPOSE_DEV exec db_legacy psql -U postgres -d railway -tAc "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'employees');" 2>/dev/null || echo "")
RAILWAY_EXISTS=$(echo "$RAILWAY_EXISTS" | tr -d '\r' | tr -d '\n' | tr -d ' ')
if [ "$RAILWAY_EXISTS" = "t" ]; then
    warn "Legacy database 'railway' with valid schema already exists, skipping restore"
else
    log "Restoring backup into db_legacy (this may take a few minutes)..."
    $COMPOSE_DEV exec db_legacy psql -U postgres -c "DROP DATABASE IF EXISTS railway;" >/dev/null 2>&1 || true
    $COMPOSE_DEV exec db_legacy psql -U postgres -c "CREATE DATABASE railway;" >/dev/null 2>&1 || true
    $COMPOSE_DEV exec db_legacy pg_restore -U postgres --no-owner --no-acl -d railway /dbbackup/bk.sql > /dev/null 2>&1 || {
        # Check if 'employees' table was created successfully
        RAILWAY_CHECK=$($COMPOSE_DEV exec db_legacy psql -U postgres -d railway -tAc "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'employees');" 2>/dev/null || echo "")
        RAILWAY_CHECK=$(echo "$RAILWAY_CHECK" | tr -d '\r' | tr -d '\n' | tr -d ' ')
        if [ "$RAILWAY_CHECK" != "t" ]; then
            err "Failed to restore backup — 'employees' table not created in 'railway' DB"
            exit 1
        fi
        warn "Restore had some warnings, but legacy schema exists"
    }
    ok "Backup restored successfully"
fi

# Ensure workspaces table exists with tenant workspace ID (needed by migration script)
$COMPOSE_DEV exec db_legacy psql -U postgres -d railway -c "CREATE TABLE IF NOT EXISTS public.workspaces (id uuid PRIMARY KEY, name varchar(100) NOT NULL);" >/dev/null
$COMPOSE_DEV exec db_legacy psql -U postgres -d railway -c "INSERT INTO public.workspaces (id, name) VALUES ('6e913c39-24f3-45b5-877d-e307c3e10c88', 'Default Workspace') ON CONFLICT DO NOTHING;" >/dev/null

# ─── Reset database schemas for rollback test ───
log "Resetting database schemas to ensure clean state for migration rollback tests..."
$COMPOSE exec db_system psql -U "${DB_USER:-admin}" -d "${DB_NAME:-leeattend}" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" >/dev/null
$COMPOSE exec db_workspace psql -U tenant_user -d tenant_db -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" >/dev/null

# Calculate number of migrations to test (all except initial squash migration)
NUM_SYSTEM_MIGS=$(find apiserver/migrations/system -maxdepth 1 -mindepth 1 -type d | wc -l)
NUM_SYSTEM_REVERT=$((NUM_SYSTEM_MIGS - 1))

NUM_TENANT_MIGS=$(find apiserver/migrations/tenant -maxdepth 1 -mindepth 1 -type d | wc -l)
NUM_TENANT_REVERT=$((NUM_TENANT_MIGS - 1))

# 4.1: Test system migrations (run -> revert -> run again)
log "Running rollback test for SYSTEM migrations (reverting $NUM_SYSTEM_REVERT migrations)..."
$COMPOSE_DEV run --rm --no-deps -e DATABASE_URL="postgres://${DB_USER:-admin}:${DB_PASSWORD:-change_this_password_immediately}@db_system:5432/${DB_NAME:-leeattend}" --entrypoint diesel apiserver migration run --migration-dir ./migrations/system
if [ "$NUM_SYSTEM_REVERT" -gt 0 ]; then
    $COMPOSE_DEV run --rm --no-deps -e DATABASE_URL="postgres://${DB_USER:-admin}:${DB_PASSWORD:-change_this_password_immediately}@db_system:5432/${DB_NAME:-leeattend}" --entrypoint diesel apiserver migration revert --number "$NUM_SYSTEM_REVERT" --migration-dir ./migrations/system
    $COMPOSE_DEV run --rm --no-deps -e DATABASE_URL="postgres://${DB_USER:-admin}:${DB_PASSWORD:-change_this_password_immediately}@db_system:5432/${DB_NAME:-leeattend}" --entrypoint diesel apiserver migration run --migration-dir ./migrations/system
fi
ok "SYSTEM migrations rollback test passed!"

# 4.2: Test tenant migrations (run -> revert -> run again)
log "Running rollback test for TENANT migrations (reverting $NUM_TENANT_REVERT migrations)..."
$COMPOSE_DEV run --rm --no-deps -e DATABASE_URL="postgres://tenant_user:tenant_password@db_workspace:5432/tenant_db" --entrypoint diesel apiserver migration run --migration-dir ./migrations/tenant
if [ "$NUM_TENANT_REVERT" -gt 0 ]; then
    $COMPOSE_DEV run --rm --no-deps -e DATABASE_URL="postgres://tenant_user:tenant_password@db_workspace:5432/tenant_db" --entrypoint diesel apiserver migration revert --number "$NUM_TENANT_REVERT" --migration-dir ./migrations/tenant
    $COMPOSE_DEV run --rm --no-deps -e DATABASE_URL="postgres://tenant_user:tenant_password@db_workspace:5432/tenant_db" --entrypoint diesel apiserver migration run --migration-dir ./migrations/tenant
fi
ok "TENANT migrations rollback test passed!"

# ─── Step 5: Determine ports ───
# Find the host port mappings for the containers
DB_PORT=$($COMPOSE_DEV port db_system 5432 2>/dev/null | cut -d: -f2 || echo "5434")
DB_LEGACY_PORT=$($COMPOSE_DEV port db_legacy 5432 2>/dev/null | cut -d: -f2 || echo "5433")

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

DB_WORKSPACE_PORT=$($COMPOSE_DEV port db_workspace 5432 2>/dev/null | cut -d: -f2 || echo "5435")

LEGACY_DB_URL="postgres://postgres:postgres@localhost:${DB_LEGACY_PORT}/railway"
SYSTEM_DB_URL="postgres://${DB_USER:-admin}:${DB_PASSWORD:-change_this_password_immediately}@localhost:${DB_PORT}/leeattend"
WORKSPACE_DB_URL="postgres://tenant_user:tenant_password@localhost:${DB_WORKSPACE_PORT}/tenant_db"

log "Legacy DB URL: localhost:${DB_LEGACY_PORT}/railway"
log "System DB URL: localhost:${DB_PORT}/leeattend"
log "Workspace DB URL: localhost:${DB_WORKSPACE_PORT}/tenant_db"

# ─── Step 6: Run Python migration ───
log "Running migration script..."
LEGACY_DB_URL="$LEGACY_DB_URL" \
SYSTEM_DB_URL="$SYSTEM_DB_URL" \
WORKSPACE_DB_URL="$WORKSPACE_DB_URL" \
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
