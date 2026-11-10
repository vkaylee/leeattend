#!/bin/sh
set -eu

PGHOST="${PGHOST:-pgbouncer}"
PGPORT="${PGPORT:-6432}"
export PGHOST PGPORT

BASELINE_VERSION="20261110000000_single_organization_baseline"
BASELINE_FILE="/workspace/apiserver/migrations/${BASELINE_VERSION}/up.sql"

until pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"; do
  sleep 1
done

migration_table_present=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 --tuples-only --no-align \
  --command="SELECT (to_regclass('public.schema_migrations') IS NOT NULL);")

if [ "$migration_table_present" = "t" ]; then
  marker_count=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align \
    --command="SELECT count(*) FROM public.schema_migrations;")
else
  marker_count=0
fi

if [ "$marker_count" = "1" ]; then
  applied_version=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align \
    --command="SELECT version FROM public.schema_migrations LIMIT 1;")
  if [ "$applied_version" != "$BASELINE_VERSION" ]; then
    printf 'migration state is stale: expected %s, found %s; reset disposable database before retrying\n' \
      "$BASELINE_VERSION" "$applied_version" >&2
    exit 1
  fi
  required_objects=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align \
    --command="SELECT count(*) FROM pg_class WHERE relnamespace = 'public'::regnamespace AND relname IN ('organization_settings', 'roles', 'user_roles', 'role_permissions');")
  if [ "$required_objects" != "4" ]; then
    printf '%s\n' 'migration marker exists but required baseline objects are missing; reset disposable database before retrying' >&2
    exit 1
  fi
elif [ "$marker_count" != "0" ]; then
  printf '%s\n' 'migration state is inconsistent: expected zero markers before fresh baseline or exactly one current marker' >&2
  exit 1
fi

application_tables=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align \
  --command="SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'public' AND c.relkind IN ('r', 'p') AND c.relname <> 'schema_migrations';")

if [ "$marker_count" = "0" ] && [ "$application_tables" != "0" ]; then
  printf '%s\n' 'database is not fresh and has no current baseline marker; reset disposable database before retrying' >&2
  exit 1
fi

if [ "$marker_count" = "0" ]; then
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 \
    --single-transaction -f "$BASELINE_FILE"
fi

exit 0
