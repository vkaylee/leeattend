package repository

import (
	"context"

	"github.com/google/uuid"
)

// WorkspaceAuthorizer evaluates system roles and workspace permissions from current DB state.
type WorkspaceAuthorizer struct{ DB DBTX }

func (a WorkspaceAuthorizer) IsSystemAdmin(ctx context.Context, userID uuid.UUID) bool {
	if a.DB == nil {
		return false
	}
	var allowed bool
	err := a.DB.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM user_system_roles usr JOIN system_roles sr ON sr.id = usr.role_id WHERE usr.user_id = $1 AND sr.name IN ('platform_admin', 'system_admin'))`, userID).Scan(&allowed)
	return err == nil && allowed
}

func (a WorkspaceAuthorizer) HasPermission(ctx context.Context, userID, workspaceID uuid.UUID, permission string) bool {
	if a.DB == nil {
		return false
	}
	var allowed bool
	err := a.DB.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1
		FROM workspace_members wm
		JOIN workspace_role_permissions wrp ON wrp.role_id = wm.role_id
		WHERE wm.user_id = $1 AND wm.workspace_id = $2 AND wrp.permission_name = $3
	)`, userID, workspaceID, permission).Scan(&allowed)
	return err == nil && allowed
}

// IsMember reports whether the user has any workspace membership row.
func (a WorkspaceAuthorizer) IsMember(ctx context.Context, userID, workspaceID uuid.UUID) (bool, error) {
	if a.DB == nil {
		return false, nil
	}
	var exists bool
	err := a.DB.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM workspace_members WHERE user_id = $1 AND workspace_id = $2)`, userID, workspaceID).Scan(&exists)
	return exists, err
}

// EffectivePermissions returns role permissions plus active extra permissions for a member.
func (a WorkspaceAuthorizer) EffectivePermissions(ctx context.Context, userID, workspaceID uuid.UUID) ([]string, []string, error) {
	if a.DB == nil {
		return nil, nil, nil
	}
	roleRows, err := a.DB.Query(ctx, `SELECT wm.role_id FROM workspace_members wm WHERE wm.user_id = $1 AND wm.workspace_id = $2
UNION
SELECT mer.role_id FROM member_extra_roles mer WHERE mer.user_id = $1 AND mer.workspace_id = $2 AND mer.expires_at > now()`, userID, workspaceID)
	if err != nil {
		return nil, nil, err
	}
	defer roleRows.Close()
	roles := []string{}
	roleIDs := make([]uuid.UUID, 0)
	for roleRows.Next() {
		var roleID uuid.UUID
		if err := roleRows.Scan(&roleID); err != nil {
			return nil, nil, err
		}
		roleIDs = append(roleIDs, roleID)
		roles = append(roles, roleID.String())
	}
	if err := roleRows.Err(); err != nil {
		return nil, nil, err
	}
	permissions := []string{}
	if len(roleIDs) > 0 {
		permRows, err := a.DB.Query(ctx, `SELECT DISTINCT permission_name FROM workspace_role_permissions WHERE role_id = ANY($1)`, roleIDs)
		if err != nil {
			return nil, nil, err
		}
		defer permRows.Close()
		for permRows.Next() {
			var permission string
			if err := permRows.Scan(&permission); err != nil {
				return nil, nil, err
			}
			permissions = append(permissions, permission)
		}
		if err := permRows.Err(); err != nil {
			return nil, nil, err
		}
	}
	extraRows, err := a.DB.Query(ctx, `SELECT DISTINCT permission FROM workspace_member_extra_permissions WHERE user_id = $1 AND workspace_id = $2 AND expires_at > now()`, userID, workspaceID)
	if err != nil {
		return nil, nil, err
	}
	defer extraRows.Close()
	for extraRows.Next() {
		var permission string
		if err := extraRows.Scan(&permission); err != nil {
			return nil, nil, err
		}
		permissions = append(permissions, permission)
	}
	if err := extraRows.Err(); err != nil {
		return nil, nil, err
	}
	return permissions, roles, nil
}
