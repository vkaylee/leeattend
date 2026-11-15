package repository

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
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
			if len(args) != 5 {
				t.Fatalf("count query args = %d, want 5", len(args))
			}
			countSeen = true
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 0
				return nil
			}}
		},
		queryFn: func(_ context.Context, query string, args ...any) (pgx.Rows, error) {
			queryCount++
			if !strings.Contains(query, "ORDER BY COALESCE(tenure.employee_number, e.employee_id_number) ASC, em.id ASC, configured_component.effective_date DESC") {
				t.Fatalf("missing stable tenure-aware ordering: %s", query)
			}
			if !strings.Contains(query, "LIMIT $6 OFFSET $7") {
				t.Fatalf("missing bounded pagination: %s", query)
			}
			if !strings.Contains(query, "configured_component.amount") {
				t.Fatalf("configured amount missing from collection query: %s", query)
			}
			if len(args) != 7 || args[5] != 25 || args[6] != 25 {
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

func TestPayrollRepository_ListSalaryManagementRows_MapsConfiguredComponentRows(t *testing.T) {
	employmentID := uuid.New()
	employeeID := uuid.New()
	employerID := uuid.New()
	departmentID := uuid.New()
	componentID := uuid.New()
	definitionID := uuid.New()
	departmentName := "Engineering"
	jobTitle := "Engineer"
	endDate := ""

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "SELECT count(*)") {
				t.Fatalf("unexpected count query: %s", query)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 1
				return nil
			}}
		},
		queryFn: func(_ context.Context, query string, _ ...any) (pgx.Rows, error) {
			if !strings.Contains(query, "configured_component.amount") {
				t.Fatalf("configured amount missing from query: %s", query)
			}
			return &salaryRows{rows: [][]any{{
				employmentID, employeeID, "EMP-001", "Employee A",
				employerID, "Acme Ltd", departmentID, departmentName, jobTitle,
				"active", "full_time", "2026-01-01", endDate,
				componentID, definitionID, "Base salary", "BASE", "5000.00", "2026-10-01",
			}}}, nil
		},
	}

	result, err := (PayrollRepository{DB: db}).ListSalaryManagementRows(context.Background(), api.SalaryManagementQuery{
		AsOf:  timePtr(time.Date(2026, 11, 12, 0, 0, 0, 0, time.UTC)),
		Page:  1,
		Limit: 25,
	})
	if err != nil {
		t.Fatalf("list salary management rows: %v", err)
	}
	if result.Total != 1 || len(result.Data) != 1 {
		t.Fatalf("unexpected result size: %+v", result)
	}
	row := result.Data[0]
	if row.EmploymentID != employmentID || row.EmployeeID != employeeID || row.ComponentID != componentID ||
		row.ComponentDefinitionID != definitionID || row.ComponentCode != "BASE" || row.Amount.String() != "5000" ||
		row.EffectiveDate != "2026-10-01" || row.SalaryStatus != "current" {
		t.Fatalf("unexpected mapped salary row: %+v", row)
	}
}

func TestPayrollRepository_ListSalaryManagementRows_ReturnsMalformedAmountError(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 1
				return nil
			}}
		},
		queryFn: func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
			return &salaryRows{rows: [][]any{{
				uuid.New(), uuid.New(), "EMP-001", "Employee A", uuid.New(), "Acme Ltd",
				nil, nil, nil, "active", "full_time", "2026-01-01", nil,
				uuid.New(), uuid.New(), "Base salary", "BASE", "not-a-decimal", "2026-10-01",
			}}}, nil
		},
	}

	_, err := (PayrollRepository{DB: db}).ListSalaryManagementRows(context.Background(), api.SalaryManagementQuery{
		AsOf:  timePtr(time.Date(2026, 11, 12, 0, 0, 0, 0, time.UTC)),
		Page:  1,
		Limit: 25,
	})
	if err == nil {
		t.Fatal("expected malformed amount error")
	}
}

type salaryRows struct {
	rows [][]any
	idx  int
}

func (r *salaryRows) Close()                                       {}
func (r *salaryRows) Err() error                                   { return nil }
func (r *salaryRows) CommandTag() pgconn.CommandTag                { return pgconn.CommandTag{} }
func (r *salaryRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *salaryRows) Next() bool {
	if r.idx < len(r.rows) {
		r.idx++
		return true
	}
	return false
}
func (r *salaryRows) Scan(dest ...any) error {
	row := r.rows[r.idx-1]
	for i, value := range row {
		switch target := dest[i].(type) {
		case *uuid.UUID:
			*target = value.(uuid.UUID)
		case *string:
			*target = value.(string)
		case **uuid.UUID:
			if value == nil {
				*target = nil
			} else {
				v := value.(uuid.UUID)
				*target = &v
			}
		case **string:
			if value == nil {
				*target = nil
			} else {
				v := value.(string)
				*target = &v
			}
		}
	}
	return nil
}
func (r *salaryRows) Values() ([]any, error) { return nil, nil }
func (r *salaryRows) RawValues() [][]byte    { return nil }
func (r *salaryRows) Conn() *pgx.Conn        { return nil }
