package repository

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func TestGetOfficialSchedulesQueryUsesCurrentRevisionAndOfficialSegments(t *testing.T) {
	queries := make([]string, 0, 2)
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			queries = append(queries, query)
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*int64)) = 0; return nil }}
		},
		queryFn: func(_ context.Context, query string, _ ...any) (pgx.Rows, error) {
			queries = append(queries, query)
			return &fakeRows{}, nil
		},
	}
	if _, err := (ScheduleRepository{DB: db}).GetOfficialSchedules(context.Background(), time.Date(2026, 10, 1, 0, 0, 0, 0, time.UTC), time.Date(2026, 10, 2, 0, 0, 0, 0, time.UTC), nil, 1, 50); err != nil {
		t.Fatalf("official schedules: %v", err)
	}
	if len(queries) != 2 || !strings.Contains(queries[1], "schedule_day_revisions") || !strings.Contains(queries[1], "state IN ('published', 'explicit_off')") || !strings.Contains(queries[1], "ws.schedule_revision_id = r.id") || !strings.Contains(queries[1], "sv.id = ws.selected_shift_version_id") {
		t.Fatalf("official query missing current revision joins: %v", queries)
	}
	if strings.Contains(queries[1], "shift_rosters") || strings.Contains(queries[1], "assignment_rules") {
		t.Fatalf("official query must not resolve legacy sources: %s", queries[1])
	}
}

func TestLoadDeptHistory_UsesEmploymentSegments(t *testing.T) {
	employeeID := uuid.New()
	departmentID := uuid.New()
	var query string
	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
			query = sql
			if len(args) != 1 || args[0].([]uuid.UUID)[0] != employeeID {
				t.Fatalf("unexpected employee IDs: %v", args)
			}
			return &fakeRows{rows: [][]any{{employeeID, departmentID, "2026-01-01"}}}, nil
		},
	}

	history, err := loadDeptHistory(context.Background(), db, []uuid.UUID{employeeID})
	if err != nil {
		t.Fatalf("load department history: %v", err)
	}
	if strings.Contains(query, "employee_department_history") {
		t.Fatalf("stale department history table query: %s", query)
	}
	if !strings.Contains(query, "FROM employments") || !strings.Contains(query, "deleted_at IS NULL") {
		t.Fatalf("expected active employment segments query, got %s", query)
	}
	entries := history[employeeID]
	if len(entries) != 1 || entries[0].departmentID != departmentID {
		t.Fatalf("unexpected department history: %+v", entries)
	}
}

func TestResolveEffectiveShift_RosterOverridesPatternAndFixed(t *testing.T) {
	employmentID, employeeID, rosterShift, fixedShift := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	date := time.Date(2026, time.October, 20, 0, 0, 0, 0, time.UTC)
	rosters := map[string]*uuid.UUID{employmentID.String() + ":2026-10-20": new(rosterShift)}
	patterns := []patternEntry{{patternID: uuid.New(), startDate: date.AddDate(0, 0, -1), scheduleDay: int(date.Weekday()), templateID: new(uuid.UUID)}}
	fixed := []fixedShiftEntry{{templateID: fixedShift, startDate: date.AddDate(0, 0, -1)}}

	shifts, source, traces := resolveEffectiveShift(employmentID, employeeID, "E-1", nil, date, int(date.Weekday()), nil, nil, rosters, patterns, nil, fixed, nil)
	if source != "roster" || len(shifts) != 1 || shifts[0] == nil || *shifts[0] != rosterShift {
		t.Fatalf("expected roster override, got source=%q shifts=%v", source, shifts)
	}
	if len(traces) != 1 || traces[0].Priority != 100 {
		t.Fatalf("expected roster trace, got %+v", traces)
	}
}

func TestResolveEffectiveShift_PatternPrecedesRuleAndFixed(t *testing.T) {
	employmentID, employeeID, patternShift, ruleShift, fixedShift := uuid.New(), uuid.New(), uuid.New(), uuid.New(), uuid.New()
	date := time.Date(2026, time.October, 20, 0, 0, 0, 0, time.UTC)
	patterns := []patternEntry{{patternID: uuid.New(), startDate: date.AddDate(0, 0, -1), scheduleDay: int(date.Weekday()), templateID: new(patternShift)}}
	rule := assignmentRuleWithDetails{id: uuid.New(), name: "Rule", priority: 10, details: map[int][]uuid.UUID{int(date.Weekday()): {ruleShift}}}
	fixed := []fixedShiftEntry{{templateID: fixedShift, startDate: date.AddDate(0, 0, -1)}}

	shifts, source, _ := resolveEffectiveShift(employmentID, employeeID, "E-1", nil, date, int(date.Weekday()), nil, nil, nil, patterns, []assignmentRuleWithDetails{rule}, fixed, nil)
	if source != "pattern" || len(shifts) != 1 || shifts[0] == nil || *shifts[0] != patternShift {
		t.Fatalf("expected pattern precedence, got source=%q shifts=%v", source, shifts)
	}
}

func TestResolveEffectiveShift_RotationUsesCalendarDates(t *testing.T) {
	employmentID, employeeID, shift := uuid.New(), uuid.New(), uuid.New()
	location, err := time.LoadLocation("America/New_York")
	if err != nil {
		t.Fatalf("load timezone: %v", err)
	}
	anchor := time.Date(2026, time.March, 7, 0, 0, 0, 0, location)
	date := time.Date(2026, time.March, 9, 0, 0, 0, 0, location)
	cycleLength := 7
	patterns := []patternEntry{{patternID: uuid.New(), startDate: anchor, anchorDate: &anchor, patternType: "rotation", cycleLengthDays: &cycleLength, scheduleDay: 2, templateID: &shift}}

	shifts, source, _ := resolveEffectiveShift(employmentID, employeeID, "E-1", nil, date, int(date.Weekday()), nil, nil, nil, patterns, nil, nil, nil)
	if source != "pattern" || len(shifts) != 1 || shifts[0] == nil || *shifts[0] != shift {
		t.Fatalf("expected calendar-day rotation across DST, got source=%q shifts=%v", source, shifts)
	}
}

func TestResolveEffectiveShift_SameHighestPriorityRulesConflict(t *testing.T) {
	employmentID, employeeID, firstShift, secondShift := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	date := time.Date(2026, time.October, 20, 0, 0, 0, 0, time.UTC)
	rules := []assignmentRuleWithDetails{
		{id: uuid.New(), name: "First", priority: 10, details: map[int][]uuid.UUID{int(date.Weekday()): {firstShift}}},
		{id: uuid.New(), name: "Second", priority: 10, details: map[int][]uuid.UUID{int(date.Weekday()): {secondShift}}},
	}

	shifts, source, traces := resolveEffectiveShift(employmentID, employeeID, "E-1", nil, date, int(date.Weekday()), nil, nil, nil, nil, rules, nil, nil)
	if source != "conflict" || len(shifts) != 1 || shifts[0] != nil || len(traces) != 2 {
		t.Fatalf("expected same-priority conflict, got source=%q shifts=%v traces=%+v", source, shifts, traces)
	}
}

func TestResolveEffectiveShift_NullRosterAndPatternAreExplicitOff(t *testing.T) {
	employmentID, employeeID := uuid.New(), uuid.New()
	date := time.Date(2026, time.October, 20, 0, 0, 0, 0, time.UTC)
	key := employmentID.String() + ":" + date.Format("2006-01-02")

	shifts, source, _ := resolveEffectiveShift(employmentID, employeeID, "E-1", nil, date, int(date.Weekday()), nil, nil, map[string]*uuid.UUID{key: nil}, nil, nil, nil, nil)
	if source != "roster" || len(shifts) != 1 || shifts[0] != nil {
		t.Fatalf("expected explicit off roster, got source=%q shifts=%v", source, shifts)
	}

	patterns := []patternEntry{{patternID: uuid.New(), startDate: date, scheduleDay: int(date.Weekday()), templateID: nil}}
	shifts, source, _ = resolveEffectiveShift(employmentID, employeeID, "E-1", nil, date, int(date.Weekday()), nil, nil, nil, patterns, nil, nil, nil)
	if source != "pattern" || len(shifts) != 1 || shifts[0] != nil {
		t.Fatalf("expected explicit off pattern, got source=%q shifts=%v", source, shifts)
	}

	shifts, source, _ = resolveEffectiveShift(employmentID, employeeID, "E-1", nil, date, int(date.Weekday()), nil, nil, nil, nil, nil, nil, nil)
	if source != "none" || len(shifts) != 1 || shifts[0] != nil {
		t.Fatalf("expected missing schedule to remain distinct, got source=%q shifts=%v", source, shifts)
	}
}

func TestSelectEffectiveShiftVersion(t *testing.T) {
	templateID, firstVersion, secondVersion := uuid.New(), uuid.New(), uuid.New()
	firstFrom := time.Date(2026, time.January, 1, 0, 0, 0, 0, time.UTC)
	firstTo := time.Date(2026, time.June, 30, 0, 0, 0, 0, time.UTC)
	secondFrom := time.Date(2026, time.July, 1, 0, 0, 0, 0, time.UTC)
	shifts := map[uuid.UUID]shiftTemplateInfo{templateID: {versions: []shiftVersionInfo{{id: firstVersion, effectiveFrom: firstFrom, effectiveTo: &firstTo}, {id: secondVersion, effectiveFrom: secondFrom}}}}

	version, selected, conflict := selectEffectiveShiftVersion(shifts, templateID, time.Date(2026, time.July, 1, 0, 0, 0, 0, time.UTC))
	if !selected || conflict || version.id != secondVersion {
		t.Fatalf("expected effective successor version, got selected=%t conflict=%t version=%v", selected, conflict, version.id)
	}
}
