package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// ExternalAttendancePullRepository stores per-source pull checkpoints and event identity metadata.
type ExternalAttendancePullRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
	BeginTx  func(context.Context, uuid.UUID) (TxBeginner, error)
}

type AttendanceSourcePullState struct {
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

func (r ExternalAttendancePullRepository) GetState(ctx context.Context, workspaceID, sourceID uuid.UUID) (AttendanceSourcePullState, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return AttendanceSourcePullState{}, err
	}
	return pullState(db, ctx, workspaceID, sourceID)
}

func (r ExternalAttendancePullRepository) CurrentCursor(ctx context.Context, workspaceID, sourceID uuid.UUID) (string, error) {
	s, err := r.GetState(ctx, workspaceID, sourceID)
	if err != nil || s.Cursor == nil {
		return "", err
	}
	return *s.Cursor, nil
}

func pullState(db DBTX, ctx context.Context, workspaceID, sourceID uuid.UUID) (AttendanceSourcePullState, error) {
	var s AttendanceSourcePullState
	err := db.QueryRow(ctx, `INSERT INTO attendance_source_pull_states (workspace_id, source_id) VALUES ($1, $2)
		ON CONFLICT (workspace_id, source_id) DO UPDATE SET updated_at = attendance_source_pull_states.updated_at
		RETURNING cursor, status, lease_owner, lease_token, lease_expires_at, last_success_at, last_failure_at, last_error`, workspaceID, sourceID).
		Scan(&s.Cursor, &s.Status, &s.LeaseOwner, &s.LeaseToken, &s.LeaseExpiresAt, &s.LastSuccessAt, &s.LastFailureAt, &s.LastError)
	return s, err
}

func (r ExternalAttendancePullRepository) AcquireLease(ctx context.Context, workspaceID, sourceID uuid.UUID, owner string, until time.Time) (uuid.UUID, bool, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return uuid.Nil, false, err
	}
	token := uuid.New()
	var claimed uuid.UUID
	err = db.QueryRow(ctx, `INSERT INTO attendance_source_pull_states
		(workspace_id, source_id, status, lease_owner, lease_token, lease_expires_at)
		VALUES ($1, $2, 'running', $3, $4, $5)
		ON CONFLICT (workspace_id, source_id) DO UPDATE SET
		status = 'running', lease_owner = EXCLUDED.lease_owner, lease_token = EXCLUDED.lease_token,
		lease_expires_at = EXCLUDED.lease_expires_at, updated_at = CURRENT_TIMESTAMP
		WHERE attendance_source_pull_states.lease_expires_at IS NULL OR attendance_source_pull_states.lease_expires_at <= CURRENT_TIMESTAMP
		RETURNING lease_token`, workspaceID, sourceID, owner, token, until).Scan(&claimed)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, false, nil
	}
	return claimed, err == nil, err
}

func (r ExternalAttendancePullRepository) RecordEvent(ctx context.Context, workspaceID uuid.UUID, providerSourceID, externalID string, occurredAt time.Time, metadata []byte) (bool, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return false, err
	}
	return recordEvent(ctx, db, workspaceID, providerSourceID, externalID, occurredAt, metadata)
}

func recordEvent(ctx context.Context, db DBTX, workspaceID uuid.UUID, providerSourceID, externalID string, occurredAt time.Time, metadata []byte) (bool, error) {
	tag, err := db.Exec(ctx, `INSERT INTO external_attendance_events
		(workspace_id, source_id, external_event_id, occurred_at, metadata)
		VALUES ($1, $2, $3, $4, $5::jsonb) ON CONFLICT (workspace_id, source_id, external_event_id) DO NOTHING`, workspaceID, providerSourceID, externalID, occurredAt, metadata)
	return tag.RowsAffected() == 1, err
}

func (r ExternalAttendancePullRepository) AdvanceCursor(ctx context.Context, workspaceID, sourceID, leaseToken uuid.UUID, cursor string) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	return advanceCursor(ctx, db, workspaceID, sourceID, leaseToken, cursor)
}
func advanceCursor(ctx context.Context, db DBTX, workspaceID, sourceID, leaseToken uuid.UUID, cursor string) error {
	tag, err := db.Exec(ctx, `UPDATE attendance_source_pull_states SET cursor = $1, status = 'idle', lease_owner = NULL, lease_token = NULL, lease_expires_at = NULL, last_success_at = CURRENT_TIMESTAMP, last_error = NULL, updated_at = CURRENT_TIMESTAMP WHERE workspace_id = $2 AND source_id = $3 AND lease_token = $4`, cursor, workspaceID, sourceID, leaseToken)
	if err != nil {
		return err
	}
	if tag.RowsAffected() != 1 {
		return errors.New("external attendance pull repository: lease is no longer owned")
	}
	return nil
}
func (r ExternalAttendancePullRepository) ReleaseLease(ctx context.Context, workspaceID, sourceID, leaseToken uuid.UUID, failure error) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE attendance_source_pull_states SET status = 'failed', lease_owner = NULL, lease_token = NULL, lease_expires_at = NULL, last_failure_at = CURRENT_TIMESTAMP, last_error = $1, updated_at = CURRENT_TIMESTAMP WHERE workspace_id = $2 AND source_id = $3 AND lease_token = $4`, failureText(failure), workspaceID, sourceID, leaseToken)
	if err == nil && tag.RowsAffected() != 1 {
		return errors.New("external attendance pull repository: lease is no longer owned")
	}
	return err
}
func (r ExternalAttendancePullRepository) ReleaseLeaseSuccess(ctx context.Context, workspaceID, sourceID, leaseToken uuid.UUID) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE attendance_source_pull_states SET status = 'idle', lease_owner = NULL, lease_token = NULL, lease_expires_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE workspace_id = $1 AND source_id = $2 AND lease_token = $3`, workspaceID, sourceID, leaseToken)
	if err == nil && tag.RowsAffected() != 1 {
		return errors.New("external attendance pull repository: lease is no longer owned")
	}
	return err
}
func failureText(err error) any {
	if err == nil {
		return nil
	}
	text := err.Error()
	if len(text) > 1024 {
		text = text[:1024]
	}
	return text
}
func (r ExternalAttendancePullRepository) RecordEventTx(ctx context.Context, tx pgx.Tx, workspaceID uuid.UUID, providerSourceID, externalID string, occurredAt time.Time, metadata []byte) (bool, error) {
	return recordEvent(ctx, tx, workspaceID, providerSourceID, externalID, occurredAt, metadata)
}
func (r ExternalAttendancePullRepository) AdvanceCursorTx(ctx context.Context, tx pgx.Tx, workspaceID, sourceID, leaseToken uuid.UUID, cursor string) error {
	return advanceCursor(ctx, tx, workspaceID, sourceID, leaseToken, cursor)
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
