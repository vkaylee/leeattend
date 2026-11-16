package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrTemplateNotFound      = errors.New("lifecycle: template not found")
	ErrTemplateItemNotFound  = errors.New("lifecycle: template item not found")
	ErrChecklistNotFound     = errors.New("lifecycle: checklist not found")
	ErrChecklistItemNotFound = errors.New("lifecycle: checklist item not found")
)

type LifecycleChecklistTemplate struct {
	ID            uuid.UUID `json:"id"`
	Name          string    `json:"name"`
	Description   *string   `json:"description"`
	ChecklistType string    `json:"checklist_type"`
	IsDefault     bool      `json:"is_default"`
	IsActive      bool      `json:"is_active"`
	CreatedAt     string    `json:"created_at"`
	UpdatedAt     string    `json:"updated_at"`
}

type LifecycleChecklistTemplateItem struct {
	ID             uuid.UUID  `json:"id"`
	TemplateID     uuid.UUID  `json:"template_id"`
	Title          string     `json:"title"`
	Description    *string    `json:"description"`
	Category       string     `json:"category"`
	AssigneeType   string     `json:"assignee_type"`
	AssigneeUserID *uuid.UUID `json:"assignee_user_id"`
	IsRequired     bool       `json:"is_required"`
	DueDaysOffset  int        `json:"due_days_offset"`
	DisplayOrder   int        `json:"display_order"`
}

type LifecycleChecklistItem struct {
	ID             uuid.UUID  `json:"id"`
	ChecklistID    uuid.UUID  `json:"checklist_id"`
	TemplateItemID *uuid.UUID `json:"template_item_id"`
	Title          string     `json:"title"`
	Description    *string    `json:"description"`
	Category       string     `json:"category"`
	AssigneeUserID *uuid.UUID `json:"assignee_user_id"`
	IsRequired     bool       `json:"is_required"`
	DueDate        *string    `json:"due_date"`
	Status         string     `json:"status"`
	CompletedAt    *string    `json:"completed_at"`
	CompletedBy    *uuid.UUID `json:"completed_by"`
	Notes          *string    `json:"notes"`
	DisplayOrder   int        `json:"display_order"`
	CreatedAt      string     `json:"created_at"`
	UpdatedAt      string     `json:"updated_at"`
}

type LifecycleChecklistResponse struct {
	ID              uuid.UUID                `json:"id"`
	EmploymentID    uuid.UUID                `json:"employment_id"`
	TemplateID      *uuid.UUID               `json:"template_id"`
	ChecklistType   string                   `json:"checklist_type"`
	Status          string                   `json:"status"`
	ProgressPercent int                      `json:"progress_percent"`
	StartedAt       string                   `json:"started_at"`
	CompletedAt     *string                  `json:"completed_at"`
	CreatedBy       *uuid.UUID               `json:"created_by"`
	CreatedAt       string                   `json:"created_at"`
	UpdatedAt       string                   `json:"updated_at"`
	Items           []LifecycleChecklistItem `json:"items"`
}
type LifecycleInstanceSummary struct {
	ID              uuid.UUID  `json:"id"`
	EmploymentID    uuid.UUID  `json:"employment_id"`
	EmployeeID      uuid.UUID  `json:"employee_id"`
	EmployeeName    string     `json:"employee_name"`
	EmployeeCode    string     `json:"employee_code"`
	DepartmentName  *string    `json:"department_name"`
	TemplateID      *uuid.UUID `json:"template_id"`
	TemplateName    *string    `json:"template_name"`
	ChecklistType   string     `json:"checklist_type"`
	Status          string     `json:"status"`
	ProgressPercent int        `json:"progress_percent"`
	TotalItems      int        `json:"total_items"`
	CompletedItems  int        `json:"completed_items"`
	StartedAt       string     `json:"started_at"`
	CompletedAt     *string    `json:"completed_at"`
	CreatedAt       string     `json:"created_at"`
	UpdatedAt       string     `json:"updated_at"`
}

type PendingTaskResponse struct {
	ID            uuid.UUID `json:"id"`
	ChecklistID   uuid.UUID `json:"checklist_id"`
	EmployeeName  string    `json:"employee_name"`
	ChecklistType string    `json:"checklist_type"`
	Title         string    `json:"title"`
	Description   *string   `json:"description"`
	Category      string    `json:"category"`
	DueDate       *string   `json:"due_date"`
	IsRequired    bool      `json:"is_required"`
}

type CreateLifecycleChecklistTemplateRequest struct {
	Name          string  `json:"name"`
	Description   *string `json:"description"`
	ChecklistType string  `json:"checklist_type"`
	IsDefault     bool    `json:"is_default"`
}

type UpdateLifecycleChecklistTemplateRequest struct {
	Name        *string `json:"name"`
	Description *string `json:"description"`
	IsDefault   *bool   `json:"is_default"`
	IsActive    *bool   `json:"is_active"`
}

type CreateLifecycleChecklistTemplateItemRequest struct {
	Title          string     `json:"title"`
	Description    *string    `json:"description"`
	Category       string     `json:"category"`
	AssigneeType   string     `json:"assignee_type"`
	AssigneeUserID *uuid.UUID `json:"assignee_user_id"`
	IsRequired     bool       `json:"is_required"`
	DueDaysOffset  int        `json:"due_days_offset"`
	DisplayOrder   int        `json:"display_order"`
}

type UpdateLifecycleChecklistTemplateItemRequest struct {
	Title             *string     `json:"title"`
	Description       *string     `json:"description"`
	Category          *string     `json:"category"`
	AssigneeType      *string     `json:"assignee_type"`
	AssigneeUserID    **uuid.UUID `json:"assignee_user_id"`
	IsRequired        *bool       `json:"is_required"`
	DueDaysOffset     *int        `json:"due_days_offset"`
	DisplayOrder      *int        `json:"display_order"`
	AssigneeIDPresent bool        `json:"-"`
}

func (r *UpdateLifecycleChecklistTemplateItemRequest) UnmarshalJSON(data []byte) error {
	type plain UpdateLifecycleChecklistTemplateItemRequest
	var value plain
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(data, &fields); err != nil {
		return err
	}
	*r = UpdateLifecycleChecklistTemplateItemRequest(value)
	_, r.AssigneeIDPresent = fields["assignee_user_id"]
	return nil
}

type CreateChecklistRequest struct {
	TemplateID uuid.UUID `json:"template_id"`
}

type UpdateChecklistItemStatusRequest struct {
	Status string  `json:"status"`
	Notes  *string `json:"notes"`
}

func isValidChecklistType(t string) bool {
	return t == "onboarding" || t == "offboarding" || t == "transfer" || t == "probation_end"
}

func isValidCategory(c string) bool {
	return c == "hr_admin" || c == "it_setup" || c == "documentation" || c == "training" || c == "equipment"
}

func isValidAssigneeType(a string) bool {
	return a == "hr_manager" || a == "direct_manager" || a == "it_admin" || a == "employee" || a == "specific_user"
}

func isValidItemStatus(s string) bool {
	return s == "pending" || s == "completed" || s == "skipped" || s == "overdue"
}

type LifecycleRepository interface {
	CreateTemplate(ctx context.Context, req CreateLifecycleChecklistTemplateRequest) (LifecycleChecklistTemplate, error)
	GetTemplate(ctx context.Context, id uuid.UUID) (LifecycleChecklistTemplate, error)
	ListTemplates(ctx context.Context, checklistType *string) ([]LifecycleChecklistTemplate, error)
	UpdateTemplate(ctx context.Context, id uuid.UUID, req UpdateLifecycleChecklistTemplateRequest) (LifecycleChecklistTemplate, error)
	DeleteTemplate(ctx context.Context, id uuid.UUID) error
	CreateTemplateItem(ctx context.Context, templateID uuid.UUID, req CreateLifecycleChecklistTemplateItemRequest) (LifecycleChecklistTemplateItem, error)
	GetTemplateItem(ctx context.Context, itemID uuid.UUID) (LifecycleChecklistTemplateItem, error)
	ListTemplateItems(ctx context.Context, templateID uuid.UUID) ([]LifecycleChecklistTemplateItem, error)
	UpdateTemplateItem(ctx context.Context, itemID uuid.UUID, req UpdateLifecycleChecklistTemplateItemRequest) (LifecycleChecklistTemplateItem, error)
	DeleteTemplateItem(ctx context.Context, itemID uuid.UUID) error
	CreateChecklistFromTemplate(ctx context.Context, employmentID, templateID uuid.UUID, actorID *uuid.UUID, usersResolver func(context.Context, string) []uuid.UUID) (LifecycleChecklistResponse, error)
	GetChecklist(ctx context.Context, checklistID uuid.UUID) (LifecycleChecklistResponse, error)
	ListChecklistsForEmployment(ctx context.Context, employmentID uuid.UUID) ([]LifecycleChecklistResponse, error)
	ListChecklists(ctx context.Context) ([]LifecycleInstanceSummary, error)
	UpdateChecklistItemStatus(ctx context.Context, itemID uuid.UUID, status string, notes *string, completedBy *uuid.UUID) (LifecycleChecklistResponse, error)
	GetItemAssigneeAndChecklist(ctx context.Context, itemID uuid.UUID) (assigneeID *uuid.UUID, checklistID uuid.UUID, err error)
	GetPendingTasksForUser(ctx context.Context, userID uuid.UUID) ([]PendingTaskResponse, error)
}

type LifecycleHandler struct {
	Repository LifecycleRepository
	Authorizer Authorizer
}

func (h *LifecycleHandler) allowed(r *http.Request, permission string) bool {
	uid, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), uid, permission)
}

func (h *LifecycleHandler) deny(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h *LifecycleHandler) writeErr(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrTemplateNotFound):
		apperror.WriteError(w, r, apperror.NotFound("error.template_not_found"))
	case errors.Is(err, ErrTemplateItemNotFound):
		apperror.WriteError(w, r, apperror.NotFound("error.template_item_not_found"))
	case errors.Is(err, ErrChecklistNotFound):
		apperror.WriteError(w, r, apperror.NotFound("error.checklist_not_found"))
	case errors.Is(err, ErrChecklistItemNotFound):
		apperror.WriteError(w, r, apperror.NotFound("error.checklist_item_not_found"))
	case errors.Is(err, ErrEmploymentNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyEmploymentNotFound))
	case errors.Is(err, ErrEmployeeNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyEmployeeNotFound))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}

func (h *LifecycleHandler) CreateTemplate(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "organization:settings:write") {
		h.deny(w, r)
		return
	}
	var req CreateLifecycleChecklistTemplateRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || strings.TrimSpace(req.Name) == "" || !isValidChecklistType(req.ChecklistType) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("repository unavailable")))
		return
	}
	created, err := h.Repository.CreateTemplate(r.Context(), req)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, created)
}

func (h *LifecycleHandler) GetTemplate(w http.ResponseWriter, r *http.Request) {
	id, ok := pathUUID(r, "id")
	if !ok || !h.allowed(r, "organization:settings:read") {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	tmpl, err := h.Repository.GetTemplate(r.Context(), id)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, tmpl)
}

func (h *LifecycleHandler) ListTemplates(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "organization:settings:read") {
		h.deny(w, r)
		return
	}
	var ct *string
	if q := r.URL.Query().Get("checklist_type"); q != "" {
		ct = &q
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []LifecycleChecklistTemplate{})
		return
	}
	list, err := h.Repository.ListTemplates(r.Context(), ct)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, list)
}

func (h *LifecycleHandler) UpdateTemplate(w http.ResponseWriter, r *http.Request) {
	id, ok := pathUUID(r, "id")
	if !ok || !h.allowed(r, "organization:settings:write") {
		h.deny(w, r)
		return
	}
	var req UpdateLifecycleChecklistTemplateRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	updated, err := h.Repository.UpdateTemplate(r.Context(), id, req)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, updated)
}

func (h *LifecycleHandler) DeleteTemplate(w http.ResponseWriter, r *http.Request) {
	id, ok := pathUUID(r, "id")
	if !ok || !h.allowed(r, "organization:settings:write") {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if err := h.Repository.DeleteTemplate(r.Context(), id); err != nil {
		h.writeErr(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *LifecycleHandler) CreateTemplateItem(w http.ResponseWriter, r *http.Request) {
	tmplID, ok := pathUUID(r, "id")
	if !ok || !h.allowed(r, "organization:settings:write") {
		h.deny(w, r)
		return
	}
	var req CreateLifecycleChecklistTemplateItemRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || strings.TrimSpace(req.Title) == "" || !isValidCategory(req.Category) || !isValidAssigneeType(req.AssigneeType) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("repository unavailable")))
		return
	}
	item, err := h.Repository.CreateTemplateItem(r.Context(), tmplID, req)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, item)
}

func (h *LifecycleHandler) ListTemplateItems(w http.ResponseWriter, r *http.Request) {
	tmplID, ok := pathUUID(r, "id")
	if !ok || !h.allowed(r, "organization:settings:read") {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []LifecycleChecklistTemplateItem{})
		return
	}
	items, err := h.Repository.ListTemplateItems(r.Context(), tmplID)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func (h *LifecycleHandler) GetTemplateItem(w http.ResponseWriter, r *http.Request) {
	iid, ok := pathUUID(r, "iid")
	if !ok || !h.allowed(r, "organization:settings:read") {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	item, err := h.Repository.GetTemplateItem(r.Context(), iid)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, item)
}

func (h *LifecycleHandler) UpdateTemplateItem(w http.ResponseWriter, r *http.Request) {
	iid, ok := pathUUID(r, "iid")
	if !ok || !h.allowed(r, "organization:settings:write") {
		h.deny(w, r)
		return
	}
	var req UpdateLifecycleChecklistTemplateItemRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || (req.Category != nil && !isValidCategory(*req.Category)) || (req.AssigneeType != nil && !isValidAssigneeType(*req.AssigneeType)) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	updated, err := h.Repository.UpdateTemplateItem(r.Context(), iid, req)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, updated)
}

func (h *LifecycleHandler) DeleteTemplateItem(w http.ResponseWriter, r *http.Request) {
	iid, ok := pathUUID(r, "iid")
	if !ok || !h.allowed(r, "organization:settings:write") {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if err := h.Repository.DeleteTemplateItem(r.Context(), iid); err != nil {
		h.writeErr(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *LifecycleHandler) CreateChecklist(w http.ResponseWriter, r *http.Request) {
	employmentID, ok := pathUUID(r, "emp_id")
	if !ok || !h.allowed(r, "employees:update") {
		h.deny(w, r)
		return
	}
	var req CreateChecklistRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || req.TemplateID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	uid, _ := httpinfra.UserIDFromContext(r.Context())
	var actorID *uuid.UUID
	if uid != uuid.Nil {
		actorID = &uid
	}
	resolver := func(ctx context.Context, permission string) []uuid.UUID {
		if h.Authorizer == nil {
			return nil
		}
		users, _ := h.Authorizer.GetUsersWithPermission(ctx, permission)
		return users
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("repository unavailable")))
		return
	}
	checklist, err := h.Repository.CreateChecklistFromTemplate(r.Context(), employmentID, req.TemplateID, actorID, resolver)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, checklist)
}

func (h *LifecycleHandler) ListEmploymentChecklists(w http.ResponseWriter, r *http.Request) {
	employmentID, ok := pathUUID(r, "emp_id")
	if !ok || !h.allowed(r, "employees:read") {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []LifecycleChecklistResponse{})
		return
	}
	list, err := h.Repository.ListChecklistsForEmployment(r.Context(), employmentID)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, list)
}

func (h *LifecycleHandler) ListChecklists(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "employees:read") {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	list, err := h.Repository.ListChecklists(r.Context())
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, list)
}

func (h *LifecycleHandler) GetChecklist(w http.ResponseWriter, r *http.Request) {
	id, ok := pathUUID(r, "id")
	if !ok || !h.allowed(r, "employees:read") {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	cl, err := h.Repository.GetChecklist(r.Context(), id)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, cl)
}

func (h *LifecycleHandler) UpdateItemStatus(w http.ResponseWriter, r *http.Request) {
	iid, ok := pathUUID(r, "iid")
	if !ok {
		h.deny(w, r)
		return
	}
	uid, hasUID := httpinfra.UserIDFromContext(r.Context())
	if !hasUID {
		h.deny(w, r)
		return
	}
	var req UpdateChecklistItemStatusRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || !isValidItemStatus(req.Status) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	assigneeID, _, err := h.Repository.GetItemAssigneeAndChecklist(r.Context(), iid)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	isAssignee := assigneeID != nil && *assigneeID == uid
	if !isAssignee && !h.allowed(r, "employees:update") {
		h.deny(w, r)
		return
	}
	res, err := h.Repository.UpdateChecklistItemStatus(r.Context(), iid, req.Status, req.Notes, &uid)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, res)
}

func (h *LifecycleHandler) GetMyTasks(w http.ResponseWriter, r *http.Request) {
	uid, hasUID := httpinfra.UserIDFromContext(r.Context())
	if !hasUID {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []PendingTaskResponse{})
		return
	}
	tasks, err := h.Repository.GetPendingTasksForUser(r.Context(), uid)
	if err != nil {
		h.writeErr(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, tasks)
}
