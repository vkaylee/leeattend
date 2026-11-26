package repository

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/jackc/pgx/v5"
)

// SystemSettingsRepository persists JSON settings in the system database.
type SystemSettingsRepository struct{ DB DBTX }

func (r SystemSettingsRepository) Get(ctx context.Context, key string) (json.RawMessage, bool, error) {
	db, err := requireDB(r.DB, "system settings repository")
	if err != nil {
		return nil, false, err
	}
	var value json.RawMessage
	err = db.QueryRow(ctx, `SELECT value_json FROM system_settings WHERE key = $1`, key).Scan(&value)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, false, nil
	}
	return value, err == nil, err
}

func (r SystemSettingsRepository) Set(ctx context.Context, key string, value json.RawMessage) error {
	db, err := requireDB(r.DB, "system settings repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO system_settings (key, value_json) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value_json = EXCLUDED.value_json, updated_at = NOW()`, key, value)
	return err
}
