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
		if _, err := tx.Exec(ctx, `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ`); err != nil {
			return err
		}
		resolution, _, err := resolveScheduleData(ctx, tx, startDate, endDate, request.EmploymentIDs, 1, len(request.EmploymentIDs))
		if err != nil {
			return err
		}
		if err := validatePreviewScope(resolution, request.EmploymentIDs, startDate, endDate); err != nil {
			return err
		}
		if request.Mode != "rotating" {
			if err := validateHumanOfficeApproval(ctx, tx, api.SchedulePublishRequest{
				EmploymentIDs:      request.EmploymentIDs,
				StartDate:          request.StartDate,
				EndDate:            request.EndDate,
				ApprovalInstanceID: request.ApprovalInstanceID,
			}); err != nil {
				return err
			}
		}
		currentRevisions, err := loadCurrentRevisionNumbers(ctx, tx, request.EmploymentIDs, request.StartDate, request.EndDate)
		if err != nil {
			return err
		}
		if err := validateExpectedRevisions(request.ExpectedRevisions, currentRevisions, request.EmploymentIDs, startDate, endDate); err != nil {
			return err
		}
		data := make([]map[string]any, 0, len(request.EmploymentIDs)*daysInRange(startDate, endDate))
		for _, employee := range resolution.employees {
			for _, day := range employee.days {
				if day.timezone != request.Timezone {
					return ErrScheduleTimezoneMismatch
				}
				item, err := sanitizedPreviewDay(day, resolution.shifts)
				if err != nil {
					return err
				}
				item["employment_id"] = employee.employmentID.String()
				data = append(data, item)
			}
		}
		result := map[string]any{"operation": "publish", "data": data, "resolver_version": lifecycleResolverVersion, "expected_revisions": expectedRevisionList(request.ExpectedRevisions)}
		requestHash, err := hashJSON(previewHashInput(actorID, request))
		if err != nil {
			return err
		}
		resultHash, err := hashJSON(result)
		if err != nil {
			return err
		}
		previewID := uuid.New()
		createdAt := time.Now().UTC()
		expiresAt := createdAt.Add(previewTTL)
		scope := map[string]any{"employment_ids": uuidStrings(request.EmploymentIDs), "start_date": request.StartDate, "end_date": request.EndDate, "timezone": request.Timezone}
		if _, err := tx.Exec(ctx, `INSERT INTO schedule_previews (id, requested_by, actor_type, operation, request_hash, result_hash, scope_snapshot, result_snapshot, resolver_version, expires_at, created_at) VALUES ($1, $2, 'human', 'publish', $3, $4, $5, $6, $7, $8, $9)`, previewID, actorID, requestHash, resultHash, jsonArg(scope), jsonArg(result), lifecycleResolverVersion, expiresAt, createdAt); err != nil {
			return err
		}
		rawData := make([]json.RawMessage, 0, len(data))
		for _, item := range data {
			encoded, err := json.Marshal(item)
			if err != nil {
				return err
			}
			rawData = append(rawData, encoded)
		}
		response = api.SchedulePreviewResponse{Provisional: true, PreviewID: previewID, PreviewHash: resultHash, ExpiresAt: expiresAt, ResolverVersion: lifecycleResolverVersion, Data: rawData}
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

func validatePreviewInput(actorID uuid.UUID, request api.SchedulePreviewRequest) (time.Time, time.Time, error) {
	if actorID == uuid.Nil || (request.Mode != "office" && request.Mode != "rotating") || len(request.EmploymentIDs) == 0 || len(request.EmploymentIDs) > 500 || strings.TrimSpace(request.Reason) == "" || (request.Mode == "office" && request.ApprovalInstanceID == nil) || (request.Mode == "rotating" && request.ApprovalInstanceID != nil) {
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

func validateExpectedRevisions(expected, current map[string]int64, employmentIDs []uuid.UUID, startDate, endDate time.Time) error {
	validKeys := make(map[string]struct{}, len(employmentIDs)*daysInRange(startDate, endDate))
	for _, employmentID := range employmentIDs {
		for date := startDate; !date.After(endDate); date = date.AddDate(0, 0, 1) {
			validKeys[employmentID.String()+":"+date.Format("2006-01-02")] = struct{}{}
		}
	}
	if len(expected) != len(validKeys) {
		return ErrSchedulePreviewScope
	}
	for key := range expected {
		if _, ok := validKeys[key]; !ok {
			return ErrSchedulePreviewScope
		}
	}
	for _, employmentID := range employmentIDs {
		for date := startDate; !date.After(endDate); date = date.AddDate(0, 0, 1) {
			key := employmentID.String() + ":" + date.Format("2006-01-02")
			value, ok := expected[key]
			if !ok || value < 0 {
				return ErrSchedulePreviewScope
			}
			actual := current[key]
			if value != actual {
				return ErrScheduleRevisionStale
			}
		}
	}
	return nil
}

func previewHashInput(actorID uuid.UUID, request api.SchedulePreviewRequest) lifecycleRequestHashInput {
	approval := ""
	if request.ApprovalInstanceID != nil {
		approval = request.ApprovalInstanceID.String()
	}
	return lifecycleRequestHashInput{Operation: "publish", ActorID: actorID.String(), EmploymentIDs: uuidStrings(request.EmploymentIDs), StartDate: request.StartDate, EndDate: request.EndDate, Timezone: request.Timezone, Mode: request.Mode, ApprovalInstanceID: stringPtr(approval), ExpectedRevisions: expectedRevisionList(request.ExpectedRevisions), Reason: strings.TrimSpace(request.Reason), Retroactive: request.Retroactive}
}

func sanitizedPreviewDay(day resolvedScheduleDay, shifts map[uuid.UUID]shiftTemplateInfo) (map[string]any, error) {
	result := map[string]any{"business_date": day.date, "status": "missing", "source": day.source, "timezone": day.timezone, "shift_version_id": nil, "intervals": []any{}, "trace": sanitizedTrace(day.traces), "warnings": []string{}}
	if day.source == "conflict" {
		result["status"] = "conflict"
		result["warnings"] = []string{"resolver_conflict"}
		return result, nil
	}
	if len(day.shiftIDs) == 0 || (len(day.shiftIDs) == 1 && day.shiftIDs[0] == nil) {
		if day.source == "roster" || day.source == "pattern" {
			result["status"] = "explicit_off"
			return result, nil
		}
		return result, nil
	}
	if len(day.shiftIDs) != 1 || day.shiftIDs[0] == nil {
		result["status"] = "conflict"
		result["warnings"] = []string{"multiple_selected_shifts"}
		return result, nil
	}
	version := shiftVersionInfo{}
	selected, conflict := false, false
	if len(day.pinnedShiftVersionIDs) == 1 && day.pinnedShiftVersionIDs[0] != nil {
		for _, candidate := range shifts[*day.shiftIDs[0]].versions {
			if candidate.id == *day.pinnedShiftVersionIDs[0] {
				version, selected = candidate, true
				break
			}
		}
	} else {
		version, selected, conflict = selectEffectiveShiftVersion(shifts, *day.shiftIDs[0], parseScheduleDate(day.date))
	}
	if conflict {
		result["status"] = "conflict"
		result["warnings"] = []string{"overlapping_shift_versions"}
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

func previewIntervals(version shiftVersionInfo, businessDate, timezone string) ([]map[string]any, error) {
	loc, err := time.LoadLocation(timezone)
	if err != nil {
		return nil, err
	}
	year, month, day := parseDateParts(businessDate)
	result := make([]map[string]any, 0, len(version.intervals))
	for _, interval := range version.intervals {
		if interval.startTime == nil || interval.endTime == nil {
			return nil, ErrScheduleResolverUnavailable
		}
		start, err := parseLocalTime(*interval.startTime)
		if err != nil {
			return nil, err
		}
		end, err := parseLocalTime(*interval.endTime)
		if err != nil {
			return nil, err
		}
		startOffset, endOffset := interval.startDayOffset, interval.endDayOffset
		if startOffset == 0 && endOffset == 0 && !end.After(start) {
			endOffset = 1
		}
		if startOffset < 0 || endOffset < startOffset {
			return nil, errors.New("schedule interval day offsets are invalid")
		}
		startDate := time.Date(year, month, day+startOffset, 0, 0, 0, 0, loc)
		endDate := time.Date(year, month, day+endOffset, 0, 0, 0, 0, loc)
		startAt, err := localDateTime(startDate, start, interval.startUTCOffsetMinutes)
		if err != nil {
			return nil, err
		}
		endAt, err := localDateTime(endDate, end, interval.endUTCOffsetMinutes)
		if err != nil {
			return nil, err
		}
		if !endAt.After(startAt) {
			return nil, errors.New("schedule interval ends before it starts")
		}
		breaks := make([]map[string]any, 0)
		if interval.breakStart != nil || interval.breakEnd != nil {
			if interval.breakStart == nil || interval.breakEnd == nil {
				return nil, ErrScheduleResolverUnavailable
			}
			breaks, err = normalizeBreaks([]api.ScheduleReplanBreak{{Start: *interval.breakStart, End: *interval.breakEnd, BreakPunchMode: valueOrDefault(interval.breakPunchMode, "none")}}, startAt, loc)
			if err != nil {
				return nil, err
			}
		}
		result = append(result, map[string]any{"sort_order": interval.sortOrder, "start": start.Format("15:04:05"), "end": end.Format("15:04:05"), "start_day_offset": interval.startDayOffset, "end_day_offset": interval.endDayOffset, "start_utc_offset_minutes": interval.startUTCOffsetMinutes, "end_utc_offset_minutes": interval.endUTCOffsetMinutes, "start_at": startAt, "end_at": endAt, "shift_version_id": version.id.String(), "breaks": breaks, "break_punch_mode": valueOrDefault(interval.breakPunchMode, "none")})
	}
	return result, nil
}
