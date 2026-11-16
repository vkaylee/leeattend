package api

import (
	"context"
	"errors"
	"net/http"
	"strconv"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var ErrNotificationNotFound = errors.New("notification: not found")

type NotificationRecord struct {
	ID        uuid.UUID `json:"id"`
	UserID    uuid.UUID `json:"user_id"`
	Type      string    `json:"type"`
	Title     string    `json:"title"`
	Content   string    `json:"content"`
	IsRead    bool      `json:"is_read"`
	Link      *string   `json:"link"`
	CreatedAt string    `json:"created_at"`
}

type NotificationResponse struct {
	Notifications []NotificationRecord `json:"notifications"`
	NextCursor    *string              `json:"next_cursor"`
	UnreadCount   int64                `json:"unread_count"`
}

type UnreadCountResponse struct {
	Count int64 `json:"count"`
}

type NotificationRepository interface {
	List(ctx context.Context, userID uuid.UUID, limit int, cursor *time.Time) (NotificationResponse, error)
	MarkRead(ctx context.Context, userID, notificationID uuid.UUID) error
	MarkAllRead(ctx context.Context, userID uuid.UUID) error
}

type NotificationHandler struct {
	Repository NotificationRepository
}

func (h *NotificationHandler) List(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	limit := 20
	if raw := r.URL.Query().Get("limit"); raw != "" {
		parsed, err := strconv.Atoi(raw)
		if err != nil || parsed < 1 {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		if parsed > 100 {
			parsed = 100
		}
		limit = parsed
	}
	var cursor *time.Time
	if raw := r.URL.Query().Get("cursor"); raw != "" {
		parsed, err := time.Parse(time.RFC3339Nano, raw)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		cursor = &parsed
	}
	result, err := h.Repository.List(r.Context(), userID, limit, cursor)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *NotificationHandler) UnreadCount(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	result, err := h.Repository.List(r.Context(), userID, 1, nil)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, UnreadCountResponse{Count: result.UnreadCount})
}

func (h *NotificationHandler) MarkRead(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	notificationID, idOK := pathUUID(r, "id")
	if !ok || !idOK {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	if err := h.Repository.MarkRead(r.Context(), userID, notificationID); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, struct{}{})
}

func (h *NotificationHandler) MarkAllRead(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	if err := h.Repository.MarkAllRead(r.Context(), userID); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, struct{}{})
}

func (h *NotificationHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrNotificationNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
