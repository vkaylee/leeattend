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
#   scripts/_go-gates.sh --integration # migrate disposable DB and run tagged DB tests
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
LINT_BASE_REF="${LINT_BASE_REF:-origin/main}"
FIX=false
INTEGRATION=false
FORMAT_FILE=""
TEST_PATTERN=""
while [ $# -gt 0 ]; do
    case "$1" in
        --lint-only) MODE="lint" ;;
        --unit-only) MODE="unit" ;;
        --integration) MODE="integration"; INTEGRATION=true ;;
        --fix) FIX=true ;;
        --format-file) FORMAT_FILE="${2:-}"; shift ;;
        --test-pattern) TEST_PATTERN="${2:-}"; shift ;;
        --package) PACKAGE="${2:-./...}"; shift ;;
        --base-ref) LINT_BASE_REF="${2:-origin/main}"; shift ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

LINT_BASE_COMMIT="$(git rev-parse --verify "${LINT_BASE_REF}^{commit}")" || {
    echo "GATE FAIL: lint base ref not found: ${LINT_BASE_REF}" >&2
    exit 1
}
LINT_GO_CHANGED=false
if ! git diff --quiet "${LINT_BASE_COMMIT}" -- ':(glob)apiserver/**/*.go'; then
    LINT_GO_CHANGED=true
fi

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

if [ "${MODE}" != "unit" ] && [ "${MODE}" != "integration" ]; then
    if [ "${FIX}" = "true" ]; then
        gofmt -w $(find . -name "*.go")
    fi
    if [ -n "${FORMAT_FILE}" ]; then
        UNFORMATTED="$(gofmt -l "${FORMAT_FILE}")" || fail "gofmt"
    else
        UNFORMATTED="$(find . -name "*.go" -exec gofmt -l {} +)" || fail "gofmt"
    fi
    [ -z "$UNFORMATTED" ] || { echo "$UNFORMATTED"; fail "gofmt listed unformatted files"; }

    echo "== go vet =="
    go vet ${PACKAGE} || fail "go vet"

    echo "== staticcheck =="
    staticcheck ${PACKAGE} || fail "staticcheck"

    echo "== golangci-lint (complexity report) =="
    golangci-lint run ${PACKAGE} --issues-exit-code=0 || fail "golangci-lint report"

    echo "== golangci-lint (new findings) =="
    if [ "${LINT_GO_CHANGED}" != "true" ]; then
        echo "No apiserver Go source changes relative to ${LINT_BASE_COMMIT}; new-finding scan skipped."
    else
        golangci-lint run ${PACKAGE} --new-from-rev="${LINT_BASE_COMMIT}" || fail "golangci-lint new findings"
    fi

    echo "== govulncheck =="
    govulncheck ${PACKAGE} || fail "govulncheck"

    echo "== go mod verify =="
    go mod verify || fail "go mod verify"

    if [ -f sqlc.yaml ] || [ -f sqlc.json ]; then
        echo "== sqlc diff =="
        sqlc diff || fail "sqlc diff"
    fi
fi

if [ "${MODE}" = "integration" ]; then
    echo "== gofmt integration source =="
    gofmt -w "${FORMAT_FILE:-internal/repository}"
    echo "== go test -tags integration =="
    LEEATTEND_DB_INTEGRATION=1 go test -tags=integration -count=1 ${PACKAGE} ${TEST_PATTERN:+-run "${TEST_PATTERN}"} || fail "go test -tags integration"
elif [ "${MODE}" != "lint" ]; then
    echo "== go test -race =="
    go test -race -cover ${PACKAGE} || fail "go test -race"
fi
echo "ALL GATES PASSED"
GATES_EOF

if [ "$INTEGRATION" = "true" ]; then
    PROFILE_COMPOSE=("${ENGINE[@]}" -p "$PROJECT_NAME" -f "$COMPOSE_FILE" --profile int-go --profile infra-db --profile infra-pooler --profile infra-redis)
    "${PROFILE_COMPOSE[@]}" up -d db pgbouncer redis "$SERVICE" >/dev/null
    "${PROFILE_COMPOSE[@]}" exec -T db sh -c 'until pg_isready -U test_user -d test_database; do sleep 1; done' </dev/null
    "${PROFILE_COMPOSE[@]}" exec -T redis sh -c 'until redis-cli ping >/dev/null; do sleep 1; done' </dev/null
    "${PROFILE_COMPOSE[@]}" exec -T db psql -v ON_ERROR_STOP=1 -U test_user -d postgres -c 'DROP DATABASE IF EXISTS test_database WITH (FORCE);' </dev/null
    "${PROFILE_COMPOSE[@]}" exec -T db psql -v ON_ERROR_STOP=1 -U test_user -d postgres -c 'CREATE DATABASE test_database;' </dev/null
    for phase in foundation hr_core attendance_schedule payroll; do
        migration_file="$PROJECT_ROOT/apiserver/migrations/phases/$phase/up.sql"
        [ -f "$migration_file" ] || fail "missing migration phase $phase"
        "${PROFILE_COMPOSE[@]}" exec -T db psql -v ON_ERROR_STOP=1 -U test_user -d test_database < "$migration_file" || fail "migration phase $phase"
    done
    "${PROFILE_COMPOSE[@]}" exec -T db psql -v ON_ERROR_STOP=1 -U test_user -d test_database \
        --set=seed_effective_date="$(date -u +%F)" < "$PROJECT_ROOT/apiserver/migrations/phases/runtime_seeds.up.sql" \
        || fail "migration phase runtime_seeds"
    "${PROFILE_COMPOSE[@]}" exec -T db psql -v ON_ERROR_STOP=1 -U test_user -d test_database \
        < "$PROJECT_ROOT/apiserver/migrations/testdata/phased_schema_contract.sql" \
        || { echo "GATE FAIL: phased schema contract" >&2; exit 1; }
    exec "${PROFILE_COMPOSE[@]}" exec -T -w /workspace/apiserver "$SERVICE" \
        env MODE="$MODE" PACKAGE="$PACKAGE" FIX="$FIX" FORMAT_FILE="$FORMAT_FILE" TEST_PATTERN="$TEST_PATTERN" LINT_BASE_COMMIT="$LINT_BASE_COMMIT" LINT_GO_CHANGED="$LINT_GO_CHANGED" bash -c "$GATES"
fi

exec "${COMPOSE[@]}" exec -T -w /workspace/apiserver "$SERVICE" \
    env MODE="$MODE" PACKAGE="$PACKAGE" FIX="$FIX" FORMAT_FILE="$FORMAT_FILE" TEST_PATTERN="$TEST_PATTERN" LINT_BASE_COMMIT="$LINT_BASE_COMMIT" LINT_GO_CHANGED="$LINT_GO_CHANGED" bash -c "$GATES"
