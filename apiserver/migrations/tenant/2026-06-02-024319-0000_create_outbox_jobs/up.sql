CREATE TABLE public.outbox_jobs (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    job_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_outbox_jobs_workspace_status ON public.outbox_jobs (workspace_id, status);
CREATE INDEX idx_outbox_jobs_created_at ON public.outbox_jobs (created_at);

CREATE TRIGGER trigger_outbox_jobs_updated_at
    BEFORE UPDATE ON public.outbox_jobs
    FOR EACH ROW
    EXECUTE FUNCTION public.fn_set_updated_at();
