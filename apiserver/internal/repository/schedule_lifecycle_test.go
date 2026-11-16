package repository

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestFinishLifecycleMutationWritesEvidenceInIndependentTransaction(t *testing.T) {
	employmentID, actorDate := uuid.New(), "2026-11-15"
	writes := 0
	tx := &fakeTx{fakeDBTX: fakeDBTX{execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
		if strings.Contains(query, "schedule_legacy_mutation_evidence") {
			writes++
		}
		return pgconn.NewCommandTag("INSERT 0 1"), nil
	}}}
	err := finishLifecycleMutation(context.Background(), fakeTxBeginner{tx: tx}, blockedMutation([]uuid.UUID{employmentID}, []string{actorDate}))
	if !errors.Is(err, ErrLifecycleMutationBlocked) || writes != 1 || !tx.committed {
		t.Fatalf("evidence transaction = %v, writes=%d, committed=%v", err, writes, tx.committed)
	}
}

func TestActivateScheduleEmployerWritesConfigAndAuditAtomically(t *testing.T) {
	actorID, employerID := uuid.New(), uuid.New()
	writes := 0
	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "FROM feature_flags") {
				t.Fatalf("unexpected activation readiness query: %s", query)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = true
				return nil
			}}
		},
		execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(query, "config_entries") || strings.Contains(query, "schedule_lifecycle.activation.enabled") {
				writes++
			}
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}}
	repo := ScheduleRepository{DB: tx}
	if err := repo.ActivateScheduleEmployer(context.Background(), employerID, "2026-11-01", "2026-11-30", "clean evidence", actorID); err != nil {
		t.Fatalf("activate employer: %v", err)
	}
	if writes != 2 || !tx.committed {
		t.Fatalf("expected config and audit writes in committed transaction, writes=%d committed=%v", writes, tx.committed)
	}
}

func TestActivateScheduleEmployerDoesNotWriteWhenNotReady(t *testing.T) {
	writes := 0
	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(context.Context, string, ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = false; return nil }}
		},
		execFn: func(context.Context, string, ...any) (pgconn.CommandTag, error) {
			writes++
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}}
	if err := (ScheduleRepository{DB: tx}).ActivateScheduleEmployer(context.Background(), uuid.New(), "2026-11-01", "2026-11-30", "not ready", uuid.New()); !errors.Is(err, api.ErrScheduleActivationNotReady) {
		t.Fatalf("expected not-ready error, got %v", err)
	}
	if writes != 0 || tx.committed {
		t.Fatalf("not-ready activation must not write or commit, writes=%d committed=%v", writes, tx.committed)
	}
}

func TestDeactivateScheduleEmployerRejectsImmutableEvidence(t *testing.T) {
	tx := &fakeTx{fakeDBTX: fakeDBTX{queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
	}}}
	err := (ScheduleRepository{DB: tx}).DeactivateScheduleEmployer(context.Background(), uuid.New(), "2026-11-01", "2026-11-30", "rollback", uuid.New())
	if !errors.Is(err, api.ErrScheduleActivationEvidence) {
		t.Fatalf("expected immutable evidence error, got %v", err)
	}
	if tx.committed {
		t.Fatal("evidence-blocked deactivation must not commit")
	}
}

func TestDeactivateScheduleEmployerWritesDisableAndAuditAtomically(t *testing.T) {
	writes := 0
	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = strings.Contains(query, "effective_from = $2::date")
				return nil
			}}
		},
		execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(query, "config_entries") || strings.Contains(query, "activation.disabled") {
				writes++
			}
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}}
	if err := (ScheduleRepository{DB: tx}).DeactivateScheduleEmployer(context.Background(), uuid.New(), "2026-11-01", "2026-11-30", "rollback", uuid.New()); err != nil {
		t.Fatalf("deactivate employer: %v", err)
	}
	if writes != 2 || !tx.committed {
		t.Fatalf("expected disable and audit writes, writes=%d committed=%v", writes, tx.committed)
	}
}

func TestEnsureMutationPeriodOpenForRangeChecksAnyIntersectingPeriod(t *testing.T) {
	var query string
	db := &fakeDBTX{queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
		query = sql
		return fakeRow{scanFn: func(dest ...any) error {
			*(dest[0].(*bool)) = true
			return nil
		}}
	}}
	if err := ensureMutationPeriodOpenForRange(context.Background(), db, []uuid.UUID{uuid.New()}, "2026-11-30", "2026-12-02"); !errors.Is(err, ErrScheduleLockedPeriod) {
		t.Fatalf("intersecting locked period error = %v, want locked period", err)
	}
	if !strings.Contains(query, "<= $3::date") || !strings.Contains(query, ">= $2::date") {
		t.Fatalf("range guard must test period intersection, query=%s", query)
	}
}

func TestLockScheduleRevisionTargetsSortsBusinessDates(t *testing.T) {
	var args []any
	db := &fakeDBTX{execFn: func(_ context.Context, query string, values ...any) (pgconn.CommandTag, error) {
		if strings.Contains(query, "FROM schedule_day_revisions") {
			args = values
		}
		return pgconn.NewCommandTag("SELECT 1"), nil
	}}
	if err := lockScheduleRevisionTargets(context.Background(), db, []scheduleRevisionLockKey{
		{employmentID: uuid.New(), businessDate: "2026-11-20"},
		{employmentID: uuid.New(), businessDate: "2026-11-16"},
	}); err != nil {
		t.Fatalf("lock schedule targets: %v", err)
	}
	if len(args) != 3 || args[1] != "2026-11-16" || args[2] != "2026-11-20" {
		t.Fatalf("lock range dates = %#v, want sorted bounds", args)
	}
}

func TestLockScheduleRevisionTargetsLocksEmployeeBeforeEmployment(t *testing.T) {
	queries := make([]string, 0, 3)
	db := &fakeDBTX{execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
		queries = append(queries, query)
		return pgconn.NewCommandTag("SELECT 1"), nil
	}}
	if err := lockScheduleRevisionTargets(context.Background(), db, []scheduleRevisionLockKey{{employmentID: uuid.New(), businessDate: "2026-11-16"}}); err != nil {
		t.Fatalf("lock schedule targets: %v", err)
	}
	if len(queries) != 3 || !strings.Contains(queries[0], "FROM employees") || !strings.Contains(queries[0], "FOR UPDATE") || !strings.Contains(queries[1], "FROM employments") || !strings.Contains(queries[2], "FROM schedule_day_revisions") {
		t.Fatalf("unexpected lock order: %v", queries)
	}
}

func TestRotatingPublishSkipsOfficeApprovalValidation(t *testing.T) {
	request := api.SchedulePublishRequest{Mode: "rotating", EmploymentIDs: []uuid.UUID{uuid.New()}, StartDate: "2026-11-16", EndDate: "2026-11-16", Timezone: "UTC", Reason: "rotating batch", PreviewID: uuid.New(), PreviewHash: strings.Repeat("a", 64)}
	if err := validatePublishInput("rotating-key", request); err != nil {
		t.Fatalf("rotating request validation: %v", err)
	}
}

func TestRejectConcreteScheduleOverlapsFailsClosed(t *testing.T) {
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
		if !strings.Contains(query, "candidate_people") || !strings.Contains(query, "scheduled_start_at") {
			t.Fatalf("overlap query missing person/timestamp checks: %s", query)
		}
		return fakeRow{scanFn: func(dest ...any) error {
			*(dest[0].(*bool)) = true
			return nil
		}}
	}}
	day := map[string]any{
		"employment_id": uuid.NewString(),
		"business_date": "2026-11-16",
		"status":        "work",
		"intervals": []map[string]any{{
			"sort_order":       0,
			"start":            "08:00:00",
			"end":              "17:00:00",
			"start_at":         "2026-11-16T08:00:00Z",
			"end_at":           "2026-11-16T17:00:00Z",
			"shift_version_id": uuid.NewString(),
			"breaks":           []map[string]any{},
		}},
	}
	if err := rejectConcreteScheduleOverlaps(context.Background(), db, []map[string]any{day}); !errors.Is(err, api.ErrScheduleConflict) {
		t.Fatalf("overlap error = %v, want schedule conflict", err)
	}
}

func TestValidateHumanOfficeApprovalRequiresApprovedPolicyCoverage(t *testing.T) {
	approvalID := uuid.New()
	employmentID := uuid.New()
	queries := 0
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
		queries++
		if !strings.Contains(query, "schedule_office_policies") || !strings.Contains(query, "approval_workflow_instances") || !strings.Contains(query, "generate_series") {
			t.Fatalf("approval coverage query missing policy/date binding: %s", query)
		}
		return fakeRow{scanFn: func(dest ...any) error {
			*(dest[0].(*bool)) = true
			return nil
		}}
	}}
	request := api.SchedulePublishRequest{EmploymentIDs: []uuid.UUID{employmentID}, StartDate: "2026-11-16", EndDate: "2026-11-17", ApprovalInstanceID: &approvalID}
	if err := validateHumanOfficeApproval(context.Background(), db, request); err != nil || queries != 1 {
		t.Fatalf("approved office policy validation err=%v queries=%d", err, queries)
	}
}

func TestValidateHumanOfficeApprovalFailsClosed(t *testing.T) {
	approvalID := uuid.New()
	db := &fakeDBTX{queryRowFn: func(context.Context, string, ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error {
			*(dest[0].(*bool)) = false
			return nil
		}}
	}}
	if err := validateHumanOfficeApproval(context.Background(), db, api.SchedulePublishRequest{EmploymentIDs: []uuid.UUID{uuid.New()}, StartDate: "2026-11-16", EndDate: "2026-11-16", ApprovalInstanceID: &approvalID}); !errors.Is(err, api.ErrScheduleApprovalInvalid) {
		t.Fatalf("expected invalid approval, got %v", err)
	}
}

func TestValidateSystemOfficePublicationAuthorityChecksApprovalEmployerAndDate(t *testing.T) {
	approvalID, employerID, employmentID := uuid.New(), uuid.New(), uuid.New()
	queries := 0
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
		queries++
		if !strings.Contains(query, "schedule_office_policies") || !strings.Contains(query, "approval_workflow_instances") || !strings.Contains(query, "e.employer_id") {
			t.Fatalf("authority query missing policy binding: %s", query)
		}
		return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
	}}
	if err := validateSystemOfficePublicationAuthority(context.Background(), db, ScheduleOfficeCoverageEvent{ApprovalInstanceID: approvalID, EmployerID: employerID, EmploymentID: employmentID, BusinessDate: "2026-11-16", Mode: "office"}); err != nil || queries != 1 {
		t.Fatalf("authority validation err=%v queries=%d", err, queries)
	}
}

func TestValidateSystemOfficePublicationAuthorityFailsClosed(t *testing.T) {
	db := &fakeDBTX{queryRowFn: func(context.Context, string, ...any) pgx.Row {
		return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = false; return nil }}
	}}
	if err := validateSystemOfficePublicationAuthority(context.Background(), db, ScheduleOfficeCoverageEvent{ApprovalInstanceID: uuid.New(), EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-11-16", Mode: "office"}); !errors.Is(err, ErrScheduleOfficePolicyUnavailable) {
		t.Fatalf("expected unavailable authority error, got %v", err)
	}
}

func TestScheduleLifecycleCreatePreviewRejectsInvalidScopeBeforeWrites(t *testing.T) {
	execCalled := false
	tx := &fakeTx{fakeDBTX: fakeDBTX{execFn: func(context.Context, string, ...any) (pgconn.CommandTag, error) {
		execCalled = true
		return pgconn.NewCommandTag("INSERT 0 1"), nil
	}}}
	repo := ScheduleRepository{DB: tx}

	_, err := repo.CreatePreview(context.Background(), uuid.New(), api.SchedulePreviewRequest{})
	if !errors.Is(err, ErrSchedulePreviewScope) {
		t.Fatalf("expected typed scope error, got %v", err)
	}
	if execCalled {
		t.Fatal("invalid scope must not write preview")
	}
}

func TestSanitizedPreviewDayWorkIncludesAllIntervalsAndNoPII(t *testing.T) {
	versionID := uuid.New()
	start, end := "08:00:00", "17:00:00"
	version := shiftVersionInfo{id: versionID, effectiveFrom: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC), intervals: []shiftIntervalInfo{{sortOrder: 0, startTime: &start, endTime: &end}, {sortOrder: 1, startTime: &start, endTime: &end}}}
	shiftID := uuid.New()
	day, err := sanitizedPreviewDay(resolvedScheduleDay{date: "2026-11-16", shiftIDs: []*uuid.UUID{&shiftID}, source: "pattern", timezone: "UTC", traces: []api.ScheduleTrace{{RuleName: "private name", Status: "applied", Priority: 80, Reason: new("private reason")}}}, map[uuid.UUID]shiftTemplateInfo{shiftID: {versions: []shiftVersionInfo{version}}})
	if err != nil {
		t.Fatalf("sanitize work preview: %v", err)
	}
	encoded, err := json.Marshal(day)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(encoded), "private name") || strings.Contains(string(encoded), "private reason") {
		t.Fatalf("preview contains resolver PII: %s", encoded)
	}
	if day["status"] != "work" || day["shift_version_id"] != versionID.String() || len(day["intervals"].([]map[string]any)) != 2 {
		t.Fatalf("unexpected work preview: %+v", day)
	}
}

func TestPreviewIntervalsPreservesSplitAndOvernightIntervals(t *testing.T) {
	firstStart, firstEnd := "08:00:00", "12:00:00"
	nightStart, nightEnd := "22:00:00", "06:00:00"
	version := shiftVersionInfo{id: uuid.New(), intervals: []shiftIntervalInfo{
		{sortOrder: 0, startTime: &firstStart, endTime: &firstEnd},
		{sortOrder: 1, startTime: &nightStart, endTime: &nightEnd},
	}}
	intervals, err := previewIntervals(version, "2026-11-16", "UTC")
	if err != nil {
		t.Fatalf("preview intervals: %v", err)
	}
	if len(intervals) != 2 {
		t.Fatalf("interval count = %d, want 2", len(intervals))
	}
	first := intervals[0]["start_at"].(time.Time)
	nightEndAt := intervals[1]["end_at"].(time.Time)
	if first.Format(time.RFC3339) != "2026-11-16T08:00:00Z" || nightEndAt.Format(time.RFC3339) != "2026-11-17T06:00:00Z" {
		t.Fatalf("unexpected interval timestamps: first=%s night_end=%s", first, nightEndAt)
	}
}

func TestPreviewIntervalsPreservesDayOffsetsAndRejectsDSTAmbiguity(t *testing.T) {
	startA, endA := "22:00:00", "02:00:00"
	startB, endB := "02:00:00", "06:00:00"
	version := shiftVersionInfo{id: uuid.New(), intervals: []shiftIntervalInfo{
		{sortOrder: 0, startTime: &startA, endTime: &endA, startDayOffset: 0, endDayOffset: 1},
		{sortOrder: 1, startTime: &startB, endTime: &endB, startDayOffset: 1, endDayOffset: 1},
	}}
	intervals, err := previewIntervals(version, "2026-04-30", "Asia/Ho_Chi_Minh")
	if err != nil {
		t.Fatalf("preview multi-segment offsets: %v", err)
	}
	if intervals[0]["end_at"].(time.Time).Format(time.RFC3339) != "2026-05-01T02:00:00+07:00" || intervals[1]["start_at"].(time.Time).Format(time.RFC3339) != "2026-05-01T02:00:00+07:00" {
		t.Fatalf("unexpected offset timestamps: %+v", intervals)
	}
	ambiguousStart, ambiguousEnd := "01:30:00", "03:30:00"
	_, err = previewIntervals(shiftVersionInfo{id: uuid.New(), intervals: []shiftIntervalInfo{{sortOrder: 0, startTime: &ambiguousStart, endTime: &ambiguousEnd}}}, "2026-11-01", "America/New_York")
	if err == nil {
		t.Fatal("ambiguous DST local time accepted without UTC offset")
	}
	nonexistentStart, nonexistentEnd := "02:30:00", "03:30:00"
	_, err = previewIntervals(shiftVersionInfo{id: uuid.New(), intervals: []shiftIntervalInfo{{sortOrder: 0, startTime: &nonexistentStart, endTime: &nonexistentEnd}}}, "2026-03-08", "America/New_York")
	if err == nil {
		t.Fatal("nonexistent DST local time accepted")
	}
}

func TestPreviewIntervalsUsesTimezoneAcrossDSTBoundary(t *testing.T) {
	start, end := "01:30:00", "03:30:00"
	version := shiftVersionInfo{id: uuid.New(), intervals: []shiftIntervalInfo{{sortOrder: 0, startTime: &start, endTime: &end}}}
	intervals, err := previewIntervals(version, "2026-03-08", "America/New_York")
	if err != nil {
		t.Fatalf("preview DST interval: %v", err)
	}
	startAt := intervals[0]["start_at"].(time.Time)
	endAt := intervals[0]["end_at"].(time.Time)
	if startAt.Location().String() != "America/New_York" || endAt.Location().String() != "America/New_York" {
		t.Fatalf("timezone lost: start=%s end=%s", startAt.Location(), endAt.Location())
	}
	if endAt.Sub(startAt) != 1*time.Hour {
		t.Fatalf("DST elapsed duration = %s, want 1h", endAt.Sub(startAt))
	}
}

func TestSanitizedPreviewDayExplicitOffMissingAndConflict(t *testing.T) {
	cases := []struct{ name, source, status string }{{"explicit off", "roster", "explicit_off"}, {"missing", "none", "missing"}, {"conflict", "conflict", "conflict"}}
	for _, test := range cases {
		t.Run(test.name, func(t *testing.T) {
			day, err := sanitizedPreviewDay(resolvedScheduleDay{date: "2026-11-16", shiftIDs: []*uuid.UUID{nil}, source: test.source, timezone: "UTC"}, nil)
			if err != nil {
				t.Fatal(err)
			}
			if day["status"] != test.status {
				t.Fatalf("expected %s, got %+v", test.status, day)
			}
		})
	}
}

func TestPublishPreviewPopulatesResponse(t *testing.T) {
	actorID := uuid.New()
	employmentID := uuid.New()
	captured := make([]string, 0)
	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if strings.Contains(query, "MAX(revision_number)") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*int64)) = 1
					return nil
				}}
			}
			return fakeRow{scanFn: func(dest ...any) error {
				return errors.New("unexpected query row")
			}}
		},
		execFn: func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
			captured = append(captured, query)
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}}
	preview := lockedPreview{ResultSnapshot: jsonArg(map[string]any{
		"data": []map[string]any{{"employment_id": employmentID.String(), "business_date": "2026-11-16", "status": "explicit_off", "source": "roster"}},
	})}
	response := api.SchedulePublicationResponse{}
	if err := publishPreview(context.Background(), tx, publicationActor{ID: &actorID, Type: "human"}, "publish-key", strings.Repeat("a", 64), api.SchedulePublishRequest{Timezone: "UTC", Reason: "approved", StartDate: "2026-11-16", EndDate: "2026-11-16"}, preview, &response); err != nil {
		t.Fatalf("publish preview: %v", err)
	}
	if response.BatchID == uuid.Nil || response.Status != "published" || len(response.RevisionIDs) != 1 || response.AffectedDays != 1 {
		t.Fatalf("publish response was not populated: %+v", response)
	}
	if len(captured) == 0 {
		t.Fatal("expected publication writes")
	}
}

func TestInsertPublishedDayCarriesPreviousRevisionID(t *testing.T) {
	previousID := uuid.New()
	employmentID := uuid.New()
	var insertArgs []any
	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "MAX(revision_number)") {
				return fakeRow{}
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 2
				return nil
			}}
		},
		execFn: func(_ context.Context, query string, args ...any) (pgconn.CommandTag, error) {
			if strings.Contains(query, "INSERT INTO schedule_day_revisions") {
				insertArgs = args
			}
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}}
	_, err := insertPublishedDay(context.Background(), tx, uuid.New(), publicationActor{Type: "human"}, "amend", "UTC", map[string]any{
		"employment_id": employmentID.String(), "business_date": "2026-11-16", "status": "explicit_off", "source": "manual",
	}, time.Date(2026, 11, 16, 12, 0, 0, 0, time.UTC), &previousID)
	if err != nil {
		t.Fatalf("insert published day: %v", err)
	}
	if len(insertArgs) < 14 || insertArgs[12] != &previousID {
		t.Fatalf("previous revision argument missing: %#v", insertArgs)
	}
}

func TestValidateExpectedRevisionsDetectsMismatch(t *testing.T) {
	employmentID := uuid.New()
	date := time.Date(2026, 11, 16, 0, 0, 0, 0, time.UTC)
	key := employmentID.String() + ":" + date.Format("2006-01-02")
	err := validateExpectedRevisions(map[string]int64{key: 2}, map[string]int64{key: 3}, []uuid.UUID{employmentID}, date, date)
	if !errors.Is(err, ErrScheduleRevisionStale) {
		t.Fatalf("expected revision mismatch, got %v", err)
	}
}

func TestValidateExpectedRevisionsRejectsExtraKeys(t *testing.T) {
	employmentID := uuid.New()
	date := time.Date(2026, 11, 16, 0, 0, 0, 0, time.UTC)
	key := employmentID.String() + ":" + date.Format("2006-01-02")
	err := validateExpectedRevisions(map[string]int64{key: 0, "extra": 0}, map[string]int64{}, []uuid.UUID{employmentID}, date, date)
	if !errors.Is(err, ErrSchedulePreviewScope) {
		t.Fatalf("expected scope error, got %v", err)
	}
}

func TestScheduleLifecyclePreviewReplanRollsBackOnStaleRevision(t *testing.T) {
	employmentID := uuid.New()
	currentID := uuid.New()
	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "FROM schedule_day_revisions") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = currentID
					*(dest[1].(*uuid.UUID)) = employmentID
					*(dest[2].(*uuid.UUID)) = uuid.New()
					*(dest[3].(*string)) = "2026-11-16"
					*(dest[4].(*string)) = "UTC"
					*(dest[5].(*int64)) = 4
					return nil
				}}
			}
			return fakeRow{}
		},
	}}
	repo := ScheduleRepository{DB: tx}

	_, err := repo.PreviewReplan(context.Background(), uuid.New(), employmentID, "2026-11-16", api.ScheduleReplanPreviewRequest{
		ExpectedRevision: 3,
		Reason:           "Correct schedule",
		Replacement: api.ScheduleReplanReplacement{
			Status:         "work",
			Source:         "manual",
			ShiftVersionID: new(uuid.UUID),
			Intervals:      []api.ScheduleReplanInterval{{SortOrder: 0, Start: "08:00", End: "17:00"}},
		},
	})
	if !errors.Is(err, ErrScheduleRevisionStale) {
		t.Fatalf("expected stale revision, got %v", err)
	}
	if tx.committed {
		t.Fatal("stale preview transaction must not commit")
	}
}

func TestScheduleLifecyclePublishReplaysBeforePreviewAndGuards(t *testing.T) {
	actorID := uuid.New()
	batchID := uuid.New()
	request := api.SchedulePublishRequest{
		EmploymentIDs:      []uuid.UUID{uuid.New()},
		StartDate:          "2026-11-16",
		EndDate:            "2026-11-16",
		Timezone:           "UTC",
		ApprovalInstanceID: new(uuid.UUID),
		PreviewID:          uuid.New(),
		PreviewHash:        strings.Repeat("a", 64),
		Reason:             "Approved schedule",
		ExpectedRevisions:  map[string]int64{},
	}
	expected, err := hashJSON(publishHashInput(actorID, request, "", ""))
	if err != nil {
		t.Fatal(err)
	}
	storedResponse := api.SchedulePublicationResponse{BatchID: batchID, Status: "published", PublishedAt: time.Date(2026, 11, 13, 1, 0, 0, 0, time.UTC), AffectedDays: 1}
	storedJSON, err := json.Marshal(storedResponse)
	if err != nil {
		t.Fatal(err)
	}
	queryCount := 0
	tx := &fakeTx{fakeDBTX: fakeDBTX{
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			if !strings.Contains(sql, "pg_advisory_xact_lock") {
				t.Fatalf("replay must not write before lookup, got %s", sql)
			}
			return pgconn.NewCommandTag("SELECT 1"), nil
		},
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			queryCount++
			if !strings.Contains(sql, "FROM schedule_publication_batches") {
				t.Fatalf("expected idempotency lookup first, got %s", sql)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = batchID
				*(dest[1].(*string)) = expected
				*(dest[2].(**uuid.UUID)) = &actorID
				*(dest[3].(*string)) = "human"
				*(dest[4].(**string)) = nil
				*(dest[5].(*[]byte)) = storedJSON
				return nil
			}}
		},
	}}
	repo := ScheduleRepository{DB: tx}
	response, err := repo.Publish(context.Background(), actorID, "replay-key", request)
	if err != nil {
		t.Fatalf("expected replay, got %v", err)
	}
	if response.BatchID != batchID || queryCount != 1 {
		t.Fatalf("unexpected replay response=%+v query_count=%d", response, queryCount)
	}
	if tx.committed == false {
		t.Fatal("idempotent replay must commit read-only transaction")
	}
}
