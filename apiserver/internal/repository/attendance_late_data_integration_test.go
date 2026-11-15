//go:build integration

package repository

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestIntegrationRawPunchIngestRemainsAllowedAfterClosedAttendanceSnapshot(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID := uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'late-data-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'late-data-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String()); err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID); err != nil {
		t.Fatalf("insert employment: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO attendance_periods (id, employer_id, period_year, period_month, status, version) VALUES ($1, $2, 2026, 11, 'closed', 2)`, uuid.New(), employerID); err != nil {
		t.Fatalf("insert closed attendance period: %v", err)
	}
	punch := time.Date(2026, 11, 20, 9, 0, 0, 0, time.UTC)
	if _, err := pool.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'import', '{}'::jsonb, 1, 0, NOW())`, uuid.New(), employmentID, punch); err != nil {
		t.Fatalf("late raw punch rejected: %v", err)
	}
	var count int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM raw_attendance_logs WHERE employment_id = $1 AND punch_time = $2`, employmentID, punch).Scan(&count); err != nil {
		t.Fatalf("count late raw punch: %v", err)
	}
	if count != 1 {
		t.Fatalf("late raw punch count = %d, want 1", count)
	}
}
