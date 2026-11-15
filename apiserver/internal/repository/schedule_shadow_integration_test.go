//go:build integration

package repository

import (
	"encoding/json"
	"errors"
	"testing"

	"github.com/google/uuid"
	api "github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationScheduleShadowMismatchUpsertAndResolution(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'shadow-test-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'shadow-test-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String()); err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID); err != nil {
		t.Fatalf("insert employment: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert actor: %v", err)
	}
	legacy, _ := json.Marshal(map[string]any{"status": "work", "provenance": "legacy_unverified"})
	resolver, _ := json.Marshal(map[string]any{"status": "explicit_off", "provenance": "resolver"})
	record := ScheduleShadowMismatchRecord{EmployerID: employerID, EmploymentID: employmentID, BusinessDate: "2026-11-20", MismatchKind: "explicit_off", LegacySnapshot: legacy, ResolverSnapshot: resolver, ResolverVersion: lifecycleResolverVersion}
	repo := ScheduleRepository{DB: pool}
	if err := repo.RecordScheduleShadowMismatch(ctx, record); err != nil {
		t.Fatalf("record mismatch: %v", err)
	}
	resolver2, _ := json.Marshal(map[string]any{"status": "work", "provenance": "resolver-v2"})
	record.ResolverSnapshot = resolver2
	if err := repo.RecordScheduleShadowMismatch(ctx, record); err != nil {
		t.Fatalf("upsert mismatch: %v", err)
	}
	var count int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_shadow_mismatches WHERE employer_id = $1 AND employment_id = $2 AND business_date = DATE '2026-11-20' AND status = 'open'`, employerID, employmentID).Scan(&count); err != nil {
		t.Fatalf("count open mismatch: %v", err)
	}
	if count != 1 {
		t.Fatalf("open mismatch count = %d, want 1", count)
	}
	var mismatchID uuid.UUID
	if err := pool.QueryRow(ctx, `SELECT id FROM schedule_shadow_mismatches WHERE employer_id = $1 AND employment_id = $2 AND business_date = DATE '2026-11-20'`, employerID, employmentID).Scan(&mismatchID); err != nil {
		t.Fatalf("read mismatch ID: %v", err)
	}
	if err := repo.ResolveScheduleShadowMismatch(ctx, mismatchID, "resolved", "reviewed by integration test", actorID); err != nil {
		t.Fatalf("resolve mismatch: %v", err)
	}
	if err := repo.ResolveScheduleShadowMismatch(ctx, mismatchID, "resolved", "second resolution", actorID); !errors.Is(err, api.ErrScheduleShadowMismatchNotFound) {
		t.Fatalf("second resolution error = %v, want not found", err)
	}
	var status string
	if err := pool.QueryRow(ctx, `SELECT status FROM schedule_shadow_mismatches WHERE id = $1`, mismatchID).Scan(&status); err != nil {
		t.Fatalf("read mismatch status: %v", err)
	}
	if status != "resolved" {
		t.Fatalf("mismatch status = %q", status)
	}
}

func TestIntegrationLegacyUnverifiedRevisionHasUniqueEmploymentDayKey(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID := uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'legacy-backfill-test-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'legacy-backfill-test-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String()); err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID); err != nil {
		t.Fatalf("insert employment: %v", err)
	}
	insert := `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, source, source_snapshot, resolver_version, reason, published_actor_type) VALUES ($1, $2, $3, DATE '2026-11-20', 'UTC', 1, 'legacy_unverified', 'legacy_unverified', '{"provenance":"legacy_unverified"}', 'legacy-import-v1', 'backfill evidence', 'system')`
	if _, err := pool.Exec(ctx, insert, uuid.New(), employmentID, employerID); err != nil {
		t.Fatalf("insert legacy revision: %v", err)
	}
	if _, err := pool.Exec(ctx, insert, uuid.New(), employmentID, employerID); err == nil {
		t.Fatal("duplicate legacy employment/day accepted")
	}
}
