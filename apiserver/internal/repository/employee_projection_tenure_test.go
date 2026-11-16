package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func TestEmployeeDetailUsesCurrentTenureNumber(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "CURRENT_DATE") {
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
			if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "CURRENT_DATE") {
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
