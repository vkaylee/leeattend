# Phase 04: Add Compound Performance Indexes

## Context
- Parent: [plan.md](./plan.md)
- Depends on: None

## Overview
- **Priority**: P2
- **Status**: ✅ complete
- **Description**: Add compound indexes for common query patterns

## Key Insights
- Employee queries often filter by workspace_id + is_active
- Current indexes are single-column

## Requirements
- Add compound index `(workspace_id, is_active)` on employees
- Consider covering index if beneficial

## Related Code Files
- Create: `apiserver/migrations/20260124173000_add_compound_indexes.sql`

## Implementation Steps

1. Create migration file
2. Add `idx_employees_workspace_active` on `employees(workspace_id, is_active)`
3. Use `IF NOT EXISTS` for idempotency

## Todo List
- [x] Create migration file: 20260124173000_add_compound_indexes.sql
- [ ] Verify query plan improvement (post-deploy monitoring)

## Review Findings
- Index created correctly for (workspace_id, is_active) pattern
- Suggestion: Monitor pg_stat_user_indexes for actual usage post-deploy

## Success Criteria
- Index created
- Employee list queries use index

## Risk Assessment
- **Low**: Index creation is non-destructive
