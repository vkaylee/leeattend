-- Move early_arrival_strategy from workspaces to the default attendance_policy_versions.rules
UPDATE attendance_policy_versions
SET rules = jsonb_set(
    rules,
    '{early_arrival_strategy}',
    to_jsonb(w.early_arrival_strategy::text)
)
FROM attendance_policies ap
JOIN workspaces w ON ap.workspace_id = w.id
WHERE attendance_policy_versions.policy_id = ap.id
AND ap.is_default = true;

-- Drop the column from workspaces
ALTER TABLE workspaces DROP COLUMN early_arrival_strategy;
