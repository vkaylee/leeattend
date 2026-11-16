ALTER TABLE public.shift_swap_requests
    ADD COLUMN requester_revision_id uuid,
    ADD COLUMN requester_revision_number bigint,
    ADD COLUMN target_revision_id uuid,
    ADD COLUMN target_revision_number bigint;

ALTER TABLE public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_requester_revision_fkey
        FOREIGN KEY (requester_revision_id) REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT,
    ADD CONSTRAINT shift_swap_requests_target_revision_fkey
        FOREIGN KEY (target_revision_id) REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT,
    ADD CONSTRAINT shift_swap_requests_revision_numbers_check
        CHECK ((requester_revision_id IS NULL AND requester_revision_number IS NULL)
            OR (requester_revision_id IS NOT NULL AND requester_revision_number >= 1)),
    ADD CONSTRAINT shift_swap_requests_target_revision_numbers_check
        CHECK ((target_revision_id IS NULL AND target_revision_number IS NULL)
            OR (target_revision_id IS NOT NULL AND target_revision_number >= 1));

CREATE INDEX shift_swap_requests_official_revisions_idx
    ON public.shift_swap_requests (requester_revision_id, target_revision_id)
    WHERE requester_revision_id IS NOT NULL OR target_revision_id IS NOT NULL;
