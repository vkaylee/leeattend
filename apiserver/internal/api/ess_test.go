package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockESSRepository struct {
	profile          MyProfileResponse
	profileRecord    EmployeeProfileRecord
	contacts         []EmployeeEmergencyContactRecord
	empID            uuid.UUID
	resolveErr       error
	getProfileErr    error
	upsertProfileErr error
	getContactsErr   error
	addContactErr    error
	deleteContactErr error
	addCalls         int
	deleteCalls      int
	upsertCalls      int
}

func (m *mockESSRepository) ResolveEmployeeForUser(context.Context, uuid.UUID, uuid.UUID) (uuid.UUID, error) {
	if m.resolveErr != nil {
		return uuid.Nil, m.resolveErr
	}
	return m.empID, nil
}
func (m *mockESSRepository) GetMyProfile(context.Context, uuid.UUID, uuid.UUID) (MyProfileResponse, error) {
	return m.profile, m.getProfileErr
}
func (m *mockESSRepository) UpsertProfile(context.Context, uuid.UUID, uuid.UUID, UpdateEmployeeProfileRequest) (EmployeeProfileRecord, error) {
	m.upsertCalls++
	return m.profileRecord, m.upsertProfileErr
}
func (m *mockESSRepository) GetEmergencyContacts(context.Context, uuid.UUID, uuid.UUID) ([]EmployeeEmergencyContactRecord, error) {
	return m.contacts, m.getContactsErr
}
func (m *mockESSRepository) AddEmergencyContact(context.Context, uuid.UUID, uuid.UUID, CreateEmergencyContactRequest) (EmployeeEmergencyContactRecord, error) {
	m.addCalls++
	return EmployeeEmergencyContactRecord{ID: uuid.New(), ContactName: "Family", Relationship: "Spouse", Phone: "123456789"}, m.addContactErr
}
func (m *mockESSRepository) DeleteEmergencyContact(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error {
	m.deleteCalls++
	return m.deleteContactErr
}

func essRequest(t *testing.T, method, path, body string, wsID uuid.UUID) *http.Request {
	t.Helper()
	req := httptest.NewRequest(method, path, strings.NewReader(body))
	req.Header.Set("X-Workspace-Id", wsID.String())
	ctx := httpinfra.WithUserIDForTest(req.Context(), uuid.New())
	ctx = httpinfra.WithWorkspaceIDForTest(ctx, wsID)
	return req.WithContext(ctx)
}

func TestESSGetProfileNoLinkedEmployeeForbidden(t *testing.T) {
	wsID := uuid.New()
	repo := &mockESSRepository{resolveErr: ErrESSNoEmployeeRecord}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/profile", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetProfile(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestESSGetProfileSuccess(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{
		empID:   empID,
		profile: MyProfileResponse{Employee: EmployeeRecord{ID: empID, FullName: "John Doe"}},
	}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/profile", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetProfile(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "John Doe") {
		t.Fatalf("profile body missing name: %s", rec.Body.String())
	}
}

func TestESSUpdateProfileValidatesDOB(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodPut, "/api/v1/ess/profile", `{"date_of_birth":"invalid-date"}`, wsID)
	rec := httptest.NewRecorder()
	handler.UpdateProfile(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad DOB, got %d", rec.Code)
	}
	if repo.upsertCalls != 0 {
		t.Fatal("upsert called on validation failure")
	}
}

func TestESSAddContactValidatesFields(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodPost, "/api/v1/ess/emergency-contacts", `{"contact_name":""}`, wsID)
	rec := httptest.NewRecorder()
	handler.AddContact(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing fields, got %d", rec.Code)
	}
	if repo.addCalls != 0 {
		t.Fatal("add contact called on validation failure")
	}
}

func TestESSDeleteContactSuccess(t *testing.T) {
	wsID, empID, contactID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodDelete, "/api/v1/ess/emergency-contacts/"+contactID.String(), "", wsID)
	req.SetPathValue("contact_id", contactID.String())
	rec := httptest.NewRecorder()
	handler.DeleteContact(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
	if repo.deleteCalls != 1 {
		t.Fatalf("expected 1 delete call, got %d", repo.deleteCalls)
	}
}
