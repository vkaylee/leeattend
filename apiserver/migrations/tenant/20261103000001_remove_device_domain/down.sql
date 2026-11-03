-- Restore removed schema from a backup before rollback. Attendance provenance and biometric rows are not recoverable from this migration.

CREATE TABLE public.devices (
    id uuid NOT NULL PRIMARY KEY,
    agent_id varchar(50) NOT NULL,
    name varchar(100) NOT NULL,
    ip varchar(45) NOT NULL,
    port integer DEFAULT 4370,
    is_enabled boolean DEFAULT true,
    last_status varchar(20) DEFAULT 'unknown',
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
    workspace_id uuid NOT NULL,
    brand text DEFAULT 'zkteco' NOT NULL,
    password varchar(50),
    timezone varchar(50) DEFAULT 'UTC'
);

ALTER TABLE public.external_attendance_events ADD COLUMN device_id text;
UPDATE public.external_attendance_events SET device_id = source_id;
ALTER TABLE public.external_attendance_events ALTER COLUMN device_id SET NOT NULL;
ALTER TABLE public.external_attendance_events DROP CONSTRAINT external_attendance_events_source_identity_unique;
ALTER TABLE public.external_attendance_events ADD CONSTRAINT external_attendance_events_identity_unique UNIQUE (workspace_id, device_id, external_event_id);
DROP INDEX IF EXISTS idx_external_attendance_events_source_time;
CREATE INDEX idx_external_attendance_events_device_time ON public.external_attendance_events (workspace_id, device_id, occurred_at);
ALTER TABLE public.external_attendance_events DROP COLUMN source_id;

CREATE TYPE public.log_source_type_old AS ENUM ('device', 'import', 'manual');
ALTER TABLE public.raw_attendance_logs ALTER COLUMN source TYPE public.log_source_type_old USING source::text::public.log_source_type_old;
DROP TYPE public.log_source_type;
ALTER TYPE public.log_source_type_old RENAME TO log_source_type;
ALTER TABLE public.raw_attendance_logs ADD COLUMN device_id varchar(50);
ALTER TABLE public.attendance_staging_logs ADD COLUMN device_id text;

DROP TABLE IF EXISTS public.attendance_source_pull_states;
DROP TABLE IF EXISTS public.attendance_sources;
