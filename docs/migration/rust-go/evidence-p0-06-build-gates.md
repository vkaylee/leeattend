# Task P0-06 — Build Gate Evidence

Date: 2026-10-19
Commit (pre-task): `456bdd4c00d6c050cbcfe8c43777234c4650aa8c`

## 1. Gate definitions (tracked)

| Gate | Command | Container |
|---|---|---|
| gofmt | `test -z "$(gofmt -l .)"` | `localhost/leeattend-test_go` |
| go vet | `go vet ./...` | same |
| staticcheck | `staticcheck ./...` | same |
| govulncheck | `govulncheck ./...` | same |
| module integrity | `go mod verify` | same |
| sqlc diff | `sqlc diff` (only when `sqlc.yaml` exists) | same |
| race tests | `go test -race -cover ./...` | same |

Entrypoints:
- `scripts/_go-gates.sh` — tracked, canonical gate runner (containerised, pinned).
- `./leedevkit test api-go [--lint-only|--unit-only]` — routes to the same runner.
- `.github/workflows/go-gates.yml` — CI job, includes generated-code diff job.

Pinned toolchain (`.compose/go/Dockerfile`):
`GO_VERSION=1.24.13`, `staticcheck v0.6.1`, `govulncheck v1.1.4`, `sqlc v1.29.0`, `oapi-codegen v2.4.1`.

## 2. Clean-state run (must pass)

`logs/run_gates_clean_lint2.log`:
```
== gofmt ==
== go vet ==
== staticcheck ==
== govulncheck ==
== go mod verify ==
ALL GATES PASSED
exit=0
```

`logs/run_gates_clean_unit.log`:
```
== go test -race ==
ok  	github.com/vkaylee/leeattend/apiserver	1.009s	coverage: [no statements]
ALL GATES PASSED
exit=0
```

## 3. Seeded-violation run (must fail per gate)

Driver: `scratch/test_seeded_gates.sh` → `logs/run_seeded_gates.log`

| Seeded violation | Gate that fired | Evidence log |
|---|---|---|
| Unformatted Go file | `GATE FAIL: gofmt listed unformatted files` | `logs/gate_fail_gofmt.log` |
| `fmt.Printf("%d", "string")` | `GATE FAIL: go vet` | `logs/gate_fail_govet.log` |
| `strings.Title` (SA1019) | `GATE FAIL: staticcheck` | `logs/gate_fail_staticcheck.log` |
| Unsynchronised `counter++` in goroutines | `WARNING: DATA RACE` + `GATE FAIL: go test -race` | `logs/gate_fail_race.log` |

```
=== 1. Clean Baseline ===
Clean baseline passed.
=== 2. Seeded gofmt Violation ===
PASS: gofmt gate successfully caught unformatted file.
=== 3. Seeded go vet Violation ===
PASS: go vet gate successfully caught format mismatch.
=== 4. Seeded staticcheck Violation ===
PASS: staticcheck gate successfully caught SA1019.
=== 5. Seeded go test -race / Test Failure Violation ===
PASS: go test -race gate successfully caught data race.
=== 6. Final Clean Verification ===
ALL SEEDED BUILD-GATE TESTS COMPLETED SUCCESSFULLY.
exit=0
```

## 4. False-green analysis

Each gate was proven independently: seeded violations were committed to the tree, the gate chain was
invoked, the gate failed, and the seed file was removed. A passing clean run after removal proves the
failure was caused by the seeded violation, not by an unrelated broken environment.

`govulncheck` and `go mod verify` are dependency-level checks; they cannot be triggered by seeding a
source file, so their clean-state pass is the only applicable evidence at this stage.
