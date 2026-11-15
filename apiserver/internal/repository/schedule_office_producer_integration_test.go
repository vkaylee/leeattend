//go:build integration

package repository

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestIntegrationScheduleOfficeProducerFiltersCoveredDaysAndApprovedPolicy(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}

	approvedEmployer, rejectedEmployer := uuid.New(), uuid.New()
	approvedEmployee, rejectedEmployee := uuid.New(), uuid.New()
	approvedEmployment, rejectedEmployment := uuid.New(), uuid.New()
	approvedInstance, rejectedInstance := uuid.New(), uuid.New()
	createdBy := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, createdBy, createdBy.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert producer actor: %v", err)
	}
	for _, employer := range []uuid.UUID{approvedEmployer, rejectedEmployer} {
		if _, err := tx.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'office-producer-employer')`, employer); err != nil {
			t.Fatalf("insert employer: %v", err)
		}
	}
	for _, employee := range []uuid.UUID{approvedEmployee, rejectedEmployee} {
		if _, err := tx.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'office-producer-employee', DATE '2026-01-01', DATE '2026-01-01')`, employee, employee.String()); err != nil {
			t.Fatalf("insert employee: %v", err)
		}
	}
	if _, err := tx.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, status, start_date, end_date) VALUES ($1, $2, $3, 'active', DATE '2026-11-15', DATE '2026-11-30'), ($4, $5, $6, 'active', DATE '2026-11-15', DATE '2026-11-30')`, approvedEmployment, approvedEmployee, approvedEmployer, rejectedEmployment, rejectedEmployee, rejectedEmployer); err != nil {
		t.Fatalf("insert employments: %v", err)
	}
	for _, instance := range []uuid.UUID{approvedInstance, rejectedInstance} {
		if _, err := tx.Exec(ctx, `INSERT INTO approval_workflow_instances (id, entity_type, entity_id, status) VALUES ($1, 'schedule_policy', $1, $2)`, instance, map[bool]string{true: "approved", false: "pending"}[instance == approvedInstance]); err != nil {
			t.Fatalf("insert approval instance: %v", err)
		}
	}
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_office_policies (id, employer_id, approval_instance_id, effective_from, effective_to, status, created_by) VALUES ($1, $2, $3, DATE '2026-11-15', DATE '2026-11-30', 'active', $4), ($5, $6, $7, DATE '2026-11-15', DATE '2026-11-30', 'active', $4)`, uuid.New(), approvedEmployer, approvedInstance, createdBy, uuid.New(), rejectedEmployer, rejectedInstance); err != nil {
		t.Fatalf("insert office policies: %v", err)
	}
	if err := tx.Commit(ctx); err != nil {
		t.Fatalf("commit producer fixture: %v", err)
	}
	source := ApprovedScheduleOfficePolicySource{}
	now := time.Date(2026, 11, 20, 10, 0, 0, 0, time.UTC)
	approvedEmployers, err := source.ApprovedEmployers(ctx, pool, now)
	if err != nil {
		t.Fatalf("approved employers: %v", err)
	}
	if len(approvedEmployers) != 1 || approvedEmployers[0] != approvedEmployer {
		t.Fatalf("approved employers = %v, want [%s]", approvedEmployers, approvedEmployer)
	}
	start, end := horizon(now)
	candidates, err := source.Coverage(ctx, pool, approvedEmployer, start, end)
	if err != nil {
		t.Fatalf("office coverage: %v", err)
	}
	if len(candidates) != 11 {
		t.Fatalf("coverage candidates = %d, want 11 employment/day candidates", len(candidates))
	}
	for _, candidate := range candidates {
		if candidate.EmployerID != approvedEmployer || candidate.EmploymentID != approvedEmployment || candidate.Mode != "office" {
			t.Fatalf("unexpected office candidate: %+v", candidate)
		}
	}
}
