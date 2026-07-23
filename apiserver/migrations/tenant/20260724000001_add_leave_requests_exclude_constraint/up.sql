-- Add EXCLUDE constraint on leave_requests to prevent overlapping approved/pre_approved leaves
-- for the same employment. This is a database-level guardrail that prevents the TOCTOU
-- race condition where two concurrent submit_leave_request calls could both read "no conflict"
-- and create overlapping leave requests.
--
-- Pattern mirrors existing constraints on:
--   leave_policy_assignments, overtime_policy_assignments, payroll_rule_assignments

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_no_overlap_active
    EXCLUDE USING gist (
        employment_id WITH =,
        daterange(start_date, end_date, '[]'::text) WITH &&
    ) WHERE (status IN ('approved', 'pre_approved'));
