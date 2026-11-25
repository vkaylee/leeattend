package workpolicy

import (
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

func validMonthlyFixedRules() VersionRules {
	denom := MonthlyDenominatorFixed
	days := decimal.NewFromInt(22)
	return VersionRules{
		EffectiveFrom:           time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
		MinutesPerCredit:        480,
		WorkUnitMinutes:         15,
		RoundingPolicy:          "floor",
		EarlyArrivalStrategy:    "clamp",
		LateArrivalGraceMinutes: 5,
		BreakAutoDeduct:         true,
		OTAggregationMode:       "aggregate",
		PayMethod:               PayMethodMonthly,
		MonthlyDenominator:      &denom,
		MonthlyStandardDays:     &days,
		PaidLeaveCreditMode:     PaidLeaveCreditStandardDay,
		OvertimePolicyVersionID: uuid.New(),
	}
}

func TestVersionRulesValidation(t *testing.T) {
	t.Run("valid monthly fixed has no blockers", func(t *testing.T) {
		r := validMonthlyFixedRules()
		blockers, warnings := r.Validate()
		if len(blockers) != 0 {
			t.Fatalf("expected 0 blockers, got %+v", blockers)
		}
		if len(warnings) != 0 {
			t.Fatalf("expected 0 warnings, got %+v", warnings)
		}
	})

	t.Run("early OT with block arrival produces blocker", func(t *testing.T) {
		r := validMonthlyFixedRules()
		r.EarlyOTEnabled = true
		r.EarlyArrivalStrategy = "block"
		blockers, _ := r.Validate()
		if len(blockers) == 0 || blockers[0].Code != "early_ot_conflicts_with_block" {
			t.Fatalf("expected early_ot_conflicts_with_block, got %+v", blockers)
		}
	})

	t.Run("hourly requires rate component and rejects monthly fields", func(t *testing.T) {
		r := validMonthlyFixedRules()
		r.PayMethod = PayMethodHourly
		blockers, _ := r.Validate()
		if len(blockers) < 2 {
			t.Fatalf("expected >=2 blockers, got %+v", blockers)
		}
		compID := uuid.New()
		r.MonthlyDenominator = nil
		r.MonthlyStandardDays = nil
		r.RateComponentDefinitionID = &compID
		blockers, _ = r.Validate()
		if len(blockers) != 0 {
			t.Fatalf("expected 0 blockers for valid hourly, got %+v", blockers)
		}
	})

	t.Run("calculate credits divides eligible minutes by minutes per credit", func(t *testing.T) {
		r := validMonthlyFixedRules()
		res, err := r.CalculateCredits(240)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if !res.Credits.Equal(decimal.RequireFromString("0.5")) {
			t.Fatalf("expected 0.5 credits, got %s", res.Credits)
		}
	})
}

func TestOvertimePolicyVersionSpecValidation(t *testing.T) {
	start, end := "22:00", "06:00"
	daily := 240
	monthly := decimal.NewFromInt(40)
	spec := OvertimePolicyVersionSpec{
		DailyCapMinutes: &daily,
		MonthlyCapHours: &monthly,
		Rules: []OvertimeRule{
			{Priority: 100, DayType: OvertimeDayWeekday, DaysOfWeek: 127, TimeStart: &start, TimeEnd: &end, Multiplier: decimal.RequireFromString("2.0")},
		},
	}
	if err := spec.Validate(); err != nil {
		t.Fatalf("expected valid spec, got %v", err)
	}
	bad := spec
	bad.Rules = []OvertimeRule{{Priority: 10, DayType: "bogus", DaysOfWeek: 127, Multiplier: decimal.NewFromInt(1)}}
	if err := bad.Validate(); err == nil {
		t.Fatal("expected invalid day_type error")
	}
	halfWindow := spec
	halfWindow.Rules = []OvertimeRule{{Priority: 10, DayType: OvertimeDayAny, DaysOfWeek: 127, TimeStart: &start, Multiplier: decimal.NewFromInt(2)}}
	if err := halfWindow.Validate(); err == nil {
		t.Fatal("expected half time window error")
	}
}

func TestResolvedPolicySegmentBounds(t *testing.T) {
	mFrom := time.Date(2026, 3, 1, 0, 0, 0, 0, time.UTC)
	mTo := time.Date(2026, 3, 16, 0, 0, 0, 0, time.UTC)
	vFrom := time.Date(2026, 3, 10, 0, 0, 0, 0, time.UTC)
	p := ResolvedPolicy{
		MembershipEffectiveFrom: mFrom,
		MembershipEffectiveTo:   &mTo,
		EffectiveFrom:           vFrom,
	}
	s, e, ok := p.SegmentBounds(
		time.Date(2026, 3, 1, 0, 0, 0, 0, time.UTC),
		time.Date(2026, 4, 1, 0, 0, 0, 0, time.UTC),
	)
	if !ok {
		t.Fatal("expected overlap")
	}
	if !s.Equal(vFrom) || !e.Equal(mTo) {
		t.Fatalf("expected [%s, %s), got [%s, %s)", vFrom.Format("2006-01-02"), mTo.Format("2006-01-02"), s.Format("2006-01-02"), e.Format("2006-01-02"))
	}
}
