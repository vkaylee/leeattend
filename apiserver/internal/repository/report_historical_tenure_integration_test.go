//go:build integration

package repository

import (
	"encoding/json"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationAttendanceReportKeepsHistoricalTenureNumber(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	employerID, employeeID, employmentID := uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'report-tenure-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, 'current-1541', 'report-tenure-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID); err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	oldTenure, newTenure := uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employee_tenures (id, employee_id, employee_number, start_date, end_date) VALUES ($1, $2, 'old-1541', DATE '2026-01-01', DATE '2026-06-30'), ($3, $2, 'new-1541', DATE '2026-07-01', NULL)`, oldTenure, employeeID, newTenure); err != nil {
		t.Fatalf("insert tenures: %v", err)
	}
	relationshipID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'terminated', DATE '2026-01-01')`, relationshipID, employeeID, employerID, oldTenure); err != nil {
		t.Fatalf("insert relationship: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, relationship_id, status, start_date, end_date) VALUES ($1, $2, $3, $4, 'terminated', DATE '2026-01-01', DATE '2026-06-30')`, employmentID, employeeID, employerID, relationshipID); err != nil {
		t.Fatalf("insert employment: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO attendance_daily_summaries (id, employment_id, date, actual_work_minutes, standard_minutes) VALUES ($1, $2, DATE '2026-06-15', 480, 480)`, uuid.New(), employmentID); err != nil {
		t.Fatalf("insert attendance summary: %v", err)
	}
	report, err := (ReportRepository{DB: pool}).GetAttendanceReport(ctx, api.AttendanceReportFilter{ReportType: "daily", StartDate: "2026-06-15", EndDate: "2026-06-15", EmployeeIDs: []uuid.UUID{employeeID}, Page: 1, Limit: 20})
	if err != nil {
		t.Fatalf("attendance report: %v", err)
	}
	if report.Total != 1 {
		t.Fatalf("report total = %d, want 1", report.Total)
	}
	var rows []api.DailyAttendanceRow
	if err := json.Unmarshal(report.Data, &rows); err != nil {
		t.Fatalf("decode report: %v", err)
	}
	if len(rows) != 1 || rows[0].EmployeeIDNumber != "old-1541" {
		t.Fatalf("historical report number = %+v, want old-1541", rows)
	}
}
