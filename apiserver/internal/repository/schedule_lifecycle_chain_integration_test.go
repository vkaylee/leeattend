//go:build integration

package repository

import (
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

func TestIntegrationFullScheduleLifecycleChain(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID := uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}

	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'full-lifecycle-employer')`, employerID)
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, employeeID.String(), "full-lifecycle-employee", "2026-01-01", "2026-11-20", "2026-11-20")
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO payroll_periods (id, employer_id, period_year, period_month, status, version) VALUES ($1, $2, 2026, 11, 'open', 1)`, uuid.New(), employerID)
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'full-lifecycle-template')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'full-lifecycle-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, ids.UserID(actorID))
	mustExec(`INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, '08:00', '17:00')`, versionID)
	mustExec(`INSERT INTO employee_shifts (id, employment_id, template_id, start_date, end_date) VALUES ($1, $2, $3, DATE '2026-11-20', DATE '2026-11-20')`, uuid.New(), employmentID, templateID)

	scheduleRepo := ScheduleRepository{DB: pool}
	date := "2026-11-20"
	key := employmentID.String() + ":" + date
	preview, err := scheduleRepo.CreatePreview(ctx, actorID, api.SchedulePreviewRequest{
		EmploymentIDs:     []uuid.UUID{employmentID},
		StartDate:         date,
		EndDate:           date,
		Timezone:          "UTC",
		Mode:              "rotating",
		Reason:            "publish rotating schedule",
		ExpectedRevisions: map[string]int64{key: 0},
	})
	if err != nil || !preview.Provisional || preview.PreviewID == uuid.Nil {
		t.Fatalf("preview response=%+v err=%v", preview, err)
	}
	var officialCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_day_revisions WHERE employment_id = $1 AND business_date = $2::date AND state IN ('published', 'explicit_off')`, employmentID, date).Scan(&officialCount); err != nil {
		t.Fatalf("count before publication: %v", err)
	}
	if officialCount != 0 {
		t.Fatalf("preview created %d official revisions", officialCount)
	}

	published, err := scheduleRepo.Publish(ctx, actorID, "full-lifecycle-publish-"+uuid.NewString(), api.SchedulePublishRequest{
		Mode:              "rotating",
		EmploymentIDs:     []uuid.UUID{employmentID},
		StartDate:         date,
		EndDate:           date,
		Timezone:          "UTC",
		PreviewID:         preview.PreviewID,
		PreviewHash:       preview.PreviewHash,
		Reason:            "publish rotating schedule",
		ExpectedRevisions: map[string]int64{key: 0},
	})
	if err != nil || published.AffectedDays != 1 || len(published.RevisionIDs) != 1 {
		t.Fatalf("publication response=%+v err=%v", published, err)
	}

	current, err := scheduleRepo.GetOfficialSchedules(ctx, parseScheduleDate(date), parseScheduleDate(date), []uuid.UUID{employmentID}, 1, 10)
	if err != nil || len(current.Data) != 1 || current.Data[0].Schedules[0].RevisionID == nil {
		t.Fatalf("official schedule=%+v err=%v", current, err)
	}
	oldRevisionID := *current.Data[0].Schedules[0].RevisionID

	replanPreview, err := scheduleRepo.PreviewReplan(ctx, actorID, employmentID, date, api.ScheduleReplanPreviewRequest{
		ExpectedRevision: 1,
		Reason:           "move shift one hour",
		Replacement: api.ScheduleReplanReplacement{
			Status:         "work",
			Source:         "manual",
			ShiftVersionID: &versionID,
			Intervals:      []api.ScheduleReplanInterval{{SortOrder: 0, Start: "09:00", End: "18:00"}},
		},
	})
	if err != nil || replanPreview.PreviewID == uuid.Nil {
		t.Fatalf("replan preview=%+v err=%v", replanPreview, err)
	}
	replanned, err := scheduleRepo.CommitReplan(ctx, actorID, employmentID, date, "full-lifecycle-replan-"+uuid.NewString(), api.ScheduleReplanRequest{
		ExpectedRevision: 1,
		Reason:           "move shift one hour",
		Replacement: api.ScheduleReplanReplacement{
			Status:         "work",
			Source:         "manual",
			ShiftVersionID: &versionID,
			Intervals:      []api.ScheduleReplanInterval{{SortOrder: 0, Start: "09:00", End: "18:00"}},
		},
		PreviewID:       replanPreview.PreviewID,
		PreviewHash:     replanPreview.PreviewHash,
		ConfirmDiffHash: replanPreview.DiffHash,
	})
	t.Logf("replan preview=%+v replacement_hash=%s", replanPreview, replacementHash(api.ScheduleReplanReplacement{Status: "work", Source: "manual", ShiftVersionID: &versionID, Intervals: []api.ScheduleReplanInterval{{SortOrder: 0, Start: "09:00", End: "18:00"}}}, "UTC", date, employmentID))
	if err != nil || replanned.OriginalRevisionID != oldRevisionID || replanned.ReplacementRevisionID == uuid.Nil {
		t.Fatalf("replan response=%+v err=%v", replanned, err)
	}
	newRevisionID := replanned.ReplacementRevisionID

	attendanceRepo := AttendanceRepository{DB: pool}
	closed, err := attendanceRepo.CloseAttendancePeriod(ctx, employerID, 2026, 11, 1, "close after replan", ids.UserID(actorID))
	if err != nil || closed.Status != "closed" || closed.Version != 2 {
		t.Fatalf("close response=%+v err=%v", closed, err)
	}
	reopenActorID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, reopenActorID, reopenActorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert independent reopen actor: %v", err)
	}
	reopened, err := attendanceRepo.ReopenAttendancePeriod(ctx, employerID, 2026, 11, closed.Version, "reopen for correction", ids.UserID(reopenActorID))
	if err != nil || reopened.Status != "reopened" || reopened.Version != 3 {
		t.Fatalf("reopen response=%+v err=%v", reopened, err)
	}
	reclosed, err := attendanceRepo.CloseAttendancePeriod(ctx, employerID, 2026, 11, reopened.Version, "reclose corrected period", ids.UserID(actorID))
	if err != nil || reclosed.Status != "closed" || reclosed.Version != 4 || reclosed.ClosureNumber != 2 {
		t.Fatalf("reclose response=%+v err=%v", reclosed, err)
	}

	evidence, err := scheduleRepo.GetScheduleEvidence(ctx, actorID, employmentID, date)
	if err != nil {
		t.Fatalf("evidence response error: %v", err)
	}
	if len(evidence.Revisions) != 2 || len(evidence.Segments) != 2 || len(evidence.ClosureDays) != 2 || len(evidence.DiffChain) != 2 {
		t.Fatalf("chain evidence counts revisions=%d segments=%d closures=%d diff=%d", len(evidence.Revisions), len(evidence.Segments), len(evidence.ClosureDays), len(evidence.DiffChain))
	}
	if evidence.Revisions[1].RevisionID != newRevisionID || evidence.Revisions[1].PreviousRevisionID == nil || *evidence.Revisions[1].PreviousRevisionID != oldRevisionID {
		t.Fatalf("evidence revision lineage=%+v", evidence.Revisions)
	}
}
