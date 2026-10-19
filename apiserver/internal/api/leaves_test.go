package api

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockLeaveRepository struct {
	submitErr          error
	approveErr         error
	typeList           []LeaveTypeRecord
	submit             LeaveRequestRecord
	balances           []EmployeeLeaveBalanceRecord
	submitCalls        int
	approveCalls       int
	updateConfigRecord LeaveTypeRecord
	updateConfigErr    error
	deleteTypeErr      error
	restoreTypeRecord  LeaveTypeRecord
	restoreTypeErr     error
}

func (m *mockLeaveRepository) CreateLeaveType(context.Context, uuid.UUID, CreateLeaveTypeRequest) (LeaveTypeRecord, error) {
	return LeaveTypeRecord{ID: uuid.New(), Code: "annual", Name: "Annual", IsPaid: true}, nil
}
func (m *mockLeaveRepository) ListLeaveTypes(context.Context, uuid.UUID) ([]LeaveTypeRecord, error) {
	return m.typeList, nil
}
func (m *mockLeaveRepository) UpdateLeaveTypeConfig(context.Context, uuid.UUID, uuid.UUID, json.RawMessage) (LeaveTypeRecord, error) {
	return m.updateConfigRecord, m.updateConfigErr
}
func (m *mockLeaveRepository) DeleteLeaveType(context.Context, uuid.UUID, uuid.UUID) error {
	return m.deleteTypeErr
}
func (m *mockLeaveRepository) RestoreLeaveType(context.Context, uuid.UUID, uuid.UUID) (LeaveTypeRecord, error) {
	return m.restoreTypeRecord, m.restoreTypeErr
}
func (m *mockLeaveRepository) SubmitLeaveRequest(_ context.Context, _, _ uuid.UUID, _ SubmitLeaveRequest, autoApprove bool) (LeaveRequestRecord, error) {
	m.submitCalls++
	m.submit.Status = "pending"
	if autoApprove {
		m.submit.Status = "pre_approved"
	}
	return m.submit, m.submitErr
}
func (m *mockLeaveRepository) ListWorkspaceLeaveRequests(context.Context, uuid.UUID, *string) ([]LeaveRequestRecord, error) {
	return []LeaveRequestRecord{}, nil
}
func (m *mockLeaveRepository) ApproveLeaveRequest(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, string) error {
	m.approveCalls++
	return m.approveErr
}
func (m *mockLeaveRepository) GetEmployeeLeaveBalances(context.Context, uuid.UUID, uuid.UUID, int) ([]EmployeeLeaveBalanceRecord, error) {
	return m.balances, nil
}
func (m *mockLeaveRepository) ResolveEmployeeForUser(context.Context, uuid.UUID, uuid.UUID) (uuid.UUID, error) {
	return uuid.Nil, nil
}

type mockLeaveAuthorizer struct {
	allowed map[string]bool
}

func (a mockLeaveAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, _ uuid.UUID, permission string) bool {
	return a.allowed[permission]
}

func leaveRequest(t *testing.T, method, path, body string, wsID uuid.UUID) *http.Request {
	t.Helper()
	req := httptest.NewRequest(method, path, strings.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	return req
}

func TestLeaveHandlerSubmitDefaultsFullDayAndPending(t *testing.T) {
	wsID, employmentID, leaveTypeID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockLeaveRepository{}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:requests:submit": true}}}
	req := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/leave/submit", `{"employment_id":"`+employmentID.String()+`","leave_type_id":"`+leaveTypeID.String()+`","start_date":"2026-10-20","end_date":"2026-10-21"}`, wsID)
	rec := httptest.NewRecorder()
	handler.Submit(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.submitCalls != 1 || repo.submit.Status != "pending" {
		t.Fatalf("unexpected submit state: calls=%d status=%s", repo.submitCalls, repo.submit.Status)
	}
}

func TestLeaveHandlerSubmitRejectsInvalidCustomHours(t *testing.T) {
	wsID, employmentID, leaveTypeID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockLeaveRepository{}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:requests:submit": true}}}
	req := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/leave/submit", `{"employment_id":"`+employmentID.String()+`","leave_type_id":"`+leaveTypeID.String()+`","start_date":"2026-10-20","end_date":"2026-10-20","duration_type":"custom_hours","start_time":"17:00","end_time":"09:00"}`, wsID)
	rec := httptest.NewRecorder()
	handler.Submit(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.submitCalls != 0 {
		t.Fatal("invalid custom-hours request reached repository")
	}
}

func TestLeaveHandlerSubmitAutoApprovesWithPermission(t *testing.T) {
	wsID := uuid.New()
	repo := &mockLeaveRepository{}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:requests:submit": true, "leave:requests:approve": true}}}
	req := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/leave/submit", `{"employment_id":"`+uuid.New().String()+`","leave_type_id":"`+uuid.New().String()+`","start_date":"2026-10-20","end_date":"2026-10-20"}`, wsID)
	rec := httptest.NewRecorder()
	handler.Submit(rec, req)
	if rec.Code != http.StatusCreated || repo.submit.Status != "pre_approved" {
		t.Fatalf("expected pre-approved 201, got %d status=%s", rec.Code, repo.submit.Status)
	}
}

func TestLeaveHandlerApproveRequiresAllowedStatus(t *testing.T) {
	wsID := uuid.New()
	repo := &mockLeaveRepository{}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:requests:approve": true}}}
	req := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/leave/requests/"+uuid.New().String()+"/approve", `{"status":"pending"}`, wsID)
	req.SetPathValue("id", uuid.New().String())
	rec := httptest.NewRecorder()
	handler.Approve(rec, req)
	if rec.Code != http.StatusBadRequest || repo.approveCalls != 0 {
		t.Fatalf("expected validation without repository call, status=%d calls=%d", rec.Code, repo.approveCalls)
	}
}

func TestLeaveHandlerBalancesPreserveDecimal(t *testing.T) {
	wsID, employeeID := uuid.New(), uuid.New()
	repo := &mockLeaveRepository{balances: []EmployeeLeaveBalanceRecord{{LeaveTypeID: uuid.New(), AccruedAmount: decimal.RequireFromString("1.25"), Remaining: decimal.RequireFromString("13.25")}}}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:balances:read": true}}}
	req := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/leave/balances", "", wsID)
	req.SetPathValue("id", employeeID.String())
	rec := httptest.NewRecorder()
	handler.Balances(rec, req)
	if rec.Code != http.StatusOK || !strings.Contains(rec.Body.String(), "1.25") {
		t.Fatalf("expected decimal balance response, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestLeaveHandlerUpdateTypeConfig(t *testing.T) {
	wsID, typeID := uuid.New(), uuid.New()
	repo := &mockLeaveRepository{
		updateConfigRecord: LeaveTypeRecord{ID: typeID, Code: "annual", Name: "Annual", IsPaid: true},
	}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:types:manage": true}}}
	req := leaveRequest(t, http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/leave/types/"+typeID.String()+"/config", `{"policy_config":{"requires_document":true,"allow_negative_balance":false,"approval_required":true}}`, wsID)
	req.SetPathValue("id", typeID.String())
	rec := httptest.NewRecorder()
	handler.UpdateTypeConfig(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestLeaveHandlerDeleteType(t *testing.T) {
	wsID, typeID := uuid.New(), uuid.New()
	repo := &mockLeaveRepository{}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:types:manage": true}}}
	req := leaveRequest(t, http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/leave/types/"+typeID.String(), "", wsID)
	req.SetPathValue("id", typeID.String())
	rec := httptest.NewRecorder()
	handler.DeleteType(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
}

func TestLeaveHandlerRestoreType(t *testing.T) {
	wsID, typeID := uuid.New(), uuid.New()
	repo := &mockLeaveRepository{
		restoreTypeRecord: LeaveTypeRecord{ID: typeID, Code: "annual", Name: "Annual", IsPaid: true},
	}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:types:manage": true}}}
	req := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/leave/types/"+typeID.String()+"/restore", "", wsID)
	req.SetPathValue("id", typeID.String())
	rec := httptest.NewRecorder()
	handler.RestoreType(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestLeaveHandlerLifecycleForbidden(t *testing.T) {
	wsID, typeID := uuid.New(), uuid.New()
	handler := LeaveHandler{Repository: &mockLeaveRepository{}, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{}}}
	req := leaveRequest(t, http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/leave/types/"+typeID.String(), "", wsID)
	req.SetPathValue("id", typeID.String())
	rec := httptest.NewRecorder()
	handler.DeleteType(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}
