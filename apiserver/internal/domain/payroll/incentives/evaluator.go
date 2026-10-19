package incentives

import (
	"bytes"
	"encoding/json"
	"errors"
	"sort"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

var ErrInvalidRuleConfig = errors.New("invalid rule config")

type Context struct {
	TotalWorkUnits  int32
	TotalLeaveDays  decimal.Decimal
	UnpaidLeaveDays decimal.Decimal
	SickLeaveDays   decimal.Decimal
	LateEarlyCount  int32
	KPIScore        decimal.Decimal
}
type Policy struct {
	Name       string
	BaseAmount decimal.Decimal
	PayoutType string
}
type Tier struct {
	ID           uuid.UUID
	PolicyID     uuid.UUID
	Priority     int32
	RuleConfig   []byte
	PayoutAmount decimal.Decimal
}
type EvaluationResult struct {
	PolicyName     string
	Amount         decimal.Decimal
	StandardAmount decimal.Decimal
	TierID         *uuid.UUID
}

func EvaluatePolicy(policy Policy, tiers []Tier, context Context) (EvaluationResult, error) {
	ordered := append([]Tier(nil), tiers...)
	sort.SliceStable(ordered, func(left, right int) bool { return ordered[left].Priority < ordered[right].Priority })
	for _, tier := range ordered {
		matched, err := MatchesTier(tier, context)
		if err != nil {
			return EvaluationResult{}, err
		}
		if matched {
			amount := tier.PayoutAmount
			if policy.PayoutType == "percentage" {
				amount = policy.BaseAmount.Mul(tier.PayoutAmount)
			}
			return EvaluationResult{PolicyName: policy.Name, Amount: amount, StandardAmount: policy.BaseAmount, TierID: new(tier.ID)}, nil
		}
	}
	return EvaluationResult{PolicyName: policy.Name, Amount: decimal.Zero, StandardAmount: policy.BaseAmount}, nil
}

func MatchesTier(tier Tier, context Context) (bool, error) {
	config := bytes.TrimSpace(tier.RuleConfig)
	if len(config) == 0 || bytes.Equal(config, []byte("null")) {
		return true, nil
	}
	if config[0] == '[' {
		var rules []dynamicRule
		if err := json.Unmarshal(config, &rules); err != nil {
			return false, ErrInvalidRuleConfig
		}
		for _, rule := range rules {
			matched, err := evaluateRule(rule, context)
			if err != nil {
				return false, err
			}
			if !matched {
				return false, nil
			}
		}
		return true, nil
	}
	var flat flatRule
	if err := json.Unmarshal(config, &flat); err != nil {
		return false, ErrInvalidRuleConfig
	}
	return flat.matches(context), nil
}

type flatRule struct {
	MaxLateEarlyCount  *int32           `json:"max_late_early_count"`
	MaxUnpaidLeaveDays *decimal.Decimal `json:"max_unpaid_leave_days"`
	MaxSickLeaveDays   *decimal.Decimal `json:"max_sick_leave_days"`
	MaxTotalLeaveDays  *decimal.Decimal `json:"max_total_leave_days"`
}

func (rule flatRule) matches(context Context) bool {
	if rule.MaxLateEarlyCount != nil && context.LateEarlyCount > *rule.MaxLateEarlyCount {
		return false
	}
	if rule.MaxUnpaidLeaveDays != nil && context.UnpaidLeaveDays.GreaterThan(*rule.MaxUnpaidLeaveDays) {
		return false
	}
	if rule.MaxSickLeaveDays != nil && context.SickLeaveDays.GreaterThan(*rule.MaxSickLeaveDays) {
		return false
	}
	if rule.MaxTotalLeaveDays != nil && context.TotalLeaveDays.GreaterThan(*rule.MaxTotalLeaveDays) {
		return false
	}
	return true
}

type dynamicRule struct {
	Metric   string          `json:"metric"`
	Operator string          `json:"operator"`
	Value    decimal.Decimal `json:"value"`
}

func evaluateRule(rule dynamicRule, context Context) (bool, error) {
	value, ok := metricValue(rule.Metric, context)
	if !ok {
		return true, nil
	}
	switch rule.Operator {
	case ">":
		return value.GreaterThan(rule.Value), nil
	case "<":
		return value.LessThan(rule.Value), nil
	case ">=":
		return value.GreaterThanOrEqual(rule.Value), nil
	case "<=":
		return value.LessThanOrEqual(rule.Value), nil
	case "==":
		return value.Equal(rule.Value), nil
	case "!=":
		return !value.Equal(rule.Value), nil
	case "":
		return value.Equal(rule.Value), nil
	default:
		return false, ErrInvalidRuleConfig
	}
}
func metricValue(metric string, context Context) (decimal.Decimal, bool) {
	switch metric {
	case "total_work_units":
		return decimal.NewFromInt32(context.TotalWorkUnits), true
	case "total_leave_days":
		return context.TotalLeaveDays, true
	case "unpaid_leave_days":
		return context.UnpaidLeaveDays, true
	case "sick_leave_days":
		return context.SickLeaveDays, true
	case "late_early_count":
		return decimal.NewFromInt32(context.LateEarlyCount), true
	case "kpi_score":
		return context.KPIScore, true
	default:
		return decimal.Zero, false
	}
}
