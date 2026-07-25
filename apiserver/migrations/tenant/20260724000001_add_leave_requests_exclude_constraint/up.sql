-- Add EXCLUDE constraint on leave_requests to prevent overlapping approved/pre_approved leaves
-- for the same employment. This is a database-level guardrail that prevents the TOCTOU
-- race condition where two concurrent submit_leave_request calls could both read "no conflict"
-- and create overlapping leave requests.
--
-- Pattern mirrors existing constraints on:
--   leave_policy_assignments, overtime_policy_assignments, payroll_rule_assignments
--
-- Step 1: Resolve any pre-existing overlapping approved/pre_approved leaves.
-- For each employment, we process leaves ordered by created_at (oldest first, UUID tiebreaker).
-- The greedy interval-scheduling algorithm keeps the earliest non-overlapping leaves and
-- cancels any that overlap with an already-kept one. This produces the maximum set of
-- non-overlapping leaves while preferring the earliest-submitted.
--
-- This cleanup is necessary because the EXCLUDE constraint cannot be created with
-- NOT VALID (PostgreSQL only supports NOT VALID for CHECK and FK constraints).

DO $$
DECLARE
    emp RECORD;
    lr RECORD;
    kept_ranges daterange[];
    current_range daterange;
    has_conflict boolean;
BEGIN
    FOR emp IN
        SELECT DISTINCT employment_id
        FROM public.leave_requests
        WHERE status IN ('approved', 'pre_approved')
    LOOP
        kept_ranges := ARRAY[]::daterange[];
        FOR lr IN
            SELECT id, start_date, end_date, created_at
            FROM public.leave_requests
            WHERE employment_id = emp.employment_id
              AND status IN ('approved', 'pre_approved')
            ORDER BY created_at ASC, id ASC
        LOOP
            current_range := daterange(lr.start_date, lr.end_date, '[]'::text);
            has_conflict := false;

            -- Check against all previously kept ranges
            FOR i IN 1..COALESCE(array_length(kept_ranges, 1), 0) LOOP
                IF kept_ranges[i] && current_range THEN
                    has_conflict := true;
                    EXIT;
                END IF;
            END LOOP;

            IF has_conflict THEN
                UPDATE public.leave_requests
                SET status = 'cancelled'::public.leave_status_type,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = lr.id;
            ELSE
                kept_ranges := array_append(kept_ranges, current_range);
            END IF;
        END LOOP;
    END LOOP;
END $$;

-- Step 2: Now that data is clean, add the exclusion constraint.
ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_no_overlap_active
    EXCLUDE USING gist (
        employment_id WITH =,
        daterange(start_date, end_date, '[]'::text) WITH &&
    ) WHERE (status IN ('approved', 'pre_approved'));
