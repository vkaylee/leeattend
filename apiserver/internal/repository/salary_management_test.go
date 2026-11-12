package repository

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestPayrollRepository_ListSalaryManagementRows_IsPaginatedAndSinglePageQuery(t *testing.T) {
	queryCount := 0
	countSeen := false
	dataSeen := false
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, args ...any) pgx.Row {
			if !strings.Contains(query, "SELECT count(*)") {
				t.Fatalf("unexpected count query: %s", query)
			}
			if len(args) != 6 {
				t.Fatalf("count query args = %d, want 6", len(args))
			}
			countSeen = true
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 0
				return nil
			}}
		},
		queryFn: func(_ context.Context, query string, args ...any) (pgx.Rows, error) {
			queryCount++
			if !strings.Contains(query, "ORDER BY e.employee_id_number ASC, em.id ASC") {
				t.Fatalf("missing stable ordering: %s", query)
			}
			if !strings.Contains(query, "LIMIT $7 OFFSET $8") {
				t.Fatalf("missing bounded pagination: %s", query)
			}
			if strings.Contains(query, "esc.amount") || strings.Contains(query, "esc.amount::text") {
				t.Fatalf("salary amount must not enter collection query: %s", query)
			}
			if len(args) != 8 || args[6] != 25 || args[7] != 25 {
				t.Fatalf("data query args = %v, want page limit/offset at 25/25", args)
			}
			dataSeen = true
			return &fakeRows{}, nil
		},
	}

	result, err := (PayrollRepository{DB: db}).ListSalaryManagementRows(context.Background(), api.SalaryManagementQuery{
		AsOf:  timePtr(time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)),
		Page:  2,
		Limit: 25,
	})
	if err != nil {
		t.Fatalf("list salary management rows: %v", err)
	}
	if queryCount != 1 || !countSeen || !dataSeen {
		t.Fatalf("query execution count/count/data = %d/%t/%t, want 1/true/true", queryCount, countSeen, dataSeen)
	}
	if result.Total != 0 || len(result.Data) != 0 || result.Page != 2 || result.Limit != 25 {
		t.Fatalf("unexpected empty page response: %+v", result)
	}
}

func timePtr(value time.Time) *time.Time {
	return &value
}
