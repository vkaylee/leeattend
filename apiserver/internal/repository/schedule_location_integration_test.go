//go:build integration

package repository

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestIntegrationResolverUsesAssignedLocationTimezoneAndHolidayOverlay(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, locationID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'timezone-resolver-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'timezone-resolver-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String()); err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-11-01')`, employmentID, employeeID, employerID); err != nil {
		t.Fatalf("insert employment: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO work_locations (id, name, timezone, status) VALUES ($1, 'timezone-resolver-location', 'America/New_York', 'active')`, locationID); err != nil {
		t.Fatalf("insert location: %v", err)
	}
	if _, err := pool.Exec(ctx, `UPDATE employments SET primary_location_id = $2 WHERE id = $1`, employmentID, locationID); err != nil {
		t.Fatalf("assign location: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO holidays (id, name, date, is_paid) VALUES ($1, 'Resolver holiday', DATE '2026-11-16', true)`, uuid.New()); err != nil {
		t.Fatalf("insert holiday: %v", err)
	}
	start := time.Date(2026, 11, 16, 0, 0, 0, 0, time.UTC)
	resolution, _, err := resolveScheduleData(ctx, pool, start, start, []uuid.UUID{employmentID}, 1, 50)
	if err != nil {
		t.Fatalf("resolve schedule data: %v", err)
	}
	if len(resolution.employees) != 1 || len(resolution.employees[0].days) != 1 {
		t.Fatalf("resolved employees/days = %d/%d", len(resolution.employees), len(resolution.employees[0].days))
	}
	day := resolution.employees[0].days[0]
	if day.timezone != "America/New_York" || !day.isHoliday || !day.holidayPaid || day.holidayName == nil || *day.holidayName != "Resolver holiday" {
		t.Fatalf("unexpected timezone/holiday overlay: %+v", day)
	}
}
