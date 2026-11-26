//go:build integration

package repository

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestIntegrationScheduleEvidencePreservesAmendmentChain(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID := uuid.New(), uuid.New()
	oldBatchID, newBatchID, oldRevisionID, newRevisionID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}

	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'evidence-chain-employer')`, employerID)
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, employeeID.String(), "evidence-chain-employee", "2026-01-01", "2026-11-20", "2026-11-20")
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'evidence-chain-shift')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'evidence-chain-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('a', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'first publication', 'UTC', DATE '2026-11-20', DATE '2026-11-20')`, oldBatchID, oldBatchID.String(), actorID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('b', 64), 'rotating', 'published', $3, 'replan', '{}'::jsonb, 'amendment', 'UTC', DATE '2026-11-20', DATE '2026-11-20')`, newBatchID, newBatchID.String(), actorID)

	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-20', 'UTC', 1, 'draft', $4, 'pattern', '{}'::jsonb, 'schedule-resolver-v1', 'first publication', $5, 'human', NULL)`, oldRevisionID, employmentID, employerID, oldBatchID, actorID)
	mustExec(`SELECT set_config('schedule.lifecycle_publication', 'on', true)`)
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, DATE '2026-11-20', 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'pattern', '{}'::jsonb, '{}'::jsonb, $2, $3, $4, TIMESTAMPTZ '2026-11-20 08:00:00+00', TIMESTAMPTZ '2026-11-20 17:00:00+00', '[]'::jsonb)`, employmentID, templateID, oldRevisionID, versionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, oldRevisionID)

	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, previous_revision_id, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-20', 'UTC', 2, 'draft', $4, $5, 'manual', '{}'::jsonb, 'schedule-resolver-v1', 'amendment', $6, 'human', NULL)`, newRevisionID, employmentID, employerID, oldRevisionID, newBatchID, actorID)
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, DATE '2026-11-20', 0, '09:00', '18:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $2, $3, $4, TIMESTAMPTZ '2026-11-20 09:00:00+00', TIMESTAMPTZ '2026-11-20 18:00:00+00', '[]'::jsonb)`, employmentID, templateID, newRevisionID, versionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'superseded', published_at = COALESCE(published_at, CURRENT_TIMESTAMP) WHERE id = $1`, oldRevisionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, newRevisionID)

	evidence, err := (ScheduleRepository{DB: pool}).GetScheduleEvidence(ctx, actorID, employmentID, "2026-11-20")
	if err != nil {
		t.Fatalf("get schedule evidence: %v", err)
	}
	if len(evidence.Revisions) != 2 || len(evidence.Segments) != 2 || len(evidence.DiffChain) != 2 {
		t.Fatalf("evidence counts revisions=%d segments=%d diff_chain=%d", len(evidence.Revisions), len(evidence.Segments), len(evidence.DiffChain))
	}
	if evidence.Revisions[1].PreviousRevisionID == nil || *evidence.Revisions[1].PreviousRevisionID != oldRevisionID {
		t.Fatalf("replacement revision lost lineage: %+v", evidence.Revisions)
	}
	if evidence.DiffChain[1].PreviousRevisionID == nil || *evidence.DiffChain[1].PreviousRevisionID != oldRevisionID {
		t.Fatalf("evidence diff chain lost lineage: %+v", evidence.DiffChain)
	}
}

func TestIntegrationResolveAuthorizedScheduleEmploymentsEnforcesRoleScope(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin tx: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	employerID := uuid.New()
	deptA, deptB := uuid.New(), uuid.New()
	empAUser, empA, empAEmployment := uuid.New(), uuid.New(), uuid.New()
	leadUser, leadEmp, leadEmployment := uuid.New(), uuid.New(), uuid.New()
	empBUser, empB, empBEmployment := uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := tx.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture exec failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'scope-authz-employer')`, employerID)
	mustExec(`INSERT INTO departments (id, name) VALUES ($1, 'Dept A'), ($2, 'Dept B')`, deptA, deptB)
	for _, u := range []uuid.UUID{empAUser, leadUser, empBUser} {
		mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'hash')`, u, u.String()+"@invalid.test")
	}
	seedScopedEmp := func(userID, employeeID, employmentID, deptID uuid.UUID, name string) {
		tenureID, relID := uuid.New(), uuid.New()
		mustExec(`INSERT INTO employees (id, full_name) VALUES ($1, $2)`, employeeID, name)
		mustExec(`INSERT INTO employee_tenures (id, employee_id, user_id, employee_number, start_date) VALUES ($1, $2, $3, $4, DATE '2026-01-01')`, tenureID, employeeID, userID, employeeID.String())
		mustExec(`INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01')`, relID, employeeID, employerID, tenureID)
		mustExec(`INSERT INTO employments (id, employee_id, employer_id, relationship_id, department_id, status, start_date) VALUES ($1, $2, $3, $4, $5, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID, relID, deptID)
	}
	seedScopedEmp(empAUser, empA, empAEmployment, deptA, "Employee A")
	seedScopedEmp(leadUser, leadEmp, leadEmployment, deptA, "Lead A")
	seedScopedEmp(empBUser, empB, empBEmployment, deptB, "Employee B")
	mustExec(`UPDATE departments SET manager_id = $1 WHERE id = $2`, leadEmp, deptA)

	repo := ScheduleRepository{DB: tx}
	start, end := time.Date(2026, 11, 1, 0, 0, 0, 0, time.UTC), time.Date(2026, 11, 30, 0, 0, 0, 0, time.UTC)

	// Employee A (includeManagedDepartments=false): sees only own employment; denied requesting Lead A or Employee B.
	ids, ok, err := repo.ResolveAuthorizedScheduleEmployments(ctx, empAUser, nil, start, end, false)
	if err != nil || !ok || len(ids) != 1 || ids[0] != empAEmployment {
		t.Fatalf("employee own scope ids=%v ok=%v err=%v, want [%s]", ids, ok, err, empAEmployment)
	}
	if _, ok, err := repo.ResolveAuthorizedScheduleEmployments(ctx, empAUser, []uuid.UUID{empBEmployment}, start, end, false); err != nil || ok {
		t.Fatalf("employee cross-employment must be denied: ok=%v err=%v", ok, err)
	}

	// Lead A (includeManagedDepartments=true): sees Dept A (own + Employee A); denied Dept B (Employee B).
	ids, ok, err = repo.ResolveAuthorizedScheduleEmployments(ctx, leadUser, nil, start, end, true)
	if err != nil || !ok || len(ids) != 2 {
		t.Fatalf("lead dept scope ids=%v ok=%v err=%v, want 2 Dept A employments", ids, ok, err)
	}
	if _, ok, err := repo.ResolveAuthorizedScheduleEmployments(ctx, leadUser, []uuid.UUID{empAEmployment}, start, end, true); err != nil || !ok {
		t.Fatalf("lead reading same-dept employee must succeed: ok=%v err=%v", ok, err)
	}
	if _, ok, err := repo.ResolveAuthorizedScheduleEmployments(ctx, leadUser, []uuid.UUID{empBEmployment}, start, end, true); err != nil || ok {
		t.Fatalf("lead reading other-dept employee must be denied: ok=%v err=%v", ok, err)
	}

	// Dual-role check: a restricted global role (without schedules:read) + a scoped role (with schedules:read) must NOT grant global schedule read.
	restrictedGlobalRole, scopedReadRole, fullGlobalRole := uuid.New(), uuid.New(), uuid.New()
	mustExec(`INSERT INTO roles (id, name, scope_type) VALUES ($1, $2, 'global'), ($3, $4, 'scoped'), ($5, $6, 'global')`,
		restrictedGlobalRole, "restricted-global-"+restrictedGlobalRole.String()[:8],
		scopedReadRole, "scoped-read-"+scopedReadRole.String()[:8],
		fullGlobalRole, "full-global-"+fullGlobalRole.String()[:8])
	mustExec(`INSERT INTO role_permissions (role_id, permission_name) VALUES ($1, 'schedules:read'), ($2, 'schedules:read')`, scopedReadRole, fullGlobalRole)
	mustExec(`INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2), ($3, $4)`, empAUser, restrictedGlobalRole, empBUser, fullGlobalRole)
	mustExec(`INSERT INTO user_extra_roles (user_id, role_id, expires_at) VALUES ($1, $2, now() + interval '1 day')`, empAUser, scopedReadRole)

	isGlobal, err := repo.HasGlobalScheduleRead(ctx, empAUser)
	if err != nil || isGlobal {
		t.Fatalf("restricted global role + scoped read role must not grant global schedule read: isGlobal=%v err=%v", isGlobal, err)
	}
	isGlobal, err = repo.HasGlobalScheduleRead(ctx, empBUser)
	if err != nil || !isGlobal {
		t.Fatalf("global role with schedules:read must grant global schedule read: isGlobal=%v err=%v", isGlobal, err)
	}
}
