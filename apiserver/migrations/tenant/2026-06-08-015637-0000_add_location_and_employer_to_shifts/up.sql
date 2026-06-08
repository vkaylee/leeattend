-- Your SQL goes here
ALTER TABLE shifts ADD COLUMN location_id UUID REFERENCES work_locations(id) ON DELETE SET NULL;
ALTER TABLE shifts ADD COLUMN employer_id UUID REFERENCES employers(id) ON DELETE SET NULL;

CREATE INDEX idx_shifts_location_id ON shifts(location_id);
CREATE INDEX idx_shifts_employer_id ON shifts(employer_id);
