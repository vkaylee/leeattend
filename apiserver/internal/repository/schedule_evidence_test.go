package repository

import (
	"context"
	"encoding/json"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
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

func TestRecordScheduleMismatchesValidatesBeforeTransactionAndWritesOnce(t *testing.T) {
	execs := 0
	tx := &fakeTx{fakeDBTX: fakeDBTX{execFn: func(_ context.Context, query string, args ...any) (pgconn.CommandTag, error) {
		execs++
		if !strings.Contains(query, "VALUES ($1, $2, $3::date") || !strings.Contains(query, "($8, $9, $10::date") {
			t.Fatalf("expected one bulk mismatch insert, got %s", query)
		}
		if len(args) != 14 {
			t.Fatalf("expected 14 bulk arguments, got %d", len(args))
		}
		return pgconn.NewCommandTag("INSERT 0 2"), nil
	}}}
	repo := ScheduleRepository{BeginTx: func(context.Context) (TxBeginner, error) {
		return fakeTxBeginner{tx: tx}, nil
	}}
	records := []ScheduleShadowMismatchRecord{
		{EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-11-14", MismatchKind: "missing", LegacySnapshot: json.RawMessage(`{"status":"missing"}`), ResolverSnapshot: json.RawMessage(`{"status":"work"}`), ResolverVersion: "v1"},
		{EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-11-15", MismatchKind: "source", LegacySnapshot: json.RawMessage(`{"status":"work"}`), ResolverSnapshot: json.RawMessage(`{"status":"work"}`), ResolverVersion: "v1"},
	}
	if err := repo.RecordScheduleMismatches(context.Background(), records); err != nil {
		t.Fatalf("record bulk mismatches: %v", err)
	}
	if execs != 1 || !tx.committed {
		t.Fatalf("expected one committed bulk write, execs=%d committed=%v", execs, tx.committed)
	}
}

func TestRecordScheduleMismatchesRejectsAnyInvalidRecordBeforeBegin(t *testing.T) {
	began := false
	repo := ScheduleRepository{BeginTx: func(context.Context) (TxBeginner, error) {
		began = true
		return nil, nil
	}}
	records := []ScheduleShadowMismatchRecord{
		{EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-11-14", MismatchKind: "missing", LegacySnapshot: json.RawMessage(`{}`), ResolverSnapshot: json.RawMessage(`{}`), ResolverVersion: "v1"},
		{EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-11-15", MismatchKind: "invalid", LegacySnapshot: json.RawMessage(`{}`), ResolverSnapshot: json.RawMessage(`{}`), ResolverVersion: "v1"},
	}
	if err := repo.RecordScheduleMismatches(context.Background(), records); err == nil {
		t.Fatal("expected invalid mismatch kind error")
	}
	if began {
		t.Fatal("transaction began before all records were validated")
	}
}

func TestGetScheduleActivationReadinessReportsIndependentBlockers(t *testing.T) {
	employerID := uuid.New()
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error {
			switch {
			case strings.Contains(query, "FROM feature_flags"):
				*(dest[0].(*bool)) = true
			case strings.Contains(query, "FROM config_entries"):
				*(dest[0].(*bool)) = true
			case strings.Contains(query, "FROM schedule_shadow_mismatches"):
				*(dest[0].(*int64)) = 2
			case strings.Contains(query, "FROM (SELECT employment_id, business_date FROM schedule_day_revisions"):
				*(dest[0].(*int64)) = 3
			case strings.Contains(query, "FROM schedule_legacy_mutation_evidence"):
				*(dest[0].(*int64)) = 1
			case strings.Contains(query, "FROM outbox_jobs"):
				*(dest[0].(*int64)) = 4
			case strings.Contains(query, "generate_series"):
				*(dest[0].(*int64)) = 5
			case strings.Contains(query, "count(DISTINCT"):
				*(dest[0].(*int64)) = 3
			default:
				t.Fatalf("unexpected readiness query: %s", query)
			}
			return nil
		}}
	}}

	result, err := (ScheduleRepository{DB: db}).GetScheduleActivationReadiness(context.Background(), employerID, "2026-11-01", "2026-11-05")
	if err != nil {
		t.Fatalf("get readiness: %v", err)
	}
	if result.EmployerID != employerID || !result.GlobalFlagEnabled || !result.EmployerActivation {
		t.Fatalf("unexpected activation state: %+v", result)
	}
	if result.OpenMismatchCount != 2 || result.CurrentDuplicateCount != 3 || result.LegacyMutationCount != 1 || result.DeadLetterCount != 4 {
		t.Fatalf("readiness counters overwritten: %+v", result)
	}
	if result.ExpectedCoverageDays != 5 || result.CoveredOfficialDays != 3 || result.MissingCoverageDays != 2 {
		t.Fatalf("unexpected coverage counters: %+v", result)
	}
	if result.Ready {
		t.Fatal("expected blocked readiness when any blocker is present")
	}
}

func TestGetScheduleActivationReadinessCleanStateIsReady(t *testing.T) {
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error {
			switch {
			case strings.Contains(query, "FROM feature_flags"), strings.Contains(query, "FROM config_entries"):
				*(dest[0].(*bool)) = true
			case strings.Contains(query, "generate_series"):
				*(dest[0].(*int64)) = 5
			case strings.Contains(query, "count(DISTINCT"):
				*(dest[0].(*int64)) = 5
			default:
				*(dest[0].(*int64)) = 0
			}
			return nil
		}}
	}}

	result, err := (ScheduleRepository{DB: db}).GetScheduleActivationReadiness(context.Background(), uuid.New(), "2026-11-01", "2026-11-05")
	if err != nil {
		t.Fatalf("get readiness: %v", err)
	}
	if !result.Ready || result.MissingCoverageDays != 0 {
		t.Fatalf("expected clean readiness, got %+v", result)
	}
}
