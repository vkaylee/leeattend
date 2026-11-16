DROP INDEX IF EXISTS public.shift_swap_requests_official_revisions_idx;

ALTER TABLE public.shift_swap_requests
    DROP CONSTRAINT IF EXISTS shift_swap_requests_revision_numbers_check,
    DROP CONSTRAINT IF EXISTS shift_swap_requests_target_revision_numbers_check,
    DROP CONSTRAINT IF EXISTS shift_swap_requests_requester_revision_fkey,
    DROP CONSTRAINT IF EXISTS shift_swap_requests_target_revision_fkey,
    DROP COLUMN IF EXISTS requester_revision_id,
    DROP COLUMN IF EXISTS requester_revision_number,
    DROP COLUMN IF EXISTS target_revision_id,
    DROP COLUMN IF EXISTS target_revision_number;
