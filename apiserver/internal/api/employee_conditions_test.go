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

type mockConditionRepo struct {
	conditions []EmployeeCondition
	notFound   bool
	noActive   bool
	deptID     *uuid.UUID
}

func (m mockConditionRepo) ListWorkspaceConditions(_ context.Context, _ uuid.UUID) ([]EmployeeConditionWithEmployee, error) {
	if m.notFound {
		return nil, ErrEmployeeNotFound
	}
	res := make([]EmployeeConditionWithEmployee, len(m.conditions))
	for i, c := range m.conditions {
		res[i] = EmployeeConditionWithEmployee{
			EmployeeCondition: c,
			EmployeeID:        uuid.New(),
			EmployeeName:      "Test Employee",
			EmployeeCode:      "EMP-001",
		}
	}
	return res, nil
}

func (m mockConditionRepo) ListEmployeeConditions(_ context.Context, _, _ uuid.UUID, page, limit int) (EmployeeConditionListResponse, error) {
	if m.notFound {
		return EmployeeConditionListResponse{}, ErrEmployeeNotFound
	}
	if m.noActive {
		return EmployeeConditionListResponse{}, ErrNoActiveEmployment
	}
	return EmployeeConditionListResponse{
		Data:  m.conditions,
		Total: int64(len(m.conditions)),
		Page:  page,
		Limit: limit,
	}, nil
}

func (m mockConditionRepo) CreateEmployeeCondition(_ context.Context, _, empID uuid.UUID, req CreateEmployeeConditionRequest) (EmployeeCondition, error) {
	if m.notFound {
		return EmployeeCondition{}, ErrEmployeeNotFound
	}
	if m.noActive {
		return EmployeeCondition{}, ErrNoActiveEmployment
	}
	return EmployeeCondition{
		ID:              uuid.New(),
		EmploymentID:    uuid.New(),
		Type:            req.Type,
		ConditionTypeID: req.ConditionTypeID,
		MinuteAllowance: req.MinuteAllowance,
		IsPaid:          req.IsPaid,
		StartDate:       req.StartDate,
		EndDate:         req.EndDate,
	}, nil
}

func (m mockConditionRepo) DeleteEmployeeCondition(_ context.Context, _, _, _ uuid.UUID) error {
	if m.notFound {
		return ErrEmployeeConditionNotFound
	}
	if m.noActive {
		return ErrNoActiveEmployment
	}
	return nil
}

func (m mockConditionRepo) CurrentEmployeeDepartment(_ context.Context, _, _ uuid.UUID) (*uuid.UUID, error) {
	if m.notFound {
		return nil, ErrEmployeeNotFound
	}
	return m.deptID, nil
}

func TestEmployeeConditions_List(t *testing.T) {
	wsID, empID, userID := uuid.New(), uuid.New(), uuid.New()
	condID := uuid.New()
	h := EmployeeConditionHandler{
		Repository: mockConditionRepo{
			conditions: []EmployeeCondition{
				{ID: condID, Type: "EARLY_LEAVE", MinuteAllowance: 60, IsPaid: true, StartDate: "2026-01-01"},
			},
		},
	}

	// 1. Success 200
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/conditions", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.List(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	// 2. No active employment -> 400
	hNoActive := EmployeeConditionHandler{Repository: mockConditionRepo{noActive: true}}
	recNoActive := httptest.NewRecorder()
	hNoActive.List(recNoActive, req)
	if recNoActive.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for no active employment, got %d", recNoActive.Code)
	}

	// 3. Employee not found -> 404
	hNotFound := EmployeeConditionHandler{Repository: mockConditionRepo{notFound: true}}
	recNotFound := httptest.NewRecorder()
	hNotFound.List(recNotFound, req)
	if recNotFound.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", recNotFound.Code)
	}
}

func TestEmployeeConditions_CreateAndDelete(t *testing.T) {
	wsID, empID, condID, userID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	deptID := uuid.New()
	h := EmployeeConditionHandler{
		Repository: mockConditionRepo{deptID: &deptID},
		Authorizer: mockAuthorizer{perms: map[string]bool{"employees:update": true}},
	}

	// 1. Create Success -> 201
	body := `{"type_":"LATE_IN","minute_allowance":30,"is_paid":false,"start_date":"2026-03-01","end_date":"2026-12-31"}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/conditions", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	// 2. Create Forbidden
	hForbidden := EmployeeConditionHandler{
		Repository: mockConditionRepo{deptID: &deptID},
		Authorizer: mockAuthorizer{perms: map[string]bool{}},
	}
	recForbidden := httptest.NewRecorder()
	reqForb := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/conditions", bytes.NewBufferString(body))
	reqForb.SetPathValue("ws_id", wsID.String())
	reqForb.SetPathValue("id", empID.String())
	reqForb = reqForb.WithContext(httpinfra.WithUserIDForTest(reqForb.Context(), userID))
	hForbidden.Create(recForbidden, reqForb)
	if recForbidden.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", recForbidden.Code)
	}

	// 3. Create Invalid Date Validation -> 400
	badBody := `{"type_":"LATE_IN","minute_allowance":30,"is_paid":false,"start_date":"2026-12-31","end_date":"2026-01-01"}`
	reqBad := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/conditions", bytes.NewBufferString(badBody))
	reqBad.SetPathValue("ws_id", wsID.String())
	reqBad.SetPathValue("id", empID.String())
	reqBad = reqBad.WithContext(httpinfra.WithUserIDForTest(reqBad.Context(), userID))
	recBad := httptest.NewRecorder()
	h.Create(recBad, reqBad)
	if recBad.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", recBad.Code)
	}

	// 4. Delete Success -> 204
	delReq := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/conditions/"+condID.String(), nil)
	delReq.SetPathValue("ws_id", wsID.String())
	delReq.SetPathValue("id", empID.String())
	delReq.SetPathValue("cond_id", condID.String())
	delReq = delReq.WithContext(httpinfra.WithUserIDForTest(delReq.Context(), userID))
	recDel := httptest.NewRecorder()
	h.Delete(recDel, delReq)
	if recDel.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", recDel.Code)
	}

	// 5. Delete Not Found -> 404
	hDelNotFound := EmployeeConditionHandler{
		Repository: mockConditionRepo{notFound: true, deptID: &deptID},
		Authorizer: mockAuthorizer{perms: map[string]bool{"employees:update": true}},
	}
	recDelNotFound := httptest.NewRecorder()
	hDelNotFound.Delete(recDelNotFound, delReq)
	if recDelNotFound.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", recDelNotFound.Code)
	}
}

func TestEmployeeConditions_ListWorkspaceConditions(t *testing.T) {
	wsID, userID := uuid.New(), uuid.New()
	cond := EmployeeCondition{
		ID:              uuid.New(),
		EmploymentID:    uuid.New(),
		Type:            "maternity",
		MinuteAllowance: 60,
		IsPaid:          true,
		StartDate:       "2026-01-01",
	}
	h := EmployeeConditionHandler{
		Repository: mockConditionRepo{conditions: []EmployeeCondition{cond}},
		Authorizer: mockAuthorizer{perms: map[string]bool{"employees:read": true}},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/working-conditions", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.ListWorkspaceConditions(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}
