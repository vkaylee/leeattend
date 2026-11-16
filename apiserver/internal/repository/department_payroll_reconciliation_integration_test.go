//go:build integration

package repository

import (
	"testing"

	"github.com/google/uuid"
)

func TestIntegrationDepartmentPayrollReconciliationReportsLegacyStates(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	type fixture struct {
		name        string
		legacyValue *float64
		canonical   *float64
	}
	matched := 22.0
	mismatch := 24.0
	fixtures := []fixture{
		{name: "reconciliation-matched", legacyValue: &matched, canonical: &matched},
		{name: "reconciliation-mismatch", legacyValue: &matched, canonical: &mismatch},
		{name: "reconciliation-missing-canonical", legacyValue: &matched},
		{name: "reconciliation-missing-legacy", canonical: &matched},
	}

	for _, item := range fixtures {
		departmentID := uuid.New()
		if item.legacyValue == nil {
			_, err = tx.Exec(ctx, `INSERT INTO departments (id, name, std_working_days) VALUES ($1, $2, NULL)`, departmentID, item.name)
		} else {
			_, err = tx.Exec(ctx, `INSERT INTO departments (id, name, std_working_days) VALUES ($1, $2, $3)`, departmentID, item.name, *item.legacyValue)
		}
		if err != nil {
			t.Fatalf("insert department %s: %v", item.name, err)
		}
		if item.canonical != nil {
			_, err = tx.Exec(ctx, `INSERT INTO config_entries (scope_type, scope_id, config_key, value_decimal, effective_from) VALUES ('department', $1, 'payroll.fixed_std_days', $2, DATE '2026-01-01')`, departmentID, *item.canonical)
			if err != nil {
				t.Fatalf("insert canonical config %s: %v", item.name, err)
			}
		}
	}

	rows, err := tx.Query(ctx, `
		SELECT reconciliation_status, count(*)
		FROM department_payroll_reconciliation
		WHERE department_name LIKE 'reconciliation-%'
		GROUP BY reconciliation_status`)
	if err != nil {
		t.Fatalf("query reconciliation view: %v", err)
	}
	defer rows.Close()

	statuses := make(map[string]int64)
	for rows.Next() {
		var status string
		var count int64
		if err := rows.Scan(&status, &count); err != nil {
			t.Fatalf("scan reconciliation status: %v", err)
		}
		statuses[status] = count
	}
	if err := rows.Err(); err != nil {
		t.Fatalf("iterate reconciliation statuses: %v", err)
	}

	want := map[string]int64{
		"matched":                  1,
		"value_mismatch":           1,
		"missing_canonical_config": 1,
		"missing_legacy_value":     1,
	}
	for status, expected := range want {
		if statuses[status] != expected {
			t.Fatalf("status %q count = %d, want %d; all statuses: %#v", status, statuses[status], expected, statuses)
		}
	}
}
