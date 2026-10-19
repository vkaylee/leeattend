package repository

import (
	"context"
	"errors"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type EmployeeRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r EmployeeRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("employee repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

func (r EmployeeRepository) CreateEmployee(ctx context.Context, workspaceID uuid.UUID, req api.CreateEmployeeRequest) (api.EmployeeRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	err = db.QueryRow(ctx, `INSERT INTO employees (workspace_id, employee_id_number, full_name, tags) VALUES ($1, $2, $3, $4) RETURNING id, workspace_id, user_id, full_name, employee_id_number, tags, created_at, updated_at`, workspaceID, req.EmployeeIDNumber, strings.TrimSpace(req.FullName), req.Tags).Scan(&record.ID, &record.WorkspaceID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &record.CreatedAt, &record.UpdatedAt)
	if err != nil {
		return api.EmployeeRecord{}, mapConstraintError(err)
	}
	return record, nil
}

func (r EmployeeRepository) ListEmployees(ctx context.Context, workspaceID uuid.UUID, page, limit int, search string) (api.EmployeeListResponse, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.EmployeeListResponse{}, err
	}
	offset := (page - 1) * limit
	pattern := "%" + search + "%"
	rows, err := db.Query(ctx, `SELECT id, workspace_id, user_id, full_name, employee_id_number, tags, created_at, updated_at, NULL::text, NULL::uuid, NULL::uuid, NULL::uuid FROM employees WHERE workspace_id = $1 AND deleted_at IS NULL AND ($2 = '%%' OR full_name ILIKE $2 OR employee_id_number ILIKE $2) ORDER BY id LIMIT $3 OFFSET $4`, workspaceID, pattern, limit, offset)
	if err != nil {
		return api.EmployeeListResponse{}, err
	}
	defer rows.Close()
	result := api.EmployeeListResponse{Data: make([]api.EmployeeRecord, 0), Page: page, Limit: limit}
	for rows.Next() {
		var record api.EmployeeRecord
		if err := rows.Scan(&record.ID, &record.WorkspaceID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &record.CreatedAt, &record.UpdatedAt, &record.DepartmentName, &record.ActiveEmploymentID, &record.EmployerID, &record.PrimaryLocationID); err != nil {
			return api.EmployeeListResponse{}, err
		}
		result.Data = append(result.Data, record)
	}
	if err := rows.Err(); err != nil {
		return api.EmployeeListResponse{}, err
	}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM employees WHERE workspace_id = $1 AND deleted_at IS NULL AND ($2 = '%%' OR full_name ILIKE $2 OR employee_id_number ILIKE $2)`, workspaceID, pattern).Scan(&result.Total); err != nil {
		return api.EmployeeListResponse{}, err
	}
	return result, nil
}

func (r EmployeeRepository) SoftDeleteEmployee(ctx context.Context, workspaceID, employeeID uuid.UUID) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	command, err := db.Exec(ctx, `UPDATE employees SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL`, employeeID, workspaceID)
	if err != nil {
		return err
	}
	if command.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}

func (r EmployeeRepository) RehireEmployee(ctx context.Context, workspaceID, employeeID uuid.UUID, _ api.RehireEmployeeRequest) (api.EmployeeRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	err = db.QueryRow(ctx, `UPDATE employees SET deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND workspace_id = $2 RETURNING id, workspace_id, user_id, full_name, employee_id_number, tags, created_at, updated_at`, employeeID, workspaceID).Scan(&record.ID, &record.WorkspaceID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &record.CreatedAt, &record.UpdatedAt)
	return record, err
}
