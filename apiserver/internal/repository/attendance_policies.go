package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

type AttendancePolicyRepository struct {
	DB       DBTX
	SystemDB DBTX
	BeginTx  func(context.Context) (TxBeginner, error)
}

func (r AttendancePolicyRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("attendance policy repository: database is not configured")
	}
	return r.DB, nil
}

func (r AttendancePolicyRepository) validateBehalfAssignee(ctx context.Context, userID uuid.UUID) error {
	db := r.SystemDB
	if db == nil {
		return errors.New("attendance policy repository: system database is not configured")
	}
	var isMember bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM user_roles WHERE user_id = $1)`, userID).Scan(&isMember); err != nil {
		return err
	}
	if !isMember {
		return apperror.Forbidden(apperror.KeyNotOrganizationMember)
	}
	return nil
}

func scanBehalf(row interface{ Scan(...any) error }) (api.BehalfAssignment, error) {
	var item api.BehalfAssignment
	var createdAt, updatedAt time.Time
	if err := row.Scan(&item.ID, &item.AssigneeUserID, &item.TargetEmployeeID, &item.TargetDepartmentID, &item.AssignmentType, &createdAt, &updatedAt, &item.EmployerID, &item.TargetEmployeeName, &item.TargetEmployeeIDNumber, &item.TargetDepartmentName); err != nil {
		return item, err
	}
	item.CreatedAt, item.UpdatedAt = createdAt.UTC().Format(time.RFC3339), updatedAt.UTC().Format(time.RFC3339)
	return item, nil
}

const behalfColumns = `ba.id, ba.assignee_user_id, ba.target_employee_id, ba.target_department_id, ba.assignment_type, ba.created_at, ba.updated_at, ba.employer_id, te.full_name, tenure.employee_number, d.name`

func (r AttendancePolicyRepository) CreateBehalfAssignment(ctx context.Context, req api.CreateBehalfAssignmentRequest, assignmentType string) (api.BehalfAssignment, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.BehalfAssignment{}, err
	}
	if (req.TargetEmployeeID == nil) == (req.TargetDepartmentID == nil) {
		return api.BehalfAssignment{}, apperror.Validation(apperror.KeyValidation)
	}
	if err := r.validateBehalfAssignee(ctx, req.AssigneeUserID); err != nil {
		return api.BehalfAssignment{}, err
	}
	return scanBehalf(db.QueryRow(ctx, `WITH inserted AS (INSERT INTO behalf_assignments (id, assignee_user_id, target_employee_id, target_department_id, assignment_type, employer_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) RETURNING *) SELECT `+behalfColumns+` FROM inserted ba LEFT JOIN employees te ON te.id = ba.target_employee_id AND te.deleted_at IS NULL LEFT JOIN departments d ON d.id = ba.target_department_id LEFT JOIN LATERAL (SELECT t.employee_number FROM employee_tenures t WHERE t.employee_id = te.id AND t.deleted_at IS NULL AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE) ORDER BY t.start_date DESC, t.id DESC LIMIT 1) tenure ON TRUE`, uuid.New(), req.AssigneeUserID, req.TargetEmployeeID, req.TargetDepartmentID, assignmentType, req.EmployerID))
}

func (r AttendancePolicyRepository) ListBehalfAssignments(ctx context.Context, assignmentType string) ([]api.BehalfAssignment, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+behalfColumns+` FROM behalf_assignments ba LEFT JOIN employees te ON te.id = ba.target_employee_id AND te.deleted_at IS NULL LEFT JOIN departments d ON d.id = ba.target_department_id LEFT JOIN LATERAL (SELECT t.employee_number FROM employee_tenures t WHERE t.employee_id = te.id AND t.deleted_at IS NULL AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE) ORDER BY t.start_date DESC, t.id DESC LIMIT 1) tenure ON TRUE WHERE ba.assignment_type = $1 ORDER BY ba.created_at DESC`, assignmentType)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.BehalfAssignment, 0)
	for rows.Next() {
		item, err := scanBehalf(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (r AttendancePolicyRepository) DeleteBehalfAssignment(ctx context.Context, assignmentID uuid.UUID, assignmentType string) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	result, err := db.Exec(ctx, `DELETE FROM behalf_assignments WHERE id = $1 AND assignment_type = $2`, assignmentID, assignmentType)
	if err != nil {
		return err
	}
	if result.RowsAffected() == 0 {
		return apperror.NotFound(apperror.KeyNotFound)
	}
	return nil
}

func (r AttendancePolicyRepository) ListMyBehalfTargets(ctx context.Context, userID uuid.UUID, assignmentType string) ([]api.EmployeeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT DISTINCT e.id, tenure.user_id, e.full_name, COALESCE(tenure.employee_number, ''), e.tags, e.created_at, e.updated_at, d.name, em.id, em.employer_id, em.primary_location_id FROM employees e JOIN employments em ON em.employee_id = e.id AND em.status = 'active' AND em.deleted_at IS NULL `+sqlEffectiveTenureLateral+` LEFT JOIN departments d ON d.id = em.department_id LEFT JOIN behalf_assignments ba ON ba.assignee_user_id = $1 AND ba.assignment_type = $2 AND (ba.target_employee_id = e.id OR ba.target_department_id = em.department_id) WHERE e.deleted_at IS NULL AND ba.id IS NOT NULL ORDER BY e.full_name, e.id`, userID, assignmentType)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.EmployeeRecord, 0)
	for rows.Next() {
		var item api.EmployeeRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&item.ID, &item.UserID, &item.FullName, &item.EmployeeIDNumber, &item.Tags, &createdAt, &updatedAt, &item.DepartmentName, &item.ActiveEmploymentID, &item.EmployerID, &item.PrimaryLocationID); err != nil {
			return nil, err
		}
		item.CreatedAt, item.UpdatedAt = createdAt.UTC().Format(time.RFC3339), updatedAt.UTC().Format(time.RFC3339)
		result = append(result, item)
	}
	return result, rows.Err()
}
