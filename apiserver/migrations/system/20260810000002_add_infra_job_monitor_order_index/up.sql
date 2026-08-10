CREATE INDEX idx_infra_job_records_monitor_created
    ON public.infra_job_records (created_at DESC, id DESC);
