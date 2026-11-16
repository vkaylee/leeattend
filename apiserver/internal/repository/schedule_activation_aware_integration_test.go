//go:build integration

package repository

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgconn"
)

func TestIntegrationActivationAwareScheduleReaderRoutesByActivatedEmploymentDays(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	repo := ScheduleRepository{DB: pool}
	actorID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert actor: %v", err)
	}

	allEmployer, allEmployment := insertActivationAwareEmployment(t, pool, ctx, "2099-01-01", "2099-01-02")
	insertActivation(t, pool, ctx, allEmployer, "2099-01-01", "2099-01-02")
	insertPublishedRevision(t, pool, ctx, actorID, allEmployment, allEmployer, "2099-01-01", 1)
	insertPublishedRevision(t, pool, ctx, actorID, allEmployment, allEmployer, "2099-01-02", 1)

	all, err := repo.GetActivationAwareSchedules(ctx, date("2099-01-01"), date("2099-01-02"), []uuid.UUID{allEmployment}, 1, 50)
	if err != nil {
		t.Fatalf("all days activated: %v", err)
	}
	if len(all.Data) != 1 || len(all.Data[0].Schedules) != 2 || all.Data[0].Schedules[0].Provisional {
		t.Fatalf("all activated days must use official reader: %+v", all.Data)
	}
	allWithoutIDs, err := repo.GetActivationAwareSchedules(ctx, date("2099-01-01"), date("2099-01-02"), nil, 1, 50)
	if err != nil {
		t.Fatalf("all eligible employments without IDs: %v", err)
	}
	if len(allWithoutIDs.Data) != 1 || len(allWithoutIDs.Data[0].Schedules) != 2 || allWithoutIDs.Data[0].Schedules[0].Provisional {
		t.Fatalf("empty employment IDs must evaluate all eligible days: %+v", allWithoutIDs.Data)
	}

	partialEmployer, partialEmployment := insertActivationAwareEmployment(t, pool, ctx, "2099-02-01", "2099-02-02")
	insertActivation(t, pool, ctx, partialEmployer, "2099-02-01", "2099-02-01")
	partial, err := repo.GetActivationAwareSchedules(ctx, date("2099-02-01"), date("2099-02-02"), []uuid.UUID{partialEmployment}, 1, 50)
	if err != nil {
		t.Fatalf("partial activation: %v", err)
	}
	if len(partial.Data) != 1 || len(partial.Data[0].Schedules) != 2 || !partial.Data[0].Schedules[0].Provisional {
		t.Fatalf("partial activation must use legacy reader: %+v", partial.Data)
	}

	_, missingEmployment := insertActivationAwareEmployment(t, pool, ctx, "2099-03-01", "2099-03-01")
	missing, err := repo.GetActivationAwareSchedules(ctx, date("2099-03-01"), date("2099-03-01"), []uuid.UUID{missingEmployment}, 1, 50)
	if err != nil {
		t.Fatalf("missing activation config: %v", err)
	}
	if len(missing.Data) != 1 || len(missing.Data[0].Schedules) != 1 || !missing.Data[0].Schedules[0].Provisional {
		t.Fatalf("missing activation config must fail closed to legacy reader: %+v", missing.Data)
	}
}

func TestIntegrationOfficialScheduleReaderClipsEmploymentAndExcludesSupersededLegacyRows(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	actorID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert actor: %v", err)
	}
	employerID, employmentID := insertActivationAwareEmployment(t, pool, ctx, "2099-04-02", "2099-04-03")
	insertActivation(t, pool, ctx, employerID, "2099-04-01", "2099-04-04")
	insertPublishedRevision(t, pool, ctx, actorID, employmentID, employerID, "2099-04-02", 1)
	insertSupersededAndLegacyRows(t, pool, ctx, actorID, employmentID, employerID, "2099-04-02")
	insertPublishedRevision(t, pool, ctx, actorID, employmentID, employerID, "2099-04-03", 1)

	result, err := (ScheduleRepository{DB: pool}).GetOfficialSchedules(ctx, date("2099-04-01"), date("2099-04-04"), []uuid.UUID{employmentID}, 1, 50)
	if err != nil {
		t.Fatalf("official reader: %v", err)
	}
	if len(result.Data) != 1 || len(result.Data[0].Schedules) != 2 {
		t.Fatalf("reader must clip to employment dates: %+v", result.Data)
	}
	for _, day := range result.Data[0].Schedules {
		if day.RevisionID == nil || day.ScheduleStatus != "explicit_off" {
			t.Fatalf("superseded and legacy rows must not be current official rows: %+v", day)
		}
	}
}

func insertActivationAwareEmployment(t *testing.T, pool interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}, ctx context.Context, start, end string) (uuid.UUID, uuid.UUID) {
	t.Helper()
	employerID, employeeID, employmentID := uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, $2)`, employerID, "activation-aware-employer-"+employerID.String())
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date, tags) VALUES ($1, $2, $3, $4::date, $4::date, '{}'::text[])`, employeeID, employeeID.String(), "activation-aware-employee-"+employeeID.String(), start)
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, status, start_date, end_date) VALUES ($1, $2, $3, 'active', $4::date, $5::date)`, employmentID, employeeID, employerID, start, end)
	return employerID, employmentID
}

func insertActivation(t *testing.T, pool interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}, ctx context.Context, employerID uuid.UUID, start, end string) {
	t.Helper()
	if _, err := pool.Exec(ctx, `INSERT INTO config_entries (scope_type, scope_id, config_key, value_bool, effective_from, effective_to) VALUES ('employer', $1, 'schedule.lifecycle.enabled', true, $2::date, $3::date)`, employerID, start, end); err != nil {
		t.Fatalf("insert activation: %v", err)
	}
}

func insertPublishedRevision(t *testing.T, pool interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}, ctx context.Context, actorID, employmentID, employerID uuid.UUID, businessDate string, revision int) {
	t.Helper()
	batchID, revisionID := uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('a', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'activation-aware reader fixture', 'UTC', $4::date, $4::date)`, batchID, batchID.String(), actorID, businessDate); err != nil {
		t.Fatalf("insert publication batch: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', $5, 'explicit_off', $6, 'manual', '{}'::jsonb, 'test', 'activation-aware reader fixture', $7, 'human', CURRENT_TIMESTAMP)`, revisionID, employmentID, employerID, businessDate, revision, batchID, actorID); err != nil {
		t.Fatalf("insert official revision: %v", err)
	}
}

func insertSupersededAndLegacyRows(t *testing.T, pool interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}, ctx context.Context, actorID, employmentID, employerID uuid.UUID, businessDate string) {
	t.Helper()
	batchID, supersededID, legacyID := uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('b', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'superseded fixture', 'UTC', $4::date, $4::date)`, batchID, batchID.String(), actorID, businessDate); err != nil {
		t.Fatalf("insert superseded batch: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 2, 'superseded', $5, 'manual', '{}'::jsonb, 'test', 'superseded fixture', $6, 'human', CURRENT_TIMESTAMP), ($7, $2, $3, $4::date, 'UTC', 3, 'legacy_unverified', NULL, 'legacy_unverified', '{"provenance":"legacy_unverified"}'::jsonb, 'test', 'legacy fixture', NULL, 'system', NULL)`, supersededID, employmentID, employerID, businessDate, batchID, actorID, legacyID); err != nil {
		t.Fatalf("insert superseded and legacy rows: %v", err)
	}
}

func date(value string) time.Time {
	parsed, _ := time.Parse("2006-01-02", value)
	return parsed
}
