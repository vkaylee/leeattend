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
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type ContractRepository struct {
	DB      DBTX
	BeginTx func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r ContractRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("contract repository: database is not configured")
	}
	return r.DB, nil
}

func (r ContractRepository) begin(ctx context.Context, wsID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, wsID)
	}
	db, err := r.db(ctx)
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
	if err := row.Scan(&c.ID, &c.WorkspaceID, &c.EmploymentID, &c.ContractNumber, &c.ContractType, &c.Status, &c.StartDate, &c.EndDate, &c.DocumentID, &c.Notes, &created, &updated); err != nil {
		return api.EmployeeContract{}, err
	}
	c.CreatedAt = created.UTC().Format(time.RFC3339)
	c.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return c, nil
}

func (r ContractRepository) ListWorkspaceContracts(ctx context.Context, wsID uuid.UUID) ([]api.EmployeeContractWithEmployee, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		c.id, c.workspace_id, c.employment_id, c.contract_number, c.contract_type::text,
		c.status::text, c.start_date::text, c.end_date::text, c.document_id, c.notes,
		c.created_at, c.updated_at,
		e.full_name, e.employee_id_number, e.id
	FROM employee_contracts c
	INNER JOIN employments em ON em.id = c.employment_id
	INNER JOIN employees e ON e.id = em.employee_id
	WHERE c.workspace_id = $1
	  AND c.deleted_at IS NULL
	  AND em.deleted_at IS NULL
	  AND e.deleted_at IS NULL
	ORDER BY c.start_date DESC`, wsID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.EmployeeContractWithEmployee, 0)
	for rows.Next() {
		var item api.EmployeeContractWithEmployee
		var created, updated time.Time
		if err := rows.Scan(
			&item.ID, &item.WorkspaceID, &item.EmploymentID, &item.ContractNumber, &item.ContractType,
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

func (r ContractRepository) ListEmployeeContracts(ctx context.Context, wsID, employeeID uuid.UUID) ([]api.EmployeeContract, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		c.id, c.workspace_id, c.employment_id, c.contract_number, c.contract_type::text,
		c.status::text, c.start_date::text, c.end_date::text, c.document_id, c.notes,
		c.created_at, c.updated_at
	FROM employee_contracts c
	INNER JOIN employments em ON em.id = c.employment_id
	WHERE c.workspace_id = $1
	  AND em.employee_id = $2
	  AND c.deleted_at IS NULL
	  AND em.deleted_at IS NULL
	ORDER BY em.start_date DESC, c.start_date DESC`, wsID, employeeID)
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

func (r ContractRepository) GetContract(ctx context.Context, wsID, contractID uuid.UUID) (api.EmployeeContract, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeContract{}, err
	}
	item, err := scanContract(db.QueryRow(ctx, `SELECT
		id, workspace_id, employment_id, contract_number, contract_type::text,
		status::text, start_date::text, end_date::text, document_id, notes,
		created_at, updated_at
	FROM employee_contracts
	WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL`, contractID, wsID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeeContract{}, api.ErrContractNotFound
	}
	return item, err
}

func (r ContractRepository) CreateContract(ctx context.Context, wsID uuid.UUID, req api.CreateContractRequest) (api.EmployeeContract, error) {
	txBeginner, err := r.begin(ctx, wsID)
	if err != nil {
		return api.EmployeeContract{}, err
	}
	if txBeginner != nil {
		var created api.EmployeeContract
		err := service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
			res, txErr := r.createContractTx(ctx, tx, wsID, req)
			if txErr != nil {
				return txErr
			}
			created = res
			return nil
		})
		return created, err
	}
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeContract{}, err
	}
	return r.createContractTx(ctx, db, wsID, req)
}

func (r ContractRepository) createContractTx(ctx context.Context, db DBTX, wsID uuid.UUID, req api.CreateContractRequest) (api.EmployeeContract, error) {
	rows, err := db.Query(ctx, `SELECT id FROM employee_contracts
		WHERE workspace_id = $1
		  AND employment_id = $2
		  AND status = 'active'
		  AND start_date <= $3::date
		  AND (end_date IS NULL OR end_date >= $3::date)
		  AND deleted_at IS NULL`, wsID, req.EmploymentID, req.StartDate)
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
		(id, workspace_id, employment_id, contract_number, contract_type, status, start_date, end_date, document_id, notes, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, 'active', $6::date, $7::date, $8, $9, NOW(), NOW())
		RETURNING id, workspace_id, employment_id, contract_number, contract_type::text, status::text, start_date::text, end_date::text, document_id, notes, created_at, updated_at`,
		contractID, wsID, req.EmploymentID, req.ContractNumber, req.ContractType, req.StartDate, req.EndDate, req.DocumentID, req.Notes))
	if err != nil {
		return api.EmployeeContract{}, mapConstraintError(err)
	}
	return item, nil
}

func (r ContractRepository) UpdateContract(ctx context.Context, wsID, contractID uuid.UUID, req api.UpdateContractRequest) (api.EmployeeContract, error) {
	db, err := r.db(ctx)
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
	args = append(args, contractID, wsID)
	query := fmt.Sprintf(`UPDATE employee_contracts SET %s WHERE id = $%d AND workspace_id = $%d AND deleted_at IS NULL
		RETURNING id, workspace_id, employment_id, contract_number, contract_type::text, status::text, start_date::text, end_date::text, document_id, notes, created_at, updated_at`,
		strings.Join(sets, ", "), len(args)-1, len(args))
	item, err := scanContract(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeeContract{}, api.ErrContractNotFound
	}
	return item, err
}

func (r ContractRepository) SoftDeleteContract(ctx context.Context, wsID, contractID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE employee_contracts SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND workspace_id = $2`, contractID, wsID)
	return err
}

type EmploymentRepository struct {
	DB DBTX
}

func (r EmploymentRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("employment repository: database is not configured")
	}
	return r.DB, nil
}

func scanEmployment(row interface{ Scan(...any) error }) (api.Employment, error) {
	var e api.Employment
	var created, updated time.Time
	if err := row.Scan(
		&e.ID, &e.WorkspaceID, &e.EmployeeID, &e.EmployerID, &e.PrimaryLocationID, &e.DepartmentID,
		&e.Status, &e.StartDate, &e.EndDate, &e.JobTitle, &e.EmploymentType, &e.ManagerEmploymentID,
		&created, &updated, &e.TerminationReason, &e.Notes,
	); err != nil {
		return api.Employment{}, err
	}
	e.CreatedAt = created.UTC().Format(time.RFC3339)
	e.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return e, nil
}

const employmentColumns = `id, workspace_id, employee_id, employer_id, primary_location_id, department_id,
	status, start_date::text, end_date::text, job_title, employment_type, manager_employment_id,
	created_at, updated_at, termination_reason, notes`

func (r EmploymentRepository) ListEmployments(ctx context.Context, wsID, employeeID uuid.UUID) ([]api.Employment, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+employmentColumns+` FROM employments
		WHERE workspace_id = $1 AND employee_id = $2 AND deleted_at IS NULL
		ORDER BY start_date DESC`, wsID, employeeID)
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

func (r EmploymentRepository) CreateEmployment(ctx context.Context, wsID uuid.UUID, req api.CreateEmploymentRequest) (api.Employment, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.Employment{}, err
	}
	empID := uuid.New()
	item, err := scanEmployment(db.QueryRow(ctx, `INSERT INTO employments
		(id, workspace_id, employee_id, employer_id, primary_location_id, department_id, status, start_date, end_date, job_title, employment_type, manager_employment_id, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8::date, $9::date, $10, $11, $12, NOW(), NOW())
		RETURNING `+employmentColumns,
		empID, wsID, req.EmployeeID, req.EmployerID, req.PrimaryLocationID, req.DepartmentID, req.Status, req.StartDate, req.EndDate, req.JobTitle, req.EmploymentType, req.ManagerEmploymentID))
	if err != nil {
		return api.Employment{}, mapConstraintError(err)
	}
	return item, nil
}

func (r EmploymentRepository) GetEmployment(ctx context.Context, wsID, id uuid.UUID) (api.Employment, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.Employment{}, err
	}
	item, err := scanEmployment(db.QueryRow(ctx, `SELECT `+employmentColumns+` FROM employments
		WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL`, id, wsID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.Employment{}, api.ErrEmploymentNotFound
	}
	return item, err
}

func (r EmploymentRepository) UpdateEmployment(ctx context.Context, wsID, id uuid.UUID, req api.UpdateEmploymentRequest) (api.Employment, error) {
	db, err := r.db(ctx)
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
	args = append(args, id, wsID)
	query := fmt.Sprintf(`UPDATE employments SET %s WHERE id = $%d AND workspace_id = $%d AND deleted_at IS NULL RETURNING `+employmentColumns,
		strings.Join(sets, ", "), len(args)-1, len(args))
	item, err := scanEmployment(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.Employment{}, api.ErrEmploymentNotFound
	}
	return item, err
}

func (r EmploymentRepository) SoftDeleteEmployment(ctx context.Context, wsID, id uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE employments SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND workspace_id = $2`, id, wsID)
	return err
}
