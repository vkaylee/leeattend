package api

import (
	"context"
	"net/http"
	"strconv"
	"strings"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type CreatePayrollInputRequest struct {
	PeriodYear     int            `json:"period_year"`
	PeriodMonth    int            `json:"period_month"`
	Source         string         `json:"source"`
	IdempotencyKey string         `json:"idempotency_key"`
	Inputs         []PayrollInput `json:"inputs"`
}

type PayrollInput struct {
	ID                uuid.UUID       `json:"id,omitempty"`
	EmploymentID      uuid.UUID       `json:"employment_id"`
	MetricCode        string          `json:"metric_code"`
	Value             decimal.Decimal `json:"value"`
	Unit              string          `json:"unit"`
	Status            string          `json:"status,omitempty"`
	ValidationMessage *string         `json:"validation_message,omitempty"`
	SourceReference   *string         `json:"source_reference,omitempty"`
}

type CorrectPayrollInputRequest struct {
	Value  decimal.Decimal `json:"value"`
	Unit   string          `json:"unit"`
	Reason string          `json:"reason"`
}

type PayrollInputBatch struct {
	ID             uuid.UUID  `json:"id"`
	PeriodYear     int        `json:"period_year"`
	PeriodMonth    int        `json:"period_month"`
	Source         string     `json:"source"`
	IdempotencyKey string     `json:"idempotency_key"`
	Status         string     `json:"status"`
	InputCount     int        `json:"input_count"`
	SubmittedBy    uuid.UUID  `json:"submitted_by"`
	ApprovedBy     *uuid.UUID `json:"approved_by,omitempty"`
	LockedBy       *uuid.UUID `json:"locked_by,omitempty"`
	CreatedAt      string     `json:"created_at"`
}

type UpdatePayrollInputBatchRequest struct {
	Version int64 `json:"version"`
}

// PayrollInputRepository manages payroll input batches and corrections.
type PayrollInputRepository interface {
	CreatePayrollInputBatch(context.Context, CreatePayrollInputRequest, ids.UserID) (PayrollInputBatch, error)
	ApprovePayrollInputBatch(context.Context, ids.PayrollInputBatchID, ids.UserID) (PayrollInputBatch, error)
	LockPayrollInputBatch(context.Context, ids.PayrollInputBatchID, ids.UserID) (PayrollInputBatch, error)
	ListPayrollInputs(context.Context, ids.PayrollInputBatchID) ([]PayrollInput, error)
	CorrectPayrollInput(context.Context, ids.PayrollInputID, CorrectPayrollInputRequest, ids.UserID) (PayrollInput, error)
}

func (h *PayrollHandler) CreatePayrollInputBatch(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:inputs:manage") && !hasPermission(r, h.Authorizer, "payroll:manage") {
		writeForbidden(w, r)
		return
	}
	userID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollInputRequest](r, 4<<20)
	if err != nil || !validPayrollInputRequest(req) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	batch, err := h.Repository.CreatePayrollInputBatch(r.Context(), req, userID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.input.submitted",
		Resource: batch.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"batch_id":     batch.ID.String(),
			"period_year":  strconv.Itoa(batch.PeriodYear),
			"period_month": strconv.Itoa(batch.PeriodMonth),
			"input_count":  strconv.Itoa(batch.InputCount),
		},
	})
	apperror.WriteJSON(w, http.StatusCreated, batch)
}

func validPayrollInputRequest(req CreatePayrollInputRequest) bool {
	if req.PeriodYear < 2000 || req.PeriodYear > 2100 || req.PeriodMonth < 1 || req.PeriodMonth > 12 {
		return false
	}
	if strings.TrimSpace(req.Source) == "" || len(req.Source) > 100 || strings.TrimSpace(req.IdempotencyKey) == "" || len(req.IdempotencyKey) > 200 || len(req.Inputs) == 0 || len(req.Inputs) > 10000 {
		return false
	}
	seen := make(map[string]struct{}, len(req.Inputs))
	for _, input := range req.Inputs {
		if input.EmploymentID == uuid.Nil || strings.TrimSpace(input.MetricCode) == "" || len(input.MetricCode) > 100 || strings.TrimSpace(input.Unit) == "" || len(input.Unit) > 40 || input.Value.IsNegative() {
			return false
		}
		key := input.EmploymentID.String() + "|" + input.MetricCode
		if _, exists := seen[key]; exists {
			return false
		}
		seen[key] = struct{}{}
	}
	return true
}

func (h *PayrollHandler) ListPayrollInputs(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		writeForbidden(w, r)
		return
	}
	batchID, ok := pathUUID(r, "batch_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	inputs, err := h.Repository.ListPayrollInputs(r.Context(), ids.PayrollInputBatchID(batchID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if inputs == nil {
		inputs = []PayrollInput{}
	}
	apperror.WriteJSON(w, http.StatusOK, inputs)
}

func (h *PayrollHandler) CorrectPayrollInput(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:manage") {
		writeForbidden(w, r)
		return
	}
	inputID, ok := pathUUID(r, "input_id")
	actorID, actorOK := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[CorrectPayrollInputRequest](r, 1<<20)
	if err != nil || req.Value.IsNegative() || strings.TrimSpace(req.Unit) == "" || len(strings.TrimSpace(req.Reason)) == 0 || len(req.Reason) > 500 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	input, err := h.Repository.CorrectPayrollInput(r.Context(), ids.PayrollInputID(inputID), req, actorID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.input.corrected",
		Resource: input.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"input_id":      input.ID.String(),
			"employment_id": input.EmploymentID.String(),
			"metric_code":   input.MetricCode,
			"status":        input.Status,
		},
	})
	apperror.WriteJSON(w, http.StatusOK, input)
}

func (h *PayrollHandler) ApprovePayrollInputBatch(w http.ResponseWriter, r *http.Request) {
	h.transitionPayrollInputBatch(w, r, "approve")
}

func (h *PayrollHandler) LockPayrollInputBatch(w http.ResponseWriter, r *http.Request) {
	h.transitionPayrollInputBatch(w, r, "lock")
}

func (h *PayrollHandler) transitionPayrollInputBatch(w http.ResponseWriter, r *http.Request, action string) {
	permission := "payroll:manage"
	if action == "approve" {
		permission = "payroll:approve"
	}
	if !hasPermission(r, h.Authorizer, permission) {
		writeForbidden(w, r)
		return
	}
	batchID, ok := pathUUID(r, "batch_id")
	actorID, actorOK := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	var batch PayrollInputBatch
	var err error
	if action == "approve" {
		batch, err = h.Repository.ApprovePayrollInputBatch(r.Context(), ids.PayrollInputBatchID(batchID), actorID)
	} else {
		batch, err = h.Repository.LockPayrollInputBatch(r.Context(), ids.PayrollInputBatchID(batchID), actorID)
	}
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	eventName := "payroll.input.approved"
	if action == "lock" {
		eventName = "payroll.input.locked"
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     eventName,
		Resource: batch.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"batch_id": batch.ID.String(),
			"status":   batch.Status,
		},
	})
	apperror.WriteJSON(w, http.StatusOK, batch)
}
