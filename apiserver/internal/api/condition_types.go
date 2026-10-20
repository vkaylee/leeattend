package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"regexp"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrConditionTypeNotFound = errors.New("condition_type: not found")
	ErrConditionTypeConflict = errors.New("condition_type: code exists")
	ErrConditionTypeSystem   = errors.New("condition_type: system type cannot be deleted")
	codeRegex                = regexp.MustCompile(`^[A-Z][A-Z0-9_]*$`)
)

type ConditionTypeRecord struct {
	ID                     uuid.UUID       `json:"id"`
	WorkspaceID            uuid.UUID       `json:"workspace_id"`
	Code                   string          `json:"code"`
	NameI18N               json.RawMessage `json:"name_i18n"`
	Category               string          `json:"category"`
	DefaultMinuteAllowance int             `json:"default_minute_allowance"`
	DefaultIsPaid          bool            `json:"default_is_paid"`
	AffectsRules           json.RawMessage `json:"affects_rules"`
	Icon                   *string         `json:"icon"`
	Color                  *string         `json:"color"`
	IsSystem               bool            `json:"is_system"`
	SortOrder              int             `json:"sort_order"`
	IsActive               bool            `json:"is_active"`
	CreatedAt              string          `json:"created_at"`
	UpdatedAt              string          `json:"updated_at"`
}

type CreateConditionTypeRequest struct {
	Code                   string          `json:"code"`
	NameI18N               json.RawMessage `json:"name_i18n"`
	Category               string          `json:"category"`
	DefaultMinuteAllowance int             `json:"default_minute_allowance"`
	DefaultIsPaid          bool            `json:"default_is_paid"`
	AffectsRules           json.RawMessage `json:"affects_rules"`
	Icon                   *string         `json:"icon"`
	Color                  *string         `json:"color"`
	SortOrder              int             `json:"sort_order"`
}

type UpdateConditionTypeRequest struct {
	NameI18N               *json.RawMessage `json:"name_i18n,omitempty"`
	Category               *string          `json:"category,omitempty"`
	DefaultMinuteAllowance *int             `json:"default_minute_allowance,omitempty"`
	DefaultIsPaid          *bool            `json:"default_is_paid,omitempty"`
	AffectsRules           *json.RawMessage `json:"affects_rules,omitempty"`
	Icon                   *string          `json:"icon,omitempty"`
	Color                  *string          `json:"color,omitempty"`
	SortOrder              *int             `json:"sort_order,omitempty"`
	IsActive               *bool            `json:"is_active,omitempty"`
}

type ConditionTypeRepository interface {
	List(ctx context.Context, wsID uuid.UUID, activeOnly bool) ([]ConditionTypeRecord, error)
	Get(ctx context.Context, wsID, id uuid.UUID) (ConditionTypeRecord, error)
	Create(ctx context.Context, record ConditionTypeRecord) (ConditionTypeRecord, error)
	Update(ctx context.Context, wsID, id uuid.UUID, req UpdateConditionTypeRequest) (ConditionTypeRecord, error)
	Delete(ctx context.Context, wsID, id uuid.UUID) error
}

type ConditionTypeHandler struct {
	Repository ConditionTypeRepository
	Authorizer WorkspaceAuthorizer
}

func (h ConditionTypeHandler) List(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, wsID, "workspace:settings:read") {
		h.forbidden(w, r)
		return
	}
	activeOnly := r.URL.Query().Get("active_only") == "true"
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []ConditionTypeRecord{})
		return
	}
	records, err := h.Repository.List(r.Context(), wsID, activeOnly)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, records)
}

func (h ConditionTypeHandler) Get(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "workspace:settings:read") {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	record, err := h.Repository.Get(r.Context(), wsID, id)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, record)
}

func (h ConditionTypeHandler) Create(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, wsID, "workspace:settings:write") {
		h.forbidden(w, r)
		return
	}
	var req CreateConditionTypeRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil {
		h.validation(w, r)
		return
	}
	req.Code = strings.TrimSpace(req.Code)
	if !codeRegex.MatchString(req.Code) || len(req.NameI18N) == 0 {
		h.validation(w, r)
		return
	}
	if req.Category == "" {
		req.Category = "general"
	}
	if len(req.AffectsRules) == 0 {
		req.AffectsRules = json.RawMessage("{}")
	}
	now := time.Now().UTC()
	record := ConditionTypeRecord{
		ID:                     uuid.New(),
		WorkspaceID:            wsID,
		Code:                   req.Code,
		NameI18N:               req.NameI18N,
		Category:               req.Category,
		DefaultMinuteAllowance: req.DefaultMinuteAllowance,
		DefaultIsPaid:          req.DefaultIsPaid,
		AffectsRules:           req.AffectsRules,
		Icon:                   req.Icon,
		Color:                  req.Color,
		IsSystem:               false,
		SortOrder:              req.SortOrder,
		IsActive:               true,
		CreatedAt:              now.Format(time.RFC3339),
		UpdatedAt:              now.Format(time.RFC3339),
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("repository unavailable")))
		return
	}
	created, err := h.Repository.Create(r.Context(), record)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, created)
}

func (h ConditionTypeHandler) Update(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "workspace:settings:write") {
		h.forbidden(w, r)
		return
	}
	var req UpdateConditionTypeRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil {
		h.validation(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	updated, err := h.Repository.Update(r.Context(), wsID, id, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, updated)
}

func (h ConditionTypeHandler) Delete(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "workspace:settings:write") {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if err := h.Repository.Delete(r.Context(), wsID, id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h ConditionTypeHandler) allowed(r *http.Request, wsID uuid.UUID, p string) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), u, wsID, p)
}

func (h ConditionTypeHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h ConditionTypeHandler) validation(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
}

func (h ConditionTypeHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrConditionTypeNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrConditionTypeConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	if errors.Is(err, ErrConditionTypeSystem) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
