---
title: "Complete Leave Management FK Migration"
description: "Migrate leave system from string-based leave_type to UUID foreign keys referencing leave_types table"
status: pending
priority: P1
effort: 8h
branch: main
tags: [database, migration, refactor, leave-management]
created: 2026-01-25
---

# Complete Leave Management FK Migration

## Overview

The leave management system has a half-completed migration from string-based `leave_type` columns to UUID foreign keys referencing the `leave_types` table. This plan completes that migration.

## Current State Analysis

### Database Schema
- `leave_types` table exists with UUID primary keys (completed)
- `leave_policy_definitions` uses `leave_type_id` FK (completed)
- `leave_requests` has BOTH `leave_type` (VARCHAR) AND `leave_type_id` (nullable UUID)
- `leave_accrual_logs` has BOTH `leave_type` (VARCHAR) AND `leave_type_id` (nullable UUID)
- `leave_balances` has only `leave_type` (VARCHAR), NO FK column yet

### Code State
- All backend code uses string-based queries (`leave_type` VARCHAR)
- `leave_type_id` FK columns are dead code (populated but never read)
- No validation that string values match `leave_types.code`
- Frontend uses `leave_type: string` everywhere

## Phases

| Phase | Name | Status | Effort |
|-------|------|--------|--------|
| 1 | [Database Migration](./phase-01-database-migration.md) | pending | 1h |
| 2 | [Backend Repository Layer](./phase-02-backend-repositories.md) | pending | 2h |
| 3 | [Backend Service Layer](./phase-03-backend-services.md) | pending | 1.5h |
| 4 | [Backend API Handlers](./phase-04-backend-handlers.md) | pending | 1h |
| 5 | [Frontend Updates](./phase-05-frontend-updates.md) | pending | 1.5h |
| 6 | [Testing & Validation](./phase-06-testing.md) | pending | 1h |

## Key Dependencies
- Phase 2 depends on Phase 1 (migration must run first)
- Phase 3 depends on Phase 2 (repositories must be updated)
- Phase 4 depends on Phase 3 (services must be updated)
- Phase 5 can run in parallel with Phases 3-4 (types preparation)
- Phase 6 requires all previous phases

## Risk Assessment
- **Data Loss**: Low - migration backfills from existing data
- **Orphan Records**: Medium - handled via fallback to default leave type
- **Breaking Changes**: Medium - API response shape changes slightly
- **Downtime**: None - migration is additive before destructive

## Success Criteria
- [ ] All leave-related queries use FK joins
- [ ] String columns dropped from database
- [ ] All tests pass
- [ ] No orphan records remain
