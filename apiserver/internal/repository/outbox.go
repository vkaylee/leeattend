package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"

	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

const (
	ScheduleOfficeCoverageRefreshJob = "ScheduleOfficeCoverageRefresh"
	maxScheduleOfficeAttempts        = 5
)

var (
	ErrOutboxSchemaUnavailable         = errors.New("outbox retry schema is unavailable")
	ErrOutboxLeaseLost                 = errors.New("outbox job lease was lost")
	ErrOutboxJobNotFound               = errors.New("outbox job was not found")
	ErrScheduleOfficePolicyUnavailable = errors.New("approved office policy schema is unavailable")
)

type ScheduleOfficeCoverageEvent struct {
	ApprovalInstanceID uuid.UUID `json:"approval_instance_id"`
	EmployerID         uuid.UUID `json:"employer_id"`
	EmploymentID       uuid.UUID `json:"employment_id"`
	BusinessDate       string    `json:"business_date"`
	Mode               string    `json:"mode"`
}

// EnqueueRosterCacheInvalidation writes a durable outbox job for roster cache invalidation.
func EnqueueRosterCacheInvalidation(ctx context.Context, db DBTX) error {
	_, err := db.Exec(ctx, `INSERT INTO outbox_jobs (id, job_type, payload, status, error_message, created_at, updated_at)
		VALUES (gen_random_uuid(), 'RosterCacheInvalidation', '{}'::jsonb, 'pending', NULL, NOW(), NOW())`)
	return err
}

// EnqueueScheduleOfficeCoverageRefresh creates one durable employment/day unit of work.
func EnqueueScheduleOfficeCoverageRefresh(ctx context.Context, db DBTX, event ScheduleOfficeCoverageEvent) (bool, error) {
	if db == nil {
		return false, errors.New("outbox database is required")
	}
	if err := validateScheduleOfficeEvent(event); err != nil {
		return false, err
	}
	payload, err := json.Marshal(event)
	if err != nil {
		return false, fmt.Errorf("marshal schedule office event: %w", err)
	}
	dedupeKey := fmt.Sprintf("schedule-office:%s:%s:%s:%s", event.ApprovalInstanceID, event.EmploymentID, event.BusinessDate, event.Mode)
	jobID := uuid.New()
	result, err := db.Exec(ctx, `
		INSERT INTO outbox_jobs (id, job_type, payload, status, error_message, created_at, updated_at, dedupe_key, attempts, available_at)
		VALUES ($1, $2, $3, 'pending', NULL, NOW(), NOW(), $4, 0, NOW())
		ON CONFLICT (dedupe_key) WHERE dedupe_key IS NOT NULL DO NOTHING`, jobID, ScheduleOfficeCoverageRefreshJob, payload, dedupeKey)
	if err != nil {
		return false, mapOutboxSchemaError(err)
	}
	return result.RowsAffected() == 1, nil
}

func validateScheduleOfficeEvent(event ScheduleOfficeCoverageEvent) error {
	if event.ApprovalInstanceID == uuid.Nil || event.EmployerID == uuid.Nil || event.EmploymentID == uuid.Nil {
		return errors.New("schedule office event IDs are required")
	}
	if event.Mode != "office" {
		return errors.New("schedule office event mode must be office")
	}
	if _, err := time.Parse("2006-01-02", event.BusinessDate); err != nil {
		return fmt.Errorf("schedule office event business date: %w", err)
	}
	return nil
}

// ClaimScheduleOfficeJob leases one due job and commits before domain publication.
func ClaimScheduleOfficeJob(ctx context.Context, db DBTX, leaseDuration time.Duration) (id string, payload []byte, attempt int, leaseToken string, found bool, err error) {
	if db == nil {
		return "", nil, 0, "", false, errors.New("outbox database is required")
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return "", nil, 0, "", false, errors.New("outbox database does not support transactions")
	}
	if leaseDuration <= 0 {
		return "", nil, 0, "", false, errors.New("outbox lease duration must be positive")
	}
	var jobID uuid.UUID
	var jobPayload []byte
	var jobAttempt int
	lease := uuid.New()
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if _, err := tx.Exec(ctx, `
			UPDATE outbox_jobs
			SET status = 'dead_letter', dead_letter_at = NOW(), updated_at = NOW()
			WHERE job_type = $1 AND status IN ('pending', 'processing') AND attempts >= $2
			  AND (status = 'pending' OR locked_until IS NULL OR locked_until <= NOW())`, ScheduleOfficeCoverageRefreshJob, maxScheduleOfficeAttempts); err != nil {
			return mapOutboxSchemaError(err)
		}
		row := tx.QueryRow(ctx, `
			SELECT id, payload, attempts
			FROM outbox_jobs
			WHERE job_type = $1 AND attempts < $2
			  AND ((status = 'pending' AND available_at <= NOW())
			    OR (status = 'processing' AND locked_until IS NOT NULL AND locked_until <= NOW()))
			ORDER BY available_at, id
			LIMIT 1
			FOR UPDATE SKIP LOCKED`, ScheduleOfficeCoverageRefreshJob, maxScheduleOfficeAttempts)
		if err := row.Scan(&jobID, &jobPayload, &jobAttempt); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return nil
			}
			return mapOutboxSchemaError(err)
		}
		jobAttempt++
		if _, err := tx.Exec(ctx, `UPDATE outbox_jobs SET status = 'processing', attempts = $2, lease_token = $3, locked_until = NOW() + ($4 * INTERVAL '1 second'), updated_at = NOW() WHERE id = $1`, jobID, jobAttempt, lease, leaseDuration.Seconds()); err != nil {
			return mapOutboxSchemaError(err)
		}
		return nil
	})
	if err != nil {
		return "", nil, 0, "", false, err
	}
	if jobID == uuid.Nil {
		return "", nil, 0, "", false, nil
	}
	return jobID.String(), jobPayload, jobAttempt, lease.String(), true, nil
}

func HeartbeatScheduleOfficeJob(ctx context.Context, db DBTX, jobID, leaseToken string, leaseDuration time.Duration) error {
	id, token, err := parseLease(jobID, leaseToken)
	if err != nil {
		return err
	}
	result, err := db.Exec(ctx, `UPDATE outbox_jobs SET locked_until = NOW() + ($3 * INTERVAL '1 second'), updated_at = NOW() WHERE id = $1 AND status = 'processing' AND lease_token = $2`, id, token, leaseDuration.Seconds())
	if err != nil {
		return mapOutboxSchemaError(err)
	}
	if result.RowsAffected() != 1 {
		return ErrOutboxLeaseLost
	}
	return nil
}

func CompleteScheduleOfficeJob(ctx context.Context, db DBTX, jobID, leaseToken, batchID string) error {
	id, token, err := parseLease(jobID, leaseToken)
	if err != nil {
		return err
	}
	batch, err := uuid.Parse(batchID)
	if err != nil {
		return fmt.Errorf("batch ID: %w", err)
	}
	result, err := db.Exec(ctx, `UPDATE outbox_jobs SET status = 'completed', payload = jsonb_set(payload, '{batch_id}', to_jsonb($3::text), true), lease_token = NULL, locked_until = NULL, updated_at = NOW() WHERE id = $1 AND status = 'processing' AND lease_token = $2`, id, token, batch.String())
	if err != nil {
		return mapOutboxSchemaError(err)
	}
	if result.RowsAffected() != 1 {
		return ErrOutboxLeaseLost
	}
	return nil
}

func FailScheduleOfficeJob(ctx context.Context, db DBTX, jobID, leaseToken string, jobAttempt int, jobErr error) error {
	id, token, err := parseLease(jobID, leaseToken)
	if err != nil {
		return err
	}
	if jobErr == nil {
		return errors.New("outbox failure requires error")
	}
	if jobAttempt < 1 || jobAttempt > maxScheduleOfficeAttempts {
		return errors.New("outbox attempt is outside supported range")
	}
	result, err := db.Exec(ctx, `
		UPDATE outbox_jobs
		SET status = CASE WHEN $3 >= $4 THEN 'dead_letter' ELSE 'pending' END,
		    available_at = CASE WHEN $3 >= $4 THEN available_at ELSE NOW() + (CASE $3 WHEN 1 THEN INTERVAL '1 minute' WHEN 2 THEN INTERVAL '5 minutes' WHEN 3 THEN INTERVAL '15 minutes' ELSE INTERVAL '1 hour' END) END,
		    dead_letter_at = CASE WHEN $3 >= $4 THEN NOW() ELSE NULL END,
		    last_error = $5, lease_token = NULL, locked_until = NULL, updated_at = NOW()
		WHERE id = $1 AND status = 'processing' AND lease_token = $2`, id, token, jobAttempt, maxScheduleOfficeAttempts, jobErr.Error())
	if err != nil {
		return mapOutboxSchemaError(err)
	}
	if result.RowsAffected() != 1 {
		return ErrOutboxLeaseLost
	}
	return nil
}

func parseLease(jobID, leaseToken string) (uuid.UUID, uuid.UUID, error) {
	id, err := uuid.Parse(jobID)
	if err != nil {
		return uuid.Nil, uuid.Nil, fmt.Errorf("job ID: %w", err)
	}
	token, err := uuid.Parse(leaseToken)
	if err != nil {
		return uuid.Nil, uuid.Nil, fmt.Errorf("lease token: %w", err)
	}
	return id, token, nil
}

func mapOutboxSchemaError(err error) error {
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) && (pgErr.Code == "42703" || pgErr.Code == "42P01" || pgErr.Code == "42704") {
		return fmt.Errorf("%w: %s", ErrOutboxSchemaUnavailable, pgErr.Message)
	}
	return err
}
