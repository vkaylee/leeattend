package repository

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
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

func TestFinishLifecycleMutationEvidenceWriteFailure(t *testing.T) {
	dbErr := errors.New("evidence disk full")
	tx := &fakeTx{fakeDBTX: fakeDBTX{execFn: func(context.Context, string, ...any) (pgconn.CommandTag, error) {
		return pgconn.CommandTag{}, dbErr
	}}}
	err := finishLifecycleMutation(context.Background(), fakeTxBeginner{tx: tx}, blockedMutation([]uuid.UUID{uuid.New()}, []string{"2026-11-15"}))
	if !errors.Is(err, api.ErrLifecycleEvidenceFailed) || !errors.Is(err, dbErr) {
		t.Fatalf("got %v, want api.ErrLifecycleEvidenceFailed wrapping %v", err, dbErr)
	}
	if errors.Is(err, api.ErrLifecycleMutationBlocked) {
		t.Fatalf("evidence failure must NOT match ErrLifecycleMutationBlocked: %v", err)
	}
	if !strings.Contains(err.Error(), api.ErrLifecycleMutationBlocked.Error()) || !strings.Contains(err.Error(), dbErr.Error()) {
		t.Fatalf("error string must preserve both causes for logging, got %q", err.Error())
	}
}

func TestFinishLifecycleMutationNilBeginnerFailsClosed(t *testing.T) {
	err := finishLifecycleMutation(context.Background(), nil, blockedMutation([]uuid.UUID{uuid.New()}, []string{"2026-11-15"}))
	if !errors.Is(err, api.ErrLifecycleEvidenceFailed) || errors.Is(err, api.ErrLifecycleMutationBlocked) {
		t.Fatalf("nil beginner got %v, want api.ErrLifecycleEvidenceFailed", err)
	}
}

func TestFinishLifecycleMutationDBWithoutBeginner(t *testing.T) {
	blockedErr := finishLifecycleMutationDB(context.Background(), &fakeDBTX{}, blockedMutation([]uuid.UUID{uuid.New()}, []string{"2026-11-15"}))
	if !errors.Is(blockedErr, api.ErrLifecycleEvidenceFailed) || errors.Is(blockedErr, api.ErrLifecycleMutationBlocked) {
		t.Fatalf("non-beginner got %v, want api.ErrLifecycleEvidenceFailed", blockedErr)
	}
	schemaErr := finishLifecycleMutationDB(context.Background(), &fakeDBTX{}, api.ErrLifecycleSchemaUnavailable)
	if !errors.Is(schemaErr, api.ErrLifecycleSchemaUnavailable) || errors.Is(schemaErr, api.ErrLifecycleEvidenceFailed) {
		t.Fatalf("schema unavailable got %v, want ErrLifecycleSchemaUnavailable", schemaErr)
	}
}

func TestGuardLifecycleShiftMutationWritesEvidence(t *testing.T) {
	calls := 0
	evidenceWrites := 0
	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(context.Context, string, ...any) pgx.Row {
			calls++
			return fakeRow{scanFn: func(dest ...any) error {
				if calls == 1 {
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
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}}
	guardErr := guardLifecycleShiftMutation(context.Background(), tx, uuid.New())
	err := finishLifecycleMutation(context.Background(), fakeTxBeginner{tx: tx}, guardErr)
	if !errors.Is(err, api.ErrLifecycleMutationBlocked) || evidenceWrites != 1 || !tx.committed {
		t.Fatalf("shift template block err=%v writes=%d committed=%v", err, evidenceWrites, tx.committed)
	}
}
