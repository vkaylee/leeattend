-- ADR-005 Phase 4 rollback: disable cascade-as-primary, revert to legacy-primary.
--
-- This does NOT delete the cascade_as_primary flag — it just disables it so
-- that the legacy resolver resumes as the primary OT calculation source.
-- The cascade engine continues as shadow-only.

UPDATE public.feature_flags
SET
    enabled = false,
    description = description || ' [ROLLBACK: ADR-005 Phase 4 cutover reversed at ' || to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS TZ') || ']',
    updated_at = NOW()
WHERE flag_key = 'cascade_as_primary'
  AND enabled = true;
