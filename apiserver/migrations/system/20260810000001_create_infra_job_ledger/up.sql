CREATE TABLE public.infra_job_records (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    parent_job_id UUID,
    task_type VARCHAR(100) NOT NULL,
    priority VARCHAR(20) NOT NULL,
    state VARCHAR(30) NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    lease_owner VARCHAR(100),
    lease_token UUID,
    lease_expires_at TIMESTAMPTZ,
    next_attempt_at TIMESTAMPTZ,
    request_id UUID,
    trace_id VARCHAR(128),
    payload_hash VARCHAR(128),
    error_code VARCHAR(100),
    error_summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    queued_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT infra_job_records_state_check CHECK (
        state IN ('dispatching', 'queued', 'processing', 'completed', 'failed', 'retryable', 'dead_letter', 'lost')
    ),
    CONSTRAINT infra_job_records_attempt_count_check CHECK (attempt_count >= 0)
);

CREATE INDEX idx_infra_job_records_workspace_created
    ON public.infra_job_records (workspace_id, created_at DESC, id DESC);
CREATE INDEX idx_infra_job_records_state_lease
    ON public.infra_job_records (state, lease_expires_at);
CREATE INDEX idx_infra_job_records_parent
    ON public.infra_job_records (parent_job_id, created_at ASC);
CREATE INDEX idx_infra_job_records_retry
    ON public.infra_job_records (state, next_attempt_at)
    WHERE state = 'retryable';

CREATE TABLE public.infra_job_events (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES public.infra_job_records(id) ON DELETE RESTRICT,
    workspace_id UUID NOT NULL,
    event_type VARCHAR(40) NOT NULL,
    from_state VARCHAR(30),
    to_state VARCHAR(30) NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    lease_token UUID,
    worker_id VARCHAR(100),
    request_id UUID,
    trace_id VARCHAR(128),
    reason_code VARCHAR(100),
    reason_summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT infra_job_events_attempt_count_check CHECK (attempt_count >= 0)
);

CREATE INDEX idx_infra_job_events_job_created
    ON public.infra_job_events (job_id, created_at ASC, id ASC);
CREATE INDEX idx_infra_job_events_workspace_created
    ON public.infra_job_events (workspace_id, created_at DESC);

CREATE TRIGGER trg_infra_job_records_updated_at
    BEFORE UPDATE ON public.infra_job_records
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

COMMENT ON TABLE public.infra_job_records IS
    'Authoritative operational lifecycle for asynchronous jobs. Raw payloads and PII are intentionally excluded.';
COMMENT ON TABLE public.infra_job_events IS
    'Append-only, PII-safe lifecycle transitions for asynchronous jobs.';
