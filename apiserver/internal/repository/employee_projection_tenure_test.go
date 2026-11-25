package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func TestEmployeeDetailUsesDisplayTenureLateral(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "tenure_status") {
				t.Fatalf("employee detail must resolve display tenure lateral: %s", query)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				return pgx.ErrNoRows
			}}
		},
	}
	_, err := (EmployeeRepository{DB: db}).GetEmployee(context.Background(), uuid.New())
	if err == nil {
		t.Fatal("expected missing employee error from fake row")
	}
}

func TestESSProfileUsesEffectiveTenureLateral(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "tenure.user_id") {
				t.Fatalf("ESS profile must resolve effective tenure user_id and number: %s", query)
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
