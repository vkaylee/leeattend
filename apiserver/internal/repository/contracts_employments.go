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
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type ContractRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r ContractRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "contract repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, nil
	}
	return beginner, nil
}

func scanContract(row interface{ Scan(...any) error }) (api.EmployeeContract, error) {
	var c api.EmployeeContract
	var created, updated time.Time
	if err := row.Scan(&c.ID, &c.EmploymentID, &c.ContractNumber, &c.ContractType, &c.Status, &c.StartDate, &c.EndDate, &c.DocumentID, &c.Notes, &created, &updated); err != nil {
		return api.EmployeeContract{}, err
	}
	c.CreatedAt = created.UTC().Format(time.RFC3339)
	c.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return c, nil
}

func (r ContractRepository) ListContracts(ctx context.Context) ([]api.EmployeeContractWithEmployee, error) {
	db, err := requireDB(r.DB, "contract repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		c.id, c.employment_id, c.contract_number, c.contract_type::text,
		c.status::text, c.start_date::text, c.end_date::text, c.document_id, c.notes,
		c.created_at, c.updated_at,
		e.full_name, COALESCE(t.employee_number, ''), e.id
	FROM employee_contracts c
	INNER JOIN employments em ON em.id = c.employment_id
	INNER JOIN employees e ON e.id = em.employee_id
	LEFT JOIN employment_relationships er ON er.id = em.relationship_id
	LEFT JOIN employee_tenures t ON t.id = er.tenure_id
		AND t.employee_id = e.id
		AND t.deleted_at IS NULL
		AND t.start_date <= c.start_date
		AND (t.end_date IS NULL OR t.end_date >= c.start_date)
	WHERE c.deleted_at IS NULL
	  AND em.deleted_at IS NULL
	  AND e.deleted_at IS NULL
	ORDER BY c.start_date DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.EmployeeContractWithEmployee, 0)
	for rows.Next() {
		var item api.EmployeeContractWithEmployee
		var created, updated time.Time
		if err := rows.Scan(
			&item.ID, &item.EmploymentID, &item.ContractNumber, &item.ContractType,
			&item.Status, &item.StartDate, &item.EndDate, &item.DocumentID, &item.Notes,
			&created, &updated,
			&item.EmployeeName, &item.EmployeeCode, &item.EmployeeID,
		); err != nil {
			return nil, err
		}
		item.CreatedAt = created.UTC().Format(time.RFC3339)
		item.UpdatedAt = updated.UTC().Format(time.RFC3339)
		result = append(result, item)
	}
	return result, rows.Err()
}

func (r ContractRepository) ListEmployeeContracts(ctx context.Context, employeeID uuid.UUID) ([]api.EmployeeContract, error) {
	db, err := requireDB(r.DB, "contract repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		c.id, c.employment_id, c.contract_number, c.contract_type::text,
		c.status::text, c.start_date::text, c.end_date::text, c.document_id, c.notes,
		c.created_at, c.updated_at
	FROM employee_contracts c
	INNER JOIN employments em ON em.id = c.employment_id
	WHERE em.employee_id = $1
	  AND c.deleted_at IS NULL
	  AND em.deleted_at IS NULL
	ORDER BY em.start_date DESC, c.start_date DESC`, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.EmployeeContract, 0)
	for rows.Next() {
		item, err := scanContract(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (r ContractRepository) GetContract(ctx context.Context, contractID uuid.UUID) (api.EmployeeContract, error) {
	db, err := requireDB(r.DB, "contract repository")
	if err != nil {
		return api.EmployeeContract{}, err
	}
	item, err := scanContract(db.QueryRow(ctx, `SELECT
		id, employment_id, contract_number, contract_type::text,
		status::text, start_date::text, end_date::text, document_id, notes,
		created_at, updated_at
	FROM employee_contracts
	WHERE id = $1 AND deleted_at IS NULL`, contractID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeeContract{}, api.ErrContractNotFound
	}
	return item, err
}

func (r ContractRepository) CreateContract(ctx context.Context, req api.CreateContractRequest) (api.EmployeeContract, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.EmployeeContract{}, err
	}
	if txBeginner != nil {
		var created api.EmployeeContract
		err := service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
			res, txErr := r.createContractTx(ctx, tx, req)
			if txErr != nil {
				return txErr
			}
			created = res
			return nil
		})
		return created, err
	}
	db, err := requireDB(r.DB, "contract repository")
	if err != nil {
		return api.EmployeeContract{}, err
	}
	return r.createContractTx(ctx, db, req)
}

func (r ContractRepository) createContractTx(ctx context.Context, db DBTX, req api.CreateContractRequest) (api.EmployeeContract, error) {
	rows, err := db.Query(ctx, `SELECT id FROM employee_contracts
		WHERE employment_id = $1
		  AND status = 'active'
		  AND start_date <= $2::date
		  AND (end_date IS NULL OR end_date >= $2::date)
		  AND deleted_at IS NULL`, req.EmploymentID, req.StartDate)
	if err != nil {
		return api.EmployeeContract{}, err
	}
	defer rows.Close()
	var overlapIDs []uuid.UUID
	for rows.Next() {
		var oID uuid.UUID
		if err := rows.Scan(&oID); err != nil {
			return api.EmployeeContract{}, err
		}
		overlapIDs = append(overlapIDs, oID)
	}
	if err := rows.Err(); err != nil {
		return api.EmployeeContract{}, err
	}

	if len(overlapIDs) > 0 {
		if req.AutoCloseOverlapping == nil || !*req.AutoCloseOverlapping {
			return api.EmployeeContract{}, api.ErrContractOverlap
		}
		startT, _ := time.Parse("2006-01-02", req.StartDate)
		newEndDate := startT.AddDate(0, 0, -1).Format("2006-01-02")
		autoCloseNote := fmt.Sprintf("Hệ thống tự động chấm dứt sớm vào ngày %s để ký hợp đồng mới", startT.AddDate(0, 0, -1).Format("02/01/2006"))
		if req.ContractNumber != nil && *req.ContractNumber != "" {
			autoCloseNote += " " + *req.ContractNumber
		}
		autoCloseNote += "."

		_, err = db.Exec(ctx, `UPDATE employee_contracts
			SET end_date = $1::date,
			    status = 'expired',
			    updated_at = NOW(),
			    notes = CASE
			        WHEN notes IS NULL OR notes = '' THEN $2
			        ELSE notes || E'\n---\n' || $2
			    END
			WHERE id = ANY($3)`, newEndDate, autoCloseNote, overlapIDs)
		if err != nil {
			return api.EmployeeContract{}, err
		}
	}

	contractID := uuid.New()
	item, err := scanContract(db.QueryRow(ctx, `INSERT INTO employee_contracts
		(id, employment_id, contract_number, contract_type, status, start_date, end_date, document_id, notes, created_at, updated_at)
		VALUES ($1, $2, $3, $4, 'active', $5::date, $6::date, $7, $8, NOW(), NOW())
		RETURNING id, employment_id, contract_number, contract_type::text, status::text, start_date::text, end_date::text, document_id, notes, created_at, updated_at`,
		contractID, req.EmploymentID, req.ContractNumber, req.ContractType, req.StartDate, req.EndDate, req.DocumentID, req.Notes))
	if err != nil {
		return api.EmployeeContract{}, mapConstraintError(err)
	}
	return item, nil
}

func (r ContractRepository) UpdateContract(ctx context.Context, contractID uuid.UUID, req api.UpdateContractRequest) (api.EmployeeContract, error) {
	db, err := requireDB(r.DB, "contract repository")
	if err != nil {
		return api.EmployeeContract{}, err
	}
	sets := []string{"updated_at = NOW()"}
	args := make([]any, 0, 9)
	add := func(col string, val any) {
		args = append(args, val)
		sets = append(sets, fmt.Sprintf("%s = $%d", col, len(args)))
	}
	if req.ContractNumber != nil {
		add("contract_number", *req.ContractNumber)
	}
	if req.ContractType != nil {
		add("contract_type", *req.ContractType)
	}
	if req.Status != nil {
		add("status", *req.Status)
	}
	if req.StartDate != nil {
		add("start_date", *req.StartDate)
	}
	if req.EndDatePresent {
		if req.EndDate == nil || *req.EndDate == nil {
			sets = append(sets, "end_date = NULL")
		} else {
			add("end_date", **req.EndDate)
		}
	}
	if req.DocumentIDPresent {
		if req.DocumentID == nil || *req.DocumentID == nil {
			sets = append(sets, "document_id = NULL")
		} else {
			add("document_id", **req.DocumentID)
		}
	}
	if req.NotesPresent {
		if req.Notes == nil || *req.Notes == nil {
			sets = append(sets, "notes = NULL")
		} else {
			add("notes", **req.Notes)
		}
	}
	args = append(args, contractID)
	query := fmt.Sprintf(`UPDATE employee_contracts SET %s WHERE id = $%d AND deleted_at IS NULL
		RETURNING id, employment_id, contract_number, contract_type::text, status::text, start_date::text, end_date::text, document_id, notes, created_at, updated_at`,
		strings.Join(sets, ", "), len(args))
	item, err := scanContract(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeeContract{}, api.ErrContractNotFound
	}
	return item, err
}

func (r ContractRepository) SoftDeleteContract(ctx context.Context, contractID uuid.UUID) error {
	db, err := requireDB(r.DB, "contract repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE employee_contracts SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1`, contractID)
	return err
}

type EmploymentRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
	Revoker SessionRevoker
}

func (r EmploymentRepository) beginEmploymentTx(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "employment repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("employment repository: database does not support transactions")
	}
	return beginner, nil
}

func scanEmployment(row interface{ Scan(...any) error }) (api.Employment, error) {
	var e api.Employment
	var created, updated time.Time
	if err := row.Scan(
		&e.ID, &e.EmployeeID, &e.EmployerID, &e.PrimaryLocationID, &e.DepartmentID,
		&e.Status, &e.StartDate, &e.EndDate, &e.JobTitle, &e.EmploymentType, &e.ManagerEmploymentID,
		&created, &updated, &e.TerminationReason, &e.Notes,
	); err != nil {
		return api.Employment{}, err
	}
	e.CreatedAt = created.UTC().Format(time.RFC3339)
	e.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return e, nil
}

const employmentColumns = `id, employee_id, employer_id, primary_location_id, department_id,
	status, start_date::text, end_date::text, job_title, employment_type, manager_employment_id,
	created_at, updated_at, termination_reason, notes`

const relationshipColumns = `id, employee_id, employer_id, tenure_id, status,
	start_date::text, end_date::text, termination_reason, notes, created_at, updated_at`

func scanEmploymentRelationship(row interface{ Scan(...any) error }) (api.EmploymentRelationship, error) {
	var relationship api.EmploymentRelationship
	var created, updated time.Time
	if err := row.Scan(
		&relationship.ID, &relationship.EmployeeID, &relationship.EmployerID, &relationship.TenureID, &relationship.Status,
		&relationship.StartDate, &relationship.EndDate, &relationship.TerminationReason, &relationship.Notes,
		&created, &updated,
	); err != nil {
		return api.EmploymentRelationship{}, err
	}
	relationship.CreatedAt = created.UTC().Format(time.RFC3339)
	relationship.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return relationship, nil
}

func (r EmploymentRepository) ListEmploymentRelationships(ctx context.Context, employeeID uuid.UUID, page, limit int, employerID *uuid.UUID, asOf *time.Time) (api.EmploymentRelationshipListResponse, error) {
	db, err := requireDB(r.DB, "employment repository")
	if err != nil {
		return api.EmploymentRelationshipListResponse{}, err
	}
	args := []any{employeeID}
	where := `employee_id = $1 AND deleted_at IS NULL`
	if employerID != nil {
		args = append(args, *employerID)
		where += fmt.Sprintf(" AND employer_id = $%d", len(args))
	}
	if asOf != nil {
		args = append(args, asOf.Format("2006-01-02"))
		where += fmt.Sprintf(" AND start_date <= $%d::date AND (end_date IS NULL OR end_date >= $%d::date)", len(args), len(args))
	}
	countRow := db.QueryRow(ctx, `SELECT count(*) FROM employment_relationships WHERE `+where, args...)
	var total int64
	if err := countRow.Scan(&total); err != nil {
		return api.EmploymentRelationshipListResponse{}, err
	}
	args = append(args, limit, (page-1)*limit)
	rows, err := db.Query(ctx, `SELECT `+relationshipColumns+` FROM employment_relationships WHERE `+where+` ORDER BY start_date DESC, id DESC LIMIT $`+fmt.Sprint(len(args)-1)+` OFFSET $`+fmt.Sprint(len(args)), args...)
	if err != nil {
		return api.EmploymentRelationshipListResponse{}, err
	}
	defer rows.Close()
	data := make([]api.EmploymentRelationship, 0)
	for rows.Next() {
		item, err := scanEmploymentRelationship(rows)
		if err != nil {
			return api.EmploymentRelationshipListResponse{}, err
		}
		data = append(data, item)
	}
	if err := rows.Err(); err != nil {
		return api.EmploymentRelationshipListResponse{}, err
	}
	return api.EmploymentRelationshipListResponse{Data: data, Total: total, Page: page, Limit: limit}, nil
}

func (r EmploymentRepository) GetEmploymentRelationship(ctx context.Context, id uuid.UUID) (api.EmploymentRelationship, error) {
	db, err := requireDB(r.DB, "employment repository")
	if err != nil {
		return api.EmploymentRelationship{}, err
	}
	item, err := scanEmploymentRelationship(db.QueryRow(ctx, `SELECT `+relationshipColumns+` FROM employment_relationships WHERE id = $1 AND deleted_at IS NULL`, id))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmploymentRelationship{}, api.ErrEmploymentRelationshipNotFound
	}
	return item, err
}

func (r EmploymentRepository) CreateEmploymentRelationship(ctx context.Context, employeeID uuid.UUID, req api.CreateEmploymentRelationshipRequest) (api.EmploymentRelationshipCreated, error) {
	txBeginner, err := r.beginEmploymentTx(ctx)
	if err != nil {
		return api.EmploymentRelationshipCreated{}, err
	}
	var result api.EmploymentRelationshipCreated
	err = service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
		var employeeExists bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&employeeExists); err != nil {
			return err
		}
		if !employeeExists {
			return api.ErrEmployeeNotFound
		}
		var employerExists bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employers WHERE id = $1 AND deleted_at IS NULL)`, req.EmployerID).Scan(&employerExists); err != nil {
			return err
		}
		if !employerExists {
			return api.ErrEmployerNotFound
		}
		if err := tx.QueryRow(ctx, `SELECT id FROM employees WHERE id = $1 AND deleted_at IS NULL FOR UPDATE`, employeeID).Scan(new(uuid.UUID)); err != nil {
			return err
		}
		var tenureID uuid.UUID
		if req.TenureID != nil && *req.TenureID != uuid.Nil {
			err = tx.QueryRow(ctx, `
				SELECT id
				FROM employee_tenures
				WHERE id = $1 AND employee_id = $2 AND deleted_at IS NULL
				FOR UPDATE
			`, *req.TenureID, employeeID).Scan(&tenureID)
			if errors.Is(err, pgx.ErrNoRows) {
				return fmt.Errorf("%w: tenure not found for employee", api.ErrInvalidInput)
			}
			if err != nil {
				return err
			}
		} else {
			err = tx.QueryRow(ctx, `
				SELECT id
				FROM employee_tenures
				WHERE employee_id = $1
				  AND deleted_at IS NULL
				  AND start_date <= $2::date
				  AND (end_date IS NULL OR end_date >= $2::date)
				ORDER BY start_date DESC, id DESC
				LIMIT 1
				FOR UPDATE
			`, employeeID, req.StartDate).Scan(&tenureID)
			if errors.Is(err, pgx.ErrNoRows) {
				return fmt.Errorf("%w: no effective tenure covers relationship start_date", api.ErrConflict)
			}
			if err != nil {
				return err
			}
		}
		relationshipID := uuid.New()
		segmentID := uuid.New()
		result.Relationship, err = scanEmploymentRelationship(tx.QueryRow(ctx, `INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date, end_date, termination_reason, notes, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6::date, $7::date, $8, $9, NOW(), NOW()) RETURNING `+relationshipColumns, relationshipID, employeeID, req.EmployerID, tenureID, req.Status, req.StartDate, req.EndDate, req.TerminationReason, req.Notes))
		if err != nil {
			return mapRelationshipConstraintError(err)
		}
		segment, err := scanEmployment(tx.QueryRow(ctx, `INSERT INTO employments (id, employee_id, employer_id, relationship_id, primary_location_id, department_id, status, start_date, end_date, job_title, employment_type, manager_employment_id, created_at, updated_at, termination_reason, notes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::date, $9::date, $10, $11, $12, NOW(), NOW(), $13, $14) RETURNING `+employmentColumns, segmentID, employeeID, req.EmployerID, relationshipID, req.PrimaryLocationID, req.DepartmentID, req.Status, req.StartDate, req.EndDate, req.JobTitle, req.EmploymentType, req.ManagerEmploymentID, req.TerminationReason, req.Notes))
		if err != nil {
			return mapRelationshipConstraintError(err)
		}
		result.Segment = segment
		return nil
	})
	if err != nil {
		return api.EmploymentRelationshipCreated{}, err
	}
	return result, nil
}

func mapRelationshipConstraintError(err error) error {
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) && (pgErr.Code == "23P01" || pgErr.Code == "23505" || pgErr.Code == "23514") {
		return fmt.Errorf("%w: %s", api.ErrConflict, pgErr.ConstraintName)
	}
	return err
}

func (r EmploymentRepository) ListEmployments(ctx context.Context, employeeID uuid.UUID) ([]api.Employment, error) {
	db, err := requireDB(r.DB, "employment repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+employmentColumns+` FROM employments
		WHERE employee_id = $1 AND deleted_at IS NULL
		ORDER BY start_date DESC`, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.Employment, 0)
	for rows.Next() {
		item, err := scanEmployment(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (r EmploymentRepository) CreateEmployment(ctx context.Context, req api.CreateEmploymentRequest) (api.Employment, error) {
	txBeginner, err := r.beginEmploymentTx(ctx)
	if err != nil {
		return api.Employment{}, err
	}
	var item api.Employment
	err = service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := tx.QueryRow(ctx, `SELECT id FROM employees WHERE id = $1 AND deleted_at IS NULL FOR UPDATE`, req.EmployeeID).Scan(new(uuid.UUID)); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrEmployeeNotFound
			}
			return err
		}

		var relationshipID uuid.UUID
		if req.RelationshipID != nil && *req.RelationshipID != uuid.Nil {
			err := tx.QueryRow(ctx, `
				SELECT id
				FROM employment_relationships
				WHERE id = $1 AND employee_id = $2 AND employer_id = $3 AND deleted_at IS NULL
				FOR UPDATE
			`, *req.RelationshipID, req.EmployeeID, req.EmployerID).Scan(&relationshipID)
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrEmploymentRelationshipNotFound
			}
			if err != nil {
				return err
			}
		} else {
			// Resolve existing covering relationship for same employee+employer, or create one under the covering tenure.
			err := tx.QueryRow(ctx, `
				SELECT id
				FROM employment_relationships
				WHERE employee_id = $1
				  AND employer_id = $2
				  AND deleted_at IS NULL
				  AND start_date <= $3::date
				  AND (end_date IS NULL OR end_date >= $3::date)
				ORDER BY start_date DESC, id DESC
				LIMIT 1
				FOR UPDATE
			`, req.EmployeeID, req.EmployerID, req.StartDate).Scan(&relationshipID)
			if errors.Is(err, pgx.ErrNoRows) {
				var tenureID uuid.UUID
				tErr := tx.QueryRow(ctx, `
					SELECT id
					FROM employee_tenures
					WHERE employee_id = $1
					  AND deleted_at IS NULL
					  AND start_date <= $2::date
					  AND (end_date IS NULL OR end_date >= $2::date)
					ORDER BY start_date DESC, id DESC
					LIMIT 1
					FOR UPDATE
				`, req.EmployeeID, req.StartDate).Scan(&tenureID)
				if errors.Is(tErr, pgx.ErrNoRows) {
					return fmt.Errorf("%w: no effective tenure covers employment start_date", api.ErrConflict)
				}
				if tErr != nil {
					return tErr
				}
				relationshipID = uuid.New()
				if _, err := tx.Exec(ctx, `
					INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date, end_date, created_at, updated_at)
					VALUES ($1, $2, $3, $4, $5, $6::date, $7::date, NOW(), NOW())
				`, relationshipID, req.EmployeeID, req.EmployerID, tenureID, req.Status, req.StartDate, req.EndDate); err != nil {
					return mapRelationshipConstraintError(err)
				}
			} else if err != nil {
				return err
			}
		}

		empID := uuid.New()
		created, err := scanEmployment(tx.QueryRow(ctx, `INSERT INTO employments
			(id, employee_id, employer_id, relationship_id, primary_location_id, department_id, status, start_date, end_date, job_title, employment_type, manager_employment_id, created_at, updated_at)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8::date, $9::date, $10, $11, $12, NOW(), NOW())
			RETURNING `+employmentColumns,
			empID, req.EmployeeID, req.EmployerID, relationshipID, req.PrimaryLocationID, req.DepartmentID, req.Status, req.StartDate, req.EndDate, req.JobTitle, req.EmploymentType, req.ManagerEmploymentID))
		if err != nil {
			return mapRelationshipConstraintError(err)
		}
		item = created
		return nil
	})
	if err != nil {
		return api.Employment{}, err
	}
	return item, nil
}

func (r EmploymentRepository) GetEmployment(ctx context.Context, id uuid.UUID) (api.Employment, error) {
	db, err := requireDB(r.DB, "employment repository")
	if err != nil {
		return api.Employment{}, err
	}
	item, err := scanEmployment(db.QueryRow(ctx, `SELECT `+employmentColumns+` FROM employments
		WHERE id = $1 AND deleted_at IS NULL`, id))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.Employment{}, api.ErrEmploymentNotFound
	}
	return item, err
}

func (r EmploymentRepository) UpdateEmployment(ctx context.Context, id uuid.UUID, req api.UpdateEmploymentRequest) (api.Employment, error) {
	db, err := requireDB(r.DB, "employment repository")
	if err != nil {
		return api.Employment{}, err
	}
	sets := []string{"updated_at = NOW()"}
	args := make([]any, 0, 13)
	add := func(col string, val any) {
		args = append(args, val)
		sets = append(sets, fmt.Sprintf("%s = $%d", col, len(args)))
	}
	if req.EmployerID != nil {
		add("employer_id", *req.EmployerID)
	}
	if req.PrimaryLocationIDPresent {
		if req.PrimaryLocationID == nil || *req.PrimaryLocationID == nil {
			sets = append(sets, "primary_location_id = NULL")
		} else {
			add("primary_location_id", **req.PrimaryLocationID)
		}
	}
	if req.DepartmentIDPresent {
		if req.DepartmentID == nil || *req.DepartmentID == nil {
			sets = append(sets, "department_id = NULL")
		} else {
			add("department_id", **req.DepartmentID)
		}
	}
	if req.Status != nil {
		add("status", *req.Status)
	}
	if req.StartDate != nil {
		add("start_date", *req.StartDate)
	}
	if req.EndDatePresent {
		if req.EndDate == nil || *req.EndDate == nil {
			sets = append(sets, "end_date = NULL")
		} else {
			add("end_date", **req.EndDate)
		}
	}
	if req.JobTitle != nil {
		add("job_title", *req.JobTitle)
	}
	if req.EmploymentType != nil {
		add("employment_type", *req.EmploymentType)
	}
	if req.ManagerEmploymentIDPresent {
		if req.ManagerEmploymentID == nil || *req.ManagerEmploymentID == nil {
			sets = append(sets, "manager_employment_id = NULL")
		} else {
			add("manager_employment_id", **req.ManagerEmploymentID)
		}
	}
	if req.TerminationReason != nil {
		add("termination_reason", *req.TerminationReason)
	}
	if req.Notes != nil {
		add("notes", *req.Notes)
	}
	args = append(args, id)
	query := fmt.Sprintf(`UPDATE employments SET %s WHERE id = $%d AND deleted_at IS NULL RETURNING `+employmentColumns,
		strings.Join(sets, ", "), len(args))
	item, err := scanEmployment(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.Employment{}, api.ErrEmploymentNotFound
	}
	return item, err
}

type lockedTerminationTarget struct {
	employerID     uuid.UUID
	relationshipID uuid.UUID
	tenureID       uuid.UUID
	tenureUserID   *uuid.UUID
	tenureEnd      *time.Time
	rehireEligible bool
}

// TerminateEmployment atomically closes the employment segment and its parent relationship.
// If it is the last open relationship in the parent tenure, the tenure is also closed and
// the tenure user's HRM sessions are revoked (fail-closed).
func (r EmploymentRepository) TerminateEmployment(
	ctx context.Context,
	employeeID, employmentID uuid.UUID,
	req api.TerminateEmploymentRequest,
) (api.TerminateEmploymentResult, error) {
	txBeginner, err := r.beginEmploymentTx(ctx)
	if err != nil {
		return api.TerminateEmploymentResult{}, err
	}
	termDate, err := time.Parse("2006-01-02", req.TerminationDate)
	if err != nil {
		return api.TerminateEmploymentResult{}, api.ErrInvalidInput
	}

	var result api.TerminateEmploymentResult
	err = service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
		target, err := lockTerminationTarget(ctx, tx, employeeID, employmentID, req.TerminationDate, termDate)
		if err != nil {
			return err
		}
		updatedSeg, relClosed, err := closeRelationshipAndSegments(ctx, tx, employmentID, target.relationshipID, req)
		if err != nil {
			return err
		}
		tenureClosed, sessionsRevoked, err := maybeCloseParentTenureOnTermination(ctx, tx, r.Revoker, target, req)
		if err != nil {
			return err
		}
		result = api.TerminateEmploymentResult{
			Employment:         updatedSeg,
			RelationshipClosed: relClosed,
			TenureClosed:       tenureClosed,
			SessionsRevoked:    sessionsRevoked,
		}
		return nil
	})
	if err != nil {
		return api.TerminateEmploymentResult{}, err
	}
	return result, nil
}

func lockTerminationTarget(ctx context.Context, tx pgx.Tx, employeeID, employmentID uuid.UUID, termDateStr string, termDate time.Time) (lockedTerminationTarget, error) {
	if err := tx.QueryRow(ctx, `SELECT id FROM employees WHERE id = $1 AND deleted_at IS NULL FOR UPDATE`, employeeID).Scan(new(uuid.UUID)); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return lockedTerminationTarget{}, api.ErrEmployeeNotFound
		}
		return lockedTerminationTarget{}, err
	}
	target, err := lockTerminationSegment(ctx, tx, employeeID, employmentID, termDateStr, termDate)
	if err != nil {
		return lockedTerminationTarget{}, err
	}
	return loadTerminationParents(ctx, tx, target, termDate)
}

func lockTerminationSegment(ctx context.Context, tx pgx.Tx, employeeID, employmentID uuid.UUID, termDateStr string, termDate time.Time) (lockedTerminationTarget, error) {
	var target lockedTerminationTarget
	var segStart time.Time
	var segStatus string
	err := tx.QueryRow(ctx, `
		SELECT employer_id, relationship_id, start_date, status
		FROM employments
		WHERE id = $1 AND employee_id = $2 AND deleted_at IS NULL
		FOR UPDATE
	`, employmentID, employeeID).Scan(&target.employerID, &target.relationshipID, &segStart, &segStatus)
	if errors.Is(err, pgx.ErrNoRows) {
		return lockedTerminationTarget{}, api.ErrEmploymentNotFound
	}
	if err != nil {
		return lockedTerminationTarget{}, err
	}
	if segStatus == "terminated" {
		return lockedTerminationTarget{}, fmt.Errorf("%w: employment is already terminated", api.ErrConflict)
	}
	if termDate.Before(segStart) {
		return lockedTerminationTarget{}, fmt.Errorf("%w: termination_date precedes segment start_date", api.ErrInvalidInput)
	}
	if err := checkLockedPayrollForEmployeeDate(ctx, tx, target.employerID, termDateStr); err != nil {
		return lockedTerminationTarget{}, err
	}
	return target, nil
}

func loadTerminationParents(ctx context.Context, tx pgx.Tx, target lockedTerminationTarget, termDate time.Time) (lockedTerminationTarget, error) {
	var relStart time.Time
	if err := tx.QueryRow(ctx, `
		SELECT tenure_id, start_date
		FROM employment_relationships
		WHERE id = $1 AND deleted_at IS NULL
		FOR UPDATE
	`, target.relationshipID).Scan(&target.tenureID, &relStart); err != nil {
		return lockedTerminationTarget{}, err
	}
	if termDate.Before(relStart) {
		return lockedTerminationTarget{}, fmt.Errorf("%w: termination_date precedes relationship start_date", api.ErrInvalidInput)
	}
	if err := tx.QueryRow(ctx, `
		SELECT user_id, end_date, rehire_eligible
		FROM employee_tenures
		WHERE id = $1 AND deleted_at IS NULL
		FOR UPDATE
	`, target.tenureID).Scan(&target.tenureUserID, &target.tenureEnd, &target.rehireEligible); err != nil {
		return lockedTerminationTarget{}, err
	}
	return target, nil
}

func closeRelationshipAndSegments(ctx context.Context, tx pgx.Tx, employmentID, relationshipID uuid.UUID, req api.TerminateEmploymentRequest) (api.Employment, bool, error) {
	if _, err := tx.Exec(ctx, `
		UPDATE employments
		SET status = 'terminated',
		    end_date = $1::date,
		    termination_reason = COALESCE($2, termination_reason),
		    notes = COALESCE($3, notes),
		    updated_at = NOW()
		WHERE relationship_id = $4
		  AND id <> $5
		  AND deleted_at IS NULL
		  AND end_date IS NULL
	`, req.TerminationDate, req.TerminationReason, req.Notes, relationshipID, employmentID); err != nil {
		return api.Employment{}, false, mapRelationshipConstraintError(err)
	}

	updatedSeg, err := scanEmployment(tx.QueryRow(ctx, `
		UPDATE employments
		SET status = 'terminated',
		    end_date = $1::date,
		    termination_reason = COALESCE($2, termination_reason),
		    notes = COALESCE($3, notes),
		    updated_at = NOW()
		WHERE id = $4 AND deleted_at IS NULL
		RETURNING `+employmentColumns,
		req.TerminationDate, req.TerminationReason, req.Notes, employmentID))
	if err != nil {
		return api.Employment{}, false, mapRelationshipConstraintError(err)
	}

	if err := closeRelationshipPolicyMemberships(ctx, tx, []uuid.UUID{relationshipID}, req.TerminationDate); err != nil {
		return api.Employment{}, false, mapRelationshipConstraintError(err)
	}

	relTag, err := tx.Exec(ctx, `
		UPDATE employment_relationships
		SET status = 'terminated',
		    end_date = $1::date,
		    termination_reason = COALESCE($2, termination_reason),
		    notes = COALESCE($3, notes),
		    updated_at = NOW()
		WHERE id = $4 AND deleted_at IS NULL
	`, req.TerminationDate, req.TerminationReason, req.Notes, relationshipID)
	if err != nil {
		return api.Employment{}, false, mapRelationshipConstraintError(err)
	}
	return updatedSeg, relTag.RowsAffected() > 0, nil
}

func maybeCloseParentTenureOnTermination(ctx context.Context, tx pgx.Tx, revoker SessionRevoker, target lockedTerminationTarget, req api.TerminateEmploymentRequest) (bool, bool, error) {
	var remainingOpen int
	if err := tx.QueryRow(ctx, `
		SELECT count(*)
		FROM employment_relationships
		WHERE tenure_id = $1
		  AND deleted_at IS NULL
		  AND end_date IS NULL
	`, target.tenureID).Scan(&remainingOpen); err != nil {
		return false, false, err
	}
	if remainingOpen > 0 || target.tenureEnd != nil {
		return false, false, nil
	}
	var effectiveTenureEnd string
	if err := tx.QueryRow(ctx, `
		SELECT GREATEST($1::date, COALESCE(MAX(end_date), $1::date))::text
		FROM employment_relationships
		WHERE tenure_id = $2 AND deleted_at IS NULL
	`, req.TerminationDate, target.tenureID).Scan(&effectiveTenureEnd); err != nil {
		return false, false, err
	}
	rehireEligible := target.rehireEligible
	if req.RehireEligible != nil {
		rehireEligible = *req.RehireEligible
	}
	if _, err := tx.Exec(ctx, `
		UPDATE employee_tenures
		SET end_date = $1::date,
		    termination_reason = COALESCE($2, termination_reason),
		    rehire_eligible = $3,
		    notes = COALESCE($4, notes),
		    updated_at = NOW()
		WHERE id = $5
	`, effectiveTenureEnd, req.TerminationReason, rehireEligible, req.Notes, target.tenureID); err != nil {
		return false, false, mapRelationshipConstraintError(err)
	}
	if target.tenureUserID == nil || *target.tenureUserID == uuid.Nil {
		return true, false, nil
	}
	if err := revokeUserSessionsDBTX(ctx, tx, revoker, *target.tenureUserID); err != nil {
		return false, false, err
	}
	return true, true, nil
}

func (r EmploymentRepository) SoftDeleteEmployment(ctx context.Context, id uuid.UUID) error {
	db, err := requireDB(r.DB, "employment repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE employments SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1`, id)
	return err
}
