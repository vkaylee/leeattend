ALTER TABLE public.shift_swap_requests DROP CONSTRAINT uq_shift_swap_pair_date;

CREATE UNIQUE INDEX idx_uq_shift_swap_pair_date 
ON public.shift_swap_requests (workspace_id, requester_id, target_id, swap_date) 
WHERE status NOT IN ('cancelled', 'rejected');