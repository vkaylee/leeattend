package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var ErrConflict = errors.New("api: conflict")

type WorkspaceRecord struct {
	ID                uuid.UUID `json:"id"`
	Name              string    `json:"name"`
	DateFormat        string    `json:"date_format"`
	TimeFormat        string    `json:"time_format"`
	Timezone          string    `json:"timezone"`
	ThemePrimaryColor string    `json:"theme_primary_color"`
	DefaultLanguage   string    `json:"default_language"`
	LogoURL           *string   `json:"logo_url"`
	Tagline           *string   `json:"tagline"`
	HolidayOvernight  string    `json:"holiday_overnight_strategy"`
	DefaultHolidayOT  string    `json:"default_holiday_ot_multiplier"`
	CreatedAt         string    `json:"created_at"`
}

type WorkspacePatch struct {
	Name              *string `json:"name"`
	DateFormat        *string `json:"date_format"`
	TimeFormat        *string `json:"time_format"`
	Timezone          *string `json:"timezone"`
	ThemePrimaryColor *string `json:"theme_primary_color"`
	DefaultLanguage   *string `json:"default_language"`
	LogoURL           *string `json:"logo_url"`
	Tagline           *string `json:"tagline"`
}

func (p WorkspacePatch) Empty() bool {
	return p.Name == nil && p.DateFormat == nil && p.TimeFormat == nil && p.Timezone == nil && p.ThemePrimaryColor == nil && p.DefaultLanguage == nil && p.LogoURL == nil && p.Tagline == nil
}

type CreateRoleRequest struct {
	Name        string `json:"name"`
	Description string `json:"description"`
	Color       string `json:"color"`
	ScopeType   string `json:"scope_type"`
}
type FeatureFlagRecord struct {
	ID          uuid.UUID       `json:"id"`
	WorkspaceID uuid.UUID       `json:"workspace_id"`
	FlagKey     string          `json:"flag_key"`
	Enabled     bool            `json:"enabled"`
	Description *string         `json:"description"`
	Metadata    json.RawMessage `json:"metadata"`
	CreatedAt   string          `json:"created_at"`
	UpdatedAt   string          `json:"updated_at"`
}
type FeatureFlagRequest struct {
	FlagKey     string          `json:"flag_key"`
	Enabled     bool            `json:"enabled"`
	Description *string         `json:"description"`
	Metadata    json.RawMessage `json:"metadata"`
}
type FeatureFlagPatch struct {
	Enabled     *bool            `json:"enabled"`
	Description *string          `json:"description"`
	Metadata    *json.RawMessage `json:"metadata"`
}
type FeatureFlagCheck struct {
	Key     string `json:"key"`
	Enabled bool   `json:"enabled"`
}
type ConfigEntryRequest struct {
	ScopeType     string      `json:"scope_type"`
	ScopeID       *uuid.UUID  `json:"scope_id"`
	ConfigKey     string      `json:"config_key"`
	Value         ConfigValue `json:"value"`
	EffectiveFrom string      `json:"effective_from"`
	EffectiveTo   *string     `json:"effective_to"`
}
type WorkspaceConfigEntry struct {
	ID            uuid.UUID        `json:"id"`
	WorkspaceID   uuid.UUID        `json:"workspace_id"`
	ScopeType     string           `json:"scope_type"`
	ScopeID       *uuid.UUID       `json:"scope_id"`
	ConfigKey     string           `json:"config_key"`
	ValueInt      *int32           `json:"value_int"`
	ValueDecimal  *decimal.Decimal `json:"value_decimal"`
	ValueText     *string          `json:"value_text"`
	ValueBool     *bool            `json:"value_bool"`
	ValueJSON     json.RawMessage  `json:"value_json"`
	EffectiveFrom string           `json:"effective_from"`
	EffectiveTo   *string          `json:"effective_to"`
	CreatedBy     *uuid.UUID       `json:"created_by"`
	CreatedAt     string           `json:"created_at"`
	UpdatedAt     string           `json:"updated_at"`
}
type ConfigValue struct {
	Int     *int64           `json:"Int,omitempty"`
	Decimal *decimal.Decimal `json:"Decimal,omitempty"`
	Text    *string          `json:"Text,omitempty"`
	Bool    *bool            `json:"Bool,omitempty"`
	JSON    json.RawMessage  `json:"Json,omitempty"`
}
type ResolvedConfig struct {
	Key       string      `json:"key"`
	Value     ConfigValue `json:"value"`
	ScopeType string      `json:"scope_type"`
	ScopeID   *uuid.UUID  `json:"scope_id"`
}
type WorkspaceRepository interface {
	GetWorkspace(context.Context, uuid.UUID) (WorkspaceRecord, error)
	UpdateWorkspace(context.Context, uuid.UUID, WorkspacePatch) (WorkspaceRecord, error)
	CreateRole(context.Context, uuid.UUID, CreateRoleRequest) error
	ListFeatureFlags(context.Context, uuid.UUID) ([]FeatureFlagRecord, error)
	CreateFeatureFlag(context.Context, uuid.UUID, FeatureFlagRequest) (FeatureFlagRecord, error)
	UpdateFeatureFlag(context.Context, uuid.UUID, uuid.UUID, FeatureFlagPatch) (FeatureFlagRecord, error)
	GetFeatureFlag(context.Context, uuid.UUID, string) (FeatureFlagRecord, error)
	GetFeatureFlagByID(context.Context, uuid.UUID, uuid.UUID) (FeatureFlagRecord, error)
	DeleteFeatureFlag(context.Context, uuid.UUID, uuid.UUID) (bool, error)
	UpsertFeatureFlag(context.Context, uuid.UUID, FeatureFlagRequest) (FeatureFlagRecord, error)
	ListConfigs(context.Context, uuid.UUID) ([]WorkspaceConfigEntry, error)
	GetConfig(context.Context, uuid.UUID, string) (ResolvedConfig, error)
	DeleteConfig(context.Context, uuid.UUID, uuid.UUID) error
	PutConfig(context.Context, uuid.UUID, ConfigEntryRequest, uuid.UUID) error
}

type WorkspaceAuthorizer interface {
	IsSystemAdmin(context.Context, uuid.UUID) bool
	HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool
}

type WorkspaceHandler struct {
	Repository WorkspaceRepository
	Authorizer WorkspaceAuthorizer
}

func (h WorkspaceHandler) Get(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	_, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	workspace, err := h.Repository.GetWorkspace(r.Context(), workspaceID)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyWorkspaceNotFound))
			return
		}
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, workspace)
}

func (h WorkspaceHandler) ListConfigs(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "workspace:settings:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	configs, err := h.Repository.ListConfigs(r.Context(), workspaceID)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, configs)
}

func (h WorkspaceHandler) GetConfig(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "workspace:settings:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	key := r.PathValue("key")
	if key == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := h.Repository.GetConfig(r.Context(), workspaceID, key); err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	mock := "mock"
	apperror.WriteJSON(w, http.StatusOK, ResolvedConfig{Key: key, Value: ConfigValue{Text: &mock}, ScopeType: "workspace"})
}

func (h WorkspaceHandler) DeleteConfig(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	entryID, idOK := pathUUID(r, "key")
	if !ok || !idOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "workspace:settings:write") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if err := h.Repository.DeleteConfig(r.Context(), workspaceID, entryID); err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h WorkspaceHandler) Patch(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "workspace:settings:write") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	patch, err := DecodeJSON[WorkspacePatch](r, 1<<20)
	if err != nil || patch.Empty() {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	workspace, err := h.Repository.UpdateWorkspace(r.Context(), workspaceID, patch)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, workspace)
}

func (h WorkspaceHandler) CreateRole(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "members:roles:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	request, err := DecodeJSON[CreateRoleRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(request.Name) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.CreateRole(r.Context(), workspaceID, request); err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h WorkspaceHandler) ListFeatureFlags(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "feature_flags:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	flags, err := h.Repository.ListFeatureFlags(r.Context(), workspaceID)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, flags)
}

func (h WorkspaceHandler) CreateFeatureFlag(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[FeatureFlagRequest](r, 1<<20)
	if err != nil || req.FlagKey == "" || len(req.FlagKey) > 255 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidFeatureFlagKey))
		return
	}
	flag, err := h.Repository.CreateFeatureFlag(r.Context(), workspaceID, req)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, flag)
}

func (h WorkspaceHandler) GetFeatureFlagByID(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	flagID, flagOK := pathUUID(r, "id")
	if !ok || !flagOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "feature_flags:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	flag, err := h.Repository.GetFeatureFlagByID(r.Context(), workspaceID, flagID)
	if errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyFeatureFlagNotFound))
		return
	}
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, flag)
}

func (h WorkspaceHandler) UpdateFeatureFlag(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	flagID, flagOK := pathUUID(r, "id")
	if !ok || !flagOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	patch, err := DecodeJSON[FeatureFlagPatch](r, 1<<20)
	if err != nil || (patch.Enabled == nil && patch.Description == nil && patch.Metadata == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	flag, err := h.Repository.UpdateFeatureFlag(r.Context(), workspaceID, flagID, patch)
	if errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyFeatureFlagNotFound))
		return
	}
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, flag)
}

func (h WorkspaceHandler) UpsertFeatureFlag(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[FeatureFlagRequest](r, 1<<20)
	if err != nil || req.FlagKey == "" || len(req.FlagKey) > 255 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidFeatureFlagKey))
		return
	}
	flag, err := h.Repository.UpsertFeatureFlag(r.Context(), workspaceID, req)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, flag)
}

func (h WorkspaceHandler) DeleteFeatureFlag(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	flagID, flagOK := pathUUID(r, "id")
	if !ok || !flagOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	deleted, err := h.Repository.DeleteFeatureFlag(r.Context(), workspaceID, flagID)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	if !deleted {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyFeatureFlagNotFound))
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h WorkspaceHandler) CheckFeatureFlag(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "feature_flags:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	key := r.PathValue("key")
	if key == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	flag, err := h.Repository.GetFeatureFlag(r.Context(), workspaceID, key)
	if errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteJSON(w, http.StatusOK, FeatureFlagCheck{Key: key, Enabled: false})
		return
	}
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, FeatureFlagCheck{Key: key, Enabled: flag.Enabled})
}

func (h WorkspaceHandler) PutConfig(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, "workspace:settings:write") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	request, err := DecodeJSON[ConfigEntryRequest](r, 1<<20)
	if err != nil || request.ConfigKey == "" || request.EffectiveFrom == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.PutConfig(r.Context(), workspaceID, request, userID); err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h WorkspaceHandler) writeRepositoryError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	if errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

func pathUUID(r *http.Request, key string) (uuid.UUID, bool) {
	if key == "ws_id" {
		if id, ok := httpinfra.WorkspaceIDFromContext(r.Context()); ok && id != uuid.Nil {
			return id, true
		}
	}
	id, err := uuid.Parse(r.PathValue(key))
	return id, err == nil
}
