//go:build integration

package repository

import (
	"context"
	"encoding/json"
	"os"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func TestIntegrationPayrollPolicySnapshotsAreAppendOnlyAcrossReruns(t *testing.T) {
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
	defer pool.Close()

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
