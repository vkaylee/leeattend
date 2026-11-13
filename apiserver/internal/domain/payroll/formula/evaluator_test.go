package formula

import (
	"errors"
	"testing"

	"github.com/shopspring/decimal"
)

func TestEvaluateSupportsSafeArithmeticAndFunctions(t *testing.T) {
	value, err := Evaluate("clamp(min(actual / target, 1.2) * weight + quality * 0.4, 0, 1.2)", map[string]decimal.Decimal{
		"actual":  decimal.NewFromInt(950),
		"target":  decimal.NewFromInt(1000),
		"weight":  decimal.RequireFromString("0.6"),
		"quality": decimal.RequireFromString("0.985"),
	})
	if err != nil {
		t.Fatal(err)
	}
	if !value.Equal(decimal.RequireFromString("0.964")) {
		t.Fatalf("value = %s, want 0.964", value)
	}
}

func TestEvaluateRejectsUnsafeOrInvalidInput(t *testing.T) {
	for _, expression := range []string{"unknown + 1", "1 / 0", "exec(1)", "1; DROP TABLE payroll_inputs"} {
		if _, err := Evaluate(expression, map[string]decimal.Decimal{}); err == nil {
			t.Fatalf("Evaluate(%q) succeeded", expression)
		}
	}
	_, err := Evaluate("missing", map[string]decimal.Decimal{})
	if !errors.Is(err, ErrUnknownVariable) {
		t.Fatalf("error = %v, want unknown variable", err)
	}
}
