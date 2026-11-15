//go:build integration

package repository

import (
	"context"
	"encoding/json"
	"errors"
	"os"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func requireIntegrationDatabase(t *testing.T) (*pgxpool.Pool, context.Context) {
	t.Helper()
	if os.Getenv("LEEATTEND_DB_INTEGRATION") != "1" {
		t.Fatal("LEEATTEND_DB_INTEGRATION=1 is required for database integration tests")
	}
	connectionString := os.Getenv("DB_PRIMARY_URL")
	if connectionString == "" {
		connectionString = os.Getenv("DATABASE_URL")
	}
	if connectionString == "" {
		t.Fatal("DB_PRIMARY_URL or DATABASE_URL is required for database integration tests")
	}
	ctx := context.Background()
	pool, err := pgxpool.New(ctx, connectionString)
	if err != nil {
		t.Fatalf("open database pool: %v", err)
	}
	t.Cleanup(pool.Close)
	return pool, ctx
}

func TestIntegrationPayrollPolicySnapshotsAreAppendOnlyAcrossReruns(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	employerID := uuid.New()
	employeeID := uuid.New()
	employmentID := uuid.New()
	payslipID := uuid.New()
	actorID := uuid.New()

	_, err = tx.Exec(ctx, `
		INSERT INTO employers (id, name)
		VALUES ($1, 'payroll-history-test-employer')`, employerID)
	if err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	_, err = tx.Exec(ctx, `
		INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date)
		VALUES ($1, $2, 'payroll-history-test-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	if err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	_, err = tx.Exec(ctx, `
		INSERT INTO employments (id, employee_id, employer_id, status, start_date)
		VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID)
	if err != nil {
		t.Fatalf("insert employment: %v", err)
	}
	_, err = tx.Exec(ctx, `
		INSERT INTO payslips (
			id, employment_id, period_start, period_end, base_pay_amount,
			overtime_pay_amount, deductions_amount, net_pay_amount, status
		)
		VALUES ($1, $2, DATE '2026-06-01', DATE '2026-06-30', 1000, 0, 0, 1000, 'draft')`, payslipID, employmentID)
	if err != nil {
		t.Fatalf("insert payslip: %v", err)
	}

	policyA := &api.WorkPolicyGroupVersion{
		ID: uuid.New(), GroupID: uuid.New(), EffectiveFrom: "2026-01-01",
		WorkCreditPolicyID: uuid.New(), PayBasisPolicyID: uuid.New(), OvertimePolicyID: uuid.New(),
		PayMethod: workpolicy.PayMethodMonthly,
	}
	policyB := &api.WorkPolicyGroupVersion{
		ID: uuid.New(), GroupID: uuid.New(), EffectiveFrom: "2026-06-01",
		WorkCreditPolicyID: uuid.New(), PayBasisPolicyID: uuid.New(), OvertimePolicyID: uuid.New(),
		PayMethod: workpolicy.PayMethodHourly,
	}

	if err := persistKPICalculation(ctx, tx, employmentID, payslipID, 2026, 6, kpiCalculation{Policy: policyA}, &actorID); err != nil {
		t.Fatalf("persist policy A snapshot: %v", err)
	}
	if err := persistKPICalculation(ctx, tx, employmentID, payslipID, 2026, 6, kpiCalculation{Policy: policyB}, &actorID); err != nil {
		t.Fatalf("persist policy B snapshot: %v", err)
	}

	var count int
	if err := tx.QueryRow(ctx, `
		SELECT count(*)
		FROM payroll_calculation_runs
		WHERE payslip_id = $1 AND formula_version_id IS NULL`, payslipID).Scan(&count); err != nil {
		t.Fatalf("count policy snapshots: %v", err)
	}
	if count != 2 {
		t.Fatalf("expected two policy snapshots for one payslip, got %d", count)
	}

	rows, err := tx.Query(ctx, `
		SELECT input_snapshot
		FROM payroll_calculation_runs
		WHERE payslip_id = $1 AND formula_version_id IS NULL
		ORDER BY created_at ASC, id ASC`, payslipID)
	if err != nil {
		t.Fatalf("load policy snapshots: %v", err)
	}
	defer rows.Close()

	seenVersions := make(map[string]bool)
	for rows.Next() {
		var raw []byte
		if err := rows.Scan(&raw); err != nil {
			t.Fatalf("scan policy snapshot: %v", err)
		}
		var snapshot map[string]any
		if err := json.Unmarshal(raw, &snapshot); err != nil {
			t.Fatalf("decode policy snapshot: %v", err)
		}
		policy, ok := snapshot["policy"].(map[string]any)
		if !ok {
			t.Fatalf("policy provenance missing from snapshot: %s", raw)
		}
		versionID, ok := policy["version_id"].(string)
		if !ok {
			t.Fatalf("policy version ID missing from snapshot: %s", raw)
		}
		seenVersions[versionID] = true
	}
	if err := rows.Err(); err != nil {
		t.Fatalf("iterate policy snapshots: %v", err)
	}
	if len(seenVersions) != 2 {
		t.Fatalf("expected two distinct policy snapshots, got %d", len(seenVersions))
	}
	if !seenVersions[policyA.ID.String()] {
		t.Fatalf("old policy snapshot provenance missing: %s", policyA.ID)
	}
	if !seenVersions[policyB.ID.String()] {
		t.Fatalf("new policy snapshot provenance missing: %s", policyB.ID)
	}
}

func TestIntegrationEnsurePayrollPeriodOpenRejectsLockedPeriod(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	employerID := uuid.New()
	_, err = tx.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'locked-period-test-employer')`, employerID)
	if err != nil {
		t.Fatalf("insert locked-period employer: %v", err)
	}
	_, err = tx.Exec(ctx, `INSERT INTO payroll_periods (employer_id, period_year, period_month, status) VALUES ($1, 2026, 6, 'locked')`, employerID)
	if err != nil {
		t.Fatalf("insert locked payroll period: %v", err)
	}

	if err := ensurePayrollPeriodOpen(ctx, tx, employerID, 2026, 6); !errors.Is(err, api.ErrPayrollPeriodConflict) {
		t.Fatalf("locked period error = %v, want conflict", err)
	}
	var status string
	if err := tx.QueryRow(ctx, `SELECT status FROM payroll_periods WHERE employer_id = $1 AND period_year = 2026 AND period_month = 6`, employerID).Scan(&status); err != nil {
		t.Fatalf("read locked payroll period: %v", err)
	}
	if status != "locked" {
		t.Fatalf("locked period status changed to %q", status)
	}
}

func TestIntegrationLegacyPayrollCalculationKeepsArithmeticAcrossReruns(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID := uuid.New()
	employeeID := uuid.New()
	employmentID := uuid.New()
	componentID := uuid.New()
	componentDefinitionID := uuid.New()

	_, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'legacy-parity-test-employer')`, employerID)
	if err != nil {
		t.Fatalf("insert parity employer: %v", err)
	}
	_, err = pool.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'legacy-parity-test-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	if err != nil {
		t.Fatalf("insert parity employee: %v", err)
	}
	_, err = pool.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID)
	if err != nil {
		t.Fatalf("insert parity employment: %v", err)
	}
	_, err = pool.Exec(ctx, `INSERT INTO salary_component_definitions (id, name, code, component_type) VALUES ($1, 'Legacy base salary', $2, 'base_salary')`, componentDefinitionID, "LEGACY_BASE_"+componentDefinitionID.String()[:8])
	if err != nil {
		t.Fatalf("insert parity salary definition: %v", err)
	}
	_, err = pool.Exec(ctx, `INSERT INTO employee_salary_components (id, component_definition_id, amount, effective_date, employment_id) VALUES ($1, $2, 2600, DATE '2026-01-01', $3)`, componentID, componentDefinitionID, employmentID)
	if err != nil {
		t.Fatalf("insert parity salary component: %v", err)
	}
	_, err = pool.Exec(ctx, `
		INSERT INTO attendance_daily_summaries (id, date, employment_id, standard_minutes)
		SELECT gen_random_uuid(), DATE '2026-01-01' + day, $1, 480
		FROM generate_series(0, 21) AS days(day)`, employmentID)
	if err != nil {
		t.Fatalf("insert parity attendance summaries: %v", err)
	}

	repo := PayrollRepository{DB: pool}
	first, err := repo.CalculateEmployeePayroll(ctx, ids.EmployeeID(employeeID), 2026, 1, ids.UserID(uuid.New()))
	if err != nil {
		t.Fatalf("first legacy payroll calculation: %v", err)
	}
	second, err := repo.CalculateEmployeePayroll(ctx, ids.EmployeeID(employeeID), 2026, 1, ids.UserID(uuid.New()))
	if err != nil {
		t.Fatalf("second legacy payroll calculation: %v", err)
	}
	if first.ID != second.ID {
		t.Fatalf("rerun changed payslip ID: first=%s second=%s", first.ID, second.ID)
	}
	if !first.BasePayAmount.Equal(decimal.RequireFromString("2200")) || !second.BasePayAmount.Equal(first.BasePayAmount) {
		t.Fatalf("legacy base pay changed: first=%s second=%s want 2200", first.BasePayAmount, second.BasePayAmount)
	}
	if !first.NetPayAmount.Equal(decimal.RequireFromString("2200")) || !second.NetPayAmount.Equal(first.NetPayAmount) {
		t.Fatalf("legacy net pay changed: first=%s second=%s want 2200", first.NetPayAmount, second.NetPayAmount)
	}
	if first.StandardUnits != 176 || second.StandardUnits != first.StandardUnits {
		t.Fatalf("legacy standard units changed: first=%d second=%d want 176", first.StandardUnits, second.StandardUnits)
	}

	var snapshotCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM payroll_calculation_runs WHERE payslip_id = $1 AND formula_version_id IS NULL`, first.ID).Scan(&snapshotCount); err != nil {
		t.Fatalf("count legacy policy snapshots: %v", err)
	}
	if snapshotCount != 2 {
		t.Fatalf("expected append-only legacy snapshots after two calculations, got %d", snapshotCount)
	}
}

func TestIntegrationResolveWorkPolicyUsesExclusiveMembershipAndVersionBoundaries(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	userID := uuid.New()
	employerID := uuid.New()
	employeeID := uuid.New()
	employmentID := uuid.New()
	attendancePolicyID := uuid.New()
	workingPatternID := uuid.New()
	overtimePolicyID := uuid.New()
	groupID := uuid.New()
	workCreditA := uuid.New()
	payBasisA := uuid.New()
	versionA := uuid.New()
	workCreditB := uuid.New()
	payBasisB := uuid.New()
	versionB := uuid.New()
	missingMembershipEmploymentID := uuid.New()

	_, err = tx.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, userID, userID.String()+"@invalid.test")
	if err != nil {
		t.Fatalf("insert resolver user: %v", err)
	}
	_, err = tx.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'resolver-test-employer')`, employerID)
	if err != nil {
		t.Fatalf("insert resolver employer: %v", err)
	}
	_, err = tx.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'resolver-test-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	if err != nil {
		t.Fatalf("insert resolver employee: %v", err)
	}
	_, err = tx.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID)
	if err != nil {
		t.Fatalf("insert resolver employment: %v", err)
	}
	_, err = tx.Exec(ctx, `INSERT INTO overtime_policy_definitions (id, name) VALUES ($1, 'resolver-test-overtime')`, overtimePolicyID)
	if err != nil {
		t.Fatalf("insert resolver overtime policy: %v", err)
	}
	_, err = tx.Exec(ctx, `INSERT INTO attendance_policies (id, name) VALUES ($1, 'resolver-test-attendance')`, attendancePolicyID)
	if err != nil {
		t.Fatalf("insert resolver attendance policy: %v", err)
	}
	_, err = tx.Exec(ctx, `INSERT INTO working_patterns (id, name) VALUES ($1, 'resolver-test-pattern')`, workingPatternID)
	if err != nil {
		t.Fatalf("insert resolver working pattern: %v", err)
	}
	_, err = tx.Exec(ctx, `INSERT INTO labor_policy_groups (id, name, created_by) VALUES ($1, 'resolver-test-group', $2)`, groupID, userID)
	if err != nil {
		t.Fatalf("insert resolver group: %v", err)
	}
	for _, definition := range []struct {
		workCredit, payBasis, version uuid.UUID
		minutes, days                 int
		from, to                      string
	}{
		{workCreditA, payBasisA, versionA, 480, 26, "2026-01-01", "2026-06-01"},
		{workCreditB, payBasisB, versionB, 450, 22, "2026-06-01", ""},
	} {
		_, err = tx.Exec(ctx, `INSERT INTO work_credit_policies (id, name, minutes_per_credit, attendance_policy_id, working_pattern_id, created_by) VALUES ($1, $2, $3, $4, $5, $6)`, definition.workCredit, definition.version.String(), definition.minutes, attendancePolicyID, workingPatternID, userID)
		if err != nil {
			t.Fatalf("insert resolver work credit policy: %v", err)
		}
		_, err = tx.Exec(ctx, `INSERT INTO pay_basis_policies (id, name, method, monthly_denominator, monthly_standard_days, created_by) VALUES ($1, $2, 'monthly', 'fixed', $3, $4)`, definition.payBasis, definition.version.String(), definition.days, userID)
		if err != nil {
			t.Fatalf("insert resolver pay basis policy: %v", err)
		}
		var effectiveTo any
		if definition.to != "" {
			effectiveTo = definition.to
		}
		_, err = tx.Exec(ctx, `INSERT INTO labor_policy_group_versions (id, group_id, work_credit_policy_id, pay_basis_policy_id, overtime_policy_id, effective_from, effective_to, created_by) VALUES ($1, $2, $3, $4, $5, $6::date, $7::date, $8)`, definition.version, groupID, definition.workCredit, definition.payBasis, overtimePolicyID, definition.from, effectiveTo, userID)
		if err != nil {
			t.Fatalf("insert resolver version: %v", err)
		}
	}
	_, err = tx.Exec(ctx, `INSERT INTO labor_policy_memberships (id, employment_id, group_id, effective_from, effective_to, created_by) VALUES ($1, $2, $3, DATE '2026-01-01', NULL, $4)`, uuid.New(), employmentID, groupID, userID)
	if err != nil {
		t.Fatalf("insert resolver membership: %v", err)
	}
	if _, err := tx.Exec(ctx, `SAVEPOINT overlapping_membership`); err != nil {
		t.Fatalf("create overlap savepoint: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO labor_policy_memberships (id, employment_id, group_id, effective_from, effective_to, created_by) VALUES ($1, $2, $3, DATE '2026-05-01', DATE '2026-07-01', $4)`, uuid.New(), employmentID, groupID, userID); err == nil {
		t.Fatal("overlapping membership must be rejected by database constraint")
	}
	if _, err := tx.Exec(ctx, `ROLLBACK TO SAVEPOINT overlapping_membership`); err != nil {
		t.Fatalf("rollback overlap savepoint: %v", err)
	}

	repo := WorkPolicyRepository{DB: tx}
	beforeBoundary, err := repo.ResolveWorkPolicy(ctx, ids.EmploymentID(employmentID), time.Date(2026, 5, 31, 0, 0, 0, 0, time.UTC))
	if err != nil {
		t.Fatalf("resolve before version boundary: %v", err)
	}
	if beforeBoundary.ID != versionA || beforeBoundary.MinutesPerCredit != 480 {
		t.Fatalf("before boundary resolved %+v, want version A/480 minutes", beforeBoundary)
	}
	onBoundary, err := repo.ResolveWorkPolicy(ctx, ids.EmploymentID(employmentID), time.Date(2026, 6, 1, 0, 0, 0, 0, time.UTC))
	if err != nil {
		t.Fatalf("resolve on version boundary: %v", err)
	}
	if onBoundary.ID != versionB || onBoundary.MinutesPerCredit != 450 {
		t.Fatalf("on boundary resolved %+v, want version B/450 minutes", onBoundary)
	}
	_, err = repo.ResolveWorkPolicy(ctx, ids.EmploymentID(missingMembershipEmploymentID), time.Date(2026, 6, 1, 0, 0, 0, 0, time.UTC))
	if !errors.Is(err, api.ErrWorkPolicyResolutionNotFound) {
		t.Fatalf("missing membership error = %v, want not found", err)
	}
}
