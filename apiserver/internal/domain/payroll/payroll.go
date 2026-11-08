package payroll

import (
	"errors"
	"sort"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type AssemblePayslipArgs struct {
	ID                      uuid.UUID
	EmploymentID            uuid.UUID
	StartDate               time.Time
	EndDate                 time.Time
	TotalBasePay            decimal.Decimal
	TotalOvertimePay        decimal.Decimal
	TotalHolidayWorkPremium decimal.Decimal
	TotalIncentiveAmount    decimal.Decimal
	DeductionsAmount        decimal.Decimal
	TotalStdUnits           int32
	TotalOTUnits            int32
	CreatedAt               time.Time
	UpdatedAt               time.Time
}

type Payslip struct {
	ID                      uuid.UUID
	EmploymentID            uuid.UUID
	StartDate               time.Time
	EndDate                 time.Time
	TotalBasePay            decimal.Decimal
	TotalOvertimePay        decimal.Decimal
	TotalHolidayWorkPremium decimal.Decimal
	TotalIncentiveAmount    decimal.Decimal
	DeductionsAmount        decimal.Decimal
	NetPay                  decimal.Decimal
	TotalStdUnits           int32
	TotalOTUnits            int32
	CreatedAt               time.Time
	UpdatedAt               time.Time
}

func AssemblePayslip(args AssemblePayslipArgs) Payslip {
	net := args.TotalBasePay.Add(args.TotalOvertimePay).Add(args.TotalHolidayWorkPremium).Add(args.TotalIncentiveAmount).Sub(args.DeductionsAmount)
	return Payslip{ID: args.ID, EmploymentID: args.EmploymentID, StartDate: args.StartDate, EndDate: args.EndDate, TotalBasePay: args.TotalBasePay, TotalOvertimePay: args.TotalOvertimePay, TotalHolidayWorkPremium: args.TotalHolidayWorkPremium, TotalIncentiveAmount: args.TotalIncentiveAmount, DeductionsAmount: args.DeductionsAmount, NetPay: net, TotalStdUnits: args.TotalStdUnits, TotalOTUnits: args.TotalOTUnits, CreatedAt: args.CreatedAt, UpdatedAt: args.UpdatedAt}
}

type DateRange struct {
	Start time.Time
	End   time.Time
}

func BuildSegments(startDate, endDate time.Time, salaryChanges, departmentChanges []time.Time) []DateRange {
	if endDate.Before(startDate) {
		return nil
	}
	points := []time.Time{startDate}
	for _, change := range append(append([]time.Time{}, salaryChanges...), departmentChanges...) {
		if change.After(startDate) && change.Before(endDate) {
			points = append(points, change)
		}
	}
	points = append(points, endDate)
	sort.Slice(points, func(left, right int) bool { return points[left].Before(points[right]) })
	result := make([]DateRange, 0, len(points)-1)
	for index := 0; index+1 < len(points); index++ {
		if index == 0 || !points[index].Equal(points[index-1]) {
			result = append(result, DateRange{Start: points[index], End: points[index+1]})
		}
	}
	return result
}

func MonthBoundaries(year, month int) (time.Time, time.Time, error) {
	if month < 1 || month > 12 {
		return time.Time{}, time.Time{}, errors.New("month must be between 1 and 12")
	}
	start := time.Date(year, time.Month(month), 1, 0, 0, 0, 0, time.UTC)
	return start, start.AddDate(0, 1, -1), nil
}
