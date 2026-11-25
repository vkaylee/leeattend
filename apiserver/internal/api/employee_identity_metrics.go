package api

import (
	"context"
	"errors"
	"log/slog"
	"sync/atomic"
	"time"

	"github.com/jackc/pgx/v5"
)

// EmployeeIdentityMetrics tracks aggregate-only counts and emits PII-free
// latency/outcome logs for employee identity matching and atomic write paths.
type EmployeeIdentityMetrics struct {
	matchTierNone  atomic.Int64
	matchTierSoft  atomic.Int64
	matchTierHard  atomic.Int64
	matchError     atomic.Int64
	createConflict atomic.Int64
	createRollback atomic.Int64
	tenureConflict atomic.Int64
	tenureRollback atomic.Int64
	logger         *slog.Logger
}

// EmployeeIdentityMetricsSnapshot holds a point-in-time aggregate snapshot.
type EmployeeIdentityMetricsSnapshot struct {
	MatchTierNone  int64
	MatchTierSoft  int64
	MatchTierHard  int64
	MatchError     int64
	CreateConflict int64
	CreateRollback int64
	TenureConflict int64
	TenureRollback int64
}

var defaultEmployeeIdentityMetrics = &EmployeeIdentityMetrics{}

// NewEmployeeIdentityMetrics creates an isolated metrics recorder (optionally with a custom logger).
func NewEmployeeIdentityMetrics(logger *slog.Logger) *EmployeeIdentityMetrics {
	return &EmployeeIdentityMetrics{logger: logger}
}

func (m *EmployeeIdentityMetrics) resolve() *EmployeeIdentityMetrics {
	if m != nil {
		return m
	}
	return defaultEmployeeIdentityMetrics
}

func (m *EmployeeIdentityMetrics) log() *slog.Logger {
	if m.logger != nil {
		return m.logger
	}
	return slog.Default()
}

// Snapshot returns aggregate counters only.
func (m *EmployeeIdentityMetrics) Snapshot() EmployeeIdentityMetricsSnapshot {
	target := m.resolve()
	return EmployeeIdentityMetricsSnapshot{
		MatchTierNone:  target.matchTierNone.Load(),
		MatchTierSoft:  target.matchTierSoft.Load(),
		MatchTierHard:  target.matchTierHard.Load(),
		MatchError:     target.matchError.Load(),
		CreateConflict: target.createConflict.Load(),
		CreateRollback: target.createRollback.Load(),
		TenureConflict: target.tenureConflict.Load(),
		TenureRollback: target.tenureRollback.Load(),
	}
}

// ObserveMatch records an aggregate match tier and latency without PII.
func (m *EmployeeIdentityMetrics) ObserveMatch(ctx context.Context, tier string, candidateCount int, duration time.Duration) {
	target := m.resolve()
	switch tier {
	case MatchTierHard:
		target.matchTierHard.Add(1)
	case MatchTierSoft:
		target.matchTierSoft.Add(1)
	case MatchTierNone:
		target.matchTierNone.Add(1)
	default:
		target.matchError.Add(1)
	}
	target.log().InfoContext(ctx, "employee_identity_metric",
		slog.String("operation", "match"),
		slog.String("outcome", tier),
		slog.Int("candidate_count", candidateCount),
		slog.Int64("latency_ms", duration.Milliseconds()),
	)
}

// ObserveWrite records aggregate conflict/rollback/success outcome and latency for create or reuse-tenure.
func (m *EmployeeIdentityMetrics) ObserveWrite(ctx context.Context, operation string, err error, duration time.Duration) {
	target := m.resolve()
	outcome := classifyWriteOutcome(operation, err, target)
	target.log().InfoContext(ctx, "employee_identity_metric",
		slog.String("operation", operation),
		slog.String("outcome", outcome),
		slog.Int64("latency_ms", duration.Milliseconds()),
	)
}

func classifyWriteOutcome(operation string, err error, target *EmployeeIdentityMetrics) string {
	if err == nil {
		return "created"
	}
	if errors.Is(err, ErrConflict) {
		if operation == "reuse_tenure" {
			target.tenureConflict.Add(1)
		} else {
			target.createConflict.Add(1)
		}
		return "conflict"
	}
	if errors.Is(err, ErrInvalidInput) || errors.Is(err, pgx.ErrNoRows) {
		return "rejected"
	}
	if operation == "reuse_tenure" {
		target.tenureRollback.Add(1)
	} else {
		target.createRollback.Add(1)
	}
	return "rollback"
}
