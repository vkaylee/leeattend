package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/jackc/pgx/v5"
)

func TestLeaveBalanceSearchIncludesHistoricalTenureNumbers(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "employee_tenures") {
				t.Fatalf("leave balance count must search tenure numbers: %s", query)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 0
				return nil
			}}
		},
		queryFn: func(_ context.Context, query string, _ ...any) (pgx.Rows, error) {
			if !strings.Contains(query, "employee_tenures") {
				t.Fatalf("leave balance data must search tenure numbers: %s", query)
			}
			return &leaveTestRows{}, nil
		},
	}
	if _, _, err := (LeaveRepository{DB: db}).GetOrganizationLeaveBalances(context.Background(), 2026, 1, 20, "1541"); err != nil {
		t.Fatalf("get organization leave balances: %v", err)
	}
}
