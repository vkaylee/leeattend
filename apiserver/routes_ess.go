package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerESSRoutes(router chi.Router, handler *api.ESSHandler) {
	registerRoute(router, "GET /api/v1/ess/profile", handler.GetProfile)
	registerRoute(router, "PUT /api/v1/ess/profile", handler.UpdateProfile)
	registerRoute(router, "GET /api/v1/ess/emergency-contacts", handler.GetContacts)
	registerRoute(router, "POST /api/v1/ess/emergency-contacts", handler.AddContact)
	registerRoute(router, "DELETE /api/v1/ess/emergency-contacts/{contact_id}", handler.DeleteContact)
	registerRoute(router, "GET /api/v1/ess/documents/pending", handler.GetPendingDocuments)
	registerRoute(router, "POST /api/v1/ess/documents/{id}/acknowledge", handler.AcknowledgeDocument)
	registerRoute(router, "GET /api/v1/ess/documents", handler.GetDocuments)
	registerRoute(router, "GET /api/v1/ess/payslips", handler.GetPayslips)
	registerRoute(router, "GET /api/v1/ess/attendance", handler.GetAttendance)
	registerRoute(router, "POST /api/v1/ess/attendance/actions", handler.AttendanceAction)
	registerRoute(router, "GET /api/v1/ess/attendance/state", handler.AttendanceState)
	registerRoute(router, "GET /api/v1/ess/time-entries", handler.GetTimeEntries)
	registerRoute(router, "GET /api/v1/ess/leave/balances", handler.GetLeaveBalances)
	registerRoute(router, "GET /api/v1/ess/leave/requests", handler.GetLeaveRequests)
	registerRoute(router, "GET /api/v1/ess/shift-swaps", handler.GetShiftSwaps)
	registerRoute(router, "POST /api/v1/ess/shift-swaps", handler.CreateShiftSwap)
	registerRoute(router, "PUT /api/v1/ess/shift-swaps/{id}/respond", handler.RespondShiftSwap)
	registerRoute(router, "DELETE /api/v1/ess/shift-swaps/{id}", handler.CancelShiftSwap)
}
