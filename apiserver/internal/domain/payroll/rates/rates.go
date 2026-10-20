package rates

import "github.com/shopspring/decimal"

func CalculateDailyRate(totalBase, standardDays decimal.Decimal) decimal.Decimal {
	if standardDays.IsZero() {
		return decimal.Zero
	}
	return totalBase.Div(standardDays)
}

func CalculateHourlyRate(otBase, standardDays, hours decimal.Decimal) decimal.Decimal {
	if standardDays.IsZero() || hours.IsZero() {
		return decimal.Zero
	}
	return otBase.Div(standardDays).Div(hours)
}

func ApplyOTCap(totalOTUnits int32, otPay, maxMonthlyHours decimal.Decimal, workUnitMinutes int32) (int32, decimal.Decimal) {
	if totalOTUnits <= 0 || workUnitMinutes <= 0 || maxMonthlyHours.IsNegative() {
		return 0, decimal.Zero
	}
	maxUnitsDecimal := maxMonthlyHours.Mul(decimal.NewFromInt32(60)).Div(decimal.NewFromInt32(workUnitMinutes))
	maxUnits := maxUnitsDecimal.IntPart()
	if maxUnits >= int64(totalOTUnits) {
		return totalOTUnits, otPay
	}
	if maxUnits <= 0 {
		return 0, decimal.Zero
	}
	adjusted := otPay.Mul(decimal.NewFromInt(maxUnits)).Div(decimal.NewFromInt(int64(totalOTUnits))).Round(0)
	return int32(maxUnits), adjusted
}
