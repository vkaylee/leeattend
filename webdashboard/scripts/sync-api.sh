#!/bin/bash
set -e

# Sync API from backend to frontend
# Requires: Cargo (Rust), Node.js

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# 1. Generate OpenAPI spec from backend
echo "Generating openapi.json from apiserver..."
cd "$PROJECT_ROOT/apiserver"
SQLX_OFFLINE=true cargo run -- gen-openapi > "$PROJECT_ROOT/openapi.json"

# 2. Generate TypeScript types
echo "Generating src/types/api-generated.ts from openapi.json..."
cd "$PROJECT_ROOT/webdashboard"
npx openapi-typescript "$PROJECT_ROOT/openapi.json" -o ./src/types/api-generated.ts

echo "✅ Sync complete!"
