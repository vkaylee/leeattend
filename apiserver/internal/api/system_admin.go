package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	goredis "github.com/redis/go-redis/v9"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type SystemRoleType string

const (
	SystemRolePlatformAdmin SystemRoleType = "platform_admin"
	SystemRoleSupport       SystemRoleType = "support"
	SystemRoleBillingAdmin  SystemRoleType = "billing_admin"
)

func ParseSystemRoleType(raw string) (SystemRoleType, bool) {
	switch raw {
	case "platform_admin":
		return SystemRolePlatformAdmin, true
	case "support":
		return SystemRoleSupport, true
	case "billing_admin":
		return SystemRoleBillingAdmin, true
	default:
		return "", false
	}
}

var (
	ErrSystemRoleNotFound = errors.New("system role not found")
	ErrSystemRoleUnknown  = errors.New("unknown system role")
	ErrLastPlatformAdmin  = errors.New("cannot revoke last platform admin")
)

type SystemRoleAssignment struct {
	UserID     uuid.UUID  `json:"user_id"`
	RoleName   string     `json:"role_name"`
	AssignedBy *uuid.UUID `json:"assigned_by"`
	AssignedAt time.Time  `json:"assigned_at"`
}

type SystemRoleInfo struct {
	ID          uuid.UUID              `json:"id"`
	Name        string                 `json:"name"`
	Description string                 `json:"description"`
	Assignees   []SystemRoleAssignment `json:"assignees"`
}

type AssignSystemRoleRequest struct {
	RoleName string `json:"role_name"`
}

type SMTPTestRequest struct {
	ToEmail string `json:"to_email"`
}

type SMTPTestResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message"`
}

type SystemRoleAdminRepository interface {
	ListSystemRoles(context.Context) ([]SystemRoleInfo, error)
	GetUserSystemRoles(context.Context, uuid.UUID) ([]string, error)
	AssignSystemRole(context.Context, uuid.UUID, uuid.UUID, SystemRoleType) error
	RevokeSystemRole(context.Context, uuid.UUID, string) error
}

type SystemSettingsRepository interface {
	Get(context.Context, string) (json.RawMessage, bool, error)
	Set(context.Context, string, json.RawMessage) error
}

type EmailSender interface {
	SendTestEmail(context.Context, string) (string, error)
}

type RedisClient interface {
	Shared() (*goredis.Client, error)
}

type SystemAdminHandler struct {
	RolesRepo   SystemRoleAdminRepository
	Settings    SystemSettingsRepository
	Roles       SystemRoleRepository
	EmailSender EmailSender
	Redis       RedisClient
}

func (h SystemAdminHandler) requirePlatformAdmin(r *http.Request) (uuid.UUID, *apperror.AppError) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Roles == nil {
		return uuid.Nil, apperror.Unauthorized(apperror.KeyUnauthorized)
	}
	roles, err := h.Roles.GetSystemRolesForUser(r.Context(), userID)
	if err != nil {
		return uuid.Nil, apperror.Internal(err)
	}
	for _, role := range roles {
		if role == "platform_admin" {
			return userID, nil
		}
	}
	return uuid.Nil, apperror.Forbidden(apperror.KeyForbidden)
}

func (h SystemAdminHandler) invalidateRedisKey(ctx context.Context, key string) {
	if h.Redis == nil {
		return
	}
	client, err := h.Redis.Shared()
	if err != nil || client == nil {
		return
	}
	_ = client.Del(ctx, key).Err()
}

// 1. GET /api/v1/system/smtp/config
func (h SystemAdminHandler) GetSMTPConfig(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	if h.Settings == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("settings repository not configured")))
		return
	}
	val, ok, err := h.Settings.Get(r.Context(), "smtp")
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if !ok || len(val) == 0 {
		val = json.RawMessage(`{}`)
	}
	// Never return SMTP credentials. Rust stores arbitrary JSON, so preserve typed JSON while masking secret keys.
	val = maskSMTPConfig(val)
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(val)
}

// 2. PUT /api/v1/system/smtp/config
func (h SystemAdminHandler) SetSMTPConfig(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	if h.Settings == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("settings repository not configured")))
		return
	}
	body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 1<<20))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if !json.Valid(body) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Settings.Set(r.Context(), "smtp", json.RawMessage(body)); err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	h.invalidateRedisKey(r.Context(), "smtp:config")
	apperror.WriteJSON(w, http.StatusOK, map[string]bool{"saved": true})
}

// 3. POST /api/v1/system/smtp/test
func (h SystemAdminHandler) TestSMTP(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	req, err := DecodeJSON[SMTPTestRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.ToEmail) == "" || !strings.Contains(req.ToEmail, "@") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	var sendErr error
	var msg string
	if h.EmailSender != nil {
		msg, sendErr = h.EmailSender.SendTestEmail(r.Context(), req.ToEmail)
	} else {
		//lint:ignore ST1005 preserve Rust-compatible SMTP diagnostic text
		sendErr = errors.New("Email is not configured. Insert a 'smtp' record into system_settings with either SMTP host/port or an 'http_api' block.")
	}

	if sendErr != nil {
		// Rust parity: SMTP failure returns 200 with success: false
		apperror.WriteJSON(w, http.StatusOK, SMTPTestResponse{
			Success: false,
			Message: sendErr.Error(),
		})
		return
	}

	if raw, ok, err := h.Settings.Get(r.Context(), "smtp"); err == nil {
		var config map[string]json.RawMessage
		if ok && len(raw) > 0 {
			_ = json.Unmarshal(raw, &config)
		}
		if config == nil {
			config = make(map[string]json.RawMessage)
		}
		timestamp, _ := json.Marshal(time.Now().UTC().Format(time.RFC3339))
		config["last_verified_at"] = timestamp
		if updated, err := json.Marshal(config); err == nil {
			_ = h.Settings.Set(r.Context(), "smtp", updated)
		}
	}

	if msg == "" {
		msg = fmt.Sprintf("Test email sent successfully. Check %s for the message.", req.ToEmail)
	}
	apperror.WriteJSON(w, http.StatusOK, SMTPTestResponse{
		Success: true,
		Message: msg,
	})
}

// 4. GET /api/v1/system/roles
func (h SystemAdminHandler) ListRoles(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	if h.RolesRepo == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("roles repository not configured")))
		return
	}
	roles, err := h.RolesRepo.ListSystemRoles(r.Context())
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if roles == nil {
		roles = []SystemRoleInfo{}
	}
	apperror.WriteJSON(w, http.StatusOK, roles)
}

// 5. GET /api/v1/system/users/{user_id}/roles
func (h SystemAdminHandler) GetUserRoles(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	userID, err := uuid.Parse(r.PathValue("user_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.RolesRepo == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("roles repository not configured")))
		return
	}
	roles, err := h.RolesRepo.GetUserSystemRoles(r.Context(), userID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if roles == nil {
		roles = []string{}
	}
	apperror.WriteJSON(w, http.StatusOK, roles)
}

// 6. POST /api/v1/system/users/{user_id}/roles
func (h SystemAdminHandler) AssignRole(w http.ResponseWriter, r *http.Request) {
	actorID, appErr := h.requirePlatformAdmin(r)
	if appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	userID, err := uuid.Parse(r.PathValue("user_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[AssignSystemRoleRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	parsedRole, ok := ParseSystemRoleType(req.RoleName)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyUnknownSystemRole))
		return
	}
	if h.RolesRepo == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("roles repository not configured")))
		return
	}
	if err := h.RolesRepo.AssignSystemRole(r.Context(), userID, actorID, parsedRole); err != nil {
		if errors.Is(err, ErrSystemRoleNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeySystemRoleNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	h.invalidateRedisKey(r.Context(), fmt.Sprintf("system_roles:%s", userID))
	w.WriteHeader(http.StatusNoContent)
}

// 7. DELETE /api/v1/system/users/{user_id}/roles/{role_name}
func (h SystemAdminHandler) RevokeRole(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	userID, err := uuid.Parse(r.PathValue("user_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	roleName := r.PathValue("role_name")
	if _, ok := ParseSystemRoleType(roleName); !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyUnknownSystemRole))
		return
	}
	if h.RolesRepo == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("roles repository not configured")))
		return
	}
	if err := h.RolesRepo.RevokeSystemRole(r.Context(), userID, roleName); err != nil {
		if errors.Is(err, ErrLastPlatformAdmin) {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyCannotRevokeLastAdmin))
			return
		}
		if errors.Is(err, ErrSystemRoleNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeySystemRoleNotFound))
			return
		}
		if errors.Is(err, ErrSystemRoleUnknown) {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyUnknownSystemRole))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	h.invalidateRedisKey(r.Context(), fmt.Sprintf("system_roles:%s", userID))
	w.WriteHeader(http.StatusNoContent)
}

func maskSMTPConfig(raw json.RawMessage) json.RawMessage {
	if len(raw) == 0 {
		return json.RawMessage(`{}`)
	}
	var config map[string]json.RawMessage
	if err := json.Unmarshal(raw, &config); err != nil {
		return json.RawMessage(`{}`)
	}
	for key, value := range config {
		lower := strings.ToLower(key)
		if strings.Contains(lower, "password") || strings.Contains(lower, "secret") || strings.Contains(lower, "token") || strings.Contains(lower, "auth_value") {
			config[key] = json.RawMessage(`"***"`)
			continue
		}
		var nested map[string]json.RawMessage
		if json.Unmarshal(value, &nested) == nil {
			config[key] = maskSMTPConfig(value)
		}
	}
	masked, err := json.Marshal(config)
	if err != nil {
		return json.RawMessage(`{}`)
	}
	return masked
}
