-- ============================================================================
-- Work Segments Architecture: Full multi-segment support per work day
--
-- Enables: split shifts (ca gãy), multi-job days, on-call/standby segments,
-- training blocks, travel segments, configurable gap compensation,
-- per-segment OT calculation, and cross-midnight shift support.
-- ============================================================================

-- 1. New ENUM types
CREATE TYPE public.segment_type AS ENUM (
    'work',
    'standby',
    'on_call',
    'training',
    'travel'
);

CREATE TYPE public.gap_compensation_type AS ENUM (
    'unpaid',
    'standby',
    'break'
);

CREATE TYPE public.segment_status_type AS ENUM (
    'scheduled',
    'in_progress',
    'completed',
    'cancelled'
);

CREATE TYPE public.ot_aggregation_mode_type AS ENUM (
    'aggregate',
    'per_segment'
);

-- 2. Add new columns to shifts table (policy template)
ALTER TABLE public.shifts
    ADD COLUMN ot_aggregation_mode public.ot_aggregation_mode_type DEFAULT 'aggregate' NOT NULL,
    ADD COLUMN is_night_shift boolean DEFAULT false NOT NULL;

COMMENT ON COLUMN public.shifts.ot_aggregation_mode IS 'How OT is calculated across segments: aggregate (sum all then apply threshold) or per_segment (OT calculated independently per segment)';
COMMENT ON COLUMN public.shifts.is_night_shift IS 'When true, shift hours crossing midnight are attributed to the start day';

-- 3. shift_default_intervals — optional template time blocks within a shift
CREATE TABLE public.shift_default_intervals (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    shift_id uuid NOT NULL,
    sort_order integer NOT NULL,
    interval_name character varying(100),
    start_time time without time zone NOT NULL,
    end_time time without time zone NOT NULL,

    -- Break config for this interval (nullable = inherit from parent shift)
    break_duration_minutes integer,
    break_start time without time zone,
    break_end time without time zone,
    is_break_auto_deduct boolean,
    break_is_overtime boolean,

    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

COMMENT ON TABLE public.shift_default_intervals IS 'Optional time blocks within a shift template. When empty, the shift start_time/end_time is used as a single virtual interval.';

-- 4. work_segments — the central runtime scheduling entity (replaces shift_rosters)
CREATE TABLE public.work_segments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    employment_id uuid NOT NULL,
    date date NOT NULL,

    -- Reference to the shift policy template
    shift_id uuid NOT NULL,

    -- Optional reference to which interval of the shift this segment instantiates
    interval_id uuid,

    -- Ordering within the day
    sort_order integer DEFAULT 0 NOT NULL,

    -- Scheduled times (override shift/interval defaults if set)
    scheduled_start time without time zone NOT NULL,
    scheduled_end time without time zone NOT NULL,

    -- Segment classification
    segment_type public.segment_type DEFAULT 'work' NOT NULL,
    segment_status public.segment_status_type DEFAULT 'scheduled' NOT NULL,

    -- Compensation
    compensation_rate numeric(3,2) DEFAULT 1.00 NOT NULL,

    -- Gap configuration before this segment
    gap_compensation_before public.gap_compensation_type DEFAULT 'unpaid' NOT NULL,
    gap_compensation_rate_before numeric(3,2) DEFAULT 0.00 NOT NULL,

    -- Override configs (nullable = inherit from shift)
    break_config_override jsonb,
    ot_config_override jsonb,

    -- Source tracking (where this segment came from)
    source character varying(30) DEFAULT 'manual' NOT NULL,
    source_detail jsonb DEFAULT '{}'::jsonb NOT NULL,

    -- Metadata
    notes text,
    generation_metadata jsonb DEFAULT '{}'::jsonb,

    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

COMMENT ON TABLE public.work_segments IS 'Work segments are the atomic unit of scheduled work. One work day can have N segments. Replaces shift_rosters as the scheduling entity.';
COMMENT ON COLUMN public.work_segments.segment_type IS 'work=regular shift, standby=paid waiting, on_call=unpaid availability, training=education, travel=business trip';
COMMENT ON COLUMN public.work_segments.gap_compensation_before IS 'How the gap BEFORE this segment is treated: unpaid (no pay), standby (partial pay), break (auto-deduct like a long break)';
COMMENT ON COLUMN public.work_segments.compensation_rate IS '1.00 = 100% pay for this segment. For standby typically 0.30-0.50';
COMMENT ON COLUMN public.work_segments.source IS 'roster | pattern | rule | fixed | manual | auto';

-- 5. Add segment_id to time_entries
ALTER TABLE public.time_entries
    ADD COLUMN segment_id uuid;

COMMENT ON COLUMN public.time_entries.segment_id IS 'Links the actual punch session to its scheduled work segment. NULL for backward compat (entries before Work Segments migration).';

-- 6. Data migration: auto-create work_segments from existing shift_rosters
INSERT INTO public.work_segments (
    workspace_id,
    employment_id,
    date,
    shift_id,
    sort_order,
    scheduled_start,
    scheduled_end,
    segment_type,
    segment_status,
    source,
    source_detail,
    notes,
    created_at,
    updated_at
)
SELECT
    s.workspace_id,
    sr.employment_id,
    sr.date,
    sr.shift_id,
    0 AS sort_order,
    s.start_time AS scheduled_start,
    s.end_time AS scheduled_end,
    'work'::public.segment_type,
    CASE
        WHEN sr.status = 'confirmed' THEN 'scheduled'::public.segment_status_type
        ELSE 'scheduled'::public.segment_status_type
    END,
    'roster'::character varying(30),
    jsonb_build_object('migrated_from', 'shift_rosters', 'original_roster_id', sr.id),
    sr.notes,
    sr.created_at,
    sr.updated_at
FROM public.shift_rosters sr
JOIN public.shifts s ON s.id = sr.shift_id
WHERE sr.shift_id IS NOT NULL;

-- Migrate day-off rosters (shift_id IS NULL) as cancelled segments
-- Only migrate if there is at least one active shift in the workspace (FK constraint)
INSERT INTO public.work_segments (
    workspace_id,
    employment_id,
    date,
    shift_id,
    sort_order,
    scheduled_start,
    scheduled_end,
    segment_type,
    segment_status,
    source,
    source_detail,
    notes,
    created_at,
    updated_at
)
SELECT
    emp.workspace_id,
    sr.employment_id,
    sr.date,
    (SELECT s.id FROM public.shifts s WHERE s.workspace_id = emp.workspace_id AND s.is_active = true LIMIT 1),
    0 AS sort_order,
    '00:00:00'::time AS scheduled_start,
    '00:00:00'::time AS scheduled_end,
    'work'::public.segment_type,
    'cancelled'::public.segment_status_type,
    'roster'::character varying(30),
    jsonb_build_object('migrated_from', 'shift_rosters', 'original_roster_id', sr.id, 'was_day_off', true),
    sr.notes,
    sr.created_at,
    sr.updated_at
FROM public.shift_rosters sr
JOIN public.employments emp ON emp.id = sr.employment_id
WHERE sr.shift_id IS NULL
  AND EXISTS (SELECT 1 FROM public.shifts s WHERE s.workspace_id = emp.workspace_id AND s.is_active = true);

-- 7. Link existing time_entries to work_segments
-- For entries within a work segment's scheduled window on the same day
UPDATE public.time_entries te
SET segment_id = ws.id
FROM public.work_segments ws
WHERE te.employment_id = ws.employment_id
  AND date_trunc('day', te.punch_in_time)::date = ws.date
  AND ws.segment_status != 'cancelled'
  AND te.segment_id IS NULL
  AND ws.shift_id = te.shift_id;

-- ============================================================================
-- Constraints & Indexes
-- ============================================================================

-- shift_default_intervals
ALTER TABLE ONLY public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_shift_id_sort_order_key UNIQUE (shift_id, sort_order);

-- work_segments
ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_pkey PRIMARY KEY (id);

-- No UNIQUE on (employment_id, date) — multiple segments per day are allowed
-- But add a partial unique index to prevent duplicate segments for the same interval
CREATE UNIQUE INDEX idx_work_segments_interval_unique
    ON public.work_segments (employment_id, date, interval_id)
    WHERE interval_id IS NOT NULL;

CREATE INDEX idx_work_segments_employment_date ON public.work_segments USING btree (employment_id, date);
CREATE INDEX idx_work_segments_shift_id ON public.work_segments USING btree (shift_id);
CREATE INDEX idx_work_segments_status ON public.work_segments USING btree (segment_status);
CREATE INDEX idx_work_segments_date_range ON public.work_segments USING btree (employment_id, date, sort_order);

-- Foreign Keys
ALTER TABLE ONLY public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_interval_id_fkey FOREIGN KEY (interval_id) REFERENCES public.shift_default_intervals(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.time_entries
    ADD CONSTRAINT time_entries_segment_id_fkey FOREIGN KEY (segment_id) REFERENCES public.work_segments(id) ON DELETE SET NULL;

-- Updated_at triggers
CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shift_default_intervals FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();
CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.work_segments FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- Source check constraint
ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_source_check CHECK (
        (source)::text = ANY (ARRAY['roster'::text, 'pattern'::text, 'rule'::text, 'fixed'::text, 'manual'::text, 'auto'::text])
    );

-- Gap compensation rate bounds
ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_gap_rate_check CHECK (
        gap_compensation_rate_before >= 0.00 AND gap_compensation_rate_before <= 1.00
    );

ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_compensation_rate_check CHECK (
        compensation_rate >= 0.00 AND compensation_rate <= 1.00
    );

-- Sort order non-negative
ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_sort_order_check CHECK (sort_order >= 0);

ALTER TABLE ONLY public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_sort_order_check CHECK (sort_order >= 0);

-- Time validity: scheduled_start must be before scheduled_end (unless cross-midnight)
-- Cross-midnight handled at application level via is_night_shift flag on parent shift
