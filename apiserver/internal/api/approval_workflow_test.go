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

func (m *mockApprovalWorkflowRepository) ListWorkflows(context.Context, uuid.UUID, *string, int, int) (ApprovalWorkflowListResponse, error) {
	return ApprovalWorkflowListResponse{Data: m.workflows, Total: int64(len(m.workflows)), Page: 1, Limit: 50}, m.listErr
}
func (m *mockApprovalWorkflowRepository) CreateWorkflow(_ context.Context, wsID, creatorID uuid.UUID, req CreateWorkflowRequest) (ApprovalWorkflowRecord, error) {
	m.createCalls++
	return ApprovalWorkflowRecord{ID: uuid.New(), WorkspaceID: wsID, EntityType: req.EntityType, Name: req.Name, TotalSteps: len(req.Steps), IsActive: req.IsActive}, m.createErr
}
func (m *mockApprovalWorkflowRepository) GetWorkflow(_ context.Context, wsID, id uuid.UUID) (ApprovalWorkflowRecord, error) {
	if m.getErr != nil {
		return ApprovalWorkflowRecord{}, m.getErr
	}
	return ApprovalWorkflowRecord{ID: id, WorkspaceID: wsID, Name: "Workflow", Steps: []WorkflowStepRecord{}}, nil
}
func (m *mockApprovalWorkflowRepository) UpdateWorkflow(_ context.Context, wsID, id uuid.UUID, req UpdateWorkflowRequest) (ApprovalWorkflowRecord, error) {
	m.updateCalls++
	name := "Workflow"
	if req.Name != nil {
		name = *req.Name
	}
	return ApprovalWorkflowRecord{ID: id, WorkspaceID: wsID, Name: name, Steps: []WorkflowStepRecord{}}, m.updateErr
}
func (m *mockApprovalWorkflowRepository) DeleteWorkflow(context.Context, uuid.UUID, uuid.UUID) error {
	return m.deleteErr
}
func (m *mockApprovalWorkflowRepository) ProcessApproval(context.Context, uuid.UUID, uuid.UUID, ProcessApprovalRequest) error {
	m.processCalls++
	return m.processErr
}
func (m *mockApprovalWorkflowRepository) ListHistory(context.Context, uuid.UUID, string, uuid.UUID) ([]ApprovalActionLogRecord, error) {
	return m.history, m.historyErr
}

func approvalRequest(t *testing.T, method, path, body string, wsID uuid.UUID) *http.Request {
	t.Helper()
	req := httptest.NewRequest(method, path, strings.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	return req
}

func TestApprovalWorkflowListForbiddenWithoutPermission(t *testing.T) {
	wsID := uuid.New()
	handler := ApprovalWorkflowHandler{Repository: &mockApprovalWorkflowRepository{}, Authorizer: mockAuthorizer{}}
	req := approvalRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/approval-workflows", "", wsID)
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestApprovalWorkflowCreateSuccess(t *testing.T) {
	wsID := uuid.New()
	repo := &mockApprovalWorkflowRepository{}
	handler := ApprovalWorkflowHandler{Repository: repo, Authorizer: mockAuthorizer{perms: map[string]bool{"approval_workflows:manage": true}}}
	body := `{"entity_type":"leave_request","name":"Standard Leave Approval","is_active":true,"steps":[{"step_order":1,"step_name":"Manager Approval","approver_type":"manager","action_required":"approve","can_skip":false}]}`
	req := approvalRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/approval-workflows", body, wsID)
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
	wsID, entityID := uuid.New(), uuid.New()
	repo := &mockApprovalWorkflowRepository{}
	handler := ApprovalWorkflowHandler{Repository: repo, Authorizer: mockAuthorizer{}}
	body := `{"entity_type":"leave_request","entity_id":"` + entityID.String() + `","action":"invalid_action"}`
	req := approvalRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/approval-workflows/approve", body, wsID)
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
	wsID, entityID := uuid.New(), uuid.New()
	repo := &mockApprovalWorkflowRepository{history: []ApprovalActionLogRecord{{ID: uuid.New(), Action: "approved"}}}
	handler := ApprovalWorkflowHandler{Repository: repo, Authorizer: mockAuthorizer{}}
	req := approvalRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/approval-workflows/history/leave_request/"+entityID.String(), "", wsID)
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
	wsID, workflowID := uuid.New(), uuid.New()
	handler := ApprovalWorkflowHandler{Repository: &mockApprovalWorkflowRepository{}, Authorizer: mockAuthorizer{perms: map[string]bool{"approval_workflows:read": true}}}
	req := approvalRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/approval-workflows/"+workflowID.String(), "", wsID)
	req.SetPathValue("id", workflowID.String())
	rec := httptest.NewRecorder()
	handler.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestApprovalWorkflowUpdateSuccess(t *testing.T) {
	wsID, workflowID := uuid.New(), uuid.New()
	repo := &mockApprovalWorkflowRepository{}
	handler := ApprovalWorkflowHandler{Repository: repo, Authorizer: mockAuthorizer{perms: map[string]bool{"approval_workflows:manage": true}}}
	req := approvalRequest(t, http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/approval-workflows/"+workflowID.String(), `{"name":"Updated"}`, wsID)
	req.SetPathValue("id", workflowID.String())
	rec := httptest.NewRecorder()
	handler.Update(rec, req)
	if rec.Code != http.StatusOK || repo.updateCalls != 1 {
		t.Fatalf("expected update 200 and one call, got %d calls=%d", rec.Code, repo.updateCalls)
	}
}

func TestApprovalWorkflowDeleteSuccess(t *testing.T) {
	wsID, workflowID := uuid.New(), uuid.New()
	handler := ApprovalWorkflowHandler{Repository: &mockApprovalWorkflowRepository{}, Authorizer: mockAuthorizer{perms: map[string]bool{"approval_workflows:manage": true}}}
	req := approvalRequest(t, http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/approval-workflows/"+workflowID.String(), "", wsID)
	req.SetPathValue("id", workflowID.String())
	rec := httptest.NewRecorder()
	handler.Delete(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}
