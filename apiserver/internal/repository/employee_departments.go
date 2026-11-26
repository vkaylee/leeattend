package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type EmployeeDepartmentRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r EmployeeDepartmentRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "employee department repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("employee department repository: database does not support transactions")
	}
	return beginner, nil
}

func (r EmployeeDepartmentRepository) CurrentEmployeeDepartment(ctx context.Context, employeeID uuid.UUID) (*uuid.UUID, error) {
	db, err := requireDB(r.DB, "employee department repository")
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

func (r EmployeeDepartmentRepository) GetEmployeeDepartmentHistory(ctx context.Context, employeeID uuid.UUID) ([]api.EmployeeDepartmentHistory, error) {
	db, err := requireDB(r.DB, "employee department repository")
	if err != nil {
		return nil, err
	}
	if err := employeeExists(ctx, db, employeeID); err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT em.id, em.employee_id, em.department_id, em.start_date::text, em.notes, em.created_at FROM employments em WHERE em.employee_id = $1 AND em.deleted_at IS NULL AND em.department_id IS NOT NULL ORDER BY em.start_date DESC`, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.EmployeeDepartmentHistory, 0)
	for rows.Next() {
		var item api.EmployeeDepartmentHistory
		var createdAt time.Time
		if err := rows.Scan(&item.ID, &item.EmployeeID, &item.DepartmentID, &item.EffectiveDate, &item.Notes, &createdAt); err != nil {
			return nil, err
		}
		item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		result = append(result, item)
	}
	return result, rows.Err()
}

type employmentSegment struct {
	ID                  uuid.UUID
	EmployeeID          uuid.UUID
	EmployerID          uuid.UUID
	RelationshipID      *uuid.UUID
	PrimaryLocationID   *uuid.UUID
	DepartmentID        *uuid.UUID
	Status              string
	StartDate           string
	EndDate             *string
	JobTitle            *string
	EmploymentType      string
	ManagerEmploymentID *uuid.UUID
	Notes               *string
	TerminationReason   *string
}

// ponytail: in-memory contiguous department chain walk. Simple and fast for small segment counts (<20).
func (r EmployeeDepartmentRepository) TransferEmployee(ctx context.Context, employeeID uuid.UUID, req api.TransferEmployeeRequest) (api.EmployeeDepartmentHistory, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.EmployeeDepartmentHistory{}, err
	}
	var history api.EmployeeDepartmentHistory
	err = service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := employeeExists(ctx, tx, employeeID); err != nil {
			return err
		}
		if err := tx.QueryRow(ctx, `SELECT id FROM employees WHERE id = $1 AND deleted_at IS NULL FOR UPDATE`, employeeID).Scan(new(uuid.UUID)); err != nil {
			return err
		}
		var deptExists bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM departments WHERE id = $1 AND deleted_at IS NULL)`, req.DepartmentID).Scan(&deptExists); err != nil {
			return err
		}
		if !deptExists {
			return api.ErrDepartmentNotFound
		}

		rows, err := tx.Query(ctx, `SELECT id, employee_id, employer_id, relationship_id, primary_location_id, department_id, status, start_date::text, end_date::text, job_title, employment_type, manager_employment_id, notes, termination_reason FROM employments WHERE employee_id = $1 AND deleted_at IS NULL ORDER BY start_date ASC, id ASC`, employeeID)
		if err != nil {
			return err
		}
		defer rows.Close()

		segments := make([]employmentSegment, 0)
		for rows.Next() {
			var s employmentSegment
			if err := rows.Scan(&s.ID, &s.EmployeeID, &s.EmployerID, &s.RelationshipID, &s.PrimaryLocationID, &s.DepartmentID, &s.Status, &s.StartDate, &s.EndDate, &s.JobTitle, &s.EmploymentType, &s.ManagerEmploymentID, &s.Notes, &s.TerminationReason); err != nil {
				return err
			}
			segments = append(segments, s)
		}
		if err := rows.Err(); err != nil {
			return err
		}

		activeIdx := -1
		for i, s := range segments {
			if s.StartDate <= req.EffectiveDate && (s.EndDate == nil || *s.EndDate >= req.EffectiveDate) {
				activeIdx = i
				break
			}
		}
		if activeIdx == -1 {
			return api.ErrNoActiveEmployment
		}

		active := segments[activeIdx]
		if err := checkLockedPayrollForEmployeeDate(ctx, tx, active.EmployerID, req.EffectiveDate); err != nil {
			return err
		}
		oldDeptID := active.DepartmentID
		var returnedID uuid.UUID

		if active.StartDate == req.EffectiveDate {
			_, err := tx.Exec(ctx, `UPDATE employments SET department_id = $1, notes = $2, updated_at = NOW() WHERE id = $3`, req.DepartmentID, req.Notes, active.ID)
			if err != nil {
				return err
			}
			returnedID = active.ID
		} else {
			effDate, err := time.Parse("2006-01-02", req.EffectiveDate)
			if err != nil {
				return err
			}
			prevEndDate := effDate.AddDate(0, 0, -1).Format("2006-01-02")
			if _, err := tx.Exec(ctx, `UPDATE employments SET end_date = $1::date, updated_at = NOW() WHERE id = $2`, prevEndDate, active.ID); err != nil {
				return err
			}
			newID := uuid.New()
			var endDateVal *string = active.EndDate
			_, err = tx.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, relationship_id, primary_location_id, department_id, status, start_date, end_date, created_at, updated_at, deleted_at, job_title, employment_type, manager_employment_id, notes, termination_reason) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::date, NULLIF($9, '')::date, NOW(), NOW(), NULL, $10, $11, $12, $13, $14)`, newID, active.EmployeeID, active.EmployerID, active.RelationshipID, active.PrimaryLocationID, req.DepartmentID, active.Status, req.EffectiveDate, func() string {
				if endDateVal != nil {
					return *endDateVal
				}
				return ""
			}(), active.JobTitle, active.EmploymentType, active.ManagerEmploymentID, req.Notes, active.TerminationReason)
			if err != nil {
				return err
			}
			returnedID = newID
		}

		for _, seg := range segments[activeIdx+1:] {
			matchesOld := (seg.DepartmentID == nil && oldDeptID == nil) || (seg.DepartmentID != nil && oldDeptID != nil && *seg.DepartmentID == *oldDeptID)
			if !matchesOld {
				break
			}
			if _, err := tx.Exec(ctx, `UPDATE employments SET department_id = $1, updated_at = NOW() WHERE id = $2`, req.DepartmentID, seg.ID); err != nil {
				return err
			}
		}

		history = api.EmployeeDepartmentHistory{
			ID:            returnedID,
			EmployeeID:    employeeID,
			DepartmentID:  req.DepartmentID,
			EffectiveDate: req.EffectiveDate,
			Notes:         req.Notes,
			CreatedAt:     time.Now().UTC().Format(time.RFC3339),
		}
		return nil
	})
	if err != nil {
		return api.EmployeeDepartmentHistory{}, err
	}
	return history, nil
}
