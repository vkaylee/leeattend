# Dual-Layer Protection & Staging Area Implementation

## Overview
Implement staging area for unknown users and fingerprint-based deduplication to prevent spam from agent retries.

## Problem
- Unknown users cause `Err(msg)` returns → Agent retries → Spam
- No deduplication on `attendance_log_errors` table

## Solution
- **Staging Table**: Valid logs with unknown users go to `attendance_staging_logs` (returns 200 OK)
- **Fingerprint Dedup**: Both staging and errors tables use fingerprint-based deduplication

## Phases

| Phase | File | Status |
|-------|------|--------|
| 1 | [phase-01-database-migration.md](./phase-01-database-migration.md) | Pending |
| 2 | [phase-02-backend-logic.md](./phase-02-backend-logic.md) | Pending |
| 3 | [phase-03-repository-updates.md](./phase-03-repository-updates.md) | Pending |

## Key Dependencies
- Phase 2 depends on Phase 1 (tables must exist)
- Phase 3 can be done in parallel with Phase 2 logic design

## Success Criteria
- [ ] Unknown users → staging table, return 200 OK
- [ ] Validation errors → error table with fingerprint, return 200 OK
- [ ] Duplicate logs ignored (no spam)
- [ ] All existing tests pass
