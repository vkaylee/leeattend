#!/bin/bash

set -uo pipefail

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FLAG_FILE="$PROJECT_ROOT/.test.sh.flag"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.test.yml"

# Parse arguments
USE_COVERAGE=false
TEST_ARGS=()

for arg in "$@"; do
  if [[ "$arg" == "--cov" ]] || [[ "$arg" == "--coverage" ]]; then
    USE_COVERAGE=true
  else
    TEST_ARGS+=("$arg")
  fi
done

# Check if tests are already running
if [ -f "$FLAG_FILE" ]; then
  echo "Error: Tests are already running (flag file exists: $FLAG_FILE)"
  exit 1
fi

# Create flag file
touch "$FLAG_FILE"

# Define cleanup function
cleanup() {
  echo "Cleaning up..."
  rm -f "$FLAG_FILE"
  # Stop containers and remove volumes (drops the database)
  docker compose -f "$COMPOSE_FILE" down -v
}

# Set trap for cleanup on exit (normal or error)
trap cleanup EXIT

DB_CONTAINER_NAME="test_db"
DB_PORT="5432"
DB_USER="test_user"
DB_PASSWORD="test_password"
DB_NAME="test_database"

DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@test_db:${DB_PORT}/${DB_NAME}"

echo "Starting test database..."
docker compose -f "$COMPOSE_FILE" up -d "$DB_CONTAINER_NAME"

echo "Waiting for database to be ready..."
until docker compose -f "$COMPOSE_FILE" exec "$DB_CONTAINER_NAME" pg_isready -U "$DB_USER" -d "$DB_NAME" > /dev/null 2>&1; do
  printf '.'
  sleep 1
done
echo "Database ready!"

# Run migrations
echo "Running database migrations..."
sqlx migrate run --source "$SCRIPT_DIR/migrations" --database-url "$DATABASE_URL"

export DATABASE_URL_FOR_TEST_GENERATION="$DATABASE_URL"
export DATABASE_URL="$DATABASE_URL"

# Navigate to apiserver directory
cd "$SCRIPT_DIR"

set +e
if [ "$USE_COVERAGE" = true ]; then
  echo "Running cargo tarpaulin (Coverage mode)..."
  cargo tarpaulin --workspace --timeout 300 \
    --engine llvm \
    --exclude-files "src/main.rs" \
    --out xml html \
    --output-dir "${SCRIPT_DIR}/.test_logs/coverage" \
    "${TEST_ARGS[@]}"
  EXIT_CODE=$?
else
  echo "Running cargo test (Standard mode)..."
  cargo test --workspace "${TEST_ARGS[@]}"
  EXIT_CODE=$?
fi
set -e

exit $EXIT_CODE
