package workpolicy

import (
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type VersionStatus string

const (
	StatusDraft     VersionStatus = "draft"
	StatusPublished VersionStatus = "published"
	StatusArchived  VersionStatus = "archived"
)

type PayMethod string

const (
	PayMethodMonthly PayMethod = "monthly"
	PayMethodHourly  PayMethod = "hourly"
	PayMethodShift   PayMethod = "shift"
)

type MonthlyDenominator string

const (
	MonthlyDenominatorFixed    MonthlyDenominator = "fixed"
	MonthlyDenominatorSchedule MonthlyDenominator = "schedule"
)

type PaidLeaveCreditMode string

const (
	PaidLeaveCreditStandardDay      PaidLeaveCreditMode = "standard_day"
	PaidLeaveCreditScheduledMinutes PaidLeaveCreditMode = "scheduled_minutes"
	PaidLeaveCreditUnpaid           PaidLeaveCreditMode = "unpaid"
)

type OvertimeDayType string

const (
	OvertimeDayWeekday OvertimeDayType = "weekday"
	OvertimeDayWeekend OvertimeDayType = "weekend"
	OvertimeDayHoliday OvertimeDayType = "holiday"
	OvertimeDayAny     OvertimeDayType = "any"
)

var (
	ErrPolicyCodeRequired        = errors.New("work policy: code is required")
	ErrPolicyNameRequired        = errors.New("work policy: name is required")
	ErrReferenceRequired         = errors.New("work policy: policy reference is required")
	ErrMinutesPerCreditRequired  = errors.New("work policy: minutes per credit must be positive")
	ErrWorkUnitMinutesRequired   = errors.New("work policy: work unit minutes must be positive")
	ErrRoundingPolicyInvalid     = errors.New("work policy: rounding policy is invalid")
	ErrEarlyArrivalInvalid       = errors.New("work policy: early arrival strategy is invalid")
	ErrEligibleMinutesInvalid    = errors.New("work policy: eligible minutes must not be negative")
	ErrPayMethodInvalid          = errors.New("work policy: pay method is invalid")
	ErrMonthlyDenominatorInvalid = errors.New("work policy: monthly denominator is invalid")
	ErrMonthlyDaysRequired       = errors.New("work policy: fixed monthly standard days must be positive")
	ErrMonthlyDaysUnexpected     = errors.New("work policy: monthly standard days only applies to fixed monthly pay")
	ErrRateComponentRequired     = errors.New("work policy: hourly and shift pay require a rate component")
	ErrRateComponentUnexpected   = errors.New("work policy: monthly pay cannot define a rate component")
	ErrPaidLeaveModeInvalid      = errors.New("work policy: paid leave credit mode is invalid")
	ErrEffectiveRangeInvalid     = errors.New("work policy: effective range is invalid")
	ErrGraceMinutesInvalid       = errors.New("work policy: grace minutes must not be negative")
	ErrEarlyOTMaxInvalid         = errors.New("work policy: early OT max must not be negative")
	ErrOTAggregationModeInvalid  = errors.New("work policy: OT aggregation mode is invalid")
	ErrOvertimeCapInvalid        = errors.New("work policy: overtime cap must not be negative")
	ErrOvertimeRuleInvalid       = errors.New("work policy: overtime rule is invalid")
)

type ValidationIssue struct {
	Code     string `json:"code"`
	Field    string `json:"field"`
	Severity string `json:"severity"`
	Message  string `json:"message"`
}

type OvertimeRule struct {
	Priority   int             `json:"priority"`
	DayType    OvertimeDayType `json:"day_type"`
	DaysOfWeek int             `json:"days_of_week"`
	TimeStart  *string         `json:"time_start,omitempty"`
	TimeEnd    *string         `json:"time_end,omitempty"`
	Multiplier decimal.Decimal `json:"multiplier"`
}

type OvertimePolicyVersionSpec struct {
	DailyCapMinutes *int
	MonthlyCapHours *decimal.Decimal
	YearlyCapHours  *decimal.Decimal
	Rules           []OvertimeRule
}

func ParseClockMinutes(raw string) (int, error) {
	trimmed := strings.TrimSpace(raw)
	for _, layout := range []string{"15:04:05", "15:04"} {
		if t, err := time.Parse(layout, trimmed); err == nil {
			return t.Hour()*60 + t.Minute(), nil
		}
	}
	return 0, ErrOvertimeRuleInvalid
}

func validClockWindow(start, end *string) error {
	hasStart := start != nil && strings.TrimSpace(*start) != ""
	hasEnd := end != nil && strings.TrimSpace(*end) != ""
	if hasStart != hasEnd {
		return ErrOvertimeRuleInvalid
	}
	if !hasStart {
		return nil
	}
	if _, err := ParseClockMinutes(*start); err != nil {
		return ErrOvertimeRuleInvalid
	}
	if _, err := ParseClockMinutes(*end); err != nil {
		return ErrOvertimeRuleInvalid
	}
	return nil
}

func (r OvertimeRule) Validate() error {
	switch r.DayType {
	case OvertimeDayWeekday, OvertimeDayWeekend, OvertimeDayHoliday, OvertimeDayAny:
	default:
		return ErrOvertimeRuleInvalid
	}
	if r.DaysOfWeek < 0 || r.DaysOfWeek > 127 || !r.Multiplier.GreaterThan(decimal.Zero) {
		return ErrOvertimeRuleInvalid
	}
	return validClockWindow(r.TimeStart, r.TimeEnd)
}

func (s OvertimePolicyVersionSpec) Validate() error {
	if s.DailyCapMinutes != nil && *s.DailyCapMinutes < 0 {
		return ErrOvertimeCapInvalid
	}
	if s.MonthlyCapHours != nil && s.MonthlyCapHours.IsNegative() {
		return ErrOvertimeCapInvalid
	}
	if s.YearlyCapHours != nil && s.YearlyCapHours.IsNegative() {
		return ErrOvertimeCapInvalid
	}
	for _, rule := range s.Rules {
		if err := rule.Validate(); err != nil {
			return err
		}
	}
	return nil
}

type WorkCreditResult struct {
	EligibleMinutes int
	Credits         decimal.Decimal
}

type VersionRules struct {
	EffectiveFrom                    time.Time
	EffectiveTo                      *time.Time
	MinutesPerCredit                 int
	WorkUnitMinutes                  int
	RoundingPolicy                   string
	EarlyArrivalStrategy             string
	LateArrivalGraceMinutes          int
	EarlyOTEnabled                   bool
	EarlyOTQualificationGraceMinutes int
	EarlyOTMaxMinutes                *int
	BreakAutoDeduct                  bool
	BreakIsOvertime                  bool
	OTAggregationMode                string
	WorkingPatternID                 *uuid.UUID
	PayMethod                        PayMethod
	MonthlyDenominator               *MonthlyDenominator
	MonthlyStandardDays              *decimal.Decimal
	RateComponentDefinitionID        *uuid.UUID
	PaidLeaveCreditMode              PaidLeaveCreditMode
	OvertimePolicyVersionID          uuid.UUID
}

func (v VersionRules) CalculateCredits(eligibleMinutes int) (WorkCreditResult, error) {
	if v.MinutesPerCredit <= 0 {
		return WorkCreditResult{}, ErrMinutesPerCreditRequired
	}
	if eligibleMinutes < 0 {
		return WorkCreditResult{}, ErrEligibleMinutesInvalid
	}
	return WorkCreditResult{
		EligibleMinutes: eligibleMinutes,
		Credits:         decimal.NewFromInt(int64(eligibleMinutes)).Div(decimal.NewFromInt(int64(v.MinutesPerCredit))),
	}, nil
}

// Validate returns structured blocker and warning issues for a work policy version.
func (v VersionRules) Validate() ([]ValidationIssue, []ValidationIssue) {
	blockers := v.validateAttendanceRules()
	payBlockers, warnings := v.validatePayBasis()
	blockers = append(blockers, payBlockers...)
	warnings = append(warnings, v.collectWarnings()...)
	return blockers, warnings
}

func (v VersionRules) validateAttendanceRules() []ValidationIssue {
	checks := []struct {
		invalid bool
		code    string
		field   string
		msg     string
	}{
		{v.EffectiveFrom.IsZero() || (v.EffectiveTo != nil && !v.EffectiveTo.After(v.EffectiveFrom)), "effective_range_invalid", "effective_from", ErrEffectiveRangeInvalid.Error()},
		{v.MinutesPerCredit <= 0, "minutes_per_credit_invalid", "minutes_per_credit", ErrMinutesPerCreditRequired.Error()},
		{v.WorkUnitMinutes <= 0, "work_unit_minutes_invalid", "work_unit_minutes", ErrWorkUnitMinutesRequired.Error()},
		{!isAllowedValue(v.RoundingPolicy, "none", "floor", "round", "ceil"), "rounding_policy_invalid", "rounding_policy", ErrRoundingPolicyInvalid.Error()},
		{!isAllowedValue(v.EarlyArrivalStrategy, "allow", "clamp", "block"), "early_arrival_strategy_invalid", "early_arrival_strategy", ErrEarlyArrivalInvalid.Error()},
		{v.LateArrivalGraceMinutes < 0, "late_arrival_grace_invalid", "late_arrival_grace_minutes", ErrGraceMinutesInvalid.Error()},
		{v.EarlyOTQualificationGraceMinutes < 0, "early_ot_grace_invalid", "early_ot_qualification_grace_minutes", ErrGraceMinutesInvalid.Error()},
		{v.EarlyOTMaxMinutes != nil && *v.EarlyOTMaxMinutes < 0, "early_ot_max_invalid", "early_ot_max_minutes", ErrEarlyOTMaxInvalid.Error()},
		{v.EarlyOTEnabled && v.EarlyArrivalStrategy == "block", "early_ot_conflicts_with_block", "early_ot_enabled", "early OT cannot be enabled when early arrival strategy is block"},
		{!isAllowedValue(v.OTAggregationMode, "aggregate", "per_segment"), "ot_aggregation_mode_invalid", "ot_aggregation_mode", ErrOTAggregationModeInvalid.Error()},
		{!isAllowedValue(string(v.PaidLeaveCreditMode), string(PaidLeaveCreditStandardDay), string(PaidLeaveCreditScheduledMinutes), string(PaidLeaveCreditUnpaid)), "paid_leave_credit_mode_invalid", "paid_leave_credit_mode", ErrPaidLeaveModeInvalid.Error()},
		{v.OvertimePolicyVersionID == uuid.Nil, "overtime_policy_version_required", "overtime_policy_version_id", ErrReferenceRequired.Error()},
	}
	blockers := make([]ValidationIssue, 0)
	for _, c := range checks {
		if c.invalid {
			blockers = append(blockers, ValidationIssue{Code: c.code, Field: c.field, Severity: "blocker", Message: c.msg})
		}
	}
	return blockers
}

func (v VersionRules) validatePayBasis() ([]ValidationIssue, []ValidationIssue) {
	var blockers, warnings []ValidationIssue
	addBlocker := func(code, field, msg string) {
		blockers = append(blockers, ValidationIssue{Code: code, Field: field, Severity: "blocker", Message: msg})
	}
	switch v.PayMethod {
	case PayMethodMonthly:
		if v.RateComponentDefinitionID != nil && *v.RateComponentDefinitionID != uuid.Nil {
			addBlocker("rate_component_unexpected", "rate_component_definition_id", ErrRateComponentUnexpected.Error())
		}
		mb, mw := v.validateMonthlyBasis()
		blockers = append(blockers, mb...)
		warnings = append(warnings, mw...)
	case PayMethodHourly, PayMethodShift:
		if v.MonthlyDenominator != nil || v.MonthlyStandardDays != nil {
			addBlocker("monthly_fields_unexpected", "monthly_denominator", ErrMonthlyDaysUnexpected.Error())
		}
		if v.RateComponentDefinitionID == nil || *v.RateComponentDefinitionID == uuid.Nil {
			addBlocker("rate_component_required", "rate_component_definition_id", ErrRateComponentRequired.Error())
		}
	default:
		addBlocker("pay_method_invalid", "pay_method", ErrPayMethodInvalid.Error())
	}
	return blockers, warnings
}

func (v VersionRules) validateMonthlyBasis() ([]ValidationIssue, []ValidationIssue) {
	var blockers, warnings []ValidationIssue
	if v.MonthlyDenominator == nil {
		return []ValidationIssue{{Code: "monthly_denominator_required", Field: "monthly_denominator", Severity: "blocker", Message: ErrMonthlyDenominatorInvalid.Error()}}, nil
	}
	switch *v.MonthlyDenominator {
	case MonthlyDenominatorFixed:
		if v.MonthlyStandardDays == nil || !v.MonthlyStandardDays.GreaterThan(decimal.Zero) {
			blockers = append(blockers, ValidationIssue{Code: "monthly_standard_days_required", Field: "monthly_standard_days", Severity: "blocker", Message: ErrMonthlyDaysRequired.Error()})
		}
	case MonthlyDenominatorSchedule:
		if v.MonthlyStandardDays != nil {
			blockers = append(blockers, ValidationIssue{Code: "monthly_standard_days_unexpected", Field: "monthly_standard_days", Severity: "blocker", Message: ErrMonthlyDaysUnexpected.Error()})
		}
		if v.WorkingPatternID == nil || *v.WorkingPatternID == uuid.Nil {
			warnings = append(warnings, ValidationIssue{Code: "schedule_denominator_no_pattern", Field: "working_pattern_id", Severity: "warning", Message: "schedule monthly denominator has no default working pattern; relies on published schedules"})
		}
	default:
		blockers = append(blockers, ValidationIssue{Code: "monthly_denominator_invalid", Field: "monthly_denominator", Severity: "blocker", Message: ErrMonthlyDenominatorInvalid.Error()})
	}
	return blockers, warnings
}

func (v VersionRules) collectWarnings() []ValidationIssue {
	warnings := make([]ValidationIssue, 0)
	if v.BreakAutoDeduct && v.BreakIsOvertime {
		warnings = append(warnings, ValidationIssue{Code: "break_deduct_and_overtime", Field: "break_is_overtime", Severity: "warning", Message: "both break_auto_deduct and break_is_overtime are enabled"})
	}
	if v.MinutesPerCredit > 0 && v.WorkUnitMinutes > 0 && v.MinutesPerCredit%v.WorkUnitMinutes != 0 {
		warnings = append(warnings, ValidationIssue{Code: "work_unit_not_divisor", Field: "work_unit_minutes", Severity: "warning", Message: fmt.Sprintf("minutes_per_credit (%d) is not evenly divisible by work_unit_minutes (%d)", v.MinutesPerCredit, v.WorkUnitMinutes)})
	}
	return warnings
}

func isAllowedValue(val string, allowed ...string) bool {
	for _, a := range allowed {
		if val == a {
			return true
		}
	}
	return false
}

func (v VersionRules) ValidateStrict() error {
	blockers, _ := v.Validate()
	if len(blockers) > 0 {
		return errors.New(blockers[0].Message)
	}
	return nil
}

type Membership struct {
	EmploymentRelationshipID uuid.UUID
	GroupID                  GroupID
	EffectiveFrom            time.Time
	EffectiveTo              *time.Time
}

func (m Membership) Validate() error {
	if m.EmploymentRelationshipID == uuid.Nil || uuid.UUID(m.GroupID) == uuid.Nil {
		return ErrReferenceRequired
	}
	if m.EffectiveFrom.IsZero() || (m.EffectiveTo != nil && !m.EffectiveTo.After(m.EffectiveFrom)) {
		return ErrEffectiveRangeInvalid
	}
	return nil
}

// ResolvedPolicy is the single canonical point-in-time / interval policy snapshot
// returned by WorkPolicyResolver for attendance, reports, payroll, and simulation.
type ResolvedPolicy struct {
	MembershipID                     uuid.UUID
	EmploymentRelationshipID         uuid.UUID
	GroupID                          uuid.UUID
	GroupCode                        string
	GroupName                        string
	VersionID                        uuid.UUID
	VersionNo                        int
	Status                           VersionStatus
	RowVersion                       int64
	EffectiveFrom                    time.Time
	EffectiveTo                      *time.Time
	MembershipEffectiveFrom          time.Time
	MembershipEffectiveTo            *time.Time
	MinutesPerCredit                 int
	WorkUnitMinutes                  int
	RoundingPolicy                   string
	EarlyArrivalStrategy             string
	LateArrivalGraceMinutes          int
	EarlyOTEnabled                   bool
	EarlyOTQualificationGraceMinutes int
	EarlyOTMaxMinutes                *int
	BreakAutoDeduct                  bool
	BreakIsOvertime                  bool
	OTAggregationMode                string
	WorkingPatternID                 *uuid.UUID
	PayMethod                        PayMethod
	MonthlyDenominator               *MonthlyDenominator
	MonthlyStandardDays              *decimal.Decimal
	RateComponentDefinitionID        *uuid.UUID
	PaidLeaveCreditMode              PaidLeaveCreditMode
	OvertimePolicyVersionID          uuid.UUID
	OvertimePolicyID                 uuid.UUID
	OvertimePolicyVersionNo          int
	DailyCapMinutes                  *int
	MonthlyCapHours                  *decimal.Decimal
	YearlyCapHours                   *decimal.Decimal
	OvertimeRules                    []OvertimeRule
}

// SegmentBounds returns the inclusive-start, exclusive-end intersection of
// membership window, version window, and [rangeStart, rangeEndExclusive).
func (p ResolvedPolicy) SegmentBounds(rangeStart, rangeEndExclusive time.Time) (time.Time, time.Time, bool) {
	start := rangeStart
	if p.MembershipEffectiveFrom.After(start) {
		start = p.MembershipEffectiveFrom
	}
	if p.EffectiveFrom.After(start) {
		start = p.EffectiveFrom
	}
	end := rangeEndExclusive
	if p.MembershipEffectiveTo != nil && p.MembershipEffectiveTo.Before(end) {
		end = *p.MembershipEffectiveTo
	}
	if p.EffectiveTo != nil && p.EffectiveTo.Before(end) {
		end = *p.EffectiveTo
	}
	if !end.After(start) {
		return time.Time{}, time.Time{}, false
	}
	return start, end, true
}

func (p ResolvedPolicy) CoversDate(date time.Time) bool {
	day := time.Date(date.Year(), date.Month(), date.Day(), 0, 0, 0, 0, time.UTC)
	if day.Before(p.MembershipEffectiveFrom) || (p.MembershipEffectiveTo != nil && !day.Before(*p.MembershipEffectiveTo)) {
		return false
	}
	if day.Before(p.EffectiveFrom) || (p.EffectiveTo != nil && !day.Before(*p.EffectiveTo)) {
		return false
	}
	return true
}

func IntervalsOverlap(leftFrom time.Time, leftTo *time.Time, rightFrom time.Time, rightTo *time.Time) bool {
	if leftTo != nil && !leftTo.After(leftFrom) || rightTo != nil && !rightTo.After(rightFrom) {
		return false
	}
	leftEnd, rightEnd := time.Time{}, time.Time{}
	if leftTo != nil {
		leftEnd = *leftTo
	}
	if rightTo != nil {
		rightEnd = *rightTo
	}
	return (rightEnd.IsZero() || leftFrom.Before(rightEnd)) && (leftEnd.IsZero() || rightFrom.Before(leftEnd))
}
