package workpolicy

import (
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

func TestWorkCreditPolicyValidateRejectsNonPositiveUnit(t *testing.T) {
	policy := WorkCreditPolicy{Name: "Custom", MinutesPerCredit: 0, AttendancePolicyID: ids.AttendancePolicyID(uuid.New())}
	if err := policy.Validate(); err != ErrMinutesPerCreditRequired {
		t.Fatalf("expected non-positive unit error, got %v", err)
	}
}

func TestWorkCreditPolicyValidateRequiresScheduleReference(t *testing.T) {
	policy := WorkCreditPolicy{Name: "Custom", MinutesPerCredit: 450, AttendancePolicyID: ids.AttendancePolicyID(uuid.New())}
	if err := policy.Validate(); err != ErrReferenceRequired {
		t.Fatalf("expected missing working pattern error, got %v", err)
	}
}

func TestWorkCreditPolicyCalculateUsesConfiguredMinutes(t *testing.T) {
	policy := WorkCreditPolicy{Name: "Custom", MinutesPerCredit: 450, AttendancePolicyID: ids.AttendancePolicyID(uuid.New()), WorkingPatternID: ids.WorkingPatternID(uuid.New())}
	result, err := policy.Calculate(675)
	if err != nil {
		t.Fatalf("calculate credits: %v", err)
	}
	if result.Credits.String() != "1.5" {
		t.Fatalf("expected 1.5 credits, got %s", result.Credits)
	}
	if _, err := policy.Calculate(-1); err != ErrEligibleMinutesInvalid {
		t.Fatalf("expected negative eligible minutes error, got %v", err)
	}
}

func TestPayBasisPolicyValidateUsesMonthlyOptionsOnlyForMonthlyPay(t *testing.T) {
	denominator := MonthlyDenominatorFixed
	days := decimal.NewFromInt(22)
	monthly := PayBasisPolicy{Name: "Monthly", Method: PayMethodMonthly, MonthlyDenominator: &denominator, MonthlyStandardDays: &days}
	if err := monthly.Validate(); err != nil {
		t.Fatalf("valid monthly policy rejected: %v", err)
	}

	hourly := PayBasisPolicy{Name: "Hourly", Method: PayMethodHourly, MonthlyDenominator: &denominator}
	if err := hourly.Validate(); err != ErrRateComponentRequired {
		t.Fatalf("expected rate component requirement for hourly pay, got %v", err)
	}
	rateComponent := uuid.New()
	hourly = PayBasisPolicy{Name: "Hourly", Method: PayMethodHourly, RateComponentDefinitionID: func() *ids.RateComponentDefinitionID {
		converted := ids.RateComponentDefinitionID(rateComponent)
		return &converted
	}()}
	if err := hourly.Validate(); err != nil {
		t.Fatalf("valid hourly policy rejected: %v", err)
	}
}

func TestLaborPolicyGroupValidateRejectsInvalidEffectiveRange(t *testing.T) {
	from := time.Date(2026, time.January, 1, 0, 0, 0, 0, time.UTC)
	to := from
	group := LaborPolicyGroup{
		Name:               "Operations",
		WorkCreditPolicyID: WorkCreditPolicyID(uuid.New()),
		PayBasisPolicyID:   PayBasisPolicyID(uuid.New()),
		OvertimePolicyID:   ids.OvertimePolicyID(uuid.New()),
		EffectiveFrom:      from,
		EffectiveTo:        &to,
	}
	if err := group.Validate(); err != ErrEffectiveRangeInvalid {
		t.Fatalf("expected invalid effective range, got %v", err)
	}
}

func TestIntervalsOverlapTreatsOpenEndedRangesAsOverlapping(t *testing.T) {
	from := time.Date(2026, time.January, 1, 0, 0, 0, 0, time.UTC)
	later := from.AddDate(0, 1, 0)
	if !IntervalsOverlap(from, nil, later, nil) {
		t.Fatal("expected open-ended ranges to overlap")
	}
	if IntervalsOverlap(from, &later, later, nil) {
		t.Fatal("adjacent half-open ranges must not overlap")
	}
}
