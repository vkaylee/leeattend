package payroll

import (
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func standardOTRules() []workpolicy.OvertimeRule {
	nightStart, nightEnd := "22:00", "06:00"
	return []workpolicy.OvertimeRule{
		{Priority: 100, DayType: workpolicy.OvertimeDayHoliday, DaysOfWeek: 127, TimeStart: &nightStart, TimeEnd: &nightEnd, Multiplier: decimal.RequireFromString("3.90")},
		{Priority: 90, DayType: workpolicy.OvertimeDayHoliday, DaysOfWeek: 127, Multiplier: decimal.RequireFromString("3.00")},
		{Priority: 80, DayType: workpolicy.OvertimeDayWeekend, DaysOfWeek: 127, TimeStart: &nightStart, TimeEnd: &nightEnd, Multiplier: decimal.RequireFromString("2.70")},
		{Priority: 70, DayType: workpolicy.OvertimeDayWeekend, DaysOfWeek: 127, Multiplier: decimal.RequireFromString("2.00")},
		{Priority: 60, DayType: workpolicy.OvertimeDayWeekday, DaysOfWeek: 127, TimeStart: &nightStart, TimeEnd: &nightEnd, Multiplier: decimal.RequireFromString("2.00")},
		{Priority: 50, DayType: workpolicy.OvertimeDayWeekday, DaysOfWeek: 127, Multiplier: decimal.RequireFromString("1.50")},
	}
}

func makeMonthlyFixedPolicy(stdDays int64, dailyCap *int, monthlyCap, yearlyCap *decimal.Decimal) workpolicy.ResolvedPolicy {
	denom := workpolicy.MonthlyDenominatorFixed
	days := decimal.NewFromInt(stdDays)
	effFrom := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	return workpolicy.ResolvedPolicy{
		MembershipID: uuid.New(), EmploymentRelationshipID: uuid.New(),
		GroupID: uuid.New(), GroupCode: "STD", GroupName: "Standard",
		VersionID: uuid.New(), VersionNo: 1, Status: workpolicy.StatusPublished,
		EffectiveFrom: effFrom, MembershipEffectiveFrom: effFrom,
		MinutesPerCredit: 480, WorkUnitMinutes: 1, RoundingPolicy: "none", EarlyArrivalStrategy: "clamp",
		PayMethod: workpolicy.PayMethodMonthly, MonthlyDenominator: &denom, MonthlyStandardDays: &days,
		PaidLeaveCreditMode: workpolicy.PaidLeaveCreditStandardDay,
		DailyCapMinutes:     dailyCap, MonthlyCapHours: monthlyCap, YearlyCapHours: yearlyCap,
		OvertimeRules: standardOTRules(),
	}
}

func newTestPeriodInput() PeriodInput {
	return PeriodInput{
		EmploymentID: uuid.New(), RelationshipID: uuid.New(), EmployeeID: uuid.New(), EmployerID: uuid.New(),
		Year: 2026, Month: 3,
		SalaryHistory: []SalaryComponentEntry{{
			ComponentID: uuid.New(), DefinitionID: uuid.New(), Code: "BASE", Name: "Base Salary",
			Amount: decimal.NewFromInt(17600000), EffectiveAt: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
		}},
		DailySummaries: map[string]DailyAttendanceSummary{}, PaidLeaveDays: map[string]PaidLeaveDay{},
		CompletedShiftCounts: map[string]int{}, ScheduledWorkdayCounts: map[string]int{},
	}
}

func TestCalculatePayslip_DayTypesAndOvernightWindow(t *testing.T) {
	in := newTestPeriodInput()
	in.DailySummaries["2026-03-02"] = DailyAttendanceSummary{
		Date: "2026-03-02", NetPaidMinutes: 480, StandardMinutes: 480, ApprovedOTMinutes: 120,
		OTTimeSlices: []OTTimeSlice{{StartMinute: 21 * 60, EndMinute: 23 * 60}},
	}
	in.DailySummaries["2026-03-07"] = DailyAttendanceSummary{Date: "2026-03-07", IsWeekend: true, ApprovedOTMinutes: 120}
	in.DailySummaries["2026-03-10"] = DailyAttendanceSummary{Date: "2026-03-10", IsHoliday: true, ApprovedOTMinutes: 60}

	res, err := CalculatePayslip(in, []workpolicy.ResolvedPolicy{makeMonthlyFixedPolicy(22, nil, nil, nil)})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !res.Payslip.TotalBasePay.Equal(decimal.NewFromInt(800000)) {
		t.Fatalf("expected base pay 800000, got %s", res.Payslip.TotalBasePay)
	}
	if !res.Payslip.TotalOvertimePay.Equal(decimal.NewFromInt(1050000)) {
		t.Fatalf("expected OT pay 1050000, got %s", res.Payslip.TotalOvertimePay)
	}
}

func TestCalculatePayslip_DailyMonthlyYearlyOTCaps(t *testing.T) {
	in := newTestPeriodInput()
	in.PriorYTDApprovedOTMinutes = 300
	in.DailySummaries["2026-03-02"] = DailyAttendanceSummary{Date: "2026-03-02", NetPaidMinutes: 480, ApprovedOTMinutes: 300}
	in.DailySummaries["2026-03-03"] = DailyAttendanceSummary{Date: "2026-03-03", NetPaidMinutes: 480, ApprovedOTMinutes: 300}
	dailyCap, monthlyCap, yearlyCap := 240, decimal.NewFromInt(6), decimal.NewFromInt(10)

	res, err := CalculatePayslip(in, []workpolicy.ResolvedPolicy{makeMonthlyFixedPolicy(22, &dailyCap, &monthlyCap, &yearlyCap)})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if res.PaidOTMinutes != 300 || res.CappedOTMinutes != 300 || !res.Payslip.TotalOvertimePay.Equal(decimal.NewFromInt(750000)) {
		t.Fatalf("unexpected capped OT result paid=%d capped=%d pay=%s", res.PaidOTMinutes, res.CappedOTMinutes, res.Payslip.TotalOvertimePay)
	}
}

func TestCalculatePayslip_MidPeriodSwitchAndFingerprint(t *testing.T) {
	in := newTestPeriodInput()
	in.DailySummaries["2026-03-02"] = DailyAttendanceSummary{Date: "2026-03-02", NetPaidMinutes: 480}
	in.DailySummaries["2026-03-20"] = DailyAttendanceSummary{Date: "2026-03-20", NetPaidMinutes: 480}

	p1 := makeMonthlyFixedPolicy(22, nil, nil, nil)
	split := time.Date(2026, 3, 16, 0, 0, 0, 0, time.UTC)
	p1.EffectiveTo = &split
	p2 := makeMonthlyFixedPolicy(20, nil, nil, nil)
	p2.GroupID, p2.VersionNo, p2.EffectiveFrom = p1.GroupID, 2, split

	resBaseline, err := CalculatePayslip(in, []workpolicy.ResolvedPolicy{makeMonthlyFixedPolicy(22, nil, nil, nil)})
	if err != nil {
		t.Fatalf("baseline error: %v", err)
	}
	resSwitched, err := CalculatePayslip(in, []workpolicy.ResolvedPolicy{p1, p2})
	if err != nil {
		t.Fatalf("switched error: %v", err)
	}
	if len(resSwitched.Segments) != 2 || resSwitched.Segments[0].WorkPolicyVersionID != p1.VersionID || resSwitched.Segments[1].WorkPolicyVersionID != p2.VersionID {
		t.Fatalf("segment policy provenance mismatch")
	}
	if !resSwitched.Payslip.TotalBasePay.Equal(decimal.NewFromInt(1680000)) || resBaseline.InputFingerprint != resSwitched.InputFingerprint {
		t.Fatalf("unexpected switched base=%s or fingerprint mismatch", resSwitched.Payslip.TotalBasePay)
	}
}

func TestCalculatePayslip_PaidLeaveCreditModes(t *testing.T) {
	in := newTestPeriodInput()
	in.PaidLeaveDays["2026-03-04"] = PaidLeaveDay{Date: "2026-03-04", Fraction: decimal.NewFromInt(1), ScheduledMinutes: 240}
	cases := []struct {
		mode workpolicy.PaidLeaveCreditMode
		want int64
	}{
		{workpolicy.PaidLeaveCreditStandardDay, 800000},
		{workpolicy.PaidLeaveCreditScheduledMinutes, 400000},
		{workpolicy.PaidLeaveCreditUnpaid, 0},
	}
	for _, tc := range cases {
		p := makeMonthlyFixedPolicy(22, nil, nil, nil)
		p.PaidLeaveCreditMode = tc.mode
		res, err := CalculatePayslip(in, []workpolicy.ResolvedPolicy{p})
		if err != nil || !res.Payslip.TotalBasePay.Equal(decimal.NewFromInt(tc.want)) {
			t.Fatalf("mode %s: got %s err=%v, want %d", tc.mode, res.Payslip.TotalBasePay, err, tc.want)
		}
	}
}

func TestCalculatePayslip_HourlyAndShiftPay(t *testing.T) {
	rateDefID := uuid.New()
	in := newTestPeriodInput()
	in.SalaryHistory = []SalaryComponentEntry{{
		ComponentID: uuid.New(), DefinitionID: rateDefID, Code: "RATE", Name: "Rate",
		Amount: decimal.NewFromInt(100000), EffectiveAt: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
	}}
	in.DailySummaries["2026-03-02"] = DailyAttendanceSummary{Date: "2026-03-02", NetPaidMinutes: 240, ApprovedOTMinutes: 60}
	in.CompletedShiftCounts["2026-03-02"] = 2

	pHourly := makeMonthlyFixedPolicy(22, nil, nil, nil)
	pHourly.PayMethod, pHourly.MonthlyDenominator, pHourly.MonthlyStandardDays, pHourly.RateComponentDefinitionID = workpolicy.PayMethodHourly, nil, nil, &rateDefID
	resH, err := CalculatePayslip(in, []workpolicy.ResolvedPolicy{pHourly})
	if err != nil || !resH.Payslip.TotalBasePay.Equal(decimal.NewFromInt(400000)) || !resH.Payslip.TotalOvertimePay.Equal(decimal.NewFromInt(150000)) {
		t.Fatalf("unexpected hourly pay base=%s ot=%s err=%v", resH.Payslip.TotalBasePay, resH.Payslip.TotalOvertimePay, err)
	}

	pShift := pHourly
	pShift.PayMethod = workpolicy.PayMethodShift
	if _, err := CalculatePayslip(in, []workpolicy.ResolvedPolicy{pShift}); err == nil {
		t.Fatal("expected error for shift pay with OT")
	}
	in.DailySummaries["2026-03-02"] = DailyAttendanceSummary{Date: "2026-03-02", NetPaidMinutes: 240}
	resS, err := CalculatePayslip(in, []workpolicy.ResolvedPolicy{pShift})
	if err != nil || !resS.Payslip.TotalBasePay.Equal(decimal.NewFromInt(200000)) {
		t.Fatalf("expected shift pay 200000, got %s err=%v", resS.Payslip.TotalBasePay, err)
	}
}
