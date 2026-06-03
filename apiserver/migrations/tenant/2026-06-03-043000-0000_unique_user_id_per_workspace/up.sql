-- up.sql
-- Enforce 1 employee 1 user per workspace
CREATE UNIQUE INDEX idx_employees_workspace_user_id ON public.employees (workspace_id, user_id) WHERE user_id IS NOT NULL;
