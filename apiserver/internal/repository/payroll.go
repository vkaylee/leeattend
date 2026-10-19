package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type PayrollRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r PayrollRepository) db(ctx context.Context, wsID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("payroll repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, wsID)
}
func scanPayslip(row interface{ Scan(...any) error }) (api.PayslipRecord, error) {
	var p api.PayslipRecord
	var base, ot, ded, net, premium string
	var created, updated time.Time
	err := row.Scan(&p.ID, &p.WorkspaceID, &p.EmploymentID, &p.PeriodStart, &p.PeriodEnd, &base, &ot, &ded, &net, &p.Status, &created, &updated, &p.StandardUnits, &p.OvertimeUnits, &premium)
	if err != nil {
		return p, err
	}
	p.BasePayAmount, _ = decimal.NewFromString(base)
	p.OvertimePayAmount, _ = decimal.NewFromString(ot)
	p.DeductionsAmount, _ = decimal.NewFromString(ded)
	p.NetPayAmount, _ = decimal.NewFromString(net)
	p.HolidayWorkPremiumAmount, _ = decimal.NewFromString(premium)
	p.CreatedAt = created.UTC().Format(time.RFC3339)
	p.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return p, nil
}
func (r PayrollRepository) ListWorkspacePayslips(ctx context.Context, wsID uuid.UUID, start, end time.Time) ([]api.PayslipRecord, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, employment_id, period_start::text, period_end::text, base_pay_amount::text, overtime_pay_amount::text, deductions_amount::text, net_pay_amount::text, status, created_at, updated_at, standard_units, overtime_units, holiday_work_premium_amount::text FROM payslips WHERE workspace_id=$1 AND period_start >= $2 AND period_start < $3 ORDER BY period_start DESC, id DESC`, wsID, start, end)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PayslipRecord, 0)
	for rows.Next() {
		p, err := scanPayslip(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, p)
	}
	return result, rows.Err()
}
func (r PayrollRepository) ListEmployeePayslips(ctx context.Context, wsID, empID uuid.UUID) ([]api.PayslipRecord, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT p.id,p.workspace_id,p.employment_id,p.period_start::text,p.period_end::text,p.base_pay_amount::text,p.overtime_pay_amount::text,p.deductions_amount::text,p.net_pay_amount::text,p.status,p.created_at,p.updated_at,p.standard_units,p.overtime_units,p.holiday_work_premium_amount::text FROM payslips p JOIN employments e ON e.id=p.employment_id WHERE p.workspace_id=$1 AND e.employee_id=$2 AND e.workspace_id=$1 ORDER BY p.period_start DESC,p.id DESC`, wsID, empID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PayslipRecord, 0)
	for rows.Next() {
		p, err := scanPayslip(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, p)
	}
	return result, rows.Err()
}
