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

func (m mockCompRepo) GetEmployeeCompensationHistory(_ context.Context, _ uuid.UUID, _ *uuid.UUID) (CompensationHistoryResponse, error) {
	if m.notFound {
		return CompensationHistoryResponse{}, ErrEmployeeNotFound
	}
	return m.response, nil
}

type compensationHistoryAuthorizer struct{ allowed bool }

func (a compensationHistoryAuthorizer) HasPermission(context.Context, uuid.UUID, string) bool {
	return a.allowed
}
func (compensationHistoryAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (compensationHistoryAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return true, nil
}
func (compensationHistoryAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (compensationHistoryAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}
func (compensationHistoryAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (compensationHistoryAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return false }

func compensationRequest(target string, userID, employeeID uuid.UUID) *http.Request {
	req := httptest.NewRequest(http.MethodGet, target, nil)
	req.SetPathValue("id", employeeID.String())
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestCompensationHistoryGetRequiresEmployeePermission(t *testing.T) {
	employeeID, userID := uuid.New(), uuid.New()
	previous := decimal.NewFromInt(10000000)
	handler := CompensationHistoryHandler{
		Repository: mockCompRepo{response: CompensationHistoryResponse{Timeline: []CompensationTimelineEvent{{
			EffectiveDate:     "2026-03-01",
			Employment:        CompensationEmploymentContext{ID: uuid.New(), EmployerName: "Acme Corp"},
			TotalCompensation: decimal.NewFromInt(12000000), ChangeType: "adjustment",
			ChangeDetails: CompensationChangeDetails{PreviousTotal: &previous, Delta: decimal.NewFromInt(2000000)},
		}}}},
		Authorizer: compensationHistoryAuthorizer{allowed: true},
	}
	req := compensationRequest("/api/v1/employees/"+employeeID.String()+"/compensation-history", userID, employeeID)
	rec := httptest.NewRecorder()
	handler.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	forbidden := CompensationHistoryHandler{Repository: mockCompRepo{}, Authorizer: compensationHistoryAuthorizer{}}
	forbiddenRec := httptest.NewRecorder()
	forbidden.Get(forbiddenRec, req)
	if forbiddenRec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without employee read permission, got %d", forbiddenRec.Code)
	}
}

func TestCompensationHistoryRejectsInvalidEmploymentFilter(t *testing.T) {
	employeeID, userID := uuid.New(), uuid.New()
	handler := CompensationHistoryHandler{Repository: mockCompRepo{}, Authorizer: compensationHistoryAuthorizer{allowed: true}}
	req := compensationRequest("/api/v1/employees/"+employeeID.String()+"/compensation-history?employment_id=invalid-uuid", userID, employeeID)
	rec := httptest.NewRecorder()
	handler.Get(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected invalid employment filter rejection, got %d", rec.Code)
	}
}

func TestCompensationHistoryReturnsEmptyTimeline(t *testing.T) {
	employeeID, userID := uuid.New(), uuid.New()
	handler := CompensationHistoryHandler{
		Repository: mockCompRepo{response: CompensationHistoryResponse{Timeline: []CompensationTimelineEvent{}}},
		Authorizer: compensationHistoryAuthorizer{allowed: true},
	}
	req := compensationRequest("/api/v1/employees/"+employeeID.String()+"/compensation-history", userID, employeeID)
	rec := httptest.NewRecorder()
	handler.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected empty timeline 200, got %d", rec.Code)
	}
}
