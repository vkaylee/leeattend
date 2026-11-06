package repository

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

var (
	ErrSystemRoleNotFound = api.ErrSystemRoleNotFound
	ErrSystemRoleUnknown  = api.ErrSystemRoleUnknown
	ErrLastPlatformAdmin  = api.ErrLastPlatformAdmin
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
	if parsed == api.SystemRoleAdmin || parsed == api.SystemRolePlatformAdmin {
		var count int64
		if err := r.DB.QueryRow(ctx, `SELECT COUNT(*) FROM user_system_roles usr JOIN system_roles sr ON sr.id = usr.role_id WHERE sr.name IN ('admin', 'platform_admin')`).Scan(&count); err != nil {
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

func (r SystemAdminRepository) Get(ctx context.Context, key string) (json.RawMessage, bool, error) {
	return SystemSettingsRepository(r).Get(ctx, key)
}

func (r SystemAdminRepository) Set(ctx context.Context, key string, value json.RawMessage) error {
	return SystemSettingsRepository(r).Set(ctx, key, value)
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
