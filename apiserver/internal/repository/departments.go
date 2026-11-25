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
	"github.com/vkaylee/leeattend/apiserver/internal/api"
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
