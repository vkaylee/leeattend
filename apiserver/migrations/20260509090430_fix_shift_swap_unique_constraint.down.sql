DROP INDEX IF EXISTS public.idx_uq_shift_swap_pair_date;

ALTER TABLE public.shift_swap_requests 
ADD CONSTRAINT uq_shift_swap_pair_date 
UNIQUE (workspace_id, requester_id, target_id, swap_date);
