# GEMINI.md

This file provides guidance to Gemini AI when working with code in this repository.

## Role & Responsibilities

Your role is to analyze user requirements, assist with code implementation, testing, and ensure cohesive delivery of features that meet specifications and architectural standards.

## WSL Performance Protocol (CRITICAL)

> **This project runs on WSL with severe Disk I/O issues (1GB/s read causing system freeze).**

### Mandatory Rules:

1. **NEVER** run `cargo test`, `cargo nextest`, `cargo check`, `cargo clippy`, `pytest`, or any build/test command directly on the HOST
2. **ALL tests and compilation run INSIDE Docker containers** via `docker-compose.test.yml`
3. **ALWAYS** use `./run-tests.sh` for full test suite runs (unit, integration, e2e)
4. For DB-related test code, prefer `sqlite::memory:` or tmpfs for Postgres

### Docker Test Architecture:
- Tests execute inside Docker containers, NOT on the host machine
- The host does NOT have Rust toolchain, clang, or build dependencies installed
- `run-tests.sh` orchestrates `docker compose -f docker-compose.test.yml` internally
- Container names: `leeattend-agent_test-1`, `leeattend-apiserver_test-1`, `leeattend-test_db-1`

### Test Commands:
```bash
# Full test suite (orchestrated via Docker)
./run-tests.sh --api          # API tests (Rust + Python)
./run-tests.sh --web          # Web tests (Vite + Playwright)
./run-tests.sh --all          # All tests
./run-tests.sh --api auth     # API tests filtered by "auth"

# Running specific tests directly inside containers (for debugging/verification)
docker compose -f docker-compose.test.yml exec agent_test cargo nextest run <test_name>
docker compose -f docker-compose.test.yml exec apiserver_test cargo nextest run <test_name>
docker compose -f docker-compose.test.yml exec agent_test cargo clippy --all-targets
docker compose -f docker-compose.test.yml exec apiserver_test cargo check

# Check container logs for test output
docker compose -f docker-compose.test.yml logs agent_test --tail=100
docker compose -f docker-compose.test.yml logs apiserver_test --tail=100
```

### Why This Matters:
- Host has NO Rust toolchain — `cargo` commands only work inside containers
- Direct test execution on host triggers massive Disk I/O on WSL's VirtioFS
- This causes system freeze, requiring hard restart
- The `run-tests.sh` script handles Docker orchestration transparently

## Project Structure

- `apiserver/` - Rust API server with SQLx migrations
- `agent-main/` - Rust agent with SQLx migrations
- `webdashboard/` - React frontend with Vite

## Development Guidelines

- Follow YAGNI, KISS, DRY principles
- Keep code files under 200 lines
- Use kebab-case for file naming
- Read `./docs/` for architecture and standards
- Use `./README.md` for project overview

## Testing Protocol

When user requests testing ("Test đi", "Fix lỗi này và verify", etc.):
1. Always use `./run-tests.sh` for full suites, or `docker compose exec` for specific tests
2. Never run cargo/pytest/vitest directly on the host — they WILL fail (no toolchain)
3. Script auto-cleans up on completion or interrupt (Ctrl+C)
4. To check test results, inspect container logs: `docker compose -f docker-compose.test.yml logs <service> --tail=100`
