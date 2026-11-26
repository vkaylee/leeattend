package repository

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type resolveUnscheduledHashInput struct {
	Operation    string `json:"operation"`
	ActorID      string `json:"actor_id"`
	EmploymentID string `json:"employment_id"`
	BusinessDate string `json:"business_date"`
	ShiftID      string `json:"shift_id"`
	Reason       string `json:"reason"`
	Timezone     string `json:"timezone"`
}

// ResolveUnscheduled atomically publishes an official manual work revision for an
// unassigned business date and pairs/recalculates attendance for that employee/day
// within a single database transaction.
func (r ScheduleRepository) ResolveUnscheduled(
	ctx context.Context,
	actorID, employmentID uuid.UUID,
	businessDate, idempotencyKey string,
	request api.ResolveUnscheduledScheduleRequest,
) (api.ResolveUnscheduledScheduleResponse, error) {
	trimmedReason := strings.TrimSpace(request.Reason)
	idempotencyKey = strings.TrimSpace(idempotencyKey)
	if actorID == uuid.Nil || employmentID == uuid.Nil || request.ShiftID == uuid.Nil || trimmedReason == "" || len(trimmedReason) > 2000 || idempotencyKey == "" || len(idempotencyKey) > 128 || !isASCIIKey(idempotencyKey) {
		return api.ResolveUnscheduledScheduleResponse{}, api.ErrScheduleValidation
	}
	parsedDate, err := time.Parse("2006-01-02", businessDate)
	if err != nil {
		return api.ResolveUnscheduledScheduleResponse{}, api.ErrScheduleValidation
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return api.ResolveUnscheduledScheduleResponse{}, err
	}
	requestHash, err := hashJSON(resolveUnscheduledHashInput{
		Operation:    "resolve_unscheduled",
		ActorID:      actorID.String(),
		EmploymentID: employmentID.String(),
		BusinessDate: businessDate,
		ShiftID:      request.ShiftID.String(),
		Reason:       trimmedReason,
		Timezone:     strings.TrimSpace(request.Timezone),
	})
	if err != nil {
		return api.ResolveUnscheduledScheduleResponse{}, err
	}
	var response api.ResolveUnscheduledScheduleResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := lockIdempotencyKey(ctx, tx, idempotencyKey); err != nil {
			return err
		}
		if stored, found, err := loadBatchByIdempotency(ctx, tx, idempotencyKey); err != nil {
			return err
		} else if found {
			return replayResolveUnscheduled(actorID, requestHash, stored, &response)
		}
		return r.resolveUnscheduledTx(ctx, tx, actorID, employmentID, businessDate, parsedDate, idempotencyKey, requestHash, request.ShiftID, trimmedReason, strings.TrimSpace(request.Timezone), &response)
	})
	if err != nil {
		return api.ResolveUnscheduledScheduleResponse{}, err
	}
	return response, nil
}

func replayResolveUnscheduled(actorID uuid.UUID, requestHash string, stored storedBatch, response *api.ResolveUnscheduledScheduleResponse) error {
	if stored.ActorType != "human" || stored.RequestedBy == nil || *stored.RequestedBy != actorID {
		return api.ErrScheduleForbidden
	}
	if strings.TrimSpace(stored.RequestHash) != requestHash {
		return ErrScheduleIdempotencyConflict
	}
	return json.Unmarshal(stored.ResponseSnapshot, response)
}

func (r ScheduleRepository) resolveUnscheduledTx(
	ctx context.Context,
	tx pgx.Tx,
	actorID, employmentID uuid.UUID,
	businessDate string,
	parsedDate time.Time,
	idempotencyKey, requestHash string,
	shiftID uuid.UUID,
	reason, requestedTimezone string,
	response *api.ResolveUnscheduledScheduleResponse,
) error {
	// Acquire per-employment advisory lock BEFORE row locks (same order as ESSAttendanceAction/PunchIn/ReprocessAttendance)
	// to prevent AB-BA deadlock between employments FOR UPDATE and FK FOR KEY SHARE.
	if _, err := tx.Exec(ctx, `SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))`, employmentID.String()); err != nil {
		return err
	}
	if err := lockScheduleRevisionTargets(ctx, tx, []scheduleRevisionLockKey{{employmentID: employmentID, businessDate: businessDate}}); err != nil {
		return err
	}
	var employerID uuid.UUID
	var primaryLocationID *uuid.UUID
	err := tx.QueryRow(ctx, `SELECT employer_id, primary_location_id
		FROM employment_segments
		WHERE id = $1 AND deleted_at IS NULL
		  AND start_date <= $2::date
		  AND (end_date IS NULL OR end_date >= $2::date)`, employmentID, businessDate).Scan(&employerID, &primaryLocationID)
	if errors.Is(err, pgx.ErrNoRows) {
		return ErrSchedulePreviewScope
	}
	if err != nil {
		return err
	}
	effectiveTZ, err := resolveEmploymentDateTimezone(ctx, tx, employmentID, primaryLocationID, businessDate)
	if err != nil {
		return err
	}
	if requestedTimezone != "" && requestedTimezone != effectiveTZ {
		return ErrScheduleTimezoneMismatch
	}
	if err := ensureMutationPeriodOpenForRange(ctx, tx, []uuid.UUID{employmentID}, businessDate, businessDate); err != nil {
		return err
	}
	if err := verifyFirstPublicationTargets(ctx, tx, api.SchedulePublishRequest{
		EmploymentIDs: []uuid.UUID{employmentID},
		StartDate:     businessDate,
		EndDate:       businessDate,
	}); err != nil {
		return err
	}
	shifts, err := loadShifts(ctx, tx)
	if err != nil {
		return err
	}
	version, selected, conflict := selectEffectiveShiftVersion(shifts, shiftID, parsedDate)
	if conflict {
		return api.ErrScheduleConflict
	}
	if !selected || len(version.intervals) == 0 {
		return api.ErrScheduleValidation
	}
	var shiftEmployerID *uuid.UUID
	if err := tx.QueryRow(ctx, `SELECT employer_id FROM shift_versions WHERE id = $1`, version.id).Scan(&shiftEmployerID); err != nil {
		return err
	}
	if shiftEmployerID != nil && *shiftEmployerID != employerID {
		return api.ErrScheduleValidation
	}
	var hasProtectedOT bool
	err = tx.QueryRow(ctx, `SELECT (ot_status IN ('approved', 'pending_manager') OR claimed_ot_minutes > 0 OR approved_ot_minutes > 0)
		FROM attendance_daily_summaries
		WHERE employment_id = $1 AND date = $2::date
		FOR UPDATE`, employmentID, businessDate).Scan(&hasProtectedOT)
	if err != nil && !errors.Is(err, pgx.ErrNoRows) {
		return err
	}
	if hasProtectedOT {
		return api.ErrScheduleConflict
	}
	intervals, err := previewIntervals(version, businessDate, effectiveTZ)
	if err != nil {
		return err
	}
	day := map[string]any{
		"employment_id":    employmentID.String(),
		"business_date":    businessDate,
		"status":           "work",
		"source":           "manual",
		"timezone":         effectiveTZ,
		"shift_version_id": version.id.String(),
		"intervals":        intervals,
		"trace":            []any{},
		"warnings":         []string{},
	}
	if err := rejectConcreteScheduleOverlaps(ctx, tx, []map[string]any{day}); err != nil {
		return err
	}
	if err := ensureUnscheduledAttendanceEvidence(ctx, tx, employmentID, businessDate, effectiveTZ, day); err != nil {
		return err
	}

	batchID := uuid.New()
	publishedAt := time.Now().UTC()
	actor := publicationActor{ID: &actorID, Type: "human"}
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, actor_type, actor_name, approval_instance_id, reason, timezone, range_start, range_end, published_at, affected_days) VALUES ($1, $2, $3, 'rotating', 'published', $4, 'publish', '{}'::jsonb, 'human', NULL, NULL, $5, $6, $7::date, $7::date, $8, 1)`, batchID, idempotencyKey, requestHash, actorID, reason, effectiveTZ, businessDate, publishedAt); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		return err
	}
	revisionID, err := insertPublishedDay(ctx, tx, batchID, actor, reason, effectiveTZ, day, publishedAt, nil)
	if err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, created_at, actor_type, actor_name) VALUES ($1, $2, 'schedule.published', $3, CURRENT_TIMESTAMP, 'human', NULL)`, actorID, batchID, jsonArg(map[string]any{"batch_id": batchID.String(), "revision_ids": []uuid.UUID{revisionID}, "affected_days": 1, "source": "resolve_unscheduled"})); err != nil {
		return err
	}
	if err := reconcileUnlinkedTimeEntries(ctx, tx, employmentID, revisionID, businessDate, effectiveTZ); err != nil {
		return err
	}
	if err := pairRawLogsForEmployment(ctx, tx, businessDate, businessDate, employmentID); err != nil {
		return err
	}
	groups, err := loadReprocessDailyGroupsForEmployment(ctx, tx, businessDate, businessDate, employmentID)
	if err != nil {
		return err
	}
	for _, item := range groups {
		if err := upsertReprocessSummary(ctx, tx, item); err != nil {
			return err
		}
	}
	resolved := api.ResolveUnscheduledScheduleResponse{
		BatchID:      batchID,
		RevisionID:   revisionID,
		PublishedAt:  publishedAt,
		Recalculated: len(groups),
	}
	responseJSON, err := json.Marshal(resolved)
	if err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_publication_batches SET response_snapshot = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $1`, batchID, responseJSON); err != nil {
		return err
	}
	*response = resolved
	return nil
}

func resolveEmploymentDateTimezone(ctx context.Context, tx DBTX, employmentID uuid.UUID, primaryLocationID *uuid.UUID, businessDate string) (string, error) {
	assignments, err := loadAssignments(ctx, tx, []uuid.UUID{employmentID}, businessDate, businessDate)
	if err != nil {
		return "", err
	}
	locationID := primaryLocationID
	if list := assignments[employmentID]; len(list) > 0 {
		id := list[0].hostLocationID
		locationID = &id
	}
	if locationID == nil {
		return "UTC", nil
	}
	zones, err := loadLocationTimezones(ctx, tx, []uuid.UUID{*locationID})
	if err != nil {
		return "", err
	}
	return resolvedTimezone(locationID, zones), nil
}

func ensureUnscheduledAttendanceEvidence(ctx context.Context, tx DBTX, employmentID uuid.UUID, businessDate, timezone string, day map[string]any) error {
	intervals, err := dayIntervals(day, timezone, businessDate)
	if err != nil || len(intervals) == 0 {
		return api.ErrScheduleValidation
	}
	minStart, maxEnd := intervals[0].StartAt, intervals[0].EndAt
	for _, iv := range intervals[1:] {
		if iv.StartAt.Before(minStart) {
			minStart = iv.StartAt
		}
		if iv.EndAt.After(maxEnd) {
			maxEnd = iv.EndAt
		}
	}
	var hasEvidence bool
	err = tx.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1 FROM time_entries
		WHERE employment_id = $1 AND segment_id IS NULL AND entry_type = 'work'
		  AND ((punch_in_time >= $2::timestamptz - INTERVAL '2 hours' AND punch_in_time <= $3)
		       OR (punch_in_time AT TIME ZONE $5)::date = $4::date)
		UNION ALL
		SELECT 1 FROM raw_attendance_logs
		WHERE employment_id = $1
		  AND punch_time >= $2::timestamptz - INTERVAL '2 hours' AND punch_time <= $3::timestamptz + INTERVAL '2 hours'
	)`, employmentID, minStart, maxEnd, businessDate, timezone).Scan(&hasEvidence)
	if err != nil {
		return err
	}
	if !hasEvidence {
		return api.ErrScheduleValidation
	}
	return nil
}

// reconcileUnlinkedTimeEntries pins pre-existing unscheduled work entries belonging
// to this business date/window strictly to segments of the newly published revisionID.
// It never touches segments from any other revision (e.g., an adjacent closed period).
func reconcileUnlinkedTimeEntries(ctx context.Context, tx DBTX, employmentID, revisionID uuid.UUID, businessDate, timezone string) error {
	type revSegment struct {
		id      uuid.UUID
		startAt time.Time
		endAt   time.Time
	}
	segRows, err := tx.Query(ctx, `SELECT id, scheduled_start_at, scheduled_end_at
		FROM work_segments
		WHERE schedule_revision_id = $1 AND segment_status <> 'cancelled'
		  AND scheduled_start_at IS NOT NULL AND scheduled_end_at IS NOT NULL
		ORDER BY scheduled_start_at, id`, revisionID)
	if err != nil {
		return err
	}
	var segments []revSegment
	for segRows.Next() {
		var seg revSegment
		if err := segRows.Scan(&seg.id, &seg.startAt, &seg.endAt); err != nil {
			segRows.Close()
			return err
		}
		segments = append(segments, seg)
	}
	if err := segRows.Err(); err != nil {
		segRows.Close()
		return err
	}
	segRows.Close()
	if len(segments) == 0 {
		return nil
	}
	minStart, maxEnd := segments[0].startAt, segments[0].endAt
	for _, seg := range segments[1:] {
		if seg.startAt.Before(minStart) {
			minStart = seg.startAt
		}
		if seg.endAt.After(maxEnd) {
			maxEnd = seg.endAt
		}
	}

	// Select unlinked work entries within this revision's concrete window OR on local
	// businessDate (excluding entries already covered by another published revision's segment).
	rows, err := tx.Query(ctx, `SELECT id, punch_in_time
		FROM time_entries te
		WHERE te.employment_id = $1 AND te.segment_id IS NULL AND te.entry_type = 'work'
		  AND (
		      (te.punch_in_time >= $2::timestamptz - INTERVAL '2 hours' AND te.punch_in_time <= $3)
		      OR (
		          (te.punch_in_time AT TIME ZONE $5)::date = $4::date
		          AND NOT EXISTS (
		              SELECT 1 FROM work_segments other_ws
		              JOIN schedule_day_revisions other_sdr ON other_sdr.id = other_ws.schedule_revision_id
		              WHERE other_ws.employment_id = $1
		                AND other_ws.schedule_revision_id <> $6
		                AND other_sdr.state = 'published'
		                AND other_ws.segment_status <> 'cancelled'
		                AND other_ws.scheduled_start_at IS NOT NULL AND other_ws.scheduled_end_at IS NOT NULL
		                AND other_ws.scheduled_start_at - INTERVAL '2 hours' <= te.punch_in_time
		                AND other_ws.scheduled_end_at >= te.punch_in_time
		          )
		      )
		  )
		ORDER BY te.punch_in_time, te.id`, employmentID, minStart, maxEnd, businessDate, timezone, revisionID)
	if err != nil {
		return err
	}
	type unlinkedEntry struct {
		id      uuid.UUID
		punchIn time.Time
	}
	var pending []unlinkedEntry
	for rows.Next() {
		var entry unlinkedEntry
		if err := rows.Scan(&entry.id, &entry.punchIn); err != nil {
			rows.Close()
			return err
		}
		pending = append(pending, entry)
	}
	if err := rows.Err(); err != nil {
		rows.Close()
		return err
	}
	rows.Close()

	matchRevisionSegment := func(punchIn time.Time) *uuid.UUID {
		var exact []uuid.UUID
		for _, seg := range segments {
			if !punchIn.Before(seg.startAt) && !punchIn.After(seg.endAt) {
				exact = append(exact, seg.id)
			}
		}
		if len(exact) == 1 {
			return &exact[0]
		}
		if len(exact) > 1 {
			return nil
		}
		var early []uuid.UUID
		for _, seg := range segments {
			if !punchIn.Before(seg.startAt.Add(-2*time.Hour)) && punchIn.Before(seg.startAt) {
				early = append(early, seg.id)
			}
		}
		if len(early) == 1 {
			return &early[0]
		}
		return nil
	}

	for _, entry := range pending {
		segID := matchRevisionSegment(entry.punchIn)
		if segID == nil {
			return api.ErrScheduleConflict
		}
		if _, err := tx.Exec(ctx, `UPDATE time_entries
			SET segment_id = $2,
			    template_id = (SELECT template_id FROM work_segments WHERE id = $2),
			    shift_version_id = (SELECT selected_shift_version_id FROM work_segments WHERE id = $2),
			    breaks_snapshot = (SELECT breaks_snapshot FROM work_segments WHERE id = $2),
			    updated_at = CURRENT_TIMESTAMP
			WHERE id = $1`, entry.id, *segID); err != nil {
			return err
		}
	}
	return nil
}
