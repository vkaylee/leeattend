package service

import (
	"context"
	"errors"
	"fmt"

	"github.com/jackc/pgx/v5"
)

// TxBeginner starts a database transaction.
type TxBeginner interface {
	Begin(context.Context) (pgx.Tx, error)
}

// WithTx executes fn atomically. It rolls back on callback error, context
// cancellation, and panic; commit happens only after fn returns nil.
func WithTx(ctx context.Context, db TxBeginner, fn func(context.Context, pgx.Tx) error) (err error) {
	if db == nil || fn == nil {
		return errors.New("transaction database and callback are required")
	}
	if err := ctx.Err(); err != nil {
		return err
	}
	tx, err := db.Begin(ctx)
	if err != nil {
		return fmt.Errorf("begin transaction: %w", err)
	}
	defer func() {
		if recovered := recover(); recovered != nil {
			rollbackErr := tx.Rollback(ctx)
			err = fmt.Errorf("transaction callback panic: %v", recovered)
			if rollbackErr != nil && !errors.Is(rollbackErr, pgx.ErrTxClosed) {
				err = errors.Join(err, fmt.Errorf("rollback transaction: %w", rollbackErr))
			}
		}
	}()
	if err = fn(ctx, tx); err != nil {
		rollbackErr := tx.Rollback(ctx)
		if rollbackErr != nil && !errors.Is(rollbackErr, pgx.ErrTxClosed) {
			return errors.Join(err, fmt.Errorf("rollback transaction: %w", rollbackErr))
		}
		return err
	}
	if err = ctx.Err(); err != nil {
		_ = tx.Rollback(ctx)
		return err
	}
	if err = tx.Commit(ctx); err != nil {
		return fmt.Errorf("commit transaction: %w", err)
	}
	return nil
}
