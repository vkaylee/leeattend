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

type mockScheduleRepository struct {
	called bool
}

func (m *mockScheduleRepository) GetWorkspaceSchedules(_ context.Context, _ uuid.UUID, _, _ time.Time, _ []uuid.UUID, page, limit int) (PaginatedEmployeeSchedules, error) {
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

func (m mockScheduleAuthorizer) IsMember(_ context.Context, _, _ uuid.UUID) (bool, error) {
	return m.isMember, nil
}
func (m mockScheduleAuthorizer) EffectivePermissions(context.Context, uuid.UUID, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func TestScheduleHandler_Unauthenticated(t *testing.T) {
	wsID := uuid.New()
	handler := ScheduleHandler{
		Repository: &mockScheduleRepository{},
		Authorizer: mockScheduleAuthorizer{isMember: true},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/schedules?start_date=2026-10-01&end_date=2026-10-31", nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	rec := httptest.NewRecorder()

	handler.List(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", rec.Code)
	}
}

func TestScheduleHandler_ForbiddenNonMember(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	handler := ScheduleHandler{
		Repository: &mockScheduleRepository{},
		Authorizer: mockScheduleAuthorizer{isMember: false},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/schedules?start_date=2026-10-01&end_date=2026-10-31", nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.List(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestScheduleHandler_ValidationRequiredDates(t *testing.T) {
	wsID := uuid.New()
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
			req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/schedules?"+tc.query, nil)
			req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
			rec := httptest.NewRecorder()

			handler.List(rec, req)
			if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
				t.Fatalf("expected validation error (400 or 422), got %d: %s", rec.Code, rec.Body.String())
			}
		})
	}
}

func TestScheduleHandler_Success(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	repo := &mockScheduleRepository{}
	handler := ScheduleHandler{
		Repository: repo,
		Authorizer: mockScheduleAuthorizer{isMember: true},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/schedules?start_date=2026-10-01&end_date=2026-10-31&page=1&limit=50", nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
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
