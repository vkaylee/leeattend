package api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockSystemAdminMock struct {
	userRoles           map[uuid.UUID][]string
	settings            map[string]json.RawMessage
	countPlatformAdmins int64
}

func newMockSystemAdmin() *mockSystemAdminMock {
	return &mockSystemAdminMock{
		userRoles:           make(map[uuid.UUID][]string),
		settings:            make(map[string]json.RawMessage),
		countPlatformAdmins: 2,
	}
}

func (m *mockSystemAdminMock) ListSystemRoles(context.Context) ([]SystemRoleInfo, error) {
	return []SystemRoleInfo{}, nil
}

func (m *mockSystemAdminMock) GetUserSystemRoles(_ context.Context, userID uuid.UUID) ([]string, error) {
	return m.userRoles[userID], nil
}

func (m *mockSystemAdminMock) AssignSystemRole(_ context.Context, userID, _ uuid.UUID, role SystemRoleType) error {
	m.userRoles[userID] = append(m.userRoles[userID], string(role))
	return nil
}

func (m *mockSystemAdminMock) RevokeSystemRole(_ context.Context, userID uuid.UUID, role string) error {
	if (role == "platform_admin" || role == "admin") && m.countPlatformAdmins <= 1 {
		return ErrLastPlatformAdmin
	}
	roles := m.userRoles[userID]
	for index, existing := range roles {
		if existing == role {
			m.userRoles[userID] = append(roles[:index], roles[index+1:]...)
			if role == "platform_admin" || role == "admin" {
				m.countPlatformAdmins--
			}
			return nil
		}
	}
	return ErrSystemRoleNotFound
}

func (m *mockSystemAdminMock) Get(_ context.Context, key string) (json.RawMessage, bool, error) {
	value, ok := m.settings[key]
	return value, ok, nil
}

func (m *mockSystemAdminMock) Set(_ context.Context, key string, value json.RawMessage) error {
	m.settings[key] = value
	return nil
}

type mockEmailService struct{ fail bool }

func (e mockEmailService) SendTestEmail(_ context.Context, _ string) (string, error) {
	if e.fail {
		return "", errors.New("connection to smtp server failed")
	}
	return "test email sent", nil
}

func adminRequest(method, path, body string, adminID uuid.UUID) *http.Request {
	var request *http.Request
	if body == "" {
		request = httptest.NewRequest(method, path, nil)
	} else {
		request = httptest.NewRequest(method, path, bytes.NewBufferString(body))
	}
	if adminID != uuid.Nil {
		request = request.WithContext(httpinfra.WithUserIDForTest(request.Context(), adminID))
	}
	return request
}

func setupTestHandler() (SystemAdminHandler, *mockSystemAdminMock, uuid.UUID) {
	adminID := uuid.New()
	mock := newMockSystemAdmin()
	mock.userRoles[adminID] = []string{"platform_admin"}
	return SystemAdminHandler{
		RolesRepo:   mock,
		Settings:    mock,
		Roles:       systemHealthRepoMock{roles: []string{"platform_admin"}},
		EmailSender: mockEmailService{},
	}, mock, adminID
}

func TestSystemAdminPlatformAdminAuth(t *testing.T) {
	handler, _, _ := setupTestHandler()
	record := httptest.NewRecorder()
	handler.GetSMTPConfig(record, adminRequest(http.MethodGet, "/api/v1/system/smtp/config", "", uuid.Nil))
	if record.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for unauthenticated request, got %d", record.Code)
	}

	record = httptest.NewRecorder()
	support := handler
	support.Roles = systemHealthRepoMock{roles: []string{"support"}}
	support.GetSMTPConfig(record, adminRequest(http.MethodGet, "/api/v1/system/smtp/config", "", uuid.New()))
	if record.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for support role, got %d", record.Code)
	}
}

func TestSystemAdminSMTPConfigAndTest(t *testing.T) {
	handler, mock, adminID := setupTestHandler()
	record := httptest.NewRecorder()
	handler.SetSMTPConfig(record, adminRequest(http.MethodPut, "/api/v1/system/smtp/config", `{"host":"smtp.example.com"}`, adminID))
	if record.Code != http.StatusOK {
		t.Fatalf("expected 200 when saving SMTP config, got %d", record.Code)
	}
	if string(mock.settings["smtp"]) != `{"host":"smtp.example.com"}` {
		t.Fatalf("SMTP config was not persisted")
	}

	record = httptest.NewRecorder()
	failed := handler
	failed.EmailSender = mockEmailService{fail: true}
	failed.TestSMTP(record, adminRequest(http.MethodPost, "/api/v1/system/smtp/test", `{"to_email":"admin@example.com"}`, adminID))
	if record.Code != http.StatusOK || !strings.Contains(record.Body.String(), `"success":false`) {
		t.Fatalf("expected SMTP failure response, got %d: %s", record.Code, record.Body.String())
	}
}

func TestSystemAdminRoles(t *testing.T) {
	handler, mock, adminID := setupTestHandler()
	targetUser := uuid.New()
	mock.userRoles[targetUser] = []string{"support"}
	request := adminRequest(http.MethodPost, "/api/v1/system/users/"+targetUser.String()+"/roles", `{"role_name":"billing_admin"}`, adminID)
	request.SetPathValue("user_id", targetUser.String())
	record := httptest.NewRecorder()
	handler.AssignRole(record, request)
	if record.Code != http.StatusNoContent {
		t.Fatalf("expected 204 when assigning role, got %d", record.Code)
	}

	mock.countPlatformAdmins = 1
	mock.userRoles[targetUser] = []string{"platform_admin"}
	request = adminRequest(http.MethodDelete, "/api/v1/system/users/"+targetUser.String()+"/roles/platform_admin", "", adminID)
	request.SetPathValue("user_id", targetUser.String())
	request.SetPathValue("role_name", "platform_admin")
	record = httptest.NewRecorder()
	handler.RevokeRole(record, request)
	if record.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 when revoking last platform admin, got %d", record.Code)
	}

	// Test assigning and revoking unified admin role
	request = adminRequest(http.MethodPost, "/api/v1/system/users/"+targetUser.String()+"/roles", `{"role_name":"admin"}`, adminID)
	request.SetPathValue("user_id", targetUser.String())
	record = httptest.NewRecorder()
	handler.AssignRole(record, request)
	if record.Code != http.StatusNoContent {
		t.Fatalf("expected 204 when assigning admin role, got %d", record.Code)
	}

	mock.countPlatformAdmins = 1
	mock.userRoles[targetUser] = []string{"admin"}
	request = adminRequest(http.MethodDelete, "/api/v1/system/users/"+targetUser.String()+"/roles/admin", "", adminID)
	request.SetPathValue("user_id", targetUser.String())
	request.SetPathValue("role_name", "admin")
	record = httptest.NewRecorder()
	handler.RevokeRole(record, request)
	if record.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 when revoking last admin, got %d", record.Code)
	}
}
