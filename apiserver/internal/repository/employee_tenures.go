package repository

import (
	"context"
	"errors"
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

func mapEmployeeWriteError(err error) error {
	if err == nil {
		return nil
	}
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) {
		switch pgErr.Code {
		case "23505", "23P01":
			return fmt.Errorf("%w: %s", api.ErrConflict, pgErr.ConstraintName)
		case "23503", "23514", "22P02":
			return fmt.Errorf("%w: %s", api.ErrInvalidInput, pgErr.ConstraintName)
		}
	}
	return err
}

func lockEmployeeAdvisoryKeys(ctx context.Context, tx pgx.Tx, keys []string) error {
	if len(keys) == 0 {
		return nil
	}
	sorted := make([]string, len(keys))
	copy(sorted, keys)
	sort.Strings(sorted)

	prev := ""
	for _, key := range sorted {
		if key == "" || key == prev {
			continue
		}
		prev = key
		if _, err := tx.Exec(ctx, `SELECT pg_advisory_xact_lock(hashtextextended($1, 0))`, key); err != nil {
			return fmt.Errorf("acquire employee advisory lock: %w", err)
		}
	}
	return nil
}

func checkNormalizedCodeOwnership(ctx context.Context, tx pgx.Tx, normalizedCode string, allowedEmployeeID uuid.UUID) error {
	var conflict bool
	err := tx.QueryRow(ctx, `
		SELECT EXISTS (
			SELECT 1 FROM employees
			WHERE deleted_at IS NULL
			  AND id <> $2
			  AND UPPER(BTRIM(employee_id_number)) = $1
			UNION ALL
			SELECT 1 FROM employee_tenures
			WHERE deleted_at IS NULL
			  AND employee_id <> $2
			  AND UPPER(BTRIM(employee_number)) = $1
		)
	`, normalizedCode, allowedEmployeeID).Scan(&conflict)
	if err != nil {
		return fmt.Errorf("check normalized employee code uniqueness: %w", err)
	}
	if conflict {
		return fmt.Errorf("%w: duplicate employee code", api.ErrConflict)
	}
	return nil
}

func checkIdentityHashUniqueness(ctx context.Context, tx pgx.Tx, nidHash, phoneHash *string) error {
	if nidHash != nil && *nidHash != "" {
		var exists bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employee_profiles WHERE national_id_lookup_hash = $1)`, *nidHash).Scan(&exists); err != nil {
			return fmt.Errorf("check national id hash uniqueness: %w", err)
		}
		if exists {
			return fmt.Errorf("%w: duplicate national_id", api.ErrConflict)
		}
	}
	if phoneHash != nil && *phoneHash != "" {
		var exists bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employee_profiles WHERE phone_lookup_hash = $1)`, *phoneHash).Scan(&exists); err != nil {
			return fmt.Errorf("check phone hash uniqueness: %w", err)
		}
		if exists {
			return fmt.Errorf("%w: duplicate phone", api.ErrConflict)
		}
	}
	return nil
}

func validateEmploymentScopeRefs(ctx context.Context, tx pgx.Tx, employerID, departmentID, locationID *uuid.UUID) error {
	if employerID != nil {
		if err := requireScopeExists(ctx, tx, `SELECT EXISTS (SELECT 1 FROM employers WHERE id = $1 AND deleted_at IS NULL)`, *employerID); err != nil {
			return err
		}
	}
	if departmentID != nil {
		if err := requireScopeExists(ctx, tx, `SELECT EXISTS (SELECT 1 FROM departments WHERE id = $1 AND deleted_at IS NULL)`, *departmentID); err != nil {
			return err
		}
	}
	if locationID != nil {
		if err := requireScopeExists(ctx, tx, `SELECT EXISTS (SELECT 1 FROM work_locations WHERE id = $1 AND status = 'active')`, *locationID); err != nil {
			return err
		}
	}
	return nil
}

func requireScopeExists(ctx context.Context, tx pgx.Tx, sqlText string, id uuid.UUID) error {
	var exists bool
	if err := tx.QueryRow(ctx, sqlText, id).Scan(&exists); err != nil {
		return err
	}
	if !exists {
		return api.ErrInvalidInput
	}
	return nil
}

type lockedEmployeeState struct {
	id              uuid.UUID
	employeeCode    string
	isActive        bool
	terminationDate *time.Time
	deletedAt       *time.Time
}

// CreateEmployeeTenure locks and revalidates an existing terminated employee, preserves employee ID,
// identity, and canonical code, and inserts a new non-overlapping tenure, relationship, and employment atomically.
func (r EmployeeRepository) CreateEmployeeTenure(
	ctx context.Context,
	employeeID uuid.UUID,
	req api.CreateEmployeeTenureRequest,
) (api.EmployeeTenureCreated, error) {
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.EmployeeTenureCreated{}, err
	}

	var created api.EmployeeTenureCreated
	err = service.WithTx(ctx, beginner, func(txCtx context.Context, tx pgx.Tx) error {
		state, lockErr := lockAndLoadEmployeeForTenure(txCtx, tx, employeeID)
		if lockErr != nil {
			return lockErr
		}
		if err := ensureEmployeeEligibleForReuse(txCtx, tx, state, req.StartDate); err != nil {
			return err
		}
		canonicalCode, err := resolveAndVerifyReuseCode(txCtx, tx, state, req.ExpectedEmployeeCode)
		if err != nil {
			return err
		}
		if err := validateEmploymentScopeRefs(txCtx, tx, &req.EmployerID, req.DepartmentID, req.PrimaryLocationID); err != nil {
			return err
		}
		created, err = insertReusedTenureAndEmployment(txCtx, tx, employeeID, canonicalCode, req)
		return err
	})
	if err != nil {
		return api.EmployeeTenureCreated{}, err
	}
	return created, nil
}

func lockAndLoadEmployeeForTenure(ctx context.Context, tx pgx.Tx, employeeID uuid.UUID) (lockedEmployeeState, error) {
	var state lockedEmployeeState
	err := tx.QueryRow(ctx, `
		SELECT id, employee_id_number, is_active, termination_date, deleted_at
		FROM employees
		WHERE id = $1
		FOR UPDATE
	`, employeeID).Scan(&state.id, &state.employeeCode, &state.isActive, &state.terminationDate, &state.deletedAt)
	if err != nil {
		return lockedEmployeeState{}, err
	}
	if state.deletedAt != nil {
		return lockedEmployeeState{}, fmt.Errorf("%w: deleted employee cannot be reused", api.ErrConflict)
	}
	return state, nil
}

func ensureEmployeeEligibleForReuse(ctx context.Context, tx pgx.Tx, state lockedEmployeeState, startDate string) error {
	startParsed, err := time.Parse("2006-01-02", startDate)
	if err != nil {
		return api.ErrInvalidInput
	}
	if state.terminationDate != nil && !startParsed.After(*state.terminationDate) {
		return fmt.Errorf("%w: start_date overlaps termination_date", api.ErrConflict)
	}

	var totalTenures int
	var hasBlockingTenure bool
	if err := tx.QueryRow(ctx, `
		SELECT
			count(*),
			COALESCE(bool_or(end_date IS NULL OR end_date >= $2::date OR start_date >= $2::date), FALSE)
		FROM employee_tenures
		WHERE employee_id = $1 AND deleted_at IS NULL
	`, state.id, startDate).Scan(&totalTenures, &hasBlockingTenure); err != nil {
		return fmt.Errorf("check employee tenure overlap: %w", err)
	}
	if hasBlockingTenure {
		return fmt.Errorf("%w: employee has open or overlapping tenure", api.ErrConflict)
	}

	if err := checkBlockingRelationshipsOrEmployments(ctx, tx, state.id, startDate); err != nil {
		return err
	}
	if state.isActive && state.terminationDate == nil && totalTenures == 0 {
		return fmt.Errorf("%w: active employee cannot be reused", api.ErrConflict)
	}
	return nil
}

func checkBlockingRelationshipsOrEmployments(ctx context.Context, tx pgx.Tx, employeeID uuid.UUID, startDate string) error {
	var blocked bool
	err := tx.QueryRow(ctx, `
		SELECT EXISTS (
			SELECT 1 FROM employment_relationships
			WHERE employee_id = $1
			  AND deleted_at IS NULL
			  AND (end_date IS NULL OR end_date >= $2::date OR start_date >= $2::date)
			UNION ALL
			SELECT 1 FROM employments
			WHERE employee_id = $1
			  AND deleted_at IS NULL
			  AND status = 'active'
			  AND (end_date IS NULL OR end_date >= $2::date)
		)
	`, employeeID, startDate).Scan(&blocked)
	if err != nil {
		return fmt.Errorf("check active employment state: %w", err)
	}
	if blocked {
		return fmt.Errorf("%w: employee has active or overlapping employment", api.ErrConflict)
	}
	return nil
}

func resolveAndVerifyReuseCode(
	ctx context.Context,
	tx pgx.Tx,
	state lockedEmployeeState,
	expectedCode *string,
) (string, error) {
	canonicalCode := strings.TrimSpace(state.employeeCode)
	var latestTenureCode string
	err := tx.QueryRow(ctx, `
		SELECT employee_number
		FROM employee_tenures
		WHERE employee_id = $1 AND deleted_at IS NULL
		ORDER BY start_date DESC, id DESC
		LIMIT 1
	`, state.id).Scan(&latestTenureCode)
	if err == nil && strings.TrimSpace(latestTenureCode) != "" {
		canonicalCode = strings.TrimSpace(latestTenureCode)
	} else if err != nil && !errors.Is(err, pgx.ErrNoRows) {
		return "", fmt.Errorf("resolve canonical employee code: %w", err)
	}

	normalizedCode := identity.NormalizeEmployeeCode(canonicalCode)
	if normalizedCode == "" {
		return "", fmt.Errorf("%w: missing canonical employee code", api.ErrConflict)
	}
	if expectedCode != nil && identity.NormalizeEmployeeCode(*expectedCode) != normalizedCode {
		return "", fmt.Errorf("%w: stale candidate employee code", api.ErrConflict)
	}
	if err := lockEmployeeAdvisoryKeys(ctx, tx, []string{"emp_code:" + normalizedCode}); err != nil {
		return "", err
	}
	if err := checkNormalizedCodeOwnership(ctx, tx, normalizedCode, state.id); err != nil {
		return "", err
	}
	return canonicalCode, nil
}

func insertReusedTenureAndEmployment(
	ctx context.Context,
	tx pgx.Tx,
	employeeID uuid.UUID,
	canonicalCode string,
	req api.CreateEmployeeTenureRequest,
) (api.EmployeeTenureCreated, error) {
	var tenureID uuid.UUID
	if err := tx.QueryRow(ctx, `
		INSERT INTO employee_tenures (employee_id, user_id, employee_number, start_date)
		VALUES ($1, NULL, $2, $3::date)
		RETURNING id
	`, employeeID, canonicalCode, req.StartDate).Scan(&tenureID); err != nil {
		return api.EmployeeTenureCreated{}, mapEmployeeWriteError(err)
	}

	relationshipID := uuid.New()
	if err := tx.QueryRow(ctx, `
		INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date, created_at, updated_at)
		VALUES ($1, $2, $3, $4, 'active', $5::date, NOW(), NOW())
		RETURNING id
	`, relationshipID, employeeID, req.EmployerID, tenureID, req.StartDate).Scan(&relationshipID); err != nil {
		return api.EmployeeTenureCreated{}, mapEmployeeWriteError(err)
	}

	employmentID := uuid.New()
	if err := tx.QueryRow(ctx, `
		INSERT INTO employments (id, employee_id, employer_id, relationship_id, primary_location_id, department_id, status, start_date, employment_type, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, 'active', $7::date, 'full_time', NOW(), NOW())
		RETURNING id
	`, employmentID, employeeID, req.EmployerID, relationshipID, req.PrimaryLocationID, req.DepartmentID, req.StartDate).Scan(&employmentID); err != nil {
		return api.EmployeeTenureCreated{}, mapEmployeeWriteError(err)
	}

	// Reactivate employee status without altering employee ID, original_hire_date, full_name, code, or profile.
	if _, err := tx.Exec(ctx, `
		UPDATE employees
		SET is_active = TRUE,
		    termination_date = NULL,
		    user_id = NULL,
		    hire_date = $2::date,
		    updated_at = CURRENT_TIMESTAMP
		WHERE id = $1
	`, employeeID, req.StartDate); err != nil {
		return api.EmployeeTenureCreated{}, mapEmployeeWriteError(err)
	}

	return api.EmployeeTenureCreated{
		TenureID:       tenureID,
		EmployeeID:     employeeID,
		EmployeeCode:   canonicalCode,
		StartDate:      req.StartDate,
		RelationshipID: relationshipID,
		EmploymentID:   employmentID,
		EmployerID:     req.EmployerID,
		Status:         api.MatchStatusActive,
	}, nil
}
