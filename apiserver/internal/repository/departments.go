package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type DepartmentRepository struct {
	DB DBTX
}

func (r DepartmentRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("department repository: database is not configured")
	}
	return r.DB, nil
}

const departmentColumns = `
	d.id, d.name, d.description, d.manager_id,
	d.created_at, d.updated_at,
	(
		SELECT count(DISTINCT em.employee_id)
		FROM employments em
		WHERE em.department_id = d.id
		  AND em.status = 'active'
		  AND em.deleted_at IS NULL
		  AND em.start_date <= CURRENT_DATE
		  AND (em.end_date IS NULL OR em.end_date >= CURRENT_DATE)
	) AS active_employee_count
`

func scanDepartment(row pgx.Row) (api.DepartmentRecord, error) {
	var record api.DepartmentRecord
	var createdAt, updatedAt time.Time
	err := row.Scan(
		&record.ID, &record.Name, &record.Description,
		&record.ManagerID, &createdAt, &updatedAt, &record.ActiveEmployeeCount,
	)
	if err != nil {
		return api.DepartmentRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

func (r DepartmentRepository) ListDepartments(ctx context.Context, page, limit int) (api.DepartmentListResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.DepartmentListResponse{}, err
	}
	offset := (page - 1) * limit
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM departments WHERE deleted_at IS NULL`).Scan(&total); err != nil {
		return api.DepartmentListResponse{}, err
	}
	query := fmt.Sprintf(`SELECT %s FROM departments d WHERE d.deleted_at IS NULL ORDER BY d.name ASC LIMIT $1 OFFSET $2`, departmentColumns)
	rows, err := db.Query(ctx, query, limit, offset)
	if err != nil {
		return api.DepartmentListResponse{}, err
	}
	defer rows.Close()
	data := make([]api.DepartmentRecord, 0)
	for rows.Next() {
		var record api.DepartmentRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&record.ID, &record.Name, &record.Description, &record.ManagerID, &createdAt, &updatedAt, &record.ActiveEmployeeCount); err != nil {
			return api.DepartmentListResponse{}, err
		}
		record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		data = append(data, record)
	}
	return api.DepartmentListResponse{
		Data:  data,
		Total: total,
		Page:  page,
		Limit: limit,
	}, rows.Err()
}

func (r DepartmentRepository) GetDepartmentByID(ctx context.Context, deptID uuid.UUID) (api.DepartmentRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.DepartmentRecord{}, err
	}
	query := fmt.Sprintf(`SELECT %s FROM departments d WHERE d.id = $1 AND d.deleted_at IS NULL`, departmentColumns)
	dept, err := scanDepartment(db.QueryRow(ctx, query, deptID))
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.DepartmentRecord{}, api.ErrDepartmentNotFound
		}
		return api.DepartmentRecord{}, err
	}
	return dept, nil
}

func (r DepartmentRepository) CreateDepartment(ctx context.Context, req api.CreateDepartmentRequest) (api.DepartmentRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.DepartmentRecord{}, err
	}
	deptID := uuid.New()
	_, err = db.Exec(ctx, `INSERT INTO departments (id, name, description, manager_id, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())`,
		deptID, req.Name, req.Description, req.ManagerID)
	if err != nil {
		var pgErr *pgconn.PgError
		if errors.As(err, &pgErr) && pgErr.Code == "23505" {
			return api.DepartmentRecord{}, api.ErrDepartmentConflict
		}
		return api.DepartmentRecord{}, err
	}
	return r.GetDepartmentByID(ctx, deptID)
}

func (r DepartmentRepository) UpdateDepartment(ctx context.Context, deptID uuid.UUID, req api.UpdateDepartmentRequest) (api.DepartmentRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.DepartmentRecord{}, err
	}
	sets := []string{"updated_at = NOW()"}
	args := make([]any, 0, 5)
	if req.Name != nil {
		args = append(args, *req.Name)
		sets = append(sets, fmt.Sprintf("name = $%d", len(args)))
	}
	if req.Description != nil {
		args = append(args, *req.Description)
		sets = append(sets, fmt.Sprintf("description = $%d", len(args)))
	}
	if req.ManagerID != nil {
		args = append(args, *req.ManagerID)
		sets = append(sets, fmt.Sprintf("manager_id = $%d", len(args)))
	}
	args = append(args, deptID)
	query := fmt.Sprintf(`UPDATE departments SET %s WHERE id = $%d AND deleted_at IS NULL`,
		strings.Join(sets, ", "), len(args))
	tag, err := db.Exec(ctx, query, args...)
	if err != nil {
		var pgErr *pgconn.PgError
		if errors.As(err, &pgErr) && pgErr.Code == "23505" {
			return api.DepartmentRecord{}, api.ErrDepartmentConflict
		}
		return api.DepartmentRecord{}, err
	}
	if tag.RowsAffected() == 0 {
		return api.DepartmentRecord{}, api.ErrDepartmentNotFound
	}
	return r.GetDepartmentByID(ctx, deptID)
}

func (r DepartmentRepository) DeleteDepartment(ctx context.Context, deptID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE departments SET deleted_at = NOW() WHERE id = $1`, deptID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrDepartmentNotFound
	}
	return nil
}

func (r DepartmentRepository) GetDepartmentPayrollConfigs(ctx context.Context, deptID uuid.UUID) ([]api.DepartmentPayrollConfig, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM departments WHERE id = $1 AND deleted_at IS NULL)`, deptID).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return []api.DepartmentPayrollConfig{}, nil
	}

	query := `SELECT id, effective_from::text, config_key, COALESCE(value_decimal, 0), created_at
		FROM config_entries
		WHERE scope_type = 'department' AND scope_id = $1
		  AND config_key IN ('payroll.fixed_std_days', 'payroll.hours_per_day')
		ORDER BY effective_from DESC, created_at DESC`
	rows, err := db.Query(ctx, query, deptID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	type entry struct {
		id        uuid.UUID
		date      string
		key       string
		val       decimal.Decimal
		createdAt time.Time
	}
	grouped := make(map[string]*api.DepartmentPayrollConfig)
	order := make([]string, 0)

	for rows.Next() {
		var e entry
		if err := rows.Scan(&e.id, &e.date, &e.key, &e.val, &e.createdAt); err != nil {
			return nil, err
		}
		cfg, exists := grouped[e.date]
		if !exists {
			cfg = &api.DepartmentPayrollConfig{
				ID:            e.id,
				DepartmentID:  deptID,
				EffectiveDate: e.date,
				CreatedAt:     e.createdAt.UTC().Format(time.RFC3339),
			}
			grouped[e.date] = cfg
			order = append(order, e.date)
		}
		if e.key == "payroll.fixed_std_days" {
			cfg.StdWorkingDays = e.val
			cfg.ID = e.id
		} else if e.key == "payroll.hours_per_day" {
			cfg.HoursPerDay = e.val
		}
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	result := make([]api.DepartmentPayrollConfig, 0, len(order))
	for _, d := range order {
		result = append(result, *grouped[d])
	}
	return result, nil
}

func (r DepartmentRepository) ListDepartmentPayrollReconciliation(ctx context.Context, page, limit int, status string) (api.DepartmentPayrollReconciliationResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.DepartmentPayrollReconciliationResponse{}, err
	}
	offset := (page - 1) * limit
	where := ""
	args := []any{limit, offset}
	if status != "" {
		where = " WHERE reconciliation_status = $3"
		args = append(args, status)
	}
	query := `SELECT department_id, department_name, legacy_std_working_days, canonical_effective_from::text, canonical_std_working_days, reconciliation_status
		FROM department_payroll_reconciliation` + where + `
		ORDER BY department_name ASC, department_id ASC
		LIMIT $1 OFFSET $2`
	rows, err := db.Query(ctx, query, args...)
	if err != nil {
		return api.DepartmentPayrollReconciliationResponse{}, err
	}
	defer rows.Close()
	result := api.DepartmentPayrollReconciliationResponse{Data: make([]api.DepartmentPayrollReconciliation, 0), Page: page, Limit: limit}
	for rows.Next() {
		var item api.DepartmentPayrollReconciliation
		if err := rows.Scan(&item.DepartmentID, &item.DepartmentName, &item.LegacyStdWorkingDays, &item.CanonicalEffectiveFrom, &item.CanonicalStdWorkingDays, &item.Status); err != nil {
			return api.DepartmentPayrollReconciliationResponse{}, err
		}
		result.Data = append(result.Data, item)
	}
	if err := rows.Err(); err != nil {
		return api.DepartmentPayrollReconciliationResponse{}, err
	}
	countQuery := `SELECT count(*) FROM department_payroll_reconciliation`
	countArgs := []any{}
	if status != "" {
		countQuery += ` WHERE reconciliation_status = $1`
		countArgs = append(countArgs, status)
	}
	if err := db.QueryRow(ctx, countQuery, countArgs...).Scan(&result.Total); err != nil {
		return api.DepartmentPayrollReconciliationResponse{}, err
	}
	return result, nil
}

func (r DepartmentRepository) SetDepartmentPayrollConfig(ctx context.Context, deptID uuid.UUID, req api.SetDepartmentPayrollConfigRequest, actorID uuid.UUID) (api.DepartmentPayrollConfig, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.DepartmentPayrollConfig{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM departments WHERE id = $1 AND deleted_at IS NULL)`, deptID).Scan(&exists); err != nil {
		return api.DepartmentPayrollConfig{}, err
	}
	if !exists {
		return api.DepartmentPayrollConfig{}, api.ErrDepartmentNotFound
	}

	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.DepartmentPayrollConfig{}, errors.New("department repository: database does not support transactions")
	}

	upsertQuery := `INSERT INTO config_entries (id, scope_type, scope_id, config_key, value_decimal, effective_from, created_by, created_at, updated_at)
		VALUES ($1, 'department', $2, $3, $4, $5::date, $6, NOW(), NOW())
		ON CONFLICT (scope_type, scope_id, config_key, effective_from)
		DO UPDATE SET value_decimal = EXCLUDED.value_decimal, updated_at = NOW()
		RETURNING id, created_at`

	fixedID := uuid.New()
	var finalID uuid.UUID
	var createdAt time.Time
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := tx.QueryRow(ctx, upsertQuery, fixedID, deptID, "payroll.fixed_std_days", req.StdWorkingDays, req.EffectiveDate, actorID).Scan(&finalID, &createdAt); err != nil {
			return err
		}
		hoursID := uuid.New()
		var dummyID uuid.UUID
		var dummyTime time.Time
		return tx.QueryRow(ctx, upsertQuery, hoursID, deptID, "payroll.hours_per_day", req.HoursPerDay, req.EffectiveDate, actorID).Scan(&dummyID, &dummyTime)
	})
	if err != nil {
		return api.DepartmentPayrollConfig{}, err
	}

	return api.DepartmentPayrollConfig{
		ID:             finalID,
		DepartmentID:   deptID,
		StdWorkingDays: req.StdWorkingDays,
		HoursPerDay:    req.HoursPerDay,
		EffectiveDate:  req.EffectiveDate,
		CreatedAt:      createdAt.UTC().Format(time.RFC3339),
	}, nil
}
