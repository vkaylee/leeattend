package repository

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

var (
	ErrScheduleResolverUnavailable = errors.New("schedule lifecycle resolver cannot produce a complete sanitized result")
	ErrScheduleIdempotencyConflict = api.ErrScheduleIdempotencyConflict
	ErrSchedulePreviewStale        = api.ErrSchedulePreviewStale
	ErrScheduleAlreadyPublished    = api.ErrScheduleAlreadyPublished
	ErrScheduleRevisionStale       = api.ErrScheduleRevisionStale
	ErrScheduleNoCurrentRevision   = api.ErrScheduleNoCurrentRevision
	ErrScheduleLockedPeriod        = api.ErrScheduleLockedPeriod
	ErrSchedulePreviewScope        = api.ErrSchedulePreviewScope
	ErrScheduleTimezoneMismatch    = api.ErrScheduleTimezoneMismatch
)

const (
	lifecycleResolverVersion = "schedule-resolver-v1"
	previewTTL               = 15 * time.Minute
)

type lifecycleExpectedRevision struct {
	Key   string `json:"key"`
	Value int64  `json:"value"`
}

type lifecycleRequestHashInput struct {
	Operation          string                         `json:"operation"`
	ActorID            string                         `json:"actor_id"`
	EmploymentID       string                         `json:"employment_id,omitempty"`
	BusinessDate       string                         `json:"business_date,omitempty"`
	EmploymentIDs      []string                       `json:"employment_ids,omitempty"`
	StartDate          string                         `json:"start_date,omitempty"`
	EndDate            string                         `json:"end_date,omitempty"`
	Timezone           string                         `json:"timezone,omitempty"`
	Mode               string                         `json:"mode,omitempty"`
	ApprovalInstanceID *string                        `json:"approval_instance_id"`
	ExpectedRevisions  []lifecycleExpectedRevision    `json:"expected_revisions,omitempty"`
	ExpectedRevision   int64                          `json:"expected_revision,omitempty"`
	Reason             string                         `json:"reason"`
	Retroactive        bool                           `json:"retroactive,omitempty"`
	Replacement        *api.ScheduleReplanReplacement `json:"replacement,omitempty"`
	PreviewID          string                         `json:"preview_id,omitempty"`
	PreviewHash        string                         `json:"preview_hash,omitempty"`
	ConfirmDiffHash    string                         `json:"confirm_diff_hash,omitempty"`
}

type lifecycleRevision struct {
	ID                 uuid.UUID
	EmploymentID       uuid.UUID
	EmployerID         uuid.UUID
	BusinessDate       string
	Timezone           string
	RevisionNumber     int64
	State              string
	BatchID            uuid.UUID
	Mode               string
	ApprovalInstanceID *uuid.UUID
	Source             string
	SourceSnapshot     []byte
	PublishedAt        *time.Time
}

type lifecycleSegment struct {
	ID                    uuid.UUID
	SortOrder             int
	TemplateID            uuid.UUID
	ShiftVersionID        uuid.UUID
	StartAt               time.Time
	EndAt                 time.Time
	Breaks                json.RawMessage
	StartDayOffset        *int
	EndDayOffset          *int
	ShiftGroupKey         *string
	StartUTCOffsetMinutes *int
	EndUTCOffsetMinutes   *int
}

func (r ScheduleRepository) beginLifecycle(ctx context.Context) (TxBeginner, error) {
	if r.DB == nil {
		return nil, errors.New("schedule lifecycle repository: database is not configured")
	}
	beginner, ok := r.DB.(TxBeginner)
	if !ok {
		return nil, errors.New("schedule lifecycle repository: database does not support transactions")
	}
	return beginner, nil
}

func (r ScheduleRepository) ActivateScheduleEmployer(ctx context.Context, employerID uuid.UUID, startDate, endDate, reason string, actorID uuid.UUID) error {
	if employerID == uuid.Nil || actorID == uuid.Nil || strings.TrimSpace(reason) == "" || len(strings.TrimSpace(reason)) > 2000 || startDate == "" || endDate == "" || startDate > endDate {
		return api.ErrScheduleActivationNotReady
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var ready bool
		if err := tx.QueryRow(ctx, `SELECT
			ff.enabled
			FROM feature_flags ff
			WHERE ff.flag_key = 'schedule_lifecycle'
			AND NOT EXISTS (SELECT 1 FROM schedule_shadow_mismatches WHERE employer_id = $1 AND business_date BETWEEN $2::date AND $3::date AND status = 'open')
			AND NOT EXISTS (SELECT 1 FROM schedule_legacy_mutation_evidence e JOIN employments em ON em.id = e.employment_id WHERE em.employer_id = $1 AND e.operation = 'guard_blocked' AND e.business_date BETWEEN $2::date AND $3::date)
			AND NOT EXISTS (SELECT 1 FROM outbox_jobs WHERE job_type = 'ScheduleOfficeCoverageRefresh' AND status = 'dead_letter' AND payload->>'employer_id' = $1::text AND payload->>'business_date' BETWEEN $2 AND $3)
			AND EXISTS (SELECT 1 FROM schedule_office_policies p JOIN approval_workflow_instances i ON i.id = p.approval_instance_id WHERE p.employer_id = $1 AND p.status = 'active' AND p.effective_from <= $3::date AND (p.effective_to IS NULL OR p.effective_to >= $2::date) AND i.entity_type = 'schedule_policy' AND i.status = 'approved')
			AND NOT EXISTS (SELECT 1 FROM schedule_day_revisions r JOIN schedule_day_revisions d ON d.employment_id = r.employment_id AND d.business_date = r.business_date AND d.state IN ('published', 'explicit_off') AND r.state IN ('published', 'explicit_off') AND d.id <> r.id WHERE r.employer_id = $1 AND r.business_date BETWEEN $2::date AND $3::date)
			AND NOT EXISTS (SELECT 1 FROM employments e JOIN schedule_office_policies p ON p.employer_id = e.employer_id AND p.status = 'active' AND p.effective_from <= $3::date AND (p.effective_to IS NULL OR p.effective_to >= $2::date) JOIN approval_workflow_instances i ON i.id = p.approval_instance_id AND i.entity_type = 'schedule_policy' AND i.status = 'approved' CROSS JOIN generate_series(GREATEST($2::date, p.effective_from), LEAST($3::date, COALESCE(p.effective_to, $3::date)), interval '1 day') days WHERE e.employer_id = $1 AND e.deleted_at IS NULL AND e.start_date <= days.day::date AND (e.end_date IS NULL OR e.end_date >= days.day::date) AND NOT EXISTS (SELECT 1 FROM schedule_day_revisions r WHERE r.employment_id = e.id AND r.business_date = days.day::date AND r.state IN ('published', 'explicit_off')))
		`, employerID, startDate, endDate).Scan(&ready); err != nil {
			return err
		}
		if !ready {
			return api.ErrScheduleActivationNotReady
		}
		if _, err := tx.Exec(ctx, `INSERT INTO config_entries (id, scope_type, scope_id, config_key, value_bool, effective_from, effective_to, created_by, created_at, updated_at)
			VALUES (gen_random_uuid(), 'employer', $1, 'schedule.lifecycle.enabled', true, $2::date, $3::date, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
			ON CONFLICT (scope_type, scope_id, config_key, effective_from)
			DO UPDATE SET value_bool = EXCLUDED.value_bool, effective_to = EXCLUDED.effective_to, updated_at = CURRENT_TIMESTAMP`, employerID, startDate, endDate, actorID); err != nil {
			return err
		}
		_, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, actor_type, created_at) VALUES ($1, $2, 'schedule_lifecycle.activation.enabled', jsonb_build_object('start_date', $3, 'end_date', $4, 'reason', $5), 'human', CURRENT_TIMESTAMP)`, actorID, employerID, startDate, endDate, strings.TrimSpace(reason))
		return err
	})
}

func (r ScheduleRepository) DeactivateScheduleEmployer(ctx context.Context, employerID uuid.UUID, startDate, endDate, reason string, actorID uuid.UUID) error {
	if employerID == uuid.Nil || actorID == uuid.Nil || strings.TrimSpace(reason) == "" || len(strings.TrimSpace(reason)) > 2000 || startDate == "" || endDate == "" || startDate > endDate {
		return api.ErrScheduleActivationNotReady
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var hasEvidence bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (
			SELECT 1 FROM schedule_day_revisions r
			WHERE r.employer_id = $1 AND r.business_date BETWEEN $2::date AND $3::date
			  AND r.state IN ('published', 'explicit_off')
			UNION ALL
			SELECT 1 FROM attendance_closure_days d
			WHERE d.employer_id = $1 AND d.business_date BETWEEN $2::date AND $3::date
		)`, employerID, startDate, endDate).Scan(&hasEvidence); err != nil {
			return err
		}
		if hasEvidence {
			return api.ErrScheduleActivationEvidence
		}
		var exactRange bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (
			SELECT 1 FROM config_entries
			WHERE scope_type = 'employer' AND scope_id = $1 AND config_key = 'schedule.lifecycle.enabled'
			  AND effective_from = $2::date AND effective_to = $3::date AND value_bool = true
		)`, employerID, startDate, endDate).Scan(&exactRange); err != nil {
			return err
		}
		if !exactRange {
			return api.ErrScheduleActivationNotReady
		}
		if _, err := tx.Exec(ctx, `UPDATE config_entries SET value_bool = false, updated_at = CURRENT_TIMESTAMP
			WHERE scope_type = 'employer' AND scope_id = $1 AND config_key = 'schedule.lifecycle.enabled'
			  AND effective_from = $2::date AND effective_to = $3::date`, employerID, startDate, endDate); err != nil {
			return err
		}
		_, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, actor_type, created_at)
			VALUES ($1, $2, 'schedule_lifecycle.activation.disabled', jsonb_build_object('start_date', $3, 'end_date', $4, 'reason', $5), 'human', CURRENT_TIMESTAMP)`, actorID, employerID, startDate, endDate, strings.TrimSpace(reason))
		return err
	})
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
	version, selected, conflict := selectEffectiveShiftVersion(shifts, *day.shiftIDs[0], parseScheduleDate(day.date))
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
			breaks, err = normalizeBreaks([]api.ScheduleReplanBreak{{Start: *interval.breakStart, End: *interval.breakEnd, BreakPunchMode: valueOrDefault(interval.breakPunchMode, "none"), BreakCreditMode: valueOrDefault(interval.breakCreditMode, "scheduled")}}, startAt, loc)
			if err != nil {
				return nil, err
			}
		}
		result = append(result, map[string]any{"sort_order": interval.sortOrder, "start": start.Format("15:04:05"), "end": end.Format("15:04:05"), "start_day_offset": interval.startDayOffset, "end_day_offset": interval.endDayOffset, "start_utc_offset_minutes": interval.startUTCOffsetMinutes, "end_utc_offset_minutes": interval.endUTCOffsetMinutes, "start_at": startAt, "end_at": endAt, "shift_version_id": version.id.String(), "breaks": breaks, "break_punch_mode": valueOrDefault(interval.breakPunchMode, "none"), "break_credit_mode": valueOrDefault(interval.breakCreditMode, "scheduled")})
	}
	return result, nil
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
		if _, err := tx.Exec(ctx, `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ`); err != nil {
			return err
		}
		current, err := loadCurrentRevision(ctx, tx, employmentID, businessDate, false)
		if err != nil {
			return err
		}
		if current.RevisionNumber != request.ExpectedRevision {
			return ErrScheduleRevisionStale
		}
		if err := ensureMutationPeriodOpen(ctx, tx, current.EmployerID, businessDate); err != nil {
			return err
		}
		before, err := loadRevisionDocument(ctx, tx, current)
		if err != nil {
			return err
		}
		replacement, err := normalizeReplacement(request.Replacement, current.Timezone, businessDate, employmentID)
		if err != nil {
			return err
		}
		diff := map[string]any{"before": before, "after": replacement}
		diffHash, err := hashJSON(map[string]any{
			"original_revision_id": current.ID.String(),
			"expected_revision":    request.ExpectedRevision,
			"diff":                 diff,
		})
		if err != nil {
			return err
		}
		scope := map[string]any{
			"employment_id": employmentID.String(),
			"business_date": businessDate,
			"timezone":      current.Timezone,
		}
		result := map[string]any{
			"operation":            "replan",
			"original_revision_id": current.ID.String(),
			"expected_revision":    request.ExpectedRevision,
			"replacement":          replacement,
			"diff":                 diff,
			"diff_hash":            diffHash,
		}
		requestHash, err := hashJSON(replanHashInput(actorID, employmentID, businessDate, request, "", "", ""))
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
		if _, err := tx.Exec(ctx, `INSERT INTO schedule_previews (id, requested_by, actor_type, operation, request_hash, result_hash, scope_snapshot, result_snapshot, resolver_version, expires_at, created_at) VALUES ($1, $2, 'human', 'replan', $3, $4, $5, $6, $7, $8, $9)`, previewID, actorID, requestHash, resultHash, jsonArg(scope), jsonArg(result), lifecycleResolverVersion, expiresAt, createdAt); err != nil {
			return err
		}
		response = api.ScheduleReplanPreviewResponse{
			PreviewID:          previewID,
			PreviewHash:        resultHash,
			ExpiresAt:          expiresAt,
			OriginalRevisionID: current.ID,
			Replacement:        replacement,
			Diff:               diff,
			DiffHash:           diffHash,
		}
		return nil
	})
	if err != nil {
		return api.ScheduleReplanPreviewResponse{}, err
	}
	return response, nil
}

func (r ScheduleRepository) Publish(ctx context.Context, actorID uuid.UUID, idempotencyKey string, request api.SchedulePublishRequest) (api.SchedulePublicationResponse, error) {
	if err := validatePublishInput(idempotencyKey, request); err != nil {
		return api.SchedulePublicationResponse{}, err
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return api.SchedulePublicationResponse{}, err
	}
	var response api.SchedulePublicationResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := lockIdempotencyKey(ctx, tx, idempotencyKey); err != nil {
			return err
		}
		requestHash, err := hashJSON(publishHashInput(actorID, request, "", ""))
		if err != nil {
			return err
		}
		stored, found, err := loadBatchByIdempotency(ctx, tx, idempotencyKey)
		if err != nil {
			return err
		}
		if found {
			return replayPublication(actorID, requestHash, stored, &response)
		}
		preview, err := lockPreview(ctx, tx, actorID, request.PreviewID, "publish", request.PreviewHash)
		if err != nil {
			return err
		}
		if preview.RequestHash != requestHash {
			return ErrSchedulePreviewStale
		}
		if err := ensureMutationPeriodOpenForRange(ctx, tx, request.EmploymentIDs, request.StartDate, request.EndDate); err != nil {
			return err
		}
		if request.Mode != "rotating" {
			if err := validateHumanOfficeApproval(ctx, tx, request); err != nil {
				return err
			}
		}
		if err := verifyFirstPublicationTargets(ctx, tx, request); err != nil {
			return err
		}
		return publishPreview(ctx, tx, publicationActor{ID: &actorID, Type: "human"}, idempotencyKey, requestHash, request, preview, &response)
	})
	if err != nil {
		return api.SchedulePublicationResponse{}, err
	}
	return response, nil
}

// PublishSystemScheduleOffice publishes one approved future employment/day using the same publication transaction as human office publication.
func validateSystemOfficePublicationAuthority(ctx context.Context, tx DBTX, event ScheduleOfficeCoverageEvent) error {
	var allowed bool
	if err := tx.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1
		FROM schedule_office_policies p
		JOIN approval_workflow_instances i ON i.id = p.approval_instance_id
		JOIN employments e ON e.id = $2 AND e.employer_id = p.employer_id AND e.deleted_at IS NULL
		WHERE p.approval_instance_id = $1 AND p.employer_id = $3
		  AND p.status = 'active' AND i.entity_type = 'schedule_policy' AND i.status = 'approved'
		  AND p.effective_from <= $4::date AND (p.effective_to IS NULL OR p.effective_to >= $4::date)
		  AND e.start_date <= $4::date AND (e.end_date IS NULL OR e.end_date >= $4::date)
	)`, event.ApprovalInstanceID, event.EmploymentID, event.EmployerID, event.BusinessDate).Scan(&allowed); err != nil {
		return err
	}
	if !allowed {
		return ErrScheduleOfficePolicyUnavailable
	}
	return nil
}

func (r ScheduleRepository) PublishSystemScheduleOffice(ctx context.Context, payload []byte, idempotencyKey string) (string, error) {
	var event ScheduleOfficeCoverageEvent
	if err := json.Unmarshal(payload, &event); err != nil {
		return "", fmt.Errorf("decode schedule office event: %w", err)
	}
	if err := validateScheduleOfficeEvent(event); err != nil {
		return "", err
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return "", err
	}
	var batchID uuid.UUID
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := lockIdempotencyKey(ctx, tx, idempotencyKey); err != nil {
			return err
		}
		stored, found, err := loadBatchByIdempotency(ctx, tx, idempotencyKey)
		if err != nil {
			return err
		}
		if found {
			if stored.ActorType != "system" || stored.ActorName == nil || *stored.ActorName != "schedule-publisher" {
				return ErrScheduleIdempotencyConflict
			}
			batchID = stored.ID
			return nil
		}
		if err := validateSystemOfficePublicationAuthority(ctx, tx, event); err != nil {
			return err
		}
		if err := ensureMutationPeriodOpen(ctx, tx, event.EmployerID, event.BusinessDate); err != nil {
			return err
		}
		request := api.SchedulePublishRequest{
			EmploymentIDs:      []uuid.UUID{event.EmploymentID},
			StartDate:          event.BusinessDate,
			EndDate:            event.BusinessDate,
			ApprovalInstanceID: &event.ApprovalInstanceID,
			Reason:             "Approved office policy publication",
			ExpectedRevisions:  map[string]int64{event.EmploymentID.String() + ":" + event.BusinessDate: 0},
		}
		if err := verifyFirstPublicationTargets(ctx, tx, request); err != nil {
			if errors.Is(err, ErrScheduleAlreadyPublished) {
				return nil
			}
			return err
		}
		resolution, _, err := resolveScheduleData(ctx, tx, parseScheduleDate(event.BusinessDate), parseScheduleDate(event.BusinessDate), request.EmploymentIDs, 1, 1)
		if err != nil {
			return err
		}
		if len(resolution.employees) != 1 || len(resolution.employees[0].days) != 1 {
			return ErrScheduleResolverUnavailable
		}
		request.Timezone = resolution.employees[0].days[0].timezone
		day, err := sanitizedPreviewDay(resolution.employees[0].days[0], resolution.shifts)
		if err != nil {
			return err
		}
		day["employment_id"] = event.EmploymentID.String()
		if day["status"] != "work" && day["status"] != "explicit_off" {
			return ErrScheduleResolverUnavailable
		}
		result := map[string]any{"operation": "publish", "data": []map[string]any{day}, "resolver_version": lifecycleResolverVersion, "expected_revisions": expectedRevisionList(request.ExpectedRevisions)}
		resultHash, err := hashJSON(result)
		if err != nil {
			return err
		}
		requestHash, err := hashJSON(map[string]any{"operation": "publish", "actor_type": "system", "actor_name": "schedule-publisher", "job_id": idempotencyKey, "approval_instance_id": event.ApprovalInstanceID.String(), "employment_id": event.EmploymentID.String(), "business_date": event.BusinessDate})
		if err != nil {
			return err
		}
		createdAt := time.Now().UTC()
		previewID := uuid.New()
		if _, err := tx.Exec(ctx, `INSERT INTO schedule_previews (id, requested_by, actor_type, actor_name, operation, request_hash, result_hash, scope_snapshot, result_snapshot, resolver_version, expires_at, created_at) VALUES ($1, NULL, 'system', 'schedule-publisher', 'publish', $2, $3, $4, $5, $6, $7, $8)`, previewID, requestHash, resultHash, jsonArg(map[string]any{"employment_ids": request.EmploymentIDs, "start_date": event.BusinessDate, "end_date": event.BusinessDate, "approval_instance_id": event.ApprovalInstanceID}), jsonArg(result), lifecycleResolverVersion, createdAt.Add(previewTTL), createdAt); err != nil {
			return err
		}
		preview := lockedPreview{ID: previewID, RequestHash: requestHash, ResultHash: resultHash, ActorType: "system", ActorName: stringPtr("schedule-publisher"), ExpiresAt: createdAt.Add(previewTTL), ResultSnapshot: jsonArg(result)}
		actorName := "schedule-publisher"
		actor := publicationActor{Type: "system", Name: &actorName}
		if err := publishPreview(ctx, tx, actor, idempotencyKey, requestHash, request, preview, nil); err != nil {
			return err
		}
		if err := tx.QueryRow(ctx, `SELECT id FROM schedule_publication_batches WHERE idempotency_key = $1`, idempotencyKey).Scan(&batchID); err != nil {
			return err
		}
		return nil
	})
	if err != nil {
		return "", err
	}
	return batchID.String(), nil
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
			return replayReplan(actorID, requestHash, stored, &response)
		}
		preview, err := lockPreview(ctx, tx, actorID, request.PreviewID, "replan", request.PreviewHash)
		if err != nil {
			return err
		}
		if err := lockScheduleRevisionTargets(ctx, tx, []scheduleRevisionLockKey{{employmentID: employmentID, businessDate: businessDate}}); err != nil {
			return err
		}
		current, err := loadCurrentRevision(ctx, tx, employmentID, businessDate, true)
		if err != nil {
			return err
		}
		if current.RevisionNumber != request.ExpectedRevision {
			return ErrScheduleRevisionStale
		}
		if err := ensureMutationPeriodOpen(ctx, tx, current.EmployerID, businessDate); err != nil {
			return err
		}
		result, err := decodeReplanResult(preview.ResultSnapshot)
		if err != nil {
			return err
		}
		storedReplacementHash := canonicalStoredReplacementHash(result.Replacement, current.Timezone, businessDate, employmentID)
		if result.DiffHash != request.ConfirmDiffHash || storedReplacementHash == "" || storedReplacementHash != replacementHash(request.Replacement, current.Timezone, businessDate, employmentID) {
			return ErrSchedulePreviewStale
		}
		return commitReplan(ctx, tx, actorID, idempotencyKey, requestHash, request, current, result, &response)
	})
	if err != nil {
		return api.ScheduleReplanResponse{}, err
	}
	return response, nil
}

// DBTX helpers keep writes inside the transaction and make rollback behavior
// testable with the same pgx abstractions used by other repositories.
func lockIdempotencyKey(ctx context.Context, tx DBTX, key string) error {
	_, err := tx.Exec(ctx, `SELECT pg_advisory_xact_lock(hashtextextended('schedule-lifecycle:' || $1, 0))`, key)
	return err
}

func loadBatchByIdempotency(ctx context.Context, tx DBTX, key string) (storedBatch, bool, error) {
	var batch storedBatch
	err := tx.QueryRow(ctx, `SELECT id, request_hash, requested_by, actor_type, actor_name, response_snapshot FROM schedule_publication_batches WHERE idempotency_key = $1`, key).Scan(&batch.ID, &batch.RequestHash, &batch.RequestedBy, &batch.ActorType, &batch.ActorName, &batch.ResponseSnapshot)
	if errors.Is(err, pgx.ErrNoRows) {
		return storedBatch{}, false, nil
	}
	return batch, err == nil, err
}

type storedBatch struct {
	ID               uuid.UUID
	RequestHash      string
	RequestedBy      *uuid.UUID
	ActorType        string
	ActorName        *string
	ResponseSnapshot []byte
}

func replayPublication(actorID uuid.UUID, requestHash string, stored storedBatch, response *api.SchedulePublicationResponse) error {
	if stored.ActorType != "human" || stored.RequestedBy == nil || *stored.RequestedBy != actorID {
		return api.ErrScheduleForbidden
	}
	if strings.TrimSpace(stored.RequestHash) != requestHash {
		return ErrScheduleIdempotencyConflict
	}
	return json.Unmarshal(stored.ResponseSnapshot, response)
}

func replayReplan(actorID uuid.UUID, requestHash string, stored storedBatch, response *api.ScheduleReplanResponse) error {
	if stored.ActorType != "human" || stored.RequestedBy == nil || *stored.RequestedBy != actorID {
		return api.ErrScheduleForbidden
	}
	if strings.TrimSpace(stored.RequestHash) != requestHash {
		return ErrScheduleIdempotencyConflict
	}
	return json.Unmarshal(stored.ResponseSnapshot, response)
}

type lockedPreview struct {
	ID             uuid.UUID
	RequestHash    string
	ResultHash     string
	RequestedBy    *uuid.UUID
	ActorType      string
	ActorName      *string
	ExpiresAt      time.Time
	ResultSnapshot []byte
}

func lockPreview(ctx context.Context, tx DBTX, actorID, id uuid.UUID, operation, resultHash string) (lockedPreview, error) {
	var preview lockedPreview
	err := tx.QueryRow(ctx, `SELECT id, request_hash, result_hash, requested_by, actor_type, actor_name, expires_at, result_snapshot FROM schedule_previews WHERE id = $1 FOR UPDATE`, id).Scan(&preview.ID, &preview.RequestHash, &preview.ResultHash, &preview.RequestedBy, &preview.ActorType, &preview.ActorName, &preview.ExpiresAt, &preview.ResultSnapshot)
	if errors.Is(err, pgx.ErrNoRows) {
		return lockedPreview{}, ErrSchedulePreviewStale
	}
	if err != nil {
		return lockedPreview{}, err
	}
	if preview.ActorType != "human" || preview.RequestedBy == nil || *preview.RequestedBy != actorID || preview.ResultHash != resultHash || preview.ExpiresAt.Before(time.Now().UTC()) {
		return lockedPreview{}, ErrSchedulePreviewStale
	}
	var storedOperation struct {
		Operation string `json:"operation"`
	}
	if err := json.Unmarshal(preview.ResultSnapshot, &storedOperation); err == nil && storedOperation.Operation != "" && storedOperation.Operation != operation {
		return lockedPreview{}, ErrSchedulePreviewStale
	}
	return preview, nil
}

func loadCurrentRevision(ctx context.Context, tx DBTX, employmentID uuid.UUID, businessDate string, forUpdate bool) (lifecycleRevision, error) {
	var revision lifecycleRevision
	err := tx.QueryRow(ctx, `SELECT r.id, r.employment_id, r.employer_id, r.business_date::text, r.timezone, r.revision_number, r.state, r.publication_batch_id, r.source, r.source_snapshot, r.published_at, b.mode, b.approval_instance_id FROM schedule_day_revisions r LEFT JOIN schedule_publication_batches b ON b.id = r.publication_batch_id WHERE r.employment_id = $1 AND r.business_date = $2::date AND r.state IN ('published', 'explicit_off') ORDER BY r.revision_number DESC LIMIT 1`, employmentID, businessDate).Scan(&revision.ID, &revision.EmploymentID, &revision.EmployerID, &revision.BusinessDate, &revision.Timezone, &revision.RevisionNumber, &revision.State, &revision.BatchID, &revision.Source, &revision.SourceSnapshot, &revision.PublishedAt, &revision.Mode, &revision.ApprovalInstanceID)
	if errors.Is(err, pgx.ErrNoRows) {
		return lifecycleRevision{}, ErrScheduleNoCurrentRevision
	}
	if err != nil {
		return lifecycleRevision{}, err
	}
	if forUpdate {
		if _, err := tx.Exec(ctx, `SELECT id FROM schedule_day_revisions WHERE id = $1 FOR UPDATE`, revision.ID); err != nil {
			return lifecycleRevision{}, err
		}
	}
	return revision, nil
}

func ensureMutationPeriodOpen(ctx context.Context, tx DBTX, employerID uuid.UUID, businessDate string) error {
	var payrollLocked, attendanceClosed bool
	err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_periods WHERE employer_id = $1 AND status = 'locked' AND make_date(period_year, period_month, 1) <= $2::date AND (make_date(period_year, period_month, 1) + interval '1 month - 1 day') >= $2::date), EXISTS (SELECT 1 FROM attendance_periods WHERE employer_id = $1 AND status = 'closed' AND make_date(period_year, period_month, 1) <= $2::date AND (make_date(period_year, period_month, 1) + interval '1 month - 1 day') >= $2::date)`, employerID, businessDate).Scan(&payrollLocked, &attendanceClosed)
	if err != nil {
		return err
	}
	if payrollLocked || attendanceClosed {
		return ErrScheduleLockedPeriod
	}
	return nil
}

func ensureMutationPeriodOpenForRange(ctx context.Context, tx DBTX, employmentIDs []uuid.UUID, startDate, endDate string) error {
	var blocked bool
	err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_periods pp JOIN employments e ON e.employer_id = pp.employer_id WHERE e.id = ANY($1) AND pp.status = 'locked' AND make_date(pp.period_year, pp.period_month, 1) <= $3::date AND (make_date(pp.period_year, pp.period_month, 1) + interval '1 month - 1 day') >= $2::date) OR EXISTS (SELECT 1 FROM attendance_periods ap JOIN employments e ON e.employer_id = ap.employer_id WHERE e.id = ANY($1) AND ap.status = 'closed' AND make_date(ap.period_year, ap.period_month, 1) <= $3::date AND (make_date(ap.period_year, ap.period_month, 1) + interval '1 month - 1 day') >= $2::date)`, employmentIDs, startDate, endDate).Scan(&blocked)
	if err != nil {
		return err
	}
	if blocked {
		return ErrScheduleLockedPeriod
	}
	return nil
}

func verifyFirstPublicationTargets(ctx context.Context, tx DBTX, request api.SchedulePublishRequest) error {
	var current bool
	err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM schedule_day_revisions WHERE employment_id = ANY($1) AND business_date BETWEEN $2::date AND $3::date AND state IN ('published', 'explicit_off'))`, request.EmploymentIDs, request.StartDate, request.EndDate).Scan(&current)
	if err != nil {
		return err
	}
	if current {
		return ErrScheduleAlreadyPublished
	}
	return nil
}

type publicationActor struct {
	ID   *uuid.UUID
	Type string
	Name *string
}

type scheduleRevisionLockKey struct {
	employmentID uuid.UUID
	businessDate string
}

func lockScheduleRevisionTargets(ctx context.Context, tx DBTX, keys []scheduleRevisionLockKey) error {
	if len(keys) == 0 {
		return nil
	}
	unique := make(map[string]scheduleRevisionLockKey, len(keys))
	for _, key := range keys {
		if key.employmentID == uuid.Nil || key.businessDate == "" {
			return ErrSchedulePreviewScope
		}
		unique[key.employmentID.String()+":"+key.businessDate] = key
	}
	keys = keys[:0]
	for _, key := range unique {
		keys = append(keys, key)
	}
	sort.Slice(keys, func(i, j int) bool {
		left := keys[i].employmentID.String() + ":" + keys[i].businessDate
		right := keys[j].employmentID.String() + ":" + keys[j].businessDate
		return left < right
	})
	employmentIDs := make([]uuid.UUID, 0, len(keys))
	dates := make([]string, 0, len(keys))
	seenEmployments := make(map[uuid.UUID]struct{}, len(keys))
	for _, key := range keys {
		dates = append(dates, key.businessDate)
		if _, exists := seenEmployments[key.employmentID]; !exists {
			employmentIDs = append(employmentIDs, key.employmentID)
			seenEmployments[key.employmentID] = struct{}{}
		}
	}
	sort.Strings(dates)
	sort.Slice(employmentIDs, func(i, j int) bool { return employmentIDs[i].String() < employmentIDs[j].String() })
	if _, err := tx.Exec(ctx, `SELECT e.id FROM employees e JOIN employments em ON em.employee_id = e.id WHERE em.id = ANY($1) ORDER BY e.id FOR UPDATE`, employmentIDs); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `SELECT id FROM employments WHERE id = ANY($1) ORDER BY id FOR UPDATE`, employmentIDs); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `SELECT id FROM schedule_day_revisions WHERE employment_id = ANY($1) AND business_date BETWEEN $2::date AND $3::date FOR UPDATE`, employmentIDs, dates[0], dates[len(dates)-1]); err != nil {
		return err
	}
	return nil
}

func rejectConcreteScheduleOverlaps(ctx context.Context, tx DBTX, days []map[string]any) error {
	type candidate struct {
		EmploymentID string    `json:"employment_id"`
		BusinessDate string    `json:"business_date"`
		StartAt      time.Time `json:"start_at"`
		EndAt        time.Time `json:"end_at"`
	}
	candidates := make([]candidate, 0)
	for _, day := range days {
		if status, _ := day["status"].(string); status != "work" {
			continue
		}
		employmentID, err := uuidFromMap(day, "employment_id")
		if err != nil {
			return err
		}
		intervals, err := dayIntervals(day, "UTC", "")
		if err != nil {
			return err
		}
		businessDate, _ := day["business_date"].(string)
		for _, interval := range intervals {
			candidates = append(candidates, candidate{EmploymentID: employmentID.String(), BusinessDate: businessDate, StartAt: interval.StartAt, EndAt: interval.EndAt})
		}
	}
	if len(candidates) == 0 {
		return nil
	}
	var conflict bool
	err := tx.QueryRow(ctx, `
		WITH candidates AS (
			SELECT * FROM jsonb_to_recordset($1::jsonb) AS c(employment_id uuid, business_date date, start_at timestamptz, end_at timestamptz)
		), candidate_people AS (
			SELECT c.*, em.employee_id
			FROM candidates c JOIN employments em ON em.id = c.employment_id
		), existing AS (
			SELECT r.business_date, ws.scheduled_start_at AS start_at, ws.scheduled_end_at AS end_at, em.employee_id, ws.employment_id
			FROM work_segments ws
			JOIN schedule_day_revisions r ON r.id = ws.schedule_revision_id AND r.state IN ('published', 'explicit_off')
			JOIN employments em ON em.id = ws.employment_id
		)
		SELECT EXISTS (
			SELECT 1 FROM candidate_people c JOIN existing e ON e.employee_id = c.employee_id
			WHERE (e.employment_id <> c.employment_id OR e.business_date <> c.business_date)
			  AND c.start_at < e.end_at AND e.start_at < c.end_at
			UNION ALL
			SELECT 1 FROM candidate_people a JOIN candidate_people b ON a.employee_id = b.employee_id
			WHERE (a.employment_id, a.business_date, a.start_at) < (b.employment_id, b.business_date, b.start_at)
			  AND a.start_at < b.end_at AND b.start_at < a.end_at
		)`, jsonArg(candidates)).Scan(&conflict)
	if err != nil {
		return err
	}
	if conflict {
		return api.ErrScheduleConflict
	}
	return nil
}

func publishPreview(ctx context.Context, tx pgx.Tx, actor publicationActor, idempotencyKey, requestHash string, request api.SchedulePublishRequest, preview lockedPreview, response *api.SchedulePublicationResponse) error {
	var snapshot struct {
		Data []map[string]any `json:"data"`
	}
	if err := json.Unmarshal(preview.ResultSnapshot, &snapshot); err != nil || len(snapshot.Data) == 0 {
		return ErrScheduleResolverUnavailable
	}
	keys := make([]scheduleRevisionLockKey, 0, len(snapshot.Data))
	for _, day := range snapshot.Data {
		employmentID, err := uuidFromMap(day, "employment_id")
		if err != nil {
			return err
		}
		businessDate, ok := day["business_date"].(string)
		if !ok || businessDate == "" {
			return ErrSchedulePreviewScope
		}
		keys = append(keys, scheduleRevisionLockKey{employmentID: employmentID, businessDate: businessDate})
	}
	if err := lockScheduleRevisionTargets(ctx, tx, keys); err != nil {
		return err
	}
	if err := rejectConcreteScheduleOverlaps(ctx, tx, snapshot.Data); err != nil {
		return err
	}
	batchID := uuid.New()
	publishedAt := time.Now().UTC()
	mode := request.Mode
	if mode == "" {
		mode = "office"
	}
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, actor_type, actor_name, approval_instance_id, reason, timezone, range_start, range_end, published_at, affected_days) VALUES ($1, $2, $3, $4, 'published', $5, 'publish', '{}'::jsonb, $6, $7, $8, $9, $10, $11::date, $12::date, $13, $14)`, batchID, idempotencyKey, requestHash, mode, actor.ID, actor.Type, actor.Name, request.ApprovalInstanceID, strings.TrimSpace(request.Reason), request.Timezone, request.StartDate, request.EndDate, publishedAt, len(snapshot.Data)); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		return err
	}
	revisionIDs := make([]uuid.UUID, 0, len(snapshot.Data))
	for _, day := range snapshot.Data {
		revisionID, err := insertPublishedDay(ctx, tx, batchID, actor, request.Reason, request.Timezone, day, publishedAt, nil)
		if err != nil {
			return err
		}
		revisionIDs = append(revisionIDs, revisionID)
	}
	publicationResponse := api.SchedulePublicationResponse{BatchID: batchID, Status: "published", PublishedAt: publishedAt, RevisionIDs: revisionIDs, AffectedDays: len(revisionIDs)}
	if response != nil {
		*response = publicationResponse
	}
	responseJSON, err := json.Marshal(publicationResponse)
	if err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_publication_batches SET response_snapshot = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $1`, batchID, responseJSON); err != nil {
		return err
	}
	if _, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, created_at, actor_type, actor_name) VALUES ($1, $2, 'schedule.published', $3, CURRENT_TIMESTAMP, $4, $5)`, actor.ID, batchID, jsonArg(map[string]any{"batch_id": batchID.String(), "revision_ids": revisionIDs, "affected_days": len(revisionIDs)}), actor.Type, actor.Name); err != nil {
		return err
	}
	return nil
}

func insertPublishedDay(ctx context.Context, tx pgx.Tx, batchID uuid.UUID, actor publicationActor, reason, timezone string, day map[string]any, publishedAt time.Time, previousRevisionID *uuid.UUID) (uuid.UUID, error) {
	employmentID, err := uuidFromMap(day, "employment_id")
	if err != nil {
		return uuid.Nil, err
	}
	businessDate, ok := day["business_date"].(string)
	if !ok || businessDate == "" {
		return uuid.Nil, errors.New("schedule lifecycle preview has invalid business date")
	}
	status, _ := day["status"].(string)
	source, _ := day["source"].(string)
	if status != "work" && status != "explicit_off" {
		return uuid.Nil, ErrScheduleResolverUnavailable
	}
	if source == "" {
		return uuid.Nil, errors.New("schedule lifecycle preview has no source")
	}
	revisionID := uuid.New()
	var revisionNumber int64
	if err := tx.QueryRow(ctx, `SELECT COALESCE(MAX(revision_number), 0) + 1 FROM schedule_day_revisions WHERE employment_id = $1 AND business_date = $2::date`, employmentID, businessDate).Scan(&revisionNumber); err != nil {
		return uuid.Nil, err
	}
	sourceSnapshot := jsonArg(map[string]any{"source": source, "resolver_version": lifecycleResolverVersion, "preview_result": day})
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_actor_name, published_at, previous_revision_id) SELECT $1, e.id, e.employer_id, $2::date, $3, $4, 'draft', $5, $6, $7, $8, $9, $10, $11, $12, NULL, $13 FROM employments e WHERE e.id = $14`, revisionID, businessDate, timezone, revisionNumber, batchID, source, sourceSnapshot, lifecycleResolverVersion, strings.TrimSpace(reason), actor.ID, actor.Type, actor.Name, previousRevisionID, employmentID); err != nil {
		return uuid.Nil, err
	}
	intervals := []normalizedInterval(nil)
	if status == "work" {
		var err error
		intervals, err = dayIntervals(day, timezone, businessDate)
		if err != nil {
			return uuid.Nil, err
		}
	}
	groupKey := fmt.Sprintf("%s:%s", employmentID, businessDate)
	for _, interval := range intervals {
		if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, template_id, interval_id, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, source, source_detail, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot, start_day_offset, end_day_offset, shift_group_key, start_utc_offset_minutes, end_utc_offset_minutes) SELECT $1, $2, $3::date, sv.template_id, NULL, $4, $5::time, $6::time, 'work', 'scheduled', $7, '{}'::jsonb, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17 FROM shift_versions sv WHERE sv.id = $9`, uuid.New(), employmentID, businessDate, interval.SortOrder, interval.StartLocal, interval.EndLocal, source, revisionID, interval.ShiftVersionID, interval.StartAt, interval.EndAt, jsonArg(interval.Breaks), interval.StartDayOffset, interval.EndDayOffset, groupKey, interval.StartUTCOffsetMinutes, interval.EndUTCOffsetMinutes); err != nil {
			return uuid.Nil, err
		}
	}
	state := "published"
	if status == "explicit_off" {
		state = "explicit_off"
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = $2, published_at = $3 WHERE id = $1`, revisionID, state, publishedAt); err != nil {
		return uuid.Nil, err
	}
	return revisionID, nil
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
	BreakCreditMode       string           `json:"break_credit_mode"`
}

func localDateTime(date time.Time, clock time.Time, explicitOffset *int) (time.Time, error) {
	loc := date.Location()
	candidates := make([]time.Time, 0, 2)
	for offset := -14 * 60; offset <= 14*60; offset += 15 {
		instant := time.Date(date.Year(), date.Month(), date.Day(), clock.Hour(), clock.Minute(), clock.Second(), 0, time.FixedZone("candidate", offset*60))
		inLocation := instant.In(loc)
		if inLocation.Year() == date.Year() && inLocation.Month() == date.Month() && inLocation.Day() == date.Day() && inLocation.Hour() == clock.Hour() && inLocation.Minute() == clock.Minute() && inLocation.Second() == clock.Second() {
			found := false
			for _, candidate := range candidates {
				if candidate.Equal(instant) {
					found = true
					break
				}
			}
			if !found {
				candidates = append(candidates, instant)
			}
		}
	}
	if len(candidates) == 0 {
		return time.Time{}, errors.New("schedule local time does not exist in timezone")
	}
	if explicitOffset != nil {
		for _, candidate := range candidates {
			_, offset := candidate.In(loc).Zone()
			if offset == *explicitOffset*60 {
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
		start, err := parseLocalTime(item.Start)
		if err != nil {
			return nil, err
		}
		end, err := parseLocalTime(item.End)
		if err != nil {
			return nil, err
		}
		year, month, day := parseDateParts(businessDate)
		if item.StartDayOffset < 0 || item.EndDayOffset < item.StartDayOffset {
			return nil, errors.New("schedule replacement day offsets are invalid")
		}
		startDate := time.Date(year, month, day+item.StartDayOffset, 0, 0, 0, 0, loc)
		endDate := time.Date(year, month, day+item.EndDayOffset, 0, 0, 0, 0, loc)
		startAt, err := localDateTime(startDate, start, item.StartUTCOffsetMinutes)
		if err != nil {
			return nil, err
		}
		endAt, err := localDateTime(endDate, end, item.EndUTCOffsetMinutes)
		if err != nil {
			return nil, err
		}
		if !endAt.After(startAt) {
			return nil, errors.New("schedule lifecycle replacement interval ends before it starts")
		}
		if !endAt.After(startAt) {
			return nil, errors.New("schedule lifecycle replacement interval ends before it starts")
		}
		breaks, err := normalizeBreaks(item.Breaks, startAt, loc)
		if err != nil {
			return nil, err
		}
		intervals = append(intervals, normalizedInterval{SortOrder: item.SortOrder, StartLocal: start.Format("15:04:05"), EndLocal: end.Format("15:04:05"), StartDayOffset: item.StartDayOffset, EndDayOffset: item.EndDayOffset, StartUTCOffsetMinutes: item.StartUTCOffsetMinutes, EndUTCOffsetMinutes: item.EndUTCOffsetMinutes, StartAt: startAt, EndAt: endAt, ShiftVersionID: *replacement.ShiftVersionID, Breaks: breaks, BreakPunchMode: valueOrDefault(&item.BreakPunchMode, "none"), BreakCreditMode: valueOrDefault(&item.BreakCreditMode, "scheduled")})
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
		result = append(result, map[string]any{"start": start.Format("15:04:05"), "end": end.Format("15:04:05"), "start_at": breakStart, "end_at": breakEnd, "break_punch_mode": valueOrDefault(&item.BreakPunchMode, "none"), "break_credit_mode": valueOrDefault(&item.BreakCreditMode, "scheduled")})
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

func hashMap(value any) string {
	hash, err := hashJSON(value)
	if err != nil {
		return ""
	}
	return hash
}

func stringPtr(value string) *string {
	if value == "" {
		return nil
	}
	return &value
}

func publishHashInput(actorID uuid.UUID, request api.SchedulePublishRequest, previewID, previewHash string) lifecycleRequestHashInput {
	approval := ""
	if request.ApprovalInstanceID != nil {
		approval = request.ApprovalInstanceID.String()
	}
	mode := request.Mode
	if mode == "" {
		mode = "office"
	}
	return lifecycleRequestHashInput{Operation: "publish", ActorID: actorID.String(), EmploymentIDs: uuidStrings(request.EmploymentIDs), StartDate: request.StartDate, EndDate: request.EndDate, Timezone: request.Timezone, Mode: mode, ApprovalInstanceID: stringPtr(approval), ExpectedRevisions: expectedRevisionList(request.ExpectedRevisions), Reason: strings.TrimSpace(request.Reason), Retroactive: request.Retroactive, PreviewID: previewID, PreviewHash: previewHash}
}

func replanHashInput(actorID, employmentID uuid.UUID, businessDate string, request api.ScheduleReplanPreviewRequest, previewID, previewHash, confirmDiffHash string) lifecycleRequestHashInput {
	return lifecycleRequestHashInput{Operation: "replan", ActorID: actorID.String(), EmploymentID: employmentID.String(), BusinessDate: businessDate, ExpectedRevision: request.ExpectedRevision, Reason: strings.TrimSpace(request.Reason), Replacement: &request.Replacement, PreviewID: previewID, PreviewHash: previewHash, ConfirmDiffHash: confirmDiffHash}
}

func expectedRevisionList(values map[string]int64) []lifecycleExpectedRevision {
	result := make([]lifecycleExpectedRevision, 0, len(values))
	for key, value := range values {
		result = append(result, lifecycleExpectedRevision{Key: strings.ToLower(strings.TrimSpace(key)), Value: value})
	}
	sort.Slice(result, func(i, j int) bool { return result[i].Key < result[j].Key })
	return result
}

func uuidStrings(values []uuid.UUID) []string {
	result := make([]string, len(values))
	for i, value := range values {
		result[i] = value.String()
	}
	sort.Strings(result)
	return result
}

func hashJSON(value any) (string, error) {
	payload, err := json.Marshal(value)
	if err != nil {
		return "", err
	}
	digest := sha256.Sum256(payload)
	return hex.EncodeToString(digest[:]), nil
}

func jsonArg(value any) []byte {
	payload, _ := json.Marshal(value)
	return payload
}

func validatePublishInput(key string, request api.SchedulePublishRequest) error {
	mode := request.Mode
	if mode == "" {
		mode = "office"
	}
	if strings.TrimSpace(key) == "" || len(key) > 128 || !isASCIIKey(key) || (mode != "office" && mode != "rotating") || (mode == "office" && request.ApprovalInstanceID == nil) || (mode == "rotating" && request.ApprovalInstanceID != nil) || request.PreviewID == uuid.Nil || !isHash(request.PreviewHash) || strings.TrimSpace(request.Reason) == "" || len(request.EmploymentIDs) == 0 {
		return api.ErrScheduleValidation
	}
	if _, err := time.Parse("2006-01-02", request.StartDate); err != nil {
		return api.ErrScheduleValidation
	}
	if _, err := time.Parse("2006-01-02", request.EndDate); err != nil {
		return api.ErrScheduleValidation
	}
	if _, err := time.LoadLocation(request.Timezone); err != nil {
		return api.ErrScheduleValidation
	}
	return nil
}

func validateReplanInput(employmentID uuid.UUID, businessDate string, request api.ScheduleReplanPreviewRequest) error {
	if employmentID == uuid.Nil || request.ExpectedRevision <= 0 || len(strings.TrimSpace(request.Reason)) == 0 || len(strings.TrimSpace(request.Reason)) > 2000 || !validReplacementShape(request.Replacement) {
		return api.ErrScheduleValidation
	}
	if _, err := time.Parse("2006-01-02", businessDate); err != nil {
		return api.ErrScheduleValidation
	}
	return nil
}

func validateCommitReplanInput(employmentID uuid.UUID, businessDate, key string, request api.ScheduleReplanRequest) error {
	if err := validateReplanInput(employmentID, businessDate, api.ScheduleReplanPreviewRequest{ExpectedRevision: request.ExpectedRevision, Reason: request.Reason, Replacement: request.Replacement}); err != nil {
		return err
	}
	if strings.TrimSpace(key) == "" || len(key) > 128 || !isASCIIKey(key) || request.PreviewID == uuid.Nil || !isHash(request.PreviewHash) || !isHash(request.ConfirmDiffHash) {
		return api.ErrScheduleValidation
	}
	return nil
}

func validReplacementShape(value api.ScheduleReplanReplacement) bool {
	return value.Status == "work" && value.ShiftVersionID != nil && value.Source != "" && len(value.Intervals) > 0
}

func isASCIIKey(value string) bool {
	for i := range value {
		if value[i] < 0x20 || value[i] > 0x7e {
			return false
		}
	}
	return true
}

func isHash(value string) bool {
	if len(value) != 64 {
		return false
	}
	_, err := hex.DecodeString(value)
	return err == nil && strings.ToLower(value) == value
}

func uuidFromMap(value map[string]any, key string) (uuid.UUID, error) {
	text, ok := value[key].(string)
	if !ok {
		return uuid.Nil, errors.New("schedule lifecycle preview has invalid UUID")
	}
	return uuid.Parse(text)
}

func parseLocalTime(value string) (time.Time, error) {
	for _, layout := range []string{"15:04", "15:04:05"} {
		if parsed, err := time.Parse(layout, value); err == nil {
			return parsed, nil
		}
	}
	return time.Time{}, fmt.Errorf("invalid local time %q", value)
}

func parseDateParts(value string) (int, time.Month, int) {
	parsed, _ := time.Parse("2006-01-02", value)
	return parsed.Year(), parsed.Month(), parsed.Day()
}

func dayIntervals(day map[string]any, timezone string, businessDate string) ([]normalizedInterval, error) {
	payload, ok := day["intervals"]
	if !ok {
		return nil, errors.New("schedule lifecycle preview has no intervals")
	}
	encoded, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}
	var intervals []struct {
		SortOrder      int              `json:"sort_order"`
		Start          string           `json:"start"`
		End            string           `json:"end"`
		StartAt        time.Time        `json:"start_at"`
		EndAt          time.Time        `json:"end_at"`
		ShiftVersionID string           `json:"shift_version_id"`
		Breaks         []map[string]any `json:"breaks"`
			BreakPunchMode  string           `json:"break_punch_mode"`
			BreakCreditMode string           `json:"break_credit_mode"`
	}
	if err := json.Unmarshal(encoded, &intervals); err != nil {
		return nil, err
	}
	result := make([]normalizedInterval, 0, len(intervals))
	for _, item := range intervals {
		versionID, err := uuid.Parse(item.ShiftVersionID)
		if err != nil {
			return nil, err
		}
		result = append(result, normalizedInterval{SortOrder: item.SortOrder, StartLocal: item.Start, EndLocal: item.End, StartAt: item.StartAt, EndAt: item.EndAt, ShiftVersionID: versionID, Breaks: item.Breaks, BreakPunchMode: item.BreakPunchMode, BreakCreditMode: item.BreakCreditMode})
	}
	if len(result) == 0 {
		return nil, errors.New("schedule lifecycle preview has no work intervals")
	}
	_ = timezone
	_ = businessDate
	return result, nil
}
