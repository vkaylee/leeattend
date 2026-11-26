package api

import (
	"context"
	"encoding/json"
	"net/http"
	"strconv"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type PaginatedAuditLogs struct {
	Data  []AuditLogRecord `json:"data"`
	Total int64            `json:"total"`
	Page  int64            `json:"page"`
	Limit int64            `json:"limit"`
}

type AuditLogRecord struct {
	ID               uuid.UUID       `json:"id"`
	ActorID          *uuid.UUID      `json:"actor_id"`
	ActorType        *string         `json:"actor_type"`
	ActorName        *string         `json:"actor_name"`
	ActorEmail       *string         `json:"actor_email"`
	TargetUserID     *uuid.UUID      `json:"target_user_id"`
	TargetResourceID *uuid.UUID      `json:"target_resource_id"`
	Action           string          `json:"action"`
	Details          json.RawMessage `json:"details"`
	IPAddress        *string         `json:"ip_address"`
	UserAgent        *string         `json:"user_agent"`
	CreatedAt        time.Time       `json:"created_at"`
}

// AuditLogRepository reads organization audit logs.
type AuditLogRepository interface {
	ListAuditLogs(ctx context.Context, limit, offset int, action *string, startDate, endDate *time.Time) ([]AuditLogRecord, int64, error)
}

// AuditLogHandler serves GET /api/v1/audit-logs independently from infra monitoring.
type AuditLogHandler struct {
	Repo       AuditLogRepository
	Authorizer Authorizer
}

func (h *AuditLogHandler) requireAuditPermission(r *http.Request) (uuid.UUID, *apperror.AppError) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return uuid.Nil, apperror.Unauthorized(apperror.KeyUnauthorized)
	}
	if h.Authorizer == nil {
		return uuid.Nil, apperror.Forbidden(apperror.KeyForbidden)
	}
	isMember, err := h.Authorizer.IsMember(r.Context(), userID)
	if err != nil || !isMember {
		return uuid.Nil, apperror.Forbidden(apperror.KeyForbidden)
	}
	if !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionAuditLogsRead)) {
		return uuid.Nil, apperror.Forbidden(apperror.KeyForbidden)
	}
	return userID, nil
}

// GetAuditLogs serves GET /api/v1/audit-logs.
func (h *AuditLogHandler) GetAuditLogs(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireAuditPermission(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	q := r.URL.Query()
	page := 1
	if parsed, err := strconv.Atoi(q.Get("page")); err == nil && parsed > 0 {
		page = parsed
	}
	limit := parseMonitorLimit(q.Get("limit"), 50, 100)
	actionPtr := optionalTrimmedParam(q.Get("action"))
	startDate := optionalRFC3339Param(q.Get("start_date"))
	endDate := optionalRFC3339Param(q.Get("end_date"))
	offset := (page - 1) * limit

	if h.Repo == nil {
		apperror.WriteJSON(w, http.StatusOK, PaginatedAuditLogs{
			Data:  make([]AuditLogRecord, 0),
			Page:  int64(page),
			Limit: int64(limit),
		})
		return
	}

	logs, total, err := h.Repo.ListAuditLogs(r.Context(), limit, offset, actionPtr, startDate, endDate)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, PaginatedAuditLogs{
		Data:  logs,
		Total: total,
		Page:  int64(page),
		Limit: int64(limit),
	})
}

// GetAuditLogs delegates to AuditLogHandler for route/test compatibility.
func (h *MonitorHandler) GetAuditLogs(w http.ResponseWriter, r *http.Request) {
	(&AuditLogHandler{Repo: h.Repo, Authorizer: h.Authorizer}).GetAuditLogs(w, r)
}
