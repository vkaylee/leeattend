-- Smoke test: verify cascade_as_primary flag was upserted for all workspaces
SELECT
    w.name AS workspace_name,
    ff.flag_key,
    ff.enabled,
    ff.description,
    ff.metadata,
    ff.updated_at
FROM public.feature_flags ff
JOIN public.workspaces w ON w.id = ff.workspace_id
WHERE ff.flag_key = 'cascade_as_primary'
ORDER BY w.name;

-- Verify per-employee canary flag is disabled post-cutover
SELECT
    w.name AS workspace_name,
    ff.flag_key,
    ff.enabled,
    ff.description
FROM public.feature_flags ff
JOIN public.workspaces w ON w.id = ff.workspace_id
WHERE ff.flag_key = 'cascade_primary_employee_ids'
  AND ff.enabled = true;
-- Expect: 0 rows (all canary flags disabled)
