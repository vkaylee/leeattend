-- Down Migration: Remove Payroll Rule Engine
DROP TABLE IF EXISTS payroll_rule_assignments;
DROP TABLE IF EXISTS payroll_rule_tiers;
DROP TABLE IF EXISTS payroll_rule_definitions;
