-- up.sql
ALTER TABLE employers ADD COLUMN parent_id UUID REFERENCES employers(id) ON DELETE SET NULL;
ALTER TABLE employers ADD COLUMN relationship_type VARCHAR(100);
CREATE INDEX idx_employers_parent_id ON employers(parent_id);
