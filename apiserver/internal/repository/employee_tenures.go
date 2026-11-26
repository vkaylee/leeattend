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

// checkNormalizedCodeOwnership enforces permanent per-person code ownership across all tenures (including closed/soft-deleted).
func checkNormalizedCodeOwnership(ctx context.Context, tx pgx.Tx, normalizedCode string, allowedEmployeeID uuid.UUID) error {
	var conflict bool
	err := tx.QueryRow(ctx, `
		SELECT EXISTS (
			SELECT 1
			FROM employee_tenures
			WHERE employee_id <> $2
			  AND employee_number_normalized = $1
		)
	`, normalizedCode, allowedEmployeeID).Scan(&conflict)
	if err != nil {
		return fmt.Errorf("check normalized employee code ownership: %w", err)
	}
	if conflict {
		return fmt.Errorf("%w: employee code permanently owned by another employee", api.ErrConflict)
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

type lockedEmployeeTenureState struct {
	id             uuid.UUID
	deletedAt      *time.Time
	latestCode     string
	rehireEligible bool
	hasPriorTenure bool
}

// CreateEmployeeTenure locks and revalidates an existing terminated employee, preserves person ID
// and identity, and atomically creates a new non-overlapping rehire tenure, relationship, and segment.
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
		if err := ensureEmployeeEligibleForRehire(txCtx, tx, state, req.StartDate); err != nil {
			return err
		}
		code, err := resolveAndVerifyRehireCode(txCtx, tx, state, req.EmployeeNumber, req.ExpectedEmployeeCode)
		if err != nil {
			return err
		}
		if err := validateEmploymentScopeRefs(txCtx, tx, &req.EmployerID, req.DepartmentID, req.PrimaryLocationID); err != nil {
			return err
		}
		if err := checkLockedPayrollForEmployeeDate(txCtx, tx, req.EmployerID, req.StartDate); err != nil {
			return err
		}
		created, err = insertRehireTenureAndEmployment(txCtx, tx, employeeID, code, req)
		return err
	})
	if err != nil {
		return api.EmployeeTenureCreated{}, err
	}
	return created, nil
}

func lockAndLoadEmployeeForTenure(ctx context.Context, tx pgx.Tx, employeeID uuid.UUID) (lockedEmployeeTenureState, error) {
	var state lockedEmployeeTenureState
	state.rehireEligible = true
	if err := tx.QueryRow(ctx, `SELECT id, deleted_at FROM employees WHERE id = $1 FOR UPDATE`, employeeID).
		Scan(&state.id, &state.deletedAt); err != nil {
		return lockedEmployeeTenureState{}, err
	}
	if state.deletedAt != nil {
		return lockedEmployeeTenureState{}, fmt.Errorf("%w: deleted employee cannot be rehired", api.ErrConflict)
	}
	err := tx.QueryRow(ctx, `
		SELECT employee_number, rehire_eligible
		FROM employee_tenures
		WHERE employee_id = $1 AND deleted_at IS NULL
		ORDER BY start_date DESC, id DESC
		LIMIT 1
	`, employeeID).Scan(&state.latestCode, &state.rehireEligible)
	if err == nil {
		state.hasPriorTenure = true
	} else if !errors.Is(err, pgx.ErrNoRows) {
		return lockedEmployeeTenureState{}, fmt.Errorf("load latest employee tenure: %w", err)
	}
	return state, nil
}

func ensureEmployeeEligibleForRehire(ctx context.Context, tx pgx.Tx, state lockedEmployeeTenureState, startDate string) error {
	if _, err := time.Parse("2006-01-02", startDate); err != nil {
		return api.ErrInvalidInput
	}
	if state.hasPriorTenure && !state.rehireEligible {
		return fmt.Errorf("%w: employee is marked not eligible for rehire", api.ErrConflict)
	}

	var hasBlockingTenure bool
	if err := tx.QueryRow(ctx, `
		SELECT COALESCE(bool_or(end_date IS NULL OR end_date >= $2::date OR start_date >= $2::date), FALSE)
		FROM employee_tenures
		WHERE employee_id = $1 AND deleted_at IS NULL
	`, state.id, startDate).Scan(&hasBlockingTenure); err != nil {
		return fmt.Errorf("check employee tenure overlap: %w", err)
	}
	if hasBlockingTenure {
		return fmt.Errorf("%w: employee has open or overlapping tenure", api.ErrConflict)
	}

	return checkBlockingRelationshipsOrEmployments(ctx, tx, state.id, startDate)
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
			SELECT 1 FROM employment_segments
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

func resolveAndVerifyRehireCode(
	ctx context.Context,
	tx pgx.Tx,
	state lockedEmployeeTenureState,
	requestedCode *string,
	expectedCode *string,
) (string, error) {
	previousCode := strings.TrimSpace(state.latestCode)
	if expectedCode != nil && previousCode != "" && identity.NormalizeEmployeeCode(*expectedCode) != identity.NormalizeEmployeeCode(previousCode) {
		return "", fmt.Errorf("%w: stale candidate employee code", api.ErrConflict)
	}

	targetCode := previousCode
	if requestedCode != nil && strings.TrimSpace(*requestedCode) != "" {
		targetCode = strings.TrimSpace(*requestedCode)
	}
	normalizedCode := identity.NormalizeEmployeeCode(targetCode)
	if normalizedCode == "" {
		return "", fmt.Errorf("%w: employee number is required", api.ErrConflict)
	}
	if err := lockEmployeeAdvisoryKeys(ctx, tx, []string{"emp_code:" + normalizedCode}); err != nil {
		return "", err
	}
	if err := checkNormalizedCodeOwnership(ctx, tx, normalizedCode, state.id); err != nil {
		return "", err
	}
	return targetCode, nil
}

func insertRehireTenureAndEmployment(
	ctx context.Context,
	tx pgx.Tx,
	employeeID uuid.UUID,
	employeeNumber string,
	req api.CreateEmployeeTenureRequest,
) (api.EmployeeTenureCreated, error) {
	priorCredit := 0
	if req.PriorServiceCreditDays != nil {
		priorCredit = *req.PriorServiceCreditDays
	}

	var tenureID uuid.UUID
	var status string
	if err := tx.QueryRow(ctx, `
		INSERT INTO employee_tenures (employee_id, user_id, employee_number, hire_type, prior_service_credit_days, start_date, notes)
		VALUES ($1, NULL, $2, 'rehire', $3, $4::date, $5)
		RETURNING id, CASE WHEN start_date > `+orgTodaySQL+` THEN 'planned' ELSE 'active' END
	`, employeeID, employeeNumber, priorCredit, req.StartDate, req.Notes).Scan(&tenureID, &status); err != nil {
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
		INSERT INTO employment_segments (id, employee_id, employer_id, relationship_id, primary_location_id, department_id, status, start_date, employment_type, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, 'active', $7::date, 'full_time', NOW(), NOW())
		RETURNING id
	`, employmentID, employeeID, req.EmployerID, relationshipID, req.PrimaryLocationID, req.DepartmentID, req.StartDate).Scan(&employmentID); err != nil {
		return api.EmployeeTenureCreated{}, mapEmployeeWriteError(err)
	}

	return api.EmployeeTenureCreated{
		TenureID:               tenureID,
		EmployeeID:             employeeID,
		EmployeeCode:           employeeNumber,
		HireType:               "rehire",
		PriorServiceCreditDays: priorCredit,
		StartDate:              req.StartDate,
		RelationshipID:         relationshipID,
		EmploymentID:           employmentID,
		EmployerID:             req.EmployerID,
		Status:                 status,
	}, nil
}

// ListEmployeeTenures returns all non-deleted tenures for an employee newest-first, with employer relationships.
func (r EmployeeRepository) ListEmployeeTenures(ctx context.Context, employeeID uuid.UUID) ([]api.EmployeeTenureRecord, error) {
	db, err := requireDB(r.DB, "employee repository")
	if err != nil {
		return nil, err
	}
	if err := requireEmployeeExistsForTenures(ctx, db, employeeID); err != nil {
		return nil, err
	}
	tenures, indexByID, err := loadEmployeeTenureRows(ctx, db, employeeID)
	if err != nil || len(tenures) == 0 {
		return tenures, err
	}
	if err := attachTenureRelationships(ctx, db, employeeID, tenures, indexByID); err != nil {
		return nil, err
	}
	return tenures, nil
}

func requireEmployeeExistsForTenures(ctx context.Context, db DBTX, employeeID uuid.UUID) error {
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&exists); err != nil {
		return err
	}
	if !exists {
		return errNoRows
	}
	return nil
}

func loadEmployeeTenureRows(ctx context.Context, db DBTX, employeeID uuid.UUID) ([]api.EmployeeTenureRecord, map[uuid.UUID]int, error) {
	rows, err := db.Query(ctx, `
		SELECT
			t.id, t.employee_id, t.employee_number, t.user_id, t.hire_type,
			t.prior_service_credit_days,
			CASE
				WHEN t.start_date > `+orgTodaySQL+` AND t.end_date IS NULL THEN 'planned'
				WHEN t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+` THEN 'active'
				ELSE 'closed'
			END AS status,
			t.start_date::text, t.end_date::text, t.termination_reason, t.rehire_eligible,
			t.notes, t.created_at, t.updated_at
		FROM employee_tenures t
		WHERE t.employee_id = $1 AND t.deleted_at IS NULL
		ORDER BY t.start_date DESC, t.id DESC
	`, employeeID)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()

	tenures := make([]api.EmployeeTenureRecord, 0)
	indexByID := make(map[uuid.UUID]int)
	for rows.Next() {
		var rec api.EmployeeTenureRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(
			&rec.ID, &rec.EmployeeID, &rec.EmployeeNumber, &rec.UserID, &rec.HireType,
			&rec.PriorServiceCreditDays, &rec.Status, &rec.StartDate, &rec.EndDate,
			&rec.TerminationReason, &rec.RehireEligible, &rec.Notes, &createdAt, &updatedAt,
		); err != nil {
			return nil, nil, err
		}
		rec.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		rec.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		rec.Relationships = make([]api.EmployeeTenureRelationshipSummary, 0)
		indexByID[rec.ID] = len(tenures)
		tenures = append(tenures, rec)
	}
	return tenures, indexByID, rows.Err()
}

func attachTenureRelationships(ctx context.Context, db DBTX, employeeID uuid.UUID, tenures []api.EmployeeTenureRecord, indexByID map[uuid.UUID]int) error {
	relRows, err := db.Query(ctx, `
		SELECT rel.tenure_id, rel.id, rel.employer_id, COALESCE(emp.name, ''), rel.status,
		       rel.start_date::text, rel.end_date::text, rel.termination_reason
		FROM employment_relationships rel
		LEFT JOIN employers emp ON emp.id = rel.employer_id
		WHERE rel.employee_id = $1 AND rel.deleted_at IS NULL
		ORDER BY rel.start_date DESC, rel.id DESC
	`, employeeID)
	if err != nil {
		return err
	}
	defer relRows.Close()

	for relRows.Next() {
		var tenureID uuid.UUID
		var rel api.EmployeeTenureRelationshipSummary
		if err := relRows.Scan(
			&tenureID, &rel.ID, &rel.EmployerID, &rel.EmployerName,
			&rel.Status, &rel.StartDate, &rel.EndDate, &rel.TerminationReason,
		); err != nil {
			return err
		}
		if idx, ok := indexByID[tenureID]; ok {
			tenures[idx].Relationships = append(tenures[idx].Relationships, rel)
		}
	}
	return relRows.Err()
}

type lockedTenureCloseState struct {
	startDate      time.Time
	userID         *uuid.UUID
	rehireEligible bool
}

// CloseEmployeeTenure atomically closes all open relationships/segments of a tenure,
// sets end_date/termination metadata on the tenure, and revokes the tenure user's HRM sessions (fail-closed).
func (r EmployeeRepository) CloseEmployeeTenure(
	ctx context.Context,
	employeeID, tenureID uuid.UUID,
	req api.CloseEmployeeTenureRequest,
) (api.EmployeeTenureClosed, error) {
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.EmployeeTenureClosed{}, err
	}
	endParsed, err := time.Parse("2006-01-02", req.EndDate)
	if err != nil {
		return api.EmployeeTenureClosed{}, api.ErrInvalidInput
	}

	var closed api.EmployeeTenureClosed
	err = service.WithTx(ctx, beginner, func(txCtx context.Context, tx pgx.Tx) error {
		var txErr error
		closed, txErr = closeEmployeeTenureTx(txCtx, tx, r.Revoker, employeeID, tenureID, req, endParsed)
		return txErr
	})
	if err != nil {
		return api.EmployeeTenureClosed{}, err
	}
	return closed, nil
}

func closeEmployeeTenureTx(
	ctx context.Context,
	tx pgx.Tx,
	revoker SessionRevoker,
	employeeID, tenureID uuid.UUID,
	req api.CloseEmployeeTenureRequest,
	endParsed time.Time,
) (api.EmployeeTenureClosed, error) {
	state, err := lockOpenTenureForClose(ctx, tx, employeeID, tenureID, endParsed)
	if err != nil {
		return api.EmployeeTenureClosed{}, err
	}
	openRelIDs, err := loadAndValidateOpenTenureRelationships(ctx, tx, tenureID, req.EndDate, endParsed)
	if err != nil {
		return api.EmployeeTenureClosed{}, err
	}
	closedRels, closedSegments, err := closeOpenTenureChildren(ctx, tx, openRelIDs, req)
	if err != nil {
		return api.EmployeeTenureClosed{}, err
	}
	if err := markTenureClosedRow(ctx, tx, tenureID, state.rehireEligible, req); err != nil {
		return api.EmployeeTenureClosed{}, err
	}
	revoked, err := revokeOptionalTenureUser(ctx, tx, revoker, state.userID)
	if err != nil {
		return api.EmployeeTenureClosed{}, err
	}
	return api.EmployeeTenureClosed{
		TenureID:            tenureID,
		EmployeeID:          employeeID,
		EndDate:             req.EndDate,
		ClosedRelationships: closedRels,
		ClosedSegments:      closedSegments,
		SessionsRevoked:     revoked,
	}, nil
}

func markTenureClosedRow(ctx context.Context, tx pgx.Tx, tenureID uuid.UUID, existingEligible bool, req api.CloseEmployeeTenureRequest) error {
	rehireEligible := existingEligible
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
	`, req.EndDate, req.TerminationReason, rehireEligible, req.Notes, tenureID); err != nil {
		return mapEmployeeWriteError(err)
	}
	return nil
}

func revokeOptionalTenureUser(ctx context.Context, tx pgx.Tx, revoker SessionRevoker, userID *uuid.UUID) (bool, error) {
	if userID == nil || *userID == uuid.Nil {
		return false, nil
	}
	if err := revokeUserSessionsDBTX(ctx, tx, revoker, *userID); err != nil {
		return false, err
	}
	return true, nil
}

func lockOpenTenureForClose(ctx context.Context, tx pgx.Tx, employeeID, tenureID uuid.UUID, endParsed time.Time) (lockedTenureCloseState, error) {
	if err := tx.QueryRow(ctx, `SELECT id FROM employees WHERE id = $1 AND deleted_at IS NULL FOR UPDATE`, employeeID).Scan(new(uuid.UUID)); err != nil {
		return lockedTenureCloseState{}, err
	}
	var state lockedTenureCloseState
	var existingEnd *time.Time
	err := tx.QueryRow(ctx, `
		SELECT start_date, end_date, user_id, rehire_eligible
		FROM employee_tenures
		WHERE id = $1 AND employee_id = $2 AND deleted_at IS NULL
		FOR UPDATE
	`, tenureID, employeeID).Scan(&state.startDate, &existingEnd, &state.userID, &state.rehireEligible)
	if err != nil {
		return lockedTenureCloseState{}, err
	}
	if existingEnd != nil {
		return lockedTenureCloseState{}, fmt.Errorf("%w: tenure is already closed", api.ErrConflict)
	}
	if endParsed.Before(state.startDate) {
		return lockedTenureCloseState{}, fmt.Errorf("%w: end_date precedes tenure start_date", api.ErrInvalidInput)
	}
	return state, nil
}

type lockedTenureRelRow struct {
	id         uuid.UUID
	employerID uuid.UUID
	startDate  time.Time
	endDate    *time.Time
}

func loadAndValidateOpenTenureRelationships(ctx context.Context, tx pgx.Tx, tenureID uuid.UUID, endDateStr string, endParsed time.Time) ([]uuid.UUID, error) {
	rels, err := lockTenureRelationshipRows(ctx, tx, tenureID)
	if err != nil {
		return nil, err
	}
	var openRelIDs []uuid.UUID
	for _, rel := range rels {
		if endParsed.Before(rel.startDate) || (rel.endDate != nil && rel.endDate.After(endParsed)) {
			return nil, fmt.Errorf("%w: end_date conflicts with child relationship dates", api.ErrConflict)
		}
		if rel.endDate == nil {
			if err := checkLockedPayrollForEmployeeDate(ctx, tx, rel.employerID, endDateStr); err != nil {
				return nil, err
			}
			openRelIDs = append(openRelIDs, rel.id)
		}
	}
	return openRelIDs, nil
}

func lockTenureRelationshipRows(ctx context.Context, tx pgx.Tx, tenureID uuid.UUID) ([]lockedTenureRelRow, error) {
	relRows, err := tx.Query(ctx, `
		SELECT id, employer_id, start_date, end_date
		FROM employment_relationships
		WHERE tenure_id = $1 AND deleted_at IS NULL
		FOR UPDATE
	`, tenureID)
	if err != nil {
		return nil, err
	}
	defer relRows.Close()
	var rels []lockedTenureRelRow
	for relRows.Next() {
		var item lockedTenureRelRow
		if err := relRows.Scan(&item.id, &item.employerID, &item.startDate, &item.endDate); err != nil {
			return nil, err
		}
		rels = append(rels, item)
	}
	return rels, relRows.Err()
}

func closeOpenTenureChildren(ctx context.Context, tx pgx.Tx, openRelIDs []uuid.UUID, req api.CloseEmployeeTenureRequest) (int, int, error) {
	if len(openRelIDs) == 0 {
		return 0, 0, nil
	}
	var invalidSeg bool
	if err := tx.QueryRow(ctx, `
		SELECT EXISTS (
			SELECT 1
			FROM employment_segments
			WHERE relationship_id = ANY($1)
			  AND deleted_at IS NULL
			  AND (
			      (end_date IS NULL AND start_date > $2::date)
			      OR (end_date IS NOT NULL AND end_date > $2::date)
			  )
		)
	`, openRelIDs, req.EndDate).Scan(&invalidSeg); err != nil {
		return 0, 0, err
	}
	if invalidSeg {
		return 0, 0, fmt.Errorf("%w: end_date conflicts with employment segment dates", api.ErrConflict)
	}

	segTag, err := tx.Exec(ctx, `
		UPDATE employment_segments
		SET status = 'terminated',
		    end_date = $1::date,
		    termination_reason = COALESCE($2, termination_reason),
		    notes = COALESCE($3, notes),
		    updated_at = NOW()
		WHERE relationship_id = ANY($4)
		  AND deleted_at IS NULL
		  AND end_date IS NULL
	`, req.EndDate, req.TerminationReason, req.Notes, openRelIDs)
	if err != nil {
		return 0, 0, mapEmployeeWriteError(err)
	}

	if err := closeRelationshipPolicyMemberships(ctx, tx, openRelIDs, req.EndDate); err != nil {
		return 0, 0, mapEmployeeWriteError(err)
	}

	relTag, err := tx.Exec(ctx, `
		UPDATE employment_relationships
		SET status = 'terminated',
		    end_date = $1::date,
		    termination_reason = COALESCE($2, termination_reason),
		    notes = COALESCE($3, notes),
		    updated_at = NOW()
		WHERE id = ANY($4)
		  AND deleted_at IS NULL
		  AND end_date IS NULL
	`, req.EndDate, req.TerminationReason, req.Notes, openRelIDs)
	if err != nil {
		return 0, 0, mapEmployeeWriteError(err)
	}
	return int(relTag.RowsAffected()), int(segTag.RowsAffected()), nil
}
