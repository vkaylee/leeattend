ALTER TABLE attendance_daily_summaries
    DROP COLUMN standard_minutes,
    DROP COLUMN early_ot_minutes,
    DROP COLUMN late_ot_minutes,
    DROP COLUMN break_ot_minutes,
    DROP COLUMN is_weekend;
