package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockOvertimeAuthorizer struct {
	allowed map[string]bool
	scoped  map[string]bool
}

func (m mockOvertimeAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return true, nil
}
func (m mockOvertimeAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return m.allowed[permission]
}
func (m mockOvertimeAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (m mockOvertimeAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (m mockOvertimeAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool {
	return false
}
func (m mockOvertimeAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, permission string, _ *uuid.UUID) bool {
	if m.scoped != nil {
		return m.scoped[permission]
	}
	return m.allowed[permission]
}
func (m mockOvertimeAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

type mockOvertimeRepo struct {
	policies      []OvertimePolicy
	policyDetail  OvertimePolicyDetail
	requests      []OvertimeRequest
	requestDetail OvertimeRequestDetail
	employee      EmployeeRecord
	submitCalls   int
	consentCalls  int
	claimCalls    int
	approveCalls  int
	err           error
}

func (m *mockOvertimeRepo) ListOvertimePolicies(_ context.Context, p, l int) (OvertimePolicyListResponse, error) {
	return OvertimePolicyListResponse{Data: m.policies, Total: int64(len(m.policies)), Page: p, Limit: l}, m.err
}
func (m *mockOvertimeRepo) CreateOvertimePolicy(_ context.Context, req CreateOvertimePolicyRequest) (OvertimePolicy, error) {
	return OvertimePolicy{ID: uuid.New(), Name: req.Name, Description: req.Description}, m.err
}
func (m *mockOvertimeRepo) GetOvertimePolicy(_ context.Context, _ uuid.UUID) (OvertimePolicyDetail, error) {
	return m.policyDetail, m.err
}
func (m *mockOvertimeRepo) AddOvertimeRule(_ context.Context, id uuid.UUID, req CreateOvertimeRuleRequest) (OvertimeRule, error) {
	return OvertimeRule{ID: uuid.New(), PolicyID: id, Priority: req.Priority, Multiplier: req.Multiplier}, m.err
}
func (m *mockOvertimeRepo) AssignOvertimePolicy(_ context.Context, _ AssignOvertimePolicyRequest) error {
	return m.err
}
func (m *mockOvertimeRepo) ListOvertimeRequests(_ context.Context, p, l int) (OvertimeRequestListResponse, error) {
	return OvertimeRequestListResponse{Data: m.requests, Total: int64(len(m.requests)), Page: p, Limit: l}, m.err
}
func (m *mockOvertimeRepo) CreateOvertimeRequest(_ context.Context, u uuid.UUID, req CreateOvertimeRequest) (OvertimeRequest, error) {
	return OvertimeRequest{ID: uuid.New(), RequestedByUserID: u, PlannedDate: req.PlannedDate, PlannedMinutes: req.PlannedMinutes, Reason: req.Reason, Status: "draft"}, m.err
}
func (m *mockOvertimeRepo) GetOvertimeRequest(_ context.Context, _ uuid.UUID) (OvertimeRequestDetail, error) {
	return m.requestDetail, m.err
}
func (m *mockOvertimeRepo) SubmitOvertimeRequest(_ context.Context, _ uuid.UUID) error {
	m.submitCalls++
	return m.err
}
func (m *mockOvertimeRepo) ConsentOvertimeRequest(_ context.Context, _, _ uuid.UUID, _ EmployeeConsentRequest) error {
	m.consentCalls++
	return m.err
}
func (m *mockOvertimeRepo) GetEmployee(_ context.Context, _ uuid.UUID) (EmployeeRecord, error) {
	return m.employee, m.err
}
func (m *mockOvertimeRepo) EmployeeDepartmentID(_ context.Context, _ uuid.UUID) (*uuid.UUID, error) {
	return nil, m.err
}
func (m *mockOvertimeRepo) ClaimOvertime(_ context.Context, _ uuid.UUID, _ ClaimOTRequest, _ uuid.UUID) error {
	m.claimCalls++
	return m.err
}
func (m *mockOvertimeRepo) ApproveOvertime(_ context.Context, _ uuid.UUID, _ ApproveOTRequest, _ uuid.UUID) error {
	m.approveCalls++
	return m.err
}

func otReq(method, path string, body []byte, userID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewReader(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	return req
}

func TestOvertimePolicies_ForbiddenWithoutPermission(t *testing.T) {
	user := uuid.New()
	h := OvertimeHandler{Repository: &mockOvertimeRepo{}, Authorizer: mockOvertimeAuthorizer{allowed: map[string]bool{}}}

	req := otReq(http.MethodPost, "/api/v1" + "/overtime/policies", []byte(`{"name":"Standard OT"}`), user)
	rec := httptest.NewRecorder()
	h.CreatePolicy(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestOvertimePolicies_CreateAndListSuccess(t *testing.T) {
	user := uuid.New()
	repo := &mockOvertimeRepo{policies: []OvertimePolicy{{ID: uuid.New(), Name: "Standard"}}}
	auth := mockOvertimeAuthorizer{allowed: map[string]bool{PermissionOvertimePoliciesManage: true}}
	h := OvertimeHandler{Repository: repo, Authorizer: auth}

	req := otReq(http.MethodPost, "/api/v1" + "/overtime/policies", []byte(`{"name":"Weekend OT"}`), user)
	rec := httptest.NewRecorder()
	h.CreatePolicy(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}

	listReq := otReq(http.MethodGet, "/api/v1" + "/overtime/policies", nil, user)
	listRec := httptest.NewRecorder()
	h.ListPolicies(listRec, listReq)
	if listRec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", listRec.Code)
	}
}

func TestOvertimePolicies_AddRuleValidation(t *testing.T) {
	policyID := uuid.New()
	user := uuid.New()
	h := OvertimeHandler{Repository: &mockOvertimeRepo{}, Authorizer: mockOvertimeAuthorizer{allowed: map[string]bool{PermissionOvertimePoliciesManage: true}}}

	// negative multiplier
	req := otReq(http.MethodPost, "/api/v1" + "/overtime/policies/"+policyID.String()+"/rules", []byte(`{"priority":1,"multiplier":"-1.5"}`), user)
	req.SetPathValue("policy_id", policyID.String())
	rec := httptest.NewRecorder()
	h.AddRule(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d", rec.Code)
	}

	// success
	req2 := otReq(http.MethodPost, "/api/v1" + "/overtime/policies/"+policyID.String()+"/rules", []byte(`{"priority":1,"multiplier":"1.50","days_of_week":127}`), user)
	req2.SetPathValue("policy_id", policyID.String())
	rec2 := httptest.NewRecorder()
	h.AddRule(rec2, req2)
	if rec2.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec2.Code)
	}
}

func TestOvertimePolicies_AssignValidation(t *testing.T) {
	policyID := uuid.New()
	user := uuid.New()
	h := OvertimeHandler{Repository: &mockOvertimeRepo{}, Authorizer: mockOvertimeAuthorizer{allowed: map[string]bool{PermissionOvertimePoliciesManage: true}}}

	// invalid assignee type
	req := otReq(http.MethodPost, "/api/v1" + "/overtime/policies/assign", []byte(`{"assignee_type":"invalid","policy_id":"`+policyID.String()+`","effective_from":"2026-10-20"}`), user)
	rec := httptest.NewRecorder()
	h.AssignPolicy(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d", rec.Code)
	}

	// valid organization assignment
	req2 := otReq(http.MethodPost, "/api/v1" + "/overtime/policies/assign", []byte(`{"assignee_type":"organization","policy_id":"`+policyID.String()+`","effective_from":"2026-10-20"}`), user)
	rec2 := httptest.NewRecorder()
	h.AssignPolicy(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec2.Code)
	}
	// valid user assignment
	targetUserID := uuid.New()
	req3 := otReq(http.MethodPost, "/api/v1" + "/overtime/policies/assign", []byte(`{"assignee_type":"user","assignee_id":"`+targetUserID.String()+`","policy_id":"`+policyID.String()+`","effective_from":"2026-10-20"}`), user)
	rec3 := httptest.NewRecorder()
	h.AssignPolicy(rec3, req3)
	if rec3.Code != http.StatusOK {
		t.Fatalf("expected 200 for user assignment, got %d", rec3.Code)
	}
}

func TestOvertimeRequest_SelfVersusBehalfBypass(t *testing.T) {
	user := uuid.New()
	otherUser := uuid.New()
	empID := uuid.New()

	// Self employee match
	selfRepo := &mockOvertimeRepo{employee: EmployeeRecord{ID: empID, UserID: &user}}
	selfAuth := mockOvertimeAuthorizer{allowed: map[string]bool{PermissionOvertimeRequestCreate: true}}
	selfHandler := OvertimeHandler{Repository: selfRepo, Authorizer: selfAuth}

	payload := `{"planned_date":"2026-10-25","planned_minutes":120,"reason":"Release preparation","employees":["` + empID.String() + `"]}`
	req := otReq(http.MethodPost, "/api/v1" + "/overtime/requests", []byte(payload), user)
	rec := httptest.NewRecorder()
	selfHandler.CreateRequest(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected self-request to succeed with 201, got %d: %s", rec.Code, rec.Body.String())
	}

	// Other employee without behalf permission -> 403
	behalfRepo := &mockOvertimeRepo{employee: EmployeeRecord{ID: empID, UserID: &otherUser}}
	behalfHandler := OvertimeHandler{Repository: behalfRepo, Authorizer: selfAuth} // no create_on_behalf perm
	req2 := otReq(http.MethodPost, "/api/v1" + "/overtime/requests", []byte(payload), user)
	rec2 := httptest.NewRecorder()
	behalfHandler.CreateRequest(rec2, req2)
	if rec2.Code != http.StatusForbidden {
		t.Fatalf("expected 403 when creating on behalf without permission, got %d", rec2.Code)
	}

	// Other employee with behalf permission -> 201
	behalfAuth := mockOvertimeAuthorizer{allowed: map[string]bool{
		PermissionOvertimeRequestCreate:       true,
		PermissionOvertimeRequestCreateBehalf: true,
	}}
	behalfHandlerWithPerm := OvertimeHandler{Repository: behalfRepo, Authorizer: behalfAuth}
	req3 := otReq(http.MethodPost, "/api/v1" + "/overtime/requests", []byte(payload), user)
	rec3 := httptest.NewRecorder()
	behalfHandlerWithPerm.CreateRequest(rec3, req3)
	if rec3.Code != http.StatusCreated {
		t.Fatalf("expected 201 when creating on behalf with permission, got %d", rec3.Code)
	}
}

func TestOvertimeRequest_SubmitWorkflow(t *testing.T) {
	user := uuid.New()
	reqID := uuid.New()
	repo := &mockOvertimeRepo{}
	h := OvertimeHandler{Repository: repo, Authorizer: mockOvertimeAuthorizer{allowed: map[string]bool{PermissionOvertimeRequestCreate: true}}}

	req := otReq(http.MethodPost, "/api/v1" + "/overtime/requests/"+reqID.String()+"/submit", nil, user)
	req.SetPathValue("id", reqID.String())
	rec := httptest.NewRecorder()
	h.SubmitRequest(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if repo.submitCalls != 1 {
		t.Fatalf("expected submit call count 1, got %d", repo.submitCalls)
	}
}

func TestOvertimeRequest_ConsentSelfAndBehalf(t *testing.T) {
	user := uuid.New()
	reqID := uuid.New()
	empID := uuid.New()

	// Self consent without special behalf permission
	repo := &mockOvertimeRepo{employee: EmployeeRecord{ID: empID, UserID: &user}}
	h := OvertimeHandler{Repository: repo, Authorizer: mockOvertimeAuthorizer{allowed: map[string]bool{}}}
	req := otReq(http.MethodPost, "/api/v1" + "/overtime/requests/"+reqID.String()+"/consent/"+empID.String(), []byte(`{"status":"accepted"}`), user)
	req.SetPathValue("id", reqID.String())
	req.SetPathValue("emp_id", empID.String())
	rec := httptest.NewRecorder()
	h.ConsentRequest(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for self consent, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestAttendanceOT_ClaimAndApprove(t *testing.T) {
	user := uuid.New()
	otherUser := uuid.New()
	empID := uuid.New()

	// Claim OT - self
	repo := &mockOvertimeRepo{employee: EmployeeRecord{ID: empID, UserID: &user}}
	h := OvertimeHandler{Repository: repo, Authorizer: mockOvertimeAuthorizer{allowed: map[string]bool{}}}
	claimReq := otReq(http.MethodPost, "/api/v1" + "/attendance/claim-ot/"+empID.String(), []byte(`{"date":"2026-10-20","claimed_minutes":60}`), user)
	claimReq.SetPathValue("employee_id", empID.String())
	claimRec := httptest.NewRecorder()
	h.ClaimOT(claimRec, claimReq)
	if claimRec.Code != http.StatusOK {
		t.Fatalf("expected 200 for self OT claim, got %d: %s", claimRec.Code, claimRec.Body.String())
	}

	// Claim OT - on behalf forbidden
	repoOther := &mockOvertimeRepo{employee: EmployeeRecord{ID: empID, UserID: &otherUser}}
	hOther := OvertimeHandler{Repository: repoOther, Authorizer: mockOvertimeAuthorizer{allowed: map[string]bool{}}}
	claimReq2 := otReq(http.MethodPost, "/api/v1" + "/attendance/claim-ot/"+empID.String(), []byte(`{"date":"2026-10-20","claimed_minutes":60}`), user)
	claimReq2.SetPathValue("employee_id", empID.String())
	claimRec2 := httptest.NewRecorder()
	hOther.ClaimOT(claimRec2, claimReq2)
	if claimRec2.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for delegated OT claim without permission, got %d", claimRec2.Code)
	}

	// Approve OT - permission check
	hApprove := OvertimeHandler{Repository: repo, Authorizer: mockOvertimeAuthorizer{allowed: map[string]bool{PermissionOvertimeApprove: true}}}
	appReq := otReq(http.MethodPost, "/api/v1" + "/attendance/approve-ot/"+empID.String(), []byte(`{"date":"2026-10-20","approved_minutes":60,"status":"approved"}`), user)
	appReq.SetPathValue("employee_id", empID.String())
	appRec := httptest.NewRecorder()
	hApprove.ApproveOT(appRec, appReq)
	if appRec.Code != http.StatusOK {
		t.Fatalf("expected 200 for OT approval, got %d: %s", appRec.Code, appRec.Body.String())
	}
}

func TestAttendanceOT_BehalfAssignments(t *testing.T) {
	user := uuid.New()
	targetEmp := uuid.New()
	behalfRepo := &mockBehalfRepo{
		assignments: []BehalfAssignment{{ID: uuid.New(), AssigneeUserID: user, TargetEmployeeID: &targetEmp, AssignmentType: OvertimeClaimBehalfAssignmentType}},
		created:     BehalfAssignment{ID: uuid.New(), AssigneeUserID: user, TargetEmployeeID: &targetEmp, AssignmentType: OvertimeClaimBehalfAssignmentType},
	}
	h := OvertimeHandler{BehalfRepository: behalfRepo, Authorizer: mockOvertimeAuthorizer{allowed: map[string]bool{PermissionOvertimePoliciesManage: true}}}

	// List claim behalf assignments (no permission needed)
	listReq := otReq(http.MethodGet, "/api/v1" + "/attendance/claim-ot/behalf-assignments", nil, user)
	listRec := httptest.NewRecorder()
	h.ListClaimBehalf(listRec, listReq)
	if listRec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", listRec.Code)
	}

	// Create claim behalf assignment
	body := `{"assignee_user_id":"` + user.String() + `","target_employee_id":"` + targetEmp.String() + `"}`
	createReq := otReq(http.MethodPost, "/api/v1" + "/attendance/claim-ot/behalf-assignments", []byte(body), user)
	createRec := httptest.NewRecorder()
	h.CreateClaimBehalf(createRec, createReq)
	if createRec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", createRec.Code)
	}
}
