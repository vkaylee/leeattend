package rates

import (
	"testing"

	"github.com/shopspring/decimal"
)

func TestCalculateRates_ZeroSafe(t *testing.T) {
	base := decimal.RequireFromString("2600.00")
	if got := CalculateDailyRate(base, decimal.Zero); !got.IsZero() {
		t.Fatalf("expected zero daily rate, got %s", got)
	}
	if got := CalculateHourlyRate(base, decimal.NewFromInt(26), decimal.Zero); !got.IsZero() {
		t.Fatalf("expected zero hourly rate, got %s", got)
	}
	if got := CalculateDailyRate(base, decimal.NewFromInt(26)); got.String() != "100" {
		t.Fatalf("expected 100, got %s", got)
	}
}

func TestApplyOTCap_RoundsProportionally(t *testing.T) {
	units, pay := ApplyOTCap(10, decimal.NewFromInt(1000), decimal.NewFromInt(1), 60)
	if units != 1 || pay.String() != "100" {
		t.Fatalf("expected 1/100, got %d/%s", units, pay)
	}
	units, pay = ApplyOTCap(10, decimal.NewFromInt(1000), decimal.NewFromInt(20), 60)
	if units != 10 || pay.String() != "1000" {
		t.Fatalf("expected uncapped result, got %d/%s", units, pay)
	}
}
