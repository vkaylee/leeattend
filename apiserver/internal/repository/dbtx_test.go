package repository

import (
	"context"
	"errors"
	"fmt"
	"testing"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

type dummyDBTX struct{}

func (d *dummyDBTX) Exec(context.Context, string, ...any) (pgconn.CommandTag, error) {
	return pgconn.CommandTag{}, nil
}

func (d *dummyDBTX) Query(context.Context, string, ...any) (pgx.Rows, error) {
	return nil, nil
}

func (d *dummyDBTX) QueryRow(context.Context, string, ...any) pgx.Row {
	return nil
}

var _ DBTX = (*dummyDBTX)(nil)

func TestRequireDB(t *testing.T) {
	if _, err := requireDB(nil, "test repo"); err == nil {
		t.Fatal("expected error for nil DB")
	}
	db, err := requireDB(&dummyDBTX{}, "test repo")
	if err != nil || db == nil {
		t.Fatalf("expected wrapped DB, got db=%v err=%v", db, err)
	}
}

func TestPGConflictHelpers(t *testing.T) {
	unique := &pgconn.PgError{Code: "23505"}
	exclusion := &pgconn.PgError{Code: "23P01"}
	for _, tc := range []struct {
		name             string
		err              error
		unique, conflict bool
	}{
		{"nil", nil, false, false},
		{"unique", unique, true, true},
		{"wrapped unique", fmt.Errorf("wrap: %w", unique), true, true},
		{"exclusion", exclusion, false, true},
		{"wrapped exclusion", fmt.Errorf("wrap: %w", exclusion), false, true},
		{"foreign key", &pgconn.PgError{Code: "23503"}, false, false},
		{"other", errors.New("other"), false, false},
	} {
		t.Run(tc.name, func(t *testing.T) {
			if got := isUniqueViolation(tc.err); got != tc.unique {
				t.Errorf("unique = %v, want %v", got, tc.unique)
			}
			if got := isConflictError(tc.err); got != tc.conflict {
				t.Errorf("conflict = %v, want %v", got, tc.conflict)
			}
		})
	}
}
