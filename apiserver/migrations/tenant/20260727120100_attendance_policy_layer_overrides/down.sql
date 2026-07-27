-- Reverse of 20260727120100_attendance_policy_layer_overrides/up.sql
DROP TRIGGER IF EXISTS trg_apsv_overrides_draft_only
    ON public.attendance_policy_shift_version_overrides;
DROP TABLE IF EXISTS public.attendance_policy_shift_version_overrides;

DROP TRIGGER IF EXISTS trg_apst_overrides_draft_only
    ON public.attendance_policy_shift_template_overrides;
DROP TABLE IF EXISTS public.attendance_policy_shift_template_overrides;

DROP TRIGGER IF EXISTS trg_apdt_overrides_draft_only
    ON public.attendance_policy_day_type_overrides;
DROP TABLE IF EXISTS public.attendance_policy_day_type_overrides;

DROP FUNCTION IF EXISTS public.enforce_policy_override_only_when_draft();
