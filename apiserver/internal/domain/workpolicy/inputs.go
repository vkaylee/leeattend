package workpolicy

import (
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

type CreateGroupInput struct {
	Name                      string
	MinutesPerCredit          int
	AttendancePolicyID        ids.AttendancePolicyID
	WorkingPatternID          ids.WorkingPatternID
	PayMethod                 PayMethod
	MonthlyDenominator        *MonthlyDenominator
	MonthlyStandardDays       *decimal.Decimal
	RateComponentDefinitionID *ids.RateComponentDefinitionID
	OvertimePolicyID          ids.OvertimePolicyID
	EffectiveFrom             string
	EffectiveTo               *string
}

type CreateGroupVersionInput struct {
	GroupID                   GroupID
	Name                      string
	MinutesPerCredit          int
	AttendancePolicyID        ids.AttendancePolicyID
	WorkingPatternID          ids.WorkingPatternID
	PayMethod                 PayMethod
	MonthlyDenominator        *MonthlyDenominator
	MonthlyStandardDays       *decimal.Decimal
	RateComponentDefinitionID *ids.RateComponentDefinitionID
	OvertimePolicyID          ids.OvertimePolicyID
	EffectiveFrom             string
	EffectiveTo               *string
}

type CreateMembershipInput struct {
	GroupID       GroupID
	EmploymentID  ids.EmploymentID
	EffectiveFrom string
	EffectiveTo   *string
	ActorID       ids.UserID
}
