package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"strconv"
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
	ErrTenantNotFound     = errors.New("tenant not found")
	ErrClusterNotFound    = errors.New("database cluster not found")
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

type SystemDBCluster struct {
	ID               uuid.UUID `json:"id"`
	Name             string    `json:"name"`
	AdminDatabaseURL string    `json:"admin_database_url"`
	Region           string    `json:"region"`
	MaxTenants       int32     `json:"max_tenants"`
	ActiveTenants    int32     `json:"active_tenants"`
	IsActive         bool      `json:"is_active"`
	CreatedAt        time.Time `json:"created_at"`
	UpdatedAt        time.Time `json:"updated_at"`
}

type CreateDBClusterRequest struct {
	Name             string `json:"name"`
	AdminDatabaseURL string `json:"admin_database_url"`
	Region           string `json:"region"`
	MaxTenants       int32  `json:"max_tenants"`
	IsActive         bool   `json:"is_active"`
}

type UpdateDBClusterStatusRequest struct {
	IsActive bool `json:"is_active"`
}

type SystemTenant struct {
	ID                   uuid.UUID `json:"id"`
	Name                 string    `json:"name"`
	IsBYOD               bool      `json:"is_byod"`
	Timezone             string    `json:"timezone"`
	CreatedAt            time.Time `json:"created_at"`
	EncryptedDatabaseURL *string   `json:"-"`
}

type SystemTenantResponse struct {
	ID        uuid.UUID `json:"id"`
	Name      string    `json:"name"`
	IsBYOD    bool      `json:"is_byod"`
	Timezone  string    `json:"timezone"`
	CreatedAt time.Time `json:"created_at"`
}

type CreateTenantRequest struct {
	Name        string  `json:"name"`
	DatabaseURL *string `json:"database_url"`
}

type UpdateTenantDBRequest struct {
	DatabaseURL string `json:"database_url"`
}

type TenantConnectionInfo struct {
	Host string `json:"host"`
	Port string `json:"port"`
}

type TenantStatsResponse struct {
	ID          uuid.UUID            `json:"id"`
	Name        string               `json:"name"`
	IsBYOD      bool                 `json:"is_byod"`
	Timezone    string               `json:"timezone"`
	CreatedAt   time.Time            `json:"created_at"`
	MaskedDBURL *string              `json:"masked_db_url"`
	Connection  TenantConnectionInfo `json:"connection"`
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

type SystemDBClusterRepository interface {
	ListDBClusters(context.Context) ([]SystemDBCluster, error)
	CreateDBCluster(context.Context, CreateDBClusterRequest) (SystemDBCluster, error)
	DeleteDBCluster(context.Context, uuid.UUID) error
	UpdateDBClusterStatus(context.Context, uuid.UUID, bool) (SystemDBCluster, error)
}

type SystemSettingsRepository interface {
	Get(context.Context, string) (json.RawMessage, bool, error)
	Set(context.Context, string, json.RawMessage) error
}

type SystemTenantRepository interface {
	ListTenants(context.Context) ([]SystemTenant, error)
	CreateTenant(context.Context, uuid.UUID, string, uuid.UUID, *string) (SystemTenant, error)
	GetTenant(context.Context, uuid.UUID) (SystemTenant, error)
	DeleteTenant(context.Context, uuid.UUID) error
	UpdateTenantDatabase(context.Context, uuid.UUID, *string) error
}

type CryptoService interface {
	Encrypt(string) (string, error)
	Decrypt(string) (string, error)
}

type EmailSender interface {
	SendTestEmail(context.Context, string) (string, error)
}

type RedisClient interface {
	Shared() (*goredis.Client, error)
}

type SystemAdminHandler struct {
	RolesRepo     SystemRoleAdminRepository
	Clusters      SystemDBClusterRepository
	Settings      SystemSettingsRepository
	Tenants       SystemTenantRepository
	Roles         SystemRoleRepository
	Crypto        CryptoService
	EmailSender   EmailSender
	Redis         RedisClient
	PgBouncerHost string
	PgBouncerPort int
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

// 8. GET /api/v1/system/db-clusters
func (h SystemAdminHandler) ListDbClusters(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	if h.Clusters == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("clusters repository not configured")))
		return
	}
	clusters, err := h.Clusters.ListDBClusters(r.Context())
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if clusters == nil {
		clusters = []SystemDBCluster{}
	}
	apperror.WriteJSON(w, http.StatusOK, clusters)
}

// 9. POST /api/v1/system/db-clusters
func (h SystemAdminHandler) CreateDbCluster(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	req, err := DecodeJSON[CreateDBClusterRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.AdminDatabaseURL) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Clusters == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("clusters repository not configured")))
		return
	}
	cluster, err := h.Clusters.CreateDBCluster(r.Context(), req)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, cluster)
}

// 10. DELETE /api/v1/system/db-clusters/{id}
func (h SystemAdminHandler) DeleteDbCluster(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	id, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Clusters == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("clusters repository not configured")))
		return
	}
	if err := h.Clusters.DeleteDBCluster(r.Context(), id); err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

// 11. PATCH /api/v1/system/db-clusters/{id}/status
func (h SystemAdminHandler) UpdateDbClusterStatus(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	id, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[UpdateDBClusterStatusRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Clusters == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("clusters repository not configured")))
		return
	}
	cluster, err := h.Clusters.UpdateDBClusterStatus(r.Context(), id, req.IsActive)
	if err != nil {
		if errors.Is(err, ErrClusterNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, cluster)
}

// 12. GET /api/v1/system/tenants
func (h SystemAdminHandler) ListTenants(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	if h.Tenants == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("tenants repository not configured")))
		return
	}
	tenants, err := h.Tenants.ListTenants(r.Context())
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	res := make([]SystemTenantResponse, 0, len(tenants))
	for _, t := range tenants {
		res = append(res, SystemTenantResponse{
			ID:        t.ID,
			Name:      t.Name,
			IsBYOD:    t.IsBYOD,
			Timezone:  t.Timezone,
			CreatedAt: t.CreatedAt,
		})
	}
	apperror.WriteJSON(w, http.StatusOK, res)
}

// 13. POST /api/v1/system/tenants
func (h SystemAdminHandler) CreateTenant(w http.ResponseWriter, r *http.Request) {
	actorID, appErr := h.requirePlatformAdmin(r)
	if appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	req, err := DecodeJSON[CreateTenantRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Tenants == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("tenants repository not configured")))
		return
	}

	var encryptedURL *string
	if req.DatabaseURL != nil && strings.TrimSpace(*req.DatabaseURL) != "" {
		raw := strings.TrimSpace(*req.DatabaseURL)
		if h.Crypto != nil {
			enc, encErr := h.Crypto.Encrypt(raw)
			if encErr != nil {
				//lint:ignore ST1005 preserve Rust-compatible diagnostic text
				apperror.WriteError(w, r, apperror.Internal(errors.New("Encryption failed")))
				return
			}
			encryptedURL = &enc
		} else {
			encryptedURL = &raw
		}
	}

	id := uuid.New()
	tenant, err := h.Tenants.CreateTenant(r.Context(), id, req.Name, actorID, encryptedURL)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, SystemTenantResponse{
		ID:        tenant.ID,
		Name:      tenant.Name,
		IsBYOD:    tenant.IsBYOD,
		Timezone:  tenant.Timezone,
		CreatedAt: tenant.CreatedAt,
	})
}

// 14. DELETE /api/v1/system/tenants/{tenant_id}
func (h SystemAdminHandler) DeleteTenant(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	id, err := uuid.Parse(r.PathValue("tenant_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Tenants == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("tenants repository not configured")))
		return
	}
	if err := h.Tenants.DeleteTenant(r.Context(), id); err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

// 15. GET /api/v1/system/tenants/{tenant_id}/stats
func (h SystemAdminHandler) GetTenantStats(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	id, err := uuid.Parse(r.PathValue("tenant_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Tenants == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("tenants repository not configured")))
		return
	}
	tenant, err := h.Tenants.GetTenant(r.Context(), id)
	if err != nil {
		if errors.Is(err, ErrTenantNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyWorkspaceNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	var maskedDBURL *string
	if tenant.EncryptedDatabaseURL != nil && strings.TrimSpace(*tenant.EncryptedDatabaseURL) != "" {
		raw := *tenant.EncryptedDatabaseURL
		var decrypted string
		if h.Crypto != nil {
			if dec, decErr := h.Crypto.Decrypt(raw); decErr == nil {
				decrypted = dec
				masked := MaskDatabaseURL(decrypted)
				maskedDBURL = &masked
			}
		} else {
			masked := MaskDatabaseURL(raw)
			maskedDBURL = &masked
		}
	}

	host := os.Getenv("PGBOUNCER_HOST")
	if host == "" {
		host = os.Getenv("DATABASE_HOST")
	}
	if host == "" && h.PgBouncerHost != "" {
		host = h.PgBouncerHost
	}
	if host == "" {
		host = "localhost"
	}

	port := os.Getenv("PGBOUNCER_PORT")
	if port == "" {
		port = os.Getenv("PGPORT")
	}
	if port == "" && h.PgBouncerPort > 0 {
		port = strconv.Itoa(h.PgBouncerPort)
	}
	if port == "" {
		port = "5432"
	}

	apperror.WriteJSON(w, http.StatusOK, TenantStatsResponse{
		ID:          tenant.ID,
		Name:        tenant.Name,
		IsBYOD:      tenant.IsBYOD,
		Timezone:    tenant.Timezone,
		CreatedAt:   tenant.CreatedAt,
		MaskedDBURL: maskedDBURL,
		Connection: TenantConnectionInfo{
			Host: host,
			Port: port,
		},
	})
}

// 16. PUT /api/v1/system/tenants/{tenant_id}/database
func (h SystemAdminHandler) UpdateTenantDatabase(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requirePlatformAdmin(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	id, err := uuid.Parse(r.PathValue("tenant_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[UpdateTenantDBRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.DatabaseURL) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Tenants == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("tenants repository not configured")))
		return
	}

	var encryptedURL *string
	raw := strings.TrimSpace(req.DatabaseURL)
	if h.Crypto != nil {
		enc, encErr := h.Crypto.Encrypt(raw)
		if encErr != nil {
			//lint:ignore ST1005 preserve Rust-compatible diagnostic text
			apperror.WriteError(w, r, apperror.Internal(errors.New("Encryption failed")))
			return
		}
		encryptedURL = &enc
	} else {
		encryptedURL = &raw
	}

	if err := h.Tenants.UpdateTenantDatabase(r.Context(), id, encryptedURL); err != nil {
		if errors.Is(err, ErrTenantNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyWorkspaceNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	// Rust parity: 200 OK with empty body
	w.WriteHeader(http.StatusOK)
}

// MaskDatabaseURL masks credentials with *** according to Rust parity.
func MaskDatabaseURL(raw string) string {
	parsed, err := url.Parse(raw)
	if err != nil || parsed.Scheme == "" || parsed.Host == "" {
		return "*** REDACTED ***"
	}
	if parsed.User != nil {
		username := parsed.User.Username()
		parsed.User = url.UserPassword(username, "***")
		masked := strings.ReplaceAll(parsed.String(), "%2A%2A%2A", "***")
		return masked
	}
	return parsed.String()
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
