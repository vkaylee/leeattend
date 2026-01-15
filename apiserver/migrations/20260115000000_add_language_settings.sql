-- Add language settings to workspaces and users

-- Default language for workspace (affects new users and guests)
ALTER TABLE workspaces ADD COLUMN default_language VARCHAR(10) NOT NULL DEFAULT 'vi';

-- Preferred language for individual users (overrides workspace default)
ALTER TABLE users ADD COLUMN preferred_language VARCHAR(10);
