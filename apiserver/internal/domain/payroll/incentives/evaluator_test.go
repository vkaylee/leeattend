package incentives

import (
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

func TestEvaluatePolicy_SelectsPriorityTier(t *testing.T) {
	policy := Policy{Name: "attendance", BaseAmount: decimal.NewFromInt(1000), PayoutType: "percentage"}
	tiers := []Tier{
		{ID: uuid.New(), Priority: 2, RuleConfig: []byte(`{"max_late_early_count":10}`), PayoutAmount: decimal.RequireFromString("0.20")},
		{ID: uuid.New(), Priority: 1, RuleConfig: []byte(`{"max_late_early_count":2}`), PayoutAmount: decimal.RequireFromString("0.10")},
	}
	result, err := EvaluatePolicy(policy, tiers, Context{LateEarlyCount: 1})
	if err != nil || result.Amount.String() != "100" || result.TierID == nil {
		t.Fatalf("unexpected evaluation: %+v, %v", result, err)
	}
}

func TestMatchesTier_DynamicDecimalOperators(t *testing.T) {
	tier := Tier{RuleConfig: []byte(`[ {"metric":"kpi_score","operator":">=","value":"99.99"} ]`)}
	matched, err := MatchesTier(tier, Context{KPIScore: decimal.RequireFromString("100.00")})
	if err != nil || !matched {
		t.Fatalf("expected match: %v, %v", matched, err)
	}
	matched, err = MatchesTier(tier, Context{KPIScore: decimal.RequireFromString("99.98")})
	if err != nil || matched {
		t.Fatalf("expected non-match: %v, %v", matched, err)
	}
}
