package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/jackc/pgx/v5"
)

func TestEmployeeAssignmentListResolvesNumberThroughEmploymentTenure(t *testing.T) {
	db := &fakeDBTX{
		queryFn: func(_ context.Context, query string, _ ...any) (pgx.Rows, error) {
			if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "ea.start_date") {
				t.Fatalf("assignment list must resolve tenure number by assignment date: %s", query)
			}
			return &rowsStub{}, nil
		},
	}
	if _, err := (EmployeeAssignmentRepository{DB: db}).ListAssignments(context.Background()); err != nil {
		t.Fatalf("list assignments: %v", err)
	}
}

func TestEmployeeConditionListResolvesNumberThroughEmploymentTenure(t *testing.T) {
	db := &fakeDBTX{
		queryFn: func(_ context.Context, query string, _ ...any) (pgx.Rows, error) {
			if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "wc.start_date") {
				t.Fatalf("condition list must resolve tenure number by condition date: %s", query)
			}
			return &rowsStub{}, nil
		},
	}
	if _, err := (EmployeeConditionRepository{DB: db}).ListOrganizationConditions(context.Background()); err != nil {
		t.Fatalf("list organization conditions: %v", err)
	}
}
