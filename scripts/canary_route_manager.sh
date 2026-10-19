#!/usr/bin/env bash
# Canary Route & Percentage Weight Manager (P8-04)
# Usage:
#   ./scripts/canary_route_manager.sh set-weight <0|1|5|25|50|100>
#   ./scripts/canary_route_manager.sh set-domain <domain_name> <rust|go>
#   ./scripts/canary_route_manager.sh status
#   ./scripts/canary_route_manager.sh reload

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CADDY_DIR="$ROOT_DIR/staging/caddy"
WEIGHT_CONF="$CADDY_DIR/canary_weights.conf"
DOMAIN_CONF="$CADDY_DIR/canary_domains.conf"

log() {
    echo -e "\033[1;36m[CANARY]\033[0m $(date -u +'%Y-%m-%d %H:%M:%SZ') $*"
}

reload_caddy() {
    log "Reloading Caddy configuration..."
    if curl -s -f -X POST http://127.0.0.1:2019/load -H "Content-Type: text/caddyfile" --data-binary @"$CADDY_DIR/Caddyfile" >/dev/null 2>&1; then
        log "Caddy hot-reloaded successfully via Admin API (:2019)."
    elif docker compose -f "$ROOT_DIR/docker-compose.staging.yml" exec caddy caddy reload --config /etc/caddy/Caddyfile >/dev/null 2>&1; then
        log "Caddy hot-reloaded successfully via docker-compose exec."
    else
        log "Note: Caddy service not currently reachable on :2019. Configuration files updated on disk."
    fi
}

set_weight() {
    local weight="$1"
    log "Setting canary traffic weight to ${weight}% (Candidate: Go :3001, Baseline: Rust :3000)..."

    case "$weight" in
        0)
            cat << 'EOF' > "$WEIGHT_CONF"
# Canary Weight: 0% (All baseline traffic routed to Rust apiserver:3000)
# No random split active.
EOF
            ;;
        1)
            cat << 'EOF' > "$WEIGHT_CONF"
# Canary Weight: 1% Go (api-go:3001), 99% Rust (apiserver:3000)
@canary_1pct {
    path /api/*
    header_regexp Cookie "canary=go"
}
handle @canary_1pct {
    reverse_proxy api-go:3001 {
        header_up X-Canary-Weight "1%"
    }
}
# Hash / pseudo-random sampling (1 out of 100 request IDs)
@canary_random_1pct {
    path /api/*
    header_regexp X-Request-ID ".*[0-9a-f]{2}-[0-9a-f]{2}0[0-9a-f]-.*"
}
handle @canary_random_1pct {
    reverse_proxy api-go:3001 {
        header_up X-Canary-Weight "1%"
    }
}
EOF
            ;;
        5)
            cat << 'EOF' > "$WEIGHT_CONF"
# Canary Weight: 5% Go (api-go:3001), 95% Rust (apiserver:3000)
@canary_5pct {
    path /api/*
    header_regexp X-Request-ID ".*[0-9a-f]{2}-[0-9a-f]{2}[0-4][0-9a-f]-.*"
}
handle @canary_5pct {
    reverse_proxy api-go:3001 {
        header_up X-Canary-Weight "5%"
    }
}
EOF
            ;;
        25)
            cat << 'EOF' > "$WEIGHT_CONF"
# Canary Weight: 25% Go (api-go:3001), 75% Rust (apiserver:3000)
@canary_25pct {
    path /api/*
    header_regexp X-Request-ID ".*[0-9a-f]{2}-[0-9a-f]{2}[0-3].*"
}
handle @canary_25pct {
    reverse_proxy api-go:3001 {
        header_up X-Canary-Weight "25%"
    }
}
EOF
            ;;
        50)
            cat << 'EOF' > "$WEIGHT_CONF"
# Canary Weight: 50% Go (api-go:3001), 50% Rust (apiserver:3000)
@canary_50pct {
    path /api/*
    header_regexp X-Request-ID ".*[0-9a-f]{2}-[0-9a-f]{2}[0-7].*"
}
handle @canary_50pct {
    reverse_proxy api-go:3001 {
        header_up X-Canary-Weight "50%"
    }
}
EOF
            ;;
        100)
            cat << 'EOF' > "$WEIGHT_CONF"
# Canary Weight: 100% Go (Full Cutover to api-go:3001)
handle /api/* {
    reverse_proxy api-go:3001 {
        header_up X-Canary-Weight "100%"
        header_up X-Forwarded-Host {host}
    }
}
EOF
            ;;
        *)
            echo "Error: Unsupported canary weight '${weight}'. Valid weights: 0, 1, 5, 25, 50, 100"
            exit 1
            ;;
    esac

    reload_caddy
    log "Canary weight updated to ${weight}%."
}

set_domain() {
    local domain="$1"
    local target="$2"
    log "Configuring domain '${domain}' to route to '${target}'..."

    case "$target" in
        go)
            # Add or update domain cutover
            grep -q "handle /api/v1/${domain}/" "$DOMAIN_CONF" || cat << EOF >> "$DOMAIN_CONF"

# Cutover Domain: ${domain}
handle /api/v1/${domain}/* {
    reverse_proxy api-go:3001 {
        header_up X-Domain-Cutover "${domain}"
    }
}
EOF
            ;;
        rust)
            # Remove domain cutover to fall back to Rust default
            sed -i "/handle \/api\/v1\/${domain}\/\*/,/}/d" "$DOMAIN_CONF"
            ;;
        *)
            echo "Error: target must be 'go' or 'rust'"
            exit 1
            ;;
    esac

    reload_caddy
    log "Domain '${domain}' updated."
}

status() {
    echo "======================================================="
    echo "            LEEATTEND CANARY ROUTING STATUS            "
    echo "======================================================="
    echo "Caddy Directory : $CADDY_DIR"
    echo ""
    echo "Active Canary Weights File ($WEIGHT_CONF):"
    cat "$WEIGHT_CONF"
    echo ""
    echo "Active Domain Cutovers ($DOMAIN_CONF):"
    if grep -E "^handle" "$DOMAIN_CONF" >/dev/null 2>&1; then
        grep -E "^handle" "$DOMAIN_CONF"
    else
        echo "  (No per-domain cutovers active; relying on weights/defaults)"
    fi
    echo "======================================================="
}

cmd="${1:-status}"
case "$cmd" in
    set-weight)
        if [[ $# -lt 2 ]]; then
            echo "Usage: $0 set-weight <0|1|5|25|50|100>"
            exit 1
        fi
        set_weight "$2"
        ;;
    set-domain)
        if [[ $# -lt 3 ]]; then
            echo "Usage: $0 set-domain <domain_name> <rust|go>"
            exit 1
        fi
        set_domain "$2" "$3"
        ;;
    status)
        status
        ;;
    reload)
        reload_caddy
        ;;
    *)
        echo "Usage: $0 {set-weight|set-domain|status|reload}"
        exit 1
        ;;
esac
