#!/bin/sh
set -eu

PGHOST="${PGHOST:-pgbouncer}"
PGPORT="${PGPORT:-6432}"
export PGHOST PGPORT

until pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"; do
  sleep 1
done

psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --command="CREATE TABLE IF NOT EXISTS public.schema_migrations (version text PRIMARY KEY, applied_at timestamptz NOT NULL DEFAULT now());"

is_applied() {
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align \
    --command="SELECT 1 FROM public.schema_migrations WHERE version = '$1';"
}

baseline_present=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align \
  --command="SELECT (to_regclass('public.users') IS NOT NULL AND to_regclass('public.workspaces') IS NOT NULL AND to_regclass('public.employees') IS NOT NULL);")

if [ "$(is_applied 20261103000000_init)" != "1" ]; then
  if [ "$baseline_present" = "t" ]; then
    psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
      --command="INSERT INTO public.schema_migrations(version) VALUES ('20261103000000_init') ON CONFLICT DO NOTHING;"
  else
    psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
      --single-transaction -f /workspace/apiserver/migrations/apply/init.sql
  fi
fi

if [ "$(is_applied 20261106000000_remove_system_roles)" != "1" ]; then
  legacy_tables=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align \
    --command="SELECT (to_regclass('public.system_roles') IS NOT NULL AND to_regclass('public.user_system_roles') IS NOT NULL AND to_regclass('public.system_db_clusters') IS NOT NULL);")
  if [ "$legacy_tables" != "t" ]; then
    printf '%s\n' 'migration 20261106000000 is unrecorded but legacy tables are missing; inspect database state' >&2
    exit 1
  fi
  organization_count=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align \
    --command="SELECT count(*) FROM public.workspaces;")
  if [ "$organization_count" = "0" ]; then
    psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
      --single-transaction -f /workspace/apiserver/migrations/apply/remove_system_roles_fresh.sql
  else
    psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
      -f /workspace/apiserver/migrations/apply/remove_system_roles_populated.sql
  fi
fi

if [ "$(is_applied 20261107000000_add_window_layout_to_dashboard_preferences)" != "1" ]; then
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
    --single-transaction -f /workspace/apiserver/migrations/apply/dashboard_preferences.sql
fi

if [ "$(is_applied 20261108000000_single_organization_cutover)" != "1" ]; then
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
    --single-transaction -f /workspace/apiserver/migrations/apply/single_organization_cutover.sql
fi

exit 0
