package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var ErrConflict = errors.New("api: conflict")

type OrganizationRecord struct {
	Name              string  `json:"name"`
	DateFormat        string  `json:"date_format"`
	TimeFormat        string  `json:"time_format"`
	Timezone          string  `json:"timezone"`
	ThemePrimaryColor string  `json:"theme_primary_color"`
	DefaultLanguage   string  `json:"default_language"`
	LogoURL           *string `json:"logo_url"`
	Tagline           *string `json:"tagline"`
	HolidayOvernight  string  `json:"holiday_overnight_strategy"`
	DefaultHolidayOT  string  `json:"default_holiday_ot_multiplier"`
	CreatedAt         string  `json:"created_at"`
}

type OrganizationPatch struct {
	Name              *string `json:"name"`
	DateFormat        *string `json:"date_format"`
	TimeFormat        *string `json:"time_format"`
	Timezone          *string `json:"timezone"`
	ThemePrimaryColor *string `json:"theme_primary_color"`
	DefaultLanguage   *string `json:"default_language"`
	LogoURL           *string `json:"logo_url"`
	Tagline           *string `json:"tagline"`
}

func (p OrganizationPatch) Empty() bool {
	return p.Name == nil && p.DateFormat == nil && p.TimeFormat == nil && p.Timezone == nil && p.ThemePrimaryColor == nil && p.DefaultLanguage == nil && p.LogoURL == nil && p.Tagline == nil
}

type FeatureFlagRecord struct {
	ID          uuid.UUID       `json:"id"`
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
type OrganizationConfigEntry struct {
	ID            uuid.UUID        `json:"id"`
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
type OrganizationRepository interface {
	GetOrganization(context.Context) (OrganizationRecord, error)
	UpdateOrganization(context.Context, OrganizationPatch) (OrganizationRecord, error)
	CreateRole(context.Context, CreateRoleRequest) error
	ListFeatureFlags(context.Context) ([]FeatureFlagRecord, error)
	CreateFeatureFlag(context.Context, FeatureFlagRequest) (FeatureFlagRecord, error)
	UpdateFeatureFlag(context.Context, uuid.UUID, FeatureFlagPatch) (FeatureFlagRecord, error)
	GetFeatureFlag(context.Context, string) (FeatureFlagRecord, error)
	GetFeatureFlagByID(context.Context, uuid.UUID) (FeatureFlagRecord, error)
	DeleteFeatureFlag(context.Context, uuid.UUID) (bool, error)
	UpsertFeatureFlag(context.Context, FeatureFlagRequest) (FeatureFlagRecord, error)
	ListConfigs(context.Context) ([]OrganizationConfigEntry, error)
	GetConfig(context.Context, string) (ResolvedConfig, error)
	DeleteConfig(context.Context, uuid.UUID) error
	PutConfig(context.Context, ConfigEntryRequest, uuid.UUID) error
}

type OrganizationHandler struct {
	Repository OrganizationRepository
	Authorizer Authorizer
}

func (h OrganizationHandler) Get(w http.ResponseWriter, r *http.Request) {
	_, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	org, err := h.Repository.GetOrganization(r.Context())
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyWorkspaceNotFound))
			return
		}
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, org)
}

func (h OrganizationHandler) ListConfigs(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "organization:settings:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	configs, err := h.Repository.ListConfigs(r.Context())
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, configs)
}

func (h OrganizationHandler) GetConfig(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "organization:settings:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	key := r.PathValue("key")
	if key == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	cfg, err := h.Repository.GetConfig(r.Context(), key)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, cfg)
}

func (h OrganizationHandler) DeleteConfig(w http.ResponseWriter, r *http.Request) {
	entryID, idOK := pathUUID(r, "key")
	if !idOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "organization:settings:write") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if err := h.Repository.DeleteConfig(r.Context(), entryID); err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h OrganizationHandler) Patch(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "organization:settings:write") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	patch, err := DecodeJSON[OrganizationPatch](r, 1<<20)
	if err != nil || patch.Empty() {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	org, err := h.Repository.UpdateOrganization(r.Context(), patch)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, org)
}

func (h OrganizationHandler) ListFeatureFlags(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "feature_flags:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	flags, err := h.Repository.ListFeatureFlags(r.Context())
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, flags)
}

func (h OrganizationHandler) CreateFeatureFlag(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[FeatureFlagRequest](r, 1<<20)
	if err != nil || req.FlagKey == "" || len(req.FlagKey) > 255 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidFeatureFlagKey))
		return
	}
	flag, err := h.Repository.CreateFeatureFlag(r.Context(), req)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, flag)
}

func (h OrganizationHandler) GetFeatureFlagByID(w http.ResponseWriter, r *http.Request) {
	flagID, flagOK := pathUUID(r, "id")
	if !flagOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "feature_flags:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	flag, err := h.Repository.GetFeatureFlagByID(r.Context(), flagID)
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

func (h OrganizationHandler) UpdateFeatureFlag(w http.ResponseWriter, r *http.Request) {
	flagID, flagOK := pathUUID(r, "id")
	if !flagOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	patch, err := DecodeJSON[FeatureFlagPatch](r, 1<<20)
	if err != nil || (patch.Enabled == nil && patch.Description == nil && patch.Metadata == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	flag, err := h.Repository.UpdateFeatureFlag(r.Context(), flagID, patch)
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

func (h OrganizationHandler) UpsertFeatureFlag(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[FeatureFlagRequest](r, 1<<20)
	if err != nil || req.FlagKey == "" || len(req.FlagKey) > 255 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidFeatureFlagKey))
		return
	}
	flag, err := h.Repository.UpsertFeatureFlag(r.Context(), req)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, flag)
}

func (h OrganizationHandler) DeleteFeatureFlag(w http.ResponseWriter, r *http.Request) {
	flagID, flagOK := pathUUID(r, "id")
	if !flagOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	deleted, err := h.Repository.DeleteFeatureFlag(r.Context(), flagID)
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

func (h OrganizationHandler) CheckFeatureFlag(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "feature_flags:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	key := r.PathValue("key")
	if key == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	flag, err := h.Repository.GetFeatureFlag(r.Context(), key)
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

func (h OrganizationHandler) PutConfig(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "organization:settings:write") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	request, err := DecodeJSON[ConfigEntryRequest](r, 1<<20)
	if err != nil || request.ConfigKey == "" || request.EffectiveFrom == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.PutConfig(r.Context(), request, userID); err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h OrganizationHandler) writeRepositoryError(w http.ResponseWriter, r *http.Request, err error) {
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

func organizationIDFromContext(r *http.Request) (uuid.UUID, bool) {
	id, ok := httpinfra.OrganizationIDFromContext(r.Context())
	return id, ok && id != uuid.Nil
}

func pathUUID(r *http.Request, key string) (uuid.UUID, bool) {
	id, err := uuid.Parse(r.PathValue(key))
	return id, err == nil
}
