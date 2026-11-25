//go:build integration

package repository

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

func strPtr(v string) *string { return &v }

type atomicWriteScopeFixture struct {
	employerA  uuid.UUID
	employerB  uuid.UUID
	department uuid.UUID
	location   uuid.UUID
}

func seedAtomicWriteScope(t *testing.T, ctx context.Context, db DBTX) atomicWriteScopeFixture {
	t.Helper()
	f := atomicWriteScopeFixture{
		employerA:  uuid.New(),
		employerB:  uuid.New(),
		department: uuid.New(),
		location:   uuid.New(),
	}
	for _, item := range []struct {
		id   uuid.UUID
		name string
	}{
		{f.employerA, "Atomic Employer A " + f.employerA.String()[:8]},
		{f.employerB, "Atomic Employer B " + f.employerB.String()[:8]},
	} {
		if _, err := db.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, $2)`, item.id, item.name); err != nil {
			t.Fatalf("insert employer: %v", err)
		}
	}
	if _, err := db.Exec(ctx, `INSERT INTO departments (id, name) VALUES ($1, $2)`, f.department, "Atomic Dept "+f.department.String()[:8]); err != nil {
		t.Fatalf("insert department: %v", err)
	}
	if _, err := db.Exec(ctx, `INSERT INTO work_locations (id, name, status) VALUES ($1, $2, 'active')`, f.location, "Atomic Loc "+f.location.String()[:8]); err != nil {
		t.Fatalf("insert location: %v", err)
	}
	return f
}

func TestIntegrationEmployeeCreateAtomicRollbackAndUniqueness(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin tx: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	scope := seedAtomicWriteScope(t, ctx, tx)
	hasher, err := identity.NewHasher(testIdentityBackfillSecret)
	if err != nil {
		t.Fatalf("NewHasher: %v", err)
	}
	repo := EmployeeRepository{
		DB:      tx,
		BeginTx: func(context.Context) (service.TxBeginner, error) { return tx, nil },
	}

	nidHash, _, _ := hasher.HashNationalID("001-234-567")
	phoneHash, _, _ := hasher.HashPhone("+84912345678")

	created, err := repo.CreateEmployee(ctx, api.CreateEmployeeRequest{
		FullName:             "Nguyễn Atomic",
		EmployeeIDNumber:     "EMP-ATM-01",
		HireDate:             "2026-01-10",
		EmployerID:           &scope.employerA,
		DepartmentID:         &scope.department,
		PrimaryLocationID:    &scope.location,
		Phone:                strPtr("+84912345678"),
		DateOfBirth:          strPtr("1992-06-15"),
		NationalID:           strPtr("001-234-567"),
		PhoneLookupHash:      &phoneHash,
		NationalIDLookupHash: &nidHash,
	})
	if err != nil {
		t.Fatalf("CreateEmployee with identity: %v", err)
	}
	assertNewEmployeeGraphExists(t, ctx, tx, created.ID, true)

	// Legacy client without identity fields remains compatible and still creates profile + first tenure.
	legacy, err := repo.CreateEmployee(ctx, api.CreateEmployeeRequest{
		FullName:         "Legacy Caller",
		EmployeeIDNumber: "EMP-LEG-01",
		HireDate:         "2026-01-10",
	})
	if err != nil {
		t.Fatalf("legacy CreateEmployee: %v", err)
	}
	assertNewEmployeeGraphExists(t, ctx, tx, legacy.ID, false)

	// Duplicate normalized code, national ID hash, and phone hash all return deterministic ErrConflict.
	assertCreateDuplicateRejected(t, ctx, repo, "  emp-atm-01 ", nil, nil)
	assertCreateDuplicateRejected(t, ctx, repo, "EMP-UNIQ-02", &nidHash, nil)
	assertCreateDuplicateRejected(t, ctx, repo, "EMP-UNIQ-03", nil, &phoneHash)

	assertInvalidScopeCreateRollsBack(t, ctx, tx, repo)
}

func assertInvalidScopeCreateRollsBack(t *testing.T, ctx context.Context, tx pgx.Tx, repo EmployeeRepository) {
	t.Helper()
	missingEmployer := uuid.New()
	if _, err := repo.CreateEmployee(ctx, api.CreateEmployeeRequest{
		FullName:         "Rollback Person",
		EmployeeIDNumber: "EMP-RB-01",
		HireDate:         "2026-01-10",
		EmployerID:       &missingEmployer,
	}); !errors.Is(err, api.ErrInvalidInput) {
		t.Fatalf("invalid employer err = %v, want ErrInvalidInput", err)
	}
	var leftover int
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM employee_tenures WHERE employee_number = 'EMP-RB-01'`).Scan(&leftover); err != nil || leftover != 0 {
		t.Fatalf("rollback leaked employee rows: count=%d err=%v", leftover, err)
	}
}

func assertNewEmployeeGraphExists(t *testing.T, ctx context.Context, db DBTX, empID uuid.UUID, expectEmployer bool) {
	t.Helper()
	var profiles, tenures, rels, emps int
	if err := db.QueryRow(ctx, `
		SELECT
			(SELECT count(*) FROM employee_profiles WHERE employee_id = $1),
			(SELECT count(*) FROM employee_tenures WHERE employee_id = $1 AND deleted_at IS NULL),
			(SELECT count(*) FROM employment_relationships WHERE employee_id = $1 AND deleted_at IS NULL),
			(SELECT count(*) FROM employments WHERE employee_id = $1 AND deleted_at IS NULL)
	`, empID).Scan(&profiles, &tenures, &rels, &emps); err != nil {
		t.Fatalf("query created graph: %v", err)
	}
	wantEmp := 0
	if expectEmployer {
		wantEmp = 1
	}
	if profiles != 1 || tenures != 1 || rels != wantEmp || emps != wantEmp {
		t.Fatalf("created graph = (profiles=%d tenures=%d rels=%d emps=%d), want (1,1,%d,%d)", profiles, tenures, rels, emps, wantEmp, wantEmp)
	}
}

func assertCreateDuplicateRejected(t *testing.T, ctx context.Context, repo EmployeeRepository, code string, nidHash, phoneHash *string) {
	t.Helper()
	_, err := repo.CreateEmployee(ctx, api.CreateEmployeeRequest{
		FullName:             "Duplicate Attempt",
		EmployeeIDNumber:     code,
		HireDate:             "2026-02-01",
		NationalIDLookupHash: nidHash,
		PhoneLookupHash:      phoneHash,
	})
	if !errors.Is(err, api.ErrConflict) {
		t.Fatalf("duplicate create code=%q err = %v, want ErrConflict", code, err)
	}
}

func TestIntegrationEmployeeReuseTenureAndDownstreamResolution(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin tx: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	scope := seedAtomicWriteScope(t, ctx, tx)
	repo := EmployeeRepository{
		DB:      tx,
		BeginTx: func(context.Context) (service.TxBeginner, error) { return tx, nil },
	}

	empID, oldTenureID := seedTerminatedEmployeeForReuse(t, ctx, tx, scope.employerA, "EMP-REUSE-01", "2025-01-01", "2025-06-30")

	// Reject date overlap, stale expected code, invalid scope, and missing employee without partial writes.
	assertReuseTenureError(t, ctx, repo, empID, api.CreateEmployeeTenureRequest{StartDate: "2025-06-30", EmployerID: scope.employerB}, api.ErrConflict)
	assertReuseTenureError(t, ctx, repo, empID, api.CreateEmployeeTenureRequest{StartDate: "2025-07-01", EmployerID: scope.employerB, ExpectedEmployeeCode: strPtr("WRONG-CODE")}, api.ErrConflict)
	assertReuseTenureError(t, ctx, repo, empID, api.CreateEmployeeTenureRequest{StartDate: "2025-07-01", EmployerID: uuid.New()}, api.ErrInvalidInput)
	assertReuseTenureError(t, ctx, repo, uuid.New(), api.CreateEmployeeTenureRequest{StartDate: "2025-07-01", EmployerID: scope.employerB}, pgx.ErrNoRows)

	// Reuse terminated employee under a DIFFERENT employer (employerB) keeping ID, code, and immutable history.
	created, err := repo.CreateEmployeeTenure(ctx, empID, api.CreateEmployeeTenureRequest{
		StartDate:            "2025-07-01",
		EmployerID:           scope.employerB,
		DepartmentID:         &scope.department,
		PrimaryLocationID:    &scope.location,
		ExpectedEmployeeCode: strPtr("emp-reuse-01"),
		ExpectedStatus:       strPtr(api.MatchStatusTerminated),
	})
	if err != nil {
		t.Fatalf("CreateEmployeeTenure reuse: %v", err)
	}
	if created.EmployeeID != empID || created.EmployeeCode != "EMP-REUSE-01" || created.EmployerID != scope.employerB || created.TenureID == oldTenureID {
		t.Fatalf("unexpected reuse result: %+v", created)
	}

	// Active employee reuse must now be rejected.
	assertReuseTenureError(t, ctx, repo, empID, api.CreateEmployeeTenureRequest{StartDate: "2026-01-01", EmployerID: scope.employerA}, api.ErrConflict)

	// Soft-deleted employee cannot be reused via tenure endpoint.
	deletedID, _ := seedTerminatedEmployeeForReuse(t, ctx, tx, scope.employerA, "EMP-DEL-01", "2025-01-01", "2025-06-30")
	if _, err := tx.Exec(ctx, `UPDATE employees SET deleted_at = NOW() WHERE id = $1`, deletedID); err != nil {
		t.Fatalf("soft delete employee: %v", err)
	}
	assertReuseTenureError(t, ctx, repo, deletedID, api.CreateEmployeeTenureRequest{StartDate: "2025-07-01", EmployerID: scope.employerB}, api.ErrConflict)

	// Checkpoint D-C: immutable history + employee detail + ESS resolve new tenure & intact profile.
	assertHistoricalImmutabilityAndDownstreamResolution(t, ctx, tx, repo, empID, oldTenureID, created, scope.employerB)
}

func seedTerminatedEmployeeForReuse(
	t *testing.T,
	ctx context.Context,
	db DBTX,
	employerID uuid.UUID,
	code, startDate, endDate string,
) (uuid.UUID, uuid.UUID) {
	t.Helper()
	empID := uuid.New()
	if _, err := db.Exec(ctx, `
		INSERT INTO employees (id, full_name)
		VALUES ($1, 'Terminated Reuse Person')
	`, empID); err != nil {
		t.Fatalf("insert terminated employee: %v", err)
	}
	if _, err := db.Exec(ctx, `INSERT INTO employee_profiles (employee_id, phone, date_of_birth) VALUES ($1, '+84900000001', DATE '1990-01-01')`, empID); err != nil {
		t.Fatalf("insert profile: %v", err)
	}
	var tenureID uuid.UUID
	if err := db.QueryRow(ctx, `
		INSERT INTO employee_tenures (employee_id, employee_number, start_date, end_date)
		VALUES ($1, $2, $3::date, $4::date)
		RETURNING id
	`, empID, code, startDate, endDate).Scan(&tenureID); err != nil {
		t.Fatalf("insert closed tenure: %v", err)
	}
	relID := uuid.New()
	if _, err := db.Exec(ctx, `
		INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date, end_date)
		VALUES ($1, $2, $3, $4, 'terminated', $5::date, $6::date)
	`, relID, empID, employerID, tenureID, startDate, endDate); err != nil {
		t.Fatalf("insert terminated relationship: %v", err)
	}
	if _, err := db.Exec(ctx, `
		INSERT INTO employments (id, employee_id, employer_id, relationship_id, status, start_date, end_date)
		VALUES ($1, $2, $3, $4, 'terminated', $5::date, $6::date)
	`, uuid.New(), empID, employerID, relID, startDate, endDate); err != nil {
		t.Fatalf("insert terminated employment: %v", err)
	}
	return empID, tenureID
}

func assertReuseTenureError(
	t *testing.T,
	ctx context.Context,
	repo EmployeeRepository,
	empID uuid.UUID,
	req api.CreateEmployeeTenureRequest,
	wantErr error,
) {
	t.Helper()
	_, err := repo.CreateEmployeeTenure(ctx, empID, req)
	if !errors.Is(err, wantErr) {
		t.Fatalf("CreateEmployeeTenure err = %v, want %v", err, wantErr)
	}
}

func assertHistoricalImmutabilityAndDownstreamResolution(
	t *testing.T,
	ctx context.Context,
	tx pgx.Tx,
	repo EmployeeRepository,
	empID, oldTenureID uuid.UUID,
	created api.EmployeeTenureCreated,
	newEmployerID uuid.UUID,
) {
	t.Helper()
	assertHistoricalTenureUnchanged(t, ctx, tx, oldTenureID)
	assertEmployeeDetailAfterReuse(t, ctx, repo, empID, created.EmploymentID, newEmployerID)
	assertESSResolvesReusedTenure(t, ctx, tx, empID, created.TenureID)
}

func assertHistoricalTenureUnchanged(t *testing.T, ctx context.Context, tx pgx.Tx, oldTenureID uuid.UUID) {
	t.Helper()
	var oldEndDate string
	var oldUserID *uuid.UUID
	if err := tx.QueryRow(ctx, `SELECT end_date::text, user_id FROM employee_tenures WHERE id = $1`, oldTenureID).Scan(&oldEndDate, &oldUserID); err != nil {
		t.Fatalf("read old tenure: %v", err)
	}
	if oldEndDate != "2025-06-30" || oldUserID != nil {
		t.Fatalf("historical tenure mutated: end_date=%s user_id=%v", oldEndDate, oldUserID)
	}
}

func assertEmployeeDetailAfterReuse(t *testing.T, ctx context.Context, repo EmployeeRepository, empID, employmentID, employerID uuid.UUID) {
	t.Helper()
	detail, err := repo.GetEmployee(ctx, empID)
	if err != nil {
		t.Fatalf("GetEmployee after reuse: %v", err)
	}
	if detail.EmployeeIDNumber != "EMP-REUSE-01" {
		t.Fatalf("employee code = %q, want EMP-REUSE-01", detail.EmployeeIDNumber)
	}
	if detail.ActiveEmploymentID == nil || *detail.ActiveEmploymentID != employmentID {
		t.Fatalf("active_employment_id = %v, want %s", detail.ActiveEmploymentID, employmentID)
	}
	if detail.EmployerID == nil || *detail.EmployerID != employerID {
		t.Fatalf("employer_id = %v, want %s", detail.EmployerID, employerID)
	}
}

func assertESSResolvesReusedTenure(t *testing.T, ctx context.Context, tx pgx.Tx, empID, newTenureID uuid.UUID) {
	t.Helper()
	userID := linkTestUserToTenure(t, ctx, tx, empID, newTenureID)
	essRepo := ESSRepository{DB: tx}
	resolvedEmpID, err := essRepo.ResolveEmployeeForUser(ctx, userID)
	if err != nil || resolvedEmpID != empID {
		t.Fatalf("ESS ResolveEmployeeForUser = (%s, %v), want %s", resolvedEmpID, err, empID)
	}
	profileResp, err := essRepo.GetMyProfile(ctx, empID)
	if err != nil {
		t.Fatalf("ESS GetMyProfile err: %v", err)
	}
	if profileResp.Employee.EmployeeIDNumber != "EMP-REUSE-01" || profileResp.Profile == nil || profileResp.Profile.Phone == nil || *profileResp.Profile.Phone != "+84900000001" {
		t.Fatalf("ESS GetMyProfile mismatch: %+v", profileResp)
	}
}

func linkTestUserToTenure(t *testing.T, ctx context.Context, tx pgx.Tx, empID, tenureID uuid.UUID) uuid.UUID {
	t.Helper()
	userID := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash, full_name) VALUES ($1, $2, 'hash', 'ESS Reuse User')`, userID, fmt.Sprintf("ess-reuse-%s@example.invalid", userID.String()[:8])); err != nil {
		t.Fatalf("insert ess user: %v", err)
	}
	if _, err := tx.Exec(ctx, `UPDATE employee_tenures SET user_id = $1 WHERE id = $2`, userID, tenureID); err != nil {
		t.Fatalf("link new tenure user: %v", err)
	}
	return userID
}

func TestIntegrationEmployeeConcurrentCreateAndReuseRaces(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	hasher, err := identity.NewHasher(testIdentityBackfillSecret)
	if err != nil {
		t.Fatalf("NewHasher: %v", err)
	}
	scope := seedAtomicWriteScope(t, ctx, pool)
	t.Cleanup(func() {
		_, _ = pool.Exec(context.Background(), `DELETE FROM employments WHERE employer_id IN ($1, $2)`, scope.employerA, scope.employerB)
		_, _ = pool.Exec(context.Background(), `DELETE FROM employment_relationships WHERE employer_id IN ($1, $2)`, scope.employerA, scope.employerB)
		_, _ = pool.Exec(context.Background(), `DELETE FROM employers WHERE id IN ($1, $2)`, scope.employerA, scope.employerB)
		_, _ = pool.Exec(context.Background(), `DELETE FROM departments WHERE id = $1`, scope.department)
		_, _ = pool.Exec(context.Background(), `DELETE FROM work_locations WHERE id = $1`, scope.location)
	})

	repo := EmployeeRepository{DB: pool}
	runConcurrentCreateCodeRace(t, ctx, pool, repo)
	runConcurrentCreateIdentityRace(t, ctx, pool, repo, hasher)
	runConcurrentReuseTenureRace(t, ctx, pool, repo, scope)
}

func runConcurrentCreateCodeRace(t *testing.T, ctx context.Context, pool *pgxpool.Pool, repo EmployeeRepository) {
	t.Helper()
	tag := uuid.NewString()[:8]
	codes := []string{"race-" + tag, "  RACE-" + tag + " "}
	t.Cleanup(func() {
		_, _ = pool.Exec(context.Background(), `DELETE FROM employee_profiles WHERE employee_id IN (SELECT employee_id FROM employee_tenures WHERE employee_number_normalized = UPPER($1))`, "race-"+tag)
		_, _ = pool.Exec(context.Background(), `DELETE FROM employee_tenures WHERE employee_number_normalized = UPPER($1)`, "race-"+tag)
		_, _ = pool.Exec(context.Background(), `DELETE FROM employees WHERE full_name LIKE 'Race Code %' AND id NOT IN (SELECT employee_id FROM employee_tenures)`)
	})

	errs := runParallel2(func(idx int) error {
		_, err := repo.CreateEmployee(ctx, api.CreateEmployeeRequest{
			FullName:         fmt.Sprintf("Race Code %d", idx),
			EmployeeIDNumber: codes[idx],
			HireDate:         "2026-03-01",
		})
		return err
	})
	assertOneWinnerOneConflict(t, "concurrent create code", errs)
}

func runConcurrentCreateIdentityRace(t *testing.T, ctx context.Context, pool *pgxpool.Pool, repo EmployeeRepository, hasher identity.Hasher) {
	t.Helper()
	tag := uuid.NewString()[:8]
	nidHash, _, _ := hasher.HashNationalID("999-" + tag)
	t.Cleanup(func() {
		_, _ = pool.Exec(context.Background(), `DELETE FROM employee_tenures WHERE employee_id IN (SELECT employee_id FROM employee_profiles WHERE national_id_lookup_hash = $1)`, nidHash)
		_, _ = pool.Exec(context.Background(), `DELETE FROM employees WHERE id IN (SELECT employee_id FROM employee_profiles WHERE national_id_lookup_hash = $1)`, nidHash)
	})

	errs := runParallel2(func(idx int) error {
		_, err := repo.CreateEmployee(ctx, api.CreateEmployeeRequest{
			FullName:             fmt.Sprintf("Race Identity %d", idx),
			EmployeeIDNumber:     fmt.Sprintf("RACE-ID-%s-%d", tag, idx),
			HireDate:             "2026-03-01",
			NationalID:           strPtr("999-" + tag),
			NationalIDLookupHash: &nidHash,
		})
		return err
	})
	assertOneWinnerOneConflict(t, "concurrent create identity", errs)
}

func runConcurrentReuseTenureRace(t *testing.T, ctx context.Context, pool *pgxpool.Pool, repo EmployeeRepository, scope atomicWriteScopeFixture) {
	t.Helper()
	empID, _ := seedTerminatedEmployeeForReuse(t, ctx, pool, scope.employerA, "RACE-REUSE-"+uuid.NewString()[:8], "2025-01-01", "2025-06-30")
	t.Cleanup(func() {
		_, _ = pool.Exec(context.Background(), `DELETE FROM employments WHERE employee_id = $1`, empID)
		_, _ = pool.Exec(context.Background(), `DELETE FROM employment_relationships WHERE employee_id = $1`, empID)
		_, _ = pool.Exec(context.Background(), `DELETE FROM employee_tenures WHERE employee_id = $1`, empID)
		_, _ = pool.Exec(context.Background(), `DELETE FROM employee_profiles WHERE employee_id = $1`, empID)
		_, _ = pool.Exec(context.Background(), `DELETE FROM employees WHERE id = $1`, empID)
	})

	errs := runParallel2(func(_ int) error {
		_, err := repo.CreateEmployeeTenure(ctx, empID, api.CreateEmployeeTenureRequest{
			StartDate:  "2025-07-01",
			EmployerID: scope.employerB,
		})
		return err
	})
	assertOneWinnerOneConflict(t, "concurrent reuse tenure", errs)

	var tenureCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM employee_tenures WHERE employee_id = $1 AND deleted_at IS NULL`, empID).Scan(&tenureCount); err != nil || tenureCount != 2 {
		t.Fatalf("after concurrent reuse tenure_count = %d (err=%v), want 2", tenureCount, err)
	}
}

func runParallel2(fn func(idx int) error) [2]error {
	var errs [2]error
	start := make(chan struct{})
	var wg sync.WaitGroup
	wg.Add(2)
	for i := 0; i < 2; i++ {
		go func(idx int) {
			defer wg.Done()
			<-start
			errs[idx] = fn(idx)
		}(i)
	}
	close(start)
	wg.Wait()
	return errs
}

func assertOneWinnerOneConflict(t *testing.T, label string, errs [2]error) {
	t.Helper()
	successes, conflicts := 0, 0
	for _, err := range errs {
		switch {
		case err == nil:
			successes++
		case errors.Is(err, api.ErrConflict):
			conflicts++
		default:
			t.Fatalf("%s unexpected error: %v", label, err)
		}
	}
	if successes != 1 || conflicts != 1 {
		t.Fatalf("%s outcomes: successes=%d conflicts=%d errs=%v, want 1 success + 1 conflict", label, successes, conflicts, errs)
	}
}
