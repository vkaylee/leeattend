#!/usr/bin/env bash
# GoReplay (Gor) Traffic Shadowing Engine (P8-02)
# Usage: ./gor-traffic-mirror.sh [live|record|replay|verify]

set -euo pipefail

PRIMARY_PORT="${PRIMARY_PORT:-3000}"
SHADOW_PORT="${SHADOW_PORT:-3001}"
CAPTURE_FILE="${CAPTURE_FILE:-/tmp/shadow_traffic.gor}"
GOR_BIN="${GOR_BIN:-gor}"

log() {
    echo "[$(date -u +'%Y-%m-%dT%H:%M:%SZ')] [SHADOW-GOR] $*"
}

check_gor() {
    if ! command -v "$GOR_BIN" >/dev/null 2>&1; then
        log "Warning: 'gor' binary not found in PATH. Checking local or container fallback."
    fi
}

cmd="${1:-live}"

case "$cmd" in
    live)
        log "Starting live shadow traffic mirroring from :$PRIMARY_PORT to :$SHADOW_PORT (Read-Only GET/HEAD safe mode)..."
        # Mirroring with HTTP filter (GET, HEAD only) to prevent double mutations
        # --input-raw captures incoming requests to Rust port 3000
        # --output-http forwards requests asynchronously to Go port 3001
        # --output-http-track-response enables response logging for zero-divergence diffing
        exec "$GOR_BIN" \
            --input-raw ":${PRIMARY_PORT}" \
            --output-http "http://127.0.0.1:${SHADOW_PORT}" \
            --http-allow-method GET \
            --http-allow-method HEAD \
            --http-allow-url "^/api/.*" \
            --output-http-track-response \
            --output-http-stats \
            --stats
        ;;

    record)
        log "Recording incoming production/staging traffic on :$PRIMARY_PORT to $CAPTURE_FILE..."
        exec "$GOR_BIN" \
            --input-raw ":${PRIMARY_PORT}" \
            --output-file "$CAPTURE_FILE" \
            --http-allow-url "^/api/.*"
        ;;

    replay)
        log "Replaying captured traffic from $CAPTURE_FILE to :$SHADOW_PORT..."
        exec "$GOR_BIN" \
            --input-file "$CAPTURE_FILE" \
            --output-http "http://127.0.0.1:${SHADOW_PORT}" \
            --output-http-track-response \
            --stats
        ;;

    verify)
        log "Checking shadow targets readiness..."
        curl -s -f "http://127.0.0.1:${PRIMARY_PORT}/api/v1/health" >/dev/null && log "Primary (:3000) is Healthy"
        curl -s -f "http://127.0.0.1:${SHADOW_PORT}/api/v1/health" >/dev/null && log "Shadow (:3001) is Healthy"
        ;;

    *)
        echo "Usage: $0 [live|record|replay|verify]"
        exit 1
        ;;
esac
