ALTER TABLE ONLY public.leave_requests
    DROP CONSTRAINT IF EXISTS leave_requests_no_overlap_active;
