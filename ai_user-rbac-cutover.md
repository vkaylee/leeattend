# User RBAC & Assignment Cutover

- Rule IDs: `database-rules §1, §4`; `access-control §1, §2`; `data-governance §1, §2`; `coding-standards`.
- Contract: clean cutover & bridge.
  1. Core IAM & RBAC reside exclusively in System DB (`workspace_members`, `workspace_roles`, `workspace_role_permissions`, `member_extra_roles`, `workspace_member_extra_permissions`).
  2. Approval Workflow tasks & pending task resolution evaluate approvers via `SystemDB` user membership and roles, eliminating broken cross-DB queries on Tenant DB.
  3. Tenant DB dead enum `member_role_type` is removed.
  4. Business policy tables (`leave_policy_assignments`, `overtime_policy_assignments`, `payroll_incentive_assignments`, `attendance_policy_assignments`) support `assignee_type = 'user'` with highest precedence in policy resolution cascades.
- Verification: unit tests passing, full regression via `./leedevkit test api`.
