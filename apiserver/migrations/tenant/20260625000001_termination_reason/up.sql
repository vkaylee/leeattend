ALTER TABLE public.employments
    ADD COLUMN IF NOT EXISTS termination_reason VARCHAR(50) NULL;
