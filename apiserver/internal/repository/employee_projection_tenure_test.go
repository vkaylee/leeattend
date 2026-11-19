package repository

import (
	"context"
	"strings"

	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func uuidPointer(value uuid.UUID) *uuid.UUID { return &value }

func TestRehireEligibilityUsesCurrentEmployment(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "FROM employments") || !strings.Contains(query, "status = 'active'") {
				t.Fatalf("rehire eligibility must resolve current employment: %s", query)
			}
			return fakeRow{scanFn: func(dest ...any) error { return pgx.ErrNoRows }}
		},
	}
	_, err := (EmployeeRepository{DB: db}).RehireEmployee(context.Background(), uuid.New(), api.RehireEmployeeRequest{HireDate: "2026-11-21", EmployerID: uuidPointer(uuid.New())})
	if err == nil {
		t.Fatal("expected missing employee error from fake row")
	}
}

func TestEmployeeDetailUsesCurrentTenureNumber(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "CURRENT_DATE") && strings.Contains(query, "emp.end_date IS NULL OR emp.end_date >= CURRENT_DATE") {
				t.Fatalf("employee detail must resolve current tenure number: %s", query)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = uuid.New()
				return pgx.ErrNoRows
			}}
		},
	}
	_, err := (EmployeeRepository{DB: db}).GetEmployee(context.Background(), uuid.New())
	if err == nil {
		t.Fatal("expected missing employee error from fake row")
	}
}

func TestESSProfileUsesCurrentTenureNumber(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "CURRENT_DATE") && strings.Contains(query, "emp.end_date IS NULL OR emp.end_date >= CURRENT_DATE") {
				t.Fatalf("ESS profile must resolve current tenure number: %s", query)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				return pgx.ErrNoRows
			}}
		},
	}
	_, err := (ESSRepository{DB: db}).GetMyProfile(context.Background(), uuid.New())
	if err == nil {
		t.Fatal("expected missing employee error from fake row")
	}
}
