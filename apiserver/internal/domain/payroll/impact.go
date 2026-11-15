package payroll

import (
	"errors"

	"github.com/shopspring/decimal"
)

var ErrImpactSnapshotIncomplete = errors.New("payroll impact: snapshot is incomplete")

type ImpactSnapshot struct {
	BasePay       *decimal.Decimal
	OvertimePay   *decimal.Decimal
	Deductions    *decimal.Decimal
	NetPay        *decimal.Decimal
	StandardUnits *int32
	OvertimeUnits *int32
}

type ImpactDelta struct {
	BasePayDelta       decimal.Decimal
	OvertimePayDelta   decimal.Decimal
	DeductionsDelta    decimal.Decimal
	NetPayDelta        decimal.Decimal
	StandardUnitsDelta int32
	OvertimeUnitsDelta int32
	Complete           bool
	Reason             string
}

func completeImpactSnapshot(snapshot ImpactSnapshot) bool {
	return snapshot.BasePay != nil && snapshot.OvertimePay != nil && snapshot.Deductions != nil && snapshot.NetPay != nil &&
		snapshot.StandardUnits != nil && snapshot.OvertimeUnits != nil
}

func CompareImpact(before, after ImpactSnapshot) (ImpactDelta, error) {
	if !completeImpactSnapshot(before) || !completeImpactSnapshot(after) {
		return ImpactDelta{}, ErrImpactSnapshotIncomplete
	}
	return ImpactDelta{
		BasePayDelta:       after.BasePay.Sub(*before.BasePay),
		OvertimePayDelta:   after.OvertimePay.Sub(*before.OvertimePay),
		DeductionsDelta:    after.Deductions.Sub(*before.Deductions),
		NetPayDelta:        after.NetPay.Sub(*before.NetPay),
		StandardUnitsDelta: *after.StandardUnits - *before.StandardUnits,
		OvertimeUnitsDelta: *after.OvertimeUnits - *before.OvertimeUnits,
		Complete:           true,
		Reason:             "same input snapshot, different policy result",
	}, nil
}
