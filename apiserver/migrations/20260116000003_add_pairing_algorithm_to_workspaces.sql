-- Up Migration: Add pairing_algorithm to workspaces
ALTER TABLE workspaces ADD COLUMN pairing_algorithm VARCHAR(50) NOT NULL DEFAULT 'dp_pattern';
