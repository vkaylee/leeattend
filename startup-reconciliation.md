# Rule Compliance Check: Startup Reconciliation & Manage Clean Dev

## Task Details
- **Task**: Startup reconciliation of stale tenants in Supavisor without database deletion; add `clean dev` environment management subcommand.
- **Task Slug**: `startup-reconciliation`

## Rule Verification Table

| Rule ID | Rule Source | Description | Planned Compliance |
| :--- | :--- | :--- | :--- |
| **CODING-STD-03** | `coding-standards.md` | Structured Errors: Use domain-specific error types (`AppError`). Never throw generic `Error`. | We will return `AppResult` or handle errors with `AppError` inside the `apiserver` code. |
| **CODING-STD-06** | `coding-standards.md` | Enterprise-Grade Premium Standard: Ensure robust startup, handling case where `_supavisor` schema doesn't exist yet, logging appropriately. | We will catch errors when querying `_supavisor.tenants` (e.g., if database is not migrated yet) and fallback gracefully. |
| **EXEC-PTY-07** | `execution-pty-safety.md` | Avoiding PTY Hangs on Scripts: Redirect command output `> run.log 2>&1 </dev/null` for tests/builds. | Every command executed for testing or building will be properly redirected. |
| **EXEC-WRAP-01** | `execution-wrappers.md` | Hermetic Execution: Always run tests via `./test.sh` and cargo commands via `./scripts/_cargo.sh`. | We will invoke `./test.sh` to run the rust and orchestrator python tests. |
