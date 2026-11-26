package repository

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"sort"
	"strings"
	"time"
)

func persistReplanPreview(ctx context.Context, tx pgx.Tx, actorID, employmentID uuid.UUID, businessDate string, request api.ScheduleReplanPreviewRequest, current lifecycleRevision, before, replacement map[string]any) (api.ScheduleReplanPreviewResponse, error) {
	diff := map[string]any{"before": before, "after": replacement}
	diffHash, err := hashJSON(map[string]any{
		"original_revision_id": current.ID.String(),
		"expected_revision":    request.ExpectedRevision,
		"diff":                 diff,
	})
	if err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	scope := map[string]any{"employment_id": employmentID.String(), "business_date": businessDate, "timezone": current.Timezone}
	result := map[string]any{
		"operation": "replan", "original_revision_id": current.ID.String(),
		"expected_revision": request.ExpectedRevision, "replacement": replacement,
		"diff": diff, "diff_hash": diffHash,
	}
	requestHash, err := hashJSON(replanHashInput(actorID, employmentID, businessDate, request, "", "", ""))
	if err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	resultHash, err := hashJSON(result)
	if err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	previewID := uuid.New()
	createdAt := time.Now().UTC()
	expiresAt := createdAt.Add(previewTTL)
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_previews (id, requested_by, actor_type, operation, request_hash, result_hash, scope_snapshot, result_snapshot, resolver_version, expires_at, created_at) VALUES ($1, $2, 'human', 'replan', $3, $4, $5, $6, $7, $8, $9)`, previewID, actorID, requestHash, resultHash, jsonArg(scope), jsonArg(result), lifecycleResolverVersion, expiresAt, createdAt); err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	return api.ScheduleReplanPreviewResponse{
		PreviewID: previewID, PreviewHash: resultHash, ExpiresAt: expiresAt,
		OriginalRevisionID: current.ID, Replacement: replacement, Diff: diff, DiffHash: diffHash,
	}, nil
}

func previewReplanTx(ctx context.Context, tx pgx.Tx, actorID, employmentID uuid.UUID, businessDate string, request api.ScheduleReplanPreviewRequest) (api.ScheduleReplanPreviewResponse, error) {
	if _, err := tx.Exec(ctx, `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ`); err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	current, err := loadCurrentRevision(ctx, tx, employmentID, businessDate, false)
	if err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	if current.RevisionNumber != request.ExpectedRevision {
		return api.ScheduleReplanPreviewResponse{}, ErrScheduleRevisionStale
	}
	if err := ensureMutationPeriodOpen(ctx, tx, current.EmployerID, businessDate); err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	before, err := loadRevisionDocument(ctx, tx, current)
	if err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	replacement, err := normalizeReplacement(request.Replacement, current.Timezone, businessDate, employmentID)
	if err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	return persistReplanPreview(ctx, tx, actorID, employmentID, businessDate, request, current, before, replacement)
}

func (r ScheduleRepository) PreviewReplan(ctx context.Context, actorID, employmentID uuid.UUID, businessDate string, request api.ScheduleReplanPreviewRequest) (api.ScheduleReplanPreviewResponse, error) {
	if err := validateReplanInput(employmentID, businessDate, request); err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	var response api.ScheduleReplanPreviewResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		resp, err := previewReplanTx(ctx, tx, actorID, employmentID, businessDate, request)
		if err != nil {
			return err
		}
		response = resp
		return nil
	})
	if err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	return response, nil
}

func validateReplanPreviewMatch(preview lockedPreview, current lifecycleRevision, businessDate string, employmentID uuid.UUID, request api.ScheduleReplanRequest) (replanResult, error) {
	result, err := decodeReplanResult(preview.ResultSnapshot)
	if err != nil {
		return replanResult{}, err
	}
	storedReplacementHash := canonicalStoredReplacementHash(result.Replacement, current.Timezone, businessDate, employmentID)
	if result.DiffHash != request.ConfirmDiffHash || storedReplacementHash == "" || storedReplacementHash != replacementHash(request.Replacement, current.Timezone, businessDate, employmentID) {
		return replanResult{}, ErrSchedulePreviewStale
	}
	return result, nil
}

func lockAndLoadReplanTarget(ctx context.Context, tx pgx.Tx, employmentID uuid.UUID, businessDate string, expectedRevision int64) (lifecycleRevision, error) {
	if err := lockScheduleRevisionTargets(ctx, tx, []scheduleRevisionLockKey{{employmentID: employmentID, businessDate: businessDate}}); err != nil {
		return lifecycleRevision{}, err
	}
	current, err := loadCurrentRevision(ctx, tx, employmentID, businessDate, true)
	if err != nil {
		return lifecycleRevision{}, err
	}
	if current.RevisionNumber != expectedRevision {
		return lifecycleRevision{}, ErrScheduleRevisionStale
	}
	if err := ensureMutationPeriodOpen(ctx, tx, current.EmployerID, businessDate); err != nil {
		return lifecycleRevision{}, err
	}
	return current, nil
}

func commitReplanTx(ctx context.Context, tx pgx.Tx, actorID, employmentID uuid.UUID, businessDate, idempotencyKey string, request api.ScheduleReplanRequest, response *api.ScheduleReplanResponse) error {
	if err := lockIdempotencyKey(ctx, tx, idempotencyKey); err != nil {
		return err
	}
	requestHash, err := hashJSON(replanHashInput(actorID, employmentID, businessDate, api.ScheduleReplanPreviewRequest{ExpectedRevision: request.ExpectedRevision, Reason: request.Reason, Replacement: request.Replacement}, request.PreviewID.String(), request.PreviewHash, request.ConfirmDiffHash))
	if err != nil {
		return err
	}
	stored, found, err := loadBatchByIdempotency(ctx, tx, idempotencyKey)
	if err != nil {
		return err
	}
	if found {
		return replayReplan(actorID, requestHash, stored, response)
	}
	preview, err := lockPreview(ctx, tx, actorID, request.PreviewID, "replan", request.PreviewHash)
	if err != nil {
		return err
	}
	current, err := lockAndLoadReplanTarget(ctx, tx, employmentID, businessDate, request.ExpectedRevision)
	if err != nil {
		return err
	}
	result, err := validateReplanPreviewMatch(preview, current, businessDate, employmentID, request)
	if err != nil {
		return err
	}
	return commitReplan(ctx, tx, actorID, idempotencyKey, requestHash, request, current, result, response)
}

func (r ScheduleRepository) CommitReplan(ctx context.Context, actorID, employmentID uuid.UUID, businessDate, idempotencyKey string, request api.ScheduleReplanRequest) (api.ScheduleReplanResponse, error) {
	if err := validateCommitReplanInput(employmentID, businessDate, idempotencyKey, request); err != nil {
		return api.ScheduleReplanResponse{}, err
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return api.ScheduleReplanResponse{}, err
	}
	var response api.ScheduleReplanResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		return commitReplanTx(ctx, tx, actorID, employmentID, businessDate, idempotencyKey, request, &response)
	})
	if err != nil {
		return api.ScheduleReplanResponse{}, err
	}
	return response, nil
}

type normalizedInterval struct {
	SortOrder             int              `json:"sort_order"`
	StartLocal            string           `json:"start"`
	EndLocal              string           `json:"end"`
	StartDayOffset        int              `json:"start_day_offset"`
	EndDayOffset          int              `json:"end_day_offset"`
	StartUTCOffsetMinutes *int             `json:"start_utc_offset_minutes,omitempty"`
	EndUTCOffsetMinutes   *int             `json:"end_utc_offset_minutes,omitempty"`
	StartAt               time.Time        `json:"start_at"`
	EndAt                 time.Time        `json:"end_at"`
	ShiftVersionID        uuid.UUID        `json:"shift_version_id"`
	Breaks                []map[string]any `json:"breaks"`
	BreakPunchMode        string           `json:"break_punch_mode"`
}

func matchesWallClock(inLocation, date, clock time.Time) bool {
	return inLocation.Year() == date.Year() && inLocation.Month() == date.Month() && inLocation.Day() == date.Day() &&
		inLocation.Hour() == clock.Hour() && inLocation.Minute() == clock.Minute() && inLocation.Second() == clock.Second()
}

func appendUniqueInstant(candidates []time.Time, instant time.Time) []time.Time {
	for _, candidate := range candidates {
		if candidate.Equal(instant) {
			return candidates
		}
	}
	return append(candidates, instant)
}

func localDateTime(date time.Time, clock time.Time, explicitOffset *int) (time.Time, error) {
	loc := date.Location()
	candidates := make([]time.Time, 0, 2)
	for offset := -14 * 60; offset <= 14*60; offset += 15 {
		instant := time.Date(date.Year(), date.Month(), date.Day(), clock.Hour(), clock.Minute(), clock.Second(), 0, time.FixedZone("candidate", offset*60))
		if matchesWallClock(instant.In(loc), date, clock) {
			candidates = appendUniqueInstant(candidates, instant)
		}
	}
	if len(candidates) == 0 {
		return time.Time{}, errors.New("schedule local time does not exist in timezone")
	}
	if explicitOffset != nil {
		for _, candidate := range candidates {
			if _, offset := candidate.In(loc).Zone(); offset == *explicitOffset*60 {
				return candidate.In(loc), nil
			}
		}
		return time.Time{}, errors.New("schedule UTC offset does not disambiguate local time")
	}
	if len(candidates) > 1 {
		return time.Time{}, errors.New("schedule local time is ambiguous; explicit UTC offset required")
	}
	return candidates[0].In(loc), nil
}

func normalizeReplacementInterval(item api.ScheduleReplanInterval, shiftVersionID uuid.UUID, businessDate string, loc *time.Location) (normalizedInterval, error) {
	start, err := parseLocalTime(item.Start)
	if err != nil {
		return normalizedInterval{}, err
	}
	end, err := parseLocalTime(item.End)
	if err != nil {
		return normalizedInterval{}, err
	}
	if item.StartDayOffset < 0 || item.EndDayOffset < item.StartDayOffset {
		return normalizedInterval{}, errors.New("schedule replacement day offsets are invalid")
	}
	year, month, day := parseDateParts(businessDate)
	startAt, err := localDateTime(time.Date(year, month, day+item.StartDayOffset, 0, 0, 0, 0, loc), start, item.StartUTCOffsetMinutes)
	if err != nil {
		return normalizedInterval{}, err
	}
	endAt, err := localDateTime(time.Date(year, month, day+item.EndDayOffset, 0, 0, 0, 0, loc), end, item.EndUTCOffsetMinutes)
	if err != nil {
		return normalizedInterval{}, err
	}
	if !endAt.After(startAt) {
		return normalizedInterval{}, errors.New("schedule lifecycle replacement interval ends before it starts")
	}
	breaks, err := normalizeBreaks(item.Breaks, startAt, loc)
	if err != nil {
		return normalizedInterval{}, err
	}
	return normalizedInterval{
		SortOrder: item.SortOrder, StartLocal: start.Format("15:04:05"), EndLocal: end.Format("15:04:05"),
		StartDayOffset: item.StartDayOffset, EndDayOffset: item.EndDayOffset,
		StartUTCOffsetMinutes: item.StartUTCOffsetMinutes, EndUTCOffsetMinutes: item.EndUTCOffsetMinutes,
		StartAt: startAt, EndAt: endAt, ShiftVersionID: shiftVersionID,
		Breaks: breaks, BreakPunchMode: valueOrDefault(&item.BreakPunchMode, "none"),
	}, nil
}

func normalizeReplacement(replacement api.ScheduleReplanReplacement, timezone, businessDate string, employmentID uuid.UUID) (map[string]any, error) {
	loc, err := time.LoadLocation(timezone)
	if err != nil {
		return nil, err
	}
	seen := make(map[int]struct{}, len(replacement.Intervals))
	intervals := make([]normalizedInterval, 0, len(replacement.Intervals))
	for _, item := range replacement.Intervals {
		if _, exists := seen[item.SortOrder]; exists {
			return nil, errors.New("schedule lifecycle replacement has duplicate interval sort order")
		}
		seen[item.SortOrder] = struct{}{}
		normalized, err := normalizeReplacementInterval(item, *replacement.ShiftVersionID, businessDate, loc)
		if err != nil {
			return nil, err
		}
		intervals = append(intervals, normalized)
	}
	sort.Slice(intervals, func(i, j int) bool { return intervals[i].SortOrder < intervals[j].SortOrder })
	for i := 1; i < len(intervals); i++ {
		if intervals[i].StartAt.Before(intervals[i-1].EndAt) {
			return nil, errors.New("schedule lifecycle replacement intervals overlap")
		}
	}
	return map[string]any{"employment_id": employmentID.String(), "business_date": businessDate, "timezone": timezone, "status": replacement.Status, "source": replacement.Source, "shift_version_id": replacement.ShiftVersionID.String(), "intervals": intervals}, nil
}

func normalizeBreaks(breaks []api.ScheduleReplanBreak, startAt time.Time, loc *time.Location) ([]map[string]any, error) {
	result := make([]map[string]any, 0, len(breaks))
	for _, item := range breaks {
		start, err := parseLocalTime(item.Start)
		if err != nil {
			return nil, err
		}
		end, err := parseLocalTime(item.End)
		if err != nil {
			return nil, err
		}
		base := startAt
		breakStart := time.Date(base.Year(), base.Month(), base.Day(), start.Hour(), start.Minute(), start.Second(), 0, loc)
		if breakStart.Before(base) {
			breakStart = breakStart.AddDate(0, 0, 1)
		}
		breakEnd := time.Date(breakStart.Year(), breakStart.Month(), breakStart.Day(), end.Hour(), end.Minute(), end.Second(), 0, loc)
		if !breakEnd.After(breakStart) {
			breakEnd = breakEnd.AddDate(0, 0, 1)
		}
		result = append(result, map[string]any{"start": start.Format("15:04:05"), "end": end.Format("15:04:05"), "start_at": breakStart, "end_at": breakEnd, "break_punch_mode": valueOrDefault(&item.BreakPunchMode, "none")})
	}
	return result, nil
}

func loadRevisionDocument(ctx context.Context, tx DBTX, revision lifecycleRevision) (map[string]any, error) {
	rows, err := tx.Query(ctx, `SELECT id, sort_order, template_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot, COALESCE(start_day_offset, 0), COALESCE(end_day_offset, 0), shift_group_key, start_utc_offset_minutes, end_utc_offset_minutes FROM work_segments WHERE schedule_revision_id = $1 ORDER BY sort_order, id`, revision.ID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	intervals := make([]map[string]any, 0)
	for rows.Next() {
		var segment lifecycleSegment
		if err := rows.Scan(&segment.ID, &segment.SortOrder, &segment.TemplateID, &segment.ShiftVersionID, &segment.StartAt, &segment.EndAt, &segment.Breaks, &segment.StartDayOffset, &segment.EndDayOffset, &segment.ShiftGroupKey, &segment.StartUTCOffsetMinutes, &segment.EndUTCOffsetMinutes); err != nil {
			return nil, err
		}
		location, err := time.LoadLocation(revision.Timezone)
		if err != nil {
			return nil, err
		}
		intervals = append(intervals, map[string]any{
			"sort_order":               segment.SortOrder,
			"shift_version_id":         segment.ShiftVersionID.String(),
			"start_day_offset":         segment.StartDayOffset,
			"end_day_offset":           segment.EndDayOffset,
			"shift_group_key":          segment.ShiftGroupKey,
			"start_utc_offset_minutes": segment.StartUTCOffsetMinutes,
			"end_utc_offset_minutes":   segment.EndUTCOffsetMinutes,
			"start":                    segment.StartAt.In(location).Format("15:04:05"),
			"end":                      segment.EndAt.In(location).Format("15:04:05"),
			"start_at":                 segment.StartAt,
			"end_at":                   segment.EndAt,
			"breaks":                   json.RawMessage(segment.Breaks),
		})
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	status := "explicit_off"
	if revision.State == "published" {
		status = "work"
	}
	return map[string]any{"employment_id": revision.EmploymentID.String(), "business_date": revision.BusinessDate, "timezone": revision.Timezone, "status": status, "source": revision.Source, "revision_id": revision.ID.String(), "revision_number": revision.RevisionNumber, "intervals": intervals}, nil
}

func commitReplan(ctx context.Context, tx pgx.Tx, actorID uuid.UUID, idempotencyKey, requestHash string, request api.ScheduleReplanRequest, current lifecycleRevision, result replanResult, response *api.ScheduleReplanResponse) error {
	batchID := uuid.New()
	publishedAt := time.Now().UTC()
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, actor_type, reason, timezone, range_start, range_end, published_at, affected_days, approval_instance_id) SELECT $1, $2, $3, COALESCE(b.mode, 'rotating'), 'published', $4, 'replan', '{}'::jsonb, 'human', $5, r.timezone, r.business_date, r.business_date, $6, 1, b.approval_instance_id FROM schedule_day_revisions r LEFT JOIN schedule_publication_batches b ON b.id = r.publication_batch_id WHERE r.id = $7`, batchID, idempotencyKey, requestHash, actorID, strings.TrimSpace(request.Reason), publishedAt, current.ID); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'superseded' WHERE id = $1 AND state IN ('published', 'explicit_off')`, current.ID); err != nil {
		return err
	}
	replacement := result.Replacement
	if err := rejectConcreteScheduleOverlaps(ctx, tx, []map[string]any{replacement}); err != nil {
		return err
	}
	newRevisionID, err := insertPublishedDay(ctx, tx, batchID, publicationActor{ID: &actorID, Type: "human"}, request.Reason, current.Timezone, replacement, publishedAt, &current.ID)
	if err != nil {
		return err
	}
	*response = api.ScheduleReplanResponse{BatchID: batchID, OriginalRevisionID: current.ID, ReplacementRevisionID: newRevisionID, RevisionNumber: current.RevisionNumber + 1, Reason: strings.TrimSpace(request.Reason), ActualTime: publishedAt, Retroactive: false, Diff: result.Diff}
	responseJSON, err := json.Marshal(response)
	if err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_publication_batches SET response_snapshot = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $1`, batchID, responseJSON); err != nil {
		return err
	}
	_, err = tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, created_at, actor_type) VALUES ($1, $2, 'schedule.superseded', $3, CURRENT_TIMESTAMP, 'human')`, actorID, batchID, jsonArg(map[string]any{"original_revision_id": current.ID.String(), "replacement_revision_id": newRevisionID.String(), "diff_hash": result.DiffHash}))
	return err
}

type replanResult struct {
	Replacement     map[string]any
	Diff            map[string]any
	DiffHash        string
	ReplacementHash string
}

func decodeReplanResult(raw []byte) (replanResult, error) {
	var value struct {
		Replacement map[string]any `json:"replacement"`
		Diff        map[string]any `json:"diff"`
		DiffHash    string         `json:"diff_hash"`
	}
	if err := json.Unmarshal(raw, &value); err != nil || value.Replacement == nil || value.Diff == nil || value.DiffHash == "" {
		return replanResult{}, ErrSchedulePreviewStale
	}
	return replanResult{Replacement: value.Replacement, Diff: value.Diff, DiffHash: value.DiffHash, ReplacementHash: hashMap(value.Replacement)}, nil
}

func canonicalStoredReplacementHash(replacement map[string]any, timezone, businessDate string, employmentID uuid.UUID) string {
	payload, err := json.Marshal(replacement)
	if err != nil {
		return ""
	}
	var typed api.ScheduleReplanReplacement
	if err := json.Unmarshal(payload, &typed); err != nil {
		return ""
	}
	return replacementHash(typed, timezone, businessDate, employmentID)
}

func replacementHash(replacement api.ScheduleReplanReplacement, timezone, businessDate string, employmentID uuid.UUID) string {
	value, err := normalizeReplacement(replacement, timezone, businessDate, employmentID)
	if err != nil {
		return ""
	}
	return hashMap(value)
}

func parseBreakWorkedRequest(actorID, employmentID uuid.UUID, request api.MarkBreakWorkedRequest) (time.Time, time.Time, error) {
	if actorID == uuid.Nil || employmentID == uuid.Nil || request.SegmentID == uuid.Nil || strings.TrimSpace(request.Reason) == "" {
		return time.Time{}, time.Time{}, api.ErrScheduleBreakOverrideInvalid
	}
	start, err := time.Parse(time.RFC3339, request.Start)
	if err != nil {
		return time.Time{}, time.Time{}, api.ErrScheduleBreakOverrideInvalid
	}
	end, err := time.Parse(time.RFC3339, request.End)
	if err != nil || !end.After(start) {
		return time.Time{}, time.Time{}, api.ErrScheduleBreakOverrideInvalid
	}
	return start, end, nil
}

type scheduledBreakInterval struct {
	StartAt time.Time `json:"start_at"`
	EndAt   time.Time `json:"end_at"`
}

func matchesScheduledBreakWindow(breakItems []scheduledBreakInterval, start, end time.Time) bool {
	for _, item := range breakItems {
		if item.EndAt.After(item.StartAt) && start.Before(item.EndAt) && end.After(item.StartAt) {
			return !start.Before(item.StartAt) && !end.After(item.EndAt)
		}
	}
	return false
}

func validateBreakWorkedWindow(breaks []byte, scheduledStart, scheduledEnd, start, end time.Time) error {
	if scheduledStart.IsZero() || scheduledEnd.IsZero() || !scheduledEnd.After(scheduledStart) {
		return api.ErrScheduleBreakOverrideInvalid
	}
	if start.Before(scheduledStart) || end.After(scheduledEnd) {
		return api.ErrScheduleBreakOverrideInvalid
	}
	if err := json.Unmarshal(breaks, &[]map[string]any{}); err != nil {
		return api.ErrScheduleBreakOverrideInvalid
	}
	var breakItems []scheduledBreakInterval
	if err := json.Unmarshal(breaks, &breakItems); err != nil {
		return api.ErrScheduleBreakOverrideInvalid
	}
	if !matchesScheduledBreakWindow(breakItems, start, end) {
		return api.ErrScheduleBreakOverrideInvalid
	}
	return nil
}

func markBreakWorkedTx(ctx context.Context, tx pgx.Tx, actorID, employmentID uuid.UUID, businessDate string, request api.MarkBreakWorkedRequest, start, end time.Time) (api.MarkBreakWorkedResponse, error) {
	var employerID uuid.UUID
	var scheduledStart, scheduledEnd time.Time
	var breaks []byte
	if err := tx.QueryRow(ctx, `SELECT e.employer_id, ws.scheduled_start_at, ws.scheduled_end_at, ws.breaks_snapshot FROM work_segments ws JOIN employments e ON e.id = ws.employment_id WHERE ws.id = $1::uuid AND ws.employment_id = $2::uuid AND ws.date = $3::date FOR UPDATE`, request.SegmentID, employmentID, businessDate).Scan(&employerID, &scheduledStart, &scheduledEnd, &breaks); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.MarkBreakWorkedResponse{}, api.ErrScheduleBreakOverrideInvalid
		}
		return api.MarkBreakWorkedResponse{}, err
	}
	if err := ensureMutationPeriodOpen(ctx, tx, employerID, businessDate); err != nil {
		return api.MarkBreakWorkedResponse{}, err
	}
	if err := validateBreakWorkedWindow(breaks, scheduledStart, scheduledEnd, start, end); err != nil {
		return api.MarkBreakWorkedResponse{}, err
	}
	var overlap bool
	if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM schedule_day_overrides WHERE employment_id = $1::uuid AND business_date = $2::date AND status IN ('approved', 'pending') AND start_at < $4::timestamptz AND end_at > $3::timestamptz)`, employmentID, businessDate, start, end).Scan(&overlap); err != nil {
		return api.MarkBreakWorkedResponse{}, err
	}
	if overlap {
		return api.MarkBreakWorkedResponse{}, api.ErrScheduleBreakOverrideInvalid
	}
	minutes := int(end.Sub(start).Minutes())
	id := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_overrides (id, employment_id, business_date, work_segment_id, override_type, start_at, end_at, minutes, reason, status, created_by, created_at) VALUES ($1::uuid, $2::uuid, $3::date, $4::uuid, 'break_worked', $5::timestamptz, $6::timestamptz, $7::integer, $8::text, 'approved', $9::uuid, CURRENT_TIMESTAMP)`, id, employmentID, businessDate, request.SegmentID, start, end, minutes, strings.TrimSpace(request.Reason), actorID); err != nil {
		return api.MarkBreakWorkedResponse{}, err
	}
	if _, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, actor_type, created_at) VALUES ($1, $2, 'schedule.break_worked.marked', jsonb_build_object('employment_id', $3::uuid, 'business_date', $4::text, 'segment_id', $5::uuid, 'start', $6::timestamptz, 'end', $7::timestamptz, 'minutes', $8::integer, 'reason', $9::text), 'human', CURRENT_TIMESTAMP)`, actorID, id, employmentID, businessDate, request.SegmentID, start, end, minutes, strings.TrimSpace(request.Reason)); err != nil {
		return api.MarkBreakWorkedResponse{}, err
	}
	return api.MarkBreakWorkedResponse{ID: id, EmploymentID: employmentID, BusinessDate: businessDate, SegmentID: request.SegmentID, Start: start, End: end, Minutes: minutes, Status: "approved"}, nil
}

func (r ScheduleRepository) MarkBreakWorked(ctx context.Context, actorID, employmentID uuid.UUID, businessDate string, request api.MarkBreakWorkedRequest) (api.MarkBreakWorkedResponse, error) {
	start, end, err := parseBreakWorkedRequest(actorID, employmentID, request)
	if err != nil {
		return api.MarkBreakWorkedResponse{}, err
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return api.MarkBreakWorkedResponse{}, err
	}
	var response api.MarkBreakWorkedResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		resp, err := markBreakWorkedTx(ctx, tx, actorID, employmentID, businessDate, request, start, end)
		if err != nil {
			return err
		}
		response = resp
		return nil
	})
	if err != nil {
		return api.MarkBreakWorkedResponse{}, err
	}
	return response, nil
}
