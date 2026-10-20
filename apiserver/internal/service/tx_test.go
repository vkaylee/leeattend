package service

import (
	"context"
	"errors"
	"testing"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

type fakeTx struct {
	committed, rolledBack bool
}

func (f *fakeTx) Commit(context.Context) error          { f.committed = true; return nil }
func (f *fakeTx) Rollback(context.Context) error        { f.rolledBack = true; return nil }
func (f *fakeTx) Begin(context.Context) (pgx.Tx, error) { return f, nil }
func (f *fakeTx) Exec(context.Context, string, ...any) (pgconn.CommandTag, error) {
	return pgconn.CommandTag{}, nil
}
func (f *fakeTx) Query(context.Context, string, ...any) (pgx.Rows, error) { return nil, nil }
func (f *fakeTx) QueryRow(context.Context, string, ...any) pgx.Row        { return nil }
func (f *fakeTx) CopyFrom(context.Context, pgx.Identifier, []string, pgx.CopyFromSource) (int64, error) {
	return 0, nil
}
func (f *fakeTx) SendBatch(context.Context, *pgx.Batch) pgx.BatchResults { return nil }
func (f *fakeTx) LargeObjects() pgx.LargeObjects                         { return pgx.LargeObjects{} }
func (f *fakeTx) Prepare(context.Context, string, string) (*pgconn.StatementDescription, error) {
	return nil, nil
}
func (f *fakeTx) Conn() *pgx.Conn { return nil }

type fakeBeginner struct {
	tx  *fakeTx
	err error
}

func (f *fakeBeginner) Begin(context.Context) (pgx.Tx, error) {
	if f.err != nil {
		return nil, f.err
	}
	return f.tx, nil
}

func TestWithTx_SuccessCommits(t *testing.T) {
	tx := &fakeTx{}
	db := &fakeBeginner{tx: tx}
	err := WithTx(context.Background(), db, func(ctx context.Context, tx pgx.Tx) error { return nil })
	if err != nil || !tx.committed || tx.rolledBack {
		t.Fatalf("expected commit, got err=%v tx=%+v", err, tx)
	}
}

func TestWithTx_ErrorRollsBack(t *testing.T) {
	tx := &fakeTx{}
	db := &fakeBeginner{tx: tx}
	targetErr := errors.New("boom")
	err := WithTx(context.Background(), db, func(ctx context.Context, tx pgx.Tx) error { return targetErr })
	if !errors.Is(err, targetErr) || tx.committed || !tx.rolledBack {
		t.Fatalf("expected rollback, got err=%v tx=%+v", err, tx)
	}
}
