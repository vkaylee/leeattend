package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerWorkflowRoutes(router chi.Router, handler *api.ApprovalWorkflowHandler) {
	registerRoute(router, "GET /api/v1/approval-workflows", handler.List)
	registerRoute(router, "POST /api/v1/approval-workflows", handler.Create)
	registerRoute(router, "POST /api/v1/approval-workflows/approve", handler.Process)
	registerRoute(router, "GET /api/v1/approval-workflows/history/{type}/{id}", handler.History)
	registerRoute(router, "GET /api/v1/approval-workflows/{id}", handler.Get)
	registerRoute(router, "PUT /api/v1/approval-workflows/{id}", handler.Update)
	registerRoute(router, "DELETE /api/v1/approval-workflows/{id}", handler.Delete)
	registerRoute(router, "GET /api/v1/approval-workflows/pending-tasks", handler.PendingTasks)
}

func registerNotificationRoutes(router chi.Router, handler *api.NotificationHandler) {
	registerRoute(router, "GET /api/v1/notifications", handler.List)
	registerRoute(router, "GET /api/v1/notifications/unread-count", handler.UnreadCount)
	registerRoute(router, "PATCH /api/v1/notifications/{id}/read", handler.MarkRead)
	registerRoute(router, "POST /api/v1/notifications/read-all", handler.MarkAllRead)
}
