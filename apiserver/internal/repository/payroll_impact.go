package repository

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
	"time"
)

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
