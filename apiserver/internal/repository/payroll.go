package repository

import (
	"context"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/timezone"
)

type PayrollRepository struct {
	DB DBTX
}

// organizationLocation resolves the singleton organization timezone and keeps
// UTC behavior when no settings row has been created yet.
func organizationLocation(ctx context.Context, db DBTX) *time.Location {
	var tzName string
	if err := db.QueryRow(ctx, `SELECT timezone FROM organization_settings LIMIT 1`).Scan(&tzName); err == nil && tzName != "" {
		return timezone.ParseTimezone(tzName)
	}
	return time.UTC
}

func scanPayslip(row interface{ Scan(...any) error }) (api.PayslipRecord, error) {
	var p api.PayslipRecord
	var base, ot, ded, net, premium string
	var created, updated time.Time
	err := row.Scan(&p.ID, &p.EmploymentID, &p.PeriodStart, &p.PeriodEnd, &base, &ot, &ded, &net, &p.Status, &created, &updated, &p.StandardUnits, &p.OvertimeUnits, &premium)
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
func (r PayrollRepository) ListPayslips(ctx context.Context, start, end time.Time) ([]api.PayslipRecord, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, employment_id, period_start::text, period_end::text, base_pay_amount::text, overtime_pay_amount::text, deductions_amount::text, net_pay_amount::text, status, created_at, updated_at, standard_units, overtime_units, holiday_work_premium_amount::text FROM payslips WHERE period_start >= $1 AND period_start < $2 ORDER BY period_start DESC, id DESC`, start, end)
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
func (r PayrollRepository) ListEmployeePayslips(ctx context.Context, empID ids.EmployeeID) ([]api.PayslipRecord, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT p.id,p.employment_id,p.period_start::text,p.period_end::text,p.base_pay_amount::text,p.overtime_pay_amount::text,p.deductions_amount::text,p.net_pay_amount::text,p.status,p.created_at,p.updated_at,p.standard_units,p.overtime_units,p.holiday_work_premium_amount::text FROM payslips p JOIN employments e ON e.id=p.employment_id JOIN employment_relationships er ON er.id=e.relationship_id AND er.employee_id=e.employee_id JOIN employee_tenures t ON t.id=er.tenure_id AND t.employee_id=er.employee_id WHERE e.employee_id=$1 AND t.deleted_at IS NULL AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE) ORDER BY p.period_start DESC,p.id DESC`, uuid.UUID(empID))
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
