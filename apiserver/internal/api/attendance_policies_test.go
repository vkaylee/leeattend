package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAttendancePolicyAuthorizer struct {
	allowed     bool
	permissions map[string]bool
}

func (m mockAttendancePolicyAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	if m.permissions != nil {
		return m.permissions[permission]
	}
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
	reqDual := httptest.NewRequest(http.MethodPost, "/api/v1/attendance/adjustments/behalf-assignments", bytes.NewReader([]byte(dualBody)))
	reqDual = reqDual.WithContext(httpinfra.WithUserIDForTest(reqDual.Context(), assignee))
	recDual := httptest.NewRecorder()
	h.CreateBehalfAssignment(recDual, reqDual)
	if recDual.Code != http.StatusBadRequest && recDual.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400 or 422 got %d", recDual.Code)
	}

	// Create Valid
	validBody := `{"assignee_user_id": "` + assignee.String() + `", "target_employee_id": "` + targetEmp.String() + `"}`
	reqValid := httptest.NewRequest(http.MethodPost, "/api/v1/attendance/adjustments/behalf-assignments", bytes.NewReader([]byte(validBody)))
	reqValid = reqValid.WithContext(httpinfra.WithUserIDForTest(reqValid.Context(), assignee))
	recValid := httptest.NewRecorder()
	h.CreateBehalfAssignment(recValid, reqValid)
	if recValid.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d: %s", recValid.Code, recValid.Body.String())
	}

	// List
	reqList := httptest.NewRequest(http.MethodGet, "/api/v1/attendance/adjustments/behalf-assignments", nil)
	reqList = reqList.WithContext(httpinfra.WithUserIDForTest(reqList.Context(), assignee))
	recList := httptest.NewRecorder()
	h.ListBehalfAssignments(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d", recList.Code)
	}

	// Delete NotFound
	hNotFound := AttendancePolicyHandler{BehalfRepository: &mockBehalfRepo{err: ErrNotFound}, Authorizer: mockAttendancePolicyAuthorizer{allowed: true}}
	reqDelNF := httptest.NewRequest(http.MethodDelete, "/api/v1/attendance/adjustments/behalf-assignments/"+uuid.NewString(), nil)
	reqDelNF.SetPathValue("id", uuid.NewString())
	reqDelNF = reqDelNF.WithContext(httpinfra.WithUserIDForTest(reqDelNF.Context(), assignee))
	recDelNF := httptest.NewRecorder()
	hNotFound.DeleteBehalfAssignment(recDelNF, reqDelNF)
	if recDelNF.Code != http.StatusNotFound {
		t.Fatalf("expected 404 got %d", recDelNF.Code)
	}

	// Delete Success
	reqDel := httptest.NewRequest(http.MethodDelete, "/api/v1/attendance/adjustments/behalf-assignments/"+behalfID.String(), nil)
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
		Authorizer: mockAttendancePolicyAuthorizer{allowed: false},
	}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/attendance/adjustments/behalf-assignments/my-targets", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))
	rec := httptest.NewRecorder()
	h.MyBehalfTargets(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d: %s", rec.Code, rec.Body.String())
	}
}
