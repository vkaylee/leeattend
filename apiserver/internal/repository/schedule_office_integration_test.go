//go:build integration

package repository

import (
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestIntegrationScheduleOfficeOutboxDeduplicatesAndLeases(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	event := ScheduleOfficeCoverageEvent{ApprovalInstanceID: uuid.New(), EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-11-20", Mode: "office"}
	first, err := EnqueueScheduleOfficeCoverageRefresh(ctx, tx, event)
	if err != nil || !first {
		t.Fatalf("first enqueue = inserted:%v err:%v", first, err)
	}
	second, err := EnqueueScheduleOfficeCoverageRefresh(ctx, tx, event)
	if err != nil || second {
		t.Fatalf("duplicate enqueue = inserted:%v err:%v", second, err)
	}
	if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'off', true)`); err != nil {
		t.Fatalf("set transaction config: %v", err)
	}
	if err := tx.Commit(ctx); err != nil {
		t.Fatalf("commit enqueue: %v", err)
	}

	id, payload, attempt, lease, found, err := ClaimScheduleOfficeJob(ctx, pool, time.Minute)
	if err != nil || !found || attempt != 1 || id == "" || len(payload) == 0 || lease == "" {
		t.Fatalf("claim = id:%q attempt:%d lease:%q found:%v err:%v", id, attempt, lease, found, err)
	}
	if err := HeartbeatScheduleOfficeJob(ctx, pool, id, lease, time.Minute); err != nil {
		t.Fatalf("heartbeat: %v", err)
	}
	batchID := uuid.New().String()
	if err := CompleteScheduleOfficeJob(ctx, pool, id, lease, batchID); err != nil {
		t.Fatalf("complete: %v", err)
	}
	var status string
	if err := pool.QueryRow(ctx, `SELECT status FROM outbox_jobs WHERE id = $1`, id).Scan(&status); err != nil {
		t.Fatalf("read completed job: %v", err)
	}
	if status != "completed" {
		t.Fatalf("completed status = %q", status)
	}
}

func TestIntegrationScheduleOfficeOutboxDeadLettersAtFiveAttempts(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	event := ScheduleOfficeCoverageEvent{ApprovalInstanceID: uuid.New(), EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-11-21", Mode: "office"}
	if inserted, err := EnqueueScheduleOfficeCoverageRefresh(ctx, pool, event); err != nil || !inserted {
		t.Fatalf("enqueue = inserted:%v err:%v", inserted, err)
	}
	for expected := 1; expected <= 5; expected++ {
		id, _, attempt, lease, found, err := ClaimScheduleOfficeJob(ctx, pool, time.Nanosecond)
		if err != nil || !found || attempt != expected {
			t.Fatalf("claim %d = id:%q attempt:%d found:%v err:%v", expected, id, attempt, found, err)
		}
		if err := FailScheduleOfficeJob(ctx, pool, id, lease, attempt, errors.New("integration failure")); err != nil {
			t.Fatalf("fail attempt %d: %v", expected, err)
		}
		if expected < 5 {
			if _, err := pool.Exec(ctx, `UPDATE outbox_jobs SET available_at = NOW() - interval '1 second' WHERE id = $1`, id); err != nil {
				t.Fatalf("make retry due: %v", err)
			}
		}
	}
	var status string
	if err := pool.QueryRow(ctx, `SELECT status FROM outbox_jobs WHERE payload->>'business_date' = '2026-11-21'`).Scan(&status); err != nil {
		t.Fatalf("read dead letter: %v", err)
	}
	if status != "dead_letter" {
		t.Fatalf("final status = %q", status)
	}
}
