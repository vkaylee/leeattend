package payroll

import (
	"encoding/json"
	"errors"
	"reflect"
	"strconv"

	"github.com/shopspring/decimal"
)

var (
	ErrImpactSnapshotIncomplete = errors.New("payroll impact: snapshot is incomplete")
	ErrImpactInputMismatch      = errors.New("payroll impact: input snapshots differ")
	ErrImpactSnapshotMalformed  = errors.New("payroll impact: snapshot is malformed")
)

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

func DecodeImpactSnapshot(raw []byte) (ImpactSnapshot, error) {
	var values map[string]json.RawMessage
	if err := json.Unmarshal(raw, &values); err != nil {
		return ImpactSnapshot{}, ErrImpactSnapshotMalformed
	}
	parseDecimal := func(key string) (*decimal.Decimal, error) {
		value, ok := values[key]
		if !ok {
			return nil, nil
		}
		var rawValue string
		if err := json.Unmarshal(value, &rawValue); err != nil {
			var number json.Number
			if err := json.Unmarshal(value, &number); err != nil {
				return nil, ErrImpactSnapshotMalformed
			}
			rawValue = number.String()
		}
		parsed, err := decimal.NewFromString(rawValue)
		if err != nil {
			return nil, ErrImpactSnapshotMalformed
		}
		return &parsed, nil
	}
	parseInt := func(key string) (*int32, error) {
		value, ok := values[key]
		if !ok {
			return nil, nil
		}
		var rawValue string
		if err := json.Unmarshal(value, &rawValue); err != nil {
			var number json.Number
			if err := json.Unmarshal(value, &number); err != nil {
				return nil, ErrImpactSnapshotMalformed
			}
			rawValue = number.String()
		}
		parsed, err := strconv.ParseInt(rawValue, 10, 32)
		if err != nil {
			return nil, ErrImpactSnapshotMalformed
		}
		result := int32(parsed)
		return &result, nil
	}
	basePay, err := parseDecimal("base_pay")
	if err != nil {
		return ImpactSnapshot{}, err
	}
	overtimePay, err := parseDecimal("overtime_pay")
	if err != nil {
		return ImpactSnapshot{}, err
	}
	deductions, err := parseDecimal("deductions")
	if err != nil {
		return ImpactSnapshot{}, err
	}
	netPay, err := parseDecimal("net_pay")
	if err != nil {
		return ImpactSnapshot{}, err
	}
	standardUnits, err := parseInt("standard_units")
	if err != nil {
		return ImpactSnapshot{}, err
	}
	overtimeUnits, err := parseInt("overtime_units")
	if err != nil {
		return ImpactSnapshot{}, err
	}
	return ImpactSnapshot{BasePay: basePay, OvertimePay: overtimePay, Deductions: deductions, NetPay: netPay, StandardUnits: standardUnits, OvertimeUnits: overtimeUnits}, nil
}

func EqualImpactInputs(before, after []byte) bool {
	var left, right any
	if json.Unmarshal(before, &left) != nil || json.Unmarshal(after, &right) != nil {
		return false
	}
	return reflect.DeepEqual(left, right)
}

func CompareImpactSnapshots(beforeInput, beforeOutput, afterInput, afterOutput []byte) (ImpactDelta, error) {
	if !EqualImpactInputs(beforeInput, afterInput) {
		return ImpactDelta{}, ErrImpactInputMismatch
	}
	before, err := DecodeImpactSnapshot(beforeOutput)
	if err != nil {
		return ImpactDelta{}, err
	}
	after, err := DecodeImpactSnapshot(afterOutput)
	if err != nil {
		return ImpactDelta{}, err
	}
	return CompareImpact(before, after)
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
