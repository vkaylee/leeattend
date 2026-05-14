-- Seed core system salary bases
INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system)
VALUES 
  ('b0000000-0000-0000-0000-000000000001', NULL, 'Gross Salary', 'gross', 'Base for all basic, taxable, and insurable income components', TRUE),
  ('b0000000-0000-0000-0000-000000000002', NULL, 'Net Salary', 'net', 'Final take-home pay after all deductions', TRUE),
  ('b0000000-0000-0000-0000-000000000003', NULL, 'Social Insurance Base', 'bhxh', 'Base for calculating social insurance, health insurance, and unemployment insurance contributions', TRUE),
  ('b0000000-0000-0000-0000-000000000004', NULL, 'Taxable Income', 'taxable', 'Base for calculating personal income tax (PIT)', TRUE)
ON CONFLICT (slug) WHERE workspace_id IS NULL DO NOTHING;
