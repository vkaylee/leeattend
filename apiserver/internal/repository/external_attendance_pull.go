package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// ExternalAttendancePullRepository stores per-device pull checkpoints and event identity metadata.
type ExternalAttendancePullRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
	BeginTx  func(context.Context, uuid.UUID) (TxBeginner, error)
}

type DevicePullState struct {
	Cursor         *string
	Status         string
	LeaseOwner     *string
	LeaseToken     *uuid.UUID
	LeaseExpiresAt *time.Time
	LastSuccessAt  *time.Time
	LastFailureAt  *time.Time
	LastError      *string
}

func (r ExternalAttendancePullRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("external attendance pull repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

// GetState returns state, creating an idle row for device on first use.
func (r ExternalAttendancePullRepository) GetState(ctx context.Context, workspaceID, deviceID uuid.UUID) (DevicePullState, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return DevicePullState{}, err
	}
	return pullState(db, ctx, workspaceID, deviceID)
}

// CurrentCursor returns checkpoint cursor, creating device state when absent.
func (r ExternalAttendancePullRepository) CurrentCursor(ctx context.Context, workspaceID, deviceID uuid.UUID) (string, error) {
	s, err := r.GetState(ctx, workspaceID, deviceID)
	if err != nil || s.Cursor == nil {
		return "", err
	}
	return *s.Cursor, nil
}

func pullState(db DBTX, ctx context.Context, workspaceID, deviceID uuid.UUID) (DevicePullState, error) {
	var s DevicePullState
	err := db.QueryRow(ctx, `INSERT INTO device_pull_states (workspace_id, device_id) VALUES ($1, $2)
		ON CONFLICT (workspace_id, device_id) DO UPDATE SET updated_at = device_pull_states.updated_at
		RETURNING cursor, status, lease_owner, lease_token, lease_expires_at, last_success_at, last_failure_at, last_error`, workspaceID, deviceID).
		Scan(&s.Cursor, &s.Status, &s.LeaseOwner, &s.LeaseToken, &s.LeaseExpiresAt, &s.LastSuccessAt, &s.LastFailureAt, &s.LastError)
	return s, err
}

// AcquireLease atomically claims device, unless active lease belongs to another owner.
func (r ExternalAttendancePullRepository) AcquireLease(ctx context.Context, workspaceID, deviceID uuid.UUID, owner string, until time.Time) (uuid.UUID, bool, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return uuid.Nil, false, err
	}
	token := uuid.New()
	var claimed uuid.UUID
	err = db.QueryRow(ctx, `INSERT INTO device_pull_states
		(workspace_id, device_id, status, lease_owner, lease_token, lease_expires_at)
		VALUES ($1, $2, 'running', $3, $4, $5)
		ON CONFLICT (workspace_id, device_id) DO UPDATE SET
		status = 'running', lease_owner = EXCLUDED.lease_owner, lease_token = EXCLUDED.lease_token,
		lease_expires_at = EXCLUDED.lease_expires_at, updated_at = CURRENT_TIMESTAMP
		WHERE device_pull_states.lease_expires_at IS NULL OR device_pull_states.lease_expires_at <= CURRENT_TIMESTAMP
		RETURNING lease_token`, workspaceID, deviceID, owner, token, until).Scan(&claimed)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, false, nil
	}
	return claimed, err == nil, err
}

// RecordEvent is idempotent within workspace and provider device. Returns false for duplicate event.
func (r ExternalAttendancePullRepository) RecordEvent(ctx context.Context, workspaceID uuid.UUID, providerDeviceID string, externalID string, occurredAt time.Time, metadata []byte) (bool, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return false, err
	}
	return recordEvent(ctx, db, workspaceID, providerDeviceID, externalID, occurredAt, metadata)
}

func recordEvent(ctx context.Context, db DBTX, workspaceID uuid.UUID, providerDeviceID string, externalID string, occurredAt time.Time, metadata []byte) (bool, error) {
	tag, err := db.Exec(ctx, `INSERT INTO external_attendance_events
		(workspace_id, device_id, external_event_id, occurred_at, metadata)
		VALUES ($1, $2, $3, $4, $5::jsonb) ON CONFLICT (workspace_id, device_id, external_event_id) DO NOTHING`, workspaceID, providerDeviceID, externalID, occurredAt, metadata)
	return tag.RowsAffected() == 1, err
}

// AdvanceCursor commits cursor and success only for current lease.
func (r ExternalAttendancePullRepository) AdvanceCursor(ctx context.Context, workspaceID, deviceID, leaseToken uuid.UUID, cursor string) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE device_pull_states SET cursor = $1, status = 'idle', lease_owner = NULL,
		lease_token = NULL, lease_expires_at = NULL, last_success_at = CURRENT_TIMESTAMP,
		last_error = NULL, updated_at = CURRENT_TIMESTAMP
		WHERE workspace_id = $2 AND device_id = $3 AND lease_token = $4`, cursor, workspaceID, deviceID, leaseToken)
	if err != nil {
		return err
	}
	if tag.RowsAffected() != 1 {
		return errors.New("external attendance pull repository: lease is no longer owned")
	}
	return nil
}

// ReleaseLease records failure and leaves cursor unchanged.
func (r ExternalAttendancePullRepository) ReleaseLease(ctx context.Context, workspaceID, deviceID, leaseToken uuid.UUID, failure error) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE device_pull_states SET status = 'failed', lease_owner = NULL,
		lease_token = NULL, lease_expires_at = NULL, last_failure_at = CURRENT_TIMESTAMP,
		last_error = $1, updated_at = CURRENT_TIMESTAMP
		WHERE workspace_id = $2 AND device_id = $3 AND lease_token = $4`, failureText(failure), workspaceID, deviceID, leaseToken)
	if err == nil && tag.RowsAffected() != 1 {
		return errors.New("external attendance pull repository: lease is no longer owned")
	}
	return err
}

// ReleaseLeaseSuccess releases lease without changing cursor or failure state.
func (r ExternalAttendancePullRepository) ReleaseLeaseSuccess(ctx context.Context, workspaceID, deviceID, leaseToken uuid.UUID) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE device_pull_states SET status = 'idle', lease_owner = NULL,
		lease_token = NULL, lease_expires_at = NULL, updated_at = CURRENT_TIMESTAMP
		WHERE workspace_id = $1 AND device_id = $2 AND lease_token = $3`, workspaceID, deviceID, leaseToken)
	if err == nil && tag.RowsAffected() != 1 {
		return errors.New("external attendance pull repository: lease is no longer owned")
	}
	return err
}

func failureText(err error) any {
	if err == nil {
		return nil
	}
	const maxFailureText = 1024
	text := err.Error()
	if len(text) > maxFailureText {
		text = text[:maxFailureText]
	}
	return text
}

// RecordEventTx inserts event metadata on caller transaction.
func (r ExternalAttendancePullRepository) RecordEventTx(ctx context.Context, tx pgx.Tx, workspaceID uuid.UUID, providerDeviceID string, externalID string, occurredAt time.Time, metadata []byte) (bool, error) {
	return recordEvent(ctx, tx, workspaceID, providerDeviceID, externalID, occurredAt, metadata)
}

func (r ExternalAttendancePullRepository) AdvanceCursorTx(ctx context.Context, tx pgx.Tx, workspaceID, deviceID, leaseToken uuid.UUID, cursor string) error {
	tag, err := tx.Exec(ctx, `UPDATE device_pull_states SET cursor = $1, status = 'idle', lease_owner = NULL,
		lease_token = NULL, lease_expires_at = NULL, last_success_at = CURRENT_TIMESTAMP,
		last_error = NULL, updated_at = CURRENT_TIMESTAMP
		WHERE workspace_id = $2 AND device_id = $3 AND lease_token = $4`, cursor, workspaceID, deviceID, leaseToken)
	if err != nil {
		return err
	}
	if tag.RowsAffected() != 1 {
		return errors.New("external attendance pull repository: lease is no longer owned")
	}
	return nil
}

func (r ExternalAttendancePullRepository) WithPageTransaction(ctx context.Context, workspaceID uuid.UUID, fn func(context.Context, pgx.Tx) error) error {
	var beginner TxBeginner
	if r.BeginTx != nil {
		var err error
		beginner, err = r.BeginTx(ctx, workspaceID)
		if err != nil {
			return err
		}
	} else {
		db, err := r.db(ctx, workspaceID)
		if err != nil {
			return err
		}
		var ok bool
		beginner, ok = db.(TxBeginner)
		if !ok {
			return errors.New("external attendance pull repository: tenant database does not support transactions")
		}
	}
	return service.WithTx(ctx, beginner, fn)
}
