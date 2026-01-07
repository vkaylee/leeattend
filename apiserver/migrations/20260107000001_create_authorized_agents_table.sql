-- Migration: Create authorized_agents table
CREATE TABLE IF NOT EXISTS authorized_agents (
    id VARCHAR(50) PRIMARY KEY, -- Khớp với AGENT_ID
    name VARCHAR(100) NOT NULL,
    token VARCHAR(255) NOT NULL, -- Secret token gửi kèm handshake
    is_trusted BOOLEAN DEFAULT false,
    last_ip VARCHAR(45),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_authorized_agents_trusted ON authorized_agents(is_trusted);
