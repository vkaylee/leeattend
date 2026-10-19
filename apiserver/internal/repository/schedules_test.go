package repository

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

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
