package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerSchedulingRoutes(
	router chi.Router,
	shift api.ShiftHandler,
	pattern api.WorkingPatternHandler,
	roster api.RosterHandler,
	schedule api.ScheduleHandler,
	swap api.ShiftSwapHandler,
	segment api.WorkSegmentHandler,
) {
	registerRoute(router, "POST /api/v1/shifts", shift.Create)
	registerRoute(router, "GET /api/v1/shifts", shift.List)
	registerRoute(router, "PUT /api/v1/shifts/{shift_id}", shift.Update)
	registerRoute(router, "GET /api/v1/shifts/{shift_id}/versions", shift.Versions)
	registerRoute(router, "POST /api/v1/shifts/assign", shift.Assign)
	registerRoute(router, "POST /api/v1/working-patterns", pattern.Create)
	registerRoute(router, "GET /api/v1/working-patterns", pattern.List)
	registerRoute(router, "GET /api/v1/working-patterns/{pattern_id}", pattern.Details)
	registerRoute(router, "PUT /api/v1/working-patterns/{pattern_id}", pattern.Update)
	registerRoute(router, "DELETE /api/v1/working-patterns/{pattern_id}", pattern.Delete)
	registerRoute(router, "POST /api/v1/employments/{employment_id}/working-patterns", pattern.Assign)
	registerRoute(router, "POST /api/v1/working-patterns/bulk-assign", pattern.BulkAssign)
	registerRoute(router, "GET /api/v1/employments/{employment_id}/working-patterns", pattern.History)
	registerRoute(router, "GET /api/v1/employees/{id}/rosters", roster.ListEmployee)
	registerRoute(router, "POST /api/v1/employees/{id}/rosters", roster.Upsert)
	registerRoute(router, "POST /api/v1/employees/{id}/rosters/bulk", roster.BulkUpsert)
	registerRoute(router, "DELETE /api/v1/employees/{id}/rosters/{date}", roster.Delete)
	registerRoute(router, "GET /api/v1/schedules", schedule.List)
	registerRoute(router, "POST /api/v1/schedules/activation/{employer_id}", schedule.Activate)
	registerRoute(router, "POST /api/v1/schedules/deactivation/{employer_id}", schedule.Deactivate)
	registerRoute(router, "POST /api/v1/schedules/preview", schedule.Preview)
	registerRoute(router, "POST /api/v1/schedules/publish", schedule.Publish)
	registerRoute(router, "POST /api/v1/schedules/{employment_id}/{business_date}/replan/preview", schedule.PreviewReplan)
	registerRoute(router, "POST /api/v1/schedules/{employment_id}/{business_date}/replan", schedule.CommitReplan)
	registerRoute(router, "GET /api/v1/schedules/history", schedule.History)
	registerRoute(router, "GET /api/v1/schedules/shadow-mismatches", schedule.ListShadowMismatches)
	registerRoute(router, "POST /api/v1/schedules/shadow-mismatches/{mismatch_id}/resolve", schedule.ResolveShadowMismatch)
	registerRoute(router, "GET /api/v1/schedules/activation-readiness", schedule.ActivationReadiness)
	registerRoute(router, "GET /api/v1/schedules/evidence/{employment_id}/{business_date}", schedule.Evidence)
	registerRoute(router, "POST /api/v1/shift-swaps", swap.Create)
	registerRoute(router, "GET /api/v1/shift-swaps", swap.List)
	registerRoute(router, "PUT /api/v1/shift-swaps/{id}/approve", swap.Approve)
	registerRoute(router, "GET /api/v1/shift-swaps/stats", swap.Stats)
	registerRoute(router, "GET /api/v1/work-segments", segment.List)
	registerRoute(router, "GET /api/v1/work-segments/{segment_id}", segment.Get)
	registerRoute(router, "POST /api/v1/work-segments", segment.Create)
	registerRoute(router, "PUT /api/v1/work-segments/{segment_id}", segment.Update)
	registerRoute(router, "DELETE /api/v1/work-segments/{segment_id}", segment.Delete)
	registerRoute(router, "POST /api/v1/work-segments/bulk", segment.BulkCreate)
}
