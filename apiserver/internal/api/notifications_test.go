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

type mockNotificationRepository struct {
	response     NotificationResponse
	listErr      error
	markErr      error
	markAllErr   error
	listCalls    int
	markCalls    int
	markAllCalls int
}

func (m *mockNotificationRepository) List(context.Context, uuid.UUID, int, *time.Time) (NotificationResponse, error) {
	m.listCalls++
	return m.response, m.listErr
}
func (m *mockNotificationRepository) MarkRead(context.Context, uuid.UUID, uuid.UUID) error {
	m.markCalls++
	return m.markErr
}
func (m *mockNotificationRepository) MarkAllRead(context.Context, uuid.UUID) error {
	m.markAllCalls++
	return m.markAllErr
}

func TestNotificationListRequiresAuth(t *testing.T) {
	handler := NotificationHandler{Repository: &mockNotificationRepository{}}
	req := httptest.NewRequest(http.MethodGet, "/v1/notifications", nil)
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", rec.Code)
	}
}

func TestNotificationListSuccess(t *testing.T) {
	repo := &mockNotificationRepository{response: NotificationResponse{Notifications: []NotificationRecord{{ID: uuid.New(), Title: "Alert"}}, UnreadCount: 1}}
	handler := NotificationHandler{Repository: repo}
	req := httptest.NewRequest(http.MethodGet, "/v1/notifications?limit=10", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.listCalls != 1 {
		t.Fatalf("expected 1 list call, got %d", repo.listCalls)
	}
}

func TestNotificationMarkReadSuccess(t *testing.T) {
	notifID := uuid.New()
	repo := &mockNotificationRepository{}
	handler := NotificationHandler{Repository: repo}
	req := httptest.NewRequest(http.MethodPatch, "/v1/notifications/"+notifID.String()+"/read", nil)
	req.SetPathValue("id", notifID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.MarkRead(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if repo.markCalls != 1 {
		t.Fatalf("expected 1 mark call, got %d", repo.markCalls)
	}
}
