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
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
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

func (r PayrollRepository) ListSalaryComponents(ctx context.Context) ([]api.SalaryComponentDefinition, error) {
	db, err := requireDB(r.DB, "payroll repository")
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
	db, err := requireDB(r.DB, "payroll repository")
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

func (r PayrollRepository) ListEmploymentSalaryComponents(ctx context.Context, employmentID ids.EmploymentID) ([]api.EmploymentSalaryComponentResponse, error) {
	db, err := requireDB(r.DB, "payroll repository")
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
	ORDER BY esc.employment_id, esc.component_definition_id, esc.effective_date DESC, esc.created_at DESC`, uuid.UUID(employmentID), refDate)
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

func (r PayrollRepository) AssignEmploymentSalaryComponent(ctx context.Context, employmentID ids.EmploymentID, req api.AssignSalaryComponentRequest) (api.EmployeeSalaryComponent, error) {
	db, err := requireDB(r.DB, "payroll repository")
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
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND deleted_at IS NULL)`, uuid.UUID(employmentID)).Scan(&empExists)
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
	err = db.QueryRow(ctx, `INSERT INTO employee_salary_components (id, employment_id, component_definition_id, amount, effective_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) RETURNING id, employment_id, component_definition_id, amount::text, effective_date::text, created_at, updated_at`, id, uuid.UUID(employmentID), req.ComponentDefinitionID, req.Amount, req.EffectiveDate).Scan(&item.ID, &item.EmploymentID, &item.ComponentDefinitionID, &amt, &item.EffectiveDate, &created, &updated)
	if err != nil {
		return api.EmployeeSalaryComponent{}, mapConstraintError(err)
	}
	item.Amount, _ = decimal.NewFromString(amt)
	item.CreatedAt = created.UTC().Format(time.RFC3339)
	item.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return item, nil
}

func (r PayrollRepository) GetIncentivePolicies(ctx context.Context) ([]api.PayrollIncentivePolicyResponse, error) {
	db, err := requireDB(r.DB, "payroll repository")
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
	db, err := requireDB(r.DB, "payroll repository")
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

func (r PayrollRepository) DeleteIncentivePolicy(ctx context.Context, policyID ids.PayrollIncentivePolicyID) error {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE payroll_incentive_policies SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL`, uuid.UUID(policyID))
	return err
}

func (r PayrollRepository) CloneIncentivePolicy(ctx context.Context, policyID ids.PayrollIncentivePolicyID) (api.PayrollIncentivePolicy, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollIncentivePolicy{}, err
	}
	var origName, origPayout, origBaseStr string
	err = db.QueryRow(ctx, `SELECT name, payout_type, base_amount::text
FROM payroll_incentive_policies
WHERE id = $1 AND deleted_at IS NULL`, uuid.UUID(policyID)).Scan(&origName, &origPayout, &origBaseStr)
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
ORDER BY priority ASC`, uuid.UUID(policyID))
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

func (r PayrollRepository) CreatePayrollInputBatch(ctx context.Context, req api.CreatePayrollInputRequest, actorID ids.UserID) (api.PayrollInputBatch, error) {
	db, err := requireDB(r.DB, "payroll repository")
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
			RETURNING id, period_year, period_month, source, idempotency_key, status, 0, submitted_by, approved_by, locked_by, created_at`, batchID, req.PeriodYear, req.PeriodMonth, strings.TrimSpace(req.Source), req.IdempotencyKey, requestHash, uuid.UUID(actorID)).Scan(
			&result.ID, &result.PeriodYear, &result.PeriodMonth, &result.Source, &result.IdempotencyKey, &result.Status, &result.InputCount, &result.SubmittedBy, &result.ApprovedBy, &result.LockedBy, &result.CreatedAt); err != nil {
			if isConflictError(err) {
				return api.ErrPayrollInputConflict
			}
			return err
		}

		valueArgs := make([]any, 0, len(req.Inputs)*6)
		valueParts := make([]string, 0, len(req.Inputs))
		for index, input := range req.Inputs {
			base := index*8 + 1
			ref := any(nil)
			if input.SourceReference != nil {
				ref = *input.SourceReference
			}
			status, validationMessage := payrollInputRowStatus(input.Value)

			valueParts = append(valueParts, fmt.Sprintf("($%d, $%d, (SELECT id FROM payroll_metric_definitions WHERE code = $%d), $%d, $%d, $%d, $%d, $%d)", base, base+1, base+2, base+3, base+4, base+5, base+6, base+7))
			valueArgs = append(valueArgs, batchID, input.EmploymentID, input.MetricCode, input.Value.String(), input.Unit, ref, status, validationMessage)
		}
		_, err = tx.Exec(ctx, `INSERT INTO payroll_inputs (batch_id, employment_id, metric_definition_id, value, unit, source_reference, status, validation_message) VALUES `+strings.Join(valueParts, ", "), valueArgs...)
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

func payrollInputRowStatus(value decimal.Decimal) (string, any) {
	if value.IsZero() {
		return "warning", "zero metric value requires review"
	}
	return "accepted", nil
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

func (r PayrollRepository) ListPayrollInputs(ctx context.Context, batchID ids.PayrollInputBatchID) ([]api.PayrollInput, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT i.id, i.employment_id, m.code, i.value::text, i.unit, i.status, i.validation_message, i.source_reference
FROM payroll_inputs i JOIN payroll_metric_definitions m ON m.id = i.metric_definition_id
WHERE i.batch_id = $1 ORDER BY i.id ASC`, uuid.UUID(batchID))
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PayrollInput, 0)
	for rows.Next() {
		var input api.PayrollInput
		var raw string
		if err := rows.Scan(&input.ID, &input.EmploymentID, &input.MetricCode, &raw, &input.Unit, &input.Status, &input.ValidationMessage, &input.SourceReference); err != nil {
			return nil, err
		}
		input.Value, err = decimal.NewFromString(raw)
		if err != nil {
			return nil, err
		}
		result = append(result, input)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CorrectPayrollInput(ctx context.Context, inputID ids.PayrollInputID, req api.CorrectPayrollInputRequest, actorID ids.UserID) (api.PayrollInput, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollInput{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollInput{}, errors.New("payroll repository: database does not support transactions")
	}
	var result api.PayrollInput
	var correctedValue string
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var oldValue, oldUnit, oldStatus string
		var batchStatus string
		if err := tx.QueryRow(ctx, `SELECT i.value::text, i.unit, i.status, b.status FROM payroll_inputs i JOIN payroll_input_batches b ON b.id = i.batch_id WHERE i.id = $1 FOR UPDATE`, uuid.UUID(inputID)).Scan(&oldValue, &oldUnit, &oldStatus, &batchStatus); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrPayrollInputNotFound
			}
			return err
		}
		if batchStatus == "locked" {
			return api.ErrPayrollInputConflict
		}
		var oldDecimal decimal.Decimal
		oldDecimal, err = decimal.NewFromString(oldValue)
		if err != nil {
			return err
		}
		if _, err := tx.Exec(ctx, `INSERT INTO payroll_input_corrections (input_id, old_value, new_value, old_unit, new_unit, reason, corrected_by) VALUES ($1, $2, $3, $4, $5, $6, $7)`, uuid.UUID(inputID), oldDecimal.String(), req.Value.String(), oldUnit, strings.TrimSpace(req.Unit), strings.TrimSpace(req.Reason), uuid.UUID(actorID)); err != nil {
			return err
		}
		return tx.QueryRow(ctx, `UPDATE payroll_inputs SET value = $1, unit = $2, status = 'corrected', validation_message = NULL, corrected_at = CURRENT_TIMESTAMP, corrected_by = $3 WHERE id = $4 RETURNING id, employment_id, (SELECT code FROM payroll_metric_definitions WHERE id = metric_definition_id), value::text, unit, status, validation_message, source_reference`, req.Value.String(), strings.TrimSpace(req.Unit), uuid.UUID(actorID), uuid.UUID(inputID)).Scan(&result.ID, &result.EmploymentID, &result.MetricCode, &correctedValue, &result.Unit, &result.Status, &result.ValidationMessage, &result.SourceReference)
	})
	if err != nil {
		return api.PayrollInput{}, err
	}
	result.Value, err = decimal.NewFromString(correctedValue)
	return result, err
}

func (r PayrollRepository) ListPayrollCalculationRuns(ctx context.Context, employmentID ids.EmploymentID, year, month, limit int) ([]api.PayrollCalculationRun, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	if limit <= 0 || limit > 100 {
		limit = 50
	}
	rows, err := db.Query(ctx, `SELECT id, employment_id, payslip_id, period_year, period_month, formula_version_id, engine_version, input_snapshot, output_snapshot, calculation_trace, created_by, created_at
FROM payroll_calculation_runs
WHERE employment_id = $1 AND period_year = $2 AND period_month = $3
ORDER BY created_at DESC, id DESC
LIMIT $4`, uuid.UUID(employmentID), year, month, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PayrollCalculationRun, 0)
	for rows.Next() {
		var run api.PayrollCalculationRun
		var created time.Time
		if err := rows.Scan(&run.ID, &run.EmploymentID, &run.PayslipID, &run.PeriodYear, &run.PeriodMonth, &run.FormulaVersionID, &run.EngineVersion, &run.InputSnapshot, &run.OutputSnapshot, &run.CalculationTrace, &run.CreatedBy, &created); err != nil {
			return nil, err
		}
		run.CreatedAt = created.UTC().Format(time.RFC3339)
		result = append(result, run)
	}
	return result, rows.Err()
}

func hasDuplicatePayrollImpactEmploymentIDs(values []uuid.UUID) bool {
	seen := make(map[uuid.UUID]struct{}, len(values))
	for _, value := range values {
		if _, exists := seen[value]; exists {
			return true
		}
		seen[value] = struct{}{}
	}
	return false
}

func (r PayrollRepository) ComparePayrollImpactBatch(ctx context.Context, req api.PayrollImpactBatchRequest) (api.PayrollImpactBatchResponse, error) {
	if len(req.EmploymentIDs) == 0 || len(req.EmploymentIDs) > 100 || hasDuplicatePayrollImpactEmploymentIDs(req.EmploymentIDs) || req.BeforeRunID == uuid.Nil || req.AfterRunID == uuid.Nil || req.BeforeRunID == req.AfterRunID {
		return api.PayrollImpactBatchResponse{}, api.ErrPayrollImpactValidation
	}
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollImpactBatchResponse{}, err
	}
	rows, err := db.Query(ctx, `SELECT employment_id, id, formula_version_id, input_snapshot, output_snapshot
FROM payroll_calculation_runs
WHERE employment_id = ANY($1::uuid[]) AND period_year = $2 AND period_month = $3 AND id = ANY($4::uuid[])
ORDER BY employment_id, id`, req.EmploymentIDs, req.Year, req.Month, []uuid.UUID{req.BeforeRunID, req.AfterRunID})
	if err != nil {
		return api.PayrollImpactBatchResponse{}, err
	}
	defer rows.Close()
	type run struct {
		formulaID     *uuid.UUID
		input, output []byte
	}
	runs := make(map[uuid.UUID]map[uuid.UUID]run)
	for rows.Next() {
		var employmentID, runID uuid.UUID
		var item run
		if err := rows.Scan(&employmentID, &runID, &item.formulaID, &item.input, &item.output); err != nil {
			return api.PayrollImpactBatchResponse{}, err
		}
		if runs[employmentID] == nil {
			runs[employmentID] = make(map[uuid.UUID]run, 2)
		}
		runs[employmentID][runID] = item
	}
	if err := rows.Err(); err != nil {
		return api.PayrollImpactBatchResponse{}, err
	}
	currentPolicies, err := currentImpactPolicies(ctx, db, req.EmploymentIDs, req.Year, req.Month)
	if err != nil {
		return api.PayrollImpactBatchResponse{}, err
	}
	evaluated := time.Now().UTC()
	evaluatedAt := evaluated.Format(time.RFC3339)
	expiresAt := evaluated.Add(payroll.ImpactFreshnessTTL).Format(time.RFC3339)
	result := api.PayrollImpactBatchResponse{Total: len(req.EmploymentIDs), EvaluatedAt: evaluatedAt, ExpiresAt: expiresAt, StaleAfterSeconds: int(payroll.ImpactFreshnessTTL / time.Second), Items: make([]api.PayrollImpactBatchItem, 0, len(req.EmploymentIDs))}
	for _, employmentID := range req.EmploymentIDs {
		item := api.PayrollImpactBatchItem{EmploymentID: employmentID}
		before, beforeOK := runs[employmentID][req.BeforeRunID]
		after, afterOK := runs[employmentID][req.AfterRunID]
		if !beforeOK || !afterOK || before.formulaID != nil || after.formulaID != nil {
			item.Reason = "calculation runs are incomplete or formula-linked"
			result.Uncomputable++
			result.Items = append(result.Items, item)
			continue
		}
		delta, compareErr := payroll.CompareImpactSnapshots(before.input, before.output, after.input, after.output)
		if compareErr != nil {
			item.Reason = compareErr.Error()
			result.Uncomputable++
			result.Items = append(result.Items, item)
			continue
		}
		item.Computed = true
		if stale, reasons := impactStalenessForPolicy(currentPolicies[employmentID], after.input); stale {
			item.Stale = true
			item.StaleReasons = reasons
		}
		item.InputFingerprint = delta.InputFingerprint
		item.Delta = &api.PayrollImpactResponse{EmploymentID: employmentID, InputFingerprint: delta.InputFingerprint, EvaluatedAt: evaluatedAt, PeriodYear: req.Year, PeriodMonth: req.Month, BeforeRunID: req.BeforeRunID, AfterRunID: req.AfterRunID, Computed: true, Stale: item.Stale, StaleReasons: item.StaleReasons, BasePayDelta: delta.BasePayDelta, OvertimePayDelta: delta.OvertimePayDelta, DeductionsDelta: delta.DeductionsDelta, NetPayDelta: delta.NetPayDelta, StandardUnitsDelta: delta.StandardUnitsDelta, OvertimeUnitsDelta: delta.OvertimeUnitsDelta}
		if !delta.BasePayDelta.IsZero() || !delta.OvertimePayDelta.IsZero() || !delta.DeductionsDelta.IsZero() || !delta.NetPayDelta.IsZero() || delta.StandardUnitsDelta != 0 || delta.OvertimeUnitsDelta != 0 {
			result.Affected++
		}
		result.Computed++
		result.Items = append(result.Items, item)
	}
	return result, nil
}

func currentImpactPolicies(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, year, month int) (map[uuid.UUID][]uuid.UUID, error) {
	startDate, endDate, err := payroll.MonthBoundaries(year, month)
	if err != nil {
		return nil, api.ErrPayrollImpactValidation
	}
	resolved, err := (WorkPolicyResolver{}).ResolveRangeForEmployments(ctx, db, employmentIDs, startDate, endDate)
	if err != nil {
		return nil, err
	}
	result := make(map[uuid.UUID][]uuid.UUID, len(employmentIDs))
	for empID, items := range resolved {
		seen := make(map[uuid.UUID]struct{}, len(items))
		idsList := make([]uuid.UUID, 0, len(items))
		for _, item := range items {
			if _, ok := seen[item.VersionID]; !ok {
				seen[item.VersionID] = struct{}{}
				idsList = append(idsList, item.VersionID)
			}
		}
		result[empID] = idsList
	}
	return result, nil
}

func comparePolicyVersionSets(currentIDs []uuid.UUID, recorded []string) (bool, []string) {
	if len(currentIDs) == 0 {
		return true, []string{"current_policy_missing"}
	}
	currentSet := make(map[uuid.UUID]struct{}, len(currentIDs))
	for _, id := range currentIDs {
		currentSet[id] = struct{}{}
	}
	recordedSet := make(map[uuid.UUID]struct{}, len(recorded))
	for _, raw := range recorded {
		vid, err := uuid.Parse(raw)
		if err != nil {
			return true, []string{"policy_version_invalid"}
		}
		recordedSet[vid] = struct{}{}
	}
	if len(recordedSet) != len(currentSet) {
		return true, []string{"policy_version_changed"}
	}
	for vid := range recordedSet {
		if _, ok := currentSet[vid]; !ok {
			return true, []string{"policy_version_changed"}
		}
	}
	return false, nil
}

func impactStalenessForPolicy(currentIDs []uuid.UUID, input []byte) (bool, []string) {
	var payload struct {
		Policy *struct {
			VersionID string `json:"version_id"`
		} `json:"policy"`
		PolicySegments []struct {
			VersionID string `json:"version_id"`
		} `json:"policy_segments"`
	}
	if err := json.Unmarshal(input, &payload); err != nil {
		return false, nil
	}
	if len(payload.PolicySegments) > 0 {
		recorded := make([]string, 0, len(payload.PolicySegments))
		for _, seg := range payload.PolicySegments {
			recorded = append(recorded, seg.VersionID)
		}
		return comparePolicyVersionSets(currentIDs, recorded)
	}
	if payload.Policy == nil || payload.Policy.VersionID == "" {
		return false, nil
	}
	return comparePolicyVersionSets(currentIDs, []string{payload.Policy.VersionID})
}

func (r PayrollRepository) ComparePayrollImpact(ctx context.Context, employmentID ids.EmploymentID, year, month int, beforeRunID, afterRunID uuid.UUID) (api.PayrollImpactResponse, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollImpactResponse{}, err
	}
	if beforeRunID == uuid.Nil || afterRunID == uuid.Nil || beforeRunID == afterRunID || year < 2000 || year > 2100 || month < 1 || month > 12 {
		return api.PayrollImpactResponse{}, api.ErrPayrollImpactValidation
	}
	rows, err := db.Query(ctx, `SELECT id, formula_version_id, input_snapshot, output_snapshot
FROM payroll_calculation_runs
WHERE employment_id = $1 AND period_year = $2 AND period_month = $3 AND id = ANY($4::uuid[])`, uuid.UUID(employmentID), year, month, []uuid.UUID{beforeRunID, afterRunID})
	if err != nil {
		return api.PayrollImpactResponse{}, err
	}
	defer rows.Close()
	type snapshotRun struct {
		formulaID *uuid.UUID
		input     []byte
		output    []byte
	}
	runs := make(map[uuid.UUID]snapshotRun, 2)
	for rows.Next() {
		var id uuid.UUID
		var run snapshotRun
		if err := rows.Scan(&id, &run.formulaID, &run.input, &run.output); err != nil {
			return api.PayrollImpactResponse{}, err
		}
		runs[id] = run
	}
	if err := rows.Err(); err != nil {
		return api.PayrollImpactResponse{}, err
	}
	before, beforeOK := runs[beforeRunID]
	after, afterOK := runs[afterRunID]
	if !beforeOK || !afterOK {
		return api.PayrollImpactResponse{}, api.ErrPayrollImpactNotFound
	}
	if before.formulaID != nil || after.formulaID != nil {
		return api.PayrollImpactResponse{}, api.ErrPayrollImpactValidation
	}
	stale, staleReason, err := impactSnapshotStaleness(ctx, db, employmentID, year, month, after.input)
	if err != nil {
		return api.PayrollImpactResponse{}, err
	}
	delta, err := payroll.CompareImpactSnapshots(before.input, before.output, after.input, after.output)
	if errors.Is(err, payroll.ErrImpactSnapshotIncomplete) {
		evaluated := time.Now().UTC()
		return api.PayrollImpactResponse{
			EmploymentID: uuid.UUID(employmentID), EvaluatedAt: evaluated.Format(time.RFC3339), ExpiresAt: evaluated.Add(payroll.ImpactFreshnessTTL).Format(time.RFC3339), StaleAfterSeconds: int(payroll.ImpactFreshnessTTL / time.Second), PeriodYear: year, PeriodMonth: month,
			BeforeRunID: beforeRunID, AfterRunID: afterRunID, Computed: false, Stale: stale, StaleReasons: staleReason,
			Reason: "calculation output is incomplete",
		}, nil
	}
	if errors.Is(err, payroll.ErrImpactInputMismatch) {
		return api.PayrollImpactResponse{}, api.ErrPayrollImpactValidation
	}
	if err != nil {
		return api.PayrollImpactResponse{}, err
	}
	evaluated := time.Now().UTC()
	return api.PayrollImpactResponse{
		EmploymentID: uuid.UUID(employmentID), InputFingerprint: delta.InputFingerprint, EvaluatedAt: evaluated.Format(time.RFC3339), ExpiresAt: evaluated.Add(payroll.ImpactFreshnessTTL).Format(time.RFC3339), StaleAfterSeconds: int(payroll.ImpactFreshnessTTL / time.Second), PeriodYear: year, PeriodMonth: month,
		BeforeRunID: beforeRunID, AfterRunID: afterRunID, Computed: delta.Complete, Stale: stale, StaleReasons: staleReason,
		Reason: delta.Reason, BasePayDelta: delta.BasePayDelta, OvertimePayDelta: delta.OvertimePayDelta,
		DeductionsDelta: delta.DeductionsDelta, NetPayDelta: delta.NetPayDelta,
		StandardUnitsDelta: delta.StandardUnitsDelta, OvertimeUnitsDelta: delta.OvertimeUnitsDelta,
	}, nil
}

func impactSnapshotStaleness(ctx context.Context, db DBTX, employmentID ids.EmploymentID, year, month int, input []byte) (bool, []string, error) {
	current, err := currentImpactPolicies(ctx, db, []uuid.UUID{uuid.UUID(employmentID)}, year, month)
	if err != nil {
		return false, nil, err
	}
	stale, reasons := impactStalenessForPolicy(current[uuid.UUID(employmentID)], input)
	return stale, reasons, nil
}

func (r PayrollRepository) ListPayrollFormulaRuleVersions(ctx context.Context, ruleID uuid.UUID) ([]api.PayrollFormulaRuleVersion, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT v.id, v.rule_id, v.version_number, v.expression, v.effective_from::text, v.status, v.created_by, v.published_by, v.published_at, v.created_at
FROM payroll_formula_rule_versions v WHERE v.rule_id = $1 ORDER BY v.version_number DESC`, ruleID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PayrollFormulaRuleVersion, 0)
	for rows.Next() {
		var version api.PayrollFormulaRuleVersion
		var publishedAt, createdAt time.Time
		if err := rows.Scan(&version.VersionID, &version.RuleID, &version.Version, &version.Expression, &version.EffectiveFrom, &version.Status, &version.CreatedBy, &version.PublishedBy, &publishedAt, &createdAt); err != nil {
			return nil, err
		}
		if !publishedAt.IsZero() {
			value := publishedAt.UTC().Format(time.RFC3339)
			version.PublishedAt = &value
		}
		version.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		result = append(result, version)
	}
	return result, rows.Err()
}

func (r PayrollRepository) ListPayrollFormulaRules(ctx context.Context) ([]api.PayrollFormulaRule, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT r.id, r.code, r.name, r.component_code, v.id, v.version_number, v.expression, v.effective_from::text, v.status
FROM payroll_formula_rules r
JOIN LATERAL (SELECT id, version_number, expression, effective_from, status FROM payroll_formula_rule_versions WHERE rule_id = r.id ORDER BY version_number DESC LIMIT 1) v ON true
ORDER BY r.code ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PayrollFormulaRule, 0)
	for rows.Next() {
		var rule api.PayrollFormulaRule
		if err := rows.Scan(&rule.ID, &rule.Code, &rule.Name, &rule.ComponentCode, &rule.VersionID, &rule.Version, &rule.Expression, &rule.EffectiveFrom, &rule.Status); err != nil {
			return nil, err
		}
		result = append(result, rule)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CreatePayrollFormulaRule(ctx context.Context, req api.CreatePayrollFormulaRuleRequest, actorID ids.UserID) (api.PayrollFormulaRule, error) {
	db, err := requireDB(r.DB, "payroll repository")
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
		if _, err := tx.Exec(ctx, `INSERT INTO payroll_formula_rules (id, code, name, component_code, created_by) VALUES ($1, $2, $3, $4, $5)`, ruleID, strings.TrimSpace(req.Code), strings.TrimSpace(req.Name), strings.TrimSpace(req.ComponentCode), uuid.UUID(actorID)); err != nil {
			if isConflictError(err) {
				return api.ErrConflict
			}
			return err
		}
		if err := tx.QueryRow(ctx, `INSERT INTO payroll_formula_rule_versions (rule_id, version_number, expression, effective_from, status, created_by) VALUES ($1, 1, $2, $3::date, 'draft', $4) RETURNING $5::uuid, $6::text, $7::text, $8::text, id, version_number, expression, effective_from::text, status`, ruleID, req.Expression, req.EffectiveFrom, uuid.UUID(actorID), ruleID, req.Code, req.Name, req.ComponentCode).Scan(
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

func (r PayrollRepository) CreatePayrollFormulaRuleVersion(ctx context.Context, ruleID ids.PayrollFormulaRuleID, req api.CreatePayrollFormulaRuleVersionRequest, actorID ids.UserID) (api.PayrollFormulaRuleVersion, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollFormulaRuleVersion{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollFormulaRuleVersion{}, errors.New("payroll repository: database does not support transactions")
	}
	var result api.PayrollFormulaRuleVersion
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var retired bool
		if err := tx.QueryRow(ctx, `SELECT retired_at IS NOT NULL FROM payroll_formula_rules WHERE id = $1 FOR UPDATE`, uuid.UUID(ruleID)).Scan(&retired); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrRuleNotFound
			}
			return err
		}
		if retired {
			return api.ErrConflict
		}
		var created time.Time
		err := tx.QueryRow(ctx, `
			INSERT INTO payroll_formula_rule_versions (rule_id, version_number, expression, effective_from, status, created_by)
			SELECT $1, COALESCE(MAX(version_number), 0) + 1, $2, $3::date, 'draft', $4
			FROM payroll_formula_rule_versions
			WHERE rule_id = $1
			RETURNING id, rule_id, version_number, expression, effective_from::text, status, created_by, created_at`, uuid.UUID(ruleID), req.Expression, req.EffectiveFrom, uuid.UUID(actorID)).Scan(
			&result.VersionID, &result.RuleID, &result.Version, &result.Expression, &result.EffectiveFrom, &result.Status, &result.CreatedBy, &created)
		if err != nil {
			if isConflictError(err) {
				return api.ErrConflict
			}
			return err
		}
		result.CreatedAt = created.UTC().Format(time.RFC3339)
		return nil
	})
	if err != nil {
		return api.PayrollFormulaRuleVersion{}, err
	}
	return result, nil
}

func (r PayrollRepository) RetirePayrollFormulaRule(ctx context.Context, ruleID ids.PayrollFormulaRuleID, actorID ids.UserID) (api.PayrollFormulaRule, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollFormulaRule{}, errors.New("payroll repository: database does not support transactions")
	}
	var result api.PayrollFormulaRule
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := tx.QueryRow(ctx, `UPDATE payroll_formula_rule_versions v
		SET status = 'retired', retired_at = CURRENT_TIMESTAMP
		FROM payroll_formula_rules r
		WHERE v.rule_id = r.id AND r.id = $1 AND v.status = 'published'
		RETURNING r.id, r.code, r.name, r.component_code, v.id, v.version_number, v.expression, v.effective_from::text, v.status`, uuid.UUID(ruleID)).Scan(
			&result.ID, &result.Code, &result.Name, &result.ComponentCode, &result.VersionID, &result.Version, &result.Expression, &result.EffectiveFrom, &result.Status); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrRuleNotFound
			}
			return err
		}
		_, err := tx.Exec(ctx, `UPDATE payroll_formula_rules SET retired_at = CURRENT_TIMESTAMP WHERE id = $1`, uuid.UUID(ruleID))
		return err
	})
	return result, err
}

func (r PayrollRepository) PublishPayrollFormulaRule(ctx context.Context, ruleID ids.PayrollFormulaRuleID, actorID ids.UserID) (api.PayrollFormulaRule, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	var result api.PayrollFormulaRule
	err = db.QueryRow(ctx, `UPDATE payroll_formula_rule_versions v
	SET status = 'published', published_by = $2, published_at = CURRENT_TIMESTAMP
	FROM payroll_formula_rules r
	WHERE v.rule_id = r.id AND r.id = $1 AND v.status = 'draft'
	RETURNING r.id, r.code, r.name, r.component_code, v.id, v.version_number, v.expression, v.effective_from::text, v.status`, uuid.UUID(ruleID), uuid.UUID(actorID)).Scan(
		&result.ID, &result.Code, &result.Name, &result.ComponentCode, &result.VersionID, &result.Version, &result.Expression, &result.EffectiveFrom, &result.Status)
	if errors.Is(err, pgx.ErrNoRows) {
		var exists bool
		if existsErr := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_formula_rules WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); existsErr != nil {
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

func (r PayrollRepository) transitionPayrollInputBatch(ctx context.Context, batchID ids.PayrollInputBatchID, actorID ids.UserID, fromStatus, toStatus string) (api.PayrollInputBatch, error) {
	db, err := requireDB(r.DB, "payroll repository")
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
	if err := db.QueryRow(ctx, query, toStatus, uuid.UUID(actorID), uuid.UUID(batchID), fromStatus).Scan(
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

func (r PayrollRepository) ApprovePayrollInputBatch(ctx context.Context, batchID ids.PayrollInputBatchID, actorID ids.UserID) (api.PayrollInputBatch, error) {
	return r.transitionPayrollInputBatch(ctx, batchID, actorID, "submitted", "approved")
}

func (r PayrollRepository) LockPayrollInputBatch(ctx context.Context, batchID ids.PayrollInputBatchID, actorID ids.UserID) (api.PayrollInputBatch, error) {
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
	db, err := requireDB(r.DB, "payroll repository")
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
	db, err := requireDB(r.DB, "payroll repository")
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

func (r PayrollRepository) ListPayrollRuleTiers(ctx context.Context, ruleID ids.PayrollRuleID) ([]api.PayrollRuleTier, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return nil, api.ErrRuleNotFound
	}

	rows, err := db.Query(ctx, `SELECT id, rule_definition_id, tier_order, min_amount::text, max_amount::text, rate::text, fixed_amount::text, created_at
FROM payroll_rule_tiers
WHERE rule_definition_id = $1
ORDER BY tier_order ASC`, uuid.UUID(ruleID))
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

func (r PayrollRepository) CreatePayrollRuleTier(ctx context.Context, ruleID ids.PayrollRuleID, req api.CreatePayrollRuleTierRequest) (api.PayrollRuleTier, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollRuleTier{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); err != nil {
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
RETURNING created_at`, uuid.UUID(tierID), uuid.UUID(ruleID), req.TierOrder, req.MinAmount.String(), maxParam, req.Rate.String(), fixed.String()).Scan(&created)
	if err != nil {
		if isConflictError(err) {
			return api.PayrollRuleTier{}, api.ErrConflict
		}
		return api.PayrollRuleTier{}, err
	}

	return api.PayrollRuleTier{
		ID:               tierID,
		RuleDefinitionID: uuid.UUID(ruleID),
		TierOrder:        req.TierOrder,
		MinAmount:        req.MinAmount,
		MaxAmount:        req.MaxAmount,
		Rate:             req.Rate,
		FixedAmount:      fixed,
		CreatedAt:        created.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) DeletePayrollRuleTier(ctx context.Context, ruleID ids.PayrollRuleID, tierID ids.PayrollRuleTierID) error {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `DELETE FROM payroll_rule_tiers WHERE id = $1 AND rule_definition_id = $2`, uuid.UUID(tierID), uuid.UUID(ruleID))
	return err
}

func (r PayrollRepository) ListPayrollRuleAssignments(ctx context.Context, ruleID ids.PayrollRuleID) ([]api.PayrollRuleAssignment, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return nil, api.ErrRuleNotFound
	}

	rows, err := db.Query(ctx, `SELECT id, rule_definition_id, assignee_type, assignee_id, effective_from::text, effective_to::text, override_config, created_at
FROM payroll_rule_assignments
WHERE rule_definition_id = $1
ORDER BY created_at DESC`, uuid.UUID(ruleID))
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

func (r PayrollRepository) AssignPayrollRule(ctx context.Context, ruleID ids.PayrollRuleID, req api.AssignPayrollRuleRequest) (api.PayrollRuleAssignment, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollRuleAssignment{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); err != nil {
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
RETURNING created_at`, assignID, uuid.UUID(ruleID), req.AssigneeType, assigneeID, req.EffectiveFrom, req.EffectiveTo, overrideParam).Scan(&created)
	if err != nil {
		if isConflictError(err) {
			return api.PayrollRuleAssignment{}, api.ErrConflict
		}
		return api.PayrollRuleAssignment{}, err
	}

	return api.PayrollRuleAssignment{
		ID:               assignID,
		RuleDefinitionID: uuid.UUID(ruleID),
		AssigneeType:     req.AssigneeType,
		AssigneeID:       assigneeID,
		EffectiveFrom:    req.EffectiveFrom,
		EffectiveTo:      req.EffectiveTo,
		OverrideConfig:   req.OverrideConfig,
		CreatedAt:        created.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) DeletePayrollRuleAssignment(ctx context.Context, ruleID ids.PayrollRuleID, assignmentID ids.PayrollRuleAssignmentID) error {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `DELETE FROM payroll_rule_assignments WHERE id = $1 AND rule_definition_id = $2`, uuid.UUID(assignmentID), uuid.UUID(ruleID))
	return err
}
