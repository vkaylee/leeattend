package api

import (
	"context"
	"errors"
	"net/http"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type SalaryComponentDefinition struct {
	ID        uuid.UUID `json:"id"`
	Name      string    `json:"name"`
	Code      string    `json:"code"`
	CreatedAt string    `json:"created_at"`
	UpdatedAt string    `json:"updated_at"`
}

type EmployeeSalaryComponent struct {
	ID                    uuid.UUID       `json:"id"`
	EmploymentID          uuid.UUID       `json:"employment_id"`
	ComponentDefinitionID uuid.UUID       `json:"component_definition_id"`
	Amount                decimal.Decimal `json:"amount"`
	EffectiveDate         string          `json:"effective_date"`
	CreatedAt             string          `json:"created_at"`
	UpdatedAt             string          `json:"updated_at"`
}

type SalaryComponentWithDefinition struct {
	Component  EmployeeSalaryComponent   `json:"component"`
	Definition SalaryComponentDefinition `json:"definition"`
}

type CompensationEmploymentContext struct {
	ID             uuid.UUID `json:"id"`
	EmployerName   string    `json:"employer_name"`
	DepartmentName *string   `json:"department_name"`
	JobTitle       *string   `json:"job_title"`
}

type CompensationChangeDetails struct {
	PreviousTotal *decimal.Decimal `json:"previous_total"`
	Delta         decimal.Decimal  `json:"delta"`
}

type CompensationTimelineEvent struct {
	EffectiveDate     string                          `json:"effective_date"`
	Employment        CompensationEmploymentContext   `json:"employment"`
	Components        []SalaryComponentWithDefinition `json:"components"`
	TotalCompensation decimal.Decimal                 `json:"total_compensation"`
	ChangeType        string                          `json:"change_type"`
	ChangeDetails     CompensationChangeDetails       `json:"change_details"`
}

type CompensationHistoryResponse struct {
	Timeline []CompensationTimelineEvent `json:"timeline"`
}

type CompensationHistoryRepository interface {
	GetEmployeeCompensationHistory(context.Context, uuid.UUID, *uuid.UUID) (CompensationHistoryResponse, error)
}
type CompensationHistoryHandler struct {
	Repository CompensationHistoryRepository
	Authorizer Authorizer
}

func (h CompensationHistoryHandler) Get(w http.ResponseWriter, r *http.Request) {
	employeeID, ok := employeePath(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionEmployeesRead)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	var employmentID *uuid.UUID
	if raw := r.URL.Query().Get("employment_id"); raw != "" {
		id, err := uuid.Parse(raw)
		if err != nil || id == uuid.Nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		employmentID = &id
	}
	result, err := h.Repository.GetEmployeeCompensationHistory(r.Context(), employeeID, employmentID)
	if err != nil {
		switch {
		case errors.Is(err, pgx.ErrNoRows), errors.Is(err, ErrEmployeeNotFound):
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		default:
			apperror.WriteError(w, r, apperror.Internal(err))
		}
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}
