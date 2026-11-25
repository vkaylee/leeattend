package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerWorkPolicyRoutes(router chi.Router, handler *api.WorkPolicyGroupHandler) {
	registerRoute(router, "GET /api/v1/payroll/work-policy-groups", handler.List)
	registerRoute(router, "POST /api/v1/payroll/work-policy-groups", handler.Create)
	registerRoute(router, "GET /api/v1/payroll/work-policy-groups/{group_id}/versions", handler.ListVersions)
	registerRoute(router, "POST /api/v1/payroll/work-policy-groups/{group_id}/versions", handler.CreateVersion)
	registerRoute(router, "GET /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}", handler.Detail)
	registerRoute(router, "PATCH /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}", handler.PatchVersion)
	registerRoute(router, "POST /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/validate", handler.Validate)
	registerRoute(router, "POST /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/publish", handler.PublishVersion)
	registerRoute(router, "GET /api/v1/payroll/work-policy-groups/{group_id}/memberships", handler.ListMemberships)
	registerRoute(router, "POST /api/v1/payroll/work-policy-groups/{group_id}/memberships", handler.CreateMembership)
	registerRoute(router, "POST /api/v1/payroll/work-policy-memberships/transfer", handler.TransferMembership)
	registerRoute(router, "GET /api/v1/employment-relationships/{relationship_id}/work-policy", handler.Resolve)
}
