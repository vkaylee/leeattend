package repository

import (
	"context"
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/formula"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/incentives"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/rules"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

type LoadedPeriodBundle struct {
	Input    payroll.PeriodInput
	Policies []workpolicy.ResolvedPolicy
}

type PayrollBatchLoader struct{}

func (PayrollBatchLoader) LoadForEmployments(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, year, month int) (map[uuid.UUID]LoadedPeriodBundle, error) {
	if len(employmentIDs) == 0 {
		return make(map[uuid.UUID]LoadedPeriodBundle), nil
	}
	if db == nil {
		return nil, errors.New("payroll loader: database is not configured")
	}
	startDate, endDate, err := payroll.MonthBoundaries(year, month)
	if err != nil {
		return nil, err
	}
	result, validEmpIDs, employeeIDs, err := loadEmploymentMetadata(ctx, db, employmentIDs, year, month)
	if err != nil || len(validEmpIDs) == 0 {
		return result, err
	}
	if err := populateBatchPoliciesAndSalary(ctx, db, result, validEmpIDs, employeeIDs, startDate, endDate); err != nil {
		return nil, err
	}
	if err := populateBatchAttendanceAndLeave(ctx, db, result, validEmpIDs, year, month, startDate, endDate); err != nil {
		return nil, err
	}
	if err := populateBatchSharedRulesAndKPI(ctx, db, result, validEmpIDs, year, month, startDate, endDate); err != nil {
		return nil, err
	}
	return result, nil
}

func loadEmploymentMetadata(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, year, month int) (map[uuid.UUID]LoadedPeriodBundle, []uuid.UUID, []uuid.UUID, error) {
	rows, err := db.Query(ctx, `SELECT id, relationship_id, employee_id, employer_id, department_id FROM employment_segments WHERE id = ANY($1::uuid[]) AND deleted_at IS NULL`, employmentIDs)
	if err != nil {
		return nil, nil, nil, err
	}
	defer rows.Close()
	result := make(map[uuid.UUID]LoadedPeriodBundle, len(employmentIDs))
	validEmpIDs := make([]uuid.UUID, 0, len(employmentIDs))
	empSet := make(map[uuid.UUID]struct{})
	employeeIDs := make([]uuid.UUID, 0, len(employmentIDs))
	for rows.Next() {
		var empID, relID, employeeID, employerID uuid.UUID
		var deptID *uuid.UUID
		if err := rows.Scan(&empID, &relID, &employeeID, &employerID, &deptID); err != nil {
			return nil, nil, nil, err
		}
		validEmpIDs = append(validEmpIDs, empID)
		if _, seen := empSet[employeeID]; !seen {
			empSet[employeeID] = struct{}{}
			employeeIDs = append(employeeIDs, employeeID)
		}
		result[empID] = LoadedPeriodBundle{
			Input: payroll.PeriodInput{
				EmploymentID: empID, RelationshipID: relID, EmployeeID: employeeID, EmployerID: employerID, DepartmentID: deptID,
				Year: year, Month: month,
				SalaryHistory: make([]payroll.SalaryComponentEntry, 0), DepartmentChanges: make([]payroll.DepartmentChange, 0),
				DailySummaries: make(map[string]payroll.DailyAttendanceSummary), PaidLeaveDays: make(map[string]payroll.PaidLeaveDay),
				CompletedShiftCounts: make(map[string]int), ScheduledWorkdayCounts: make(map[string]int),
			},
		}
	}
	return result, validEmpIDs, employeeIDs, rows.Err()
}

func populateBatchPoliciesAndSalary(ctx context.Context, db DBTX, result map[uuid.UUID]LoadedPeriodBundle, validEmpIDs, employeeIDs []uuid.UUID, startDate, endDate time.Time) error {
	policiesByEmp, err := (WorkPolicyResolver{}).ResolveRangeForEmployments(ctx, db, validEmpIDs, startDate, endDate)
	if err != nil {
		return err
	}
	for empID, b := range result {
		b.Policies = policiesByEmp[empID]
		result[empID] = b
	}
	endStr := endDate.Format("2006-01-02")
	if err := loadBatchSalaryComponents(ctx, db, result, validEmpIDs, endStr); err != nil {
		return err
	}
	return loadBatchDepartmentChanges(ctx, db, result, employeeIDs, endStr)
}

func loadBatchSalaryComponents(ctx context.Context, db DBTX, result map[uuid.UUID]LoadedPeriodBundle, validEmpIDs []uuid.UUID, endStr string) error {
	rows, err := db.Query(ctx, `
		SELECT esc.employment_id, esc.id, esc.component_definition_id, esc.amount::text, esc.effective_date,
		       esc.created_at, scd.name, scd.code, scd.created_at, scd.updated_at
		FROM employee_salary_components esc
		JOIN salary_component_definitions scd ON scd.id = esc.component_definition_id
		WHERE esc.employment_id = ANY($1::uuid[]) AND esc.effective_date <= $2::date
		ORDER BY esc.employment_id, esc.effective_date ASC, esc.id ASC`, validEmpIDs, endStr)
	if err != nil {
		return err
	}
	defer rows.Close()
	for rows.Next() {
		var empID uuid.UUID
		var entry payroll.SalaryComponentEntry
		var amtStr string
		if err := rows.Scan(&empID, &entry.ComponentID, &entry.DefinitionID, &amtStr, &entry.EffectiveAt, &entry.CreatedAt, &entry.Name, &entry.Code, &entry.DefCreatedAt, &entry.DefUpdatedAt); err != nil {
			return err
		}
		entry.Amount, _ = decimal.NewFromString(amtStr)
		entry.EffectiveAt = normalizeDateUTC(entry.EffectiveAt)
		b := result[empID]
		b.Input.SalaryHistory = append(b.Input.SalaryHistory, entry)
		result[empID] = b
	}
	return rows.Err()
}

func loadBatchDepartmentChanges(ctx context.Context, db DBTX, result map[uuid.UUID]LoadedPeriodBundle, employeeIDs []uuid.UUID, endStr string) error {
	rows, err := db.Query(ctx, `
		SELECT employee_id, department_id, start_date
		FROM employment_segments
		WHERE employee_id = ANY($1::uuid[]) AND deleted_at IS NULL AND department_id IS NOT NULL AND start_date <= $2::date
		ORDER BY employee_id, start_date ASC, id ASC`, employeeIDs, endStr)
	if err != nil {
		return err
	}
	defer rows.Close()
	byEmp := make(map[uuid.UUID][]payroll.DepartmentChange, len(employeeIDs))
	for rows.Next() {
		var employeeID uuid.UUID
		var dc payroll.DepartmentChange
		if err := rows.Scan(&employeeID, &dc.DepartmentID, &dc.EffectiveAt); err != nil {
			return err
		}
		dc.EffectiveAt = normalizeDateUTC(dc.EffectiveAt)
		byEmp[employeeID] = append(byEmp[employeeID], dc)
	}
	if err := rows.Err(); err != nil {
		return err
	}
	for empID, b := range result {
		b.Input.DepartmentChanges = byEmp[b.Input.EmployeeID]
		result[empID] = b
	}
	return nil
}

func populateBatchAttendanceAndLeave(ctx context.Context, db DBTX, result map[uuid.UUID]LoadedPeriodBundle, validEmpIDs []uuid.UUID, year, month int, startDate, endDate time.Time) error {
	startStr, endStr := startDate.Format("2006-01-02"), endDate.Format("2006-01-02")
	if err := loadBatchDailySummaries(ctx, db, result, validEmpIDs, startStr, endStr); err != nil {
		return err
	}
	if month > 1 {
		yearStartStr := time.Date(year, 1, 1, 0, 0, 0, 0, time.UTC).Format("2006-01-02")
		if err := loadBatchPriorYTDOT(ctx, db, result, validEmpIDs, yearStartStr, startStr); err != nil {
			return err
		}
	}
	if err := loadBatchPaidLeave(ctx, db, result, validEmpIDs, startDate, endDate); err != nil {
		return err
	}
	return loadBatchWorkSegments(ctx, db, result, validEmpIDs, startStr, endStr)
}

func loadBatchDailySummaries(ctx context.Context, db DBTX, result map[uuid.UUID]LoadedPeriodBundle, validEmpIDs []uuid.UUID, startStr, endStr string) error {
	rows, err := db.Query(ctx, `
		SELECT employment_id, date::text, COALESCE(net_paid_minutes, 0), COALESCE(standard_minutes, 0),
		       COALESCE(approved_ot_minutes, 0), COALESCE(is_weekend, false), COALESCE(is_holiday, false)
		FROM attendance_daily_summaries
		WHERE employment_id = ANY($1::uuid[]) AND date >= $2::date AND date <= $3::date`, validEmpIDs, startStr, endStr)
	if err != nil {
		return err
	}
	defer rows.Close()
	for rows.Next() {
		var empID uuid.UUID
		var item payroll.DailyAttendanceSummary
		if err := rows.Scan(&empID, &item.Date, &item.NetPaidMinutes, &item.StandardMinutes, &item.ApprovedOTMinutes, &item.IsWeekend, &item.IsHoliday); err != nil {
			return err
		}
		b := result[empID]
		b.Input.DailySummaries[item.Date] = item
		result[empID] = b
	}
	return rows.Err()
}

func loadBatchPriorYTDOT(ctx context.Context, db DBTX, result map[uuid.UUID]LoadedPeriodBundle, validEmpIDs []uuid.UUID, yearStartStr, startStr string) error {
	rows, err := db.Query(ctx, `
		SELECT employment_id, COALESCE(SUM(approved_ot_minutes), 0)::int
		FROM attendance_daily_summaries
		WHERE employment_id = ANY($1::uuid[]) AND date >= $2::date AND date < $3::date
		GROUP BY employment_id`, validEmpIDs, yearStartStr, startStr)
	if err != nil {
		return err
	}
	defer rows.Close()
	for rows.Next() {
		var empID uuid.UUID
		var mins int
		if err := rows.Scan(&empID, &mins); err != nil {
			return err
		}
		b := result[empID]
		b.Input.PriorYTDApprovedOTMinutes = mins
		result[empID] = b
	}
	return rows.Err()
}

func loadBatchPaidLeave(ctx context.Context, db DBTX, result map[uuid.UUID]LoadedPeriodBundle, validEmpIDs []uuid.UUID, startDate, endDate time.Time) error {
	rows, err := db.Query(ctx, `
		SELECT lr.employment_id, lr.start_date, lr.end_date, lr.duration_type
		FROM leave_requests lr
		JOIN leave_types lt ON lt.id = lr.leave_type_id
		WHERE lr.employment_id = ANY($1::uuid[])
		  AND lr.status = 'approved' AND lt.is_paid = true
		  AND lr.start_date <= $3::date AND lr.end_date >= $2::date`,
		validEmpIDs, startDate.Format("2006-01-02"), endDate.Format("2006-01-02"))
	if err != nil {
		return err
	}
	defer rows.Close()
	for rows.Next() {
		var empID uuid.UUID
		var lStart, lEnd time.Time
		var durationType string
		if err := rows.Scan(&empID, &lStart, &lEnd, &durationType); err != nil {
			return err
		}
		expandPaidLeaveDays(result[empID].Input.PaidLeaveDays, lStart, lEnd, startDate, endDate, durationType)
	}
	return rows.Err()
}

func expandPaidLeaveDays(target map[string]payroll.PaidLeaveDay, lStart, lEnd, startDate, endDate time.Time, durationType string) {
	frac := decimal.NewFromInt(1)
	if strings.Contains(strings.ToLower(durationType), "half") {
		frac = decimal.RequireFromString("0.5")
	}
	d := normalizeDateUTC(lStart)
	if d.Before(startDate) {
		d = startDate
	}
	stop := normalizeDateUTC(lEnd)
	if stop.After(endDate) {
		stop = endDate
	}
	for ; !d.After(stop); d = d.AddDate(0, 0, 1) {
		dStr := d.Format("2006-01-02")
		target[dStr] = payroll.PaidLeaveDay{Date: dStr, Fraction: frac}
	}
}

func loadBatchWorkSegments(ctx context.Context, db DBTX, result map[uuid.UUID]LoadedPeriodBundle, validEmpIDs []uuid.UUID, startStr, endStr string) error {
	rows, err := db.Query(ctx, `
		SELECT ws.employment_id, ws.date::text,
		       COUNT(DISTINCT CASE WHEN ws.segment_status = 'completed' AND te.id IS NOT NULL THEN ws.id END)::int,
		       BOOL_OR(ws.segment_type = 'work' AND ws.segment_status <> 'cancelled'
		         AND (sdr.state = 'published' OR (ws.schedule_revision_id IS NULL AND NOT EXISTS (
		              SELECT 1 FROM schedule_day_revisions cur
		              WHERE cur.employment_id = ws.employment_id AND cur.business_date = ws.date
		                AND cur.state IN ('published', 'explicit_off')
		         ))))
		FROM work_segments ws
		LEFT JOIN schedule_day_revisions sdr ON sdr.id = ws.schedule_revision_id
		LEFT JOIN time_entries te ON te.segment_id = ws.id AND te.punch_out_time IS NOT NULL
		WHERE ws.employment_id = ANY($1::uuid[]) AND ws.date >= $2::date AND ws.date <= $3::date
		GROUP BY ws.employment_id, ws.date`, validEmpIDs, startStr, endStr)
	if err != nil {
		return err
	}
	defer rows.Close()
	for rows.Next() {
		var empID uuid.UUID
		var dStr string
		var completed int
		var hasWork bool
		if err := rows.Scan(&empID, &dStr, &completed, &hasWork); err != nil {
			return err
		}
		b := result[empID]
		if completed > 0 {
			b.Input.CompletedShiftCounts[dStr] = completed
		}
		if hasWork {
			b.Input.ScheduledWorkdayCounts[dStr] = 1
		}
		result[empID] = b
	}
	return rows.Err()
}

func populateBatchSharedRulesAndKPI(ctx context.Context, db DBTX, result map[uuid.UUID]LoadedPeriodBundle, validEmpIDs []uuid.UUID, year, month int, startDate, endDate time.Time) error {
	startStr, endStr := startDate.Format("2006-01-02"), endDate.Format("2006-01-02")
	incInputs, err := loadBatchIncentivePolicies(ctx, db, startStr, endStr)
	if err != nil {
		return err
	}
	allRules, err := loadBatchPayrollRuleAssignments(ctx, db, startStr, endStr)
	if err != nil {
		return err
	}
	kpiByEmp, err := loadBatchKPIBonuses(ctx, db, validEmpIDs, year, month, endStr)
	if err != nil {
		return err
	}
	for empID, b := range result {
		b.Input.IncentivePolicies = incInputs
		b.Input.RuleAssignments = matchRuleAssignmentsForEmployee(allRules, b.Input.DepartmentID, b.Input.EmployeeID)
		if kpiVal, ok := kpiByEmp[empID]; ok {
			copyKPI := kpiVal
			b.Input.KPIBonus = &copyKPI
		}
		result[empID] = b
	}
	return nil
}

func (PayrollBatchLoader) ResolveEmploymentIDsForRelationships(ctx context.Context, db DBTX, relationshipIDs []uuid.UUID, year, month int) (map[uuid.UUID]uuid.UUID, error) {
	out := make(map[uuid.UUID]uuid.UUID, len(relationshipIDs))
	if len(relationshipIDs) == 0 {
		return out, nil
	}
	startDate, endDate, err := payroll.MonthBoundaries(year, month)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `
		SELECT DISTINCT ON (relationship_id) relationship_id, id
		FROM employment_segments
		WHERE relationship_id = ANY($1::uuid[]) AND deleted_at IS NULL
		  AND start_date <= $3::date AND (end_date IS NULL OR end_date >= $2::date)
		ORDER BY relationship_id, start_date DESC, id DESC`,
		relationshipIDs, startDate.Format("2006-01-02"), endDate.Format("2006-01-02"))
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var relID, empID uuid.UUID
		if err := rows.Scan(&relID, &empID); err != nil {
			return nil, err
		}
		out[relID] = empID
	}
	return out, rows.Err()
}

type incentiveHeader struct {
	id  uuid.UUID
	obj incentives.Policy
}

func loadBatchIncentivePolicies(ctx context.Context, db DBTX, startStr, endStr string) ([]payroll.IncentivePolicyInput, error) {
	pols, ids, err := queryIncentiveHeaders(ctx, db, startStr, endStr)
	if err != nil || len(ids) == 0 {
		return nil, err
	}
	tiersByPolicy, err := queryIncentiveTiers(ctx, db, ids)
	if err != nil {
		return nil, err
	}
	out := make([]payroll.IncentivePolicyInput, 0, len(pols))
	for _, p := range pols {
		out = append(out, payroll.IncentivePolicyInput{Policy: p.obj, Tiers: tiersByPolicy[p.id]})
	}
	return out, nil
}

func queryIncentiveHeaders(ctx context.Context, db DBTX, startStr, endStr string) ([]incentiveHeader, []uuid.UUID, error) {
	rows, err := db.Query(ctx, `
		SELECT p.id, p.name, p.base_amount::text, p.payout_type
		FROM payroll_incentive_policies p
		WHERE p.is_active = true AND p.deleted_at IS NULL
		  AND p.effective_from <= $1::date AND (p.effective_to IS NULL OR p.effective_to >= $2::date)
		ORDER BY p.created_at ASC`, endStr, startStr)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()
	var pols []incentiveHeader
	var ids []uuid.UUID
	for rows.Next() {
		var id uuid.UUID
		var name, baseStr, payoutType string
		if err := rows.Scan(&id, &name, &baseStr, &payoutType); err != nil {
			return nil, nil, err
		}
		amt, _ := decimal.NewFromString(baseStr)
		pols = append(pols, incentiveHeader{id: id, obj: incentives.Policy{Name: name, BaseAmount: amt, PayoutType: payoutType}})
		ids = append(ids, id)
	}
	return pols, ids, rows.Err()
}

func queryIncentiveTiers(ctx context.Context, db DBTX, ids []uuid.UUID) (map[uuid.UUID][]incentives.Tier, error) {
	rows, err := db.Query(ctx, `SELECT id, policy_id, priority, payout_amount::text, rule_config FROM payroll_incentive_tiers WHERE policy_id = ANY($1::uuid[]) ORDER BY priority ASC`, ids)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	out := make(map[uuid.UUID][]incentives.Tier, len(ids))
	for rows.Next() {
		var t incentives.Tier
		var pid uuid.UUID
		var pStr string
		if err := rows.Scan(&t.ID, &pid, &t.Priority, &pStr, &t.RuleConfig); err != nil {
			return nil, err
		}
		t.PayoutAmount, _ = decimal.NewFromString(pStr)
		out[pid] = append(out[pid], t)
	}
	return out, rows.Err()
}

type scopedRuleAssignment struct {
	assigneeType string
	assigneeID   *uuid.UUID
	assignment   rules.Assignment
}

type rawRuleAssign struct {
	ruleID       uuid.UUID
	assigneeType string
	assigneeID   *uuid.UUID
	overrideCfg  []byte
	code         string
	calcMethod   string
	defaultCfg   []byte
}

func loadBatchPayrollRuleAssignments(ctx context.Context, db DBTX, startStr, endStr string) ([]scopedRuleAssignment, error) {
	raws, tieredIDs, err := queryRawRuleAssignments(ctx, db, startStr, endStr)
	if err != nil {
		return nil, err
	}
	tiersByRule, err := queryPayrollRuleTiers(ctx, db, tieredIDs)
	if err != nil {
		return nil, err
	}
	out := make([]scopedRuleAssignment, 0, len(raws))
	for _, r := range raws {
		out = append(out, scopedRuleAssignment{
			assigneeType: r.assigneeType, assigneeID: r.assigneeID,
			assignment: rules.Assignment{
				Code: r.code, CalculationMethod: r.calcMethod, DefaultConfig: r.defaultCfg,
				OverrideConfig: r.overrideCfg, Tiers: tiersByRule[r.ruleID],
			},
		})
	}
	return out, nil
}

func queryRawRuleAssignments(ctx context.Context, db DBTX, startStr, endStr string) ([]rawRuleAssign, []uuid.UUID, error) {
	rows, err := db.Query(ctx, `
		SELECT a.rule_definition_id, a.assignee_type, a.assignee_id, a.override_config, d.code, d.calculation_method, d.rule_config
		FROM payroll_rule_assignments a
		JOIN payroll_rule_definitions d ON d.id = a.rule_definition_id
		WHERE a.effective_from <= $1::date AND (a.effective_to IS NULL OR a.effective_to >= $2::date)
		  AND d.is_active = true`, endStr, startStr)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()
	var raws []rawRuleAssign
	var tieredIDs []uuid.UUID
	seen := make(map[uuid.UUID]struct{})
	for rows.Next() {
		var r rawRuleAssign
		if err := rows.Scan(&r.ruleID, &r.assigneeType, &r.assigneeID, &r.overrideCfg, &r.code, &r.calcMethod, &r.defaultCfg); err != nil {
			return nil, nil, err
		}
		raws = append(raws, r)
		if r.calcMethod == "tiered" {
			if _, ok := seen[r.ruleID]; !ok {
				seen[r.ruleID] = struct{}{}
				tieredIDs = append(tieredIDs, r.ruleID)
			}
		}
	}
	return raws, tieredIDs, rows.Err()
}

func queryPayrollRuleTiers(ctx context.Context, db DBTX, tieredIDs []uuid.UUID) (map[uuid.UUID][]rules.Tier, error) {
	out := make(map[uuid.UUID][]rules.Tier, len(tieredIDs))
	if len(tieredIDs) == 0 {
		return out, nil
	}
	rows, err := db.Query(ctx, `SELECT rule_definition_id, tier_order, min_amount::text, max_amount::text, rate::text, fixed_amount::text FROM payroll_rule_tiers WHERE rule_definition_id = ANY($1::uuid[]) ORDER BY tier_order ASC`, tieredIDs)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var rid uuid.UUID
		var t rules.Tier
		var minStr, rateStr, fixStr string
		var maxStr *string
		if err := rows.Scan(&rid, &t.TierOrder, &minStr, &maxStr, &rateStr, &fixStr); err != nil {
			return nil, err
		}
		t.MinAmount, _ = decimal.NewFromString(minStr)
		t.MaxAmount, _ = parseOptionalDecimal(maxStr)
		t.Rate, _ = decimal.NewFromString(rateStr)
		t.FixedAmount, _ = decimal.NewFromString(fixStr)
		out[rid] = append(out[rid], t)
	}
	return out, rows.Err()
}

func matchRuleAssignmentsForEmployee(all []scopedRuleAssignment, deptID *uuid.UUID, employeeID uuid.UUID) []rules.Assignment {
	matched := make([]rules.Assignment, 0)
	for _, item := range all {
		switch item.assigneeType {
		case "organization":
			matched = append(matched, item.assignment)
		case "department":
			if deptID != nil && item.assigneeID != nil && *item.assigneeID == *deptID {
				matched = append(matched, item.assignment)
			}
		case "employee":
			if item.assigneeID != nil && *item.assigneeID == employeeID {
				matched = append(matched, item.assignment)
			}
		}
	}
	return matched
}

func loadBatchKPIBonuses(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, year, month int, asOf string) (map[uuid.UUID]payroll.KPIBonusInput, error) {
	inputsByEmp, valuesByEmp, err := queryLockedKPIInputs(ctx, db, employmentIDs, year, month)
	if err != nil || len(valuesByEmp) == 0 {
		return make(map[uuid.UUID]payroll.KPIBonusInput), err
	}
	weights, err := queryOrgKPIWeights(ctx, db, asOf)
	if err != nil {
		return nil, err
	}
	formulaID, expression, found, err := queryPublishedKPIFormula(ctx, db, asOf)
	if err != nil {
		return nil, err
	}
	return evaluateBatchKPIFormulas(inputsByEmp, valuesByEmp, weights, formulaID, expression, found)
}

func queryLockedKPIInputs(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, year, month int) (map[uuid.UUID]map[string]string, map[uuid.UUID]map[string]decimal.Decimal, error) {
	rows, err := db.Query(ctx, `
		SELECT DISTINCT ON (i.employment_id, m.code) i.employment_id, m.code, i.value::text
		FROM payroll_inputs i
		JOIN payroll_input_batches b ON b.id = i.batch_id
		JOIN payroll_metric_definitions m ON m.id = i.metric_definition_id
		WHERE i.employment_id = ANY($1::uuid[])
		  AND b.period_year = $2 AND b.period_month = $3
		  AND b.status = 'locked' AND i.status IN ('accepted', 'corrected', 'warning')
		ORDER BY i.employment_id, m.code, b.created_at DESC, i.id DESC`, employmentIDs, year, month)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()
	inputsByEmp := make(map[uuid.UUID]map[string]string)
	valuesByEmp := make(map[uuid.UUID]map[string]decimal.Decimal)
	metricVar := map[string]string{"production.output": "actual", "production.target_output": "target", "production.quality_rate": "quality"}
	for rows.Next() {
		var empID uuid.UUID
		var code, raw string
		if err := rows.Scan(&empID, &code, &raw); err != nil {
			return nil, nil, err
		}
		val, err := decimal.NewFromString(raw)
		if err != nil {
			return nil, nil, err
		}
		if inputsByEmp[empID] == nil {
			inputsByEmp[empID] = make(map[string]string)
			valuesByEmp[empID] = make(map[string]decimal.Decimal)
		}
		inputsByEmp[empID][code] = val.String()
		if varName, ok := metricVar[code]; ok {
			valuesByEmp[empID][varName] = val
		}
	}
	return inputsByEmp, valuesByEmp, rows.Err()
}

func queryOrgKPIWeights(ctx context.Context, db DBTX, asOf string) (map[string]decimal.Decimal, error) {
	keys := []string{"payroll.kpi.output_weight", "payroll.kpi.quality_weight", "payroll.kpi.score_cap", "payroll.kpi.target_bonus", "payroll.kpi.bonus_cap"}
	keyToVar := map[string]string{
		"payroll.kpi.output_weight": "output_weight", "payroll.kpi.quality_weight": "quality_weight",
		"payroll.kpi.score_cap": "score_cap", "payroll.kpi.target_bonus": "target_bonus", "payroll.kpi.bonus_cap": "bonus_cap",
	}
	rows, err := db.Query(ctx, `
		SELECT config_key, value_decimal::text FROM config_entries
		WHERE scope_type = 'organization' AND scope_id IS NULL AND config_key = ANY($1) AND effective_from <= $2::date
		ORDER BY effective_from DESC, created_at DESC`, keys, asOf)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	out := make(map[string]decimal.Decimal, len(keys))
	seen := make(map[string]bool)
	for rows.Next() {
		var key, raw string
		if err := rows.Scan(&key, &raw); err != nil {
			return nil, err
		}
		if seen[key] {
			continue
		}
		seen[key] = true
		val, err := decimal.NewFromString(raw)
		if err != nil {
			return nil, err
		}
		out[keyToVar[key]] = val
	}
	return out, rows.Err()
}

func queryPublishedKPIFormula(ctx context.Context, db DBTX, asOf string) (uuid.UUID, string, bool, error) {
	var formulaID uuid.UUID
	var expression string
	err := db.QueryRow(ctx, `
		SELECT v.id, v.expression
		FROM payroll_formula_rules r
		JOIN payroll_formula_rule_versions v ON v.rule_id = r.id
		WHERE r.code = 'kpi.bonus' AND v.status = 'published' AND v.effective_from <= $1::date
		ORDER BY v.effective_from DESC, v.version_number DESC LIMIT 1`, asOf).Scan(&formulaID, &expression)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, "", false, nil
	}
	return formulaID, expression, err == nil, err
}

func evaluateBatchKPIFormulas(inputsByEmp map[uuid.UUID]map[string]string, valuesByEmp map[uuid.UUID]map[string]decimal.Decimal, weights map[string]decimal.Decimal, formulaID uuid.UUID, expression string, found bool) (map[uuid.UUID]payroll.KPIBonusInput, error) {
	out := make(map[uuid.UUID]payroll.KPIBonusInput, len(inputsByEmp))
	if !found {
		for empID, inMap := range inputsByEmp {
			out[empID] = payroll.KPIBonusInput{Inputs: inMap}
		}
		return out, nil
	}
	for empID, vals := range valuesByEmp {
		for k, v := range weights {
			vals[k] = v
		}
		amount, err := formula.Evaluate(expression, vals)
		if err != nil {
			return nil, err
		}
		fid := formulaID
		out[empID] = payroll.KPIBonusInput{
			Amount: amount.Round(2), FormulaID: &fid, Inputs: inputsByEmp[empID],
			Trace: []map[string]string{
				{"step": "input", "rule": "kpi.bonus", "expression": "locked KPI inputs", "result": "loaded"},
				{"step": "formula", "rule": "kpi.bonus", "expression": expression, "result": amount.String()},
			},
		}
	}
	return out, nil
}
