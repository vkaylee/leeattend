-- Down Migration: Remove paper form entry metadata columns from attendance_daily_summaries

ALTER TABLE attendance_daily_summaries
DROP COLUMN IF EXISTS claim_entered_by_user_id,
DROP COLUMN IF EXISTS claim_entered_at,
DROP COLUMN IF EXISTS paper_form_photo_url,
DROP COLUMN IF EXISTS entered_via;
