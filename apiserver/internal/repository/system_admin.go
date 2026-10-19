package repository

import (
	"context"
	"encoding/json"
	"errors"
	"net/url"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

var (
	ErrSystemRoleNotFound = api.ErrSystemRoleNotFound
	ErrSystemRoleUnknown  = api.ErrSystemRoleUnknown
	ErrLastPlatformAdmin  = api.ErrLastPlatformAdmin
	ErrTenantNotFound     = api.ErrTenantNotFound
	ErrClusterNotFound    = api.ErrClusterNotFound
)

type SystemAdminRepository struct{ DB DBTX }

func (r SystemAdminRepository) ListSystemRoles(ctx context.Context) ([]api.SystemRoleInfo, error) {
	if r.DB == nil {
		return nil, errors.New("system admin repository: nil database")
	}
	rows, err := r.DB.Query(ctx, `SELECT id, name, description FROM system_roles ORDER BY name`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	type role struct {
		id, name, description string
	}
	roles := make([]role, 0)
	ids := make([]uuid.UUID, 0)
	for rows.Next() {
		var item role
		var id uuid.UUID
		if err := rows.Scan(&id, &item.name, &item.description); err != nil {
			return nil, err
		}
		item.id = id.String()
		ids = append(ids, id)
		roles = append(roles, item)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	assignees := make(map[uuid.UUID][]api.SystemRoleAssignment)
	if len(ids) > 0 {
		assignmentRows, err := r.DB.Query(ctx, `SELECT role_id, user_id, assigned_by, assigned_at FROM user_system_roles WHERE role_id = ANY($1) ORDER BY assigned_at`, ids)
		if err != nil {
			return nil, err
		}
		defer assignmentRows.Close()
		for assignmentRows.Next() {
			var roleID, userID uuid.UUID
			var assignedBy *uuid.UUID
			var assignedAt time.Time
			if err := assignmentRows.Scan(&roleID, &userID, &assignedBy, &assignedAt); err != nil {
				return nil, err
			}
			assignees[roleID] = append(assignees[roleID], api.SystemRoleAssignment{
				UserID: userID, RoleName: "", AssignedBy: assignedBy, AssignedAt: assignedAt,
			})
		}
		if err := assignmentRows.Err(); err != nil {
			return nil, err
		}
	}
	result := make([]api.SystemRoleInfo, 0, len(roles))
	for _, item := range roles {
		roleID, err := uuid.Parse(item.id)
		if err != nil {
			return nil, err
		}
		assignments := assignees[roleID]
		for i := range assignments {
			assignments[i].RoleName = item.name
		}
		result = append(result, api.SystemRoleInfo{ID: roleID, Name: item.name, Description: item.description, Assignees: assignments})
	}
	return result, nil
}

func (r SystemAdminRepository) GetUserSystemRoles(ctx context.Context, userID uuid.UUID) ([]string, error) {
	if r.DB == nil {
		return nil, errors.New("system admin repository: nil database")
	}
	rows, err := r.DB.Query(ctx, `SELECT sr.name FROM user_system_roles usr JOIN system_roles sr ON sr.id = usr.role_id WHERE usr.user_id = $1 ORDER BY sr.name`, userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	roles := make([]string, 0)
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return nil, err
		}
		roles = append(roles, name)
	}
	return roles, rows.Err()
}

func (r SystemAdminRepository) AssignSystemRole(ctx context.Context, userID, actorID uuid.UUID, role api.SystemRoleType) error {
	if r.DB == nil {
		return errors.New("system admin repository: nil database")
	}
	var roleID uuid.UUID
	if err := r.DB.QueryRow(ctx, `SELECT id FROM system_roles WHERE name = $1`, string(role)).Scan(&roleID); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return ErrSystemRoleNotFound
		}
		return err
	}
	_, err := r.DB.Exec(ctx, `INSERT INTO user_system_roles (user_id, role_id, assigned_by) VALUES ($1, $2, $3) ON CONFLICT (user_id, role_id) DO NOTHING`, userID, roleID, actorID)
	return err
}

func (r SystemAdminRepository) RevokeSystemRole(ctx context.Context, userID uuid.UUID, role string) error {
	if r.DB == nil {
		return errors.New("system admin repository: nil database")
	}
	parsed, ok := api.ParseSystemRoleType(role)
	if !ok {
		return ErrSystemRoleUnknown
	}
	if parsed == api.SystemRolePlatformAdmin {
		var count int64
		if err := r.DB.QueryRow(ctx, `SELECT COUNT(*) FROM user_system_roles usr JOIN system_roles sr ON sr.id = usr.role_id WHERE sr.name = 'platform_admin'`).Scan(&count); err != nil {
			return err
		}
		if count <= 1 {
			return ErrLastPlatformAdmin
		}
	}
	tag, err := r.DB.Exec(ctx, `DELETE FROM user_system_roles WHERE user_id = $1 AND role_id = (SELECT id FROM system_roles WHERE name = $2)`, userID, role)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return ErrSystemRoleNotFound
	}
	return nil
}

func (r SystemAdminRepository) ListDBClusters(ctx context.Context) ([]api.SystemDBCluster, error) {
	if r.DB == nil {
		return nil, errors.New("system admin repository: nil database")
	}
	rows, err := r.DB.Query(ctx, `SELECT id, name, admin_database_url, region, max_tenants, active_tenants, is_active, created_at, updated_at FROM system_db_clusters ORDER BY created_at DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.SystemDBCluster, 0)
	for rows.Next() {
		cluster, err := scanCluster(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, cluster)
	}
	return result, rows.Err()
}

func (r SystemAdminRepository) CreateDBCluster(ctx context.Context, req api.CreateDBClusterRequest) (api.SystemDBCluster, error) {
	if r.DB == nil {
		return api.SystemDBCluster{}, errors.New("system admin repository: nil database")
	}
	id := uuid.New()
	now := time.Now().UTC()
	var cluster api.SystemDBCluster
	var rawURL string
	err := r.DB.QueryRow(ctx, `INSERT INTO system_db_clusters (id, name, admin_database_url, region, max_tenants, active_tenants, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 0, $6, $7, $7) RETURNING id, name, admin_database_url, region, max_tenants, active_tenants, is_active, created_at, updated_at`, id, req.Name, req.AdminDatabaseURL, req.Region, req.MaxTenants, req.IsActive, now).Scan(&cluster.ID, &cluster.Name, &rawURL, &cluster.Region, &cluster.MaxTenants, &cluster.ActiveTenants, &cluster.IsActive, &cluster.CreatedAt, &cluster.UpdatedAt)
	if err != nil {
		return api.SystemDBCluster{}, err
	}
	cluster.AdminDatabaseURL = maskDatabaseURL(rawURL)
	return cluster, nil
}

func (r SystemAdminRepository) DeleteDBCluster(ctx context.Context, id uuid.UUID) error {
	if r.DB == nil {
		return errors.New("system admin repository: nil database")
	}
	_, err := r.DB.Exec(ctx, `DELETE FROM system_db_clusters WHERE id = $1`, id)
	return err
}

func (r SystemAdminRepository) UpdateDBClusterStatus(ctx context.Context, id uuid.UUID, active bool) (api.SystemDBCluster, error) {
	if r.DB == nil {
		return api.SystemDBCluster{}, errors.New("system admin repository: nil database")
	}
	var cluster api.SystemDBCluster
	var rawURL string
	err := r.DB.QueryRow(ctx, `UPDATE system_db_clusters SET is_active = $1, updated_at = $2 WHERE id = $3 RETURNING id, name, admin_database_url, region, max_tenants, active_tenants, is_active, created_at, updated_at`, active, time.Now().UTC(), id).Scan(&cluster.ID, &cluster.Name, &rawURL, &cluster.Region, &cluster.MaxTenants, &cluster.ActiveTenants, &cluster.IsActive, &cluster.CreatedAt, &cluster.UpdatedAt)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.SystemDBCluster{}, ErrClusterNotFound
	}
	if err != nil {
		return api.SystemDBCluster{}, err
	}
	cluster.AdminDatabaseURL = maskDatabaseURL(rawURL)
	return cluster, nil
}

func (r SystemAdminRepository) Get(ctx context.Context, key string) (json.RawMessage, bool, error) {
	return SystemSettingsRepository(r).Get(ctx, key)
}

func (r SystemAdminRepository) Set(ctx context.Context, key string, value json.RawMessage) error {
	return SystemSettingsRepository(r).Set(ctx, key, value)
}

func (r SystemAdminRepository) ListTenants(ctx context.Context) ([]api.SystemTenant, error) {
	return SystemTenantRepository(r).ListTenants(ctx)
}

func (r SystemAdminRepository) CreateTenant(ctx context.Context, id uuid.UUID, name string, ownerID uuid.UUID, encryptedURL *string) (api.SystemTenant, error) {
	return SystemTenantRepository(r).CreateTenant(ctx, id, name, ownerID, encryptedURL)
}

func (r SystemAdminRepository) GetTenant(ctx context.Context, id uuid.UUID) (api.SystemTenant, error) {
	return SystemTenantRepository(r).GetTenant(ctx, id)
}

func (r SystemAdminRepository) DeleteTenant(ctx context.Context, id uuid.UUID) error {
	return SystemTenantRepository(r).DeleteTenant(ctx, id)
}

func (r SystemAdminRepository) UpdateTenantDatabase(ctx context.Context, id uuid.UUID, encryptedURL *string) error {
	return SystemTenantRepository(r).UpdateTenantDatabase(ctx, id, encryptedURL)
}

func scanCluster(row pgx.Row) (api.SystemDBCluster, error) {
	var cluster api.SystemDBCluster
	var rawURL string
	err := row.Scan(&cluster.ID, &cluster.Name, &rawURL, &cluster.Region, &cluster.MaxTenants, &cluster.ActiveTenants, &cluster.IsActive, &cluster.CreatedAt, &cluster.UpdatedAt)
	cluster.AdminDatabaseURL = maskDatabaseURL(rawURL)
	return cluster, err
}

func maskDatabaseURL(raw string) string {
	parsed, err := url.Parse(raw)
	if err != nil || parsed.Scheme == "" || parsed.Host == "" {
		return "*** REDACTED ***"
	}
	if parsed.User != nil {
		username := parsed.User.Username()
		parsed.User = url.UserPassword(username, "***")
	}
	return parsed.String()
}

// SystemSettingsRepository persists JSON settings in the system database.
type SystemSettingsRepository struct{ DB DBTX }

func (r SystemSettingsRepository) Get(ctx context.Context, key string) (json.RawMessage, bool, error) {
	if r.DB == nil {
		return nil, false, errors.New("system settings repository: nil database")
	}
	var value json.RawMessage
	err := r.DB.QueryRow(ctx, `SELECT value_json FROM system_settings WHERE key = $1`, key).Scan(&value)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, false, nil
	}
	return value, err == nil, err
}

func (r SystemSettingsRepository) Set(ctx context.Context, key string, value json.RawMessage) error {
	if r.DB == nil {
		return errors.New("system settings repository: nil database")
	}
	_, err := r.DB.Exec(ctx, `INSERT INTO system_settings (key, value_json) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value_json = EXCLUDED.value_json, updated_at = NOW()`, key, value)
	return err
}

type SystemTenantRepository struct{ DB DBTX }

func (r SystemTenantRepository) ListTenants(ctx context.Context) ([]api.SystemTenant, error) {
	if r.DB == nil {
		return nil, errors.New("system tenant repository: nil database")
	}
	rows, err := r.DB.Query(ctx, `SELECT id, name, timezone, custom_db_url_encrypted, created_at FROM workspaces ORDER BY created_at DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.SystemTenant, 0)
	for rows.Next() {
		var tenant api.SystemTenant
		if err := rows.Scan(&tenant.ID, &tenant.Name, &tenant.Timezone, &tenant.EncryptedDatabaseURL, &tenant.CreatedAt); err != nil {
			return nil, err
		}
		tenant.IsBYOD = tenant.EncryptedDatabaseURL != nil
		result = append(result, tenant)
	}
	return result, rows.Err()
}

func (r SystemTenantRepository) CreateTenant(ctx context.Context, id uuid.UUID, name string, ownerID uuid.UUID, encryptedURL *string) (api.SystemTenant, error) {
	if r.DB == nil {
		return api.SystemTenant{}, errors.New("system tenant repository: nil database")
	}
	var tenant api.SystemTenant
	err := r.DB.QueryRow(ctx, `INSERT INTO workspaces (id, name, custom_db_url_encrypted) VALUES ($1, $2, $3) RETURNING id, name, timezone, custom_db_url_encrypted, created_at`, id, strings.TrimSpace(name), encryptedURL).Scan(&tenant.ID, &tenant.Name, &tenant.Timezone, &tenant.EncryptedDatabaseURL, &tenant.CreatedAt)
	if err != nil {
		return api.SystemTenant{}, err
	}
	// Owner membership mirrors Rust workspace creation. Default global owner role is stable in system seed.
	if _, err := r.DB.Exec(ctx, `INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3)`, id, ownerID, uuid.MustParse("a0000000-0000-0000-0000-000000000001")); err != nil {
		return api.SystemTenant{}, err
	}
	tenant.IsBYOD = tenant.EncryptedDatabaseURL != nil
	return tenant, nil
}

func (r SystemTenantRepository) GetTenant(ctx context.Context, id uuid.UUID) (api.SystemTenant, error) {
	if r.DB == nil {
		return api.SystemTenant{}, errors.New("system tenant repository: nil database")
	}
	var tenant api.SystemTenant
	err := r.DB.QueryRow(ctx, `SELECT id, name, timezone, custom_db_url_encrypted, created_at FROM workspaces WHERE id = $1`, id).Scan(&tenant.ID, &tenant.Name, &tenant.Timezone, &tenant.EncryptedDatabaseURL, &tenant.CreatedAt)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.SystemTenant{}, ErrTenantNotFound
	}
	if err != nil {
		return api.SystemTenant{}, err
	}
	tenant.IsBYOD = tenant.EncryptedDatabaseURL != nil
	return tenant, nil
}

func (r SystemTenantRepository) DeleteTenant(ctx context.Context, id uuid.UUID) error {
	if r.DB == nil {
		return errors.New("system tenant repository: nil database")
	}
	_, err := r.DB.Exec(ctx, `DELETE FROM workspaces WHERE id = $1`, id)
	return err
}

func (r SystemTenantRepository) UpdateTenantDatabase(ctx context.Context, id uuid.UUID, encryptedURL *string) error {
	if r.DB == nil {
		return errors.New("system tenant repository: nil database")
	}
	tag, err := r.DB.Exec(ctx, `UPDATE workspaces SET custom_db_url_encrypted = $1 WHERE id = $2`, encryptedURL, id)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return ErrTenantNotFound
	}
	return nil
}
