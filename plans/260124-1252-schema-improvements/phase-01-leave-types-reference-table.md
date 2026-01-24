# Phase 01: Create leave_types Reference Table

## Context
- Parent: [plan.md](./plan.md)
- Docs: [code-standards.md](../../docs/code-standards.md)

## Overview
- **Priority**: P2
- **Status**: ✅ complete
- **Description**: Replace VARCHAR leave_type strings with FK reference to standardized table

## Key Insights
- Current: `leave_requests.leave_type` and `leave_accrual_logs.leave_type` use VARCHAR(50)
- Problem: No validation, inconsistent naming (vacation vs annual_leave)
- Solution: Create `leave_types` workspace-scoped reference table

## Requirements
- Create `leave_types` table with workspace_id (multi-tenant)
- Seed default types: vacation, sick, personal, unpaid, maternity, paternity
- Add FK constraints to existing tables (without breaking existing data)
- Preserve backward compatibility

## Architecture

```sql
leave_types (new)
├── id UUID PK
├── workspace_id UUID FK
├── code VARCHAR(50) UNIQUE per workspace
├── name TEXT
├── is_paid BOOLEAN
└── created_at TIMESTAMPTZ

leave_requests.leave_type_id UUID FK → leave_types (new column)
leave_accrual_logs.leave_type_id UUID FK → leave_types (new column)
```

## Related Code Files
- Create: `apiserver/migrations/20260124170000_create_leave_types.sql`
- Modify: Rust models if any (check after migration)

## Implementation Steps

1. Create `leave_types` table with workspace scope
2. Seed default leave types for all existing workspaces
3. Add `leave_type_id` column to `leave_requests` (nullable initially)
4. Add `leave_type_id` column to `leave_accrual_logs` (nullable initially)
5. Migrate existing VARCHAR values to FK references
6. Add NOT NULL constraint after migration
7. Drop legacy VARCHAR columns (optional, or deprecate)

## Todo List
- [x] Create migration file: 20260124170000_create_leave_types.sql
- [x] Test migration on dev database
- [x] Verify existing data preserved

## Review Findings
- Migration creates leave_types table with proper workspace scoping
- Backward compatible (keeps legacy VARCHAR columns)
- Suggestion: Add validation for unmigrated records (see code review report)

## Success Criteria
- leave_types table created
- Default types seeded
- FK references working
- No data loss

## Risk Assessment
- **Medium**: Data migration complexity
- **Mitigation**: Keep legacy columns initially, migrate in phases

## Security Considerations
- workspace_id enforces multi-tenant isolation
- ON DELETE CASCADE for workspace cleanup

## Next Steps
- Phase 02: Add missing indexes
