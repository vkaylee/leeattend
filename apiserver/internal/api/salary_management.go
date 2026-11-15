package api

import (
	"context"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

// SalaryManagementQuery defines bounded, server-side collection filters.
type SalaryManagementQuery struct {
	AsOf             *time.Time
	Search           string
	EmployerID       *ids.EmployerID
	DepartmentID     *ids.DepartmentID
	EmploymentStatus string
	Unconfigured     *bool
	Page             int
	Limit            int
}

// SalaryManagementRow represents one configured salary component for one employment.
// Amount is returned under the approved payroll-read T2 exception and never logged or cached.
type SalaryManagementRow struct {
	EmploymentID          uuid.UUID       `json:"employment_id"`
	EmployeeID            uuid.UUID       `json:"employee_id"`
	EmployeeIDNumber      string          `json:"employee_id_number"`
	EmployeeName          string          `json:"employee_name"`
	EmployerID            uuid.UUID       `json:"employer_id"`
	EmployerName          string          `json:"employer_name"`
	DepartmentID          *uuid.UUID      `json:"department_id"`
	DepartmentName        *string         `json:"department_name"`
	JobTitle              *string         `json:"job_title"`
	EmploymentStatus      string          `json:"employment_status"`
	EmploymentType        string          `json:"employment_type"`
	EmploymentStartDate   string          `json:"employment_start_date"`
	EmploymentEndDate     *string         `json:"employment_end_date"`
	ComponentID           uuid.UUID       `json:"component_id"`
	ComponentDefinitionID uuid.UUID       `json:"component_definition_id"`
	ComponentName         string          `json:"component_name"`
	ComponentCode         string          `json:"component_code"`
	Amount                decimal.Decimal `json:"amount"`
	EffectiveDate         string          `json:"effective_date"`
	SalaryStatus          string          `json:"salary_status"`
}

type SalaryManagementResponse struct {
	Data  []SalaryManagementRow `json:"data"`
	Total int64                 `json:"total"`
	Page  int                   `json:"page"`
	Limit int                   `json:"limit"`
	AsOf  string                `json:"as_of"`
}

type PayrollAudit interface {
	Emit(context.Context, audit.Event) error
}

func (h PayrollHandler) ListSalaryManagement(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		h.forbidden(w, r)
		return
	}

	query, err := parseSalaryManagementQuery(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	result, err := h.Repository.ListSalaryManagementRows(r.Context(), query)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if result.Data == nil {
		result.Data = []SalaryManagementRow{}
	}
	if h.Audit != nil {
		if userID, ok := httpinfra.UserIDFromContext(r.Context()); ok {
			_ = h.Audit.Emit(r.Context(), audit.Event{
				Name:     "salary_management.read",
				ActorID:  userID.String(),
				Resource: "salary-management",
				Outcome:  "success",
				Details: map[string]string{
					"row_count": strconv.Itoa(len(result.Data)),
					"page":      strconv.Itoa(result.Page),
					"limit":     strconv.Itoa(result.Limit),
				},
				At: time.Now().UTC(),
			})
		}
	}
	w.Header().Set("Cache-Control", "no-store")
	apperror.WriteJSON(w, http.StatusOK, result)
}

func parseSalaryManagementQuery(r *http.Request) (SalaryManagementQuery, error) {
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil || pagination.Offset < 0 {
		return SalaryManagementQuery{}, ErrPayrollValidation
	}

	query := SalaryManagementQuery{
		Search:           strings.TrimSpace(r.URL.Query().Get("search")),
		EmploymentStatus: strings.TrimSpace(r.URL.Query().Get("status")),
		Page:             pagination.Page,
		Limit:            pagination.Limit,
	}
	if query.EmploymentStatus == "" {
		query.EmploymentStatus = "active"
	}
	if len(query.Search) > 100 || len(query.EmploymentStatus) > 50 {
		return SalaryManagementQuery{}, ErrPayrollValidation
	}

	if raw := r.URL.Query().Get("as_of"); raw != "" {
		asOf, parseErr := time.Parse("2006-01-02", raw)
		if parseErr != nil {
			return SalaryManagementQuery{}, ErrPayrollValidation
		}
		query.AsOf = &asOf
	}
	if query.EmployerID, err = parseOptionalID[ids.EmployerID](r.URL.Query().Get("employer_id")); err != nil {
		return SalaryManagementQuery{}, ErrPayrollValidation
	}
	if query.DepartmentID, err = parseOptionalID[ids.DepartmentID](r.URL.Query().Get("department_id")); err != nil {
		return SalaryManagementQuery{}, ErrPayrollValidation
	}
	if raw := r.URL.Query().Get("unconfigured"); raw != "" {
		value, parseErr := strconv.ParseBool(raw)
		if parseErr != nil {
			return SalaryManagementQuery{}, ErrPayrollValidation
		}
		query.Unconfigured = &value
	}
	return query, nil
}

func parseOptionalID[T ~[16]byte](raw string) (*T, error) {
	if raw == "" {
		return nil, nil
	}
	id, err := uuid.Parse(raw)
	if err != nil || id == uuid.Nil {
		return nil, ErrPayrollValidation
	}
	converted := T(id)
	return &converted, nil
}
