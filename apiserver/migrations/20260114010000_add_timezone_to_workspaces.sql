-- Up
ALTER TABLE workspaces ADD COLUMN timezone VARCHAR(50) NOT NULL DEFAULT 'Asia/Ho_Chi_Minh';

-- Update existing workspaces (if any needed special ones, but default is fine for now)
-- UPDATE workspaces SET timezone = 'Asia/Ho_Chi_Minh' WHERE timezone IS NULL;
