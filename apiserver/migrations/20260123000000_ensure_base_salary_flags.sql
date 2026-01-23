-- Update "Base" components to be explicitly base for OT
UPDATE salary_component_definitions
SET is_base_for_ot = TRUE
WHERE (code = \047BASE\047 OR name = \047Lương cơ bản\047) AND is_base_for_ot = FALSE;
