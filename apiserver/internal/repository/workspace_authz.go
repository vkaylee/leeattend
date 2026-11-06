package repository

import (
	"context"

	"github.com/google/uuid"
)

// WorkspaceAuthorizer evaluates system roles and workspace permissions from current DB state.
type WorkspaceAuthorizer struct {
	DB DBTX
}

func (a WorkspaceAuthorizer) IsSystemAdmin(ctx context.Context, userID uuid.UUID) bool {
	if a.DB == nil {
		return false
	}
	var allowed bool
	err := a.DB.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM user_system_roles usr JOIN system_roles sr ON sr.id = usr.role_id WHERE usr.user_id = $1 AND sr.name IN ('admin', 'platform_admin', 'system_admin'))`, userID).Scan(&allowed)
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

// HasScopedPermission enforces the Rust require_scoped_permission rule: the
// permission must be held, global-scope roles bypass department scoping, and
// department-scoped callers must match the employee's current department.
func (a WorkspaceAuthorizer) HasScopedPermission(ctx context.Context, userID, workspaceID uuid.UUID, permission string, targetDepartmentID *uuid.UUID) bool {
	if !a.HasPermission(ctx, userID, workspaceID, permission) {
		return false
	}
	if a.isGlobalManager(ctx, userID, workspaceID) {
		return true
	}
	if targetDepartmentID == nil || a.DB == nil {
		return false
	}
	db := a.DB
	var managed bool
	err := db.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1
		FROM employees e
		JOIN employments em ON em.employee_id = e.id
		WHERE e.workspace_id = $1 AND e.user_id = $2
		  AND e.deleted_at IS NULL AND em.deleted_at IS NULL
		  AND em.department_id = $3
	)`, workspaceID, userID, *targetDepartmentID).Scan(&managed)
	return err == nil && managed
}

func (a WorkspaceAuthorizer) isGlobalManager(ctx context.Context, userID, workspaceID uuid.UUID) bool {
	if a.DB == nil {
		return false
	}
	var global bool
	err := a.DB.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1 FROM workspace_members wm
		JOIN workspace_roles wr ON wr.id = wm.role_id
		WHERE wm.user_id = $1 AND wm.workspace_id = $2 AND wr.scope_type = 'global'
		UNION ALL
		SELECT 1 FROM member_extra_roles mer
		JOIN workspace_roles wr ON wr.id = mer.role_id
		WHERE mer.user_id = $1 AND mer.workspace_id = $2 AND mer.expires_at > now()
		  AND wr.scope_type = 'global'
	)`, userID, workspaceID).Scan(&global)
	return err == nil && global
}

// GetManagedDepartmentIDs returns managed department IDs for a user.
// Returns (nil, true, nil) if the user is a global manager (unconstrained).
// Returns (deptIDs, false, nil) if the user is scoped to specific departments.
func (a WorkspaceAuthorizer) GetManagedDepartmentIDs(ctx context.Context, userID, workspaceID uuid.UUID) ([]uuid.UUID, bool, error) {
	if a.isGlobalManager(ctx, userID, workspaceID) {
		return nil, true, nil
	}
	if a.DB == nil {
		return []uuid.UUID{}, false, nil
	}
	db := a.DB
	rows, err := db.Query(ctx, `SELECT DISTINCT em.department_id
		FROM employees e
		JOIN employments em ON em.employee_id = e.id
		WHERE e.workspace_id = $1 AND e.user_id = $2
		  AND e.deleted_at IS NULL AND em.deleted_at IS NULL
		  AND em.department_id IS NOT NULL`, workspaceID, userID)
	if err != nil {
		return nil, false, err
	}
	defer rows.Close()
	var depts []uuid.UUID
	for rows.Next() {
		var id uuid.UUID
		if err := rows.Scan(&id); err != nil {
			return nil, false, err
		}
		depts = append(depts, id)
	}
	return depts, false, rows.Err()
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

// GetUsersWithPermission returns all users in a workspace who hold the specified permission
// either through their primary role, an active extra role, or an active extra permission.
func (a WorkspaceAuthorizer) GetUsersWithPermission(ctx context.Context, workspaceID uuid.UUID, permission string) ([]uuid.UUID, error) {
	if a.DB == nil {
		return nil, nil
	}
	rows, err := a.DB.Query(ctx, `SELECT DISTINCT u.user_id FROM (
		SELECT wm.user_id
		FROM workspace_members wm
		JOIN workspace_roles wr ON wr.id = wm.role_id
		LEFT JOIN workspace_role_permissions wrp ON wrp.role_id = wr.id
		WHERE wm.workspace_id = $1 AND (wr.scope_type = 'global' OR wrp.permission_name = $2)
		UNION
		SELECT mer.user_id
		FROM member_extra_roles mer
		JOIN workspace_roles wr ON wr.id = mer.role_id
		LEFT JOIN workspace_role_permissions wrp ON wrp.role_id = wr.id
		WHERE mer.workspace_id = $1 AND mer.expires_at > NOW() AND (wr.scope_type = 'global' OR wrp.permission_name = $2)
		UNION
		SELECT wmep.user_id
		FROM workspace_member_extra_permissions wmep
		WHERE wmep.workspace_id = $1 AND wmep.permission = $2 AND wmep.expires_at > NOW()
	) u`, workspaceID, permission)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var users []uuid.UUID
	for rows.Next() {
		var uid uuid.UUID
		if err := rows.Scan(&uid); err != nil {
			return nil, err
		}
		users = append(users, uid)
	}
	return users, rows.Err()
}
