package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerAttendanceRoutes(router chi.Router, attendance *api.AttendanceHandler, logs *api.AttendanceLogsHandler, policy *api.AttendancePolicyHandler) {
	registerRoute(router, "POST /api/v1/time/punch-in", attendance.PunchIn)
	registerRoute(router, "POST /api/v1/time/punch-out/{employee_id}", attendance.PunchOut)
	registerRoute(router, "GET /api/v1/time/history/{employee_id}", attendance.TimeHistory)
	registerRoute(router, "POST /api/v1/attendance/adjustments", attendance.CreateAdjustment)
	registerRoute(router, "GET /api/v1/attendance/adjustments", attendance.ListAdjustments)
	registerRoute(router, "PUT /api/v1/attendance/adjustments/{id}/approve", attendance.ApproveAdjustment)
	registerRoute(router, "GET /api/v1/employees/{id}/attendance/adjustments", attendance.ListEmployeeAdjustments)
	registerRoute(router, "GET /api/v1/attendance/summaries", attendance.Summaries)
	registerRoute(router, "POST /api/v1/attendance/reprocess-all", attendance.ReprocessAll)
	registerRoute(router, "POST /api/v1/attendance/reprocess/{employee_id}", attendance.ReprocessEmployee)
	registerRoute(router, "PUT /api/v1/time/entries/{id}", attendance.UpdateTimeEntry)
	registerRoute(router, "DELETE /api/v1/time/entries/{id}", attendance.DeleteTimeEntry)
	registerRoute(router, "POST /api/v1/attendance/upload", logs.Upload)
	registerRoute(router, "POST /api/v1/attendance/logs/upload", logs.UploadLogs)
	registerRoute(router, "POST /api/v1/attendance/bulk-upload-paper-ot", logs.BulkPaperOT)
	registerRoute(router, "POST /api/v1/attendance/paper-forms/overtime/bulk", logs.BulkPaperOT)
	registerRoute(router, "POST /api/v1/attendance/enter-paper-ot/{employee_id}", logs.EnterPaperOT)
	registerRoute(router, "POST /api/v1/attendance/paper-forms/overtime/{employee_id}", logs.EnterPaperOT)
	registerRoute(router, "GET /api/v1/attendance/import-jobs/latest", logs.Latest)
	registerRoute(router, "GET /api/v1/attendance/import-jobs/{id}", logs.Job)
	registerRoute(router, "GET /api/v1/attendance/import-jobs/{id}/error-file", logs.ErrorFile)
	registerRoute(router, "GET /api/v1/attendance/logs", logs.Logs)
	registerRoute(router, "GET /api/v1/attendance/raw-logs", logs.RawLogs)
	registerRoute(router, "GET /api/v1/attendance/staging-logs", logs.Staging)
	registerRoute(router, "POST /api/v1/attendance/staging-logs/{id}/resolve", logs.Resolve)
	registerRoute(router, "DELETE /api/v1/attendance/staging-logs/{id}", logs.Dismiss)
	registerRoute(router, "GET /api/v1/attendance/adjustments/behalf-assignments", policy.ListBehalfAssignments)
	registerRoute(router, "POST /api/v1/attendance/adjustments/behalf-assignments", policy.CreateBehalfAssignment)
	registerRoute(router, "DELETE /api/v1/attendance/adjustments/behalf-assignments/{id}", policy.DeleteBehalfAssignment)
	registerRoute(router, "GET /api/v1/attendance/adjustments/behalf-assignments/my-targets", policy.MyBehalfTargets)
	registerRoute(router, "GET /api/v1/attendance/periods/{year}/{month}", attendance.GetAttendancePeriodStatus)
	registerRoute(router, "POST /api/v1/attendance/periods/{year}/{month}/close", attendance.CloseAttendancePeriod)
	registerRoute(router, "POST /api/v1/attendance/periods/{year}/{month}/reopen", attendance.ReopenAttendancePeriod)
}
