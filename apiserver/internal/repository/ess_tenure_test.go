package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func TestESSRepository_ResolveEmployeeForUserUsesCurrentTenureAccount(t *testing.T) {
	userID := uuid.New()
	employeeID := uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, args ...any) pgx.Row {
			if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "t.user_id") {
				t.Fatalf("resolution query must use current tenure account, got %s", query)
			}
			if len(args) != 1 || args[0] != userID {
				t.Fatalf("resolution args = %v, want user ID %s", args, userID)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employeeID
				return nil
			}}
		},
	}

	resolved, err := (ESSRepository{DB: db}).ResolveEmployeeForUser(context.Background(), userID)
	if err != nil {
		t.Fatalf("resolve employee: %v", err)
	}
	if resolved != employeeID {
		t.Fatalf("resolved employee = %s, want %s", resolved, employeeID)
	}
}
