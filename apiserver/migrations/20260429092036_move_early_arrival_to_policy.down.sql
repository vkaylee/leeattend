-- Re-add the column to workspaces
ALTER TABLE workspaces ADD COLUMN early_arrival_strategy early_arrival_strategy_type NOT NULL DEFAULT 'block';

-- Move data back from the default attendance_policy_versions.rules
UPDATE workspaces w
SET early_arrival_strategy = COALESCE(
    (
        SELECT (apv.rules->>'early_arrival_strategy')::early_arrival_strategy_type
        FROM attendance_policy_versions apv
        JOIN attendance_policies ap ON apv.policy_id = ap.id
        WHERE ap.workspace_id = w.id AND ap.is_default = true
        ORDER BY apv.valid_from DESC
        LIMIT 1
    ),
    'block'::early_arrival_strategy_type
);

-- Clean up the JSON field from rules
UPDATE attendance_policy_versions
SET rules = rules - 'early_arrival_strategy';
