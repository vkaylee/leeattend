package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerSystemRoutes(router chi.Router, healthHandler api.SystemHealthHandler, adminHandler api.SystemAdminHandler) {
	registerRoute(router, "GET /api/v1/system/health", healthHandler.Get)
	registerRoute(router, "GET /api/v1/system/smtp/config", adminHandler.GetSMTPConfig)
	registerRoute(router, "PUT /api/v1/system/smtp/config", adminHandler.SetSMTPConfig)
	registerRoute(router, "POST /api/v1/system/smtp/test", adminHandler.TestSMTP)
}
