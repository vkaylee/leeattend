package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type scheduleEvidenceRepositoryStub struct {
	called          bool
	readiness       ScheduleActivationReadiness
	resolvedID      uuid.UUID
	resolvedStatus  string
	resolvedNote    string
	resolvedActorID uuid.UUID
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

func (s *scheduleEvidenceRepositoryStub) ListScheduleShadowMismatches(context.Context, uuid.UUID, string, string) ([]ScheduleShadowMismatch, error) {
	return nil, nil
}

func (s *scheduleEvidenceRepositoryStub) ResolveScheduleShadowMismatch(_ context.Context, mismatchID uuid.UUID, status, note string, actorID uuid.UUID) error {
	s.called = true
	s.resolvedID, s.resolvedStatus, s.resolvedNote, s.resolvedActorID = mismatchID, status, note, actorID
	return nil
}

func (s *scheduleEvidenceRepositoryStub) GetScheduleActivationReadiness(context.Context, uuid.UUID, string, string) (ScheduleActivationReadiness, error) {
	s.called = true
	return s.readiness, nil
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

func TestActivationReadiness_AuthorizesBeforeValidation(t *testing.T) {
	stub := &scheduleEvidenceRepositoryStub{}
	handler := ScheduleHandler{Shadow: stub, Authorizer: scheduleEvidenceAuthorizerStub{allowed: false}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules/activation-readiness?employer_id=not-a-uuid&start_date=bad&end_date=bad", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.ActivationReadiness(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusForbidden)
	}
	if stub.called {
		t.Fatal("repository called before authorization")
	}
}

func TestActivationReadiness_ValidatesAndCallsRepository(t *testing.T) {
	stub := &scheduleEvidenceRepositoryStub{readiness: ScheduleActivationReadiness{EmployerID: uuid.New(), Ready: true}}
	handler := ScheduleHandler{Shadow: stub, Authorizer: scheduleEvidenceAuthorizerStub{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules/activation-readiness?employer_id="+stub.readiness.EmployerID.String()+"&start_date=2026-11-01&end_date=2026-11-30", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.ActivationReadiness(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d: %s", rec.Code, http.StatusOK, rec.Body.String())
	}
	if !stub.called {
		t.Fatal("expected readiness repository call")
	}
}

func TestActivationReadiness_InvalidDateRangeIsRejected(t *testing.T) {
	stub := &scheduleEvidenceRepositoryStub{}
	handler := ScheduleHandler{Shadow: stub, Authorizer: scheduleEvidenceAuthorizerStub{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules/activation-readiness?employer_id="+uuid.NewString()+"&start_date=2026-11-30&end_date=2026-11-01", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.ActivationReadiness(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusBadRequest)
	}
	if stub.called {
		t.Fatal("repository called with invalid date range")
	}
}

func TestResolveShadowMismatch_ValidatesAndPassesTrimmedInput(t *testing.T) {
	mismatchID, actorID := uuid.New(), uuid.New()
	stub := &scheduleEvidenceRepositoryStub{}
	handler := ScheduleHandler{Shadow: stub, Authorizer: scheduleEvidenceAuthorizerStub{allowed: true}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/shadow-mismatches/"+mismatchID.String()+"/resolve", strings.NewReader(`{"status":"resolved","note":"  approved  "}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	req.SetPathValue("mismatch_id", mismatchID.String())
	rec := httptest.NewRecorder()

	handler.ResolveShadowMismatch(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusNoContent)
	}
	if !stub.called || stub.resolvedID != mismatchID || stub.resolvedStatus != "resolved" || stub.resolvedNote != "approved" || stub.resolvedActorID != actorID {
		t.Fatalf("unexpected resolution call: %+v", stub)
	}
}

func TestResolveShadowMismatch_RejectsInvalidRequest(t *testing.T) {
	stub := &scheduleEvidenceRepositoryStub{}
	handler := ScheduleHandler{Shadow: stub, Authorizer: scheduleEvidenceAuthorizerStub{allowed: true}}
	for _, body := range []string{`{"status":"open","note":"x"}`, `{"status":"resolved","note":"   "}`, `{"status":"resolved"}`} {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/shadow-mismatches/"+uuid.NewString()+"/resolve", strings.NewReader(body))
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
		req.SetPathValue("mismatch_id", uuid.NewString())
		rec := httptest.NewRecorder()
		handler.ResolveShadowMismatch(rec, req)
		if rec.Code != http.StatusBadRequest {
			t.Fatalf("body %s: status = %d, want %d", body, rec.Code, http.StatusBadRequest)
		}
	}
	if stub.called {
		t.Fatal("repository called for invalid resolution")
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
