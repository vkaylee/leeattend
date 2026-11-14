#!/bin/bash
# LeeAttend Go build gates (P0-06).
#
# Canonical, tracked gate chain for the Go apiserver. Runs inside the pinned
# Go container defined by .compose/go/Dockerfile so tool versions are fixed by
# the tracked image, never by host state or devkit internals.
#
# Usage:
#   scripts/_go-gates.sh              # all gates + race tests
#   scripts/_go-gates.sh --lint-only  # format/vet/staticcheck/govulncheck/mod verify/generated diff
#   scripts/_go-gates.sh --unit-only  # go test -race only
#   scripts/_go-gates.sh --package ./internal/domain/pairing
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
export PROJECT_ROOT
COMPOSE_FILE="$PROJECT_ROOT/.compose/docker-compose.test.yml"
SERVICE="apiserver"
PROJECT_NAME="${COMPOSE_PROJECT_NAME:-leeattend-test}"

MODE="all"
PACKAGE="./..."
FIX=false
while [ $# -gt 0 ]; do
    case "$1" in
        --lint-only) MODE="lint" ;;
        --unit-only) MODE="unit" ;;
        --fix) FIX=true ;;
        --package) PACKAGE="${2:-./...}"; shift ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

ENGINE=("podman-compose")
if ! command -v podman-compose >/dev/null 2>&1; then
    if command -v docker-compose >/dev/null 2>&1; then
        ENGINE=("docker-compose")
    elif command -v docker >/dev/null 2>&1; then
        ENGINE=("docker" "compose")
    else
        echo "No supported Compose engine found" >&2
        exit 1
    fi
fi
COMPOSE=("${ENGINE[@]}" -p "$PROJECT_NAME" -f "$COMPOSE_FILE" --profile go)

cd "$PROJECT_ROOT"
"${COMPOSE[@]}" up -d --no-deps "$SERVICE" >/dev/null

read -r -d '' GATES <<'GATES_EOF' || true
set -euo pipefail
fail() { echo "GATE FAIL: $1" >&2; exit 1; }

git config --global --add safe.directory '*' 2>/dev/null || true
export GOFLAGS="-mod=mod -buildvcs=false"

if [ "${MODE}" != "unit" ]; then
    if [ "${FIX}" = "true" ]; then
        gofmt -w $(find . -name "*.go")
    fi
    UNFORMATTED="$(find . -name "*.go" -exec gofmt -l {} +)" || fail "gofmt"
    [ -z "$UNFORMATTED" ] || { echo "$UNFORMATTED"; fail "gofmt listed unformatted files"; }

    echo "== go vet =="
    go vet ${PACKAGE} || fail "go vet"

    echo "== staticcheck =="
    staticcheck ${PACKAGE} || fail "staticcheck"

    # Existing complexity debt is report-only during gradual remediation.
    # Tool/configuration failures still fail; lint findings do not block the gate.
    echo "== golangci-lint (complexity report) =="
    golangci-lint run ${PACKAGE} --issues-exit-code=0 || fail "golangci-lint"

    echo "== govulncheck =="
    govulncheck ${PACKAGE} || fail "govulncheck"

    echo "== go mod verify =="
    go mod verify || fail "go mod verify"

    if [ -f sqlc.yaml ] || [ -f sqlc.json ]; then
        echo "== sqlc diff =="
        sqlc diff || fail "sqlc diff"
    fi
fi

if [ "${MODE}" != "lint" ]; then
    echo "== go test -race =="
    go test -race -cover ${PACKAGE} || fail "go test -race"
fi
echo "ALL GATES PASSED"
GATES_EOF

exec "${COMPOSE[@]}" exec -T -w /workspace/apiserver "$SERVICE" \
    env MODE="$MODE" PACKAGE="$PACKAGE" FIX="$FIX" bash -c "$GATES"
