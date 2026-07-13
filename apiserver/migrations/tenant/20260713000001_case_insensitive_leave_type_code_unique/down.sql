-- Rollback to the prior state: exact-case table constraint + partial
-- case-insensitive index on active rows only.

DROP INDEX IF EXISTS public.leave_types_workspace_id_code_key;

ALTER TABLE public.leave_types
    ADD CONSTRAINT leave_types_workspace_id_code_key UNIQUE (workspace_id, code);

CREATE UNIQUE INDEX uq_leave_types_code
    ON public.leave_types USING btree (workspace_id, lower((code)::text))
    WHERE (deleted_at IS NULL);
