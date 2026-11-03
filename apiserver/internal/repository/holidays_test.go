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

func TestHolidayRepository_GetHolidayByID_NotFound(t *testing.T) {
	wsID := uuid.New()
	holidayID := uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(_ ...any) error {
				return pgx.ErrNoRows
			}}
		},
	}
	repo := HolidayRepository{
		DB: db,
	}

	_, err := repo.GetHolidayByID(context.Background(), wsID, holidayID)
	if err != api.ErrHolidayNotFound {
		t.Fatalf("expected ErrHolidayNotFound, got %v", err)
	}
}

func TestHolidayRepository_GetHolidayByID_Success(t *testing.T) {
	wsID := uuid.New()
	holidayID := uuid.New()
	now := time.Now().UTC()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = holidayID
				*(dest[1].(*uuid.UUID)) = wsID
				*(dest[2].(*string)) = "National Day"
				*(dest[3].(*string)) = "2026-09-02"
				*(dest[4].(*string)) = "national"
				*(dest[5].(*bool)) = true
				*(dest[6].(**uuid.UUID)) = nil
				*(dest[7].(**decimal.Decimal)) = nil
				*(dest[8].(**string)) = nil
				*(dest[9].(**int32)) = nil
				*(dest[10].(*bool)) = true
				*(dest[11].(*time.Time)) = now
				*(dest[12].(*time.Time)) = now
				*(dest[13].(**uuid.UUID)) = nil
				return nil
			}}
		},
	}
	repo := HolidayRepository{
		DB: db,
	}

	rec, err := repo.GetHolidayByID(context.Background(), wsID, holidayID)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if rec.ID != holidayID || rec.Name != "National Day" || rec.Date != "2026-09-02" {
		t.Fatalf("unexpected record: %+v", rec)
	}
}

func TestHolidayRepository_CreateHoliday_Conflict(t *testing.T) {
	wsID := uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(_ ...any) error {
				return &pgconn.PgError{Code: "23505"}
			}}
		},
	}
	repo := HolidayRepository{
		DB: db,
	}

	_, err := repo.CreateHoliday(context.Background(), wsID, api.CreateHolidayRequest{
		Name: "National Day",
		Date: "2026-09-02",
	})
	if err != api.ErrHolidayAlreadyExists {
		t.Fatalf("expected ErrHolidayAlreadyExists, got %v", err)
	}
}

func TestHolidayRepository_DeleteHoliday_SoftDeleteAndNotFound(t *testing.T) {
	wsID := uuid.New()
	hID := uuid.New()

	var executedSQL string
	db := &fakeDBTX{
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(sql, "UPDATE holidays") {
				executedSQL = sql
			}
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}
	repo := HolidayRepository{
		DB: db,
	}

	if err := repo.DeleteHoliday(context.Background(), wsID, hID); err != nil {
		t.Fatalf("expected success, got %v", err)
	}
	if !strings.Contains(executedSQL, "deleted_at = NOW()") {
		t.Fatalf("expected soft delete, got SQL: %s", executedSQL)
	}

	dbNotFound := &fakeDBTX{
		execFn: func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
			return pgconn.NewCommandTag("UPDATE 0"), nil
		},
	}
	repoNotFound := HolidayRepository{
		DB: dbNotFound,
	}
	if err := repoNotFound.DeleteHoliday(context.Background(), wsID, hID); err != api.ErrHolidayNotFound {
		t.Fatalf("expected ErrHolidayNotFound, got %v", err)
	}
}

func TestHolidayRepository_ProcessHolidayLeave_PaidHoliday(t *testing.T) {
	wsID := uuid.New()
	hID := uuid.New()

	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "FROM holidays") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*string)) = "2026-09-02"
					*(dest[1].(*string)) = "National Day"
					*(dest[2].(*bool)) = true
					*(dest[3].(**uuid.UUID)) = nil
					return nil
				}}
			}
			return fakeRow{}
		},
	}}
	repo := HolidayRepository{
		BeginTx: func(_ context.Context, _ uuid.UUID) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	results, err := repo.ProcessHolidayLeave(context.Background(), wsID, hID)
	if err != nil {
		t.Fatalf("expected success, got %v", err)
	}
	if len(results) != 1 || results[0].Type != "Paid" {
		t.Fatalf("expected [Paid], got %+v", results)
	}
}
