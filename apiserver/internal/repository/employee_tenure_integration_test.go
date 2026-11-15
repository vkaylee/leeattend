//go:build integration

package repository

import (
	"context"
	"os"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
)

func requireTenureIntegrationDatabase(t *testing.T) (*pgxpool.Pool, context.Context) {
	t.Helper()
	if os.Getenv("LEEATTEND_DB_INTEGRATION") != "1" {
		t.Fatal("LEEATTEND_DB_INTEGRATION=1 is required for database integration tests")
	}
	url := os.Getenv("DB_PRIMARY_URL")
	if url == "" {
		url = os.Getenv("DATABASE_URL")
	}
	if url == "" {
		t.Fatal("DB_PRIMARY_URL or DATABASE_URL is required for database integration tests")
	}
	ctx := context.Background()
	pool, err := pgxpool.New(ctx, url)
	if err != nil {
		t.Fatalf("open database pool: %v", err)
	}
	t.Cleanup(pool.Close)
	return pool, ctx
}

func TestIntegrationEmployeeTenuresReuseNumberSequentially(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	firstEmployee, secondEmployee := uuid.New(), uuid.New()
	for _, id := range []uuid.UUID{firstEmployee, secondEmployee} {
		if _, err := tx.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'tenure-integration', DATE '2026-01-01', DATE '2026-01-01')`, id, id.String()); err != nil {
			t.Fatalf("insert employee: %v", err)
		}
	}
	if _, err := tx.Exec(ctx, `INSERT INTO employee_tenures (id, employee_id, employee_number, start_date, end_date) VALUES ($1, $2, '1541', DATE '2026-01-01', DATE '2026-06-30'), ($3, $2, '1541', DATE '2026-07-01', NULL)`, uuid.New(), firstEmployee, uuid.New()); err != nil {
		t.Fatalf("reuse number for one employee: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO employee_tenures (id, employee_id, employee_number, start_date, end_date) VALUES ($1, $2, '1541', DATE '2026-01-01', NULL)`, uuid.New(), secondEmployee); err == nil {
		t.Fatal("expected overlapping number across people to fail")
	}
}

func TestIntegrationEmployeeTenuresKeepHistoricalNumbersDateBound(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	employeeID, relationshipID, employmentID := uuid.New(), uuid.New(), uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, 'current', 'tenure-history-integration', DATE '2026-01-01', DATE '2026-01-01')`, employeeID); err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	employerID := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'tenure-history-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO employee_tenures (id, employee_id, employee_number, start_date, end_date) VALUES ($1, $2, 'old-1541', DATE '2026-01-01', DATE '2026-06-30'), ($3, $2, 'new-1541', DATE '2026-07-01', NULL)`, uuid.New(), employeeID, uuid.New()); err != nil {
		t.Fatalf("insert tenures: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, (SELECT id FROM employee_tenures WHERE employee_id = $2 AND employee_number = 'old-1541'), 'terminated', DATE '2026-01-01')`, relationshipID, employeeID, employerID); err != nil {
		t.Fatalf("insert relationship: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, relationship_id, status, start_date, end_date) VALUES ($1, $2, $3, $4, 'terminated', DATE '2026-01-01', DATE '2026-06-30')`, employmentID, employeeID, employerID, relationshipID); err != nil {
		t.Fatalf("insert employment: %v", err)
	}
	var number string
	if err := tx.QueryRow(ctx, `SELECT t.employee_number FROM employments em JOIN employment_relationships er ON er.id = em.relationship_id JOIN employee_tenures t ON t.id = er.tenure_id WHERE em.id = $1 AND t.start_date <= DATE '2026-06-30' AND (t.end_date IS NULL OR t.end_date >= DATE '2026-06-30')`, employmentID).Scan(&number); err != nil {
		t.Fatalf("read historical number: %v", err)
	}
	if number != "old-1541" {
		t.Fatalf("historical number = %q, want old-1541", number)
	}
}
