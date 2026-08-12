CREATE TABLE public.monitor_alerts (
    id UUID PRIMARY KEY,
    fingerprint VARCHAR(128) NOT NULL,
    alert_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    title VARCHAR(255) NOT NULL,
    summary TEXT NOT NULL,
    source VARCHAR(100) NOT NULL,
    first_seen_at TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT monitor_alerts_status_check CHECK (status IN ('active', 'resolved')),
    CONSTRAINT monitor_alerts_severity_check CHECK (severity IN ('info', 'warning', 'critical')),
    CONSTRAINT monitor_alerts_occurrence_count_check CHECK (occurrence_count > 0)
);

CREATE UNIQUE INDEX uq_monitor_alerts_active_fingerprint
    ON public.monitor_alerts (fingerprint)
    WHERE resolved_at IS NULL;
CREATE INDEX idx_monitor_alerts_last_seen
    ON public.monitor_alerts (last_seen_at DESC, id DESC);
CREATE INDEX idx_monitor_alerts_status_severity
    ON public.monitor_alerts (status, severity, last_seen_at DESC);

CREATE TABLE public.monitor_alert_events (
    id UUID PRIMARY KEY,
    alert_id UUID NOT NULL REFERENCES public.monitor_alerts(id) ON DELETE RESTRICT,
    event_type VARCHAR(40) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    summary TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT monitor_alert_events_severity_check CHECK (severity IN ('info', 'warning', 'critical'))
);

CREATE INDEX idx_monitor_alert_events_alert_occurred
    ON public.monitor_alert_events (alert_id, occurred_at ASC, id ASC);

CREATE TRIGGER trg_monitor_alerts_updated_at
    BEFORE UPDATE ON public.monitor_alerts
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

COMMENT ON TABLE public.monitor_alerts IS
    'Persisted, PII-safe infrastructure monitor alerts. Raw evaluator payloads are intentionally excluded.';
COMMENT ON TABLE public.monitor_alert_events IS
    'Append-only, PII-safe lifecycle events for persisted monitor alerts.';
