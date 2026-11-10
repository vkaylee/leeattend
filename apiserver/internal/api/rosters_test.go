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

type mockRosterRepository struct{}

func (mockRosterRepository) ResolveActiveEmployment(context.Context, uuid.UUID) (uuid.UUID, error) {
	return uuid.New(), nil
}
func (mockRosterRepository) ListRosters(context.Context, uuid.UUID, string, string, int, int) (PaginatedShiftRosters, error) {
	return PaginatedShiftRosters{Data: []ShiftRoster{}, Page: 1, Limit: 50}, nil
}
func (mockRosterRepository) UpsertRoster(_ context.Context, req CreateRosterRequest) (ShiftRoster, error) {
	return ShiftRoster{ID: uuid.New(), EmploymentID: req.EmploymentID, Date: req.Date, Status: "confirmed"}, nil
}
func (mockRosterRepository) BulkUpsertRoster(context.Context, uuid.UUID, []RosterItem) error {
	return nil
}
func (mockRosterRepository) DeleteRoster(context.Context, uuid.UUID, string) error {
	return nil
}

func TestRosterHandler_BulkRejectsDuplicateDates(t *testing.T) {
	employeeID := uuid.New()
	handler := RosterHandler{Repository: mockRosterRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"items":[{"date":"2026-10-20"},{"date":"2026-10-20"}]}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/employees/"+employeeID.String()+"/rosters/bulk", bytes.NewReader(body))

	req.SetPathValue("id", employeeID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.BulkUpsert(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected validation status, got %d", rec.Code)
	}
}

func TestRosterHandler_BulkReturnsAccepted(t *testing.T) {
	employeeID := uuid.New()
	handler := RosterHandler{Repository: mockRosterRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"items":[{"date":"2026-10-20"}]}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/employees/"+employeeID.String()+"/rosters/bulk", bytes.NewReader(body))

	req.SetPathValue("id", employeeID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.BulkUpsert(rec, req)
	if rec.Code != http.StatusAccepted {
		t.Fatalf("expected 202, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestRosterHandler_UpsertReturnsConfirmedRoster(t *testing.T) {
	employeeID := uuid.New()
	handler := RosterHandler{Repository: mockRosterRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"date":"2026-10-20"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/employees/"+employeeID.String()+"/rosters", bytes.NewReader(body))

	req.SetPathValue("id", employeeID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Upsert(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}
