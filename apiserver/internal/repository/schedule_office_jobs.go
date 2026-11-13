package repository

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"

	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

var ErrScheduleOfficePublisherUnavailable = errors.New("schedule office publisher is unavailable")

type ScheduleOfficeCoverageCandidate struct {
	ApprovalInstanceID uuid.UUID
	EmployerID         uuid.UUID
	EmploymentID       uuid.UUID
	BusinessDate       string
	Mode               string
}

type ScheduleOfficeCoverageSource interface {
	ApprovedEmployers(context.Context, DBTX, time.Time) ([]uuid.UUID, error)
	Coverage(context.Context, DBTX, uuid.UUID, time.Time, time.Time) ([]ScheduleOfficeCoverageCandidate, error)
}

type ScheduleOfficeProducer struct {
	DB     DBTX
	Source ScheduleOfficeCoverageSource
	Now    func() time.Time
}

func (p ScheduleOfficeProducer) Scan(ctx context.Context) (int, error) {
	if p.DB == nil || p.Source == nil {
		return 0, ErrScheduleOfficePolicyUnavailable
	}
	now := time.Now().UTC()
	if p.Now != nil {
		now = p.Now().UTC()
	}
	employers, err := p.Source.ApprovedEmployers(ctx, p.DB, now)
	if err != nil {
		return 0, err
	}
	created := 0
	for _, employerID := range employers {
		count, err := p.scanEmployer(ctx, employerID, now)
		if err != nil {
			return created, err
		}
		created += count
	}
	return created, nil
}

func (p ScheduleOfficeProducer) scanEmployer(ctx context.Context, employerID uuid.UUID, now time.Time) (int, error) {
	beginner, ok := p.DB.(TxBeginner)
	if !ok {
		return 0, errors.New("schedule office producer database does not support transactions")
	}
	created := 0
	err := service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if _, err := tx.Exec(ctx, `SELECT pg_advisory_xact_lock(hashtextextended('schedule-office-producer:' || $1::text, 0))`, employerID); err != nil {
			return err
		}
		start, end := horizon(now)
		candidates, err := p.Source.Coverage(ctx, tx, employerID, start, end)
		if err != nil {
			return err
		}
		for _, candidate := range candidates {
			if candidate.EmployerID != employerID {
				return fmt.Errorf("schedule office candidate employer mismatch: %w", ErrScheduleOfficePolicyUnavailable)
			}
			inserted, err := EnqueueScheduleOfficeCoverageRefresh(ctx, tx, ScheduleOfficeCoverageEvent(candidate))
			if err != nil {
				return err
			}
			if inserted {
				created++
			}
		}
		return nil
	})
	return created, err
}

func horizon(now time.Time) (time.Time, time.Time) {
	local := now.UTC()
	start := time.Date(local.Year(), local.Month(), local.Day(), 0, 0, 0, 0, time.UTC)
	end := time.Date(local.Year(), local.Month()+2, 0, 0, 0, 0, 0, time.UTC)
	return start, end
}

type UnsupportedScheduleOfficePolicySource struct{}

func (UnsupportedScheduleOfficePolicySource) ApprovedEmployers(context.Context, DBTX, time.Time) ([]uuid.UUID, error) {
	return nil, ErrScheduleOfficePolicyUnavailable
}

func (UnsupportedScheduleOfficePolicySource) Coverage(context.Context, DBTX, uuid.UUID, time.Time, time.Time) ([]ScheduleOfficeCoverageCandidate, error) {
	return nil, ErrScheduleOfficePolicyUnavailable
}

type ScheduleOfficeOutboxStore struct{ DB DBTX }

func (s ScheduleOfficeOutboxStore) ClaimScheduleOfficeJob(ctx context.Context, lease time.Duration) (string, []byte, int, string, bool, error) {
	return ClaimScheduleOfficeJob(ctx, s.DB, lease)
}

func (s ScheduleOfficeOutboxStore) HeartbeatScheduleOfficeJob(ctx context.Context, id, token string, lease time.Duration) error {
	return HeartbeatScheduleOfficeJob(ctx, s.DB, id, token, lease)
}

func (s ScheduleOfficeOutboxStore) CompleteScheduleOfficeJob(ctx context.Context, id, token, batch string) error {
	return CompleteScheduleOfficeJob(ctx, s.DB, id, token, batch)
}

func (s ScheduleOfficeOutboxStore) FailScheduleOfficeJob(ctx context.Context, id, token string, attempt int, cause error) error {
	return FailScheduleOfficeJob(ctx, s.DB, id, token, attempt, cause)
}

// ScheduleOfficePublisher fails closed until policy and lifecycle repositories expose
// the approved system-publication service. It never fabricates a batch or revision.
type ScheduleOfficePublisher struct {
	Publisher interface {
		PublishSystemScheduleOffice(context.Context, []byte, string) (string, error)
	}
}

func (p ScheduleOfficePublisher) PublishScheduleOffice(ctx context.Context, payload []byte, jobID string) (string, error) {
	if p.Publisher == nil {
		return "", ErrScheduleOfficePublisherUnavailable
	}
	return p.Publisher.PublishSystemScheduleOffice(ctx, payload, jobID)
}
