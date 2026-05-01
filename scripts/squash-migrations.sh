#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# Migration Squash Script
# ═══════════════════════════════════════════════════════════════════════════════
# Consolidates all SQLx migrations into a single initial_schema.sql
#
# Strategy: pg_dump --schema-only from a DB with all old migrations applied,
#           then verify by running the new single migration on a fresh DB
#           and comparing schema fingerprints.
#
# Usage:
#   ./scripts/squash-migrations.sh              # Full squash + verify
#   ./scripts/squash-migrations.sh --dry-run    # Verify only, no file changes
#   ./scripts/squash-migrations.sh --dump-only  # Only generate the dump
# ═══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MIGRATIONS_DIR="$PROJECT_ROOT/apiserver/migrations"
SCRATCH_DIR="$PROJECT_ROOT/.migration-squash-work"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; BOLD='\033[1m'; NC='\033[0m'

# Parse args
DRY_RUN=false
DUMP_ONLY=false
for arg in "$@"; do
    case "$arg" in
        --dry-run)  DRY_RUN=true ;;
        --dump-only) DUMP_ONLY=true ;;
    esac
done

# ── Compose helpers ───────────────────────────────────────────────────────────
_ENGINE="podman-compose"
command -v podman-compose &>/dev/null || _ENGINE="docker-compose"
COMPOSE_CMD="$_ENGINE -p leeattend-squash -f $PROJECT_ROOT/docker-compose.yml -f $PROJECT_ROOT/.compose/docker-compose.test.yml"

dc() { $COMPOSE_CMD "$@" </dev/null; }

# For piping SQL via stdin we need a different helper
dc_stdin() { $COMPOSE_CMD "$@"; }

cleanup() {
    echo -e "\n${BLUE}🧹 Cleaning up squash containers...${NC}"
    dc down --volumes --remove-orphans </dev/null >/dev/null 2>&1 || true
}
trap cleanup EXIT

# ── DB helpers ────────────────────────────────────────────────────────────────
DB_USER="test_user"
DB_NAME="test_database"
BEFORE_DB="squash_before"
AFTER_DB="squash_after"

# For scalar queries (count, etc.) — tuples-only, unaligned
psql_scalar() {
    local db="$1"; shift
    dc exec -T db psql -U "$DB_USER" -d "$db" -t -A "$@" </dev/null 2>/dev/null
}

# For CSV fingerprint queries — use --csv format
psql_csv() {
    local db="$1"; shift
    dc exec -T db psql -U "$DB_USER" -d "$db" --csv "$@" </dev/null 2>/dev/null
}

# Pipe SQL from a file into psql via stdin
psql_file() {
    local db="$1"
    local file="$2"
    cat "$file" | dc_stdin exec -T db psql -U "$DB_USER" -d "$db" -v ON_ERROR_STOP=1 2>&1
}

create_db() {
    local db="$1"
    dc exec -T db psql -U "$DB_USER" -d "$DB_NAME" -c "DROP DATABASE IF EXISTS $db;" </dev/null 2>/dev/null || true
    dc exec -T db psql -U "$DB_USER" -d "$DB_NAME" -c "CREATE DATABASE $db;" </dev/null 2>/dev/null
}

# ── Fingerprint functions ─────────────────────────────────────────────────────
capture_fingerprint() {
    local db="$1"
    local prefix="$2"

    echo -e "${BLUE}  📊 Capturing fingerprint for ${BOLD}$db${NC}..."

    # 1. Tables + Columns
    psql_csv "$db" -c "
        SELECT table_name, column_name, data_type, is_nullable,
               column_default, character_maximum_length,
               numeric_precision, numeric_scale
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name NOT LIKE '_sqlx%'
        ORDER BY table_name, ordinal_position
    " > "$SCRATCH_DIR/${prefix}_columns.csv"

    # 2. Indexes
    psql_csv "$db" -c "
        SELECT tablename, indexname, indexdef
        FROM pg_indexes
        WHERE schemaname = 'public'
          AND tablename NOT LIKE '_sqlx%'
        ORDER BY tablename, indexname
    " > "$SCRATCH_DIR/${prefix}_indexes.csv"

    # 3. Constraints (exclude auto-generated NOT NULL names that contain OIDs)
    psql_csv "$db" -c "
        SELECT tc.table_name, tc.constraint_name, tc.constraint_type,
               string_agg(kcu.column_name, ',' ORDER BY kcu.ordinal_position) as columns
        FROM information_schema.table_constraints tc
        LEFT JOIN information_schema.key_column_usage kcu
          ON tc.constraint_name = kcu.constraint_name
          AND tc.table_schema = kcu.table_schema
        WHERE tc.table_schema = 'public'
          AND tc.table_name NOT LIKE '_sqlx%'
          AND tc.constraint_name NOT SIMILAR TO '[0-9]+_[0-9]+_[0-9]+_not_null'
        GROUP BY tc.table_name, tc.constraint_name, tc.constraint_type
        ORDER BY tc.table_name, tc.constraint_name
    " > "$SCRATCH_DIR/${prefix}_constraints.csv"

    # 4. Enums
    psql_csv "$db" -c "
        SELECT t.typname, string_agg(e.enumlabel, ',' ORDER BY e.enumsortorder) as labels
        FROM pg_type t
        JOIN pg_enum e ON t.oid = e.enumtypid
        JOIN pg_namespace n ON t.typnamespace = n.oid
        WHERE n.nspname = 'public'
        GROUP BY t.typname
        ORDER BY t.typname
    " > "$SCRATCH_DIR/${prefix}_enums.csv"

    # 5. Foreign Keys (detailed)
    psql_csv "$db" -c "
        SELECT
            tc.table_name AS from_table,
            kcu.column_name AS from_column,
            ccu.table_name AS to_table,
            ccu.column_name AS to_column,
            rc.update_rule,
            rc.delete_rule
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
          ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage ccu
          ON tc.constraint_name = ccu.constraint_name
        JOIN information_schema.referential_constraints rc
          ON tc.constraint_name = rc.constraint_name
        WHERE tc.table_schema = 'public'
          AND tc.constraint_type = 'FOREIGN KEY'
        ORDER BY tc.table_name, kcu.column_name
    " > "$SCRATCH_DIR/${prefix}_fkeys.csv"

    # 6. Triggers
    psql_csv "$db" -c "
        SELECT trigger_name, event_manipulation, event_object_table,
               action_timing, action_statement
        FROM information_schema.triggers
        WHERE trigger_schema = 'public'
        ORDER BY event_object_table, trigger_name
    " > "$SCRATCH_DIR/${prefix}_triggers.csv"

    # 7. Functions
    psql_csv "$db" -c "
        SELECT p.proname, pg_get_functiondef(p.oid)
        FROM pg_proc p
        JOIN pg_namespace n ON p.pronamespace = n.oid
        WHERE n.nspname = 'public'
        ORDER BY p.proname
    " > "$SCRATCH_DIR/${prefix}_functions.csv"

    local table_count=$(psql_scalar "$db" -c "
        SELECT count(*) FROM information_schema.tables
        WHERE table_schema='public' AND table_name NOT LIKE '_sqlx%'
    ")
    local col_count=$(wc -l < "$SCRATCH_DIR/${prefix}_columns.csv")
    local idx_count=$(wc -l < "$SCRATCH_DIR/${prefix}_indexes.csv")
    echo -e "  ${GREEN}✓${NC} Tables: $table_count | Columns: $col_count | Indexes: $idx_count"
}

compare_fingerprints() {
    echo -e "\n${BOLD}${BLUE}═══ Phase: Schema Comparison ═══${NC}"
    local all_match=true
    local files=("columns" "indexes" "constraints" "enums" "fkeys" "triggers" "functions")

    for f in "${files[@]}"; do
        if diff -q "$SCRATCH_DIR/before_${f}.csv" "$SCRATCH_DIR/after_${f}.csv" >/dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} ${f}: identical"
        else
            echo -e "  ${RED}✗${NC} ${f}: MISMATCH"
            echo -e "    ${YELLOW}Diff:${NC}"
            diff --unified=3 "$SCRATCH_DIR/before_${f}.csv" "$SCRATCH_DIR/after_${f}.csv" > "$SCRATCH_DIR/diff_${f}.txt" 2>&1 || true
            head -40 "$SCRATCH_DIR/diff_${f}.txt"
            all_match=false
        fi
    done

    if $all_match; then
        echo -e "\n  ${GREEN}${BOLD}🎉 ALL FINGERPRINTS MATCH — Schema equivalence confirmed!${NC}"
        return 0
    else
        echo -e "\n  ${RED}${BOLD}❌ SCHEMA MISMATCH DETECTED — Squash FAILED!${NC}"
        echo -e "  ${YELLOW}Check diffs in: $SCRATCH_DIR/${NC}"
        return 1
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════════════

echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════${NC}"
echo -e "${BOLD}${BLUE}  Migration Squash: 144 files → 1${NC}"
echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════${NC}"
echo ""

if $DRY_RUN; then
    echo -e "${YELLOW}📋 DRY RUN — no files will be modified${NC}\n"
fi

# ── Step 0: Setup ─────────────────────────────────────────────────────────────
mkdir -p "$SCRATCH_DIR"

echo -e "${BOLD}${BLUE}═══ Phase: Starting test database ═══${NC}"
dc up -d db </dev/null
echo "Waiting for PostgreSQL to be ready..."
for i in $(seq 1 30); do
    if dc exec -T db pg_isready -U "$DB_USER" </dev/null >/dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} PostgreSQL ready"
        break
    fi
    sleep 1
done

# ── Step 1: "Before" — run all old migrations ────────────────────────────────
echo -e "\n${BOLD}${BLUE}═══ Phase 1: Capture BEFORE fingerprint ═══${NC}"
echo "Creating fresh database '$BEFORE_DB'..."
create_db "$BEFORE_DB"

echo "Running all existing migrations into '$BEFORE_DB'..."

# Run each migration file in order via stdin pipe
MIGRATION_COUNT=0
FAILED=false
find "$MIGRATIONS_DIR" -maxdepth 1 -name '*.sql' ! -name '*.down.*' ! -name '*.down.sql' > "$SCRATCH_DIR/migration_list.txt" 2>&1
sort "$SCRATCH_DIR/migration_list.txt" -o "$SCRATCH_DIR/migration_list_sorted.txt"
while IFS= read -r f; do
    fname=$(basename "$f")
    # Pipe the SQL file content into psql via stdin
    result=$(cat "$f" | dc_stdin exec -T db psql -U "$DB_USER" -d "$BEFORE_DB" -v ON_ERROR_STOP=1 2>&1) || {
        echo -e "${RED}✗ Failed on: $fname${NC}"
        echo "$result" > "$SCRATCH_DIR/migration_error.txt"
        tail -5 "$SCRATCH_DIR/migration_error.txt"
        FAILED=true
        break
    }
    MIGRATION_COUNT=$((MIGRATION_COUNT + 1))
    # Progress indicator every 20 migrations
    if (( MIGRATION_COUNT % 20 == 0 )); then
        echo -e "  ... $MIGRATION_COUNT migrations applied"
    fi
done < "$SCRATCH_DIR/migration_list_sorted.txt"

if $FAILED; then
    exit 1
fi
echo -e "${GREEN}✓${NC} Applied $MIGRATION_COUNT migrations"

capture_fingerprint "$BEFORE_DB" "before"

# ── Step 2: pg_dump → single migration ────────────────────────────────────────
echo -e "\n${BOLD}${BLUE}═══ Phase 2: Generate squashed schema ═══${NC}"

dc exec -T db pg_dump --schema-only --no-owner --no-privileges \
    --no-comments --no-publications --no-subscriptions \
    --exclude-table='_sqlx_migrations' \
    -U "$DB_USER" -d "$BEFORE_DB" </dev/null > "$SCRATCH_DIR/raw_dump.sql" 2>/dev/null

# Clean up pg_dump output: remove SET statements and pg_catalog calls
# but keep actual DDL (CREATE, ALTER, etc.)
{
    echo "-- ═══════════════════════════════════════════════════════════════════"
    echo "-- LeeAttend — Initial Schema (squashed from $MIGRATION_COUNT migrations)"
    echo "-- Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "-- ═══════════════════════════════════════════════════════════════════"
    echo ""
    # Filter out pg_dump boilerplate but keep DDL
    # Removes: empty lines, comments, SET, SELECT pg_catalog, COMMENT ON,
    #          REVOKE, GRANT, and psql meta-commands (\restrict, \unrestrict, etc.)
    grep -v -E '^\s*$|^--|^SET |^SELECT pg_catalog\.|^COMMENT ON|^REVOKE|^GRANT|^\\\\' \
        "$SCRATCH_DIR/raw_dump.sql" || true
} > "$SCRATCH_DIR/initial_schema.sql"

# Generate down migration
{
    echo "-- ═══════════════════════════════════════════════════════════════════"
    echo "-- LeeAttend — Drop entire schema (rollback initial migration)"
    echo "-- ═══════════════════════════════════════════════════════════════════"
    echo ""
    echo "DROP SCHEMA public CASCADE;"
    echo "CREATE SCHEMA public;"
    echo "GRANT ALL ON SCHEMA public TO public;"
} > "$SCRATCH_DIR/initial_schema.down.sql"

DUMP_SIZE=$(wc -c < "$SCRATCH_DIR/initial_schema.sql")
TABLE_COUNT=$(grep -ci "CREATE TABLE" "$SCRATCH_DIR/initial_schema.sql" || true)
INDEX_COUNT=$(grep -ci "CREATE.*INDEX" "$SCRATCH_DIR/initial_schema.sql" || true)
echo -e "${GREEN}✓${NC} Dump generated: ${DUMP_SIZE} bytes, ${TABLE_COUNT} tables, ${INDEX_COUNT} indexes"

if $DUMP_ONLY; then
    echo -e "\n${GREEN}Dump saved to: $SCRATCH_DIR/initial_schema.sql${NC}"
    exit 0
fi

# ── Step 3: "After" — run single new migration ───────────────────────────────
echo -e "\n${BOLD}${BLUE}═══ Phase 3: Capture AFTER fingerprint ═══${NC}"
echo "Creating fresh database '$AFTER_DB'..."
create_db "$AFTER_DB"

echo "Running squashed migration into '$AFTER_DB'..."
result=$(cat "$SCRATCH_DIR/initial_schema.sql" | dc_stdin exec -T db psql -U "$DB_USER" -d "$AFTER_DB" -v ON_ERROR_STOP=1 2>&1) || {
    echo -e "${RED}✗ Squashed migration FAILED to apply!${NC}"
    echo "$result" > "$SCRATCH_DIR/squash_error.txt"
    tail -20 "$SCRATCH_DIR/squash_error.txt"
    exit 1
}
echo -e "${GREEN}✓${NC} Squashed migration applied successfully"

capture_fingerprint "$AFTER_DB" "after"

# ── Step 4: Compare ──────────────────────────────────────────────────────────
compare_fingerprints || exit 1

# ── Step 5: Apply (unless dry-run) ────────────────────────────────────────────
if $DRY_RUN; then
    echo -e "\n${YELLOW}📋 DRY RUN complete. Schema verified. No files changed.${NC}"
    echo -e "   Review: $SCRATCH_DIR/initial_schema.sql"
    exit 0
fi

echo -e "\n${BOLD}${BLUE}═══ Phase 5: Replace migration files ═══${NC}"

# Backup old migrations
BACKUP_DIR="$SCRATCH_DIR/old-migrations-backup"
mkdir -p "$BACKUP_DIR"
cp "$MIGRATIONS_DIR"/*.sql "$BACKUP_DIR/"
ls -1 "$BACKUP_DIR"/*.sql > "$SCRATCH_DIR/backup_list.txt" 2>&1
OLD_COUNT=$(wc -l < "$SCRATCH_DIR/backup_list.txt")
echo -e "${GREEN}✓${NC} Backed up $OLD_COUNT files to $BACKUP_DIR/"

# Remove old migrations
rm -f "$MIGRATIONS_DIR"/*.sql
echo -e "${GREEN}✓${NC} Removed old migration files"

# Place new migrations
cp "$SCRATCH_DIR/initial_schema.sql" "$MIGRATIONS_DIR/20260101000000_initial_schema.sql"
cp "$SCRATCH_DIR/initial_schema.down.sql" "$MIGRATIONS_DIR/20260101000000_initial_schema.down.sql"
echo -e "${GREEN}✓${NC} New migration files placed"

echo -e "\n${BOLD}${GREEN}═══════════════════════════════════════════════${NC}"
echo -e "${BOLD}${GREEN}  ✅ Migration squash complete!${NC}"
echo -e "${BOLD}${GREEN}═══════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${BOLD}Before:${NC} $OLD_COUNT files"
echo -e "  ${BOLD}After:${NC}  2 files (up + down)"
echo -e "  ${BOLD}Backup:${NC} $BACKUP_DIR/"
echo ""
echo -e "  ${YELLOW}Next steps:${NC}"
echo -e "  1. Regenerate .sqlx cache:  ./scripts/_sqlx.sh prepare"
echo -e "  2. Verify compile:          ./scripts/_cargo.sh check"
echo -e "  3. Run tests:               ./run-tests.sh --api </dev/null"
echo -e "  4. Commit:                  git add -A && AGENT_MODE=1 git commit --no-gpg-sign -m 'chore(db): squash migrations into initial schema' </dev/null"
