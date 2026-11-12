package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/health"
)

func registerPublicRoutes(router chi.Router, healthHandler *health.Handler) {
	router.Get("/api/v1/health", healthHandler.Liveness)
	router.Get("/api/v1/health/ready", healthHandler.Readiness)
}
