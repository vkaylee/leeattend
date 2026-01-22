# Implementation Plan: Fix System Hang During Test Compilation

The system hangs during integration tests, specifically at the `Compiling apiserver` step. This is caused by `cargo` exhausting system RAM/CPU while multiple Docker containers and parallel test workers are also active.

### ## Root Cause Analysis
1. **Symptom:** System becomes unresponsive or crashes when `cargo test` starts compiling `apiserver`.
2. **Reproduction:** Run `./run-tests.sh` on a machine with limited RAM (8-16GB) or high core count (which triggers too many parallel rustc jobs).
3. **Affected components:** `apiserver`, `setup-test-env.sh`, and `scripts/test-integration.sh`.

---

### ## Approach
- **Limit Parallelism:** Use `CARGO_BUILD_JOBS` to restrict the number of concurrent compiler processes.
- **Pre-compilation:** Compile the backend binaries *before* spinning up the heavy Docker environment to avoid resource contention.

---

### ## Steps

1. **Environment Configuration** (5 min)
   - Set `CARGO_BUILD_JOBS` to a conservative value (e.g., `2`).
   ```bash
   export CARGO_BUILD_JOBS=2
   ```

2. **Modify Integration Script** (10 min)
   - Files to modify: `scripts/test-integration.sh`
   - Update the pre-compilation step to use the job limit and handle errors gracefully.
   ```bash
   # Inside scripts/test-integration.sh
   echo "  Pre-compiling backend (limited jobs)..."
   (cd apiserver && cargo test --no-run -j ${CARGO_BUILD_JOBS:-2})
   ```

3. **Optimization in `setup-test-env.sh`** (10 min)
   - Ensure the DB setup doesn't start until the heavy compilation is done, or vice versa.
   - Files to modify: `setup-test-env.sh`

4. **Verification** (30 min)
   - Run the full suite: `./run-tests.sh`
   - Monitor RAM usage during the compilation phase.

---

### ## Timeline
| Phase | Duration |
|-------|----------|
| Analysis & Config | 5 min |
| Script Updates | 20 min |
| Testing & Verification | 35 min |
| **Total** | **1 hour** |

---

### ## Rollback Plan
- Revert changes to `scripts/test-integration.sh`.
- Unset `CARGO_BUILD_JOBS`.

---

### ## Resource Safety Checklist
- [x] Limit `CARGO_BUILD_JOBS`
- [x] Pre-build with `--no-run`
- [x] Monitor swap usage
- [x] Ensure Docker containers are healthy before tests start
