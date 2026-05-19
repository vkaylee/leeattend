-- Phase 3: PostgreSQL Native Partitioning for High-Volume Tables

-- 1. Rename existing tables
ALTER TABLE audit_logs RENAME TO audit_logs_old;

-- 2. Create Partitioned table: audit_logs
CREATE TABLE audit_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    actor_id uuid,
    target_user_id uuid,
    target_resource_id uuid,
    action character varying(255) NOT NULL,
    details jsonb,
    ip_address text,
    user_agent text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Create Partitions for audit_logs (2026)
CREATE TABLE audit_logs_y2026m01 PARTITION OF audit_logs FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');

CREATE TABLE audit_logs_y2026m02 PARTITION OF audit_logs FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');

CREATE TABLE audit_logs_y2026m03 PARTITION OF audit_logs FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

CREATE TABLE audit_logs_y2026m04 PARTITION OF audit_logs FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');

CREATE TABLE audit_logs_y2026m05 PARTITION OF audit_logs FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');

CREATE TABLE audit_logs_y2026m06 PARTITION OF audit_logs FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');

CREATE TABLE audit_logs_y2026m07 PARTITION OF audit_logs FOR VALUES FROM ('2026-07-01') TO ('2026-08-01');

CREATE TABLE audit_logs_y2026m08 PARTITION OF audit_logs FOR VALUES FROM ('2026-08-01') TO ('2026-09-01');

CREATE TABLE audit_logs_y2026m09 PARTITION OF audit_logs FOR VALUES FROM ('2026-09-01') TO ('2026-10-01');

CREATE TABLE audit_logs_y2026m10 PARTITION OF audit_logs FOR VALUES FROM ('2026-10-01') TO ('2026-11-01');

CREATE TABLE audit_logs_y2026m11 PARTITION OF audit_logs FOR VALUES FROM ('2026-11-01') TO ('2026-12-01');

CREATE TABLE audit_logs_y2026m12 PARTITION OF audit_logs FOR VALUES FROM ('2026-12-01') TO ('2027-01-01');

CREATE TABLE audit_logs_default PARTITION OF audit_logs DEFAULT;

-- 3. Copy data from old table to new partitioned table
INSERT INTO audit_logs (id, workspace_id, actor_id, target_user_id, target_resource_id, action, details, ip_address, user_agent, created_at)
SELECT id, workspace_id, actor_id, target_user_id, target_resource_id, action, details, ip_address, user_agent, COALESCE(created_at, now())
FROM audit_logs_old;

-- 4. Drop old table
DROP TABLE audit_logs_old CASCADE;

-- 5. Add indexes on partitioned table
CREATE INDEX idx_audit_logs_workspace_created ON audit_logs(workspace_id, created_at DESC);

CREATE INDEX idx_audit_logs_action ON audit_logs(action);

CREATE INDEX idx_audit_logs_target_user ON audit_logs(target_user_id);