package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockApprovalWorkflowRepository struct {
	workflows    []ApprovalWorkflowRecord
	history      []ApprovalActionLogRecord
	createErr    error
	processErr   error
	listErr      error
	historyErr   error
	getErr       error
	updateErr    error
	deleteErr    error
	createCalls  int
	processCalls int
	updateCalls  int
}

func (m *mockApprovalWorkflowRepository) ListWorkflows(context.Context, *string, int, int) (ApprovalWorkflowListResponse, error) {
	return ApprovalWorkflowListResponse{Data: m.workflows, Total: int64(len(m.workflows)), Page: 1, Limit: 50}, m.listErr
}
func (m *mockApprovalWorkflowRepository) CreateWorkflow(_ context.Context, creatorID uuid.UUID, req CreateWorkflowRequest) (ApprovalWorkflowRecord, error) {
	m.createCalls++
	return ApprovalWorkflowRecord{ID: uuid.New(), EntityType: req.EntityType, Name: req.Name, TotalSteps: len(req.Steps), IsActive: req.IsActive}, m.createErr
}
func (m *mockApprovalWorkflowRepository) GetWorkflow(_ context.Context, id uuid.UUID) (ApprovalWorkflowRecord, error) {
	if m.getErr != nil {
		return ApprovalWorkflowRecord{}, m.getErr
	}
	return ApprovalWorkflowRecord{ID: id, Name: "Workflow", Steps: []WorkflowStepRecord{}}, nil
}
func (m *mockApprovalWorkflowRepository) UpdateWorkflow(_ context.Context, id uuid.UUID, req UpdateWorkflowRequest) (ApprovalWorkflowRecord, error) {
	m.updateCalls++
	name := "Workflow"
	if req.Name != nil {
		name = *req.Name
	}
	return ApprovalWorkflowRecord{ID: id, Name: name, Steps: []WorkflowStepRecord{}}, m.updateErr
}
func (m *mockApprovalWorkflowRepository) DeleteWorkflow(context.Context, uuid.UUID) error {
	return m.deleteErr
}
func (m *mockApprovalWorkflowRepository) ProcessApproval(context.Context, uuid.UUID, ProcessApprovalRequest) error {
	m.processCalls++
	return m.processErr
}
func (m *mockApprovalWorkflowRepository) ListHistory(context.Context, string, uuid.UUID) ([]ApprovalActionLogRecord, error) {
	return m.history, m.historyErr
}
func (m *mockApprovalWorkflowRepository) GetPendingTasksForUser(context.Context, uuid.UUID, int, int) (PaginatedTasksResponse, error) {
	return PaginatedTasksResponse{Data: []ApprovalWorkflowTaskRecord{}, Total: 0, Page: 1, Limit: 50}, nil
}

func approvalRequest(t *testing.T, method, path, body string) *http.Request {
	t.Helper()
	req := httptest.NewRequest(method, path, strings.NewReader(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	return req
}

func TestApprovalWorkflowListForbiddenWithoutPermission(t *testing.T) {
	handler := ApprovalWorkflowHandler{Repository: &mockApprovalWorkflowRepository{}, Authorizer: mockAuthorizer{}}
	req := approvalRequest(t, http.MethodGet, "/api/v1" + "/approval-workflows", "")
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestApprovalWorkflowCreateSuccess(t *testing.T) {
	repo := &mockApprovalWorkflowRepository{}
	handler := ApprovalWorkflowHandler{Repository: repo, Authorizer: mockAuthorizer{perms: map[string]bool{"approval_workflows:manage": true}}}
	body := `{"entity_type":"leave_request","name":"Standard Leave Approval","is_active":true,"steps":[{"step_order":1,"step_name":"Manager Approval","approver_type":"manager","action_required":"approve","can_skip":false}]}`
	req := approvalRequest(t, http.MethodPost, "/api/v1" + "/approval-workflows", body)
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.createCalls != 1 {
		t.Fatalf("expected 1 create call, got %d", repo.createCalls)
	}
}

func TestApprovalWorkflowProcessValidationRejectsInvalidAction(t *testing.T) {
	entityID := uuid.New()
	repo := &mockApprovalWorkflowRepository{}
	handler := ApprovalWorkflowHandler{Repository: repo, Authorizer: mockAuthorizer{}}
	body := `{"entity_type":"leave_request","entity_id":"` + entityID.String() + `","action":"invalid_action"}`
	req := approvalRequest(t, http.MethodPost, "/api/v1" + "/approval-workflows/approve", body)
	rec := httptest.NewRecorder()
	handler.Process(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid action, got %d", rec.Code)
	}
	if repo.processCalls != 0 {
		t.Fatal("process approval called on validation failure")
	}
}

func TestApprovalWorkflowHistorySuccess(t *testing.T) {
	entityID := uuid.New()
	repo := &mockApprovalWorkflowRepository{history: []ApprovalActionLogRecord{{ID: uuid.New(), Action: "approved"}}}
	handler := ApprovalWorkflowHandler{Repository: repo, Authorizer: mockAuthorizer{}}
	req := approvalRequest(t, http.MethodGet, "/api/v1" + "/approval-workflows/history/leave_request/"+entityID.String(), "")
	req.SetPathValue("type", "leave_request")
	req.SetPathValue("id", entityID.String())
	rec := httptest.NewRecorder()
	handler.History(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if !strings.Contains(rec.Body.String(), "approved") {
		t.Fatalf("history response missing action: %s", rec.Body.String())
	}
}

func TestApprovalWorkflowGetSuccess(t *testing.T) {
	workflowID := uuid.New()
	handler := ApprovalWorkflowHandler{Repository: &mockApprovalWorkflowRepository{}, Authorizer: mockAuthorizer{perms: map[string]bool{"approval_workflows:read": true}}}
	req := approvalRequest(t, http.MethodGet, "/api/v1" + "/approval-workflows/"+workflowID.String(), "")
	req.SetPathValue("id", workflowID.String())
	rec := httptest.NewRecorder()
	handler.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestApprovalWorkflowUpdateSuccess(t *testing.T) {
	workflowID := uuid.New()
	repo := &mockApprovalWorkflowRepository{}
	handler := ApprovalWorkflowHandler{Repository: repo, Authorizer: mockAuthorizer{perms: map[string]bool{"approval_workflows:manage": true}}}
	req := approvalRequest(t, http.MethodPut, "/api/v1" + "/approval-workflows/"+workflowID.String(), `{"name":"Updated"}`)
	req.SetPathValue("id", workflowID.String())
	rec := httptest.NewRecorder()
	handler.Update(rec, req)
	if rec.Code != http.StatusOK || repo.updateCalls != 1 {
		t.Fatalf("expected update 200 and one call, got %d calls=%d", rec.Code, repo.updateCalls)
	}
}

func TestApprovalWorkflowDeleteSuccess(t *testing.T) {
	workflowID := uuid.New()
	handler := ApprovalWorkflowHandler{Repository: &mockApprovalWorkflowRepository{}, Authorizer: mockAuthorizer{perms: map[string]bool{"approval_workflows:manage": true}}}
	req := approvalRequest(t, http.MethodDelete, "/api/v1" + "/approval-workflows/"+workflowID.String(), "")
	req.SetPathValue("id", workflowID.String())
	rec := httptest.NewRecorder()
	handler.Delete(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestApprovalWorkflowPendingTasksSuccess(t *testing.T) {
	handler := ApprovalWorkflowHandler{
		Repository: &mockApprovalWorkflowRepository{},
		Authorizer: mockAuthorizer{perms: map[string]bool{"approval_workflows:read": true}},
	}
	req := approvalRequest(t, http.MethodGet, "/api/v1" + "/approval-workflows/pending-tasks", "")
	rec := httptest.NewRecorder()
	handler.PendingTasks(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}
