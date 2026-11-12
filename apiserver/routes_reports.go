package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerReportRoutes(router chi.Router, handler api.ReportHandler) {
	registerRoute(router, "GET /api/v1/reports/attendance", handler.AttendanceReport)
	registerRoute(router, "GET /api/v1/reports/no-show", handler.NoShowReport)
	registerRoute(router, "POST /api/v1/reports/export", handler.CreateExport)
	registerRoute(router, "GET /api/v1/reports/jobs/{job_id}", handler.Status)
	registerRoute(router, "DELETE /api/v1/reports/jobs/{job_id}", handler.Cancel)
	registerRoute(router, "GET /api/v1/reports/jobs/{job_id}/download", handler.Download)
}
