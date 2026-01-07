-- Add migration script here
CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY,
    agent_id VARCHAR(50) NOT NULL,
    name VARCHAR(100) NOT NULL,
    ip VARCHAR(45) NOT NULL,
    port INTEGER DEFAULT 4370,
    is_enabled BOOLEAN DEFAULT true,
    last_status VARCHAR(20) DEFAULT 'unknown',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_devices_agent ON devices(agent_id);