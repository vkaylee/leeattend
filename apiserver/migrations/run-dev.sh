#!/bin/sh
set -eu

PGHOST="${PGHOST:-pgbouncer}"
PGPORT="${PGPORT:-6432}"
export PGHOST PGPORT

until pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"; do
  sleep 1
done

psql \
  -U "$POSTGRES_USER" \
  -d "$POSTGRES_DB" \
  -v ON_ERROR_STOP=1 \
  --command="CREATE TABLE IF NOT EXISTS public.schema_migrations (version text PRIMARY KEY, applied_at timestamptz NOT NULL DEFAULT now());"

applied=$(psql \
  -U "$POSTGRES_USER" \
  -d "$POSTGRES_DB" \
  --tuples-only \
  --no-align \
  --command="SELECT 1 FROM public.schema_migrations WHERE version = '20261103000000_init';"
)

if [ "$applied" != "1" ]; then
  baseline_present=$(psql \
    -U "$POSTGRES_USER" \
    -d "$POSTGRES_DB" \
    --tuples-only \
    --no-align \
    --command="SELECT (to_regclass('public.users') IS NOT NULL AND to_regclass('public.workspaces') IS NOT NULL AND to_regclass('public.employees') IS NOT NULL);"
  )

  if [ "$baseline_present" = "t" ]; then
    psql \
      -U "$POSTGRES_USER" \
      -d "$POSTGRES_DB" \
      -v ON_ERROR_STOP=1 \
      --command="INSERT INTO public.schema_migrations(version) VALUES ('20261103000000_init') ON CONFLICT DO NOTHING;"
  else
    psql \
      -U "$POSTGRES_USER" \
      -d "$POSTGRES_DB" \
      -v ON_ERROR_STOP=1 \
      --single-transaction \
      -f /workspace/apiserver/migrations/20261103000000_init/up.sql

    psql \
      -U "$POSTGRES_USER" \
      -d "$POSTGRES_DB" \
      -v ON_ERROR_STOP=1 \
      --command="INSERT INTO public.schema_migrations(version) VALUES ('20261103000000_init');"
  fi
fi

if [ "$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align --command="SELECT 1 FROM public.schema_migrations WHERE version = '20261106000000_remove_system_roles';")" != "1" ]; then
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 -f /workspace/apiserver/migrations/20261106000000_remove_system_roles/up.sql
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --command="INSERT INTO public.schema_migrations(version) VALUES ('20261106000000_remove_system_roles');"
fi

if [ "$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align --command="SELECT 1 FROM public.schema_migrations WHERE version = '20261107000000_add_window_layout_to_dashboard_preferences';")" != "1" ]; then
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 -f /workspace/apiserver/migrations/20261107000000_add_window_layout_to_dashboard_preferences/up.sql
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --command="INSERT INTO public.schema_migrations(version) VALUES ('20261107000000_add_window_layout_to_dashboard_preferences');"
fi
exit 0
