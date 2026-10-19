package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/timezone"
)

type PayrollRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
	SystemDB DBTX
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

func (r PayrollRepository) ListSalaryComponents(ctx context.Context, wsID uuid.UUID) ([]api.SalaryComponentDefinition, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, name, code, created_at, updated_at FROM salary_component_definitions WHERE workspace_id = $1 ORDER BY name ASC, code ASC`, wsID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.SalaryComponentDefinition, 0)
	for rows.Next() {
		var d api.SalaryComponentDefinition
		var created, updated time.Time
		if err := rows.Scan(&d.ID, &d.WorkspaceID, &d.Name, &d.Code, &created, &updated); err != nil {
			return nil, err
		}
		d.CreatedAt = created.UTC().Format(time.RFC3339)
		d.UpdatedAt = updated.UTC().Format(time.RFC3339)
		result = append(result, d)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CreateSalaryComponent(ctx context.Context, wsID uuid.UUID, req api.CreateSalaryComponentRequest) (api.SalaryComponentDefinition, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.SalaryComponentDefinition{}, err
	}
	if req.BaseGroupIDs != nil && len(*req.BaseGroupIDs) > 0 {
		uniqueIDs := make([]uuid.UUID, 0, len(*req.BaseGroupIDs))
		seen := make(map[uuid.UUID]struct{})
		for _, id := range *req.BaseGroupIDs {
			if _, exists := seen[id]; !exists {
				seen[id] = struct{}{}
				uniqueIDs = append(uniqueIDs, id)
			}
		}
		var validCount int
		err := db.QueryRow(ctx, `SELECT count(DISTINCT id) FROM salary_base_groups WHERE id = ANY($1) AND (workspace_id = $2 OR workspace_id IS NULL)`, uniqueIDs, wsID).Scan(&validCount)
		if err != nil {
			return api.SalaryComponentDefinition{}, err
		}
		if validCount != len(uniqueIDs) {
			return api.SalaryComponentDefinition{}, api.ErrInvalidBaseGroup
		}
	}

	id := uuid.New()
	var def api.SalaryComponentDefinition
	var created, updated time.Time
	err = db.QueryRow(ctx, `INSERT INTO salary_component_definitions (id, workspace_id, name, code, component_type, created_at, updated_at) VALUES ($1, $2, $3, $4, 'earning', NOW(), NOW()) RETURNING id, workspace_id, name, code, created_at, updated_at`, id, wsID, strings.TrimSpace(req.Name), strings.TrimSpace(req.Code)).Scan(&def.ID, &def.WorkspaceID, &def.Name, &def.Code, &created, &updated)
	if err != nil {
		return api.SalaryComponentDefinition{}, mapConstraintError(err)
	}
	def.CreatedAt = created.UTC().Format(time.RFC3339)
	def.UpdatedAt = updated.UTC().Format(time.RFC3339)

	if req.BaseGroupIDs != nil && len(*req.BaseGroupIDs) > 0 {
		for _, bgID := range *req.BaseGroupIDs {
			_, err = db.Exec(ctx, `INSERT INTO salary_component_base_members (component_definition_id, base_group_id, created_at) VALUES ($1, $2, NOW()) ON CONFLICT (component_definition_id, base_group_id) DO NOTHING`, def.ID, bgID)
			if err != nil {
				return api.SalaryComponentDefinition{}, err
			}
		}
	}
	return def, nil
}

func (r PayrollRepository) ListEmploymentSalaryComponents(ctx context.Context, wsID, employmentID uuid.UUID) ([]api.EmploymentSalaryComponentResponse, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	tz := time.UTC
	if r.SystemDB != nil {
		var tzName string
		if err := r.SystemDB.QueryRow(ctx, `SELECT timezone FROM workspaces WHERE id = $1`, wsID).Scan(&tzName); err == nil && tzName != "" {
			tz = timezone.ParseWorkspaceTZ(tzName)
		}
	}
	refDate := time.Now().In(tz).Format("2006-01-02")

	rows, err := db.Query(ctx, `SELECT DISTINCT ON (esc.employment_id, esc.component_definition_id)
		esc.id, esc.employment_id, esc.component_definition_id, esc.amount::text, esc.effective_date::text,
		scd.name, scd.code
	FROM employee_salary_components esc
	INNER JOIN salary_component_definitions scd ON scd.id = esc.component_definition_id
	WHERE esc.employment_id = $1
	  AND esc.effective_date <= $2
	  AND scd.workspace_id = $3
	ORDER BY esc.employment_id, esc.component_definition_id, esc.effective_date DESC, esc.created_at DESC`, employmentID, refDate, wsID)
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

func (r PayrollRepository) AssignEmploymentSalaryComponent(ctx context.Context, wsID, employmentID uuid.UUID, req api.AssignSalaryComponentRequest) (api.EmployeeSalaryComponent, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.EmployeeSalaryComponent{}, err
	}

	var compDefExists bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM salary_component_definitions WHERE id = $1 AND workspace_id = $2)`, req.ComponentDefinitionID, wsID).Scan(&compDefExists)
	if err != nil {
		return api.EmployeeSalaryComponent{}, err
	}
	if !compDefExists {
		return api.EmployeeSalaryComponent{}, api.ErrComponentNotFound
	}

	var empExists bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, employmentID, wsID).Scan(&empExists)
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

func (r PayrollRepository) ListSalaryBases(ctx context.Context, wsID uuid.UUID) (api.SalaryBasesResponse, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.SalaryBasesResponse{}, err
	}

	groupRows, err := db.Query(ctx, `SELECT id, workspace_id, name, slug, description, is_system, created_at, updated_at FROM salary_base_groups WHERE workspace_id = $1 OR workspace_id IS NULL ORDER BY is_system DESC, name ASC`, wsID)
	if err != nil {
		return api.SalaryBasesResponse{}, err
	}
	defer groupRows.Close()

	groups := make([]api.SalaryBaseGroup, 0)
	for groupRows.Next() {
		var g api.SalaryBaseGroup
		var created, updated time.Time
		if err := groupRows.Scan(&g.ID, &g.WorkspaceID, &g.Name, &g.Slug, &g.Description, &g.IsSystem, &created, &updated); err != nil {
			return api.SalaryBasesResponse{}, err
		}
		g.CreatedAt = created.UTC().Format(time.RFC3339)
		g.UpdatedAt = updated.UTC().Format(time.RFC3339)
		groups = append(groups, g)
	}
	if err := groupRows.Err(); err != nil {
		return api.SalaryBasesResponse{}, err
	}

	memberRows, err := db.Query(ctx, `SELECT scbm.component_definition_id, scbm.base_group_id, scbm.created_at
FROM salary_component_base_members scbm
INNER JOIN salary_base_groups sbg ON sbg.id = scbm.base_group_id
WHERE sbg.workspace_id = $1 OR sbg.workspace_id IS NULL
ORDER BY scbm.created_at ASC`, wsID)
	if err != nil {
		return api.SalaryBasesResponse{}, err
	}
	defer memberRows.Close()

	members := make([]api.SalaryBaseGroupMember, 0)
	for memberRows.Next() {
		var m api.SalaryBaseGroupMember
		var created time.Time
		if err := memberRows.Scan(&m.ComponentDefinitionID, &m.BaseGroupID, &created); err != nil {
			return api.SalaryBasesResponse{}, err
		}
		m.CreatedAt = created.UTC().Format(time.RFC3339)
		members = append(members, m)
	}
	if err := memberRows.Err(); err != nil {
		return api.SalaryBasesResponse{}, err
	}

	return api.SalaryBasesResponse{Groups: groups, Members: members}, nil
}

func (r PayrollRepository) CreateSalaryBase(ctx context.Context, wsID uuid.UUID, req api.CreateSalaryBaseRequest) (api.SalaryBaseGroup, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.SalaryBaseGroup{}, err
	}

	id := uuid.New()
	var g api.SalaryBaseGroup
	var created, updated time.Time
	err = db.QueryRow(ctx, `INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, FALSE, NOW(), NOW()) RETURNING id, workspace_id, name, slug, description, is_system, created_at, updated_at`, id, wsID, strings.TrimSpace(req.Name), strings.TrimSpace(req.Slug), req.Description).Scan(&g.ID, &g.WorkspaceID, &g.Name, &g.Slug, &g.Description, &g.IsSystem, &created, &updated)
	if err != nil {
		return api.SalaryBaseGroup{}, mapConstraintError(err)
	}
	g.CreatedAt = created.UTC().Format(time.RFC3339)
	g.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return g, nil
}

func (r PayrollRepository) AssignBaseMember(ctx context.Context, wsID, baseID, componentID uuid.UUID) error {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return err
	}

	var validGroup bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM salary_base_groups WHERE id = $1 AND workspace_id = $2)`, baseID, wsID).Scan(&validGroup)
	if err != nil {
		return err
	}
	if !validGroup {
		return api.ErrInvalidBaseGroup
	}

	var validComp bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM salary_component_definitions WHERE id = $1 AND workspace_id = $2)`, componentID, wsID).Scan(&validComp)
	if err != nil {
		return err
	}
	if !validComp {
		return api.ErrComponentNotFound
	}

	_, err = db.Exec(ctx, `INSERT INTO salary_component_base_members (component_definition_id, base_group_id, created_at) VALUES ($1, $2, NOW()) ON CONFLICT (component_definition_id, base_group_id) DO NOTHING`, componentID, baseID)
	return err
}

func (r PayrollRepository) UnassignBaseMember(ctx context.Context, wsID, baseID, componentID uuid.UUID) error {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return err
	}

	_, err = db.Exec(ctx, `DELETE FROM salary_component_base_members
WHERE component_definition_id = $1
  AND base_group_id = $2
  AND base_group_id IN (SELECT id FROM salary_base_groups WHERE id = $2 AND workspace_id = $3)`, componentID, baseID, wsID)
	return err
}

func (r PayrollRepository) GetPayrollSettings(ctx context.Context, wsID uuid.UUID) (*api.PayrollSettings, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	tz := time.UTC
	if r.SystemDB != nil {
		var tzName string
		if err := r.SystemDB.QueryRow(ctx, `SELECT timezone FROM workspaces WHERE id = $1`, wsID).Scan(&tzName); err == nil && tzName != "" {
			tz = timezone.ParseWorkspaceTZ(tzName)
		}
	}
	today := time.Now().In(tz).Format("2006-01-02")

	rows, err := db.Query(ctx, `SELECT config_key, value_int, value_decimal, value_text, effective_from::text, created_at, updated_at
FROM workspace_config_entries
WHERE workspace_id = $1 AND scope_type = 'workspace' AND scope_id IS NULL AND effective_from <= $2::date
ORDER BY effective_from DESC, created_at DESC`, wsID, today)
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
		WorkspaceID:        wsID,
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

func (r PayrollRepository) GetPayrollSettingsHistory(ctx context.Context, wsID uuid.UUID) ([]api.PayrollSettings, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT config_key, value_int, value_decimal, value_text, effective_from::text, created_at, updated_at
FROM workspace_config_entries
WHERE workspace_id = $1 AND scope_type = 'workspace' AND scope_id IS NULL
ORDER BY effective_from DESC, created_at DESC`, wsID)
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
			WorkspaceID:        wsID,
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

func (r PayrollRepository) UpdatePayrollSettings(ctx context.Context, wsID uuid.UUID, req api.UpdatePayrollSettingsRequest) (api.PayrollSettings, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.PayrollSettings{}, err
	}
	tz := time.UTC
	if r.SystemDB != nil {
		var tzName string
		if err := r.SystemDB.QueryRow(ctx, `SELECT timezone FROM workspaces WHERE id = $1`, wsID).Scan(&tzName); err == nil && tzName != "" {
			tz = timezone.ParseWorkspaceTZ(tzName)
		}
	}
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
FROM workspace_config_entries
WHERE workspace_id = $1 AND scope_type = 'workspace' AND scope_id IS NULL AND effective_from <= $2::date
ORDER BY effective_from DESC, created_at DESC`, wsID, effectiveDate)
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
			err := db.QueryRow(ctx, `SELECT id FROM workspace_config_entries
WHERE workspace_id = $1 AND scope_type = 'workspace' AND scope_id IS NULL AND config_key = $2 AND effective_from = $3::date`, wsID, op.key, effectiveDate).Scan(&existingID)
			if err == nil {
				_, err = db.Exec(ctx, `UPDATE workspace_config_entries
SET value_int = $1, value_decimal = $2, value_text = $3, updated_at = NOW()
WHERE id = $4`, op.valInt, op.valDec, op.valTxt, existingID)
				if err != nil {
					return api.PayrollSettings{}, err
				}
			} else {
				newID := uuid.New()
				_, err = db.Exec(ctx, `INSERT INTO workspace_config_entries (id, workspace_id, scope_type, scope_id, config_key, value_int, value_decimal, value_text, effective_from, created_at, updated_at)
VALUES ($1, $2, 'workspace', NULL, $3, $4, $5, $6, $7::date, NOW(), NOW())`, newID, wsID, op.key, op.valInt, op.valDec, op.valTxt, effectiveDate)
				if err != nil {
					return api.PayrollSettings{}, err
				}
			}
		} else {
			_, err = db.Exec(ctx, `DELETE FROM workspace_config_entries
WHERE workspace_id = $1 AND scope_type = 'workspace' AND scope_id IS NULL AND config_key = $2 AND effective_from = $3::date`, wsID, op.key, effectiveDate)
			if err != nil {
				return api.PayrollSettings{}, err
			}
		}
	}

	return api.PayrollSettings{
		WorkspaceID:        wsID,
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

func (r PayrollRepository) GetEmployeePayrollConfigs(ctx context.Context, wsID, employeeID uuid.UUID) ([]api.EmployeePayrollConfig, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	var empExists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, employeeID, wsID).Scan(&empExists); err != nil {
		return nil, err
	}
	if !empExists {
		return nil, api.ErrEmployeeNotFound
	}

	var activeEmploymentID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID, wsID).Scan(&activeEmploymentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, api.ErrNoActiveEmployment
	}
	if err != nil {
		return nil, err
	}

	query := `SELECT id, effective_from::text, config_key, COALESCE(value_decimal, 0), created_at
FROM workspace_config_entries
WHERE workspace_id = $1 AND scope_type = 'employment' AND scope_id = $2
  AND config_key IN ('payroll.fixed_std_days', 'payroll.hours_per_day')
ORDER BY effective_from DESC, created_at DESC`
	rows, err := db.Query(ctx, query, wsID, activeEmploymentID)
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

func (r PayrollRepository) SetEmployeePayrollConfig(ctx context.Context, wsID, employeeID uuid.UUID, req api.SetEmployeePayrollConfigRequest, actorID uuid.UUID) (api.EmployeePayrollConfig, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.EmployeePayrollConfig{}, err
	}
	var empExists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, employeeID, wsID).Scan(&empExists); err != nil {
		return api.EmployeePayrollConfig{}, err
	}
	if !empExists {
		return api.EmployeePayrollConfig{}, api.ErrEmployeeNotFound
	}

	var activeEmploymentID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID, wsID).Scan(&activeEmploymentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeePayrollConfig{}, api.ErrNoActiveEmployment
	}
	if err != nil {
		return api.EmployeePayrollConfig{}, err
	}

	upsertQuery := `INSERT INTO workspace_config_entries (id, workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from, created_by, created_at, updated_at)
VALUES ($1, $2, 'employment', $3, $4, $5, $6::date, $7, NOW(), NOW())
ON CONFLICT (workspace_id, scope_type, scope_id, config_key, effective_from)
DO UPDATE SET value_decimal = EXCLUDED.value_decimal, updated_at = NOW()
RETURNING id, created_at`

	fixedID := uuid.New()
	var finalID uuid.UUID
	var createdAt time.Time
	err = db.QueryRow(ctx, upsertQuery, fixedID, wsID, activeEmploymentID, "payroll.fixed_std_days", req.StdWorkingDays, req.EffectiveDate, actorID).Scan(&finalID, &createdAt)
	if err != nil {
		return api.EmployeePayrollConfig{}, err
	}

	hoursID := uuid.New()
	_, err = db.Exec(ctx, upsertQuery, hoursID, wsID, activeEmploymentID, "payroll.hours_per_day", req.HoursPerDay, req.EffectiveDate, actorID)
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

func (r PayrollRepository) GetIncentivePolicies(ctx context.Context, wsID uuid.UUID) ([]api.PayrollIncentivePolicyResponse, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, name, base_amount::text, payout_type, is_active, created_at, effective_from::text, effective_to::text
FROM payroll_incentive_policies
WHERE workspace_id = $1 AND is_active = true AND deleted_at IS NULL
ORDER BY created_at ASC`, wsID)
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
		if err := rows.Scan(&p.ID, &p.WorkspaceID, &p.Name, &baseStr, &p.PayoutType, &p.IsActive, &created, &p.EffectiveFrom, &effTo); err != nil {
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

func (r PayrollRepository) CreateIncentivePolicy(ctx context.Context, wsID uuid.UUID, req api.CreatePayrollIncentivePolicyRequest) (api.PayrollIncentivePolicy, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	policyID := uuid.New()
	var createdAt time.Time
	payoutType := req.PayoutType
	if payoutType == "" {
		payoutType = "fixed"
	}
	err = db.QueryRow(ctx, `INSERT INTO payroll_incentive_policies (id, workspace_id, name, base_amount, payout_type, is_active, effective_from, effective_to, created_at, deleted_at)
VALUES ($1, $2, $3, $4, $5, true, $6::date, NULL, NOW(), NULL)
RETURNING created_at`, policyID, wsID, strings.TrimSpace(req.Name), req.BaseAmount.InexactFloat64(), payoutType, req.EffectiveFrom).Scan(&createdAt)
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
		WorkspaceID:   wsID,
		Name:          strings.TrimSpace(req.Name),
		BaseAmount:    req.BaseAmount,
		PayoutType:    payoutType,
		IsActive:      true,
		CreatedAt:     createdAt.UTC().Format(time.RFC3339),
		EffectiveFrom: req.EffectiveFrom,
		EffectiveTo:   nil,
	}, nil
}

func (r PayrollRepository) DeleteIncentivePolicy(ctx context.Context, wsID, policyID uuid.UUID) error {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE payroll_incentive_policies SET deleted_at = NOW() WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL`, policyID, wsID)
	return err
}

func (r PayrollRepository) CloneIncentivePolicy(ctx context.Context, wsID, policyID uuid.UUID) (api.PayrollIncentivePolicy, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	var origName, origPayout, origBaseStr string
	err = db.QueryRow(ctx, `SELECT name, payout_type, base_amount::text
FROM payroll_incentive_policies
WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL`, policyID, wsID).Scan(&origName, &origPayout, &origBaseStr)
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
	err = db.QueryRow(ctx, `INSERT INTO payroll_incentive_policies (id, workspace_id, name, base_amount, payout_type, is_active, effective_from, effective_to, created_at, deleted_at)
VALUES ($1, $2, $3, $4, $5, false, CURRENT_DATE, NULL, NOW(), NULL)
RETURNING created_at, effective_from::text`, newPolicyID, wsID, newName, origBase.InexactFloat64(), origPayout).Scan(&createdAt, &effFrom)
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
		WorkspaceID:   wsID,
		Name:          newName,
		BaseAmount:    origBase,
		PayoutType:    origPayout,
		IsActive:      false,
		CreatedAt:     createdAt.UTC().Format(time.RFC3339),
		EffectiveFrom: effFrom,
		EffectiveTo:   nil,
	}, nil
}
