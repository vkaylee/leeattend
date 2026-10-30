CREATE TABLE public.device_pull_states (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    workspace_id UUID NOT NULL,
    device_id UUID NOT NULL,
    cursor TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'idle',
    lease_owner VARCHAR(100),
    lease_token UUID,
    lease_expires_at TIMESTAMPTZ,
    last_success_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT device_pull_states_status_check CHECK (status IN ('idle', 'running', 'failed')),
    CONSTRAINT device_pull_states_workspace_device_unique UNIQUE (workspace_id, device_id)
);

CREATE INDEX idx_device_pull_states_lease
    ON public.device_pull_states (status, lease_expires_at);

CREATE TABLE public.external_attendance_events (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    workspace_id UUID NOT NULL,
    device_id TEXT NOT NULL,
    external_event_id VARCHAR(255) NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    received_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT external_attendance_events_identity_unique
        UNIQUE (workspace_id, device_id, external_event_id)
);
CREATE INDEX idx_external_attendance_events_cleanup
    ON public.external_attendance_events (workspace_id, received_at);
CREATE INDEX idx_external_attendance_events_device_time
    ON public.external_attendance_events (workspace_id, device_id, occurred_at);
