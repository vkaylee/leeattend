-- ============================================================================
-- ADR-005 Phase 4: Cascade engine cutover migration
--
-- Upserts cascade_as_primary=true for every workspace that has a feature_flags
-- table (i.e., all workspaces), enabling the cascade engine as the primary
-- policy rules source for OT calculation.
--
-- The legacy resolver continues to run as a "reverse shadow" for 30 days
-- post-cutover (ADR-005 §3). The diff is logged via tracing::info! through
-- run_cascade_shadow() in attendance_summary_service.rs.
--
-- Idempotent: re-running this migration is safe (UPSERT semantics).
-- ============================================================================

INSERT INTO public.feature_flags (workspace_id, flag_key, enabled, description, metadata)
SELECT
    w.id,
    'cascade_as_primary',
    true,
    'ADR-005 Phase 4 cutover — cascade engine promoted to primary OT calculation source',
    '{"cutover_at": "2026-07-29T00:00:00Z", "cutover_phase": "4", "adr": "ADR-005"}'
FROM public.workspaces w
ON CONFLICT (workspace_id, flag_key)
DO UPDATE SET
    enabled = true,
    description = EXCLUDED.description,
    metadata = EXCLUDED.metadata,
    updated_at = NOW();

-- Also remove the per-employee canary flag if it exists, since cutover is
-- now workspace-wide. Keep the flag record for audit trail but disable it.
UPDATE public.feature_flags
SET
    enabled = false,
    description = description || ' [AUTO-DISABLED: cascaded by ADR-005 Phase 4 cutover at ' || to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS TZ') || ']',
    updated_at = NOW()
WHERE flag_key = 'cascade_primary_employee_ids'
  AND enabled = true;

COMMENT ON TABLE public.feature_flags IS
    'Workspace-scoped feature flags including ADR-005 cascade_as_primary (Phase 4 cutover applied at 2026-07-29).';
