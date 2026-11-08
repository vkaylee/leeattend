package api

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	authpkg "github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/crypto"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrRBACNotFound     = errors.New("organization rbac: not found")
	ErrRBACProtected    = errors.New("organization rbac: protected role")
	ErrRBACUserNotFound = errors.New("organization rbac: user not found")
	ErrRBACLastAdmin    = errors.New("organization rbac: cannot remove last global administrator")
)

type AddMemberRequest struct {
	Email  string    `json:"email"`
	RoleID uuid.UUID `json:"role_id"`
}
type UpdateMemberRoleRequest struct {
	RoleID uuid.UUID `json:"role_id"`
}
type AssignRoleRequest struct {
	RoleID    uuid.UUID `json:"role_id"`
	ExpiresAt time.Time `json:"expires_at"`
}
type AssignPermissionRequest struct {
	Permission string    `json:"permission"`
	ExpiresAt  time.Time `json:"expires_at"`
}

type MemberDetail struct {
	UserID    uuid.UUID `json:"user_id"`
	RoleID    uuid.UUID `json:"role_id"`
	Email     string    `json:"email"`
	FullName  *string   `json:"full_name"`
	AvatarURL *string   `json:"avatar_url"`
}
type MemberDetailResponse struct {
	MemberDetail
	ExtraRoleIDs []uuid.UUID `json:"extra_role_ids"`
}
type MemberListResponse struct {
	Data  []MemberDetailResponse `json:"data"`
	Total int64                  `json:"total"`
	Page  int                    `json:"page"`
	Limit int                    `json:"limit"`
}
type MemberExtraRole struct {
	ID         uuid.UUID  `json:"id"`
	UserID     uuid.UUID  `json:"user_id"`
	RoleID     uuid.UUID  `json:"role_id"`
	ExpiresAt  time.Time  `json:"expires_at"`
	AssignedAt *time.Time `json:"assigned_at,omitempty"`
}
type MemberExtraPermission struct {
	ID         uuid.UUID  `json:"id"`
	UserID     uuid.UUID  `json:"user_id"`
	Permission string     `json:"permission"`
	AssignedAt *time.Time `json:"assigned_at,omitempty"`
	ExpiresAt  time.Time  `json:"expires_at"`
	AssignedBy *uuid.UUID `json:"assigned_by,omitempty"`
}

type RoleScopeType string

const (
	RoleScopeGlobal RoleScopeType = "global"
	RoleScopeScoped RoleScopeType = "scoped"
)

type CreateRoleRequest struct {
	Name        string        `json:"name"`
	Description *string       `json:"description"`
	Color       string        `json:"color"`
	ScopeType   RoleScopeType `json:"scope_type"`
}
type UpdateRoleRequest struct {
	Name        *string `json:"name"`
	Description *string `json:"description"`
	Color       *string `json:"color"`
}
type UpdateRolePermissionsRequest struct {
	Permissions []string `json:"permissions"`
}
type Role struct {
	ID          uuid.UUID     `json:"id"`
	Name        string        `json:"name"`
	Description *string       `json:"description"`
	Color       string        `json:"color"`
	IsSystem    bool          `json:"is_system"`
	ScopeType   RoleScopeType `json:"scope_type"`
	CreatedAt   time.Time     `json:"created_at"`
	UpdatedAt   time.Time     `json:"updated_at"`
}
type RoleListResponse struct {
	Data  []Role `json:"data"`
	Total int64  `json:"total"`
	Page  int    `json:"page"`
	Limit int    `json:"limit"`
}
type PermissionCatalogItem struct {
	ID       string `json:"id"`
	Group    string `json:"group"`
	LabelKey string `json:"label_key"`
}

type StorageConfigRequest struct {
	EndpointURL      string  `json:"endpoint_url"`
	Region           string  `json:"region"`
	BucketName       string  `json:"bucket_name"`
	PathPrefix       *string `json:"path_prefix"`
	AccessKey        string  `json:"access_key"`
	SecretKey        string  `json:"secret_key"`
	UsePathStyle     bool    `json:"use_path_style"`
	PresignedTTLSecs int32   `json:"presigned_ttl_secs"`
	MaxFileSize      int64   `json:"max_file_size"`
}
type StorageConfigResponse struct {
	ID              uuid.UUID `json:"id"`
	EndpointURL     string    `json:"endpoint_url"`
	Region          string    `json:"region"`
	BucketName      string    `json:"bucket_name"`
	PathPrefix      *string   `json:"path_prefix"`
	AccessKeyMasked string    `json:"access_key_masked"`
	UsePathStyle    bool      `json:"use_path_style"`
	IsActive        bool      `json:"is_active"`
}
type StorageConfigRecord struct {
	StorageConfigResponse
	AccessKeyEnc         string
	SecretKeyEnc         string
	EncryptionKeyVersion string
	PresignedTTLSecs     int32
	MaxFileSize          int64
}

type OrganizationRBACRepository interface {
	ListMembers(context.Context, int, int) ([]MemberDetail, int64, error)
	ListExtraRoles(context.Context) ([]MemberExtraRole, error)
	FindUserByEmail(context.Context, string) (uuid.UUID, error)
	AddMember(context.Context, uuid.UUID, uuid.UUID) error
	FindPrimaryRole(context.Context, uuid.UUID) (*uuid.UUID, error)
	UpdateMemberRole(context.Context, uuid.UUID, uuid.UUID) error
	RemoveMember(context.Context, uuid.UUID) error
	ListMemberRoles(context.Context, uuid.UUID) (*Role, []MemberExtraRole, error)
	AssignExtraRole(context.Context, uuid.UUID, uuid.UUID, time.Time) (MemberExtraRole, error)
	RevokeExtraRole(context.Context, uuid.UUID) (int64, error)
	ListMemberPermissions(context.Context, uuid.UUID) ([]MemberExtraPermission, error)
	AssignExtraPermission(context.Context, uuid.UUID, string, time.Time, uuid.UUID) (MemberExtraPermission, error)
	RevokeExtraPermission(context.Context, uuid.UUID) (int64, error)
	ListRoles(context.Context, int, int) ([]Role, int64, error)
	CreateRole(context.Context, CreateRoleRequest) (Role, error)
	UpdateRole(context.Context, uuid.UUID, UpdateRoleRequest) (Role, error)
	DeleteRole(context.Context, uuid.UUID) (int64, error)
	RolePermissions(context.Context, uuid.UUID) ([]string, error)
	RoleIsMutable(context.Context, uuid.UUID) (bool, error)
	SetRolePermissions(context.Context, uuid.UUID, []string) error
	GetStorageConfig(context.Context) (*StorageConfigRecord, error)
	UpsertStorageConfig(context.Context, StorageConfigRecord) (*StorageConfigRecord, error)
	DeleteStorageConfig(context.Context) error
}

type StorageConnectionTester interface {
	Test(context.Context, StorageConfigRequest) error
}
type OrganizationRBACHandler struct {
	Repository    OrganizationRBACRepository
	Authorizer    Authorizer
	Crypto        *crypto.Service
	StorageTester StorageConnectionTester
}

func (h OrganizationRBACHandler) permission(r *http.Request, permission string) (uuid.UUID, bool) {
	uid, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), uid, permission) {
		return uuid.Nil, false
	}
	return uid, true
}
func writeRBACError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrRBACUserNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyUserNotFound))
	case errors.Is(err, ErrRBACNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrRBACProtected):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyRoleNotFoundOrSystem))
	case errors.Is(err, ErrConflict), errors.Is(err, ErrRBACLastAdmin):
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
func (h OrganizationRBACHandler) require(w http.ResponseWriter, r *http.Request, p string) (uuid.UUID, bool) {
	uid, ok := h.permission(r, p)
	if !ok {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
	}
	return uid, ok
}

func (h OrganizationRBACHandler) ListMembers(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.memberContext(w, r); !ok {
		return
	}
	p, e := ParsePagination(r.URL.Query())
	if e != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	members, total, e := h.Repository.ListMembers(r.Context(), p.Page, p.Limit)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	extras, e := h.Repository.ListExtraRoles(r.Context())
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	byUser := map[uuid.UUID][]uuid.UUID{}
	for _, x := range extras {
		byUser[x.UserID] = append(byUser[x.UserID], x.RoleID)
	}
	data := make([]MemberDetailResponse, 0, len(members))
	for _, m := range members {
		data = append(data, MemberDetailResponse{MemberDetail: m, ExtraRoleIDs: byUser[m.UserID]})
	}
	apperror.WriteJSON(w, http.StatusOK, MemberListResponse{Data: data, Total: total, Page: p.Page, Limit: p.Limit})
}
func (h OrganizationRBACHandler) memberContext(w http.ResponseWriter, r *http.Request) (uuid.UUID, bool) {
	uid, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return uuid.Nil, false
	}
	if h.Authorizer == nil {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return uuid.Nil, false
	}
	member, err := h.Authorizer.IsMember(r.Context(), uid)
	if err != nil || !member {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return uuid.Nil, false
	}
	return uid, true
}
func (h OrganizationRBACHandler) AddMember(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.require(w, r, "members:invite"); !ok {
		return
	}
	req, e := DecodeJSON[AddMemberRequest](r, 1<<20)
	if e != nil || strings.TrimSpace(req.Email) == "" || req.RoleID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	uid, e := h.Repository.FindUserByEmail(r.Context(), strings.TrimSpace(req.Email))
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	if e = h.Repository.AddMember(r.Context(), uid, req.RoleID); e != nil {
		writeRBACError(w, r, e)
		return
	}
	w.WriteHeader(http.StatusCreated)
}
func (h OrganizationRBACHandler) UpdateMemberRole(w http.ResponseWriter, r *http.Request) {
	user, userOK := pathUUID(r, "user_id")
	if !userOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	req, e := DecodeJSON[UpdateMemberRoleRequest](r, 1<<20)
	if e != nil || req.RoleID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if e = h.Repository.UpdateMemberRole(r.Context(), user, req.RoleID); e != nil {
		writeRBACError(w, r, e)
		return
	}
	w.WriteHeader(http.StatusOK)
}
func (h OrganizationRBACHandler) RemoveMember(w http.ResponseWriter, r *http.Request) {
	user, userOK := pathUUID(r, "user_id")
	if !userOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	actor, ok := h.require(w, r, "members:remove")
	if !ok {
		return
	}
	if actor == user {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyCannotRemoveSelf))
		return
	}
	if e := h.Repository.RemoveMember(r.Context(), user); e != nil {
		writeRBACError(w, r, e)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}
func (h OrganizationRBACHandler) GetMemberRoles(w http.ResponseWriter, r *http.Request) {
	user, userOK := pathUUID(r, "user_id")
	if !userOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.memberContext(w, r); !ok {
		return
	}
	primary, extra, e := h.Repository.ListMemberRoles(r.Context(), user)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]any{"primary": primary, "extra": extra})
}
func (h OrganizationRBACHandler) AssignExtraRole(w http.ResponseWriter, r *http.Request) {
	user, userOK := pathUUID(r, "user_id")
	if !userOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	req, e := DecodeJSON[AssignRoleRequest](r, 1<<20)
	if e != nil || req.RoleID == uuid.Nil || !req.ExpiresAt.After(time.Now()) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyExpirationFuture))
		return
	}
	x, e := h.Repository.AssignExtraRole(r.Context(), user, req.RoleID, req.ExpiresAt)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, x)
}
func (h OrganizationRBACHandler) RevokeExtraRole(w http.ResponseWriter, r *http.Request) {
	_, userOK := pathUUID(r, "user_id")
	role, roleOK := pathUUID(r, "role_id")
	if !userOK || !roleOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	n, e := h.Repository.RevokeExtraRole(r.Context(), role)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	if n == 0 {
		writeRBACError(w, r, ErrRBACNotFound)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}
func (h OrganizationRBACHandler) GetMemberPermissions(w http.ResponseWriter, r *http.Request) {
	user, userOK := pathUUID(r, "user_id")
	if !userOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.require(w, r, "members:read"); !ok {
		return
	}
	x, e := h.Repository.ListMemberPermissions(r.Context(), user)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, x)
}
func (h OrganizationRBACHandler) AssignExtraPermission(w http.ResponseWriter, r *http.Request) {
	user, userOK := pathUUID(r, "user_id")
	if !userOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	actor, ok := h.require(w, r, "members:roles:manage")
	if !ok {
		return
	}
	req, e := DecodeJSON[AssignPermissionRequest](r, 1<<20)
	if e != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, valid := authpkg.ParsePermission(req.Permission); !valid {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyUnknownPermission))
		return
	}
	if !req.ExpiresAt.After(time.Now()) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyExpirationFuture))
		return
	}
	x, e := h.Repository.AssignExtraPermission(r.Context(), user, req.Permission, req.ExpiresAt, actor)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, x)
}
func (h OrganizationRBACHandler) RevokeExtraPermission(w http.ResponseWriter, r *http.Request) {
	_, userOK := pathUUID(r, "user_id")
	perm, permOK := pathUUID(r, "perm_id")
	if !userOK || !permOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	n, e := h.Repository.RevokeExtraPermission(r.Context(), perm)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	if n == 0 {
		writeRBACError(w, r, ErrRBACNotFound)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}
func (h OrganizationRBACHandler) ListRoles(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.memberContext(w, r); !ok {
		return
	}
	p, e := ParsePagination(r.URL.Query())
	if e != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	x, total, e := h.Repository.ListRoles(r.Context(), p.Page, p.Limit)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, RoleListResponse{Data: x, Total: total, Page: p.Page, Limit: p.Limit})
}
func (h OrganizationRBACHandler) CreateRole(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	req, e := DecodeJSON[CreateRoleRequest](r, 1<<20)
	if e != nil || strings.TrimSpace(req.Name) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.ScopeType == "" {
		req.ScopeType = RoleScopeScoped
	}
	if req.ScopeType != RoleScopeGlobal && req.ScopeType != RoleScopeScoped {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	x, e := h.Repository.CreateRole(r.Context(), req)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, x)
}
func (h OrganizationRBACHandler) UpdateRole(w http.ResponseWriter, r *http.Request) {
	role, roleOK := pathUUID(r, "role_id")
	if !roleOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	req, e := DecodeJSON[UpdateRoleRequest](r, 1<<20)
	if e != nil || req.Name == nil && req.Description == nil && req.Color == nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	x, e := h.Repository.UpdateRole(r.Context(), role, req)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, x)
}
func (h OrganizationRBACHandler) DeleteRole(w http.ResponseWriter, r *http.Request) {
	role, roleOK := pathUUID(r, "role_id")
	if !roleOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	n, e := h.Repository.DeleteRole(r.Context(), role)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	if n == 0 {
		writeRBACError(w, r, ErrRBACProtected)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}
func (h OrganizationRBACHandler) GetRolePermissions(w http.ResponseWriter, r *http.Request) {
	role, roleOK := pathUUID(r, "role_id")
	if !roleOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	x, e := h.Repository.RolePermissions(r.Context(), role)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, x)
}
func (h OrganizationRBACHandler) SetRolePermissions(w http.ResponseWriter, r *http.Request) {
	role, roleOK := pathUUID(r, "role_id")
	if !roleOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	mutable, e := h.Repository.RoleIsMutable(r.Context(), role)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	if !mutable {
		writeRBACError(w, r, ErrRBACProtected)
		return
	}
	req, e := DecodeJSON[UpdateRolePermissionsRequest](r, 1<<20)
	if e != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if e = h.Repository.SetRolePermissions(r.Context(), role, req.Permissions); e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, req.Permissions)
}
func (h OrganizationRBACHandler) PermissionCatalog(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	x := make([]PermissionCatalogItem, 0, len(authpkg.AllPermissions()))
	for _, p := range authpkg.AllPermissions() {
		id := string(p)
		x = append(x, PermissionCatalogItem{ID: id, Group: permissionGroup(id), LabelKey: "permissions.items." + id + ".name"})
	}
	apperror.WriteJSON(w, http.StatusOK, x)
}
func (h OrganizationRBACHandler) AvailablePermissions(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.require(w, r, "members:roles:manage"); !ok {
		return
	}
	x := make([]string, 0, len(authpkg.AllPermissions()))
	for _, p := range authpkg.AllPermissions() {
		x = append(x, string(p))
	}
	apperror.WriteJSON(w, http.StatusOK, x)
}
func permissionGroup(id string) string {
	switch strings.Split(id, ":")[0] {
	case "organization", "approval_workflows":
		return "organization"
	case "members":
		return "members"
	case "employees":
		return "employees"
	case "departments":
		return "departments"
	case "attendance":
		return "attendance"
	case "overtime":
		return "overtime"
	case "leave":
		return "leave"
	case "shifts", "shift_swaps", "patterns", "rosters", "assignment_rules":
		return "shifts"
	case "payroll":
		return "payroll"
	case "holidays":
		return "holidays"
	case "reports", "dashboard":
		return "reports"
	case "documents":
		return "documents"
	case "audit_logs", "security":
		return "security"
	case "feature_flags", "infra":
		return "infra"
	case "recruitment":
		return "recruitment"
	case "employment_phases":
		return "employment_phases"
	case "ess":
		return "ess"
	default:
		return "other"
	}
}

func (h OrganizationRBACHandler) GetStorage(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.memberContext(w, r); !ok {
		return
	}
	x, e := h.Repository.GetStorageConfig(r.Context())
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	if x == nil {
		apperror.WriteJSON(w, http.StatusOK, nil)
		return
	}
	response := x.StorageConfigResponse
	response.AccessKeyMasked = "***"
	apperror.WriteJSON(w, http.StatusOK, response)
}
func (h OrganizationRBACHandler) PutStorage(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.memberContext(w, r); !ok {
		return
	}
	req, e := DecodeJSON[StorageConfigRequest](r, 1<<20)
	if e != nil || strings.TrimSpace(req.EndpointURL) == "" || strings.TrimSpace(req.BucketName) == "" || req.AccessKey == "" || req.SecretKey == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Crypto == nil {
		apperror.WriteError(w, r, apperror.InternalString("storage encryption unavailable"))
		return
	}
	access, e := h.Crypto.Encrypt(req.AccessKey)
	if e != nil {
		apperror.WriteError(w, r, apperror.InternalString("failed to encrypt credentials"))
		return
	}
	secret, e := h.Crypto.Encrypt(req.SecretKey)
	if e != nil {
		apperror.WriteError(w, r, apperror.InternalString("failed to encrypt credentials"))
		return
	}
	rec := StorageConfigRecord{StorageConfigResponse: StorageConfigResponse{ID: uuid.New(), EndpointURL: req.EndpointURL, Region: req.Region, BucketName: req.BucketName, PathPrefix: req.PathPrefix, UsePathStyle: req.UsePathStyle, IsActive: true}, AccessKeyEnc: access, SecretKeyEnc: secret, EncryptionKeyVersion: strings.Split(access, ":")[0], PresignedTTLSecs: req.PresignedTTLSecs, MaxFileSize: req.MaxFileSize}
	x, e := h.Repository.UpsertStorageConfig(r.Context(), rec)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	response := x.StorageConfigResponse
	response.AccessKeyMasked = "***"
	apperror.WriteJSON(w, http.StatusOK, response)
}
func (h OrganizationRBACHandler) DeleteStorage(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.memberContext(w, r); !ok {
		return
	}
	if e := h.Repository.DeleteStorageConfig(r.Context()); e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, struct{}{})
}
func (h OrganizationRBACHandler) TestStorage(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.memberContext(w, r); !ok {
		return
	}
	req, e := DecodeJSON[StorageConfigRequest](r, 1<<20)
	if e != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.StorageTester != nil {
		if e = h.StorageTester.Test(r.Context(), req); e != nil {
			apperror.WriteError(w, r, apperror.Internal(e))
			return
		}
	}
	apperror.WriteJSON(w, http.StatusOK, true)
}
