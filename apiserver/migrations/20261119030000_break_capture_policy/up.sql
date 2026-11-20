BEGIN;

ALTER TABLE public.shift_default_intervals
    ADD COLUMN IF NOT EXISTS break_punch_mode text NOT NULL DEFAULT 'none',
    ADD COLUMN IF NOT EXISTS break_credit_mode text NOT NULL DEFAULT 'scheduled';

UPDATE public.shift_default_intervals
SET break_credit_mode = CASE
    WHEN is_break_auto_deduct THEN 'scheduled'
    ELSE 'none'
END
WHERE break_credit_mode = 'scheduled';

ALTER TABLE public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_break_punch_mode_check
        CHECK (break_punch_mode IN ('none', 'required')),
    ADD CONSTRAINT shift_default_intervals_break_credit_mode_check
        CHECK (break_credit_mode IN ('scheduled', 'actual', 'none')),
    ADD CONSTRAINT shift_default_intervals_break_policy_check
        CHECK (NOT (break_punch_mode = 'required' AND break_credit_mode = 'scheduled'));

CREATE TABLE public.time_entry_breaks (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    time_entry_id uuid NOT NULL,
    shift_interval_id uuid,
    break_start_at timestamp with time zone NOT NULL,
    break_end_at timestamp with time zone,
    source text NOT NULL DEFAULT 'punch',
    close_reason text,
    start_idempotency_key text,
    end_idempotency_key text,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT time_entry_breaks_pkey PRIMARY KEY (id),
    CONSTRAINT time_entry_breaks_entry_fkey FOREIGN KEY (time_entry_id)
        REFERENCES public.time_entries(id) ON DELETE RESTRICT,
    CONSTRAINT time_entry_breaks_interval_fkey FOREIGN KEY (shift_interval_id)
        REFERENCES public.shift_default_intervals(id) ON DELETE SET NULL,
    CONSTRAINT time_entry_breaks_source_check CHECK (source IN ('punch', 'system_implied', 'manual')),
    CONSTRAINT time_entry_breaks_close_reason_check CHECK (
        close_reason IS NULL OR close_reason IN ('punch', 'work_out', 'manual_correction')
    ),
    CONSTRAINT time_entry_breaks_order_check CHECK (break_end_at IS NULL OR break_end_at >= break_start_at)
);

CREATE INDEX time_entry_breaks_time_entry_idx
    ON public.time_entry_breaks (time_entry_id, break_start_at);
CREATE UNIQUE INDEX time_entry_breaks_one_open_idx
    ON public.time_entry_breaks (time_entry_id)
    WHERE break_end_at IS NULL;

CREATE TABLE public.attendance_action_idempotency (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_id uuid NOT NULL,
    idempotency_key text NOT NULL,
    action text NOT NULL,
    request_hash character(64) NOT NULL,
    response_snapshot jsonb NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT attendance_action_idempotency_pkey PRIMARY KEY (id),
    CONSTRAINT attendance_action_idempotency_employment_fkey FOREIGN KEY (employment_id)
        REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_action_idempotency_key_key UNIQUE (employment_id, idempotency_key),
    CONSTRAINT attendance_action_idempotency_action_check CHECK (
        action IN ('work_in', 'work_out', 'break_start', 'break_end')
    ),
    CONSTRAINT attendance_action_idempotency_hash_check CHECK (request_hash::text ~ '^[0-9a-f]{64}$'),
    CONSTRAINT attendance_action_idempotency_response_check CHECK (jsonb_typeof(response_snapshot) = 'object')
);

CREATE INDEX attendance_action_idempotency_created_idx
    ON public.attendance_action_idempotency (created_at);

ALTER TABLE public.attendance_daily_summaries
    ADD COLUMN scheduled_break_minutes integer NOT NULL DEFAULT 0,
    ADD COLUMN actual_break_minutes integer NOT NULL DEFAULT 0,
    ADD COLUMN deducted_break_minutes integer NOT NULL DEFAULT 0,
    ADD COLUMN net_work_minutes integer NOT NULL DEFAULT 0,
    ADD COLUMN net_paid_minutes integer NOT NULL DEFAULT 0,
    ADD COLUMN break_provisional boolean NOT NULL DEFAULT false,
    ADD COLUMN break_exception_code text,
    ADD CONSTRAINT attendance_daily_summaries_break_minutes_check CHECK (
        scheduled_break_minutes >= 0
        AND actual_break_minutes >= 0
        AND deducted_break_minutes >= 0
        AND net_work_minutes >= 0
        AND net_paid_minutes >= 0
    ),
    ADD CONSTRAINT attendance_daily_summaries_break_exception_check CHECK (
        (break_provisional AND break_exception_code IS NOT NULL)
        OR (NOT break_provisional)
    );

INSERT INTO public.schema_migrations(version)
VALUES ('20261119030000_break_capture_policy');

COMMIT;
