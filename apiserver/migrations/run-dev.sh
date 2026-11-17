#!/bin/sh
set -eu

PGHOST="${PGHOST:-pgbouncer}"
PGPORT="${PGPORT:-6432}"
export PGHOST PGPORT

if [ "${SEED_EFFECTIVE_DATE:-}" != "" ] && ! printf '%s' "$SEED_EFFECTIVE_DATE" | grep -Eq '^[0-9]{4}-[0-9]{2}-[0-9]{2}$'; then
  printf 'invalid SEED_EFFECTIVE_DATE: expected YYYY-MM-DD\n' >&2
  exit 1
fi

MIGRATIONS_DIR="/workspace/apiserver/migrations/phases"
DEV_ACCOUNTS_FILE="/workspace/scripts/seed_dev_accounts.sql"
SEED_EFFECTIVE_DATE="${SEED_EFFECTIVE_DATE:-$(date -u +%F)}"
SEED_DEV_ACCOUNTS="${SEED_DEV_ACCOUNTS:-true}"

until pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"; do
  sleep 1
done

migration_table_present=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --tuples-only --no-align \
  --command="SELECT (to_regclass('public.schema_migrations') IS NOT NULL);")

if [ "$migration_table_present" = "t" ]; then
  marker_count=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --tuples-only --no-align \
    --command="SELECT count(*) FROM public.schema_migrations;")
else
  marker_count=0
fi

if [ "$marker_count" = "0" ]; then
  application_objects=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --tuples-only --no-align \
    --command="SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'public' AND c.relkind IN ('r', 'p', 'v', 'm', 'S', 'f') AND c.relname <> 'schema_migrations';")
  if [ "$application_objects" != "0" ]; then
    printf '%s\n' 'database is not fresh and has no phase marker; reset disposable database before retrying' >&2
    exit 1
  fi

  for phase in foundation hr_core attendance_schedule payroll; do
    psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
      --single-transaction -f "$MIGRATIONS_DIR/$phase/up.sql"
  done
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
    --set="seed_effective_date=$SEED_EFFECTIVE_DATE" \
    --single-transaction -f "$MIGRATIONS_DIR/runtime_seeds.up.sql"
fi

expected_markers="'fresh_foundation_v1', 'fresh_hr_core_v1', 'fresh_attendance_schedule_v1', 'fresh_payroll_v1', 'fresh_runtime_seeds_v1'"
marker_state=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --tuples-only --no-align \
  --command="SELECT count(*) || ':' || count(*) FILTER (WHERE version NOT IN ($expected_markers)) FROM public.schema_migrations;")
if [ "$marker_state" != '5:0' ]; then
  printf 'migration state is incomplete or stale: expected 5 phase markers, found %s; reset disposable database before retrying\n' \
    "$marker_state" >&2
  exit 1
fi

required_objects=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --tuples-only --no-align \
  --command="SELECT count(*) FROM pg_class WHERE relnamespace = 'public'::regnamespace AND relname IN ('organization_settings', 'roles', 'user_roles', 'role_permissions', 'config_entries', 'employment_relationships', 'attendance_periods', 'payroll_periods', 'payroll_formula_rules', 'payroll_calculation_runs');")
if [ "$required_objects" != "10" ]; then
  printf 'phase marker exists but required objects are missing: expected 10, found %s; reset disposable database before retrying\n' \
    "$required_objects" >&2
  exit 1
fi

if [ "$SEED_DEV_ACCOUNTS" = "true" ]; then
  if [ ! -f "$DEV_ACCOUNTS_FILE" ]; then
    printf '%s\n' 'development account seed file is missing' >&2
    exit 1
  fi
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
    --single-transaction -f "$DEV_ACCOUNTS_FILE"
fi

exit 0
