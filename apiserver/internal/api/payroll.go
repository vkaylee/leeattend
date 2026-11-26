package api

import (
	"errors"
	"net/http"

	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrPayrollValidation       = errors.New("payroll: validation failed")
	ErrPayrollPolicyMissing    = errors.New("payroll: work policy is missing")
	ErrComponentNotFound       = errors.New("payroll: component not found")
	ErrComponentConflict       = errors.New("payroll: component already exists")
	ErrPayrollSettingsNotFound = errors.New("payroll: settings not found")
	ErrIncentivePolicyNotFound = errors.New("payroll: incentive policy not found")
	ErrRuleNotFound            = errors.New("payroll: rule not found")
	ErrTierNotFound            = errors.New("payroll: tier not found")
	ErrAssignmentNotFound      = errors.New("payroll: assignment not found")
	ErrAssignmentConflict      = errors.New("payroll: assignment conflict")
	ErrPayrollPeriodNotFound   = errors.New("payroll: period not found")
	ErrPayrollPeriodConflict   = errors.New("payroll: period conflict")
	ErrPayrollPeriodValidation = errors.New("payroll: period validation failed")
	ErrPayslipConflict         = errors.New("payroll: payslip conflict")
	ErrPayrollInputConflict    = errors.New("payroll: input batch conflict")
	ErrPayrollInputNotFound    = errors.New("payroll: input batch not found")
	ErrPayrollImpactNotFound   = errors.New("payroll: impact run not found")
	ErrPayrollImpactValidation = errors.New("payroll: impact runs are invalid")
)

// PayrollRepository composes the segregated payroll sub-domain repository interfaces.
type PayrollRepository interface {
	PayrollCatalogRepository
	PayrollFormulaRepository
	PayrollInputRepository
	PayrollRunRepository
}

type PayrollHandler struct {
	Repository PayrollRepository
	Authorizer Authorizer
	Audit      AuditEmitter
}

func (h *PayrollHandler) allowedCalculate(r *http.Request) bool {
	return hasPermission(r, h.Authorizer, "payroll:calculate")
}

func (h *PayrollHandler) allowedRead(r *http.Request) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	if h.Authorizer.HasPermission(r.Context(), u, "payroll:read") {
		return true
	}
	member, err := h.Authorizer.IsMember(r.Context(), u)
	return err == nil && member
}

func (h *PayrollHandler) allowedMember(r *http.Request) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	if member, err := h.Authorizer.IsMember(r.Context(), u); err == nil && member {
		return true
	}
	return h.Authorizer.HasPermission(r.Context(), u, "payroll:read")
}

func (h *PayrollHandler) allowedManage(r *http.Request) bool {
	return hasPermission(r, h.Authorizer, "payroll:components:manage") ||
		hasPermission(r, h.Authorizer, "payroll:manage")
}

func (h *PayrollHandler) emitPayrollAudit(r *http.Request, event audit.Event) {
	if h.Audit == nil {
		return
	}
	actorID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok {
		return
	}
	event.ActorID = actorID.String()
	_ = h.Audit.Emit(r.Context(), event)
}

func (h *PayrollHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	writeMappedError(w, r, err, func(err error) *apperror.AppError {
		switch {
		case errors.Is(err, ErrNotFound),
			errors.Is(err, ErrComponentNotFound),
			errors.Is(err, ErrEmploymentNotFound),
			errors.Is(err, ErrPayrollSettingsNotFound),
			errors.Is(err, ErrIncentivePolicyNotFound),
			errors.Is(err, ErrPayrollInputNotFound),
			errors.Is(err, ErrRuleNotFound),
			errors.Is(err, ErrTierNotFound),
			errors.Is(err, ErrAssignmentNotFound),
			errors.Is(err, ErrPayrollPeriodNotFound),
			errors.Is(err, ErrEmployerNotFound),
			errors.Is(err, ErrWorkPolicyGroupNotFound),
			errors.Is(err, ErrWorkPolicyResolutionNotFound),
			errors.Is(err, ErrPayrollImpactNotFound):
			return apperror.NotFound(apperror.KeyNotFound)
		case errors.Is(err, ErrEmployeeNotFound):
			return apperror.NotFound(apperror.KeyEmployeeNotFound)
		case errors.Is(err, ErrNoActiveEmployment):
			return apperror.Validation("error.no_active_employment")
		case errors.Is(err, ErrPayrollImpactValidation),
			errors.Is(err, ErrPayrollValidation),
			errors.Is(err, ErrPayrollPolicyMissing),
			errors.Is(err, ErrPayrollPeriodValidation):
			return apperror.Validation(apperror.KeyValidation)
		case errors.Is(err, ErrComponentConflict),
			errors.Is(err, ErrConflict),
			errors.Is(err, ErrAssignmentConflict),
			errors.Is(err, ErrPayrollPeriodConflict),
			errors.Is(err, ErrPayslipConflict),
			errors.Is(err, ErrPayrollInputConflict):
			return apperror.Conflict(apperror.KeyConflict)
		default:
			return nil
		}
	})
}
