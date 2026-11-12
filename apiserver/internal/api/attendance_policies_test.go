package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAttendancePolicyAuthorizer struct {
	allowed bool
}

func (m mockAttendancePolicyAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, _ string) bool {
	return m.allowed
}
func (m mockAttendancePolicyAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return m.allowed, nil
}
func (mockAttendancePolicyAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockAttendancePolicyAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (m mockAttendancePolicyAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool {
	return m.allowed
}
func (m mockAttendancePolicyAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, _ string, _ *uuid.UUID) bool {
	return m.allowed
}
func (mockAttendancePolicyAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

type mockAttendancePolicyRepo struct {
	rules            AttendanceRules
	versions         []AttendancePolicyVersionResponse
	versionResult    AttendancePolicyVersionResponse
	assignments      []PolicyAssignmentResponse
	assignmentResult PolicyAssignmentResponse
	err              error
}

func (m *mockAttendancePolicyRepo) GetDefaultPolicyRules(_ context.Context) (AttendanceRules, error) {
	return m.rules, m.err
}
func (m *mockAttendancePolicyRepo) GetDefaultPolicyVersions(_ context.Context) ([]AttendancePolicyVersionResponse, error) {
	return m.versions, m.err
}
func (m *mockAttendancePolicyRepo) UpdateDefaultPolicyRules(_ context.Context, _ uuid.UUID, _ UpdateAttendanceRulesRequest) (AttendancePolicyVersionResponse, error) {
	return m.versionResult, m.err
}
func (m *mockAttendancePolicyRepo) ListPolicyAssignments(_ context.Context, page, limit int, _, _ string) (PolicyAssignmentListResponse, error) {
	return PolicyAssignmentListResponse{Data: m.assignments, Total: int64(len(m.assignments)), Page: page, Limit: limit}, m.err
}
func (m *mockAttendancePolicyRepo) UpsertPolicyAssignment(_ context.Context, _ uuid.UUID, _ UpsertPolicyAssignmentRequest) (PolicyAssignmentResponse, error) {
	return m.assignmentResult, m.err
}
func (m *mockAttendancePolicyRepo) DeletePolicyAssignment(_ context.Context, _ uuid.UUID) error {
	return m.err
}

type mockBehalfRepo struct {
	created     BehalfAssignment
	assignments []BehalfAssignment
	targets     []EmployeeRecord
	err         error
}

func (m *mockBehalfRepo) CreateBehalfAssignment(_ context.Context, _ CreateBehalfAssignmentRequest, _ string) (BehalfAssignment, error) {
	return m.created, m.err
}
func (m *mockBehalfRepo) ListBehalfAssignments(_ context.Context, _ string) ([]BehalfAssignment, error) {
	return m.assignments, m.err
}
func (m *mockBehalfRepo) DeleteBehalfAssignment(_ context.Context, _ uuid.UUID, _ string) error {
	return m.err
}
func (m *mockBehalfRepo) ListMyBehalfTargets(_ context.Context, _ uuid.UUID, _ string) ([]EmployeeRecord, error) {
	return m.targets, m.err
}

func TestAttendancePolicy_GetRules_Forbidden(t *testing.T) {
	h := AttendancePolicyHandler{PolicyRepository: &mockAttendancePolicyRepo{}, Authorizer: mockAttendancePolicyAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/rules", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.GetRules(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 got %d", rec.Code)
	}
}

func TestAttendancePolicy_GetRules_Success(t *testing.T) {
	grace := 15
	expected := AttendanceRules{WorkUnitMinutes: 30, RoundingPolicy: "floor", EarlyArrivalStrategy: "block", GracePeriodMinutes: &grace}
	h := AttendancePolicyHandler{PolicyRepository: &mockAttendancePolicyRepo{rules: expected}, Authorizer: mockAttendancePolicyAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/rules", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.GetRules(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d", rec.Code)
	}
	var res AttendanceRules
	if err := json.Unmarshal(rec.Body.Bytes(), &res); err != nil || res.WorkUnitMinutes != 30 {
		t.Fatalf("unexpected response: %s", rec.Body.String())
	}
}

func TestAttendancePolicy_UpdateRules_InvalidDate(t *testing.T) {
	h := AttendancePolicyHandler{PolicyRepository: &mockAttendancePolicyRepo{}, Authorizer: mockAttendancePolicyAuthorizer{allowed: true}}
	body := []byte(`{"valid_from": "not-a-date"}`)
	req := httptest.NewRequest(http.MethodPatch, "/api/v1"+"/attendance/rules", bytes.NewReader(body))

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.UpdateRules(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400 or 422 got %d", rec.Code)
	}
}

func TestAttendancePolicy_UpdateRules_Success(t *testing.T) {
	user := uuid.New()
	h := AttendancePolicyHandler{PolicyRepository: &mockAttendancePolicyRepo{versionResult: AttendancePolicyVersionResponse{ID: uuid.NewString(), WorkUnitMinutes: 15, RoundingPolicy: "round", EarlyArrivalStrategy: "allow", GracePeriodMinutes: 15, ValidFrom: "2026-03-01T00:00:00Z", CreatedAt: "2026-03-01T00:00:00Z"}}, Authorizer: mockAttendancePolicyAuthorizer{allowed: true}}
	body := []byte(`{"work_unit_minutes": 15, "rounding_policy": "round", "early_arrival_strategy": "allow", "valid_from": "2026-03-01"}`)
	req := httptest.NewRequest(http.MethodPatch, "/api/v1"+"/attendance/rules", bytes.NewReader(body))

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))
	rec := httptest.NewRecorder()
	h.UpdateRules(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestAttendancePolicy_ListRuleVersions_Success(t *testing.T) {
	h := AttendancePolicyHandler{PolicyRepository: &mockAttendancePolicyRepo{versions: []AttendancePolicyVersionResponse{{ID: uuid.NewString(), WorkUnitMinutes: 30, RoundingPolicy: "floor", EarlyArrivalStrategy: "block", GracePeriodMinutes: 15, ValidFrom: "2026-01-01T00:00:00Z", CreatedAt: "2026-01-01T00:00:00Z"}}}, Authorizer: mockAttendancePolicyAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/rules/versions", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.ListRuleVersions(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d", rec.Code)
	}
}

func TestAttendancePolicy_Assignments_ListAndUpsert(t *testing.T) {
	empID := uuid.New()
	targetID := empID.String()
	h := AttendancePolicyHandler{
		PolicyRepository: &mockAttendancePolicyRepo{
			assignments:      []PolicyAssignmentResponse{{ID: uuid.NewString(), TargetType: "USER", TargetID: targetID, ValidFrom: "2026-01-01T00:00:00Z", Rules: AttendanceRules{WorkUnitMinutes: 15, RoundingPolicy: "round", EarlyArrivalStrategy: "clamp"}}},
			assignmentResult: PolicyAssignmentResponse{ID: uuid.NewString(), TargetType: "USER", TargetID: targetID, ValidFrom: "2026-01-01T00:00:00Z", Rules: AttendanceRules{WorkUnitMinutes: 15, RoundingPolicy: "round", EarlyArrivalStrategy: "clamp"}},
		},
		Authorizer: mockAttendancePolicyAuthorizer{allowed: true},
	}

	// List
	reqList := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/policies/assignments", nil)

	reqList = reqList.WithContext(httpinfra.WithUserIDForTest(reqList.Context(), uuid.New()))
	recList := httptest.NewRecorder()
	h.ListAssignments(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d", recList.Code)
	}

	// Upsert
	upsertBody := `{"target_type": "USER", "target_id": "` + targetID + `", "work_unit_minutes": 15, "rounding_policy": "round", "early_arrival_strategy": "clamp"}`
	reqUpsert := httptest.NewRequest(http.MethodPost, "/api/v1"+"/attendance/policies/assignments", bytes.NewReader([]byte(upsertBody)))

	reqUpsert = reqUpsert.WithContext(httpinfra.WithUserIDForTest(reqUpsert.Context(), uuid.New()))
	recUpsert := httptest.NewRecorder()
	h.UpsertAssignment(recUpsert, reqUpsert)
	if recUpsert.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d: %s", recUpsert.Code, recUpsert.Body.String())
	}
}

func TestAttendancePolicy_DeleteAssignment_NoContent(t *testing.T) {
	assignID := uuid.New()
	h := AttendancePolicyHandler{PolicyRepository: &mockAttendancePolicyRepo{}, Authorizer: mockAttendancePolicyAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1"+"/attendance/policies/assignments/"+assignID.String(), nil)

	req.SetPathValue("assignment_id", assignID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.DeleteAssignment(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 got %d", rec.Code)
	}
}

func TestAttendancePolicy_BehalfAssignments_CRUD(t *testing.T) {
	assignee := uuid.New()
	targetEmp := uuid.New()
	behalfID := uuid.New()
	h := AttendancePolicyHandler{
		BehalfRepository: &mockBehalfRepo{
			created:     BehalfAssignment{ID: behalfID, AssigneeUserID: assignee, TargetEmployeeID: &targetEmp, AssignmentType: "attendance_adjustment"},
			assignments: []BehalfAssignment{{ID: behalfID, AssigneeUserID: assignee, TargetEmployeeID: &targetEmp, AssignmentType: "attendance_adjustment"}},
		},
		Authorizer: mockAttendancePolicyAuthorizer{allowed: true},
	}

	// Create Dual Target -> 400
	dualBody := `{"assignee_user_id": "` + assignee.String() + `", "target_employee_id": "` + targetEmp.String() + `", "target_department_id": "` + uuid.NewString() + `"}`
	reqDual := httptest.NewRequest(http.MethodPost, "/api/v1"+"/attendance/adjustments/behalf-assignments", bytes.NewReader([]byte(dualBody)))

	reqDual = reqDual.WithContext(httpinfra.WithUserIDForTest(reqDual.Context(), assignee))
	recDual := httptest.NewRecorder()
	h.CreateBehalfAssignment(recDual, reqDual)
	if recDual.Code != http.StatusBadRequest && recDual.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400 or 422 got %d", recDual.Code)
	}

	// Create Valid
	validBody := `{"assignee_user_id": "` + assignee.String() + `", "target_employee_id": "` + targetEmp.String() + `"}`
	reqValid := httptest.NewRequest(http.MethodPost, "/api/v1"+"/attendance/adjustments/behalf-assignments", bytes.NewReader([]byte(validBody)))

	reqValid = reqValid.WithContext(httpinfra.WithUserIDForTest(reqValid.Context(), assignee))
	recValid := httptest.NewRecorder()
	h.CreateBehalfAssignment(recValid, reqValid)
	if recValid.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d: %s", recValid.Code, recValid.Body.String())
	}

	// List
	reqList := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/adjustments/behalf-assignments", nil)

	reqList = reqList.WithContext(httpinfra.WithUserIDForTest(reqList.Context(), assignee))
	recList := httptest.NewRecorder()
	h.ListBehalfAssignments(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d", recList.Code)
	}

	// Delete NotFound
	hNotFound := AttendancePolicyHandler{BehalfRepository: &mockBehalfRepo{err: pgx.ErrNoRows}, Authorizer: mockAttendancePolicyAuthorizer{allowed: true}}
	reqDelNF := httptest.NewRequest(http.MethodDelete, "/api/v1"+"/attendance/adjustments/behalf-assignments/"+uuid.NewString(), nil)

	reqDelNF.SetPathValue("id", uuid.NewString())
	reqDelNF = reqDelNF.WithContext(httpinfra.WithUserIDForTest(reqDelNF.Context(), assignee))
	recDelNF := httptest.NewRecorder()
	hNotFound.DeleteBehalfAssignment(recDelNF, reqDelNF)
	if recDelNF.Code != http.StatusNotFound {
		t.Fatalf("expected 404 got %d", recDelNF.Code)
	}

	// Delete Success
	reqDel := httptest.NewRequest(http.MethodDelete, "/api/v1"+"/attendance/adjustments/behalf-assignments/"+behalfID.String(), nil)

	reqDel.SetPathValue("id", behalfID.String())
	reqDel = reqDel.WithContext(httpinfra.WithUserIDForTest(reqDel.Context(), assignee))
	recDel := httptest.NewRecorder()
	h.DeleteBehalfAssignment(recDel, reqDel)
	if recDel.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d", recDel.Code)
	}
}

func TestAttendancePolicy_MyBehalfTargets_Success(t *testing.T) {
	user := uuid.New()
	targetEmp := uuid.New()
	h := AttendancePolicyHandler{
		BehalfRepository: &mockBehalfRepo{
			targets: []EmployeeRecord{{ID: targetEmp, FullName: "Target Worker", EmployeeIDNumber: "EMP01", CreatedAt: time.Now().UTC().Format(time.RFC3339), UpdatedAt: time.Now().UTC().Format(time.RFC3339)}},
		},
		Authorizer: mockAttendancePolicyAuthorizer{allowed: false}, // No permission required!
	}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/adjustments/behalf-assignments/my-targets", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))
	rec := httptest.NewRecorder()
	h.MyBehalfTargets(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d: %s", rec.Code, rec.Body.String())
	}
}
