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
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type StandaloneAuthorizer interface {
	HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool
}

type SMTPTestRequest struct {
	ToEmail string `json:"to_email"`
}

type SMTPTestResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message"`
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
	Authorizer  StandaloneAuthorizer
	Settings    SystemSettingsRepository
	EmailSender EmailSender
	Redis       RedisClient
}

func (h SystemAdminHandler) requireStandalonePermission(r *http.Request, permission string) (uuid.UUID, *apperror.AppError) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || userID == uuid.Nil {
		return uuid.Nil, apperror.Unauthorized(apperror.KeyUnauthorized)
	}
	workspaceID, ok := httpinfra.WorkspaceIDFromContext(r.Context())
	if !ok || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, permission) {
		return uuid.Nil, apperror.Forbidden(apperror.KeyForbidden)
	}
	return userID, nil
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
	if _, appErr := h.requireStandalonePermission(r, string(auth.PermissionSecuritySettingsManage)); appErr != nil {
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
	val = maskSMTPConfig(val)
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(val)
}

// 2. PUT /api/v1/system/smtp/config
func (h SystemAdminHandler) SetSMTPConfig(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireStandalonePermission(r, string(auth.PermissionSecuritySettingsManage)); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	if h.Settings == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("settings repository not configured")))
		return
	}
	body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 1<<20))
	if err != nil || !json.Valid(body) {
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
	if _, appErr := h.requireStandalonePermission(r, string(auth.PermissionSecuritySettingsManage)); appErr != nil {
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
		sendErr = errors.New("email is not configured; insert a 'smtp' record into system_settings with either SMTP host/port or an 'http_api' block")
	}
	if sendErr != nil {
		apperror.WriteJSON(w, http.StatusOK, SMTPTestResponse{Success: false, Message: sendErr.Error()})
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
	apperror.WriteJSON(w, http.StatusOK, SMTPTestResponse{Success: true, Message: msg})
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
