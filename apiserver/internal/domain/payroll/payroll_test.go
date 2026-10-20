package payroll

import (
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

func TestAssemblePayslip_ComputesNetPay(t *testing.T) {
	args := AssemblePayslipArgs{ID: uuid.New(), TotalBasePay: decimal.NewFromInt(1000), TotalOvertimePay: decimal.NewFromInt(100), TotalHolidayWorkPremium: decimal.NewFromInt(50), TotalIncentiveAmount: decimal.NewFromInt(25), DeductionsAmount: decimal.NewFromInt(75)}
	payslip := AssemblePayslip(args)
	if payslip.NetPay.String() != "1100" {
		t.Fatalf("expected net 1100, got %s", payslip.NetPay)
	}
}

func TestBuildSegments_DeduplicatesSortedChanges(t *testing.T) {
	start := time.Date(2026, 10, 1, 0, 0, 0, 0, time.UTC)
	end := time.Date(2026, 10, 31, 0, 0, 0, 0, time.UTC)
	change := start.AddDate(0, 0, 10)
	segments := BuildSegments(start, end, []time.Time{change}, []time.Time{change})
	if len(segments) != 2 || !segments[0].Start.Equal(start) || !segments[0].End.Equal(change) || !segments[1].Start.Equal(change) {
		t.Fatalf("unexpected segments: %+v", segments)
	}
}

func TestMonthBoundaries_LeapYear(t *testing.T) {
	start, end, err := MonthBoundaries(2028, 2)
	if err != nil || start.Day() != 1 || end.Day() != 29 {
		t.Fatalf("unexpected boundaries: %v, %v, %v", start, end, err)
	}
	if _, _, err := MonthBoundaries(2028, 13); err == nil {
		t.Fatal("expected invalid month error")
	}
}
