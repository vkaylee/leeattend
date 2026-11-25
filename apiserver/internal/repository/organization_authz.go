package repository

import (
	"context"

	"github.com/google/uuid"
)

// OrganizationAuthorizer evaluates single-organization permissions directly from database state.
type OrganizationAuthorizer struct {
	DB DBTX
}

func (a OrganizationAuthorizer) HasPermission(ctx context.Context, userID uuid.UUID, permission string) bool {
	if a.DB == nil {
		return false
	}
	var allowed bool
	err := a.DB.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1
		FROM user_roles ur
		JOIN role_permissions rp ON rp.role_id = ur.role_id
		WHERE ur.user_id = $1 AND rp.permission_name = $2
		UNION ALL
		SELECT 1
		FROM user_extra_roles uer
		JOIN role_permissions rp ON rp.role_id = uer.role_id
		WHERE uer.user_id = $1 AND uer.expires_at > now() AND rp.permission_name = $2
		UNION ALL
		SELECT 1
		FROM user_extra_permissions uep
		WHERE uep.user_id = $1 AND uep.expires_at > now() AND uep.permission = $2
	)`, userID, permission).Scan(&allowed)
	return err == nil && allowed
}

func (a OrganizationAuthorizer) HasScopedPermission(ctx context.Context, userID uuid.UUID, permission string, targetDepartmentID *uuid.UUID) bool {
	if !a.HasPermission(ctx, userID, permission) {
		return false
	}
	if a.IsGlobalManager(ctx, userID) {
		return true
	}
	if targetDepartmentID == nil || a.DB == nil {
		return false
	}
	var managed bool
	err := a.DB.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1
		FROM employee_tenures t
		JOIN employees e ON e.id = t.employee_id
		JOIN employment_relationships er ON er.tenure_id = t.id AND er.employee_id = t.employee_id AND er.deleted_at IS NULL
		JOIN employments em ON em.relationship_id = er.id AND em.deleted_at IS NULL
		WHERE t.user_id = $1
		  AND t.deleted_at IS NULL
		  AND e.deleted_at IS NULL
		  AND t.start_date <= `+orgTodaySQL+`
		  AND (t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+`)
		  AND em.department_id = $2
	)`, userID, *targetDepartmentID).Scan(&managed)
	return err == nil && managed
}

func (a OrganizationAuthorizer) IsGlobalManager(ctx context.Context, userID uuid.UUID) bool {
	if a.DB == nil {
		return false
	}
	var global bool
	err := a.DB.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1 FROM user_roles ur
		JOIN roles r ON r.id = ur.role_id
		WHERE ur.user_id = $1 AND r.scope_type = 'global'
		UNION ALL
		SELECT 1 FROM user_extra_roles uer
		JOIN roles r ON r.id = uer.role_id
		WHERE uer.user_id = $1 AND uer.expires_at > now()
		  AND r.scope_type = 'global'
	)`, userID).Scan(&global)
	return err == nil && global
}

func (a OrganizationAuthorizer) GetManagedDepartmentIDs(ctx context.Context, userID uuid.UUID) ([]uuid.UUID, error) {
	if a.DB == nil {
		return []uuid.UUID{}, nil
	}
	rows, err := a.DB.Query(ctx, `SELECT DISTINCT em.department_id
		FROM employee_tenures t
		JOIN employees e ON e.id = t.employee_id
		JOIN employment_relationships er ON er.tenure_id = t.id AND er.employee_id = t.employee_id AND er.deleted_at IS NULL
		JOIN employments em ON em.relationship_id = er.id AND em.deleted_at IS NULL
		WHERE t.user_id = $1
		  AND t.deleted_at IS NULL
		  AND e.deleted_at IS NULL
		  AND t.start_date <= `+orgTodaySQL+`
		  AND (t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+`)
		  AND em.department_id IS NOT NULL`, userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var depts []uuid.UUID
	for rows.Next() {
		var id uuid.UUID
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		depts = append(depts, id)
	}
	if depts == nil {
		depts = []uuid.UUID{}
	}
	return depts, rows.Err()
}

func (a OrganizationAuthorizer) IsMember(ctx context.Context, userID uuid.UUID) (bool, error) {
	if a.DB == nil {
		return false, nil
	}
	var exists bool
	err := a.DB.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM user_roles WHERE user_id = $1)`, userID).Scan(&exists)
	return exists, err
}

func (a OrganizationAuthorizer) EffectivePermissions(ctx context.Context, userID uuid.UUID) ([]string, []string, error) {
	if a.DB == nil {
		return nil, nil, nil
	}
	roleRows, err := a.DB.Query(ctx, `SELECT ur.role_id FROM user_roles ur WHERE ur.user_id = $1
UNION
SELECT uer.role_id FROM user_extra_roles uer WHERE uer.user_id = $1 AND uer.expires_at > now()`, userID)
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
		permRows, err := a.DB.Query(ctx, `SELECT DISTINCT permission_name FROM role_permissions WHERE role_id = ANY($1)`, roleIDs)
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
	extraRows, err := a.DB.Query(ctx, `SELECT DISTINCT permission FROM user_extra_permissions WHERE user_id = $1 AND expires_at > now()`, userID)
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

func (a OrganizationAuthorizer) GetUsersWithPermission(ctx context.Context, permission string) ([]uuid.UUID, error) {
	if a.DB == nil {
		return nil, nil
	}
	rows, err := a.DB.Query(ctx, `SELECT DISTINCT u.user_id FROM (
		SELECT ur.user_id
		FROM user_roles ur
		JOIN roles r ON r.id = ur.role_id
		LEFT JOIN role_permissions rp ON rp.role_id = r.id
		WHERE (r.scope_type = 'global' OR rp.permission_name = $1)
		UNION
		SELECT uer.user_id
		FROM user_extra_roles uer
		JOIN roles r ON r.id = uer.role_id
		LEFT JOIN role_permissions rp ON rp.role_id = r.id
		WHERE uer.expires_at > NOW() AND (r.scope_type = 'global' OR rp.permission_name = $1)
		UNION
		SELECT uep.user_id
		FROM user_extra_permissions uep
		WHERE uep.permission = $1 AND uep.expires_at > NOW()
	) u`, permission)
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
	if users == nil {
		users = []uuid.UUID{}
	}
	return users, rows.Err()
}
