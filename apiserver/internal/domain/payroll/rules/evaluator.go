package rules

import (
	"encoding/json"
	"errors"
	"sort"

	"github.com/shopspring/decimal"
)

var ErrInvalidRuleConfig = errors.New("invalid rule config")

type Assignment struct {
	Code              string
	CalculationMethod string
	DefaultConfig     []byte
	OverrideConfig    []byte
	Tiers             []Tier
}
type Tier struct {
	MinAmount   decimal.Decimal
	MaxAmount   *decimal.Decimal
	Rate        decimal.Decimal
	FixedAmount decimal.Decimal
	TierOrder   int32
}
type EvaluationResult struct {
	RuleCode string
	Amount   decimal.Decimal
}
type config struct {
	Rate        decimal.Decimal `json:"rate"`
	FixedAmount decimal.Decimal `json:"fixed_amount"`
	CapAmount   decimal.Decimal `json:"cap_amount"`
}

func EvaluateRules(assignments []Assignment, baseValues map[string]decimal.Decimal) ([]EvaluationResult, error) {
	results := make([]EvaluationResult, 0, len(assignments))
	for _, assignment := range assignments {
		base, ok := baseValues[assignment.Code]
		if !ok {
			continue
		}
		amount, err := calculate(assignment, base)
		if err != nil {
			return nil, err
		}
		if amount.GreaterThan(decimal.Zero) {
			results = append(results, EvaluationResult{RuleCode: assignment.Code, Amount: amount})
		}
	}
	return results, nil
}

func calculate(assignment Assignment, base decimal.Decimal) (decimal.Decimal, error) {
	cfg := assignment.DefaultConfig
	if len(assignment.OverrideConfig) > 0 {
		cfg = assignment.OverrideConfig
	}
	var values config
	if len(cfg) > 0 {
		if err := json.Unmarshal(cfg, &values); err != nil {
			return decimal.Zero, ErrInvalidRuleConfig
		}
	}
	switch assignment.CalculationMethod {
	case "percentage":
		amount := base.Mul(values.Rate).Div(decimal.NewFromInt(100))
		if values.CapAmount.GreaterThan(decimal.Zero) && amount.GreaterThan(values.CapAmount) {
			amount = values.CapAmount
		}
		return amount, nil
	case "fixed":
		return values.FixedAmount, nil
	case "tiered":
		return calculateTiered(assignment.Tiers, base), nil
	default:
		return decimal.Zero, ErrInvalidRuleConfig
	}
}

func calculateTiered(tiers []Tier, base decimal.Decimal) decimal.Decimal {
	ordered := append([]Tier(nil), tiers...)
	sort.SliceStable(ordered, func(left, right int) bool { return ordered[left].TierOrder < ordered[right].TierOrder })
	amount := decimal.Zero
	for _, tier := range ordered {
		if !base.GreaterThan(tier.MinAmount) {
			continue
		}
		upper := base
		if tier.MaxAmount != nil && upper.GreaterThan(*tier.MaxAmount) {
			upper = *tier.MaxAmount
		}
		taxable := upper.Sub(tier.MinAmount)
		if taxable.IsNegative() {
			continue
		}
		amount = amount.Add(taxable.Mul(tier.Rate).Div(decimal.NewFromInt(100)).Add(tier.FixedAmount))
	}
	return amount
}
