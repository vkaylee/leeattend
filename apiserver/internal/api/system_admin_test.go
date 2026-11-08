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
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockSystemSettings struct {
	settings map[string]json.RawMessage
}

func (m *mockSystemSettings) Get(_ context.Context, key string) (json.RawMessage, bool, error) {
	value, ok := m.settings[key]
	return value, ok, nil
}

func (m *mockSystemSettings) Set(_ context.Context, key string, value json.RawMessage) error {
	m.settings[key] = value
	return nil
}

type mockSystemAuthorizer struct {
	allowed bool
}

func (m mockSystemAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return m.allowed && permission == string(auth.PermissionSecuritySettingsManage)
}
func (m mockSystemAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, _ string, _ *uuid.UUID) bool {
	return m.allowed
}
func (m mockSystemAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return m.allowed, nil }
func (m mockSystemAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (m mockSystemAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}
func (m mockSystemAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (m mockSystemAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool {
	return m.allowed
}

type mockEmailService struct{ fail bool }

func (e mockEmailService) SendTestEmail(_ context.Context, _ string) (string, error) {
	if e.fail {
		return "", errors.New("connection to smtp server failed")
	}
	return "test email sent", nil
}

func adminRequest(method, path, body string, userID uuid.UUID) *http.Request {
	var request *http.Request
	if body == "" {
		request = httptest.NewRequest(method, path, nil)
	} else {
		request = httptest.NewRequest(method, path, bytes.NewBufferString(body))
	}
	return request.WithContext(httpinfra.WithUserIDForTest(request.Context(), userID))
}

func setupTestHandler() (SystemAdminHandler, *mockSystemSettings, uuid.UUID) {
	userID := uuid.New()
	settings := &mockSystemSettings{settings: make(map[string]json.RawMessage)}
	return SystemAdminHandler{
		Authorizer:  mockSystemAuthorizer{allowed: true},
		Settings:    settings,
		EmailSender: mockEmailService{},
	}, settings, userID
}

func TestSystemAdminRequiresStandalonePermission(t *testing.T) {
	handler, _, userID := setupTestHandler()
	handler.Authorizer = mockSystemAuthorizer{allowed: false}
	record := httptest.NewRecorder()
	handler.GetSMTPConfig(record, adminRequest(http.MethodGet, "/api/v1/system/smtp/config", "", userID))
	if record.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", record.Code)
	}

	record = httptest.NewRecorder()
	handler.GetSMTPConfig(record, adminRequest(http.MethodGet, "/api/v1/system/smtp/config", "", uuid.Nil))
	if record.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", record.Code)
	}
}

func TestSystemAdminSMTPConfigAndTest(t *testing.T) {
	handler, settings, userID := setupTestHandler()
	record := httptest.NewRecorder()
	handler.SetSMTPConfig(record, adminRequest(http.MethodPut, "/api/v1/system/smtp/config", `{"host":"smtp.example.com"}`, userID))
	if record.Code != http.StatusOK || string(settings.settings["smtp"]) != `{"host":"smtp.example.com"}` {
		t.Fatalf("expected SMTP config to persist, got %d: %s", record.Code, record.Body.String())
	}

	record = httptest.NewRecorder()
	failed := handler
	failed.EmailSender = mockEmailService{fail: true}
	failed.TestSMTP(record, adminRequest(http.MethodPost, "/api/v1/system/smtp/test", `{"to_email":"admin@example.com"}`, userID))
	if record.Code != http.StatusOK || !strings.Contains(record.Body.String(), `"success":false`) {
		t.Fatalf("expected SMTP failure response, got %d: %s", record.Code, record.Body.String())
	}
}
