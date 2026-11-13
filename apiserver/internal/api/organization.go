package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"regexp"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var ErrConflict = errors.New("api: conflict")

var featureFlagKeyPattern = regexp.MustCompile(`^ff_[a-z0-9]+(?:_[a-z0-9]+)*$`)

const (
	maxFeatureFlagKeyLength         = 128
	maxFeatureFlagDescriptionLength = 1000
	maxFeatureFlagMetadataBytes     = 16 << 10
)

func validFeatureFlagMetadata(metadata json.RawMessage) bool {
	if len(metadata) == 0 {
		return true
	}
	if len(metadata) > maxFeatureFlagMetadataBytes {
		return false
	}
	var object map[string]json.RawMessage
	return json.Unmarshal(metadata, &object) == nil && object != nil
}

func validFeatureFlagRequest(req FeatureFlagRequest) bool {
	if len(req.FlagKey) == 0 || len(req.FlagKey) > maxFeatureFlagKeyLength || !featureFlagKeyPattern.MatchString(req.FlagKey) {
		return false
	}
	if req.Description != nil && len(*req.Description) > maxFeatureFlagDescriptionLength {
		return false
	}
	return validFeatureFlagMetadata(req.Metadata)
}

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
type ConfigReadRequest struct {
	ScopeType string
	ScopeID   *uuid.UUID
	AsOf      time.Time
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
	GetConfig(context.Context, string, ConfigReadRequest) (ResolvedConfig, error)
	DeleteConfig(context.Context, uuid.UUID) error
	PutConfig(context.Context, ConfigEntryRequest, uuid.UUID) error
}

type OrganizationFeatureFlagAudit interface {
	Emit(context.Context, audit.Event) error
}

type OrganizationHandler struct {
	Repository OrganizationRepository
	Authorizer Authorizer
	Audit      OrganizationFeatureFlagAudit
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
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyOrganizationNotFound))
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
	readRequest, err := parseConfigReadRequest(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if readRequest.ScopeType == "employer" && !h.Authorizer.IsGlobalManager(r.Context(), userID) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	cfg, err := h.Repository.GetConfig(r.Context(), key, readRequest)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, cfg)
}

func parseConfigReadRequest(r *http.Request) (ConfigReadRequest, error) {
	request := ConfigReadRequest{ScopeType: "organization", AsOf: time.Now().UTC()}
	query := r.URL.Query()
	if scopeType := strings.TrimSpace(query.Get("scope_type")); scopeType != "" {
		request.ScopeType = scopeType
	}
	if request.ScopeType != "organization" && request.ScopeType != "employer" {
		return ConfigReadRequest{}, errConfigScope
	}
	if scopeID := strings.TrimSpace(query.Get("scope_id")); scopeID != "" {
		id, err := uuid.Parse(scopeID)
		if err != nil {
			return ConfigReadRequest{}, errConfigScopeID
		}
		request.ScopeID = &id
	}
	if request.ScopeType == "organization" && request.ScopeID != nil {
		return ConfigReadRequest{}, errConfigScopeID
	}
	if request.ScopeType == "employer" && (request.ScopeID == nil || *request.ScopeID == uuid.Nil) {
		return ConfigReadRequest{}, errConfigScopeID
	}
	if asOf := strings.TrimSpace(query.Get("as_of")); asOf != "" {
		parsed, err := time.Parse("2006-01-02", asOf)
		if err != nil {
			return ConfigReadRequest{}, errConfigDate
		}
		request.AsOf = parsed.UTC()
	}
	return request, nil
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

// ListEffectiveFeatureFlags exposes only runtime state needed by authenticated clients.
// Management metadata remains protected by feature_flags:read.
func (h OrganizationHandler) ListEffectiveFeatureFlags(w http.ResponseWriter, r *http.Request) {
	if _, ok := httpinfra.UserIDFromContext(r.Context()); !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	flags, err := h.Repository.ListFeatureFlags(r.Context())
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	effective := make([]FeatureFlagCheck, 0, len(flags))
	for _, flag := range flags {
		effective = append(effective, FeatureFlagCheck{Key: flag.FlagKey, Enabled: flag.Enabled})
	}
	apperror.WriteJSON(w, http.StatusOK, effective)
}

func (h OrganizationHandler) CreateFeatureFlag(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[FeatureFlagRequest](r, 1<<20)
	if err != nil || !validFeatureFlagRequest(req) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidFeatureFlagKey))
		return
	}
	flag, err := h.Repository.CreateFeatureFlag(r.Context(), req)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	h.emitFeatureFlagAudit(r, "feature_flag.created", flag)
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
	if err != nil || (patch.Enabled == nil && patch.Description == nil && patch.Metadata == nil) ||
		(patch.Description != nil && len(*patch.Description) > maxFeatureFlagDescriptionLength) ||
		(patch.Metadata != nil && !validFeatureFlagMetadata(*patch.Metadata)) {
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
	h.emitFeatureFlagAudit(r, "feature_flag.updated", flag)
	apperror.WriteJSON(w, http.StatusOK, flag)
}

func (h OrganizationHandler) UpsertFeatureFlag(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "feature_flags:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[FeatureFlagRequest](r, 1<<20)
	if err != nil || !validFeatureFlagRequest(req) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidFeatureFlagKey))
		return
	}
	flag, err := h.Repository.UpsertFeatureFlag(r.Context(), req)
	if err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	h.emitFeatureFlagAudit(r, "feature_flag.upserted", flag)
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
	if h.Audit != nil {
		if userID, ok := httpinfra.UserIDFromContext(r.Context()); ok {
			_ = h.Audit.Emit(r.Context(), audit.Event{
				Name: "feature_flag.deleted", ActorID: userID.String(), Resource: flagID.String(), Outcome: "success",
				Details: map[string]string{"flag_id": flagID.String()},
			})
		}
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h OrganizationHandler) emitFeatureFlagAudit(r *http.Request, name string, flag FeatureFlagRecord) {
	if h.Audit == nil {
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return
	}
	_ = h.Audit.Emit(r.Context(), audit.Event{
		Name: name, ActorID: userID.String(), Resource: flag.ID.String(), Outcome: "success",
		Details: map[string]string{"flag_key": flag.FlagKey, "enabled": fmt.Sprintf("%t", flag.Enabled)},
	})
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

func isSupportedConfigScope(scopeType string) bool {
	switch scopeType {
	case "organization", "employer", "department", "employee", "employment":
		return true
	default:
		return false
	}
}

func (h OrganizationHandler) PutConfig(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "organization:settings:write") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	request, err := DecodeJSON[ConfigEntryRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	request.ConfigKey = strings.TrimSpace(request.ConfigKey)
	if request.EffectiveTo != nil && strings.TrimSpace(*request.EffectiveTo) == "" {
		request.EffectiveTo = nil
	}
	if err := validateConfigEntry(request); err != nil {
		key := apperror.KeyValidation
		if errors.Is(err, errConfigScope) {
			key = apperror.KeyInvalidScopeType
		}
		apperror.WriteError(w, r, apperror.Validation(key))
		return
	}
	if request.ConfigKey == "schedule.lifecycle.enabled" && !h.Authorizer.IsGlobalManager(r.Context(), userID) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if err := h.Repository.PutConfig(r.Context(), request, userID); err != nil {
		h.writeRepositoryError(w, r, err)
		return
	}
	h.emitConfigAudit(r, request)
	w.WriteHeader(http.StatusOK)
}

func (h OrganizationHandler) emitConfigAudit(r *http.Request, request ConfigEntryRequest) {
	if h.Audit == nil {
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return
	}
	name := "config.updated"
	if request.ConfigKey == "schedule.lifecycle.enabled" {
		name = "schedule_lifecycle.activation.updated"
	}
	resource := request.ConfigKey
	if request.ScopeID != nil {
		resource = request.ScopeID.String()
	}
	details := map[string]string{
		"config_key": request.ConfigKey,
		"scope_type": request.ScopeType,
	}
	if request.ScopeID != nil {
		details["scope_id"] = request.ScopeID.String()
	}
	_ = h.Audit.Emit(r.Context(), audit.Event{
		Name: name, ActorID: userID.String(), Resource: resource, Outcome: "success", Details: details,
	})
}

func (h OrganizationHandler) writeRepositoryError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrEmployerNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

func pathUUID(r *http.Request, key string) (uuid.UUID, bool) {
	id, err := uuid.Parse(r.PathValue(key))
	return id, err == nil
}
