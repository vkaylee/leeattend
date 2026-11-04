#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
MIGRATION_DIR="$PROJECT_DIR/apiserver/migrations"

usage() {
    cat <<'EOF'
Usage:
  scripts/run_migration.sh
  scripts/run_migration.sh --rollback-check

Required environment:
  DATABASE_URL  PostgreSQL URL for the single LeeAttend database.

The default command applies pending migrations only. --rollback-check is
destructive: it reverts every migration and reapplies the baseline. It requires
LEEATTEND_DISPOSABLE_DATABASE=1 and MUST target a disposable database.
EOF
}

rollback_check=false
case "${1:-}" in
    "") ;;
    --rollback-check) rollback_check=true ;;
    --help|-h) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
esac

if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "DATABASE_URL is required for the single LeeAttend database" >&2
    exit 1
fi

if [[ ! -d "$MIGRATION_DIR" || ! -f "$MIGRATION_DIR/20261103000000_init/up.sql" || ! -f "$MIGRATION_DIR/20261103000000_init/down.sql" ]]; then
    echo "unified migration baseline not found at $MIGRATION_DIR" >&2
    exit 1
fi

cd "$PROJECT_DIR"

if [[ "$rollback_check" == true ]]; then
    if [[ "${LEEATTEND_DISPOSABLE_DATABASE:-}" != "1" ]]; then
        echo "--rollback-check requires LEEATTEND_DISPOSABLE_DATABASE=1" >&2
        exit 1
    fi
    DATABASE_URL="$DATABASE_URL" ./scripts/_diesel.sh migration revert --all --migration-dir ./apiserver/migrations
    DATABASE_URL="$DATABASE_URL" ./scripts/_diesel.sh migration run --migration-dir ./apiserver/migrations
else
    DATABASE_URL="$DATABASE_URL" ./scripts/_diesel.sh migration run --migration-dir ./apiserver/migrations
fi
