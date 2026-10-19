#!/bin/bash
# Seeded-violation verification for the Go build gates (ledger task P0-06).
#
# Proves each gate is real: seeds one violation at a time, asserts the gate
# fails, removes the seed, and asserts a clean run passes. A gate that stays
# green on a seeded violation is a false green and fails this script.
#
# Usage: scripts/verify-go-gates.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$PROJECT_ROOT/logs"
SOURCE_DIR="$PROJECT_ROOT/apiserver"
mkdir -p "$LOG_DIR"
cd "$PROJECT_ROOT"

SEEDS=()

cleanup() {
    for seed in "${SEEDS[@]:-}"; do
        [ -n "$seed" ] && rm -f "$SOURCE_DIR/$seed"
    done
    return 0
}
trap cleanup EXIT

expect_fail() {
    local gate="$1" seed="$2" log="$3"
    SEEDS+=("$seed")
    if ./scripts/_go-gates.sh "$flag" >"$LOG_DIR/$log" 2>&1; then
        echo "FALSE GREEN: $gate stayed green on seeded violation $seed" >&2
        exit 1
    fi
    echo "PASS: $gate caught seeded violation ($seed)"
    rm -f "$SOURCE_DIR/$seed"
    SEEDS=("${SEEDS[@]:0:${#SEEDS[@]}-1}")
}

echo "== clean baseline =="
flag="--lint-only"
./scripts/_go-gates.sh --lint-only >"$LOG_DIR/gate_baseline_lint.log" 2>&1
flag="--unit-only"
./scripts/_go-gates.sh --unit-only >"$LOG_DIR/gate_baseline_unit.log" 2>&1
echo "PASS: clean baseline"

cat >"$SOURCE_DIR/bad_fmt.go" <<'EOF'
package main

func badFmt( ) {
var x = 1
_ = x
}
EOF
flag="--lint-only"
expect_fail "gofmt" "bad_fmt.go" "gate_fail_gofmt.log"

cat >"$SOURCE_DIR/bad_vet.go" <<'EOF'
package main

import "fmt"

func badVet() {
	fmt.Printf("%d", "not an int")
}
EOF
flag="--lint-only"
expect_fail "go vet" "bad_vet.go" "gate_fail_govet.log"

cat >"$SOURCE_DIR/bad_staticcheck.go" <<'EOF'
package main

import "strings"

func badStaticcheck(s string) string {
	return strings.Title(s)
}
EOF
flag="--lint-only"
expect_fail "staticcheck" "bad_staticcheck.go" "gate_fail_staticcheck.log"

cat >"$SOURCE_DIR/bad_race_test.go" <<'EOF'
package main

import (
	"sync"
	"testing"
)

func TestDataRace(t *testing.T) {
	var counter int
	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			counter++
		}()
	}
	wg.Wait()
}
EOF
flag="--unit-only"
expect_fail "go test -race" "bad_race_test.go" "gate_fail_race.log"

echo "== clean re-verification =="
./scripts/_go-gates.sh --lint-only >"$LOG_DIR/gate_final_lint.log" 2>&1
./scripts/_go-gates.sh --unit-only >"$LOG_DIR/gate_final_unit.log" 2>&1
echo "ALL SEEDED GATE CHECKS PASSED"
