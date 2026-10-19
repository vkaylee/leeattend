package api

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockLeaveRepository struct {
	submitErr           error
	approveErr          error
	typeList            []LeaveTypeRecord
	submit              LeaveRequestRecord
	balances            []EmployeeLeaveBalanceRecord
	submitCalls         int
	approveCalls        int
	updateConfigRecord  LeaveTypeRecord
	updateConfigErr     error
	deleteTypeErr       error
	restoreTypeRecord   LeaveTypeRecord
	restoreTypeErr      error
	wsBalances          []WorkspaceLeaveBalance
	wsTotal             int64
	wsErr               error
	history             []LeaveHistoryItem
	historyErr          error
	adjustCalls         int
	adjustErr           error
	policies            []LeavePolicyDefinition
	policiesErr         error
	createdPolicy       LeavePolicyDefinition
	createPolicyErr     error
	updatedPolicy       LeavePolicyDefinition
	updatePolicyErr     error
	deletePolicyErr     error
	assignedPolicy      LeavePolicyAssignment
	assignPolicyErr     error
	assignments         []LeavePolicyAssignmentDetail
	assignmentsErr      error
	deleteAssignmentErr error
	requests            []LeaveRequestRecord
	requestsErr         error
	accrualJobID        uuid.UUID
	accrualErr          error
	jobStatus           LeaveJobStatusResponse
	jobStatusErr        error
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
func (m *mockLeaveRepository) GetWorkspaceLeaveBalances(context.Context, uuid.UUID, int, int, int, string) ([]WorkspaceLeaveBalance, int64, error) {
	return m.wsBalances, m.wsTotal, m.wsErr
}
func (m *mockLeaveRepository) GetEmployeeLeaveHistory(context.Context, uuid.UUID, uuid.UUID, int, *uuid.UUID) ([]LeaveHistoryItem, error) {
	return m.history, m.historyErr
}
func (m *mockLeaveRepository) AdjustLeaveBalance(context.Context, uuid.UUID, uuid.UUID, AdjustBalanceRequest) error {
	m.adjustCalls++
	return m.adjustErr
}
func (m *mockLeaveRepository) ListLeavePolicies(context.Context, uuid.UUID) ([]LeavePolicyDefinition, error) {
	return m.policies, m.policiesErr
}
func (m *mockLeaveRepository) CreateLeavePolicy(context.Context, uuid.UUID, CreateLeavePolicyRequest) (LeavePolicyDefinition, error) {
	return m.createdPolicy, m.createPolicyErr
}
func (m *mockLeaveRepository) UpdateLeavePolicy(context.Context, uuid.UUID, uuid.UUID, UpdateLeavePolicyRequest) (LeavePolicyDefinition, error) {
	return m.updatedPolicy, m.updatePolicyErr
}
func (m *mockLeaveRepository) DeleteLeavePolicy(context.Context, uuid.UUID, uuid.UUID) error {
	return m.deletePolicyErr
}
func (m *mockLeaveRepository) AssignLeavePolicy(context.Context, uuid.UUID, AssignLeavePolicyRequest) (LeavePolicyAssignment, error) {
	return m.assignedPolicy, m.assignPolicyErr
}
func (m *mockLeaveRepository) ListLeavePolicyAssignments(context.Context, uuid.UUID) ([]LeavePolicyAssignmentDetail, error) {
	return m.assignments, m.assignmentsErr
}
func (m *mockLeaveRepository) DeleteLeavePolicyAssignment(context.Context, uuid.UUID, uuid.UUID) error {
	return m.deleteAssignmentErr
}
func (m *mockLeaveRepository) ListEmployeeLeaveRequests(context.Context, uuid.UUID, uuid.UUID) ([]LeaveRequestRecord, error) {
	return m.requests, m.requestsErr
}
func (m *mockLeaveRepository) CreateAccrualJob(context.Context, uuid.UUID, uuid.UUID, string) (uuid.UUID, error) {
	return m.accrualJobID, m.accrualErr
}
func (m *mockLeaveRepository) GetJobStatus(context.Context, uuid.UUID, uuid.UUID) (LeaveJobStatusResponse, error) {
	return m.jobStatus, m.jobStatusErr
}

type mockLeaveBehalfRepo struct {
	created     BehalfAssignment
	assignments []BehalfAssignment
	targets     []EmployeeRecord
	err         error
}

func (m *mockLeaveBehalfRepo) CreateBehalfAssignment(context.Context, uuid.UUID, CreateBehalfAssignmentRequest, string) (BehalfAssignment, error) {
	return m.created, m.err
}
func (m *mockLeaveBehalfRepo) ListBehalfAssignments(context.Context, uuid.UUID, string) ([]BehalfAssignment, error) {
	return m.assignments, m.err
}
func (m *mockLeaveBehalfRepo) DeleteBehalfAssignment(context.Context, uuid.UUID, uuid.UUID) error {
	return m.err
}
func (m *mockLeaveBehalfRepo) ListMyBehalfTargets(context.Context, uuid.UUID, uuid.UUID, string) ([]EmployeeRecord, error) {
	return m.targets, m.err
}

type mockLeaveImportRepo struct {
	job        *LeaveImportJob
	createdJob LeaveImportJob
	errorBytes []byte
	createErr  error
	processErr error
	getErr     error
}

func (m *mockLeaveImportRepo) CreateLeaveImportJob(context.Context, uuid.UUID, uuid.UUID, string) (LeaveImportJob, error) {
	return m.createdJob, m.createErr
}
func (m *mockLeaveImportRepo) ProcessLeaveImport(context.Context, uuid.UUID, uuid.UUID, LeaveImportJob, []LeaveImportRow, bool) error {
	return m.processErr
}
func (m *mockLeaveImportRepo) GetLatestLeaveImport(context.Context, uuid.UUID) (*LeaveImportJob, error) {
	return m.job, m.getErr
}
func (m *mockLeaveImportRepo) GetLeaveImport(context.Context, uuid.UUID, uuid.UUID) (*LeaveImportJob, error) {
	return m.job, m.getErr
}
func (m *mockLeaveImportRepo) GetLeaveImportErrorFile(context.Context, uuid.UUID, uuid.UUID) ([]byte, error) {
	return m.errorBytes, m.getErr
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
func TestLeaveHandlerWorkspaceBalances(t *testing.T) {
	wsID := uuid.New()
	repo := &mockLeaveRepository{
		wsBalances: []WorkspaceLeaveBalance{
			{
				EmployeeID:    uuid.New(),
				EmployeeName:  "John Doe",
				LeaveTypeID:   uuid.New(),
				LeaveTypeCode: "ANNUAL",
				LeaveTypeName: "Annual Leave",
				Entitlement:   12,
				AccruedAmount: decimal.NewFromInt(6),
				Used:          2,
				Remaining:     decimal.NewFromInt(16),
			},
		},
		wsTotal: 1,
	}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:balances:read": true}}}
	req := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/leave/balances?page=1&limit=20", "", wsID)
	rec := httptest.NewRecorder()
	handler.WorkspaceBalances(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var res PaginatedWorkspaceLeaveBalances
	if err := json.NewDecoder(rec.Body).Decode(&res); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if len(res.Data) != 1 || res.Total != 1 {
		t.Fatalf("expected 1 record, got %d (total: %d)", len(res.Data), res.Total)
	}

	// Test forbidden
	handlerForbidden := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{}}}
	reqForbidden := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/leave/balances", "", wsID)
	recForbidden := httptest.NewRecorder()
	handlerForbidden.WorkspaceBalances(recForbidden, reqForbidden)
	if recForbidden.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", recForbidden.Code)
	}
}

func TestLeaveHandlerEmployeeHistory(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockLeaveRepository{
		history: []LeaveHistoryItem{
			{
				Date:            "2026-01-01",
				TransactionType: "accrual",
				Amount:          decimal.NewFromFloat(1.5),
			},
		},
	}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:balances:read": true}}}
	req := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/leave/history?year=2026", "", wsID)
	req.SetPathValue("id", empID.String())
	rec := httptest.NewRecorder()
	handler.EmployeeHistory(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestLeaveHandlerAdjustBalance(t *testing.T) {
	wsID, empID, leaveTypeID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockLeaveRepository{}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:balances:adjust": true}}}
	body := `{"leave_type_id":"` + leaveTypeID.String() + `","amount":2.5,"reason":"Correction","transaction_date":"2026-06-01"}`
	req := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/leave/adjust", body, wsID)
	req.SetPathValue("id", empID.String())
	rec := httptest.NewRecorder()
	handler.AdjustBalance(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.adjustCalls != 1 {
		t.Fatalf("expected 1 adjust call, got %d", repo.adjustCalls)
	}

	// Test validation error: missing effective_date
	badBody := `{"leave_type_id":"` + leaveTypeID.String() + `","amount":2.5,"reason":"Correction"}`
	reqBad := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/leave/adjust", badBody, wsID)
	reqBad.SetPathValue("id", empID.String())
	recBad := httptest.NewRecorder()
	handler.AdjustBalance(recBad, reqBad)
	if recBad.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", recBad.Code, recBad.Body.String())
	}
}

func TestLeaveHandlerMyRequests(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockLeaveRepository{
		requests: []LeaveRequestRecord{
			{
				ID:        uuid.New(),
				Status:    "approved",
				StartDate: "2026-05-01",
				EndDate:   "2026-05-03",
			},
		},
	}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:requests:read": true}}}
	req := leaveRequest(t, http.MethodGet, "/api/v1/employees/"+empID.String()+"/leave/requests", "", wsID)
	req.SetPathValue("id", empID.String())
	rec := httptest.NewRecorder()
	handler.MyRequests(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestLeaveHandlerPoliciesCRUD(t *testing.T) {
	wsID, policyID, typeID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockLeaveRepository{
		policies: []LeavePolicyDefinition{
			{ID: policyID, WorkspaceID: wsID, LeaveTypeID: typeID, Name: "Standard Policy", RuleConfig: LeavePolicyConfig{Frequency: "monthly"}},
		},
		createdPolicy: LeavePolicyDefinition{ID: policyID, WorkspaceID: wsID, LeaveTypeID: typeID, Name: "New Policy", RuleConfig: LeavePolicyConfig{Frequency: "monthly"}},
		updatedPolicy: LeavePolicyDefinition{ID: policyID, WorkspaceID: wsID, LeaveTypeID: typeID, Name: "Updated Policy", RuleConfig: LeavePolicyConfig{Frequency: "monthly"}},
	}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:policies:manage": true}}}

	// List
	reqList := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/leave/policies", "", wsID)
	recList := httptest.NewRecorder()
	handler.ListPolicies(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recList.Code)
	}

	// Create
	createBody := `{"leave_type_id":"` + typeID.String() + `","name":"New Policy","rule_config":{}}`
	reqCreate := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/leave/policies", createBody, wsID)
	recCreate := httptest.NewRecorder()
	handler.CreatePolicy(recCreate, reqCreate)
	if recCreate.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", recCreate.Code, recCreate.Body.String())
	}

	// Update
	updateBody := `{"name":"Updated Policy"}`
	reqUpdate := leaveRequest(t, http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/leave/policies/"+policyID.String(), updateBody, wsID)
	reqUpdate.SetPathValue("policy_id", policyID.String())
	recUpdate := httptest.NewRecorder()
	handler.UpdatePolicy(recUpdate, reqUpdate)
	if recUpdate.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recUpdate.Code, recUpdate.Body.String())
	}

	// Delete
	reqDelete := leaveRequest(t, http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/leave/policies/"+policyID.String(), "", wsID)
	reqDelete.SetPathValue("policy_id", policyID.String())
	recDelete := httptest.NewRecorder()
	handler.DeletePolicy(recDelete, reqDelete)
	if recDelete.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", recDelete.Code, recDelete.Body.String())
	}
}

func TestLeaveHandlerPolicyAssignments(t *testing.T) {
	wsID, assignmentID, policyID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockLeaveRepository{
		assignedPolicy: LeavePolicyAssignment{
			ID:            assignmentID,
			WorkspaceID:   wsID,
			AssigneeType:  "workspace",
			PolicyID:      policyID,
			EffectiveFrom: "2026-01-01",
		},
		assignments: []LeavePolicyAssignmentDetail{
			{
				LeavePolicyAssignment: LeavePolicyAssignment{
					ID:            assignmentID,
					WorkspaceID:   wsID,
					AssigneeType:  "workspace",
					PolicyID:      policyID,
					EffectiveFrom: "2026-01-01",
				},
				PolicyName:    "Standard",
				LeaveTypeName: "Annual",
			},
		},
	}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:policies:manage": true}}}

	// Assign
	assignBody := `{"policy_id":"` + policyID.String() + `","assignee_type":"workspace","effective_from":"2026-01-01"}`
	reqAssign := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/leave/policies/assign", assignBody, wsID)
	recAssign := httptest.NewRecorder()
	handler.AssignPolicy(recAssign, reqAssign)
	if recAssign.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", recAssign.Code, recAssign.Body.String())
	}

	// List assignments
	reqList := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/leave/policies/assignments", "", wsID)
	recList := httptest.NewRecorder()
	handler.ListPolicyAssignments(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recList.Code, recList.Body.String())
	}

	// Delete assignment
	reqDelete := leaveRequest(t, http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/leave/policies/assignments/"+assignmentID.String(), "", wsID)
	reqDelete.SetPathValue("assignment_id", assignmentID.String())
	recDelete := httptest.NewRecorder()
	handler.DeletePolicyAssignment(recDelete, reqDelete)
	if recDelete.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", recDelete.Code, recDelete.Body.String())
	}
}

func TestLeaveHandlerAccrualAndJobStatus(t *testing.T) {
	wsID, jobID := uuid.New(), uuid.New()
	repo := &mockLeaveRepository{
		accrualJobID: jobID,
		jobStatus: LeaveJobStatusResponse{
			JobID:  jobID,
			Status: "completed",
		},
	}
	handler := LeaveHandler{Repository: repo, Authorizer: mockLeaveAuthorizer{allowed: map[string]bool{"leave:accrual:run": true}}}

	// Trigger accrual
	reqRun := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/leave/accrual/run", `{"date":"2026-10-20"}`, wsID)
	recRun := httptest.NewRecorder()
	handler.TriggerAccrual(recRun, reqRun)
	if recRun.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recRun.Code, recRun.Body.String())
	}

	// Check status
	reqStatus := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/jobs/"+jobID.String()+"/status", "", wsID)
	reqStatus.SetPathValue("job_id", jobID.String())
	recStatus := httptest.NewRecorder()
	handler.JobStatus(recStatus, reqStatus)
	if recStatus.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recStatus.Code, recStatus.Body.String())
	}
}

func TestLeaveHandlerBehalfAssignments(t *testing.T) {
	wsID, behalfID, targetEmpID := uuid.New(), uuid.New(), uuid.New()
	behalfRepo := &mockLeaveBehalfRepo{
		created: BehalfAssignment{
			ID:               behalfID,
			WorkspaceID:      wsID,
			TargetEmployeeID: &targetEmpID,
			AssignmentType:   "employee",
		},
		assignments: []BehalfAssignment{
			{
				ID:               behalfID,
				WorkspaceID:      wsID,
				TargetEmployeeID: &targetEmpID,
				AssignmentType:   "employee",
			},
		},
		targets: []EmployeeRecord{
			{
				ID:       targetEmpID,
				FullName: "Subordinate Employee",
			},
		},
	}
	handler := LeaveHandler{
		Repository:       &mockLeaveRepository{},
		BehalfRepository: behalfRepo,
		Authorizer:       mockLeaveAuthorizer{allowed: map[string]bool{"leave:policies:manage": true}},
	}

	// List behalf assignments
	reqList := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/leave/behalf-assignments", "", wsID)
	recList := httptest.NewRecorder()
	handler.ListBehalfAssignments(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recList.Code, recList.Body.String())
	}

	// Create behalf assignment
	createBody := `{"assignee_user_id":"` + uuid.New().String() + `","target_employee_id":"` + targetEmpID.String() + `"}`
	reqCreate := leaveRequest(t, http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/leave/behalf-assignments", createBody, wsID)
	recCreate := httptest.NewRecorder()
	handler.CreateBehalfAssignment(recCreate, reqCreate)
	if recCreate.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recCreate.Code, recCreate.Body.String())
	}

	// My targets
	reqTargets := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/leave/behalf-assignments/my-targets", "", wsID)
	recTargets := httptest.NewRecorder()
	handler.GetMyBehalfTargets(recTargets, reqTargets)
	if recTargets.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recTargets.Code, recTargets.Body.String())
	}

	// Delete behalf assignment
	reqDelete := leaveRequest(t, http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/leave/behalf-assignments/"+behalfID.String(), "", wsID)
	reqDelete.SetPathValue("id", behalfID.String())
	recDelete := httptest.NewRecorder()
	handler.DeleteBehalfAssignment(recDelete, reqDelete)
	if recDelete.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recDelete.Code, recDelete.Body.String())
	}
}

func TestLeaveHandlerImportEndpoints(t *testing.T) {
	wsID, jobID := uuid.New(), uuid.New()
	now := time.Now()
	importRepo := &mockLeaveImportRepo{
		job: &LeaveImportJob{
			ID:            jobID,
			WorkspaceID:   wsID,
			Status:        "completed",
			TotalRows:     10,
			ProcessedRows: 8,
			CreatedAt:     now,
		},
		errorBytes: []byte("PK\x03\x04fake-xlsx-content"),
	}
	handler := LeaveHandler{
		Repository:       &mockLeaveRepository{},
		ImportRepository: importRepo,
		Authorizer:       mockLeaveAuthorizer{allowed: map[string]bool{"leave:requests:submit": true}},
	}

	// Template download
	reqTpl := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/leave/upload-template", "", wsID)
	recTpl := httptest.NewRecorder()
	handler.UploadTemplate(recTpl, reqTpl)
	if recTpl.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recTpl.Code)
	}
	if recTpl.Header().Get("Content-Type") != "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" {
		t.Fatalf("unexpected content type: %s", recTpl.Header().Get("Content-Type"))
	}

	// Latest import
	reqLatest := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/leave/import/latest", "", wsID)
	recLatest := httptest.NewRecorder()
	handler.LatestImport(recLatest, reqLatest)
	if recLatest.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recLatest.Code, recLatest.Body.String())
	}

	// Import errors
	reqErr := leaveRequest(t, http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/leave/import/"+jobID.String()+"/errors", "", wsID)
	reqErr.SetPathValue("job_id", jobID.String())
	recErr := httptest.NewRecorder()
	handler.ImportErrors(recErr, reqErr)
	if recErr.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recErr.Code, recErr.Body.String())
	}
}

func TestLeaveDecimalCalculations(t *testing.T) {
	tests := []struct {
		name        string
		entitlement int
		carriedOver int
		accrued     string
		used        int
		expectedRem string
	}{
		{
			name:        "Standard accrual calculation",
			entitlement: 12,
			carriedOver: 2,
			accrued:     "3.5",
			used:        5,
			expectedRem: "12.5",
		},
		{
			name:        "Negative adjustment deduction",
			entitlement: 10,
			carriedOver: 0,
			accrued:     "-2.25",
			used:        3,
			expectedRem: "4.75",
		},
		{
			name:        "Zero values",
			entitlement: 0,
			carriedOver: 0,
			accrued:     "0",
			used:        0,
			expectedRem: "0",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			acc, err := decimal.NewFromString(tc.accrued)
			if err != nil {
				t.Fatalf("parse accrued: %v", err)
			}
			base := decimal.NewFromInt(int64(tc.entitlement + tc.carriedOver)).Add(acc)
			rem := base.Sub(decimal.NewFromInt(int64(tc.used)))
			expected, _ := decimal.NewFromString(tc.expectedRem)
			if !rem.Equal(expected) {
				t.Fatalf("expected remaining %s, got %s", expected.String(), rem.String())
			}
		})
	}
}
