package httpinfra

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

const testSecret = "test-secret-at-least-32-chars-long-for-jwt!!"

func TestRequestIDMiddleware_Generated(t *testing.T) {
	handler := RequestIDMiddleware()(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	reqID := rec.Header().Get("x-request-id")
	if reqID == "" {
		t.Fatal("expected x-request-id in response header")
	}
	if _, err := uuid.Parse(reqID); err != nil {
		t.Fatalf("expected valid UUID, got %s: %v", reqID, err)
	}
}

func TestRequestIDMiddleware_Propagated(t *testing.T) {
	incomingID := "01946850-891a-7b3b-9a4f-9588fb7cfa02"
	handler := RequestIDMiddleware()(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set("x-request-id", incomingID)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	if rec.Header().Get("x-request-id") != incomingID {
		t.Errorf("expected propagated id %s, got %s", incomingID, rec.Header().Get("x-request-id"))
	}
}

func TestAuthMiddleware_PublicBypass(t *testing.T) {
	called := false
	handler := AuthMiddleware(testSecret, []string{"/api/v1/health"})(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		called = true
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	if !called {
		t.Fatal("expected public route to bypass auth")
	}
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestAuthMiddleware_Unauthorized(t *testing.T) {
	handler := AuthMiddleware(testSecret, []string{"/public"})(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/api/v1/protected", nil)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", rec.Code)
	}
	if !strings.Contains(rec.Body.String(), "Authentication required") {
		t.Errorf("expected plain text error, got %s", rec.Body.String())
	}
}

func TestAuthMiddleware_Authorized(t *testing.T) {
	userID := ids.UserID(uuid.New())
	token, err := auth.GenerateToken(userID, testSecret, 15*time.Minute)
	if err != nil {
		t.Fatalf("token gen error: %v", err)
	}
	var extractedID uuid.UUID
	handler := AuthMiddleware(testSecret, nil)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var ok bool
		extractedID, ok = UserIDFromContext(r.Context())
		if !ok {
			t.Error("expected user id in context")
		}
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/api/v1/employees", nil)
	req.Header.Set("Authorization", "Bearer "+token)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if extractedID != userID.UUID() {
		t.Errorf("expected user id %s, got %s", userID, extractedID)
	}
}
