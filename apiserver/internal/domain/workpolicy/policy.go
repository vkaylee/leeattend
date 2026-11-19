package workpolicy

import (
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
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

var (
	ErrPolicyNameRequired        = errors.New("work policy: name is required")
	ErrReferenceRequired         = errors.New("work policy: policy reference is required")
	ErrMinutesPerCreditRequired  = errors.New("work policy: minutes per credit must be positive")
	ErrEligibleMinutesInvalid    = errors.New("work policy: eligible minutes must not be negative")
	ErrPayMethodInvalid          = errors.New("work policy: pay method is invalid")
	ErrMonthlyDenominatorInvalid = errors.New("work policy: monthly denominator is invalid")
	ErrMonthlyDaysRequired       = errors.New("work policy: fixed monthly standard days must be positive")
	ErrMonthlyDaysUnexpected     = errors.New("work policy: monthly standard days only applies to fixed monthly pay")
	ErrRateComponentRequired     = errors.New("work policy: hourly and shift pay require a rate component")
	ErrRateComponentUnexpected   = errors.New("work policy: monthly pay cannot define a rate component")
	ErrEffectiveRangeInvalid           = errors.New("work policy: effective range is invalid")
	ErrGraceMinutesInvalid             = errors.New("work policy: grace minutes must not be negative")
	ErrEarlyOTMaxInvalid               = errors.New("work policy: early OT max must not be negative")
	ErrOTAggregationModeInvalid        = errors.New("work policy: OT aggregation mode is invalid")
)

type WorkCreditPolicy struct {
	ID                               WorkCreditPolicyID
	Name                             string
	MinutesPerCredit                 int
	AttendancePolicyID               ids.AttendancePolicyID
	WorkingPatternID                 ids.WorkingPatternID
	LateArrivalGraceMinutes          int
	EarlyOTEnabled                   bool
	EarlyOTQualificationGraceMinutes int
	EarlyOTMaxMinutes                *int
	BreakAutoDeduct                  bool
	BreakIsOvertime                  bool
	OTAggregationMode                string
}

type WorkCreditResult struct {
	EligibleMinutes int
	Credits         decimal.Decimal
}

func (p WorkCreditPolicy) Calculate(eligibleMinutes int) (WorkCreditResult, error) {
	if err := p.Validate(); err != nil {
		return WorkCreditResult{}, err
	}
	if eligibleMinutes < 0 {
		return WorkCreditResult{}, ErrEligibleMinutesInvalid
	}
	return WorkCreditResult{
		EligibleMinutes: eligibleMinutes,
		Credits:         decimal.NewFromInt(int64(eligibleMinutes)).Div(decimal.NewFromInt(int64(p.MinutesPerCredit))),
	}, nil
}

func (p WorkCreditPolicy) Validate() error {
	if strings.TrimSpace(p.Name) == "" {
		return ErrPolicyNameRequired
	}
	if p.MinutesPerCredit <= 0 {
		return ErrMinutesPerCreditRequired
	}
	if uuid.UUID(p.AttendancePolicyID) == uuid.Nil || uuid.UUID(p.WorkingPatternID) == uuid.Nil {
		return ErrReferenceRequired
	}
	if p.LateArrivalGraceMinutes < 0 || p.EarlyOTQualificationGraceMinutes < 0 {
		return ErrGraceMinutesInvalid
	}
	if p.EarlyOTMaxMinutes != nil && *p.EarlyOTMaxMinutes < 0 {
		return ErrEarlyOTMaxInvalid
	}
	if p.OTAggregationMode != "aggregate" && p.OTAggregationMode != "per_segment" {
		return ErrOTAggregationModeInvalid
	}
	return nil
}

type PayBasisPolicy struct {
	ID                        PayBasisPolicyID
	Name                      string
	Method                    PayMethod
	MonthlyDenominator        *MonthlyDenominator
	MonthlyStandardDays       *decimal.Decimal
	RateComponentDefinitionID *ids.RateComponentDefinitionID
}

func (p PayBasisPolicy) Validate() error {
	if strings.TrimSpace(p.Name) == "" {
		return ErrPolicyNameRequired
	}
	if p.Method == PayMethodMonthly {
		if p.RateComponentDefinitionID != nil {
			return ErrRateComponentUnexpected
		}
		return p.validateMonthly()
	}
	if p.Method == PayMethodHourly || p.Method == PayMethodShift {
		if p.RateComponentDefinitionID == nil || uuid.UUID(*p.RateComponentDefinitionID) == uuid.Nil {
			return ErrRateComponentRequired
		}
		return p.validateNonMonthly()
	}
	return ErrPayMethodInvalid
}

func (p PayBasisPolicy) validateMonthly() error {
	if p.MonthlyDenominator == nil {
		return ErrMonthlyDenominatorInvalid
	}
	if *p.MonthlyDenominator == MonthlyDenominatorFixed {
		if p.MonthlyStandardDays == nil || !p.MonthlyStandardDays.GreaterThan(decimal.Zero) {
			return ErrMonthlyDaysRequired
		}
		return nil
	}
	if *p.MonthlyDenominator == MonthlyDenominatorSchedule {
		if p.MonthlyStandardDays != nil {
			return ErrMonthlyDaysUnexpected
		}
		return nil
	}
	return ErrMonthlyDenominatorInvalid
}

func (p PayBasisPolicy) validateNonMonthly() error {
	if p.MonthlyDenominator != nil || p.MonthlyStandardDays != nil {
		return ErrMonthlyDaysUnexpected
	}
	return nil
}

type LaborPolicyGroup struct {
	ID                 GroupID
	Name               string
	WorkCreditPolicyID WorkCreditPolicyID
	PayBasisPolicyID   PayBasisPolicyID
	OvertimePolicyID   ids.OvertimePolicyID
	EffectiveFrom      time.Time
	EffectiveTo        *time.Time
}

func (g LaborPolicyGroup) Validate() error {
	if strings.TrimSpace(g.Name) == "" {
		return ErrPolicyNameRequired
	}
	if uuid.UUID(g.WorkCreditPolicyID) == uuid.Nil || uuid.UUID(g.PayBasisPolicyID) == uuid.Nil || uuid.UUID(g.OvertimePolicyID) == uuid.Nil {
		return ErrReferenceRequired
	}
	if g.EffectiveFrom.IsZero() || (g.EffectiveTo != nil && !g.EffectiveTo.After(g.EffectiveFrom)) {
		return ErrEffectiveRangeInvalid
	}
	return nil
}

type Membership struct {
	EmploymentID  ids.EmploymentID
	GroupID       GroupID
	EffectiveFrom time.Time
	EffectiveTo   *time.Time
}

func (m Membership) Validate() error {
	if uuid.UUID(m.EmploymentID) == uuid.Nil || uuid.UUID(m.GroupID) == uuid.Nil {
		return ErrReferenceRequired
	}
	if m.EffectiveFrom.IsZero() || (m.EffectiveTo != nil && !m.EffectiveTo.After(m.EffectiveFrom)) {
		return ErrEffectiveRangeInvalid
	}
	return nil
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
