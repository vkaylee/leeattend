#!/bin/bash
# Generate SQLx metadata files for offline compilation

set -e

echo "Starting test database..."
docker compose -p leeattend-test -f ../docker-compose.test.yml up -d test_db

echo "Waiting for database..."
sleep 10

DATABASE_URL="postgres://test_user:test_password@test_db:5432/test_database"

echo "Building temporary container with sqlx-cli..."
docker run --rm -it \
  --network leeattend-test_default \
  -v "$(pwd):/workspace" \
  -w /workspace \
  -e DATABASE_URL="$DATABASE_URL" \
  rust:1.88 bash -c "
    apt-get update && apt-get install -y postgresql-client
    cargo install sqlx-cli --no-default-features --features postgres --locked
    sqlx migrate run
    cargo sqlx prepare --workspace -- --all-targets
  "

echo "SQLx data generated successfully!"
echo "Stopping test database..."
docker compose -p leeattend-test -f ../docker-compose.test.yml down

