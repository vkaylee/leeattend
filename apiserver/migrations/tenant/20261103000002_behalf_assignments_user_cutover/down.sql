-- Restore exact pre-cutover state. Any assignments created after cutover are discarded during rollback.
DELETE FROM public.behalf_assignments;

INSERT INTO public.behalf_assignments
SELECT * FROM public.behalf_assignments_user_cutover_backup;

DROP TABLE public.behalf_assignments_user_cutover_backup;
