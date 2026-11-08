package repository

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/jackc/pgx/v5/pgconn"
)

func TestEnqueueRosterCacheInvalidation(t *testing.T) {
	var executedSQL string
	db := &fakeDBTX{
		execFn: func(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
			executedSQL = sql
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}

	if err := EnqueueRosterCacheInvalidation(context.Background(), db); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(executedSQL, "RosterCacheInvalidation") {
		t.Errorf("expected RosterCacheInvalidation job type, got: %s", executedSQL)
	}
}

func TestEnqueueRosterCacheInvalidationPropagatesError(t *testing.T) {
	expectedErr := errors.New("db failure")
	db := &fakeDBTX{
		execFn: func(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
			return pgconn.CommandTag{}, expectedErr
		},
	}

	if err := EnqueueRosterCacheInvalidation(context.Background(), db); !errors.Is(err, expectedErr) {
		t.Fatalf("expected %v, got %v", expectedErr, err)
	}
}
