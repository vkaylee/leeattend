package repository

import (
	"context"
	"encoding/json"
	"errors"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/formula"
)

type kpiCalculation struct {
	Amount    decimal.Decimal
	FormulaID *uuid.UUID
	Inputs    map[string]string
	Trace     []map[string]string
}

func calculateKPIBonus(ctx context.Context, db DBTX, employmentID uuid.UUID, year, month int, asOf string) (kpiCalculation, error) {
	rows, err := db.Query(ctx, `SELECT DISTINCT ON (m.code) m.code, i.value::text
FROM payroll_inputs i
JOIN payroll_input_batches b ON b.id = i.batch_id
JOIN payroll_metric_definitions m ON m.id = i.metric_definition_id
WHERE i.employment_id = $1 AND b.period_year = $2 AND b.period_month = $3 AND b.status = 'locked'
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

func persistKPICalculation(ctx context.Context, db DBTX, employmentID uuid.UUID, year, month int, calc kpiCalculation, actorID *uuid.UUID) error {
	if calc.FormulaID == nil {
		return nil
	}
	inputSnapshot, err := json.Marshal(calc.Inputs)
	if err != nil {
		return err
	}
	outputSnapshot, err := json.Marshal(map[string]string{"KPI_BONUS": calc.Amount.String()})
	if err != nil {
		return err
	}
	trace, err := json.Marshal(calc.Trace)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO payroll_calculation_runs (employment_id, period_year, period_month, formula_version_id, engine_version, input_snapshot, output_snapshot, calculation_trace, created_by)
VALUES ($1, $2, $3, $4, 'formula-v1', $5, $6, $7, $8)`, employmentID, year, month, calc.FormulaID, inputSnapshot, outputSnapshot, trace, actorID)
	return err
}
