package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type WorkspaceRBACRepository struct {
	SystemDB DBTX
}

func (r WorkspaceRBACRepository) ListMembers(ctx context.Context, ws uuid.UUID, page, limit int) ([]api.WorkspaceMemberDetail, int64, error) {
	if r.SystemDB == nil {
		return nil, 0, errors.New("nil system db")
	}
	var total int64
	err := r.SystemDB.QueryRow(ctx, `SELECT COUNT(*) FROM workspace_members wm JOIN users u ON u.id = wm.user_id WHERE wm.workspace_id = $1`, ws).Scan(&total)
	if err != nil {
		return nil, 0, err
	}
	offset := (page - 1) * limit
	rows, err := r.SystemDB.Query(ctx, `SELECT wm.workspace_id, wm.user_id, wm.role_id, u.email, u.full_name, u.avatar_url FROM workspace_members wm JOIN users u ON u.id = wm.user_id WHERE wm.workspace_id = $1 ORDER BY wm.user_id LIMIT $2 OFFSET $3`, ws, limit, offset)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	var list []api.WorkspaceMemberDetail
	for rows.Next() {
		var d api.WorkspaceMemberDetail
		if err := rows.Scan(&d.WorkspaceID, &d.UserID, &d.RoleID, &d.Email, &d.FullName, &d.AvatarURL); err != nil {
			return nil, 0, err
		}
		list = append(list, d)
	}
	if list == nil {
		list = []api.WorkspaceMemberDetail{}
	}
	return list, total, rows.Err()
}

func (r WorkspaceRBACRepository) ListExtraRoles(ctx context.Context, ws uuid.UUID) ([]api.MemberExtraRole, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT id, workspace_id, user_id, role_id, expires_at, assigned_at FROM member_extra_roles WHERE workspace_id = $1 AND expires_at > NOW()`, ws)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var list []api.MemberExtraRole
	for rows.Next() {
		var x api.MemberExtraRole
		if err := rows.Scan(&x.ID, &x.WorkspaceID, &x.UserID, &x.RoleID, &x.ExpiresAt, &x.AssignedAt); err != nil {
			return nil, err
		}
		list = append(list, x)
	}
	if list == nil {
		list = []api.MemberExtraRole{}
	}
	return list, rows.Err()
}

func (r WorkspaceRBACRepository) FindUserByEmail(ctx context.Context, email string) (uuid.UUID, error) {
	if r.SystemDB == nil {
		return uuid.Nil, errors.New("nil system db")
	}
	var id uuid.UUID
	err := r.SystemDB.QueryRow(ctx, `SELECT id FROM users WHERE LOWER(email) = LOWER($1)`, email).Scan(&id)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, api.ErrRBACUserNotFound
	}
	return id, err
}

func (r WorkspaceRBACRepository) AddMember(ctx context.Context, ws, user, role uuid.UUID) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	_, err := r.SystemDB.Exec(ctx, `INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3) ON CONFLICT (workspace_id, user_id) DO NOTHING`, ws, user, role)
	return mapConstraintError(err)
}

func (r WorkspaceRBACRepository) FindPrimaryRole(ctx context.Context, ws, user uuid.UUID) (*uuid.UUID, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	var role uuid.UUID
	err := r.SystemDB.QueryRow(ctx, `SELECT role_id FROM workspace_members WHERE workspace_id = $1 AND user_id = $2`, ws, user).Scan(&role)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &role, nil
}

func (r WorkspaceRBACRepository) UpdateMemberRole(ctx context.Context, ws, user, role uuid.UUID) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	tag, err := r.SystemDB.Exec(ctx, `UPDATE workspace_members SET role_id = $1 WHERE workspace_id = $2 AND user_id = $3`, role, ws, user)
	if err != nil {
		return mapConstraintError(err)
	}
	if tag.RowsAffected() == 0 {
		return api.ErrRBACNotFound
	}
	return nil
}

func (r WorkspaceRBACRepository) RemoveMember(ctx context.Context, ws, user uuid.UUID) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	tag, err := r.SystemDB.Exec(ctx, `DELETE FROM workspace_members WHERE workspace_id = $1 AND user_id = $2`, ws, user)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrRBACNotFound
	}
	return nil
}

func (r WorkspaceRBACRepository) ListMemberRoles(ctx context.Context, ws, user uuid.UUID) (*api.WorkspaceRole, []api.MemberExtraRole, error) {
	if r.SystemDB == nil {
		return nil, nil, errors.New("nil system db")
	}
	var primary *api.WorkspaceRole
	var p api.WorkspaceRole
	err := r.SystemDB.QueryRow(ctx, `SELECT r.id, r.workspace_id, r.name, r.description, r.color, r.is_system, r.scope_type, r.created_at, r.updated_at FROM workspace_members wm JOIN workspace_roles r ON r.id = wm.role_id WHERE wm.workspace_id = $1 AND wm.user_id = $2`, ws, user).Scan(&p.ID, &p.WorkspaceID, &p.Name, &p.Description, &p.Color, &p.IsSystem, &p.ScopeType, &p.CreatedAt, &p.UpdatedAt)
	if err == nil {
		primary = &p
	} else if !errors.Is(err, pgx.ErrNoRows) {
		return nil, nil, err
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT id, workspace_id, user_id, role_id, expires_at, assigned_at FROM member_extra_roles WHERE workspace_id = $1 AND user_id = $2 AND expires_at > NOW()`, ws, user)
	if err != nil {
		return primary, nil, err
	}
	defer rows.Close()
	var extras []api.MemberExtraRole
	for rows.Next() {
		var x api.MemberExtraRole
		if err := rows.Scan(&x.ID, &x.WorkspaceID, &x.UserID, &x.RoleID, &x.ExpiresAt, &x.AssignedAt); err != nil {
			return nil, nil, err
		}
		extras = append(extras, x)
	}
	if extras == nil {
		extras = []api.MemberExtraRole{}
	}
	return primary, extras, rows.Err()
}

func (r WorkspaceRBACRepository) AssignExtraRole(ctx context.Context, ws, user, role uuid.UUID, expires time.Time) (api.MemberExtraRole, error) {
	if r.SystemDB == nil {
		return api.MemberExtraRole{}, errors.New("nil system db")
	}
	id := uuid.New()
	now := time.Now().UTC()
	_, err := r.SystemDB.Exec(ctx, `INSERT INTO member_extra_roles (id, workspace_id, user_id, role_id, expires_at, assigned_at) VALUES ($1, $2, $3, $4, $5, $6)`, id, ws, user, role, expires, now)
	if err != nil {
		return api.MemberExtraRole{}, mapConstraintError(err)
	}
	return api.MemberExtraRole{ID: id, WorkspaceID: ws, UserID: user, RoleID: role, ExpiresAt: expires, AssignedAt: &now}, nil
}

func (r WorkspaceRBACRepository) RevokeExtraRole(ctx context.Context, ws, extraRoleID uuid.UUID) (int64, error) {
	if r.SystemDB == nil {
		return 0, errors.New("nil system db")
	}
	tag, err := r.SystemDB.Exec(ctx, `DELETE FROM member_extra_roles WHERE id = $1 AND workspace_id = $2`, extraRoleID, ws)
	return tag.RowsAffected(), err
}

func (r WorkspaceRBACRepository) ListMemberPermissions(ctx context.Context, ws, user uuid.UUID) ([]api.MemberExtraPermission, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT id, workspace_id, user_id, permission, assigned_at, expires_at, assigned_by FROM workspace_member_extra_permissions WHERE workspace_id = $1 AND user_id = $2 AND expires_at > NOW() ORDER BY assigned_at DESC`, ws, user)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var perms []api.MemberExtraPermission
	for rows.Next() {
		var p api.MemberExtraPermission
		if err := rows.Scan(&p.ID, &p.WorkspaceID, &p.UserID, &p.Permission, &p.AssignedAt, &p.ExpiresAt, &p.AssignedBy); err != nil {
			return nil, err
		}
		perms = append(perms, p)
	}
	if perms == nil {
		perms = []api.MemberExtraPermission{}
	}
	return perms, rows.Err()
}

func (r WorkspaceRBACRepository) AssignExtraPermission(ctx context.Context, ws, user uuid.UUID, perm string, expires time.Time, actor uuid.UUID) (api.MemberExtraPermission, error) {
	if r.SystemDB == nil {
		return api.MemberExtraPermission{}, errors.New("nil system db")
	}
	id := uuid.New()
	now := time.Now().UTC()
	_, err := r.SystemDB.Exec(ctx, `INSERT INTO workspace_member_extra_permissions (id, workspace_id, user_id, permission, expires_at, assigned_by, assigned_at) VALUES ($1, $2, $3, $4, $5, $6, $7)`, id, ws, user, perm, expires, actor, now)
	if err != nil {
		return api.MemberExtraPermission{}, mapConstraintError(err)
	}
	return api.MemberExtraPermission{ID: id, WorkspaceID: ws, UserID: user, Permission: perm, ExpiresAt: expires, AssignedBy: &actor, AssignedAt: &now}, nil
}

func (r WorkspaceRBACRepository) RevokeExtraPermission(ctx context.Context, ws, permID uuid.UUID) (int64, error) {
	if r.SystemDB == nil {
		return 0, errors.New("nil system db")
	}
	tag, err := r.SystemDB.Exec(ctx, `DELETE FROM workspace_member_extra_permissions WHERE id = $1 AND workspace_id = $2`, permID, ws)
	return tag.RowsAffected(), err
}

func (r WorkspaceRBACRepository) ListRoles(ctx context.Context, ws uuid.UUID, page, limit int) ([]api.WorkspaceRole, int64, error) {
	if r.SystemDB == nil {
		return nil, 0, errors.New("nil system db")
	}
	var total int64
	err := r.SystemDB.QueryRow(ctx, `SELECT COUNT(*) FROM workspace_roles WHERE workspace_id = $1 OR is_system = true`, ws).Scan(&total)
	if err != nil {
		return nil, 0, err
	}
	offset := (page - 1) * limit
	rows, err := r.SystemDB.Query(ctx, `SELECT id, workspace_id, name, description, color, is_system, scope_type, created_at, updated_at FROM workspace_roles WHERE workspace_id = $1 OR is_system = true ORDER BY is_system DESC, created_at ASC LIMIT $2 OFFSET $3`, ws, limit, offset)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	var roles []api.WorkspaceRole
	for rows.Next() {
		var role api.WorkspaceRole
		if err := rows.Scan(&role.ID, &role.WorkspaceID, &role.Name, &role.Description, &role.Color, &role.IsSystem, &role.ScopeType, &role.CreatedAt, &role.UpdatedAt); err != nil {
			return nil, 0, err
		}
		roles = append(roles, role)
	}
	if roles == nil {
		roles = []api.WorkspaceRole{}
	}
	return roles, total, rows.Err()
}

func (r WorkspaceRBACRepository) CreateRole(ctx context.Context, ws uuid.UUID, req api.CreateWorkspaceRoleRequest) (api.WorkspaceRole, error) {
	if r.SystemDB == nil {
		return api.WorkspaceRole{}, errors.New("nil system db")
	}
	id := uuid.New()
	now := time.Now().UTC()
	var res api.WorkspaceRole
	err := r.SystemDB.QueryRow(ctx, `INSERT INTO workspace_roles (id, workspace_id, name, description, color, is_system, scope_type, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, false, $6, $7, $8) RETURNING id, workspace_id, name, description, color, is_system, scope_type, created_at, updated_at`, id, ws, req.Name, req.Description, req.Color, req.ScopeType, now, now).Scan(&res.ID, &res.WorkspaceID, &res.Name, &res.Description, &res.Color, &res.IsSystem, &res.ScopeType, &res.CreatedAt, &res.UpdatedAt)
	return res, mapConstraintError(err)
}

func (r WorkspaceRBACRepository) UpdateRole(ctx context.Context, ws, role uuid.UUID, req api.UpdateWorkspaceRoleRequest) (api.WorkspaceRole, error) {
	if r.SystemDB == nil {
		return api.WorkspaceRole{}, errors.New("nil system db")
	}
	sets := []string{}
	args := []any{}
	add := func(col string, val any) {
		args = append(args, val)
		sets = append(sets, fmt.Sprintf("%s = $%d", col, len(args)))
	}
	if req.Name != nil {
		add("name", *req.Name)
	}
	if req.Description != nil {
		add("description", *req.Description)
	}
	if req.Color != nil {
		add("color", *req.Color)
	}
	add("updated_at", time.Now().UTC())
	args = append(args, role, ws)
	query := fmt.Sprintf(`UPDATE workspace_roles SET %s WHERE id = $%d AND workspace_id = $%d AND is_system = false RETURNING id, workspace_id, name, description, color, is_system, scope_type, created_at, updated_at`, strings.Join(sets, ", "), len(args)-1, len(args))
	var res api.WorkspaceRole
	err := r.SystemDB.QueryRow(ctx, query, args...).Scan(&res.ID, &res.WorkspaceID, &res.Name, &res.Description, &res.Color, &res.IsSystem, &res.ScopeType, &res.CreatedAt, &res.UpdatedAt)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.WorkspaceRole{}, api.ErrRBACProtected
	}
	return res, mapConstraintError(err)
}

func (r WorkspaceRBACRepository) DeleteRole(ctx context.Context, ws, role uuid.UUID) (int64, error) {
	if r.SystemDB == nil {
		return 0, errors.New("nil system db")
	}
	tag, err := r.SystemDB.Exec(ctx, `DELETE FROM workspace_roles WHERE id = $1 AND workspace_id = $2 AND is_system = false`, role, ws)
	return tag.RowsAffected(), err
}

func (r WorkspaceRBACRepository) RolePermissions(ctx context.Context, ws, role uuid.UUID) ([]string, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT wrp.permission_name FROM workspace_role_permissions wrp JOIN workspace_roles wr ON wr.id = wrp.role_id WHERE wrp.role_id = $1 AND (wr.workspace_id = $2 OR wr.is_system = true)`, role, ws)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var list []string
	for rows.Next() {
		var p string
		if err := rows.Scan(&p); err != nil {
			return nil, err
		}
		list = append(list, p)
	}
	if list == nil {
		list = []string{}
	}
	return list, rows.Err()
}

func (r WorkspaceRBACRepository) RoleIsMutable(ctx context.Context, ws, role uuid.UUID) (bool, error) {
	if r.SystemDB == nil {
		return false, errors.New("nil system db")
	}
	var exists bool
	err := r.SystemDB.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM workspace_roles WHERE id = $1 AND workspace_id = $2 AND is_system = false)`, role, ws).Scan(&exists)
	return exists, err
}

func (r WorkspaceRBACRepository) SetRolePermissions(ctx context.Context, role uuid.UUID, permissions []string) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	if _, err := r.SystemDB.Exec(ctx, `DELETE FROM workspace_role_permissions WHERE role_id = $1`, role); err != nil {
		return err
	}
	for _, perm := range permissions {
		if strings.TrimSpace(perm) == "" {
			continue
		}
		if _, err := r.SystemDB.Exec(ctx, `INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ($1, $2) ON CONFLICT (role_id, permission_name) DO NOTHING`, role, perm); err != nil {
			return err
		}
	}
	return nil
}

func (r WorkspaceRBACRepository) GetStorageConfig(ctx context.Context, ws uuid.UUID) (*api.StorageConfigRecord, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	var rec api.StorageConfigRecord
	err := r.SystemDB.QueryRow(ctx, `SELECT id, workspace_id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active FROM storage_configs WHERE workspace_id = $1`, ws).Scan(&rec.ID, &rec.WorkspaceID, &rec.EndpointURL, &rec.Region, &rec.BucketName, &rec.PathPrefix, &rec.AccessKeyEnc, &rec.SecretKeyEnc, &rec.EncryptionKeyVersion, &rec.UsePathStyle, &rec.PresignedTTLSecs, &rec.MaxFileSize, &rec.IsActive)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &rec, nil
}

func (r WorkspaceRBACRepository) UpsertStorageConfig(ctx context.Context, ws uuid.UUID, rec api.StorageConfigRecord) (*api.StorageConfigRecord, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	id := rec.ID
	if id == uuid.Nil {
		id = uuid.New()
	}
	var res api.StorageConfigRecord
	err := r.SystemDB.QueryRow(ctx, `INSERT INTO storage_configs (id, workspace_id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
ON CONFLICT (workspace_id) DO UPDATE SET
endpoint_url = EXCLUDED.endpoint_url,
region = EXCLUDED.region,
bucket_name = EXCLUDED.bucket_name,
path_prefix = EXCLUDED.path_prefix,
access_key_enc = EXCLUDED.access_key_enc,
secret_key_enc = EXCLUDED.secret_key_enc,
encryption_key_version = EXCLUDED.encryption_key_version,
use_path_style = EXCLUDED.use_path_style,
presigned_ttl_secs = EXCLUDED.presigned_ttl_secs,
max_file_size = EXCLUDED.max_file_size,
is_active = EXCLUDED.is_active,
updated_at = NOW()
RETURNING id, workspace_id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active`, id, ws, rec.EndpointURL, rec.Region, rec.BucketName, rec.PathPrefix, rec.AccessKeyEnc, rec.SecretKeyEnc, rec.EncryptionKeyVersion, rec.UsePathStyle, rec.PresignedTTLSecs, rec.MaxFileSize, rec.IsActive).Scan(&res.ID, &res.WorkspaceID, &res.EndpointURL, &res.Region, &res.BucketName, &res.PathPrefix, &res.AccessKeyEnc, &res.SecretKeyEnc, &res.EncryptionKeyVersion, &res.UsePathStyle, &res.PresignedTTLSecs, &res.MaxFileSize, &res.IsActive)
	if err != nil {
		return nil, mapConstraintError(err)
	}
	return &res, nil
}

func (r WorkspaceRBACRepository) DeleteStorageConfig(ctx context.Context, ws uuid.UUID) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	_, err := r.SystemDB.Exec(ctx, `DELETE FROM storage_configs WHERE workspace_id = $1`, ws)
	return err
}
