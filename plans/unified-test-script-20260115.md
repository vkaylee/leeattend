# Implementation Plan: Unified Test Script with Parallelism

### ## Approach
Create a top-level shell script `run-tests.sh` that orchestrates tests in three phases:
1. **Unit Phase (Parallel)**: Executes lightweight unit tests for all modules simultaneously to maximize CPU utilization.
2. **Environment Phase**: Sets up necessary infrastructure (Postgres via Docker Compose) for integration testing.
3. **Integration Phase (Parallel)**: Executes heavy integration/E2E tests (Rust Integration + Playwright).

### ## Steps

1. **Pre-requisites Check** (5 min)
   - Verify `docker-compose`, `cargo`, `npm`, and `python/pytest` are available.

2. **Core Implementation** (30 min)
   - Create `run-tests.sh` at the project root.
   - Implement log redirection to temporary files to keep the console clean during parallel execution.
   - Implement a summary reporter that shows which tests passed/failed at the end.

3. **Database Orchestration** (10 min)
   - Use `docker-compose.test.yml` to spin up Postgres.
   - Implement a robust wait-for-db loop using `pg_isready`.

4. **Refactor apiserver tests** (optional but recommended)
   - Ensure integration tests in Rust can run against the dockerized Postgres (port 5433).

### ## Timeline
| Phase | Duration |
|-------|----------|
| Pre-requisites | 5 min |
| run-tests.sh coding | 30 min |
| DB Orchestration | 10 min |
| Testing & Debugging | 15 min |
| **Total** | **1 hour** |

### ## run-tests.sh structure (Logic)
```bash
# 1. Parallel Unit Tests
(cd apiserver && cargo test --lib) &
(cd agent-main && cargo test --lib) &
(cd agent-zk-worker && venv/bin/pytest) &
(cd webdashboard && npm run test) &
wait

# 2. Setup Environment
docker-compose -f docker-compose.test.yml up -d
# Wait loop...

# 3. Parallel Integration Tests
(cd apiserver && cargo test --test '*') &
(cd webdashboard && npx playwright test) &
wait
```

### ## Security Checklist
- [ ] No hardcoded secrets in the script.
- [ ] Ensure cleanup of Docker resources on exit.
