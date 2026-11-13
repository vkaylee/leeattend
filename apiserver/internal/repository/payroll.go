package repository

import (
	"context"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/incentives"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/rates"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/rules"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/timezone"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type PayrollRepository struct {
	DB DBTX
}

func isConflictError(err error) bool {
	var pgErr *pgconn.PgError
	return errors.As(err, &pgErr) && (pgErr.Code == "23505" || pgErr.Code == "23P01")
}
func (r PayrollRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("payroll repository: database is not configured")
	}
	return r.DB, nil
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
	db, err := r.db(ctx)
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
func (r PayrollRepository) ListEmployeePayslips(ctx context.Context, empID uuid.UUID) ([]api.PayslipRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT p.id,p.employment_id,p.period_start::text,p.period_end::text,p.base_pay_amount::text,p.overtime_pay_amount::text,p.deductions_amount::text,p.net_pay_amount::text,p.status,p.created_at,p.updated_at,p.standard_units,p.overtime_units,p.holiday_work_premium_amount::text FROM payslips p JOIN employments e ON e.id=p.employment_id WHERE e.employee_id=$1 ORDER BY p.period_start DESC,p.id DESC`, empID)
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

func (r PayrollRepository) ListSalaryComponents(ctx context.Context) ([]api.SalaryComponentDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, name, code, created_at, updated_at FROM salary_component_definitions ORDER BY name ASC, code ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.SalaryComponentDefinition, 0)
	for rows.Next() {
		var d api.SalaryComponentDefinition
		var created, updated time.Time
		if err := rows.Scan(&d.ID, &d.Name, &d.Code, &created, &updated); err != nil {
			return nil, err
		}
		d.CreatedAt = created.UTC().Format(time.RFC3339)
		d.UpdatedAt = updated.UTC().Format(time.RFC3339)
		result = append(result, d)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CreateSalaryComponent(ctx context.Context, req api.CreateSalaryComponentRequest) (api.SalaryComponentDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.SalaryComponentDefinition{}, err
	}

	id := uuid.New()
	var def api.SalaryComponentDefinition
	var created, updated time.Time
	err = db.QueryRow(ctx, `INSERT INTO salary_component_definitions (id, name, code, component_type, created_at, updated_at) VALUES ($1, $2, $3, 'earning', NOW(), NOW()) RETURNING id, name, code, created_at, updated_at`, id, strings.TrimSpace(req.Name), strings.TrimSpace(req.Code)).Scan(&def.ID, &def.Name, &def.Code, &created, &updated)
	if err != nil {
		return api.SalaryComponentDefinition{}, mapConstraintError(err)
	}
	def.CreatedAt = created.UTC().Format(time.RFC3339)
	def.UpdatedAt = updated.UTC().Format(time.RFC3339)

	return def, nil
}

func (r PayrollRepository) ListEmploymentSalaryComponents(ctx context.Context, employmentID uuid.UUID) ([]api.EmploymentSalaryComponentResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	tz := organizationLocation(ctx, db)
	refDate := time.Now().In(tz).Format("2006-01-02")

	rows, err := db.Query(ctx, `SELECT DISTINCT ON (esc.employment_id, esc.component_definition_id)
		esc.id, esc.employment_id, esc.component_definition_id, esc.amount::text, esc.effective_date::text,
		scd.name, scd.code
	FROM employee_salary_components esc
	INNER JOIN salary_component_definitions scd ON scd.id = esc.component_definition_id
	WHERE esc.employment_id = $1
	  AND esc.effective_date <= $2
	ORDER BY esc.employment_id, esc.component_definition_id, esc.effective_date DESC, esc.created_at DESC`, employmentID, refDate)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.EmploymentSalaryComponentResponse, 0)
	for rows.Next() {
		var item api.EmploymentSalaryComponentResponse
		var amt string
		if err := rows.Scan(&item.ID, &item.EmploymentID, &item.ComponentDefinitionID, &amt, &item.EffectiveDate, &item.Name, &item.Code); err != nil {
			return nil, err
		}
		item.Amount, _ = decimal.NewFromString(amt)
		result = append(result, item)
	}
	return result, rows.Err()
}

func (r PayrollRepository) AssignEmploymentSalaryComponent(ctx context.Context, employmentID uuid.UUID, req api.AssignSalaryComponentRequest) (api.EmployeeSalaryComponent, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeSalaryComponent{}, err
	}

	var compDefExists bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM salary_component_definitions WHERE id = $1)`, req.ComponentDefinitionID).Scan(&compDefExists)
	if err != nil {
		return api.EmployeeSalaryComponent{}, err
	}
	if !compDefExists {
		return api.EmployeeSalaryComponent{}, api.ErrComponentNotFound
	}

	var empExists bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND deleted_at IS NULL)`, employmentID).Scan(&empExists)
	if err != nil {
		return api.EmployeeSalaryComponent{}, err
	}
	if !empExists {
		return api.EmployeeSalaryComponent{}, api.ErrEmploymentNotFound
	}

	id := uuid.New()
	var item api.EmployeeSalaryComponent
	var amt string
	var created, updated time.Time
	err = db.QueryRow(ctx, `INSERT INTO employee_salary_components (id, employment_id, component_definition_id, amount, effective_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) RETURNING id, employment_id, component_definition_id, amount::text, effective_date::text, created_at, updated_at`, id, employmentID, req.ComponentDefinitionID, req.Amount, req.EffectiveDate).Scan(&item.ID, &item.EmploymentID, &item.ComponentDefinitionID, &amt, &item.EffectiveDate, &created, &updated)
	if err != nil {
		return api.EmployeeSalaryComponent{}, mapConstraintError(err)
	}
	item.Amount, _ = decimal.NewFromString(amt)
	item.CreatedAt = created.UTC().Format(time.RFC3339)
	item.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return item, nil
}

func (r PayrollRepository) GetPayrollSettings(ctx context.Context) (*api.PayrollSettings, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	tz := organizationLocation(ctx, db)
	today := time.Now().In(tz).Format("2006-01-02")

	rows, err := db.Query(ctx, `SELECT config_key, value_int, value_decimal, value_text, effective_from::text, created_at, updated_at
FROM config_entries
WHERE scope_type = 'organization' AND scope_id IS NULL AND effective_from <= $1::date
ORDER BY effective_from DESC, created_at DESC`, today)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	hasRows := false
	mode := "fixed"
	fixedDays := decimal.NewFromInt(26)
	daysPerWeek := 5
	hoursPerDay := decimal.NewFromInt(8)
	var maxOtDay *decimal.Decimal
	var maxOtMonth *decimal.Decimal
	maxEffectiveDate := today
	var maxCreatedAt, maxUpdatedAt time.Time

	seen := make(map[string]bool)
	for rows.Next() {
		hasRows = true
		var key string
		var valInt *int
		var valDec *decimal.Decimal
		var valText *string
		var effDate string
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&key, &valInt, &valDec, &valText, &effDate, &createdAt, &updatedAt); err != nil {
			return nil, err
		}
		if createdAt.After(maxCreatedAt) {
			maxCreatedAt = createdAt
		}
		if updatedAt.After(maxUpdatedAt) {
			maxUpdatedAt = updatedAt
		}
		if seen[key] {
			continue
		}
		seen[key] = true
		switch key {
		case "payroll.std_working_day_mode":
			if valText != nil && *valText != "" {
				mode = *valText
			}
			maxEffectiveDate = effDate
		case "payroll.fixed_std_days":
			if valDec != nil {
				fixedDays = *valDec
			}
		case "payroll.working_days_per_week":
			if valInt != nil {
				daysPerWeek = *valInt
			}
		case "payroll.hours_per_day":
			if valDec != nil {
				hoursPerDay = *valDec
			}
		case "payroll.max_ot_hours_per_day":
			maxOtDay = valDec
		case "payroll.max_ot_hours_per_month":
			maxOtMonth = valDec
		}
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	if !hasRows {
		return nil, api.ErrPayrollSettingsNotFound
	}
	if maxCreatedAt.IsZero() {
		maxCreatedAt = time.Now().UTC()
	}
	if maxUpdatedAt.IsZero() {
		maxUpdatedAt = time.Now().UTC()
	}

	return &api.PayrollSettings{
		StdWorkingDayMode:  mode,
		FixedStdDays:       fixedDays,
		WorkingDaysPerWeek: daysPerWeek,
		HoursPerDay:        hoursPerDay,
		MaxOtHoursPerDay:   maxOtDay,
		MaxOtHoursPerMonth: maxOtMonth,
		EffectiveDate:      maxEffectiveDate,
		CreatedAt:          maxCreatedAt.UTC().Format(time.RFC3339),
		UpdatedAt:          maxUpdatedAt.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) GetPayrollSettingsHistory(ctx context.Context) ([]api.PayrollSettings, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT config_key, value_int, value_decimal, value_text, effective_from::text, created_at, updated_at
FROM config_entries
WHERE scope_type = 'organization' AND scope_id IS NULL
ORDER BY effective_from DESC, created_at DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	type entryRow struct {
		key       string
		valInt    *int
		valDec    *decimal.Decimal
		valText   *string
		effDate   string
		createdAt time.Time
		updatedAt time.Time
	}
	var allEntries []entryRow
	var dates []string
	dateSeen := make(map[string]bool)

	for rows.Next() {
		var e entryRow
		if err := rows.Scan(&e.key, &e.valInt, &e.valDec, &e.valText, &e.effDate, &e.createdAt, &e.updatedAt); err != nil {
			return nil, err
		}
		allEntries = append(allEntries, e)
		if !dateSeen[e.effDate] {
			dateSeen[e.effDate] = true
			dates = append(dates, e.effDate)
		}
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	if len(dates) == 0 {
		return []api.PayrollSettings{}, nil
	}

	result := make([]api.PayrollSettings, 0, len(dates))
	for _, d := range dates {
		mode := "fixed"
		fixedDays := decimal.NewFromInt(26)
		daysPerWeek := 5
		hoursPerDay := decimal.NewFromInt(8)
		var maxOtDay *decimal.Decimal
		var maxOtMonth *decimal.Decimal
		var maxCreatedAt, maxUpdatedAt time.Time
		seen := make(map[string]bool)

		for _, e := range allEntries {
			if e.effDate > d {
				continue
			}
			if e.createdAt.After(maxCreatedAt) {
				maxCreatedAt = e.createdAt
			}
			if e.updatedAt.After(maxUpdatedAt) {
				maxUpdatedAt = e.updatedAt
			}
			if seen[e.key] {
				continue
			}
			seen[e.key] = true
			switch e.key {
			case "payroll.std_working_day_mode":
				if e.valText != nil && *e.valText != "" {
					mode = *e.valText
				}
			case "payroll.fixed_std_days":
				if e.valDec != nil {
					fixedDays = *e.valDec
				}
			case "payroll.working_days_per_week":
				if e.valInt != nil {
					daysPerWeek = *e.valInt
				}
			case "payroll.hours_per_day":
				if e.valDec != nil {
					hoursPerDay = *e.valDec
				}
			case "payroll.max_ot_hours_per_day":
				maxOtDay = e.valDec
			case "payroll.max_ot_hours_per_month":
				maxOtMonth = e.valDec
			}
		}

		if maxCreatedAt.IsZero() {
			maxCreatedAt = time.Now().UTC()
		}
		if maxUpdatedAt.IsZero() {
			maxUpdatedAt = time.Now().UTC()
		}

		result = append(result, api.PayrollSettings{
			StdWorkingDayMode:  mode,
			FixedStdDays:       fixedDays,
			WorkingDaysPerWeek: daysPerWeek,
			HoursPerDay:        hoursPerDay,
			MaxOtHoursPerDay:   maxOtDay,
			MaxOtHoursPerMonth: maxOtMonth,
			EffectiveDate:      d,
			CreatedAt:          maxCreatedAt.UTC().Format(time.RFC3339),
			UpdatedAt:          maxUpdatedAt.UTC().Format(time.RFC3339),
		})
	}

	return result, nil
}

func (r PayrollRepository) UpdatePayrollSettings(ctx context.Context, req api.UpdatePayrollSettingsRequest) (api.PayrollSettings, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollSettings{}, err
	}
	tz := organizationLocation(ctx, db)
	effectiveDate := time.Now().In(tz).Format("2006-01-02")
	if req.EffectiveDate != nil && *req.EffectiveDate != "" {
		effectiveDate = *req.EffectiveDate
	}

	currentMode := "fixed"
	currentFixedDays := decimal.NewFromInt(26)
	currentDaysPerWeek := 6
	currentHoursPerDay := decimal.NewFromInt(8)
	var currentMaxOtDay *decimal.Decimal
	var currentMaxOtMonth *decimal.Decimal

	rows, err := db.Query(ctx, `SELECT config_key, value_int, value_decimal, value_text
FROM config_entries
WHERE scope_type = 'organization' AND scope_id IS NULL AND effective_from <= $1::date
ORDER BY effective_from DESC, created_at DESC`, effectiveDate)
	if err == nil {
		defer rows.Close()
		seen := make(map[string]bool)
		for rows.Next() {
			var k string
			var vInt *int
			var vDec *decimal.Decimal
			var vText *string
			if scanErr := rows.Scan(&k, &vInt, &vDec, &vText); scanErr == nil {
				if !seen[k] {
					seen[k] = true
					switch k {
					case "payroll.std_working_day_mode":
						if vText != nil && *vText != "" {
							currentMode = *vText
						}
					case "payroll.fixed_std_days":
						if vDec != nil {
							currentFixedDays = *vDec
						}
					case "payroll.working_days_per_week":
						if vInt != nil {
							currentDaysPerWeek = *vInt
						}
					case "payroll.hours_per_day":
						if vDec != nil {
							currentHoursPerDay = *vDec
						}
					case "payroll.max_ot_hours_per_day":
						currentMaxOtDay = vDec
					case "payroll.max_ot_hours_per_month":
						currentMaxOtMonth = vDec
					}
				}
			}
		}
	}

	mode := currentMode
	if req.StdWorkingDayMode != nil && *req.StdWorkingDayMode != "" {
		mode = *req.StdWorkingDayMode
	}
	fixedDays := currentFixedDays
	if req.FixedStdDays != nil {
		fixedDays = *req.FixedStdDays
	}
	daysPerWeek := currentDaysPerWeek
	if req.WorkingDaysPerWeek != nil {
		daysPerWeek = *req.WorkingDaysPerWeek
	}
	hoursPerDay := currentHoursPerDay
	if req.HoursPerDay != nil {
		hoursPerDay = *req.HoursPerDay
	}
	maxOtDay := currentMaxOtDay
	if req.MaxOtHoursPerDay != nil {
		maxOtDay = *req.MaxOtHoursPerDay
	}
	maxOtMonth := currentMaxOtMonth
	if req.MaxOtHoursPerMonth != nil {
		maxOtMonth = *req.MaxOtHoursPerMonth
	}

	type keyOp struct {
		key    string
		valInt *int
		valDec *decimal.Decimal
		valTxt *string
	}
	ops := []keyOp{
		{key: "payroll.std_working_day_mode", valTxt: &mode},
		{key: "payroll.fixed_std_days", valDec: &fixedDays},
		{key: "payroll.working_days_per_week", valInt: &daysPerWeek},
		{key: "payroll.hours_per_day", valDec: &hoursPerDay},
		{key: "payroll.max_ot_hours_per_day", valDec: maxOtDay},
		{key: "payroll.max_ot_hours_per_month", valDec: maxOtMonth},
	}

	for _, op := range ops {
		hasVal := op.valInt != nil || op.valDec != nil || (op.valTxt != nil && *op.valTxt != "")
		if hasVal {
			var existingID uuid.UUID
			err := db.QueryRow(ctx, `SELECT id FROM config_entries
WHERE scope_type = 'organization' AND scope_id IS NULL AND config_key = $1 AND effective_from = $2::date`, op.key, effectiveDate).Scan(&existingID)
			if err == nil {
				_, err = db.Exec(ctx, `UPDATE config_entries
SET value_int = $1, value_decimal = $2, value_text = $3, updated_at = NOW()
WHERE id = $4`, op.valInt, op.valDec, op.valTxt, existingID)
				if err != nil {
					return api.PayrollSettings{}, err
				}
			} else {
				newID := uuid.New()
				_, err = db.Exec(ctx, `INSERT INTO config_entries (id, scope_type, scope_id, config_key, value_int, value_decimal, value_text, effective_from, created_at, updated_at)
VALUES ($1, 'organization', NULL, $2, $3, $4, $5, $6::date, NOW(), NOW())`, newID, op.key, op.valInt, op.valDec, op.valTxt, effectiveDate)
				if err != nil {
					return api.PayrollSettings{}, err
				}
			}
		} else {
			_, err = db.Exec(ctx, `DELETE FROM config_entries
WHERE scope_type = 'organization' AND scope_id IS NULL AND config_key = $1 AND effective_from = $2::date`, op.key, effectiveDate)
			if err != nil {
				return api.PayrollSettings{}, err
			}
		}
	}

	return api.PayrollSettings{
		StdWorkingDayMode:  mode,
		FixedStdDays:       fixedDays,
		WorkingDaysPerWeek: daysPerWeek,
		HoursPerDay:        hoursPerDay,
		MaxOtHoursPerDay:   maxOtDay,
		MaxOtHoursPerMonth: maxOtMonth,
		EffectiveDate:      effectiveDate,
		CreatedAt:          time.Now().UTC().Format(time.RFC3339),
		UpdatedAt:          time.Now().UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) GetEmployeePayrollConfigs(ctx context.Context, employeeID uuid.UUID) ([]api.EmployeePayrollConfig, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	var empExists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&empExists); err != nil {
		return nil, err
	}
	if !empExists {
		return nil, api.ErrEmployeeNotFound
	}

	var activeEmploymentID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID).Scan(&activeEmploymentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, api.ErrNoActiveEmployment
	}
	if err != nil {
		return nil, err
	}

	query := `SELECT id, effective_from::text, config_key, COALESCE(value_decimal, 0), created_at
FROM config_entries
WHERE scope_type = 'employment' AND scope_id = $1
  AND config_key IN ('payroll.fixed_std_days', 'payroll.hours_per_day')
ORDER BY effective_from DESC, created_at DESC`
	rows, err := db.Query(ctx, query, activeEmploymentID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	type entry struct {
		id        uuid.UUID
		date      string
		key       string
		val       decimal.Decimal
		createdAt time.Time
	}
	grouped := make(map[string]*api.EmployeePayrollConfig)
	order := make([]string, 0)

	for rows.Next() {
		var e entry
		if err := rows.Scan(&e.id, &e.date, &e.key, &e.val, &e.createdAt); err != nil {
			return nil, err
		}
		cfg, exists := grouped[e.date]
		if !exists {
			cfg = &api.EmployeePayrollConfig{
				ID:            e.id,
				EmploymentID:  activeEmploymentID,
				EffectiveDate: e.date,
				CreatedAt:     e.createdAt.UTC().Format(time.RFC3339),
			}
			grouped[e.date] = cfg
			order = append(order, e.date)
		}
		if e.key == "payroll.fixed_std_days" {
			cfg.StdWorkingDays = e.val
			cfg.ID = e.id
		} else if e.key == "payroll.hours_per_day" {
			cfg.HoursPerDay = e.val
		}
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	result := make([]api.EmployeePayrollConfig, 0, len(order))
	for _, d := range order {
		result = append(result, *grouped[d])
	}
	return result, nil
}

func (r PayrollRepository) SetEmployeePayrollConfig(ctx context.Context, employeeID uuid.UUID, req api.SetEmployeePayrollConfigRequest, actorID uuid.UUID) (api.EmployeePayrollConfig, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeePayrollConfig{}, err
	}
	var empExists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&empExists); err != nil {
		return api.EmployeePayrollConfig{}, err
	}
	if !empExists {
		return api.EmployeePayrollConfig{}, api.ErrEmployeeNotFound
	}

	var activeEmploymentID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID).Scan(&activeEmploymentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeePayrollConfig{}, api.ErrNoActiveEmployment
	}
	if err != nil {
		return api.EmployeePayrollConfig{}, err
	}

	upsertQuery := `INSERT INTO config_entries (id, scope_type, scope_id, config_key, value_decimal, effective_from, created_by, created_at, updated_at)
VALUES ($1, 'employment', $2, $3, $4, $5::date, $6, NOW(), NOW())
ON CONFLICT (scope_type, scope_id, config_key, effective_from)
DO UPDATE SET value_decimal = EXCLUDED.value_decimal, updated_at = NOW()
RETURNING id, created_at`

	fixedID := uuid.New()
	var finalID uuid.UUID
	var createdAt time.Time
	err = db.QueryRow(ctx, upsertQuery, fixedID, activeEmploymentID, "payroll.fixed_std_days", req.StdWorkingDays, req.EffectiveDate, actorID).Scan(&finalID, &createdAt)
	if err != nil {
		return api.EmployeePayrollConfig{}, err
	}

	hoursID := uuid.New()
	_, err = db.Exec(ctx, upsertQuery, hoursID, activeEmploymentID, "payroll.hours_per_day", req.HoursPerDay, req.EffectiveDate, actorID)
	if err != nil {
		return api.EmployeePayrollConfig{}, err
	}

	return api.EmployeePayrollConfig{
		ID:             finalID,
		EmploymentID:   activeEmploymentID,
		StdWorkingDays: req.StdWorkingDays,
		HoursPerDay:    req.HoursPerDay,
		EffectiveDate:  req.EffectiveDate,
		CreatedAt:      createdAt.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) GetIncentivePolicies(ctx context.Context) ([]api.PayrollIncentivePolicyResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, name, base_amount::text, payout_type, is_active, created_at, effective_from::text, effective_to::text
FROM payroll_incentive_policies
WHERE is_active = true AND deleted_at IS NULL
ORDER BY created_at ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var policies []api.PayrollIncentivePolicy
	var policyIDs []uuid.UUID
	for rows.Next() {
		var p api.PayrollIncentivePolicy
		var baseStr string
		var created time.Time
		var effTo *string
		if err := rows.Scan(&p.ID, &p.Name, &baseStr, &p.PayoutType, &p.IsActive, &created, &p.EffectiveFrom, &effTo); err != nil {
			return nil, err
		}
		p.BaseAmount, _ = decimal.NewFromString(baseStr)
		p.CreatedAt = created.UTC().Format(time.RFC3339)
		p.EffectiveTo = effTo
		policies = append(policies, p)
		policyIDs = append(policyIDs, p.ID)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	if len(policies) == 0 {
		return []api.PayrollIncentivePolicyResponse{}, nil
	}

	tierRows, err := db.Query(ctx, `SELECT id, policy_id, priority, payout_amount::text, rule_config
FROM payroll_incentive_tiers
WHERE policy_id = ANY($1)
ORDER BY priority ASC`, policyIDs)
	if err != nil {
		return nil, err
	}
	defer tierRows.Close()

	tiersMap := make(map[uuid.UUID][]api.IncentiveTier)
	for tierRows.Next() {
		var t api.IncentiveTier
		var payoutStr string
		if err := tierRows.Scan(&t.ID, &t.PolicyID, &t.Priority, &payoutStr, &t.RuleConfig); err != nil {
			return nil, err
		}
		t.PayoutAmount, _ = decimal.NewFromString(payoutStr)
		tiersMap[t.PolicyID] = append(tiersMap[t.PolicyID], t)
	}
	if err := tierRows.Err(); err != nil {
		return nil, err
	}

	result := make([]api.PayrollIncentivePolicyResponse, 0, len(policies))
	for _, p := range policies {
		tiers := tiersMap[p.ID]
		if tiers == nil {
			tiers = []api.IncentiveTier{}
		}
		result = append(result, api.PayrollIncentivePolicyResponse{
			PayrollIncentivePolicy: p,
			Tiers:                  tiers,
		})
	}
	return result, nil
}

func (r PayrollRepository) CreateIncentivePolicy(ctx context.Context, req api.CreatePayrollIncentivePolicyRequest) (api.PayrollIncentivePolicy, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	policyID := uuid.New()
	var createdAt time.Time
	payoutType := req.PayoutType
	if payoutType == "" {
		payoutType = "fixed"
	}
	err = db.QueryRow(ctx, `INSERT INTO payroll_incentive_policies (id, name, base_amount, payout_type, is_active, effective_from, effective_to, created_at, deleted_at)
VALUES ($1, $2, $3, $4, true, $5::date, NULL, NOW(), NULL)
RETURNING created_at`, policyID, strings.TrimSpace(req.Name), req.BaseAmount.InexactFloat64(), payoutType, req.EffectiveFrom).Scan(&createdAt)
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}

	for _, tier := range req.Tiers {
		tierID := uuid.New()
		ruleCfg := tier.RuleConfig
		if len(ruleCfg) == 0 {
			ruleCfg = []byte("{}")
		}
		_, err = db.Exec(ctx, `INSERT INTO payroll_incentive_tiers (id, policy_id, priority, payout_amount, rule_config)
VALUES ($1, $2, $3, $4, $5)`, tierID, policyID, tier.Priority, tier.PayoutAmount.InexactFloat64(), ruleCfg)
		if err != nil {
			return api.PayrollIncentivePolicy{}, err
		}
	}

	return api.PayrollIncentivePolicy{
		ID:            policyID,
		Name:          strings.TrimSpace(req.Name),
		BaseAmount:    req.BaseAmount,
		PayoutType:    payoutType,
		IsActive:      true,
		CreatedAt:     createdAt.UTC().Format(time.RFC3339),
		EffectiveFrom: req.EffectiveFrom,
		EffectiveTo:   nil,
	}, nil
}

func (r PayrollRepository) DeleteIncentivePolicy(ctx context.Context, policyID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE payroll_incentive_policies SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL`, policyID)
	return err
}

func (r PayrollRepository) CloneIncentivePolicy(ctx context.Context, policyID uuid.UUID) (api.PayrollIncentivePolicy, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	var origName, origPayout, origBaseStr string
	err = db.QueryRow(ctx, `SELECT name, payout_type, base_amount::text
FROM payroll_incentive_policies
WHERE id = $1 AND deleted_at IS NULL`, policyID).Scan(&origName, &origPayout, &origBaseStr)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.PayrollIncentivePolicy{}, api.ErrIncentivePolicyNotFound
	}
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	origBase, _ := decimal.NewFromString(origBaseStr)

	tierRows, err := db.Query(ctx, `SELECT priority, payout_amount::text, rule_config
FROM payroll_incentive_tiers
WHERE policy_id = $1
ORDER BY priority ASC`, policyID)
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	defer tierRows.Close()

	type tierToClone struct {
		priority int
		payout   decimal.Decimal
		config   []byte
	}
	var origTiers []tierToClone
	for tierRows.Next() {
		var priority int
		var payoutStr string
		var cfg []byte
		if err := tierRows.Scan(&priority, &payoutStr, &cfg); err != nil {
			return api.PayrollIncentivePolicy{}, err
		}
		payout, _ := decimal.NewFromString(payoutStr)
		origTiers = append(origTiers, tierToClone{priority: priority, payout: payout, config: cfg})
	}
	if err := tierRows.Err(); err != nil {
		return api.PayrollIncentivePolicy{}, err
	}

	newPolicyID := uuid.New()
	newName := fmt.Sprintf("%s (Copy)", origName)
	var createdAt time.Time
	var effFrom string
	err = db.QueryRow(ctx, `INSERT INTO payroll_incentive_policies (id, name, base_amount, payout_type, is_active, effective_from, effective_to, created_at, deleted_at)
VALUES ($1, $2, $3, $4, false, CURRENT_DATE, NULL, NOW(), NULL)
RETURNING created_at, effective_from::text`, newPolicyID, newName, origBase.InexactFloat64(), origPayout).Scan(&createdAt, &effFrom)
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}

	for _, t := range origTiers {
		tierID := uuid.New()
		cfg := t.config
		if len(cfg) == 0 {
			cfg = []byte("{}")
		}
		_, err = db.Exec(ctx, `INSERT INTO payroll_incentive_tiers (id, policy_id, priority, payout_amount, rule_config)
VALUES ($1, $2, $3, $4, $5)`, tierID, newPolicyID, t.priority, t.payout.InexactFloat64(), cfg)
		if err != nil {
			return api.PayrollIncentivePolicy{}, err
		}
	}

	return api.PayrollIncentivePolicy{
		ID:            newPolicyID,
		Name:          newName,
		BaseAmount:    origBase,
		PayoutType:    origPayout,
		IsActive:      false,
		CreatedAt:     createdAt.UTC().Format(time.RFC3339),
		EffectiveFrom: effFrom,
		EffectiveTo:   nil,
	}, nil
}

func (r PayrollRepository) CreatePayrollInputBatch(ctx context.Context, req api.CreatePayrollInputRequest, actorID uuid.UUID) (api.PayrollInputBatch, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollInputBatch{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollInputBatch{}, errors.New("payroll repository: database does not support transactions")
	}
	requestHash, err := payrollInputRequestHash(req)
	if err != nil {
		return api.PayrollInputBatch{}, err
	}

	var result api.PayrollInputBatch
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var existingHash string
		var existingID uuid.UUID
		err := tx.QueryRow(ctx, `SELECT id, request_hash FROM payroll_input_batches WHERE idempotency_key = $1 FOR UPDATE`, req.IdempotencyKey).Scan(&existingID, &existingHash)
		if err == nil {
			if existingHash != requestHash {
				return api.ErrPayrollInputConflict
			}
			batch, scanErr := scanPayrollInputBatch(tx.QueryRow(ctx, `SELECT b.id, b.period_year, b.period_month, b.source, b.idempotency_key, b.status, count(i.id), b.submitted_by, b.approved_by, b.locked_by, b.created_at
				FROM payroll_input_batches b LEFT JOIN payroll_inputs i ON i.batch_id = b.id
				WHERE b.id = $1 GROUP BY b.id`, existingID))
			if scanErr != nil {
				return scanErr
			}
			result = batch
			return nil
		}
		if !errors.Is(err, pgx.ErrNoRows) {
			return err
		}

		metricCodes := make([]string, 0, len(req.Inputs))
		employmentIDs := make([]uuid.UUID, 0, len(req.Inputs))
		for _, input := range req.Inputs {
			metricCodes = append(metricCodes, input.MetricCode)
			employmentIDs = append(employmentIDs, input.EmploymentID)
		}

		metricRows, err := tx.Query(ctx, `SELECT code, unit FROM payroll_metric_definitions WHERE code = ANY($1) AND is_active = true`, metricCodes)
		if err != nil {
			return err
		}
		metricUnits := make(map[string]string, len(metricCodes))
		for metricRows.Next() {
			var code, unit string
			if err := metricRows.Scan(&code, &unit); err != nil {
				metricRows.Close()
				return err
			}
			metricUnits[code] = unit
		}
		if err := metricRows.Err(); err != nil {
			metricRows.Close()
			return err
		}
		metricRows.Close()
		for _, input := range req.Inputs {
			unit, exists := metricUnits[input.MetricCode]
			if !exists || unit != input.Unit {
				return api.ErrPayrollValidation
			}
		}

		employmentRows, err := tx.Query(ctx, `SELECT id FROM employments WHERE id = ANY($1) AND deleted_at IS NULL`, employmentIDs)
		if err != nil {
			return err
		}
		validEmployments := make(map[uuid.UUID]struct{}, len(employmentIDs))
		for employmentRows.Next() {
			var id uuid.UUID
			if err := employmentRows.Scan(&id); err != nil {
				employmentRows.Close()
				return err
			}
			validEmployments[id] = struct{}{}
		}
		if err := employmentRows.Err(); err != nil {
			employmentRows.Close()
			return err
		}
		employmentRows.Close()
		if len(validEmployments) != len(uniqueUUIDs(employmentIDs)) {
			return api.ErrEmploymentNotFound
		}

		var locked bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (
			SELECT 1 FROM payroll_periods p
			JOIN employments e ON e.employer_id = p.employer_id
			WHERE e.id = ANY($1) AND p.period_year = $2 AND p.period_month = $3 AND p.status = 'locked'
		)`, employmentIDs, req.PeriodYear, req.PeriodMonth).Scan(&locked); err != nil {
			return err
		}
		if locked {
			return api.ErrPayrollInputConflict
		}

		batchID := uuid.New()
		if err := tx.QueryRow(ctx, `INSERT INTO payroll_input_batches (id, period_year, period_month, source, idempotency_key, request_hash, status, submitted_by)
			VALUES ($1, $2, $3, $4, $5, $6, 'submitted', $7)
			RETURNING id, period_year, period_month, source, idempotency_key, status, 0, submitted_by, approved_by, locked_by, created_at`, batchID, req.PeriodYear, req.PeriodMonth, strings.TrimSpace(req.Source), req.IdempotencyKey, requestHash, actorID).Scan(
			&result.ID, &result.PeriodYear, &result.PeriodMonth, &result.Source, &result.IdempotencyKey, &result.Status, &result.InputCount, &result.SubmittedBy, &result.ApprovedBy, &result.LockedBy, &result.CreatedAt); err != nil {
			if isConflictError(err) {
				return api.ErrPayrollInputConflict
			}
			return err
		}

		valueArgs := make([]any, 0, len(req.Inputs)*6)
		valueParts := make([]string, 0, len(req.Inputs))
		for index, input := range req.Inputs {
			base := index*6 + 1
			ref := any(nil)
			if input.SourceReference != nil {
				ref = *input.SourceReference
			}
			valueParts = append(valueParts, fmt.Sprintf("($%d, $%d, (SELECT id FROM payroll_metric_definitions WHERE code = $%d), $%d, $%d, $%d)", base, base+1, base+2, base+3, base+4, base+5))
			valueArgs = append(valueArgs, batchID, input.EmploymentID, input.MetricCode, input.Value.String(), input.Unit, ref)
		}
		_, err = tx.Exec(ctx, `INSERT INTO payroll_inputs (batch_id, employment_id, metric_definition_id, value, unit, source_reference) VALUES `+strings.Join(valueParts, ", "), valueArgs...)
		if err != nil {
			return err
		}
		result.InputCount = len(req.Inputs)
		return nil
	})
	if err != nil {
		return api.PayrollInputBatch{}, err
	}
	return result, nil
}

func scanPayrollInputBatch(row interface{ Scan(...any) error }) (api.PayrollInputBatch, error) {
	var batch api.PayrollInputBatch
	var created time.Time
	if err := row.Scan(&batch.ID, &batch.PeriodYear, &batch.PeriodMonth, &batch.Source, &batch.IdempotencyKey, &batch.Status, &batch.InputCount, &batch.SubmittedBy, &batch.ApprovedBy, &batch.LockedBy, &created); err != nil {
		return api.PayrollInputBatch{}, err
	}
	batch.CreatedAt = created.UTC().Format(time.RFC3339)
	return batch, nil
}

func payrollInputRequestHash(req api.CreatePayrollInputRequest) (string, error) {
	type hashInput struct {
		EmploymentID uuid.UUID       `json:"employment_id"`
		MetricCode   string          `json:"metric_code"`
		Value        decimal.Decimal `json:"value"`
		Unit         string          `json:"unit"`
		Reference    *string         `json:"source_reference,omitempty"`
	}
	inputs := make([]hashInput, 0, len(req.Inputs))
	for _, input := range req.Inputs {
		inputs = append(inputs, hashInput{input.EmploymentID, input.MetricCode, input.Value, input.Unit, input.SourceReference})
	}
	sort.Slice(inputs, func(i, j int) bool {
		if inputs[i].EmploymentID != inputs[j].EmploymentID {
			return inputs[i].EmploymentID.String() < inputs[j].EmploymentID.String()
		}
		return inputs[i].MetricCode < inputs[j].MetricCode
	})
	payload, err := json.Marshal(struct {
		Year   int         `json:"year"`
		Month  int         `json:"month"`
		Source string      `json:"source"`
		Inputs []hashInput `json:"inputs"`
	}{req.PeriodYear, req.PeriodMonth, strings.TrimSpace(req.Source), inputs})
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("%x", sha256.Sum256(payload)), nil
}

func (r PayrollRepository) CreatePayrollFormulaRule(ctx context.Context, req api.CreatePayrollFormulaRuleRequest, actorID uuid.UUID) (api.PayrollFormulaRule, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollFormulaRule{}, errors.New("payroll repository: database does not support transactions")
	}
	var result api.PayrollFormulaRule
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		ruleID := uuid.New()
		if _, err := tx.Exec(ctx, `INSERT INTO payroll_formula_rules (id, code, name, component_code, created_by) VALUES ($1, $2, $3, $4, $5)`, ruleID, strings.TrimSpace(req.Code), strings.TrimSpace(req.Name), strings.TrimSpace(req.ComponentCode), actorID); err != nil {
			if isConflictError(err) {
				return api.ErrConflict
			}
			return err
		}
		if err := tx.QueryRow(ctx, `INSERT INTO payroll_formula_rule_versions (rule_id, version_number, expression, effective_from, status, created_by) VALUES ($1, 1, $2, $3::date, 'draft', $4) RETURNING $5::uuid, $6::text, $7::text, $8::text, id, version_number, expression, effective_from::text, status`, ruleID, req.Expression, req.EffectiveFrom, actorID, ruleID, req.Code, req.Name, req.ComponentCode).Scan(
			&result.ID, &result.Code, &result.Name, &result.ComponentCode, &result.VersionID, &result.Version, &result.Expression, &result.EffectiveFrom, &result.Status); err != nil {
			return err
		}
		return nil
	})
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	return result, nil
}

func (r PayrollRepository) PublishPayrollFormulaRule(ctx context.Context, ruleID, actorID uuid.UUID) (api.PayrollFormulaRule, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	var result api.PayrollFormulaRule
	err = db.QueryRow(ctx, `UPDATE payroll_formula_rule_versions v
	SET status = 'published', published_by = $2, published_at = CURRENT_TIMESTAMP
	FROM payroll_formula_rules r
	WHERE v.rule_id = r.id AND r.id = $1 AND v.status = 'draft'
	RETURNING r.id, r.code, r.name, r.component_code, v.id, v.version_number, v.expression, v.effective_from::text, v.status`, ruleID, actorID).Scan(
		&result.ID, &result.Code, &result.Name, &result.ComponentCode, &result.VersionID, &result.Version, &result.Expression, &result.EffectiveFrom, &result.Status)
	if errors.Is(err, pgx.ErrNoRows) {
		var exists bool
		if existsErr := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_formula_rules WHERE id = $1)`, ruleID).Scan(&exists); existsErr != nil {
			return api.PayrollFormulaRule{}, existsErr
		}
		if !exists {
			return api.PayrollFormulaRule{}, api.ErrRuleNotFound
		}
		return api.PayrollFormulaRule{}, api.ErrConflict
	}
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	return result, nil
}

func (r PayrollRepository) transitionPayrollInputBatch(ctx context.Context, batchID, actorID uuid.UUID, fromStatus, toStatus string) (api.PayrollInputBatch, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollInputBatch{}, err
	}
	var batch api.PayrollInputBatch
	var created time.Time
	query := `UPDATE payroll_input_batches
	SET status = $1, approved_by = CASE WHEN $1 = 'approved' THEN $2 ELSE approved_by END,
	    locked_by = CASE WHEN $1 = 'locked' THEN $2 ELSE locked_by END, updated_at = CURRENT_TIMESTAMP
	WHERE id = $3 AND status = $4
	RETURNING id, period_year, period_month, source, idempotency_key, status, 0, submitted_by, approved_by, locked_by, created_at`
	if err := db.QueryRow(ctx, query, toStatus, actorID, batchID, fromStatus).Scan(
		&batch.ID, &batch.PeriodYear, &batch.PeriodMonth, &batch.Source, &batch.IdempotencyKey, &batch.Status, &batch.InputCount, &batch.SubmittedBy, &batch.ApprovedBy, &batch.LockedBy, &created); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			var exists bool
			if existsErr := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_input_batches WHERE id = $1)`, batchID).Scan(&exists); existsErr != nil {
				return api.PayrollInputBatch{}, existsErr
			}
			if !exists {
				return api.PayrollInputBatch{}, api.ErrPayrollInputNotFound
			}
			return api.PayrollInputBatch{}, api.ErrPayrollInputConflict
		}
		return api.PayrollInputBatch{}, err
	}
	batch.CreatedAt = created.UTC().Format(time.RFC3339)
	return batch, nil
}

func (r PayrollRepository) ApprovePayrollInputBatch(ctx context.Context, batchID, actorID uuid.UUID) (api.PayrollInputBatch, error) {
	return r.transitionPayrollInputBatch(ctx, batchID, actorID, "submitted", "approved")
}

func (r PayrollRepository) LockPayrollInputBatch(ctx context.Context, batchID, actorID uuid.UUID) (api.PayrollInputBatch, error) {
	return r.transitionPayrollInputBatch(ctx, batchID, actorID, "approved", "locked")
}

func uniqueUUIDs(ids []uuid.UUID) map[uuid.UUID]struct{} {
	unique := make(map[uuid.UUID]struct{}, len(ids))
	for _, id := range ids {
		unique[id] = struct{}{}
	}
	return unique
}

func (r PayrollRepository) ListPayrollRules(ctx context.Context) ([]api.PayrollRuleDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, name, description, code, category, applies_to, calculation_method, rule_config, is_mandatory, is_active, display_order, created_at, updated_at
FROM payroll_rule_definitions
ORDER BY display_order ASC, created_at DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.PayrollRuleDefinition, 0)
	for rows.Next() {
		var d api.PayrollRuleDefinition
		var desc *string
		var cfg []byte
		var created, updated time.Time
		if err := rows.Scan(&d.ID, &d.Name, &desc, &d.Code, &d.Category, &d.AppliesTo, &d.CalculationMethod, &cfg, &d.IsMandatory, &d.IsActive, &d.DisplayOrder, &created, &updated); err != nil {
			return nil, err
		}
		d.Description = desc
		d.RuleConfig = cfg
		d.CreatedAt = created.UTC().Format(time.RFC3339)
		d.UpdatedAt = updated.UTC().Format(time.RFC3339)
		result = append(result, d)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CreatePayrollRule(ctx context.Context, req api.CreatePayrollRuleDefinitionRequest) (api.PayrollRuleDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollRuleDefinition{}, err
	}

	cat := req.Category
	if cat == "" {
		cat = "deduction"
	}
	applies := req.AppliesTo
	if applies == "" {
		applies = "employee"
	}
	calcMethod := req.CalculationMethod
	if calcMethod == "" {
		calcMethod = "percentage"
	}
	cfg := req.RuleConfig
	if len(cfg) == 0 {
		cfg = []byte("{}")
	}
	isMandatory := false
	if req.IsMandatory != nil {
		isMandatory = *req.IsMandatory
	}
	displayOrder := 0
	if req.DisplayOrder != nil {
		displayOrder = *req.DisplayOrder
	}

	ruleID := uuid.New()
	var created, updated time.Time
	err = db.QueryRow(ctx, `INSERT INTO payroll_rule_definitions (id, name, description, code, category, applies_to, calculation_method, rule_config, is_mandatory, is_active, display_order, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10, NOW(), NOW())
RETURNING created_at, updated_at`, ruleID, strings.TrimSpace(req.Name), req.Description, strings.TrimSpace(req.Code), cat, applies, calcMethod, cfg, isMandatory, displayOrder).Scan(&created, &updated)
	if err != nil {
		if isConflictError(err) {
			return api.PayrollRuleDefinition{}, api.ErrConflict
		}
		return api.PayrollRuleDefinition{}, err
	}

	return api.PayrollRuleDefinition{
		ID:                ruleID,
		Name:              strings.TrimSpace(req.Name),
		Description:       req.Description,
		Code:              strings.TrimSpace(req.Code),
		Category:          cat,
		AppliesTo:         applies,
		CalculationMethod: calcMethod,
		RuleConfig:        cfg,
		IsMandatory:       isMandatory,
		IsActive:          true,
		DisplayOrder:      displayOrder,
		CreatedAt:         created.UTC().Format(time.RFC3339),
		UpdatedAt:         updated.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) ListPayrollRuleTiers(ctx context.Context, ruleID uuid.UUID) ([]api.PayrollRuleTier, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, ruleID).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return nil, api.ErrRuleNotFound
	}

	rows, err := db.Query(ctx, `SELECT id, rule_definition_id, tier_order, min_amount::text, max_amount::text, rate::text, fixed_amount::text, created_at
FROM payroll_rule_tiers
WHERE rule_definition_id = $1
ORDER BY tier_order ASC`, ruleID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.PayrollRuleTier, 0)
	for rows.Next() {
		var t api.PayrollRuleTier
		var minStr, rateStr, fixedStr string
		var maxStr *string
		var created time.Time
		if err := rows.Scan(&t.ID, &t.RuleDefinitionID, &t.TierOrder, &minStr, &maxStr, &rateStr, &fixedStr, &created); err != nil {
			return nil, err
		}
		t.MinAmount, _ = decimal.NewFromString(minStr)
		if maxStr != nil {
			maxDec, _ := decimal.NewFromString(*maxStr)
			t.MaxAmount = &maxDec
		}
		t.Rate, _ = decimal.NewFromString(rateStr)
		t.FixedAmount, _ = decimal.NewFromString(fixedStr)
		t.CreatedAt = created.UTC().Format(time.RFC3339)
		result = append(result, t)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CreatePayrollRuleTier(ctx context.Context, ruleID uuid.UUID, req api.CreatePayrollRuleTierRequest) (api.PayrollRuleTier, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollRuleTier{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, ruleID).Scan(&exists); err != nil {
		return api.PayrollRuleTier{}, err
	}
	if !exists {
		return api.PayrollRuleTier{}, api.ErrRuleNotFound
	}
	if req.MaxAmount != nil && req.MaxAmount.LessThanOrEqual(req.MinAmount) {
		return api.PayrollRuleTier{}, api.ErrPayrollValidation
	}

	fixed := decimal.Zero
	if req.FixedAmount != nil {
		fixed = *req.FixedAmount
	}

	tierID := uuid.New()
	var created time.Time
	var maxParam any
	if req.MaxAmount != nil {
		maxParam = req.MaxAmount.String()
	}
	err = db.QueryRow(ctx, `INSERT INTO payroll_rule_tiers (id, rule_definition_id, tier_order, min_amount, max_amount, rate, fixed_amount, created_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
RETURNING created_at`, tierID, ruleID, req.TierOrder, req.MinAmount.String(), maxParam, req.Rate.String(), fixed.String()).Scan(&created)
	if err != nil {
		if isConflictError(err) {
			return api.PayrollRuleTier{}, api.ErrConflict
		}
		return api.PayrollRuleTier{}, err
	}

	return api.PayrollRuleTier{
		ID:               tierID,
		RuleDefinitionID: ruleID,
		TierOrder:        req.TierOrder,
		MinAmount:        req.MinAmount,
		MaxAmount:        req.MaxAmount,
		Rate:             req.Rate,
		FixedAmount:      fixed,
		CreatedAt:        created.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) DeletePayrollRuleTier(ctx context.Context, ruleID, tierID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `DELETE FROM payroll_rule_tiers WHERE id = $1 AND rule_definition_id = $2`, tierID, ruleID)
	return err
}

func (r PayrollRepository) ListPayrollRuleAssignments(ctx context.Context, ruleID uuid.UUID) ([]api.PayrollRuleAssignment, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, ruleID).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return nil, api.ErrRuleNotFound
	}

	rows, err := db.Query(ctx, `SELECT id, rule_definition_id, assignee_type, assignee_id, effective_from::text, effective_to::text, override_config, created_at
FROM payroll_rule_assignments
WHERE rule_definition_id = $1
ORDER BY created_at DESC`, ruleID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.PayrollRuleAssignment, 0)
	for rows.Next() {
		var a api.PayrollRuleAssignment
		var assigneeID *uuid.UUID
		var effTo *string
		var overrideCfg []byte
		var created time.Time
		if err := rows.Scan(&a.ID, &a.RuleDefinitionID, &a.AssigneeType, &assigneeID, &a.EffectiveFrom, &effTo, &overrideCfg, &created); err != nil {
			return nil, err
		}
		a.AssigneeID = assigneeID
		a.EffectiveTo = effTo
		if len(overrideCfg) > 0 {
			raw := json.RawMessage(overrideCfg)
			a.OverrideConfig = &raw
		}
		a.CreatedAt = created.UTC().Format(time.RFC3339)
		result = append(result, a)
	}
	return result, rows.Err()
}

func (r PayrollRepository) AssignPayrollRule(ctx context.Context, ruleID uuid.UUID, req api.AssignPayrollRuleRequest) (api.PayrollRuleAssignment, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PayrollRuleAssignment{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, ruleID).Scan(&exists); err != nil {
		return api.PayrollRuleAssignment{}, err
	}
	if !exists {
		return api.PayrollRuleAssignment{}, api.ErrRuleNotFound
	}

	// Organization-wide assignments are implied by the singleton organization,
	// so they carry no target row identifier.
	assigneeID := req.AssigneeID
	if req.AssigneeType == "organization" {
		assigneeID = nil
	}

	assignID := uuid.New()
	var created time.Time
	var overrideParam any
	if req.OverrideConfig != nil && len(*req.OverrideConfig) > 0 {
		overrideParam = []byte(*req.OverrideConfig)
	}
	err = db.QueryRow(ctx, `INSERT INTO payroll_rule_assignments (id, rule_definition_id, assignee_type, assignee_id, effective_from, effective_to, override_config, created_at)
VALUES ($1, $2, $3, $4, $5::date, $6::date, $7, NOW())
RETURNING created_at`, assignID, ruleID, req.AssigneeType, assigneeID, req.EffectiveFrom, req.EffectiveTo, overrideParam).Scan(&created)
	if err != nil {
		if isConflictError(err) {
			return api.PayrollRuleAssignment{}, api.ErrConflict
		}
		return api.PayrollRuleAssignment{}, err
	}

	return api.PayrollRuleAssignment{
		ID:               assignID,
		RuleDefinitionID: ruleID,
		AssigneeType:     req.AssigneeType,
		AssigneeID:       assigneeID,
		EffectiveFrom:    req.EffectiveFrom,
		EffectiveTo:      req.EffectiveTo,
		OverrideConfig:   req.OverrideConfig,
		CreatedAt:        created.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) DeletePayrollRuleAssignment(ctx context.Context, ruleID, assignmentID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `DELETE FROM payroll_rule_assignments WHERE id = $1 AND rule_definition_id = $2`, assignmentID, ruleID)
	return err
}

func (r PayrollRepository) CalculateEmployeePayroll(ctx context.Context, employeeID uuid.UUID, year, month int) (api.FullPayslipResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.FullPayslipResponse{}, errors.New("payroll repository: database does not support transactions")
	}

	var response api.FullPayslipResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var empExists bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&empExists); err != nil {
			return err
		}
		if !empExists {
			return api.ErrEmployeeNotFound
		}

		var employmentID, employerID uuid.UUID
		var deptID *uuid.UUID
		err := tx.QueryRow(ctx, `SELECT id, department_id, employer_id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID).Scan(&employmentID, &deptID, &employerID)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrNoActiveEmployment
		}
		if err != nil {
			return err
		}
		if err := ensurePayrollPeriodOpen(ctx, tx, employerID, year, month); err != nil {
			return err
		}
		response, err = r.calculateEmploymentPayslip(ctx, tx, employmentID, employeeID, deptID, year, month)
		return err
	})
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	return response, nil
}

func ensurePayrollPeriodOpen(ctx context.Context, tx pgx.Tx, employerID uuid.UUID, year, month int) error {
	if year < 2000 || year > 2100 || month < 1 || month > 12 {
		return api.ErrPayrollPeriodValidation
	}
	if _, err := tx.Exec(ctx, `
		INSERT INTO payroll_periods (employer_id, period_year, period_month)
		VALUES ($1, $2, $3)
		ON CONFLICT (employer_id, period_year, period_month) DO NOTHING`, employerID, year, month); err != nil {
		return mapPayrollPeriodError(err)
	}
	var status string
	if err := tx.QueryRow(ctx, `SELECT status FROM payroll_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3 FOR UPDATE`, employerID, year, month).Scan(&status); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrPayrollPeriodNotFound
		}
		return err
	}
	if status != "open" {
		return api.ErrPayrollPeriodConflict
	}
	return nil
}

func (r PayrollRepository) CalculateBatchPayroll(ctx context.Context, req api.BatchCalculatePayrollRequest) (api.BatchCalculatePayrollResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.BatchCalculatePayrollResponse{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.BatchCalculatePayrollResponse{}, errors.New("payroll repository: database does not support transactions")
	}

	seen := make(map[uuid.UUID]bool)
	deduped := make([]uuid.UUID, 0, len(req.EmploymentIDs))
	for _, id := range req.EmploymentIDs {
		if !seen[id] {
			seen[id] = true
			deduped = append(deduped, id)
		}
	}

	var response api.BatchCalculatePayrollResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		response.Results = make([]api.BatchCalculateResult, 0, len(deduped))
		for _, employmentID := range deduped {
			var employeeID, employerID uuid.UUID
			var deptID *uuid.UUID
			var status string
			err := tx.QueryRow(ctx, `SELECT employee_id, employer_id, department_id, status FROM employments WHERE id = $1 AND deleted_at IS NULL`, employmentID).Scan(&employeeID, &employerID, &deptID, &status)
			if err != nil {
				msg := "employment not found"
				response.Results = append(response.Results, api.BatchCalculateResult{EmploymentID: employmentID, Status: "error", ErrorMessage: &msg})
				continue
			}
			if err := ensurePayrollPeriodOpen(ctx, tx, employerID, req.Year, req.Month); err != nil {
				msg := err.Error()
				response.Results = append(response.Results, api.BatchCalculateResult{EmploymentID: employmentID, EmployeeID: employeeID, Status: "error", ErrorMessage: &msg})
				continue
			}
			fullResp, err := r.calculateEmploymentPayslip(ctx, tx, employmentID, employeeID, deptID, req.Year, req.Month)
			if err != nil {
				msg := err.Error()
				response.Results = append(response.Results, api.BatchCalculateResult{EmploymentID: employmentID, EmployeeID: employeeID, Status: "error", ErrorMessage: &msg})
				continue
			}
			payslipCopy := fullResp.PayslipRecord
			response.Results = append(response.Results, api.BatchCalculateResult{EmploymentID: employmentID, EmployeeID: employeeID, Status: "success", Payslip: &payslipCopy})
		}
		return nil
	})
	if err != nil {
		return api.BatchCalculatePayrollResponse{}, err
	}
	return response, nil
}

func (r PayrollRepository) calculateEmploymentPayslip(ctx context.Context, db DBTX, employmentID, employeeID uuid.UUID, deptID *uuid.UUID, year, month int) (api.FullPayslipResponse, error) {
	startDate, endDate, err := payroll.MonthBoundaries(year, month)
	if err != nil {
		return api.FullPayslipResponse{}, api.ErrPayrollValidation
	}
	startStr := startDate.Format("2006-01-02")
	endStr := endDate.Format("2006-01-02")

	type salaryRow struct {
		escID     uuid.UUID
		defID     uuid.UUID
		amount    decimal.Decimal
		effDate   string
		escCreate time.Time
		defName   string
		defCode   string
		defCreate time.Time
		defUpdate time.Time
	}
	salaryRows, err := db.Query(ctx, `SELECT esc.id, esc.component_definition_id, esc.amount::text, esc.effective_date::text, esc.created_at, scd.name, scd.code, scd.created_at, scd.updated_at
FROM employee_salary_components esc
JOIN salary_component_definitions scd ON scd.id = esc.component_definition_id
WHERE esc.employment_id = $1 AND esc.effective_date <= $2::date
ORDER BY esc.effective_date ASC, esc.id ASC`, employmentID, endStr)
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	defer salaryRows.Close()

	var salaryHistory []salaryRow
	var salaryChanges []time.Time
	activeDefMap := make(map[uuid.UUID]salaryRow)

	for salaryRows.Next() {
		var rRow salaryRow
		var amtStr string
		if err := salaryRows.Scan(&rRow.escID, &rRow.defID, &amtStr, &rRow.effDate, &rRow.escCreate, &rRow.defName, &rRow.defCode, &rRow.defCreate, &rRow.defUpdate); err != nil {
			return api.FullPayslipResponse{}, err
		}
		rRow.amount, _ = decimal.NewFromString(amtStr)
		salaryHistory = append(salaryHistory, rRow)
		activeDefMap[rRow.defID] = rRow

		if t, err := time.Parse("2006-01-02", rRow.effDate); err == nil {
			if t.After(startDate) && t.Before(endDate.AddDate(0, 0, 1)) {
				salaryChanges = append(salaryChanges, t)
			}
		}
	}
	if err := salaryRows.Err(); err != nil {
		return api.FullPayslipResponse{}, err
	}

	totalBaseSalary := decimal.Zero
	salaryComponents := make([]api.SalaryComponentWithDefinition, 0, len(activeDefMap))
	for _, item := range activeDefMap {
		totalBaseSalary = totalBaseSalary.Add(item.amount)
		salaryComponents = append(salaryComponents, api.SalaryComponentWithDefinition{
			Component: api.EmployeeSalaryComponent{
				ID:                    item.escID,
				EmploymentID:          employmentID,
				ComponentDefinitionID: item.defID,
				Amount:                item.amount,
				EffectiveDate:         item.effDate,
				CreatedAt:             item.escCreate.UTC().Format(time.RFC3339),
			},
			Definition: api.SalaryComponentDefinition{
				ID:        item.defID,
				Name:      item.defName,
				Code:      item.defCode,
				CreatedAt: item.defCreate.UTC().Format(time.RFC3339),
				UpdatedAt: item.defUpdate.UTC().Format(time.RFC3339),
			},
		})
	}

	var deptChanges []time.Time
	currentDept := deptID
	deptRows, err := db.Query(ctx, `SELECT department_id, start_date::text
FROM employments
WHERE employee_id = $1 AND deleted_at IS NULL AND department_id IS NOT NULL AND start_date <= $2::date
ORDER BY start_date ASC, id ASC`, employeeID, endStr)
	if err == nil {
		defer deptRows.Close()
		for deptRows.Next() {
			var dID *uuid.UUID
			var effStr string
			if err := deptRows.Scan(&dID, &effStr); err == nil {
				if t, err := time.Parse("2006-01-02", effStr); err == nil {
					if !t.After(startDate) {
						currentDept = dID
					} else if t.Before(endDate.AddDate(0, 0, 1)) {
						deptChanges = append(deptChanges, t)
					}
				}
			}
		}
	}

	segmentsList := payroll.BuildSegments(startDate, endDate, salaryChanges, deptChanges)
	if len(segmentsList) == 0 {
		segmentsList = []payroll.DateRange{{Start: startDate, End: endDate}}
	}

	stdDays := decimal.NewFromInt(26)
	hoursPerDay := decimal.NewFromInt(8)
	var maxOtHours *decimal.Decimal

	orgCfgRows, err := db.Query(ctx, `SELECT config_key, value_decimal::text
FROM config_entries
WHERE scope_type = 'organization' AND scope_id IS NULL AND effective_from <= $1::date
ORDER BY effective_from DESC, created_at DESC`, endStr)
	if err == nil {
		defer orgCfgRows.Close()
		seenKeys := make(map[string]bool)
		for orgCfgRows.Next() {
			var k string
			var valDec *string
			if err := orgCfgRows.Scan(&k, &valDec); err == nil && !seenKeys[k] {
				seenKeys[k] = true
				if valDec != nil {
					if d, err := decimal.NewFromString(*valDec); err == nil {
						switch k {
						case "payroll.fixed_std_days":
							if d.GreaterThan(decimal.Zero) {
								stdDays = d
							}
						case "payroll.hours_per_day":
							if d.GreaterThan(decimal.Zero) {
								hoursPerDay = d
							}
						case "payroll.max_ot_hours_per_month":
							maxOtHours = &d
						}
					}
				}
			}
		}
	}

	cfgRows, err := db.Query(ctx, `SELECT config_key, value_decimal::text
FROM config_entries
WHERE scope_type = 'employment' AND scope_id = $1 AND effective_from <= $2::date
ORDER BY effective_from DESC`, employmentID, endStr)
	if err == nil {
		defer cfgRows.Close()
		seenKeys := make(map[string]bool)
		for cfgRows.Next() {
			var k, val string
			if err := cfgRows.Scan(&k, &val); err == nil && !seenKeys[k] {
				seenKeys[k] = true
				if d, err := decimal.NewFromString(val); err == nil {
					if k == "payroll.fixed_std_days" && d.GreaterThan(decimal.Zero) {
						stdDays = d
					} else if k == "payroll.hours_per_day" && d.GreaterThan(decimal.Zero) {
						hoursPerDay = d
					}
				}
			}
		}
	}

	type summaryItem struct {
		stdMin int
		otMin  int
	}
	summariesMap := make(map[string]summaryItem)
	sumRows, err := db.Query(ctx, `SELECT date::text, COALESCE(standard_minutes, 0), COALESCE(early_ot_minutes + late_ot_minutes, 0)
FROM attendance_daily_summaries
WHERE employment_id = $1 AND date >= $2::date AND date <= $3::date`, employmentID, startStr, endStr)
	if err == nil {
		defer sumRows.Close()
		for sumRows.Next() {
			var dStr string
			var sMin, oMin int
			if err := sumRows.Scan(&dStr, &sMin, &oMin); err == nil {
				summariesMap[dStr] = summaryItem{stdMin: sMin, otMin: oMin}
			}
		}
	}

	totalBasePay := decimal.Zero
	var totalStdUnits int32
	var totalOtUnits int32
	stdSource := "organization_settings"

	type segSnapshot struct {
		id           uuid.UUID
		start        time.Time
		end          time.Time
		appliedBase  decimal.Decimal
		stdDays      decimal.Decimal
		hoursPerDay  decimal.Decimal
		actualDays   decimal.Decimal
		amount       decimal.Decimal
		reason       string
		source       *string
		departmentID *uuid.UUID
	}
	var segmentSnapshots []segSnapshot

	minutesPerDay := hoursPerDay.Mul(decimal.NewFromInt(60))
	if minutesPerDay.IsZero() {
		minutesPerDay = decimal.NewFromInt(480)
	}

	for _, seg := range segmentsList {
		segBase := decimal.Zero
		seenDef := make(map[uuid.UUID]bool)
		for i := len(salaryHistory) - 1; i >= 0; i-- {
			item := salaryHistory[i]
			if !seenDef[item.defID] {
				if t, err := time.Parse("2006-01-02", item.effDate); err == nil && !t.After(seg.Start) {
					segBase = segBase.Add(item.amount)
					seenDef[item.defID] = true
				}
			}
		}
		if segBase.IsZero() {
			segBase = totalBaseSalary
		}

		dailyRate := rates.CalculateDailyRate(segBase, stdDays)
		segStdMin := 0
		segOtMin := 0

		curr := seg.Start
		for !curr.After(seg.End) {
			dStr := curr.Format("2006-01-02")
			if item, ok := summariesMap[dStr]; ok {
				segStdMin += item.stdMin
				segOtMin += item.otMin
			}
			curr = curr.AddDate(0, 0, 1)
		}

		actualDays := decimal.Zero
		if segStdMin > 0 {
			actualDays = decimal.NewFromInt(int64(segStdMin)).Div(minutesPerDay)
		}
		segAmount := dailyRate.Mul(actualDays).Round(2)
		totalBasePay = totalBasePay.Add(segAmount)

		stdUnits := int32(segStdMin / 60)
		otUnits := int32(segOtMin / 60)
		totalStdUnits += stdUnits
		totalOtUnits += otUnits

		reason := "Standard"
		if seg.Start.Day() != 1 {
			if segBase.GreaterThan(decimal.Zero) {
				reason = "Salary Change"
			} else {
				reason = "Adjustment"
			}
		}

		segmentSnapshots = append(segmentSnapshots, segSnapshot{
			id:           uuid.New(),
			start:        seg.Start,
			end:          seg.End,
			appliedBase:  segBase,
			stdDays:      stdDays,
			hoursPerDay:  hoursPerDay,
			actualDays:   actualDays,
			amount:       segAmount,
			reason:       reason,
			source:       &stdSource,
			departmentID: currentDept,
		})
	}

	hourlyRate := rates.CalculateHourlyRate(totalBaseSalary, stdDays, hoursPerDay)
	totalOvertimePay := hourlyRate.Mul(decimal.NewFromInt(int64(totalOtUnits))).Mul(decimal.RequireFromString("1.5")).Round(2)
	if maxOtHours != nil {
		cappedUnits, cappedPay := rates.ApplyOTCap(totalOtUnits, totalOvertimePay, *maxOtHours, 60)
		totalOtUnits = cappedUnits
		totalOvertimePay = cappedPay
	}

	totalHolidayPremium := decimal.Zero

	var incentiveResults []api.IncentiveEvaluationResult
	totalIncentives := decimal.Zero

	incRows, err := db.Query(ctx, `SELECT p.id, p.name, p.base_amount::text, p.payout_type
FROM payroll_incentive_policies p
WHERE p.is_active = true AND p.deleted_at IS NULL
  AND p.effective_from <= $1::date AND (p.effective_to IS NULL OR p.effective_to >= $2::date)
ORDER BY p.created_at ASC`, endStr, startStr)
	if err == nil {
		defer incRows.Close()
		type incPolicyInfo struct {
			id         uuid.UUID
			name       string
			baseAmount decimal.Decimal
			payoutType string
		}
		var policiesToEval []incPolicyInfo
		var policyIDs []uuid.UUID
		for incRows.Next() {
			var p incPolicyInfo
			var bStr string
			if err := incRows.Scan(&p.id, &p.name, &bStr, &p.payoutType); err == nil {
				p.baseAmount, _ = decimal.NewFromString(bStr)
				policiesToEval = append(policiesToEval, p)
				policyIDs = append(policyIDs, p.id)
			}
		}

		if len(policyIDs) > 0 {
			tierRows, err := db.Query(ctx, `SELECT id, policy_id, priority, payout_amount::text, rule_config
FROM payroll_incentive_tiers
WHERE policy_id = ANY($1)
ORDER BY priority ASC`, policyIDs)
			if err == nil {
				defer tierRows.Close()
				tiersByPolicy := make(map[uuid.UUID][]incentives.Tier)
				for tierRows.Next() {
					var t incentives.Tier
					var pid uuid.UUID
					var pStr string
					var cfg []byte
					if err := tierRows.Scan(&t.ID, &pid, &t.Priority, &pStr, &cfg); err == nil {
						t.PayoutAmount, _ = decimal.NewFromString(pStr)
						t.RuleConfig = cfg
						tiersByPolicy[pid] = append(tiersByPolicy[pid], t)
					}
				}

				incCtx := incentives.Context{
					TotalWorkUnits: totalStdUnits,
				}
				for _, p := range policiesToEval {
					policyObj := incentives.Policy{
						Name:       p.name,
						BaseAmount: p.baseAmount,
						PayoutType: p.payoutType,
					}
					evalRes, err := incentives.EvaluatePolicy(policyObj, tiersByPolicy[p.id], incCtx)
					if err == nil && evalRes.Amount.GreaterThan(decimal.Zero) {
						totalIncentives = totalIncentives.Add(evalRes.Amount)
						incentiveResults = append(incentiveResults, api.IncentiveEvaluationResult{
							PolicyName:     evalRes.PolicyName,
							Amount:         evalRes.Amount,
							StandardAmount: evalRes.StandardAmount,
							TierID:         evalRes.TierID,
						})
					}
				}
			}
		}
	}

	var ruleEvaluationRecords []api.PayslipRuleEvaluationRecord
	totalDeductions := decimal.Zero

	ruleRows, err := db.Query(ctx, `SELECT a.rule_definition_id, a.override_config, d.code, d.calculation_method, d.rule_config
FROM payroll_rule_assignments a
JOIN payroll_rule_definitions d ON d.id = a.rule_definition_id
WHERE (
    (a.assignee_type = 'organization')
    OR (a.assignee_type = 'department' AND a.assignee_id = $1)
    OR (a.assignee_type = 'employee' AND a.assignee_id = $2)
  )
  AND a.effective_from <= $3::date AND (a.effective_to IS NULL OR a.effective_to >= $4::date)
  AND d.is_active = true`, currentDept, employeeID, endStr, startStr)
	if err == nil {
		defer ruleRows.Close()
		type ruleAssignInfo struct {
			ruleID      uuid.UUID
			overrideCfg []byte
			code        string
			calcMethod  string
			defaultCfg  []byte
		}
		var assigns []ruleAssignInfo
		var tieredRuleIDs []uuid.UUID
		for ruleRows.Next() {
			var rInfo ruleAssignInfo
			var overCfg, defCfg []byte
			if err := ruleRows.Scan(&rInfo.ruleID, &overCfg, &rInfo.code, &rInfo.calcMethod, &defCfg); err == nil {
				rInfo.overrideCfg = overCfg
				rInfo.defaultCfg = defCfg
				assigns = append(assigns, rInfo)
				if rInfo.calcMethod == "tiered" {
					tieredRuleIDs = append(tieredRuleIDs, rInfo.ruleID)
				}
			}
		}

		tiersByRule := make(map[uuid.UUID][]rules.Tier)
		if len(tieredRuleIDs) > 0 {
			tRows, err := db.Query(ctx, `SELECT rule_definition_id, tier_order, min_amount::text, max_amount::text, rate::text, fixed_amount::text
FROM payroll_rule_tiers
WHERE rule_definition_id = ANY($1)
ORDER BY tier_order ASC`, tieredRuleIDs)
			if err == nil {
				defer tRows.Close()
				for tRows.Next() {
					var rid uuid.UUID
					var t rules.Tier
					var minStr, rateStr, fixStr string
					var maxStr *string
					if err := tRows.Scan(&rid, &t.TierOrder, &minStr, &maxStr, &rateStr, &fixStr); err == nil {
						t.MinAmount, _ = decimal.NewFromString(minStr)
						if maxStr != nil {
							mDec, _ := decimal.NewFromString(*maxStr)
							t.MaxAmount = &mDec
						}
						t.Rate, _ = decimal.NewFromString(rateStr)
						t.FixedAmount, _ = decimal.NewFromString(fixStr)
						tiersByRule[rid] = append(tiersByRule[rid], t)
					}
				}
			}
		}

		gross := totalBasePay.Add(totalOvertimePay).Add(totalHolidayPremium).Add(totalIncentives)
		baseValues := map[string]decimal.Decimal{
			"gross":     gross,
			"base_pay":  totalBasePay,
			"bhxh_base": totalBasePay,
		}

		domainAssignments := make([]rules.Assignment, 0, len(assigns))
		for _, a := range assigns {
			domainAssignments = append(domainAssignments, rules.Assignment{
				Code:              a.code,
				CalculationMethod: a.calcMethod,
				DefaultConfig:     a.defaultCfg,
				OverrideConfig:    a.overrideCfg,
				Tiers:             tiersByRule[a.ruleID],
			})
		}

		ruleResults, err := rules.EvaluateRules(domainAssignments, baseValues)
		if err == nil {
			for _, res := range ruleResults {
				totalDeductions = totalDeductions.Add(res.Amount)
				ruleEvaluationRecords = append(ruleEvaluationRecords, api.PayslipRuleEvaluationRecord{
					ID:        uuid.New(),
					RuleCode:  res.RuleCode,
					Amount:    res.Amount,
					CreatedAt: time.Now().UTC().Format(time.RFC3339),
				})
			}
		}
	}

	payslipID := uuid.New()
	assembled := payroll.AssemblePayslip(payroll.AssemblePayslipArgs{
		ID:                      payslipID,
		EmploymentID:            employmentID,
		StartDate:               startDate,
		EndDate:                 endDate,
		TotalBasePay:            totalBasePay,
		TotalOvertimePay:        totalOvertimePay,
		TotalHolidayWorkPremium: totalHolidayPremium,
		TotalIncentiveAmount:    totalIncentives,
		DeductionsAmount:        totalDeductions,
		TotalStdUnits:           totalStdUnits,
		TotalOTUnits:            totalOtUnits,
		CreatedAt:               time.Now().UTC(),
		UpdatedAt:               time.Now().UTC(),
	})

	var existingID uuid.UUID
	var existingStatus string
	err = db.QueryRow(ctx, `SELECT id, status FROM payslips WHERE employment_id = $1 AND period_start = $2::date FOR UPDATE`, employmentID, startStr).Scan(&existingID, &existingStatus)
	var finalPayslipID uuid.UUID
	var createdTime, updatedTime time.Time

	if err == nil {
		if existingStatus == "paid" || existingStatus == "finalized" {
			return api.FullPayslipResponse{}, api.ErrPayslipConflict
		}
		finalPayslipID = existingID
		if _, err := db.Exec(ctx, `DELETE FROM payslip_segments WHERE payslip_id = $1`, finalPayslipID); err != nil {
			return api.FullPayslipResponse{}, err
		}
		if _, err := db.Exec(ctx, `DELETE FROM payslip_incentives WHERE payslip_id = $1`, finalPayslipID); err != nil {
			return api.FullPayslipResponse{}, err
		}
		if _, err := db.Exec(ctx, `DELETE FROM payslip_rule_evaluations WHERE payslip_id = $1`, finalPayslipID); err != nil {
			return api.FullPayslipResponse{}, err
		}

		err = db.QueryRow(ctx, `UPDATE payslips SET
base_pay_amount = $1, overtime_pay_amount = $2, deductions_amount = $3, net_pay_amount = $4, status = 'draft', standard_units = $5, overtime_units = $6, holiday_work_premium_amount = $7, updated_at = NOW()
WHERE id = $8 RETURNING created_at, updated_at`,
			assembled.TotalBasePay.String(), assembled.TotalOvertimePay.String(), assembled.DeductionsAmount.String(), assembled.NetPay.String(), assembled.TotalStdUnits, assembled.TotalOTUnits, assembled.TotalHolidayWorkPremium.String(), finalPayslipID).Scan(&createdTime, &updatedTime)
		if err != nil {
			return api.FullPayslipResponse{}, err
		}
	} else if errors.Is(err, pgx.ErrNoRows) {
		finalPayslipID = payslipID
		err = db.QueryRow(ctx, `INSERT INTO payslips (id, employment_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, status, standard_units, overtime_units, holiday_work_premium_amount, created_at, updated_at)
VALUES ($1, $2, $3::date, $4::date, $5, $6, $7, $8, 'draft', $9, $10, $11, NOW(), NOW())
RETURNING created_at, updated_at`,
			finalPayslipID, employmentID, startStr, endStr, assembled.TotalBasePay.String(), assembled.TotalOvertimePay.String(), assembled.DeductionsAmount.String(), assembled.NetPay.String(), assembled.TotalStdUnits, assembled.TotalOTUnits, assembled.TotalHolidayWorkPremium.String()).Scan(&createdTime, &updatedTime)
		if err != nil {
			return api.FullPayslipResponse{}, err
		}
	} else {
		return api.FullPayslipResponse{}, err
	}

	segmentResponses := make([]api.PayslipSegmentResponse, 0, len(segmentSnapshots))
	for _, snap := range segmentSnapshots {
		segID := snap.id
		_, err := db.Exec(ctx, `INSERT INTO payslip_segments (id, payslip_id, start_date, end_date, department_id, applied_base_salary, applied_std_days, actual_work_days, paid_work_days, amount, created_at, applied_hours_per_day, std_days_source)
VALUES ($1, $2, $3::date, $4::date, $5, $6, $7, $8, $8, $9, NOW(), $10, $11)`,
			segID, finalPayslipID, snap.start.Format("2006-01-02"), snap.end.Format("2006-01-02"), snap.departmentID, snap.appliedBase.String(), snap.stdDays.String(), snap.actualDays.String(), snap.amount.String(), snap.hoursPerDay.String(), snap.source)
		if err != nil {
			return api.FullPayslipResponse{}, err
		}
		segmentResponses = append(segmentResponses, api.PayslipSegmentResponse{
			ID:                 segID,
			PayslipID:          finalPayslipID,
			StartDate:          snap.start.Format("2006-01-02"),
			EndDate:            snap.end.Format("2006-01-02"),
			DepartmentID:       snap.departmentID,
			AppliedBaseSalary:  snap.appliedBase,
			AppliedStdDays:     snap.stdDays,
			AppliedHoursPerDay: snap.hoursPerDay,
			ActualWorkDays:     snap.actualDays,
			PaidWorkDays:       snap.actualDays,
			Amount:             snap.amount,
			Reason:             snap.reason,
			StdDaysSource:      snap.source,
		})
	}

	for i := range incentiveResults {
		incID := uuid.New()
		if incentiveResults[i].TierID != nil {
			_, err = db.Exec(ctx, `INSERT INTO payslip_incentives (id, payslip_id, policy_name, amount, tier_id, standard_amount, created_at)
VALUES ($1, $2, $3, $4, $5, $6, NOW())`, incID, finalPayslipID, incentiveResults[i].PolicyName, incentiveResults[i].Amount.InexactFloat64(), incentiveResults[i].TierID, incentiveResults[i].StandardAmount.InexactFloat64())
		} else {
			_, err = db.Exec(ctx, `INSERT INTO payslip_incentives (id, payslip_id, policy_name, amount, standard_amount, created_at)
VALUES ($1, $2, $3, $4, $5, NOW())`, incID, finalPayslipID, incentiveResults[i].PolicyName, incentiveResults[i].Amount.InexactFloat64(), incentiveResults[i].StandardAmount.InexactFloat64())
		}
		if err != nil {
			return api.FullPayslipResponse{}, err
		}
	}

	for i := range ruleEvaluationRecords {
		ruleEvaluationRecords[i].PayslipID = finalPayslipID
		_, err = db.Exec(ctx, `INSERT INTO payslip_rule_evaluations (id, payslip_id, rule_code, amount, created_at)
VALUES ($1, $2, $3, $4, NOW())`, ruleEvaluationRecords[i].ID, finalPayslipID, ruleEvaluationRecords[i].RuleCode, ruleEvaluationRecords[i].Amount.String())
		if err != nil {
			return api.FullPayslipResponse{}, err
		}
	}

	return api.FullPayslipResponse{
		PayslipRecord: api.PayslipRecord{
			ID:                       finalPayslipID,
			EmploymentID:             employmentID,
			PeriodStart:              startStr,
			PeriodEnd:                endStr,
			BasePayAmount:            assembled.TotalBasePay,
			OvertimePayAmount:        assembled.TotalOvertimePay,
			DeductionsAmount:         assembled.DeductionsAmount,
			NetPayAmount:             assembled.NetPay,
			Status:                   "draft",
			StandardUnits:            int(assembled.TotalStdUnits),
			OvertimeUnits:            int(assembled.TotalOTUnits),
			HolidayWorkPremiumAmount: assembled.TotalHolidayWorkPremium,
			CreatedAt:                createdTime.UTC().Format(time.RFC3339),
			UpdatedAt:                updatedTime.UTC().Format(time.RFC3339),
		},
		Segments:         segmentResponses,
		Incentives:       incentiveResults,
		RuleEvaluations:  ruleEvaluationRecords,
		SalaryComponents: salaryComponents,
	}, nil
}
