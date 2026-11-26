package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	payrolldomain "github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
)

func scanPayrollPeriod(row interface{ Scan(...any) error }) (api.PayrollPeriod, error) {
	var period api.PayrollPeriod
	var lockedAt, reopenedAt *time.Time
	if err := row.Scan(
		&period.ID, &period.EmployerID, &period.Year, &period.Month, &period.Status,
		&period.Version, &lockedAt, &period.LockedBy, &reopenedAt, &period.ReopenedBy,
		&period.ReopenReason,
	); err != nil {
		return api.PayrollPeriod{}, err
	}
	if lockedAt != nil {
		value := lockedAt.UTC().Format(time.RFC3339)
		period.LockedAt = &value
	}
	if reopenedAt != nil {
		value := reopenedAt.UTC().Format(time.RFC3339)
		period.ReopenedAt = &value
	}
	return period, nil
}

const payrollPeriodColumns = `id, employer_id, period_year, period_month, status, version,
	locked_at, locked_by, reopened_at, reopened_by, reopen_reason`

func (r PayrollRepository) GetPayrollPeriod(ctx context.Context, employerID ids.EmployerID, year, month int) (api.PayrollPeriod, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollPeriod{}, err
	}
	return scanPayrollPeriod(db.QueryRow(ctx,
		`SELECT `+payrollPeriodColumns+` FROM payroll_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3`,
		uuid.UUID(employerID), year, month,
	))
}

func (r PayrollRepository) LockPayrollPeriod(ctx context.Context, input payrolldomain.PayrollPeriodMutation) (api.PayrollPeriod, error) {
	return r.mutatePayrollPeriod(ctx, input, "lock", "payroll_period.lock")
}

func (r PayrollRepository) ReopenPayrollPeriod(ctx context.Context, input payrolldomain.PayrollPeriodMutation) (api.PayrollPeriod, error) {
	return r.mutatePayrollPeriod(ctx, input, "reopen", "payroll_period.reopen")
}

func (r PayrollRepository) mutatePayrollPeriod(ctx context.Context, input payrolldomain.PayrollPeriodMutation, action, auditAction string) (api.PayrollPeriod, error) {
	if input.ExpectedVersion < 1 || input.Year < 2000 || input.Year > 2100 || input.Month < 1 || input.Month > 12 {
		return api.PayrollPeriod{}, api.ErrPayrollPeriodValidation
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.PayrollPeriod{}, err
	}
	tx, err := beginner.Begin(ctx)
	if err != nil {
		return api.PayrollPeriod{}, err
	}
	committed := false
	defer func() {
		if !committed {
			_ = tx.Rollback(ctx)
		}
	}()

	var employerExists bool
	if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employers WHERE id = $1)`, uuid.UUID(input.EmployerID)).Scan(&employerExists); err != nil {
		return api.PayrollPeriod{}, err
	}
	if !employerExists {
		return api.PayrollPeriod{}, api.ErrEmployerNotFound
	}

	if _, err := tx.Exec(ctx, `
		INSERT INTO payroll_periods (employer_id, period_year, period_month)
		VALUES ($1, $2, $3)
		ON CONFLICT (employer_id, period_year, period_month) DO NOTHING`, uuid.UUID(input.EmployerID), input.Year, input.Month); err != nil {
		return api.PayrollPeriod{}, mapPayrollPeriodError(err)
	}

	var period api.PayrollPeriod
	var lockedAt, reopenedAt *time.Time
	if err := tx.QueryRow(ctx, `SELECT `+payrollPeriodColumns+` FROM payroll_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3 FOR UPDATE`, uuid.UUID(input.EmployerID), input.Year, input.Month).Scan(
		&period.ID, &period.EmployerID, &period.Year, &period.Month, &period.Status,
		&period.Version, &lockedAt, &period.LockedBy, &reopenedAt, &period.ReopenedBy, &period.ReopenReason,
	); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.PayrollPeriod{}, api.ErrPayrollPeriodNotFound
		}
		return api.PayrollPeriod{}, err
	}
	if period.Version != input.ExpectedVersion || (action == "lock" && period.Status == "locked") || (action == "reopen" && period.Status == "open") {
		return api.PayrollPeriod{}, api.ErrPayrollPeriodConflict
	}
	if action == "lock" {
		var attendanceStatus string
		err := tx.QueryRow(ctx, `SELECT status FROM attendance_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3 FOR UPDATE`, uuid.UUID(input.EmployerID), input.Year, input.Month).Scan(&attendanceStatus)
		if errors.Is(err, pgx.ErrNoRows) || err != nil || attendanceStatus != "closed" {
			if err != nil && !errors.Is(err, pgx.ErrNoRows) {
				return api.PayrollPeriod{}, err
			}
			return api.PayrollPeriod{}, api.ErrPayrollPeriodConflict
		}
	}
	if action == "reopen" && input.Reason == "" {
		return api.PayrollPeriod{}, api.ErrPayrollPeriodValidation
	}

	var query string
	var args []any
	if action == "lock" {
		query = `UPDATE payroll_periods SET status = 'locked', version = version + 1, locked_at = CURRENT_TIMESTAMP, locked_by = $1, reopened_at = NULL, reopened_by = NULL, reopen_reason = NULL WHERE id = $2 RETURNING ` + payrollPeriodColumns
		args = []any{uuid.UUID(input.ActorID), period.ID}
	} else {
		query = `UPDATE payroll_periods SET status = 'open', version = version + 1, reopened_at = CURRENT_TIMESTAMP, reopened_by = $1, reopen_reason = $2 WHERE id = $3 RETURNING ` + payrollPeriodColumns
		args = []any{uuid.UUID(input.ActorID), input.Reason, period.ID}
	}
	updated, err := scanPayrollPeriod(tx.QueryRow(ctx, query, args...))
	if err != nil {
		return api.PayrollPeriod{}, err
	}
	details, err := json.Marshal(map[string]any{"year": input.Year, "month": input.Month, "version": updated.Version})
	if err != nil {
		return api.PayrollPeriod{}, err
	}
	if _, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details) VALUES ($1, $2, $3, $4)`, uuid.UUID(input.ActorID), updated.ID, auditAction, details); err != nil {
		return api.PayrollPeriod{}, fmt.Errorf("payroll period audit: %w", err)
	}
	if err := tx.Commit(ctx); err != nil {
		return api.PayrollPeriod{}, err
	}
	committed = true
	return updated, nil
}

func (r PayrollRepository) begin(ctx context.Context) (TxBeginner, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("payroll repository: database does not support transactions")
	}
	return beginner, nil
}

func mapPayrollPeriodError(err error) error {
	if isUniqueViolation(err) {
		return api.ErrPayrollPeriodConflict
	}
	return err
}
