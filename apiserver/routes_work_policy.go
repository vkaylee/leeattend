package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerWorkPolicyRoutes(router chi.Router, handler api.WorkPolicyGroupHandler) {
	registerRoute(router, "GET /api/v1/payroll/work-policy-groups", handler.List)
	registerRoute(router, "GET /api/v1/payroll/work-policy-groups/{group_id}/history", handler.History)
	registerRoute(router, "GET /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}", handler.Detail)
	registerRoute(router, "GET /api/v1/employments/{employment_id}/work-policy", handler.Resolve)
	registerRoute(router, "POST /api/v1/payroll/work-policy-groups", handler.Create)
	registerRoute(router, "POST /api/v1/payroll/work-policy-groups/{group_id}/versions", handler.CreateVersion)
	registerRoute(router, "GET /api/v1/payroll/work-policy-groups/{group_id}/memberships", handler.ListMemberships)
	registerRoute(router, "POST /api/v1/payroll/work-policy-groups/{group_id}/memberships", handler.CreateMembership)
}
