ALTER TABLE attendance_staging_logs RENAME TO attendance_staging_logs_old;

-- 3. Create Partitioned table: attendance_staging_logs
CREATE TABLE attendance_staging_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    device_id text,
    raw_user_id text NOT NULL,
    punch_time timestamp with time zone NOT NULL,
    raw_data jsonb,
    fingerprint text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Create Partitions for attendance_staging_logs (2026)
CREATE TABLE attendance_staging_logs_y2026m01 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');

CREATE TABLE attendance_staging_logs_y2026m02 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');

CREATE TABLE attendance_staging_logs_y2026m03 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

CREATE TABLE attendance_staging_logs_y2026m04 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');

CREATE TABLE attendance_staging_logs_y2026m05 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');

CREATE TABLE attendance_staging_logs_y2026m06 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');

CREATE TABLE attendance_staging_logs_y2026m07 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-07-01') TO ('2026-08-01');

CREATE TABLE attendance_staging_logs_y2026m08 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-08-01') TO ('2026-09-01');

CREATE TABLE attendance_staging_logs_y2026m09 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-09-01') TO ('2026-10-01');

CREATE TABLE attendance_staging_logs_y2026m10 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-10-01') TO ('2026-11-01');

CREATE TABLE attendance_staging_logs_y2026m11 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-11-01') TO ('2026-12-01');

CREATE TABLE attendance_staging_logs_y2026m12 PARTITION OF attendance_staging_logs FOR VALUES FROM ('2026-12-01') TO ('2027-01-01');

CREATE TABLE attendance_staging_logs_default PARTITION OF attendance_staging_logs DEFAULT;

-- 4. Copy data from old tables to new partitioned tables
INSERT INTO audit_logs (id, workspace_id, actor_id, target_user_id, target_resource_id, action, details, ip_address, user_agent, created_at)
SELECT id, workspace_id, actor_id, target_user_id, target_resource_id, action, details, ip_address, user_agent, COALESCE(created_at, now())
FROM audit_logs_old;

INSERT INTO attendance_staging_logs (id, workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint, status, created_at)
SELECT id, workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint, status, COALESCE(created_at, now())
FROM attendance_staging_logs_old;

-- 5. Drop old tables
DROP TABLE audit_logs_old CASCADE;

DROP TABLE attendance_staging_logs_old CASCADE;

-- 6. Add indexes
CREATE INDEX idx_audit_logs_workspace_created ON audit_logs(workspace_id, created_at DESC);

CREATE INDEX idx_audit_logs_action ON audit_logs(action);

CREATE INDEX idx_audit_logs_target_user ON audit_logs(target_user_id);

CREATE INDEX idx_staging_logs_workspace_status ON attendance_staging_logs(workspace_id, status);

CREATE INDEX idx_staging_logs_fingerprint ON attendance_staging_logs(fingerprint);