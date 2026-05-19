#!/bin/sh
set -e

# ──────────────────────────────────────────────────────────────────────
# CloudBeaver Dev Entrypoint
# ──────────────────────────────────────────────────────────────────────
# The wizard is skipped via ENV vars: CB_SERVER_NAME, CB_SERVER_URL,
# CB_ADMIN_NAME, CB_ADMIN_PASSWORD (handled natively by CloudBeaver).
#
# This script only injects the pre-configured PostgreSQL data-source
# with real credentials from the environment on first run.
# ──────────────────────────────────────────────────────────────────────

MOUNTED_CONF="/opt/cloudbeaver/conf/initial-data-sources.conf.tpl"
TARGET_CONF="/opt/cloudbeaver/conf/initial-data-sources.conf"

# Only inject credentials on first run (workspace not yet initialized)
if [ ! -f /opt/cloudbeaver/workspace/.data/cb.h2v2.dat ] && [ -f "$MOUNTED_CONF" ]; then
    echo "[entrypoint] First run — injecting DB credentials into data-sources config"
    sed \
        -e "s/\${DB_NAME}/${DB_NAME}/g" \
        -e "s/\${DB_USER}/${DB_USER}/g" \
        -e "s/\${DB_PASSWORD}/${DB_PASSWORD}/g" \
        "$MOUNTED_CONF" > "$TARGET_CONF"
fi

# Hand off to the original CloudBeaver launcher
exec /opt/cloudbeaver/run-cloudbeaver-server.sh "$@"
