package kpi

import (
	"errors"

	"github.com/shopspring/decimal"
)

var (
	ErrInvalidTarget   = errors.New("kpi: target must be greater than zero")
	ErrInvalidWeight   = errors.New("kpi: weights must be between zero and one")
	ErrInvalidBonus    = errors.New("kpi: target bonus must not be negative")
	ErrInvalidScoreCap = errors.New("kpi: score cap must be greater than zero")
	ErrInvalidBonusCap = errors.New("kpi: bonus cap must not be negative")
)

type Inputs struct {
	ActualOutput  decimal.Decimal
	TargetOutput  decimal.Decimal
	QualityRate   decimal.Decimal
	OutputWeight  decimal.Decimal
	QualityWeight decimal.Decimal
	TargetBonus   decimal.Decimal
	ScoreCap      decimal.Decimal
	BonusCap      *decimal.Decimal
}

type Result struct {
	OutputScore  decimal.Decimal
	QualityScore decimal.Decimal
	TotalScore   decimal.Decimal
	BonusAmount  decimal.Decimal
}

func CalculateBonus(input Inputs) (Result, error) {
	if !input.TargetOutput.GreaterThan(decimal.Zero) {
		return Result{}, ErrInvalidTarget
	}
	if input.TargetBonus.IsNegative() {
		return Result{}, ErrInvalidBonus
	}
	if input.ScoreCap.IsNegative() || input.ScoreCap.IsZero() {
		return Result{}, ErrInvalidScoreCap
	}
	if !validWeight(input.OutputWeight) || !validWeight(input.QualityWeight) {
		return Result{}, ErrInvalidWeight
	}
	if input.BonusCap != nil && input.BonusCap.IsNegative() {
		return Result{}, ErrInvalidBonusCap
	}

	outputScore := input.ActualOutput.Div(input.TargetOutput)
	if outputScore.GreaterThan(input.ScoreCap) {
		outputScore = input.ScoreCap
	}
	qualityScore := input.QualityRate
	totalScore := outputScore.Mul(input.OutputWeight).Add(qualityScore.Mul(input.QualityWeight))
	bonus := input.TargetBonus.Mul(totalScore).Round(2)
	if input.BonusCap != nil && bonus.GreaterThan(*input.BonusCap) {
		bonus = *input.BonusCap
	}
	return Result{
		OutputScore:  outputScore,
		QualityScore: qualityScore,
		TotalScore:   totalScore,
		BonusAmount:  bonus,
	}, nil
}

func validWeight(value decimal.Decimal) bool {
	return !value.IsNegative() && !value.GreaterThan(decimal.NewFromInt(1))
}
