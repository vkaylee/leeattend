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
	tenureID, relID := uuid.New(), uuid.New()
	mustExec(`INSERT INTO employees (id, full_name) VALUES ($1, 'concurrency-schedule-employee')`, employeeID)
	mustExec(`INSERT INTO employee_tenures (id, employee_id, employee_number, hire_type, start_date) VALUES ($1, $2, $3, 'initial', DATE '2026-01-01')`, tenureID, employeeID, employeeID.String())
	mustExec(`INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01')`, relID, employeeID, employerID, tenureID)
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, relationship_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01'), ($5, $2, $3, $4, 'active', DATE '2026-01-01')`, employmentA, employeeID, employerID, relID, employmentB)

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
