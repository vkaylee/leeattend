# GEMINI.md

This file provides guidance to Gemini AI when working with code in this repository.

## Role & Responsibilities

Your role is to analyze user requirements, assist with code implementation, testing, and ensure cohesive delivery of features that meet specifications and architectural standards.

## WSL Performance Protocol (CRITICAL)

> **This project runs on WSL with severe Disk I/O issues (1GB/s read causing system freeze).**

### Mandatory Rules:

1. **NEVER** run `cargo test`, `cargo nextest`, `pytest`, or any test command directly
2. **ALWAYS** use `./run-tests.sh` for ALL testing requests (unit, integration, e2e)
3. The script automatically moves migrations to RAM (`/dev/shm`) to prevent system freeze
4. For DB-related test code, prefer `sqlite::memory:` or tmpfs for Postgres

### Test Commands:
```bash
./run-tests.sh --api          # API tests (Rust + Python)
./run-tests.sh --web          # Web tests (Vite + Playwright)
./run-tests.sh --all          # All tests
./run-tests.sh --api auth     # API tests filtered by "auth"
./run-tests.sh --no-ram --api # Disable RAM optimization (not recommended on WSL)
```

### Why This Matters:
- Direct test execution triggers massive Disk I/O on WSL's VirtioFS
- This causes system freeze, requiring hard restart
- The `run-tests.sh` script handles RAM disk setup/cleanup transparently

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
1. Always use `./run-tests.sh` with appropriate flags
2. Never run cargo/pytest/vitest directly
3. Wait for RAM disk setup message before expecting test output
4. Script auto-cleans up on completion or interrupt (Ctrl+C)
