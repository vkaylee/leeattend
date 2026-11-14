package repository

import (
	"context"
	"encoding/json"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgconn"
)

func TestRecordScheduleShadowMismatchValidatesAndUpserts(t *testing.T) {
	calls := 0
	db := &fakeDBTX{execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
		calls++
		if !strings.Contains(query, "schedule_shadow_mismatches") || !strings.Contains(query, "ON CONFLICT") {
			t.Fatalf("expected mismatch upsert query, got %s", query)
		}
		return pgconn.NewCommandTag("INSERT 1"), nil
	}}
	repo := ScheduleRepository{DB: db}
	err := repo.RecordScheduleShadowMismatch(context.Background(), ScheduleShadowMismatchRecord{
		EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-11-14", MismatchKind: "missing",
		LegacySnapshot: json.RawMessage(`{"status":"legacy"}`), ResolverSnapshot: json.RawMessage(`{"status":"missing"}`), ResolverVersion: "schedule-resolver-v1",
	})
	if err != nil || calls != 1 {
		t.Fatalf("expected one mismatch upsert, calls=%d err=%v", calls, err)
	}
}

func TestRecordScheduleShadowMismatchRejectsInvalidSnapshot(t *testing.T) {
	err := (ScheduleRepository{DB: &fakeDBTX{}}).RecordScheduleShadowMismatch(context.Background(), ScheduleShadowMismatchRecord{
		EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-11-14", MismatchKind: "missing", ResolverVersion: "v1",
		LegacySnapshot: json.RawMessage(`[]`), ResolverSnapshot: json.RawMessage(`{}`),
	})
	if err == nil {
		t.Fatal("expected invalid snapshot error")
	}
}
