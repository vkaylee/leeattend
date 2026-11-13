package main

import (
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerOrganizationSettingsRoutes(router chi.Router, get, patch http.HandlerFunc) {
	registerRoute(router, "PATCH /api/v1", patch)
	registerRoute(router, "GET /api/v1", get)
}

func registerOrganizationRoutes(router chi.Router, organization api.OrganizationHandler, rbac api.OrganizationRBACHandler) {
	registerOrganizationSettingsRoutes(router, organization.Get, organization.Patch)
	registerRoute(router, "GET /api/v1/feature-flags", organization.ListFeatureFlags)
	registerRoute(router, "GET /api/v1/feature-flags/effective", organization.ListEffectiveFeatureFlags)
	registerRoute(router, "GET /api/v1/members", rbac.ListMembers)
	registerRoute(router, "POST /api/v1/members", rbac.AddMember)
	registerRoute(router, "PATCH /api/v1/members/{user_id}", rbac.UpdateMemberRole)
	registerRoute(router, "DELETE /api/v1/members/{user_id}", rbac.RemoveMember)
	registerRoute(router, "GET /api/v1/members/{user_id}/roles", rbac.GetMemberRoles)
	registerRoute(router, "POST /api/v1/members/{user_id}/roles", rbac.AssignExtraRole)
	registerRoute(router, "DELETE /api/v1/members/{user_id}/roles/{role_id}", rbac.RevokeExtraRole)
	registerRoute(router, "GET /api/v1/members/{user_id}/permissions", rbac.GetMemberPermissions)
	registerRoute(router, "POST /api/v1/members/{user_id}/permissions", rbac.AssignExtraPermission)
	registerRoute(router, "DELETE /api/v1/members/{user_id}/permissions/{perm_id}", rbac.RevokeExtraPermission)
	registerRoute(router, "GET /api/v1/roles", rbac.ListRoles)
	registerRoute(router, "POST /api/v1/roles", rbac.CreateRole)
	registerRoute(router, "PUT /api/v1/roles/{role_id}", rbac.UpdateRole)
	registerRoute(router, "DELETE /api/v1/roles/{role_id}", rbac.DeleteRole)
	registerRoute(router, "GET /api/v1/roles/{role_id}/permissions", rbac.GetRolePermissions)
	registerRoute(router, "PUT /api/v1/roles/{role_id}/permissions", rbac.SetRolePermissions)
	registerRoute(router, "GET /api/v1/permissions/catalog", rbac.PermissionCatalog)
	registerRoute(router, "GET /api/v1/permissions/available", rbac.AvailablePermissions)
	registerRoute(router, "GET /api/v1/settings/storage", rbac.GetStorage)
	registerRoute(router, "PUT /api/v1/settings/storage", rbac.PutStorage)
	registerRoute(router, "DELETE /api/v1/settings/storage", rbac.DeleteStorage)
	registerRoute(router, "POST /api/v1/settings/storage/test", rbac.TestStorage)
	registerRoute(router, "POST /api/v1/feature-flags", organization.CreateFeatureFlag)
	registerRoute(router, "PUT /api/v1/feature-flags", organization.UpsertFeatureFlag)
	registerRoute(router, "GET /api/v1/feature-flags/{id}", organization.GetFeatureFlagByID)
	registerRoute(router, "PATCH /api/v1/feature-flags/{id}", organization.UpdateFeatureFlag)
	registerRoute(router, "DELETE /api/v1/feature-flags/{id}", organization.DeleteFeatureFlag)
	registerRoute(router, "GET /api/v1/feature-flags/check/{key}", organization.CheckFeatureFlag)
	registerRoute(router, "PUT /api/v1/configs", organization.PutConfig)
	registerRoute(router, "GET /api/v1/configs", organization.ListConfigs)
	registerRoute(router, "GET /api/v1/configs/{key}", organization.GetConfig)
	registerRoute(router, "DELETE /api/v1/configs/{key}", organization.DeleteConfig)
}
