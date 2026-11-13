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
