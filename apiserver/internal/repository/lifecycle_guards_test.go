package repository

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func TestGuardLifecycleDateMutationPreservesLegacyBeforeActivation(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				if !strings.Contains(query, "information_schema.tables") {
					t.Fatalf("unexpected lifecycle query: %s", query)
				}
				for _, value := range dest {
					*(value.(*bool)) = false
				}
				return nil
			}}
		},
	}
	if err := guardLifecycleDateMutation(context.Background(), db, []uuid.UUID{uuid.New()}, []string{"2026-11-13"}); err != nil {
		t.Fatalf("legacy target should remain compatible before activation: %v", err)
	}
}

func TestGuardLifecycleDateRangeChecksActivatedScope(t *testing.T) {
	employmentID := uuid.New()
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, args ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error {
			switch {
			case strings.Contains(query, "information_schema.tables"):
				if len(dest) != 4 {
					t.Fatalf("expected four lifecycle schema flags, got %d", len(dest))
				}
				*(dest[0].(*bool)) = true
				*(dest[1].(*bool)) = false
				*(dest[2].(*bool)) = false
				*(dest[3].(*bool)) = true
			case strings.Contains(query, "schedule_day_revisions"):
				*(dest[0].(*bool)) = false
			case strings.Contains(query, "feature_flags") && strings.Contains(query, "daterange") && strings.Contains(query, "schedule.lifecycle.enabled"):
				if len(args) != 3 || args[1] != "2026-11-01" || args[2] != "2026-11-30" {
					t.Fatalf("unexpected activation args: %v", args)
				}
				*(dest[0].(*bool)) = true
			default:
				t.Fatalf("unexpected lifecycle query: %s", query)
			}
			return nil
		}}
	}}
	if err := guardLifecycleDateRange(context.Background(), db, []uuid.UUID{employmentID}, "2026-11-01", "2026-11-30"); !errors.Is(err, ErrLifecycleMutationBlocked) {
		t.Fatalf("expected activated scope mutation block, got %v", err)
	}
}

func TestGuardLifecycleDateMutationBlocksOfficialDate(t *testing.T) {
	calls := 0
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
	}
	if err := guardLifecycleDateMutation(context.Background(), db, []uuid.UUID{uuid.New()}, []string{"2026-11-13"}); !errors.Is(err, ErrLifecycleMutationBlocked) {
		t.Fatalf("expected official date mutation block, got %v", err)
	}
}
