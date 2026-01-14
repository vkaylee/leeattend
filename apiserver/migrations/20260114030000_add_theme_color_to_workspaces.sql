-- Add theme_primary_color to workspaces table
ALTER TABLE workspaces ADD COLUMN theme_primary_color TEXT DEFAULT '#7c3aed';
