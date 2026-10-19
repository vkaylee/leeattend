package repository

import (
	"context"

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
