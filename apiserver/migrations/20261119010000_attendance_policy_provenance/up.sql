BEGIN;

ALTER TABLE public.attendance_daily_summaries
    ADD COLUMN work_policy_version_id uuid REFERENCES public.labor_policy_group_versions(id),
    ADD COLUMN shift_version_id uuid REFERENCES public.shift_versions(id),
    ADD CONSTRAINT attendance_daily_summaries_minutes_nonnegative CHECK (
        actual_work_minutes >= 0 AND standard_minutes >= 0 AND system_detected_ot_minutes >= 0
        AND early_ot_minutes >= 0 AND late_ot_minutes >= 0 AND break_ot_minutes >= 0
    );

CREATE INDEX attendance_daily_summaries_policy_provenance_idx
    ON public.attendance_daily_summaries (employment_id, date, work_policy_version_id, shift_version_id);

INSERT INTO public.schema_migrations(version)
VALUES ('20261119010000_attendance_policy_provenance');

COMMIT;
