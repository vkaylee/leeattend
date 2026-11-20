package buckets

import (
	"errors"
	"testing"
	"time"
)

func TestCalculateUnits_IntegerRounding(t *testing.T) {
	if got := CalculateUnits(61, 60, RoundingPolicyFloor); got != 1 {
		t.Fatalf("floor: got %d", got)
	}
	if got := CalculateUnits(61, 60, RoundingPolicyCeil); got != 2 {
		t.Fatalf("ceil: got %d", got)
	}
	if got := CalculateUnits(91, 60, RoundingPolicyRound); got != 2 {
		t.Fatalf("round: got %d", got)
	}
}

func TestCalculateBuckets_StandardAndOvertime(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	punchIn := date.Add(8 * time.Hour)
	punchOut := date.Add(18 * time.Hour)
	shiftStart := date.Add(9 * time.Hour)
	shiftEnd := date.Add(17 * time.Hour)
	input := DailyCalcInput{PunchIn: punchIn, PunchOut: punchOut, ShiftStart: &shiftStart, ShiftEnd: &shiftEnd, WorkUnit: 60, AllowEarlyOT: true}
	buckets := CalculateBuckets(input)
	if buckets.StandardMinutes != 8*60 || buckets.EarlyOTMinutes != 60 || buckets.LateOTMinutes != 60 {
		t.Fatalf("unexpected buckets: %+v", buckets)
	}
}

func TestCalculateBuckets_AppliesPolicyGraceAndCap(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	punchIn := date.Add(7 * time.Hour)
	punchOut := date.Add(17 * time.Hour)
	shiftStart := date.Add(8 * time.Hour)
	shiftEnd := date.Add(17 * time.Hour)
	max := int32(20)
	buckets := CalculateBuckets(DailyCalcInput{
		PunchIn: punchIn, PunchOut: punchOut, ShiftStart: &shiftStart, ShiftEnd: &shiftEnd,
		AllowEarlyOT: true, GracePeriodMinutes: 15, MaxEarlyOTMinutes: &max,
	})
	if buckets.EarlyOTMinutes != 20 {
		t.Fatalf("expected 20 eligible early OT minutes after cap, got %d", buckets.EarlyOTMinutes)
	}
}

func TestCalculateBreakDeduction_DisabledPolicyDoesNotDeduct(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	breakStart := date.Add(12 * time.Hour)
	breakEnd := date.Add(13 * time.Hour)
	punchIn := date.Add(8 * time.Hour)
	punchOut := date.Add(17 * time.Hour)
	if got := CalculateBreakDeduction(BreakDeductionInput{IsBreakAutoDeduct: false, BreakStart: &breakStart, BreakEnd: &breakEnd, PunchIn: &punchIn, PunchOut: &punchOut}); got != 0 {
		t.Fatalf("expected no deduction, got %d", got)
	}
}

func TestCalculateBreakDeduction_IntersectsPunchWindow(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	breakStart := date.Add(12 * time.Hour)
	breakEnd := date.Add(13 * time.Hour)
	punchIn := date.Add(12*time.Hour + 30*time.Minute)
	punchOut := date.Add(12*time.Hour + 45*time.Minute)
	got := CalculateBreakDeduction(BreakDeductionInput{IsBreakAutoDeduct: true, BreakStart: &breakStart, BreakEnd: &breakEnd, PunchIn: &punchIn, PunchOut: &punchOut})
	if got != 15 {
		t.Fatalf("expected 15 minutes, got %d", got)
	}
	if got := CalculateBreakOvertime(BreakDeductionInput{IsBreakOvertime: true, BreakStart: &breakStart, BreakEnd: &breakEnd, PunchIn: &punchIn, PunchOut: &punchOut}); got != 15 {
		t.Fatalf("expected 15 break overtime minutes, got %d", got)
	}
}

func TestCalculateAttendance_ScheduledBreakUsesScheduledDeduction(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	result, err := CalculateAttendance(AttendanceCalculationInput{
		PunchIn: date.Add(8 * time.Hour), PunchOut: date.Add(17 * time.Hour),
		BreakPunchMode: BreakPunchModeNone, BreakCreditMode: BreakCreditModeScheduled,
		ScheduledBreakMinutes: 60,
	})
	if err != nil {
		t.Fatalf("calculate attendance: %v", err)
	}
	if result.GrossMinutes != 540 || result.ScheduledUnpaidBreakMinutes != 60 || result.ActualBreakMinutes != 0 || result.PayableMinutes != 480 {
		t.Fatalf("unexpected scheduled result: %+v", result)
	}
}

func TestCalculateAttendance_NoBreakCreditDoesNotDeduct(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	result, err := CalculateAttendance(AttendanceCalculationInput{
		PunchIn: date.Add(8 * time.Hour), PunchOut: date.Add(17 * time.Hour),
		BreakPunchMode: BreakPunchModeNone, BreakCreditMode: BreakCreditModeNone,
		ScheduledBreakMinutes: 60,
	})
	if err != nil {
		t.Fatalf("calculate attendance: %v", err)
	}
	if result.ScheduledUnpaidBreakMinutes != 0 || result.PayableMinutes != 540 {
		t.Fatalf("unexpected no-credit result: %+v", result)
	}
}

func TestCalculateAttendance_RequiredBreakUsesActualPairsOnly(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	result, err := CalculateAttendance(AttendanceCalculationInput{
		PunchIn: date.Add(8 * time.Hour), PunchOut: date.Add(17 * time.Hour),
		BreakPunchMode: BreakPunchModeRequired, BreakCreditMode: BreakCreditModeActual,
		ScheduledBreakMinutes: 60,
		ActualBreaks:          []BreakPair{{Start: date.Add(12 * time.Hour), End: date.Add(13 * time.Hour)}},
	})
	if err != nil {
		t.Fatalf("calculate attendance: %v", err)
	}
	if result.GrossMinutes != 540 || result.ScheduledUnpaidBreakMinutes != 0 || result.ActualBreakMinutes != 60 || result.PayableMinutes != 480 {
		t.Fatalf("unexpected required result: %+v", result)
	}
}

func TestCalculateAttendance_RequiredBreakMissingIsExceptionWithoutSilentDeduction(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	result, err := CalculateAttendance(AttendanceCalculationInput{
		PunchIn: date.Add(8 * time.Hour), PunchOut: date.Add(17 * time.Hour),
		BreakPunchMode: BreakPunchModeRequired, BreakCreditMode: BreakCreditModeActual,
		ScheduledBreakMinutes: 60,
	})
	if err != nil {
		t.Fatalf("calculate attendance: %v", err)
	}
	if result.ExceptionCode != AttendanceExceptionMissingBreakPair || result.ScheduledUnpaidBreakMinutes != 0 || result.PayableMinutes != 540 {
		t.Fatalf("unexpected missing-break result: %+v", result)
	}
}

func TestCalculateAttendance_UnexpectedBreakDoesNotDoubleDeduct(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	result, err := CalculateAttendance(AttendanceCalculationInput{
		PunchIn: date.Add(8 * time.Hour), PunchOut: date.Add(17 * time.Hour),
		BreakPunchMode: BreakPunchModeNone, BreakCreditMode: BreakCreditModeScheduled,
		ScheduledBreakMinutes: 60,
		ActualBreaks:          []BreakPair{{Start: date.Add(12 * time.Hour), End: date.Add(13 * time.Hour)}},
	})
	if err != nil {
		t.Fatalf("calculate attendance: %v", err)
	}
	if result.ExceptionCode != AttendanceExceptionUnexpectedBreakPunch || result.PayableMinutes != 480 || result.ActualBreakMinutes != 60 || result.ScheduledUnpaidBreakMinutes != 60 {
		t.Fatalf("unexpected duplicate-deduction result: %+v", result)
	}
}

func TestCalculateAttendance_PlannedBreakWorkedRemovesScheduledDeduction(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	result, err := CalculateAttendance(AttendanceCalculationInput{
		PunchIn: date.Add(8 * time.Hour), PunchOut: date.Add(17 * time.Hour),
		BreakPunchMode: BreakPunchModeNone, BreakCreditMode: BreakCreditModeScheduled,
		ScheduledBreakMinutes: 60, PlannedBreakWorkedMinutes: 60,
		ApprovedOvertimeMinutes: 60,
	})
	if err != nil {
		t.Fatalf("calculate attendance: %v", err)
	}
	if result.ScheduledUnpaidBreakMinutes != 0 || result.PaidBreakWorkMinutes != 60 || result.ApprovedOvertimeMinutes != 60 || result.PayableMinutes != 540 {
		t.Fatalf("unexpected break-worked result: %+v", result)
	}
}

func TestCalculateAttendance_OvernightBreakUsesAbsoluteOrdering(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	result, err := CalculateAttendance(AttendanceCalculationInput{
		PunchIn: date.Add(22 * time.Hour), PunchOut: date.Add(30 * time.Hour),
		BreakPunchMode: BreakPunchModeRequired, BreakCreditMode: BreakCreditModeActual,
		ActualBreaks: []BreakPair{{Start: date.Add(26 * time.Hour), End: date.Add(27 * time.Hour)}},
	})
	if err != nil {
		t.Fatalf("calculate overnight attendance: %v", err)
	}
	if result.GrossMinutes != 480 || result.ActualBreakMinutes != 60 || result.PayableMinutes != 420 {
		t.Fatalf("unexpected overnight result: %+v", result)
	}
}

func TestCalculateAttendance_RejectsInvalidBreakPair(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	_, err := CalculateAttendance(AttendanceCalculationInput{
		PunchIn: date.Add(8 * time.Hour), PunchOut: date.Add(17 * time.Hour),
		BreakPunchMode: BreakPunchModeRequired, BreakCreditMode: BreakCreditModeActual,
		ActualBreaks: []BreakPair{{Start: date.Add(13 * time.Hour), End: date.Add(12 * time.Hour)}},
	})
	if !errors.Is(err, ErrInvalidAttendanceCalculation) {
		t.Fatalf("expected invalid calculation error, got %v", err)
	}
}
