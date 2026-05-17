DELETE FROM salary_base_groups 
WHERE is_system = TRUE 
  AND workspace_id IS NULL 
  AND slug IN ('gross', 'net', 'bhxh', 'taxable');