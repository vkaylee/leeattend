package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrWorkflowValidation = errors.New("approval workflow: validation failed")
	ErrWorkflowNotFound   = errors.New("approval workflow: not found")
	ErrWorkflowAction     = errors.New("approval workflow: invalid action")
)

type WorkflowStepRequest struct {
	StepOrder      int             `json:"step_order"`
	StepName       string          `json:"step_name"`
	ApproverType   string          `json:"approver_type"`
	ApproverConfig json.RawMessage `json:"approver_config"`
	ActionRequired string          `json:"action_required"`
	CanSkip        bool            `json:"can_skip"`
}

type CreateWorkflowRequest struct {
	EntityType  string                `json:"entity_type"`
	Name        string                `json:"name"`
	Description *string               `json:"description"`
	IsActive    bool                  `json:"is_active"`
	Steps       []WorkflowStepRequest `json:"steps"`
}

type UpdateWorkflowRequest struct {
	Name        *string `json:"name"`
	Description *string `json:"description"`
	IsActive    *bool   `json:"is_active"`
}

type ApprovalWorkflowRecord struct {
	ID          uuid.UUID            `json:"id"`
	WorkspaceID uuid.UUID            `json:"workspace_id"`
	EntityType  string               `json:"entity_type"`
	Name        string               `json:"name"`
	Description *string              `json:"description"`
	TotalSteps  int                  `json:"total_steps"`
	IsActive    bool                 `json:"is_active"`
	CreatedBy   *uuid.UUID           `json:"created_by"`
	CreatedAt   string               `json:"created_at"`
	UpdatedAt   string               `json:"updated_at"`
	Steps       []WorkflowStepRecord `json:"steps,omitempty"`
}

type ApprovalWorkflowListResponse struct {
	Data  []ApprovalWorkflowRecord `json:"data"`
	Total int64                    `json:"total"`
	Page  int                      `json:"page"`
	Limit int                      `json:"limit"`
}

type WorkflowStepRecord struct {
	ID             uuid.UUID       `json:"id"`
	WorkflowID     uuid.UUID       `json:"workflow_id"`
	StepOrder      int             `json:"step_order"`
	StepName       string          `json:"step_name"`
	ApproverType   string          `json:"approver_type"`
	ApproverConfig json.RawMessage `json:"approver_config"`
	ActionRequired string          `json:"action_required"`
	CanSkip        bool            `json:"can_skip"`
}

type ProcessApprovalRequest struct {
	EntityType        string     `json:"entity_type"`
	EntityID          uuid.UUID  `json:"entity_id"`
	Action            string     `json:"action"`
	Note              *string    `json:"note"`
	NewApproverUserID *uuid.UUID `json:"new_approver_user_id"`
}

type ApprovalActionLogRecord struct {
	ID            uuid.UUID `json:"id"`
	WorkspaceID   uuid.UUID `json:"workspace_id"`
	EntityType    string    `json:"entity_type"`
	EntityID      uuid.UUID `json:"entity_id"`
	StepOrder     int       `json:"step_order"`
	StepName      *string   `json:"step_name"`
	ActedByUserID uuid.UUID `json:"acted_by_user_id"`
	Action        string    `json:"action"`
	Note          *string   `json:"note"`
	ActedAt       string    `json:"acted_at"`
}

type ApprovalWorkflowRepository interface {
	ListWorkflows(context.Context, uuid.UUID, *string, int, int) (ApprovalWorkflowListResponse, error)
	CreateWorkflow(context.Context, uuid.UUID, uuid.UUID, CreateWorkflowRequest) (ApprovalWorkflowRecord, error)
	ProcessApproval(context.Context, uuid.UUID, uuid.UUID, ProcessApprovalRequest) error
	ListHistory(context.Context, uuid.UUID, string, uuid.UUID) ([]ApprovalActionLogRecord, error)
}

type ApprovalWorkflowHandler struct {
	Repository ApprovalWorkflowRepository
	Authorizer WorkspaceAuthorizer
}

func (h ApprovalWorkflowHandler) List(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, wsID, "approval_workflows:read") {
		h.forbidden(w, r)
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	var entityType *string
	if value := strings.TrimSpace(r.URL.Query().Get("entity_type")); value != "" {
		entityType = &value
	}
	items, err := h.Repository.ListWorkflows(r.Context(), wsID, entityType, pagination.Page, pagination.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func (h ApprovalWorkflowHandler) Create(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || !h.allowed(r, wsID, "approval_workflows:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateWorkflowRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.EntityType) == "" || strings.TrimSpace(req.Name) == "" || len(req.Steps) == 0 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	workflow, err := h.Repository.CreateWorkflow(r.Context(), wsID, userID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, workflow)
}

func (h ApprovalWorkflowHandler) Process(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[ProcessApprovalRequest](r, 1<<20)
	if err != nil || req.EntityID == uuid.Nil || strings.TrimSpace(req.EntityType) == "" || !validApprovalAction(req.Action) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.EntityType == "ot_request" && !h.Authorizer.HasPermission(r.Context(), userID, wsID, "overtime:approve_workflow") {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.ProcessApproval(r.Context(), wsID, userID, req); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h ApprovalWorkflowHandler) History(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	entityID, entityOK := pathUUID(r, "id")
	if !ok || !entityOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	entityType := r.PathValue("type")
	items, err := h.Repository.ListHistory(r.Context(), wsID, entityType, entityID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func validApprovalAction(action string) bool {
	switch action {
	case "approve", "approved", "reject", "rejected", "return", "returned", "reassign", "reassigned", "skip", "skipped":
		return true
	default:
		return false
	}
}
func (h ApprovalWorkflowHandler) allowed(r *http.Request, wsID uuid.UUID, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, wsID, permission)
}
func (h ApprovalWorkflowHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}
func (h ApprovalWorkflowHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrWorkflowNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrWorkflowValidation) || errors.Is(err, ErrWorkflowAction) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
