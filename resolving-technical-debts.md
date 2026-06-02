# Rule Compliance: Resolving Technical Debts

## 1. Compliance Rule IDs
The following rule IDs from our core rulebooks are verified and adhered to in this task:
- **MULTI_TENANCY_ISOLATION** (From `database-rules.md` Section 1): Every query targeting tenant data must explicitly filter by `workspace_id`.
- **TRANSACTIONAL_BOUNDARIES** (From `database-rules.md` Section 3): Business operations mutating multiple records/tables must be wrapped in a database transaction.
- **PTY_SAFETY** (From `GEMINI.md` Layer 0 Section 5): All command executions must redirect stdout/stderr to a log file.
- **HERMETIC_WRAPPERS** (From `GEMINI.md` Layer 0 Section 6): Do not call `cargo`, `npm`, `diesel`, or `python` directly; use scripts from the `scripts/` directory.
- **DEPENDENCY_INJECTION** (From `coding-standards.md` Section 9): Decouple service layers from infrastructure by using traits/interfaces.

## 2. Compliance Details
- **PTY Safety & Redirection**: All commands will be run with output redirection like `> logs/run_{slug}.log 2>&1 </dev/null`.
- **Tenant Isolation**: Any queries to the newly added `outbox_jobs` table will strictly filter by `workspace_id`.
- **No Direct CLI Calls**: Wrap command executions through `./scripts/_cargo.sh`, `./scripts/_npm.sh`, and `./scripts/_diesel.sh`.
