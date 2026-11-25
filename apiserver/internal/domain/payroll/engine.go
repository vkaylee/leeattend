package payroll

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"sort"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/incentives"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/rates"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/rules"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

var (
	ErrEnginePolicyMissing = errors.New("payroll engine: work policy is missing for period")
	ErrEngineValidation    = errors.New("payroll engine: validation failed")
)

type SalaryComponentEntry struct {
	ComponentID  uuid.UUID       `json:"component_id"`
	DefinitionID uuid.UUID       `json:"definition_id"`
	Code         string          `json:"code"`
	Name         string          `json:"name"`
	Amount       decimal.Decimal `json:"amount"`
	EffectiveAt  time.Time       `json:"effective_at"`
	CreatedAt    time.Time       `json:"created_at"`
	DefCreatedAt time.Time       `json:"def_created_at"`
	DefUpdatedAt time.Time       `json:"def_updated_at"`
}

type DepartmentChange struct {
	DepartmentID *uuid.UUID `json:"department_id,omitempty"`
	EffectiveAt  time.Time  `json:"effective_at"`
}

type OTTimeSlice struct {
	StartMinute int `json:"start_minute"`
	EndMinute   int `json:"end_minute"`
}

type DailyAttendanceSummary struct {
	Date              string        `json:"date"`
	NetPaidMinutes    int           `json:"net_paid_minutes"`
	StandardMinutes   int           `json:"standard_minutes"`
	ApprovedOTMinutes int           `json:"approved_ot_minutes"`
	IsWeekend         bool          `json:"is_weekend"`
	IsHoliday         bool          `json:"is_holiday"`
	OTTimeSlices      []OTTimeSlice `json:"ot_time_slices,omitempty"`
}

type PaidLeaveDay struct {
	Date             string          `json:"date"`
	Fraction         decimal.Decimal `json:"fraction"`
	ScheduledMinutes int             `json:"scheduled_minutes"`
}

type IncentivePolicyInput struct {
	Policy incentives.Policy `json:"policy"`
	Tiers  []incentives.Tier `json:"tiers"`
}

type KPIBonusInput struct {
	Amount    decimal.Decimal     `json:"amount"`
	FormulaID *uuid.UUID          `json:"formula_id,omitempty"`
	Inputs    map[string]string   `json:"inputs,omitempty"`
	Trace     []map[string]string `json:"trace,omitempty"`
}

type PeriodInput struct {
	EmploymentID              uuid.UUID                         `json:"employment_id"`
	RelationshipID            uuid.UUID                         `json:"relationship_id"`
	EmployeeID                uuid.UUID                         `json:"employee_id"`
	EmployerID                uuid.UUID                         `json:"employer_id"`
	DepartmentID              *uuid.UUID                        `json:"department_id,omitempty"`
	Year                      int                               `json:"year"`
	Month                     int                               `json:"month"`
	SalaryHistory             []SalaryComponentEntry            `json:"salary_history"`
	DepartmentChanges         []DepartmentChange                `json:"department_changes"`
	DailySummaries            map[string]DailyAttendanceSummary `json:"daily_summaries"`
	PaidLeaveDays             map[string]PaidLeaveDay           `json:"paid_leave_days"`
	CompletedShiftCounts      map[string]int                    `json:"completed_shift_counts"`
	ScheduledWorkdayCounts    map[string]int                    `json:"scheduled_workday_counts"`
	PriorYTDApprovedOTMinutes int                               `json:"prior_ytd_approved_ot_minutes"`
	IncentivePolicies         []IncentivePolicyInput            `json:"incentive_policies"`
	RuleAssignments           []rules.Assignment                `json:"rule_assignments"`
	KPIBonus                  *KPIBonusInput                    `json:"kpi_bonus,omitempty"`
}

type CalculatedSegment struct {
	ID                  uuid.UUID
	StartDate           time.Time
	EndDate             time.Time
	DepartmentID        *uuid.UUID
	AppliedBaseSalary   decimal.Decimal
	AppliedStdDays      decimal.Decimal
	AppliedHoursPerDay  decimal.Decimal
	ActualWorkDays      decimal.Decimal
	PaidWorkDays        decimal.Decimal
	Amount              decimal.Decimal
	Reason              string
	StdDaysSource       string
	WorkPolicyVersionID uuid.UUID
}

type CalculatedIncentive struct {
	PolicyName     string
	Amount         decimal.Decimal
	StandardAmount decimal.Decimal
	TierID         *uuid.UUID
}

type CalculatedRuleEvaluation struct {
	RuleCode string
	Amount   decimal.Decimal
}

type CalculationTraceStep struct {
	Step       string `json:"step"`
	Rule       string `json:"rule"`
	Expression string `json:"expression"`
	Result     string `json:"result"`
}

type CalculationResult struct {
	Payslip           Payslip
	GrossPay          decimal.Decimal
	TotalCredits      decimal.Decimal
	ApprovedOTMinutes int
	PaidOTMinutes     int
	CappedOTMinutes   int
	Segments          []CalculatedSegment
	Incentives        []CalculatedIncentive
	RuleEvaluations   []CalculatedRuleEvaluation
	ActiveComponents  []SalaryComponentEntry
	PrimaryPolicy     *workpolicy.ResolvedPolicy
	Trace             []CalculationTraceStep
	InputFingerprint  string
	KPIBonus          *KPIBonusInput
}

func ComputeInputFingerprint(input PeriodInput) (string, error) {
	raw, err := json.Marshal(input)
	if err != nil {
		return "", err
	}
	sum := sha256.Sum256(raw)
	return hex.EncodeToString(sum[:]), nil
}

type engineAccum struct {
	basePay         decimal.Decimal
	overtimePay     decimal.Decimal
	holidayPremium  decimal.Decimal
	credits         decimal.Decimal
	stdMinutes      int
	approvedOTMins  int
	paidOTMins      int
	cappedOTMins    int
	monthPaidOT     int
	yearPaidOT      int
	segments        []CalculatedSegment
	trace           []CalculationTraceStep
	primaryPolicy   *workpolicy.ResolvedPolicy
	monthSchedDays  int
	totalBaseSalary decimal.Decimal
	defaultDeptID   *uuid.UUID
}

// CalculatePayslip is a pure, deterministic payroll calculation engine (0 DB calls, 0 side effects).
func CalculatePayslip(input PeriodInput, policies []workpolicy.ResolvedPolicy) (CalculationResult, error) {
	startDate, endDate, err := MonthBoundaries(input.Year, input.Month)
	if err != nil {
		return CalculationResult{}, ErrEngineValidation
	}
	if len(policies) == 0 {
		return CalculationResult{}, ErrEnginePolicyMissing
	}
	fingerprint, err := ComputeInputFingerprint(input)
	if err != nil {
		return CalculationResult{}, err
	}

	activeComponents, totalBaseSalary, salaryChanges := extractActiveComponents(input.SalaryHistory, startDate, endDate)
	defaultDept, deptChanges := extractDepartmentChanges(input.DepartmentID, input.DepartmentChanges, startDate, endDate)
	policyChanges := extractPolicyBoundaries(policies, startDate, endDate)

	segmentsList := BuildSegments(startDate, endDate, append(salaryChanges, policyChanges...), deptChanges)
	if len(segmentsList) == 0 {
		segmentsList = []DateRange{{Start: startDate, End: endDate}}
	}

	acc := &engineAccum{
		yearPaidOT:      input.PriorYTDApprovedOTMinutes,
		segments:        make([]CalculatedSegment, 0, len(segmentsList)),
		trace:           make([]CalculationTraceStep, 0, 16),
		monthSchedDays:  countScheduledWorkdays(input.ScheduledWorkdayCounts, startDate, endDate),
		totalBaseSalary: totalBaseSalary,
		defaultDeptID:   defaultDept,
	}

	for _, seg := range segmentsList {
		if err := evaluateSegment(input, policies, seg, acc); err != nil {
			return CalculationResult{}, err
		}
	}

	totalStdUnits := int32(acc.stdMinutes / 60)
	totalOTUnits := int32(acc.paidOTMins / 60)
	incList, totalIncentives := evaluateIncentivesAndKPI(input, totalStdUnits, acc)
	gross := acc.basePay.Add(acc.overtimePay).Add(acc.holidayPremium).Add(totalIncentives)
	ruleEvals, totalDeductions := evaluateDeductionRules(input.RuleAssignments, gross, acc.basePay, acc)

	now := time.Now().UTC()
	assembled := AssemblePayslip(AssemblePayslipArgs{
		ID: uuid.New(), EmploymentID: input.EmploymentID, StartDate: startDate, EndDate: endDate,
		TotalBasePay: acc.basePay, TotalOvertimePay: acc.overtimePay, TotalHolidayWorkPremium: acc.holidayPremium,
		TotalIncentiveAmount: totalIncentives, DeductionsAmount: totalDeductions,
		TotalStdUnits: totalStdUnits, TotalOTUnits: totalOTUnits, CreatedAt: now, UpdatedAt: now,
	})
	appendFinalTrace(acc, gross, totalDeductions, assembled.NetPay)

	return CalculationResult{
		Payslip: assembled, GrossPay: gross, TotalCredits: acc.credits,
		ApprovedOTMinutes: acc.approvedOTMins, PaidOTMinutes: acc.paidOTMins, CappedOTMinutes: acc.cappedOTMins,
		Segments: acc.segments, Incentives: incList, RuleEvaluations: ruleEvals,
		ActiveComponents: activeComponents, PrimaryPolicy: acc.primaryPolicy,
		Trace: acc.trace, InputFingerprint: fingerprint, KPIBonus: input.KPIBonus,
	}, nil
}

func extractActiveComponents(history []SalaryComponentEntry, startDate, endDate time.Time) ([]SalaryComponentEntry, decimal.Decimal, []time.Time) {
	endExclusive := endDate.AddDate(0, 0, 1)
	activeMap := make(map[uuid.UUID]SalaryComponentEntry)
	changes := make([]time.Time, 0)
	for _, entry := range history {
		if entry.EffectiveAt.After(endDate) {
			continue
		}
		activeMap[entry.DefinitionID] = entry
		if entry.EffectiveAt.After(startDate) && entry.EffectiveAt.Before(endExclusive) {
			changes = append(changes, entry.EffectiveAt)
		}
	}
	total := decimal.Zero
	list := make([]SalaryComponentEntry, 0, len(activeMap))
	for _, entry := range activeMap {
		total = total.Add(entry.Amount)
		list = append(list, entry)
	}
	sort.Slice(list, func(i, j int) bool { return list[i].Code < list[j].Code })
	return list, total, changes
}

func extractDepartmentChanges(baseDept *uuid.UUID, changes []DepartmentChange, startDate, endDate time.Time) (*uuid.UUID, []time.Time) {
	endExclusive := endDate.AddDate(0, 0, 1)
	current := baseDept
	points := make([]time.Time, 0)
	for _, dc := range changes {
		if !dc.EffectiveAt.After(startDate) {
			current = dc.DepartmentID
		} else if dc.EffectiveAt.Before(endExclusive) {
			points = append(points, dc.EffectiveAt)
		}
	}
	return current, points
}

func extractPolicyBoundaries(policies []workpolicy.ResolvedPolicy, startDate, endDate time.Time) []time.Time {
	endExclusive := endDate.AddDate(0, 0, 1)
	points := make([]time.Time, 0)
	for _, p := range policies {
		s, e, ok := p.SegmentBounds(startDate, endExclusive)
		if !ok {
			continue
		}
		if s.After(startDate) && s.Before(endExclusive) {
			points = append(points, s)
		}
		if e.After(startDate) && e.Before(endExclusive) {
			points = append(points, e)
		}
	}
	return points
}

func countScheduledWorkdays(counts map[string]int, startDate, endDate time.Time) int {
	n := 0
	for d := startDate; !d.After(endDate); d = d.AddDate(0, 0, 1) {
		if counts[d.Format("2006-01-02")] > 0 {
			n++
		}
	}
	return n
}

type segmentRateBasis struct {
	stdDays       decimal.Decimal
	stdDaysSource string
	dailyRate     decimal.Decimal
	hourlyRate    decimal.Decimal
	shiftRate     decimal.Decimal
}

func resolveSegmentRateBasis(policy workpolicy.ResolvedPolicy, segBase, hoursPerDay decimal.Decimal, history []SalaryComponentEntry, segStart time.Time, monthSchedDays int) (segmentRateBasis, error) {
	switch policy.PayMethod {
	case workpolicy.PayMethodMonthly:
		return resolveMonthlyRateBasis(policy, segBase, hoursPerDay, monthSchedDays)
	case workpolicy.PayMethodHourly:
		rateVal, ok := resolveComponentAmountAt(history, policy.RateComponentDefinitionID, segStart)
		if !ok || !rateVal.IsPositive() {
			return segmentRateBasis{}, ErrEngineValidation
		}
		return segmentRateBasis{hourlyRate: rateVal, stdDaysSource: "work_policy_hourly"}, nil
	case workpolicy.PayMethodShift:
		rateVal, ok := resolveComponentAmountAt(history, policy.RateComponentDefinitionID, segStart)
		if !ok || !rateVal.IsPositive() {
			return segmentRateBasis{}, ErrEngineValidation
		}
		return segmentRateBasis{shiftRate: rateVal, stdDaysSource: "work_policy_shift"}, nil
	default:
		return segmentRateBasis{}, ErrEngineValidation
	}
}

func resolveMonthlyRateBasis(policy workpolicy.ResolvedPolicy, segBase, hoursPerDay decimal.Decimal, monthSchedDays int) (segmentRateBasis, error) {
	if policy.MonthlyDenominator == nil {
		return segmentRateBasis{}, ErrEngineValidation
	}
	var stdDays decimal.Decimal
	var src string
	switch *policy.MonthlyDenominator {
	case workpolicy.MonthlyDenominatorFixed:
		if policy.MonthlyStandardDays == nil || !policy.MonthlyStandardDays.GreaterThan(decimal.Zero) {
			return segmentRateBasis{}, ErrEngineValidation
		}
		stdDays = *policy.MonthlyStandardDays
		src = "work_policy_fixed"
	case workpolicy.MonthlyDenominatorSchedule:
		if monthSchedDays <= 0 {
			return segmentRateBasis{}, ErrEngineValidation
		}
		stdDays = decimal.NewFromInt(int64(monthSchedDays))
		src = "work_policy_schedule"
	default:
		return segmentRateBasis{}, ErrEngineValidation
	}
	return segmentRateBasis{
		stdDays:       stdDays,
		stdDaysSource: src,
		dailyRate:     rates.CalculateDailyRate(segBase, stdDays),
		hourlyRate:    rates.CalculateHourlyRate(segBase, stdDays, hoursPerDay),
	}, nil
}

func resolveSegmentBaseSalary(history []SalaryComponentEntry, segStart time.Time, fallback decimal.Decimal) decimal.Decimal {
	segBase := decimal.Zero
	seenDef := make(map[uuid.UUID]bool)
	for i := len(history) - 1; i >= 0; i-- {
		item := history[i]
		if !seenDef[item.DefinitionID] && !item.EffectiveAt.After(segStart) {
			segBase = segBase.Add(item.Amount)
			seenDef[item.DefinitionID] = true
		}
	}
	if segBase.IsZero() {
		return fallback
	}
	return segBase
}

func resolveSegmentDepartment(defaultDept *uuid.UUID, changes []DepartmentChange, segStart time.Time) *uuid.UUID {
	segDept := defaultDept
	for _, dc := range changes {
		if !dc.EffectiveAt.After(segStart) {
			segDept = dc.DepartmentID
		}
	}
	return segDept
}

func evaluateSegment(input PeriodInput, policies []workpolicy.ResolvedPolicy, seg DateRange, acc *engineAccum) error {
	policy, found := findPolicyForDate(policies, seg.Start)
	if !found {
		return ErrEnginePolicyMissing
	}
	policyCopy := policy
	acc.primaryPolicy = &policyCopy
	if policy.MinutesPerCredit <= 0 {
		return ErrEngineValidation
	}

	minutesPerCreditDec := decimal.NewFromInt(int64(policy.MinutesPerCredit))
	hoursPerDay := minutesPerCreditDec.Div(decimal.NewFromInt(60))
	segBase := resolveSegmentBaseSalary(input.SalaryHistory, seg.Start, acc.totalBaseSalary)
	segDept := resolveSegmentDepartment(acc.defaultDeptID, input.DepartmentChanges, seg.Start)

	basis, err := resolveSegmentRateBasis(policy, segBase, hoursPerDay, input.SalaryHistory, seg.Start, acc.monthSchedDays)
	if err != nil {
		return err
	}

	segEligibleMins, segPaidOTMins, segCompletedShifts, segOTPay, err := accumulateSegmentDays(input, policy, seg, basis.hourlyRate, acc)
	if err != nil {
		return err
	}

	actualDays := decimal.Zero
	if segEligibleMins > 0 {
		actualDays = decimal.NewFromInt(int64(segEligibleMins)).Div(minutesPerCreditDec)
	}
	segAmount, err := computeSegmentBasePay(policy.PayMethod, basis, actualDays, segEligibleMins, segCompletedShifts, len(input.CompletedShiftCounts))
	if err != nil {
		return err
	}

	acc.basePay = acc.basePay.Add(segAmount)
	acc.overtimePay = acc.overtimePay.Add(segOTPay)
	acc.stdMinutes += segEligibleMins
	acc.credits = acc.credits.Add(actualDays)

	reason := "Standard"
	if seg.Start.Day() != 1 {
		reason = "Policy or Salary Change"
	}
	acc.segments = append(acc.segments, CalculatedSegment{
		ID: uuid.New(), StartDate: seg.Start, EndDate: seg.End, DepartmentID: segDept,
		AppliedBaseSalary: segBase, AppliedStdDays: basis.stdDays, AppliedHoursPerDay: hoursPerDay,
		ActualWorkDays: actualDays, PaidWorkDays: actualDays, Amount: segAmount,
		Reason: reason, StdDaysSource: basis.stdDaysSource, WorkPolicyVersionID: policy.VersionID,
	})
	acc.trace = append(acc.trace, CalculationTraceStep{
		Step: "segment", Rule: fmt.Sprintf("%s:v%d (%s)", policy.GroupCode, policy.VersionNo, policy.PayMethod),
		Expression: fmt.Sprintf("%s..%s eligible=%dm ot=%dm", seg.Start.Format("2006-01-02"), seg.End.Format("2006-01-02"), segEligibleMins, segPaidOTMins),
		Result:     segAmount.StringFixed(2),
	})
	return nil
}

func accumulateSegmentDays(input PeriodInput, policy workpolicy.ResolvedPolicy, seg DateRange, hourlyRate decimal.Decimal, acc *engineAccum) (int, int, int, decimal.Decimal, error) {
	segEligibleMins, segPaidOTMins, segShifts := 0, 0, 0
	segOTPay := decimal.Zero
	for day := seg.Start; !day.After(seg.End); day = day.AddDate(0, 0, 1) {
		dStr := day.Format("2006-01-02")
		summary, hasSummary := input.DailySummaries[dStr]
		segEligibleMins += dayEligibleMinutes(policy, summary, hasSummary, input.PaidLeaveDays[dStr])
		segShifts += input.CompletedShiftCounts[dStr]
		if !hasSummary || summary.ApprovedOTMinutes <= 0 {
			continue
		}
		if policy.PayMethod == workpolicy.PayMethodShift {
			return 0, 0, 0, decimal.Zero, ErrEngineValidation
		}
		paid, pay, err := processDayOvertime(day, dStr, summary, policy, hourlyRate, acc)
		if err != nil {
			return 0, 0, 0, decimal.Zero, err
		}
		segPaidOTMins += paid
		segOTPay = segOTPay.Add(pay)
	}
	return segEligibleMins, segPaidOTMins, segShifts, segOTPay, nil
}

func dayEligibleMinutes(policy workpolicy.ResolvedPolicy, summary DailyAttendanceSummary, hasSummary bool, leave PaidLeaveDay) int {
	mins := 0
	if hasSummary {
		if summary.NetPaidMinutes > 0 {
			mins = summary.NetPaidMinutes
		} else {
			mins = summary.StandardMinutes
		}
	}
	if leave.Fraction.GreaterThan(decimal.Zero) {
		mins += paidLeaveMinutes(policy, leave)
	}
	return mins
}

func applyOTCaps(dStr string, approved int, policy workpolicy.ResolvedPolicy, acc *engineAccum) int {
	payable := approved
	if policy.DailyCapMinutes != nil && payable > *policy.DailyCapMinutes {
		capped := payable - *policy.DailyCapMinutes
		payable = *policy.DailyCapMinutes
		acc.trace = append(acc.trace, CalculationTraceStep{
			Step: "ot_cap_daily", Rule: "daily_cap_minutes",
			Expression: fmt.Sprintf("%s: min(%d, %d)", dStr, approved, *policy.DailyCapMinutes),
			Result:     fmt.Sprintf("capped %d min", capped),
		})
	}
	if policy.MonthlyCapHours != nil {
		limitMins := int(policy.MonthlyCapHours.Mul(decimal.NewFromInt(60)).IntPart())
		remaining := max(0, limitMins-acc.monthPaidOT)
		if payable > remaining {
			capped := payable - remaining
			payable = remaining
			acc.trace = append(acc.trace, CalculationTraceStep{
				Step: "ot_cap_monthly", Rule: "monthly_cap_hours",
				Expression: fmt.Sprintf("%s: remaining %d min of %d min", dStr, remaining, limitMins),
				Result:     fmt.Sprintf("capped %d min", capped),
			})
		}
	}
	if policy.YearlyCapHours != nil {
		limitMins := int(policy.YearlyCapHours.Mul(decimal.NewFromInt(60)).IntPart())
		remaining := max(0, limitMins-acc.yearPaidOT)
		if payable > remaining {
			capped := payable - remaining
			payable = remaining
			acc.trace = append(acc.trace, CalculationTraceStep{
				Step: "ot_cap_yearly", Rule: "yearly_cap_hours",
				Expression: fmt.Sprintf("%s: remaining %d min of %d min", dStr, remaining, limitMins),
				Result:     fmt.Sprintf("capped %d min", capped),
			})
		}
	}
	return payable
}

func processDayOvertime(day time.Time, dStr string, summary DailyAttendanceSummary, policy workpolicy.ResolvedPolicy, hourlyRate decimal.Decimal, acc *engineAccum) (int, decimal.Decimal, error) {
	approved := summary.ApprovedOTMinutes
	acc.approvedOTMins += approved
	payable := applyOTCaps(dStr, approved, policy, acc)
	if payable < approved {
		acc.cappedOTMins += approved - payable
	}
	if payable <= 0 {
		return 0, decimal.Zero, nil
	}
	dayOTPay, ruleSummary, err := evaluateDayOvertimePay(day, summary, payable, hourlyRate, policy.OvertimeRules)
	if err != nil {
		return 0, decimal.Zero, err
	}
	acc.paidOTMins += payable
	acc.monthPaidOT += payable
	acc.yearPaidOT += payable
	acc.trace = append(acc.trace, CalculationTraceStep{
		Step: "ot_eval", Rule: ruleSummary,
		Expression: fmt.Sprintf("%s: %d min @ hourly %s", dStr, payable, hourlyRate.StringFixed(2)),
		Result:     dayOTPay.StringFixed(2),
	})
	return payable, dayOTPay, nil
}

func computeSegmentBasePay(method workpolicy.PayMethod, basis segmentRateBasis, actualDays decimal.Decimal, eligibleMins, completedShifts, totalShiftDays int) (decimal.Decimal, error) {
	switch method {
	case workpolicy.PayMethodMonthly:
		return basis.dailyRate.Mul(actualDays).Round(2), nil
	case workpolicy.PayMethodHourly:
		return basis.hourlyRate.Mul(decimal.NewFromInt(int64(eligibleMins))).Div(decimal.NewFromInt(60)).Round(2), nil
	case workpolicy.PayMethodShift:
		if completedShifts == 0 && totalShiftDays == 0 {
			return decimal.Zero, ErrEngineValidation
		}
		return basis.shiftRate.Mul(decimal.NewFromInt(int64(completedShifts))).Round(2), nil
	default:
		return decimal.Zero, ErrEngineValidation
	}
}

func evaluateIncentivesAndKPI(input PeriodInput, totalStdUnits int32, acc *engineAccum) ([]CalculatedIncentive, decimal.Decimal) {
	out := make([]CalculatedIncentive, 0)
	total := decimal.Zero
	incCtx := incentives.Context{TotalWorkUnits: totalStdUnits}
	for _, inc := range input.IncentivePolicies {
		res, evalErr := incentives.EvaluatePolicy(inc.Policy, inc.Tiers, incCtx)
		if evalErr == nil && res.Amount.GreaterThan(decimal.Zero) {
			total = total.Add(res.Amount)
			out = append(out, CalculatedIncentive{PolicyName: res.PolicyName, Amount: res.Amount, StandardAmount: res.StandardAmount, TierID: res.TierID})
		}
	}
	if input.KPIBonus != nil {
		for _, t := range input.KPIBonus.Trace {
			acc.trace = append(acc.trace, CalculationTraceStep{Step: t["step"], Rule: t["rule"], Expression: t["expression"], Result: t["result"]})
		}
		if input.KPIBonus.Amount.GreaterThan(decimal.Zero) {
			total = total.Add(input.KPIBonus.Amount)
			out = append(out, CalculatedIncentive{PolicyName: "KPI_BONUS", Amount: input.KPIBonus.Amount, StandardAmount: input.KPIBonus.Amount})
		}
	}
	return out, total
}

func evaluateDeductionRules(assignments []rules.Assignment, gross, basePay decimal.Decimal, acc *engineAccum) ([]CalculatedRuleEvaluation, decimal.Decimal) {
	out := make([]CalculatedRuleEvaluation, 0)
	total := decimal.Zero
	if len(assignments) == 0 {
		return out, total
	}
	baseValues := map[string]decimal.Decimal{"gross": gross, "base_pay": basePay, "bhxh_base": basePay}
	ruleResults, err := rules.EvaluateRules(assignments, baseValues)
	if err != nil {
		return out, total
	}
	for _, res := range ruleResults {
		total = total.Add(res.Amount)
		out = append(out, CalculatedRuleEvaluation{RuleCode: res.RuleCode, Amount: res.Amount})
		acc.trace = append(acc.trace, CalculationTraceStep{Step: "deduction", Rule: res.RuleCode, Expression: "payroll rule", Result: res.Amount.StringFixed(2)})
	}
	return out, total
}

func appendFinalTrace(acc *engineAccum, gross, deductions, net decimal.Decimal) {
	acc.trace = append(acc.trace, CalculationTraceStep{
		Step: "work_credit", Rule: "minutes_per_credit", Expression: "segment effective policy minutes", Result: acc.credits.Round(4).String(),
	})
	if acc.primaryPolicy != nil {
		acc.trace = append(acc.trace, CalculationTraceStep{
			Step: "policy", Rule: "work_policy_version",
			Expression: fmt.Sprintf("%s:v%d", acc.primaryPolicy.GroupCode, acc.primaryPolicy.VersionNo),
			Result:     acc.primaryPolicy.VersionID.String(),
		})
	}
	acc.trace = append(acc.trace, CalculationTraceStep{
		Step: "net", Rule: "gross_minus_deductions",
		Expression: fmt.Sprintf("%s - %s", gross.StringFixed(2), deductions.StringFixed(2)),
		Result:     net.StringFixed(2),
	})
}

func findPolicyForDate(policies []workpolicy.ResolvedPolicy, day time.Time) (workpolicy.ResolvedPolicy, bool) {
	for i := len(policies) - 1; i >= 0; i-- {
		if policies[i].CoversDate(day) {
			return policies[i], true
		}
	}
	return workpolicy.ResolvedPolicy{}, false
}

func resolveComponentAmountAt(history []SalaryComponentEntry, defID *uuid.UUID, asOf time.Time) (decimal.Decimal, bool) {
	if defID == nil || *defID == uuid.Nil {
		return decimal.Zero, false
	}
	for i := len(history) - 1; i >= 0; i-- {
		item := history[i]
		if item.DefinitionID == *defID && !item.EffectiveAt.After(asOf) {
			return item.Amount, true
		}
	}
	return decimal.Zero, false
}

func paidLeaveMinutes(policy workpolicy.ResolvedPolicy, leave PaidLeaveDay) int {
	switch policy.PaidLeaveCreditMode {
	case workpolicy.PaidLeaveCreditUnpaid:
		return 0
	case workpolicy.PaidLeaveCreditScheduledMinutes:
		base := leave.ScheduledMinutes
		if base <= 0 {
			base = policy.MinutesPerCredit
		}
		return int(leave.Fraction.Mul(decimal.NewFromInt(int64(base))).Round(0).IntPart())
	default:
		return int(leave.Fraction.Mul(decimal.NewFromInt(int64(policy.MinutesPerCredit))).Round(0).IntPart())
	}
}

func classifyDayType(summary DailyAttendanceSummary) workpolicy.OvertimeDayType {
	if summary.IsHoliday {
		return workpolicy.OvertimeDayHoliday
	}
	if summary.IsWeekend {
		return workpolicy.OvertimeDayWeekend
	}
	return workpolicy.OvertimeDayWeekday
}

func ruleMatchesDay(rule workpolicy.OvertimeRule, day time.Time, dayType workpolicy.OvertimeDayType) bool {
	if rule.DayType != workpolicy.OvertimeDayAny && rule.DayType != dayType {
		return false
	}
	dayBit := 1 << int(day.Weekday())
	return rule.DaysOfWeek&dayBit != 0
}

func ruleMatchesClockMinute(rule workpolicy.OvertimeRule, minuteOfDay int) (bool, error) {
	if rule.TimeStart == nil || rule.TimeEnd == nil {
		return true, nil
	}
	startMin, err := workpolicy.ParseClockMinutes(*rule.TimeStart)
	if err != nil {
		return false, err
	}
	endMin, err := workpolicy.ParseClockMinutes(*rule.TimeEnd)
	if err != nil {
		return false, err
	}
	m := ((minuteOfDay % 1440) + 1440) % 1440
	if startMin == endMin {
		return true, nil
	}
	if startMin < endMin {
		return m >= startMin && m < endMin, nil
	}
	return m >= startMin || m < endMin, nil
}

func filterMatchingOTRules(day time.Time, dayType workpolicy.OvertimeDayType, rulesList []workpolicy.OvertimeRule) ([]workpolicy.OvertimeRule, error) {
	sorted := make([]workpolicy.OvertimeRule, len(rulesList))
	copy(sorted, rulesList)
	sort.SliceStable(sorted, func(i, j int) bool { return sorted[i].Priority > sorted[j].Priority })
	candidates := make([]workpolicy.OvertimeRule, 0, len(sorted))
	for _, r := range sorted {
		if !r.Multiplier.IsPositive() {
			return nil, ErrEngineValidation
		}
		if ruleMatchesDay(r, day, dayType) {
			candidates = append(candidates, r)
		}
	}
	if len(candidates) == 0 {
		return nil, ErrEngineValidation
	}
	return candidates, nil
}

func matchRuleForMinute(candidates []workpolicy.OvertimeRule, minuteOfDay int) (workpolicy.OvertimeRule, error) {
	for i := range candidates {
		ok, err := ruleMatchesClockMinute(candidates[i], minuteOfDay)
		if err != nil {
			return workpolicy.OvertimeRule{}, ErrEngineValidation
		}
		if ok {
			return candidates[i], nil
		}
	}
	return candidates[len(candidates)-1], nil
}

func evaluateWindowedOTPay(slices []OTTimeSlice, payableMinutes int, hourlyRate decimal.Decimal, candidates []workpolicy.OvertimeRule) (decimal.Decimal, error) {
	remaining := payableMinutes
	byMultiplier := make(map[string]int)
	for _, sl := range slices {
		for m := sl.StartMinute; m < sl.EndMinute && remaining > 0; m++ {
			matched, err := matchRuleForMinute(candidates, m)
			if err != nil {
				return decimal.Zero, err
			}
			byMultiplier[matched.Multiplier.String()]++
			remaining--
		}
	}
	if remaining > 0 {
		fallback := pickUntimedCandidate(candidates)
		byMultiplier[fallback.Multiplier.String()] += remaining
	}
	keys := make([]string, 0, len(byMultiplier))
	for k := range byMultiplier {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	totalPay := decimal.Zero
	for _, k := range keys {
		mult := decimal.RequireFromString(k)
		mins := byMultiplier[k]
		totalPay = totalPay.Add(hourlyRate.Mul(decimal.NewFromInt(int64(mins))).Div(decimal.NewFromInt(60)).Mul(mult))
	}
	return totalPay.Round(2), nil
}

func evaluateDayOvertimePay(day time.Time, summary DailyAttendanceSummary, payableMinutes int, hourlyRate decimal.Decimal, rulesList []workpolicy.OvertimeRule) (decimal.Decimal, string, error) {
	if len(rulesList) == 0 {
		return decimal.Zero, "", ErrEngineValidation
	}
	dayType := classifyDayType(summary)
	candidates, err := filterMatchingOTRules(day, dayType, rulesList)
	if err != nil {
		return decimal.Zero, "", err
	}
	if len(summary.OTTimeSlices) > 0 {
		pay, err := evaluateWindowedOTPay(summary.OTTimeSlices, payableMinutes, hourlyRate, candidates)
		return pay, fmt.Sprintf("%s_windowed", dayType), err
	}
	selected := pickUntimedCandidate(candidates)
	pay := hourlyRate.Mul(decimal.NewFromInt(int64(payableMinutes))).Div(decimal.NewFromInt(60)).Mul(selected.Multiplier).Round(2)
	return pay, fmt.Sprintf("%s:%sx", dayType, selected.Multiplier.StringFixed(2)), nil
}

func pickUntimedCandidate(candidates []workpolicy.OvertimeRule) workpolicy.OvertimeRule {
	for _, r := range candidates {
		if r.TimeStart == nil && r.TimeEnd == nil {
			return r
		}
	}
	return candidates[0]
}
