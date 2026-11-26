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
	if _, err := pool.Exec(ctx, `INSERT INTO employees (id, full_name) VALUES ($1, 'report-tenure-employee')`, employeeID); err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	oldTenure, newTenure := uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employee_tenures (id, employee_id, employee_number, start_date, end_date) VALUES ($1, $2, 'old-1541', DATE '2026-01-01', DATE '2026-06-30'), ($3, $2, 'new-1541', DATE '2026-07-01', NULL)`, oldTenure, employeeID, newTenure); err != nil {
		t.Fatalf("insert tenures: %v", err)
	}
	relationshipID, newRelationshipID := uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date, end_date) VALUES ($1, $2, $3, $4, 'terminated', DATE '2026-01-01', DATE '2026-06-30'), ($5, $2, $3, $6, 'active', DATE '2026-07-01', NULL)`, relationshipID, employeeID, employerID, oldTenure, newRelationshipID, newTenure); err != nil {
		t.Fatalf("insert relationships: %v", err)
	}
	newEmploymentID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employment_segments (id, employee_id, employer_id, relationship_id, status, start_date, end_date) VALUES ($1, $2, $3, $4, 'terminated', DATE '2026-01-01', DATE '2026-06-30'), ($5, $2, $3, $6, 'active', DATE '2026-07-01', NULL)`, employmentID, employeeID, employerID, relationshipID, newEmploymentID, newRelationshipID); err != nil {
		t.Fatalf("insert employments: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO attendance_daily_summaries (id, employment_id, date, actual_work_minutes, standard_minutes) VALUES ($1, $2, DATE '2026-06-15', 480, 480), ($3, $4, DATE '2026-07-15', 480, 480)`, uuid.New(), employmentID, uuid.New(), newEmploymentID); err != nil {
		t.Fatalf("insert attendance summaries: %v", err)
	}
	report, err := (ReportRepository{DB: pool}).GetAttendanceReport(ctx, api.AttendanceReportFilter{ReportType: "daily", StartDate: "2026-06-15", EndDate: "2026-07-15", EmployeeIDs: []uuid.UUID{employeeID}, Page: 1, Limit: 20})
	if err != nil {
		t.Fatalf("attendance report: %v", err)
	}
	if report.Total != 2 {
		t.Fatalf("report total = %d, want 2", report.Total)
	}
	var rows []api.DailyAttendanceRow
	if err := json.Unmarshal(report.Data, &rows); err != nil {
		t.Fatalf("decode report: %v", err)
	}
	if len(rows) != 2 || rows[0].EmployeeIDNumber != "old-1541" || rows[1].EmployeeIDNumber != "new-1541" {
		t.Fatalf("historical report numbers = %+v, want old-1541 then new-1541", rows)
	}
}
