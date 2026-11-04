package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"

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

func (r EmployeeRepository) CreateEmployee(ctx context.Context, workspaceID uuid.UUID, req api.CreateEmployeeRequest) (api.EmployeeRecord, error) {
	db, err := r.db(ctx)
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
	db, err := r.db(ctx)
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

func (r EmployeeRepository) GetEmployee(ctx context.Context, workspaceID, employeeID uuid.UUID) (api.EmployeeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	err = db.QueryRow(ctx, `SELECT id, workspace_id, user_id, full_name, employee_id_number, tags, created_at, updated_at, NULL::text, NULL::uuid, NULL::uuid, NULL::uuid FROM employees WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL`, employeeID, workspaceID).
		Scan(&record.ID, &record.WorkspaceID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &record.CreatedAt, &record.UpdatedAt, &record.DepartmentName, &record.ActiveEmploymentID, &record.EmployerID, &record.PrimaryLocationID)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	return record, nil
}

func (r EmployeeRepository) UpdateEmployee(ctx context.Context, workspaceID, employeeID uuid.UUID, req api.UpdateEmployeeRequest) (api.EmployeeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	if req.UserID != nil && *req.UserID != nil && req.ForceTransfer != nil && *req.ForceTransfer {
		_, _ = db.Exec(ctx, `UPDATE employees SET user_id = NULL WHERE user_id = $1 AND workspace_id = $2 AND id != $3`, **req.UserID, workspaceID, employeeID)
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
	args = append(args, employeeID, workspaceID)
	idIdx := len(args) - 1
	wsIdx := len(args)
	query := fmt.Sprintf(`UPDATE employees SET %s WHERE id = $%d AND workspace_id = $%d AND deleted_at IS NULL RETURNING id, workspace_id, user_id, full_name, employee_id_number, tags, created_at, updated_at`, strings.Join(sets, ", "), idIdx, wsIdx)
	var record api.EmployeeRecord
	err = db.QueryRow(ctx, query, args...).Scan(&record.ID, &record.WorkspaceID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &record.CreatedAt, &record.UpdatedAt)
	if err != nil {
		return api.EmployeeRecord{}, mapConstraintError(err)
	}
	return record, nil
}

func (r EmployeeRepository) SoftDeleteEmployee(ctx context.Context, workspaceID, employeeID uuid.UUID) error {
	db, err := r.db(ctx)
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
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	err = db.QueryRow(ctx, `UPDATE employees SET deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND workspace_id = $2 RETURNING id, workspace_id, user_id, full_name, employee_id_number, tags, created_at, updated_at`, employeeID, workspaceID).Scan(&record.ID, &record.WorkspaceID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &record.CreatedAt, &record.UpdatedAt)
	return record, err
}
