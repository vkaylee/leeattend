package repository

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type EmployeeConditionRepository struct {
	DB DBTX
}

func employeeExists(ctx context.Context, db DBTX, employeeID uuid.UUID) error {
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&exists); err != nil {
		return err
	}
	if !exists {
		return api.ErrEmployeeNotFound
	}
	return nil
}

func activeEmployment(ctx context.Context, db DBTX, employeeID uuid.UUID) (uuid.UUID, error) {
	if err := employeeExists(ctx, db, employeeID); err != nil {
		return uuid.Nil, err
	}
	var employmentID uuid.UUID
	if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID).Scan(&employmentID); err != nil {
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

func (r EmployeeConditionRepository) ListOrganizationConditions(ctx context.Context) ([]api.EmployeeConditionWithEmployee, error) {
	db, err := requireDB(r.DB, "employee condition repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		wc.id, wc.employment_id, wc.type, wc.condition_type_id, wc.minute_allowance, wc.is_paid,
		wc.start_date::text, wc.end_date::text,
		e.id, e.full_name, COALESCE(t.employee_number, '')
		FROM working_conditions wc
		JOIN employments em ON em.id = wc.employment_id
		JOIN employees e ON e.id = em.employee_id
		LEFT JOIN employment_relationships er ON er.id = em.relationship_id AND er.employee_id = em.employee_id
		LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= wc.start_date AND (t.end_date IS NULL OR t.end_date >= wc.start_date)
		WHERE em.deleted_at IS NULL AND e.deleted_at IS NULL
		ORDER BY wc.start_date DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	items := make([]api.EmployeeConditionWithEmployee, 0)
	for rows.Next() {
		var item api.EmployeeConditionWithEmployee
		var startDate string
		var endDate *string
		if err := rows.Scan(
			&item.ID, &item.EmploymentID, &item.Type, &item.ConditionTypeID,
			&item.MinuteAllowance, &item.IsPaid, &startDate, &endDate,
			&item.EmployeeID, &item.EmployeeName, &item.EmployeeCode,
		); err != nil {
			return nil, err
		}
		item.StartDate = startDate
		item.EndDate = endDate
		items = append(items, item)
	}
	return items, rows.Err()
}

func (r EmployeeConditionRepository) ListEmployeeConditions(ctx context.Context, employeeID uuid.UUID, page, limit int) (api.EmployeeConditionListResponse, error) {
	db, err := requireDB(r.DB, "employee condition repository")
	if err != nil {
		return api.EmployeeConditionListResponse{}, err
	}
	employmentID, err := activeEmployment(ctx, db, employeeID)
	if err != nil {
		return api.EmployeeConditionListResponse{}, err
	}
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM working_conditions WHERE employment_id = $1`, employmentID).Scan(&total); err != nil {
		return api.EmployeeConditionListResponse{}, err
	}
	offset := (page - 1) * limit
	rows, err := db.Query(ctx, `SELECT id, employment_id, type, condition_type_id, minute_allowance, is_paid, start_date::text, end_date::text FROM working_conditions WHERE employment_id = $1 ORDER BY start_date DESC LIMIT $2 OFFSET $3`, employmentID, limit, offset)
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

func (r EmployeeConditionRepository) CreateEmployeeCondition(ctx context.Context, employeeID uuid.UUID, req api.CreateEmployeeConditionRequest) (api.EmployeeCondition, error) {
	db, err := requireDB(r.DB, "employee condition repository")
	if err != nil {
		return api.EmployeeCondition{}, err
	}
	employmentID, err := activeEmployment(ctx, db, employeeID)
	if err != nil {
		return api.EmployeeCondition{}, err
	}
	conditionID := uuid.New()
	var startDate, endDate string
	endDateInput := ""
	if req.EndDate != nil {
		endDateInput = *req.EndDate
	}
	err = db.QueryRow(ctx, `INSERT INTO working_conditions (id, employment_id, type, condition_type_id, minute_allowance, is_paid, start_date, end_date) SELECT $1, id, $2, $3, $4, $5, $6::date, NULLIF($7, '')::date FROM employments WHERE id = $8 AND deleted_at IS NULL RETURNING start_date::text, end_date::text`, conditionID, req.Type, req.ConditionTypeID, req.MinuteAllowance, req.IsPaid, req.StartDate, endDateInput, employmentID).Scan(&startDate, &endDate)
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

func (r EmployeeConditionRepository) DeleteEmployeeCondition(ctx context.Context, employeeID, conditionID uuid.UUID) error {
	db, err := requireDB(r.DB, "employee condition repository")
	if err != nil {
		return err
	}
	employmentID, err := activeEmployment(ctx, db, employeeID)
	if err != nil {
		return err
	}
	command, err := db.Exec(ctx, `DELETE FROM working_conditions WHERE id = $1 AND employment_id = $2`, conditionID, employmentID)
	if err != nil {
		return err
	}
	if command.RowsAffected() == 0 {
		return api.ErrEmployeeConditionNotFound
	}
	return nil
}

func (r EmployeeConditionRepository) CurrentEmployeeDepartment(ctx context.Context, employeeID uuid.UUID) (*uuid.UUID, error) {
	db, err := requireDB(r.DB, "employee condition repository")
	if err != nil {
		return nil, err
	}
	if err := employeeExists(ctx, db, employeeID); err != nil {
		return nil, err
	}
	var departmentID *uuid.UUID
	err = db.QueryRow(ctx, `SELECT department_id FROM employments WHERE employee_id = $1 AND deleted_at IS NULL AND start_date <= CURRENT_DATE AND (end_date IS NULL OR end_date >= CURRENT_DATE) ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID).Scan(&departmentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	return departmentID, err
}
