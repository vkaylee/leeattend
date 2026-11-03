-- Remove device domain and replace provider checkpoints with attendance-source state.
-- This migration is destructive: take a tenant backup and export biometric data before applying.

CREATE TABLE public.attendance_sources (
    id uuid PRIMARY KEY,
    workspace_id uuid NOT NULL,
    provider_source_id text NOT NULL,
    is_enabled boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT attendance_sources_workspace_provider_unique UNIQUE (workspace_id, provider_source_id)
);
CREATE INDEX idx_attendance_sources_workspace_enabled ON public.attendance_sources (workspace_id, is_enabled);

INSERT INTO public.attendance_sources (id, workspace_id, provider_source_id, is_enabled, created_at)
SELECT id, workspace_id, id::text, is_enabled, created_at
FROM public.devices;

CREATE TABLE public.attendance_source_pull_states (
    id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    workspace_id uuid NOT NULL,
    source_id uuid NOT NULL,
    cursor text,
    status varchar(20) NOT NULL DEFAULT 'idle',
    lease_owner varchar(100),
    lease_token uuid,
    lease_expires_at timestamptz,
    last_success_at timestamptz,
    last_failure_at timestamptz,
    last_error text,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT attendance_source_pull_states_status_check CHECK (status IN ('idle', 'running', 'failed')),
    CONSTRAINT attendance_source_pull_states_unique UNIQUE (workspace_id, source_id)
);
CREATE INDEX idx_attendance_source_pull_states_lease ON public.attendance_source_pull_states (status, lease_expires_at);

INSERT INTO public.attendance_source_pull_states (workspace_id, source_id, cursor, status, lease_owner, lease_token, lease_expires_at, last_success_at, last_failure_at, last_error, updated_at)
SELECT workspace_id, device_id, cursor, status, lease_owner, lease_token, lease_expires_at, last_success_at, last_failure_at, last_error, updated_at
FROM public.device_pull_states;

ALTER TABLE public.external_attendance_events ADD COLUMN source_id text;
UPDATE public.external_attendance_events SET source_id = device_id;
ALTER TABLE public.external_attendance_events ALTER COLUMN source_id SET NOT NULL;
ALTER TABLE public.external_attendance_events DROP CONSTRAINT external_attendance_events_identity_unique;
ALTER TABLE public.external_attendance_events ADD CONSTRAINT external_attendance_events_source_identity_unique UNIQUE (workspace_id, source_id, external_event_id);
DROP INDEX IF EXISTS idx_external_attendance_events_device_time;
CREATE INDEX idx_external_attendance_events_source_time ON public.external_attendance_events (workspace_id, source_id, occurred_at);
ALTER TABLE public.external_attendance_events DROP COLUMN device_id;

ALTER TABLE public.raw_attendance_logs DROP COLUMN device_id;
ALTER TABLE public.attendance_staging_logs DROP COLUMN device_id;
ALTER TYPE public.log_source_type RENAME TO log_source_type_old;
CREATE TYPE public.log_source_type AS ENUM ('import', 'manual');
ALTER TABLE public.raw_attendance_logs ALTER COLUMN source TYPE public.log_source_type USING CASE WHEN source::text = 'device' THEN 'import'::public.log_source_type ELSE source::text::public.log_source_type END;
DROP TYPE public.log_source_type_old;

DROP TABLE IF EXISTS public.device_enrollments CASCADE;
DROP TABLE IF EXISTS public.employee_biometric_templates CASCADE;
DROP TABLE IF EXISTS public.devices CASCADE;
DROP TABLE IF EXISTS public.device_pull_states CASCADE;
