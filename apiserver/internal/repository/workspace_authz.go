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
