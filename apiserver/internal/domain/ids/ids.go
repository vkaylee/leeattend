// Package ids defines semantic identifiers shared across domain boundaries.
package ids

import "github.com/google/uuid"

type UserID uuid.UUID

func ParseUserID(value string) (UserID, error) {
	parsed, err := uuid.Parse(value)
	if err != nil {
		return UserID{}, err
	}
	return UserID(parsed), nil
}

func (id UserID) UUID() uuid.UUID {
	return uuid.UUID(id)
}

func (id UserID) String() string {
	return id.UUID().String()
}
type EmployerID uuid.UUID
type DepartmentID uuid.UUID
type EmployeeID uuid.UUID
type PayrollInputBatchID uuid.UUID
type PayrollInputID uuid.UUID
type PayrollFormulaRuleID uuid.UUID
type PayrollFormulaRuleVersionID uuid.UUID
type PayrollRuleID uuid.UUID
type PayrollRuleTierID uuid.UUID
type PayrollRuleAssignmentID uuid.UUID
type PayrollIncentivePolicyID uuid.UUID
type EmploymentID uuid.UUID
type AttendancePolicyID uuid.UUID
type WorkingPatternID uuid.UUID
type OvertimePolicyID uuid.UUID
type RateComponentDefinitionID uuid.UUID
