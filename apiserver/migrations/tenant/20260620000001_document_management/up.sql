-- Create document_folders table for nested directories
CREATE TABLE document_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    parent_folder_id UUID REFERENCES document_folders(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for fast lookup within a workspace and hierarchy
CREATE INDEX idx_document_folders_workspace ON document_folders(workspace_id);
CREATE INDEX idx_document_folders_parent ON document_folders(parent_folder_id);

-- Attach foreign key to existing workspace_files
ALTER TABLE workspace_files ADD CONSTRAINT fk_workspace_files_folder_id FOREIGN KEY (folder_id) REFERENCES document_folders(id) ON DELETE SET NULL;

-- Create signature_requests table for E-Sign Tracking
CREATE TABLE signature_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES workspace_files(id) ON DELETE CASCADE,
    signer_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, signed, declined
    signed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_signature_requests_workspace ON signature_requests(workspace_id);
CREATE INDEX idx_signature_requests_document ON signature_requests(document_id);
CREATE INDEX idx_signature_requests_signer ON signature_requests(signer_id);

-- Create file_acknowledgements table for tracking who read/understood a policy
CREATE TABLE file_acknowledgements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES workspace_files(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, acknowledged
    acknowledged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_file_acknowledgements_workspace ON file_acknowledgements(workspace_id);
CREATE INDEX idx_file_acknowledgements_document ON file_acknowledgements(document_id);
CREATE INDEX idx_file_acknowledgements_employee ON file_acknowledgements(employee_id);
