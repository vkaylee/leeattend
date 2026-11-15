package payroll

import "github.com/vkaylee/leeattend/apiserver/internal/domain/ids"

type PayrollPeriodMutation struct {
	EmployerID      ids.EmployerID
	Year            int
	Month           int
	ExpectedVersion int64
	ActorID         ids.UserID
	Reason          string
}
