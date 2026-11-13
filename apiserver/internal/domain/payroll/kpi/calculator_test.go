package kpi

import (
	"testing"

	"github.com/shopspring/decimal"
)

func TestCalculateBonus(t *testing.T) {
	bonusCap := decimal.RequireFromString("5000000")
	result, err := CalculateBonus(Inputs{
		ActualOutput:  decimal.NewFromInt(950),
		TargetOutput:  decimal.NewFromInt(1000),
		QualityRate:   decimal.RequireFromString("0.985"),
		OutputWeight:  decimal.RequireFromString("0.6"),
		QualityWeight: decimal.RequireFromString("0.4"),
		TargetBonus:   decimal.NewFromInt(5000000),
		ScoreCap:      decimal.RequireFromString("1.2"),
		BonusCap:      &bonusCap,
	})
	if err != nil {
		t.Fatalf("CalculateBonus() error = %v", err)
	}
	if !result.TotalScore.Equal(decimal.RequireFromString("0.964")) {
		t.Fatalf("total score = %s, want 0.964", result.TotalScore)
	}
	if !result.BonusAmount.Equal(decimal.RequireFromString("4820000")) {
		t.Fatalf("bonus = %s, want 4820000", result.BonusAmount)
	}
}

func TestCalculateBonusRejectsZeroTarget(t *testing.T) {
	_, err := CalculateBonus(Inputs{TargetOutput: decimal.Zero, ScoreCap: decimal.NewFromInt(1)})
	if err != ErrInvalidTarget {
		t.Fatalf("error = %v, want %v", err, ErrInvalidTarget)
	}
}
