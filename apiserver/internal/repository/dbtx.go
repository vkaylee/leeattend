package repository

import (
	"context"
	"errors"
	"fmt"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// TxBeginner starts a database transaction.
type TxBeginner = service.TxBeginner

// DBTX is the minimal database transaction interface satisfied by *pgxpool.Pool, *pgx.Conn, and pgx.Tx.
type DBTX interface {
	Exec(ctx context.Context, sql string, arguments ...any) (commandTag pgconn.CommandTag, err error)
	Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error)
	QueryRow(ctx context.Context, sql string, args ...any) pgx.Row
}

var errNoRows = fmt.Errorf("%w: %w", api.ErrNotFound, pgx.ErrNoRows)

type noRowsRow struct {
	pgx.Row
}

func (r noRowsRow) Scan(dest ...any) error {
	err := r.Row.Scan(dest...)
	if errors.Is(err, pgx.ErrNoRows) {
		return fmt.Errorf("%w: %w", api.ErrNotFound, err)
	}
	return err
}

type noRowsDB struct {
	DBTX
}

func (d noRowsDB) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row {
	return noRowsRow{Row: d.DBTX.QueryRow(ctx, sql, args...)}
}

type noRowsTx struct {
	pgx.Tx
}

func (t noRowsTx) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row {
	return noRowsRow{Row: t.Tx.QueryRow(ctx, sql, args...)}
}

func (t noRowsTx) Begin(ctx context.Context) (pgx.Tx, error) {
	tx, err := t.Tx.Begin(ctx)
	if err != nil {
		return nil, err
	}
	return noRowsTx{Tx: tx}, nil
}

type noRowsTxDB struct {
	noRowsDB
	beginner TxBeginner
}

func (d noRowsTxDB) Begin(ctx context.Context) (pgx.Tx, error) {
	tx, err := d.beginner.Begin(ctx)
	if err != nil {
		return nil, err
	}
	return noRowsTx{Tx: tx}, nil
}

func wrapNoRows(db DBTX) DBTX {
	switch db.(type) {
	case noRowsDB, noRowsTxDB, noRowsTx:
		return db
	}
	wrapped := noRowsDB{DBTX: db}
	if beginner, ok := db.(TxBeginner); ok {
		return noRowsTxDB{noRowsDB: wrapped, beginner: beginner}
	}
	return wrapped
}

func requireDB(db DBTX, scope string) (DBTX, error) {
	if db == nil {
		return nil, fmt.Errorf("%s: database is not configured", scope)
	}
	return wrapNoRows(db), nil
}

func isUniqueViolation(err error) bool {
	var pgErr *pgconn.PgError
	return errors.As(err, &pgErr) && pgErr.Code == "23505"
}

func isConflictError(err error) bool {
	var pgErr *pgconn.PgError
	return errors.As(err, &pgErr) && (pgErr.Code == "23505" || pgErr.Code == "23P01")
}
