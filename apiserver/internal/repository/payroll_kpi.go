package repository

import (
	"context"
	"encoding/json"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

type policySegmentSnapshot struct {
	StartDate string    `json:"start_date"`
	EndDate   string    `json:"end_date"`
	VersionID uuid.UUID `json:"version_id"`
}

type kpiCalculation struct {
	Amount          decimal.Decimal
	FormulaID       *uuid.UUID
	Inputs          map[string]string
	Trace           []map[string]string
	Policy          *workpolicy.ResolvedPolicy
	PolicySegments  []policySegmentSnapshot
	StandardUnits   int32
	OvertimeUnits   int32
	StandardCredits decimal.Decimal
	BasePay         decimal.Decimal
	OvertimePay     decimal.Decimal
	Deductions      decimal.Decimal
	NetPay          decimal.Decimal
}

func persistKPICalculation(ctx context.Context, db DBTX, employmentID, payslipID uuid.UUID, year, month int, calc kpiCalculation, actorID *uuid.UUID) error {
	inputSnapshot, err := marshalKPICalculationInputs(calc)
	if err != nil {
		return err
	}
	outputSnapshot, err := json.Marshal(map[string]string{
		"KPI_BONUS":        calc.Amount.String(),
		"standard_units":   decimal.NewFromInt32(calc.StandardUnits).String(),
		"overtime_units":   decimal.NewFromInt32(calc.OvertimeUnits).String(),
		"standard_credits": calc.StandardCredits.String(),
		"base_pay":         calc.BasePay.String(),
		"overtime_pay":     calc.OvertimePay.String(),
		"deductions":       calc.Deductions.String(),
		"net_pay":          calc.NetPay.String(),
	})
	if err != nil {
		return err
	}
	trace, err := json.Marshal(calc.Trace)
	if err != nil {
		return err
	}
	if calc.FormulaID == nil {
		_, err = db.Exec(ctx, `INSERT INTO payroll_calculation_runs (employment_id, payslip_id, period_year, period_month, formula_version_id, engine_version, input_snapshot, output_snapshot, calculation_trace, created_by)
		VALUES ($1, $2, $3, $4, NULL, 'payroll-v2', $5, $6, $7, $8)`, employmentID, payslipID, year, month, inputSnapshot, outputSnapshot, trace, actorID)
		return err
	}
	formulaID := *calc.FormulaID
	_, err = db.Exec(ctx, `INSERT INTO payroll_calculation_runs (employment_id, payslip_id, period_year, period_month, formula_version_id, engine_version, input_snapshot, output_snapshot, calculation_trace, created_by)
VALUES ($1, $2, $3, $4, $5, 'formula-v2', $6, $7, $8, $9)
ON CONFLICT (payslip_id, formula_version_id) WHERE payslip_id IS NOT NULL AND formula_version_id IS NOT NULL DO UPDATE SET input_snapshot = EXCLUDED.input_snapshot, output_snapshot = EXCLUDED.output_snapshot, calculation_trace = EXCLUDED.calculation_trace, created_by = EXCLUDED.created_by, engine_version = EXCLUDED.engine_version`, employmentID, payslipID, year, month, formulaID, inputSnapshot, outputSnapshot, trace, actorID)
	return err
}

func marshalKPICalculationInputs(calc kpiCalculation) ([]byte, error) {
	inputPayload := map[string]any{"kpi_inputs": calc.Inputs}
	if calc.Policy != nil {
		var effTo *string
		if calc.Policy.EffectiveTo != nil {
			v := calc.Policy.EffectiveTo.Format("2006-01-02")
			effTo = &v
		}
		inputPayload["policy"] = map[string]any{
			"group_id":                   calc.Policy.GroupID.String(),
			"group_code":                 calc.Policy.GroupCode,
			"version_id":                 calc.Policy.VersionID.String(),
			"version_no":                 calc.Policy.VersionNo,
			"effective_from":             calc.Policy.EffectiveFrom.Format("2006-01-02"),
			"effective_to":               effTo,
			"overtime_policy_version_id": calc.Policy.OvertimePolicyVersionID.String(),
			"pay_method":                 string(calc.Policy.PayMethod),
		}
	}
	if len(calc.PolicySegments) > 0 {
		inputPayload["policy_segments"] = calc.PolicySegments
	}
	return json.Marshal(inputPayload)
}
