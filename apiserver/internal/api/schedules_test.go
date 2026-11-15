package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockScheduleRepository struct {
	called bool
}

type officialScheduleRepositoryMock struct {
	mockScheduleRepository
	officialCalled bool
}

type activationAwareScheduleRepositoryMock struct {
	mockScheduleRepository
	activationAwareCalled bool
}

func (m *activationAwareScheduleRepositoryMock) GetActivationAwareSchedules(_ context.Context, _, _ time.Time, _ []uuid.UUID, page, limit int) (PaginatedEmployeeSchedules, error) {
	m.activationAwareCalled = true
	return PaginatedEmployeeSchedules{Page: page, Limit: limit}, nil
}

func (m *officialScheduleRepositoryMock) GetOfficialSchedules(_ context.Context, _, _ time.Time, _ []uuid.UUID, page, limit int) (PaginatedEmployeeSchedules, error) {
	m.officialCalled = true
	return PaginatedEmployeeSchedules{Data: []EmployeeSchedule{{Schedules: []EffectiveSchedule{{Date: "2026-10-20", RevisionID: new(uuid.UUID), ScheduleStatus: "missing", Provisional: false}}}}, Page: page, Limit: limit}, nil
}

func (m *mockScheduleRepository) GetActivationAwareSchedules(ctx context.Context, start, end time.Time, ids []uuid.UUID, page, limit int) (PaginatedEmployeeSchedules, error) {
	return m.GetOrganizationSchedules(ctx, start, end, ids, page, limit)
}

func (m *mockScheduleRepository) GetOrganizationSchedules(_ context.Context, _, _ time.Time, _ []uuid.UUID, page, limit int) (PaginatedEmployeeSchedules, error) {
	m.called = true
	return PaginatedEmployeeSchedules{
		Data: []EmployeeSchedule{
			{
				EmploymentID:     uuid.New(),
				EmployeeID:       uuid.New(),
				EmployeeIDNumber: "EMP001",
				FullName:         "Test Employee",
				Schedules: []EffectiveSchedule{
					{
						Date:      "2026-10-20",
						ShiftID:   nil,
						Source:    "none",
						Traces:    []ScheduleTrace{},
						IsHoliday: false,
					},
				},
			},
		},
		Total: 1,
		Page:  page,
		Limit: limit,
	}, nil
}

type mockScheduleAuthorizer struct {
	isMember bool
}

type schedulePermissionAuthorizer struct {
	allowed bool
}

func (a schedulePermissionAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return true, nil
}
func (a schedulePermissionAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (a schedulePermissionAuthorizer) HasPermission(context.Context, uuid.UUID, string) bool {
	return a.allowed
}
func (a schedulePermissionAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (a schedulePermissionAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return false }
func (a schedulePermissionAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (a schedulePermissionAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

type activationRepositoryMock struct {
	called bool
	err    error
}

func (m *activationRepositoryMock) ActivateScheduleEmployer(context.Context, uuid.UUID, string, string, string, uuid.UUID) error {
	m.called = true
	return m.err
}
func (m *activationRepositoryMock) DeactivateScheduleEmployer(context.Context, uuid.UUID, string, string, string, uuid.UUID) error {
	m.called = true
	return m.err
}

type lifecycleRepositoryMock struct{}

type lifecycleErrorMock struct{ err error }

func (m lifecycleErrorMock) CreatePreview(context.Context, uuid.UUID, SchedulePreviewRequest) (SchedulePreviewResponse, error) {
	return SchedulePreviewResponse{}, m.err
}
func (lifecycleErrorMock) Publish(context.Context, uuid.UUID, string, SchedulePublishRequest) (SchedulePublicationResponse, error) {
	return SchedulePublicationResponse{}, ErrScheduleLifecycleUnavailable
}
func (lifecycleErrorMock) PreviewReplan(context.Context, uuid.UUID, uuid.UUID, string, ScheduleReplanPreviewRequest) (ScheduleReplanPreviewResponse, error) {
	return ScheduleReplanPreviewResponse{}, ErrScheduleLifecycleUnavailable
}
func (lifecycleErrorMock) CommitReplan(context.Context, uuid.UUID, uuid.UUID, string, string, ScheduleReplanRequest) (ScheduleReplanResponse, error) {
	return ScheduleReplanResponse{}, ErrScheduleLifecycleUnavailable
}

func (lifecycleRepositoryMock) CreatePreview(context.Context, uuid.UUID, SchedulePreviewRequest) (SchedulePreviewResponse, error) {
	return SchedulePreviewResponse{}, ErrScheduleLifecycleUnavailable
}
func (lifecycleRepositoryMock) Publish(context.Context, uuid.UUID, string, SchedulePublishRequest) (SchedulePublicationResponse, error) {
	return SchedulePublicationResponse{}, ErrScheduleLifecycleUnavailable
}
func (lifecycleRepositoryMock) PreviewReplan(context.Context, uuid.UUID, uuid.UUID, string, ScheduleReplanPreviewRequest) (ScheduleReplanPreviewResponse, error) {
	return ScheduleReplanPreviewResponse{}, ErrScheduleLifecycleUnavailable
}
func (lifecycleRepositoryMock) CommitReplan(context.Context, uuid.UUID, uuid.UUID, string, string, ScheduleReplanRequest) (ScheduleReplanResponse, error) {
	return ScheduleReplanResponse{}, ErrScheduleLifecycleUnavailable
}

func (m mockScheduleAuthorizer) IsMember(_ context.Context, _ uuid.UUID) (bool, error) {
	return m.isMember, nil
}
func (m mockScheduleAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (m mockScheduleAuthorizer) HasPermission(context.Context, uuid.UUID, string) bool {
	return true
}
func (m mockScheduleAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (m mockScheduleAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool {
	return false
}
func (m mockScheduleAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (m mockScheduleAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func TestScheduleHandler_ActivationRequiresPermissionBeforeValidation(t *testing.T) {
	activation := &activationRepositoryMock{}
	handler := ScheduleHandler{Activation: activation, Authorizer: schedulePermissionAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/activation/not-a-uuid", strings.NewReader(`{"start_date":"bad"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Activate(rec, req)
	if rec.Code != http.StatusForbidden || activation.called {
		t.Fatalf("expected early authorization denial, status=%d called=%v", rec.Code, activation.called)
	}
}

func TestScheduleHandler_ActivationRejectsInvalidRange(t *testing.T) {
	activation := &activationRepositoryMock{}
	handler := ScheduleHandler{Activation: activation, Authorizer: schedulePermissionAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/activation/"+uuid.NewString(), strings.NewReader(`{"start_date":"2026-11-30","end_date":"2026-11-01","reason":"ready"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	req.SetPathValue("employer_id", uuid.NewString())
	rec := httptest.NewRecorder()

	handler.Activate(rec, req)
	if rec.Code != http.StatusBadRequest || activation.called {
		t.Fatalf("expected range validation, status=%d called=%v", rec.Code, activation.called)
	}
}

func TestScheduleHandler_DeactivationRejectsInvalidRange(t *testing.T) {
	activation := &activationRepositoryMock{}
	handler := ScheduleHandler{Activation: activation, Authorizer: schedulePermissionAuthorizer{allowed: true}}
	employerID := uuid.New()
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/deactivation/"+employerID.String(), strings.NewReader(`{"start_date":"2026-11-30","end_date":"2026-11-01","reason":"rollback"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	req.SetPathValue("employer_id", employerID.String())
	rec := httptest.NewRecorder()
	handler.Deactivate(rec, req)
	if rec.Code != http.StatusBadRequest || activation.called {
		t.Fatalf("expected deactivation range validation, status=%d called=%v", rec.Code, activation.called)
	}
}

func TestScheduleHandler_DeactivationPassesReasonAndReturnsNoContent(t *testing.T) {
	activation := &activationRepositoryMock{}
	handler := ScheduleHandler{Activation: activation, Authorizer: schedulePermissionAuthorizer{allowed: true}}
	employerID := uuid.New()
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/deactivation/"+employerID.String(), strings.NewReader(`{"start_date":"2026-11-01","end_date":"2026-11-30","reason":"rollback"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	req.SetPathValue("employer_id", employerID.String())
	rec := httptest.NewRecorder()
	handler.Deactivate(rec, req)
	if rec.Code != http.StatusNoContent || !activation.called {
		t.Fatalf("expected deactivation call, status=%d called=%v", rec.Code, activation.called)
	}
}

func TestScheduleHandler_ActivationPassesReasonAndReturnsNoContent(t *testing.T) {
	activation := &activationRepositoryMock{}
	handler := ScheduleHandler{Activation: activation, Authorizer: schedulePermissionAuthorizer{allowed: true}}
	employerID := uuid.New()
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/activation/"+employerID.String(), strings.NewReader(`{"start_date":"2026-11-01","end_date":"2026-11-30","reason":"clean evidence"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	req.SetPathValue("employer_id", employerID.String())
	rec := httptest.NewRecorder()

	handler.Activate(rec, req)
	if rec.Code != http.StatusNoContent || !activation.called {
		t.Fatalf("expected activation call, status=%d called=%v", rec.Code, activation.called)
	}
}

func TestScheduleHandler_Unauthenticated(t *testing.T) {
	handler := ScheduleHandler{
		Repository: &mockScheduleRepository{},
		Authorizer: mockScheduleAuthorizer{isMember: true},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/schedules?start_date=2026-10-01&end_date=2026-10-31", nil)

	rec := httptest.NewRecorder()

	handler.List(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", rec.Code)
	}
}

func TestScheduleHandler_ForbiddenNonMember(t *testing.T) {
	userID := uuid.New()
	handler := ScheduleHandler{
		Repository: &mockScheduleRepository{},
		Authorizer: mockScheduleAuthorizer{isMember: false},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/schedules?start_date=2026-10-01&end_date=2026-10-31", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.List(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestScheduleHandler_ValidationRequiredDates(t *testing.T) {
	userID := uuid.New()
	handler := ScheduleHandler{
		Repository: &mockScheduleRepository{},
		Authorizer: mockScheduleAuthorizer{isMember: true},
	}

	testCases := []struct {
		name  string
		query string
	}{
		{"missing both", ""},
		{"missing start", "end_date=2026-10-31"},
		{"missing end", "start_date=2026-10-01"},
		{"invalid start format", "start_date=invalid&end_date=2026-10-31"},
		{"invalid end format", "start_date=2026-10-01&end_date=bad"},
		{"end before start", "start_date=2026-10-31&end_date=2026-10-01"},
		{"invalid employment id", "start_date=2026-10-01&end_date=2026-10-31&employment_ids=not-a-uuid"},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/schedules?"+tc.query, nil)

			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
			rec := httptest.NewRecorder()

			handler.List(rec, req)
			if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
				t.Fatalf("expected validation error (400 or 422), got %d: %s", rec.Code, rec.Body.String())
			}
		})
	}
}

func TestScheduleHandler_LifecycleAuthRunsBeforeValidation(t *testing.T) {
	userID := uuid.New()
	handler := ScheduleHandler{
		Lifecycle:  lifecycleRepositoryMock{},
		Authorizer: schedulePermissionAuthorizer{allowed: false},
	}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/preview", strings.NewReader(`{"unknown":true}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.Preview(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected auth denial before payload validation, got %d", rec.Code)
	}
}

func TestScheduleHandler_PublishRequiresIdempotencyKey(t *testing.T) {
	userID := uuid.New()
	handler := ScheduleHandler{
		Lifecycle:  lifecycleRepositoryMock{},
		Authorizer: schedulePermissionAuthorizer{allowed: true},
	}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/publish", strings.NewReader(`{}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.Publish(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected missing idempotency key validation, got %d", rec.Code)
	}
}

func TestScheduleHandler_LifecycleConflictErrorsReturn409(t *testing.T) {
	handler := ScheduleHandler{}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules", nil)
	for _, lifecycleErr := range []error{ErrSchedulePreviewStale, ErrScheduleRevisionStale, ErrScheduleAlreadyPublished, ErrScheduleLockedPeriod, ErrScheduleNoCurrentRevision, ErrScheduleIdempotencyConflict} {
		rec := httptest.NewRecorder()
		handler.writeLifecycleError(rec, req, lifecycleErr)
		if rec.Code != http.StatusConflict {
			t.Fatalf("error %v status=%d want 409", lifecycleErr, rec.Code)
		}
	}
}

func TestScheduleHandler_LifecycleUnavailableIsExplicit(t *testing.T) {
	userID := uuid.New()
	handler := ScheduleHandler{
		Lifecycle:  lifecycleRepositoryMock{},
		Authorizer: schedulePermissionAuthorizer{allowed: true},
	}
	request := `{"employment_ids":["` + uuid.NewString() + `"],"start_date":"2026-11-16","end_date":"2026-11-16","timezone":"UTC","mode":"office","expected_revisions":{"` + uuid.NewString() + `:2026-11-16":0}}`
	// Keep request structurally valid enough to reach repository capability check.
	var payload map[string]any
	if err := json.Unmarshal([]byte(request), &payload); err != nil {
		t.Fatal(err)
	}
	employmentID := payload["employment_ids"].([]any)[0].(string)
	payload["expected_revisions"] = map[string]int64{employmentID + ":2026-11-16": 0}
	body, err := json.Marshal(payload)
	if err != nil {
		t.Fatal(err)
	}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/schedules/preview", bytes.NewReader(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.Preview(rec, req)
	if rec.Code != http.StatusServiceUnavailable {
		t.Fatalf("expected explicit lifecycle unavailability, got %d", rec.Code)
	}
}

func TestScheduleHandler_ActivationAwareViewUsesActivationCapability(t *testing.T) {
	repo := &activationAwareScheduleRepositoryMock{}
	handler := ScheduleHandler{Repository: repo, Authorizer: mockScheduleAuthorizer{isMember: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules?start_date=2026-10-01&end_date=2026-10-31&view=activation-aware", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusOK || !repo.activationAwareCalled || repo.called {
		t.Fatalf("activation-aware view status=%d activation=%v legacy=%v", rec.Code, repo.activationAwareCalled, repo.called)
	}
}

func TestScheduleHandler_OfficialViewUsesOfficialCapability(t *testing.T) {
	repo := &officialScheduleRepositoryMock{}
	handler := ScheduleHandler{Repository: repo, Authorizer: mockScheduleAuthorizer{isMember: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules?start_date=2026-10-01&end_date=2026-10-31&view=official", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusOK || !repo.officialCalled || repo.called {
		t.Fatalf("official view status=%d official=%v legacy=%v", rec.Code, repo.officialCalled, repo.called)
	}
}

func TestScheduleHandler_OfficialViewUnavailableFailsClosed(t *testing.T) {
	repo := &mockScheduleRepository{}
	handler := ScheduleHandler{Repository: repo, Authorizer: mockScheduleAuthorizer{isMember: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules?start_date=2026-10-01&end_date=2026-10-31&view=official", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusServiceUnavailable || repo.called {
		t.Fatalf("official capability absence status=%d legacy=%v", rec.Code, repo.called)
	}
}

func TestScheduleHandler_UnknownViewRejected(t *testing.T) {
	repo := &mockScheduleRepository{}
	handler := ScheduleHandler{Repository: repo, Authorizer: mockScheduleAuthorizer{isMember: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules?start_date=2026-10-01&end_date=2026-10-31&view=unknown", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusBadRequest || repo.called {
		t.Fatalf("unknown view status=%d legacy=%v", rec.Code, repo.called)
	}
}

func TestScheduleHandler_DefaultViewUsesActivationAwareCapability(t *testing.T) {
	repo := &activationAwareScheduleRepositoryMock{}
	handler := ScheduleHandler{Repository: repo, Authorizer: mockScheduleAuthorizer{isMember: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/schedules?start_date=2026-10-01&end_date=2026-10-31", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusOK || !repo.activationAwareCalled || repo.called {
		t.Fatalf("default view status=%d activation=%v legacy=%v", rec.Code, repo.activationAwareCalled, repo.called)
	}
}

func TestScheduleHandler_Success(t *testing.T) {
	userID := uuid.New()
	repo := &mockScheduleRepository{}
	handler := ScheduleHandler{
		Repository: repo,
		Authorizer: mockScheduleAuthorizer{isMember: true},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/schedules?start_date=2026-10-01&end_date=2026-10-31&page=1&limit=50", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.List(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !repo.called {
		t.Fatal("expected repository to be called")
	}
}
