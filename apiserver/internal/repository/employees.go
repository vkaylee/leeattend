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
	Revoker SessionRevoker
}

func (r EmployeeRepository) begin(ctx context.Context) (service.TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "employee repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return nil, errors.New("employee repository: database does not support transactions")
	}
	return beginner, nil
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
	code := strings.TrimSpace(req.EmployeeIDNumber)
	if err := tx.QueryRow(ctx, `INSERT INTO employees (full_name, tags) VALUES ($1, $2) RETURNING id, full_name, tags, created_at, updated_at`, strings.TrimSpace(req.FullName), req.Tags).
		Scan(&record.ID, &record.FullName, &record.Tags, &createdAt, &updatedAt); err != nil {
		return api.EmployeeRecord{}, mapEmployeeWriteError(err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO employee_profiles (employee_id, phone, date_of_birth, national_id, phone_lookup_hash, national_id_lookup_hash) VALUES ($1, $2, $3::date, $4, $5, $6)`, record.ID, req.Phone, req.DateOfBirth, req.NationalID, req.PhoneLookupHash, req.NationalIDLookupHash); err != nil {
		return api.EmployeeRecord{}, mapEmployeeWriteError(err)
	}
	var tenureID uuid.UUID
	var tenureStatus string
	if err := tx.QueryRow(ctx, `
		INSERT INTO employee_tenures (employee_id, user_id, employee_number, hire_type, start_date)
		VALUES ($1, $2, $3, 'initial', $4::date)
		RETURNING id, CASE WHEN start_date > `+orgTodaySQL+` THEN 'planned' ELSE 'active' END
	`, record.ID, req.UserID, code, req.HireDate).Scan(&tenureID, &tenureStatus); err != nil {
		return api.EmployeeRecord{}, mapEmployeeWriteError(err)
	}
	record.UserID = req.UserID
	record.EmployeeIDNumber = code
	record.TenureID = &tenureID
	record.TenureStatus = &tenureStatus
	hireType := "initial"
	record.HireType = &hireType

	if req.EmployerID != nil {
		relationshipID := uuid.New()
		if err := tx.QueryRow(ctx, `INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date, created_at, updated_at) VALUES ($1, $2, $3, $4, 'active', $5::date, NOW(), NOW()) RETURNING id`, relationshipID, record.ID, *req.EmployerID, tenureID, req.HireDate).Scan(new(uuid.UUID)); err != nil {
			return api.EmployeeRecord{}, mapEmployeeWriteError(err)
		}
		employmentID := uuid.New()
		if _, err := tx.Exec(ctx, `INSERT INTO employment_segments (id, employee_id, employer_id, relationship_id, primary_location_id, department_id, status, start_date, employment_type, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 'active', $7::date, 'full_time', NOW(), NOW())`, employmentID, record.ID, *req.EmployerID, relationshipID, req.PrimaryLocationID, req.DepartmentID, req.HireDate); err != nil {
			return api.EmployeeRecord{}, mapEmployeeWriteError(err)
		}
		record.ActiveEmploymentID = &employmentID
		record.EmployerID = req.EmployerID
		record.PrimaryLocationID = req.PrimaryLocationID
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

const sqlEmployeeDisplaySelect = `
	SELECT
		e.id,
		tenure.user_id,
		e.full_name,
		COALESCE(tenure.employee_number, ''),
		e.tags,
		e.created_at,
		e.updated_at,
		d.name AS department_name,
		emp.id AS active_employment_id,
		emp.employer_id,
		emp.primary_location_id,
		tenure.id AS tenure_id,
		tenure.tenure_status,
		tenure.hire_type
	FROM employees e
	` + sqlDisplayTenureLateral + `
	LEFT JOIN LATERAL (
		SELECT em.id, em.employer_id, em.department_id, em.primary_location_id
		FROM employment_segments em
		JOIN employment_relationships rel ON rel.id = em.relationship_id AND rel.deleted_at IS NULL
		WHERE em.employee_id = e.id
		  AND em.deleted_at IS NULL
		  AND em.status = 'active'
		  AND (tenure.id IS NULL OR rel.tenure_id = tenure.id)
		ORDER BY
			CASE
				WHEN em.start_date <= ` + orgTodaySQL + ` AND (em.end_date IS NULL OR em.end_date >= ` + orgTodaySQL + `) THEN 0
				WHEN em.start_date > ` + orgTodaySQL + ` AND em.end_date IS NULL THEN 1
				ELSE 2
			END ASC,
			em.start_date DESC,
			em.id DESC
		LIMIT 1
	) emp ON TRUE
	LEFT JOIN departments d ON d.id = emp.department_id
`

func scanEmployeeRecord(row interface{ Scan(...any) error }) (api.EmployeeRecord, error) {
	var record api.EmployeeRecord
	var createdAt, updatedAt time.Time
	if err := row.Scan(
		&record.ID,
		&record.UserID,
		&record.FullName,
		&record.EmployeeIDNumber,
		&record.Tags,
		&createdAt,
		&updatedAt,
		&record.DepartmentName,
		&record.ActiveEmploymentID,
		&record.EmployerID,
		&record.PrimaryLocationID,
		&record.TenureID,
		&record.TenureStatus,
		&record.HireType,
	); err != nil {
		return api.EmployeeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

func (r EmployeeRepository) ListEmployees(ctx context.Context, page, limit int, search string) (api.EmployeeListResponse, error) {
	db, err := requireDB(r.DB, "employee repository")
	if err != nil {
		return api.EmployeeListResponse{}, err
	}
	offset := (page - 1) * limit
	pattern := "%" + search + "%"
	rows, err := db.Query(ctx, sqlEmployeeDisplaySelect+`
		WHERE e.deleted_at IS NULL
		  AND (
		      $1 = '%%'
		      OR e.full_name ILIKE $1
		      OR EXISTS (
		          SELECT 1
		          FROM employee_tenures search_t
		          WHERE search_t.employee_id = e.id
		            AND search_t.deleted_at IS NULL
		            AND search_t.employee_number ILIKE $1
		      )
		  )
		ORDER BY COALESCE(tenure.employee_number, '') ASC, e.id ASC
		LIMIT $2 OFFSET $3
	`, pattern, limit, offset)
	if err != nil {
		return api.EmployeeListResponse{}, err
	}
	defer rows.Close()
	result := api.EmployeeListResponse{Data: make([]api.EmployeeRecord, 0), Page: page, Limit: limit}
	for rows.Next() {
		record, err := scanEmployeeRecord(rows)
		if err != nil {
			return api.EmployeeListResponse{}, err
		}
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
	db, err := requireDB(r.DB, "employee repository")
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	return scanEmployeeRecord(db.QueryRow(ctx, sqlEmployeeDisplaySelect+` WHERE e.id = $1 AND e.deleted_at IS NULL`, employeeID))
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
		sets := []string{"updated_at = CURRENT_TIMESTAMP"}
		args := []any{}
		add := func(col string, val any) {
			args = append(args, val)
			sets = append(sets, fmt.Sprintf("%s = $%d", col, len(args)))
		}
		if req.FullName != nil {
			add("full_name", strings.TrimSpace(*req.FullName))
		}
		if req.Tags != nil {
			add("tags", *req.Tags)
		}
		if req.RecruitmentSourceID != nil {
			add("recruitment_source_id", *req.RecruitmentSourceID)
		}
		args = append(args, employeeID)
		idIdx := len(args)
		query := fmt.Sprintf(`UPDATE employees SET %s WHERE id = $%d AND deleted_at IS NULL RETURNING id`, strings.Join(sets, ", "), idIdx)
		var updatedID uuid.UUID
		if err := tx.QueryRow(ctx, query, args...).Scan(&updatedID); err != nil {
			return mapEmployeeWriteError(err)
		}

		if req.UserID != nil {
			var openTenureID uuid.UUID
			var prevUserID *uuid.UUID
			err := tx.QueryRow(ctx, `
				SELECT id, user_id
				FROM employee_tenures
				WHERE employee_id = $1
				  AND deleted_at IS NULL
				  AND end_date IS NULL
				ORDER BY start_date DESC, id DESC
				LIMIT 1
				FOR UPDATE
			`, employeeID).Scan(&openTenureID, &prevUserID)
			if errors.Is(err, pgx.ErrNoRows) {
				return fmt.Errorf("%w: no open tenure for user account link", api.ErrConflict)
			}
			if err != nil {
				return err
			}

			var targetUserID *uuid.UUID
			if *req.UserID != nil {
				targetUserID = *req.UserID
				if req.ForceTransfer != nil && *req.ForceTransfer {
					if _, err := tx.Exec(ctx, `
						UPDATE employee_tenures
						SET user_id = NULL, updated_at = CURRENT_TIMESTAMP
						WHERE user_id = $1
						  AND id <> $2
						  AND end_date IS NULL
						  AND deleted_at IS NULL
					`, *targetUserID, openTenureID); err != nil {
						return err
					}
					if err := revokeUserSessionsDBTX(ctx, tx, r.Revoker, *targetUserID); err != nil {
						return err
					}
				}
			}

			if _, err := tx.Exec(ctx, `UPDATE employee_tenures SET user_id = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2`, targetUserID, openTenureID); err != nil {
				return mapEmployeeWriteError(err)
			}
			if prevUserID != nil && (targetUserID == nil || *prevUserID != *targetUserID) {
				if err := revokeUserSessionsDBTX(ctx, tx, r.Revoker, *prevUserID); err != nil {
					return err
				}
			}
		}

		loaded, err := scanEmployeeRecord(tx.QueryRow(ctx, sqlEmployeeDisplaySelect+` WHERE e.id = $1 AND e.deleted_at IS NULL`, employeeID))
		if err != nil {
			return err
		}
		record = loaded
		return nil
	})
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	return record, nil
}

func (r EmployeeRepository) SoftDeleteEmployee(ctx context.Context, employeeID uuid.UUID) error {
	beginner, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		command, err := tx.Exec(ctx, `UPDATE employees SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NULL`, employeeID)
		if err != nil {
			return err
		}
		if command.RowsAffected() == 0 {
			return errNoRows
		}
		rows, err := tx.Query(ctx, `SELECT DISTINCT user_id FROM employee_tenures WHERE employee_id = $1 AND user_id IS NOT NULL AND end_date IS NULL AND deleted_at IS NULL`, employeeID)
		if err != nil {
			return err
		}
		var userIDs []uuid.UUID
		for rows.Next() {
			var uid uuid.UUID
			if err := rows.Scan(&uid); err != nil {
				rows.Close()
				return err
			}
			userIDs = append(userIDs, uid)
		}
		rows.Close()
		if err := rows.Err(); err != nil {
			return err
		}
		for _, uid := range userIDs {
			if err := revokeUserSessionsDBTX(ctx, tx, r.Revoker, uid); err != nil {
				return err
			}
		}
		return nil
	})
}
