# Phase 03: Standardize PK Types

## Context
- Parent: [plan.md](./plan.md)
- Depends on: None

## Overview
- **Priority**: P3
- **Status**: ✅ complete
- **Description**: Convert SERIAL to UUID in payslip_incentives for consistency

## Key Insights
- All other tables use UUID primary keys
- `payslip_incentives.id` uses SERIAL (integer)
- Inconsistency complicates ORM mappings and client code

## Requirements
- Convert `payslip_incentives.id` from SERIAL to UUID
- Preserve existing data
- Maintain FK relationship with payslips

## Architecture

```sql
-- Before
payslip_incentives.id SERIAL PRIMARY KEY

-- After
payslip_incentives.id UUID PRIMARY KEY DEFAULT gen_random_uuid()
```

## Related Code Files
- Create: `apiserver/migrations/20260124172000_standardize_payslip_incentives_pk.sql`
- Check: Rust model for payslip_incentives (update if exists)

## Implementation Steps

1. Add new UUID column `new_id`
2. Populate with gen_random_uuid()
3. Drop old PK constraint
4. Drop old `id` column
5. Rename `new_id` to `id`
6. Add PK constraint on new `id`

## Todo List
- [x] Create migration file: 20260124172000_standardize_payslip_incentives_pk.sql
- [x] Update Rust model if needed: Not required (no direct id references in code)
- [x] Test migration

## Review Findings
- Migration correctly transitions SERIAL to UUID via temporary column
- No external FK dependencies verified in codebase
- Irreversible migration (documented in down migration comments)

## Success Criteria
- payslip_incentives.id is UUID type
- Existing records preserved with new UUIDs
- No FK breakage

## Risk Assessment
- **Medium**: PK type change requires careful migration
- **Mitigation**: No external FKs reference this table (verified)

## Security Considerations
- None specific
