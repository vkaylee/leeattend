#!/bin/bash
set -e

# Default values
DB_HOST="${DB_HOST:-test_db}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-test_user}"
DB_PASS="${DB_PASS:-test_password}"
DB_NAME="${DB_NAME:-test_database}"
TEMPLATE_DB="leeattend_test_template"

echo "Waiting for database at $DB_HOST:$DB_PORT..."
until PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c '\q' > /dev/null 2>&1; do
  echo -n "."
  sleep 1
done
echo " Database is up!"

# Create template database if not exists
echo "Check/Create template database: $TEMPLATE_DB"
PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tc "SELECT 1 FROM pg_database WHERE datname = '$TEMPLATE_DB'" | grep -q 1 || \
PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "CREATE DATABASE $TEMPLATE_DB"

# Run migrations on the template database
echo "Running migrations on $TEMPLATE_DB..."
# We use the special setup_migrations test to run migrations without installing sqlx-cli
export DATABASE_URL="postgres://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$TEMPLATE_DB"
cargo test --test setup_migrations -- --ignored

echo "Running API Server Tests..."
# Unset DATABASE_URL so tests pick it up from .env.test or generate their own valid connection strings
# Actually, apiserver tests usually require DATABASE_URL pointing to a valid admin-capable DB (like test_database)
# to create new ephemeral DBs.
export DATABASE_URL="postgres://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$DB_NAME"

# Run the actual tests
cargo test --workspace --exclude setup_migrations
