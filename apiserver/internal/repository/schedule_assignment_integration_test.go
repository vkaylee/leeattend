//go:build integration

package repository

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestIntegrationResolverLoadsPatternAndExplicitOffRoster(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID := uuid.New(), uuid.New(), uuid.New()
	patternID, templateID, versionID := uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'assignment-resolver-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'assignment-resolver-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String()); err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, status, start_date, end_date) VALUES ($1, $2, $3, 'active', DATE '2026-11-01', DATE '2026-11-30')`, employmentID, employeeID, employerID); err != nil {
		t.Fatalf("insert employment: %v", err)
	}
	actorID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert actor: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO working_patterns (id, name, pattern_type) VALUES ($1, 'weekly resolver pattern', 'weekly')`, patternID); err != nil {
		t.Fatalf("insert pattern: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_templates (id, name) VALUES ($1, 'weekly resolver shift')`, templateID); err != nil {
		t.Fatalf("insert template: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'weekly resolver version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID); err != nil {
		t.Fatalf("insert version: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, '08:00', '17:00')`, versionID); err != nil {
		t.Fatalf("insert interval: %v", err)
	}
	monday := time.Date(2026, 11, 16, 0, 0, 0, 0, time.UTC)
	if _, err := pool.Exec(ctx, `INSERT INTO working_pattern_details (id, pattern_id, schedule_day, template_id) VALUES (gen_random_uuid(), $1, 1, $2)`, patternID, templateID); err != nil {
		t.Fatalf("insert pattern detail: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employee_working_patterns (id, employment_id, pattern_id, start_date, end_date, anchor_date) VALUES (gen_random_uuid(), $1, $2, DATE '2026-11-01', DATE '2026-11-30', DATE '2026-11-01')`, employmentID, patternID); err != nil {
		t.Fatalf("assign pattern: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_rosters (id, employment_id, template_id, date, status) VALUES (gen_random_uuid(), $1, NULL, DATE '2026-11-17', 'confirmed')`, employmentID); err != nil {
		t.Fatalf("insert explicit off roster: %v", err)
	}
	resolution, _, err := resolveScheduleData(ctx, pool, monday, monday.AddDate(0, 0, 1), []uuid.UUID{employmentID}, 1, 50)
	if err != nil {
		t.Fatalf("resolve schedule data: %v", err)
	}
	if len(resolution.employees) != 1 || len(resolution.employees[0].days) != 2 {
		t.Fatalf("resolved employees/days = %d/%d", len(resolution.employees), len(resolution.employees[0].days))
	}
	if resolution.employees[0].days[0].source != "pattern" || len(resolution.employees[0].days[0].shiftIDs) != 1 || resolution.employees[0].days[0].shiftIDs[0] == nil {
		t.Fatalf("Monday did not resolve pattern work: %+v", resolution.employees[0].days[0])
	}
	if resolution.employees[0].days[1].source != "roster" || resolution.employees[0].days[1].shiftIDs[0] != nil {
		t.Fatalf("Tuesday did not resolve explicit off: %+v", resolution.employees[0].days[1])
	}
}
