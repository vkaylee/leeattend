# Scout Report: Increase Test Coverage

## Exploration Scope
- Target: Current test suite, coverage generation, and gaps.
- Boundaries: `apiserver`, `agent-main`, and test scripts (`run-tests.sh`, `scripts/test-coverage.sh`).

## Patterns Discovered
### Pattern: Test Coverage Script
- **Location**: `scripts/test-coverage.sh`
- **Usage**: Invoked via `./run-tests.sh --api --coverage` to generate LCOV and HTML coverage reports.
- **Must Follow**: Yes

### Pattern: SQLx Compilation Failure
- **Location**: `apiserver`
- **Usage**: `sqlx::query!` macros require a live database matching the schema or an offline `.sqlx` data directory to compile.
- **Problem**: `test-coverage.sh` fails during `cargo llvm-cov` because it does not prepare the `.sqlx` offline data, nor does it provide a migrated database to the `apiserver_test` container. There is no `.sqlx` directory in `apiserver`.
- **Must Follow**: We MUST fix test compilation by either generating `.sqlx` data before running coverage, or running migrations on the test DB before `cargo llvm-cov`.

## Integration Points
| Point | File | Function | New Code Location |
| ------ | ------ | -------- | ----------------- |
| Coverage Script | `scripts/test-coverage.sh` | `leeattend_run_coverage` | Modify to ensure it handles SQLx properly (e.g. `SQLX_OFFLINE=true` + ensuring `.sqlx` exists) |
| API Tests | `apiserver/tests/` | Various | Add missing test cases here |
| Agent Tests | `agent-main/tests/` | Various | Add missing test cases here |

## Conventions
- Naming: `*_test.rs` under `tests/` directories.
- Run tests via `./run-tests.sh` to ensure PTY safety and parallel test isolation.

## Warnings
- ⚠️ The `test-coverage.sh` script is currently broken for `apiserver` due to SQLx compile-time query checks. This must be fixed first before we can measure baseline coverage or verify the 50% target.
