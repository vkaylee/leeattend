# Implementation Plan: Improve Test Orchestrator

## Approach
Improve the existing `run-tests.sh` and supporting scripts to allow selective testing of components (`--api`, `--web`, `--all`) with automatic environment management.

### Key Innovations:
1.  **Filtering in `test-utils.sh`**: Add a tagging/filtering mechanism to `run_parallel_ordered` so it only executes relevant jobs.
2.  **Environment-Aware Orchestration**: `run-tests.sh` will decide whether to call `setup-test-env.sh` based on the requested component and test phase (Integration/E2E).
3.  **Unified Help & Interface**: A polished CLI with ANSI colors and clear usage instructions.

## Steps

1.  **Enhance `scripts/test-utils.sh`** (10 min)
    - Modify `run_parallel_ordered` to support an optional filter variable.
    - Add a logic to skip jobs that don't match the active "filter tag".

2.  **Update Phase Scripts** (15 min)
    - Modify `scripts/test-lint.sh`, `scripts/test-unit.sh`, and `scripts/test-integration.sh`.
    - Allow these scripts to accept a filter argument (e.g., `api` or `web`) and pass it to the runner.

3.  **Refactor `run-tests.sh`** (20 min)
    - Implement `getopts` or `case` based argument parsing.
    - Add `show_help` function.
    - Orchestrate the phases:
        - `Lint` -> `Unit` -> `Integration/E2E`.
        - Wrap Integration tests with `./setup-test-env.sh`.

4.  **Verification** (15 min)
    - Run `./run-tests.sh --help`.
    - Run `./run-tests.sh --api` (Verify no web tests run).
    - Run `./run-tests.sh --web` (Verify only web tests run).
    - Run `./run-tests.sh --all` (Verify full suite).

## Timeline
| Phase | Duration |
|-------|----------|
| Utilities | 10 min |
| Phase Scripts | 15 min |
| Orchestrator | 20 min |
| Verification | 15 min |
| **Total** | **1 hour** |

## Rollback Plan
- Revert `run-tests.sh` and `scripts/` to their previous git state.
- `git checkout run-tests.sh scripts/`

## Security Checklist
- [x] Input validation (Strict argument parsing)
- [x] Error handling (Trap EXIT for environment cleanup)
- [ ] Auth checks (N/A - local tests)
- [ ] Rate limiting (N/A)
