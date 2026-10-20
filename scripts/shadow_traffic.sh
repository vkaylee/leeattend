#!/usr/bin/env bash
# Shadow Traffic Mirroring Orchestration CLI (P8-02)
# Usage: ./scripts/shadow_traffic.sh [start|stop|status|logs]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PID_FILE="/tmp/leeattend_shadow.pid"
LOG_FILE="/tmp/leeattend_shadow.log"

log() {
    echo -e "\033[1;34m[SHADOW]\033[0m $(date -u +'%Y-%m-%d %H:%M:%SZ') $*"
}

status() {
    if [[ -f "$PID_FILE" ]] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
        echo -e "\033[1;32m● Active\033[0m Shadow mirroring running (PID: $(cat "$PID_FILE"))"
        return 0
    else
        echo -e "\033[1;31m○ Inactive\033[0m Shadow mirroring is not running"
        return 1
    fi
}

start() {
    if status >/dev/null 2>&1; then
        log "Shadow mirroring already running with PID $(cat "$PID_FILE")"
        exit 0
    fi

    log "Checking primary (Rust :3000) and shadow (Go :3001) connectivity..."
    if ! curl -s -f "http://127.0.0.1:3000/api/v1/health" >/dev/null 2>&1; then
        log "WARNING: Primary (Rust :3000) not responding on /api/v1/health"
    fi
    if ! curl -s -f "http://127.0.0.1:3001/api/v1/health" >/dev/null 2>&1; then
        log "WARNING: Shadow (Go :3001) not responding on /api/v1/health"
    fi

    log "Starting GoReplay shadow mirror daemon (GET/HEAD read-only safe mode)..."
    if command -v gor >/dev/null 2>&1; then
        nohup gor --input-raw :3000 --output-http http://127.0.0.1:3001 \
            --http-allow-method GET --http-allow-method HEAD \
            --http-allow-url "^/api/.*" --output-http-track-response > "$LOG_FILE" 2>&1 &
        echo $! > "$PID_FILE"
        log "Gor started in background (PID: $!). Logs: $LOG_FILE"
    else
        log "gor binary not in PATH. Starting python dual-proxy shadow daemon..."
        python3 - << 'EOF' > "$LOG_FILE" 2>&1 &
import http.server
import socketserver
import urllib.request
import urllib.error
import threading
import os
import sys

PRIMARY = "http://127.0.0.1:3000"
SHADOW  = "http://127.0.0.1:3001"

class ShadowHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.proxy_request("GET")
    def do_HEAD(self):
        self.proxy_request("HEAD")

    def proxy_request(self, method):
        # 1. Forward to primary
        req_prim = urllib.request.Request(f"{PRIMARY}{self.path}", method=method)
        for h, v in self.headers.items():
            if h.lower() not in ("host", "content-length"):
                req_prim.add_header(h, v)
        try:
            with urllib.request.urlopen(req_prim, timeout=10) as resp_prim:
                status = resp_prim.status
                body = resp_prim.read()
                self.send_response(status)
                for h, v in resp_prim.headers.items():
                    if h.lower() not in ("transfer-encoding", "content-length"):
                        self.send_header(h, v)
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
        except urllib.error.HTTPError as e:
            self.send_response(e.code)
            self.end_headers()
            self.wfile.write(e.read())
        except Exception as e:
            self.send_response(502)
            self.end_headers()
            self.wfile.write(str(e).encode())

        # 2. Asynchronously mirror to shadow (non-blocking)
        def mirror():
            req_shad = urllib.request.Request(f"{SHADOW}{self.path}", method=method)
            for h, v in self.headers.items():
                if h.lower() not in ("host", "content-length"):
                    req_shad.add_header(h, v)
            req_shad.add_header("X-Shadow-Request", "true")
            try:
                with urllib.request.urlopen(req_shad, timeout=5) as r:
                    _ = r.read()
            except Exception:
                pass

        threading.Thread(target=mirror, daemon=True).start()

server = socketserver.TCPServer(("127.0.0.1", 3010), ShadowHandler)
server.serve_forever()
EOF
        echo $! > "$PID_FILE"
        log "Dual-proxy shadow daemon started on :3010 (PID: $!). Logs: $LOG_FILE"
    fi
}

stop() {
    if [[ -f "$PID_FILE" ]]; then
        pid=$(cat "$PID_FILE")
        log "Stopping shadow daemon (PID: $pid)..."
        kill "$pid" 2>/dev/null || true
        rm -f "$PID_FILE"
        log "Stopped."
    else
        log "No active shadow daemon PID file found."
    fi
}

logs() {
    if [[ -f "$LOG_FILE" ]]; then
        tail -n 50 "$LOG_FILE"
    else
        echo "No log file found at $LOG_FILE"
    fi
}

case "${1:-status}" in
    start)  start ;;
    stop)   stop ;;
    status) status ;;
    logs)   logs ;;
    *)
        echo "Usage: $0 {start|stop|status|logs}"
        exit 1
        ;;
esac
