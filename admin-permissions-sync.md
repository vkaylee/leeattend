# Admin permission synchronization

## Scope
Synchronize system role `admin` with every canonical permission without deleting legacy permission rows.

## Rule IDs
- `access-control.md §2`: permissions use granular `resource:action` grants; default roles receive relevant grants.
- `database-rules.md §4`: migration paths need rollback; active project uses a fresh-only baseline and does not upgrade old databases.
- `development-workflow.md §2, §6`: cover persistence, idempotency, compatibility, and authorization regression risk.
- `testing-standards.md §1-2`: run focused verification and the applicable full LeeDevKit target.

## Decision
Do not add a standalone migration version. `apiserver/migrations/run-dev.sh` intentionally accepts only the single fresh baseline and fails on stale databases. `EnsureDefaultRolesAndPermissions` already runs inside bootstrap transactions and uses `ON CONFLICT DO NOTHING`, so extending its seed is the supported idempotent reconciliation path.

## Test impact matrix
- Success: fresh/bootstrap admin receives every canonical permission.
- Regression: existing legacy admin permission rows remain untouched.
- Idempotency: repeated bootstrap does not duplicate primary-key grants.
- Failure: existing catalog seed execution errors continue to abort bootstrap.
- Security: only `admin` gains missing canonical grants; no wildcard or cross-organization permission is introduced.
- Compatibility: additive rows are tolerated by current and older readers.
