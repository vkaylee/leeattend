package repository

import (
	"context"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"strings"
)

func validScheduleActivationParams(employerID, actorID uuid.UUID, startDate, endDate, reason string) bool {
	trimmed := strings.TrimSpace(reason)
	return employerID != uuid.Nil && actorID != uuid.Nil && trimmed != "" && len(trimmed) <= 2000 && startDate != "" && endDate != "" && startDate <= endDate
}

func (r ScheduleRepository) ActivateScheduleEmployer(ctx context.Context, employerID uuid.UUID, startDate, endDate, reason string, actorID uuid.UUID) error {
	if !validScheduleActivationParams(employerID, actorID, startDate, endDate, reason) {
		return api.ErrScheduleActivationNotReady
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var ready bool
		if err := tx.QueryRow(ctx, `SELECT
			ff.enabled
			FROM feature_flags ff
			WHERE ff.flag_key = 'schedule_lifecycle'
			AND NOT EXISTS (SELECT 1 FROM schedule_shadow_mismatches WHERE employer_id = $1 AND business_date BETWEEN $2::date AND $3::date AND status = 'open')
			AND NOT EXISTS (SELECT 1 FROM schedule_legacy_mutation_evidence e JOIN employments em ON em.id = e.employment_id WHERE em.employer_id = $1 AND e.operation = 'guard_blocked' AND e.business_date BETWEEN $2::date AND $3::date)
			AND NOT EXISTS (SELECT 1 FROM outbox_jobs WHERE job_type = 'ScheduleOfficeCoverageRefresh' AND status = 'dead_letter' AND payload->>'employer_id' = $1::text AND payload->>'business_date' BETWEEN $2 AND $3)
			AND EXISTS (SELECT 1 FROM schedule_office_policies p JOIN approval_workflow_instances i ON i.id = p.approval_instance_id WHERE p.employer_id = $1 AND p.status = 'active' AND p.effective_from <= $3::date AND (p.effective_to IS NULL OR p.effective_to >= $2::date) AND i.entity_type = 'schedule_policy' AND i.status = 'approved')
			AND NOT EXISTS (SELECT 1 FROM schedule_day_revisions r JOIN schedule_day_revisions d ON d.employment_id = r.employment_id AND d.business_date = r.business_date AND d.state IN ('published', 'explicit_off') AND r.state IN ('published', 'explicit_off') AND d.id <> r.id WHERE r.employer_id = $1 AND r.business_date BETWEEN $2::date AND $3::date)
			AND NOT EXISTS (SELECT 1 FROM employments e JOIN schedule_office_policies p ON p.employer_id = e.employer_id AND p.status = 'active' AND p.effective_from <= $3::date AND (p.effective_to IS NULL OR p.effective_to >= $2::date) JOIN approval_workflow_instances i ON i.id = p.approval_instance_id AND i.entity_type = 'schedule_policy' AND i.status = 'approved' CROSS JOIN generate_series(GREATEST($2::date, p.effective_from), LEAST($3::date, COALESCE(p.effective_to, $3::date)), interval '1 day') days WHERE e.employer_id = $1 AND e.deleted_at IS NULL AND e.start_date <= days.day::date AND (e.end_date IS NULL OR e.end_date >= days.day::date) AND NOT EXISTS (SELECT 1 FROM schedule_day_revisions r WHERE r.employment_id = e.id AND r.business_date = days.day::date AND r.state IN ('published', 'explicit_off')))
		`, employerID, startDate, endDate).Scan(&ready); err != nil {
			return err
		}
		if !ready {
			return api.ErrScheduleActivationNotReady
		}
		if _, err := tx.Exec(ctx, `INSERT INTO config_entries (id, scope_type, scope_id, config_key, value_bool, effective_from, effective_to, created_by, created_at, updated_at)
			VALUES (gen_random_uuid(), 'employer', $1, 'schedule.lifecycle.enabled', true, $2::date, $3::date, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
			ON CONFLICT (scope_type, scope_id, config_key, effective_from)
			DO UPDATE SET value_bool = EXCLUDED.value_bool, effective_to = EXCLUDED.effective_to, updated_at = CURRENT_TIMESTAMP`, employerID, startDate, endDate, actorID); err != nil {
			return err
		}
		_, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, actor_type, created_at) VALUES ($1, $2, 'schedule_lifecycle.activation.enabled', jsonb_build_object('start_date', $3, 'end_date', $4, 'reason', $5), 'human', CURRENT_TIMESTAMP)`, actorID, employerID, startDate, endDate, strings.TrimSpace(reason))
		return err
	})
}

func (r ScheduleRepository) DeactivateScheduleEmployer(ctx context.Context, employerID uuid.UUID, startDate, endDate, reason string, actorID uuid.UUID) error {
	if !validScheduleActivationParams(employerID, actorID, startDate, endDate, reason) {
		return api.ErrScheduleActivationNotReady
	}
	beginner, err := r.beginLifecycle(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var hasEvidence bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (
			SELECT 1 FROM schedule_day_revisions r
			WHERE r.employer_id = $1 AND r.business_date BETWEEN $2::date AND $3::date
			  AND r.state IN ('published', 'explicit_off')
			UNION ALL
			SELECT 1 FROM attendance_closure_days d
			WHERE d.employer_id = $1 AND d.business_date BETWEEN $2::date AND $3::date
		)`, employerID, startDate, endDate).Scan(&hasEvidence); err != nil {
			return err
		}
		if hasEvidence {
			return api.ErrScheduleActivationEvidence
		}
		var exactRange bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (
			SELECT 1 FROM config_entries
			WHERE scope_type = 'employer' AND scope_id = $1 AND config_key = 'schedule.lifecycle.enabled'
			  AND effective_from = $2::date AND effective_to = $3::date AND value_bool = true
		)`, employerID, startDate, endDate).Scan(&exactRange); err != nil {
			return err
		}
		if !exactRange {
			return api.ErrScheduleActivationNotReady
		}
		if _, err := tx.Exec(ctx, `UPDATE config_entries SET value_bool = false, updated_at = CURRENT_TIMESTAMP
			WHERE scope_type = 'employer' AND scope_id = $1 AND config_key = 'schedule.lifecycle.enabled'
			  AND effective_from = $2::date AND effective_to = $3::date`, employerID, startDate, endDate); err != nil {
			return err
		}
		_, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, actor_type, created_at)
			VALUES ($1, $2, 'schedule_lifecycle.activation.disabled', jsonb_build_object('start_date', $3, 'end_date', $4, 'reason', $5), 'human', CURRENT_TIMESTAMP)`, actorID, employerID, startDate, endDate, strings.TrimSpace(reason))
		return err
	})
}
