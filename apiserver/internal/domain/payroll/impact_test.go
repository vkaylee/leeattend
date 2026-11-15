package payroll

import (
	"errors"
	"testing"

	"github.com/shopspring/decimal"
)

func TestCompareImpactReturnsPayAndUnitDeltas(t *testing.T) {
	beforeBase := decimal.RequireFromString("1000")
	afterBase := decimal.RequireFromString("1200")
	beforeOT := decimal.RequireFromString("100")
	afterOT := decimal.RequireFromString("150")
	beforeDed := decimal.RequireFromString("50")
	afterDed := decimal.RequireFromString("60")
	beforeNet := decimal.RequireFromString("1050")
	afterNet := decimal.RequireFromString("1290")
	beforeStd, afterStd := int32(10), int32(12)
	beforeOTUnits, afterOTUnits := int32(2), int32(3)

	got, err := CompareImpact(
		ImpactSnapshot{BasePay: &beforeBase, OvertimePay: &beforeOT, Deductions: &beforeDed, NetPay: &beforeNet, StandardUnits: &beforeStd, OvertimeUnits: &beforeOTUnits},
		ImpactSnapshot{BasePay: &afterBase, OvertimePay: &afterOT, Deductions: &afterDed, NetPay: &afterNet, StandardUnits: &afterStd, OvertimeUnits: &afterOTUnits},
	)
	if err != nil {
		t.Fatalf("compare impact: %v", err)
	}
	if !got.NetPayDelta.Equal(decimal.RequireFromString("240")) || got.StandardUnitsDelta != 2 || got.OvertimeUnitsDelta != 1 || !got.Complete {
		t.Fatalf("unexpected impact: %+v", got)
	}
}

func TestCompareImpactRejectsIncompleteSnapshot(t *testing.T) {
	_, err := CompareImpact(ImpactSnapshot{}, ImpactSnapshot{})
	if !errors.Is(err, ErrImpactSnapshotIncomplete) {
		t.Fatalf("expected incomplete snapshot error, got %v", err)
	}
}
