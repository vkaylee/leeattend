# Implementation Plan: Standardize Schema Enums and Constraints

## Approach
- **Why this solution:** Using Postgres native `ENUM` types instead of generic `VARCHAR` columns ensures data integrity at the lowest level, improves query performance through better indexing, and serves as self-documentation for the schema.
- **Alternatives considered:** Keeping `VARCHAR` with `CHECK` constraints. While more flexible for future changes, it lacks the type-safety and performance benefits of true ENUMs in Postgres.

## Steps
1. **Create Database Migration** (30 min)
   - Create `apiserver/migrations/20260117170000_standardize_enums.sql`.
   - Define types:
     - `member_role_type` ('owner', 'admin', 'member')
     - `leave_status_type` ('pending', 'approved', 'rejected')
     - `adjustment_status_type` ('pending', 'approved', 'rejected')
     - `log_source_type` ('device', 'import', 'manual')
   - Update columns using `USING column_name::new_type` syntax.

2. **Update Backend Models** (30 min)
   - Modify `apiserver/src/models.rs`.
   - Implement `sqlx::Type` for new Rust enums to map directly to DB types.
   - Update repository queries if needed to handle strict type casting.

3. **Verification & Testing** (30 min)
   - Run `./run-tests.sh` to ensure existing logic remains compatible.
   - Add a new test case in `apiserver/tests/db_trap_test.rs` to verify that invalid strings are rejected by the database.

4. **Frontend Alignment** (15 min)
   - Ensure Typescript interfaces in `webdashboard/src/lib/api/types.ts` match the strict enum values.

## Timeline
| Phase | Duration |
|-------|----------|
| DB Migration | 30 min |
| Backend Refactor | 30 min |
| Verification | 30 min |
| Frontend Update | 15 min |
| **Total** | **1h 45m** |

## Rollback Plan
- Create a corresponding `.down.sql` migration that alters columns back to `VARCHAR(50)` and drops the custom ENUM types.

## Security Checklist
- [x] Data integrity: Validating all existing data before type conversion.
- [x] SQL Safety: Using SQLx for type-safe query binding.
- [x] Access Control: Ensuring `member_role_type` strictly matches the RBAC system.
