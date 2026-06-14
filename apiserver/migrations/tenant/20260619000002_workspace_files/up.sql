-- Up migration: create workspace_files and alter employee_profiles/leave_requests
CREATE TABLE workspace_files (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID NOT NULL,
    
    file_name       VARCHAR(255) NOT NULL,
    storage_key     VARCHAR(500) NOT NULL UNIQUE,  -- Full S3 key
    file_size       BIGINT NOT NULL,                -- Support >2GB
    mime_type       VARCHAR(100) NOT NULL,
    content_hash    VARCHAR(64),                    -- SHA-256
    
    category        VARCHAR(50) NOT NULL,
    folder_id       UUID,                           -- Optional folder placement (Phase 5)
    
    owner_type      VARCHAR(50) NOT NULL,
    owner_id        UUID NOT NULL,
    
    uploaded_by     UUID,                           -- user_id who uploaded
    is_active       BOOLEAN NOT NULL DEFAULT true,
    expires_at      TIMESTAMPTZ,
    scan_status     VARCHAR(20) NOT NULL DEFAULT 'pending',
    scanned_at      TIMESTAMPTZ,
    
    version         INT NOT NULL DEFAULT 1,
    parent_file_id  UUID REFERENCES workspace_files(id),
    is_latest       BOOLEAN NOT NULL DEFAULT true,  -- Denormalized for fast lookup
    
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Performance indexes
CREATE INDEX idx_wf_owner ON workspace_files(workspace_id, owner_type, owner_id) WHERE is_active = true;
CREATE INDEX idx_wf_category ON workspace_files(workspace_id, category) WHERE is_active = true;
CREATE INDEX idx_wf_latest ON workspace_files(workspace_id, owner_type, owner_id, is_latest) WHERE is_latest = true;
CREATE INDEX idx_wf_scan_pending ON workspace_files(scan_status) WHERE scan_status = 'pending';
CREATE INDEX idx_wf_expires ON workspace_files(expires_at) WHERE expires_at IS NOT NULL AND is_active = true;

-- Alter existing tables
ALTER TABLE employee_profiles ADD COLUMN avatar_file_id UUID REFERENCES workspace_files(id);
ALTER TABLE leave_requests ADD COLUMN attachment_file_ids UUID[] DEFAULT '{}';
