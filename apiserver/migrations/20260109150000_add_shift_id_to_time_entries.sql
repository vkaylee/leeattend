ALTER TABLE time_entries ADD COLUMN shift_id UUID REFERENCES shifts(id);
