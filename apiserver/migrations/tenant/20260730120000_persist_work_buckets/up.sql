ALTER TABLE attendance_daily_summaries
    ADD COLUMN standard_minutes integer DEFAULT 0 NOT NULL,
    ADD COLUMN early_ot_minutes integer DEFAULT 0 NOT NULL,
    ADD COLUMN late_ot_minutes integer DEFAULT 0 NOT NULL,
    ADD COLUMN break_ot_minutes integer DEFAULT 0 NOT NULL,
    ADD COLUMN is_weekend boolean DEFAULT false NOT NULL;
