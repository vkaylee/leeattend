-- Update "Base" components to be explicitly base for OT
UPDATE salary_component_definitions
SET is_base_for_ot = TRUE
WHERE (code = 'BASE' OR name = 'Lương cơ bản') AND is_base_for_ot = FALSE;
