package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type AssignmentHistoryRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r AssignmentHistoryRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("assignment history repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

func scanAssignmentHistory(row interface{ Scan(...any) error }) (api.AssignmentHistory, error) {
	var item api.AssignmentHistory
	var created time.Time
	if err := row.Scan(&item.ID, &item.EmployeeID, &item.EmployeeName, &item.AssignmentType, &item.ShiftName, &item.DateRange, &created); err != nil {
		return api.AssignmentHistory{}, err
	}
	item.CreatedAt = created.UTC().Format(time.RFC3339)
	return item, nil
}

const assignmentHistoryQuery = `
	SELECT id, employee_id, employee_name, assignment_type, shift_name, date_range, created_at
	FROM (
		SELECT es.id, emp.employee_id,
			COALESCE((SELECT full_name FROM employees WHERE id = emp.employee_id), '') AS employee_name,
			'fixed' AS assignment_type, st.name AS shift_name,
			TO_CHAR(es.start_date, 'YYYY-MM-DD') || CASE WHEN es.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(es.end_date, 'YYYY-MM-DD') END AS date_range,
			es.created_at
		FROM employee_shifts es
		JOIN shift_templates st ON st.id = es.template_id
		JOIN employments emp ON emp.id = es.employment_id
		WHERE emp.workspace_id = $1
		UNION ALL
		SELECT ewp.id, emp.employee_id,
			COALESCE((SELECT full_name FROM employees WHERE id = emp.employee_id), '') AS employee_name,
			'pattern' AS assignment_type, wp.name AS shift_name,
			TO_CHAR(ewp.start_date, 'YYYY-MM-DD') || CASE WHEN ewp.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(ewp.end_date, 'YYYY-MM-DD') END AS date_range,
			ewp.created_at
		FROM employee_working_patterns ewp
		JOIN working_patterns wp ON wp.id = ewp.pattern_id
		JOIN employments emp ON emp.id = ewp.employment_id
		WHERE emp.workspace_id = $1
		UNION ALL
		SELECT sr.id, emp.employee_id,
			COALESCE((SELECT full_name FROM employees WHERE id = emp.employee_id), '') AS employee_name,
			'roster' AS assignment_type,
			CASE WHEN sr.template_id IS NULL THEN 'OFF' ELSE st.name END AS shift_name,
			TO_CHAR(sr.date, 'YYYY-MM-DD') AS date_range,
			sr.created_at
		FROM shift_rosters sr
		LEFT JOIN shift_templates st ON st.id = sr.template_id
		JOIN employments emp ON emp.id = sr.employment_id
		WHERE emp.workspace_id = $1
	) history
`

func (r AssignmentHistoryRepository) ListAssignmentHistory(ctx context.Context, workspaceID uuid.UUID, page, limit int) ([]api.AssignmentHistory, int64, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, 0, err
	}
	var total int64
	if err := db.QueryRow(ctx, `SELECT COUNT(*) FROM (`+assignmentHistoryQuery+`) history`, workspaceID).Scan(&total); err != nil {
		return nil, 0, err
	}
	rows, err := db.Query(ctx, assignmentHistoryQuery+` ORDER BY created_at DESC LIMIT $2 OFFSET $3`, workspaceID, limit, (page-1)*limit)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	items := make([]api.AssignmentHistory, 0)
	for rows.Next() {
		item, err := scanAssignmentHistory(rows)
		if err != nil {
			return nil, 0, err
		}
		items = append(items, item)
	}
	return items, total, rows.Err()
}
