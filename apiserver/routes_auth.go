package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerPublicAuthRoutes(router chi.Router, handler *api.AuthHandler) {
	registerRoute(router, "POST /api/v1/auth/login", handler.Login)
	registerRoute(router, "POST /api/v1/auth/refresh", handler.Refresh)
	registerRoute(router, "POST /api/v1/auth/forgot-password", handler.ForgotPassword)
	registerRoute(router, "POST /api/v1/auth/reset-password", handler.ResetPassword)
}

func registerAuthRoutes(router chi.Router, handler *api.AuthHandler) {
	registerRoute(router, "POST /api/v1/auth/logout", handler.Logout)
	registerRoute(router, "POST /api/v1/auth/change-password", handler.ChangePassword)
	registerRoute(router, "GET /api/v1/me", handler.Me)
	registerRoute(router, "PATCH /api/v1/me", handler.UpdateMe)
	registerRoute(router, "GET /api/v1/me/permissions", handler.MyPermissions)
	registerRoute(router, "GET /api/v1/users", handler.OrganizationUsers)
}
