package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrCustomFieldNotFound = errors.New("custom_field: not found")
	ErrCustomFieldConflict = errors.New("custom_field: conflict")
)

type CustomFieldDefinitionRecord struct {
	ID                  uuid.UUID       `json:"id"`
	WorkspaceID         uuid.UUID       `json:"workspace_id"`
	EntityType          string          `json:"entity_type"`
	FieldKey            string          `json:"field_key"`
	FieldLabel          string          `json:"field_label"`
	FieldType           string          `json:"field_type"`
	Options             json.RawMessage `json:"options,omitempty"`
	DefaultValue        *string         `json:"default_value,omitempty"`
	IsRequired          bool            `json:"is_required"`
	IsVisibleToEmployee bool            `json:"is_visible_to_employee"`
	IsFilterable        bool            `json:"is_filterable"`
	IsExportable        bool            `json:"is_exportable"`
	DisplayOrder        int             `json:"display_order"`
	SectionName         *string         `json:"section_name,omitempty"`
	ValidationRules     json.RawMessage `json:"validation_rules,omitempty"`
	CreatedAt           string          `json:"created_at"`
	UpdatedAt           string          `json:"updated_at"`
}

type CreateCustomFieldDefinitionRequest struct {
	EntityType          string          `json:"entity_type"`
	FieldKey            string          `json:"field_key"`
	FieldLabel          string          `json:"field_label"`
	FieldType           string          `json:"field_type"`
	Options             json.RawMessage `json:"options,omitempty"`
	DefaultValue        *string         `json:"default_value,omitempty"`
	IsRequired          bool            `json:"is_required"`
	IsVisibleToEmployee bool            `json:"is_visible_to_employee"`
	IsFilterable        bool            `json:"is_filterable"`
	IsExportable        bool            `json:"is_exportable"`
	DisplayOrder        int             `json:"display_order"`
	SectionName         *string         `json:"section_name,omitempty"`
	ValidationRules     json.RawMessage `json:"validation_rules,omitempty"`
}

type UpdateCustomFieldDefinitionRequest struct {
	FieldLabel          *string          `json:"field_label,omitempty"`
	Options             *json.RawMessage `json:"options,omitempty"`
	DefaultValue        *string          `json:"default_value,omitempty"`
	IsRequired          *bool            `json:"is_required,omitempty"`
	IsVisibleToEmployee *bool            `json:"is_visible_to_employee,omitempty"`
	IsFilterable        *bool            `json:"is_filterable,omitempty"`
	IsExportable        *bool            `json:"is_exportable,omitempty"`
	DisplayOrder        *int             `json:"display_order,omitempty"`
	SectionName         *string          `json:"section_name,omitempty"`
	ValidationRules     *json.RawMessage `json:"validation_rules,omitempty"`
}

type SetCustomFieldValueItem struct {
	DefinitionID uuid.UUID       `json:"definition_id"`
	Value        json.RawMessage `json:"value"`
}

type BulkSetCustomFieldValuesRequest struct {
	Values []SetCustomFieldValueItem `json:"values"`
}

type CustomFieldValueResponse struct {
	DefinitionID uuid.UUID       `json:"definition_id"`
	FieldKey     string          `json:"field_key"`
	FieldLabel   string          `json:"field_label"`
	FieldType    string          `json:"field_type"`
	Value        json.RawMessage `json:"value"`
}

type CustomFieldRepository interface {
	CreateDefinition(ctx context.Context, record CustomFieldDefinitionRecord) (CustomFieldDefinitionRecord, error)
	GetDefinition(ctx context.Context, wsID, id uuid.UUID) (CustomFieldDefinitionRecord, error)
	ListDefinitions(ctx context.Context, wsID uuid.UUID, entityType *string) ([]CustomFieldDefinitionRecord, error)
	UpdateDefinition(ctx context.Context, wsID, id uuid.UUID, req UpdateCustomFieldDefinitionRequest) (CustomFieldDefinitionRecord, error)
	DeleteDefinition(ctx context.Context, wsID, id uuid.UUID) error
	GetValues(ctx context.Context, wsID, entityID uuid.UUID) ([]CustomFieldValueResponse, error)
	SetValues(ctx context.Context, wsID, entityID uuid.UUID, entityType string, values []SetCustomFieldValueItem) ([]CustomFieldValueResponse, error)
}

type CustomFieldHandler struct {
	Repository CustomFieldRepository
	Authorizer WorkspaceAuthorizer
}

func (h CustomFieldHandler) CreateDefinition(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowed(r, wsID, "workspace:settings:write") {
		h.forbidden(w, r)
		return
	}
	var req CreateCustomFieldDefinitionRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil {
		h.validation(w, r)
		return
	}
	req.FieldKey = strings.TrimSpace(req.FieldKey)
	req.FieldLabel = strings.TrimSpace(req.FieldLabel)
	if req.FieldKey == "" || req.FieldLabel == "" || req.EntityType == "" || req.FieldType == "" {
		h.validation(w, r)
		return
	}
	now := time.Now().UTC()
	record := CustomFieldDefinitionRecord{
		ID:                  uuid.New(),
		WorkspaceID:         wsID,
		EntityType:          req.EntityType,
		FieldKey:            req.FieldKey,
		FieldLabel:          req.FieldLabel,
		FieldType:           req.FieldType,
		Options:             req.Options,
		DefaultValue:        req.DefaultValue,
		IsRequired:          req.IsRequired,
		IsVisibleToEmployee: req.IsVisibleToEmployee,
		IsFilterable:        req.IsFilterable,
		IsExportable:        req.IsExportable,
		DisplayOrder:        req.DisplayOrder,
		SectionName:         req.SectionName,
		ValidationRules:     req.ValidationRules,
		CreatedAt:           now.Format(time.RFC3339),
		UpdatedAt:           now.Format(time.RFC3339),
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("repository unavailable")))
		return
	}
	created, err := h.Repository.CreateDefinition(r.Context(), record)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, created)
}

func (h CustomFieldHandler) GetDefinition(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "workspace:settings:read") {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	record, err := h.Repository.GetDefinition(r.Context(), wsID, id)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, record)
}

func (h CustomFieldHandler) ListDefinitions(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok {
		h.forbidden(w, r)
		return
	}
	var entityType *string
	if raw := r.URL.Query().Get("entity_type"); raw != "" {
		entityType = &raw
	}
	allowed := h.allowed(r, wsID, "workspace:settings:read")
	if !allowed && entityType != nil {
		switch *entityType {
		case "employee", "employment":
			allowed = h.allowed(r, wsID, "employees:read")
		case "department":
			allowed = h.allowed(r, wsID, "departments:read")
		}
	}
	if !allowed {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []CustomFieldDefinitionRecord{})
		return
	}
	list, err := h.Repository.ListDefinitions(r.Context(), wsID, entityType)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, list)
}

func (h CustomFieldHandler) UpdateDefinition(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "workspace:settings:write") {
		h.forbidden(w, r)
		return
	}
	var req UpdateCustomFieldDefinitionRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil {
		h.validation(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	updated, err := h.Repository.UpdateDefinition(r.Context(), wsID, id, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, updated)
}

func (h CustomFieldHandler) DeleteDefinition(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "workspace:settings:write") {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if err := h.Repository.DeleteDefinition(r.Context(), wsID, id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h CustomFieldHandler) GetValues(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	entityType := r.PathValue("entity_type")
	entityID, idOK := pathUUID(r, "entity_id")
	if !ok || !idOK {
		h.validation(w, r)
		return
	}
	perm := "employees:read"
	if entityType == "department" {
		perm = "departments:read"
	}
	if !h.allowed(r, wsID, perm) {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []CustomFieldValueResponse{})
		return
	}
	values, err := h.Repository.GetValues(r.Context(), wsID, entityID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, values)
}

func (h CustomFieldHandler) SetValues(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	entityType := r.PathValue("entity_type")
	entityID, idOK := pathUUID(r, "entity_id")
	if !ok || !idOK {
		h.validation(w, r)
		return
	}
	perm := "employees:update"
	if entityType == "department" {
		perm = "departments:update"
	}
	if !h.allowed(r, wsID, perm) {
		h.forbidden(w, r)
		return
	}
	var req BulkSetCustomFieldValuesRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil {
		h.validation(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []CustomFieldValueResponse{})
		return
	}
	values, err := h.Repository.SetValues(r.Context(), wsID, entityID, entityType, req.Values)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, values)
}

func (h CustomFieldHandler) allowed(r *http.Request, wsID uuid.UUID, p string) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), u, wsID, p)
}

func (h CustomFieldHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h CustomFieldHandler) validation(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
}

func (h CustomFieldHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrCustomFieldNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrCustomFieldConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
