package repository

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

func (r PayrollRepository) CalculateEmployeePayroll(ctx context.Context, employeeID ids.EmployeeID, year, month int, actorID ids.UserID) (api.FullPayslipResponse, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.FullPayslipResponse{}, errors.New("payroll repository: database does not support transactions")
	}
	var response api.FullPayslipResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		employmentID, employerID, err := lookupActiveEmployeeEmployment(ctx, tx, uuid.UUID(employeeID))
		if err != nil {
			return err
		}
		if err := ensurePayrollPeriodOpen(ctx, tx, employerID, year, month); err != nil {
			return err
		}
		response, err = calculateAndPersistSingleEmployment(ctx, tx, employmentID, year, month, uuid.UUID(actorID))
		return err
	})
	return response, err
}

func lookupActiveEmployeeEmployment(ctx context.Context, db DBTX, employeeID uuid.UUID) (uuid.UUID, uuid.UUID, error) {
	var empExists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&empExists); err != nil {
		return uuid.Nil, uuid.Nil, err
	}
	if !empExists {
		return uuid.Nil, uuid.Nil, api.ErrEmployeeNotFound
	}
	var employmentID, employerID uuid.UUID
	err := db.QueryRow(ctx, `SELECT id, employer_id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID).Scan(&employmentID, &employerID)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, uuid.Nil, api.ErrNoActiveEmployment
	}
	return employmentID, employerID, err
}

func calculateAndPersistSingleEmployment(ctx context.Context, tx pgx.Tx, employmentID uuid.UUID, year, month int, actorID uuid.UUID) (api.FullPayslipResponse, error) {
	bundles, err := (PayrollBatchLoader{}).LoadForEmployments(ctx, tx, []uuid.UUID{employmentID}, year, month)
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	bundle, ok := bundles[employmentID]
	if !ok {
		return api.FullPayslipResponse{}, api.ErrNoActiveEmployment
	}
	calcRes, err := payroll.CalculatePayslip(bundle.Input, bundle.Policies)
	if err != nil {
		return api.FullPayslipResponse{}, mapPayrollEngineError(err)
	}
	return (PayrollPersistenceWriter{}).PersistCalculationResult(ctx, tx, calcRes, year, month, actorID)
}

func ensurePayrollPeriodOpen(ctx context.Context, tx pgx.Tx, employerID uuid.UUID, year, month int) error {
	if year < 2000 || year > 2100 || month < 1 || month > 12 {
		return api.ErrPayrollPeriodValidation
	}
	if _, err := tx.Exec(ctx, `INSERT INTO payroll_periods (employer_id, period_year, period_month) VALUES ($1, $2, $3) ON CONFLICT (employer_id, period_year, period_month) DO NOTHING`, employerID, year, month); err != nil {
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

func (r PayrollRepository) SimulateEmployeePayroll(ctx context.Context, employeeID ids.EmployeeID, year, month int, _ ids.UserID) (api.FullPayslipResponse, error) {
	return r.simulateEmployeePayrollPure(ctx, employeeID, year, month, nil)
}

func (r PayrollRepository) SimulateEmployeePayrollWithPolicy(ctx context.Context, employeeID ids.EmployeeID, year, month int, policyVersionID uuid.UUID, _ ids.UserID) (api.FullPayslipResponse, error) {
	if policyVersionID == uuid.Nil {
		return api.FullPayslipResponse{}, api.ErrPayrollValidation
	}
	return r.simulateEmployeePayrollPure(ctx, employeeID, year, month, &policyVersionID)
}

func (r PayrollRepository) simulateEmployeePayrollPure(ctx context.Context, employeeID ids.EmployeeID, year, month int, policyVersionID *uuid.UUID) (api.FullPayslipResponse, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	employmentID, err := lookupActiveEmploymentIDOnly(ctx, db, uuid.UUID(employeeID))
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	bundles, err := (PayrollBatchLoader{}).LoadForEmployments(ctx, db, []uuid.UUID{employmentID}, year, month)
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	bundle, ok := bundles[employmentID]
	if !ok {
		return api.FullPayslipResponse{}, api.ErrNoActiveEmployment
	}
	policies, err := resolveSimulationPolicies(ctx, db, bundle, policyVersionID, year, month)
	if err != nil {
		return api.FullPayslipResponse{}, err
	}
	calcRes, err := payroll.CalculatePayslip(bundle.Input, policies)
	if err != nil {
		return api.FullPayslipResponse{}, mapPayrollEngineError(err)
	}
	now := time.Now().UTC()
	return CalculationResultToResponse(calcRes.Payslip.ID, now, now, calcRes), nil
}

func lookupActiveEmploymentIDOnly(ctx context.Context, db DBTX, employeeID uuid.UUID) (uuid.UUID, error) {
	var employmentID uuid.UUID
	err := db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employeeID).Scan(&employmentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, api.ErrNoActiveEmployment
	}
	return employmentID, err
}

func resolveSimulationPolicies(ctx context.Context, db DBTX, bundle LoadedPeriodBundle, policyVersionID *uuid.UUID, year, month int) ([]workpolicy.ResolvedPolicy, error) {
	if policyVersionID == nil {
		return bundle.Policies, nil
	}
	var groupID uuid.UUID
	if err := db.QueryRow(ctx, `SELECT group_id FROM work_policy_versions WHERE id = $1`, *policyVersionID).Scan(&groupID); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, api.ErrPayrollValidation
		}
		return nil, err
	}
	candidate, err := (WorkPolicyResolver{}).LoadResolvedVersion(ctx, db, groupID, *policyVersionID)
	if err != nil {
		return nil, err
	}
	return applyCandidatePolicyOverride(ctx, db, bundle.Input.RelationshipID, bundle.Policies, candidate, year, month)
}

func (r PayrollRepository) SimulateWorkPolicyVersionBatch(ctx context.Context, groupID, versionID uuid.UUID, req api.WorkPolicyBatchSimulationRequest) (api.WorkPolicyBatchSimulationResponse, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.WorkPolicyBatchSimulationResponse{}, err
	}
	candidate, err := (WorkPolicyResolver{}).LoadResolvedVersion(ctx, db, groupID, versionID)
	if err != nil {
		return api.WorkPolicyBatchSimulationResponse{}, err
	}
	startDate, endDate, err := payroll.MonthBoundaries(req.Year, req.Month)
	if err != nil {
		return api.WorkPolicyBatchSimulationResponse{}, api.ErrPayrollValidation
	}
	endExclusive := endDate.AddDate(0, 0, 1)
	relToEmp, bundles, membershipsByRel, err := loadBatchSimulationContext(ctx, db, groupID, req, startDate, endExclusive)
	if err != nil {
		return api.WorkPolicyBatchSimulationResponse{}, err
	}
	return evaluateBatchSimulationResponse(groupID, versionID, req, candidate, relToEmp, bundles, membershipsByRel, startDate, endExclusive), nil
}

func loadBatchSimulationContext(ctx context.Context, db DBTX, groupID uuid.UUID, req api.WorkPolicyBatchSimulationRequest, startDate, endExclusive time.Time) (map[uuid.UUID]uuid.UUID, map[uuid.UUID]LoadedPeriodBundle, map[uuid.UUID][]groupMembershipWindow, error) {
	loader := PayrollBatchLoader{}
	relToEmp, err := loader.ResolveEmploymentIDsForRelationships(ctx, db, req.EmploymentRelationshipIDs, req.Year, req.Month)
	if err != nil {
		return nil, nil, nil, err
	}
	empIDs := make([]uuid.UUID, 0, len(relToEmp))
	for _, empID := range relToEmp {
		empIDs = append(empIDs, empID)
	}
	bundles, err := loader.LoadForEmployments(ctx, db, empIDs, req.Year, req.Month)
	if err != nil {
		return nil, nil, nil, err
	}
	membershipsByRel, err := loadGroupMembershipsForRelationships(ctx, db, groupID, req.EmploymentRelationshipIDs, startDate, endExclusive)
	return relToEmp, bundles, membershipsByRel, err
}

func evaluateBatchSimulationResponse(groupID, versionID uuid.UUID, req api.WorkPolicyBatchSimulationRequest, candidate workpolicy.ResolvedPolicy, relToEmp map[uuid.UUID]uuid.UUID, bundles map[uuid.UUID]LoadedPeriodBundle, membershipsByRel map[uuid.UUID][]groupMembershipWindow, startDate, endExclusive time.Time) api.WorkPolicyBatchSimulationResponse {
	evaluated := time.Now().UTC()
	resp := api.WorkPolicyBatchSimulationResponse{
		GroupID: groupID, VersionID: versionID, PeriodYear: req.Year, PeriodMonth: req.Month,
		Total: len(req.EmploymentRelationshipIDs), EvaluatedAt: evaluated.Format(time.RFC3339),
		ExpiresAt:         evaluated.Add(payroll.ImpactFreshnessTTL).Format(time.RFC3339),
		StaleAfterSeconds: int(payroll.ImpactFreshnessTTL / time.Second),
		Items:             make([]api.WorkPolicyBatchSimulationItem, 0, len(req.EmploymentRelationshipIDs)),
	}
	fingerprints := make([]string, 0, len(req.EmploymentRelationshipIDs))
	for _, relID := range req.EmploymentRelationshipIDs {
		item := simulateSingleRelationshipItem(relID, relToEmp, bundles, membershipsByRel[relID], candidate, startDate, endExclusive)
		if item.Status == "computed" {
			resp.Computed++
			if item.Affected {
				resp.Affected++
			}
			fingerprints = append(fingerprints, item.InputFingerprint)
		} else {
			resp.Uncomputable++
		}
		resp.Items = append(resp.Items, item)
	}
	hash := sha256.Sum256([]byte(strings.Join(fingerprints, "|")))
	resp.InputFingerprint = hex.EncodeToString(hash[:])
	return resp
}

func simulateSingleRelationshipItem(relID uuid.UUID, relToEmp map[uuid.UUID]uuid.UUID, bundles map[uuid.UUID]LoadedPeriodBundle, memberships []groupMembershipWindow, candidate workpolicy.ResolvedPolicy, startDate, endExclusive time.Time) api.WorkPolicyBatchSimulationItem {
	item := api.WorkPolicyBatchSimulationItem{EmploymentRelationshipID: relID, Status: "uncomputable"}
	empID, hasEmp := relToEmp[relID]
	if !hasEmp {
		item.Reason = "no active employment segment covers period"
		return item
	}
	empCopy := empID
	item.EmploymentID = &empCopy
	bundle, hasBundle := bundles[empID]
	if !hasBundle {
		item.Reason = "employment data could not be loaded"
		return item
	}
	baselineRes, err := payroll.CalculatePayslip(bundle.Input, bundle.Policies)
	if err != nil {
		item.Reason = "baseline calculation failed: " + err.Error()
		return item
	}
	candidatePolicies := substituteCandidatePolicies(bundle.Policies, memberships, candidate, startDate, endExclusive)
	candidateRes, err := payroll.CalculatePayslip(bundle.Input, candidatePolicies)
	if err != nil {
		item.Reason = "candidate calculation failed: " + err.Error()
		return item
	}
	return buildComputedSimItem(relID, empCopy, baselineRes, candidateRes)
}

func buildComputedSimItem(relID, empID uuid.UUID, baseRes, candRes payroll.CalculationResult) api.WorkPolicyBatchSimulationItem {
	baseGross, candGross := baseRes.GrossPay, candRes.GrossPay
	grossDelta := candGross.Sub(baseGross)
	baseNet, candNet := baseRes.Payslip.NetPay, candRes.Payslip.NetPay
	netDelta := candNet.Sub(baseNet)
	baseOT, candOT := baseRes.Payslip.TotalOvertimePay, candRes.Payslip.TotalOvertimePay
	otDelta := candOT.Sub(baseOT)
	return api.WorkPolicyBatchSimulationItem{
		EmploymentRelationshipID: relID, EmploymentID: &empID, Status: "computed",
		Affected:         !grossDelta.IsZero() || !netDelta.IsZero() || !otDelta.IsZero(),
		InputFingerprint: baseRes.InputFingerprint,
		BaselineGross:    &baseGross, CandidateGross: &candGross, GrossDelta: &grossDelta,
		BaselineNet: &baseNet, CandidateNet: &candNet, NetDelta: &netDelta,
		BaselineOTPay: &baseOT, CandidateOTPay: &candOT, OTPayDelta: &otDelta,
		Trace: mapCalculationTrace(candRes.Trace),
	}
}

func (r PayrollRepository) CalculateBatchPayroll(ctx context.Context, req api.BatchCalculatePayrollRequest, actorID ids.UserID) (api.BatchCalculatePayrollResponse, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.BatchCalculatePayrollResponse{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.BatchCalculatePayrollResponse{}, errors.New("payroll repository: database does not support transactions")
	}
	deduped := dedupeUUIDs(req.EmploymentIDs)
	var response api.BatchCalculatePayrollResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		bundles, loadErr := (PayrollBatchLoader{}).LoadForEmployments(ctx, tx, deduped, req.Year, req.Month)
		if loadErr != nil {
			return loadErr
		}
		response.Results = executeBatchCalculationLoop(ctx, tx, deduped, bundles, req.Year, req.Month, uuid.UUID(actorID))
		return nil
	})
	return response, err
}

func dedupeUUIDs(list []uuid.UUID) []uuid.UUID {
	seen := make(map[uuid.UUID]bool, len(list))
	out := make([]uuid.UUID, 0, len(list))
	for _, id := range list {
		if !seen[id] {
			seen[id] = true
			out = append(out, id)
		}
	}
	return out
}

func executeBatchCalculationLoop(ctx context.Context, tx pgx.Tx, empIDs []uuid.UUID, bundles map[uuid.UUID]LoadedPeriodBundle, year, month int, actorID uuid.UUID) []api.BatchCalculateResult {
	lockedEmployers := make(map[uuid.UUID]error)
	writer := PayrollPersistenceWriter{}
	results := make([]api.BatchCalculateResult, 0, len(empIDs))
	for _, empID := range empIDs {
		results = append(results, processSingleBatchEmployment(ctx, tx, writer, empID, bundles, lockedEmployers, year, month, actorID))
	}
	return results
}

func processSingleBatchEmployment(ctx context.Context, tx pgx.Tx, writer PayrollPersistenceWriter, empID uuid.UUID, bundles map[uuid.UUID]LoadedPeriodBundle, lockedEmployers map[uuid.UUID]error, year, month int, actorID uuid.UUID) api.BatchCalculateResult {
	bundle, exists := bundles[empID]
	if !exists {
		msg := "employment not found"
		return api.BatchCalculateResult{EmploymentID: empID, Status: "error", ErrorMessage: &msg}
	}
	periodErr, checked := lockedEmployers[bundle.Input.EmployerID]
	if !checked {
		periodErr = ensurePayrollPeriodOpen(ctx, tx, bundle.Input.EmployerID, year, month)
		lockedEmployers[bundle.Input.EmployerID] = periodErr
	}
	if periodErr != nil {
		msg := periodErr.Error()
		return api.BatchCalculateResult{EmploymentID: empID, EmployeeID: bundle.Input.EmployeeID, Status: "error", ErrorMessage: &msg}
	}
	calcRes, calcErr := payroll.CalculatePayslip(bundle.Input, bundle.Policies)
	if calcErr != nil {
		msg := mapPayrollEngineError(calcErr).Error()
		return api.BatchCalculateResult{EmploymentID: empID, EmployeeID: bundle.Input.EmployeeID, Status: "error", ErrorMessage: &msg}
	}
	fullResp, writeErr := writer.PersistCalculationResult(ctx, tx, calcRes, year, month, actorID)
	if writeErr != nil {
		msg := writeErr.Error()
		return api.BatchCalculateResult{EmploymentID: empID, EmployeeID: bundle.Input.EmployeeID, Status: "error", ErrorMessage: &msg}
	}
	payslipCopy := fullResp.PayslipRecord
	return api.BatchCalculateResult{EmploymentID: empID, EmployeeID: bundle.Input.EmployeeID, Status: "success", Payslip: &payslipCopy}
}

type groupMembershipWindow struct {
	id            uuid.UUID
	effectiveFrom time.Time
	effectiveTo   *time.Time
}

func loadGroupMembershipsForRelationships(ctx context.Context, db DBTX, groupID uuid.UUID, relIDs []uuid.UUID, start, endExclusive time.Time) (map[uuid.UUID][]groupMembershipWindow, error) {
	out := make(map[uuid.UUID][]groupMembershipWindow, len(relIDs))
	if len(relIDs) == 0 {
		return out, nil
	}
	rows, err := db.Query(ctx, `
		SELECT id, employment_relationship_id, effective_from, effective_to
		FROM work_policy_memberships
		WHERE group_id = $1 AND employment_relationship_id = ANY($2::uuid[])
		  AND effective_from < $4::date AND (effective_to IS NULL OR effective_to > $3::date)
		ORDER BY employment_relationship_id, effective_from ASC`,
		groupID, relIDs, start.Format("2006-01-02"), endExclusive.Format("2006-01-02"))
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var w groupMembershipWindow
		var relID uuid.UUID
		if err := rows.Scan(&w.id, &relID, &w.effectiveFrom, &w.effectiveTo); err != nil {
			return nil, err
		}
		w.effectiveFrom = normalizeDateUTC(w.effectiveFrom)
		w.effectiveTo = normalizeOptionalDateUTC(w.effectiveTo)
		out[relID] = append(out[relID], w)
	}
	return out, rows.Err()
}

func applyCandidatePolicyOverride(ctx context.Context, db DBTX, relationshipID uuid.UUID, baseline []workpolicy.ResolvedPolicy, candidate workpolicy.ResolvedPolicy, year, month int) ([]workpolicy.ResolvedPolicy, error) {
	startDate, endDate, err := payroll.MonthBoundaries(year, month)
	if err != nil {
		return nil, api.ErrPayrollValidation
	}
	endExclusive := endDate.AddDate(0, 0, 1)
	memberships, err := loadGroupMembershipsForRelationships(ctx, db, candidate.GroupID, []uuid.UUID{relationshipID}, startDate, endExclusive)
	if err != nil {
		return nil, err
	}
	if len(memberships[relationshipID]) == 0 {
		return nil, api.ErrPayrollValidation
	}
	out := substituteCandidatePolicies(baseline, memberships[relationshipID], candidate, startDate, endExclusive)
	if len(out) == 0 {
		return nil, api.ErrPayrollValidation
	}
	return out, nil
}

func trimBaselineAroundCandidate(p, candidate workpolicy.ResolvedPolicy, start, endExclusive time.Time) []workpolicy.ResolvedPolicy {
	var kept []workpolicy.ResolvedPolicy
	if p.EffectiveFrom.Before(candidate.EffectiveFrom) {
		left := p
		cut := candidate.EffectiveFrom
		if left.EffectiveTo == nil || cut.Before(*left.EffectiveTo) {
			left.EffectiveTo = &cut
		}
		if _, _, ok := left.SegmentBounds(start, endExclusive); ok {
			kept = append(kept, left)
		}
	}
	if candidate.EffectiveTo != nil && (p.EffectiveTo == nil || p.EffectiveTo.After(*candidate.EffectiveTo)) {
		right := p
		if right.EffectiveFrom.Before(*candidate.EffectiveTo) {
			right.EffectiveFrom = *candidate.EffectiveTo
		}
		if _, _, ok := right.SegmentBounds(start, endExclusive); ok {
			kept = append(kept, right)
		}
	}
	return kept
}

func substituteCandidatePolicies(baseline []workpolicy.ResolvedPolicy, groupMemberships []groupMembershipWindow, candidate workpolicy.ResolvedPolicy, start, endExclusive time.Time) []workpolicy.ResolvedPolicy {
	out := make([]workpolicy.ResolvedPolicy, 0, len(baseline)+len(groupMemberships))
	for _, p := range baseline {
		if p.GroupID != candidate.GroupID {
			out = append(out, p)
			continue
		}
		out = append(out, trimBaselineAroundCandidate(p, candidate, start, endExclusive)...)
	}
	for _, m := range groupMemberships {
		c := candidate
		c.MembershipID = m.id
		c.MembershipEffectiveFrom = m.effectiveFrom
		c.MembershipEffectiveTo = m.effectiveTo
		if _, _, ok := c.SegmentBounds(start, endExclusive); ok {
			out = append(out, c)
		}
	}
	return out
}

func mapPayrollEngineError(err error) error {
	switch {
	case errors.Is(err, payroll.ErrEnginePolicyMissing):
		return api.ErrPayrollPolicyMissing
	case errors.Is(err, payroll.ErrEngineValidation):
		return api.ErrPayrollValidation
	default:
		return err
	}
}
