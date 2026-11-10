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
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type OrganizationRBACRepository struct {
	SystemDB DBTX
	BeginTx  func(context.Context) (TxBeginner, error)
}

func (r OrganizationRBACRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	if r.SystemDB == nil {
		return nil, errors.New("organization rbac repository: system database is not configured")
	}
	beginner, ok := r.SystemDB.(TxBeginner)
	if !ok {
		return nil, errors.New("organization rbac repository: system database does not support transactions")
	}
	return beginner, nil
}

func (r OrganizationRBACRepository) withTx(ctx context.Context, fn func(ctx context.Context, tx DBTX) error) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	if tx, ok := r.SystemDB.(pgx.Tx); ok {
		return fn(ctx, tx)
	}
	beginner, err := r.begin(ctx)
	if err == nil {
		return service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
			return fn(ctx, tx)
		})
	}
	return fn(ctx, r.SystemDB)
}

func (r OrganizationRBACRepository) ListMembers(ctx context.Context, page, limit int) ([]api.MemberDetail, int64, error) {
	if r.SystemDB == nil {
		return nil, 0, errors.New("nil system db")
	}
	var total int64
	err := r.SystemDB.QueryRow(ctx, `SELECT COUNT(*) FROM user_roles ur JOIN users u ON u.id = ur.user_id`).Scan(&total)
	if err != nil {
		return nil, 0, err
	}
	offset := (page - 1) * limit
	if offset < 0 {
		offset = 0
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT ur.user_id, ur.role_id, u.email, u.full_name, u.avatar_url FROM user_roles ur JOIN users u ON u.id = ur.user_id ORDER BY ur.user_id LIMIT $1 OFFSET $2`, limit, offset)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	var list []api.MemberDetail
	for rows.Next() {
		var d api.MemberDetail
		if err := rows.Scan(&d.UserID, &d.RoleID, &d.Email, &d.FullName, &d.AvatarURL); err != nil {
			return nil, 0, err
		}
		list = append(list, d)
	}
	if list == nil {
		list = []api.MemberDetail{}
	}
	return list, total, rows.Err()
}

func (r OrganizationRBACRepository) ListExtraRoles(ctx context.Context) ([]api.MemberExtraRole, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT id, user_id, role_id, expires_at, assigned_at FROM user_extra_roles WHERE expires_at > NOW()`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var list []api.MemberExtraRole
	for rows.Next() {
		var x api.MemberExtraRole
		if err := rows.Scan(&x.ID, &x.UserID, &x.RoleID, &x.ExpiresAt, &x.AssignedAt); err != nil {
			return nil, err
		}
		list = append(list, x)
	}
	if list == nil {
		list = []api.MemberExtraRole{}
	}
	return list, rows.Err()
}

func (r OrganizationRBACRepository) FindUserByEmail(ctx context.Context, email string) (uuid.UUID, error) {
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

func (r OrganizationRBACRepository) AddMember(ctx context.Context, user, role uuid.UUID) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	_, err := r.SystemDB.Exec(ctx, `INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT (user_id) DO NOTHING`, user, role)
	return mapConstraintError(err)
}

func (r OrganizationRBACRepository) FindPrimaryRole(ctx context.Context, user uuid.UUID) (*uuid.UUID, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	var role uuid.UUID
	err := r.SystemDB.QueryRow(ctx, `SELECT role_id FROM user_roles WHERE user_id = $1`, user).Scan(&role)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &role, nil
}

func (r OrganizationRBACRepository) UpdateMemberRole(ctx context.Context, user, role uuid.UUID) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	return r.withTx(ctx, func(ctx context.Context, tx DBTX) error {
		var newScope, newName string
		err := tx.QueryRow(ctx, `SELECT scope_type, name FROM roles WHERE id = $1`, role).Scan(&newScope, &newName)
		if err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrRBACNotFound
			}
			return mapConstraintError(err)
		}

		var currentRoleID uuid.UUID
		var curScope, curName string
		err = tx.QueryRow(ctx, `SELECT ur.role_id, r.scope_type, r.name FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1`, user).Scan(&currentRoleID, &curScope, &curName)
		if err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrRBACNotFound
			}
			return err
		}

		if currentRoleID == role {
			return nil
		}

		isCurGlobal := curScope == string(api.RoleScopeGlobal) || curName == "admin"
		isNewGlobal := newScope == string(api.RoleScopeGlobal) || newName == "admin"
		if isCurGlobal && !isNewGlobal {
			var hasExtraGlobal bool
			_ = tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM user_extra_roles uer JOIN roles r ON r.id = uer.role_id WHERE uer.user_id = $1 AND (r.scope_type = 'global' OR r.name = 'admin') AND uer.expires_at > NOW())`, user).Scan(&hasExtraGlobal)
			if !hasExtraGlobal {
				var globalAdminCount int64
				err = tx.QueryRow(ctx, `SELECT COUNT(DISTINCT user_id) FROM (
					SELECT ur.user_id FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE r.scope_type = 'global' OR r.name = 'admin'
					UNION
					SELECT uer.user_id FROM user_extra_roles uer JOIN roles r ON r.id = uer.role_id WHERE (r.scope_type = 'global' OR r.name = 'admin') AND uer.expires_at > NOW()
				) admins`).Scan(&globalAdminCount)
				if err != nil {
					return err
				}
				if globalAdminCount <= 1 {
					return api.ErrRBACLastAdmin
				}
			}
		}

		tag, err := tx.Exec(ctx, `UPDATE user_roles SET role_id = $1 WHERE user_id = $2`, role, user)
		if err != nil {
			return mapConstraintError(err)
		}
		if tag.RowsAffected() == 0 {
			return api.ErrRBACNotFound
		}
		return nil
	})
}

func (r OrganizationRBACRepository) RemoveMember(ctx context.Context, user uuid.UUID) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	return r.withTx(ctx, func(ctx context.Context, tx DBTX) error {
		var curScope, curName string
		err := tx.QueryRow(ctx, `SELECT r.scope_type, r.name FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1`, user).Scan(&curScope, &curName)
		if err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrRBACNotFound
			}
			return err
		}

		var hasExtraGlobal bool
		_ = tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM user_extra_roles uer JOIN roles r ON r.id = uer.role_id WHERE uer.user_id = $1 AND (r.scope_type = 'global' OR r.name = 'admin') AND uer.expires_at > NOW())`, user).Scan(&hasExtraGlobal)

		isGlobal := curScope == string(api.RoleScopeGlobal) || curName == "admin" || hasExtraGlobal
		if isGlobal {
			var globalAdminCount int64
			err = tx.QueryRow(ctx, `SELECT COUNT(DISTINCT user_id) FROM (
				SELECT ur.user_id FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE r.scope_type = 'global' OR r.name = 'admin'
				UNION
				SELECT uer.user_id FROM user_extra_roles uer JOIN roles r ON r.id = uer.role_id WHERE (r.scope_type = 'global' OR r.name = 'admin') AND uer.expires_at > NOW()
			) admins`).Scan(&globalAdminCount)
			if err != nil {
				return err
			}
			if globalAdminCount <= 1 {
				return api.ErrRBACLastAdmin
			}
		}

		if _, err := tx.Exec(ctx, `DELETE FROM user_extra_roles WHERE user_id = $1`, user); err != nil {
			return err
		}
		if _, err := tx.Exec(ctx, `DELETE FROM user_extra_permissions WHERE user_id = $1`, user); err != nil {
			return err
		}
		tag, err := tx.Exec(ctx, `DELETE FROM user_roles WHERE user_id = $1`, user)
		if err != nil {
			return err
		}
		if tag.RowsAffected() == 0 {
			return api.ErrRBACNotFound
		}
		return nil
	})
}

func (r OrganizationRBACRepository) ListMemberRoles(ctx context.Context, user uuid.UUID) (*api.Role, []api.MemberExtraRole, error) {
	if r.SystemDB == nil {
		return nil, nil, errors.New("nil system db")
	}
	var primary *api.Role
	var p api.Role
	err := r.SystemDB.QueryRow(ctx, `SELECT r.id, r.name, r.description, r.color, r.is_system, r.scope_type, r.created_at, r.updated_at FROM user_roles ur JOIN roles r ON r.id = ur.role_id WHERE ur.user_id = $1`, user).Scan(&p.ID, &p.Name, &p.Description, &p.Color, &p.IsSystem, &p.ScopeType, &p.CreatedAt, &p.UpdatedAt)
	if err == nil {
		primary = &p
	} else if !errors.Is(err, pgx.ErrNoRows) {
		return nil, nil, err
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT id, user_id, role_id, expires_at, assigned_at FROM user_extra_roles WHERE user_id = $1 AND expires_at > NOW()`, user)
	if err != nil {
		return primary, nil, err
	}
	defer rows.Close()
	var extras []api.MemberExtraRole
	for rows.Next() {
		var x api.MemberExtraRole
		if err := rows.Scan(&x.ID, &x.UserID, &x.RoleID, &x.ExpiresAt, &x.AssignedAt); err != nil {
			return nil, nil, err
		}
		extras = append(extras, x)
	}
	if extras == nil {
		extras = []api.MemberExtraRole{}
	}
	return primary, extras, rows.Err()
}

func (r OrganizationRBACRepository) AssignExtraRole(ctx context.Context, user, role uuid.UUID, expires time.Time) (api.MemberExtraRole, error) {
	if r.SystemDB == nil {
		return api.MemberExtraRole{}, errors.New("nil system db")
	}
	id := uuid.New()
	now := time.Now().UTC()
	_, err := r.SystemDB.Exec(ctx, `INSERT INTO user_extra_roles (id, user_id, role_id, expires_at, assigned_at) VALUES ($1, $2, $3, $4, $5)`, id, user, role, expires, now)
	if err != nil {
		return api.MemberExtraRole{}, mapConstraintError(err)
	}
	return api.MemberExtraRole{ID: id, UserID: user, RoleID: role, ExpiresAt: expires, AssignedAt: &now}, nil
}

func (r OrganizationRBACRepository) RevokeExtraRole(ctx context.Context, extraRoleID uuid.UUID) (int64, error) {
	if r.SystemDB == nil {
		return 0, errors.New("nil system db")
	}
	tag, err := r.SystemDB.Exec(ctx, `DELETE FROM user_extra_roles WHERE id = $1`, extraRoleID)
	return tag.RowsAffected(), err
}

func (r OrganizationRBACRepository) ListMemberPermissions(ctx context.Context, user uuid.UUID) ([]api.MemberExtraPermission, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT id, user_id, permission, assigned_at, expires_at, assigned_by FROM user_extra_permissions WHERE user_id = $1 AND expires_at > NOW() ORDER BY assigned_at DESC`, user)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var perms []api.MemberExtraPermission
	for rows.Next() {
		var p api.MemberExtraPermission
		if err := rows.Scan(&p.ID, &p.UserID, &p.Permission, &p.AssignedAt, &p.ExpiresAt, &p.AssignedBy); err != nil {
			return nil, err
		}
		perms = append(perms, p)
	}
	if perms == nil {
		perms = []api.MemberExtraPermission{}
	}
	return perms, rows.Err()
}

func (r OrganizationRBACRepository) AssignExtraPermission(ctx context.Context, user uuid.UUID, perm string, expires time.Time, actor uuid.UUID) (api.MemberExtraPermission, error) {
	if r.SystemDB == nil {
		return api.MemberExtraPermission{}, errors.New("nil system db")
	}
	id := uuid.New()
	now := time.Now().UTC()
	var assignedBy *uuid.UUID
	if actor != uuid.Nil {
		assignedBy = &actor
	}
	_, err := r.SystemDB.Exec(ctx, `INSERT INTO user_extra_permissions (id, user_id, permission, expires_at, assigned_by, assigned_at) VALUES ($1, $2, $3, $4, $5, $6)`, id, user, perm, expires, assignedBy, now)
	if err != nil {
		return api.MemberExtraPermission{}, mapConstraintError(err)
	}
	return api.MemberExtraPermission{ID: id, UserID: user, Permission: perm, ExpiresAt: expires, AssignedBy: assignedBy, AssignedAt: &now}, nil
}

func (r OrganizationRBACRepository) RevokeExtraPermission(ctx context.Context, permID uuid.UUID) (int64, error) {
	if r.SystemDB == nil {
		return 0, errors.New("nil system db")
	}
	tag, err := r.SystemDB.Exec(ctx, `DELETE FROM user_extra_permissions WHERE id = $1`, permID)
	return tag.RowsAffected(), err
}

func (r OrganizationRBACRepository) ListRoles(ctx context.Context, page, limit int) ([]api.Role, int64, error) {
	if r.SystemDB == nil {
		return nil, 0, errors.New("nil system db")
	}
	var total int64
	err := r.SystemDB.QueryRow(ctx, `SELECT COUNT(*) FROM roles`).Scan(&total)
	if err != nil {
		return nil, 0, err
	}
	offset := (page - 1) * limit
	if offset < 0 {
		offset = 0
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT id, name, description, color, is_system, scope_type, created_at, updated_at FROM roles ORDER BY is_system DESC, created_at ASC LIMIT $1 OFFSET $2`, limit, offset)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	var roles []api.Role
	for rows.Next() {
		var role api.Role
		if err := rows.Scan(&role.ID, &role.Name, &role.Description, &role.Color, &role.IsSystem, &role.ScopeType, &role.CreatedAt, &role.UpdatedAt); err != nil {
			return nil, 0, err
		}
		roles = append(roles, role)
	}
	if roles == nil {
		roles = []api.Role{}
	}
	return roles, total, rows.Err()
}

func (r OrganizationRBACRepository) CreateRole(ctx context.Context, req api.CreateRoleRequest) (api.Role, error) {
	if r.SystemDB == nil {
		return api.Role{}, errors.New("nil system db")
	}
	id := uuid.New()
	now := time.Now().UTC()
	scopeType := req.ScopeType
	if scopeType == "" {
		scopeType = api.RoleScopeScoped
	}
	var res api.Role
	err := r.SystemDB.QueryRow(ctx, `INSERT INTO roles (id, name, description, color, is_system, scope_type, created_at, updated_at) VALUES ($1, $2, $3, $4, false, $5, $6, $7) RETURNING id, name, description, color, is_system, scope_type, created_at, updated_at`, id, req.Name, req.Description, req.Color, scopeType, now, now).Scan(&res.ID, &res.Name, &res.Description, &res.Color, &res.IsSystem, &res.ScopeType, &res.CreatedAt, &res.UpdatedAt)
	return res, mapConstraintError(err)
}

func (r OrganizationRBACRepository) UpdateRole(ctx context.Context, role uuid.UUID, req api.UpdateRoleRequest) (api.Role, error) {
	if r.SystemDB == nil {
		return api.Role{}, errors.New("nil system db")
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
	args = append(args, role)
	query := fmt.Sprintf(`UPDATE roles SET %s WHERE id = $%d AND is_system = false RETURNING id, name, description, color, is_system, scope_type, created_at, updated_at`, strings.Join(sets, ", "), len(args))
	var res api.Role
	err := r.SystemDB.QueryRow(ctx, query, args...).Scan(&res.ID, &res.Name, &res.Description, &res.Color, &res.IsSystem, &res.ScopeType, &res.CreatedAt, &res.UpdatedAt)
	if errors.Is(err, pgx.ErrNoRows) {
		var isSystem bool
		if checkErr := r.SystemDB.QueryRow(ctx, `SELECT is_system FROM roles WHERE id = $1`, role).Scan(&isSystem); checkErr == nil && isSystem {
			return api.Role{}, api.ErrRBACProtected
		}
		return api.Role{}, api.ErrRBACNotFound
	}
	return res, mapConstraintError(err)
}

func (r OrganizationRBACRepository) DeleteRole(ctx context.Context, role uuid.UUID) (int64, error) {
	if r.SystemDB == nil {
		return 0, errors.New("nil system db")
	}
	tag, err := r.SystemDB.Exec(ctx, `DELETE FROM roles WHERE id = $1 AND is_system = false`, role)
	return tag.RowsAffected(), err
}

func (r OrganizationRBACRepository) RolePermissions(ctx context.Context, role uuid.UUID) ([]string, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	rows, err := r.SystemDB.Query(ctx, `SELECT rp.permission_name FROM role_permissions rp WHERE rp.role_id = $1 ORDER BY rp.permission_name`, role)
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

func (r OrganizationRBACRepository) RoleIsMutable(ctx context.Context, role uuid.UUID) (bool, error) {
	if r.SystemDB == nil {
		return false, errors.New("nil system db")
	}
	var exists bool
	err := r.SystemDB.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM roles WHERE id = $1 AND is_system = false)`, role).Scan(&exists)
	return exists, err
}

func (r OrganizationRBACRepository) SetRolePermissions(ctx context.Context, role uuid.UUID, permissions []string) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	return r.withTx(ctx, func(ctx context.Context, tx DBTX) error {
		var isSystem bool
		err := tx.QueryRow(ctx, `SELECT is_system FROM roles WHERE id = $1`, role).Scan(&isSystem)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrRBACNotFound
		}
		if err != nil {
			return err
		}
		if isSystem {
			return api.ErrRBACProtected
		}

		if _, err := tx.Exec(ctx, `DELETE FROM role_permissions WHERE role_id = $1`, role); err != nil {
			return err
		}
		for _, perm := range permissions {
			clean := strings.TrimSpace(perm)
			if clean == "" {
				continue
			}
			if _, err := tx.Exec(ctx, `INSERT INTO role_permissions (role_id, permission_name) VALUES ($1, $2) ON CONFLICT (role_id, permission_name) DO NOTHING`, role, clean); err != nil {
				return err
			}
		}
		return nil
	})
}

func (r OrganizationRBACRepository) GetStorageConfig(ctx context.Context) (*api.StorageConfigRecord, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	var rec api.StorageConfigRecord
	err := r.SystemDB.QueryRow(ctx, `SELECT id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active FROM storage_configs LIMIT 1`).Scan(&rec.ID, &rec.EndpointURL, &rec.Region, &rec.BucketName, &rec.PathPrefix, &rec.AccessKeyEnc, &rec.SecretKeyEnc, &rec.EncryptionKeyVersion, &rec.UsePathStyle, &rec.PresignedTTLSecs, &rec.MaxFileSize, &rec.IsActive)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &rec, nil
}

func (r OrganizationRBACRepository) UpsertStorageConfig(ctx context.Context, rec api.StorageConfigRecord) (*api.StorageConfigRecord, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	id := rec.ID
	if id == uuid.Nil {
		id = uuid.New()
	}
	var res api.StorageConfigRecord
	err := r.SystemDB.QueryRow(ctx, `UPDATE storage_configs SET
endpoint_url = $1,
region = $2,
bucket_name = $3,
path_prefix = $4,
access_key_enc = $5,
secret_key_enc = $6,
encryption_key_version = $7,
use_path_style = $8,
presigned_ttl_secs = $9,
max_file_size = $10,
is_active = $11,
updated_at = NOW()
RETURNING id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active`,
		rec.EndpointURL, rec.Region, rec.BucketName, rec.PathPrefix, rec.AccessKeyEnc, rec.SecretKeyEnc, rec.EncryptionKeyVersion, rec.UsePathStyle, rec.PresignedTTLSecs, rec.MaxFileSize, rec.IsActive).Scan(&res.ID, &res.EndpointURL, &res.Region, &res.BucketName, &res.PathPrefix, &res.AccessKeyEnc, &res.SecretKeyEnc, &res.EncryptionKeyVersion, &res.UsePathStyle, &res.PresignedTTLSecs, &res.MaxFileSize, &res.IsActive)
	if err == nil {
		return &res, nil
	}
	if !errors.Is(err, pgx.ErrNoRows) {
		return nil, mapConstraintError(err)
	}

	err = r.SystemDB.QueryRow(ctx, `INSERT INTO storage_configs (id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
RETURNING id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active`,
		id, rec.EndpointURL, rec.Region, rec.BucketName, rec.PathPrefix, rec.AccessKeyEnc, rec.SecretKeyEnc, rec.EncryptionKeyVersion, rec.UsePathStyle, rec.PresignedTTLSecs, rec.MaxFileSize, rec.IsActive).Scan(&res.ID, &res.EndpointURL, &res.Region, &res.BucketName, &res.PathPrefix, &res.AccessKeyEnc, &res.SecretKeyEnc, &res.EncryptionKeyVersion, &res.UsePathStyle, &res.PresignedTTLSecs, &res.MaxFileSize, &res.IsActive)
	if err != nil {
		return nil, mapConstraintError(err)
	}
	return &res, nil
}

func (r OrganizationRBACRepository) DeleteStorageConfig(ctx context.Context) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	_, err := r.SystemDB.Exec(ctx, `DELETE FROM storage_configs`)
	return err
}
