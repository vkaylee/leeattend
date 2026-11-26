package api

import (
	"context"
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

type MonitorAlertResponse struct {
	ID              uuid.UUID  `json:"id"`
	AlertType       string     `json:"alert_type"`
	Severity        string     `json:"severity"`
	Status          string     `json:"status"`
	Title           string     `json:"title"`
	Summary         string     `json:"summary"`
	Source          string     `json:"source"`
	FirstSeenAt     time.Time  `json:"first_seen_at"`
	LastSeenAt      time.Time  `json:"last_seen_at"`
	OccurrenceCount int32      `json:"occurrence_count"`
	ResolvedAt      *time.Time `json:"resolved_at"`
}

type MonitorAlertsResponse struct {
	Data       []MonitorAlertResponse `json:"data"`
	Total      int64                  `json:"total"`
	Page       int64                  `json:"page"`
	Limit      int64                  `json:"limit"`
	NextCursor *string                `json:"next_cursor"`
}

type MonitorAlertEventResponse struct {
	ID         uuid.UUID `json:"id"`
	EventType  string    `json:"event_type"`
	Severity   string    `json:"severity"`
	Summary    string    `json:"summary"`
	OccurredAt time.Time `json:"occurred_at"`
}

type MonitorAlertDetailResponse struct {
	MonitorAlertResponse
	Events []MonitorAlertEventResponse `json:"events"`
}

type MonitorAlertMutationRequest struct {
	Reason string `json:"reason"`
}

// MonitorAlertRepository manages monitor alert queries and state mutations.
type MonitorAlertRepository interface {
	ListAlerts(ctx context.Context, status, severity *string, limit int, cursor *string) ([]MonitorAlertResponse, int64, *string, error)
	GetAlert(ctx context.Context, alertID uuid.UUID) (*MonitorAlertResponse, []MonitorAlertEventResponse, error)
	MutateAlert(ctx context.Context, alertID uuid.UUID, action string, reason string, actorID uuid.UUID) (*MonitorAlertResponse, error)
}

func (h *MonitorHandler) ListAlerts(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireInfraMonitor(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	q := r.URL.Query()
	limit := 20
	if l := q.Get("limit"); l != "" {
		if parsed, err := strconv.Atoi(l); err == nil && parsed > 0 {
			if parsed > 100 {
				parsed = 100
			}
			limit = parsed
		}
	}

	var statusPtr *string
	if s := strings.TrimSpace(q.Get("status")); s != "" {
		statusPtr = &s
	}
	var severityPtr *string
	if s := strings.TrimSpace(q.Get("severity")); s != "" {
		severityPtr = &s
	}
	var cursorPtr *string
	if c := strings.TrimSpace(q.Get("cursor")); c != "" {
		cursorPtr = &c
	}

	if h.Repo == nil {
		apperror.WriteJSON(w, http.StatusOK, MonitorAlertsResponse{
			Data:       make([]MonitorAlertResponse, 0),
			Total:      0,
			Page:       1,
			Limit:      int64(limit),
			NextCursor: nil,
		})
		return
	}

	data, total, nextCursor, err := h.Repo.ListAlerts(r.Context(), statusPtr, severityPtr, limit, cursorPtr)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, MonitorAlertsResponse{
		Data:       data,
		Total:      total,
		Page:       1,
		Limit:      int64(limit),
		NextCursor: nextCursor,
	})
}

// 6. GET /api/v1/monitor/alerts/{alert_id}
func (h *MonitorHandler) GetAlert(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireInfraMonitor(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	idStr := r.PathValue("alert_id")
	alertID, err := uuid.Parse(idStr)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	if h.Repo == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}

	alert, events, err := h.Repo.GetAlert(r.Context(), alertID)
	if err != nil {
		if errors.Is(err, ErrMonitorNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if alert == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, MonitorAlertDetailResponse{
		MonitorAlertResponse: *alert,
		Events:               events,
	})
}

// 7. POST /api/v1/monitor/alerts/{alert_id}/acknowledge
func (h *MonitorHandler) AcknowledgeAlert(w http.ResponseWriter, r *http.Request) {
	h.mutateAlert(w, r, "acknowledge")
}

// 8. POST /api/v1/monitor/alerts/{alert_id}/resolve
func (h *MonitorHandler) ResolveAlert(w http.ResponseWriter, r *http.Request) {
	h.mutateAlert(w, r, "resolve")
}

func (h *MonitorHandler) mutateAlert(w http.ResponseWriter, r *http.Request, action string) {
	userID, appErr := h.requireInfraMonitor(r)
	if appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	alertID, err := uuid.Parse(r.PathValue("alert_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	req, err := DecodeJSON[MonitorAlertMutationRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	reason := strings.TrimSpace(req.Reason)
	if reason == "" || len([]rune(reason)) > 500 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	reason = sanitizeSummary(reason)

	if h.Repo == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}

	alert, err := h.Repo.MutateAlert(r.Context(), alertID, action, reason, userID)
	if err != nil {
		writeMappedError(w, r, err, func(err error) *apperror.AppError {
			switch {
			case errors.Is(err, ErrMonitorNotFound):
				return apperror.NotFound(apperror.KeyNotFound)
			case errors.Is(err, ErrMonitorConflict):
				return apperror.Conflict(apperror.KeyConflict)
			default:
				return nil
			}
		})
		return
	}

	apperror.WriteJSON(w, http.StatusOK, alert)
}
