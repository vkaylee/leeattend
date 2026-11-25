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
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type EmployeeRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (service.TxBeginner, error)
}

func (r EmployeeRepository) begin(ctx context.Context) (service.TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return nil, errors.New("employee repository: database does not support transactions")
	}
	return beginner, nil
}

func (r EmployeeRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("employee repository: database is not configured")
	}
	return r.DB, nil
}

func (r EmployeeRepository) CreateEmployee(ctx context.Context, req api.CreateEmployeeRequest) (api.EmployeeRecord, error) {
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := lockAndVerifyNewEmployeeUniqueness(ctx, tx, req); err != nil {
			return err
		}
		if err := validateEmploymentScopeRefs(ctx, tx, req.EmployerID, req.DepartmentID, req.PrimaryLocationID); err != nil {
			return err
		}
		created, err := insertNewEmployeeGraph(ctx, tx, req)
		if err != nil {
			return err
		}
		record = created
		return nil
	})
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	return record, nil
}

func lockAndVerifyNewEmployeeUniqueness(ctx context.Context, tx pgx.Tx, req api.CreateEmployeeRequest) error {
	normalizedCode := identity.NormalizeEmployeeCode(req.EmployeeIDNumber)
	if normalizedCode == "" {
		return api.ErrInvalidInput
	}
	keys := []string{"emp_code:" + normalizedCode}
	if req.NationalIDLookupHash != nil && *req.NationalIDLookupHash != "" {
		keys = append(keys, "emp_nid:"+*req.NationalIDLookupHash)
	}
	if req.PhoneLookupHash != nil && *req.PhoneLookupHash != "" {
		keys = append(keys, "emp_phone:"+*req.PhoneLookupHash)
	}
	if err := lockEmployeeAdvisoryKeys(ctx, tx, keys); err != nil {
		return err
	}
	if err := checkNormalizedCodeOwnership(ctx, tx, normalizedCode, uuid.Nil); err != nil {
		return err
	}
	return checkIdentityHashUniqueness(ctx, tx, req.NationalIDLookupHash, req.PhoneLookupHash)
}

func insertNewEmployeeGraph(ctx context.Context, tx pgx.Tx, req api.CreateEmployeeRequest) (api.EmployeeRecord, error) {
	var record api.EmployeeRecord
	var createdAt, updatedAt time.Time
	if err := tx.QueryRow(ctx, `INSERT INTO employees (employee_id_number, user_id, full_name, tags, hire_date, original_hire_date) VALUES ($1, $2, $3, $4, $5::date, $5::date) RETURNING id, user_id, full_name, employee_id_number, tags, created_at, updated_at`, strings.TrimSpace(req.EmployeeIDNumber), req.UserID, strings.TrimSpace(req.FullName), req.Tags, req.HireDate).
		Scan(&record.ID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &createdAt, &updatedAt); err != nil {
		return api.EmployeeRecord{}, mapEmployeeWriteError(err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO employee_profiles (employee_id, phone, date_of_birth, national_id, phone_lookup_hash, national_id_lookup_hash) VALUES ($1, $2, $3::date, $4, $5, $6)`, record.ID, req.Phone, req.DateOfBirth, req.NationalID, req.PhoneLookupHash, req.NationalIDLookupHash); err != nil {
		return api.EmployeeRecord{}, mapEmployeeWriteError(err)
	}
	var tenureID uuid.UUID
	if err := tx.QueryRow(ctx, `INSERT INTO employee_tenures (employee_id, user_id, employee_number, start_date) VALUES ($1, $2, $3, $4::date) RETURNING id`, record.ID, req.UserID, record.EmployeeIDNumber, req.HireDate).Scan(&tenureID); err != nil {
		return api.EmployeeRecord{}, mapEmployeeWriteError(err)
	}
	if req.EmployerID != nil {
		relationshipID := uuid.New()
		if err := tx.QueryRow(ctx, `INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date, created_at, updated_at) VALUES ($1, $2, $3, $4, 'active', $5::date, NOW(), NOW()) RETURNING id`, relationshipID, record.ID, *req.EmployerID, tenureID, req.HireDate).Scan(new(uuid.UUID)); err != nil {
			return api.EmployeeRecord{}, mapEmployeeWriteError(err)
		}
		if _, err := tx.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, relationship_id, primary_location_id, department_id, status, start_date, employment_type, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 'active', $7::date, 'full_time', NOW(), NOW())`, uuid.New(), record.ID, *req.EmployerID, relationshipID, req.PrimaryLocationID, req.DepartmentID, req.HireDate); err != nil {
			return api.EmployeeRecord{}, mapEmployeeWriteError(err)
		}
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
	rows, err := db.Query(ctx, `SELECT e.id, e.user_id, e.full_name, COALESCE(tenure.employee_number, ''), e.tags, e.created_at, e.updated_at, d.name AS department_name, emp.id AS active_employment_id, emp.employer_id, NULL::uuid AS primary_location_id FROM employees e LEFT JOIN employments emp ON emp.employee_id = e.id AND emp.status = 'active' AND emp.deleted_at IS NULL AND emp.start_date <= CURRENT_DATE AND (emp.end_date IS NULL OR emp.end_date >= CURRENT_DATE) LEFT JOIN departments d ON d.id = emp.department_id LEFT JOIN LATERAL (SELECT t.employee_number FROM employee_tenures t WHERE t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE) ORDER BY t.start_date DESC, t.id DESC LIMIT 1) tenure ON TRUE WHERE e.deleted_at IS NULL AND ($1 = '%%' OR e.full_name ILIKE $1 OR EXISTS (SELECT 1 FROM employee_tenures search_t WHERE search_t.employee_id = e.id AND search_t.deleted_at IS NULL AND search_t.employee_number ILIKE $1)) ORDER BY COALESCE(tenure.employee_number, '') ASC, e.id ASC LIMIT $2 OFFSET $3`, pattern, limit, offset)
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
	if err := db.QueryRow(ctx, `SELECT count(*) FROM employees e WHERE e.deleted_at IS NULL AND ($1 = '%%' OR e.full_name ILIKE $1 OR EXISTS (SELECT 1 FROM employee_tenures t WHERE t.employee_id = e.id AND t.deleted_at IS NULL AND t.employee_number ILIKE $1))`, pattern).Scan(&result.Total); err != nil {
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
	err = db.QueryRow(ctx, `SELECT e.id, e.user_id, e.full_name, COALESCE(tenure.employee_number, ''), e.tags, e.created_at, e.updated_at, d.name AS department_name, emp.id AS active_employment_id, emp.employer_id, NULL::uuid AS primary_location_id FROM employees e LEFT JOIN employments emp ON emp.employee_id = e.id AND emp.status = 'active' AND emp.deleted_at IS NULL AND emp.start_date <= CURRENT_DATE AND (emp.end_date IS NULL OR emp.end_date >= CURRENT_DATE) LEFT JOIN departments d ON d.id = emp.department_id LEFT JOIN LATERAL (SELECT t.employee_number FROM employee_tenures t WHERE t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE) ORDER BY t.start_date DESC, t.id DESC LIMIT 1) tenure ON TRUE WHERE e.id = $1 AND e.deleted_at IS NULL`, employeeID).
		Scan(&record.ID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &createdAt, &updatedAt, &record.DepartmentName, &record.ActiveEmploymentID, &record.EmployerID, &record.PrimaryLocationID)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

func (r EmployeeRepository) UpdateEmployee(ctx context.Context, employeeID uuid.UUID, req api.UpdateEmployeeRequest) (api.EmployeeRecord, error) {
	if req.EmployeeIDNumber != nil {
		return api.EmployeeRecord{}, api.ErrConflict
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if req.UserID != nil && *req.UserID != nil && req.ForceTransfer != nil && *req.ForceTransfer {
			if _, err := tx.Exec(ctx, `UPDATE employees SET user_id = NULL WHERE user_id = $1 AND id != $2`, **req.UserID, employeeID); err != nil {
				return err
			}
			if _, err := tx.Exec(ctx, `UPDATE employee_tenures SET user_id = NULL, updated_at = CURRENT_TIMESTAMP WHERE user_id = $1`, **req.UserID); err != nil {
				return err
			}
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
		var createdAt, updatedAt time.Time
		if err := tx.QueryRow(ctx, query, args...).Scan(&record.ID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &createdAt, &updatedAt); err != nil {
			return mapConstraintError(err)
		}
		if req.UserID != nil {
			var userID any
			if *req.UserID != nil {
				userID = **req.UserID
			}
			if _, err := tx.Exec(ctx, `UPDATE employee_tenures SET user_id = $1, updated_at = CURRENT_TIMESTAMP WHERE employee_id = $2 AND deleted_at IS NULL AND start_date <= CURRENT_DATE AND (end_date IS NULL OR end_date >= CURRENT_DATE)`, userID, employeeID); err != nil {
				return err
			}
		}
		record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		return nil
	})
	if err != nil {
		return api.EmployeeRecord{}, err
	}
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

func (r EmployeeRepository) RehireEmployee(ctx context.Context, employeeID uuid.UUID, req api.RehireEmployeeRequest) (api.EmployeeRecord, error) {
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var current api.EmployeeRecord
		var createdAt, updatedAt time.Time
		if err := tx.QueryRow(ctx, `SELECT id, user_id, full_name, employee_id_number, tags, created_at, updated_at FROM employees WHERE id = $1 AND (deleted_at IS NOT NULL OR NOT EXISTS (SELECT 1 FROM employments WHERE employee_id = employees.id AND status = 'active' AND deleted_at IS NULL AND start_date <= $2::date AND (end_date IS NULL OR end_date >= $2::date))) FOR UPDATE`, employeeID, req.HireDate).
			Scan(&current.ID, &current.UserID, &current.FullName, &current.EmployeeIDNumber, &current.Tags, &createdAt, &updatedAt); err != nil {
			return err
		}
		employeeNumber := current.EmployeeIDNumber
		if req.EmployeeIDNumber != nil && strings.TrimSpace(*req.EmployeeIDNumber) != "" {
			employeeNumber = strings.TrimSpace(*req.EmployeeIDNumber)
		}
		if req.EmployerID == nil {
			return errors.New("rehire employer is required")
		}
		if _, err := tx.Exec(ctx, `UPDATE employee_tenures SET end_date = ($2::date - 1), updated_at = CURRENT_TIMESTAMP WHERE employee_id = $1 AND deleted_at IS NULL AND end_date IS NULL`, employeeID, req.HireDate); err != nil {
			return err
		}
		if _, err := tx.Exec(ctx, `UPDATE employment_relationships SET status = 'terminated', end_date = ($2::date - 1), updated_at = CURRENT_TIMESTAMP WHERE employee_id = $1 AND deleted_at IS NULL AND end_date IS NULL`, employeeID, req.HireDate); err != nil {
			return err
		}
		if _, err := tx.Exec(ctx, `UPDATE employees SET employee_id_number = $2, user_id = NULL, deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = $1`, employeeID, employeeNumber); err != nil {
			return err
		}
		var tenureID uuid.UUID
		if err := tx.QueryRow(ctx, `INSERT INTO employee_tenures (employee_id, employee_number, start_date) VALUES ($1, $2, $3::date) RETURNING id`, employeeID, employeeNumber, req.HireDate).Scan(&tenureID); err != nil {
			return mapConstraintError(err)
		}
		relationshipID := uuid.New()
		if err := tx.QueryRow(ctx, `INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date, created_at, updated_at) VALUES ($1, $2, $3, $4, 'active', $5::date, NOW(), NOW()) RETURNING id`, relationshipID, employeeID, *req.EmployerID, tenureID, req.HireDate).Scan(new(uuid.UUID)); err != nil {
			return mapConstraintError(err)
		}
		if err := tx.QueryRow(ctx, `INSERT INTO employments (id, employee_id, employer_id, relationship_id, primary_location_id, status, start_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'active', $6::date, NOW(), NOW()) RETURNING id`, uuid.New(), employeeID, *req.EmployerID, relationshipID, req.PrimaryLocationID, req.HireDate).Scan(new(uuid.UUID)); err != nil {
			return mapConstraintError(err)
		}
		current.UserID = nil
		current.EmployeeIDNumber = employeeNumber
		current.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		current.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		record = current
		return nil
	})
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	return record, nil
}
