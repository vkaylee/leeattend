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

type payrollPeriodTestTx struct {
	fakeDBTX
	committed  bool
	rolledBack bool
}

func (tx *payrollPeriodTestTx) Commit(context.Context) error          { tx.committed = true; return nil }
func (tx *payrollPeriodTestTx) Rollback(context.Context) error        { tx.rolledBack = true; return nil }
func (tx *payrollPeriodTestTx) Begin(context.Context) (pgx.Tx, error) { return tx, nil }
func (tx *payrollPeriodTestTx) CopyFrom(context.Context, pgx.Identifier, []string, pgx.CopyFromSource) (int64, error) {
	return 0, nil
}
func (tx *payrollPeriodTestTx) SendBatch(context.Context, *pgx.Batch) pgx.BatchResults { return nil }
func (tx *payrollPeriodTestTx) LargeObjects() pgx.LargeObjects                         { return pgx.LargeObjects{} }
func (tx *payrollPeriodTestTx) Prepare(context.Context, string, string) (*pgconn.StatementDescription, error) {
	return nil, nil
}
func (tx *payrollPeriodTestTx) Conn() *pgx.Conn { return nil }

func TestPayrollRepositoryLockPeriodAuditsAtomically(t *testing.T) {
	employerID := uuid.New()
	periodID := uuid.New()
	actorID := uuid.New()
	tx := &payrollPeriodTestTx{}
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, _ ...any) pgx.Row {
		switch {
		case strings.Contains(query, "FROM employers"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
		case strings.Contains(query, "attendance_periods"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*string)) = "closed"; return nil }}
		case strings.Contains(query, "FOR UPDATE"):
			return fakeRow{scanFn: payrollPeriodScan(periodID, employerID, 1, "open")}
		case strings.Contains(query, "UPDATE payroll_periods"):
			return fakeRow{scanFn: payrollPeriodScan(periodID, employerID, 2, "locked")}
		default:
			return fakeRow{}
		}
	}
	tx.fakeDBTX.execFn = func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
		if strings.Contains(query, "audit_logs") {
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		}
		return pgconn.NewCommandTag("INSERT 0 1"), nil
	}
	repo := PayrollRepository{DB: tx}
	period, err := repo.LockPayrollPeriod(context.Background(), employerID, 2026, 1, 1, actorID)
	if err != nil {
		t.Fatalf("lock payroll period: %v", err)
	}
	if period.Status != "locked" || period.Version != 2 || !tx.committed || tx.rolledBack {
		t.Fatalf("period/transaction = %s/%d/%t/%t, want locked/2/true/false", period.Status, period.Version, tx.committed, tx.rolledBack)
	}
}

func TestPayrollRepositoryLockPeriodRequiresClosedAttendance(t *testing.T) {
	tx := &payrollPeriodTestTx{}
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, _ ...any) pgx.Row {
		switch {
		case strings.Contains(query, "FROM employers"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
		case strings.Contains(query, "attendance_periods"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*string)) = "open"; return nil }}
		default:
			return fakeRow{scanFn: payrollPeriodScan(uuid.New(), uuid.New(), 1, "open")}
		}
	}
	tx.fakeDBTX.execFn = func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
		return pgconn.NewCommandTag("INSERT 0 0"), nil
	}
	_, err := (PayrollRepository{DB: tx}).LockPayrollPeriod(context.Background(), uuid.New(), 2026, 1, 1, uuid.New())
	if !errors.Is(err, api.ErrPayrollPeriodConflict) || tx.committed || !tx.rolledBack {
		t.Fatalf("error/transaction = %v/%t/%t, want attendance conflict/false/true", err, tx.committed, tx.rolledBack)
	}
}

func TestPayrollRepositoryPeriodConflictDoesNotCommit(t *testing.T) {
	tx := &payrollPeriodTestTx{}
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, _ ...any) pgx.Row {
		if strings.Contains(query, "FROM employers") {
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
		}
		return fakeRow{scanFn: payrollPeriodScan(uuid.New(), uuid.New(), 2, "open")}
	}
	tx.fakeDBTX.execFn = func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
		return pgconn.NewCommandTag("INSERT 0 0"), nil
	}
	_, err := (PayrollRepository{DB: tx}).LockPayrollPeriod(context.Background(), uuid.New(), 2026, 1, 1, uuid.New())
	if !errors.Is(err, api.ErrPayrollPeriodConflict) || tx.committed || !tx.rolledBack {
		t.Fatalf("error/transaction = %v/%t/%t, want period conflict/false/true", err, tx.committed, tx.rolledBack)
	}
}

func payrollPeriodScan(id, employerID uuid.UUID, version int64, status string) func(...any) error {
	return func(dest ...any) error {
		*(dest[0].(*uuid.UUID)) = id
		*(dest[1].(*uuid.UUID)) = employerID
		*(dest[2].(*int)) = 2026
		*(dest[3].(*int)) = 1
		*(dest[4].(*string)) = status
		*(dest[5].(*int64)) = version
		*(dest[6].(**time.Time)) = nil
		*(dest[7].(**uuid.UUID)) = nil
		*(dest[8].(**time.Time)) = nil
		*(dest[9].(**uuid.UUID)) = nil
		*(dest[10].(**string)) = nil
		return nil
	}
}
