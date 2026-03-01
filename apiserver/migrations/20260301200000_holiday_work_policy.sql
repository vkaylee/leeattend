-- Holiday work policy fields
-- work_threshold_percent: % of shift worked to reverse leave (NULL = default 50%)
-- is_credit_on_work: whether to credit holiday pay when employee works on holiday
ALTER TABLE holidays
  ADD COLUMN work_threshold_percent INTEGER DEFAULT NULL,
  ADD COLUMN is_credit_on_work BOOLEAN NOT NULL DEFAULT true;
