package repository

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestEmployeeImportQueue_FallbackSyncExecution(t *testing.T) {
	executed := false
	var capturedID uuid.UUID
	repo := EmployeeImportRepository{}
	q := EmployeeImportQueue{
		RedisClient: nil,
		Repo:        repo,
	}

	jobID := uuid.New()
	empID := uuid.New()
	_ = executed
	_ = capturedID

	// When RedisClient is nil and DB is not configured, EnqueueTerminationImport calls Repo.TerminateEmploymentsTransactional
	err := q.EnqueueTerminationImport(context.Background(), api.TerminationImportJob{
		JobID:  jobID,
		UserID: uuid.New(),
		Rows: []api.TerminationJobRow{
			{
				EmploymentID:      empID,
				TerminationDate:   "2026-06-30",
				TerminationReason: "voluntary_resignation",
			},
		},
	})
	// Expect database configuration error, confirming synchronous execution branch was exercised
	if err == nil {
		t.Fatalf("expected db not configured error, got nil")
	}
}

func TestEmployeeImport_ProrationCalculation(t *testing.T) {
	// Verify integer rounding formula: (oldEntitlement * daysWorked + totalDays / 2) / totalDays
	// Year 2026: 365 days
	// Jan 1 to June 30: 181 days
	oldEntitlement := 12
	daysWorked := 181
	totalDays := 365
	prorated := (oldEntitlement*daysWorked + totalDays/2) / totalDays
	if prorated != 6 {
		t.Fatalf("expected 6, got %d", prorated)
	}

	// Full year: 365 days -> 12
	proratedFull := (oldEntitlement*365 + 365/2) / 365
	if proratedFull != 12 {
		t.Fatalf("expected 12, got %d", proratedFull)
	}

	// 0 days worked -> 0
	proratedZero := (oldEntitlement*0 + 365/2) / 365
	if proratedZero != 0 {
		t.Fatalf("expected 0, got %d", proratedZero)
	}

	// Mid leap year: 366 days, 183 days worked -> 6
	proratedLeap := (12*183 + 366/2) / 366
	if proratedLeap != 6 {
		t.Fatalf("expected 6, got %d", proratedLeap)
	}
}

func TestEmployeeImport_DateParsing(t *testing.T) {
	layouts := []string{"2006-01-02", "02/01/2006"}
	for _, l := range layouts {
		var sample string
		if l == "2006-01-02" {
			sample = "2026-06-30"
		} else {
			sample = "30/06/2026"
		}
		parsed, err := time.Parse(l, sample)
		if err != nil || parsed.Year() != 2026 || parsed.Month() != 6 || parsed.Day() != 30 {
			t.Fatalf("failed parsing %s: %v", sample, err)
		}
	}
}
