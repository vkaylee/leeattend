-- Delegation assignees are workspace users, not employees.
-- Preserve a tenant-local rollback copy before removing legacy assignments.
CREATE TABLE public.behalf_assignments_user_cutover_backup AS
SELECT * FROM public.behalf_assignments;

DELETE FROM public.behalf_assignments;

COMMENT ON TABLE public.behalf_assignments_user_cutover_backup IS
    'Rollback backup for 20261103000002_behalf_assignments_user_cutover; remove after verified release rollback window.';
