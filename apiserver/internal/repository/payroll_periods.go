package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
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

func (r PayrollRepository) GetPayrollPeriod(ctx context.Context, employerID uuid.UUID, year, month int) (api.PayrollPeriod, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollPeriod{}, err
	}
	return scanPayrollPeriod(db.QueryRow(ctx,
		`SELECT `+payrollPeriodColumns+` FROM payroll_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3`,
		employerID, year, month,
	))
}

func (r PayrollRepository) LockPayrollPeriod(ctx context.Context, employerID uuid.UUID, year, month int, expectedVersion int64, actorID uuid.UUID) (api.PayrollPeriod, error) {
	return r.mutatePayrollPeriod(ctx, employerID, year, month, expectedVersion, actorID, "lock", "", "payroll_period.lock")
}

func (r PayrollRepository) ReopenPayrollPeriod(ctx context.Context, employerID uuid.UUID, year, month int, expectedVersion int64, reason string, actorID uuid.UUID) (api.PayrollPeriod, error) {
	return r.mutatePayrollPeriod(ctx, employerID, year, month, expectedVersion, actorID, "reopen", reason, "payroll_period.reopen")
}

func (r PayrollRepository) mutatePayrollPeriod(ctx context.Context, employerID uuid.UUID, year, month int, expectedVersion int64, actorID uuid.UUID, action, reason, auditAction string) (api.PayrollPeriod, error) {
	if expectedVersion < 1 || year < 2000 || year > 2100 || month < 1 || month > 12 {
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
	if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employers WHERE id = $1)`, employerID).Scan(&employerExists); err != nil {
		return api.PayrollPeriod{}, err
	}
	if !employerExists {
		return api.PayrollPeriod{}, api.ErrEmployerNotFound
	}

	if _, err := tx.Exec(ctx, `
		INSERT INTO payroll_periods (employer_id, period_year, period_month)
		VALUES ($1, $2, $3)
		ON CONFLICT (employer_id, period_year, period_month) DO NOTHING`, employerID, year, month); err != nil {
		return api.PayrollPeriod{}, mapPayrollPeriodError(err)
	}

	var period api.PayrollPeriod
	var lockedAt, reopenedAt *time.Time
	if err := tx.QueryRow(ctx, `SELECT `+payrollPeriodColumns+` FROM payroll_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3 FOR UPDATE`, employerID, year, month).Scan(
		&period.ID, &period.EmployerID, &period.Year, &period.Month, &period.Status,
		&period.Version, &lockedAt, &period.LockedBy, &reopenedAt, &period.ReopenedBy, &period.ReopenReason,
	); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.PayrollPeriod{}, api.ErrPayrollPeriodNotFound
		}
		return api.PayrollPeriod{}, err
	}
	if period.Version != expectedVersion || (action == "lock" && period.Status == "locked") || (action == "reopen" && period.Status == "open") {
		return api.PayrollPeriod{}, api.ErrPayrollPeriodConflict
	}
	if action == "reopen" && reason == "" {
		return api.PayrollPeriod{}, api.ErrPayrollPeriodValidation
	}

	var query string
	var args []any
	if action == "lock" {
		query = `UPDATE payroll_periods SET status = 'locked', version = version + 1, locked_at = CURRENT_TIMESTAMP, locked_by = $1, reopened_at = NULL, reopened_by = NULL, reopen_reason = NULL WHERE id = $2 RETURNING ` + payrollPeriodColumns
		args = []any{actorID, period.ID}
	} else {
		query = `UPDATE payroll_periods SET status = 'open', version = version + 1, reopened_at = CURRENT_TIMESTAMP, reopened_by = $1, reopen_reason = $2 WHERE id = $3 RETURNING ` + payrollPeriodColumns
		args = []any{actorID, reason, period.ID}
	}
	updated, err := scanPayrollPeriod(tx.QueryRow(ctx, query, args...))
	if err != nil {
		return api.PayrollPeriod{}, err
	}
	details, err := json.Marshal(map[string]any{"year": year, "month": month, "version": updated.Version})
	if err != nil {
		return api.PayrollPeriod{}, err
	}
	if _, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details) VALUES ($1, $2, $3, $4)`, actorID, updated.ID, auditAction, details); err != nil {
		return api.PayrollPeriod{}, fmt.Errorf("payroll period audit: %w", err)
	}
	if err := tx.Commit(ctx); err != nil {
		return api.PayrollPeriod{}, err
	}
	committed = true
	return updated, nil
}

func (r PayrollRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.DB == nil {
		return nil, errors.New("payroll repository: database is not configured")
	}
	beginner, ok := r.DB.(TxBeginner)
	if !ok {
		return nil, errors.New("payroll repository: database does not support transactions")
	}
	return beginner, nil
}

func mapPayrollPeriodError(err error) error {
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) && pgErr.Code == "23505" {
		return api.ErrPayrollPeriodConflict
	}
	return err
}
