#!/bin/bash
# Migration helper script for converting test files to single binary structure
# Usage: ./migrate-tests.sh <domain> <source_file>
# Example: ./migrate-tests.sh auth auth_tests.rs

set -e

DOMAIN=$1
SOURCE_FILE=$2
BASE_NAME=$(basename "$SOURCE_FILE" .rs | sed 's/_tests$//' | sed 's/_test$//')

if [ -z "$DOMAIN" ] || [ -z "$SOURCE_FILE" ]; then
    echo "Usage: $0 <domain> <source_file>"
    echo "Example: $0 auth auth_tests.rs"
    exit 1
fi

SOURCE_PATH="tests/$SOURCE_FILE"
TARGET_DIR="tests/integration/$DOMAIN"
TARGET_PATH="$TARGET_DIR/$BASE_NAME.rs"

if [ ! -f "$SOURCE_PATH" ]; then
    echo "Error: Source file $SOURCE_PATH not found"
    exit 1
fi

mkdir -p "$TARGET_DIR"

# Copy and transform imports
sed -e 's/^mod common;$/use crate::common::db::TestDatabase;\nuse crate::common::factory::TestFactory;\nuse crate::common::app::{spawn_app, spawn_app_with_factory, setup_app_tower};/' \
    -e 's/use common::db::TestDatabase;/use crate::common::db::TestDatabase;/' \
    -e 's/use common::factory::TestFactory;/use crate::common::factory::TestFactory;/' \
    -e 's/#\[serial\]$//' \
    -e 's/use serial_test::serial;$//' \
    "$SOURCE_PATH" > "$TARGET_PATH"

echo "Migrated: $SOURCE_PATH -> $TARGET_PATH"
echo "NOTE: You may need to manually:"
echo "  1. Add to $TARGET_DIR/mod.rs: mod $BASE_NAME;"
echo "  2. Remove local spawn_app() and use shared version"
echo "  3. Verify imports work correctly"
