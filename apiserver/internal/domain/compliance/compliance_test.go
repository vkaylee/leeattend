package compliance

import (
	"errors"
	"testing"

	"github.com/google/uuid"
)

func defaultRules() Rules {
	return Rules{
		IsEnabled:            true,
		MinRestPeriodHours:   12,
		MaxWeeklyHours:       48,
		MaxMonthlyOTHours:    40,
		AllowManagerOverride: true,
	}
}

func TestValidate_Disabled(t *testing.T) {
	rules := defaultRules()
	rules.IsEnabled = false
	shiftID := uuid.New()
	res, err := Validate(rules, Request{
		EmploymentID: uuid.New(),
		ShiftID:      shiftID,
		Dates:        []string{"2026-10-19"},
	}, nil, nil)
	if err != nil || !res.Compliant {
		t.Fatalf("expected compliant when disabled, got %+v, err: %v", res, err)
	}
}

func TestValidate_HardOverlap(t *testing.T) {
	rules := defaultRules()
	empID := uuid.New()
	shiftA := uuid.New()
	shiftB := uuid.New()

	shifts := map[uuid.UUID][]Interval{
		shiftA: {{StartTime: "08:00:00", EndTime: "17:00:00", BreakDurationMinutes: 60, IsBreakAutoDeduct: true}},
		shiftB: {{StartTime: "12:00:00", EndTime: "20:00:00", BreakDurationMinutes: 60, IsBreakAutoDeduct: true}},
	}
	existing := []Roster{
		{EmploymentID: empID, ShiftID: &shiftA, Date: "2026-10-19"},
	}

	res, err := Validate(rules, Request{
		EmploymentID: empID,
		ShiftID:      shiftB,
		Dates:        []string{"2026-10-19"},
	}, existing, shifts)

	if !errors.Is(err, ErrHardViolation) {
		t.Fatalf("expected ErrHardViolation, got %v", err)
	}
	if res.Compliant {
		t.Fatal("expected not compliant")
	}
	if len(res.Violations) != 1 || res.Violations[0].Type != ViolationOverlap {
		t.Fatalf("unexpected violations: %+v", res.Violations)
	}
}

func TestValidate_SoftRestPeriod_WithoutOverride(t *testing.T) {
	rules := defaultRules()
	empID := uuid.New()
	shiftA := uuid.New()
	shiftB := uuid.New()

	shifts := map[uuid.UUID][]Interval{
		shiftA: {{StartTime: "14:00:00", EndTime: "23:00:00", BreakDurationMinutes: 60, IsBreakAutoDeduct: true}},
		shiftB: {{StartTime: "06:00:00", EndTime: "14:00:00", BreakDurationMinutes: 60, IsBreakAutoDeduct: true}},
	}
	existing := []Roster{
		{EmploymentID: empID, ShiftID: &shiftA, Date: "2026-10-19"},
	}

	res, err := Validate(rules, Request{
		EmploymentID: empID,
		ShiftID:      shiftB,
		Dates:        []string{"2026-10-20"},
	}, existing, shifts)

	if !errors.Is(err, ErrSoftViolation) {
		t.Fatalf("expected ErrSoftViolation, got %v", err)
	}
	if !res.RequiresOverride {
		t.Fatal("expected requires override")
	}
}

func TestValidate_SoftRestPeriod_WithOverride(t *testing.T) {
	rules := defaultRules()
	empID := uuid.New()
	shiftA := uuid.New()
	shiftB := uuid.New()

	shifts := map[uuid.UUID][]Interval{
		shiftA: {{StartTime: "14:00:00", EndTime: "23:00:00", BreakDurationMinutes: 60, IsBreakAutoDeduct: true}},
		shiftB: {{StartTime: "06:00:00", EndTime: "14:00:00", BreakDurationMinutes: 60, IsBreakAutoDeduct: true}},
	}
	existing := []Roster{
		{EmploymentID: empID, ShiftID: &shiftA, Date: "2026-10-19"},
	}
	reason := "Urgent operational need approved by director"
	res, err := Validate(rules, Request{
		EmploymentID:   empID,
		ShiftID:        shiftB,
		Dates:          []string{"2026-10-20"},
		AllowWarnings:  true,
		OverrideReason: &reason,
	}, existing, shifts)

	if err != nil {
		t.Fatalf("expected nil error with override, got %v", err)
	}
	if !res.Compliant {
		t.Fatal("expected compliant with override")
	}
	if len(res.Violations) != 1 {
		t.Fatalf("expected soft violation captured, got %+v", res.Violations)
	}
}

func TestValidate_WeeklyHoursExceeded(t *testing.T) {
	rules := defaultRules()
	rules.MaxWeeklyHours = 40
	empID := uuid.New()
	shiftA := uuid.New()

	shifts := map[uuid.UUID][]Interval{
		shiftA: {{StartTime: "08:00:00", EndTime: "18:00:00", BreakDurationMinutes: 0, IsBreakAutoDeduct: false}},
	}
	// 5 days x 10h = 50h > 40h
	dates := []string{"2026-10-19", "2026-10-20", "2026-10-21", "2026-10-22", "2026-10-23"}

	res, err := Validate(rules, Request{
		EmploymentID: empID,
		ShiftID:      shiftA,
		Dates:        dates,
	}, nil, shifts)

	if !errors.Is(err, ErrSoftViolation) {
		t.Fatalf("expected ErrSoftViolation for weekly hours, got %v", err)
	}
	foundWeekly := false
	for _, v := range res.Violations {
		if v.Type == ViolationWeeklyHours {
			foundWeekly = true
		}
	}
	if !foundWeekly {
		t.Fatalf("expected WeeklyHours violation in %+v", res.Violations)
	}
}

func TestValidate_MonthlyOTExceeded(t *testing.T) {
	rules := defaultRules()
	rules.MaxMonthlyOTHours = 10
	empID := uuid.New()
	shiftA := uuid.New()

	shifts := map[uuid.UUID][]Interval{
		shiftA: {{StartTime: "08:00:00", EndTime: "20:00:00", BreakDurationMinutes: 0, IsBreakAutoDeduct: false}},
	}
	// 12 hours/day -> 4 hours OT/day. 3 days -> 12h OT > 10h limit
	dates := []string{"2026-10-19", "2026-10-20", "2026-10-21"}

	res, err := Validate(rules, Request{
		EmploymentID: empID,
		ShiftID:      shiftA,
		Dates:        dates,
	}, nil, shifts)

	if !errors.Is(err, ErrSoftViolation) {
		t.Fatalf("expected ErrSoftViolation for monthly OT, got %v", err)
	}
	foundOT := false
	for _, v := range res.Violations {
		if v.Type == ViolationMonthlyOvertime {
			foundOT = true
		}
	}
	if !foundOT {
		t.Fatalf("expected MonthlyOvertime violation in %+v", res.Violations)
	}
}

func TestValidate_OvernightInterval(t *testing.T) {
	rules := defaultRules()
	empID := uuid.New()
	shiftNight := uuid.New()
	shiftDay := uuid.New()

	shifts := map[uuid.UUID][]Interval{
		shiftNight: {{StartTime: "22:00:00", EndTime: "06:00:00", BreakDurationMinutes: 0, IsBreakAutoDeduct: false}},
		shiftDay:   {{StartTime: "04:00:00", EndTime: "12:00:00", BreakDurationMinutes: 0, IsBreakAutoDeduct: false}},
	}
	existing := []Roster{
		{EmploymentID: empID, ShiftID: &shiftNight, Date: "2026-10-19"},
	}

	res, err := Validate(rules, Request{
		EmploymentID: empID,
		ShiftID:      shiftDay,
		Dates:        []string{"2026-10-20"},
	}, existing, shifts)

	if !errors.Is(err, ErrHardViolation) {
		t.Fatalf("expected overlap with overnight shift, got %v, res: %+v", err, res)
	}
}

func TestAdjacentNightShiftOverlap(t *testing.T) {
	empA := uuid.New()
	shiftNight, shiftMorning := uuid.New(), uuid.New()
	shifts := map[uuid.UUID][]Interval{
		shiftNight:   {{StartTime: "22:00:00", EndTime: "06:00:00"}},
		shiftMorning: {{StartTime: "05:00:00", EndTime: "13:00:00"}},
	}
	existing := []Roster{{EmploymentID: empA, ShiftID: &shiftNight, Date: "2026-11-20"}}
	res, err := Validate(defaultRules(), Request{
		EmploymentID: empA,
		ShiftID:      shiftMorning,
		Dates:        []string{"2026-11-21"},
	}, existing, shifts)
	if !errors.Is(err, ErrHardViolation) {
		t.Fatalf("expected ErrHardViolation for adjacent overnight overlap, got err=%v res=%+v", err, res)
	}
	if len(res.Violations) == 0 || res.Violations[0].Type != ViolationOverlap {
		t.Fatalf("expected ViolationOverlap, got %+v", res.Violations)
	}
}

func TestAdjacentNightShiftTouchingAndOverridePolicy(t *testing.T) {
	empA, empB := uuid.New(), uuid.New()
	shiftNight, shiftTouching := uuid.New(), uuid.New()
	shifts := map[uuid.UUID][]Interval{
		shiftNight:    {{StartTime: "22:00:00", EndTime: "06:00:00"}},
		shiftTouching: {{StartTime: "06:00:00", EndTime: "14:00:00"}},
	}

	existing := []Roster{{EmploymentID: empA, ShiftID: &shiftNight, Date: "2026-11-20"}}
	res, err := Validate(defaultRules(), Request{
		EmploymentID: empA,
		ShiftID:      shiftTouching,
		Dates:        []string{"2026-11-21"},
	}, existing, shifts)
	if !errors.Is(err, ErrSoftViolation) || len(res.Violations) != 1 || res.Violations[0].Type != ViolationRestPeriod {
		t.Fatalf("expected rest-only soft violation on touching 06:00 endpoints, got err=%v res=%+v", err, res)
	}

	noOverrideRules := defaultRules()
	noOverrideRules.AllowManagerOverride = false
	reason := "Manager override attempt"
	res, err = Validate(noOverrideRules, Request{
		EmploymentID:   empA,
		ShiftID:        shiftTouching,
		Dates:          []string{"2026-11-21"},
		AllowWarnings:  true,
		OverrideReason: &reason,
	}, existing, shifts)
	if !errors.Is(err, ErrHardViolation) || res.Compliant || res.RequiresOverride {
		t.Fatalf("expected hard block when AllowManagerOverride=false, got err=%v res=%+v", err, res)
	}

	crossExisting := []Roster{{EmploymentID: empB, ShiftID: &shiftTouching, Date: "2026-11-20"}}
	res, err = Validate(defaultRules(), Request{
		EmploymentID: empA,
		ShiftID:      shiftTouching,
		Dates:        []string{"2026-11-20"},
	}, crossExisting, shifts)
	if !errors.Is(err, ErrHardViolation) {
		t.Fatalf("expected cross-employment overlap, got err=%v res=%+v", err, res)
	}
}
