package workpolicy

import (
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

type VersionDraftFields struct {
	EffectiveFrom                    string
	EffectiveTo                      *string
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

type CreateGroupInput struct {
	Code        string
	Name        string
	Description *string
	Draft       VersionDraftFields
}

type CreateVersionInput struct {
	GroupID            GroupID
	CloneFromVersionID *uuid.UUID
	Draft              *VersionDraftFields
}

type PatchVersionInput struct {
	GroupID            GroupID
	VersionID          VersionID
	ExpectedRowVersion int64
	Draft              VersionDraftFields
}

type PublishVersionInput struct {
	GroupID            GroupID
	VersionID          VersionID
	ExpectedRowVersion int64
	IdempotencyKey     string
	AutoClosePrevious  bool
	ActorID            ids.UserID
}

type CreateMembershipInput struct {
	GroupID                  GroupID
	EmploymentRelationshipID uuid.UUID
	EffectiveFrom            string
	EffectiveTo              *string
	ActorID                  ids.UserID
}

type TransferMembershipInput struct {
	EmploymentRelationshipID uuid.UUID
	TargetGroupID            GroupID
	EffectiveDate            string
	ActorID                  ids.UserID
}
