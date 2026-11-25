package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
)

// PayrollPersistenceWriter writes pure engine CalculationResult rows to PostgreSQL.
// Simulation NEVER invokes this writer.
type PayrollPersistenceWriter struct{}

// PersistCalculationResult upserts the draft payslip, segments, incentives, rule evaluations,
// and append-only calculation run snapshot inside an open transaction.
func (PayrollPersistenceWriter) PersistCalculationResult(ctx context.Context, db DBTX, res payroll.CalculationResult, year, month int, actorID uuid.UUID) (api.FullPayslipResponse, error) {
	finalPayslipID, createdTime, updatedTime, err := upsertDraftPayslipHeader(ctx, db, res.Payslip)
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	policySegments, err := insertPayslipSegments(ctx, db, finalPayslipID, res.Segments)
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	if err := insertPayslipIncentives(ctx, db, finalPayslipID, res.Incentives); err != nil {
		return api.FullPayslipResponse{}, err
	}
	ruleRecords, err := insertPayslipRuleEvaluations(ctx, db, finalPayslipID, res.RuleEvaluations)
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	if err := persistCalculationRunSnapshot(ctx, db, finalPayslipID, year, month, res, policySegments, actorID); err != nil {
		return api.FullPayslipResponse{}, err
	}
	resp := CalculationResultToResponse(finalPayslipID, createdTime, updatedTime, res)
	resp.RuleEvaluations = ruleRecords
	return resp, nil
}

func upsertDraftPayslipHeader(ctx context.Context, db DBTX, assembled payroll.Payslip) (uuid.UUID, time.Time, time.Time, error) {
	startStr := assembled.StartDate.Format("2006-01-02")
	endStr := assembled.EndDate.Format("2006-01-02")
	var existingID uuid.UUID
	var existingStatus string
	err := db.QueryRow(ctx, `SELECT id, status FROM payslips WHERE employment_id = $1 AND period_start = $2::date FOR UPDATE`, assembled.EmploymentID, startStr).Scan(&existingID, &existingStatus)
	if errors.Is(err, pgx.ErrNoRows) {
		var created, updated time.Time
		err = db.QueryRow(ctx, `
			INSERT INTO payslips (id, employment_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, status, standard_units, overtime_units, holiday_work_premium_amount, created_at, updated_at)
			VALUES ($1, $2, $3::date, $4::date, $5, $6, $7, $8, 'draft', $9, $10, $11, NOW(), NOW())
			RETURNING created_at, updated_at`,
			assembled.ID, assembled.EmploymentID, startStr, endStr,
			assembled.TotalBasePay.String(), assembled.TotalOvertimePay.String(), assembled.DeductionsAmount.String(),
			assembled.NetPay.String(), assembled.TotalStdUnits, assembled.TotalOTUnits, assembled.TotalHolidayWorkPremium.String(),
		).Scan(&created, &updated)
		return assembled.ID, created, updated, err
	}
	if err != nil {
		return uuid.Nil, time.Time{}, time.Time{}, err
	}
	if existingStatus == "paid" || existingStatus == "finalized" {
		return uuid.Nil, time.Time{}, time.Time{}, api.ErrPayslipConflict
	}
	for _, stmt := range []string{
		`DELETE FROM payslip_segments WHERE payslip_id = $1`,
		`DELETE FROM payslip_incentives WHERE payslip_id = $1`,
		`DELETE FROM payslip_rule_evaluations WHERE payslip_id = $1`,
	} {
		if _, err := db.Exec(ctx, stmt, existingID); err != nil {
			return uuid.Nil, time.Time{}, time.Time{}, err
		}
	}
	var created, updated time.Time
	err = db.QueryRow(ctx, `
		UPDATE payslips
		SET base_pay_amount = $1, overtime_pay_amount = $2, deductions_amount = $3, net_pay_amount = $4,
		    status = 'draft', standard_units = $5, overtime_units = $6, holiday_work_premium_amount = $7, updated_at = NOW()
		WHERE id = $8 RETURNING created_at, updated_at`,
		assembled.TotalBasePay.String(), assembled.TotalOvertimePay.String(), assembled.DeductionsAmount.String(),
		assembled.NetPay.String(), assembled.TotalStdUnits, assembled.TotalOTUnits, assembled.TotalHolidayWorkPremium.String(),
		existingID,
	).Scan(&created, &updated)
	return existingID, created, updated, err
}

func insertPayslipSegments(ctx context.Context, db DBTX, payslipID uuid.UUID, segments []payroll.CalculatedSegment) ([]policySegmentSnapshot, error) {
	snaps := make([]policySegmentSnapshot, 0, len(segments))
	for _, seg := range segments {
		src := seg.StdDaysSource
		vid := seg.WorkPolicyVersionID
		_, err := db.Exec(ctx, `
			INSERT INTO payslip_segments (id, payslip_id, start_date, end_date, department_id, applied_base_salary, applied_std_days, actual_work_days, paid_work_days, amount, created_at, applied_hours_per_day, std_days_source, work_policy_version_id)
			VALUES ($1, $2, $3::date, $4::date, $5, $6, $7, $8, $9, $10, NOW(), $11, $12, $13)`,
			seg.ID, payslipID, seg.StartDate.Format("2006-01-02"), seg.EndDate.Format("2006-01-02"),
			seg.DepartmentID, seg.AppliedBaseSalary.String(), seg.AppliedStdDays.String(),
			seg.ActualWorkDays.String(), seg.PaidWorkDays.String(), seg.Amount.String(),
			seg.AppliedHoursPerDay.String(), &src, &vid)
		if err != nil {
			return nil, err
		}
		snaps = append(snaps, policySegmentSnapshot{
			StartDate: seg.StartDate.Format("2006-01-02"), EndDate: seg.EndDate.Format("2006-01-02"), VersionID: vid,
		})
	}
	return snaps, nil
}

func insertPayslipIncentives(ctx context.Context, db DBTX, payslipID uuid.UUID, items []payroll.CalculatedIncentive) error {
	for _, inc := range items {
		var err error
		if inc.TierID != nil {
			_, err = db.Exec(ctx, `INSERT INTO payslip_incentives (id, payslip_id, policy_name, amount, tier_id, standard_amount, created_at) VALUES ($1, $2, $3, $4, $5, $6, NOW())`,
				uuid.New(), payslipID, inc.PolicyName, inc.Amount.InexactFloat64(), inc.TierID, inc.StandardAmount.InexactFloat64())
		} else {
			_, err = db.Exec(ctx, `INSERT INTO payslip_incentives (id, payslip_id, policy_name, amount, standard_amount, created_at) VALUES ($1, $2, $3, $4, $5, NOW())`,
				uuid.New(), payslipID, inc.PolicyName, inc.Amount.InexactFloat64(), inc.StandardAmount.InexactFloat64())
		}
		if err != nil {
			return err
		}
	}
	return nil
}

func insertPayslipRuleEvaluations(ctx context.Context, db DBTX, payslipID uuid.UUID, evals []payroll.CalculatedRuleEvaluation) ([]api.PayslipRuleEvaluationRecord, error) {
	records := make([]api.PayslipRuleEvaluationRecord, 0, len(evals))
	nowStr := time.Now().UTC().Format(time.RFC3339)
	for _, rEval := range evals {
		recID := uuid.New()
		if _, err := db.Exec(ctx, `INSERT INTO payslip_rule_evaluations (id, payslip_id, rule_code, amount, created_at) VALUES ($1, $2, $3, $4, NOW())`,
			recID, payslipID, rEval.RuleCode, rEval.Amount.String()); err != nil {
			return nil, err
		}
		records = append(records, api.PayslipRuleEvaluationRecord{ID: recID, PayslipID: payslipID, RuleCode: rEval.RuleCode, Amount: rEval.Amount, CreatedAt: nowStr})
	}
	return records, nil
}

func persistCalculationRunSnapshot(ctx context.Context, db DBTX, payslipID uuid.UUID, year, month int, res payroll.CalculationResult, policySegments []policySegmentSnapshot, actorID uuid.UUID) error {
	traceMaps := make([]map[string]string, 0, len(res.Trace))
	for _, step := range res.Trace {
		traceMaps = append(traceMaps, map[string]string{"step": step.Step, "rule": step.Rule, "expression": step.Expression, "result": step.Result})
	}
	kpiSnap := kpiCalculation{
		Policy: res.PrimaryPolicy, PolicySegments: policySegments,
		StandardUnits: res.Payslip.TotalStdUnits, OvertimeUnits: res.Payslip.TotalOTUnits,
		StandardCredits: res.TotalCredits, BasePay: res.Payslip.TotalBasePay,
		OvertimePay: res.Payslip.TotalOvertimePay, Deductions: res.Payslip.DeductionsAmount,
		NetPay: res.Payslip.NetPay, Trace: traceMaps,
	}
	if res.KPIBonus != nil {
		kpiSnap.Amount = res.KPIBonus.Amount
		kpiSnap.FormulaID = res.KPIBonus.FormulaID
		kpiSnap.Inputs = res.KPIBonus.Inputs
	}
	return persistKPICalculation(ctx, db, res.Payslip.EmploymentID, payslipID, year, month, kpiSnap, &actorID)
}

// CalculationResultToResponse formats a pure CalculationResult into api.FullPayslipResponse
// without touching the database.
func CalculationResultToResponse(payslipID uuid.UUID, createdAt, updatedAt time.Time, res payroll.CalculationResult) api.FullPayslipResponse {
	assembled := res.Payslip
	return api.FullPayslipResponse{
		PayslipRecord: api.PayslipRecord{
			ID: payslipID, EmploymentID: assembled.EmploymentID,
			PeriodStart: assembled.StartDate.Format("2006-01-02"), PeriodEnd: assembled.EndDate.Format("2006-01-02"),
			BasePayAmount: assembled.TotalBasePay, OvertimePayAmount: assembled.TotalOvertimePay,
			DeductionsAmount: assembled.DeductionsAmount, NetPayAmount: assembled.NetPay,
			Status: "draft", StandardUnits: int(assembled.TotalStdUnits), OvertimeUnits: int(assembled.TotalOTUnits),
			HolidayWorkPremiumAmount: assembled.TotalHolidayWorkPremium,
			CreatedAt:                createdAt.UTC().Format(time.RFC3339), UpdatedAt: updatedAt.UTC().Format(time.RFC3339),
		},
		Segments:         mapCalculatedSegments(payslipID, res.Segments),
		Incentives:       mapCalculatedIncentives(res.Incentives),
		RuleEvaluations:  mapCalculatedRuleEvaluations(payslipID, updatedAt, res.RuleEvaluations),
		SalaryComponents: mapActiveComponents(assembled.EmploymentID, res.ActiveComponents),
		CalculationTrace: mapCalculationTrace(res.Trace),
	}
}

func mapCalculatedSegments(payslipID uuid.UUID, list []payroll.CalculatedSegment) []api.PayslipSegmentResponse {
	out := make([]api.PayslipSegmentResponse, 0, len(list))
	for _, seg := range list {
		src, vid := seg.StdDaysSource, seg.WorkPolicyVersionID
		out = append(out, api.PayslipSegmentResponse{
			ID: seg.ID, PayslipID: payslipID, StartDate: seg.StartDate.Format("2006-01-02"), EndDate: seg.EndDate.Format("2006-01-02"),
			DepartmentID: seg.DepartmentID, AppliedBaseSalary: seg.AppliedBaseSalary, AppliedStdDays: seg.AppliedStdDays,
			AppliedHoursPerDay: seg.AppliedHoursPerDay, ActualWorkDays: seg.ActualWorkDays, PaidWorkDays: seg.PaidWorkDays,
			Amount: seg.Amount, Reason: seg.Reason, StdDaysSource: &src, WorkPolicyVersionID: &vid,
		})
	}
	return out
}

func mapCalculatedIncentives(list []payroll.CalculatedIncentive) []api.IncentiveEvaluationResult {
	out := make([]api.IncentiveEvaluationResult, 0, len(list))
	for _, inc := range list {
		out = append(out, api.IncentiveEvaluationResult{PolicyName: inc.PolicyName, Amount: inc.Amount, StandardAmount: inc.StandardAmount, TierID: inc.TierID})
	}
	return out
}

func mapCalculatedRuleEvaluations(payslipID uuid.UUID, at time.Time, list []payroll.CalculatedRuleEvaluation) []api.PayslipRuleEvaluationRecord {
	nowStr := at.UTC().Format(time.RFC3339)
	out := make([]api.PayslipRuleEvaluationRecord, 0, len(list))
	for _, re := range list {
		out = append(out, api.PayslipRuleEvaluationRecord{ID: uuid.New(), PayslipID: payslipID, RuleCode: re.RuleCode, Amount: re.Amount, CreatedAt: nowStr})
	}
	return out
}

func mapActiveComponents(employmentID uuid.UUID, list []payroll.SalaryComponentEntry) []api.SalaryComponentWithDefinition {
	out := make([]api.SalaryComponentWithDefinition, 0, len(list))
	for _, c := range list {
		out = append(out, api.SalaryComponentWithDefinition{
			Component:  api.EmployeeSalaryComponent{ID: c.ComponentID, EmploymentID: employmentID, ComponentDefinitionID: c.DefinitionID, Amount: c.Amount, EffectiveDate: c.EffectiveAt.Format("2006-01-02"), CreatedAt: c.CreatedAt.UTC().Format(time.RFC3339)},
			Definition: api.SalaryComponentDefinition{ID: c.DefinitionID, Name: c.Name, Code: c.Code, CreatedAt: c.DefCreatedAt.UTC().Format(time.RFC3339), UpdatedAt: c.DefUpdatedAt.UTC().Format(time.RFC3339)},
		})
	}
	return out
}

func mapCalculationTrace(list []payroll.CalculationTraceStep) []api.PayrollCalculationTrace {
	out := make([]api.PayrollCalculationTrace, 0, len(list))
	for _, s := range list {
		out = append(out, api.PayrollCalculationTrace{Step: s.Step, Rule: s.Rule, Expression: s.Expression, Result: s.Result})
	}
	return out
}
