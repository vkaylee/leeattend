//go:build integration

package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

type legacyMutationGuardFixture struct {
	activeEmployment    uuid.UUID
	inactiveEmployment  uuid.UUID
	unrelatedEmployment uuid.UUID
	activePattern       uuid.UUID
	inactivePattern     uuid.UUID
	activeTemplate      uuid.UUID
	activeVersion       uuid.UUID
	actorID             uuid.UUID
	officialDate        string
}

func TestIntegrationLegacyScheduleMutationGuardsRejectAllEightSources(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	fixture := createLegacyMutationGuardFixture(t, ctx, tx)
	mutations := []struct {
		name string
		stmt string
		args func() []any
	}{
		{
			name: "shift_rosters",
			stmt: `INSERT INTO shift_rosters (id, employment_id, template_id, date, status, notes) VALUES ($1, $2, $3, $4::date, 'confirmed', 'guard test')`,
			args: func() []any {
				return []any{uuid.New(), fixture.activeEmployment, fixture.activeTemplate, fixture.officialDate}
			},
		},
		{
			name: "employee_shifts",
			stmt: `INSERT INTO employee_shifts (id, employment_id, template_id, start_date, end_date) VALUES ($1, $2, $3, $4::date, $4::date)`,
			args: func() []any {
				return []any{uuid.New(), fixture.activeEmployment, fixture.activeTemplate, fixture.officialDate}
			},
		},
		{
			name: "employee_working_patterns",
			stmt: `INSERT INTO employee_working_patterns (id, employment_id, pattern_id, start_date, end_date, anchor_date) VALUES ($1, $2, $3, $4::date, $4::date, $4::date)`,
			args: func() []any {
				return []any{uuid.New(), fixture.activeEmployment, fixture.activePattern, fixture.officialDate}
			},
		},
		{
			name: "working_pattern_details",
			stmt: `INSERT INTO working_pattern_details (id, pattern_id, schedule_day, template_id) VALUES ($1, $2, 6, $3)`,
			args: func() []any {
				return []any{uuid.New(), fixture.activePattern, fixture.activeTemplate}
			},
		},
		{
			name: "shift_versions",
			stmt: `INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 99, 'blocked version', $3::date, 'revise', $4)`,
			args: func() []any {
				return []any{uuid.New(), fixture.activeTemplate, fixture.officialDate, fixture.actorID}
			},
		},
		{
			name: "shift_templates",
			stmt: `UPDATE shift_templates SET name = 'blocked template' WHERE id = $1`,
			args: func() []any { return []any{fixture.activeTemplate} },
		},
		{
			name: "assignment_rules",
			stmt: `INSERT INTO assignment_rules (id, pattern_id, name, priority, conditions) VALUES ($1, $2, 'blocked rule', 1, '{}'::jsonb)`,
			args: func() []any { return []any{uuid.New(), fixture.activePattern} },
		},
		{
			name: "working_patterns",
			stmt: `UPDATE working_patterns SET name = 'blocked pattern' WHERE id = $1`,
			args: func() []any { return []any{fixture.activePattern} },
		},
	}

	for _, mutation := range mutations {
		t.Run(mutation.name, func(t *testing.T) {
			assertLegacyMutationBlocked(t, ctx, tx, mutation.stmt, mutation.args()...)
		})
	}
}

func TestIntegrationLegacyScheduleMutationGuardAllowsInactiveAndUnrelatedScopes(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	fixture := createLegacyMutationGuardFixture(t, ctx, tx)

	t.Run("inactive employer assignment rule", func(t *testing.T) {
		assertLegacyMutationAllowed(t, ctx, tx,
			`INSERT INTO assignment_rules (id, pattern_id, name, priority, conditions) VALUES ($1, $2, 'inactive rule', 1, '{}'::jsonb)`,
			uuid.New(), fixture.inactivePattern,
		)
	})
	t.Run("unrelated employment roster", func(t *testing.T) {
		assertLegacyMutationAllowed(t, ctx, tx,
			`INSERT INTO shift_rosters (id, employment_id, template_id, date, status, notes) VALUES ($1, $2, $3, $4::date, 'confirmed', 'unrelated scope')`,
			uuid.New(), fixture.unrelatedEmployment, fixture.activeTemplate, fixture.officialDate,
		)
	})
}

func TestIntegrationLegacyMutationGuardRollbackRefusesOfficialEvidence(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	createLegacyMutationGuardFixture(t, ctx, tx)
	var officialCount int
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM schedule_day_revisions WHERE state IN ('published', 'explicit_off')`).Scan(&officialCount); err != nil {
		t.Fatalf("count official revisions: %v", err)
	}
	if officialCount < 2 {
		t.Fatalf("official revision count = %d, want at least 2", officialCount)
	}

	if _, err := tx.Exec(ctx, "SAVEPOINT rollback_guard"); err != nil {
		t.Fatalf("create rollback savepoint: %v", err)
	}
	_, err = tx.Exec(ctx, `DO $$ BEGIN
		IF EXISTS (SELECT 1 FROM schedule_day_revisions WHERE state IN ('published', 'explicit_off')) THEN
			RAISE EXCEPTION 'legacy mutation guard rollback blocked: official schedule evidence exists';
		END IF;
	END $$;`)
	if err == nil || !strings.Contains(err.Error(), "legacy mutation guard rollback blocked: official schedule evidence exists") {
		_ = rollbackSavepoint(ctx, tx, "rollback_guard")
		t.Fatalf("rollback guard error = %v", err)
	}
	if err := rollbackSavepoint(ctx, tx, "rollback_guard"); err != nil {
		t.Fatalf("rollback rollback guard: %v", err)
	}
}

func createLegacyMutationGuardFixture(t *testing.T, ctx context.Context, tx dbtx) legacyMutationGuardFixture {
	t.Helper()
	fixture := legacyMutationGuardFixture{
		activeEmployment:    uuid.New(),
		inactiveEmployment:  uuid.New(),
		unrelatedEmployment: uuid.New(),
		activePattern:       uuid.New(),
		inactivePattern:     uuid.New(),
		activeTemplate:      uuid.New(),
		activeVersion:       uuid.New(),
		actorID:             uuid.New(),
		officialDate:        "2026-11-20",
	}
	activeEmployer, inactiveEmployer, unrelatedEmployer := uuid.New(), uuid.New(), uuid.New()
	activeEmployee, inactiveEmployee, unrelatedEmployee := uuid.New(), uuid.New(), uuid.New()
	batchID, inactiveBatchID := uuid.New(), uuid.New()
	activeRevision, inactiveRevision := uuid.New(), uuid.New()
	inactivePattern := fixture.inactivePattern

	execFixture := func(query string, args ...any) {
		t.Helper()
		if _, err := tx.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	for _, employer := range []uuid.UUID{activeEmployer, inactiveEmployer, unrelatedEmployer} {
		execFixture(`INSERT INTO employers (id, name) VALUES ($1, $2)`, employer, "guard-test-employer")
	}
	for _, employee := range []struct {
		id   uuid.UUID
		name string
	}{
		{activeEmployee, "guard-test-employee"},
		{inactiveEmployee, "guard-test-employee"},
		{unrelatedEmployee, "guard-test-employee"},
	} {
		execFixture(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, $3, DATE '2026-01-01', DATE '2026-01-01')`, employee.id, employee.id.String(), employee.name)
	}
	execFixture(`INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, fixture.activeEmployment, activeEmployee, activeEmployer)
	execFixture(`INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, fixture.inactiveEmployment, inactiveEmployee, inactiveEmployer)
	execFixture(`INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, fixture.unrelatedEmployment, unrelatedEmployee, unrelatedEmployer)
	execFixture(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, fixture.actorID, fixture.actorID.String()+"@invalid.test")
	execFixture(`INSERT INTO feature_flags (flag_key, enabled, description) VALUES ('schedule_lifecycle', true, 'guard integration test') ON CONFLICT (flag_key) DO UPDATE SET enabled = true`)
	execFixture(`INSERT INTO config_entries (scope_type, scope_id, config_key, value_bool, effective_from) VALUES ('employer', $1, 'schedule.lifecycle.enabled', true, DATE '2026-01-01')`, activeEmployer)
	execFixture(`INSERT INTO working_patterns (id, name, pattern_type) VALUES ($1, 'active pattern', 'weekly')`, fixture.activePattern)
	execFixture(`INSERT INTO working_patterns (id, name, pattern_type) VALUES ($1, 'inactive pattern', 'weekly')`, inactivePattern)
	execFixture(`INSERT INTO shift_templates (id, name, current_version_id) VALUES ($1, 'active template', NULL)`, fixture.activeTemplate)
	execFixture(`INSERT INTO shift_versions (id, template_id, version_number, name, employer_id, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'active version', $3, DATE '2026-01-01', 'create', $4)`, fixture.activeVersion, fixture.activeTemplate, activeEmployer, fixture.actorID)
	execFixture(`UPDATE shift_templates SET current_version_id = $2 WHERE id = $1`, fixture.activeTemplate, fixture.activeVersion)
	execFixture(`INSERT INTO employee_working_patterns (id, employment_id, pattern_id, start_date, end_date, anchor_date) VALUES ($1, $2, $3, DATE '2026-01-01', NULL, DATE '2026-01-01')`, uuid.New(), fixture.activeEmployment, fixture.activePattern)
	execFixture(`INSERT INTO employee_working_patterns (id, employment_id, pattern_id, start_date, end_date, anchor_date) VALUES ($1, $2, $3, DATE '2026-01-01', NULL, DATE '2026-01-01')`, uuid.New(), fixture.inactiveEmployment, inactivePattern)

	insertPublicationBatch := func(id uuid.UUID) {
		execFixture(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('a', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'integration fixture', 'UTC', $4::date, $4::date)`, id, id.String(), fixture.actorID, fixture.officialDate)
	}
	insertPublicationBatch(batchID)
	insertPublicationBatch(inactiveBatchID)
	insertRevision := func(id, employment, employer, batch uuid.UUID) {
		execFixture(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type) VALUES ($1, $2, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'integration fixture', $6, 'human')`, id, employment, employer, fixture.officialDate, batch, fixture.actorID)
	}
	insertRevision(activeRevision, fixture.activeEmployment, activeEmployer, batchID)
	insertRevision(inactiveRevision, fixture.inactiveEmployment, inactiveEmployer, inactiveBatchID)
	execFixture(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2::date, 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $3, $4, $5, TIMESTAMPTZ '2026-11-20 08:00:00+00', TIMESTAMPTZ '2026-11-20 17:00:00+00', '[]'::jsonb)`, fixture.activeEmployment, fixture.officialDate, fixture.activeTemplate, activeRevision, fixture.activeVersion)
	execFixture(`UPDATE schedule_day_revisions SET state = 'published', published_at = TIMESTAMPTZ '2026-11-19 12:00:00+00' WHERE id IN ($1, $2)`, activeRevision, inactiveRevision)
	return fixture
}

func assertLegacyMutationBlocked(t *testing.T, ctx context.Context, tx dbtx, query string, args ...any) {
	t.Helper()
	const savepoint = "legacy_trigger_case"
	if _, err := tx.Exec(ctx, "SAVEPOINT "+savepoint); err != nil {
		t.Fatalf("create savepoint: %v", err)
	}
	_, err := tx.Exec(ctx, query, args...)
	if err == nil || !strings.Contains(err.Error(), "legacy schedule source mutation blocked") {
		_ = rollbackSavepoint(ctx, tx, savepoint)
		t.Fatalf("mutation error = %v, want legacy trigger rejection", err)
	}
	if rollbackErr := rollbackSavepoint(ctx, tx, savepoint); rollbackErr != nil {
		t.Fatalf("rollback blocked mutation: %v", rollbackErr)
	}
}

func assertLegacyMutationAllowed(t *testing.T, ctx context.Context, tx dbtx, query string, args ...any) {
	t.Helper()
	const savepoint = "legacy_trigger_allowed_case"
	if _, err := tx.Exec(ctx, "SAVEPOINT "+savepoint); err != nil {
		t.Fatalf("create savepoint: %v", err)
	}
	if _, err := tx.Exec(ctx, query, args...); err != nil {
		_ = rollbackSavepoint(ctx, tx, savepoint)
		t.Fatalf("allowed mutation failed: %v", err)
	}
	if err := rollbackSavepoint(ctx, tx, savepoint); err != nil {
		t.Fatalf("rollback allowed mutation: %v", err)
	}
}

func rollbackSavepoint(ctx context.Context, tx dbtx, name string) error {
	if _, err := tx.Exec(ctx, "ROLLBACK TO SAVEPOINT "+name); err != nil {
		return errors.Join(err, fmt.Errorf("savepoint %s", name))
	}
	_, err := tx.Exec(ctx, "RELEASE SAVEPOINT "+name)
	return err
}

type dbtx interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
	QueryRow(context.Context, string, ...any) pgx.Row
}
