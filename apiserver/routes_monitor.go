package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerMonitorRoutes(router chi.Router, handler *api.MonitorHandler, auditHandler *api.AuditLogHandler) {
	registerRoute(router, "GET /api/v1/monitor/overview", handler.GetOverview)
	registerRoute(router, "GET /api/v1/monitor/jobs", handler.GetMonitorJobs)
	registerRoute(router, "GET /api/v1/monitor/correlations", handler.GetMonitorCorrelations)
	registerRoute(router, "GET /api/v1/monitor/stream", handler.Stream)
	registerRoute(router, "GET /api/v1/monitor/alerts", handler.ListAlerts)
	registerRoute(router, "GET /api/v1/monitor/alerts/{alert_id}", handler.GetAlert)
	registerRoute(router, "POST /api/v1/monitor/alerts/{alert_id}/acknowledge", handler.AcknowledgeAlert)
	registerRoute(router, "POST /api/v1/monitor/alerts/{alert_id}/resolve", handler.ResolveAlert)
	registerRoute(router, "GET /api/v1/audit-logs", auditHandler.GetAuditLogs)
	registerRoute(router, "GET /api/v1/jobs/{job_id}", handler.GetJobStatus)
	registerRoute(router, "GET /api/v1/monitor/jobs/{job_id}", handler.GetMonitorJob)
}
