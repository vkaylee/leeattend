# Phase 02: Add Missing FK Indexes

## Context
- Parent: [plan.md](./plan.md)
- Depends on: Phase 01 (optional)

## Overview
- **Priority**: P2
- **Status**: ✅ complete
- **Description**: Add missing indexes on foreign key columns for query performance

## Key Insights
- FK columns without indexes cause slow JOINs and deletions
- Identified missing indexes from Schema Review

## Requirements
- Add index on `leave_requests.approved_by_user_id`
- Add index on `working_pattern_details.shift_id`

## Related Code Files
- Create: `apiserver/migrations/20260124171000_add_missing_fk_indexes.sql`

## Implementation Steps

1. Create migration file
2. Add `idx_leave_requests_approved_by` on `leave_requests(approved_by_user_id)`
3. Add `idx_working_pattern_details_shift` on `working_pattern_details(shift_id)`
4. Use `IF NOT EXISTS` for idempotency

## Todo List
- [x] Create migration file: 20260124171000_add_missing_fk_indexes.sql
- [x] Test migration

## Review Findings
- Indexes created correctly with IF NOT EXISTS
- Suggestion: Consider CONCURRENTLY for production (requires manual execution)

## Success Criteria
- Indexes created
- Query plans show index usage for relevant queries

## Risk Assessment
- **Low**: Index creation is non-destructive
- Index creation on large tables may lock briefly (use CONCURRENTLY if needed)
