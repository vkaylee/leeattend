package repository

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestEmployeeImportQueue_FallbackSyncExecution(t *testing.T) {
	repo := EmployeeImportRepository{}
	q := EmployeeImportQueue{RedisClient: nil, Repo: repo}

	err := q.EnqueueTerminationImport(context.Background(), api.TerminationImportJob{
		JobID:  uuid.New(),
		UserID: uuid.New(),
		Rows: []api.TerminationJobRow{{
			EmploymentID:      uuid.New(),
			TerminationDate:   "2026-06-30",
			TerminationReason: "voluntary_resignation",
		}},
	})
	if err == nil {
		t.Fatalf("expected db not configured error, got nil")
	}
}

func TestEmployeeImport_ProrationCalculation(t *testing.T) {
	oldEntitlement := 12
	daysWorked := 181
	totalDays := 365
	prorated := (oldEntitlement*daysWorked + totalDays/2) / totalDays
	if prorated != 6 {
		t.Fatalf("expected 6, got %d", prorated)
	}

	proratedFull := (oldEntitlement*365 + 365/2) / 365
	if proratedFull != 12 {
		t.Fatalf("expected 12, got %d", proratedFull)
	}

	proratedZero := (oldEntitlement*0 + 365/2) / 365
	if proratedZero != 0 {
		t.Fatalf("expected 0, got %d", proratedZero)
	}

	proratedLeap := (12*183 + 366/2) / 366
	if proratedLeap != 6 {
		t.Fatalf("expected 6, got %d", proratedLeap)
	}
}

func TestEmployeeImport_DateParsing(t *testing.T) {
	layouts := []string{"2006-01-02", "02/01/2006"}
	for _, layout := range layouts {
		sample := "2026-06-30"
		if layout == "02/01/2006" {
			sample = "30/06/2026"
		}
		parsed, err := time.Parse(layout, sample)
		if err != nil || parsed.Year() != 2026 || parsed.Month() != 6 || parsed.Day() != 30 {
			t.Fatalf("failed parsing %s: %v", sample, err)
		}
	}
}

func TestEmployeeImportRepository_TerminationUpdatesRelationship(t *testing.T) {
	employeeID := uuid.New()
	employerID := uuid.New()
	employmentID := uuid.New()
	relationshipID := uuid.New()
	tenureID := uuid.New()
	segmentUpdated := false
	relationshipLocked := false
	relationshipUpdated := false
	tenureClosed := false
	tx := &fakeTx{}
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, _ ...any) pgx.Row {
		switch {
		case strings.Contains(query, "FROM employment_segments"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employeeID
				*(dest[1].(*uuid.UUID)) = employerID
				*(dest[2].(*string)) = "active"
				*(dest[3].(*time.Time)) = time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
				*(dest[4].(*uuid.UUID)) = relationshipID
				return nil
			}}
		case strings.Contains(query, "FROM payroll_periods"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		case strings.Contains(query, "SELECT tenure_id FROM employment_relationships"):
			if !strings.Contains(query, "FOR UPDATE") {
				t.Fatalf("relationship lookup must lock row: %s", query)
			}
			relationshipLocked = true
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = tenureID
				return nil
			}}
		case strings.Contains(query, "SELECT count(*) FROM employment_relationships"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int)) = 0
				return nil
			}}
		case strings.Contains(query, "FROM employee_tenures"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(**uuid.UUID)) = nil
				*(dest[1].(**time.Time)) = nil
				return nil
			}}
		case strings.Contains(query, "SELECT GREATEST"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*string)) = "2026-06-30"
				return nil
			}}
		default:
			t.Fatalf("unexpected query row: %s", query)
			return fakeRow{}
		}
	}
	tx.fakeDBTX.queryFn = func(_ context.Context, query string, _ ...any) (pgx.Rows, error) {
		if !strings.Contains(query, "FROM leave_balances") {
			t.Fatalf("unexpected query: %s", query)
		}
		return &fakeRows{}, nil
	}
	tx.fakeDBTX.execFn = func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
		switch {
		case strings.Contains(query, "UPDATE employment_segments"):
			segmentUpdated = true
		case strings.Contains(query, "UPDATE employment_relationships"):
			relationshipUpdated = true
		case strings.Contains(query, "UPDATE employee_tenures"):
			tenureClosed = true
		}
		return pgconn.NewCommandTag("UPDATE 1"), nil
	}

	repo := EmployeeImportRepository{BeginTx: func(context.Context) (TxBeginner, error) {
		return fakeTxBeginner{tx: tx}, nil
	}}
	count, err := repo.TerminateEmploymentsTransactional(context.Background(), []api.TerminationJobRow{{
		EmploymentID:      employmentID,
		TerminationDate:   "2026-06-30",
		TerminationReason: "voluntary_resignation",
	}})
	if err != nil {
		t.Fatalf("terminate employment: %v", err)
	}
	if count != 1 || !segmentUpdated || !relationshipLocked || !relationshipUpdated || !tenureClosed || !tx.committed {
		t.Fatalf("termination count/segment/relLock/relUpdate/tenureClosed/commit = %d/%t/%t/%t/%t/%t, want 1/true/true/true/true/true", count, segmentUpdated, relationshipLocked, relationshipUpdated, tenureClosed, tx.committed)
	}
}

func TestEmployeeImportRepository_TerminationAcceptsStartDate(t *testing.T) {
	employeeID := uuid.New()
	employerID := uuid.New()
	relationshipID := uuid.New()
	tenureID := uuid.New()
	tx := &fakeTx{}
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, _ ...any) pgx.Row {
		switch {
		case strings.Contains(query, "FROM employment_segments"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employeeID
				*(dest[1].(*uuid.UUID)) = employerID
				*(dest[2].(*string)) = "active"
				*(dest[3].(*time.Time)) = time.Date(2026, 6, 30, 0, 0, 0, 0, time.UTC)
				*(dest[4].(*uuid.UUID)) = relationshipID
				return nil
			}}
		case strings.Contains(query, "FROM payroll_periods"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		case strings.Contains(query, "SELECT tenure_id FROM employment_relationships"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = tenureID
				return nil
			}}
		case strings.Contains(query, "SELECT count(*) FROM employment_relationships"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int)) = 1
				return nil
			}}
		default:
			return fakeRow{scanFn: func(dest ...any) error {
				return nil
			}}
		}
	}
	tx.fakeDBTX.queryFn = func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
		return &fakeRows{}, nil
	}

	repo := EmployeeImportRepository{BeginTx: func(context.Context) (TxBeginner, error) {
		return fakeTxBeginner{tx: tx}, nil
	}}
	count, err := repo.TerminateEmploymentsTransactional(context.Background(), []api.TerminationJobRow{{
		EmploymentID:    uuid.New(),
		TerminationDate: "2026-06-30",
	}})
	if err != nil || count != 1 || !tx.committed {
		t.Fatalf("same-day termination count/error/commit = %d/%v/%t, want 1/nil/true", count, err, tx.committed)
	}
}

func TestEmployeeImportRepository_TerminationRejectsDateBeforeStart(t *testing.T) {
	tx := &fakeTx{}
	segmentUpdated := false
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, _ ...any) pgx.Row {
		if !strings.Contains(query, "FROM employment_segments") {
			t.Fatalf("unexpected query after invalid date: %s", query)
		}
		return fakeRow{scanFn: func(dest ...any) error {
			*(dest[0].(*uuid.UUID)) = uuid.New()
			*(dest[1].(*uuid.UUID)) = uuid.New()
			*(dest[2].(*string)) = "active"
			*(dest[3].(*time.Time)) = time.Date(2026, 6, 1, 0, 0, 0, 0, time.UTC)
			*(dest[4].(*uuid.UUID)) = uuid.New()
			return nil
		}}
	}
	tx.fakeDBTX.execFn = func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
		if strings.Contains(query, "UPDATE employment_segments") {
			segmentUpdated = true
		}
		return pgconn.NewCommandTag("UPDATE 1"), nil
	}

	repo := EmployeeImportRepository{BeginTx: func(context.Context) (TxBeginner, error) {
		return fakeTxBeginner{tx: tx}, nil
	}}
	count, err := repo.TerminateEmploymentsTransactional(context.Background(), []api.TerminationJobRow{{
		EmploymentID:    uuid.New(),
		TerminationDate: "2026-05-31",
	}})
	if err == nil || !strings.Contains(err.Error(), "precedes employment start date") {
		t.Fatalf("invalid termination error = %v, want start-date validation error", err)
	}
	if count != 0 || segmentUpdated || tx.committed {
		t.Fatalf("invalid termination count/segment update/commit = %d/%t/%t, want 0/false/false", count, segmentUpdated, tx.committed)
	}
}

func TestEmployeeImportRepository_TerminationRollsBackWhenRelationshipUpdateFails(t *testing.T) {
	employeeID := uuid.New()
	employerID := uuid.New()
	relationshipID := uuid.New()
	tenureID := uuid.New()
	relationshipErr := errors.New("relationship update failed")
	tx := &fakeTx{}
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, _ ...any) pgx.Row {
		switch {
		case strings.Contains(query, "FROM employment_segments"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employeeID
				*(dest[1].(*uuid.UUID)) = employerID
				*(dest[2].(*string)) = "active"
				*(dest[3].(*time.Time)) = time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
				*(dest[4].(*uuid.UUID)) = relationshipID
				return nil
			}}
		case strings.Contains(query, "FROM payroll_periods"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		case strings.Contains(query, "FROM employment_relationships"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = tenureID
				return nil
			}}
		default:
			t.Fatalf("unexpected query row: %s", query)
			return fakeRow{}
		}
	}
	tx.fakeDBTX.queryFn = func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
		return &fakeRows{}, nil
	}
	tx.fakeDBTX.execFn = func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
		if strings.Contains(query, "UPDATE employment_relationships") {
			return pgconn.CommandTag{}, relationshipErr
		}
		return pgconn.NewCommandTag("UPDATE 1"), nil
	}

	repo := EmployeeImportRepository{BeginTx: func(context.Context) (TxBeginner, error) {
		return fakeTxBeginner{tx: tx}, nil
	}}
	count, err := repo.TerminateEmploymentsTransactional(context.Background(), []api.TerminationJobRow{{
		EmploymentID:    uuid.New(),
		TerminationDate: "2026-06-30",
	}})
	if !errors.Is(err, relationshipErr) {
		t.Fatalf("termination error = %v, want %v", err, relationshipErr)
	}
	if count != 0 || tx.committed {
		t.Fatalf("failed termination count/commit = %d/%t, want 0/false", count, tx.committed)
	}
}
