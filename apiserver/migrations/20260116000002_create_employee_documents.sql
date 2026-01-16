-- Employee Self-Service: Documents table
-- Stores metadata for employee-uploaded documents

CREATE TABLE employee_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    document_type VARCHAR(50) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    expires_at DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    uploaded_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for faster lookups
CREATE INDEX idx_employee_documents_employee_id ON employee_documents(employee_id);
CREATE INDEX idx_employee_documents_workspace_id ON employee_documents(workspace_id);
CREATE INDEX idx_employee_documents_type ON employee_documents(document_type);

-- Index for finding expiring documents
CREATE INDEX idx_employee_documents_expires_at ON employee_documents(expires_at)
    WHERE expires_at IS NOT NULL AND is_active = true;
