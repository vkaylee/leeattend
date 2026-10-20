package repository

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type EmployeeConditionRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r EmployeeConditionRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("employee condition repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

func employeeExists(ctx context.Context, db DBTX, workspaceID, employeeID uuid.UUID) error {
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, employeeID, workspaceID).Scan(&exists); err != nil {
		return err
	}
	if !exists {
		return api.ErrEmployeeNotFound
	}
	return nil
}

func activeEmployment(ctx context.Context, db DBTX, workspaceID, employeeID uuid.UUID) (uuid.UUID, error) {
	if err := employeeExists(ctx, db, workspaceID, employeeID); err != nil {
		return uuid.Nil, err
	}
	var employmentID uuid.UUID
	if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID, workspaceID).Scan(&employmentID); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return uuid.Nil, api.ErrNoActiveEmployment
		}
		return uuid.Nil, err
	}
	return employmentID, nil
}

func scanEmployeeCondition(row interface{ Scan(...any) error }) (api.EmployeeCondition, error) {
	var c api.EmployeeCondition
	var startDate string
	var endDate *string
	if err := row.Scan(&c.ID, &c.EmploymentID, &c.Type, &c.ConditionTypeID, &c.MinuteAllowance, &c.IsPaid, &startDate, &endDate); err != nil {
		return c, err
	}
	c.StartDate, c.EndDate = startDate, endDate
	return c, nil
}

func (r EmployeeConditionRepository) ListEmployeeConditions(ctx context.Context, workspaceID, employeeID uuid.UUID, page, limit int) (api.EmployeeConditionListResponse, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.EmployeeConditionListResponse{}, err
	}
	employmentID, err := activeEmployment(ctx, db, workspaceID, employeeID)
	if err != nil {
		return api.EmployeeConditionListResponse{}, err
	}
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM working_conditions wc JOIN employments e ON e.id = wc.employment_id WHERE wc.employment_id = $1 AND e.workspace_id = $2 AND e.deleted_at IS NULL`, employmentID, workspaceID).Scan(&total); err != nil {
		return api.EmployeeConditionListResponse{}, err
	}
	offset := (page - 1) * limit
	rows, err := db.Query(ctx, `SELECT wc.id, wc.employment_id, wc.type, wc.condition_type_id, wc.minute_allowance, wc.is_paid, wc.start_date::text, wc.end_date::text FROM working_conditions wc JOIN employments e ON e.id = wc.employment_id WHERE wc.employment_id = $1 AND e.workspace_id = $2 AND e.deleted_at IS NULL ORDER BY wc.start_date DESC LIMIT $3 OFFSET $4`, employmentID, workspaceID, limit, offset)
	if err != nil {
		return api.EmployeeConditionListResponse{}, err
	}
	defer rows.Close()
	data := make([]api.EmployeeCondition, 0)
	for rows.Next() {
		condition, err := scanEmployeeCondition(rows)
		if err != nil {
			return api.EmployeeConditionListResponse{}, err
		}
		data = append(data, condition)
	}
	if err := rows.Err(); err != nil {
		return api.EmployeeConditionListResponse{}, err
	}
	return api.EmployeeConditionListResponse{Data: data, Total: total, Page: page, Limit: limit, Truncated: nil}, nil
}

func (r EmployeeConditionRepository) CreateEmployeeCondition(ctx context.Context, workspaceID, employeeID uuid.UUID, req api.CreateEmployeeConditionRequest) (api.EmployeeCondition, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.EmployeeCondition{}, err
	}
	employmentID, err := activeEmployment(ctx, db, workspaceID, employeeID)
	if err != nil {
		return api.EmployeeCondition{}, err
	}
	conditionID := uuid.New()
	var startDate, endDate string
	if req.EndDate == nil {
		endDate = ""
	}
	err = db.QueryRow(ctx, `INSERT INTO working_conditions (id, employment_id, type, condition_type_id, minute_allowance, is_paid, start_date, end_date) SELECT $1, e.id, $3, $4, $5, $6, $7::date, NULLIF($8, '')::date FROM employments e WHERE e.id = $2 AND e.workspace_id = $9 AND e.deleted_at IS NULL RETURNING start_date::text, end_date::text`, conditionID, employmentID, req.Type, req.ConditionTypeID, req.MinuteAllowance, req.IsPaid, req.StartDate, func() string {
		if req.EndDate != nil {
			return *req.EndDate
		}
		return ""
	}(), workspaceID).Scan(&startDate, &endDate)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.EmployeeCondition{}, api.ErrEmploymentNotFound
		}
		return api.EmployeeCondition{}, err
	}
	var end *string
	if endDate != "" {
		end = &endDate
	}
	return api.EmployeeCondition{ID: conditionID, EmploymentID: employmentID, Type: req.Type, ConditionTypeID: req.ConditionTypeID, MinuteAllowance: req.MinuteAllowance, IsPaid: req.IsPaid, StartDate: startDate, EndDate: end}, nil
}

func (r EmployeeConditionRepository) DeleteEmployeeCondition(ctx context.Context, workspaceID, employeeID, conditionID uuid.UUID) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	employmentID, err := activeEmployment(ctx, db, workspaceID, employeeID)
	if err != nil {
		return err
	}
	command, err := db.Exec(ctx, `DELETE FROM working_conditions wc USING employments e WHERE wc.id = $1 AND wc.employment_id = $2 AND wc.employment_id = e.id AND e.workspace_id = $3 AND e.deleted_at IS NULL`, conditionID, employmentID, workspaceID)
	if err != nil {
		return err
	}
	if command.RowsAffected() == 0 {
		return api.ErrEmployeeConditionNotFound
	}
	return nil
}

func (r EmployeeConditionRepository) CurrentEmployeeDepartment(ctx context.Context, workspaceID, employeeID uuid.UUID) (*uuid.UUID, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	if err := employeeExists(ctx, db, workspaceID, employeeID); err != nil {
		return nil, err
	}
	var departmentID *uuid.UUID
	err = db.QueryRow(ctx, `SELECT department_id FROM employments WHERE employee_id = $1 AND workspace_id = $2 AND deleted_at IS NULL AND start_date <= CURRENT_DATE AND (end_date IS NULL OR end_date >= CURRENT_DATE) ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID, workspaceID).Scan(&departmentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	return departmentID, err
}
