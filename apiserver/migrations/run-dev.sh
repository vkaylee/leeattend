#!/bin/sh
set -eu

until pg_isready -h db -U "$POSTGRES_USER" -d "$POSTGRES_DB"; do
  sleep 1
done

psql \
  -h db \
  -U "$POSTGRES_USER" \
  -d "$POSTGRES_DB" \
  -v ON_ERROR_STOP=1 \
  --command="CREATE TABLE IF NOT EXISTS public.schema_migrations (version text PRIMARY KEY, applied_at timestamptz NOT NULL DEFAULT now());"

applied=$(psql \
  -h db \
  -U "$POSTGRES_USER" \
  -d "$POSTGRES_DB" \
  --tuples-only \
  --no-align \
  --command="SELECT 1 FROM public.schema_migrations WHERE version = '20261103000000_init';"
)

if [ "$applied" != "1" ]; then
  baseline_present=$(psql \
    -h db \
    -U "$POSTGRES_USER" \
    -d "$POSTGRES_DB" \
    --tuples-only \
    --no-align \
    --command="SELECT (to_regclass('public.users') IS NOT NULL AND to_regclass('public.workspaces') IS NOT NULL AND to_regclass('public.employees') IS NOT NULL);"
  )

  if [ "$baseline_present" = "t" ]; then
    psql \
      -h db \
      -U "$POSTGRES_USER" \
      -d "$POSTGRES_DB" \
      -v ON_ERROR_STOP=1 \
      --command="INSERT INTO public.schema_migrations(version) VALUES ('20261103000000_init') ON CONFLICT DO NOTHING;"
  else
    psql \
      -h db \
      -U "$POSTGRES_USER" \
      -d "$POSTGRES_DB" \
      -v ON_ERROR_STOP=1 \
      --single-transaction \
      -f /workspace/apiserver/migrations/20261103000000_init/up.sql

    psql \
      -h db \
      -U "$POSTGRES_USER" \
      -d "$POSTGRES_DB" \
      -v ON_ERROR_STOP=1 \
      --command="INSERT INTO public.schema_migrations(version) VALUES ('20261103000000_init');"
  fi
fi

if [ "$(psql -h db -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align --command="SELECT 1 FROM public.schema_migrations WHERE version = '20261106000000_remove_system_roles';")" != "1" ]; then
  psql -h db -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 -f /workspace/apiserver/migrations/20261106000000_remove_system_roles/up.sql
  psql -h db -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --command="INSERT INTO public.schema_migrations(version) VALUES ('20261106000000_remove_system_roles');"
fi
exit 0
