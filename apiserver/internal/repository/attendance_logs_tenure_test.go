package repository

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func TestResolveImportedEmploymentUsesTenureNumberAndPunchDate(t *testing.T) {
	var query string
	employmentID := uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			query = sql
			if len(args) != 2 || args[0] != "001541" {
				t.Fatalf("expected normalized import arguments, got %v", args)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int)) = 1
				*(dest[1].(**uuid.UUID)) = &employmentID
				return nil
			}}
		},
	}

	resolved, err := resolveImportedEmployment(context.Background(), db, "001541", time.Date(2026, 1, 15, 0, 0, 0, 0, time.UTC))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if resolved != employmentID {
		t.Fatalf("expected employment %s, got %s", employmentID, resolved)
	}
	for _, fragment := range []string{
		"employee_tenures t",
		"t.employee_number",
		"t.start_date <= $2::date",
		"t.end_date",
		"employment_relationships er",
	} {
		if !strings.Contains(query, fragment) {
			t.Fatalf("resolver query missing %q: %s", fragment, query)
		}
	}
}

func TestResolveImportedEmploymentRejectsAmbiguousEffectiveNumber(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int)) = 2
				return nil
			}}
		},
	}

	_, err := resolveImportedEmployment(context.Background(), db, "1541", time.Date(2026, 1, 15, 0, 0, 0, 0, time.UTC))
	if !errors.Is(err, errAmbiguousImportedEmployment) {
		t.Fatalf("expected ambiguous-number error, got %v", err)
	}
}
