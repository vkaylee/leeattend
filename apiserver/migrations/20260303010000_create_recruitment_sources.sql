-- Recruitment Source Tracking
-- Track whether employees came from direct hiring, employee referral, or agency

-- Enum for source type
CREATE TYPE recruitment_source_type AS ENUM ('direct', 'referral', 'agency');

-- Sources table
CREATE TABLE recruitment_sources (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id          UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    source_type           recruitment_source_type NOT NULL,
    name                  VARCHAR(255) NOT NULL,
    -- Referral fields (source_type = 'referral')
    referrer_employee_id  UUID REFERENCES employees(id) ON DELETE SET NULL,
    referral_bonus_amount NUMERIC(15, 2),
    referral_bonus_paid   BOOLEAN NOT NULL DEFAULT FALSE,
    -- Agency fields (source_type = 'agency')
    agency_contact_name   VARCHAR(255),
    agency_contact_email  VARCHAR(255),
    agency_fee_amount     NUMERIC(15, 2),
    agency_fee_paid       BOOLEAN NOT NULL DEFAULT FALSE,
    -- Common
    notes                 TEXT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Link employees to their recruitment source
ALTER TABLE employees ADD COLUMN recruitment_source_id UUID REFERENCES recruitment_sources(id) ON DELETE SET NULL;

-- Indexes for dashboard queries
CREATE INDEX idx_recruitment_sources_workspace ON recruitment_sources(workspace_id);
CREATE INDEX idx_employees_recruitment_source ON employees(recruitment_source_id);
