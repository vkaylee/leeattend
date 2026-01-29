-- Up Migration: Add paper form entry metadata columns to attendance_daily_summaries

-- Add columns for paper form entry tracking
ALTER TABLE attendance_daily_summaries
ADD COLUMN IF NOT EXISTS claim_entered_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS claim_entered_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS paper_form_photo_url TEXT,
ADD COLUMN IF NOT EXISTS entered_via TEXT DEFAULT 'self_service';

-- Add comments for documentation
COMMENT ON COLUMN attendance_daily_summaries.claim_entered_by_user_id IS 'User ID who entered the paper form claim (NULL if self-service or system-generated)';
COMMENT ON COLUMN attendance_daily_summaries.claim_entered_at IS 'Timestamp when the paper form claim was manually entered (NULL if self-service)';
COMMENT ON COLUMN attendance_daily_summaries.paper_form_photo_url IS 'URL to uploaded photo of paper form (NULL if self-service or no photo)';
COMMENT ON COLUMN attendance_daily_summaries.entered_via IS 'Entry method: self_service | paper_form';
