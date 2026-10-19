package buckets

import (
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
}
