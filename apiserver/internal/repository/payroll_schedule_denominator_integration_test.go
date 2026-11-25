//go:build integration

package repository

import (
	"testing"

	"github.com/google/uuid"
)

func TestIntegrationLoadScheduledWorkdayCountsUsesCanonicalWorkSegments(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	employerID := uuid.New()
	employeeID := uuid.New()
	employmentID := uuid.New()
	templateID := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'schedule-denominator-test-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	seedIntegrationEmployeeGraphWithRange(t, ctx, tx, employeeID, employerID, employmentID, employeeID.String(), "schedule-denominator-test-employee", "2026-01-01", "2026-01-01", "")
	if _, err := tx.Exec(ctx, `INSERT INTO shift_templates (id, name) VALUES ($1, 'schedule-denominator-test-template')`, templateID); err != nil {
		t.Fatalf("insert shift template: %v", err)
	}

	sortOrder := 0
	insertSegment := func(date, status string) {
		t.Helper()
		sortOrder++
		_, err := tx.Exec(ctx, `INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, template_id) VALUES ($1, $2::date, $3, '08:00', '12:00', 'work', $4::segment_status_type, $5)`, employmentID, date, sortOrder, status, templateID)
		if err != nil {
			t.Fatalf("insert work segment: %v", err)
		}
	}
	insertSegment("2026-01-05", "scheduled")
	insertSegment("2026-01-05", "scheduled")
	insertSegment("2026-01-06", "completed")
	insertSegment("2026-01-07", "cancelled")

	counts, err := loadScheduledWorkdayCounts(ctx, tx, employmentID, "2026-01-01", "2026-01-31")
	if err != nil {
		t.Fatalf("load scheduled workday counts: %v", err)
	}
	if len(counts) != 2 || counts["2026-01-05"] != 1 || counts["2026-01-06"] != 1 {
		t.Fatalf("unexpected scheduled workday counts: %#v", counts)
	}
	if _, exists := counts["2026-01-07"]; exists {
		t.Fatalf("cancelled work segment must not count: %#v", counts)
	}
}
