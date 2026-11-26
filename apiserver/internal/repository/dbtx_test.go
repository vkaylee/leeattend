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
	uniqueErr := &pgconn.PgError{Code: "23505"}
	exclusionErr := &pgconn.PgError{Code: "23P01"}
	fkErr := &pgconn.PgError{Code: "23503"}
	wrappedUnique := fmt.Errorf("wrap: %w", uniqueErr)
	wrappedExclusion := fmt.Errorf("wrap: %w", exclusionErr)

	if !isUniqueViolation(uniqueErr) || !isUniqueViolation(wrappedUnique) {
		t.Fatal("expected 23505 to match isUniqueViolation")
	}
	if isUniqueViolation(exclusionErr) || isUniqueViolation(fkErr) || isUniqueViolation(errors.New("other")) {
		t.Fatal("unexpected match in isUniqueViolation")
	}
	if !isConflictError(uniqueErr) || !isConflictError(wrappedUnique) || !isConflictError(exclusionErr) || !isConflictError(wrappedExclusion) {
		t.Fatal("expected 23505 and 23P01 to match isConflictError")
	}
	if isConflictError(fkErr) || isConflictError(errors.New("other")) {
		t.Fatal("unexpected match in isConflictError")
	}
}
