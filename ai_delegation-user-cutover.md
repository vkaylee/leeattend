# Delegation User Cutover

- Rule IDs: `database-rules §1, §4`; `access-control §1, §2`; `data-governance §1, §2`; `development-workflow §2–§7`; `api-governance §1, §2, §6`.
- Contract: clean cutover. `behalf_assignments.assignee_user_id` identifies any current `workspace_members` row, never an employee. Existing assignment rows are deleted in migration. Target employee/department scope remains unchanged.
- Runtime: validate assignee membership on create; resolve active delegation per request; list uses user identity fields, not an employee join.
- Tests: creation accepts member without employee; rejects non-member; dynamic target resolution remains user-based; frontend submits selected workspace user.
