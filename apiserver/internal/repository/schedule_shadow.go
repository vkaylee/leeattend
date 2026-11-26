package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"sort"
	"time"

	"github.com/google/uuid"
)

const (
	scheduleShadowMaxDays             = 62
	scheduleShadowEmploymentBatchSize = 100
)

func shadowDateOnly(value time.Time) time.Time {
	return time.Date(value.Year(), value.Month(), value.Day(), 0, 0, 0, 0, time.UTC)
}

func shadowDaysInRange(startDate, endDate time.Time) int {
	return int(endDate.Sub(startDate).Hours()/24) + 1
}

type ScheduleShadowRunResult struct {
	ComparedDays  int
	MismatchCount int
}

type legacyShadowDay struct {
	EmploymentID uuid.UUID
	BusinessDate string
	Status       string
	TemplateIDs  []uuid.UUID
	Intervals    []shadowInterval
	RosterIDs    []uuid.UUID
	SegmentIDs   []uuid.UUID
}

type shadowInterval struct {
	SortOrder int    `json:"sort_order"`
	Start     string `json:"start"`
	End       string `json:"end"`
}

func (r ScheduleRepository) RunScheduleShadowComparison(ctx context.Context, employerID uuid.UUID, startDate, endDate time.Time) (ScheduleShadowRunResult, error) {
	startDate, endDate = shadowDateOnly(startDate), shadowDateOnly(endDate)
	if employerID == uuid.Nil || endDate.Before(startDate) || shadowDaysInRange(startDate, endDate) > scheduleShadowMaxDays {
		return ScheduleShadowRunResult{}, errors.New("schedule shadow comparison scope is invalid")
	}
	db, err := requireDB(r.DB, "schedule repository")
	if err != nil {
		return ScheduleShadowRunResult{}, err
	}
	start, end := startDate.Format("2006-01-02"), endDate.Format("2006-01-02")
	employmentIDs, err := loadShadowEmploymentIDs(ctx, db, employerID, start, end)
	if err != nil {
		return ScheduleShadowRunResult{}, err
	}
	if len(employmentIDs) == 0 {
		return ScheduleShadowRunResult{}, nil
	}
	result := ScheduleShadowRunResult{}
	for offset := 0; offset < len(employmentIDs); offset += scheduleShadowEmploymentBatchSize {
		endOffset := offset + scheduleShadowEmploymentBatchSize
		if endOffset > len(employmentIDs) {
			endOffset = len(employmentIDs)
		}
		batchResult, err := r.runScheduleShadowBatch(ctx, db, employerID, employmentIDs[offset:endOffset], startDate, endDate, start, end)
		if err != nil {
			return ScheduleShadowRunResult{}, err
		}
		result.ComparedDays += batchResult.ComparedDays
		result.MismatchCount += batchResult.MismatchCount
	}
	return result, nil
}

func (r ScheduleRepository) runScheduleShadowBatch(ctx context.Context, db DBTX, employerID uuid.UUID, employmentIDs []uuid.UUID, startDate, endDate time.Time, start, end string) (ScheduleShadowRunResult, error) {
	legacy, err := loadLegacyShadowDays(ctx, db, employmentIDs, start, end)
	if err != nil {
		return ScheduleShadowRunResult{}, err
	}
	resolution, _, err := resolveScheduleData(ctx, db, startDate, endDate, employmentIDs, 1, len(employmentIDs))
	if err != nil {
		return ScheduleShadowRunResult{}, err
	}
	resolver := make(map[string]map[string]any, len(employmentIDs)*scheduleShadowMaxDays)
	for _, employee := range resolution.employees {
		for _, day := range employee.days {
			snapshot, err := sanitizedPreviewDay(day, resolution.shifts)
			if err != nil {
				return ScheduleShadowRunResult{}, err
			}
			snapshot["employment_id"] = employee.employmentID.String()
			resolver[shadowDayKey(employee.employmentID, day.date)] = snapshot
		}
	}
	records := make([]ScheduleShadowMismatchRecord, 0)
	compared := 0
	for _, employmentID := range employmentIDs {
		for date := startDate; !date.After(endDate); date = date.AddDate(0, 0, 1) {
			dateString := date.Format("2006-01-02")
			key := shadowDayKey(employmentID, dateString)
			resolverSnapshot := resolver[key]
			if resolverSnapshot == nil {
				return ScheduleShadowRunResult{}, fmt.Errorf("resolver omitted %s", key)
			}
			legacyDay := legacy[key]
			if legacyDay == nil {
				legacyDay = &legacyShadowDay{EmploymentID: employmentID, BusinessDate: dateString, Status: "missing"}
			}
			legacySnapshot := legacyShadowSnapshot(*legacyDay)
			for _, kind := range classifyShadowMismatch(legacySnapshot, resolverSnapshot) {
				legacyJSON, err := json.Marshal(legacySnapshot)
				if err != nil {
					return ScheduleShadowRunResult{}, err
				}
				resolverJSON, err := json.Marshal(resolverSnapshot)
				if err != nil {
					return ScheduleShadowRunResult{}, err
				}
				records = append(records, ScheduleShadowMismatchRecord{EmployerID: employerID, EmploymentID: employmentID, BusinessDate: dateString, MismatchKind: kind, LegacySnapshot: legacyJSON, ResolverSnapshot: resolverJSON, ResolverVersion: lifecycleResolverVersion})
			}
			compared++
		}
	}
	if err := r.RecordScheduleMismatches(ctx, records); err != nil {
		return ScheduleShadowRunResult{}, err
	}
	return ScheduleShadowRunResult{ComparedDays: compared, MismatchCount: len(records)}, nil
}

func loadShadowEmploymentIDs(ctx context.Context, db DBTX, employerID uuid.UUID, startDate, endDate string) ([]uuid.UUID, error) {
	rows, err := db.Query(ctx, `SELECT id FROM employment_segments WHERE employer_id = $1 AND deleted_at IS NULL AND start_date <= $2::date AND (end_date IS NULL OR end_date >= $3::date) ORDER BY id`, employerID, endDate, startDate)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]uuid.UUID, 0)
	for rows.Next() {
		var employmentID uuid.UUID
		if err := rows.Scan(&employmentID); err != nil {
			return nil, err
		}
		result = append(result, employmentID)
	}
	return result, rows.Err()
}

func loadLegacyShadowDays(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, startDate, endDate string) (map[string]*legacyShadowDay, error) {
	result := make(map[string]*legacyShadowDay)
	rows, err := db.Query(ctx, `SELECT id, employment_id, template_id, date::text FROM shift_rosters WHERE employment_id = ANY($1) AND date BETWEEN $2::date AND $3::date ORDER BY date, id`, employmentIDs, startDate, endDate)
	if err != nil {
		return nil, err
	}
	for rows.Next() {
		var id, employmentID uuid.UUID
		var templateID *uuid.UUID
		var date string
		if err := rows.Scan(&id, &employmentID, &templateID, &date); err != nil {
			rows.Close()
			return nil, err
		}
		day := result[shadowDayKey(employmentID, date)]
		if day == nil {
			day = &legacyShadowDay{EmploymentID: employmentID, BusinessDate: date}
			result[shadowDayKey(employmentID, date)] = day
		}
		day.RosterIDs = append(day.RosterIDs, id)
		if templateID == nil {
			day.Status = "explicit_off"
			continue
		}
		day.Status = "work"
		day.TemplateIDs = append(day.TemplateIDs, *templateID)
	}
	if err := rows.Err(); err != nil {
		rows.Close()
		return nil, err
	}
	rows.Close()
	rows, err = db.Query(ctx, `SELECT id, employment_id, date::text, template_id, sort_order, scheduled_start::text, scheduled_end::text FROM work_segments WHERE employment_id = ANY($1) AND date BETWEEN $2::date AND $3::date AND schedule_revision_id IS NULL AND segment_status <> 'cancelled' ORDER BY date, employment_id, sort_order, id`, employmentIDs, startDate, endDate)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var id, employmentID, templateID uuid.UUID
		var date, start, end string
		var sortOrder int
		if err := rows.Scan(&id, &employmentID, &date, &templateID, &sortOrder, &start, &end); err != nil {
			return nil, err
		}
		day := result[shadowDayKey(employmentID, date)]
		if day == nil {
			day = &legacyShadowDay{EmploymentID: employmentID, BusinessDate: date}
			result[shadowDayKey(employmentID, date)] = day
		}
		day.Status = "work"
		day.SegmentIDs = append(day.SegmentIDs, id)
		day.TemplateIDs = append(day.TemplateIDs, templateID)
		day.Intervals = append(day.Intervals, shadowInterval{SortOrder: sortOrder, Start: start, End: end})
	}
	return result, rows.Err()
}

func legacyShadowSnapshot(day legacyShadowDay) map[string]any {
	intervals := make([]shadowInterval, len(day.Intervals))
	copy(intervals, day.Intervals)
	sort.SliceStable(intervals, func(i, j int) bool { return intervals[i].SortOrder < intervals[j].SortOrder })
	return map[string]any{"employment_id": day.EmploymentID.String(), "business_date": day.BusinessDate, "status": day.Status, "source": "legacy", "timezone": "unknown", "template_ids": uuidStringsForShadow(day.TemplateIDs), "intervals": intervals, "legacy_row_ids": map[string]any{"roster_ids": uuidStringsForShadow(day.RosterIDs), "work_segment_ids": uuidStringsForShadow(day.SegmentIDs)}, "provenance": "legacy_unverified"}
}

func classifyShadowMismatch(legacy, resolver map[string]any) []string {
	legacyStatus, _ := legacy["status"].(string)
	resolverStatus, _ := resolver["status"].(string)
	result := make([]string, 0, 2)
	if legacyStatus != resolverStatus {
		if resolverStatus == "explicit_off" || legacyStatus == "explicit_off" {
			return append(result, "explicit_off")
		}
		return append(result, "missing")
	}
	if legacyStatus != "work" {
		return result
	}
	legacyIntervals := shadowIntervalSignature(legacy["intervals"])
	resolverIntervals := shadowIntervalSignature(resolver["intervals"])
	if !equalShadowIntervals(legacyIntervals, resolverIntervals) {
		kind := "intervals"
		if hasOvernightShadowInterval(legacyIntervals) || hasOvernightShadowInterval(resolverIntervals) {
			kind = "overnight"
		}
		result = append(result, kind)
	}
	return result
}

func shadowIntervalSignature(value any) []shadowInterval {
	if typed, ok := value.([]shadowInterval); ok {
		result := append([]shadowInterval(nil), typed...)
		sort.SliceStable(result, func(i, j int) bool { return result[i].SortOrder < result[j].SortOrder })
		return result
	}
	items := make([]map[string]any, 0)
	switch typed := value.(type) {
	case []any:
		for _, item := range typed {
			fields, ok := item.(map[string]any)
			if ok {
				items = append(items, fields)
			}
		}
	case []map[string]any:
		items = typed
	default:
		return nil
	}
	result := make([]shadowInterval, 0, len(items))
	for _, fields := range items {
		order := 0
		if raw, ok := fields["sort_order"].(float64); ok {
			order = int(raw)
		}
		start, _ := fields["start"].(string)
		end, _ := fields["end"].(string)
		result = append(result, shadowInterval{SortOrder: order, Start: start, End: end})
	}
	sort.SliceStable(result, func(i, j int) bool { return result[i].SortOrder < result[j].SortOrder })
	return result
}

func equalShadowIntervals(left, right []shadowInterval) bool {
	if len(left) != len(right) {
		return false
	}
	for i := range left {
		if left[i].SortOrder != right[i].SortOrder || left[i].Start != right[i].Start || left[i].End != right[i].End {
			return false
		}
	}
	return true
}

func hasOvernightShadowInterval(intervals []shadowInterval) bool {
	for _, interval := range intervals {
		if interval.Start != "" && interval.End != "" && interval.End < interval.Start {
			return true
		}
	}
	return false
}

func shadowDayKey(employmentID uuid.UUID, date string) string {
	return employmentID.String() + ":" + date
}

func uuidStringsForShadow(ids []uuid.UUID) []string {
	result := make([]string, len(ids))
	for i, id := range ids {
		result[i] = id.String()
	}
	return result
}
