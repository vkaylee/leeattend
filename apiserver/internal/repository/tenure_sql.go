package repository

import (
	"context"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

// orgTodaySQL evaluates the organization's current calendar date in its configured timezone.
const orgTodaySQL = `(NOW() AT TIME ZONE COALESCE((SELECT timezone FROM organization_settings LIMIT 1), 'Asia/Ho_Chi_Minh'))::date`

// sqlEffectiveTenureLateral selects the employee's active tenure as of the organization's local date.
const sqlEffectiveTenureLateral = `LEFT JOIN LATERAL (
	SELECT t.id, t.employee_number, t.user_id, t.hire_type, t.prior_service_credit_days,
	       t.start_date, t.end_date, t.termination_reason, t.rehire_eligible
	FROM employee_tenures t
	WHERE t.employee_id = e.id
	  AND t.deleted_at IS NULL
	  AND t.start_date <= ` + orgTodaySQL + `
	  AND (t.end_date IS NULL OR t.end_date >= ` + orgTodaySQL + `)
	ORDER BY t.start_date DESC, t.id DESC
	LIMIT 1
) tenure ON TRUE`

// sqlDisplayTenureLateral selects the best tenure to display for an employee record:
// 1) active as of orgTodaySQL, 2) planned future open tenure, 3) latest closed tenure.
const sqlDisplayTenureLateral = `LEFT JOIN LATERAL (
	SELECT t.id, t.employee_number, t.user_id, t.hire_type, t.prior_service_credit_days,
	       t.start_date, t.end_date, t.termination_reason, t.rehire_eligible,
	       CASE
	           WHEN t.start_date > ` + orgTodaySQL + ` AND t.end_date IS NULL THEN 'planned'
	           WHEN t.end_date IS NULL OR t.end_date >= ` + orgTodaySQL + ` THEN 'active'
	           ELSE 'closed'
	       END AS tenure_status
	FROM employee_tenures t
	WHERE t.employee_id = e.id
	  AND t.deleted_at IS NULL
	ORDER BY
	    CASE
	        WHEN t.start_date <= ` + orgTodaySQL + ` AND (t.end_date IS NULL OR t.end_date >= ` + orgTodaySQL + `) THEN 0
	        WHEN t.start_date > ` + orgTodaySQL + ` AND t.end_date IS NULL THEN 1
	        ELSE 2
	    END ASC,
	    t.start_date DESC,
	    t.id DESC
	LIMIT 1
) tenure ON TRUE`

// SessionRevoker invalidates active JWT access tokens in Redis (matches redis.Client.SetUserRevocation).
type SessionRevoker interface {
	SetUserRevocation(ctx context.Context, userID uuid.UUID, ttl time.Duration) error
}

const sessionRevocationTTL = 15 * time.Minute

// revokeUserSessionsDBTX revokes all refresh sessions in PostgreSQL and sets the Redis JWT revocation marker.
// Fails closed if either step returns an error so callers inside a DB transaction roll back cleanly.
func revokeUserSessionsDBTX(ctx context.Context, tx DBTX, revoker SessionRevoker, userID uuid.UUID) error {
	if userID == uuid.Nil {
		return nil
	}
	if _, err := tx.Exec(ctx, `UPDATE sessions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL`, userID); err != nil {
		return fmt.Errorf("revoke db sessions: %w", err)
	}
	if revoker != nil {
		if err := revoker.SetUserRevocation(ctx, userID, sessionRevocationTTL); err != nil {
			return fmt.Errorf("revoke jwt cache: %w", err)
		}
	}
	return nil
}

// checkLockedPayrollForEmployeeDate rejects mutations whose date falls inside a locked payroll period.
func checkLockedPayrollForEmployeeDate(ctx context.Context, tx DBTX, employerID uuid.UUID, effectiveDate string) error {
	var locked bool
	err := tx.QueryRow(ctx, `
		SELECT EXISTS (
			SELECT 1
			FROM payroll_periods pp
			WHERE pp.employer_id = $1
			  AND pp.status = 'locked'
			  AND make_date(pp.period_year, pp.period_month, 1) <= $2::date
			  AND (make_date(pp.period_year, pp.period_month, 1) + INTERVAL '1 month - 1 day')::date >= $2::date
		)
	`, employerID, effectiveDate).Scan(&locked)
	if err != nil {
		return fmt.Errorf("check locked payroll period: %w", err)
	}
	if locked {
		return fmt.Errorf("%w: payroll period is locked for %s", api.ErrConflict, effectiveDate)
	}
	return nil
}
