package rules

import (
	"testing"

	"github.com/shopspring/decimal"
)

func TestEvaluateRules_PercentageCapAndFixed(t *testing.T) {
	assignments := []Assignment{
		{Code: "social", CalculationMethod: "percentage", DefaultConfig: []byte(`{"rate":"10","cap_amount":"50"}`)},
		{Code: "meal", CalculationMethod: "fixed", DefaultConfig: []byte(`{"fixed_amount":"25"}`)},
	}
	results, err := EvaluateRules(assignments, map[string]decimal.Decimal{"social": decimal.NewFromInt(1000), "meal": decimal.NewFromInt(1)})
	if err != nil || len(results) != 2 {
		t.Fatalf("unexpected results: %+v, %v", results, err)
	}
	if results[0].Amount.String() != "50" || results[1].Amount.String() != "25" {
		t.Fatalf("unexpected amounts: %+v", results)
	}
}

func TestEvaluateRules_TieredAndSuppressZero(t *testing.T) {
	max := decimal.NewFromInt(1000)
	assignments := []Assignment{
		{Code: "tax", CalculationMethod: "tiered", Tiers: []Tier{{MinAmount: decimal.Zero, MaxAmount: &max, Rate: decimal.NewFromInt(10), TierOrder: 1}}},
		{Code: "none", CalculationMethod: "fixed", DefaultConfig: []byte(`{"fixed_amount":"0"}`)},
	}
	results, err := EvaluateRules(assignments, map[string]decimal.Decimal{"tax": decimal.NewFromInt(500), "none": decimal.NewFromInt(1)})
	if err != nil || len(results) != 1 || results[0].Amount.String() != "50" {
		t.Fatalf("unexpected results: %+v, %v", results, err)
	}
}
