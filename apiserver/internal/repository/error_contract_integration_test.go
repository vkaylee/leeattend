//go:build integration

package repository

import (
	"errors"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationMonitorErrorContract(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	repo := MonitorRepository{SystemDB: pool, DB: pool}
	missingID := uuid.New()

	if _, _, err := repo.GetAlert(ctx, missingID); !errors.Is(err, api.ErrMonitorNotFound) {
		t.Fatalf("GetAlert missing err=%v, want api.ErrMonitorNotFound", err)
	}
	if _, err := repo.MutateAlert(ctx, missingID, "acknowledge", "investigating", uuid.New()); !errors.Is(err, api.ErrMonitorNotFound) {
		t.Fatalf("MutateAlert missing err=%v, want api.ErrMonitorNotFound", err)
	}
	if _, err := repo.GetDurableJob(ctx, missingID); !errors.Is(err, api.ErrMonitorNotFound) {
		t.Fatalf("GetDurableJob missing err=%v, want api.ErrMonitorNotFound", err)
	}
}

func TestIntegrationReferenceAndValidationErrorContract(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID := uuid.New()
	employeeID := uuid.New()
	employmentID := uuid.New()
	deptID := uuid.New()

	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'err-contract-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO departments (id, name) VALUES ($1, 'err-contract-dept')`, deptID); err != nil {
		t.Fatalf("insert department: %v", err)
	}
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, "EC-"+employeeID.String()[:8], "Error Contract Employee", "2026-06-01", "2026-06-01", "")
	t.Cleanup(func() {
		_, _ = pool.Exec(ctx, `DELETE FROM employments WHERE id = $1`, employmentID)
		_, _ = pool.Exec(ctx, `DELETE FROM employment_relationships WHERE employee_id = $1`, employeeID)
		_, _ = pool.Exec(ctx, `DELETE FROM employee_tenures WHERE employee_id = $1`, employeeID)
		_, _ = pool.Exec(ctx, `DELETE FROM employees WHERE id = $1`, employeeID)
		_, _ = pool.Exec(ctx, `DELETE FROM departments WHERE id = $1`, deptID)
		_, _ = pool.Exec(ctx, `DELETE FROM employers WHERE id = $1`, employerID)
	})

	t.Run("work segment missing employment returns ErrEmploymentNotFound", func(t *testing.T) {
		_, err := (WorkSegmentRepository{DB: pool}).CreateSegment(ctx, api.CreateWorkSegmentRequest{
			EmploymentID:   uuid.New(),
			Date:           "2026-06-15",
			TemplateID:     uuid.New(),
			ScheduledStart: "08:00",
			ScheduledEnd:   "17:00",
			SegmentType:    "work",
			SegmentStatus:  "scheduled",
		})
		if !errors.Is(err, api.ErrEmploymentNotFound) {
			t.Fatalf("got %v, want api.ErrEmploymentNotFound", err)
		}
	})

	t.Run("work segment missing shift returns ErrShiftNotFound", func(t *testing.T) {
		_, err := (WorkSegmentRepository{DB: pool}).CreateSegment(ctx, api.CreateWorkSegmentRequest{
			EmploymentID:   employmentID,
			Date:           "2026-06-15",
			TemplateID:     uuid.New(),
			ScheduledStart: "08:00",
			ScheduledEnd:   "17:00",
			SegmentType:    "work",
			SegmentStatus:  "scheduled",
		})
		if !errors.Is(err, api.ErrShiftNotFound) {
			t.Fatalf("got %v, want api.ErrShiftNotFound", err)
		}
	})

	t.Run("working pattern assign missing employment returns ErrEmploymentNotFound", func(t *testing.T) {
		_, err := (WorkingPatternRepository{DB: pool}).AssignPattern(ctx, uuid.New(), api.AssignPatternRequest{
			PatternID: uuid.New(),
			StartDate: "2026-06-15",
		})
		if !errors.Is(err, api.ErrEmploymentNotFound) {
			t.Fatalf("got %v, want api.ErrEmploymentNotFound", err)
		}
	})

	t.Run("transfer before employment start returns ErrNoActiveEmployment", func(t *testing.T) {
		_, err := (EmployeeDepartmentRepository{DB: pool}).TransferEmployee(ctx, employeeID, api.TransferEmployeeRequest{
			DepartmentID:  deptID,
			EffectiveDate: "2026-01-01",
		})
		if !errors.Is(err, api.ErrNoActiveEmployment) {
			t.Fatalf("got %v, want api.ErrNoActiveEmployment", err)
		}
	})

	t.Run("terminate employment before start_date returns ErrInvalidInput", func(t *testing.T) {
		_, err := (EmploymentRepository{DB: pool}).TerminateEmployment(ctx, employeeID, employmentID, api.TerminateEmploymentRequest{
			TerminationDate: "2026-05-01",
		})
		if !errors.Is(err, api.ErrInvalidInput) {
			t.Fatalf("got %v, want api.ErrInvalidInput", err)
		}
	})
}

func TestIntegrationLifecycleGuardRollbackAndIndependentEvidence(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID := uuid.New()
	employeeID := uuid.New()
	employmentID := uuid.New()
	actorID := uuid.New()
	batchID := uuid.New()
	revisionID := uuid.New()
	const officialDate = "2026-11-22"

	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'lc-evidence-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, "LC-"+employeeID.String()[:8], "Lifecycle Evidence Employee", "2026-01-01", "2026-01-01", "")
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert user: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('b', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'evidence integration test', 'UTC', $4::date, $4::date)`, batchID, batchID.String(), actorID, officialDate); err != nil {
		t.Fatalf("insert batch: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type) VALUES ($1, $2, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'evidence test', $6, 'human')`, revisionID, employmentID, employerID, officialDate, batchID, actorID); err != nil {
		t.Fatalf("insert revision: %v", err)
	}
	if _, err := pool.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = NOW() WHERE id = $1`, revisionID); err != nil {
		t.Fatalf("publish revision: %v", err)
	}
	t.Cleanup(func() {
		_, _ = pool.Exec(ctx, `DELETE FROM schedule_legacy_mutation_evidence WHERE employment_id = $1`, employmentID)
		_, _ = pool.Exec(ctx, `DELETE FROM shift_rosters WHERE employment_id = $1`, employmentID)
		_, _ = pool.Exec(ctx, `DELETE FROM schedule_day_revisions WHERE id = $1`, revisionID)
		_, _ = pool.Exec(ctx, `DELETE FROM schedule_publication_batches WHERE id = $1`, batchID)
		_, _ = pool.Exec(ctx, `DELETE FROM employments WHERE id = $1`, employmentID)
		_, _ = pool.Exec(ctx, `DELETE FROM employment_relationships WHERE employee_id = $1`, employeeID)
		_, _ = pool.Exec(ctx, `DELETE FROM employee_tenures WHERE employee_id = $1`, employeeID)
		_, _ = pool.Exec(ctx, `DELETE FROM employees WHERE id = $1`, employeeID)
		_, _ = pool.Exec(ctx, `DELETE FROM users WHERE id = $1`, actorID)
		_, _ = pool.Exec(ctx, `DELETE FROM employers WHERE id = $1`, employerID)
	})

	_, err := (RosterRepository{DB: pool}).UpsertRoster(ctx, api.CreateRosterRequest{
		EmploymentID: employmentID,
		Date:         officialDate,
	})
	if !errors.Is(err, api.ErrLifecycleMutationBlocked) {
		t.Fatalf("UpsertRoster err=%v, want api.ErrLifecycleMutationBlocked", err)
	}

	var rosterCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM shift_rosters WHERE employment_id = $1 AND date = $2::date`, employmentID, officialDate).Scan(&rosterCount); err != nil {
		t.Fatalf("count rosters: %v", err)
	}
	if rosterCount != 0 {
		t.Fatalf("rosterCount=%d after guard block, want 0 (rollback)", rosterCount)
	}

	var evidenceCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_legacy_mutation_evidence WHERE employment_id = $1 AND business_date = $2::date AND operation = 'guard_blocked'`, employmentID, officialDate).Scan(&evidenceCount); err != nil {
		t.Fatalf("count evidence: %v", err)
	}
	if evidenceCount < 1 {
		t.Fatalf("evidenceCount=%d, want >= 1 committed in independent transaction", evidenceCount)
	}
}
