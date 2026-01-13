-- Up Migration
ALTER TABLE workspaces ADD COLUMN date_format VARCHAR(20) NOT NULL DEFAULT 'DD/MM/YYYY';

-- Down Migration
-- ALTER TABLE workspaces DROP COLUMN date_format;
