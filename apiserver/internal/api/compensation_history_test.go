package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockCompRepo struct {
	response CompensationHistoryResponse
	notFound bool
}

func (m mockCompRepo) GetEmployeeCompensationHistory(_ context.Context, _, _ uuid.UUID, _ *uuid.UUID) (CompensationHistoryResponse, error) {
	if m.notFound {
		return CompensationHistoryResponse{}, ErrEmployeeNotFound
	}
	return m.response, nil
}

func TestCompensationHistory_Get(t *testing.T) {
	wsID, empID, userID := uuid.New(), uuid.New(), uuid.New()
	prev := decimal.NewFromInt(10000000)
	h := CompensationHistoryHandler{
		Repository: mockCompRepo{
			response: CompensationHistoryResponse{
				Timeline: []CompensationTimelineEvent{
					{
						EffectiveDate: "2026-03-01",
						Employment: CompensationEmploymentContext{
							ID:           uuid.New(),
							EmployerName: "Acme Corp",
						},
						TotalCompensation: decimal.NewFromInt(12000000),
						ChangeType:        "adjustment",
						ChangeDetails: CompensationChangeDetails{
							PreviousTotal: &prev,
							Delta:         decimal.NewFromInt(2000000),
						},
					},
				},
			},
		},
		Authorizer: mockAuthorizer{perms: map[string]bool{"employees:read": true}},
	}

	// 1. Success 200
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/compensation-history", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	// 2. Missing employees:read permission -> 403
	hForbidden := CompensationHistoryHandler{
		Repository: mockCompRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{}},
	}
	recForb := httptest.NewRecorder()
	hForbidden.Get(recForb, req)
	if recForb.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", recForb.Code)
	}

	// 3. Invalid employment_id UUID query -> 400
	reqInvalid := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/compensation-history?employment_id=invalid-uuid", nil)
	reqInvalid.SetPathValue("ws_id", wsID.String())
	reqInvalid.SetPathValue("id", empID.String())
	reqInvalid = reqInvalid.WithContext(httpinfra.WithUserIDForTest(reqInvalid.Context(), userID))
	recInvalid := httptest.NewRecorder()
	h.Get(recInvalid, reqInvalid)
	if recInvalid.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad query param, got %d", recInvalid.Code)
	}

	// 4. Empty Timeline when no employments -> 200
	hEmpty := CompensationHistoryHandler{
		Repository: mockCompRepo{response: CompensationHistoryResponse{Timeline: []CompensationTimelineEvent{}}},
		Authorizer: mockAuthorizer{perms: map[string]bool{"employees:read": true}},
	}
	recEmpty := httptest.NewRecorder()
	hEmpty.Get(recEmpty, req)
	if recEmpty.Code != http.StatusOK {
		t.Fatalf("expected 200 for empty timeline, got %d", recEmpty.Code)
	}
}
