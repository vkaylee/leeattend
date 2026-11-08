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

type EmployeeRepository struct {
	DB DBTX
}

func (r EmployeeRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("employee repository: database is not configured")
	}
	return r.DB, nil
}

func (r EmployeeRepository) CreateEmployee(ctx context.Context, req api.CreateEmployeeRequest) (api.EmployeeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `INSERT INTO employees (employee_id_number, full_name, tags, hire_date, original_hire_date) VALUES ($1, $2, $3, $4::date, $4::date) RETURNING id, user_id, full_name, employee_id_number, tags, created_at, updated_at`, req.EmployeeIDNumber, strings.TrimSpace(req.FullName), req.Tags, req.HireDate).Scan(&record.ID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &createdAt, &updatedAt)
	if err != nil {
		return api.EmployeeRecord{}, mapConstraintError(err)
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

func (r EmployeeRepository) ListEmployees(ctx context.Context, page, limit int, search string) (api.EmployeeListResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeListResponse{}, err
	}
	offset := (page - 1) * limit
	pattern := "%" + search + "%"
	rows, err := db.Query(ctx, `SELECT e.id, e.user_id, e.full_name, e.employee_id_number, e.tags, e.created_at, e.updated_at, d.name AS department_name, emp.id AS active_employment_id, emp.employer_id, NULL::uuid AS primary_location_id FROM employees e LEFT JOIN employments emp ON emp.employee_id = e.id AND emp.status = 'active' AND emp.deleted_at IS NULL LEFT JOIN departments d ON d.id = emp.department_id WHERE e.deleted_at IS NULL AND ($1 = '%%' OR e.full_name ILIKE $1 OR e.employee_id_number ILIKE $1) ORDER BY e.employee_id_number ASC, e.id ASC LIMIT $2 OFFSET $3`, pattern, limit, offset)
	if err != nil {
		return api.EmployeeListResponse{}, err
	}
	defer rows.Close()
	result := api.EmployeeListResponse{Data: make([]api.EmployeeRecord, 0), Page: page, Limit: limit}
	for rows.Next() {
		var record api.EmployeeRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&record.ID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &createdAt, &updatedAt, &record.DepartmentName, &record.ActiveEmploymentID, &record.EmployerID, &record.PrimaryLocationID); err != nil {
			return api.EmployeeListResponse{}, err
		}
		record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		result.Data = append(result.Data, record)
	}
	if err := rows.Err(); err != nil {
		return api.EmployeeListResponse{}, err
	}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM employees WHERE deleted_at IS NULL AND ($1 = '%%' OR full_name ILIKE $1 OR employee_id_number ILIKE $1)`, pattern).Scan(&result.Total); err != nil {
		return api.EmployeeListResponse{}, err
	}
	return result, nil
}

func (r EmployeeRepository) GetEmployee(ctx context.Context, employeeID uuid.UUID) (api.EmployeeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `SELECT e.id, e.user_id, e.full_name, e.employee_id_number, e.tags, e.created_at, e.updated_at, d.name AS department_name, emp.id AS active_employment_id, emp.employer_id, NULL::uuid AS primary_location_id FROM employees e LEFT JOIN employments emp ON emp.employee_id = e.id AND emp.status = 'active' AND emp.deleted_at IS NULL LEFT JOIN departments d ON d.id = emp.department_id WHERE e.id = $1 AND e.deleted_at IS NULL`, employeeID).
		Scan(&record.ID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &createdAt, &updatedAt, &record.DepartmentName, &record.ActiveEmploymentID, &record.EmployerID, &record.PrimaryLocationID)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

func (r EmployeeRepository) UpdateEmployee(ctx context.Context, employeeID uuid.UUID, req api.UpdateEmployeeRequest) (api.EmployeeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	if req.UserID != nil && *req.UserID != nil && req.ForceTransfer != nil && *req.ForceTransfer {
		_, _ = db.Exec(ctx, `UPDATE employees SET user_id = NULL WHERE user_id = $1 AND id != $2`, **req.UserID, employeeID)
	}
	sets := []string{"updated_at = CURRENT_TIMESTAMP"}
	args := []any{}
	add := func(col string, val any) {
		args = append(args, val)
		sets = append(sets, fmt.Sprintf("%s = $%d", col, len(args)))
	}
	if req.UserID != nil {
		if *req.UserID == nil {
			sets = append(sets, "user_id = NULL")
		} else {
			add("user_id", **req.UserID)
		}
	}
	if req.FullName != nil {
		add("full_name", strings.TrimSpace(*req.FullName))
	}
	if req.EmployeeIDNumber != nil {
		add("employee_id_number", strings.TrimSpace(*req.EmployeeIDNumber))
	}
	if req.Tags != nil {
		add("tags", *req.Tags)
	}
	if req.RecruitmentSourceID != nil {
		add("recruitment_source_id", *req.RecruitmentSourceID)
	}
	args = append(args, employeeID)
	idIdx := len(args)
	query := fmt.Sprintf(`UPDATE employees SET %s WHERE id = $%d AND deleted_at IS NULL RETURNING id, user_id, full_name, employee_id_number, tags, created_at, updated_at`, strings.Join(sets, ", "), idIdx)
	var record api.EmployeeRecord
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, query, args...).Scan(&record.ID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &createdAt, &updatedAt)
	if err != nil {
		return api.EmployeeRecord{}, mapConstraintError(err)
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

func (r EmployeeRepository) SoftDeleteEmployee(ctx context.Context, employeeID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	command, err := db.Exec(ctx, `UPDATE employees SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NULL`, employeeID)
	if err != nil {
		return err
	}
	if command.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}

func (r EmployeeRepository) RehireEmployee(ctx context.Context, employeeID uuid.UUID, _ api.RehireEmployeeRequest) (api.EmployeeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `UPDATE employees SET deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = $1 RETURNING id, user_id, full_name, employee_id_number, tags, created_at, updated_at`, employeeID).Scan(&record.ID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &createdAt, &updatedAt)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}
