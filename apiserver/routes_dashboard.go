package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerDashboardRoutes(router chi.Router, preferences api.DashboardPreferencesHandler, analytics api.DashboardAnalyticsHandler) {
	registerRoute(router, "GET /api/v1/dashboard/preferences", preferences.Get)
	registerRoute(router, "PUT /api/v1/dashboard/preferences", preferences.Put)
	registerRoute(router, "GET /api/v1/dashboard/executive-overview", analytics.ExecutiveOverview)
	registerRoute(router, "GET /api/v1/dashboard/overtime-breakdown", analytics.OvertimeBreakdown)
	registerRoute(router, "GET /api/v1/dashboard/organization-stats", analytics.OrganizationStats)
	registerRoute(router, "GET /api/v1/dashboard/attendance-trend", analytics.AttendanceTrend)
	registerRoute(router, "GET /api/v1/dashboard/department-attendance", analytics.DepartmentAttendance)
	registerRoute(router, "GET /api/v1/dashboard/unexcused-absences", analytics.UnexcusedAbsences)
	registerRoute(router, "GET /api/v1/dashboard/overtime-trend", analytics.OvertimeTrend)
	registerRoute(router, "GET /api/v1/dashboard/peak-times", analytics.PeakTimes)
	registerRoute(router, "GET /api/v1/dashboard/leave-types", analytics.LeaveTypes)
	registerRoute(router, "GET /api/v1/dashboard/lateness-by-day", analytics.LatenessByDay)
	registerRoute(router, "GET /api/v1/dashboard/turnover-analytics", analytics.TurnoverAnalytics)
	registerRoute(router, "GET /api/v1/dashboard/recruitment-effectiveness", analytics.RecruitmentEffectiveness)
}
