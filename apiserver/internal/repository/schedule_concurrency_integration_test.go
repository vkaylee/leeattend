//go:build integration

package repository

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestIntegrationScheduleLocksSerializeCrossEmploymentSameEmployee(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID := uuid.New(), uuid.New()
	employmentA, employmentB := uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'concurrency-schedule-employer')`, employerID)
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'concurrency-schedule-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01'), ($4, $2, $3, 'active', DATE '2026-01-01')`, employmentA, employeeID, employerID, employmentB)

	first, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin first lock transaction: %v", err)
	}
	second, err := pool.Begin(ctx)
	if err != nil {
		_ = first.Rollback(ctx)
		t.Fatalf("begin second lock transaction: %v", err)
	}
	defer func() {
		_ = first.Rollback(ctx)
		_ = second.Rollback(ctx)
	}()

	if err := lockScheduleRevisionTargets(ctx, first, []scheduleRevisionLockKey{{employmentID: employmentA, businessDate: "2026-11-20"}}); err != nil {
		t.Fatalf("first lock: %v", err)
	}

	secondDone := make(chan error, 1)
	secondCtx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	go func() {
		secondDone <- lockScheduleRevisionTargets(secondCtx, second, []scheduleRevisionLockKey{{employmentID: employmentB, businessDate: "2026-11-20"}})
	}()

	select {
	case err := <-secondDone:
		t.Fatalf("second lock completed before first commit: %v", err)
	case <-time.After(100 * time.Millisecond):
	}
	if err := first.Commit(ctx); err != nil {
		t.Fatalf("commit first lock transaction: %v", err)
	}
	if err := <-secondDone; err != nil {
		t.Fatalf("second lock after first commit: %v", err)
	}
}
