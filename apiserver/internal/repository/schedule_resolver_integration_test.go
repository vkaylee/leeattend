//go:build integration

package repository

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestIntegrationLoadShiftsReadsEffectiveVersionsAndAllIntervals(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	actorID := uuid.New()
	templateID := uuid.New()
	firstVersion := uuid.New()
	secondVersion := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert actor: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_templates (id, name) VALUES ($1, 'resolver-effective-template')`, templateID); err != nil {
		t.Fatalf("insert template: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_versions (id, template_id, version_number, name, report_symbol, effective_from, effective_to, change_type, created_by) VALUES ($1, $2, 1, 'old', 'OLD', DATE '2026-01-01', DATE '2026-06-30', 'create', $3), ($4, $2, 2, 'new', 'NEW', DATE '2026-07-01', NULL, 'revise', $3)`, firstVersion, templateID, actorID, secondVersion); err != nil {
		t.Fatalf("insert versions: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time, break_duration_minutes) VALUES (gen_random_uuid(), $1, 0, '08:00', '12:00', 0), (gen_random_uuid(), $1, 1, '13:00', '17:00', 0)`, secondVersion); err != nil {
		t.Fatalf("insert intervals: %v", err)
	}
	shifts, err := loadShifts(ctx, pool)
	if err != nil {
		t.Fatalf("load shifts: %v", err)
	}
	meta := shifts[templateID]
	if len(meta.versions) != 2 || len(meta.versions[1].intervals) != 2 {
		t.Fatalf("loaded versions=%d intervals=%d, want 2/2", len(meta.versions), len(meta.versions[1].intervals))
	}
	old, selected, conflict := selectEffectiveShiftVersion(shifts, templateID, time.Date(2026, 6, 30, 0, 0, 0, 0, time.UTC))
	if !selected || conflict || old.id != firstVersion {
		t.Fatalf("before boundary selected=%v conflict=%v version=%s", selected, conflict, old.id)
	}
	newVersion, selected, conflict := selectEffectiveShiftVersion(shifts, templateID, time.Date(2026, 7, 1, 0, 0, 0, 0, time.UTC))
	if !selected || conflict || newVersion.id != secondVersion {
		t.Fatalf("on boundary selected=%v conflict=%v version=%s", selected, conflict, newVersion.id)
	}
}
