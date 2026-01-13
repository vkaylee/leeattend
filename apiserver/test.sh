#!/bin/bash

set -euo pipefail

DB_CONTAINER_NAME="test_db"
DB_PORT="5433"
DB_USER="test_user"
DB_PASSWORD="test_password"
DB_NAME="test_database"

DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"

echo "Starting test database..."
docker-compose -f ./docker-compose.test.yml up -d "$DB_CONTAINER_NAME"

echo "Waiting for database to be ready..."
until docker-compose -f ../docker-compose.test.yml exec "$DB_CONTAINER_NAME" pg_isready -U "$DB_USER" -d "$DB_NAME"; do
  printf '.'
  sleep 1
done
echo "Database ready!"

export DATABASE_URL_FOR_TEST_GENERATION="$DATABASE_URL"
export DATABASE_URL="$DATABASE_URL" # Also set default DATABASE_URL for cargo tarpaulin

echo "Running cargo tarpaulin for apiserver..."
cd apiserver
cargo tarpaulin --verbose --workspace --timeout 120 --exclude-files "src/main.rs"

# Capture the exit code of tarpaulin
TARPAULIN_EXIT_CODE=$?

echo "Stopping test database..."
docker-compose -f ./docker-compose.test.yml down

exit $TARPAULIN_EXIT_CODE