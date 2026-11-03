package repository

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestOvertimeRepository_ListOvertimePolicies(t *testing.T) {
	wsID := uuid.New()
	policyID := uuid.New()
	now := time.Now().UTC()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "SELECT count(*)") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*int64)) = 1
					return nil
				}}
			}
			return fakeRow{}
		},
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			desc := "Standard Overtime Policy"
			return &rowsStub{
				data: [][]any{
					{policyID, wsID, "Standard", &desc, now, now},
				},
			}, nil
		},
	}
	repo := OvertimeRepository{
		DB: db,
	}
	res, err := repo.ListOvertimePolicies(context.Background(), wsID, 1, 50)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if res.Total != 1 || len(res.Data) != 1 || res.Data[0].ID != policyID {
		t.Fatalf("unexpected policies: %+v", res)
	}
}

func TestOvertimeRepository_CreateAndGetPolicy(t *testing.T) {
	wsID := uuid.New()
	policyID := uuid.New()
	ruleID := uuid.New()
	now := time.Now().UTC()
	mult, _ := decimal.NewFromString("1.50")
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			desc := "Created Policy"
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = policyID
				*(dest[1].(*uuid.UUID)) = wsID
				*(dest[2].(*string)) = "Standard"
				*(dest[3].(**string)) = &desc
				*(dest[4].(*time.Time)) = now
				*(dest[5].(*time.Time)) = now
				return nil
			}}
		},
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			startTime := time.Date(0, 1, 1, 18, 0, 0, 0, time.UTC)
			endTime := time.Date(0, 1, 1, 22, 0, 0, 0, time.UTC)
			return &rowsStub{
				data: [][]any{
					{ruleID, policyID, 1, mult, &startTime, &endTime, 127, false, now, now},
				},
			}, nil
		},
	}
	repo := OvertimeRepository{
		DB: db,
	}
	created, err := repo.CreateOvertimePolicy(context.Background(), wsID, api.CreateOvertimePolicyRequest{Name: "Standard"})
	if err != nil {
		t.Fatalf("unexpected create err: %v", err)
	}
	if created.ID != policyID {
		t.Fatalf("unexpected created policy: %+v", created)
	}
	detail, err := repo.GetOvertimePolicy(context.Background(), wsID, policyID)
	if err != nil {
		t.Fatalf("unexpected get err: %v", err)
	}
	if len(detail.Rules) != 1 || detail.Rules[0].ID != ruleID || !detail.Rules[0].Multiplier.Equal(mult) {
		t.Fatalf("unexpected detail: %+v", detail)
	}
}

func TestOvertimeRepository_SubmitOvertimeRequest(t *testing.T) {
	wsID := uuid.New()
	reqID := uuid.New()
	empID := uuid.New()

	tx := &fakeTx{}
	tx.execFn = func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
		return pgconn.NewCommandTag("UPDATE 1"), nil
	}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		if strings.Contains(sql, "FROM overtime_request_employees") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = empID
				return nil
			}}
		}
		if strings.Contains(sql, "FROM approval_workflows") {
			return fakeRow{scanFn: func(_ ...any) error {
				return pgx.ErrNoRows // fallback snapshot
			}}
		}
		return fakeRow{}
	}

	repo := OvertimeRepository{
		BeginTx: func(_ context.Context, _ uuid.UUID) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	err := repo.SubmitOvertimeRequest(context.Background(), wsID, reqID)
	if err != nil {
		t.Fatalf("unexpected submit err: %v", err)
	}
	if !tx.committed {
		t.Fatal("expected submit transaction to commit")
	}
}

func TestOvertimeRepository_ClaimAndApprove(t *testing.T) {
	wsID := uuid.New()
	empID := uuid.New()
	user := uuid.New()
	employmentID := uuid.New()

	tx := &fakeTx{}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		if strings.Contains(sql, "FROM employments") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employmentID
				return nil
			}}
		}
		if strings.Contains(sql, "INSERT INTO attendance_daily_summaries") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = uuid.New()
				return nil
			}}
		}
		if strings.Contains(sql, "FROM approval_workflows") {
			return fakeRow{scanFn: func(_ ...any) error {
				return pgx.ErrNoRows
			}}
		}
		return fakeRow{}
	}
	tx.execFn = func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
		return pgconn.NewCommandTag("UPDATE 1"), nil
	}

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "FROM employments") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = employmentID
					return nil
				}}
			}
			return fakeRow{}
		},
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}

	repo := OvertimeRepository{
		DB: db,
		BeginTx: func(_ context.Context, _ uuid.UUID) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	err := repo.ClaimOvertime(context.Background(), wsID, empID, api.ClaimOTRequest{Date: "2026-10-20", ClaimedMinutes: 60}, user)
	if err != nil {
		t.Fatalf("unexpected claim err: %v", err)
	}
	if !tx.committed {
		t.Fatal("expected claim transaction commit")
	}

	err = repo.ApproveOvertime(context.Background(), wsID, empID, api.ApproveOTRequest{Date: "2026-10-20", ApprovedMinutes: 60, Status: "approved"}, user)
	if err != nil {
		t.Fatalf("unexpected approve err: %v", err)
	}
}
