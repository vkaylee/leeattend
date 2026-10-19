package repository

import (
	"context"
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
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
