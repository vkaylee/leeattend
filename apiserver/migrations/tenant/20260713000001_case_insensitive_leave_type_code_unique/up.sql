-- ============================================================================
-- Make the leave_type `code` reservation case-insensitive AND reserved forever
-- (including soft-deleted rows), so reusing a retired code is always a conflict
-- that can only be resolved by RESTORE, never by re-inserting a new row.
-- ============================================================================

-- 1. Remove the contradictory partial index: case-insensitive but only enforced
--    on ACTIVE rows, so it does not protect the reserved-code invariant across
--    soft-deletes and is redundant once the permanent index below exists.
DROP INDEX IF EXISTS public.uq_leave_types_code;

-- 2. Remove the exact-case table constraint that currently spans all rows
--    (including soft-deleted). Note: ADD CONSTRAINT ... UNIQUE only accepts
--    column names, not expressions like lower(code), so the case-insensitive
--    version below is created as a UNIQUE INDEX instead.
ALTER TABLE public.leave_types
    DROP CONSTRAINT IF EXISTS leave_types_workspace_id_code_key;

-- 3. Re-create as a case-insensitive UNIQUE INDEX with NO WHERE clause, so the
--    code is reserved forever (active or soft-deleted). We keep the same name so
--    the repository's UniqueViolation matcher keeps working.
--    NOTE: if data already violates this (an active row and a soft-deleted row
--    differing only by case), Postgres raises immediately and the migration
--    aborts - fail-fast, which is the safe outcome.
CREATE UNIQUE INDEX leave_types_workspace_id_code_key
    ON public.leave_types (workspace_id, lower(code));
