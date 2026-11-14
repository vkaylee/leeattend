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

func TestInsertApprovedAdjustmentBlocksOfficialDate(t *testing.T) {
	calls := 0
	adjustmentExecCalls := 0
	db := &fakeDBTX{
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
				return pgconn.NewCommandTag("INSERT 1"), nil
			}
			adjustmentExecCalls++
			return pgconn.NewCommandTag("INSERT 1"), nil
		},
	}

	err := insertApprovedAdjustment(context.Background(), db, uuid.New(), uuid.New(), api.AttendanceImportRow{
		PunchTime: time.Date(2026, 11, 13, 9, 0, 0, 0, time.UTC),
	})
	if !errors.Is(err, ErrLifecycleMutationBlocked) {
		t.Fatalf("expected official date mutation block, got %v", err)
	}
	if adjustmentExecCalls != 0 {
		t.Fatalf("expected no adjustment insert after lifecycle block, got %d adjustment exec calls", adjustmentExecCalls)
	}
}

func TestEnterPaperOTBlocksClosedDate(t *testing.T) {
	employmentID := uuid.New()
	calls := 0
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			calls++
			return fakeRow{scanFn: func(dest ...any) error {
				switch {
				case strings.Contains(query, "FROM employments"):
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
	}

	_, err := (AttendanceRepository{DB: db}).EnterPaperOT(context.Background(), uuid.New(), api.PaperOTRequest{
		Date:           "2026-11-13",
		ClaimedMinutes: 60,
		Reason:         "paper form",
	}, uuid.New())
	if !errors.Is(err, ErrLifecycleMutationBlocked) {
		t.Fatalf("expected closed date mutation block, got %v", err)
	}
}
