-- Down Migration: Remove salary component enhancements
ALTER TABLE salary_component_definitions DROP COLUMN IF EXISTS linked_rule_id;
ALTER TABLE salary_component_definitions DROP COLUMN IF EXISTS is_insurance_base;
ALTER TABLE salary_component_definitions DROP COLUMN IF EXISTS component_type;
