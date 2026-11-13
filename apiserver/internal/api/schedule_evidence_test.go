package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type scheduleEvidenceRepositoryStub struct {
	called bool
}

func (s *scheduleEvidenceRepositoryStub) GetOrganizationSchedules(context.Context, time.Time, time.Time, []uuid.UUID, int, int) (PaginatedEmployeeSchedules, error) {
	return PaginatedEmployeeSchedules{}, nil
}

func (s *scheduleEvidenceRepositoryStub) AuthorizeScheduleHistory(context.Context, uuid.UUID, uuid.UUID) (bool, error) {
	return true, nil
}

func (s *scheduleEvidenceRepositoryStub) AuthorizeScheduleEmployment(context.Context, uuid.UUID, uuid.UUID, string) (bool, error) {
	return true, nil
}

func (s *scheduleEvidenceRepositoryStub) ListScheduleHistory(context.Context, ScheduleHistoryQuery) (ScheduleHistoryResponse, error) {
	s.called = true
	return ScheduleHistoryResponse{Data: []ScheduleHistoryEntry{}}, nil
}

func (s *scheduleEvidenceRepositoryStub) GetScheduleEvidence(context.Context, uuid.UUID, uuid.UUID, string) (ScheduleEvidenceResponse, error) {
	s.called = true
	return ScheduleEvidenceResponse{}, nil
}

type scheduleEvidenceAuthorizerStub struct {
	allowed bool
}

func (s scheduleEvidenceAuthorizerStub) IsMember(context.Context, uuid.UUID) (bool, error) {
	return true, nil
}
func (s scheduleEvidenceAuthorizerStub) HasPermission(_ context.Context, _ uuid.UUID, _ string) bool {
	return s.allowed
}
func (s scheduleEvidenceAuthorizerStub) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (s scheduleEvidenceAuthorizerStub) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (s scheduleEvidenceAuthorizerStub) IsGlobalManager(context.Context, uuid.UUID) bool {
	return false
}
func (s scheduleEvidenceAuthorizerStub) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (s scheduleEvidenceAuthorizerStub) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func TestScheduleHistory_AuthorizesBeforeParsing(t *testing.T) {
	stub := &scheduleEvidenceRepositoryStub{}
	handler := ScheduleHandler{Repository: stub, Authorizer: scheduleEvidenceAuthorizerStub{allowed: false}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules/history?employment_id=not-a-uuid", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.History(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusForbidden)
	}
	if stub.called {
		t.Fatal("repository called before authorization")
	}
}

func TestScheduleEvidence_AuthorizesBeforeParsing(t *testing.T) {
	stub := &scheduleEvidenceRepositoryStub{}
	handler := ScheduleHandler{Repository: stub, Authorizer: scheduleEvidenceAuthorizerStub{allowed: false}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules/evidence/not-a-uuid/not-a-date", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Evidence(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusForbidden)
	}
	if stub.called {
		t.Fatal("repository called before authorization")
	}
}

func TestScheduleHistory_InvalidCursorIsRejected(t *testing.T) {
	stub := &scheduleEvidenceRepositoryStub{}
	handler := ScheduleHandler{Repository: stub, Authorizer: scheduleEvidenceAuthorizerStub{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules/history?employment_id="+uuid.NewString()+"&cursor=invalid", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.History(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusBadRequest)
	}
	if stub.called {
		t.Fatal("repository called with invalid cursor")
	}
}
