package repository

import (
	"bytes"
	"context"
	"errors"
	"log/slog"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestInsertApprovedAdjustmentBlocksOfficialDate(t *testing.T) {
	calls := 0
	adjustmentExecCalls := 0
	evidenceWrites := 0
	db := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			calls++
			return fakeRow{scanFn: func(dest ...any) error {
				if calls == 1 {
					for _, value := range dest {
						*(value.(*bool)) = true
					}
					return nil
				}
				*(dest[0].(*bool)) = true
				return nil
			}}
		},
		execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(query, "schedule_legacy_mutation_evidence") {
				evidenceWrites++
				return pgconn.NewCommandTag("INSERT 1"), nil
			}
			adjustmentExecCalls++
			return pgconn.NewCommandTag("INSERT 1"), nil
		},
	}}

	err := insertApprovedAdjustment(context.Background(), db, uuid.New(), uuid.New(), api.AttendanceImportRow{
		PunchTime: time.Date(2026, 11, 13, 9, 0, 0, 0, time.UTC),
	})
	if !errors.Is(err, ErrLifecycleMutationBlocked) {
		t.Fatalf("expected official date mutation block, got %v", err)
	}
	if adjustmentExecCalls != 0 {
		t.Fatalf("expected no adjustment insert after lifecycle block, got %d adjustment exec calls", adjustmentExecCalls)
	}
	if evidenceWrites != 1 || !db.committed {
		t.Fatalf("expected committed evidence write, got writes=%d committed=%v", evidenceWrites, db.committed)
	}
}

func TestInsertApprovedAdjustmentFailsClosedWithoutTxBeginner(t *testing.T) {
	calls := 0
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			calls++
			return fakeRow{scanFn: func(dest ...any) error {
				if calls == 1 {
					for _, value := range dest {
						*(value.(*bool)) = true
					}
					return nil
				}
				*(dest[0].(*bool)) = true
				return nil
			}}
		},
	}

	err := insertApprovedAdjustment(context.Background(), db, uuid.New(), uuid.New(), api.AttendanceImportRow{
		PunchTime: time.Date(2026, 11, 13, 9, 0, 0, 0, time.UTC),
	})
	if !errors.Is(err, ErrLifecycleEvidenceFailed) || errors.Is(err, ErrLifecycleMutationBlocked) {
		t.Fatalf("expected ErrLifecycleEvidenceFailed without TxBeginner, got %v", err)
	}
}

func TestEnterPaperOTBlocksClosedDate(t *testing.T) {
	employmentID := uuid.New()
	calls := 0
	evidenceWrites := 0
	db := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			calls++
			return fakeRow{scanFn: func(dest ...any) error {
				switch {
				case strings.Contains(query, "FROM employment_segments"):
					*(dest[0].(*uuid.UUID)) = employmentID
				case calls == 2:
					for _, value := range dest {
						*(value.(*bool)) = true
					}
				case calls == 3:
					*(dest[0].(*bool)) = false
				case calls == 4:
					*(dest[0].(*bool)) = true
				case calls == 5:
					*(dest[0].(*bool)) = false
				}
				return nil
			}}
		},
		execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(query, "schedule_legacy_mutation_evidence") {
				evidenceWrites++
			}
			return pgconn.NewCommandTag("INSERT 1"), nil
		},
	}}

	_, err := (AttendanceRepository{DB: db}).EnterPaperOT(context.Background(), uuid.New(), api.PaperOTRequest{
		Date:           "2026-11-13",
		ClaimedMinutes: 60,
		Reason:         "paper form",
	}, uuid.New())
	if !errors.Is(err, ErrLifecycleMutationBlocked) {
		t.Fatalf("expected closed date mutation block, got %v", err)
	}
	if evidenceWrites != 1 || !db.committed {
		t.Fatalf("expected committed evidence write, got writes=%d committed=%v", evidenceWrites, db.committed)
	}
}

func TestBulkPaperOTMasksDBUnavailable(t *testing.T) {
	row := api.BulkPaperOTRow{LineNumber: 2, EmployeeCode: "EMP001", Date: "2026-11-13", ClaimedMinutes: 60, Reason: "overtime"}
	res := (AttendanceRepository{}).BulkPaperOT(context.Background(), []api.BulkPaperOTRow{row}, uuid.New())
	if res.FailedCount != 1 || len(res.Errors) != 1 || res.Errors[0].ErrorMessage != "internal error" {
		t.Fatalf("nil DB bulk errors = %+v, want masked internal error", res.Errors)
	}
}

func TestBulkPaperOTMasksUpsertFailure(t *testing.T) {
	employmentID := uuid.New()
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error {
			switch {
			case strings.Contains(query, "WITH candidates AS"):
				*(dest[0].(*int)) = 1
				*(dest[1].(**uuid.UUID)) = &employmentID
			case strings.Contains(query, "information_schema.tables"):
				for _, v := range dest {
					*(v.(*bool)) = false
				}
			default:
				return errors.New("secret upsert constraint sql error")
			}
			return nil
		}}
	}}
	row := api.BulkPaperOTRow{LineNumber: 2, EmployeeCode: "EMP001", Date: "2026-11-13", ClaimedMinutes: 60, Reason: "overtime"}
	res := (AttendanceRepository{DB: db}).BulkPaperOT(context.Background(), []api.BulkPaperOTRow{row}, uuid.New())
	if res.FailedCount != 1 || len(res.Errors) != 1 || res.Errors[0].ErrorMessage != "internal error" {
		t.Fatalf("upsert failure bulk errors = %+v, want masked internal error", res.Errors)
	}
}

func newImportBlockTrackingDB(employmentID uuid.UUID, updatedStatus *string, updatedErrorLog **string, evidenceWrites *int) *fakeTx {
	return &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				switch {
				case strings.Contains(query, "WITH candidates AS"):
					*(dest[0].(*int)) = 1
					*(dest[1].(**uuid.UUID)) = &employmentID
				case strings.Contains(query, "information_schema.tables"):
					for _, v := range dest {
						*(v.(*bool)) = true
					}
				default:
					*(dest[0].(*bool)) = true
				}
				return nil
			}}
		},
		execFn: func(_ context.Context, query string, args ...any) (pgconn.CommandTag, error) {
			if strings.Contains(query, "schedule_legacy_mutation_evidence") {
				*evidenceWrites++
				return pgconn.NewCommandTag("INSERT 1"), nil
			}
			if strings.Contains(query, "UPDATE attendance_import_jobs") {
				*updatedStatus = args[1].(string)
				if s, ok := args[4].(*string); ok {
					*updatedErrorLog = s
				}
				return pgconn.NewCommandTag("UPDATE 1"), nil
			}
			return pgconn.NewCommandTag("INSERT 1"), nil
		},
	}}
}

func TestProcessAttendanceImportMarksJobFailedOnLifecycleBlock(t *testing.T) {
	var updatedStatus string
	var updatedErrorLog *string
	evidenceWrites := 0
	db := newImportBlockTrackingDB(uuid.New(), &updatedStatus, &updatedErrorLog, &evidenceWrites)

	err := (AttendanceRepository{DB: db}).ProcessAttendanceImport(context.Background(), uuid.New(), uuid.New(), "/tmp/import.xlsx", "adjustment", []api.AttendanceImportRow{{
		LineNumber:   2,
		EmployeeCode: "EMP001",
		PunchTime:    time.Date(2026, 11, 13, 9, 0, 0, 0, time.UTC),
	}})
	if !errors.Is(err, ErrLifecycleMutationBlocked) || evidenceWrites != 1 || !db.committed {
		t.Fatalf("err=%v writes=%d committed=%v", err, evidenceWrites, db.committed)
	}
	if updatedStatus != "failed" || updatedErrorLog == nil || *updatedErrorLog != ErrLifecycleMutationBlocked.Error() {
		t.Fatalf("job update status=%q errorLog=%v, want failed / %q", updatedStatus, updatedErrorLog, ErrLifecycleMutationBlocked.Error())
	}
}

func TestBulkPaperOTLookupDBErrorReturnsMaskedInternal(t *testing.T) {
	dbErr := &pgconn.PgError{Code: "08006", Message: "connection failed", Detail: "secret-pii@example.com"}
	var logBuf bytes.Buffer
	prev := slog.Default()
	slog.SetDefault(slog.New(slog.NewTextHandler(&logBuf, nil)))
	t.Cleanup(func() { slog.SetDefault(prev) })

	db := &fakeDBTX{queryRowFn: func(context.Context, string, ...any) pgx.Row {
		return fakeRow{scanFn: func(...any) error { return dbErr }}
	}}
	res := (AttendanceRepository{DB: db}).BulkPaperOT(context.Background(), []api.BulkPaperOTRow{{
		LineNumber: 2, EmployeeCode: "EMP001", Date: "2026-11-13", ClaimedMinutes: 60, Reason: "overtime",
	}}, uuid.New())
	if res.FailedCount != 1 || len(res.Errors) != 1 || res.Errors[0].ErrorMessage != "internal error" {
		t.Fatalf("lookup DB error bulk result = %+v, want masked internal error", res.Errors)
	}
	if strings.Contains(logBuf.String(), "secret-pii@example.com") || !strings.Contains(logBuf.String(), "pg:08006") {
		t.Fatalf("log output leaked PII or missed pg code: %q", logBuf.String())
	}
}

func TestProcessAttendanceImportLookupDBErrorMarksJobFailedEvenOnCanceledCtx(t *testing.T) {
	dbErr := &pgconn.PgError{Code: "57014", Message: "query canceled", Detail: "secret-pii@example.com"}
	var updatedStatus string
	var updateCtxErr error
	var logBuf bytes.Buffer
	prev := slog.Default()
	slog.SetDefault(slog.New(slog.NewTextHandler(&logBuf, nil)))
	t.Cleanup(func() { slog.SetDefault(prev) })

	db := &fakeDBTX{
		queryRowFn: func(context.Context, string, ...any) pgx.Row {
			return fakeRow{scanFn: func(...any) error { return dbErr }}
		},
		execFn: func(ctx context.Context, query string, args ...any) (pgconn.CommandTag, error) {
			if strings.Contains(query, "UPDATE attendance_import_jobs") {
				updateCtxErr = ctx.Err()
				updatedStatus = args[1].(string)
				return pgconn.NewCommandTag("UPDATE 1"), nil
			}
			return pgconn.NewCommandTag("INSERT 1"), nil
		},
	}
	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	err := (AttendanceRepository{DB: db}).ProcessAttendanceImport(ctx, uuid.New(), uuid.New(), "/tmp/import.xlsx", "adjustment", []api.AttendanceImportRow{{
		LineNumber: 2, EmployeeCode: "EMP001", PunchTime: time.Date(2026, 11, 13, 9, 0, 0, 0, time.UTC),
	}})
	if !errors.Is(err, dbErr) || updatedStatus != "failed" || updateCtxErr != nil {
		t.Fatalf("err=%v status=%q updateCtxErr=%v", err, updatedStatus, updateCtxErr)
	}
	if strings.Contains(logBuf.String(), "secret-pii@example.com") || !strings.Contains(logBuf.String(), "pg:57014") {
		t.Fatalf("log output leaked PII or missed pg code: %q", logBuf.String())
	}
}

func TestBulkPaperOTMasksUnexpectedGuardError(t *testing.T) {
	employmentID := uuid.New()
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error {
			if strings.Contains(query, "WITH candidates AS") {
				*(dest[0].(*int)) = 1
				*(dest[1].(**uuid.UUID)) = &employmentID
				return nil
			}
			return errors.New("secret database detail")
		}}
	}}
	result := (AttendanceRepository{DB: db}).BulkPaperOT(context.Background(), []api.BulkPaperOTRow{{
		LineNumber: 2, EmployeeCode: "EMP001", Date: "2026-11-13", ClaimedMinutes: 60, Reason: "overtime",
	}}, uuid.New())
	if len(result.Errors) != 1 || strings.Contains(result.Errors[0].ErrorMessage, "secret") {
		t.Fatalf("unexpected bulk errors: %+v", result.Errors)
	}
}

func TestBulkPaperOTMasksEvidenceWriteFailure(t *testing.T) {
	employmentID := uuid.New()
	secretErr := errors.New("secret pg credential leak in evidence insert")
	db := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				switch {
				case strings.Contains(query, "WITH candidates AS"):
					*(dest[0].(*int)) = 1
					*(dest[1].(**uuid.UUID)) = &employmentID
				case strings.Contains(query, "information_schema.tables"):
					for _, value := range dest {
						*(value.(*bool)) = true
					}
				default:
					*(dest[0].(*bool)) = true
				}
				return nil
			}}
		},
		execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(query, "schedule_legacy_mutation_evidence") {
				return pgconn.CommandTag{}, secretErr
			}
			return pgconn.NewCommandTag("INSERT 1"), nil
		},
	}}

	res := (AttendanceRepository{DB: db}).BulkPaperOT(context.Background(), []api.BulkPaperOTRow{{
		LineNumber:     2,
		EmployeeCode:   "EMP001",
		Date:           "2026-11-13",
		ClaimedMinutes: 60,
		Reason:         "overtime",
	}}, uuid.New())

	if res.FailedCount != 1 || len(res.Errors) != 1 {
		t.Fatalf("expected 1 failed row, got %+v", res)
	}
	if strings.Contains(res.Errors[0].ErrorMessage, "secret") {
		t.Fatalf("bulk row error leaked internal evidence failure: %q", res.Errors[0].ErrorMessage)
	}
	if res.Errors[0].ErrorMessage != ErrLifecycleEvidenceFailed.Error() {
		t.Fatalf("bulk row error = %q, want %q", res.Errors[0].ErrorMessage, ErrLifecycleEvidenceFailed.Error())
	}
}

func newEvidenceTrackingDB(queryFn func(context.Context, string, ...any) (pgx.Rows, error)) (*fakeTx, *int) {
	evidenceWrites := 0
	db := &fakeTx{fakeDBTX: fakeDBTX{
		queryFn: queryFn,
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				if strings.Contains(query, "information_schema.tables") {
					for _, v := range dest {
						*(v.(*bool)) = true
					}
					return nil
				}
				*(dest[0].(*bool)) = true
				return nil
			}}
		},
		execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(query, "schedule_legacy_mutation_evidence") {
				evidenceWrites++
			}
			return pgconn.NewCommandTag("INSERT 1"), nil
		},
	}}
	return db, &evidenceWrites
}

func TestDeletePatternWritesEvidenceOnBlock(t *testing.T) {
	db, writes := newEvidenceTrackingDB(func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
		return &rowsStub{}, nil
	})
	err := (WorkingPatternRepository{DB: db}).DeletePattern(context.Background(), uuid.New())
	if !errors.Is(err, ErrLifecycleMutationBlocked) || *writes != 1 || !db.committed {
		t.Fatalf("DeletePattern err=%v writes=%d committed=%v", err, *writes, db.committed)
	}
}

func TestDeleteAssignmentRuleWritesEvidenceOnBlock(t *testing.T) {
	db, writes := newEvidenceTrackingDB(nil)
	err := (AssignmentRuleRepository{DB: db}).DeleteAssignmentRule(context.Background(), uuid.New())
	if !errors.Is(err, ErrLifecycleMutationBlocked) || *writes != 1 || !db.committed {
		t.Fatalf("DeleteAssignmentRule err=%v writes=%d committed=%v", err, *writes, db.committed)
	}
}

func newEvidenceTx(writes *int, committed *bool) *trackingFakeTx {
	return &trackingFakeTx{
		fakeDBTX: fakeDBTX{
			execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
				if strings.Contains(query, "schedule_legacy_mutation_evidence") {
					*writes++
				}
				return pgconn.NewCommandTag("INSERT 1"), nil
			},
		},
		onCommit: func() { *committed = true },
	}
}

func TestProcessHolidayLeaveWritesEvidenceOnBlock(t *testing.T) {
	empID, employmentID, leaveTypeID := uuid.New(), uuid.New(), uuid.New()
	evidenceWrites, txCount := 0, 0
	mainTxCommitted, evidenceTxCommitted := false, false
	pool := fakePoolBeginner{beginFn: func(context.Context) (pgx.Tx, error) {
		txCount++
		if txCount > 1 {
			return newEvidenceTx(&evidenceWrites, &evidenceTxCommitted), nil
		}
		return &trackingFakeTx{
			fakeDBTX: fakeDBTX{
				queryRowFn: holidayLeaveGuardRowFn(employmentID, &leaveTypeID),
				queryFn: func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
					return &rowsStub{data: [][]any{{empID}}}, nil
				},
				execFn: func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
					return pgconn.NewCommandTag("INSERT 1"), nil
				},
			},
			onCommit: func() { mainTxCommitted = true },
		}, nil
	}}

	_, err := (HolidayRepository{DB: &pool}).ProcessHolidayLeave(context.Background(), uuid.New())
	if !errors.Is(err, ErrLifecycleMutationBlocked) || mainTxCommitted || !evidenceTxCommitted || evidenceWrites != 1 {
		t.Fatalf("ProcessHolidayLeave err=%v main=%v evidence=%v writes=%d", err, mainTxCommitted, evidenceTxCommitted, evidenceWrites)
	}
}

func holidayLeaveGuardRowFn(employmentID uuid.UUID, leaveTypeID *uuid.UUID) func(context.Context, string, ...any) pgx.Row {
	return func(_ context.Context, query string, _ ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error {
			switch {
			case strings.Contains(query, "FROM holidays"):
				*(dest[0].(*string)) = "2026-11-15"
				*(dest[1].(*string)) = "Holiday"
				*(dest[2].(*bool)) = false
				*(dest[3].(**uuid.UUID)) = leaveTypeID
			case strings.Contains(query, "FROM leave_types"):
				*(dest[0].(*string)) = "Annual"
			case strings.Contains(query, "FROM employment_segments"):
				*(dest[0].(*uuid.UUID)) = employmentID
			case strings.Contains(query, "FROM leave_requests WHERE employment_id = $1 AND status"):
				*(dest[0].(*bool)) = false
			case strings.Contains(query, "FROM leave_balances"):
				*(dest[0].(*int)) = 5
				*(dest[1].(*int)) = 0
				*(dest[2].(*string)) = "0"
			case strings.Contains(query, "SELECT count(*) FROM leave_requests"):
				*(dest[0].(*int64)) = 0
			case strings.Contains(query, "information_schema.tables"):
				for _, v := range dest {
					*(v.(*bool)) = true
				}
			default:
				*(dest[0].(*bool)) = true
			}
			return nil
		}}
	}
}

func runLegacyApproveSwapGuard(schemaComplete bool) (bool, bool, int, error) {
	requesterID, targetID := uuid.New(), uuid.New()
	reqEmpID, tgtEmpID := uuid.New(), uuid.New()
	evidenceWrites, txCount, empLookup := 0, 0, 0
	mainCommitted, evidenceCommitted := false, false
	pool := fakePoolBeginner{beginFn: func(context.Context) (pgx.Tx, error) {
		txCount++
		if txCount > 1 {
			return newEvidenceTx(&evidenceWrites, &evidenceCommitted), nil
		}
		return &trackingFakeTx{
			fakeDBTX: fakeDBTX{
				queryRowFn: legacySwapGuardRowFn(requesterID, targetID, reqEmpID, tgtEmpID, &empLookup, schemaComplete),
				execFn: func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
					return pgconn.NewCommandTag("UPDATE 1"), nil
				},
			},
			onCommit: func() { mainCommitted = true },
		}, nil
	}}
	err := (ShiftSwapRepository{DB: &pool}).ApproveSwap(context.Background(), uuid.New(), uuid.New(), api.ShiftSwapApproved)
	return mainCommitted, evidenceCommitted, evidenceWrites, err
}

func legacySwapGuardRowFn(requesterID, targetID, reqEmpID, tgtEmpID uuid.UUID, empLookup *int, schemaComplete bool) func(context.Context, string, ...any) pgx.Row {
	return func(_ context.Context, query string, _ ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error {
			switch {
			case strings.Contains(query, "FROM shift_swap_requests"):
				*(dest[0].(*string)) = string(api.ShiftSwapPendingAdmin)
				*(dest[1].(*uuid.UUID)) = requesterID
				*(dest[2].(*uuid.UUID)) = targetID
				*(dest[3].(*string)) = "2026-11-15"
			case strings.Contains(query, "FROM employment_segments"):
				*empLookup++
				if *empLookup == 1 {
					*(dest[0].(*uuid.UUID)) = reqEmpID
				} else {
					*(dest[0].(*uuid.UUID)) = tgtEmpID
				}
			case strings.Contains(query, "information_schema.tables"):
				*(dest[0].(*bool)) = true
				*(dest[1].(*bool)) = schemaComplete
				*(dest[2].(*bool)) = schemaComplete
				*(dest[3].(*bool)) = schemaComplete
			default:
				*(dest[0].(*bool)) = true
			}
			return nil
		}}
	}
}

func TestApproveSwapLegacyLifecycleGuardAndEvidence(t *testing.T) {
	mainCommitted, evidenceCommitted, writes, err := runLegacyApproveSwapGuard(true)
	if !errors.Is(err, ErrLifecycleMutationBlocked) || mainCommitted || !evidenceCommitted || writes != 1 {
		t.Fatalf("blocked swap err=%v main=%v evidence=%v writes=%d", err, mainCommitted, evidenceCommitted, writes)
	}
	mainCommitted, evidenceCommitted, writes, err = runLegacyApproveSwapGuard(false)
	if !errors.Is(err, ErrLifecycleSchemaUnavailable) || mainCommitted || evidenceCommitted || writes != 0 {
		t.Fatalf("schema unavailable swap err=%v main=%v evidence=%v writes=%d", err, mainCommitted, evidenceCommitted, writes)
	}
}

type fakePoolBeginner struct {
	fakeDBTX
	beginFn func(context.Context) (pgx.Tx, error)
}

func (f fakePoolBeginner) Begin(ctx context.Context) (pgx.Tx, error) {
	return f.beginFn(ctx)
}

type trackingFakeTx struct {
	fakeDBTX
	onCommit func()
}

func (f *trackingFakeTx) Commit(context.Context) error {
	if f.onCommit != nil {
		f.onCommit()
	}
	return nil
}
func (f *trackingFakeTx) Rollback(context.Context) error        { return nil }
func (f *trackingFakeTx) Begin(context.Context) (pgx.Tx, error) { return f, nil }
func (f *trackingFakeTx) CopyFrom(context.Context, pgx.Identifier, []string, pgx.CopyFromSource) (int64, error) {
	return 0, nil
}
func (f *trackingFakeTx) SendBatch(context.Context, *pgx.Batch) pgx.BatchResults { return nil }
func (f *trackingFakeTx) LargeObjects() pgx.LargeObjects                         { return pgx.LargeObjects{} }
func (f *trackingFakeTx) Prepare(context.Context, string, string) (*pgconn.StatementDescription, error) {
	return nil, nil
}
func (f *trackingFakeTx) Conn() *pgx.Conn { return nil }
