package repository

import (
	"context"
	"encoding/json"
	"errors"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/formula"
)

type kpiCalculation struct {
	Amount    decimal.Decimal
	FormulaID *uuid.UUID
	Inputs    map[string]string
	Trace     []map[string]string
	Policy    *api.WorkPolicyGroupVersion
}

func calculateKPIBonus(ctx context.Context, db DBTX, employmentID uuid.UUID, year, month int, asOf string) (kpiCalculation, error) {
	rows, err := db.Query(ctx, `SELECT DISTINCT ON (m.code) m.code, i.value::text
FROM payroll_inputs i
JOIN payroll_input_batches b ON b.id = i.batch_id
JOIN payroll_metric_definitions m ON m.id = i.metric_definition_id
WHERE i.employment_id = $1 AND b.period_year = $2 AND b.period_month = $3 AND b.status = 'locked' AND i.status IN ('accepted', 'corrected', 'warning')
ORDER BY m.code, b.created_at DESC, i.id DESC`, employmentID, year, month)
	if err != nil {
		return kpiCalculation{}, err
	}
	defer rows.Close()
	inputs := make(map[string]string)
	values := make(map[string]decimal.Decimal)
	for rows.Next() {
		var code, raw string
		if err := rows.Scan(&code, &raw); err != nil {
			return kpiCalculation{}, err
		}
		value, err := decimal.NewFromString(raw)
		if err != nil {
			return kpiCalculation{}, err
		}
		inputs[code] = value.String()
		switch code {
		case "production.output":
			values["actual"] = value
		case "production.target_output":
			values["target"] = value
		case "production.quality_rate":
			values["quality"] = value
		}
	}
	if err := rows.Err(); err != nil {
		return kpiCalculation{}, err
	}
	if len(values) == 0 {
		return kpiCalculation{Inputs: inputs}, nil
	}
	var outputWeight, qualityWeight, scoreCap, targetBonus, bonusCap decimal.Decimal
	cfgRows, cfgErr := db.Query(ctx, `SELECT config_key, value_decimal::text
FROM config_entries
WHERE scope_type = 'organization' AND scope_id IS NULL AND config_key = ANY($1) AND effective_from <= $2::date
ORDER BY effective_from DESC, created_at DESC`, []string{"payroll.kpi.output_weight", "payroll.kpi.quality_weight", "payroll.kpi.score_cap", "payroll.kpi.target_bonus", "payroll.kpi.bonus_cap"}, asOf)
	if cfgErr != nil {
		return kpiCalculation{}, cfgErr
	}
	seenConfig := make(map[string]bool)
	for cfgRows.Next() {
		var key, raw string
		if err := cfgRows.Scan(&key, &raw); err != nil {
			cfgRows.Close()
			return kpiCalculation{}, err
		}
		if seenConfig[key] {
			continue
		}
		seenConfig[key] = true
		value, err := decimal.NewFromString(raw)
		if err != nil {
			cfgRows.Close()
			return kpiCalculation{}, err
		}
		switch key {
		case "payroll.kpi.output_weight":
			outputWeight = value
		case "payroll.kpi.quality_weight":
			qualityWeight = value
		case "payroll.kpi.score_cap":
			scoreCap = value
		case "payroll.kpi.target_bonus":
			targetBonus = value
		case "payroll.kpi.bonus_cap":
			bonusCap = value
		}
	}
	if err := cfgRows.Err(); err != nil {
		cfgRows.Close()
		return kpiCalculation{}, err
	}
	cfgRows.Close()
	values["output_weight"] = outputWeight
	values["quality_weight"] = qualityWeight
	values["score_cap"] = scoreCap
	values["target_bonus"] = targetBonus
	values["bonus_cap"] = bonusCap

	var formulaID uuid.UUID
	var expression string
	err = db.QueryRow(ctx, `SELECT v.id, v.expression
FROM payroll_formula_rules r
JOIN payroll_formula_rule_versions v ON v.rule_id = r.id
WHERE r.code = 'kpi.bonus' AND v.status = 'published' AND v.effective_from <= $1::date
ORDER BY v.effective_from DESC, v.version_number DESC LIMIT 1`, asOf).Scan(&formulaID, &expression)
	if errors.Is(err, pgx.ErrNoRows) {
		return kpiCalculation{Inputs: inputs}, nil
	}
	if err != nil {
		return kpiCalculation{}, err
	}
	amount, err := formula.Evaluate(expression, values)
	if err != nil {
		return kpiCalculation{}, err
	}
	trace := []map[string]string{{"step": "input", "rule": "kpi.bonus", "expression": "locked KPI inputs", "result": "loaded"}, {"step": "formula", "rule": "kpi.bonus", "expression": expression, "result": amount.String()}}
	return kpiCalculation{Amount: amount.Round(2), FormulaID: &formulaID, Inputs: inputs, Trace: trace}, nil
}

func persistKPICalculation(ctx context.Context, db DBTX, employmentID, payslipID uuid.UUID, year, month int, calc kpiCalculation, actorID *uuid.UUID) error {
	if calc.FormulaID == nil {
		return nil
	}
	inputSnapshot, err := marshalKPICalculationInputs(calc)
	if err != nil {
		return err
	}
	outputSnapshot, err := json.Marshal(map[string]string{"KPI_BONUS": calc.Amount.String()})
	if err != nil {
		return err
	}
	traceSteps := append([]map[string]string(nil), calc.Trace...)
	if calc.Policy != nil {
		traceSteps = append(traceSteps, map[string]string{"step": "policy", "rule": "labor_policy_group", "expression": "effective membership resolution", "result": calc.Policy.ID.String()})
	}
	trace, err := json.Marshal(traceSteps)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO payroll_calculation_runs (employment_id, payslip_id, period_year, period_month, formula_version_id, engine_version, input_snapshot, output_snapshot, calculation_trace, created_by)
VALUES ($1, $2, $3, $4, $5, 'formula-v1', $6, $7, $8, $9)
ON CONFLICT (payslip_id, formula_version_id) WHERE payslip_id IS NOT NULL AND formula_version_id IS NOT NULL DO UPDATE SET input_snapshot = EXCLUDED.input_snapshot, output_snapshot = EXCLUDED.output_snapshot, calculation_trace = EXCLUDED.calculation_trace, created_by = EXCLUDED.created_by`, employmentID, payslipID, year, month, calc.FormulaID, inputSnapshot, outputSnapshot, trace, actorID)
	return err
}

func marshalKPICalculationInputs(calc kpiCalculation) ([]byte, error) {
	inputPayload := map[string]any{"kpi_inputs": calc.Inputs}
	if calc.Policy != nil {
		inputPayload["policy"] = map[string]any{
			"group_id": calc.Policy.GroupID.String(), "version_id": calc.Policy.ID.String(),
			"effective_from": calc.Policy.EffectiveFrom, "effective_to": calc.Policy.EffectiveTo,
			"work_credit_policy_id": calc.Policy.WorkCreditPolicyID.String(), "pay_basis_policy_id": calc.Policy.PayBasisPolicyID.String(),
			"overtime_policy_id": calc.Policy.OvertimePolicyID.String(), "pay_method": string(calc.Policy.PayMethod),
		}
	}
	return json.Marshal(inputPayload)
}
