# Legacy System-Role Removal Migration

## Scope
Remove `user_system_roles`, `system_roles`, and `system_db_clusters` after standalone organization-admin access is reconciled. `system_settings` and workspace RBAC tables remain.

## Preconditions
- Take an encrypted PostgreSQL backup and record its checksum outside application logs.
- Confirm application version supports standalone workspace-admin authorization.
- Confirm at least one active `workspace_members` row uses the global `admin` workspace role for the singleton organization.
- Record counts for `system_db_clusters`, `system_roles`, `user_system_roles`, `workspace_roles`, and `workspace_members`.
- Abort if the singleton organization or global admin role is missing, if no active admin membership exists, or if foreign-key inspection differs from the expected baseline.

## Forward migration
1. Run backup and preflight checks in a maintenance window.
2. Stop old application versions before schema contraction. The old application reads the removed tables and is not mixed-version compatible after this migration.
3. Apply the application cutover that no longer reads or writes legacy system-role tables.
4. Verify the active organization administrator can authenticate and reach retained SMTP, health, and monitor routes.
5. Drop `user_system_roles` first, then `system_roles` and `system_db_clusters`, with dependent legacy constraints/indexes removed by the table drops.
6. Recheck table absence, workspace-admin membership, and retained route authorization.

The migration MUST run in one transaction where PostgreSQL lock duration is acceptable. If deployment tooling requires separate transactions, stop after each failed step and do not start the application until reconciliation succeeds.

## Rollback
The checked-in `down.sql` is intentionally fail-closed; it does not recreate deleted platform data. Rollback before new-version writes uses the encrypted pre-migration database backup and the previous application image. Restore the complete database, not only the dropped tables, because assignment timestamps, actor relationships, and cluster metadata must remain consistent.

After the new version writes data, roll-forward is safer than partial rollback. Restore only during a planned outage after assessing data written since backup and recording reconciliation results.

## Verification and reconciliation
- `system_db_clusters`, `system_roles`, and `user_system_roles` are absent.
- `workspace_roles` retains exactly one global `admin` role for the singleton organization.
- Every configured administrator retains an active `workspace_members` assignment to that role.
- No orphan `workspace_members`, role-permission, or user rows are introduced.
- Bootstrap remains idempotent and does not recreate legacy tables or assignments.
- Admin login succeeds; unauthenticated and non-admin requests to retained system routes return 401/403; authorization lookup failures deny access.
- Backup restore rehearsal recreates a usable pre-migration instance.

## Abort conditions
Abort and restore the pre-migration backup when admin access cannot be proven, any unexpected foreign key/dependency exists, migration lock/latency thresholds are exceeded, or reconciliation counts differ from preflight without an approved explanation.
