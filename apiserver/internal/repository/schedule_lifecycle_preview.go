package repository

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"strings"
	"time"
)

func buildPreviewDays(resolution scheduleResolution, requestTimezone string, capacity int) ([]map[string]any, error) {
	data := make([]map[string]any, 0, capacity)
	for _, employee := range resolution.employees {
		for _, day := range employee.days {
			if day.timezone != requestTimezone {
				return nil, ErrScheduleTimezoneMismatch
			}
			item, err := sanitizedPreviewDay(day, resolution.shifts)
			if err != nil {
				return nil, err
			}
			item["employment_id"] = employee.employmentID.String()
			data = append(data, item)
		}
	}
	return data, nil
}

func marshalPreviewRows(data []map[string]any) ([]json.RawMessage, error) {
	rawData := make([]json.RawMessage, 0, len(data))
	for _, item := range data {
		encoded, err := json.Marshal(item)
		if err != nil {
			return nil, err
		}
		rawData = append(rawData, encoded)
	}
	return rawData, nil
}

func persistPreviewSnapshot(ctx context.Context, tx pgx.Tx, actorID uuid.UUID, request api.SchedulePreviewRequest, data []map[string]any) (api.SchedulePreviewResponse, error) {
	result := map[string]any{"operation": "publish", "data": data, "resolver_version": lifecycleResolverVersion, "expected_revisions": expectedRevisionList(request.ExpectedRevisions)}
	requestHash, err := hashJSON(previewHashInput(actorID, request))
	if err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	resultHash, err := hashJSON(result)
	if err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	previewID := uuid.New()
	createdAt := time.Now().UTC()
	expiresAt := createdAt.Add(previewTTL)
	scope := map[string]any{"employment_ids": uuidStrings(request.EmploymentIDs), "start_date": request.StartDate, "end_date": request.EndDate, "timezone": request.Timezone}
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_previews (id, requested_by, actor_type, operation, request_hash, result_hash, scope_snapshot, result_snapshot, resolver_version, expires_at, created_at) VALUES ($1, $2, 'human', 'publish', $3, $4, $5, $6, $7, $8, $9)`, previewID, actorID, requestHash, resultHash, jsonArg(scope), jsonArg(result), lifecycleResolverVersion, expiresAt, createdAt); err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	rawData, err := marshalPreviewRows(data)
	if err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	return api.SchedulePreviewResponse{Provisional: true, PreviewID: previewID, PreviewHash: resultHash, ExpiresAt: expiresAt, ResolverVersion: lifecycleResolverVersion, Data: rawData}, nil
}

func (r ScheduleRepository) createPreviewTx(ctx context.Context, tx pgx.Tx, actorID uuid.UUID, request api.SchedulePreviewRequest, startDate, endDate time.Time) (api.SchedulePreviewResponse, error) {
	if _, err := tx.Exec(ctx, `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ`); err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	resolution, _, err := resolveScheduleData(ctx, tx, startDate, endDate, request.EmploymentIDs, 1, len(request.EmploymentIDs))
	if err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	if err := validatePreviewScope(resolution, request.EmploymentIDs, startDate, endDate); err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	if request.Mode != "rotating" {
		if err := validateHumanOfficeApproval(ctx, tx, api.SchedulePublishRequest{
			EmploymentIDs:      request.EmploymentIDs,
			StartDate:          request.StartDate,
			EndDate:            request.EndDate,
			ApprovalInstanceID: request.ApprovalInstanceID,
		}); err != nil {
			return api.SchedulePreviewResponse{}, err
		}
	}
	currentRevisions, err := loadCurrentRevisionNumbers(ctx, tx, request.EmploymentIDs, request.StartDate, request.EndDate)
	if err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	if err := validateExpectedRevisions(request.ExpectedRevisions, currentRevisions, request.EmploymentIDs, startDate, endDate); err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	data, err := buildPreviewDays(resolution, request.Timezone, len(request.EmploymentIDs)*daysInRange(startDate, endDate))
	if err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	return persistPreviewSnapshot(ctx, tx, actorID, request, data)
}

func (r ScheduleRepository) CreatePreview(ctx context.Context, actorID uuid.UUID, request api.SchedulePreviewRequest) (api.SchedulePreviewResponse, error) {
	startDate, endDate, err := validatePreviewInput(actorID, request)
	if err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	var response api.SchedulePreviewResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		resp, err := r.createPreviewTx(ctx, tx, actorID, request, startDate, endDate)
		if err != nil {
			return err
		}
		response = resp
		return nil
	})
	if err != nil {
		return api.SchedulePreviewResponse{}, err
	}
	return response, nil
}

func validateHumanOfficeApproval(ctx context.Context, db DBTX, request api.SchedulePublishRequest) error {
	if request.ApprovalInstanceID == nil || *request.ApprovalInstanceID == uuid.Nil || len(request.EmploymentIDs) == 0 {
		return api.ErrScheduleApprovalInvalid
	}
	var valid bool
	err := db.QueryRow(ctx, `
		WITH target_employments AS (
			SELECT e.id, e.employer_id, e.start_date, e.end_date
			FROM employments e
			WHERE e.id = ANY($1::uuid[]) AND e.deleted_at IS NULL
		), uncovered_days AS (
			SELECT e.id
			FROM target_employments e
			CROSS JOIN LATERAL generate_series(
				GREATEST(e.start_date, $2::date),
				LEAST(COALESCE(e.end_date, $3::date), $3::date),
				interval '1 day'
			) days(day)
			WHERE NOT EXISTS (
				SELECT 1
				FROM schedule_office_policies p
				JOIN approval_workflow_instances i ON i.id = p.approval_instance_id
				WHERE p.approval_instance_id = $4
				  AND p.employer_id = e.employer_id
				  AND p.status = 'active'
				  AND p.effective_from <= days.day::date
				  AND (p.effective_to IS NULL OR p.effective_to >= days.day::date)
				  AND i.entity_type = 'schedule_policy'
				  AND i.status = 'approved'
			)
		)
		SELECT (SELECT count(*) FROM target_employments) = cardinality($1::uuid[])
		   AND NOT EXISTS (SELECT 1 FROM uncovered_days)`, request.EmploymentIDs, request.StartDate, request.EndDate, *request.ApprovalInstanceID).Scan(&valid)
	if err != nil {
		return err
	}
	if !valid {
		return api.ErrScheduleApprovalInvalid
	}
	return nil
}

func validPreviewModeAndApproval(request api.SchedulePreviewRequest) bool {
	switch request.Mode {
	case "office":
		return request.ApprovalInstanceID != nil
	case "rotating":
		return request.ApprovalInstanceID == nil
	default:
		return false
	}
}

func validPreviewRequestScope(actorID uuid.UUID, request api.SchedulePreviewRequest) bool {
	return actorID != uuid.Nil && validPreviewModeAndApproval(request) && len(request.EmploymentIDs) > 0 && len(request.EmploymentIDs) <= 500 && strings.TrimSpace(request.Reason) != ""
}

func validatePreviewInput(actorID uuid.UUID, request api.SchedulePreviewRequest) (time.Time, time.Time, error) {
	if !validPreviewRequestScope(actorID, request) {
		return time.Time{}, time.Time{}, ErrSchedulePreviewScope
	}
	startDate, err := time.Parse("2006-01-02", request.StartDate)
	if err != nil {
		return time.Time{}, time.Time{}, err
	}
	endDate, err := time.Parse("2006-01-02", request.EndDate)
	if err != nil || endDate.Before(startDate) || daysInRange(startDate, endDate) > 62 {
		return time.Time{}, time.Time{}, ErrSchedulePreviewScope
	}
	if _, err := time.LoadLocation(request.Timezone); err != nil {
		return time.Time{}, time.Time{}, err
	}
	return startDate, endDate, nil
}

func daysInRange(startDate, endDate time.Time) int { return int(endDate.Sub(startDate).Hours()/24) + 1 }

func validatePreviewScope(resolution scheduleResolution, employmentIDs []uuid.UUID, startDate, endDate time.Time) error {
	seen := make(map[uuid.UUID]int, len(resolution.employees))
	for _, employee := range resolution.employees {
		seen[employee.employmentID] = len(employee.days)
	}
	for _, employmentID := range employmentIDs {
		if seen[employmentID] != daysInRange(startDate, endDate) {
			return ErrSchedulePreviewScope
		}
	}
	return nil
}

func loadCurrentRevisionNumbers(ctx context.Context, tx DBTX, employmentIDs []uuid.UUID, startDate, endDate string) (map[string]int64, error) {
	rows, err := tx.Query(ctx, `SELECT employment_id, business_date::text, revision_number FROM schedule_day_revisions WHERE employment_id = ANY($1) AND business_date BETWEEN $2::date AND $3::date AND state IN ('published', 'explicit_off')`, employmentIDs, startDate, endDate)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make(map[string]int64)
	for rows.Next() {
		var employmentID uuid.UUID
		var businessDate string
		var revision int64
		if err := rows.Scan(&employmentID, &businessDate, &revision); err != nil {
			return nil, err
		}
		result[employmentID.String()+":"+businessDate] = revision
	}
	return result, rows.Err()
}

func buildExpectedRevisionKeys(employmentIDs []uuid.UUID, startDate, endDate time.Time) map[string]struct{} {
	validKeys := make(map[string]struct{}, len(employmentIDs)*daysInRange(startDate, endDate))
	for _, employmentID := range employmentIDs {
		for date := startDate; !date.After(endDate); date = date.AddDate(0, 0, 1) {
			validKeys[employmentID.String()+":"+date.Format("2006-01-02")] = struct{}{}
		}
	}
	return validKeys
}

func verifyExpectedRevisionValues(expected, current map[string]int64, employmentIDs []uuid.UUID, startDate, endDate time.Time) error {
	for _, employmentID := range employmentIDs {
		for date := startDate; !date.After(endDate); date = date.AddDate(0, 0, 1) {
			key := employmentID.String() + ":" + date.Format("2006-01-02")
			value, ok := expected[key]
			if !ok || value < 0 {
				return ErrSchedulePreviewScope
			}
			if value != current[key] {
				return ErrScheduleRevisionStale
			}
		}
	}
	return nil
}

func validateExpectedRevisions(expected, current map[string]int64, employmentIDs []uuid.UUID, startDate, endDate time.Time) error {
	validKeys := buildExpectedRevisionKeys(employmentIDs, startDate, endDate)
	if len(expected) != len(validKeys) {
		return ErrSchedulePreviewScope
	}
	for key := range expected {
		if _, ok := validKeys[key]; !ok {
			return ErrSchedulePreviewScope
		}
	}
	return verifyExpectedRevisionValues(expected, current, employmentIDs, startDate, endDate)
}

func previewHashInput(actorID uuid.UUID, request api.SchedulePreviewRequest) lifecycleRequestHashInput {
	approval := ""
	if request.ApprovalInstanceID != nil {
		approval = request.ApprovalInstanceID.String()
	}
	return lifecycleRequestHashInput{Operation: "publish", ActorID: actorID.String(), EmploymentIDs: uuidStrings(request.EmploymentIDs), StartDate: request.StartDate, EndDate: request.EndDate, Timezone: request.Timezone, Mode: request.Mode, ApprovalInstanceID: stringPtr(approval), ExpectedRevisions: expectedRevisionList(request.ExpectedRevisions), Reason: strings.TrimSpace(request.Reason), Retroactive: request.Retroactive}
}

func resolvePreviewShiftVersion(day resolvedScheduleDay, shifts map[uuid.UUID]shiftTemplateInfo) (shiftVersionInfo, bool, bool) {
	if len(day.pinnedShiftVersionIDs) == 1 && day.pinnedShiftVersionIDs[0] != nil {
		for _, candidate := range shifts[*day.shiftIDs[0]].versions {
			if candidate.id == *day.pinnedShiftVersionIDs[0] {
				return candidate, true, false
			}
		}
		return shiftVersionInfo{}, false, false
	}
	return selectEffectiveShiftVersion(shifts, *day.shiftIDs[0], parseScheduleDate(day.date))
}

func isEmptyShiftSelection(shiftIDs []*uuid.UUID) bool {
	return len(shiftIDs) == 0 || (len(shiftIDs) == 1 && shiftIDs[0] == nil)
}

func isExplicitOffSource(source string) bool {
	return source == "roster" || source == "pattern"
}

func sanitizedPreviewDay(day resolvedScheduleDay, shifts map[uuid.UUID]shiftTemplateInfo) (map[string]any, error) {
	result := map[string]any{"business_date": day.date, "status": "missing", "source": day.source, "timezone": day.timezone, "shift_version_id": nil, "intervals": []any{}, "trace": sanitizedTrace(day.traces), "warnings": []string{}}
	if day.source == "conflict" {
		result["status"], result["warnings"] = "conflict", []string{"resolver_conflict"}
		return result, nil
	}
	if isEmptyShiftSelection(day.shiftIDs) {
		if isExplicitOffSource(day.source) {
			result["status"] = "explicit_off"
		}
		return result, nil
	}
	if len(day.shiftIDs) != 1 || day.shiftIDs[0] == nil {
		result["status"], result["warnings"] = "conflict", []string{"multiple_selected_shifts"}
		return result, nil
	}
	version, selected, conflict := resolvePreviewShiftVersion(day, shifts)
	if conflict {
		result["status"], result["warnings"] = "conflict", []string{"overlapping_shift_versions"}
		return result, nil
	}
	if !selected || len(version.intervals) == 0 {
		return nil, ErrScheduleResolverUnavailable
	}
	intervals, err := previewIntervals(version, day.date, day.timezone)
	if err != nil {
		return nil, err
	}
	result["status"], result["shift_version_id"], result["intervals"] = "work", version.id.String(), intervals
	return result, nil
}

func sanitizedTrace(traces []api.ScheduleTrace) []map[string]any {
	result := make([]map[string]any, 0, len(traces))
	for _, trace := range traces {
		item := map[string]any{"priority": trace.Priority, "status": trace.Status}
		if trace.RuleID != nil {
			item["rule_id"] = trace.RuleID.String()
		}
		if trace.Status == "applied" {
			item["reason"] = "selected by resolver precedence"
		} else if trace.Status == "overridden" {
			item["reason"] = "overridden by resolver precedence"
		} else if trace.Status == "conflict" {
			item["reason"] = "same-precedence resolver conflict"
		}
		result = append(result, item)
	}
	return result
}

func valueOrDefault(value *string, fallback string) string {
	if value == nil || *value == "" {
		return fallback
	}
	return *value
}

func resolveIntervalDayOffsets(startOffset, endOffset int, start, end time.Time) (int, int, error) {
	if startOffset == 0 && endOffset == 0 && !end.After(start) {
		endOffset = 1
	}
	if startOffset < 0 || endOffset < startOffset {
		return 0, 0, errors.New("schedule interval day offsets are invalid")
	}
	return startOffset, endOffset, nil
}

func parseIntervalClockBounds(interval shiftIntervalInfo, year int, month time.Month, day int, loc *time.Location) (time.Time, time.Time, time.Time, time.Time, error) {
	if interval.startTime == nil || interval.endTime == nil {
		return time.Time{}, time.Time{}, time.Time{}, time.Time{}, ErrScheduleResolverUnavailable
	}
	start, err := parseLocalTime(*interval.startTime)
	if err != nil {
		return time.Time{}, time.Time{}, time.Time{}, time.Time{}, err
	}
	end, err := parseLocalTime(*interval.endTime)
	if err != nil {
		return time.Time{}, time.Time{}, time.Time{}, time.Time{}, err
	}
	startOffset, endOffset, err := resolveIntervalDayOffsets(interval.startDayOffset, interval.endDayOffset, start, end)
	if err != nil {
		return time.Time{}, time.Time{}, time.Time{}, time.Time{}, err
	}
	startAt, err := localDateTime(time.Date(year, month, day+startOffset, 0, 0, 0, 0, loc), start, interval.startUTCOffsetMinutes)
	if err != nil {
		return time.Time{}, time.Time{}, time.Time{}, time.Time{}, err
	}
	endAt, err := localDateTime(time.Date(year, month, day+endOffset, 0, 0, 0, 0, loc), end, interval.endUTCOffsetMinutes)
	if err != nil {
		return time.Time{}, time.Time{}, time.Time{}, time.Time{}, err
	}
	return start, end, startAt, endAt, nil
}

func buildPreviewInterval(interval shiftIntervalInfo, versionID uuid.UUID, year int, month time.Month, day int, loc *time.Location) (map[string]any, error) {
	start, end, startAt, endAt, err := parseIntervalClockBounds(interval, year, month, day, loc)
	if err != nil {
		return nil, err
	}
	if !endAt.After(startAt) {
		return nil, errors.New("schedule interval ends before it starts")
	}
	breaks, err := buildPreviewIntervalBreaks(interval, startAt, loc)
	if err != nil {
		return nil, err
	}
	return map[string]any{
		"sort_order": interval.sortOrder, "start": start.Format("15:04:05"), "end": end.Format("15:04:05"),
		"start_day_offset": interval.startDayOffset, "end_day_offset": interval.endDayOffset,
		"start_utc_offset_minutes": interval.startUTCOffsetMinutes, "end_utc_offset_minutes": interval.endUTCOffsetMinutes,
		"start_at": startAt, "end_at": endAt, "shift_version_id": versionID.String(),
		"breaks": breaks, "break_punch_mode": valueOrDefault(interval.breakPunchMode, "none"),
	}, nil
}

func buildPreviewIntervalBreaks(interval shiftIntervalInfo, startAt time.Time, loc *time.Location) ([]map[string]any, error) {
	if interval.breakStart == nil && interval.breakEnd == nil {
		return make([]map[string]any, 0), nil
	}
	if interval.breakStart == nil || interval.breakEnd == nil {
		return nil, ErrScheduleResolverUnavailable
	}
	return normalizeBreaks([]api.ScheduleReplanBreak{{Start: *interval.breakStart, End: *interval.breakEnd, BreakPunchMode: valueOrDefault(interval.breakPunchMode, "none")}}, startAt, loc)
}

func previewIntervals(version shiftVersionInfo, businessDate, timezone string) ([]map[string]any, error) {
	loc, err := time.LoadLocation(timezone)
	if err != nil {
		return nil, err
	}
	year, month, day := parseDateParts(businessDate)
	result := make([]map[string]any, 0, len(version.intervals))
	for _, interval := range version.intervals {
		item, err := buildPreviewInterval(interval, version.id, year, month, day, loc)
		if err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, nil
}
