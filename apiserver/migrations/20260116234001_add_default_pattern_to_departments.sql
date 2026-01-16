-- Add default_working_pattern_id to departments
ALTER TABLE departments 
ADD COLUMN default_working_pattern_id UUID REFERENCES working_patterns(id) ON DELETE SET NULL;
